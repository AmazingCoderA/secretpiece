//! Thin OS integration layer.
//!
//! Planned contents (see `docs/TZ.ru.md`, section 7.5):
//!
//! - removable media detection, filesystem type, free space;
//! - secure memory (`VirtualLock`/`mlock`), core-dump suppression;
//! - screen lock notifications, eject, screenshot protection (best effort);
//! - temporary files with restrictive permissions.
//!
//! Everything here must degrade gracefully: if a feature is unavailable, the
//! application warns and continues without leaking secrets.

#![allow(unsafe_code)]
