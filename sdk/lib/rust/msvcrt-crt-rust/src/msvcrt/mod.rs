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

// math.c. Note what the C build does to this file before you diff it: CMakeLists.txt defines
// `-Dmath_error=_invoke_matherr`, `-Dasm_sqrt=sqrt`, `-Dasm_sqrtf=sqrtf` and `-Dx87_asin=asin`,
// so three of the hand-written assembly stubs are DEFINED-BUT-DEAD (their call sites are
// rewritten to libm) and `math_error` ships under a name it is never spelled by. The object's
// symbol table is the authority on what this file exports, not the source's apparent shape —
// reading it that way is what caught `MSVCRT__rotl64`/`MSVCRT__rotr64` translated under a bare
// `_rotl64` that would have left the C copies live.
//
// Nine symbols here are owned by cnt-rust's difftest-pinned `cabi/math.rs`, which lands in the
// SAME archive, and two strong definitions in one archive resolve by member order. Those are
// `#[cfg(any())]` with the reason at each site, as are the `#ifndef __REACTOS__` blocks (the
// 22 `__libm_sse2_*` wrappers, `_fpclass`, `ldiv`) and the `_MSVCR_VER`-gated `fenv_encode` /
// `fenv_decode`. The i386 `_CI*`/`_ftol`/`_adj_*` stubs ARE exports, so they are defined here
// in `global_asm!` — leaving them out would drag math.c.obj back in and duplicate everything.
pub mod math;
// process.c: all 50 functions. `msvcrt_spawn` and `_wpopen` are the two the paid translation
// could not get past the deterministic call-envelope audit across nine runs; they are
// hand-written. `system`/`_wsystem` live inside `#ifndef __REACTOS__` upstream and are
// `#[cfg(any())]` here for that reason — but NOT `msvcrt_get_comspec`, which is only absent
// from the object because the compiler inlined it into its sole caller.
//
// The sixteen `_exec*`/`_spawn*` WINAPIV entry points are `va_start` trampolines in
// dll/win32/msvcrt/rust_process_varargs_shim.c over `_v`-prefixed Rust twins here, for the
// same pinned-stable reason as file.c's twenty: receiving a `va_list` is fine, starting one
// is not. The four `_execle`-family twins walk that one snapshot TWICE, which is sound only
// because `va_list` is a flat pointer on every Windows target we build and
// `msvcrt_valisttos` takes it by value — the reasoning is written out at the twins.
//
// Behind the `stdio` feature for the same reason file.c is: `_popen`/`_wpopen`/`_pclose`
// stand on file.c's fd table and FILE ring, so this module cannot compile without it. The
// scanf difftest turns `stdio` off to own the character-source names itself, and an
// ungated `pub mod process;` broke that harness with an unresolved `crate::msvcrt::file`.
#[cfg(feature = "stdio")]
pub mod process;
