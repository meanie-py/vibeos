//! rtl-diag: Rust translation of the ReactOS `sdk/lib/rtl/` diagnostics +
//! PE-image + exception/VEH cluster.
//!
//! One module per source C file:
//!   image      — RtlImageNtHeader / RtlImageDirectoryEntryToData / reloc
//!   assert     — RtlAssert
//!   appverifier— RtlApplicationVerifierStop
//!   trace      — RtlWalkFrameChain / stack-backtrace capture
//!   message    — RtlFindMessage / RtlFormatMessage
//!   debug      — DbgPrint / debug-string machinery
//!   dbgbuffer  — RtlCreateQueryDebugBuffer / RtlQueryProcessDebugInformation
//!   wine_debug — Wine debug-channel helpers (FIXME/ERR/WARN/__wine_dbg_*)
//!   exception  — RtlRaiseException / RtlRaiseStatus / dispatch
//!   vectoreh   — RtlAddVectoredExceptionHandler / vectored dispatch
//!
//! librtl is shared by ntoskrnl AND ntdll, so these feed both.
//! Semantics track ReactOS rtl, not textbook.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
// File-statics translate to `static mut`; access is from `unsafe` fns.
// rustconv-style transliteration keeps some explicit casts / dead locals.
#![allow(unused_unsafe, unused_parens, unused_variables, unused_assignments, dead_code)]

// Shared glue (libc externs, TRACE/FIXME macros, local FORCEINLINE gaps).
#[cfg(feature = "image")]
#[macro_use]
pub mod shim;

#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "cabi")]
pub mod assert;
#[cfg(feature = "cabi")]
pub mod appverifier;
#[cfg(feature = "cabi")]
pub mod trace;
#[cfg(feature = "cabi")]
pub mod message;
#[cfg(feature = "cabi")]
pub mod dbgbuffer;
#[cfg(feature = "cabi")]
pub mod exception;
#[cfg(feature = "cabi")]
pub mod vectoreh;
// debug/wine_debug: the 6 C-variadic entry points live in the C shim
// dbgprint_variadic.c; everything else (incl. the va_list workers) is Rust.
#[cfg(feature = "cabi")]
pub mod debug;
#[cfg(feature = "cabi")]
pub mod wine_debug;
