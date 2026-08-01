//! The sibling's build script, verbatim — ONE reference-building machine, two
//! consumers.
//!
//! Everything this crate needs (the six reference TUs, the rename walls, refimpl /
//! subjectimpl / fileimpl, the generated headers) is identical to msvcrt-difftest's,
//! so it is included rather than copied: a second copy of 1,900 lines would drift, and
//! the thing that would drift is the definition of the ORACLE.
//!
//! The two crates differ only in which features of msvcrt-crt-rust they pull; see
//! Cargo.toml. The included script reads `CARGO_PKG_NAME` to tell them apart and
//! panics on an unknown one rather than guessing.
//!
//! Paths inside resolve two ways, deliberately: SOURCES are rooted at
//! msvcrt-difftest (shared), while everything generated hangs off OUT_DIR (per-crate).
//! `include!` is relative to THIS file, so the path below is the only coupling.
include!("../msvcrt-difftest/build.rs");
