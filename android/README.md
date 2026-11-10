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
| Password field, unlock flow | ✅ UI |
| Argon2id / XChaCha20-Poly1305 | ⏳ wired through `sp-ffi` in v1.4 |
| File browser inside the vault | ⏳ v1.4 |

Until the crypto core lands, the unlock button shows a dialog explaining that
the UI and USB access are ready and the format work is in progress. **Do not use
Android builds for real secrets yet.**

## Scenario: USB drive + phone

1. Plug the USB drive into the phone with an OTG adapter (or USB-C drive).
2. The system mounts it read/write (exFAT/NTFS/ext4/FAT32 — whatever the phone supports).
3. Tap **Choose .spv file**, pick `personal.spv` from the drive.
4. Enter the password.
5. In v1.4: browse, preview, copy files to and from the drive.

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

Requires JDK 17 and the Android SDK (compileSdk 34). With Gradle 8.7+:

```bash
cd android
gradle assembleDebug
# apk: app/build/outputs/apk/debug/app-debug.apk
```

Or open the `android/` folder in Android Studio.

## Security notes

- The app has **no `INTERNET` permission** — nothing leaves the device.
- Storage is accessed through the Storage Access Framework; the app receives a
  content URI, not a raw block device.
- Do not copy the container to shared storage or cloud sync folders while it is
  unlocked.
- Android may cache thumbnails or previews of exported files; prefer the in-app
  viewer in v1.4.
- USB drives can disappear mid-write (unplug, power loss). The Rust core will
  keep the same crash-safe commit model as on desktop.
