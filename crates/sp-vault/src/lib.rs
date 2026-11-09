//! High-level vault operations and sessions.
//!
//! Planned contents (see `docs/TZ.ru.md`, sections 7.3, 10, 15.1):
//!
//! - `Vault` session type; unlock, lock, auto-lock, policy enforcement;
//! - import/export streaming without loading whole files into memory;
//! - profiles (Fast/Balanced/Paranoid), snapshots, trash, recipients;
//! - no global keys: key material lives only inside the session.

#![forbid(unsafe_code)]
