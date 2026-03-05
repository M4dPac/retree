mod pattern;

pub use pattern::GlobPattern;

use crate::error::TreeError;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Filter {
    include: Option<GlobPattern>,
    exclude: Vec<GlobPattern>,
    ignore_case: bool,
    match_dirs: bool,
}

impl Filter {
    pub fn new(
        include_pattern: Option<&str>,
        exclude_patterns: &[String],
        match_dirs: bool,
        ignore_case: bool,
    ) -> Result<Self, TreeError> {
        let include = if let Some(p) = include_pattern {
            Some(GlobPattern::new(p, ignore_case)?)
        } else {
            None
        };

        let exclude = exclude_patterns
            .iter()
            .map(|p| GlobPattern::new(p, ignore_case))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Filter {
            include,
            exclude,
            ignore_case,
            match_dirs,
        })
    }

    pub fn matches(&self, name: &str, is_dir: bool) -> bool {
        // Check exclude patterns first (applies to both files and dirs)
        for pattern in &self.exclude {
            if pattern.matches(name) {
                return false;
            }
        }

        // Directories are never filtered by -P (always shown and traversed)
        if is_dir {
            return true;
        }

        // Apply include pattern to files
        if let Some(ref pattern) = self.include {
            return pattern.matches(name);
        }

        true
    }

    /// Check if an entry is excluded by -I patterns.
    /// Always applies to both files and directories (GNU tree behavior).
    /// Unlike -P, -I does not require --matchdirs to affect directories.
    pub fn excluded(&self, name: &str) -> bool {
        for pattern in &self.exclude {
            if pattern.matches(name) {
                return true;
            }
        }
        false
    }

    /// Check if directory name matches the -P include pattern.
    /// Used with --matchdirs: if a directory matches, all its descendants bypass -P filter.
    /// Also protects the directory from --prune.
    pub fn dir_matches_include(&self, name: &str) -> bool {
        if !self.match_dirs {
            return false;
        }
        if let Some(ref pattern) = self.include {
            return pattern.matches(name);
        }
        false
    }
}
