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

        // For directories, only apply include pattern if match_dirs is true
        if is_dir && !self.match_dirs {
            return true;
        }

        // Apply include pattern
        if let Some(ref pattern) = self.include {
            return pattern.matches(name);
        }

        true
    }
}

