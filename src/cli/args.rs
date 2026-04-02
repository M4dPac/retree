pub use crate::core::sorter::SortType;
pub use crate::style::icons::IconStyle;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

// Для справки используем статические английские строки
// Локализованные сообщения используются в runtime

#[derive(Parser, Debug, Clone)]
#[command(name = "rt")]
#[command(author, version)]
#[command(about = "List directory contents in a tree-like format")]
#[command(
    after_help = "GNU tree compatible CLI for Windows\n\nSet TREE_LANG=ru for Russian interface"
)]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Directories to list
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    // === Listing Options ===
    /// Show all files including hidden / Показывать скрытые файлы
    #[arg(short = 'a', long, help_heading = "Listing Options")]
    pub all: bool,

    /// List directories only / Только каталоги
    #[arg(short = 'd', long = "dirs-only", help_heading = "Listing Options")]
    pub dirs_only: bool,

    /// Follow symbolic links / Следовать по симлинкам
    #[arg(short = 'l', long = "follow", help_heading = "Listing Options")]
    pub follow_symlinks: bool,

    /// Print full path prefix for each file / Полный путь
    #[arg(short = 'f', long = "full-path", help_heading = "Listing Options")]
    pub full_path: bool,

    /// Stay on current filesystem / Не выходить за пределы ФС
    #[arg(short = 'x', long = "one-fs", help_heading = "Listing Options")]
    pub one_fs: bool,

    /// Descend only N levels deep / Глубина обхода
    #[arg(
        short = 'L',
        long = "level",
        value_name = "N",
        help_heading = "Listing Options"
    )]
    pub max_depth: Option<usize>,

    /// Do not descend dirs with more than N entries / Лимит файлов
    #[arg(long = "filelimit", value_name = "N", help_heading = "Listing Options")]
    pub file_limit: Option<usize>,

    /// Omit final report / Без статистики
    #[arg(long = "noreport", help_heading = "Listing Options")]
    pub no_report: bool,

    // === Filtering Options ===
    /// List only files matching pattern / Фильтр по шаблону
    #[arg(
        short = 'P',
        long = "pattern",
        value_name = "PATTERN",
        help_heading = "Filtering"
    )]
    pub pattern: Option<String>,

    /// Exclude files matching pattern / Исключить по шаблону
    #[arg(short = 'I', long = "exclude", value_name = "PATTERN", action = clap::ArgAction::Append, help_heading = "Filtering")]
    pub exclude: Vec<String>,

    /// Apply patterns to directories as well / Применять к каталогам
    #[arg(long = "matchdirs", help_heading = "Filtering")]
    pub match_dirs: bool,

    /// Case insensitive pattern matching / Без учёта регистра
    #[arg(long = "ignore-case", help_heading = "Filtering")]
    pub ignore_case: bool,

    /// Do not print empty directories / Без пустых каталогов
    #[arg(long = "prune", help_heading = "Filtering")]
    pub prune: bool,

    // === Sorting Options ===
    /// Natural sort (version sort) / Натуральная сортировка
    #[arg(short = 'v', long = "version-sort", help_heading = "Sorting")]
    pub version_sort: bool,

    /// Sort by modification time / По времени изменения
    #[arg(short = 't', long = "timesort", help_heading = "Sorting")]
    pub time_sort: bool,

    /// Sort by change time / По времени метаданных
    #[arg(short = 'c', long = "ctime", help_heading = "Filtering")]
    pub ctime_sort: bool,

    /// Leave files unsorted / Без сортировки
    #[arg(short = 'U', long = "unsorted", help_heading = "Filtering")]
    pub unsorted: bool,

    /// Reverse sort order / Обратный порядок
    #[arg(short = 'r', long = "reverse", help_heading = "Filtering")]
    pub reverse: bool,

    /// List directories before files / Каталоги первыми
    #[arg(long = "dirsfirst", help_heading = "Filtering")]
    pub dirs_first: bool,

    /// List files before directories / Файлы первыми
    #[arg(long = "filesfirst", help_heading = "Filtering")]
    pub files_first: bool,

    /// Sort by specific criteria / Критерий сортировки
    #[arg(long = "sort", value_name = "TYPE", help_heading = "Filtering")]
    pub sort: Option<SortType>,

    // === Output Format ===
    /// Don't print indentation lines / Без отступов
    #[arg(short = 'i', long = "noindent", help_heading = "Display")]
    pub no_indent: bool,

    /// Use ANSI line graphics
    #[arg(short = 'A', long = "ansi", help_heading = "Display")]
    pub ansi: bool,

    /// Use CP437 line graphics
    #[arg(short = 'S', long = "cp437", help_heading = "Display")]
    pub cp437: bool,

    /// Turn colorization off / Без цвета
    #[arg(short = 'n', long = "nocolor", help_heading = "Display")]
    pub no_color: bool,

    /// Turn colorization on always / Цвет всегда
    #[arg(short = 'C', long = "color-always", help_heading = "Display")]
    pub color_always: bool,

    /// When to use color / Режим цвета
    #[arg(
        long = "color",
        value_name = "WHEN",
        default_value = "auto",
        help_heading = "Display"
    )]
    pub color: ColorWhen,

    // === File Info ===
    /// Print size in bytes / Размер в байтах
    #[arg(short = 's', long = "size", help_heading = "File Information")]
    pub size: bool,

    /// Print human readable sizes / Человекочитаемый размер
    #[arg(short = 'h', long = "human", help_heading = "File Information")]
    pub human_readable: bool,

    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help, help_heading = "File Information")]
    pub help: Option<bool>,

    /// Use SI units (powers of 1000) / Единицы СИ
    #[arg(long = "si", help_heading = "File Information")]
    pub si_units: bool,

    /// Print modification date / Дата изменения
    #[arg(short = 'D', long = "date", help_heading = "File Information")]
    pub date: bool,

    /// Time format string / Формат даты
    #[arg(
        long = "timefmt",
        value_name = "FMT",
        default_value = "%Y-%m-%d %H:%M",
        help_heading = "File Information"
    )]
    pub time_fmt: String,

    /// Print permissions / Права доступа
    #[arg(short = 'p', long = "perm", help_heading = "File Information")]
    pub permissions: bool,

    /// Print file owner / Владелец
    #[arg(short = 'u', long = "uid", help_heading = "File Information")]
    pub uid: bool,

    /// Print file group / Группа
    #[arg(short = 'g', long = "gid", help_heading = "File Information")]
    pub gid: bool,

    /// Print inode number
    #[arg(long = "inodes", help_heading = "File Information")]
    pub inodes: bool,

    /// Print device number
    #[arg(long = "device", help_heading = "File Information")]
    pub device: bool,

    /// Append file type indicator / Индикатор типа
    #[arg(short = 'F', long = "classify", help_heading = "File Information")]
    pub classify: bool,

    /// Replace non-printable chars with ?
    #[arg(short = 'q', long = "safe", help_heading = "File Information")]
    pub safe_print: bool,

    /// Print non-printable chars as-is
    #[arg(short = 'N', long = "literal", help_heading = "File Information")]
    pub literal: bool,

    /// Output character encoding / Кодировка
    #[arg(
        long = "charset",
        value_name = "CHARSET",
        help_heading = "File Information"
    )]
    pub charset: Option<String>,

    // === Export Options ===
    /// Output to file / Вывод в файл
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        help_heading = "Export"
    )]
    pub output_file: Option<PathBuf>,

    /// HTML output with base URL
    #[arg(
        short = 'H',
        long = "html",
        value_name = "URL",
        help_heading = "Export"
    )]
    pub html_base: Option<String>,

    /// HTML page title
    #[arg(
        short = 'T',
        long = "title",
        value_name = "TITLE",
        help_heading = "Export"
    )]
    pub html_title: Option<String>,

    /// Turn off hyperlinks in HTML
    #[arg(long = "nolinks", help_heading = "Export")]
    pub no_links: bool,

    /// Use custom HTML intro file
    #[arg(long = "hintro", value_name = "FILE", help_heading = "Export")]
    pub html_intro: Option<PathBuf>,

    /// Use custom HTML outro file
    #[arg(long = "houtro", value_name = "FILE", help_heading = "Export")]
    pub html_outro: Option<PathBuf>,

    /// XML output
    #[arg(short = 'X', long = "xml", help_heading = "Export")]
    pub xml: bool,

    /// JSON output
    #[arg(short = 'J', long = "json", help_heading = "Export")]
    pub json: bool,

    /// Pretty-print JSON output (fully formatted) / Форматированный JSON
    #[arg(long = "json-pretty", help_heading = "Export")]
    pub json_pretty: bool,

    // === Icons ===
    /// Show icons (default: auto) / Показывать иконки
    #[arg(
        long = "icons",
        value_enum,
        default_value = "auto",
        default_missing_value = "always",
        help_heading = "Icons"
    )]
    pub icons: IconsWhen,

    /// Disable icons / Без иконок
    #[arg(long = "no-icons", help_heading = "Icons")]
    pub no_icons: bool,

    /// Icon style / Стиль иконок
    #[arg(
        long = "icon-style",
        value_name = "STYLE",
        default_value = "nerd",
        help_heading = "Icons"
    )]
    pub icon_style: IconStyle,

    // === Windows-specific ===
    /// Show NTFS Alternate Data Streams
    #[arg(long = "show-streams", help_heading = "Windows")]
    pub show_streams: bool,

    /// Show junction points with targets
    #[arg(long = "show-junctions", help_heading = "Windows")]
    pub show_junctions: bool,

    /// Hide system files even with -a
    #[arg(long = "hide-system", help_heading = "Windows")]
    pub hide_system: bool,

    /// Permission format (posix/windows)
    #[arg(
        long = "permissions",
        value_name = "MODE",
        default_value = "windows",
        help_heading = "Windows"
    )]
    pub perm_mode: PermMode,

    /// Force long path prefix (\\?\)
    #[arg(long = "long-paths", help_heading = "Windows")]
    pub long_paths: bool,

    // === Language ===
    /// Interface language (en/ru) / Язык интерфейса
    #[arg(
        long = "lang",
        value_name = "LANG",
        env = "TREE_LANG",
        help_heading = "Localization"
    )]
    pub lang: Option<String>,

    // === Parallel execution ===
    /// Enable parallel directory traversal (faster for large directories)
    #[arg(long = "parallel", help_heading = "Performance")]
    pub parallel: bool,

    /// Streaming mode: render output during traversal without building full tree
    #[arg(long = "streaming", help_heading = "Performance")]
    pub streaming: bool,

    /// Number of worker threads for parallel mode (default: CPU cores)
    #[arg(
        long = "threads",
        value_name = "N",
        value_parser = clap::value_parser!(u64).range(1..=256),
        help_heading = "Performance"
    )]
    pub threads: Option<u64>,

    /// Maximum concurrent directory reads in parallel mode
    #[arg(
        long = "queue-cap",
        value_name = "N",
        default_value = "64",
        value_parser = clap::value_parser!(u64).range(1..=65536),
        help_heading = "Performance"
    )]
    pub queue_cap: Option<u64>,

    /// Maximum total entries to display (0 = unlimited)
    #[arg(
        long = "max-entries",
        value_name = "N",
        help_heading = "Listing Options"
    )]
    pub max_entries: Option<usize>,

    /// Generate shell completions and exit
    #[arg(long = "completions", value_name = "SHELL", hide = true, value_enum)]
    pub completions: Option<clap_complete::Shell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorWhen {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum IconsWhen {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum PermMode {
    Posix,
    #[default]
    Windows,
}
