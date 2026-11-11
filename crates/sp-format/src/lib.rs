//! The SPV1 container format: a fixed header and an encrypted JSON index.
//!
//! Layout of a single-file container:
//!
//! ```text
//! 0x0000  header slot 0 (4096 bytes)
//! 0x1000  header slot 1 (4096 bytes)
//! 0x2000  data area: encrypted index snapshots and 64 KiB file chunks
//! ```
//!
//! Headers alternate between the two slots and carry a generation number, so a
//! crash in the middle of a commit leaves the previous generation readable.
//! Every header is checksummed with `BLAKE3`; the index is encrypted with the
//! vault master key.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sp_core::{ErrorCategory, ErrorCode, SpError};
use sp_crypto::KdfParams;

/// Magic bytes at the start of every container.
pub const MAGIC: [u8; 8] = *b"SPV1\0\0\0\0";
/// On-disk size of one header slot.
pub const HEADER_SIZE: usize = 4096;
/// Number of header slots.
pub const HEADER_SLOTS: u64 = 2;
/// Offset of the first data byte (after both header slots).
pub const DATA_START: u64 = HEADER_SIZE as u64 * HEADER_SLOTS;
/// Container format version written into headers.
pub const FORMAT_VERSION: u32 = 1;
/// Maximum serialized index size in bytes.
pub const MAX_INDEX_SIZE: u32 = 64 * 1024 * 1024;
/// Number of bytes covered by the header checksum.
const CHECKSUM_OFFSET: usize = HEADER_SIZE - 32;
/// Offset of the wrapped master key.
const WRAPPED_KEY_OFFSET: usize = 112;
/// Length of the wrapped master key (`32` key bytes + `16` tag bytes).
pub const WRAPPED_KEY_SIZE: usize = 48;

/// Fixed-size container header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Monotonic commit generation.
    pub generation: u64,
    /// `Argon2id` cost parameters.
    pub kdf: KdfParams,
    /// Salt for `Argon2id`.
    pub salt: [u8; 16],
    /// Nonce of the wrapped master key.
    pub key_nonce: [u8; 24],
    /// Master key sealed with the password-derived key.
    pub wrapped_key: [u8; WRAPPED_KEY_SIZE],
    /// Offset of the encrypted index snapshot, or `0` when empty.
    pub index_offset: u64,
    /// Length of the encrypted index snapshot.
    pub index_len: u32,
    /// Nonce of the encrypted index snapshot.
    pub index_nonce: [u8; 24],
}

impl Header {
    /// Byte offset of a header slot.
    #[must_use]
    pub const fn slot_offset(slot: u64) -> u64 {
        slot % HEADER_SLOTS * HEADER_SIZE as u64
    }

    /// Serializes the header into exactly [`HEADER_SIZE`] bytes.
    #[must_use]
    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut out = [0_u8; HEADER_SIZE];
        out[0..8].copy_from_slice(&MAGIC);
        out[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&0_u32.to_le_bytes());
        out[16..24].copy_from_slice(&self.generation.to_le_bytes());
        out[24..28].copy_from_slice(&self.kdf.m_cost.to_le_bytes());
        out[28..32].copy_from_slice(&self.kdf.t_cost.to_le_bytes());
        out[32..36].copy_from_slice(&self.kdf.p_cost.to_le_bytes());
        out[36..52].copy_from_slice(&self.salt);
        out[52..60].copy_from_slice(&self.index_offset.to_le_bytes());
        out[60..64].copy_from_slice(&self.index_len.to_le_bytes());
        out[64..88].copy_from_slice(&self.index_nonce);
        out[88..112].copy_from_slice(&self.key_nonce);
        out[WRAPPED_KEY_OFFSET..WRAPPED_KEY_OFFSET + WRAPPED_KEY_SIZE]
            .copy_from_slice(&self.wrapped_key);
        let digest = sp_crypto::checksum(&out[..CHECKSUM_OFFSET]);
        out[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        out
    }

    /// Parses and validates a header slot.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::Corrupted`] for a bad magic, version, checksum, or
    /// out-of-range field.
    pub fn decode(bytes: &[u8]) -> Result<Self, SpError> {
        if bytes.len() < HEADER_SIZE {
            return Err(corrupted("header is truncated"));
        }
        if bytes[0..8] != MAGIC {
            return Err(corrupted("not a SecretPiece container"));
        }
        let version = u32::from_le_bytes(read4(bytes, 8)?);
        if version != FORMAT_VERSION {
            return Err(SpError::new(
                ErrorCode::Corrupted,
                ErrorCategory::Corruption,
                "unsupported container format version",
            ));
        }
        let expected = sp_crypto::checksum(&bytes[..CHECKSUM_OFFSET]);
        if bytes[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 32] != expected {
            return Err(corrupted("header checksum mismatch"));
        }
        let kdf = KdfParams {
            m_cost: u32::from_le_bytes(read4(bytes, 24)?),
            t_cost: u32::from_le_bytes(read4(bytes, 28)?),
            p_cost: u32::from_le_bytes(read4(bytes, 32)?),
        };
        kdf.validate()?;
        let index_offset = u64::from_le_bytes(read8(bytes, 52)?);
        if index_offset != 0 && index_offset < DATA_START {
            return Err(corrupted("index points inside the header area"));
        }
        let index_len = u32::from_le_bytes(read4(bytes, 60)?);
        if index_len > MAX_INDEX_SIZE {
            return Err(corrupted("index is too large"));
        }
        let mut salt = [0_u8; 16];
        salt.copy_from_slice(&bytes[36..52]);
        let mut index_nonce = [0_u8; 24];
        index_nonce.copy_from_slice(&bytes[64..88]);
        let mut key_nonce = [0_u8; 24];
        key_nonce.copy_from_slice(&bytes[88..112]);
        let mut wrapped_key = [0_u8; WRAPPED_KEY_SIZE];
        wrapped_key
            .copy_from_slice(&bytes[WRAPPED_KEY_OFFSET..WRAPPED_KEY_OFFSET + WRAPPED_KEY_SIZE]);
        Ok(Self {
            generation: u64::from_le_bytes(read8(bytes, 16)?),
            kdf,
            salt,
            key_nonce,
            wrapped_key,
            index_offset,
            index_len,
            index_nonce,
        })
    }
}

fn read4(bytes: &[u8], offset: usize) -> Result<[u8; 4], SpError> {
    bytes
        .get(offset..offset + 4)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| corrupted("header is truncated"))
}

fn read8(bytes: &[u8], offset: usize) -> Result<[u8; 8], SpError> {
    bytes
        .get(offset..offset + 8)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| corrupted("header is truncated"))
}

fn corrupted(message: &str) -> SpError {
    SpError::new(ErrorCode::Corrupted, ErrorCategory::Corruption, message)
}

/// Kind of a namespace entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    /// A regular file stored as encrypted chunks.
    File,
    /// A directory; no payload.
    Dir,
}

/// Location of one encrypted chunk inside the data area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRef {
    /// Absolute byte offset of the nonce inside the container.
    pub offset: u64,
    /// Bytes from the nonce start to the end of the tag.
    pub len: u32,
}

/// One file or directory record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// File or directory.
    pub kind: EntryKind,
    /// Plaintext length for files, `0` for directories.
    pub size: u64,
    /// Modification time in Unix seconds.
    pub mtime: i64,
    /// Encrypted chunks; empty for directories.
    #[serde(default)]
    pub chunks: Vec<ChunkRef>,
}

impl Entry {
    /// Creates a directory record.
    #[must_use]
    pub const fn dir(mtime: i64) -> Self {
        Self {
            kind: EntryKind::Dir,
            size: 0,
            mtime,
            chunks: Vec::new(),
        }
    }
}

/// Decrypted namespace index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// Index generation, mirrored into the header on commit.
    #[serde(default)]
    pub generation: u64,
    /// Path to entry. Paths use `/`, no leading slash.
    #[serde(default)]
    pub entries: BTreeMap<String, Entry>,
}

impl Index {
    /// Serializes the index to JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::Io`] when serialization fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SpError> {
        serde_json::to_vec(self).map_err(|_| {
            SpError::new(
                ErrorCode::Io,
                ErrorCategory::Io,
                "cannot serialize the vault index",
            )
        })
    }

    /// Parses an index and rejects oversized payloads.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::Corrupted`] when the payload is not a valid index.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SpError> {
        if bytes.len() > MAX_INDEX_SIZE as usize {
            return Err(corrupted("index is too large"));
        }
        serde_json::from_slice(bytes).map_err(|_| corrupted("cannot parse the vault index"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Header {
        Header {
            generation: 7,
            kdf: KdfParams::LIGHT,
            salt: [1_u8; 16],
            key_nonce: [2_u8; 24],
            wrapped_key: [3_u8; WRAPPED_KEY_SIZE],
            index_offset: DATA_START,
            index_len: 128,
            index_nonce: [4_u8; 24],
        }
    }

    #[test]
    fn header_roundtrip() {
        let original = header();
        let encoded = original.encode();
        let decoded = Header::decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn header_detects_corruption() {
        let mut encoded = header().encode();
        encoded[200] ^= 0xFF;
        assert!(Header::decode(&encoded).is_err());
    }

    #[test]
    fn header_rejects_foreign_files() {
        let mut encoded = header().encode();
        encoded[0] = b'X';
        assert_eq!(
            Header::decode(&encoded).unwrap_err().code(),
            ErrorCode::Corrupted
        );
    }

    #[test]
    fn index_roundtrip() {
        let mut index = Index {
            generation: 3,
            ..Index::default()
        };
        index.entries.insert(
            "docs/readme.txt".to_owned(),
            Entry {
                kind: EntryKind::File,
                size: 11,
                mtime: 42,
                chunks: vec![ChunkRef {
                    offset: DATA_START,
                    len: 27,
                }],
            },
        );
        index.entries.insert("docs".to_owned(), Entry::dir(41));
        let bytes = index.to_bytes().unwrap();
        let decoded = Index::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, index);
    }

    #[test]
    fn slot_offsets_alternate() {
        assert_eq!(Header::slot_offset(0), 0);
        assert_eq!(Header::slot_offset(1), HEADER_SIZE as u64);
        assert_eq!(Header::slot_offset(2), 0);
    }
}
