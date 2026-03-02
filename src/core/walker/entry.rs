//! Re-export of core entry types for backward compatibility.
//!
//! This module is deprecated. Use `crate::core::entry` instead.

#[allow(unused_imports)]
pub use crate::core::entry::{Entry as TreeEntry, EntryMetadata, EntryType, WinAttributes};
