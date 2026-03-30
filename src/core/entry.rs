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
    pub filelimit_exceeded: Option<usize>,
    pub recursive_link: bool,
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
    pub device: u64,
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
        needs_file_id: bool,
        needs_attrs: bool,
    ) -> Result<Self, TreeError> {
        let name = path
            .file_name()
            .map(|n| n.to_owned())
            .unwrap_or_else(|| path.as_os_str().to_owned());

        let symlink_meta = std::fs::symlink_metadata(path)
            .map_err(|e| TreeError::from_io(path.to_path_buf(), e))?;

        let entry_type = determine_entry_type(path, &symlink_meta, needs_file_id)?;
        let metadata = gather_metadata(path, &symlink_meta, needs_file_id, needs_attrs)?;

        Ok(Entry {
            path: path.to_path_buf(),
            name,
            entry_type,
            metadata: Some(metadata),
            depth,
            is_last: false,
            ancestors_last: Vec::new(),
            filelimit_exceeded: None,
            recursive_link: false,
        })
    }

    pub fn from_dir_entry(
        entry: &std::fs::DirEntry,
        depth: usize,
        needs_file_id: bool,
        needs_attrs: bool,
    ) -> Result<Self, TreeError> {
        let path = entry.path();
        let name = entry.file_name();

        let symlink_meta = entry
            .metadata()
            .map_err(|e| TreeError::from_io(path.clone(), e))?;

        let entry_type = determine_entry_type(&path, &symlink_meta, needs_file_id)?;
        let metadata = gather_metadata(&path, &symlink_meta, needs_file_id, needs_attrs)?;

        Ok(Entry {
            path,
            name,
            entry_type,
            metadata: Some(metadata),
            depth,
            is_last: false,
            ancestors_last: Vec::new(),
            filelimit_exceeded: None,
            recursive_link: false,
        })
    }

    /// Create an entry representing an NTFS Alternate Data Stream.
    ///
    /// The display name is set to `:stream_name` following NTFS convention.
    /// The path points to the *parent file* (used for `--full-path` rendering).
    pub fn from_ads(
        parent_path: &Path,
        stream_name: String,
        stream_size: u64,
        depth: usize,
    ) -> Self {
        Entry {
            path: parent_path.to_path_buf(),
            name: OsString::from(format!(":{}", stream_name)),
            entry_type: EntryType::Ads { stream_name },
            metadata: Some(EntryMetadata {
                size: stream_size,
                ..Default::default()
            }),
            depth,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        }
    }

    pub fn name_str(&self) -> std::borrow::Cow<'_, str> {
        self.name.to_string_lossy()
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
                // Check if symlink target is valid.
                // For relative targets, resolve against the symlink's parent directory,
                // not the symlink itself.
                let broken = if target.is_absolute() {
                    !target.exists()
                } else {
                    // parent of symlink + relative target
                    match path.parent() {
                        Some(parent) => !parent.join(&target).exists(),
                        None => !target.exists(),
                    }
                };
                Ok(EntryType::Symlink { target, broken })
            }
            Err(e) => Err(TreeError::SymlinkError(path.to_path_buf(), e)),
        }
    } else if file_type.is_dir() {
        // Check for junction point (Windows-only, returns None on other platforms)
        // First check reparse attribute to avoid expensive DeviceIoControl call
        if let Some(attrs) = crate::platform::get_file_attributes_raw(path) {
            if attrs & 0x400 != 0 {
                // FILE_ATTRIBUTE_REPARSE_POINT
                if let Some(target) = crate::platform::get_junction_target(path) {
                    return Ok(EntryType::Junction { target });
                }
            }
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

    // Get owner/group
    meta.owner = crate::platform::get_file_owner(path);
    meta.group = crate::platform::get_file_group(path);

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ══════════════════════════════════════════════
    // WinAttributes::from_raw
    // ══════════════════════════════════════════════

    #[test]
    fn win_attrs_all_zero() {
        let a = WinAttributes::from_raw(0);
        assert!(!a.readonly && !a.hidden && !a.system && !a.archive);
        assert!(!a.compressed && !a.encrypted && !a.offline);
        assert!(!a.sparse && !a.temporary && !a.reparse);
    }

    #[test]
    fn win_attrs_readonly() {
        let a = WinAttributes::from_raw(0x1);
        assert!(a.readonly);
        assert!(!a.hidden);
    }

    #[test]
    fn win_attrs_hidden() {
        let a = WinAttributes::from_raw(0x2);
        assert!(a.hidden);
    }

    #[test]
    fn win_attrs_system() {
        let a = WinAttributes::from_raw(0x4);
        assert!(a.system);
    }

    #[test]
    fn win_attrs_archive() {
        let a = WinAttributes::from_raw(0x20);
        assert!(a.archive);
    }

    #[test]
    fn win_attrs_temporary() {
        let a = WinAttributes::from_raw(0x100);
        assert!(a.temporary);
    }

    #[test]
    fn win_attrs_sparse() {
        let a = WinAttributes::from_raw(0x200);
        assert!(a.sparse);
    }

    #[test]
    fn win_attrs_reparse() {
        let a = WinAttributes::from_raw(0x400);
        assert!(a.reparse);
    }

    #[test]
    fn win_attrs_compressed() {
        let a = WinAttributes::from_raw(0x800);
        assert!(a.compressed);
    }

    #[test]
    fn win_attrs_offline() {
        let a = WinAttributes::from_raw(0x1000);
        assert!(a.offline);
    }

    #[test]
    fn win_attrs_encrypted() {
        let a = WinAttributes::from_raw(0x4000);
        assert!(a.encrypted);
    }

    #[test]
    fn win_attrs_combined_rha() {
        let a = WinAttributes::from_raw(0x1 | 0x2 | 0x20); // R+H+A
        assert!(a.readonly && a.hidden && a.archive);
        assert!(!a.system && !a.compressed && !a.encrypted);
    }

    #[test]
    fn win_attrs_all_bits_set() {
        let a = WinAttributes::from_raw(
            0x1 | 0x2 | 0x4 | 0x20 | 0x100 | 0x200 | 0x400 | 0x800 | 0x1000 | 0x4000,
        );
        assert!(a.readonly && a.hidden && a.system && a.archive);
        assert!(a.temporary && a.sparse && a.reparse);
        assert!(a.compressed && a.offline && a.encrypted);
    }

    // ══════════════════════════════════════════════
    // WinAttributes::to_string_short
    // ══════════════════════════════════════════════

    #[test]
    fn win_attrs_short_all_clear() {
        assert_eq!(WinAttributes::from_raw(0).to_string_short(), "------");
    }

    #[test]
    fn win_attrs_short_all_set() {
        let a = WinAttributes::from_raw(0x1 | 0x2 | 0x4 | 0x20 | 0x800 | 0x4000);
        assert_eq!(a.to_string_short(), "RHSACE");
    }

    #[test]
    fn win_attrs_short_readonly_only() {
        assert_eq!(WinAttributes::from_raw(0x1).to_string_short(), "R-----");
    }

    #[test]
    fn win_attrs_short_hidden_only() {
        assert_eq!(WinAttributes::from_raw(0x2).to_string_short(), "-H----");
    }

    #[test]
    fn win_attrs_short_system_only() {
        assert_eq!(WinAttributes::from_raw(0x4).to_string_short(), "--S---");
    }

    #[test]
    fn win_attrs_short_archive_only() {
        assert_eq!(WinAttributes::from_raw(0x20).to_string_short(), "---A--");
    }

    #[test]
    fn win_attrs_short_compressed_only() {
        assert_eq!(WinAttributes::from_raw(0x800).to_string_short(), "----C-");
    }

    #[test]
    fn win_attrs_short_encrypted_only() {
        assert_eq!(WinAttributes::from_raw(0x4000).to_string_short(), "-----E");
    }

    #[test]
    fn win_attrs_short_length() {
        // Always 6 characters
        for raw in [0, 0x1, 0x23, 0xFFFF] {
            assert_eq!(WinAttributes::from_raw(raw).to_string_short().len(), 6);
        }
    }

    #[test]
    fn win_attrs_default_equals_zero() {
        assert_eq!(
            WinAttributes::default().to_string_short(),
            WinAttributes::from_raw(0).to_string_short()
        );
    }

    // ══════════════════════════════════════════════
    // EntryType predicates
    // ══════════════════════════════════════════════

    #[test]
    fn entry_type_directory() {
        assert!(EntryType::Directory.is_directory());
        assert!(!EntryType::Directory.is_file());
        assert!(!EntryType::Directory.is_symlink());
    }

    #[test]
    fn entry_type_file() {
        assert!(EntryType::File.is_file());
        assert!(!EntryType::File.is_directory());
        assert!(!EntryType::File.is_symlink());
    }

    #[test]
    fn entry_type_symlink() {
        let s = EntryType::Symlink {
            target: PathBuf::from("t"),
            broken: false,
        };
        assert!(s.is_symlink());
        assert!(!s.is_file());
        assert!(!s.is_directory());
    }

    #[test]
    fn entry_type_broken_symlink_still_symlink() {
        let s = EntryType::Symlink {
            target: PathBuf::from("gone"),
            broken: true,
        };
        assert!(s.is_symlink());
    }

    #[test]
    fn entry_type_junction_is_symlink() {
        let j = EntryType::Junction {
            target: PathBuf::from("t"),
        };
        assert!(j.is_symlink());
        assert!(!j.is_directory());
    }

    #[test]
    fn entry_type_hardlink_not_symlink() {
        let h = EntryType::HardLink { link_count: 3 };
        assert!(!h.is_symlink());
        assert!(!h.is_directory());
        assert!(!h.is_file());
    }

    #[test]
    fn entry_type_ads_no_predicates() {
        let a = EntryType::Ads {
            stream_name: "s".into(),
        };
        assert!(!a.is_file() && !a.is_directory() && !a.is_symlink());
    }

    #[test]
    fn entry_type_other() {
        assert!(!EntryType::Other.is_file());
        assert!(!EntryType::Other.is_directory());
        assert!(!EntryType::Other.is_symlink());
    }

    // ══════════════════════════════════════════════
    // Entry::from_ads
    // ══════════════════════════════════════════════

    #[test]
    fn from_ads_basic() {
        let e = Entry::from_ads(&PathBuf::from("/f.txt"), "Zone.Identifier".into(), 42, 3);
        assert_eq!(e.name, ":Zone.Identifier");
        assert_eq!(e.path, PathBuf::from("/f.txt"));
        assert_eq!(e.depth, 3);
        assert!(!e.is_last);
        assert!(!e.recursive_link);
        assert!(e.ancestors_last.is_empty());
        assert!(e.filelimit_exceeded.is_none());
        assert_eq!(e.metadata.as_ref().unwrap().size, 42);
        match &e.entry_type {
            EntryType::Ads { stream_name } => assert_eq!(stream_name, "Zone.Identifier"),
            _ => panic!("expected Ads"),
        }
    }

    #[test]
    fn from_ads_zero_size() {
        let e = Entry::from_ads(&PathBuf::from("x"), "empty".into(), 0, 0);
        assert_eq!(e.metadata.as_ref().unwrap().size, 0);
    }

    #[test]
    fn from_ads_name_str_has_colon_prefix() {
        let e = Entry::from_ads(&PathBuf::from("x"), "data".into(), 1, 0);
        assert_eq!(e.name_str(), ":data");
    }

    // ══════════════════════════════════════════════
    // Entry::name_str
    // ══════════════════════════════════════════════

    #[test]
    fn name_str_ascii() {
        let e = Entry {
            path: PathBuf::from("test.txt"),
            name: OsString::from("test.txt"),
            entry_type: EntryType::File,
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        };
        assert_eq!(e.name_str(), "test.txt");
    }

    #[test]
    fn name_str_unicode() {
        let e = Entry {
            path: PathBuf::from("файл.txt"),
            name: OsString::from("файл.txt"),
            entry_type: EntryType::File,
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        };
        assert_eq!(e.name_str(), "файл.txt");
    }

    // ══════════════════════════════════════════════
    // Entry::from_path (real filesystem)
    // ══════════════════════════════════════════════

    #[test]
    fn from_path_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("hello.txt");
        std::fs::write(&f, "hello world").unwrap();

        let e = Entry::from_path(&f, 1, false, false).unwrap();
        assert_eq!(e.name, "hello.txt");
        assert_eq!(e.depth, 1);
        assert!(!e.is_last);
        assert!(e.ancestors_last.is_empty());
        assert!(matches!(e.entry_type, EntryType::File));
        assert_eq!(e.metadata.as_ref().unwrap().size, 11);
    }

    #[test]
    fn from_path_directory() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();

        let e = Entry::from_path(&sub, 0, false, false).unwrap();
        assert_eq!(e.name, "subdir");
        assert!(matches!(e.entry_type, EntryType::Directory));
    }

    #[test]
    fn from_path_nonexistent_errors() {
        let result = Entry::from_path(&PathBuf::from("/no/such/path/xyz_42"), 0, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn from_path_metadata_timestamps() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("ts.txt");
        std::fs::write(&f, "x").unwrap();

        let e = Entry::from_path(&f, 0, false, false).unwrap();
        let meta = e.metadata.as_ref().unwrap();
        assert!(meta.modified.is_some());
    }

    // ══════════════════════════════════════════════
    // Entry::from_dir_entry (real filesystem)
    // ══════════════════════════════════════════════

    #[test]
    fn from_dir_entry_basic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("readme.md"), "# Hi").unwrap();

        let de = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap();

        let e = Entry::from_dir_entry(&de, 2, false, false).unwrap();
        assert_eq!(e.name, "readme.md");
        assert_eq!(e.depth, 2);
        assert!(!e.is_last);
        assert!(e.ancestors_last.is_empty());
    }

    // ══════════════════════════════════════════════
    // EntryMetadata::default
    // ══════════════════════════════════════════════

    #[test]
    fn metadata_default_zeroed() {
        let m = EntryMetadata::default();
        assert_eq!(m.size, 0);
        assert_eq!(m.inode, 0);
        assert_eq!(m.device, 0);
        assert_eq!(m.nlinks, 0);
        assert!(m.created.is_none());
        assert!(m.modified.is_none());
        assert!(m.accessed.is_none());
        assert!(m.owner.is_none());
        assert!(m.group.is_none());
        assert!(m.permissions.is_none());
        assert!(m.mode.is_none());
    }

    // ══════════════════════════════════════════════
    // Symlink tests (Unix only)
    // ══════════════════════════════════════════════

    #[cfg(unix)]
    #[test]
    fn from_path_symlink_valid() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        std::fs::write(&target, "data").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let e = Entry::from_path(&link, 0, false, false).unwrap();
        match &e.entry_type {
            EntryType::Symlink { target: t, broken } => {
                assert!(!broken);
                assert_eq!(*t, target);
            }
            other => panic!("expected Symlink, got {:?}", other),
        }
    }

    #[cfg(unix)]
    #[test]
    fn from_path_symlink_broken() {
        let dir = tempfile::tempdir().unwrap();
        let link = dir.path().join("broken_link");
        std::os::unix::fs::symlink("/no/such/target", &link).unwrap();

        let e = Entry::from_path(&link, 0, false, false).unwrap();
        match &e.entry_type {
            EntryType::Symlink { broken, .. } => assert!(broken),
            other => panic!("expected Symlink, got {:?}", other),
        }
    }

    #[cfg(unix)]
    #[test]
    fn from_path_relative_symlink_not_broken() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("real.txt");
        let link = dir.path().join("rel_link");
        std::fs::write(&target, "x").unwrap();
        std::os::unix::fs::symlink("real.txt", &link).unwrap();

        let e = Entry::from_path(&link, 0, false, false).unwrap();
        match &e.entry_type {
            EntryType::Symlink { broken, .. } => assert!(!broken),
            other => panic!("expected Symlink, got {:?}", other),
        }
    }
}
