//! Implementation of the `SecretPiece` command-line interface.
//!
//! The CLI operates on the same `.spv` containers as the GUI: a single file
//! with an authenticated header and an encrypted index. Commands accept the
//! password from `SP_PASSWORD`, from stdin (`--password-stdin`), or from an
//! interactive prompt.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod i18n;

use std::io::Write;
use std::path::Path;

use i18n::{Lang, Line, Msg};
use sp_core::{ErrorCategory, ErrorCode, SpError};
use sp_crypto::KdfParams;
use sp_format::EntryKind;
use sp_vault::Vault;
use zeroize::Zeroizing;

/// Runs the CLI and returns a process exit status.
///
/// `--lang ru|en` may appear before or after the command and overrides the
/// language detected from `SP_LANG`, `LC_ALL`, `LC_MESSAGES`, or `LANG`.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    execute(args, None)
}

/// Parsed command line.
#[allow(clippy::struct_excessive_bools)]
struct Parsed {
    lang: Lang,
    password_stdin: bool,
    force: bool,
    profile: KdfParams,
    positionals: Vec<String>,
    want_help: bool,
    want_version: bool,
}

/// Runs the CLI with an injected password (used by tests).
#[must_use]
pub fn execute(args: &[String], injected: Option<&[u8]>) -> i32 {
    let Parsed {
        lang,
        password_stdin,
        force,
        profile,
        positionals,
        want_help,
        want_version,
    } = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(status) => return status,
    };

    if want_help || (positionals.is_empty() && !want_version) {
        println!("{}", lang.help());
        return 0;
    }
    if want_version {
        println!(
            "{}",
            lang.version(env!("CARGO_PKG_VERSION"), sp_core::FORMAT_VERSION)
        );
        return 0;
    }

    let command = positionals[0].as_str();
    let rest = &positionals[1..];

    let needs_password = !matches!(command, "version" | "help");
    let password = if needs_password {
        match injected {
            Some(secret) => Zeroizing::new(secret.to_vec()),
            None => match acquire_password(lang, password_stdin, command == "init") {
                Ok(secret) => secret,
                Err(error) => return fail(lang, &error),
            },
        }
    } else {
        Zeroizing::new(Vec::new())
    };

    let result = match command {
        "version" => {
            println!(
                "{}",
                lang.version(env!("CARGO_PKG_VERSION"), sp_core::FORMAT_VERSION)
            );
            Ok(())
        }
        "help" => {
            println!("{}", lang.help());
            Ok(())
        }
        "init" => cmd_init(lang, rest, profile, force, &password),
        "ls" | "list" => cmd_ls(lang, rest, &password),
        "put" | "add" => cmd_put(lang, rest, &password),
        "get" | "extract" => cmd_get(lang, rest, &password),
        "mkdir" => cmd_mkdir(lang, rest, &password),
        "rm" | "remove" => cmd_rm(lang, rest, &password),
        "df" | "info" => cmd_df(lang, rest, &password),
        "verify" | "check" => cmd_verify(lang, rest, &password),
        "doctor" => cmd_doctor(lang, rest, &password),
        other => Err(usage(&lang.line(Line::UnknownCommand(other)))),
    };

    match result {
        Ok(()) => 0,
        Err(error) => fail(lang, &error),
    }
}

fn parse_args(args: &[String]) -> Result<Parsed, i32> {
    let mut parsed = Parsed {
        lang: Lang::detect(),
        password_stdin: false,
        force: false,
        profile: KdfParams::DEFAULT,
        positionals: Vec::new(),
        want_help: false,
        want_version: false,
    };

    let mut index = 1;
    while index < args.len() {
        let arg = args[index].as_str();
        index += 1;

        match arg {
            "--lang" => {
                let Some(value) = args.get(index) else {
                    return Err(fail_text(parsed.lang, Msg::MissingLanguage));
                };
                index += 1;
                apply_lang(value, &mut parsed.lang)?;
            }
            _ if arg.starts_with("--lang=") => {
                apply_lang(arg.trim_start_matches("--lang="), &mut parsed.lang)?;
            }
            "--profile" => {
                let Some(value) = args.get(index) else {
                    return Err(fail_arg(parsed.lang, "light|default|paranoid"));
                };
                index += 1;
                let Ok(chosen) = parse_profile(value) else {
                    return Err(fail_line(parsed.lang, Line::UnknownOption(value)));
                };
                parsed.profile = chosen;
            }
            _ if arg.starts_with("--profile=") => {
                let value = arg.trim_start_matches("--profile=");
                let Ok(chosen) = parse_profile(value) else {
                    return Err(fail_line(parsed.lang, Line::UnknownOption(value)));
                };
                parsed.profile = chosen;
            }
            "--password-stdin" => parsed.password_stdin = true,
            "--force" | "-f" => parsed.force = true,
            "-h" | "--help" => parsed.want_help = true,
            "-V" | "--version" => parsed.want_version = true,
            _ if arg.starts_with('-') && arg != "-" => {
                return Err(fail_line(parsed.lang, Line::UnknownOption(arg)));
            }
            other => parsed.positionals.push(other.to_owned()),
        }
    }

    Ok(parsed)
}

fn cmd_init(
    lang: Lang,
    rest: &[String],
    profile: KdfParams,
    force: bool,
    password: &[u8],
) -> Result<(), SpError> {
    let path = requirement(rest, 0, "vault", lang)?;
    let target = Path::new(path);
    if !force && target.exists() {
        return Err(SpError::new(
            ErrorCode::Busy,
            ErrorCategory::Busy,
            format!("{path} already exists (use --force to overwrite)"),
        ));
    }
    let vault = Vault::create(target, password, profile)?;
    let _ = vault.info()?;
    vault.close();
    println!("{}", lang.line(Line::VaultCreated(path)));
    Ok(())
}

fn cmd_ls(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let path = requirement(rest, 0, "vault", lang)?;
    let prefix = rest.get(1).map(String::as_str);
    let vault = Vault::open(Path::new(path), password)?;
    let mut entries = vault.list(prefix);
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    if entries.is_empty() {
        println!("{}", lang.text(Msg::EmptyListing));
    } else {
        for entry in entries {
            let size = match entry.kind {
                EntryKind::Dir => "-".to_owned(),
                EntryKind::File => human_size(entry.size),
            };
            println!(
                "{}",
                lang.line(Line::EntryLine {
                    kind: entry.kind,
                    size: &size,
                    path: &entry.path,
                })
            );
        }
    }
    vault.close();
    Ok(())
}

fn cmd_put(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let vault_path = requirement(rest, 0, "vault", lang)?;
    let local = requirement(rest, 1, "local-file", lang)?;
    let mut vault = Vault::open(Path::new(vault_path), password)?;
    let data = std::fs::read(local).map_err(|error| io_error(&error))?;
    let remote = rest.get(2).cloned().unwrap_or_else(|| {
        Path::new(local).file_name().map_or_else(
            || local.to_owned(),
            |name| name.to_string_lossy().into_owned(),
        )
    });
    vault.put(&remote, &data)?;
    println!(
        "{}",
        lang.line(Line::PutOk {
            path: &remote,
            size: &human_size(data.len() as u64),
        })
    );
    vault.close();
    Ok(())
}

fn cmd_get(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let vault_path = requirement(rest, 0, "vault", lang)?;
    let remote = requirement(rest, 1, "vault-path", lang)?;
    let mut vault = Vault::open(Path::new(vault_path), password)?;
    let data = vault.get(remote)?;
    let local = rest
        .get(2)
        .cloned()
        .unwrap_or_else(|| remote.rsplit('/').next().unwrap_or(remote).to_owned());
    let mut file = std::fs::File::create(&local).map_err(|error| io_error(&error))?;
    file.write_all(&data).map_err(|error| io_error(&error))?;
    file.sync_all().map_err(|error| io_error(&error))?;
    println!(
        "{}",
        lang.line(Line::GetOk {
            path: &local,
            size: &human_size(data.len() as u64),
        })
    );
    vault.close();
    Ok(())
}

fn cmd_mkdir(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let vault_path = requirement(rest, 0, "vault", lang)?;
    let remote = requirement(rest, 1, "vault-path", lang)?;
    let mut vault = Vault::open(Path::new(vault_path), password)?;
    vault.mkdir(remote)?;
    println!("{}", lang.line(Line::MkdirOk(remote)));
    vault.close();
    Ok(())
}

fn cmd_rm(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let vault_path = requirement(rest, 0, "vault", lang)?;
    let targets = &rest[1..];
    if targets.is_empty() {
        return Err(missing_argument("vault-path", lang));
    }
    let mut vault = Vault::open(Path::new(vault_path), password)?;
    for target in targets {
        vault.remove(target)?;
        println!("{}", lang.line(Line::RemovedOk(target)));
    }
    vault.close();
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn cmd_df(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let path = requirement(rest, 0, "vault", lang)?;
    let vault = Vault::open(Path::new(path), password)?;
    let info = vault.info()?;
    let fill = if info.file_size == 0 {
        "0.0".to_owned()
    } else {
        let ratio = info.payload_bytes as f64 / info.file_size as f64 * 100.0;
        format!("{ratio:.1}")
    };
    println!(
        "{}",
        lang.line(Line::Info {
            info,
            human_file: &human_size(info.file_size),
            human_payload: &human_size(info.payload_bytes),
            fill: &fill,
        })
    );
    vault.close();
    Ok(())
}

fn cmd_verify(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let path = requirement(rest, 0, "vault", lang)?;
    let mut vault = Vault::open(Path::new(path), password)?;
    let report = vault.verify()?;
    println!(
        "{}",
        lang.line(Line::VerifyOk {
            files: report.files,
            bytes: &human_size(report.bytes),
        })
    );
    vault.close();
    Ok(())
}

fn cmd_doctor(lang: Lang, rest: &[String], password: &[u8]) -> Result<(), SpError> {
    let path = requirement(rest, 0, "vault", lang)?;
    let mut vault = Vault::open(Path::new(path), password)?;
    let info = vault.info()?;
    let report = vault.verify()?;
    println!(
        "{}",
        lang.line(Line::DoctorOk {
            files: report.files,
            bytes: &human_size(report.bytes),
            generation: info.generation,
        })
    );
    vault.close();
    Ok(())
}

fn requirement<'a>(
    rest: &'a [String],
    index: usize,
    name: &'static str,
    lang: Lang,
) -> Result<&'a str, SpError> {
    rest.get(index)
        .map(String::as_str)
        .ok_or_else(|| missing_argument(name, lang))
}

fn missing_argument(name: &str, lang: Lang) -> SpError {
    usage(&lang.line(Line::MissingArgument(name)))
}

fn usage(message: &str) -> SpError {
    SpError::new(ErrorCode::InvalidArguments, ErrorCategory::Usage, message)
}

fn acquire_password(
    lang: Lang,
    from_stdin: bool,
    confirm: bool,
) -> Result<Zeroizing<Vec<u8>>, SpError> {
    if from_stdin {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .map_err(|error| io_error(&error))?;
        let trimmed = buffer.trim_end_matches(['\r', '\n']).to_owned();
        if trimmed.is_empty() {
            return Err(SpError::new(
                ErrorCode::InvalidArguments,
                ErrorCategory::Usage,
                lang.text(Msg::EmptyPassword),
            ));
        }
        return Ok(Zeroizing::new(trimmed.into_bytes()));
    }
    if let Ok(value) = std::env::var("SP_PASSWORD") {
        if value.is_empty() {
            return Err(SpError::new(
                ErrorCode::InvalidArguments,
                ErrorCategory::Usage,
                lang.text(Msg::EmptyPassword),
            ));
        }
        return Ok(Zeroizing::new(value.into_bytes()));
    }
    let secret = rpassword::prompt_password(lang.text(Msg::PasswordPrompt))
        .map_err(|error| io_error(&error))?;
    if secret.is_empty() {
        return Err(SpError::new(
            ErrorCode::InvalidArguments,
            ErrorCategory::Usage,
            lang.text(Msg::EmptyPassword),
        ));
    }
    if confirm {
        let again = rpassword::prompt_password(lang.text(Msg::PasswordConfirm))
            .map_err(|error| io_error(&error))?;
        if secret != again {
            return Err(SpError::new(
                ErrorCode::Cancelled,
                ErrorCategory::Cancelled,
                lang.text(Msg::PasswordMismatch),
            ));
        }
    }
    Ok(Zeroizing::new(secret.into_bytes()))
}

fn parse_profile(value: &str) -> Result<KdfParams, ()> {
    match value {
        "light" | "fast" => Ok(KdfParams::LIGHT),
        "default" | "balanced" => Ok(KdfParams::DEFAULT),
        "paranoid" | "strong" => Ok(KdfParams::PARANOID),
        _ => Err(()),
    }
}

fn apply_lang(value: &str, lang: &mut Lang) -> Result<(), i32> {
    if let Some(chosen) = Lang::from_code(value) {
        *lang = chosen;
        Ok(())
    } else {
        Err(fail_line(*lang, Line::UnknownLanguage(value)))
    }
}

fn fail(lang: Lang, error: &SpError) -> i32 {
    eprintln!("{}", lang.line(Line::Error(error)));
    error.code().exit_status()
}

fn fail_text(lang: Lang, msg: Msg) -> i32 {
    eprintln!("{}", lang.text(msg));
    ErrorCode::InvalidArguments.exit_status()
}

fn fail_line(lang: Lang, line: Line<'_>) -> i32 {
    eprintln!("{}", lang.line(line));
    ErrorCode::InvalidArguments.exit_status()
}

fn fail_arg(lang: Lang, name: &str) -> i32 {
    fail_line(lang, Line::MissingArgument(name))
}

#[allow(clippy::cast_precision_loss)]
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn io_error(error: &std::io::Error) -> SpError {
    let category = match error.kind() {
        std::io::ErrorKind::PermissionDenied => ErrorCategory::Permission,
        std::io::ErrorKind::NotFound => ErrorCategory::NotFound,
        std::io::ErrorKind::StorageFull => ErrorCategory::NoSpace,
        _ => ErrorCategory::Io,
    };
    let code = match category {
        ErrorCategory::Permission => ErrorCode::PermissionDenied,
        ErrorCategory::NotFound => ErrorCode::NotFound,
        ErrorCategory::NoSpace => ErrorCode::NoSpace,
        _ => ErrorCode::Io,
    };
    SpError::new(code, category, "local file operation failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn no_args_prints_help() {
        assert_eq!(execute(&args(&["sp-cli"]), None), 0);
    }

    #[test]
    fn unknown_command_uses_invalid_arguments_code() {
        assert_eq!(
            execute(&args(&["sp-cli", "definitely-not-a-command"]), Some(b"pw")),
            ErrorCode::InvalidArguments.exit_status()
        );
    }

    #[test]
    fn invalid_language_is_rejected() {
        assert_eq!(
            execute(&args(&["sp-cli", "--lang", "de"]), None),
            ErrorCode::InvalidArguments.exit_status()
        );
    }

    #[test]
    fn version_needs_no_password() {
        assert_eq!(execute(&args(&["sp-cli", "--version"]), None), 0);
        assert_eq!(execute(&args(&["sp-cli", "version"]), None), 0);
    }

    #[test]
    fn sizes_are_human_readable() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1536), "1.5 KiB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MiB");
    }

    #[test]
    fn profiles_are_parsed() {
        assert_eq!(parse_profile("light"), Ok(KdfParams::LIGHT));
        assert_eq!(parse_profile("default"), Ok(KdfParams::DEFAULT));
        assert_eq!(parse_profile("paranoid"), Ok(KdfParams::PARANOID));
        assert_eq!(parse_profile("weak"), Err(()));
    }

    #[test]
    fn full_roundtrip_over_cli() {
        let dir = std::env::temp_dir().join(format!("sp-cli-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let vault = dir.join("vault.spv");
        let vault = vault.to_string_lossy().into_owned();
        let local = dir.join("hello.txt");
        std::fs::write(&local, b"hello secret").unwrap();
        let local = local.to_string_lossy().into_owned();
        let password = Some(b"correct horse".as_slice());

        assert_eq!(
            execute(
                &args(&["sp-cli", "init", &vault, "--profile=light"]),
                password
            ),
            0
        );
        assert_eq!(
            execute(
                &args(&["sp-cli", "put", &vault, &local, "docs/hi.txt"]),
                password
            ),
            0
        );
        assert_eq!(
            execute(&args(&["sp-cli", "mkdir", &vault, "empty"]), password),
            0
        );
        assert_eq!(execute(&args(&["sp-cli", "ls", &vault]), password), 0);
        assert_eq!(execute(&args(&["sp-cli", "df", &vault]), password), 0);
        assert_eq!(execute(&args(&["sp-cli", "verify", &vault]), password), 0);
        assert_eq!(execute(&args(&["sp-cli", "doctor", &vault]), password), 0);
        let out = dir.join("out.txt");
        let out = out.to_string_lossy().into_owned();
        assert_eq!(
            execute(
                &args(&["sp-cli", "get", &vault, "docs/hi.txt", &out]),
                password
            ),
            0
        );
        assert_eq!(std::fs::read(&out).unwrap(), b"hello secret");
        assert_eq!(
            execute(&args(&["sp-cli", "rm", &vault, "docs"]), password),
            0
        );
        assert_eq!(
            execute(
                &args(&["sp-cli", "get", &vault, "docs/hi.txt", &out]),
                password
            ),
            ErrorCode::NotFound.exit_status()
        );
        assert_eq!(
            execute(&args(&["sp-cli", "init", &vault]), password),
            ErrorCode::Busy.exit_status(),
            "existing vaults are not overwritten without --force"
        );
        assert_eq!(
            execute(&args(&["sp-cli", "init", &vault, "--force"]), password),
            0
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_password_is_reported_as_unlock_failure() {
        let dir = std::env::temp_dir().join(format!("sp-cli-wrong-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let vault = dir.join("vault.spv");
        let vault = vault.to_string_lossy().into_owned();
        assert_eq!(
            execute(
                &args(&["sp-cli", "init", &vault, "--profile=light"]),
                Some(b"right".as_slice())
            ),
            0
        );
        assert_eq!(
            execute(&args(&["sp-cli", "ls", &vault]), Some(b"wrong".as_slice())),
            ErrorCode::UnlockFailed.exit_status()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
