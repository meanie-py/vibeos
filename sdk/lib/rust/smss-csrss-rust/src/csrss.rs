//! csrss.exe `_main` — SMSS→CSRSS spike translation (csrss.c, 102 LOC).
//!
//! ⚠ RAW rustconv spike output (sonnet, 2026-05-16, ~$0.03). Preserved
//! per the translation philosophy (never discard paid .rs; broken-in-
//! tree with the failure visible is the correct state). This module is
//! NOT yet in lib.rs / NOT linked into any cmake target — it is the
//! spike artifact that flushed the systematic Phase-1 pipeline gaps
//! below. DO NOT hand-patch these away one file at a time; they are
//! "fix once for all ~60 files" pipeline issues (the win32k gdiobj.c
//! cascade pattern).
//!
//! SYSTEMATIC GAPS FLUSHED BY THIS SPIKE:
//!
//! 1. WRONG FFI CRATE. rustconv emitted `use ntkrnl_sys::{ddk,base,
//!    constants,globals}::*` — the *kernel* crate. User-mode targets
//!    must resolve to `ntdll_sys::nt::*`. The base system_prompt.md is
//!    kernel-oriented; smss-csrss needs a user-mode prompt (or a
//!    postprocess crate-path rewrite keyed on target == "smss-csrss",
//!    mirroring win32k's enforce_win32k_abi). PIPELINE FIX.
//!
//! 2. FORCEINLINE NtCurrent* MIS-TRANSLATED. rustconv declared
//!    `extern "system" { fn NtCurrentPeb/Process/Thread }` + a bogus
//!    hand-rolled `struct PEB { _padding:[u8;0x1d4], SessionId }`.
//!    These are FORCEINLINE TEB/PEB-self reads, NOT ntdll exports —
//!    declaring them extern WILL NOT LINK, and the hardcoded 0x1d4
//!    PEB offset is arch-wrong. Needs a hand-written ntdll-sys inline
//!    module (the user-mode analog of ke-inlines): NtCurrentPeb/Teb/
//!    Process/Thread as real Rust inlines + the genuine _PEB/_TEB from
//!    ntdll-sys bindgen. PIPELINE FIX (ntdll-sys substrate addition).
//!
//! 3. i386 `_main` ENTRYPOINT DECORATION unverified. C `_cdecl _main`
//!    is called by the `nt` native CRT stub; the Rust `#[no_mangle]
//!    extern "C" fn _main` symbol decoration on i686-pc-windows-gnu
//!    must match exactly. Validate via `nm` before the cohabitation
//!    flip ([[feedback_i386_pe_decoration]]).
//!
//! Otherwise the body transliteration is faithful (diffed vs csrss.c:
//! priority set, x86 IOPL+hard-error, CsrServerInitialization, session-0
//! critical, terminate-thread — all present, no stubbed bodies).
//! STATUS_IO_PRIVILEGE_FAILED correctly inlined as 0xC0000137 (the
//! accepted cast-macro-const class).

// ---- RAW SPIKE OUTPUT BELOW (unmodified; does not compile until the
// ---- pipeline fixes above land) -------------------------------------

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    CsrServerInitialization, DbgPrint, KPRIORITY, NtRaiseHardError, NtSetInformationProcess,
    NtTerminateProcess, NtTerminateThread, OptionOk, PROCESS_PRIORITY_NORMAL_FOREGROUND,
    ProcessBasePriority, ProcessDefaultHardErrorMode, ProcessUserModeIOPL,
    RtlSetProcessIsCritical,
};

extern "system" {
    fn NtCurrentPeb() -> *mut core::ffi::c_void;
    fn NtCurrentProcess() -> *mut core::ffi::c_void;
    fn NtCurrentThread() -> *mut core::ffi::c_void;
}

#[repr(C)]
struct PEB {
    _padding: [u8; 0x1d4],
    SessionId: u32,
}

unsafe fn CsrpSetDefaultProcessHardErrorMode() {
    let mut DefaultHardErrorMode: u32 = 0;

    NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessDefaultHardErrorMode,
        &mut DefaultHardErrorMode as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _main(
    argc: i32,
    argv: *mut *mut i8,
    _envp: *mut *mut i8,
    _DebugFlag: i32,
) -> i32 {
    let mut BasePriority: KPRIORITY = (PROCESS_PRIORITY_NORMAL_FOREGROUND + 4) as KPRIORITY;
    let mut Status: i32;

    DbgPrint(b"[CSRSS] _main entered (argc=%d)\n\0".as_ptr() as *const i8, argc);

    NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessBasePriority,
        &mut BasePriority as *mut KPRIORITY as *mut core::ffi::c_void,
        core::mem::size_of::<KPRIORITY>() as u32,
    );

    #[cfg(target_arch = "x86")]
    {
        let mut Response: u32 = 0;

        Status = NtSetInformationProcess(
            NtCurrentProcess(),
            ProcessUserModeIOPL,
            core::ptr::null_mut(),
            0,
        );
        if Status < 0 {
            DbgPrint(
                b"CSRSS: Could not raise IOPL, Status: 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            Status = NtRaiseHardError(
                0xC0000137u32 as i32,
                0,
                0,
                core::ptr::null_mut(),
                OptionOk,
                &mut Response,
            );
        }
    }

    DbgPrint(b"CSRSS: Calling CsrServerInitialization\n\0".as_ptr() as *const i8);
    Status = CsrServerInitialization(argc, argv);
    DbgPrint(
        b"CSRSS: CsrServerInitialization returned 0x%lx\n\0".as_ptr() as *const i8,
        Status,
    );
    if Status < 0 {
        DbgPrint(
            b"CSRSS: Unable to initialize server, Status: 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        NtTerminateProcess(NtCurrentProcess(), Status);
    }

    CsrpSetDefaultProcessHardErrorMode();

    let peb = NtCurrentPeb() as *mut PEB;
    if (*peb).SessionId == 0 {
        RtlSetProcessIsCritical(1u8, core::ptr::null_mut(), 0u8);
    }

    NtTerminateThread(NtCurrentThread(), Status);
    0
}
