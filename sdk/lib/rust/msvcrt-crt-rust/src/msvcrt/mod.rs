//! Whole-file translations of `dll/win32/msvcrt/*.c`, one module per C file.
//!
//! Distinct from the hand-assembled families in `lib.rs`: those delegate to
//! `cnt_rust::core_rs` and were written family-by-family, while these are
//! rustconv translations of a WHOLE C file, kept 1:1 with their source so the
//! next re-translation lands as a readable diff.
//!
//! The cohab rule is unchanged — every `#[no_mangle]` here wins over the C copy
//! in `msvcrt_static` by lld first-match. A function this module does NOT define
//! simply stays C, which is why a partial file is a safe state.

pub mod mbcs;
