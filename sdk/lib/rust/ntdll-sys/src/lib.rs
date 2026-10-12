//! ntdll-sys — FFI declarations for the user-mode Native API + the
//! SM/CSR/BASE subsystem protocol.
//!
//! Auto-generated via `bindgen` for all 3 architectures from a wrapper
//! that pulls `<ndk/ntndk.h>` (the NT-internal umbrella — `Nt*`/`Zw*`/
//! `Rtl*`/`Ldr*`/`Dbg*` + um/ke/mm/ob/ps/se) plus the subsystem
//! protocol headers `<sm/ns.h>`, `<sm/smmsg.h>`, `<csr/csr.h>`,
//! `<csr/csrmsg.h>`, `<csr/csrsrv.h>`, `<win/base.h>`,
//! `<win/basemsg.h>`.
//!
//! This is the user-mode analog of `ntkrnl-sys`/`win32k-sys`, for the
//! SMSS→CSRSS translation track (smss.exe, csrss.exe, csrsrv.dll,
//! basesrv.dll, winsrv.dll). It exists because `ntkrnl-sys` bindgens
//! the *kernel-internal* surface only — without this crate every
//! `CSR_API_MESSAGE`/`SM_API_MSG`/`BASESRV_API_MSG` layout and every
//! `Csr*`/`Sm*`/`Rtl*` call would be invented per-file by the
//! translator, inconsistently. See
//! `docs/engineering/smss-csrss-translation-audit.md` §Substrate.
//!
//! Types use bindgen naming with the Windows-style aliases bindgen
//! also emits. NOT a standalone static library — an `rlib` declaration
//! dependency.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
// The bindgen also re-declares some NT types via the ndk pulled by the
// subsys headers. Within this crate those are self-consistent.
#![allow(clashing_extern_declarations)]

mod bindgen_common;
mod constants;
mod inlines;
pub mod seh;

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;

#[cfg(target_arch = "x86")]
mod bindgen_i386;

// Compile-time struct-size assertions (verify_sizes.py) — fail the
// build if a bindgen struct ever drifts from the C layout.
#[cfg(target_arch = "aarch64")]
mod size_check_arm64;

#[cfg(target_arch = "x86_64")]
mod size_check_amd64;

#[cfg(target_arch = "x86")]
mod size_check_i386;

/// User-mode Native API + SM/CSR/BASE subsystem protocol — auto-
/// generated bindings with correct per-arch calling conventions
/// (i386 stdcall/fastcall decoration too).
pub mod nt {
    pub use super::bindgen_common::*;

    #[cfg(target_arch = "aarch64")]
    pub use super::bindgen_arm64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::bindgen_amd64::*;

    #[cfg(target_arch = "x86")]
    pub use super::bindgen_i386::*;

    // Hand-written FORCEINLINE user-mode accessors (NtCurrentPeb/Teb/
    // Process/Thread) — not in bindgen (they're macros). EXPLICIT
    // named re-export, NOT a glob: bindgen also emits an (unusable
    // extern) `NtCurrentTeb` decl, and two globs exporting the same
    // name is an ambiguous-glob-reexport (the name becomes
    // unresolvable — "cannot find NtCurrentTeb"). An explicit `pub
    // use` shadows the bindgen glob, so the real inline wins.
    pub use super::inlines::{
        NtCurrentPeb, NtCurrentProcess, NtCurrentTeb, NtCurrentThread,
        ZwCurrentProcess, ZwCurrentThread,
        // Universal LIST_ENTRY inlines (also macros, not in bindgen —
        // safe to name explicitly; same reason as NtCurrent*).
        InitializeListHead, IsListEmpty, InsertHeadList, InsertTailList,
        RemoveEntryList, RemoveHeadList, RemoveTailList,
        // ndk/rtlfuncs.h FORCEINLINE: bindgen's extern for it links against nothing.
        RtlInitEmptyUnicodeString,
    };

    // Hand-written NTSTATUS codes (ntstatus.h cast-macros are not
    // bindgen-evaluable). STATUS_* names don't exist in bindgen so
    // this glob can't clash with it.
    pub use super::constants::*;
}
