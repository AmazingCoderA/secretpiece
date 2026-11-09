//! C ABI for the `SecretPiece` core, used by the Tauri backend and other clients.
//!
//! Planned surface:
//!
//! `sp_vault_open`, `sp_vault_close`, `sp_vault_put`, `sp_vault_get`,
//! `sp_vault_list`, `sp_vault_delete`, `sp_vault_lock`.
//!
//! Rules: UTF-8 strings with explicit ownership, errors returned as a code plus
//! a thread-safe message that never contains secrets, and a generated header
//! (`cbindgen`) once the surface stabilizes.

#![allow(unsafe_code)]
