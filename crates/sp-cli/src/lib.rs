//! Shared CLI entry point for the `sp` and `secretpiece` binaries.
//!
//! Command parsing will be implemented with `clap` in milestone v1.3. For now
//! this is a minimal, dependency-free skeleton so the workspace builds and CI
//! stays green, with Russian/English localization already wired in.

#![forbid(unsafe_code)]

pub mod i18n;

pub use i18n::Lang;

/// Runs the CLI and returns a process exit status.
///
/// `--lang ru|en` may appear before or after the command and overrides the
/// language detected from `SP_LANG`, `LC_ALL`, `LC_MESSAGES`, or `LANG`.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    let mut lang = Lang::detect();
    let mut command: Option<&str> = None;
    let mut index = 0;

    while index < args.len() {
        let arg = args[index].as_str();
        index += 1;

        // Skip the program name.
        if index == 1 {
            continue;
        }

        match arg {
            "--lang" => {
                let Some(value) = args.get(index) else {
                    eprintln!("{}", lang.error(i18n::ErrorText::MissingLanguage));
                    eprintln!("{}", lang.error(i18n::ErrorText::Hint));
                    return sp_core::ErrorCode::InvalidArguments.exit_status();
                };
                index += 1;
                if let Err(status) = apply_lang(value, &mut lang) {
                    return status;
                }
            }
            _ if arg.starts_with("--lang=") => {
                let value = arg.trim_start_matches("--lang=");
                if let Err(status) = apply_lang(value, &mut lang) {
                    return status;
                }
            }
            other => {
                // Planned commands take options that this skeleton does not parse
                // yet; only the first positional argument is the command.
                if command.is_none() {
                    command = Some(other);
                }
            }
        }
    }

    match command {
        None | Some("-h" | "--help" | "help") => {
            println!("{}", lang.help());
            0
        }
        Some("-V" | "--version" | "version") => {
            println!(
                "{}",
                lang.version(env!("CARGO_PKG_VERSION"), sp_core::FORMAT_VERSION)
            );
            0
        }
        Some(other) => {
            eprintln!("{}", lang.error(i18n::ErrorText::UnknownCommand(other)));
            eprintln!("{}", lang.error(i18n::ErrorText::Hint));
            sp_core::ErrorCode::InvalidArguments.exit_status()
        }
    }
}

/// Applies a `--lang` value, printing a localized error when it is unsupported.
fn apply_lang(value: &str, lang: &mut Lang) -> Result<(), i32> {
    if let Some(chosen) = Lang::from_code(value) {
        *lang = chosen;
        Ok(())
    } else {
        eprintln!("{}", lang.error(i18n::ErrorText::UnknownLanguage(value)));
        eprintln!("{}", lang.error(i18n::ErrorText::Hint));
        Err(sp_core::ErrorCode::InvalidArguments.exit_status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn no_args_prints_help() {
        assert_eq!(run(&args(&["sp"])), 0);
    }

    #[test]
    fn unknown_command_uses_invalid_arguments_code() {
        assert_eq!(
            run(&args(&["sp", "definitely-not-a-command"])),
            sp_core::ErrorCode::InvalidArguments.exit_status()
        );
    }

    #[test]
    fn invalid_language_is_rejected() {
        assert_eq!(
            run(&args(&["sp", "--lang", "de"])),
            sp_core::ErrorCode::InvalidArguments.exit_status()
        );
        assert_eq!(
            run(&args(&["sp", "--lang=", "version"])),
            sp_core::ErrorCode::InvalidArguments.exit_status()
        );
    }

    #[test]
    fn language_flag_can_follow_the_command() {
        assert_eq!(run(&args(&["sp", "--version", "--lang", "ru"])), 0);
    }
}
