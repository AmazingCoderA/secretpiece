//! Virtual filesystem: paths, extents, versions, snapshots, trash.
//!
//! Planned contents (see `docs/TZ.ru.md`, section 10):
//!
//! - UTF-8, NFC-normalized paths with strict validation;
//! - file records with per-version keys, extents, refcounts, tags;
//! - copy-on-write index updates and retention policies;
//! - namespace operations that never expose plaintext to disk.

#![forbid(unsafe_code)]
