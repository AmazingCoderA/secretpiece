//! High-level vault operations on top of the SPV1 format.
//!
//! A [`Vault`] owns an open container file, the unwrapped master key, and the
//! decrypted namespace index. Every mutation is committed by writing the new
//! encrypted index and then a checksummed header with a higher generation, so
//! a crash leaves the previous generation readable.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sp_core::{CHUNK_SIZE, ErrorCategory, ErrorCode, SpError};
use sp_crypto::{KdfParams, Key};
use sp_format::{
    ChunkRef, DATA_START, Entry, EntryKind, HEADER_SIZE, HEADER_SLOTS, Header, Index,
    MAX_INDEX_SIZE,
};

const KEY_AAD: &[u8] = b"spv1-key";
const INDEX_AAD: &[u8] = b"spv1-index";

/// One namespace entry returned by [`Vault::list`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEntry {
    /// Normalized path without a leading slash.
    pub path: String,
    /// File or directory.
    pub kind: EntryKind,
    /// Plaintext size in bytes.
    pub size: u64,
    /// Modification time in Unix seconds.
    pub mtime: i64,
}

/// Aggregate usage of an open vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultInfo {
    /// Physical container size in bytes.
    pub file_size: u64,
    /// Number of files.
    pub files: u64,
    /// Number of directories.
    pub dirs: u64,
    /// Total plaintext bytes stored in files.
    pub payload_bytes: u64,
    /// Current commit generation.
    pub generation: u64,
    /// Offset of the active index snapshot.
    pub index_offset: u64,
    /// Length of the active index snapshot.
    pub index_len: u32,
    /// `Argon2id` parameters recorded in the header.
    pub kdf: KdfParams,
}

/// Result of a full integrity pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyReport {
    /// Files verified.
    pub files: u64,
    /// Plaintext bytes read and authenticated.
    pub bytes: u64,
}

/// An unlocked vault.
pub struct Vault {
    path: PathBuf,
    file: File,
    header: Header,
    index: Index,
    key: Key,
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault")
            .field("path", &self.path)
            .field("generation", &self.header.generation)
            .field("entries", &self.index.entries.len())
            .finish_non_exhaustive()
    }
}

impl Vault {
    /// Creates a new container, truncating an existing file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the file cannot be created, or a usage error
    /// when the password is empty.
    pub fn create(path: &Path, password: &[u8], kdf: KdfParams) -> Result<Self, SpError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|error| io_error(&error))?;
        Self::create_file(file, path, password, kdf)
    }

    /// Creates a vault inside an already-open file (used by the JNI bridge).
    ///
    /// # Errors
    ///
    /// Same as [`Vault::create`].
    pub fn create_file(
        mut file: File,
        path: &Path,
        password: &[u8],
        kdf: KdfParams,
    ) -> Result<Self, SpError> {
        if password.is_empty() {
            return Err(SpError::new(
                ErrorCode::InvalidArguments,
                ErrorCategory::Usage,
                "the password must not be empty",
            ));
        }
        kdf.validate()?;
        let salt = sp_crypto::random_salt()?;
        let kek = sp_crypto::derive_key(password, &salt, kdf)?;
        let master = Key::random()?;
        let key_nonce = sp_crypto::random_nonce()?;
        let wrapped = sp_crypto::seal(&kek, &key_nonce, KEY_AAD, master.as_bytes())?;
        let mut wrapped_key = [0_u8; sp_format::WRAPPED_KEY_SIZE];
        if wrapped.len() != wrapped_key.len() {
            return Err(SpError::new(
                ErrorCode::General,
                ErrorCategory::Usage,
                "unexpected key slot size",
            ));
        }
        wrapped_key.copy_from_slice(&wrapped);
        let header = Header {
            generation: 0,
            kdf,
            salt,
            key_nonce,
            wrapped_key,
            index_offset: 0,
            index_len: 0,
            index_nonce: [0_u8; 24],
        };
        file.set_len(DATA_START).map_err(|error| io_error(&error))?;
        for slot in 0..HEADER_SLOTS {
            write_header(&mut file, slot, &header)?;
        }
        file.sync_all().map_err(|error| io_error(&error))?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            header,
            index: Index::default(),
            key: master,
        })
    }

    /// Opens and unlocks an existing container.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::UnlockFailed`] for a wrong password or damaged key
    /// slot, and [`ErrorCode::Corrupted`] for an unreadable index.
    pub fn open(path: &Path, password: &[u8]) -> Result<Self, SpError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| io_error(&error))?;
        Self::open_file(file, path.to_path_buf(), password)
    }

    /// Opens and unlocks a vault from an already-open file.
    ///
    /// # Errors
    ///
    /// Same as [`Vault::open`].
    pub fn open_file(mut file: File, path: PathBuf, password: &[u8]) -> Result<Self, SpError> {
        let header = read_best_header(&mut file)?;
        let kek = sp_crypto::derive_key(password, &header.salt, header.kdf)?;
        let master = sp_crypto::open(&kek, &header.key_nonce, KEY_AAD, &header.wrapped_key)
            .map_err(|_| SpError::unlock())?;
        let key = Key::from_slice(&master).ok_or_else(SpError::unlock)?;
        let index = if header.index_len == 0 {
            Index::default()
        } else {
            let offset = header.index_offset;
            let len = header.index_len as usize;
            if offset < DATA_START || len == 0 || len > MAX_INDEX_SIZE as usize {
                return Err(corrupted("index location is invalid"));
            }
            file.seek(SeekFrom::Start(offset))
                .map_err(|error| io_error(&error))?;
            let mut ciphertext = vec![0_u8; len];
            file.read_exact(&mut ciphertext)
                .map_err(|error| io_error(&error))?;
            let plaintext = sp_crypto::open(&key, &header.index_nonce, INDEX_AAD, &ciphertext)?;
            Index::from_bytes(&plaintext)?
        };
        Ok(Self {
            path,
            file,
            header,
            index,
            key,
        })
    }

    /// The container path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Aggregate usage information.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the file metadata cannot be read.
    pub fn info(&self) -> Result<VaultInfo, SpError> {
        let mut files = 0_u64;
        let mut dirs = 0_u64;
        let mut payload_bytes = 0_u64;
        for entry in self.index.entries.values() {
            match entry.kind {
                EntryKind::File => {
                    files += 1;
                    payload_bytes = payload_bytes.saturating_add(entry.size);
                }
                EntryKind::Dir => dirs += 1,
            }
        }
        Ok(VaultInfo {
            file_size: self
                .file
                .metadata()
                .map_err(|error| io_error(&error))?
                .len(),
            files,
            dirs,
            payload_bytes,
            generation: self.header.generation,
            index_offset: self.header.index_offset,
            index_len: self.header.index_len,
            kdf: self.header.kdf,
        })
    }

    /// Lists entries, optionally filtered by a path prefix.
    #[must_use]
    pub fn list(&self, prefix: Option<&str>) -> Vec<ListEntry> {
        let normalized = prefix.map(normalize_lossy);
        self.index
            .entries
            .iter()
            .filter(|(path, _)| match &normalized {
                Some(prefix) if !prefix.is_empty() => {
                    path.starts_with(prefix) && path.as_str() != prefix
                }
                _ => true,
            })
            .map(|(path, entry)| ListEntry {
                path: path.clone(),
                kind: entry.kind,
                size: entry.size,
                mtime: entry.mtime,
            })
            .collect()
    }

    /// Creates a directory and any missing parents.
    ///
    /// # Errors
    ///
    /// Returns a usage error for invalid paths and a conflict error when a file
    /// already exists at that path.
    pub fn mkdir(&mut self, path: &str) -> Result<(), SpError> {
        let path = normalize(path)?;
        if self
            .index
            .entries
            .get(&path)
            .is_some_and(|entry| entry.kind == EntryKind::File)
        {
            return Err(SpError::new(
                ErrorCode::Busy,
                ErrorCategory::Busy,
                "a file with this name already exists",
            ));
        }
        self.ensure_parents(&path)?;
        self.index
            .entries
            .entry(path)
            .or_insert_with(|| Entry::dir(now_unix()));
        self.save()
    }

    /// Writes (or replaces) a file.
    ///
    /// # Errors
    ///
    /// Returns a usage error for invalid paths, a conflict error when a
    /// directory exists at the target path, and an I/O error on write failure.
    pub fn put(&mut self, path: &str, data: &[u8]) -> Result<(), SpError> {
        let path = normalize(path)?;
        if path.is_empty() {
            return Err(invalid_path());
        }
        if self
            .index
            .entries
            .get(&path)
            .is_some_and(|entry| entry.kind == EntryKind::Dir)
        {
            return Err(SpError::new(
                ErrorCode::Busy,
                ErrorCategory::Busy,
                "a directory with this name already exists",
            ));
        }
        self.ensure_parents(&path)?;
        let mut chunks = Vec::with_capacity(data.len().div_ceil(CHUNK_SIZE));
        for (index, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let nonce = sp_crypto::random_nonce()?;
            let aad = chunk_aad(&path, index);
            let ciphertext = sp_crypto::seal(&self.key, &nonce, &aad, chunk)?;
            let mut record = Vec::with_capacity(nonce.len() + ciphertext.len());
            record.extend_from_slice(&nonce);
            record.extend_from_slice(&ciphertext);
            let offset = self.append(&record)?;
            let len = u32::try_from(record.len()).map_err(|_| no_space())?;
            chunks.push(ChunkRef { offset, len });
        }
        self.index.entries.insert(
            path,
            Entry {
                kind: EntryKind::File,
                size: data.len() as u64,
                mtime: now_unix(),
                chunks,
            },
        );
        self.save()
    }

    /// Reads and authenticates a file.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::NotFound`] for a missing path, a usage error for a
    /// directory, and [`ErrorCode::Integrity`] when a chunk fails to decrypt.
    pub fn get(&mut self, path: &str) -> Result<Vec<u8>, SpError> {
        let path = normalize(path)?;
        let entry = self
            .index
            .entries
            .get(&path)
            .cloned()
            .ok_or_else(not_found)?;
        if entry.kind != EntryKind::File {
            return Err(SpError::new(
                ErrorCode::InvalidArguments,
                ErrorCategory::Usage,
                "the path is a directory",
            ));
        }
        let mut output = Vec::with_capacity(usize::try_from(entry.size).unwrap_or(0));
        for (index, chunk) in entry.chunks.iter().enumerate() {
            let len = chunk.len as usize;
            if len < sp_crypto::NONCE_SIZE + sp_crypto::TAG_SIZE {
                return Err(corrupted("chunk is too small"));
            }
            self.file
                .seek(SeekFrom::Start(chunk.offset))
                .map_err(|error| io_error(&error))?;
            let mut record = vec![0_u8; len];
            self.file
                .read_exact(&mut record)
                .map_err(|error| io_error(&error))?;
            let (nonce, ciphertext) = record.split_at(sp_crypto::NONCE_SIZE);
            let nonce: [u8; 24] = nonce
                .try_into()
                .map_err(|_| corrupted("invalid chunk nonce"))?;
            let aad = chunk_aad(&path, index);
            let plaintext = sp_crypto::open(&self.key, &nonce, &aad, ciphertext)?;
            output.extend_from_slice(&plaintext);
        }
        if output.len() as u64 != entry.size {
            return Err(corrupted("file size does not match the index"));
        }
        Ok(output)
    }

    /// Removes a file or a directory tree.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::NotFound`] when the path does not exist.
    pub fn remove(&mut self, path: &str) -> Result<(), SpError> {
        let path = normalize(path)?;
        if path.is_empty() {
            return Err(invalid_path());
        }
        let is_dir = match self.index.entries.get(&path) {
            Some(entry) => entry.kind == EntryKind::Dir,
            None => return Err(not_found()),
        };
        if is_dir {
            let prefix = format!("{path}/");
            self.index
                .entries
                .retain(|key, _| key != &path && !key.starts_with(&prefix));
        } else {
            self.index.entries.remove(&path);
        }
        self.save()
    }

    /// Re-reads and authenticates every file.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::Integrity`] when any chunk fails authentication.
    pub fn verify(&mut self) -> Result<VerifyReport, SpError> {
        let paths: Vec<String> = self
            .index
            .entries
            .iter()
            .filter(|(_, entry)| entry.kind == EntryKind::File)
            .map(|(path, _)| path.clone())
            .collect();
        let mut files = 0_u64;
        let mut bytes = 0_u64;
        for path in paths {
            let data = self.get(&path)?;
            files += 1;
            bytes = bytes.saturating_add(data.len() as u64);
        }
        Ok(VerifyReport { files, bytes })
    }

    /// Closes the vault, wiping the in-memory key.
    pub fn close(self) {
        // `Key` zeroizes on drop.
    }

    fn ensure_parents(&mut self, path: &str) -> Result<(), SpError> {
        let now = now_unix();
        let mut current = String::new();
        let segments: Vec<&str> = path.split('/').collect();
        for segment in &segments[..segments.len().saturating_sub(1)] {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(segment);
            match self.index.entries.get(&current) {
                Some(entry) if entry.kind == EntryKind::File => {
                    return Err(SpError::new(
                        ErrorCode::Busy,
                        ErrorCategory::Busy,
                        "a parent path is a file",
                    ));
                }
                Some(_) => {}
                None => {
                    self.index.entries.insert(current.clone(), Entry::dir(now));
                }
            }
        }
        Ok(())
    }

    fn append(&mut self, bytes: &[u8]) -> Result<u64, SpError> {
        let offset = self
            .file
            .seek(SeekFrom::End(0))
            .map_err(|error| io_error(&error))?;
        self.file
            .write_all(bytes)
            .map_err(|error| io_error(&error))?;
        Ok(offset)
    }

    fn save(&mut self) -> Result<(), SpError> {
        let mut index = self.index.clone();
        index.generation = self.header.generation + 1;
        let plaintext = index.to_bytes()?;
        if plaintext.len() > MAX_INDEX_SIZE as usize {
            return Err(no_space());
        }
        let nonce = sp_crypto::random_nonce()?;
        let ciphertext = sp_crypto::seal(&self.key, &nonce, INDEX_AAD, &plaintext)?;
        let offset = self.append(&ciphertext)?;
        self.file.sync_data().map_err(|error| io_error(&error))?;
        let mut header = self.header.clone();
        header.generation = index.generation;
        header.index_offset = offset;
        header.index_len = u32::try_from(ciphertext.len()).map_err(|_| no_space())?;
        header.index_nonce = nonce;
        write_header(&mut self.file, header.generation % HEADER_SLOTS, &header)?;
        self.file.sync_all().map_err(|error| io_error(&error))?;
        self.header = header;
        self.index = index;
        Ok(())
    }
}

fn read_best_header(file: &mut File) -> Result<Header, SpError> {
    let mut best: Option<Header> = None;
    for slot in 0..HEADER_SLOTS {
        file.seek(SeekFrom::Start(Header::slot_offset(slot)))
            .map_err(|error| io_error(&error))?;
        let mut bytes = vec![0_u8; HEADER_SIZE];
        if file.read_exact(&mut bytes).is_err() {
            continue;
        }
        if let Ok(header) = Header::decode(&bytes) {
            if best
                .as_ref()
                .is_none_or(|current| header.generation > current.generation)
            {
                best = Some(header);
            }
        }
    }
    best.ok_or_else(|| corrupted("no valid header found"))
}

fn write_header(file: &mut File, slot: u64, header: &Header) -> Result<(), SpError> {
    file.seek(SeekFrom::Start(Header::slot_offset(slot)))
        .map_err(|error| io_error(&error))?;
    file.write_all(&header.encode())
        .map_err(|error| io_error(&error))
}

fn chunk_aad(path: &str, index: usize) -> Vec<u8> {
    format!("spv1-chunk\0{path}\0{index}").into_bytes()
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}

fn normalize_lossy(raw: &str) -> String {
    raw.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/")
}

/// Normalizes a user-supplied vault path.
///
/// # Errors
///
/// Returns a usage error for absolute paths, `..` segments, and NUL bytes.
pub fn normalize(raw: &str) -> Result<String, SpError> {
    if raw.contains('\0') {
        return Err(invalid_path());
    }
    let unified = raw.replace('\\', "/");
    let trimmed = unified.trim_matches('/');
    let mut parts = Vec::new();
    for part in trimmed.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err(invalid_path());
        }
        parts.push(part);
    }
    Ok(parts.join("/"))
}

fn invalid_path() -> SpError {
    SpError::new(
        ErrorCode::InvalidArguments,
        ErrorCategory::Usage,
        "invalid vault path",
    )
}

fn corrupted(message: &str) -> SpError {
    SpError::new(ErrorCode::Corrupted, ErrorCategory::Corruption, message)
}

fn not_found() -> SpError {
    SpError::new(
        ErrorCode::NotFound,
        ErrorCategory::NotFound,
        "no such file or directory",
    )
}

fn no_space() -> SpError {
    SpError::new(
        ErrorCode::NoSpace,
        ErrorCategory::NoSpace,
        "the container or the drive is full",
    )
}

fn io_error(error: &std::io::Error) -> SpError {
    let raw = error.raw_os_error();
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return SpError::new(
            ErrorCode::PermissionDenied,
            ErrorCategory::Permission,
            "permission denied",
        );
    }
    if error.kind() == std::io::ErrorKind::NotFound {
        return not_found();
    }
    // ENOSPC on Unix, ERROR_DISK_FULL / ERROR_HANDLE_DISK_FULL on Windows.
    if matches!(raw, Some(28 | 39 | 112)) {
        return no_space();
    }
    SpError::new(ErrorCode::Io, ErrorCategory::Io, "I/O error")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempVault {
        path: PathBuf,
    }

    impl TempVault {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("sp-vault-test-{}-{name}.spv", std::process::id()));
            let _ = std::fs::remove_file(&path);
            Self { path }
        }
    }

    impl Drop for TempVault {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    const PASSWORD: &[u8] = b"test-password";

    fn create(temp: &TempVault) -> Vault {
        Vault::create(&temp.path, PASSWORD, KdfParams::LIGHT).unwrap()
    }

    #[test]
    fn create_open_and_reopen() {
        let temp = TempVault::new("reopen");
        {
            let mut vault = create(&temp);
            vault.put("hello.txt", b"hello world").unwrap();
            vault.mkdir("docs").unwrap();
            vault.put("docs/a.txt", b"alpha").unwrap();
        }
        let mut vault = Vault::open(&temp.path, PASSWORD).unwrap();
        assert_eq!(vault.get("hello.txt").unwrap(), b"hello world");
        assert_eq!(vault.get("docs/a.txt").unwrap(), b"alpha");
        let entries = vault.list(None);
        assert!(entries.iter().any(|entry| entry.path == "docs"));
        assert!(entries.iter().any(|entry| entry.path == "docs/a.txt"));
    }

    #[test]
    fn wrong_password_is_rejected() {
        let temp = TempVault::new("wrong-pass");
        create(&temp);
        let error = Vault::open(&temp.path, b"nope").unwrap_err();
        assert_eq!(error.code(), ErrorCode::UnlockFailed);
    }

    #[test]
    fn chunks_roundtrip_large_file() {
        let temp = TempVault::new("large");
        let mut vault = create(&temp);
        let data: Vec<u8> = (0..200_000_u32)
            .map(|value| u8::try_from(value % 251).unwrap())
            .collect();
        vault.put("big.bin", &data).unwrap();
        assert_eq!(vault.get("big.bin").unwrap(), data);
        let report = vault.verify().unwrap();
        assert_eq!(report.files, 1);
        assert_eq!(report.bytes, data.len() as u64);
    }

    #[test]
    fn remove_directories_recursively() {
        let temp = TempVault::new("remove");
        let mut vault = create(&temp);
        vault.mkdir("a/b").unwrap();
        vault.put("a/b/c.txt", b"x").unwrap();
        vault.remove("a").unwrap();
        assert_eq!(vault.list(None).len(), 0);
        assert_eq!(
            vault.get("a/b/c.txt").unwrap_err().code(),
            ErrorCode::NotFound
        );
    }

    #[test]
    fn traversal_is_rejected() {
        let temp = TempVault::new("traversal");
        let mut vault = create(&temp);
        assert_eq!(
            vault.put("../evil", b"x").unwrap_err().code(),
            ErrorCode::InvalidArguments
        );
        assert_eq!(
            vault.get("..\\evil").unwrap_err().code(),
            ErrorCode::InvalidArguments
        );
    }

    #[test]
    fn empty_password_is_rejected() {
        let temp = TempVault::new("empty-pass");
        assert_eq!(
            Vault::create(&temp.path, b"", KdfParams::LIGHT)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArguments
        );
    }

    #[test]
    fn tampered_chunk_fails_authentication() {
        let temp = TempVault::new("tamper");
        {
            let mut vault = create(&temp);
            vault.put("secret.txt", b"important").unwrap();
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&temp.path)
            .unwrap();
        let offset = DATA_START;
        file.seek(SeekFrom::Start(offset)).unwrap();
        let mut byte = [0_u8; 1];
        file.read_exact(&mut byte).unwrap();
        byte[0] ^= 0xFF;
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&byte).unwrap();
        file.sync_all().unwrap();
        let mut vault = Vault::open(&temp.path, PASSWORD).unwrap();
        assert_eq!(
            vault.get("secret.txt").unwrap_err().code(),
            ErrorCode::Integrity
        );
    }

    #[test]
    fn generations_advance_and_headers_alternate() {
        let temp = TempVault::new("generation");
        let mut vault = create(&temp);
        vault.put("a", b"1").unwrap();
        let first = vault.info().unwrap().generation;
        vault.put("b", b"2").unwrap();
        let second = vault.info().unwrap().generation;
        assert_eq!(second, first + 1);
    }

    #[test]
    fn list_prefix_filters_without_returning_the_root() {
        let temp = TempVault::new("prefix");
        let mut vault = create(&temp);
        vault.mkdir("docs").unwrap();
        vault.put("docs/a.txt", b"a").unwrap();
        vault.put("other.txt", b"b").unwrap();
        let listed = vault.list(Some("docs"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].path, "docs/a.txt");
    }

    #[test]
    fn info_counts_files_and_directories() {
        let temp = TempVault::new("info");
        let mut vault = create(&temp);
        vault.put("file.txt", b"12345").unwrap();
        vault.mkdir("dir").unwrap();
        let info = vault.info().unwrap();
        assert_eq!(info.files, 1);
        assert_eq!(info.dirs, 1);
        assert_eq!(info.payload_bytes, 5);
        assert!(info.file_size >= DATA_START);
    }
}
