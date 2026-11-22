# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.24.0] - 2026-11-22

Polish pass: faster preview, multi-file import and Explorer selection.

### Changed

- **Android import now accepts multiple files** at once; reports how many were
  added and how many failed, and refreshes the list only once.
- Binary/text detection on Android scans only the first 8 KB instead of the
  whole file, so opening a large file is much faster.
- **Desktop selection behaves like Explorer**: Ctrl+click toggles a file,
  Shift+click selects the whole range, plain click resets selection.

## [1.23.0] - 2026-11-22

SSH on Android, file creation and 40 themes everywhere.

### Added

- **SSH/SFTP server presets on Android** using the JSch Java client (password
  auth), alongside FTP. Presets store the connection kind.
- **Create an empty file** inside a vault on Android (📄 + in the browser).
- **30 additional themes on both platforms** (40 total): Aurora, Cyber, Coal,
  Cream, Lavender, Honey, Winter, Desert, Storm, Emerald, Ruby, Amber, Slate,
  Orchid, Coffee, Meadow, Coral, Indigo, Sand, Frost, Ember, Lilac, Olive, Sky,
  Plum, Copper, Charcoal, Pearl, Cobalt, Velvet — light and dark mixed.
- Desktop media (images, video, audio) continues to open through the existing
  external-preview setting; Android previews images/video inline.

## [1.22.0] - 2026-11-21

Media preview on Android.

### Added

- **Photo and video preview on Android** (experimental setting in Settings,
  "Фото/видео из ОЗУ"): images render inline from decrypted bytes; video is
  written to a temporary cache file for playback and wiped on close or lock.
- On the desktop, non-text media continues to open through the existing
  "external preview" setting (the same experimental mode in spirit).

## [1.21.0] - 2026-11-21

Explorer-style tree, async server upload and ten themes on both platforms.

### Added

- **Drag-and-drop in the desktop vault tree**: drag a file row or a folder onto
  any folder (or root) to move it. The tree is now a permanent side panel
  instead of a collapsed header.
- **Vault password in a server preset** on Android: the preset can store the
  container password separately from the FTP password, so signing in to the
  server needs no extra typing. Delete presets with the ✕ button.
- **Asynchronous upload on lock** — the UI no longer blocks while the vault is
  being sent back to the server.
- **Sepia, Ocean and Rose themes on the desktop**, giving parity with Android
  at ten themes on both platforms.
- Movable preview window on Android (drag the title bar) and a compact,
  height-limited Settings dialog.
- openSUSE `zypper install` instructions in the README.

## [1.20.0] - 2026-11-21

Server-only memory mode, custom preview limit and a Flathub submission.

### Changed

- **Memory-only vault is now a per-preset server option only** — the global
  toggle was removed, as requested. It is one mechanic of the server preset,
  not a separate global mode.
- Preview size limit is user-configurable in Android Settings
  (`Макс. размер просмотра, МБ`, 1–2048 MB, default 16), not hard-coded.

### Added

- Flathub submission files under `packaging/flathub` and a PR to Flathub
  (#11302). The app page will be
  `https://flathub.org/apps/dev.secretpiece.SecretPiece` once accepted.

## [1.19.0] - 2026-11-21

Memory-only server sessions, locked exports and an installed Windows GUI.

### Added

- **Memory-only server mode** on Android (per preset or global in Settings):
  the vault is downloaded into an `ashmem` `MemoryFile`, never written to
  disk, uploaded back on lock and then released. No local copy at all.
- **Locked exports**: settings/theme/language export freely, while key shares
  and server presets require entering the vault password in Settings first.
- Preset export to JSON.
- Windows GUI archives now include `install.ps1` — a user-level installer that
  creates a desktop shortcut; the same archive remains fully portable.
- Preview size limit raised from 2 MB to 16 MB.

### Changed

- README documents installed vs portable GUI builds and states plainly that
  there is no Flathub package (release asset only).

## [1.18.0] - 2026-11-21

Server presets on Android.

### Added

- **FTP server presets on Android**: save host/port/user/password/path in
  Settings, pick a preset at sign-in, test the connection, and open the vault
  straight from the server.
- On lock the vault is uploaded back and the temporary file is wiped; if the
  upload fails, the local copy is kept and the error is shown.
- The JNI bridge gained `ftpTest`, `ftpDownload` and `ftpUpload`.

### Changed

- Android now keeps the session password in memory so key-share export works
  after unlocking.

## [1.17.0] - 2026-11-21

Parity pass: recovery keys, ten themes, tables and a stricter release gate.

### Added

- **Recovery shares (Shamir) on both platforms**: split a secret into `K of N`
  shares, save them to a file, and recombine them. On Android this lives in
  Settings → "Ключи (Shamir)"; on the desktop in Settings.
- **Ten Android themes** (Night, Light, Paper, Mint, Sunset, Forest, Neon,
  Sepia, Ocean, Rose) with proper Material 3 light/dark colour schemes.
- **CSV/TSV table preview on Android**: delimited files open as a scrollable
  table instead of raw text.
- **Settings export on Android** — language, theme, sky and disguise state are
  written to a JSON file from Settings.
- CI compiles the Android Kotlin sources on every push.

### Fixed

- **Server presets**: the picker selection and the new-preset name no longer
  share one field, so selecting a preset no longer overwrites it on save;
  the stale connection status is cleared on selection.
- **Releases**: the Release workflow now verifies that the tag matches
  `Cargo.toml`, the Android `versionName` and the changelog before publishing,
  and the Android build is no longer `continue-on-error`. A broken APK can no
  longer delay or corrupt a release.

## [1.16.0] - 2026-11-21

Themes on Android and full write access on the phone.

### Added

- **Five Android themes** — Night, Light, Paper, Mint, Sunset — picked in
  Settings, persisted across launches, and applied to the whole UI including
  dialog surfaces and text colours.
- **Android write operations**: create a folder, import a file from the system
  picker, rename or move an entry, delete a file or a folder tree, and export a
  file back to the device. The JNI bridge (`sp-ffi`) gained `put`, `mkdir`,
  `remove` and `move`.
- CI now compiles the Android Kotlin sources on every push, so a broken APK can
  no longer reach a release unnoticed.

### Changed

- README Flatpak instructions now point at the current bundle name.

## [1.15.0] - 2026-11-21

Android: folder tree and crash guards.

### Fixed

- Opening a container no longer crashes the app when a native call fails:
  listing, statistics and file reads are wrapped, and the error is shown in the
  UI instead of killing the process.

### Added

- **Folder tree on Android**: a monospace tree of every folder (up to five
  levels) with the current folder highlighted; tap to jump anywhere in the vault.

## [1.14.0] - 2026-11-20

No more freezes on server work.

### Added

- **Background server operations**: test, download and upload run on a worker
  thread, so the window stays responsive on slow links. Settings has an on/off
  switch for background mode if the old synchronous behaviour is preferred.
- **Quick open from a preset**: the sign-in screen shows the preset picker and an
  "Open from server" button — pick SSH/FTP, press once, enter the password. No
  extra dialogs and no permanent local copy.
- Lock upload is asynchronous; the temp copy is wiped only after the server
  confirms the upload.

## [1.13.0] - 2026-11-20

Remote sessions without a permanent local copy.

### Added

- **Server mode without leftovers**: "Download and open" now places the vault in
  a uniquely named temp file, keeps working from it, uploads it back on lock and
  then overwrites and deletes the local copy. If the upload fails, the temp copy
  is kept so nothing is lost.
- Remote-session upload runs even when automatic sync is off, because the server
  file is the source of truth in this mode.

## [1.12.0] - 2026-11-20

Preview fixes, multi-file import, extraction choices, upload verification, and
the Shamir secret-sharing core. The vault format is unchanged.

### Fixed

- **Opening the same image or video twice** no longer breaks playback: every
  external preview gets its own uniquely named temp file, and earlier files are
  kept until the vault is locked instead of being wiped mid-playback.
- **Empty vault on the server**: after an FTP upload the remote size is checked
  against the local file (`SIZE`), and an empty local file is rejected before
  uploading, so a zero-byte container is reported instead of silently created.

### Added

- **Multi-file import**: the Add file dialog now accepts several files at once.
- **Extraction choice** in vault settings: system TEMP, the folder next to the
  container, or any custom folder — with a one-click "Beside the container".
- **Shamir secret sharing** in `sp-crypto`: split a secret into `n` shares, any
  `k` of which rebuild it, with GF(2^8) arithmetic and tests.

## [1.11.0] - 2026-11-19

Folders, multi-select moves, a table viewer, built-in help and honest docs. The
vault format is unchanged.

### Added

- **Vault tree**: a collapsible tree of every folder with the current root, so
  nested folders are one click away.
- **Multi-select and move**: tick any number of files and folders, press Move and
  pick a target folder (or root). Folders move together with their contents.
- **Table view**: CSV and TSV files get a grid with a bold header row, up to 500
  rows and 24 columns, next to the plain-text and hex views.
- **Built-in help** in Russian and English: creation, servers, folders, preview,
  sync, privacy and supported formats.
- **Default shell is no longer `.spv`**: new vaults use `.mp4`, and the file
  dialog offers `.mp4`, `.mp3`, `.opus`, `.avi`, `.png`, `.jpg`, `.jpeg`, `.gif`,
  `.docx`, `.pptx`, `.md`, `.sh`, `.pdf` and `.zip`.

### Changed

- **README (RU/EN) rewritten** to match what actually ships: single-file SPV1
  only, real encryption stack, migration between ciphers is manual for now, and
  the roadmap now shows v1.10/v1.11 as done.
- Container size is limited by free space; the index cap is documented as
  256 MiB instead of promising split or resize that do not exist yet.

## [1.10.0] - 2026-11-19

Server presets, per-container options, two more light themes and a proper text
viewer. The vault format is unchanged.

### Added

- **Server dialog at sign-in**: a wide "SSH / FTP server" plaque under the form
  and a Server button in the header open host, port, login, password and path
  fields, with Test, Download and open, and Create here.
- **Saved presets**: name a destination and store it; the picker appears at
  sign-in and in the create form when enabled in Settings. Password saving is a
  separate opt-in checkbox with a plain-text warning.
- **Per-container settings**: a Vault button after unlock opens options stored
  inside the container (`.secretpiece/settings`): external preview on/off,
  custom extraction folder, auto-upload on lock, and a per-vault server path.
- **Two more light themes**: Paper (warm) and Mint (green), joining Light.
- **Text preview improvements**: line count, word-wrap toggle, copy button and
  a cleaner header.
- **Privacy toggles** in Settings: show presets, allow saving passwords, upload
  on lock.

### Changed

- Remote files are extracted to the configured folder (system TEMP by default)
  and wiped on close; binary files can stay in the app as a hex view instead of
  being written to disk.

## [1.9.0] - 2026-11-19

Vaults can be created straight on a server, FTP joins SSH, and the theme
picker became a single menu. The vault format is unchanged.

### Added

- **Destination picker when creating a vault**: local file, network folder
  (UNC path), SSH/SCP or FTP. The vault is built locally and published to the
  server in the same step; the local copy stays open for editing.
- **FTP client** built on `std::net::TcpStream` with 4-8 second timeouts:
  connection test, upload and download, login plus password (kept in memory
  only, never written to settings).
- **SSH key/agent auth** is picked up automatically; the port defaults to 22
  for SSH and 21 for FTP.
- **Theme menu in the header**: one button with all five palettes and colour
  previews, replacing the row of swatches.

### Changed

- Availability checks are faster: `ConnectTimeout=3` for SSH/SCP and short
  socket timeouts for FTP.
- Server settings now remember the protocol and show a password field.
- Locking the vault uploads the current file to the configured server, so the
  remote copy stays in sync.

## [1.8.0] - 2026-11-18

Five themes, sorting, external preview with a configurable wipe, password
repack and an SSH server tab. The vault format is unchanged.

### Added

- **Sorting in the vault browser**: by name, size, date or kind, ascending or
  descending, chosen from a menu in the breadcrumb bar; folders stay on top.
- **Two more themes**: Sunset (orange/pink) and Forest (green), joining
  Midnight, Neon and Light; the choice is remembered.
- **Any file can be viewed**: text-like formats (including `json`, `fb2`, `xml`,
  `csv`, code) still open in the in-app viewer, while everything else — video,
  images, `epub`, archives — is decrypted into a temporary folder and opened
  with the system application after a warning.
- **Configurable wipe**: temporary preview copies are overwritten with random
  bytes N times (1–7, default 3) and deleted on close or via "Wipe now"; the
  count lives in the new Settings window.
- **Change password / repack**: decrypts the vault and rewrites it with a new
  password and Argon2id profile, keeping the data and replacing the file
  atomically with a backup.
- **Server tab (SSH/SCP)**: host, port, user and remote path, with Test,
  Download and Upload buttons. Any user works (root is not required); keys or
  ssh-agent are used, passwords are never stored.

## [1.7.0] - 2026-11-18

Faster and roomier vaults, drag-and-drop on the desktop, more disguise shells,
and an Android build that installs over 1.5. The container format is unchanged:
old vaults open as before, and new chunks are simply larger.

### Added

- **Drag & drop on the desktop**: drop files onto an open vault to add them,
  with the same overwrite confirmation as the "Add file" button; dropping a
  vault onto the unlock screen still fills the path.
- **More disguise shells**: `.spv`, `.bin`, `.mp3`, `.jpg`, `.pdf`, `.zip`,
  `.iso` and `.dat` chips on the create screen (any extension still works).
- **Android `versionCode 17` / `versionName 1.7`**, so the new APK upgrades an
  installed 1.5 build instead of being rejected as a downgrade.

### Changed

- **Chunk size 64 KiB → 1 MiB**: far fewer AEAD calls, seeks and allocations per
  file, which makes adding and extracting large files noticeably faster. Each
  chunk stores its own length, so 64 KiB containers stay readable.
- **Index limit 64 MiB → 256 MiB** and **metadata limit 256 MiB → 1 GiB**: room
  for hundreds of thousands of entries in a single vault.
- Workspace version bumped to 1.7.0.

## [1.5.0] - 2026-11-17

Desktop and Android polish: a redesigned desktop workspace and create screen on
both platforms, a dedicated container-creation flow on the phone, more Linux
architectures, and hardened releases. The vault format is unchanged — SPV1 files
from v1.4 open as before.

### Added

- **Desktop workspace redesign**: a settings bar at the top with three live
  theme swatches (Midnight, Neon, Light), an RU/EN segmented switch, a gradient
  accent line, soft radial glows, a responsive unlock card, and a styled vault
  browser. The chosen theme and language are remembered between launches.
- **In-container preview**: text and hex views read straight from the encrypted
  vault into RAM — nothing is written to disk, and the buffer is wiped when the
  dialog closes. Double-click a file to preview it.
- **Overwrite confirmation**: adding a path that already exists asks before the
  old data is replaced instead of overwriting silently.
- **Secure delete**: the delete dialog offers a "shred" option that overwrites
  every chunk with random bytes, flushes them, and only then removes the file
  (best effort on flash media).
- **Open a vault from the command line**: `sp path/to/vault.spv` prefills the
  file field, so the desktop app can be used from a file manager.
- **Redesigned desktop create screen**: a hero panel with the product summary,
  password visibility toggle, a strength meter, and a choice of three Argon2id
  profiles (light 16 MiB / default 64 MiB / paranoid 256 MiB) written into the
  header.
- **File disguise on desktop**: the shell extension can be `.spv`, `.bin`,
  `.mp3` or `.jpg` (or anything typed by hand). The container is still SPV1;
  the picker now accepts **all files**, not only `*.spv`.
- **Create-vault dialog on Android**: a separate screen with the password typed
  twice (minimum 8 characters, mismatch is rejected before the system picker
  opens). The empty SPV1 container is written straight to the USB drive or phone
  storage through `sp-ffi::createFd`.
- **Breadcrumb and copy** in the Android browser: the current path is shown as
  `root / folder / sub`, and text previews can be copied to the clipboard.
- **Linux ARMv7 and RISC-V 64 CLI builds** (`linux-armv7-cli`,
  `linux-riscv64-cli`) in the release matrix.
- **`.deb` and `.rpm` packages**: the Linux GUI with a desktop entry, icon and
  AppStream metadata, built from the release archive with `nfpm`.
- **Flatpak bundle** (`dev.secretpiece.SecretPiece`): a manifest in
  `packaging/flatpak`, a bundle in Releases, and USB/home access in the sandbox.
- **Demo video and real screenshots**: `docs/assets/demo.mp4`, `demo.gif` and
  `docs/assets/screens/*` are embedded in both READMEs.
- **`sp-ffi` tests** for the exact creation path used by JNI: create + reopen a
  container, and reject an empty password.

### Changed

- **Desktop browser toolbar**: Preview and Extract are enabled only for files,
  integrity check sits on the right, rows highlight on selection, and results
  appear as coloured status banners.
- **Android password field**: lock icon, eye toggle, and IME "Done" that submits
  the form; the unlock card is ordered picker → password → primary action →
  create.
- **Rotation no longer closes the vault**: `MainActivity` handles
  `orientation|screenSize|screenLayout|keyboardHidden` itself, so the open
  container and the typed state survive a turn.
- **Releases cannot silently ship a stale APK**: the publish job fails when the
  Android artifact is missing instead of uploading the previous build.
- **Docs**: refreshed desktop and Android mockups, README RU/EN and
  `docs/ANDROID.md` describe the new desktop workspace, on-device creation,
  settings and ambience.
- Workspace version bumped to 1.5.0; MSRV stays at Rust 1.85 (edition 2024).

## [1.4.0] - 2026-11-11

The crypto core, the SPV1 container, separate GUI/CLI applications, and Android
read support.

### Added

- **Real crypto core** (`sp-crypto`): Argon2id password KDF with three profiles
  (light 16 MiB, default 64 MiB, paranoid 256 MiB), XChaCha20-Poly1305 AEAD with
  per-chunk random nonces and path/chunk AAD, BLAKE3 header checksums, and
  zeroizing key material that is never formatted into logs.
- **SPV1 on-disk format** (`sp-format`): two 4 KiB header slots with alternating
  generations, checksums covering the whole slot, an encrypted JSON index, and
  64 KiB encrypted chunks. A crash leaves the previous generation readable.
- **Vault engine** (`sp-vault`): `create`, `open`, `put`, `get`, `mkdir`,
  `remove`, `list`, `verify`, `info`, with path normalization (`..` rejected),
  chunk authentication, and no-space/permission error mapping.
- **CLI** (`sp-cli`): `init`, `ls`, `put`, `get`, `mkdir`, `rm`, `df`, `verify`,
  `doctor`, RU/EN output, `--profile`, `--password-stdin`, `SP_PASSWORD`.
- **Desktop GUI** (`sp-gui`, egui/eframe): unlock/create screen, vault browser,
  add/extract/delete/new-folder, integrity check, live size counters, RU/EN,
  drag-and-drop, dark cyan/violet material design.
- **Android reads vaults** through `sp-ffi`: the SAF file descriptor is detached
  and passed to `Vault::open_file`, so files are read directly from the USB drive
  without copying the container into app storage. Compose browser with folder
  navigation, text preview, and integrity check.
- **Android can create a vault** through `sp-ffi::createFd`: the system
  "create document" picker gives a writable descriptor, and the empty SPV1
  container is written straight to the drive or phone storage.
- **Android settings and ambience**: RU/EN, optional disguise, home-screen
  shortcut pinning, and animated snow / rain / sun backgrounds rendered in
  Compose Canvas (no extra permissions, no network).
- **New release assets**: Alpine `alpine-x64-cli`, GUI archives for Windows,
  Linux and macOS (`*-gui`), and CLI archives (`*-cli`) for every platform.

### Changed

- **Binaries are separated by interface**: `sp` and `secretpiece` are the GUI,
  `sp-cli` is the CLI. Packaging recipes and CI were updated accordingly.
- Workspace version bumped to 1.4.0; MSRV stays at Rust 1.85 (edition 2024).

## [1.3.0] - 2026-11-11

### Added

- **Android app disguise** (optional, under "Advanced"): the launcher entry can
  appear as **Calculator** with its own icon. Implemented with two
  `activity-alias` entries toggled via `PackageManager.setComponentEnabledSetting`,
  persisted in `SharedPreferences`, no extra permissions. The container file and
  password are unchanged.
- `docs/SPV.md` (RU/EN): what an `.spv` file is, the SPV1 layout (headers,
  key slots, superblocks, chunks, hidden area), why the extension is cosmetic,
  and how renaming masks the file.
- `docs/ANDROID.md` and `android/README.md` now document the disguise and the
  difference between app masking and file masking.

## [1.2.0] - 2026-11-10

Packaging and visuals release. Still a skeleton: cryptography and the container
format are **not** implemented yet — do not use for real data.

### Added

- New hero banner (`docs/assets/hero.svg`) and a bilingual CLI preview
  (`docs/assets/cli.svg`) in Russian and English.
- Arch Linux `pacman` package built in CI and attached to releases
  (`secretpiece-1.2-1-x86_64.pkg.tar.zst`).
- AUR recipes for `secretpiece` (source build) and `secretpiece-bin`
  (prebuilt binary), with `yay`/`paru` instructions.
- Windows ARM64, Linux ARM64, FreeBSD and OpenBSD builds in the release matrix.
- Download buttons for Windows, Linux, macOS, BSD and Arch Linux.
- Android app skeleton (`android/`): Compose UI in RU/EN, SAF file picker for a
  `.spv` on a USB drive over OTG, `VIEW` intent, no `INTERNET` permission.
  Crypto wiring via `sp-ffi` is planned for v1.4 (`docs/ANDROID.md`).

### Changed

- Version scheme moved to `1.x` tags (`v1.1`, `v1.2`, `v2.0`), without
  micro-patch releases.
- License changed to **GPL-3.0-only**.
- Release workflow now runs on every push to `main`, on pull requests, and on
  `v*` tags; tag pushes publish a GitHub Release with `SHA256SUMS.txt`.

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
- `packaging/arch/PKGBUILD` for Arch Linux, AUR recipes (`secretpiece`,
  `secretpiece-bin`), and a real `.pkg.tar.zst` built in CI.
- Architecture diagram (`docs/assets/architecture.svg`), a new hero banner
  (`docs/assets/hero.svg`), and a bilingual CLI preview (`docs/assets/cli.svg`).
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

[Unreleased]: https://github.com/AmazingCoderA/secretpiece/compare/v1.5...HEAD
[1.5.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.5
[1.4.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.4
[1.3.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.3
[1.2.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.2
[1.1.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v1.1
[0.1.0]: https://github.com/AmazingCoderA/secretpiece/releases/tag/v0.1.0
