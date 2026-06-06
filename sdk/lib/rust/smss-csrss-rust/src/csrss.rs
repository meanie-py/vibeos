//! csrss.exe `_main` — Rust translation of csrss.c (102 LOC).
//!
//! rustconv translate (sonnet, 2026-05-16) AFTER the P1a pipeline
//! fixes (user-mode FFI routing + system_prompt_user.md addendum +
//! ntdll-sys NtCurrent* inlines + collapse_sys_crate + the
//! strip_user_inline_redeclarations net). Clean: single
//! `use ntdll_sys::nt::*;`, no hand-rolled extern accessors, no fake
//! PEB struct — uses the genuine `_PEB`/`NtCurrent*` from ntdll-sys.
//! Body diffed faithful vs csrss.c (no stubbed bodies);
//! STATUS_IO_PRIVILEGE_FAILED inlined 0xC0000137 (accepted
//! cast-macro-const class). i386 `_main` cdecl decoration verified
//! vs the `nt` native CRT stub. See
//! docs/engineering/smss-csrss-translation-audit.md.

use ntdll_sys::nt::*;

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
    let BasePriority: KPRIORITY = (PROCESS_PRIORITY_NORMAL_FOREGROUND + 4) as KPRIORITY;
    let mut Status: NTSTATUS;
    #[cfg(target_arch = "x86")]
    let mut Response: u32 = 0;

    crate::dbg_print!(b"[CSRSS] _main entered (argc=%d)\n\0", argc);

    NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessBasePriority,
        &BasePriority as *const KPRIORITY as *mut core::ffi::c_void,
        core::mem::size_of::<KPRIORITY>() as u32,
    );

    #[cfg(target_arch = "x86")]
    {
        Status = NtSetInformationProcess(
            NtCurrentProcess(),
            ProcessUserModeIOPL,
            core::ptr::null_mut(),
            0,
        );
        if Status < 0 {
            crate::dbg_print!(b"CSRSS: Could not raise IOPL, Status: 0x%08lx\n\0", Status,);
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

    crate::dbg_print!(b"CSRSS: Calling CsrServerInitialization\n\0");
    Status = CsrServerInitialization(argc as u32, argv);
    crate::dbg_print!(b"CSRSS: CsrServerInitialization returned 0x%lx\n\0", Status,);
    if Status < 0 {
        crate::dbg_print!(
            b"CSRSS: Unable to initialize server, Status: 0x%08lx\n\0",
            Status,
        );
        NtTerminateProcess(NtCurrentProcess(), Status);
    }

    CsrpSetDefaultProcessHardErrorMode();

    let peb = NtCurrentPeb() as *mut _PEB;
    if (*peb).SessionId == 0 {
        RtlSetProcessIsCritical(1u8, core::ptr::null_mut(), 0u8);
    }

    NtTerminateThread(NtCurrentThread(), Status);
    0
}
