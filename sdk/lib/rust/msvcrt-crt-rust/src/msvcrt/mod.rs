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

// environ.c (11 of 11 exports) and dir.c (48 of 48), both whole-file.
//
// environ.c is whole-file for the same reason errno.c and time.c are: the four environment
// blocks (`MSVCRT__environ`/`__winitenv`/`_wenviron`/`__initenv`) are ONE contract, built by
// `env_init` and rewritten in place by every `_putenv`/`_wputenv`, so a partial flip would
// leave some readers on the C's copy of that state. Its eight file-static helpers
// (`env_get_index`, `env_set`, `putenv_helper`, …) stay private exactly as in the C, so the
// object's 11 GLOBAL symbols are the whole cohab obligation and all 11 are defined.
//
// dir.c is the directory/path surface — `_findfirst*`/`_findnext*` (all four time-width
// variants), `_chdir`/`_getcwd`/`_getdcwd`, `_mkdir`/`_rmdir`, the
// `_makepath`/`_splitpath`/`_fullpath` path algebra, and `_searchenv*`.
//
// Both carry the `dir-environ` gate, and NOT because either compiles against file.c's fd
// table — neither does. It is a DEFAULT feature that both host harnesses drop (they take this
// crate with `default-features = false`), because the harness defines two of these exports
// itself: `fileimpl.c` provides `_getdrive` as a constant 3, so file.c's `_fstat` yields a
// deterministic `st_dev`, and `_wgetenv` as a host-controlled fake. dir.c exports the real
// `_getdrive` and environ.c the real `_wgetenv`, so linking them into the harness is two
// duplicate symbols — which is how this was found, because `cargo check` never links there
// and cannot see it. It cannot ride on `stdio`, which msvcrt-difftest turns ON deliberately
// so that it can grade file.c.
//
// Neither real body could serve the harness anyway: `_getdrive` reads `GetCurrentDirectoryW`,
// which does not exist on the Linux host. The cost is that the difftest cannot grade these two
// files — the same limitation file.c and process.c already carry, recorded rather than worked
// around; this crate's Cargo.toml states what is owed to lift it.
#[cfg(feature = "dir-environ")]
pub mod environ;
#[cfg(feature = "dir-environ")]
pub mod dir;

// console.c (29 of 29 exports), misc.c (17 of 17) and exit.c (18 of 18), all whole-file.
//
// console.c owns the `_cputs`/`_getch`/`_cscanf`/`_cprintf` conio surface plus the two lazily
// opened CONIN$/CONOUT$ handles, and those handles are the reason it is whole-file: every
// entry point reaches the same pair through `msvcrt_input_console`/`msvcrt_output_console`.
//
// exit.c carries `_aexit_rtn` — an exported writable function pointer, NOT a file-static, which
// is recorded at its definition because the translation got that wrong and only the object's
// symbol table said so.
//
// All three needed their C `#define LOCK_x _lock(N)` spelled as a `macro_rules!` rather than a
// helper `fn`, per the standing rule about out-of-line no-ops uncloaking ABI mismatches. Doing
// that is also what exposed exit.c's `_EXIT_LOCK1` arriving as 8 (`_CONIO_LOCK`) instead of 13.
#[cfg(feature = "conio-exit")]
pub mod console;
#[cfg(feature = "conio-exit")]
pub mod exit;
pub mod misc;

// locale.c: 29 exports plus the eight globals it OWNS — `MSVCRT_locale`, `MSVCRT__pctype`,
// `MSVCRT___lc_codepage`, `MSVCRT___lc_collate_cp`, `MSVCRT___lc_handle`,
// `MSVCRT___mb_cur_max`, `initial_locale` and `cloc_time_data`. Whole-file for the reason
// errno.c, time.c and environ.c are: those globals plus the `cloc_static_locinfo`/
// `cloc_time_data` C-locale singletons are ONE contract, published by `update_thread_locale`
// and read by every other file in the CRT through `get_locinfo`/`get_mbcinfo`. Half of them
// C and half Rust is not a subset of either behaviour.
//
// This is the file the rest of msvcrt leans on hardest: mbcs.c, time.c, string.c and the
// printf engine all reach `get_locinfo()->pctype` and the case-mapping tables, which is why
// `msvcrt-sys` derives `threadlocaleinfostruct`'s layout rather than transcribing it.
//
// Behind the `locale` DEFAULT feature, which the host harness drops: it defines eight of
// these symbols itself as its NLS oracle (see this crate's Cargo.toml). Every build that
// produces msvcrt.dll has the feature on.
#[cfg(feature = "locale")]
pub mod locale;

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
