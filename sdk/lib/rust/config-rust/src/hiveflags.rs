//! `HHIVE::HiveFlags` bits — the one true copy, transcribed from
//! `sdk/lib/cmlib/hivedata.h`.
//!
//! This module exists because the translation grind open-coded these as bare
//! literals in eight places across five files and got FOUR of them wrong, in a
//! way no compiler and no existing scanner could see: a literal has no name for
//! `verify_constants` to check, and a *single* wrong private `const` has nothing
//! for `scan_const_contradictions` to disagree with. The damage was mutual —
//! `CmUnloadKey` set 0x40 while `CmpParseKey` tested 0x4 and
//! `CmpCreateKeyControlBlock` tested 0x200 — so the C's "don't touch a hive that
//! is unloading" guard could not fire on the unload path it exists for.
//!
//! Import these. Do not re-declare them, and do not inline the value: an
//! open-coded bit is how this family broke twice (cmlib-rust's hiveinit.rs had
//! `HIVE_NOLAZYFLUSH = 0x10000` before this).

/// The hive lives in memory only and is never written to disk.
pub const HIVE_VOLATILE: u32 = 1;
/// The lazy flusher must skip this hive; only explicit flushes reach the disk.
pub const HIVE_NOLAZYFLUSH: u32 = 2;
pub const HIVE_HAS_BEEN_REPLACED: u32 = 4;
pub const HIVE_HAS_BEEN_FREED: u32 = 8;
pub const HIVE_UNKNOWN: u32 = 0x10;
/// Load or unload in progress — every path that resolves a key in this hive must
/// refuse unless it is the thread doing the (un)loading (`CMHIVE::CreatorOwner`).
pub const HIVE_IS_UNLOADING: u32 = 0x20;
