//! rtl-except — Rust translation of the per-arch rtl exception/unwind tail
//! (`sdk/lib/rtl/{amd64,i386,arm64}/{unwind,except,stubs,dynfntbl,thread}.c`).
//!
//! librtl builds with `-D_NTOSKRNL_` globally (upstream ReactOS quirk), so the
//! `#ifndef _NTOSKRNL_` VEH-dispatch blocks in the C are dead code in the only
//! variant ever built — the translation matches that variant exactly.
//!
//! ABI notes: NTAPI == stdcall on i386 (`extern "system"`); __C_specific_handler
//! is __cdecl (`extern "C"`). The asm trampolines (RtlpExecuteHandlerForException/
//! Unwind, RtlpRestoreContextInternal, RtlCaptureContext/RtlpCaptureContext) and
//! the per-consumer libsupp helpers remain external symbols.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

pub mod types;

#[cfg(target_arch = "x86_64")]
pub mod amd64;

#[cfg(target_arch = "x86")]
pub mod i386;

#[cfg(target_arch = "aarch64")]
pub mod arm64;
