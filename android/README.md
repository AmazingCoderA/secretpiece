# SecretPiece for Android

![SecretPiece for Android](../docs/assets/android.svg)

Compose app that opens a `.spv` container directly from a USB drive connected
over OTG/USB-C. No root, no drivers, no `MANAGE_EXTERNAL_STORAGE`.

## Current status

| Part | Status |
|---|---|
| Compose UI (RU/EN) | ✅ |
| System file picker (SAF) | ✅ |
| Open a `.spv` from a file manager (`VIEW` intent) | ✅ |
| App disguise (calculator) | ✅ |
| Password field, unlock flow | ✅ |
| Argon2id / XChaCha20-Poly1305 / BLAKE3 | ✅ v1.4 through `sp-ffi` |
| Folder browsing and file reads | ✅ v1.4 (read-only) |
| Create a new vault on the phone | ✅ v1.5 (password + confirmation dialog) |
| Settings: language, ambience, disguise, shortcut | ✅ v1.4 |
| Ambient effects: snow / rain / sun | ✅ v1.4 |
| Writing/deleting files on the phone | ⏳ later |
| Image preview | ⏳ later |

## How the container is read

The app never copies the `.spv` into internal storage. It takes the
`ParcelFileDescriptor` from the Storage Access Framework, calls `detachFd()`, and
passes the descriptor to the Rust core (`sp-ffi`). There it becomes a `File` and
is opened with `Vault::open_file`, using the same `sp-vault` crate as the desktop
GUI and CLI.

The bridge is **read-oriented**: `list`, `read`, `info`, `verify`, `close`, plus
`createFd` for a brand-new empty container. Writing into an existing vault,
renaming and deleting stay in the desktop app and `sp-cli`.

## Scenario: USB drive + phone

1. Plug the USB drive into the phone with an OTG adapter (or USB-C drive).
2. The system mounts it read/write (exFAT/NTFS/ext4/FAT32 — whatever the phone supports).
3. Tap **Choose .spv file**, pick `personal.spv` from the drive.
4. Enter the password.
5. Browse folders and tap a file to preview its text.

Alternatively, tap the `.spv` file in any file manager: SecretPiece registers a
`VIEW` intent and receives the content URI.

## What is an `.spv` file

See [`../docs/SPV.md`](../docs/SPV.md). Short version: a regular file with an
encrypted vault inside. The extension is cosmetic — the format comes from the
header, so the same file still opens after renaming it to `photo.jpg`.

## App disguise (optional)

Under **Advanced** there is a **Disguise app** switch. When it is on:

- the launcher shows **Calculator** with a calculator icon instead of
  SecretPiece;
- tapping Calculator opens SecretPiece;
- the `.spv` container, its name and the password are unchanged.

Turn the switch off to restore the normal icon and name. The choice is stored in
`SharedPreferences` and re-applied on launch.

Implementation: two `activity-alias` entries (`LauncherSecretPiece` and
`LauncherCalculator`), toggled with `PackageManager.setComponentEnabledSetting`.
No extra permissions are required.

> App disguise does not hide the container file and is not a replacement for the
> hidden volume. To mask a file, rename it and/or use a hidden volume.

## Build

Requires JDK 17, the Android SDK (compileSdk 34), the NDK, and Rust Android
targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a \
  -o android/app/src/main/jniLibs build --release -p sp-ffi
cd android
gradle assembleDebug
# apk: app/build/outputs/apk/debug/app-debug.apk
```

Or open the `android/` folder in Android Studio; the `.so` files in `jniLibs`
are picked up automatically.

## Security notes

- The app has **no `INTERNET` permission** — nothing leaves the device.
- Storage is accessed through the Storage Access Framework; the app receives a
  content URI, not a raw block device.
- The key lives in the Rust core's memory and is zeroized on `close`/exit.
- Android may cache thumbnails or previews of exported files; the in-app viewer
  only shows text and writes nothing to disk.
- USB drives can disappear mid-read (unplug, power loss). Reads never modify the
  container.
