//! `sp` — short alias for the `SecretPiece` CLI.

#![forbid(unsafe_code)]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(sp_cli::run(&args));
}
