//! Shared helpers for all render backends.
//!
//! Eliminates code duplication across text/xml/json/html renderers.

use crate::core::entry::EntryType;
pub use crate::core::walker::count_stats;

const SI_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
const IEC_UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

const XML_REPLACEMENTS: &[(&str, &str)] = &[
    ("&", "&amp;"),
    ("<", "&lt;"),
    (">", "&gt;"),
    ("\"", "&quot;"),
    ("'", "&apos;"),
];
const HTML_REPLACEMENTS: &[(&str, &str)] = &[
    ("&", "&amp;"),
    ("<", "&lt;"),
    (">", "&gt;"),
    ("\"", "&quot;"),
    ("'", "&#39;"),
];

/// Format size in human-readable form.
///
/// Uses IEC units (KiB, MiB, ...) by default,
/// or SI units (KB, MB, ...) when `si` is true.
pub fn format_human_size(size: u64, si: bool) -> String {
    let (divisor, units) = if si {
        (1000.0, SI_UNITS)
    } else {
        (1024.0, IEC_UNITS)
    };

    let mut val = size as f64;
    let mut unit_idx = 0;

    while val >= divisor && unit_idx < units.len() - 1 {
        val /= divisor;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", val, units[unit_idx])
    } else {
        format!("{:.1}{}", val, units[unit_idx])
    }
}

/// Check if character is a Unicode bidi override or zero-width character
/// that can be used for visual spoofing of filenames.
///
/// Shared by text renderer (sanitize_for_terminal) and
/// HTML/XML escapers (strip from structured output).
pub fn is_bidi_or_zw(c: char) -> bool {
    matches!(c,
        '\u{200B}'..='\u{200F}' | // zero-width space, ZWNJ, ZWJ, LRM, RLM
        '\u{202A}'..='\u{202E}' | // bidi embedding and override
        '\u{2060}'..='\u{2069}' | // word joiner, bidi isolates
        '\u{FEFF}'                 // BOM / zero-width no-break space
    )
}

/// Escape special characters for XML output.
/// Also strips control characters that are illegal in XML 1.0 (§2.2).
pub fn escape_xml(s: &str) -> String {
    escape_and_sanitize(s, XML_REPLACEMENTS)
}

/// Escape special characters for HTML output.
pub fn escape_html(s: &str) -> String {
    escape_and_sanitize(s, HTML_REPLACEMENTS)
}

/// Percent-encode a path for use in URL href attributes.
/// Encodes characters unsafe in URLs while preserving `/`.
/// Follows RFC 3986: only unreserved characters and `/` pass through.
pub fn encode_uri_path(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    // Worst case: every byte → %XX (3 chars), so capacity could be s.len() * 3.
    // s.len() is a reasonable lower-bound hint to reduce reallocations for typical paths.
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(byte as char)
            }
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0xf) as usize] as char);
            }
        }
    }
    out
}

/// Get entry type as a static string (for JSON/XML output).
pub fn entry_type_str(entry_type: &EntryType) -> &'static str {
    match entry_type {
        EntryType::File => "file",
        EntryType::Directory => "directory",
        EntryType::Symlink { .. } => "link",
        EntryType::Junction { .. } => "link",
        EntryType::HardLink { .. } => "file",
        EntryType::Ads { .. } => "stream",
        EntryType::Other => "other",
    }
}

fn escape_and_sanitize(s: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = s.to_string();
    for (from, to) in replacements {
        result = result.replace(from, to);
    }

    result
        .chars()
        .filter(|&c| matches!(c, '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..) && !is_bidi_or_zw(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entry::{Entry, EntryType};
    use crate::core::walker::TreeStats;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn make_entry(name: &str, entry_type: EntryType) -> Entry {
        Entry {
            path: PathBuf::from(name),
            name: OsString::from(name),
            entry_type,
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        }
    }

    // ══════════════════════════════════════════════
    // format_human_size — IEC (default, base 1024)
    // ══════════════════════════════════════════════

    #[test]
    fn human_size_zero() {
        assert_eq!(format_human_size(0, false), "0B");
    }

    #[test]
    fn human_size_one_byte() {
        assert_eq!(format_human_size(1, false), "1B");
    }

    #[test]
    fn human_size_below_kib() {
        assert_eq!(format_human_size(999, false), "999B");
    }

    #[test]
    fn human_size_1023_bytes() {
        assert_eq!(format_human_size(1023, false), "1023B");
    }

    #[test]
    fn human_size_exact_kib() {
        assert_eq!(format_human_size(1024, false), "1.0KiB");
    }

    #[test]
    fn human_size_1_5_kib() {
        assert_eq!(format_human_size(1536, false), "1.5KiB");
    }

    #[test]
    fn human_size_exact_mib() {
        assert_eq!(format_human_size(1048576, false), "1.0MiB");
    }

    #[test]
    fn human_size_exact_gib() {
        assert_eq!(format_human_size(1073741824, false), "1.0GiB");
    }

    #[test]
    fn human_size_exact_tib() {
        assert_eq!(format_human_size(1099511627776, false), "1.0TiB");
    }

    #[test]
    fn human_size_exact_pib() {
        assert_eq!(format_human_size(1125899906842624, false), "1.0PiB");
    }

    #[test]
    fn human_size_large_value() {
        // 2.5 GiB
        let size = 2 * 1073741824 + 1073741824 / 2;
        assert_eq!(format_human_size(size, false), "2.5GiB");
    }

    // ══════════════════════════════════════════════
    // format_human_size — SI (base 1000)
    // ══════════════════════════════════════════════

    #[test]
    fn human_size_si_zero() {
        assert_eq!(format_human_size(0, true), "0B");
    }

    #[test]
    fn human_size_si_below_kb() {
        assert_eq!(format_human_size(999, true), "999B");
    }

    #[test]
    fn human_size_si_exact_kb() {
        assert_eq!(format_human_size(1000, true), "1.0KB");
    }

    #[test]
    fn human_size_si_1500() {
        assert_eq!(format_human_size(1500, true), "1.5KB");
    }

    #[test]
    fn human_size_si_exact_mb() {
        assert_eq!(format_human_size(1_000_000, true), "1.0MB");
    }

    #[test]
    fn human_size_si_exact_gb() {
        assert_eq!(format_human_size(1_000_000_000, true), "1.0GB");
    }

    #[test]
    fn human_size_si_exact_tb() {
        assert_eq!(format_human_size(1_000_000_000_000, true), "1.0TB");
    }

    #[test]
    fn human_size_si_vs_iec_differ() {
        // 1024 bytes: IEC = "1.0KiB", SI = "1.0KB" (since 1024 >= 1000)
        let iec = format_human_size(1024, false);
        let si = format_human_size(1024, true);
        assert_ne!(iec, si);
        assert!(iec.contains("KiB"));
        assert!(si.contains("KB"));
    }

    // ══════════════════════════════════════════════
    // is_bidi_or_zw
    // ══════════════════════════════════════════════

    #[test]
    fn bidi_regular_ascii_false() {
        for c in ['a', 'Z', '0', ' ', '!', '/'] {
            assert!(!is_bidi_or_zw(c), "ASCII '{}' should not be bidi/zw", c);
        }
    }

    #[test]
    fn bidi_zero_width_space() {
        assert!(is_bidi_or_zw('\u{200B}'));
    }

    #[test]
    fn bidi_zwnj() {
        assert!(is_bidi_or_zw('\u{200C}'));
    }

    #[test]
    fn bidi_zwj() {
        assert!(is_bidi_or_zw('\u{200D}'));
    }

    #[test]
    fn bidi_lrm() {
        assert!(is_bidi_or_zw('\u{200E}'));
    }

    #[test]
    fn bidi_rlm() {
        assert!(is_bidi_or_zw('\u{200F}'));
    }

    #[test]
    fn bidi_embedding_range() {
        for cp in 0x202A..=0x202E {
            let c = char::from_u32(cp).unwrap();
            assert!(is_bidi_or_zw(c), "U+{:04X} should be bidi", cp);
        }
    }

    #[test]
    fn bidi_word_joiner() {
        assert!(is_bidi_or_zw('\u{2060}'));
    }

    #[test]
    fn bidi_isolates_range() {
        for cp in 0x2066..=0x2069 {
            let c = char::from_u32(cp).unwrap();
            assert!(is_bidi_or_zw(c), "U+{:04X} should be bidi", cp);
        }
    }

    #[test]
    fn bidi_bom() {
        assert!(is_bidi_or_zw('\u{FEFF}'));
    }

    #[test]
    fn bidi_just_before_range_false() {
        assert!(!is_bidi_or_zw('\u{200A}')); // hair space
        assert!(!is_bidi_or_zw('\u{2029}')); // paragraph separator
    }

    #[test]
    fn bidi_just_after_range_false() {
        assert!(!is_bidi_or_zw('\u{2010}')); // hyphen
        assert!(!is_bidi_or_zw('\u{202F}')); // NNBSP
        assert!(!is_bidi_or_zw('\u{206A}')); // inhibit symmetric swapping (outside range)
    }

    #[test]
    fn bidi_regular_unicode_false() {
        assert!(!is_bidi_or_zw('ñ'));
        assert!(!is_bidi_or_zw('日'));
        assert!(!is_bidi_or_zw('🎉'));
    }

    // ══════════════════════════════════════════════
    // escape_xml
    // ══════════════════════════════════════════════

    #[test]
    fn xml_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn xml_no_special_chars() {
        assert_eq!(escape_xml("hello world"), "hello world");
    }

    #[test]
    fn xml_ampersand() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
    }

    #[test]
    fn xml_less_than() {
        assert_eq!(escape_xml("a<b"), "a&lt;b");
    }

    #[test]
    fn xml_greater_than() {
        assert_eq!(escape_xml("a>b"), "a&gt;b");
    }

    #[test]
    fn xml_double_quote() {
        assert_eq!(escape_xml("a\"b"), "a&quot;b");
    }

    #[test]
    fn xml_apostrophe() {
        assert_eq!(escape_xml("a'b"), "a&apos;b");
    }

    #[test]
    fn xml_all_special() {
        assert_eq!(escape_xml("&<>\"'"), "&amp;&lt;&gt;&quot;&apos;");
    }

    #[test]
    fn xml_mixed_content() {
        assert_eq!(
            escape_xml("Tom & Jerry <show>"),
            "Tom &amp; Jerry &lt;show&gt;"
        );
    }

    #[test]
    fn xml_strips_control_chars() {
        assert_eq!(escape_xml("hello\x00world"), "helloworld");
        assert_eq!(escape_xml("a\x01b"), "ab");
    }

    #[test]
    fn xml_preserves_tab_and_newline() {
        assert_eq!(escape_xml("a\tb\nc"), "a\tb\nc");
    }

    #[test]
    fn xml_strips_bidi() {
        assert_eq!(escape_xml("test\u{202E}gpj.exe"), "testgpj.exe");
    }

    #[test]
    fn xml_strips_zwj() {
        assert_eq!(escape_xml("join\u{200D}er"), "joiner");
    }

    #[test]
    fn xml_double_ampersand() {
        assert_eq!(escape_xml("a&&b"), "a&amp;&amp;b");
    }

    // ══════════════════════════════════════════════
    // escape_html
    // ══════════════════════════════════════════════

    #[test]
    fn html_empty_string() {
        assert_eq!(escape_html(""), "");
    }

    #[test]
    fn html_no_special() {
        assert_eq!(escape_html("hello"), "hello");
    }

    #[test]
    fn html_ampersand() {
        assert_eq!(escape_html("a&b"), "a&amp;b");
    }

    #[test]
    fn html_less_than() {
        assert_eq!(escape_html("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn html_double_quote() {
        assert_eq!(escape_html("a\"b"), "a&quot;b");
    }

    #[test]
    fn html_apostrophe_uses_numeric() {
        // HTML uses &#39; not &apos;
        assert_eq!(escape_html("it's"), "it&#39;s");
    }

    #[test]
    fn html_apostrophe_differs_from_xml() {
        let xml = escape_xml("it's");
        let html = escape_html("it's");
        assert_ne!(xml, html);
        assert!(xml.contains("&apos;"));
        assert!(html.contains("&#39;"));
    }

    #[test]
    fn html_full_title_escaping() {
        assert_eq!(
            escape_html("A&B 'quoted' <tag>"),
            "A&amp;B &#39;quoted&#39; &lt;tag&gt;"
        );
    }

    #[test]
    fn html_strips_bidi() {
        assert_eq!(escape_html("evil\u{202E}txt"), "eviltxt");
    }

    #[test]
    fn html_strips_control() {
        assert_eq!(escape_html("a\x00b\x01c"), "abc");
    }

    #[test]
    fn html_preserves_whitespace() {
        assert_eq!(escape_html("a\tb\nc"), "a\tb\nc");
    }

    // ══════════════════════════════════════════════
    // encode_uri_path
    // ══════════════════════════════════════════════

    #[test]
    fn uri_empty() {
        assert_eq!(encode_uri_path(""), "");
    }

    #[test]
    fn uri_simple_filename() {
        assert_eq!(encode_uri_path("file.txt"), "file.txt");
    }

    #[test]
    fn uri_preserves_slashes() {
        assert_eq!(encode_uri_path("a/b/c"), "a/b/c");
    }

    #[test]
    fn uri_preserves_unreserved() {
        assert_eq!(encode_uri_path("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn uri_encodes_space() {
        assert_eq!(encode_uri_path("hello world"), "hello%20world");
    }

    #[test]
    fn uri_encodes_hash() {
        assert_eq!(encode_uri_path("report#2024"), "report%232024");
    }

    #[test]
    fn uri_encodes_percent() {
        assert_eq!(encode_uri_path("50%off"), "50%25off");
    }

    #[test]
    fn uri_encodes_question_mark() {
        assert_eq!(encode_uri_path("file?q=1"), "file%3Fq%3D1");
    }

    #[test]
    fn uri_encodes_ampersand() {
        assert_eq!(encode_uri_path("a&b"), "a%26b");
    }

    #[test]
    fn uri_encodes_at_sign() {
        assert_eq!(encode_uri_path("user@host"), "user%40host");
    }

    #[test]
    fn uri_encodes_brackets() {
        assert_eq!(encode_uri_path("[file]"), "%5Bfile%5D");
    }

    #[test]
    fn uri_path_with_dirs() {
        assert_eq!(
            encode_uri_path("docs/my file/readme.md"),
            "docs/my%20file/readme.md"
        );
    }

    #[test]
    fn uri_multibyte_utf8() {
        let encoded = encode_uri_path("café");
        assert!(encoded.starts_with("caf"));
        assert!(encoded.contains("%"));
        // é = U+00E9 = 0xC3 0xA9
        assert!(encoded.contains("%C3%A9"));
    }

    // ══════════════════════════════════════════════
    // entry_type_str
    // ══════════════════════════════════════════════

    #[test]
    fn type_str_file() {
        assert_eq!(entry_type_str(&EntryType::File), "file");
    }

    #[test]
    fn type_str_directory() {
        assert_eq!(entry_type_str(&EntryType::Directory), "directory");
    }

    #[test]
    fn type_str_symlink() {
        let s = EntryType::Symlink {
            target: PathBuf::from("t"),
            broken: false,
        };
        assert_eq!(entry_type_str(&s), "link");
    }

    #[test]
    fn type_str_broken_symlink() {
        let s = EntryType::Symlink {
            target: PathBuf::from("gone"),
            broken: true,
        };
        assert_eq!(entry_type_str(&s), "link");
    }

    #[test]
    fn type_str_junction() {
        let j = EntryType::Junction {
            target: PathBuf::from("t"),
        };
        assert_eq!(entry_type_str(&j), "link");
    }

    #[test]
    fn type_str_hardlink() {
        let h = EntryType::HardLink { link_count: 3 };
        assert_eq!(entry_type_str(&h), "file");
    }

    #[test]
    fn type_str_ads() {
        let a = EntryType::Ads {
            stream_name: "data".into(),
        };
        assert_eq!(entry_type_str(&a), "stream");
    }

    #[test]
    fn type_str_other() {
        assert_eq!(entry_type_str(&EntryType::Other), "other");
    }

    // ══════════════════════════════════════════════
    // count_stats
    // ══════════════════════════════════════════════

    #[test]
    fn stats_file_increments_files() {
        let mut stats = TreeStats::default();
        count_stats(&make_entry("f.txt", EntryType::File), &mut stats);
        assert_eq!(stats.files, 1);
        assert_eq!(stats.directories, 0);
        assert_eq!(stats.symlinks, 0);
    }

    #[test]
    fn stats_directory_increments_dirs() {
        let mut stats = TreeStats::default();
        count_stats(&make_entry("src", EntryType::Directory), &mut stats);
        assert_eq!(stats.directories, 1);
        assert_eq!(stats.files, 0);
    }

    #[test]
    fn stats_multiple_files() {
        let mut stats = TreeStats::default();
        for i in 0..5 {
            count_stats(&make_entry(&format!("f{i}"), EntryType::File), &mut stats);
        }
        assert_eq!(stats.files, 5);
    }

    #[test]
    fn stats_mixed() {
        let mut stats = TreeStats::default();
        count_stats(&make_entry("src", EntryType::Directory), &mut stats);
        count_stats(&make_entry("lib.rs", EntryType::File), &mut stats);
        count_stats(&make_entry("main.rs", EntryType::File), &mut stats);
        count_stats(&make_entry("docs", EntryType::Directory), &mut stats);
        assert_eq!(stats.directories, 2);
        assert_eq!(stats.files, 2);
    }

    #[test]
    fn stats_hardlink_counts_as_file() {
        let mut stats = TreeStats::default();
        count_stats(
            &make_entry("link", EntryType::HardLink { link_count: 3 }),
            &mut stats,
        );
        assert_eq!(stats.files, 1);
    }

    #[test]
    fn stats_other_counts_as_file() {
        let mut stats = TreeStats::default();
        count_stats(&make_entry("sock", EntryType::Other), &mut stats);
        assert_eq!(stats.files, 1);
    }

    #[test]
    fn stats_ads_counts_as_file() {
        let mut stats = TreeStats::default();
        count_stats(
            &make_entry(
                ":data",
                EntryType::Ads {
                    stream_name: "data".into(),
                },
            ),
            &mut stats,
        );
        assert_eq!(stats.files, 1);
    }

    #[test]
    fn stats_symlink_increments_symlinks() {
        let mut stats = TreeStats::default();
        let entry = make_entry(
            "link",
            EntryType::Symlink {
                target: PathBuf::from("/nonexistent"),
                broken: true,
            },
        );
        count_stats(&entry, &mut stats);
        assert_eq!(stats.symlinks, 1);
        // broken symlink → counts as file
        assert_eq!(stats.files, 1);
        assert_eq!(stats.directories, 0);
    }

    #[test]
    fn stats_symlink_to_real_directory() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target_dir");
        std::fs::create_dir(&target).unwrap();

        let entry = Entry {
            path: dir.path().join("link"), // parent = dir.path()
            name: OsString::from("link"),
            entry_type: EntryType::Symlink {
                target: PathBuf::from("target_dir"), // relative
                broken: false,
            },
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        };

        let mut stats = TreeStats::default();
        count_stats(&entry, &mut stats);
        assert_eq!(stats.symlinks, 1);
        assert_eq!(stats.directories, 1, "symlink to dir counts as directory");
        assert_eq!(stats.files, 0);
    }

    #[test]
    fn stats_symlink_to_file_not_dir() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        std::fs::write(&target, "x").unwrap();

        let entry = Entry {
            path: dir.path().join("link"),
            name: OsString::from("link"),
            entry_type: EntryType::Symlink {
                target: PathBuf::from("target.txt"),
                broken: false,
            },
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        };

        let mut stats = TreeStats::default();
        count_stats(&entry, &mut stats);
        assert_eq!(stats.symlinks, 1);
        assert_eq!(stats.files, 1, "symlink to file counts as file");
        assert_eq!(stats.directories, 0);
    }
}
