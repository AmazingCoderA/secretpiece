//! Shared CLI entry point for the `sp` and `secretpiece` binaries.
//!
//! Command parsing will be implemented with `clap` in milestone M2. For now this
//! is a minimal, dependency-free skeleton so the workspace builds and CI is green.

#![forbid(unsafe_code)]

/// Runs the CLI and returns a process exit status.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    match args.get(1).map(String::as_str) {
        None | Some("-h" | "--help" | "help") => {
            print_help();
            0
        }
        Some("-V" | "--version" | "version") => {
            println!(
                "{} {} (format SPV1, skeleton)",
                sp_core::PRODUCT_NAME,
                env!("CARGO_PKG_VERSION")
            );
            0
        }
        Some(other) => {
            eprintln!("error: unknown command `{other}`");
            eprintln!("run `--help` for the list of commands");
            2
        }
    }
}

fn print_help() {
    println!(
        "{name} — portable, deniable encrypted vault.\n\n\
         Status: early skeleton (specification stage).\n\n\
         Usage:\n  sp <command> [options]\n\n\
         Commands (planned):\n  \
         init      create a vault\n  \
         open      unlock a vault\n  \
         shell     interactive session\n  \
         ls/put/get/cp/mv/rm   file operations\n  \
         snapshot  versions and snapshots\n  \
         verify    integrity check\n  \
         doctor    repair after a crash\n  \
         wipe-free free-space wipe\n  \
         destroy   crypto-erase the container\n  \
         version   print version\n\n\
         Full specification: docs/TZ.ru.md",
        name = sp_core::PRODUCT_NAME
    );
}
