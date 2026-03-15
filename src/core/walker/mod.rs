mod engine;
mod entry;

pub use engine::{Node, OrderedEngine, TraversalResult};
pub use entry::{EntryType, TreeEntry, WinAttributes};

/// Statistics gathered during tree traversal
#[derive(Debug, Default, Clone)]
pub struct TreeStats {
    pub directories: u64,
    pub files: u64,
    pub symlinks: u64,
    pub errors: u64,
}
