use super::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum MessageKey {
    // CLI Help
    AppDescription,
    AppAfterHelp,

    // Arguments
    ArgPaths,
    ArgAll,
    ArgDirsOnly,
    ArgFollow,
    ArgFullPath,
    ArgOneFs,
    ArgLevel,
    ArgFileLimit,
    ArgNoReport,
    ArgPattern,
    ArgExclude,
    ArgMatchDirs,
    ArgIgnoreCase,
    ArgPrune,
    ArgVersionSort,
    ArgTimeSort,
    ArgCtimeSort,
    ArgUnsorted,
    ArgReverse,
    ArgDirsFirst,
    ArgFilesFirst,
    ArgSort,
    ArgNoIndent,
    ArgAnsi,
    ArgCp437,
    ArgNoColor,
    ArgColorAlways,
    ArgColor,
    ArgSize,
    ArgHuman,
    ArgSi,
    ArgDate,
    ArgTimeFmt,
    ArgPerm,
    ArgUid,
    ArgGid,
    ArgInodes,
    ArgDevice,
    ArgClassify,
    ArgSafe,
    ArgLiteral,
    ArgCharset,
    ArgOutput,
    ArgHtml,
    ArgTitle,
    ArgNoLinks,
    ArgHtmlIntro,
    ArgHtmlOutro,
    ArgXml,
    ArgJson,
    ArgIcons,
    ArgNoIcons,
    ArgIconStyle,
    ArgShowStreams,
    ArgShowJunctions,
    ArgHideSystem,
    ArgPermissions,
    ArgLongPaths,
    ArgLang,
    ArgVersion,
    ArgHelp,
    ArgJsonPretty,
    ArgParallel,
    ArgStreaming,
    ArgThreads,
    ArgQueueCap,
    ArgMaxEntries,

    // Sort types
    SortName,
    SortSize,
    SortMtime,
    SortCtime,
    SortVersion,
    SortNone,

    // Color options
    ColorAuto,
    ColorAlways,
    ColorNever,

    // Icon options
    IconsAuto,
    IconsAlways,
    IconsNever,
    IconStyleNerd,
    IconStyleUnicode,
    IconStyleAscii,

    // Permission modes
    PermPosix,
    PermWindows,

    // Output messages
    Directories,
    Files,
    Directory,
    File,
    DirectoriesAndFiles,

    // Error messages
    ErrAccessDenied,
    ErrNotFound,
    ErrNotDirectory,
    ErrSymlinkLoop,
    ErrSymlinkError,
    ErrPathTooLong,
    ErrReservedName,
    ErrIo,
    ErrInvalidPattern,

    // Misc
    BrokenLink,
    RecursiveLink,
    ExceedsFileLimit,
    HtmlTitle,
    XmlEncoding,

    // Entry types
    TypeFile,
    TypeDirectory,
    TypeLink,
    TypeJunction,
    TypeStream,
    TypeOther,

    // Report
    ReportFormat,

    // Section headings
    HeadingOptions,
    HeadingListingOptions,
    HeadingFiltering,
    HeadingSorting,
    HeadingDisplay,
    HeadingFileInformation,
    HeadingExport,
    HeadingPerformance,
    HeadingIcons,
    HeadingWindows,
    HeadingLocalization,
}

pub fn get_message(lang: Language, key: MessageKey) -> &'static str {
    match lang {
        Language::English => get_message_en(key),
        Language::Russian => get_message_ru(key),
    }
}

fn get_message_en(key: MessageKey) -> &'static str {
    match key {
        // CLI Help
        MessageKey::AppDescription => "List directory contents in a tree-like format",
        MessageKey::AppAfterHelp => "GNU tree compatible CLI for Windows",

        // Arguments
        MessageKey::ArgPaths => "Directories to list",
        MessageKey::ArgAll => "Show all files including hidden",
        MessageKey::ArgDirsOnly => "List directories only",
        MessageKey::ArgFollow => "Follow symbolic links",
        MessageKey::ArgFullPath => "Print full path prefix for each file",
        MessageKey::ArgOneFs => "Stay on current filesystem",
        MessageKey::ArgLevel => "Descend only N levels deep",
        MessageKey::ArgFileLimit => "Do not descend dirs with more than N entries",
        MessageKey::ArgNoReport => "Omit final report of directories/files count",
        MessageKey::ArgPattern => "List only files matching pattern",
        MessageKey::ArgExclude => "Exclude files matching pattern",
        MessageKey::ArgMatchDirs => "Apply patterns to directories as well",
        MessageKey::ArgIgnoreCase => "Case insensitive pattern matching",
        MessageKey::ArgPrune => "Do not print empty directories",
        MessageKey::ArgVersionSort => "Natural sort (version sort)",
        MessageKey::ArgTimeSort => "Sort by modification time",
        MessageKey::ArgCtimeSort => "Sort by change time (Windows: metadata change)",
        MessageKey::ArgUnsorted => "Leave files unsorted",
        MessageKey::ArgReverse => "Reverse sort order",
        MessageKey::ArgDirsFirst => "List directories before files",
        MessageKey::ArgFilesFirst => "List files before directories",
        MessageKey::ArgSort => "Sort by specific criteria",
        MessageKey::ArgNoIndent => "Don't print indentation lines",
        MessageKey::ArgAnsi => "Use ANSI line graphics",
        MessageKey::ArgCp437 => "Use CP437 line graphics",
        MessageKey::ArgNoColor => "Turn colorization off",
        MessageKey::ArgColorAlways => "Turn colorization on always",
        MessageKey::ArgColor => "When to use color",
        MessageKey::ArgSize => "Print size in bytes",
        MessageKey::ArgHuman => "Print human readable sizes",
        MessageKey::ArgSi => "Use SI units (powers of 1000)",
        MessageKey::ArgDate => "Print modification date",
        MessageKey::ArgTimeFmt => "Time format string",
        MessageKey::ArgPerm => "Print permissions",
        MessageKey::ArgUid => "Print file owner",
        MessageKey::ArgGid => "Print file group",
        MessageKey::ArgInodes => "Print inode number",
        MessageKey::ArgDevice => "Print device number",
        MessageKey::ArgClassify => "Append file type indicator",
        MessageKey::ArgSafe => "Replace non-printable chars with ?",
        MessageKey::ArgLiteral => "Print non-printable chars as-is",
        MessageKey::ArgCharset => "Output character encoding",
        MessageKey::ArgOutput => "Output to file",
        MessageKey::ArgHtml => "HTML output with base URL",
        MessageKey::ArgTitle => "HTML page title",
        MessageKey::ArgNoLinks => "Turn off hyperlinks in HTML",
        MessageKey::ArgHtmlIntro => "Use custom HTML intro file",
        MessageKey::ArgHtmlOutro => "Use custom HTML outro file",
        MessageKey::ArgXml => "XML output",
        MessageKey::ArgJson => "JSON output",
        MessageKey::ArgIcons => "Show icons (auto/always/never)",
        MessageKey::ArgNoIcons => "Disable icons",
        MessageKey::ArgIconStyle => "Icon style",
        MessageKey::ArgShowStreams => "Show NTFS Alternate Data Streams",
        MessageKey::ArgShowJunctions => "Show junction points with targets",
        MessageKey::ArgHideSystem => "Hide system files even with -a",
        MessageKey::ArgPermissions => "Permission format (posix/windows)",
        MessageKey::ArgLongPaths => "Force long path prefix (\\\\?\\)",
        MessageKey::ArgLang => "Interface language (en/ru)",
        MessageKey::ArgVersion => "Print version",
        MessageKey::ArgHelp => "Print help information",
        MessageKey::ArgJsonPretty => "Pretty-print JSON output (fully formatted)",
        MessageKey::ArgParallel => {
            "Enable parallel directory traversal (faster for large directories)"
        }
        MessageKey::ArgStreaming => {
            "Streaming mode: render output during traversal without building full"
        }
        MessageKey::ArgThreads => "Number of worker threads for parallel mode (default: CPU cores)",
        MessageKey::ArgQueueCap => "Maximum concurrent directory reads in parallel mode",
        MessageKey::ArgMaxEntries => "Maximum total entries to display (limits memory usage)",

        // Sort types
        MessageKey::SortName => "name",
        MessageKey::SortSize => "size",
        MessageKey::SortMtime => "mtime",
        MessageKey::SortCtime => "ctime",
        MessageKey::SortVersion => "version",
        MessageKey::SortNone => "none",

        // Color options
        MessageKey::ColorAuto => "auto",
        MessageKey::ColorAlways => "always",
        MessageKey::ColorNever => "never",

        // Icon options
        MessageKey::IconsAuto => "auto",
        MessageKey::IconsAlways => "always",
        MessageKey::IconsNever => "never",
        MessageKey::IconStyleNerd => "nerd",
        MessageKey::IconStyleUnicode => "unicode",
        MessageKey::IconStyleAscii => "ascii",

        // Permission modes
        MessageKey::PermPosix => "posix",
        MessageKey::PermWindows => "windows",

        // Output messages
        MessageKey::Directories => "directories",
        MessageKey::Files => "files",
        MessageKey::Directory => "directory",
        MessageKey::File => "file",
        MessageKey::DirectoriesAndFiles => "{} directories, {} files",

        // Error messages
        MessageKey::ErrAccessDenied => "cannot access '{}': Permission denied",
        MessageKey::ErrNotFound => "'{}': No such file or directory",
        MessageKey::ErrNotDirectory => "'{}': Not a directory",
        MessageKey::ErrSymlinkLoop => "'{}': Symbolic link loop detected",
        MessageKey::ErrSymlinkError => "'{}': {}",
        MessageKey::ErrPathTooLong => "'{}': Path too long",
        MessageKey::ErrReservedName => "Reserved Windows device name (skipped): '{}'",
        MessageKey::ErrIo => "'{}': {}",
        MessageKey::ErrInvalidPattern => "Invalid pattern: {}",

        // Misc
        MessageKey::BrokenLink => "broken",
        MessageKey::RecursiveLink => "recursive, not followed",
        MessageKey::ExceedsFileLimit => "{} entries exceeds filelimit, not opening dir",
        MessageKey::HtmlTitle => "Directory Tree",
        MessageKey::XmlEncoding => "UTF-8",

        // Entry types
        MessageKey::TypeFile => "file",
        MessageKey::TypeDirectory => "directory",
        MessageKey::TypeLink => "link",
        MessageKey::TypeJunction => "junction",
        MessageKey::TypeStream => "stream",
        MessageKey::TypeOther => "other",

        // Report
        MessageKey::ReportFormat => "{} {}, {} {}",

        // Section headings
        MessageKey::HeadingOptions => "Options",
        MessageKey::HeadingListingOptions => "Listing Options",
        MessageKey::HeadingFiltering => "Filtering",
        MessageKey::HeadingSorting => "Sorting",
        MessageKey::HeadingDisplay => "Display",
        MessageKey::HeadingFileInformation => "File Information",
        MessageKey::HeadingExport => "Export",
        MessageKey::HeadingPerformance => "Performance",
        MessageKey::HeadingIcons => "Icons",
        MessageKey::HeadingWindows => "Windows",
        MessageKey::HeadingLocalization => "Localization",
    }
}

fn get_message_ru(key: MessageKey) -> &'static str {
    match key {
        // CLI Help
        MessageKey::AppDescription => "Отображение содержимого каталогов в виде дерева",
        MessageKey::AppAfterHelp => "GNU tree-совместимый CLI для Windows",

        // Arguments
        MessageKey::ArgPaths => "Каталоги для отображения",
        MessageKey::ArgAll => "Показывать все файлы, включая скрытые",
        MessageKey::ArgDirsOnly => "Показывать только каталоги",
        MessageKey::ArgFollow => "Следовать по символическим ссылкам",
        MessageKey::ArgFullPath => "Выводить полный путь для каждого файла",
        MessageKey::ArgOneFs => "Не выходить за пределы файловой системы",
        MessageKey::ArgLevel => "Глубина обхода (N уровней)",
        MessageKey::ArgFileLimit => "Пропускать каталоги с более чем N элементами",
        MessageKey::ArgNoReport => "Не выводить итоговую статистику",
        MessageKey::ArgPattern => "Показывать только файлы, соответствующие шаблону",
        MessageKey::ArgExclude => "Исключить файлы, соответствующие шаблону",
        MessageKey::ArgMatchDirs => "Применять шаблоны к каталогам",
        MessageKey::ArgIgnoreCase => "Регистронезависимый поиск",
        MessageKey::ArgPrune => "Не показывать пустые каталоги",
        MessageKey::ArgVersionSort => "Натуральная сортировка (версионная)",
        MessageKey::ArgTimeSort => "Сортировка по времени модификации",
        MessageKey::ArgCtimeSort => "Сортировка по времени изменения метаданных",
        MessageKey::ArgUnsorted => "Без сортировки",
        MessageKey::ArgReverse => "Обратный порядок сортировки",
        MessageKey::ArgDirsFirst => "Каталоги в начале списка",
        MessageKey::ArgFilesFirst => "Файлы в начале списка",
        MessageKey::ArgSort => "Сортировать по указанному критерию",
        MessageKey::ArgNoIndent => "Без отступов и линий",
        MessageKey::ArgAnsi => "Использовать ANSI-графику",
        MessageKey::ArgCp437 => "Использовать CP437-графику",
        MessageKey::ArgNoColor => "Отключить цвета",
        MessageKey::ArgColorAlways => "Всегда использовать цвета",
        MessageKey::ArgColor => "Режим цветового вывода",
        MessageKey::ArgSize => "Показывать размер в байтах",
        MessageKey::ArgHuman => "Показывать размер в человекочитаемом формате",
        MessageKey::ArgSi => "Использовать единицы СИ (степени 1000)",
        MessageKey::ArgDate => "Показывать дату модификации",
        MessageKey::ArgTimeFmt => "Формат даты и времени",
        MessageKey::ArgPerm => "Показывать права доступа",
        MessageKey::ArgUid => "Показывать владельца файла",
        MessageKey::ArgGid => "Показывать группу файла",
        MessageKey::ArgInodes => "Показывать номер inode",
        MessageKey::ArgDevice => "Показывать номер устройства",
        MessageKey::ArgClassify => "Добавлять индикатор типа файла",
        MessageKey::ArgSafe => "Заменять непечатаемые символы на ?",
        MessageKey::ArgLiteral => "Выводить непечатаемые символы как есть",
        MessageKey::ArgCharset => "Кодировка вывода",
        MessageKey::ArgOutput => "Записать в файл",
        MessageKey::ArgHtml => "HTML-вывод с базовым URL",
        MessageKey::ArgTitle => "Заголовок HTML-страницы",
        MessageKey::ArgNoLinks => "Отключить гиперссылки в HTML",
        MessageKey::ArgHtmlIntro => "Использовать свой HTML-заголовок",
        MessageKey::ArgHtmlOutro => "Использовать своё HTML-окончание",
        MessageKey::ArgXml => "XML-вывод",
        MessageKey::ArgJson => "JSON-вывод",
        MessageKey::ArgIcons => "Показывать иконки (auto/always/never)",
        MessageKey::ArgNoIcons => "Отключить иконки",
        MessageKey::ArgIconStyle => "Стиль иконок",
        MessageKey::ArgShowStreams => "Показывать альтернативные потоки данных NTFS",
        MessageKey::ArgShowJunctions => "Показывать точки соединения с целевым путём",
        MessageKey::ArgHideSystem => "Скрывать системные файлы даже при -a",
        MessageKey::ArgPermissions => "Формат прав доступа (posix/windows)",
        MessageKey::ArgLongPaths => "Принудительно использовать префикс \\\\?\\",
        MessageKey::ArgLang => "Язык интерфейса (en/ru)",
        MessageKey::ArgVersion => "Показать версию",
        MessageKey::ArgHelp => "Показать справку",
        MessageKey::ArgJsonPretty => "Форматированный JSON-вывод",
        MessageKey::ArgParallel => "Параллельный обход каталогов (быстрее для больших деревьев)",
        MessageKey::ArgStreaming => {
            "Потоковый режим: вывод результатов во время обхода без построения полного дерева"
        }
        MessageKey::ArgThreads => "Количество рабочих потоков (по умолчанию: число ядер ЦП)",
        MessageKey::ArgQueueCap => {
            "Максимальное число одновременных чтений директорий в параллельном режиме"
        }
        MessageKey::ArgMaxEntries => {
            "Максимальное количество выводимых элементов (ограничивает потребление памяти)"
        }

        // Sort types
        MessageKey::SortName => "имя",
        MessageKey::SortSize => "размер",
        MessageKey::SortMtime => "время изменения",
        MessageKey::SortCtime => "время создания",
        MessageKey::SortVersion => "версия",
        MessageKey::SortNone => "без сортировки",

        // Color options
        MessageKey::ColorAuto => "авто",
        MessageKey::ColorAlways => "всегда",
        MessageKey::ColorNever => "никогда",

        // Icon options
        MessageKey::IconsAuto => "авто",
        MessageKey::IconsAlways => "всегда",
        MessageKey::IconsNever => "никогда",
        MessageKey::IconStyleNerd => "nerd",
        MessageKey::IconStyleUnicode => "unicode",
        MessageKey::IconStyleAscii => "ascii",

        // Permission modes
        MessageKey::PermPosix => "posix",
        MessageKey::PermWindows => "windows",

        // Output messages
        MessageKey::Directories => "каталогов",
        MessageKey::Files => "файлов",
        MessageKey::Directory => "каталог",
        MessageKey::File => "файл",
        MessageKey::DirectoriesAndFiles => "{} каталогов, {} файлов",

        // Error messages
        MessageKey::ErrAccessDenied => "нет доступа к '{}': Доступ запрещён",
        MessageKey::ErrNotFound => "'{}': Файл или каталог не найден",
        MessageKey::ErrNotDirectory => "'{}': Не является каталогом",
        MessageKey::ErrSymlinkLoop => "'{}': Обнаружен цикл символических ссылок",
        MessageKey::ErrSymlinkError => "'{}': {}",
        MessageKey::ErrPathTooLong => "'{}': Слишком длинный путь",
        MessageKey::ErrReservedName => "Зарезервированное имя устройства Windows (пропущено): '{}'",
        MessageKey::ErrIo => "'{}': {}",
        MessageKey::ErrInvalidPattern => "Некорректный шаблон: {}",

        // Misc
        MessageKey::BrokenLink => "битая ссылка",
        MessageKey::RecursiveLink => "рекурсивная, не раскрыта",
        MessageKey::ExceedsFileLimit => "{} элементов превышает лимит, каталог не раскрыт",
        MessageKey::HtmlTitle => "Дерево каталогов",
        MessageKey::XmlEncoding => "UTF-8",

        // Entry types
        MessageKey::TypeFile => "файл",
        MessageKey::TypeDirectory => "каталог",
        MessageKey::TypeLink => "ссылка",
        MessageKey::TypeJunction => "точка соединения",
        MessageKey::TypeStream => "поток",
        MessageKey::TypeOther => "другое",

        // Report
        MessageKey::ReportFormat => "{} {}, {} {}",

        // Section headings
        MessageKey::HeadingOptions => "Параметры",
        MessageKey::HeadingListingOptions => "Параметры отображения",
        MessageKey::HeadingFiltering => "Фильтрация",
        MessageKey::HeadingSorting => "Сортировка",
        MessageKey::HeadingDisplay => "Вывод",
        MessageKey::HeadingFileInformation => "Информация о файлах",
        MessageKey::HeadingExport => "Экспорт",
        MessageKey::HeadingPerformance => "Производительность",
        MessageKey::HeadingIcons => "Иконки",
        MessageKey::HeadingWindows => "Windows",
        MessageKey::HeadingLocalization => "Локализация",
    }
}

// Helper function for pluralization in Russian
pub fn pluralize_ru<'a>(count: u64, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let n = count % 100;
    if (11..=19).contains(&n) {
        return many;
    }
    match n % 10 {
        1 => one,
        2..=4 => few,
        _ => many,
    }
}

// Helper function for pluralization in English
pub fn pluralize_en<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

/// Format the directory/file count with proper pluralization
pub fn format_report(lang: Language, dirs: u64, files: u64) -> String {
    match lang {
        Language::English => {
            let dir_word = pluralize_en(dirs, "directory", "directories");
            let file_word = pluralize_en(files, "file", "files");
            format!("{} {}, {} {}", dirs, dir_word, files, file_word)
        }
        Language::Russian => {
            let dir_word = pluralize_ru(dirs, "каталог", "каталога", "каталогов");
            let file_word = pluralize_ru(files, "файл", "файла", "файлов");
            format!("{} {}, {} {}", dirs, dir_word, files, file_word)
        }
    }
}

#[cfg(test)]
#[path = "i18n_tests.rs"]
mod tests;
