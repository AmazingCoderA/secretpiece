//! Secure deletion, free-space wiping, and crypto-erase.
//!
//! Planned contents:
//!
//! - shred of file chunks (overwrite, fsync, drop file key, commit);
//! - free-space wipe of a vault and of a filesystem (with verification);
//! - container destroy: overwrite headers/superblocks, remove parts;
//! - honest reports: what was overwritten and what cannot be guaranteed on
//!   flash/SSD due to wear leveling.

#![forbid(unsafe_code)]
