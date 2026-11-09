//! Domain types, error codes, and policies for SecretPiece.
//!
//! This crate is deliberately free of I/O and cryptography. It defines the
//! vocabulary shared by every other crate: error codes, storage profiles, and
//! operational limits.

#![forbid(unsafe_code)]

use core::fmt;

/// Human-readable name of the product.
pub const PRODUCT_NAME: &str = "SecretPiece";

/// Version of the container format implemented by this build.
pub const FORMAT_VERSION: u32 = 1;

/// Open-plaintext chunk size (64 KiB), as fixed by the SPV1 format.
pub const CHUNK_SIZE: usize = 64 * 1024;

/// Size of the AEAD tag appended to every on-disk chunk.
pub const AEAD_TAG_SIZE: usize = 16;

/// Size of an on-disk encrypted chunk.
pub const ENCRYPTED_CHUNK_SIZE: usize = CHUNK_SIZE + AEAD_TAG_SIZE;

/// Fixed header area size (64 KiB).
pub const HEADER_SIZE: usize = 64 * 1024;

/// Maximum metadata size in v1 (256 MiB).
pub const MAX_METADATA_SIZE: u64 = 256 * 1024 * 1024;

/// Category of an error, used to decide whether to go read-only or lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Wrong arguments or unsupported operation.
    Usage,
    /// Unlock failure. Must not reveal which slot or header was close.
    Unlock,
    /// Corrupted container or unsupported format.
    Corruption,
    /// Underlying I/O failure.
    Io,
    /// Permission denied.
    Permission,
    /// Object not found.
    NotFound,
    /// Resource busy or locked by another process.
    Busy,
    /// Not enough space.
    NoSpace,
    /// Operation cancelled by the user.
    Cancelled,
    /// Integrity verification failed.
    Integrity,
}

/// Stable numeric error code, used by the CLI exit status and the FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ErrorCode {
    /// Generic failure.
    General = 1,
    /// Invalid arguments.
    InvalidArguments = 2,
    /// The vault could not be unlocked.
    UnlockFailed = 3,
    /// Corrupted or incompatible format.
    Corrupted = 4,
    /// I/O error.
    Io = 5,
    /// Permission denied.
    PermissionDenied = 6,
    /// Not found.
    NotFound = 7,
    /// Conflict or busy.
    Busy = 8,
    /// No space left.
    NoSpace = 9,
    /// Cancelled by the user.
    Cancelled = 10,
    /// Unsupported operation.
    Unsupported = 11,
    /// Integrity check failed.
    Integrity = 12,
}

impl ErrorCode {
    /// Returns the process exit status associated with this code.
    ///
    /// `General` maps to 1, every other variant to its stable number.
    #[must_use]
    pub const fn exit_status(self) -> i32 {
        self as i32
    }
}

/// An error that is safe to show to the user and to log.
///
/// The message never contains paths, file names, passwords, or key material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpError {
    code: ErrorCode,
    category: ErrorCategory,
    message: String,
}

impl SpError {
    /// Creates a new error from a code, category, and non-sensitive message.
    #[must_use]
    pub fn new(code: ErrorCode, category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            code,
            category,
            message: message.into(),
        }
    }

    /// The single unlock error. Deliberately does not say what failed.
    #[must_use]
    pub fn unlock() -> Self {
        Self::new(
            ErrorCode::UnlockFailed,
            ErrorCategory::Unlock,
            "cannot unlock: wrong password/key or damaged header",
        )
    }

    /// The stable numeric code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// The category used for recovery policy.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    /// The user-visible message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SpError {}

/// Durability profile. A policy layer on top of the same cryptography and format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Profile {
    /// No versions, no trash, no journal.
    Fast,
    /// Versions, trash, journal, write verification. The default.
    #[default]
    Balanced,
    /// Long retention, replicas, read-back verification.
    Paranoid,
    /// User-defined values.
    Custom,
}

impl Profile {
    /// Default auto-lock timeout in seconds for this profile.
    #[must_use]
    pub const fn auto_lock_seconds(self) -> u32 {
        match self {
            Self::Fast => 300,
            Self::Balanced => 180,
            Self::Paranoid => 60,
            Self::Custom => 180,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_layout_is_consistent() {
        assert_eq!(ENCRYPTED_CHUNK_SIZE, 65_552);
    }

    #[test]
    fn unlock_error_has_stable_code() {
        let err = SpError::unlock();
        assert_eq!(err.code(), ErrorCode::UnlockFailed);
        assert_eq!(err.code().exit_status(), 3);
        assert_eq!(err.category(), ErrorCategory::Unlock);
    }

    #[test]
    fn balanced_is_default() {
        assert_eq!(Profile::default(), Profile::Balanced);
        assert_eq!(Profile::Paranoid.auto_lock_seconds(), 60);
    }
}
