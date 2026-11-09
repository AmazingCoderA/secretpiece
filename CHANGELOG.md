# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-11-09

### Added

- Download buttons and a repeated language switch at the bottom of the README.
- Auto-build artifacts on every push to `main` and on pull requests.

### Changed

- License changed to **GPL-3.0-only**.
- README redesigned: bilingual language switch, encryption overview, any-drive support.

### Removed

- The draft technical specification was removed from the repository (kept in git
  history). Design docs (`FORMAT.md`, `CRYPTO.md`) will be published with M1.

## [0.1.0] - 2026-11-09

First public milestone (M0): project skeleton. Cryptography and the container
format are **not** implemented yet — do not use for real data.

### Added

- Bilingual README (Russian / English) with banner and language switcher.
- Cargo workspace skeleton (`sp-core`, `sp-crypto`, `sp-format`, `sp-storage`,
  `sp-fs`, `sp-vault`, `sp-wipe`, `sp-platform`, `sp-cli`, `sp-ffi`,
  `sp-testkit`).
- Minimal CLI skeleton with `sp` and `secretpiece` binaries.
- Shared error codes, categories, and durability profiles in `sp-core`.
- Project policies: `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
- CI workflow: fmt, clippy, rustdoc, MSRV 1.85, tests on Windows/macOS/Linux.
- Release workflow: tag-driven builds for Windows x64, Linux x64 (static musl),
  macOS universal, archives with `SHA256SUMS.txt`, automatic GitHub Release.
- Dependabot and issue templates.

[Unreleased]: https://github.com/AmazingCoderA/secretpiece/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v0.1.1
[0.1.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v0.1.0
