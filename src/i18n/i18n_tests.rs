//! Unit tests for i18n: pluralization, message keys, language detection, format_report

use super::*;
use crate::i18n::Language;

// ============================================================================
// Russian Pluralization Tests
// ============================================================================

#[test]
fn test_pluralize_ru_one() {
    // 1, 21, 31, 101, 121 — "каталог"
    assert_eq!(
        pluralize_ru(1, "каталог", "каталога", "каталогов"),
        "каталог"
    );
    assert_eq!(
        pluralize_ru(21, "каталог", "каталога", "каталогов"),
        "каталог"
    );
    assert_eq!(
        pluralize_ru(31, "каталог", "каталога", "каталогов"),
        "каталог"
    );
    assert_eq!(
        pluralize_ru(101, "каталог", "каталога", "каталогов"),
        "каталог"
    );
    assert_eq!(
        pluralize_ru(121, "каталог", "каталога", "каталогов"),
        "каталог"
    );
}

#[test]
fn test_pluralize_ru_few() {
    // 2-4, 22-24, 32-34 — "каталога"
    assert_eq!(
        pluralize_ru(2, "каталог", "каталога", "каталогов"),
        "каталога"
    );
    assert_eq!(
        pluralize_ru(3, "каталог", "каталога", "каталогов"),
        "каталога"
    );
    assert_eq!(
        pluralize_ru(4, "каталог", "каталога", "каталогов"),
        "каталога"
    );
    assert_eq!(
        pluralize_ru(22, "каталог", "каталога", "каталогов"),
        "каталога"
    );
    assert_eq!(
        pluralize_ru(23, "каталог", "каталога", "каталогов"),
        "каталога"
    );
    assert_eq!(
        pluralize_ru(34, "каталог", "каталога", "каталогов"),
        "каталога"
    );
}

#[test]
fn test_pluralize_ru_many() {
    // 0, 5-20, 25-30, 100 — "каталогов"
    assert_eq!(
        pluralize_ru(0, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(5, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(10, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(11, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(12, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(14, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(15, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(19, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(20, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
    assert_eq!(
        pluralize_ru(100, "каталог", "каталога", "каталогов"),
        "каталогов"
    );
}

#[test]
fn test_pluralize_ru_teens_special() {
    // 11-19 are always "many" regardless of last digit
    assert_eq!(pluralize_ru(11, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(12, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(13, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(14, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(111, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(112, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(213, "файл", "файла", "файлов"), "файлов");
}

// ============================================================================
// English Pluralization Tests
// ============================================================================

#[test]
fn test_pluralize_en_singular() {
    assert_eq!(pluralize_en(1, "directory", "directories"), "directory");
    assert_eq!(pluralize_en(1, "file", "files"), "file");
}

#[test]
fn test_pluralize_en_plural() {
    assert_eq!(pluralize_en(0, "directory", "directories"), "directories");
    assert_eq!(pluralize_en(2, "directory", "directories"), "directories");
    assert_eq!(pluralize_en(5, "file", "files"), "files");
    assert_eq!(pluralize_en(100, "file", "files"), "files");
}

// ============================================================================
// format_report Tests
// ============================================================================

#[test]
fn test_format_report_en_singular() {
    assert_eq!(
        format_report(Language::English, 1, 1),
        "1 directory, 1 file"
    );
}

#[test]
fn test_format_report_en_plural() {
    assert_eq!(
        format_report(Language::English, 0, 0),
        "0 directories, 0 files"
    );
    assert_eq!(
        format_report(Language::English, 3, 5),
        "3 directories, 5 files"
    );
    assert_eq!(
        format_report(Language::English, 10, 100),
        "10 directories, 100 files"
    );
}

#[test]
fn test_format_report_ru_one() {
    assert_eq!(format_report(Language::Russian, 1, 1), "1 каталог, 1 файл");
    assert_eq!(
        format_report(Language::Russian, 21, 21),
        "21 каталог, 21 файл"
    );
}

#[test]
fn test_format_report_ru_few() {
    assert_eq!(
        format_report(Language::Russian, 2, 3),
        "2 каталога, 3 файла"
    );
    assert_eq!(
        format_report(Language::Russian, 4, 4),
        "4 каталога, 4 файла"
    );
}

#[test]
fn test_format_report_ru_many() {
    assert_eq!(
        format_report(Language::Russian, 0, 0),
        "0 каталогов, 0 файлов"
    );
    assert_eq!(
        format_report(Language::Russian, 5, 5),
        "5 каталогов, 5 файлов"
    );
    assert_eq!(
        format_report(Language::Russian, 11, 15),
        "11 каталогов, 15 файлов"
    );
}

#[test]
fn test_format_report_ru_mixed() {
    // 1 каталог, 5 файлов
    assert_eq!(
        format_report(Language::Russian, 1, 5),
        "1 каталог, 5 файлов"
    );
    // 3 каталога, 11 файлов
    assert_eq!(
        format_report(Language::Russian, 3, 11),
        "3 каталога, 11 файлов"
    );
    // 13 каталогов, 1 файл
    assert_eq!(
        format_report(Language::Russian, 13, 1),
        "13 каталогов, 1 файл"
    );
}

// ============================================================================
// Language::from_code Tests
// ============================================================================

#[test]
fn test_language_from_code_russian() {
    assert_eq!(Language::from_code("ru"), Language::Russian);
    assert_eq!(Language::from_code("RU"), Language::Russian);
    assert_eq!(Language::from_code("ru_RU"), Language::Russian);
    assert_eq!(Language::from_code("ru_RU.UTF-8"), Language::Russian);
}

#[test]
fn test_language_from_code_english() {
    assert_eq!(Language::from_code("en"), Language::English);
    assert_eq!(Language::from_code("EN"), Language::English);
    assert_eq!(Language::from_code("en_US"), Language::English);
    assert_eq!(Language::from_code("en_US.UTF-8"), Language::English);
}

#[test]
fn test_language_from_code_unknown_defaults_english() {
    assert_eq!(Language::from_code("de"), Language::English);
    assert_eq!(Language::from_code("fr"), Language::English);
    assert_eq!(Language::from_code("ja"), Language::English);
    assert_eq!(Language::from_code(""), Language::English);
}

// ============================================================================
// Language::code Tests
// ============================================================================

#[test]
fn test_language_code() {
    assert_eq!(Language::English.code(), "en");
    assert_eq!(Language::Russian.code(), "ru");
}

// ============================================================================
// get_message Tests — All Keys Return Non-Empty
// ============================================================================

#[test]
fn test_all_message_keys_en_non_empty() {
    let keys = all_message_keys();
    for key in &keys {
        let msg = get_message(Language::English, *key);
        assert!(!msg.is_empty(), "English message for {:?} is empty", key);
    }
}

#[test]
fn test_all_message_keys_ru_non_empty() {
    let keys = all_message_keys();
    for key in &keys {
        let msg = get_message(Language::Russian, *key);
        assert!(!msg.is_empty(), "Russian message for {:?} is empty", key);
    }
}

// ============================================================================
// get_message Tests — Specific Key Values
// ============================================================================

#[test]
fn test_broken_link_messages() {
    assert_eq!(
        get_message(Language::English, MessageKey::BrokenLink),
        "broken"
    );
    assert_eq!(
        get_message(Language::Russian, MessageKey::BrokenLink),
        "битая ссылка"
    );
}

#[test]
fn test_entry_type_messages() {
    assert_eq!(get_message(Language::English, MessageKey::TypeFile), "file");
    assert_eq!(get_message(Language::Russian, MessageKey::TypeFile), "файл");
    assert_eq!(
        get_message(Language::English, MessageKey::TypeDirectory),
        "directory"
    );
    assert_eq!(
        get_message(Language::Russian, MessageKey::TypeDirectory),
        "каталог"
    );
}

#[test]
fn test_html_title_messages() {
    assert_eq!(
        get_message(Language::English, MessageKey::HtmlTitle),
        "Directory Tree"
    );
    assert_eq!(
        get_message(Language::Russian, MessageKey::HtmlTitle),
        "Дерево каталогов"
    );
}

#[test]
fn test_xml_encoding_same_both_languages() {
    assert_eq!(
        get_message(Language::English, MessageKey::XmlEncoding),
        get_message(Language::Russian, MessageKey::XmlEncoding)
    );
    assert_eq!(
        get_message(Language::English, MessageKey::XmlEncoding),
        "UTF-8"
    );
}

// ============================================================================
// get_message Tests — EN vs RU Differ Where Expected
// ============================================================================

#[test]
fn test_messages_differ_between_languages() {
    // These should definitely be different
    let must_differ = vec![
        MessageKey::AppDescription,
        MessageKey::ArgAll,
        MessageKey::ArgDirsOnly,
        MessageKey::Directories,
        MessageKey::Files,
        MessageKey::Directory,
        MessageKey::File,
        MessageKey::BrokenLink,
        MessageKey::TypeFile,
        MessageKey::TypeDirectory,
        MessageKey::HtmlTitle,
    ];

    for key in must_differ {
        let en = get_message(Language::English, key);
        let ru = get_message(Language::Russian, key);
        assert_ne!(
            en, ru,
            "Messages for {:?} should differ between EN and RU, both are: {:?}",
            key, en
        );
    }
}

// ============================================================================
// Helper: list all MessageKey variants
// ============================================================================

fn all_message_keys() -> Vec<MessageKey> {
    vec![
        MessageKey::AppDescription,
        MessageKey::AppAfterHelp,
        MessageKey::ArgPaths,
        MessageKey::ArgAll,
        MessageKey::ArgDirsOnly,
        MessageKey::ArgFollow,
        MessageKey::ArgFullPath,
        MessageKey::ArgOneFs,
        MessageKey::ArgLevel,
        MessageKey::ArgFileLimit,
        MessageKey::ArgNoReport,
        MessageKey::ArgPattern,
        MessageKey::ArgExclude,
        MessageKey::ArgMatchDirs,
        MessageKey::ArgIgnoreCase,
        MessageKey::ArgPrune,
        MessageKey::ArgVersionSort,
        MessageKey::ArgTimeSort,
        MessageKey::ArgCtimeSort,
        MessageKey::ArgUnsorted,
        MessageKey::ArgReverse,
        MessageKey::ArgDirsFirst,
        MessageKey::ArgFilesFirst,
        MessageKey::ArgSort,
        MessageKey::ArgNoIndent,
        MessageKey::ArgAnsi,
        MessageKey::ArgCp437,
        MessageKey::ArgNoColor,
        MessageKey::ArgColorAlways,
        MessageKey::ArgColor,
        MessageKey::ArgSize,
        MessageKey::ArgHuman,
        MessageKey::ArgSi,
        MessageKey::ArgDate,
        MessageKey::ArgTimeFmt,
        MessageKey::ArgPerm,
        MessageKey::ArgUid,
        MessageKey::ArgGid,
        MessageKey::ArgInodes,
        MessageKey::ArgDevice,
        MessageKey::ArgClassify,
        MessageKey::ArgSafe,
        MessageKey::ArgLiteral,
        MessageKey::ArgCharset,
        MessageKey::ArgOutput,
        MessageKey::ArgHtml,
        MessageKey::ArgTitle,
        MessageKey::ArgNoLinks,
        MessageKey::ArgHtmlIntro,
        MessageKey::ArgHtmlOutro,
        MessageKey::ArgXml,
        MessageKey::ArgJson,
        MessageKey::ArgIcons,
        MessageKey::ArgNoIcons,
        MessageKey::ArgIconStyle,
        MessageKey::ArgShowStreams,
        MessageKey::ArgShowJunctions,
        MessageKey::ArgHideSystem,
        MessageKey::ArgPermissions,
        MessageKey::ArgLongPaths,
        MessageKey::ArgLang,
        MessageKey::SortName,
        MessageKey::SortSize,
        MessageKey::SortMtime,
        MessageKey::SortCtime,
        MessageKey::SortVersion,
        MessageKey::SortNone,
        MessageKey::ColorAuto,
        MessageKey::ColorAlways,
        MessageKey::ColorNever,
        MessageKey::IconsAuto,
        MessageKey::IconsAlways,
        MessageKey::IconsNever,
        MessageKey::IconStyleNerd,
        MessageKey::IconStyleUnicode,
        MessageKey::IconStyleAscii,
        MessageKey::PermPosix,
        MessageKey::PermWindows,
        MessageKey::Directories,
        MessageKey::Files,
        MessageKey::Directory,
        MessageKey::File,
        MessageKey::DirectoriesAndFiles,
        MessageKey::ErrAccessDenied,
        MessageKey::ErrNotFound,
        MessageKey::ErrNotDirectory,
        MessageKey::ErrSymlinkLoop,
        MessageKey::ErrSymlinkError,
        MessageKey::ErrPathTooLong,
        MessageKey::ErrInvalidName,
        MessageKey::ErrIo,
        MessageKey::ErrInvalidPattern,
        MessageKey::ErrConfig,
        MessageKey::BrokenLink,
        MessageKey::ExceedsFileLimit,
        MessageKey::HtmlTitle,
        MessageKey::XmlEncoding,
        MessageKey::TypeFile,
        MessageKey::TypeDirectory,
        MessageKey::TypeLink,
        MessageKey::TypeJunction,
        MessageKey::TypeStream,
        MessageKey::TypeOther,
        MessageKey::ReportFormat,
    ]
}
