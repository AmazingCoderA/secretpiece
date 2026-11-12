<p align="center">
  <img src="docs/assets/hero.svg" alt="SecretPiece — encrypted vault in a single file" width="100%">
</p>

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-1f2937?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-6ea8ff?style=for-the-badge"></a>
</p>

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml"><img alt="Build" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases"><img alt="Release" src="https://img.shields.io/github/v/release/AmazingCoderA/secretpiece?label=latest&color=6ea8ff"></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20BSD-4de8ff">
  <a href="#license"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-blue"></a>
</p>

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Download latest release" src="https://img.shields.io/badge/⬇_Download_latest-2ea043?style=for-the-badge&logo=github&logoColor=white"></a>
  <a href="#installation"><img alt="Install" src="https://img.shields.io/badge/Install-by_platform-9b6bff?style=for-the-badge"></a>
</p>

---

**SecretPiece** is a cross-platform Rust tool that creates an encrypted vault inside an ordinary file. It works on a USB stick, an external HDD/SSD, an internal disk, or a network folder. No disk partitioning, no formatting, no drivers, no administrator rights. Open source, GPL-3.0.

> 🔐 **Your secret, in pieces.** One file. Inside — an encrypted virtual volume, a hidden volume with its own password, and everything needed to survive the loss of the drive.

<p align="center">
  <img src="https://img.shields.io/badge/status-v1.4%20·%20crypto%20core-9b6bff" alt="status">
  <img src="https://img.shields.io/badge/Argon2id%20%2B%20XChaCha20%20%2B%20BLAKE3-implemented-2ea043" alt="crypto implemented">
</p>

> ⚠️ **Beta.** The SPV1 format and the crypto core are implemented and covered by
> tests, but no external audit has been performed yet. Keep a backup of the container.

## ✨ Why SecretPiece

| | |
|---|---|
| 🧩 **Does not break the drive** | Only regular files. MBR/GPT, boot sector and the filesystem stay untouched. |
| 💾 **Any drive** | USB stick, external HDD/SSD, internal disk, SMB/NFS. Removable media is the primary scenario. |
| 🕵️ **Hidden volume** | The area inside the container looks like free space. It opens with a separate password (planned for v1.7). |
| 🔐 **Ciphers** | XChaCha20-Poly1305 ships in v1.4; ChaCha20-Poly1305 and AES-256-GCM come later, parameters recorded in the header. |
| 🗂️ **Multiple formats** | Single file, split parts for FAT32, vault folder, nested container, k-of-n shards. |
| 🚀 **Every platform** | Windows x64/ARM64, macOS universal, Linux x64/ARM64, FreeBSD/OpenBSD/NetBSD, Arch Linux. |
| 🤖 **Android** | APK: USB drive over OTG, SAF + JNI, on-device vault creation, RU/EN, disguise, snow/rain/sun. |
| 🛟 **Recoverability** | Two headers, two superblocks, `doctor`, BIP39 recovery, Shamir secret sharing, snapshots. |
| 🧹 **Secure deletion** | File and free-space shred, container crypto-erase — with an honest report. |
| 🌍 **RU / EN** | Interface and hints in Russian and English, auto-detection, `--lang`. |

## 📦 Installation

<p align="center">
  <a href="#windows"><img alt="Windows" src="https://img.shields.io/badge/Windows-x64_/_ARM64-0078D4?style=for-the-badge&logo=windows11&logoColor=white"></a>
  <a href="#macos"><img alt="macOS" src="https://img.shields.io/badge/macOS-universal-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="#android"><img alt="Android" src="https://img.shields.io/badge/Android-APK-3DDC84?style=for-the-badge&logo=android&logoColor=white"></a>
  <a href="#linux"><img alt="Linux" src="https://img.shields.io/badge/Linux-x64_/_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black"></a>
  <a href="#arch-linux"><img alt="Arch Linux" src="https://img.shields.io/badge/Arch_Linux-pkg-1793D1?style=for-the-badge&logo=archlinux&logoColor=white"></a>
  <a href="#bsd"><img alt="BSD" src="https://img.shields.io/badge/BSD-FreeBSD_/_OpenBSD_/_NetBSD-AB2B28?style=for-the-badge&logo=freebsd&logoColor=white"></a>
</p>

All builds live in [Releases](https://github.com/AmazingCoderA/secretpiece/releases/latest) together with `SHA256SUMS.txt`. Every push to `main` builds fresh artifacts automatically.

### Windows

```powershell
# GUI: secretpiece-<version>-windows-x64-gui.zip → sp.exe / secretpiece.exe
# CLI: secretpiece-<version>-windows-x64-cli.zip → sp-cli.exe
.\sp.exe                          # desktop application
.\sp-cli.exe --version            # console
```

Windows ARM64 — the `windows-arm64-cli` archive. Portable build: nothing is written to the registry, the config stays next to the binary.

### macOS

```bash
# universal: Intel + Apple Silicon
tar -xzf secretpiece-<version>-macos-universal-gui.tar.gz   # sp / secretpiece
tar -xzf secretpiece-<version>-macos-universal-cli.tar.gz   # sp-cli
xattr -dr com.apple.quarantine ./sp ./secretpiece ./sp-cli
./sp
```

### Android

```bash
# APK from Releases
adb install -r secretpiece-<version>-android.apk
```

Or enable "Install unknown apps" and open the APK on the phone (Android 8+,
ARM64). Scenario: USB drive over OTG/USB-C → "Choose .spv file" → password →
work with files. A new empty vault is created right there: "Create new vault" →
password and confirmation → "Save as" to the drive or phone storage. No root, no
`INTERNET` permission. Settings hold the language, an optional **calculator
disguise**, a home-screen shortcut, and an ambient background: snow, rain or
sun. Details: [`docs/ANDROID.md`](docs/ANDROID.md), and on the format itself:
[`docs/SPV.md`](docs/SPV.md).

> The APK talks to the shared core through `sp-ffi`: the SAF descriptor is passed
> directly, so files are read from the drive without copying the container into
> app storage, and a new container is written straight to the medium.

<p align="center">
  <img src="docs/assets/android.svg" alt="SecretPiece for Android: read a vault from a USB drive over OTG and create a container on the phone" width="100%">
</p>

### Linux

```bash
# separate archives: GUI and CLI
tar -xzf secretpiece-<version>-linux-x64-gui.tar.gz   # sp, secretpiece
tar -xzf secretpiece-<version>-linux-x64-cli.tar.gz   # sp-cli
./sp                  # GUI
./sp-cli --version    # CLI
```

ARM64 — `linux-arm64-cli` and `linux-arm64-gui` (GUI is built against glibc).
The CLI is static musl: it runs on Ubuntu 22.04+, Fedora, Debian, Arch and Alpine.
A dedicated Alpine archive is `alpine-x64-cli`.

### Arch Linux

```bash
# ready-made pacman package from Releases (x86_64)
sudo pacman -U secretpiece-1.4-1-x86_64.pkg.tar.zst

# AUR (once the recipe is published)
yay -S secretpiece-bin    # prebuilt binary
yay -S secretpiece        # build from source
# or: paru -S secretpiece-bin

# manually from source
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece/packaging/arch
makepkg -si
```

Recipes: [`packaging/arch/PKGBUILD`](packaging/arch/PKGBUILD),
[`packaging/aur/secretpiece-bin`](packaging/aur/secretpiece-bin/PKGBUILD) and
[`packaging/aur/secretpiece`](packaging/aur/secretpiece/PKGBUILD). Releases ship
both an `arch-x64` archive and a real `.pkg.tar.zst`.

### BSD

```sh
# FreeBSD
pkg install rust
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece && cargo build --release --locked -p sp-cli

# OpenBSD
pkg_add rust

# NetBSD
pkgin install rust
```

BSD builds in CI are marked experimental: a release still ships even if a BSD VM runner is unavailable.

## 🔐 Encryption

| Layer | Algorithm | Status |
|---|---|---|
| Data and metadata | **XChaCha20-Poly1305** (256-bit key, 192-bit nonce) | ✅ v1.4 |
| Password KDF | **Argon2id**: light 16 MiB · default 64 MiB · paranoid 256 MiB | ✅ v1.4 |
| Hashing and MAC | **BLAKE3** (headers and integrity checks) | ✅ v1.4 |
| Alternative | **ChaCha20-Poly1305** (RFC 8439, 96-bit nonce) | optional, later |
| Alternative | **AES-256-GCM** (AES-NI / ARMv8 Crypto) | optional, later |
| Alternative | **AES-256-GCM-SIV** (nonce-misuse resistant) | optional, later |
| Key exchange | **X25519** for recipient slots | v1.7 |
| Recovery | **BIP39** (24 words) + **Shamir** k-of-n | v1.7 |
| Unlock slots | password · keyfile · FIDO2 hmac-secret · recovery · X25519 | v1.4–v1.7 |
| Manifest signatures | **Ed25519** (releases only, not volumes) | v2.0 |

- `algorithm_id` and all KDF parameters are written into the header; silent changes are forbidden.
- Changing the algorithm is only possible through `sp migrate`, which creates a new container.
- No custom ciphers, ECB, CBC without MAC, SHA-1/MD5, or "obfuscation instead of encryption".
- Slots and header payload are indistinguishable from random data without the key.

## 🗂️ Container formats

| Format | What is created | Constraints |
|---|---|---|
| **SPV1 · single file** | one `*.spv` | up to 4 TiB, best for exFAT/NTFS/APFS/ext4 |
| **SPV1 · split** | `*.spv.000`, `.001`, … | FAT32, 3.5 GiB parts |
| **Vault folder (L2)** | folder of blocks + index | convenient for cloud and large volumes |
| **Nested (L4)** | container inside the outer volume | no free-space overlap |
| **Sharded (L5)** | k-of-n Reed-Solomon shards | each part < 4 GiB, no FAT32 split |
| **Archive export** | encrypted copy for backup | requires password/recovery |

## 🗺️ Architecture

<p align="center">
  <img src="docs/assets/architecture.svg" alt="SecretPiece architecture: interfaces, vault, format, crypto, SPV1 layout" width="100%">
</p>

```
0x00000  header A       ← outer volume
0x10000  header A'      ← backup
0x20000  header B       ← hidden volume
0x30000  header B'      ← backup
0x40000  superblock A
0x50000  superblock B
0x60000  chunks…        ← data and metadata
N−hidden hidden area    ← overlaps free space (deniable)
```

## 🎛️ Storage modes

| Level | What it is | When to use |
|---|---|---|
| **L1 · Sealed file** | a single `*.spv` | backups, cloud, transfer |
| **L2 · Vault folder** | a folder of blocks + index | large volumes, sync |
| **L3 · Hidden volume** | outer + hidden volume at the end of the file | the main mode for USB drives |
| **L4 · Nested vault** | a container inside the outer volume | no free-space overlap |
| **L5 · Sharded vault** | k-of-n Reed-Solomon shards | disaster recovery, split storage |
| **L6 · Paranoid** | L3/L5 + snapshots + 2FA + replicas | maximum durability |

Durability profiles: **Fast** · **Balanced** (default) · **Paranoid** · **Custom**. A profile changes versioning, trash and verification policy — never cryptography or the format.

## 🚀 Quick start

```bash
# CLI: password via prompt, SP_PASSWORD, or --password-stdin
sp-cli init personal.spv
sp-cli put personal.spv .\Documents\report.pdf docs/report.pdf
sp-cli ls personal.spv
sp-cli get personal.spv docs/report.pdf .\report.pdf
sp-cli df personal.spv
sp-cli verify personal.spv

# GUI: a separate application
sp            # or secretpiece — dark UI, RU/EN
```

The password is never stored: the key is derived with Argon2id and lives only in
memory. The container is a regular `.spv` file and can be renamed to `report.pdf`.

## 🌍 Languages

- Russian and English — complete, including hints and error messages.
- Auto-detection: `LC_ALL`, `LC_MESSAGES`, `LANG`, `SP_LANG`.
- Explicit: `sp-cli --lang ru …` or `sp-cli --lang en …`.
- In the GUI — an RU/EN toggle in the header.
- New languages — through resource files, without touching the core.

<p align="center">
  <img src="docs/assets/cli.svg" alt="SecretPiece CLI in Russian and English" width="100%">
</p>

## 🧯 When you run out of space

SecretPiece checks free space **before** writing and never leaves partial commits. If space runs out:

| Situation | What the app does | What the user should do |
|---|---|---|
| Less than 10% free | warning + progress | empty the trash, delete old versions |
| 0 bytes free | refuses **before** writing, code `E_NOSPC` (9), volume → read-only | free space or grow the container |
| Import larger than free space | preflight refusal, no garbage | import in parts or into the outer volume |
| Media actually full | `E_NOSPC`, read-only, index stays intact | empty trash/snapshots, export data |
| FAT32 and a file > 4 GiB | automatic split into 3.5 GiB parts | use exFAT/NTFS |
| Need more capacity | `sp-cli` resize (planned v1.6) | create a new container and move data |

Hints are everywhere: in `sp-cli` errors, in `sp-cli df`, in the GUI banner, and in `sp-cli doctor`. A 64 MiB reserve is always kept for metadata and superblocks.

## 🛡️ Security

- The app makes **no network calls** and collects no telemetry.
- Passwords and keys never appear in argv, logs, or temporary files.
- The hidden volume is indistinguishable from random data without the password, **but the presence of a container file cannot be hidden**.
- Overwriting on flash/SSD does not guarantee physical destruction because of wear leveling. The reliable option is **crypto-erase**.
- Vulnerabilities: privately via [GitHub Security Advisories](https://github.com/AmazingCoderA/secretpiece/security/advisories/new), see [`SECURITY.md`](SECURITY.md).

## 🖥️ Platforms and architectures

| OS | Ready builds | Status |
|---|---|---|
| Windows 10/11 | x64 GUI + CLI, ARM64 CLI | v1.4 |
| macOS 13+ | universal (Intel + Apple Silicon) GUI + CLI | v1.4 |
| Android 8+ | APK (SAF + JNI), ARM64/ARMv7 | v1.4 (read + create) |
| Linux | x64 GUI (glibc), x64/ARM64 CLI (static musl) | v1.4 |
| Alpine Linux | x64 CLI (musl) | v1.4 |
| Arch Linux | x64 CLI, pacman package, PKGBUILD, AUR (yay/paru) | v1.4 |
| FreeBSD | x64 (VM build) | experimental |
| OpenBSD | x64 (VM build) | experimental |
| NetBSD | x64 (VM build) | experimental |
| Linux ARMv7 / RISC-V | build from source | planned v1.5 |
| Flatpak / AppImage | sandbox and portable image | planned v1.9 |

## 🛠️ Build and releases

```bash
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece
cargo build --workspace
cargo test --workspace
cargo run -p sp-cli --bin sp-cli -- --version   # CLI
cargo run -p sp-gui --bin sp                    # GUI (egui/eframe)
```

Requires Rust stable (MSRV **1.85**, edition 2024).

- **Auto-build** — on every push to `main` and every PR: artifacts on the workflow run page.
- **Release** — on a `v*` tag: archives + `SHA256SUMS.txt` in GitHub Releases.
- **Release assets:** `*-cli` and `*-gui` for `windows-x64`, `linux-x64`, `macos-universal`; `linux-arm64-cli`, `windows-arm64-cli`, `alpine-x64-cli`, `arch-x64-cli`, `*.pkg.tar.zst`, `android.apk` + `SHA256SUMS.txt`.
- Versions — `1.1`, `1.2`, `1.3`, `1.4`, `2.0` (no micro-patch tags).

## 📚 Documentation

| Document | Contents |
|---|---|
| [docs/SPV.md](docs/SPV.md) | what an `.spv` file is, SPV1 layout, file masking |
| [docs/ANDROID.md](docs/ANDROID.md) | Android: OTG, read + create, settings, disguise, ambience |
| [SECURITY.md](SECURITY.md) | vulnerability disclosure, scope, guarantees |
| [CONTRIBUTING.md](CONTRIBUTING.md) | development and review rules |
| [ADR](docs/ADR/README.md) | architecture decision records |
| [CHANGELOG.md](CHANGELOG.md) | change history |

The byte-level specification lives in [`docs/SPV.md`](docs/SPV.md); a separate `docs/FORMAT.md` will follow.

## 🧭 Roadmap

| Version | Status | Outcome |
|---|---|---|
| **v1.1** | ✅ | skeleton, CI, auto-build, GPL-3.0, any drive |
| **v1.2** | ✅ | visuals, RU/EN CLI, pacman/AUR, Android skeleton, Windows ARM64, Linux ARM64, BSD |
| **v1.3** | ✅ | Android: app disguise, `docs/SPV.md`, polish |
| **v1.4** | ✅ | SPV1 crypto core (Argon2id, XChaCha20-Poly1305, BLAKE3), CLI, egui GUI, Android read + on-device create, Alpine |
| **v1.5** | ⏳ | extents, FAT32 split, ARMv7/RISC-V, format migration |
| **v1.6** | ⏳ | reliability: COW, snapshots, trash, `doctor`, resize, crash tests |
| **v1.7** | ⏳ | hidden volume, recovery, Shamir, X25519 recipient slots |
| **v1.8** | ⏳ | shred, free-space wipe, destroy, Reed-Solomon sharding |
| **v1.9** | ⏳ | GUI: creation wizard, file manager, drag-and-drop, themes |
| **v2.0** | ⏳ | hardening, fuzzing, Miri, external audit, signatures, SBOM |

## 🤝 Contributing

The project is open to review, fuzzing and audit. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening a PR. Cryptographic and format changes require an ADR first.

## License

**GNU General Public License v3.0 only** (GPL-3.0-only). Full text: [`LICENSE`](LICENSE).

Dual-use tool. Use it only to protect your own data and within the law. The authors are not liable for data loss, misuse, or crypto-export restrictions in your jurisdiction.

## Download

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Windows" src="https://img.shields.io/badge/Windows-x64_/_ARM64-0078D4?style=for-the-badge&logo=windows11&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Linux" src="https://img.shields.io/badge/Linux-x64_/_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="macOS" src="https://img.shields.io/badge/macOS-universal-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="BSD" src="https://img.shields.io/badge/BSD-FreeBSD_/_OpenBSD_/_NetBSD-AB2B28?style=for-the-badge&logo=freebsd&logoColor=white"></a>
</p>

<p align="center"><sub>encrypted USB vault · hidden volume · VeraCrypt alternative · Rust · GPL-3.0 · open source</sub></p>

---

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-9b6bff?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
</p>

<p align="center">
  <sub>Made with ❤️ and paranoia · <b>SecretPiece</b> · Your secret, in pieces.</sub>
</p>
