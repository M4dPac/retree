use std::cmp::Ordering;

/// Natural (version) sort comparison
/// e.g., "file2" < "file10"
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    // Compare numeric sequences
                    let a_num = parse_number(&mut a_chars);
                    let b_num = parse_number(&mut b_chars);

                    match a_num.cmp(&b_num) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    // Compare characters (case-insensitive first, then case-sensitive)
                    let ac_lower = ac.to_lowercase().next().unwrap_or(ac);
                    let bc_lower = bc.to_lowercase().next().unwrap_or(bc);

                    match ac_lower.cmp(&bc_lower) {
                        Ordering::Equal => {
                            // Same letter, compare case
                            match ac.cmp(&bc) {
                                Ordering::Equal => {
                                    a_chars.next();
                                    b_chars.next();
                                    continue;
                                }
                                other => return other,
                            }
                        }
                        other => return other,
                    }
                }
            }
        }
    }
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> u64 {
    let mut num = 0u64;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num = num
                .saturating_mul(10)
                .saturating_add(c.to_digit(10).unwrap() as u64);
            chars.next();
        } else {
            break;
        }
    }
    num
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort() {
        let mut files = vec!["file10", "file2", "file1", "file20"];
        files.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(files, vec!["file1", "file2", "file10", "file20"]);
    }

    #[test]
    fn test_mixed_content() {
        let mut files = vec!["a2b", "a10b", "a1b"];
        files.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(files, vec!["a1b", "a2b", "a10b"]);
    }

    #[test]
    fn test_case_insensitive() {
        let mut files = vec!["B", "a", "c"];
        files.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(files, vec!["a", "B", "c"]);
    }
}
