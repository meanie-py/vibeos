//! Rust translation of `dll/ntdll/nt_0600/DllMain.c`.
//!
//! Entry point of the ntdll_vista.dll companion module. On
//! DLL_PROCESS_ATTACH it disables thread callouts, initializes the
//! keyed-event substrate and the locale table; on DLL_PROCESS_DETACH it
//! tears the keyed event back down. This `#[no_mangle]` `DllMain` IS the
//! module ENTRYPOINT (`set_module_type ntdll_vista win32dll ENTRYPOINT
//! DllMain 12`); on i386 `extern "system"` yields the required
//! `_DllMain@12` decoration.

use core::ffi::c_void;
use ntdll_sys::nt::NTSTATUS;

// winnt.h DLL_* reason codes.
const DLL_PROCESS_DETACH: u32 = 0;
const DLL_PROCESS_ATTACH: u32 = 1;

// winnt.h BOOL.
const FALSE: i32 = 0;
const TRUE: i32 = 1;

/// `NT_SUCCESS(Status)` — non-negative NTSTATUS. NOT `Status < 0`-style
/// sign tricks; the macro is `>= 0`.
#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

extern "system" {
    // ndk/umfuncs.h — ntdll export. Resolved via the ntdll import lib
    // (add_importlibs(ntdll_vista ntdll)). The main `ldr` module is NOT compiled
    // into the `vista` build (see lib.rs), so there is no static loader def in
    // ntdll_vista's staticlib to shadow this import — it binds to ntdll.dll's
    // export and the main loader is never dragged in.
    fn LdrDisableThreadCalloutsForDll(base_address: *mut c_void) -> NTSTATUS;
    // ntdllp.h — ntdll-private (rtl/libsupp.c). Both return VOID/NTAPI.
    fn RtlpInitializeKeyedEvent();
    fn RtlpCloseKeyedEvent();
    // Forward-declared in DllMain.c; defined in the locale support code.
    fn RtlpInitializeLocaleTable() -> NTSTATUS;
    // rtl-threadpool, linked in through rtl-vista-rust: this module's OWN copy of the NT
    // threadpool (the Tp* exports), distinct from ntdll's, so it needs its own init.
    fn RtlpInitializeThreadPooling();
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    h_dll: *mut c_void,
    dw_reason: u32,
    _lp_reserved: *mut c_void,
) -> i32 {
    if dw_reason == DLL_PROCESS_ATTACH {
        LdrDisableThreadCalloutsForDll(h_dll);
        RtlpInitializeKeyedEvent();
        let status = RtlpInitializeLocaleTable();
        if !nt_success(status) {
            RtlpCloseKeyedEvent();
            return FALSE;
        }
        // Divergence from DllMain.c, which never makes this call: upstream links threadpool.c
        // into rtl_vista too, but only ntdll's LdrpInitializeProcess runs the init, so this
        // copy's timer/wait/io queue critical sections and pending-timer list stay zeroed and
        // the first CreateThreadpoolTimer/Wait would deadlock or fault. Work objects don't
        // touch those queues, which is why it goes unnoticed.
        RtlpInitializeThreadPooling();
    } else if dw_reason == DLL_PROCESS_DETACH {
        RtlpCloseKeyedEvent();
    }
    TRUE
}
