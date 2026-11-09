//! Test fixtures, generators, and fault injection for SecretPiece.
//!
//! Planned contents (see `docs/TZ.ru.md`, section 19):
//!
//! - deterministic vault builders for integration tests;
//! - a fault-injecting storage backend (fail/short-write on the N-th call);
//! - crash scenario helpers: run, abort at a chosen step, reopen, run doctor;
//! - filesystem-matrix helpers and corpus generators for fuzzing.

#![forbid(unsafe_code)]
