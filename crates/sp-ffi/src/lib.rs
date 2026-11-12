//! JNI bridge for the Android application, plus the start of the C ABI.
//!
//! Android opens the `.spv` container through the Storage Access Framework and
//! passes the raw file descriptor to [`Vault::open_file`], so the vault is read
//! directly from the USB drive without copying it into app storage.
//!
//! The bridge is deliberately read-only: list, read, and inspect. Mutations stay
//! in the desktop application and the CLI.

#![allow(unsafe_code)]

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jbyteArray, jint, jlong, jstring};
use sp_format::EntryKind;
use sp_vault::Vault;

const TAG: &str = "sp-ffi";

/// One live vault owned by Kotlin.
struct Handle {
    vault: Vault,
}

fn to_jstring(env: &mut JNIEnv<'_>, value: &str) -> jstring {
    match env.new_string(value) {
        Ok(string) => string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn read_password(env: &mut JNIEnv<'_>, password: &JString<'_>) -> Option<Vec<u8>> {
    let text: String = env.get_string(password).ok()?.into();
    Some(text.into_bytes())
}

fn handle_from(raw: jlong) -> Option<&'static mut Handle> {
    if raw == 0 {
        return None;
    }
    // The pointer was created by `Box::into_raw` in `openFd`.
    Some(unsafe { &mut *(raw as *mut Handle) })
}

fn open_from_fd(fd: i32, password: &[u8]) -> Option<Handle> {
    #[cfg(unix)]
    {
        use std::fs::File;
        use std::os::unix::io::FromRawFd;
        use std::path::PathBuf;
        // SAFETY: the descriptor is detached by Kotlin and owned by us now.
        let file = unsafe { File::from_raw_fd(fd) };
        match Vault::open_file(file, PathBuf::from("android-fd"), password) {
            Ok(vault) => Some(Handle { vault }),
            Err(_) => None,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (fd, password);
        None
    }
}

fn create_from_fd(fd: i32, password: &[u8]) -> Option<Handle> {
    #[cfg(unix)]
    {
        use std::fs::File;
        use std::os::unix::io::FromRawFd;
        use std::path::PathBuf;
        // SAFETY: the descriptor is detached by Kotlin and owned by us now.
        let file = unsafe { File::from_raw_fd(fd) };
        match Vault::create_file(
            file,
            PathBuf::from("android-fd"),
            password,
            sp_crypto::KdfParams::DEFAULT,
        ) {
            Ok(vault) => Some(Handle { vault }),
            Err(_) => None,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (fd, password);
        None
    }
}

/// Creates a vault inside an empty file descriptor. Returns `0` on failure.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_createFd(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    fd: jint,
    password: JString<'_>,
) -> jlong {
    let Some(password) = read_password(&mut env, &password) else {
        return 0;
    };
    match create_from_fd(fd, &password) {
        Some(handle) => Box::into_raw(Box::new(handle)) as jlong,
        None => 0,
    }
}

/// Opens a vault from a detached file descriptor. Returns `0` on failure.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_openFd(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    fd: jint,
    password: JString<'_>,
) -> jlong {
    let Some(password) = read_password(&mut env, &password) else {
        return 0;
    };
    match open_from_fd(fd, &password) {
        Some(handle) => Box::into_raw(Box::new(handle)) as jlong,
        None => 0,
    }
}

/// Lists direct children of `prefix` as a JSON array.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_list(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    prefix: JString<'_>,
) -> jstring {
    let Some(handle) = handle_from(raw) else {
        return to_jstring(&mut env, "[]");
    };
    let prefix: String = env
        .get_string(&prefix)
        .map_or_else(|_| String::new(), Into::into);
    let trimmed = prefix.trim_matches('/');
    let mut items = Vec::new();
    for entry in handle.vault.list(if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }) {
        let name = entry
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&entry.path)
            .to_owned();
        let direct = if trimmed.is_empty() {
            !entry.path.contains('/')
        } else {
            entry
                .path
                .strip_prefix(trimmed)
                .is_some_and(|rest| rest.starts_with('/') && !rest[1..].contains('/'))
        };
        if !direct {
            continue;
        }
        items.push(serde_json::json!({
            "name": name,
            "path": entry.path,
            "kind": match entry.kind {
                EntryKind::Dir => "dir",
                EntryKind::File => "file",
            },
            "size": entry.size,
        }));
    }
    let payload = serde_json::Value::Array(items).to_string();
    to_jstring(&mut env, &payload)
}

/// Reads a file and returns its bytes, or `null` on failure.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_read(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    path: JString<'_>,
) -> jbyteArray {
    let Some(handle) = handle_from(raw) else {
        return std::ptr::null_mut();
    };
    let Ok(path): Result<String, _> = env.get_string(&path).map(Into::into) else {
        return std::ptr::null_mut();
    };
    let Ok(data) = handle.vault.get(&path) else {
        return std::ptr::null_mut();
    };
    match env.byte_array_from_slice(&data) {
        Ok(array) => array.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns container statistics as a JSON object, or `null`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_info(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
) -> jstring {
    let Some(handle) = handle_from(raw) else {
        return std::ptr::null_mut();
    };
    let Ok(info) = handle.vault.info() else {
        return std::ptr::null_mut();
    };
    let payload = serde_json::json!({
        "bytes": info.file_size,
        "payload": info.payload_bytes,
        "files": info.files,
        "dirs": info.dirs,
        "generation": info.generation,
    })
    .to_string();
    to_jstring(&mut env, &payload)
}

/// Re-authenticates every file. Returns `true` when the vault is intact.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_verify(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
) -> jint {
    let Some(handle) = handle_from(raw) else {
        return 0;
    };
    i32::from(handle.vault.verify().is_ok())
}

/// Releases the vault and wipes the in-memory key.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_close(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
) {
    if raw == 0 {
        return;
    }
    // SAFETY: the pointer came from `Box::into_raw` and is released once.
    drop(unsafe { Box::from_raw(raw as *mut Handle) });
}

/// Reports the library version, useful for `System.loadLibrary` diagnostics.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_version(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    to_jstring(
        &mut env,
        &format!(
            "SecretPiece {} · SPV{} · {TAG}",
            env!("CARGO_PKG_VERSION"),
            sp_core::FORMAT_VERSION
        ),
    )
}
