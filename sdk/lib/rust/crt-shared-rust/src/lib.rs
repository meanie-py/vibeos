//! crt-shared-rust — the Rust twin of cmake's `msvcrt_shared` static library.
//!
//! These are the TUs `dll/win32/msvcrt` compiles TWICE: once into msvcrt.dll's own body,
//! and once with `-D__UCRTSUPPORT__` into ucrtbase.dll, which is otherwise vendored
//! Microsoft UCRT (`sdk/lib/ucrt`, see vendor.yaml). Everything here therefore has to be
//! correct under BOTH CRTs' contracts, which is the standard this crate is held to and the
//! reason it is separated from msvcrt's own floor at all — see Cargo.toml for the 856-body
//! shadowing measurement that forced the split.
//!
//! Two rules follow from being linked into two different CRTs:
//!
//! * **Call your host's CRT, never msvcrt's specifically.** A sibling like `atoi` is an
//!   `extern "C"` here, so it binds to msvcrt-crt-rust's Rust body in msvcrt.dll and to the
//!   genuine UCRT's in ucrtbase.dll. A `use` would have hard-wired one of them into both.
//! * **The live set is a cfg, not an assumption.** `ucrtsupport` swaps bodies in and out in
//!   both directions; Cargo.toml lists exactly which, and why the OUT half is the dangerous
//!   one.
#![no_std]
#![no_builtins]
#![allow(clippy::missing_safety_doc)]

/// The single import surface these translations compile against — transcribed from the
/// translated TU's own include closure, never inferred from a call site. Lives here rather
/// than in msvcrt-crt-rust because this is the crate that cannot assume which CRT it is
/// inside; msvcrt-crt-rust re-exports it, so `crate::prelude::*` still resolves there.
// Wine's ERR/FIXME/WARN/TRACE for the whole crate; `#[macro_use]` must precede the
// modules that call them (macro_rules! is textually scoped).
#[macro_use]
mod winedebug;

pub mod prelude;

// undname.c: msvcrt's name demangler, and the most self-contained file in msvcrt. No locale,
// no fd table, no Win32 beyond the caller-supplied malloc/free in `parsed_symbol`, and no
// global state — a `parsed_symbol` carries its own arena (`und_alloc` bump-allocates and
// never frees) and its own three `struct array` stacks. The whole thing is a recursive-descent
// parser over a mangled name.
//
// `str_printf` is HAND-WRITTEN and is the only one; the reason it could not be a C shim like
// file.c's and process.c's twenty is written out at its definition.
#[cfg(feature = "undname")]
pub mod undname;

// cpp.c: the C++ RTTI / exception-object subsystem — `type_info`, the `bad_cast`/`bad_typeid`/
// `exception` object families, `__RTDynamicCast` and friends.
//
// The bodies that only exist at `_MSVCR_VER >= 80` / `>= 140` (the four `__std_type_info_*`,
// `_is_exception_typeof`, …) are behind `ucrtsupport`: msvcrt compiles at `_MSVCR_VER = 0`
// without `__UCRTSUPPORT__`, so nothing there builds them, and until the ucrtbase flip they
// had no Rust body at all.
#[cfg(feature = "cpp")]
pub mod cpp;
#[cfg(feature = "cpp")]
pub mod cpp_rtti;

// except.c: the exception DISPATCHER — the C signal layer (`signal`/`raise` and the SEH filter
// that turns a hardware fault into a C signal) plus the C++ frame handler that walks a
// function's try blocks looking for a catch.
//
// Rides the `cpp` feature with cpp.c because it is not separable from it: the catch-matching
// path calls `find_caught_type` and `copy_exception`, cppexcept.h inlines that live in
// `cpp_rtti`, and both are equally unlinkable in an ELF host binary.
//
// The arch split is the thing to know before editing: everything from `ip_to_state` to
// `__CxxFrameHandler` is `#[cfg(not(target_arch = "x86"))]` because it is inside
// `#ifndef __i386__` in the C, where i386 gets a DIFFERENT implementation from except_i386.c.
// Cohab resolves by first-match, so an ungated body there would replace except_i386.c's
// 32-bit SEH walk with the RVA one and link without complaint. `dump_function_descr` and
// `find_catch_handler` sit above that gate and are called BY those still-C arch files.
#[cfg(feature = "cpp")]
pub mod except;

// except_i386.c: the OTHER side of that gate — 32-bit SEH, where an exception frame is a
// linked list threaded through `fs:0` rather than a table the unwinder looks up by PC. The
// `#ifdef __i386__` around the whole TU is why both modules are `#![cfg(target_arch = "x86")]`
// from the inside as well as gated here: on amd64/arm64 this file compiles to nothing at all,
// and `except.rs` supplies the same exported names from the other implementation.
//
// `seh_i386` is its hand-written companion, the same relationship `cpp_rtti` has to `cpp`: six
// `__ASM_GLOBAL_FUNC` thunks that run on somebody else's frame, and wine/exception.h's two
// `static inline` frame-list helpers. None of them has C text a translator could read.
//
// Seven bodies of the C are NOT here, on purpose: `_global_unwind2`, `_except_handler2/3`,
// `_local_unwind2`, `longjmp` and the two `__regs__setjmp*` are all inside `#ifndef
// __REACTOS__` and are in no object this tree builds.
#[cfg(all(feature = "cpp", target_arch = "x86"))]
pub mod except_i386;

// ucrtbase's thread_data_t slot (was dll/win32/msvcrt/reactos/ucrt_tls_sup.c).
#[cfg(all(feature = "ucrtsupport", target_os = "windows"))]
mod ucrt_tls;

// msvcrt_shared's i386 .s files (RtlUnwind wrapper, exception_ptr thunks, port I/O).
#[cfg(all(target_arch = "x86", target_os = "windows"))]
mod asm_i386;
// crtmath's i386 entry points (_ftol*, _CI*, __libm_sse2_*), for every CRT DLL.
#[cfg(all(target_arch = "x86", target_os = "windows"))]
mod crtmath_i386;
#[cfg(all(feature = "cpp", target_arch = "x86"))]
pub mod seh_i386;

// The other two arms of the same gate, and the reason `except.rs`'s `unsafe extern "C"` block
// declaring `call_catch_handler`/`call_unwind_handler`/`get_exception_pc`/`handle_fpieee_flt`
// is now satisfied by Rust on every arch rather than by a surviving C sibling. Each is tiny —
// four bodies and one `__ASM_GLOBAL_FUNC` — because that is genuinely all the 64-bit
// dispatcher cannot say portably. Both carry their asm inline instead of a `seh_*` companion;
// one thunk does not earn a module.
#[cfg(all(feature = "cpp", target_arch = "x86_64"))]
pub mod except_x86_64;
#[cfg(all(feature = "cpp", target_arch = "aarch64"))]
pub mod except_arm64;
