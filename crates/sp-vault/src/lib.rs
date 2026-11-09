//! High-level vault operations and sessions.
//!
//! Planned contents:
//!
//! - `Vault` session type; unlock, lock, auto-lock, policy enforcement;
//! - import/export streaming without loading whole files into memory;
//! - profiles (Fast/Balanced/Paranoid), snapshots, trash, recipients;
//! - no global keys: key material lives only inside the session.

#![forbid(unsafe_code)]
