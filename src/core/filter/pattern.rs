use crate::error::TreeError;

#[derive(Debug, Clone)]
pub struct GlobPattern {
    patterns: Vec<String>,
    ignore_case: bool,
}

impl GlobPattern {
    pub fn new(pattern: &str, ignore_case: bool) -> Result<Self, TreeError> {
        // Split by unescaped '|' to support regex-like OR: "pat1|pat2|pat3"
        let sub_patterns = split_pattern_by_pipe(pattern);

        // Validate each sub-pattern
        for sub in &sub_patterns {
            let mut chars = sub.chars().peekable();
            let mut in_bracket = false;

            while let Some(c) = chars.next() {
                match c {
                    '[' => in_bracket = true,
                    ']' => in_bracket = false,
                    '\\' => {
                        chars.next();
                    }
                    _ => {}
                }
            }

            if in_bracket {
                return Err(TreeError::InvalidPattern(format!(
                    "Unclosed bracket in pattern: {}",
                    sub
                )));
            }
        }

        Ok(GlobPattern {
            patterns: sub_patterns,
            ignore_case,
        })
    }

    pub fn matches(&self, name: &str) -> bool {
        let name = if self.ignore_case {
            name.to_lowercase()
        } else {
            name.to_string()
        };

        // OR logic: any sub-pattern matches
        self.patterns.iter().any(|pat| {
            let pattern = if self.ignore_case {
                pat.to_lowercase()
            } else {
                pat.clone()
            };
            glob_match(&pattern, &name)
        })
    }
}

/// Split pattern by unescaped '|' character
/// Supports escape sequence: \| for literal pipe
fn split_pattern_by_pipe(pattern: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if next == '|' {
                    // Escaped pipe - add literal '|'
                    chars.next();
                    current.push('|');
                    continue;
                }
            }
            // Regular backslash
            current.push(c);
        } else if c == '|' {
            // Unescaped pipe - split point
            result.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }

    result.push(current);
    result
}

/// Maximum number of matching steps to prevent ReDoS
/// with pathological patterns like `*a*a*a*a*b`.
const MAX_GLOB_STEPS: usize = 10_000;

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let mut steps = 0;

    glob_match_recursive(&pattern, &text, 0, 0, &mut steps)
}

fn glob_match_recursive(
    pattern: &[char],
    text: &[char],
    pi: usize,
    ti: usize,
    steps: &mut usize,
) -> bool {
    *steps += 1;
    if *steps > MAX_GLOB_STEPS {
        return false;
    }

    if pi >= pattern.len() {
        return ti >= text.len();
    }

    match pattern[pi] {
        '*' => {
            // Match zero or more characters
            for i in ti..=text.len() {
                if glob_match_recursive(pattern, text, pi + 1, i, steps) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // Match exactly one character
            if ti < text.len() {
                glob_match_recursive(pattern, text, pi + 1, ti + 1, steps)
            } else {
                false
            }
        }
        '[' => {
            // Character class
            if ti >= text.len() {
                return false;
            }

            let mut end = pi + 1;
            while end < pattern.len() && pattern[end] != ']' {
                end += 1;
            }

            if end >= pattern.len() {
                return false; // Unclosed bracket
            }

            let class = &pattern[pi + 1..end];
            let negated = !class.is_empty() && (class[0] == '!' || class[0] == '^');
            let class = if negated { &class[1..] } else { class };

            let matches = char_in_class(text[ti], class);
            let matches = if negated { !matches } else { matches };

            if matches {
                glob_match_recursive(pattern, text, end + 1, ti + 1, steps)
            } else {
                false
            }
        }
        '\\' if pi + 1 < pattern.len() => {
            // Escaped character
            if ti < text.len() && text[ti] == pattern[pi + 1] {
                glob_match_recursive(pattern, text, pi + 2, ti + 1, steps)
            } else {
                false
            }
        }
        c => {
            // Literal character
            if ti < text.len() && text[ti] == c {
                glob_match_recursive(pattern, text, pi + 1, ti + 1, steps)
            } else {
                false
            }
        }
    }
}

fn char_in_class(c: char, class: &[char]) -> bool {
    let mut i = 0;
    while i < class.len() {
        if i + 2 < class.len() && class[i + 1] == '-' {
            // Range
            if c >= class[i] && c <= class[i + 2] {
                return true;
            }
            i += 3;
        } else {
            if c == class[i] {
                return true;
            }
            i += 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let p = GlobPattern::new("*.rs", false).unwrap();
        assert!(p.matches("main.rs"));
        assert!(p.matches("lib.rs"));
        assert!(!p.matches("main.py"));
    }

    #[test]
    fn test_question_mark() {
        let p = GlobPattern::new("file?.txt", false).unwrap();
        assert!(p.matches("file1.txt"));
        assert!(p.matches("fileX.txt"));
        assert!(!p.matches("file10.txt"));
    }

    #[test]
    fn test_char_class() {
        let p = GlobPattern::new("file[0-9].txt", false).unwrap();
        assert!(p.matches("file1.txt"));
        assert!(p.matches("file9.txt"));
        assert!(!p.matches("fileX.txt"));
    }

    #[test]
    fn test_ignore_case() {
        let p = GlobPattern::new("*.RS", true).unwrap();
        assert!(p.matches("main.rs"));
        assert!(p.matches("MAIN.RS"));
    }

    #[test]
    fn test_pipe_separator() {
        let p = GlobPattern::new("*.rs|*.toml|*.lock", false).unwrap();
        assert!(p.matches("main.rs"));
        assert!(p.matches("Cargo.toml"));
        assert!(p.matches("Cargo.lock"));
        assert!(!p.matches("readme.md"));
    }

    #[test]
    fn test_pipe_with_glob() {
        let p = GlobPattern::new(".git|target|test*", false).unwrap();
        assert!(p.matches(".git"));
        assert!(p.matches("target"));
        assert!(p.matches("tests"));
        assert!(p.matches("test_utils.rs"));
        assert!(!p.matches("src"));
    }

    #[test]
    fn test_escaped_pipe() {
        let p = GlobPattern::new("file\\|name|*.txt", false).unwrap();
        assert!(p.matches("file|name"));
        assert!(p.matches("readme.txt"));
        assert!(!p.matches("file"));
    }

    #[test]
    fn test_single_pattern_no_pipe() {
        let p = GlobPattern::new("*.rs", false).unwrap();
        assert!(p.matches("main.rs"));
        assert!(!p.matches("main.py"));
    }
}
