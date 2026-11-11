//! Cryptographic primitives for `SecretPiece`.
//!
//! Everything the container needs: `Argon2id` for password stretching,
//! `XChaCha20-Poly1305` for authenticated encryption, and `BLAKE3` for
//! checksums. Keys are zeroized on drop and never formatted into logs.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use core::fmt;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use sp_core::{ErrorCategory, ErrorCode, SpError};
use zeroize::Zeroize;

/// Size of a symmetric key in bytes.
pub const KEY_SIZE: usize = 32;
/// Size of an `XChaCha20` nonce in bytes.
pub const NONCE_SIZE: usize = 24;
/// Size of a `Poly1305` tag in bytes.
pub const TAG_SIZE: usize = 16;
/// Size of an `Argon2id` salt in bytes.
pub const SALT_SIZE: usize = 16;

/// `Argon2id` cost parameters stored in every container header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in kibibytes.
    pub m_cost: u32,
    /// Number of passes.
    pub t_cost: u32,
    /// Degree of parallelism.
    pub p_cost: u32,
}

impl KdfParams {
    /// Fast profile: 16 MiB, 2 passes. Useful for tests and low-end devices.
    pub const LIGHT: Self = Self {
        m_cost: 16 * 1024,
        t_cost: 2,
        p_cost: 1,
    };
    /// Default profile: 64 MiB, 3 passes.
    pub const DEFAULT: Self = Self {
        m_cost: 64 * 1024,
        t_cost: 3,
        p_cost: 1,
    };
    /// Paranoid profile: 256 MiB, 6 passes, 2 lanes.
    pub const PARANOID: Self = Self {
        m_cost: 256 * 1024,
        t_cost: 6,
        p_cost: 2,
    };

    /// Validates the parameters against the `Argon2` limits.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::InvalidArguments`] when a parameter is out of range.
    pub fn validate(self) -> Result<(), SpError> {
        Params::new(self.m_cost, self.t_cost, self.p_cost, Some(KEY_SIZE))
            .map(|_| ())
            .map_err(|_| {
                SpError::new(
                    ErrorCode::InvalidArguments,
                    ErrorCategory::Usage,
                    "invalid Argon2 parameters",
                )
            })
    }

    /// Human-readable profile name used by the CLI and `doctor`.
    #[must_use]
    pub fn profile_name(self) -> &'static str {
        match self {
            Self::LIGHT => "light",
            Self::DEFAULT => "default",
            Self::PARANOID => "paranoid",
            _ => "custom",
        }
    }
}

impl Default for KdfParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A 256-bit symmetric key. Zeroized on drop, redacted in `Debug`.
#[derive(Clone)]
pub struct Key([u8; KEY_SIZE]);

impl Key {
    /// Generates a random key from the operating system CSPRNG.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::Io`] when the system random source fails.
    pub fn random() -> Result<Self, SpError> {
        let mut bytes = [0_u8; KEY_SIZE];
        random_into(&mut bytes)?;
        Ok(Self(bytes))
    }

    /// Builds a key from exactly 32 bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; KEY_SIZE]) -> Self {
        Self(bytes)
    }

    /// Builds a key from a slice, returning `None` for a wrong length.
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let array: [u8; KEY_SIZE] = bytes.try_into().ok()?;
        Some(Self(array))
    }

    /// The raw key bytes. Never log the result.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    /// Wipes the key material.
    pub fn wipe(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key(<redacted>)")
    }
}

/// Fills a buffer with cryptographically secure random bytes.
///
/// # Errors
///
/// Returns [`ErrorCode::Io`] when the system random source fails.
pub fn random_into(buffer: &mut [u8]) -> Result<(), SpError> {
    getrandom::getrandom(buffer).map_err(|_| {
        SpError::new(
            ErrorCode::Io,
            ErrorCategory::Io,
            "the system random source is unavailable",
        )
    })
}

/// Returns a fresh random nonce.
///
/// # Errors
///
/// Returns [`ErrorCode::Io`] when the system random source fails.
pub fn random_nonce() -> Result<[u8; NONCE_SIZE], SpError> {
    let mut nonce = [0_u8; NONCE_SIZE];
    random_into(&mut nonce)?;
    Ok(nonce)
}

/// Returns a fresh random `Argon2id` salt.
///
/// # Errors
///
/// Returns [`ErrorCode::Io`] when the system random source fails.
pub fn random_salt() -> Result<[u8; SALT_SIZE], SpError> {
    let mut salt = [0_u8; SALT_SIZE];
    random_into(&mut salt)?;
    Ok(salt)
}

/// Derives a key from a password with `Argon2id`.
///
/// # Errors
///
/// Returns [`ErrorCode::InvalidArguments`] for bad cost parameters and
/// [`ErrorCode::Io`] when the random source fails.
pub fn derive_key(
    password: &[u8],
    salt: &[u8; SALT_SIZE],
    params: KdfParams,
) -> Result<Key, SpError> {
    params.validate()?;
    let argon = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_SIZE)).map_err(|_| {
            SpError::new(
                ErrorCode::InvalidArguments,
                ErrorCategory::Usage,
                "invalid Argon2 parameters",
            )
        })?,
    );
    let mut output = [0_u8; KEY_SIZE];
    argon
        .hash_password_into(password, salt, &mut output)
        .map_err(|_| {
            SpError::new(
                ErrorCode::General,
                ErrorCategory::Usage,
                "key derivation failed",
            )
        })?;
    Ok(Key(output))
}

fn cipher(key: &Key) -> Result<XChaCha20Poly1305, SpError> {
    XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| {
        SpError::new(
            ErrorCode::General,
            ErrorCategory::Usage,
            "invalid key length",
        )
    })
}

/// Encrypts `plaintext` with `XChaCha20-Poly1305`.
///
/// The returned buffer contains the ciphertext followed by the 16-byte tag.
///
/// # Errors
///
/// Returns [`ErrorCode::General`] when encryption fails.
pub fn seal(
    key: &Key,
    nonce: &[u8; NONCE_SIZE],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, SpError> {
    cipher(key)?
        .encrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| {
            SpError::new(
                ErrorCode::General,
                ErrorCategory::Usage,
                "encryption failed",
            )
        })
}

/// Decrypts and verifies a `ciphertext || tag` buffer.
///
/// # Errors
///
/// Returns [`ErrorCode::Integrity`] when the tag does not match; callers map
/// that to an unlock failure only when the failing object is a key slot.
pub fn open(
    key: &Key,
    nonce: &[u8; NONCE_SIZE],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, SpError> {
    cipher(key)?
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| {
            SpError::new(
                ErrorCode::Integrity,
                ErrorCategory::Integrity,
                "authentication tag mismatch",
            )
        })
}

/// Computes an unkeyed `BLAKE3` checksum used for header integrity.
#[must_use]
pub fn checksum(bytes: &[u8]) -> [u8; KEY_SIZE] {
    *blake3::hash(bytes).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &[u8] = b"correct horse battery staple";

    #[test]
    fn roundtrip_derives_the_same_key() {
        let salt = [7_u8; SALT_SIZE];
        let a = derive_key(PASSWORD, &salt, KdfParams::LIGHT).unwrap();
        let b = derive_key(PASSWORD, &salt, KdfParams::LIGHT).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn wrong_password_derives_a_different_key() {
        let salt = [9_u8; SALT_SIZE];
        let a = derive_key(PASSWORD, &salt, KdfParams::LIGHT).unwrap();
        let b = derive_key(b"wrong", &salt, KdfParams::LIGHT).unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn seal_and_open_roundtrip() {
        let key = Key::random().unwrap();
        let nonce = random_nonce().unwrap();
        let plaintext = b"attack at dawn";
        let sealed = seal(&key, &nonce, b"ctx", plaintext).unwrap();
        let opened = open(&key, &nonce, b"ctx", &sealed).unwrap();
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn tampering_is_rejected() {
        let key = Key::random().unwrap();
        let nonce = random_nonce().unwrap();
        let mut sealed = seal(&key, &nonce, b"ctx", b"payload").unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0xFF;
        assert_eq!(
            open(&key, &nonce, b"ctx", &sealed).unwrap_err().code(),
            ErrorCode::Integrity
        );
    }

    #[test]
    fn wrong_aad_is_rejected() {
        let key = Key::random().unwrap();
        let nonce = random_nonce().unwrap();
        let sealed = seal(&key, &nonce, b"one", b"payload").unwrap();
        assert!(open(&key, &nonce, b"two", &sealed).is_err());
    }

    #[test]
    fn debug_does_not_leak_key_bytes() {
        let key = Key::from_bytes([42_u8; KEY_SIZE]);
        assert_eq!(format!("{key:?}"), "Key(<redacted>)");
    }

    #[test]
    fn kdf_profiles_validate() {
        assert!(KdfParams::LIGHT.validate().is_ok());
        assert!(KdfParams::DEFAULT.validate().is_ok());
        assert!(KdfParams::PARANOID.validate().is_ok());
        assert!(
            KdfParams {
                m_cost: 1,
                t_cost: 0,
                p_cost: 1
            }
            .validate()
            .is_err()
        );
    }
}
