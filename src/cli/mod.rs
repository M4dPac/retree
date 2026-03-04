pub mod args;
pub mod early_lang;
pub mod localized;

// Re-export all public items for backward compatibility
pub use args::{parse_args, Args, ColorWhen, IconStyle, IconsWhen, PermMode, SortType};
pub use early_lang::{detect_language_early, has_help_flag};
pub use localized::build_localized_command;

