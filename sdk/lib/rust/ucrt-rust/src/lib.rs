//! ucrt-rust — our Rust bodies for ucrtbase.dll's own surface, replacing the
//! vendored Microsoft UCRT in `sdk/lib/ucrt` one family at a time.
//!
//! Every symbol here deliberately displaces a genuine Microsoft UCRT body, so
//! every symbol here must be declared in `tools/ucrt-takeover.yaml` with the
//! oracle that grades it. See Cargo.toml for why that declaration is the whole
//! difference between this crate and the accident `crt-shared-rust` guards
//! against, and for why this crate must never depend on `msvcrt-crt-rust`.
//!
//! Progress: `python3 tools/ucrt_burndown.py --arch i386`.
#![no_std]

#[cfg(feature = "bits")]
pub mod bits;

#[cfg(feature = "ismbbyte")]
pub mod ismbbyte;

#[cfg(feature = "ctype")]
pub mod ctype;

#[cfg(feature = "ismbc")]
pub mod ismbc;

#[cfg(feature = "tocase")]
pub mod tocase;

#[cfg(feature = "strtox")]
pub mod strtox;

// Named, not just depended on: a staticlib drops a dependency nothing references,
// and its #[no_mangle] bodies never reach libcrt_shared_cargo.a.
#[cfg(feature = "portable-string")]
extern crate crt_portable_rust;

// Always compiled: it defines no symbols, only UCRT's locale layout and the
// compile-time assertions pinning it. Gating it would mean the pins only hold
// when some other family happens to be on.
pub mod locale;
pub mod locale_update;
pub mod ptd;

// Captured ground truth, not yet consumed by any body. Both are DATA ONLY --
// no `#[no_mangle]`, so they take over nothing and the cohab gate has nothing
// to say about them. They are here rather than in staging because they were
// GENERATED from public fact sources (the Unicode Character Database; published
// DBCS codepage definitions) and verified against Microsoft's tables while
// those still exist in `sdk/lib/ucrt`. After those files are deleted there is
// nothing left in the tree to check them against, so landing them is what makes
// the verification durable instead of a thing that happened once.
pub mod ctype_table;
pub mod mbctype_table;
