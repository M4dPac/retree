mod messages;

pub use messages::*;

use std::env;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

static CURRENT_LANGUAGE: OnceLock<Language> = OnceLock::new();

impl Language {
    /// Detect language from environment
    pub fn detect() -> Self {
        // Check TREE_LANG first
        if let Ok(lang) = env::var("TREE_LANG") {
            return Self::from_code(&lang);
        }

        // Check LANG
        if let Ok(lang) = env::var("LANG") {
            return Self::from_code(&lang);
        }

        // Check LC_ALL
        if let Ok(lang) = env::var("LC_ALL") {
            return Self::from_code(&lang);
        }

        // Check LC_MESSAGES
        if let Ok(lang) = env::var("LC_MESSAGES") {
            return Self::from_code(&lang);
        }

        // Windows-specific: check LANGUAGE or system locale
        #[cfg(windows)]
        {
            if let Some(lang) = detect_windows_language() {
                return lang;
            }
        }

        Language::English
    }

    pub fn from_code(code: &str) -> Self {
        let code = code.to_lowercase();
        if code.starts_with("ru") {
            Language::Russian
        } else {
            Language::English
        }
    }

    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }
}

#[cfg(windows)]
fn detect_windows_language() -> Option<Language> {
    use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

    unsafe {
        let lang_id = GetUserDefaultUILanguage();
        let primary_lang = lang_id & 0x3FF;

        // Russian = 0x19
        if primary_lang == 0x19 {
            return Some(Language::Russian);
        }
    }

    None
}

/// Initialize the language system
pub fn init(lang: Option<&str>) {
    let language = match lang {
        Some(code) => Language::from_code(code),
        None => Language::detect(),
    };

    let _ = CURRENT_LANGUAGE.set(language);
}

/// Get current language
pub fn current() -> Language {
    *CURRENT_LANGUAGE.get_or_init(Language::detect)
}

/// Localization macro for simple messages
#[macro_export]
macro_rules! t {
    ($key:ident) => {
        $crate::i18n::get_message(
            $crate::i18n::current(),
            $crate::i18n::MessageKey::$key
        )
    };
    ($key:ident, $($arg:tt)*) => {
        format!(
            $crate::i18n::get_message(
                $crate::i18n::current(),
                $crate::i18n::MessageKey::$key
            ),
            $($arg)*
        )
    };
}
