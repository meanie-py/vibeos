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
