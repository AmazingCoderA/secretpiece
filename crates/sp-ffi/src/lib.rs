//! JNI bridge for the Android application, plus the start of the C ABI.
//!
//! Android opens the `.spv` container through the Storage Access Framework and
//! passes the raw file descriptor to [`Vault::open_file`], so the vault is read
//! directly from the USB drive without copying it into app storage. A new empty
//! container is written the same way through [`Vault::create_file`].
//!
//! The bridge is read-oriented: list, read, inspect, and create. Mutations of an
//! existing vault stay in the desktop application and the CLI.

#![allow(unsafe_code)]

mod ftp;

use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyteArray, jint, jlong, jstring};
use sp_crypto::shamir::split;
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

fn jtext(env: &mut JNIEnv<'_>, value: &JString<'_>) -> String {
    env.get_string(value)
        .map_or_else(|_| String::new(), Into::into)
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
        use std::path::Path;
        // SAFETY: the descriptor is detached by Kotlin and owned by us now.
        let file = unsafe { File::from_raw_fd(fd) };
        match Vault::create_file(
            file,
            Path::new("android-fd"),
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

/// Writes a file into the vault. Returns 1 on success.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_put(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    path: JString<'_>,
    data: JByteArray<'_>,
) -> jint {
    let Some(handle) = handle_from(raw) else {
        return 0;
    };
    let Ok(path): Result<String, _> = env.get_string(&path).map(Into::into) else {
        return 0;
    };
    let Ok(bytes) = env.convert_byte_array(&data) else {
        return 0;
    };
    i32::from(handle.vault.put(&path, &bytes).is_ok())
}

/// Creates a folder. Returns 1 on success.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_mkdir(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    path: JString<'_>,
) -> jint {
    let Some(handle) = handle_from(raw) else {
        return 0;
    };
    let Ok(path): Result<String, _> = env.get_string(&path).map(Into::into) else {
        return 0;
    };
    i32::from(handle.vault.mkdir(&path).is_ok())
}

/// Removes a file or a folder tree. Returns 1 on success.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_remove(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    path: JString<'_>,
) -> jint {
    let Some(handle) = handle_from(raw) else {
        return 0;
    };
    let Ok(path): Result<String, _> = env.get_string(&path).map(Into::into) else {
        return 0;
    };
    i32::from(handle.vault.remove(&path).is_ok())
}

/// Moves a file or a folder tree to a new path. Returns 1 on success.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_move(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    from: JString<'_>,
    to: JString<'_>,
) -> jint {
    let Some(handle) = handle_from(raw) else {
        return 0;
    };
    let Ok(from): Result<String, _> = env.get_string(&from).map(Into::into) else {
        return 0;
    };
    let Ok(to): Result<String, _> = env.get_string(&to).map(Into::into) else {
        return 0;
    };
    let entries: Vec<(String, EntryKind)> = handle
        .vault
        .list(None)
        .into_iter()
        .map(|entry| (entry.path, entry.kind))
        .collect();
    let mut result = Ok(());
    for (path, kind) in entries {
        let new_path = if path == from {
            to.clone()
        } else if let Some(rest) = path.strip_prefix(&format!("{from}/")) {
            format!("{to}/{rest}")
        } else {
            continue;
        };
        result = match kind {
            EntryKind::Dir => handle.vault.mkdir(&new_path),
            EntryKind::File => handle.vault.get(&path).and_then(|mut data| {
                let put = handle.vault.put(&new_path, &data);
                data.fill(0);
                put
            }),
        };
        if result.is_err() {
            break;
        }
    }
    if result.is_ok() && from != to {
        let is_dir = handle
            .vault
            .list(None)
            .iter()
            .any(|entry| entry.path == from && entry.kind == EntryKind::Dir);
        if is_dir {
            result = handle.vault.remove(&from);
        } else {
            result = handle.vault.shred(&from);
        }
    }
    i32::from(result.is_ok())
}

fn result_string(env: &mut JNIEnv<'_>, value: Result<(), String>) -> jstring {
    to_jstring(env, &value.map_or_else(|error| error, |()| "ok".to_owned()))
}

/// Checks an FTP server preset. Returns "ok" or the error text.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_ftpTest(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    host: JString<'_>,
    port: JString<'_>,
    user: JString<'_>,
    password: JString<'_>,
    path: JString<'_>,
) -> jstring {
    let (host, port, user, password, path) = (
        jtext(&mut env, &host),
        jtext(&mut env, &port),
        jtext(&mut env, &user),
        jtext(&mut env, &password),
        jtext(&mut env, &path),
    );
    let result = ftp::Ftp::connect(&host, &port, &user, &password).and_then(|mut ftp| {
        if path.trim().is_empty() {
            return Ok(());
        }
        ftp.retr(path.trim()).map(|_| ())
    });
    result_string(&mut env, result)
}

/// Downloads a remote vault to a local file. Returns "ok" or the error text.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_ftpDownload(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    host: JString<'_>,
    port: JString<'_>,
    user: JString<'_>,
    password: JString<'_>,
    path: JString<'_>,
    local: JString<'_>,
) -> jstring {
    let (host, port, user, password, path, local) = (
        jtext(&mut env, &host),
        jtext(&mut env, &port),
        jtext(&mut env, &user),
        jtext(&mut env, &password),
        jtext(&mut env, &path),
        jtext(&mut env, &local),
    );
    let result = ftp::Ftp::connect(&host, &port, &user, &password)
        .and_then(|mut ftp| ftp.retr(path.trim()))
        .and_then(|bytes| {
            if bytes.is_empty() {
                return Err("remote vault is empty".to_owned());
            }
            std::fs::write(&local, bytes).map_err(|error| format!("{local}: {error}"))
        });
    result_string(&mut env, result)
}

/// Uploads the local vault back and verifies the remote size.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_ftpUpload(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    host: JString<'_>,
    port: JString<'_>,
    user: JString<'_>,
    password: JString<'_>,
    path: JString<'_>,
    local: JString<'_>,
) -> jstring {
    let (host, port, user, password, path, local) = (
        jtext(&mut env, &host),
        jtext(&mut env, &port),
        jtext(&mut env, &user),
        jtext(&mut env, &password),
        jtext(&mut env, &path),
        jtext(&mut env, &local),
    );
    let result = std::fs::read(&local)
        .map_err(|error| format!("{local}: {error}"))
        .and_then(|bytes| {
            if bytes.is_empty() {
                return Err("local vault is empty".to_owned());
            }
            ftp::Ftp::connect(&host, &port, &user, &password)
                .and_then(|mut ftp| ftp.stor(path.trim(), &bytes))
                .map(|remote| {
                    if remote != bytes.len() as u64 {
                        return Err(format!(
                            "server has {remote} bytes, expected {}",
                            bytes.len()
                        ));
                    }
                    Ok(())
                })?
        });
    result_string(&mut env, result)
}

/// Splits the vault password into Shamir shares (`x:hex` lines as JSON array).
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_secretpiece_app_NativeVault_shares(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    raw: jlong,
    secret: JString<'_>,
    n: jint,
    k: jint,
) -> jstring {
    if handle_from(raw).is_none() {
        return to_jstring(&mut env, "[]");
    }
    let Ok(secret): Result<String, _> = env.get_string(&secret).map(Into::into) else {
        return to_jstring(&mut env, "[]");
    };
    let parts = split(
        secret.as_bytes(),
        u8::try_from(k).unwrap_or(0),
        u8::try_from(n).unwrap_or(0),
    )
    .map(|shares| {
        shares
            .iter()
            .map(|share| {
                let mut body = String::with_capacity(share.y.len() * 2);
                {
                    use std::fmt::Write as FmtWrite;
                    for byte in &share.y {
                        let _ = write!(body, "{byte:02x}");
                    }
                }
                format!("{}:{}", share.x, body)
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    to_jstring(&mut env, &serde_json::json!(parts).to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let mut path = std::env::temp_dir();
        path.push(format!("sp-ffi-{tag}-{}-{stamp}", std::process::id()));
        path
    }

    fn writable(path: &std::path::Path) -> std::fs::File {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("open temporary container")
    }

    #[test]
    fn create_file_writes_a_readable_container() {
        let path = temp_path("create");
        let vault = Vault::create_file(
            writable(&path),
            &path,
            b"correct horse battery staple",
            sp_crypto::KdfParams::LIGHT,
        )
        .expect("create vault");
        assert_eq!(vault.info().expect("info").files, 0);
        drop(vault);

        let reopened = Vault::open(&path, b"correct horse battery staple").expect("reopen vault");
        assert_eq!(reopened.info().expect("info").files, 0);
        drop(reopened);

        std::fs::remove_file(&path).expect("remove temporary container");
    }

    #[test]
    fn create_file_rejects_an_empty_password() {
        let path = temp_path("empty");
        let result = Vault::create_file(writable(&path), &path, b"", sp_crypto::KdfParams::LIGHT);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }
}
