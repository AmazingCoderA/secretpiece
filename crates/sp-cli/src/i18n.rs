//! Localization for the `SecretPiece` CLI: Russian and English.
//!
//! The language is chosen in this order:
//!
//! 1. `--lang ru` / `--lang en` on the command line;
//! 2. `SP_LANG`;
//! 3. `LC_ALL`, `LC_MESSAGES`, `LANG` (locale, e.g. `ru_RU.UTF-8`);
//! 4. English as the fallback.

#![forbid(unsafe_code)]

use sp_core::{ErrorCode, SpError};
use sp_format::EntryKind;
use sp_vault::VaultInfo;

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
                "{name} — портативное зашифрованное хранилище в одном файле .spv.\n\n\
                 Использование:\n  sp-cli [--lang ru|en] <команда> <хранилище> [параметры]\n\n\
                 Команды:\n  \
                 init <файл>              создать хранилище\n  \
                 ls <файл> [путь]         список файлов и папок\n  \
                 put <файл> <локальный> [путь]   добавить файл\n  \
                 get <файл> <путь> [локальный]   извлечь файл\n  \
                 mkdir <файл> <путь>      создать папку\n  \
                 rm <файл> <путь>...      удалить файл или папку\n  \
                 df <файл>                размеры и заполненность\n  \
                 verify <файл>            проверить целостность\n  \
                 doctor <файл>            диагностика после сбоя\n  \
                 version                  версия программы\n\n\
                 Параметры:\n  \
                 --profile light|default|paranoid   стойкость Argon2id при init\n  \
                 --password-stdin          прочитать пароль из stdin\n  \
                 --lang ru|en              язык интерфейса\n\n\
                 Пароль также можно задать переменной SP_PASSWORD (для скриптов).\n\
                 Документация: https://github.com/AmazingCoderA/secretpiece#readme",
                name = sp_core::PRODUCT_NAME
            ),
            Self::En => format!(
                "{name} — a portable encrypted vault in a single .spv file.\n\n\
                 Usage:\n  sp-cli [--lang ru|en] <command> <vault> [options]\n\n\
                 Commands:\n  \
                 init <file>              create a vault\n  \
                 ls <file> [path]         list files and folders\n  \
                 put <file> <local> [path]       add a file\n  \
                 get <file> <path> [local]       extract a file\n  \
                 mkdir <file> <path>      create a folder\n  \
                 rm <file> <path>...      delete a file or folder\n  \
                 df <file>                sizes and fill level\n  \
                 verify <file>            check integrity\n  \
                 doctor <file>            diagnose after a crash\n  \
                 version                  print version\n\n\
                 Options:\n  \
                 --profile light|default|paranoid   Argon2id strength for init\n  \
                 --password-stdin          read the password from stdin\n  \
                 --lang ru|en              interface language\n\n\
                 The password can also be supplied via the SP_PASSWORD variable.\n\
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
                "{} {} (формат SPV{})",
                sp_core::PRODUCT_NAME,
                version,
                format
            ),
            Self::En => format!(
                "{} {} (format SPV{})",
                sp_core::PRODUCT_NAME,
                version,
                format
            ),
        }
    }

    /// A fixed message.
    #[must_use]
    pub const fn text(self, msg: Msg) -> &'static str {
        match (self, msg) {
            (Self::Ru, Msg::Hint) => "подсказка: запустите `sp-cli --help`",
            (Self::En, Msg::Hint) => "hint: run `sp-cli --help`",
            (Self::Ru, Msg::MissingLanguage) => {
                "ошибка: после `--lang` нужен код языка (ru или en)"
            }
            (Self::En, Msg::MissingLanguage) => "error: `--lang` needs a language code (ru or en)",
            (Self::Ru, Msg::EmptyPassword) => "ошибка: пароль не может быть пустым",
            (Self::En, Msg::EmptyPassword) => "error: the password must not be empty",
            (Self::Ru, Msg::PasswordMismatch) => "ошибка: пароли не совпадают",
            (Self::En, Msg::PasswordMismatch) => "error: the passwords do not match",
            (Self::Ru, Msg::PasswordPrompt) => "Пароль: ",
            (Self::En, Msg::PasswordPrompt) => "Password: ",
            (Self::Ru, Msg::PasswordConfirm) => "Повторите пароль: ",
            (Self::En, Msg::PasswordConfirm) => "Repeat password: ",
            (Self::Ru, Msg::PasswordStdin) => "ошибка: не удалось прочитать пароль из stdin",
            (Self::En, Msg::PasswordStdin) => "error: cannot read the password from stdin",
            (Self::Ru, Msg::EmptyListing) => "хранилище пусто",
            (Self::En, Msg::EmptyListing) => "the vault is empty",
            (Self::Ru, Msg::Created) => "хранилище создано",
            (Self::En, Msg::Created) => "vault created",
            (Self::Ru, Msg::VerifyOk) => "целостность в порядке",
            (Self::En, Msg::VerifyOk) => "integrity check passed",
            (Self::Ru, Msg::DoctorOk) => "диагностика: проблем не найдено",
            (Self::En, Msg::DoctorOk) => "doctor: no problems found",
        }
    }

    /// A parameterized message.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn line(self, msg: Line<'_>) -> String {
        match (self, msg) {
            (Self::Ru, Line::UnknownCommand(cmd)) => {
                format!("ошибка: неизвестная команда `{cmd}`")
            }
            (Self::En, Line::UnknownCommand(cmd)) => {
                format!("error: unknown command `{cmd}`")
            }
            (Self::Ru, Line::UnknownLanguage(code)) => {
                format!("ошибка: неподдерживаемый язык `{code}` (доступны: ru, en)")
            }
            (Self::En, Line::UnknownLanguage(code)) => {
                format!("error: unsupported language `{code}` (available: ru, en)")
            }
            (Self::Ru, Line::UnknownOption(flag)) => {
                format!("ошибка: неизвестный параметр `{flag}`")
            }
            (Self::En, Line::UnknownOption(flag)) => {
                format!("error: unknown option `{flag}`")
            }
            (Self::Ru, Line::MissingArgument(what)) => {
                format!("ошибка: не хватает аргумента <{what}>")
            }
            (Self::En, Line::MissingArgument(what)) => {
                format!("error: missing argument <{what}>")
            }
            (Self::Ru, Line::VaultCreated(path)) => {
                format!("{path}: {}", Self::Ru.text(Msg::Created))
            }
            (Self::En, Line::VaultCreated(path)) => {
                format!("{path}: {}", Self::En.text(Msg::Created))
            }
            (Self::Ru | Self::En, Line::EntryLine { kind, size, path }) => {
                let mark = match kind {
                    EntryKind::Dir => "d",
                    EntryKind::File => "f",
                };
                format!("{mark}  {size:>8}  {path}")
            }
            (Self::Ru, Line::PutOk { path, size }) => {
                format!("добавлено: {path} ({size})")
            }
            (Self::En, Line::PutOk { path, size }) => {
                format!("added: {path} ({size})")
            }
            (Self::Ru, Line::GetOk { path, size }) => {
                format!("извлечено: {path} ({size})")
            }
            (Self::En, Line::GetOk { path, size }) => {
                format!("extracted: {path} ({size})")
            }
            (Self::Ru, Line::MkdirOk(path)) => format!("папка создана: {path}"),
            (Self::En, Line::MkdirOk(path)) => format!("folder created: {path}"),
            (Self::Ru, Line::RemovedOk(path)) => format!("удалено: {path}"),
            (Self::En, Line::RemovedOk(path)) => format!("removed: {path}"),
            (Self::Ru, Line::VerifyOk { files, bytes }) => {
                format!(
                    "{}: файлов {files}, прочитано {bytes}",
                    Self::Ru.text(Msg::VerifyOk)
                )
            }
            (Self::En, Line::VerifyOk { files, bytes }) => {
                format!(
                    "{}: {files} files, {bytes} read",
                    Self::En.text(Msg::VerifyOk)
                )
            }
            (
                Self::Ru,
                Line::DoctorOk {
                    files,
                    bytes,
                    generation,
                },
            ) => {
                format!(
                    "{}: поколение {generation}, файлов {files}, прочитано {bytes}",
                    Self::Ru.text(Msg::DoctorOk)
                )
            }
            (
                Self::En,
                Line::DoctorOk {
                    files,
                    bytes,
                    generation,
                },
            ) => {
                format!(
                    "{}: generation {generation}, {files} files, {bytes} read",
                    Self::En.text(Msg::DoctorOk)
                )
            }
            (
                Self::Ru,
                Line::Info {
                    info,
                    human_file,
                    human_payload,
                    fill,
                },
            ) => {
                format!(
                    "файл:        {human_file} ({size} байт)\n\
                     полезные:    {human_payload} в {files} файл(ах)\n\
                     папок:       {dirs}\n\
                     заполнение:  {fill}%\n\
                     поколение:   {generation}\n\
                     Argon2id:    m={m_cost} KiB, t={t_cost}, p={p_cost}\n\
                     индекс:      {index_len} байт @ {index_offset}",
                    size = info.file_size,
                    files = info.files,
                    dirs = info.dirs,
                    generation = info.generation,
                    m_cost = info.kdf.m_cost,
                    t_cost = info.kdf.t_cost,
                    p_cost = info.kdf.p_cost,
                    index_len = info.index_len,
                    index_offset = info.index_offset,
                )
            }
            (
                Self::En,
                Line::Info {
                    info,
                    human_file,
                    human_payload,
                    fill,
                },
            ) => {
                format!(
                    "file:        {human_file} ({size} bytes)\n\
                     payload:     {human_payload} in {files} file(s)\n\
                     folders:     {dirs}\n\
                     fill level:  {fill}%\n\
                     generation:  {generation}\n\
                     Argon2id:    m={m_cost} KiB, t={t_cost}, p={p_cost}\n\
                     index:       {index_len} bytes @ {index_offset}",
                    size = info.file_size,
                    files = info.files,
                    dirs = info.dirs,
                    generation = info.generation,
                    m_cost = info.kdf.m_cost,
                    t_cost = info.kdf.t_cost,
                    p_cost = info.kdf.p_cost,
                    index_len = info.index_len,
                    index_offset = info.index_offset,
                )
            }
            (Self::Ru, Line::Error(error)) => format!("ошибка: {}", describe_ru(error)),
            (Self::En, Line::Error(error)) => format!("error: {}", describe_en(error)),
        }
    }
}

/// Fixed, non-parameterized messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    /// Reminder to run `--help`.
    Hint,
    /// `--lang` without a value.
    MissingLanguage,
    /// The vault password is empty.
    EmptyPassword,
    /// The two entered passwords differ.
    PasswordMismatch,
    /// Interactive password prompt.
    PasswordPrompt,
    /// Interactive password confirmation prompt.
    PasswordConfirm,
    /// Reading the password from stdin failed.
    PasswordStdin,
    /// The vault has no entries.
    EmptyListing,
    /// A new vault was created.
    Created,
    /// Integrity check passed.
    VerifyOk,
    /// Diagnostic pass found no problems.
    DoctorOk,
}

/// Parameterized messages.
#[derive(Debug, Clone, Copy)]
pub enum Line<'a> {
    /// Unknown subcommand.
    UnknownCommand(&'a str),
    /// Unsupported `--lang` value.
    UnknownLanguage(&'a str),
    /// Unsupported option.
    UnknownOption(&'a str),
    /// Required positional argument.
    MissingArgument(&'a str),
    /// Vault created at a path.
    VaultCreated(&'a str),
    /// One line of `ls`.
    EntryLine {
        /// Entry kind.
        kind: EntryKind,
        /// Human-readable size.
        size: &'a str,
        /// Vault path.
        path: &'a str,
    },
    /// A file was stored.
    PutOk {
        /// Vault path.
        path: &'a str,
        /// Human-readable size.
        size: &'a str,
    },
    /// A file was extracted.
    GetOk {
        /// Local path.
        path: &'a str,
        /// Human-readable size.
        size: &'a str,
    },
    /// A folder was created.
    MkdirOk(&'a str),
    /// An entry was removed.
    RemovedOk(&'a str),
    /// Integrity report.
    VerifyOk {
        /// Files checked.
        files: u64,
        /// Bytes authenticated.
        bytes: &'a str,
    },
    /// Diagnostic report.
    DoctorOk {
        /// Files checked.
        files: u64,
        /// Bytes authenticated.
        bytes: &'a str,
        /// Commit generation.
        generation: u64,
    },
    /// `df` report.
    Info {
        /// Raw counters.
        info: VaultInfo,
        /// Human-readable container size.
        human_file: &'a str,
        /// Human-readable payload size.
        human_payload: &'a str,
        /// Payload as a percentage of the container, one decimal.
        fill: &'a str,
    },
    /// A vault error translated for the user.
    Error(&'a SpError),
}

/// Human-readable description of an error code.
fn describe_en(error: &SpError) -> String {
    let base = match error.code() {
        ErrorCode::General => "operation failed",
        ErrorCode::InvalidArguments => "invalid arguments",
        ErrorCode::UnlockFailed => "cannot unlock: wrong password or damaged header",
        ErrorCode::Corrupted => "the vault is corrupted",
        ErrorCode::Io => "I/O error",
        ErrorCode::PermissionDenied => "permission denied",
        ErrorCode::NotFound => "not found",
        ErrorCode::Busy => "the entry is busy or already exists",
        ErrorCode::NoSpace => "not enough space on the drive",
        ErrorCode::Cancelled => "cancelled",
        ErrorCode::Unsupported => "unsupported operation",
        ErrorCode::Integrity => "integrity check failed: data is damaged",
    };
    match error.code() {
        ErrorCode::InvalidArguments
        | ErrorCode::Corrupted
        | ErrorCode::Io
        | ErrorCode::Busy
        | ErrorCode::NotFound => format!("{base} ({})", error.message()),
        _ => base.to_owned(),
    }
}

/// Human-readable description of an error code.
fn describe_ru(error: &SpError) -> String {
    let base = match error.code() {
        ErrorCode::General => "операция не выполнена",
        ErrorCode::InvalidArguments => "неверные аргументы",
        ErrorCode::UnlockFailed => "не удалось открыть: неверный пароль или повреждён заголовок",
        ErrorCode::Corrupted => "хранилище повреждено",
        ErrorCode::Io => "ошибка ввода-вывода",
        ErrorCode::PermissionDenied => "доступ запрещён",
        ErrorCode::NotFound => "не найдено",
        ErrorCode::Busy => "запись занята или уже существует",
        ErrorCode::NoSpace => "на диске закончилось место",
        ErrorCode::Cancelled => "отменено",
        ErrorCode::Unsupported => "операция не поддерживается",
        ErrorCode::Integrity => "проверка целостности не пройдена: данные повреждены",
    };
    match error.code() {
        ErrorCode::InvalidArguments
        | ErrorCode::Corrupted
        | ErrorCode::Io
        | ErrorCode::Busy
        | ErrorCode::NotFound => format!("{base} ({})", error.message()),
        _ => base.to_owned(),
    }
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
        assert!(ru.contains("sp-cli"));
        assert!(ru.contains("--password-stdin"));

        let en = Lang::En.help();
        assert!(en.contains("Usage"));
        assert!(en.contains("sp-cli"));
        assert!(!en.contains("Использование"));

        assert!(Lang::Ru.text(Msg::Hint).contains("подсказка"));
        assert!(
            Lang::En
                .line(Line::UnknownCommand("nope"))
                .contains("unknown command")
        );
    }

    #[test]
    fn version_line_mentions_format() {
        assert!(Lang::Ru.version("1.1.0", 1).contains("SPV1"));
        assert!(Lang::En.version("1.1.0", 1).contains("SPV1"));
    }
}
