use std::ffi::OsString;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::TreeError;

#[derive(Debug, Clone)]
pub struct TreeEntry {
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
    #[cfg(windows)]
    pub fn from_raw(attrs: u32) -> Self {
        use windows_sys::Win32::Storage::FileSystem::*;

        WinAttributes {
            readonly: attrs & FILE_ATTRIBUTE_READONLY != 0,
            hidden: attrs & FILE_ATTRIBUTE_HIDDEN != 0,
            system: attrs & FILE_ATTRIBUTE_SYSTEM != 0,
            archive: attrs & FILE_ATTRIBUTE_ARCHIVE != 0,
            compressed: attrs & FILE_ATTRIBUTE_COMPRESSED != 0,
            encrypted: attrs & FILE_ATTRIBUTE_ENCRYPTED != 0,
            offline: attrs & FILE_ATTRIBUTE_OFFLINE != 0,
            sparse: attrs & FILE_ATTRIBUTE_SPARSE_FILE != 0,
            temporary: attrs & FILE_ATTRIBUTE_TEMPORARY != 0,
            reparse: attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0,
        }
    }

    #[cfg(not(windows))]
    pub fn from_raw(_attrs: u32) -> Self {
        WinAttributes::default()
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

impl TreeEntry {
    pub fn from_path(
        path: &Path,
        depth: usize,
        is_last: bool,
        ancestors_last: Vec<bool>,
    ) -> Result<Self, TreeError> {
        let name = path
            .file_name()
            .map(|n| n.to_owned())
            .unwrap_or_else(|| path.as_os_str().to_owned());

        let symlink_meta =
            fs::symlink_metadata(path).map_err(|e| TreeError::Io(path.to_path_buf(), e))?;

        let entry_type = determine_entry_type(path, &symlink_meta)?;
        let metadata = gather_metadata(path, &symlink_meta)?;

        Ok(TreeEntry {
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
        entry: &fs::DirEntry,
        depth: usize,
        is_last: bool,
        ancestors_last: Vec<bool>,
    ) -> Result<Self, TreeError> {
        let path = entry.path();
        let name = entry.file_name();

        let symlink_meta = entry
            .metadata()
            .map_err(|e| TreeError::Io(path.clone(), e))?;

        let entry_type = determine_entry_type(&path, &symlink_meta)?;
        let metadata = gather_metadata(&path, &symlink_meta)?;

        Ok(TreeEntry {
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

fn determine_entry_type(path: &Path, symlink_meta: &Metadata) -> Result<EntryType, TreeError> {
    let file_type = symlink_meta.file_type();

    if file_type.is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                let broken = !target.exists() && !path.join(&target).exists();
                Ok(EntryType::Symlink { target, broken })
            }
            Err(e) => Err(TreeError::SymlinkError(path.to_path_buf(), e)),
        }
    } else if file_type.is_dir() {
        // Check for junction point on Windows
        #[cfg(windows)]
        {
            if let Some(target) = crate::windows::reparse::get_junction_target(path) {
                return Ok(EntryType::Junction { target });
            }
        }
        Ok(EntryType::Directory)
    } else if file_type.is_file() {
        #[cfg(windows)]
        {
            // Check for hard links first
            if let Ok(info) = crate::windows::attributes::get_file_id(path) {
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

fn gather_metadata(path: &Path, symlink_meta: &Metadata) -> Result<EntryMetadata, TreeError> {
    let mut meta = EntryMetadata {
        size: symlink_meta.len(),
        created: symlink_meta.created().ok(),
        modified: symlink_meta.modified().ok(),
        accessed: symlink_meta.accessed().ok(),
        ..Default::default()
    };

    #[cfg(windows)]
    {
        if let Ok(info) = crate::windows::attributes::get_file_id(path) {
            meta.inode = info.file_id;
            meta.device = info.volume_serial;
            meta.nlinks = info.number_of_links;
        }

        if let Ok(attrs) = crate::windows::attributes::get_file_attributes(path) {
            meta.attributes = WinAttributes::from_raw(attrs);
        }
    }

    Ok(meta)
}
