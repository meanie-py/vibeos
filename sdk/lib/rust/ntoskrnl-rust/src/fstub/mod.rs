//! fstub subsystem — translated to Rust.
//!
//! Status: full Rust. Smoke test on i386 hits STATUS_OBJECT_PATH_NOT_FOUND
//! in SMSS opening `\KnownDlls` — semantic regression in the auto-translated
//! disksup or fstubex (likely partition-read/drive-letter logic). Build is
//! intentionally left in this broken state so the smoke harness shows the
//! regression as a diagnostic signal — debug, don't hide.

pub mod disksup;
pub mod fstubex;
pub mod halstub;
pub mod translate;
