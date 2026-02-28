use tree_rs::i18n::{
    self,
    messages::{format_report, get_message, pluralize_en, pluralize_ru, MessageKey},
    Language,
};

#[test]
fn test_language_detection_from_code() {
    assert_eq!(Language::from_code("ru"), Language::Russian);
    assert_eq!(Language::from_code("ru_RU.UTF-8"), Language::Russian);
    assert_eq!(Language::from_code("en"), Language::English);
    assert_eq!(Language::from_code("en_US.UTF-8"), Language::English);
    assert_eq!(Language::from_code("de"), Language::English); // Fallback
}

#[test]
fn test_russian_pluralization() {
    // 1 файл
    assert_eq!(pluralize_ru(1, "файл", "файла", "файлов"), "файл");
    // 2, 3, 4 файла
    assert_eq!(pluralize_ru(2, "файл", "файла", "файлов"), "файла");
    assert_eq!(pluralize_ru(3, "файл", "файла", "файлов"), "файла");
    assert_eq!(pluralize_ru(4, "файл", "файла", "файлов"), "файла");
    // 5-20 файлов
    assert_eq!(pluralize_ru(5, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(11, "файл", "файла", "файлов"), "файлов");
    assert_eq!(pluralize_ru(19, "файл", "файла", "файлов"), "файлов");
    // 21 файл
    assert_eq!(pluralize_ru(21, "файл", "файла", "файлов"), "файл");
    // 22 файла
    assert_eq!(pluralize_ru(22, "файл", "файла", "файлов"), "файла");
    // 100 файлов
    assert_eq!(pluralize_ru(100, "файл", "файла", "файлов"), "файлов");
    // 101 файл
    assert_eq!(pluralize_ru(101, "файл", "файла", "файлов"), "файл");
}

#[test]
fn test_english_pluralization() {
    assert_eq!(pluralize_en(1, "file", "files"), "file");
    assert_eq!(pluralize_en(2, "file", "files"), "files");
    assert_eq!(pluralize_en(0, "file", "files"), "files");
    assert_eq!(pluralize_en(100, "file", "files"), "files");
}

#[test]
fn test_format_report_english() {
    let report = format_report(Language::English, 1, 1);
    assert_eq!(report, "1 directory, 1 file");

    let report = format_report(Language::English, 5, 10);
    assert_eq!(report, "5 directories, 10 files");
}

#[test]
fn test_format_report_russian() {
    let report = format_report(Language::Russian, 1, 1);
    assert_eq!(report, "1 каталог, 1 файл");

    let report = format_report(Language::Russian, 2, 3);
    assert_eq!(report, "2 каталога, 3 файла");

    let report = format_report(Language::Russian, 5, 10);
    assert_eq!(report, "5 каталогов, 10 файлов");

    let report = format_report(Language::Russian, 21, 22);
    assert_eq!(report, "21 каталог, 22 файла");
}

#[test]
fn test_message_keys() {
    // English
    let msg = get_message(Language::English, MessageKey::AppDescription);
    assert!(msg.contains("tree"));

    // Russian
    let msg = get_message(Language::Russian, MessageKey::AppDescription);
    assert!(msg.contains("дерева") || msg.contains("каталог"));
}

#[test]
fn test_error_messages() {
    let msg = get_message(Language::English, MessageKey::ErrNotFound);
    assert!(msg.contains("No such file"));

    let msg = get_message(Language::Russian, MessageKey::ErrNotFound);
    assert!(msg.contains("не найден"));
}
