//! The SPV1 container format: headers, key slots, superblocks, metadata.
//!
//! Planned contents (see `docs/TZ.ru.md`, section 9 and `docs/FORMAT.md`):
//!
//! - fixed layout: outer header A/A', hidden header B/B', superblocks;
//! - 512-byte key slots with wrapped master keys;
//! - encrypted header payload and encrypted index snapshots;
//! - generation-based commit and rollback detection;
//! - FAT32 part splitting and Reed-Solomon shard headers.
//!
//! All parsers must be total: arbitrary bytes may never cause a panic.

#![forbid(unsafe_code)]
