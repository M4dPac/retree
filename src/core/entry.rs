//! Core domain model for tree entries with full metadata support.

use std::ffi::OsString;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::TreeError;

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub name: OsString,
    pub entry_type: EntryType,
    pub metadata: Option<EntryMetadata>,
    pub depth: usize,
    pub is_last: bool,
    pub ancestors_last: Vec<bool>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EntryType {
    File,
    Directory,
    Symlink { target: PathBuf, broken: bool },
    Junction { target: PathBuf },
    HardLink { link_count: u32 },
    Ads { stream_name: String },
    Other,
}

impl EntryType {
    pub fn is_directory(&self) -> bool {
        matches!(self, EntryType::Directory)
    }

    #[allow(dead_code)]
    pub fn is_file(&self) -> bool {
        matches!(self, EntryType::File)
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self, EntryType::Symlink { .. } | EntryType::Junction { .. })
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct EntryMetadata {
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub attributes: WinAttributes,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub inode: u64,
    pub device: u32,
    pub nlinks: u32,
    pub mode: Option<u32>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WinAttributes {
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub compressed: bool,
    pub encrypted: bool,
    pub offline: bool,
    pub sparse: bool,
    pub temporary: bool,
    pub reparse: bool,
}

impl WinAttributes {
    /// Parse raw Windows file attribute flags.
    /// Uses hardcoded constants (stable since Windows NT).
    /// Works on all platforms — on non-Windows, raw attrs always come as 0.
    pub fn from_raw(attrs: u32) -> Self {
        WinAttributes {
            readonly: attrs & 0x1 != 0,
            hidden: attrs & 0x2 != 0,
            system: attrs & 0x4 != 0,
            archive: attrs & 0x20 != 0,
            compressed: attrs & 0x800 != 0,
            encrypted: attrs & 0x4000 != 0,
            offline: attrs & 0x1000 != 0,
            sparse: attrs & 0x200 != 0,
            temporary: attrs & 0x100 != 0,
            reparse: attrs & 0x400 != 0,
        }
    }

    pub fn to_string_short(&self) -> String {
        let mut s = String::with_capacity(6);
        s.push(if self.readonly { 'R' } else { '-' });
        s.push(if self.hidden { 'H' } else { '-' });
        s.push(if self.system { 'S' } else { '-' });
        s.push(if self.archive { 'A' } else { '-' });
        s.push(if self.compressed { 'C' } else { '-' });
        s.push(if self.encrypted { 'E' } else { '-' });
        s
    }
}

impl Entry {
    pub fn from_path(
        path: &Path,
        depth: usize,
        is_last: bool,
        ancestors_last: Vec<bool>,
        needs_file_id: bool,
        needs_attrs: bool,
    ) -> Result<Self, TreeError> {
        let name = path
            .file_name()
            .map(|n| n.to_owned())
            .unwrap_or_else(|| path.as_os_str().to_owned());

        let symlink_meta =
            std::fs::symlink_metadata(path).map_err(|e| TreeError::Io(path.to_path_buf(), e))?;

        let entry_type = determine_entry_type(path, &symlink_meta, needs_file_id)?;
        let metadata = gather_metadata(path, &symlink_meta, needs_file_id, needs_attrs)?;

        Ok(Entry {
            path: path.to_path_buf(),
            name,
            entry_type,
            metadata: Some(metadata),
            depth,
            is_last,
            ancestors_last,
        })
    }

    pub fn from_dir_entry(
        entry: &std::fs::DirEntry,
        depth: usize,
        is_last: bool,
        ancestors_last: Vec<bool>,
        needs_file_id: bool,
        needs_attrs: bool,
    ) -> Result<Self, TreeError> {
        let path = entry.path();
        let name = entry.file_name();

        let symlink_meta = entry
            .metadata()
            .map_err(|e| TreeError::Io(path.clone(), e))?;

        let entry_type = determine_entry_type(&path, &symlink_meta, needs_file_id)?;
        let metadata = gather_metadata(&path, &symlink_meta, needs_file_id, needs_attrs)?;

        Ok(Entry {
            path,
            name,
            entry_type,
            metadata: Some(metadata),
            depth,
            is_last,
            ancestors_last,
        })
    }

    pub fn name_str(&self) -> &str {
        self.name.to_str().unwrap_or("<invalid>")
    }
}

fn determine_entry_type(
    path: &Path,
    symlink_meta: &Metadata,
    needs_file_id: bool,
) -> Result<EntryType, TreeError> {
    let file_type = symlink_meta.file_type();

    if file_type.is_symlink() {
        match std::fs::read_link(path) {
            Ok(target) => {
                let broken = !target.exists() && !path.join(&target).exists();
                Ok(EntryType::Symlink { target, broken })
            }
            Err(e) => Err(TreeError::SymlinkError(path.to_path_buf(), e)),
        }
    } else if file_type.is_dir() {
        // Check for junction point (Windows-only, returns None on other platforms)
        if let Some(target) = crate::platform::get_junction_target(path) {
            return Ok(EntryType::Junction { target });
        }
        Ok(EntryType::Directory)
    } else if file_type.is_file() {
        // Check for hard links (Windows-only via file ID, returns None on other platforms)
        if needs_file_id {
            if let Some(info) = crate::platform::get_file_id(path) {
                if info.number_of_links > 1 {
                    return Ok(EntryType::HardLink {
                        link_count: info.number_of_links,
                    });
                }
            }
        }
        Ok(EntryType::File)
    } else {
        Ok(EntryType::Other)
    }
}

fn gather_metadata(
    path: &Path,
    symlink_meta: &Metadata,
    needs_file_id: bool,
    needs_attrs: bool,
) -> Result<EntryMetadata, TreeError> {
    let mut meta = EntryMetadata {
        size: symlink_meta.len(),
        created: symlink_meta.created().ok(),
        modified: symlink_meta.modified().ok(),
        accessed: symlink_meta.accessed().ok(),
        ..Default::default()
    };

    if needs_attrs {
        if let Some(attrs) = crate::platform::get_file_attributes_raw(path) {
            meta.attributes = WinAttributes::from_raw(attrs);
        }
    }

    if needs_file_id {
        if let Some(info) = crate::platform::get_file_id(path) {
            meta.inode = info.file_id;
            meta.device = info.volume_serial;
            meta.nlinks = info.number_of_links;
        }
    }

    // Get Posix mode (Unix only)
    meta.mode = crate::platform::get_file_mode(path);

    Ok(meta)
}
