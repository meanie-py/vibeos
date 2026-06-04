//! lsass.c — wWinMain: make the process critical, initialize the LSA server
//! DLL (lsasrv) and the SAM server DLL (samsrv), start the security services,
//! then terminate. Faithful translation of the original `goto ByeBye` flow:
//! every failure (and the success fall-through) lands on the shared
//! NtTerminateThread epilogue with the appropriate NTSTATUS.

use core::ffi::c_void;
use ntdll_sys::nt::*;

// lsasrv.dll / samsrv.dll exports. All three are `NTSTATUS WINAPI/NTAPI(VOID)`
// → stdcall on i386 (`extern "system"`), cdecl on amd64/arm64. They resolve
// against the lsasrv / samsrv import libs (see add_importlibs in
// base/system/lsass/CMakeLists.txt), exactly as the C `WINAPI`/`NTAPI`
// prototypes did.
extern "system" {
    fn LsapInitLsa() -> NTSTATUS;    // lsasrv.dll
    fn ServiceInit() -> NTSTATUS;    // lsasrv.dll
    fn SamIInitialize() -> NTSTATUS; // samsrv.dll
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    _hInstance: *mut c_void,
    _hPrevInstance: *mut c_void,
    _lpCmdLine: *mut u16,
    _nShowCmd: i32,
) -> i32 {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // DPRINT("Local Security Authority Subsystem\n");
    // DPRINT("  Initializing...\n");

    // Make us critical (TRUE, NULL, TRUE).
    RtlSetProcessIsCritical(1u8, core::ptr::null_mut(), 1u8);

    // The C used `goto ByeBye` on each failure; the success path falls through.
    // A `loop { ...; break; }` reproduces that exactly: every path breaks out
    // and hits the single NtTerminateThread epilogue with the right Status.
    loop {
        // Initialize the LSA server DLL.
        Status = LsapInitLsa();
        if Status < 0 {
            break; // DPRINT1("LsapInitLsa() failed ...")
        }

        // Initialize the SAM server DLL.
        Status = SamIInitialize();
        if Status < 0 {
            break; // DPRINT1("SamIInitialize() failed ...")
        }

        // Start the security services.
        Status = ServiceInit();
        if Status < 0 {
            break; // DPRINT1("ServiceInit() failed ...")
        }

        // FIXME: More initialization
        // DPRINT("  Done...\n");
        break;
    }

    // ByeBye:
    NtTerminateThread(NtCurrentThread(), Status);

    0
}
