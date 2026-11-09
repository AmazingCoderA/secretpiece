//! Chunk I/O, random access, file locking, fsync, and FAT32 splitting.
//!
//! Planned contents:
//!
//! - a `ChunkStore`-style abstraction with positional reads and writes;
//! - one-file and multi-part backends (`.spv.000`, `.spv.001`, ...);
//! - platform-correct durability (`FlushFileBuffers`, `F_FULLFSYNC`, `fsync`);
//! - exclusive/shared locks and read-only fallback.
//!
//! This is one of the few crates where `unsafe` may appear (e.g. `mmap`,
//! `O_DIRECT`), always with a `// SAFETY:` comment and review.
