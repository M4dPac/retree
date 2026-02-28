use clap::{Parser, ValueEnum};
use std::path::PathBuf;

// Для справки используем статические английские строки
// Локализованные сообщения используются в runtime

#[derive(Parser, Debug)]
#[command(name = "rtree")]
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
    #[arg(short = 'a', long)]
    pub all: bool,

    /// List directories only / Только каталоги
    #[arg(short = 'd', long = "dirs-only")]
    pub dirs_only: bool,

    /// Follow symbolic links / Следовать по симлинкам
    #[arg(short = 'l', long = "follow")]
    pub follow_symlinks: bool,

    /// Print full path prefix for each file / Полный путь
    #[arg(short = 'f', long = "full-path")]
    pub full_path: bool,

    /// Stay on current filesystem / Не выходить за пределы ФС
    #[arg(short = 'x', long = "one-fs")]
    pub one_fs: bool,

    /// Descend only N levels deep / Глубина обхода
    #[arg(short = 'L', long = "level", value_name = "N")]
    pub max_depth: Option<usize>,

    /// Do not descend dirs with more than N entries / Лимит файлов
    #[arg(long = "filelimit", value_name = "N")]
    pub file_limit: Option<usize>,

    /// Omit final report / Без статистики
    #[arg(long = "noreport")]
    pub no_report: bool,

    // === Filtering Options ===
    /// List only files matching pattern / Фильтр по шаблону
    #[arg(short = 'P', long = "pattern", value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Exclude files matching pattern / Исключить по шаблону
    #[arg(short = 'I', long = "exclude", value_name = "PATTERN", action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Apply patterns to directories as well / Применять к каталогам
    #[arg(long = "matchdirs")]
    pub match_dirs: bool,

    /// Case insensitive pattern matching / Без учёта регистра
    #[arg(long = "ignore-case")]
    pub ignore_case: bool,

    /// Do not print empty directories / Без пустых каталогов
    #[arg(long = "prune")]
    pub prune: bool,

    // === Sorting Options ===
    /// Natural sort (version sort) / Натуральная сортировка
    #[arg(short = 'v', long = "version-sort")]
    pub version_sort: bool,

    /// Sort by modification time / По времени изменения
    #[arg(short = 't', long = "timesort")]
    pub time_sort: bool,

    /// Sort by change time / По времени метаданных
    #[arg(short = 'c', long = "ctime")]
    pub ctime_sort: bool,

    /// Leave files unsorted / Без сортировки
    #[arg(short = 'U', long = "unsorted")]
    pub unsorted: bool,

    /// Reverse sort order / Обратный порядок
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// List directories before files / Каталоги первыми
    #[arg(long = "dirsfirst")]
    pub dirs_first: bool,

    /// List files before directories / Файлы первыми
    #[arg(long = "filesfirst")]
    pub files_first: bool,

    /// Sort by specific criteria / Критерий сортировки
    #[arg(long = "sort", value_name = "TYPE")]
    pub sort: Option<SortType>,

    // === Output Format ===
    /// Don't print indentation lines / Без отступов
    #[arg(short = 'i', long = "noindent")]
    pub no_indent: bool,

    /// Use ANSI line graphics
    #[arg(short = 'A', long = "ansi")]
    pub ansi: bool,

    /// Use CP437 line graphics
    #[arg(short = 'S', long = "cp437")]
    pub cp437: bool,

    /// Turn colorization off / Без цвета
    #[arg(short = 'n', long = "nocolor")]
    pub no_color: bool,

    /// Turn colorization on always / Цвет всегда
    #[arg(short = 'C', long = "color-always")]
    pub color_always: bool,

    /// When to use color / Режим цвета
    #[arg(long = "color", value_name = "WHEN", default_value = "auto")]
    pub color: ColorWhen,

    // === File Info ===
    /// Print size in bytes / Размер в байтах
    #[arg(short = 's', long = "size")]
    pub size: bool,

    /// Print human readable sizes / Человекочитаемый размер
    #[arg(short = 'h', long = "human")]
    pub human_readable: bool,

    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: Option<bool>,

    /// Use SI units (powers of 1000) / Единицы СИ
    #[arg(long = "si")]
    pub si_units: bool,

    /// Print modification date / Дата изменения
    #[arg(short = 'D', long = "date")]
    pub date: bool,

    /// Time format string / Формат даты
    #[arg(long = "timefmt", value_name = "FMT", default_value = "%Y-%m-%d %H:%M")]
    pub time_fmt: String,

    /// Print permissions / Права доступа
    #[arg(short = 'p', long = "perm")]
    pub permissions: bool,

    /// Print file owner / Владелец
    #[arg(short = 'u', long = "uid")]
    pub uid: bool,

    /// Print file group / Группа
    #[arg(short = 'g', long = "gid")]
    pub gid: bool,

    /// Print inode number
    #[arg(long = "inodes")]
    pub inodes: bool,

    /// Print device number
    #[arg(long = "device")]
    pub device: bool,

    /// Append file type indicator / Индикатор типа
    #[arg(short = 'F', long = "classify")]
    pub classify: bool,

    /// Replace non-printable chars with ?
    #[arg(short = 'q', long = "safe")]
    pub safe_print: bool,

    /// Print non-printable chars as-is
    #[arg(short = 'N', long = "literal")]
    pub literal: bool,

    /// Output character encoding / Кодировка
    #[arg(long = "charset", value_name = "CHARSET")]
    pub charset: Option<String>,

    // === Export Options ===
    /// Output to file / Вывод в файл
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// HTML output with base URL
    #[arg(short = 'H', long = "html", value_name = "URL")]
    pub html_base: Option<String>,

    /// HTML page title
    #[arg(short = 'T', long = "title", value_name = "TITLE")]
    pub html_title: Option<String>,

    /// Turn off hyperlinks in HTML
    #[arg(long = "nolinks")]
    pub no_links: bool,

    /// Use custom HTML intro file
    #[arg(long = "hintro", value_name = "FILE")]
    pub html_intro: Option<PathBuf>,

    /// Use custom HTML outro file
    #[arg(long = "houtro", value_name = "FILE")]
    pub html_outro: Option<PathBuf>,

    /// XML output
    #[arg(short = 'X', long = "xml")]
    pub xml: bool,

    /// JSON output
    #[arg(short = 'J', long = "json")]
    pub json: bool,

    // === Icons ===
    /// Show icons (default: auto) / Показывать иконки
    #[arg(
        long = "icons",
        default_value = "auto",
        default_missing_value = "always"
    )]
    pub icons: String,

    /// Disable icons / Без иконок
    #[arg(long = "no-icons")]
    pub no_icons: bool,

    /// Icon style / Стиль иконок
    #[arg(long = "icon-style", value_name = "STYLE", default_value = "nerd")]
    pub icon_style: IconStyle,

    // === Windows-specific ===
    /// Show NTFS Alternate Data Streams
    #[arg(long = "show-streams")]
    pub show_streams: bool,

    /// Show junction points with targets
    #[arg(long = "show-junctions")]
    pub show_junctions: bool,

    /// Hide system files even with -a
    #[arg(long = "hide-system")]
    pub hide_system: bool,

    /// Permission format (posix/windows)
    #[arg(long = "permissions", value_name = "MODE", default_value = "windows")]
    pub perm_mode: PermMode,

    /// Force long path prefix (\\?\)
    #[arg(long = "long-paths")]
    pub long_paths: bool,

    // === Language ===
    /// Interface language (en/ru) / Язык интерфейса
    #[arg(long = "lang", value_name = "LANG", env = "TREE_LANG")]
    pub lang: Option<String>,

    // === Parallel execution ===
    /// Enable parallel directory traversal (faster for large directories)
    #[arg(long = "parallel")]
    pub parallel: bool,

    /// Number of worker threads for parallel mode (default: CPU cores)
    #[arg(long = "threads", value_name = "N")]
    pub threads: Option<usize>,

    /// Internal queue capacity per thread for parallel mode
    #[arg(long = "queue-cap", value_name = "N", default_value = "4096")]
    pub queue_cap: Option<usize>,
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
pub enum IconStyle {
    #[default]
    Nerd,
    Unicode,
    Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum PermMode {
    Posix,
    #[default]
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortType {
    Name,
    Size,
    Mtime,
    Ctime,
    Version,
    None,
}

pub fn parse_args() -> Args {
    Args::parse()
}

impl Args {
    pub fn effective_color(&self) -> ColorWhen {
        if self.no_color {
            ColorWhen::Never
        } else if self.color_always {
            ColorWhen::Always
        } else {
            self.color
        }
    }

    pub fn effective_icons(&self) -> IconsWhen {
        if self.no_icons {
            IconsWhen::Never
        } else {
            match self.icons.to_lowercase().as_str() {
                "always" => IconsWhen::Always,
                "never" => IconsWhen::Never,
                _ => IconsWhen::Auto,
            }
        }
    }
}
