//! Cryptographic primitives and key hierarchy for SecretPiece.
//!
//! Planned contents (see `docs/TZ.ru.md`, section 8):
//!
//! - Argon2id password KDF and HKDF-SHA256 domain separation;
//! - XChaCha20-Poly1305 AEAD for headers, slots, metadata, and chunks;
//! - BLAKE3 hashing and keyed BLAKE3 MAC;
//! - X25519 recipient slots, Shamir secret sharing, BIP39 recovery codes;
//! - secure key storage (`secrecy`, `zeroize`), CSPRNG access.
//!
//! No custom cryptography is allowed here. Every primitive must be a
//! well-reviewed crate, backed by test vectors.

#![forbid(unsafe_code)]
