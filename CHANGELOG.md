# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-11-09

Second public milestone. Still a skeleton: cryptography and the container format
are **not** implemented yet — do not use for real data.

### Added

- Russian/English CLI localization: `--lang ru|en`, `SP_LANG`, and locale
  auto-detection (`LC_ALL`, `LC_MESSAGES`, `LANG`). Localized help, version line
  and errors, including free-space hints.
- Release matrix: Windows x64, Windows ARM64 (experimental), Linux x64 and
  ARM64 static musl, macOS universal, Arch Linux package build, and experimental
  FreeBSD/OpenBSD/NetBSD VM builds.
- `packaging/arch/PKGBUILD` for Arch Linux.
- Architecture diagram (`docs/assets/architecture.svg`).
- Bilingual README with install anchors, download buttons, encryption and format
  tables, platform matrix, out-of-space guidance, and a repeated language switch
  at the bottom.
- Download section with per-platform buttons and checksums.

### Changed

- Version scheme moved to `1.x` tags (`v1.1`, `v1.2`, `v2.0`), without
  micro-patch releases.
- License changed to **GPL-3.0-only**.
- Release workflow now runs on every push to `main`, on pull requests, and on
  `v*` tags; tag pushes publish a GitHub Release with `SHA256SUMS.txt`.

### Removed

- The draft technical specification was removed from the repository (kept in git
  history and locally). Design docs (`FORMAT.md`, `CRYPTO.md`) will be published
  with v1.2.

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

[Unreleased]: https://github.com/AmazingCoderA/secretpiece/compare/v1.1...HEAD
[1.1.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.1
[0.1.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v0.1.0
