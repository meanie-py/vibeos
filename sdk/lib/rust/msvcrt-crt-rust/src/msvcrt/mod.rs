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

// file.c: 189/189 exports, LIVE. This `mod` line is the flip itself — the Rust archive
// links ahead of msvcrt_static, so first-match hands every symbol here to Rust and the C
// bodies go dead in place. It went in as ONE change, never partially, because file.c's
// entry points share one fd table and one FILE ring; the general "an undefined function
// just stays C" property of cohab is unsafe for this particular file. The 20 C-variadic
// entry points are in dll/win32/msvcrt/rust_file_varargs_shim.c as va_start trampolines
// (pinned stable: no `#![feature(c_variadic)]`), and delegate straight back here.
//
// The `stdio` feature is DEFAULT-ON and is off for exactly one consumer, the host harness
// msvcrt-difftest, whose scanf suite owns the plain `fgetc`/`ungetc`/… names as its
// character-source model. Every build that produces msvcrt.dll has it on; see this crate's
// Cargo.toml for the full reason and what is owed to remove it.
#[cfg(feature = "stdio")]
pub mod file;
// errno.c: 18/18 exports at `_MSVCR_VER=0` (the `>= 80` and `>= 140` blocks are not compiled,
// so `__sys_nerr`, `__sys_errlist`, the `_invalid_parameter_noinfo` pair and the thread-local
// handler accessors do not exist here). This file owns the Win32-error -> errno mapping that
// file.c reaches from ~60 call sites, which is why it is a whole-file flip rather than a
// partial: `msvcrt_set_errno` and the `_errno`/`__doserrno` storage are one contract.
pub mod errno;
pub mod mbcs;
// time.c: 52 of 52 exports at `_MSVCR_VER=0`. Whole-file, like errno.c and for the same
// reason: `_tzset`'s four exported globals (`MSVCRT___timezone`, `___daylight`, `__dstbias`,
// `__tzname`) and the file-static `tzi` it fills are ONE contract, read by `is_dst`,
// `mktime_helper` and `_ftime64` alike. A partial flip would leave some readers on the C's
// copy of that state and some on Rust's, which is not a subset of either behaviour.
//
// Seven names that LOOK like exports of this file are not compiled at this `_MSVCR_VER`:
// `_get_dstbias`/`_get_timezone`/`_get_daylight` are `#if >= 80`, `_Wcsftime` is `>= 110`,
// and `_timespec32_get`/`_timespec64_get`/`_strftime_l` are `>= 140`. The object is the
// authority on that, not the source's apparent shape.
pub mod time;
