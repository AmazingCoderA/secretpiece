//! `sp-cli` — the `SecretPiece` command-line interface.
//!
//! The graphical application is built from the `sp-gui` crate and installs the
//! `sp` / `secretpiece` binaries. This crate only ships `sp-cli`.

#![forbid(unsafe_code)]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(sp_cli::run(&args));
}
