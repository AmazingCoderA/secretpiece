//! Localization for the `SecretPiece` CLI: Russian and English.
//!
//! The language is chosen in this order:
//!
//! 1. `--lang ru` / `--lang en` on the command line;
//! 2. `SP_LANG`;
//! 3. `LC_ALL`, `LC_MESSAGES`, `LANG` (locale, e.g. `ru_RU.UTF-8`);
//! 4. English as the fallback.

#![forbid(unsafe_code)]

/// Supported interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    /// English.
    #[default]
    En,
    /// Russian.
    Ru,
}

impl Lang {
    /// All supported languages, in the order used in help.
    pub const ALL: [Self; 2] = [Self::Ru, Self::En];

    /// Parses a language code or locale (`ru`, `RU`, `ru_RU.UTF-8`, `en-US`).
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        if code.is_empty() {
            return None;
        }
        let primary = code
            .split(['.', '@', '_', '-'])
            .next()
            .unwrap_or(code)
            .trim();
        if primary.eq_ignore_ascii_case("ru") {
            Some(Self::Ru)
        } else if primary.eq_ignore_ascii_case("en") {
            Some(Self::En)
        } else {
            None
        }
    }

    /// Detects the language from the environment.
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_from(|name| std::env::var(name).ok())
    }

    /// Detects the language using a custom environment lookup (for tests).
    #[must_use]
    pub fn detect_from(lookup: impl Fn(&str) -> Option<String>) -> Self {
        for name in ["SP_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Some(value) = lookup(name) {
                if let Some(lang) = Self::from_code(&value) {
                    return lang;
                }
            }
        }
        Self::default()
    }

    /// The short language code (`ru` or `en`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::En => "en",
        }
    }

    /// The full help screen.
    #[must_use]
    pub fn help(self) -> String {
        match self {
            Self::Ru => format!(
                "{name} — портативное правдоподобно-отрицаемое зашифрованное хранилище.\n\n\
                 Статус: ранний каркас (этап спецификации). Криптография ещё не реализована.\n\n\
                 Использование:\n  sp [--lang ru|en] <команда> [параметры]\n\n\
                 Планируемые команды:\n  \
                 init      создать хранилище\n  \
                 open      открыть хранилище\n  \
                 shell     интерактивная сессия\n  \
                 ls/put/get/cp/mv/rm   операции с файлами\n  \
                 snapshot  версии и снапшоты\n  \
                 verify    проверка целостности\n  \
                 doctor    восстановление после сбоя\n  \
                 df        свободное место и прогноз\n  \
                 wipe-free затирание свободного места\n  \
                 destroy   crypto-erase контейнера\n  \
                 version   версия программы\n\n\
                 Язык: --lang ru|en, переменные SP_LANG, LC_ALL, LC_MESSAGES, LANG.\n\
                 Если закончилось место: очистите корзину (`sp trash empty`), удалите старые\n\
                 снапшоты (`sp snapshot prune`), проверьте `sp df`, освободите место на\n\
                 носителе или создайте контейнер большего размера.\n\n\
                 Документация: https://github.com/AmazingCoderA/secretpiece#readme",
                name = sp_core::PRODUCT_NAME
            ),
            Self::En => format!(
                "{name} — portable, deniable encrypted vault.\n\n\
                 Status: early skeleton (specification stage). Cryptography is not implemented yet.\n\n\
                 Usage:\n  sp [--lang ru|en] <command> [options]\n\n\
                 Planned commands:\n  \
                 init      create a vault\n  \
                 open      unlock a vault\n  \
                 shell     interactive session\n  \
                 ls/put/get/cp/mv/rm   file operations\n  \
                 snapshot  versions and snapshots\n  \
                 verify    integrity check\n  \
                 doctor    repair after a crash\n  \
                 df        free space and forecast\n  \
                 wipe-free free-space wipe\n  \
                 destroy   crypto-erase the container\n  \
                 version   print version\n\n\
                 Language: --lang ru|en, environment SP_LANG, LC_ALL, LC_MESSAGES, LANG.\n\
                 Out of space: empty the trash (`sp trash empty`), prune old snapshots\n\
                 (`sp snapshot prune`), check `sp df`, free space on the drive, or create\n\
                 a larger container.\n\n\
                 Documentation: https://github.com/AmazingCoderA/secretpiece#readme",
                name = sp_core::PRODUCT_NAME
            ),
        }
    }

    /// Version line. `format` is the container format version.
    #[must_use]
    pub fn version(self, version: &str, format: u32) -> String {
        match self {
            Self::Ru => format!(
                "{} {} (формат SPV{}, каркас)",
                sp_core::PRODUCT_NAME,
                version,
                format
            ),
            Self::En => format!(
                "{} {} (format SPV{}, skeleton)",
                sp_core::PRODUCT_NAME,
                version,
                format
            ),
        }
    }

    /// Localized text for a failure that happened before any command ran.
    #[must_use]
    pub fn error(self, kind: ErrorText) -> String {
        match (self, kind) {
            (Self::Ru, ErrorText::UnknownCommand(cmd)) => {
                format!("ошибка: неизвестная команда `{cmd}`")
            }
            (Self::En, ErrorText::UnknownCommand(cmd)) => {
                format!("error: unknown command `{cmd}`")
            }
            (Self::Ru, ErrorText::UnknownLanguage(code)) => {
                format!("ошибка: неподдерживаемый язык `{code}` (доступны: ru, en)")
            }
            (Self::En, ErrorText::UnknownLanguage(code)) => {
                format!("error: unsupported language `{code}` (available: ru, en)")
            }
            (Self::Ru, ErrorText::MissingLanguage) => {
                "ошибка: после `--lang` нужен код языка (ru или en)".to_owned()
            }
            (Self::En, ErrorText::MissingLanguage) => {
                "error: `--lang` needs a language code (ru or en)".to_owned()
            }
            (Self::Ru, ErrorText::Hint) => "подсказка: запустите `--help`".to_owned(),
            (Self::En, ErrorText::Hint) => "hint: run `--help`".to_owned(),
        }
    }
}

/// Pre-command messages that need localization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorText<'a> {
    /// Unknown subcommand.
    UnknownCommand(&'a str),
    /// Unsupported `--lang` value.
    UnknownLanguage(&'a str),
    /// `--lang` without a value.
    MissingLanguage,
    /// Reminder to run `--help`.
    Hint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_language_codes_and_locales() {
        assert_eq!(Lang::from_code("ru"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("RU_ru.UTF-8"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("en-US"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::from_code(""), None);
        assert_eq!(Lang::from_code("   "), None);
    }

    #[test]
    fn detects_in_priority_order() {
        let lang = Lang::detect_from(|name| match name {
            "SP_LANG" => Some("ru_RU.UTF-8".to_owned()),
            "LANG" => Some("en_US.UTF-8".to_owned()),
            _ => None,
        });
        assert_eq!(lang, Lang::Ru);

        let lang = Lang::detect_from(|name| match name {
            "LC_ALL" => Some("C".to_owned()),
            "LANG" => Some("ru_RU.UTF-8".to_owned()),
            _ => None,
        });
        assert_eq!(lang, Lang::Ru, "unknown locales are skipped");

        assert_eq!(Lang::detect_from(|_| None), Lang::En);
    }

    #[test]
    fn help_and_errors_are_localized() {
        let ru = Lang::Ru.help();
        assert!(ru.contains("Использование"));
        assert!(ru.contains("закончилось место"));
        assert!(ru.contains("--lang"));

        let en = Lang::En.help();
        assert!(en.contains("Usage"));
        assert!(en.contains("Out of space"));
        assert!(!en.contains("Использование"));

        assert!(Lang::Ru.error(ErrorText::Hint).contains("подсказка"));
        assert!(
            Lang::En
                .error(ErrorText::UnknownCommand("nope"))
                .contains("unknown command")
        );
    }

    #[test]
    fn version_line_mentions_format() {
        assert!(Lang::Ru.version("1.1.0", 1).contains("SPV1"));
        assert!(Lang::En.version("1.1.0", 1).contains("SPV1"));
    }
}
