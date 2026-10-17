//! main.c — psapi.dll's Win2k3-style entry point, hand-ported ($0).
//!
//! Built only with the `dllmain` feature, which psapi.dll's own build enables: kernel32 and
//! kernelbase link psapi-rust too (through psapi_static), and must not get a second DllMain.
//! main.c defines NDEBUG, so `UNIMPLEMENTED` is empty there as it is here.
use crate::prelude::*;
use ntdll_sys::nt::{NtCurrentPeb, RTL_USER_PROCESS_PARAMETERS_PROFILE_USER};

extern "system" {
    fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
}

// winnt.h
const DLL_PROCESS_DETACH: DWORD = 0;
const DLL_PROCESS_ATTACH: DWORD = 1;

unsafe extern "system" fn PsParseCommandLine() {
    UNIMPLEMENTED!();
}

unsafe extern "system" fn PsInitializeAndStartProfile() {
    UNIMPLEMENTED!();
}

unsafe extern "system" fn PsStopAndAnalyzeProfile() {
    UNIMPLEMENTED!();
}

unsafe fn profiling_requested() -> bool {
    (*(*NtCurrentPeb()).ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_PROFILE_USER != 0
}

/// `BOOLEAN WINAPI DllMain(...)`: the loader's PDLL_INIT_ROUTINE returns BOOLEAN too.
#[no_mangle]
pub unsafe extern "system" fn DllMain(hDllHandle: HINSTANCE, nReason: DWORD, Reserved: LPVOID) -> BOOLEAN {
    match nReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hDllHandle as HMODULE);
            if profiling_requested() {
                PsParseCommandLine();
                PsInitializeAndStartProfile();
            }
        }
        DLL_PROCESS_DETACH => {
            if profiling_requested() {
                PsStopAndAnalyzeProfile();
            }
        }
        _ => {}
    }
    1 // TRUE
}
