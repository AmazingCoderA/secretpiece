# Contributing to SecretPiece

Thanks for wanting to help. This project handles secrets, so the rules are a bit
stricter than usual.

## Before you start

- Read the technical specification: [`docs/TZ.ru.md`](docs/TZ.ru.md).
- For anything touching cryptography, the container format, or the threat model,
  open an issue/ADR **before** writing code.
- By contributing you agree to license your work under **MIT OR Apache-2.0**.

## Development setup

```bash
rustup toolchain install stable
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

MSRV is **1.85** (edition 2024). CI checks stable and MSRV.

## Rules

1. **No custom cryptography.** Use the primitives and crates fixed in the spec.
2. **No secrets in logs, argv, error messages, or temp files.** No `Debug` for
   types that hold keys.
3. **`unsafe` is forbidden** in `sp-core`, `sp-crypto`, `sp-format`, `sp-fs`,
   `sp-vault`. Any `unsafe` elsewhere needs a `// SAFETY:` comment and review.
4. **No new dependency** without justification; must pass `cargo deny` / `cargo audit`.
5. **Tests are mandatory** for parsing, bounds, failure paths, and anything that
   touches data integrity. Fuzz targets for new parsers.
6. **No network calls** in the core. No telemetry.
7. Keep public APIs independent of CLI/GUI.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`, `security:`.

## Pull requests

- One logical change per PR.
- `cargo fmt` and `clippy -D warnings` clean.
- Describe security impact explicitly.
- Reference the relevant TZ section or ADR.
