// Определяет язык из сырых аргументов командной строки и переменных окружения
// ДО полного парсинга clap. Нужно для вывода --help на правильном языке.

use crate::i18n::Language;

/// Сканирует `std::env::args()` в поисках `--lang <VALUE>` или `--lang=<VALUE>`,
/// затем проверяет `TREE_LANG`. Если ничего не найдено — делегирует `Language::detect()`.
pub fn detect_language_early() -> Language {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1).peekable();

    while let Some(arg) = iter.next() {
        if arg == "--lang" {
            if let Some(value) = iter.next() {
                return Language::from_code(value);
            }
        } else if let Some(value) = arg.strip_prefix("--lang=") {
            return Language::from_code(value);
        }
    }

    // Проверяем TREE_LANG (env var), затем системный язык
    Language::detect()
}

/// Возвращает `true`, если в аргументах присутствует флаг --help.
pub fn has_help_flag() -> bool {
    std::env::args().skip(1).any(|a| a == "--help")
}
