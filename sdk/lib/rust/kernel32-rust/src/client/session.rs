use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn DosPathToSessionPathW(
    SessionID: DWORD,
    InPath: LPWSTR,
    OutPath: *mut LPWSTR,
) -> DWORD {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"DosPathToSessionPathW\0" as *const u8 as *const ::core::ffi::c_char,
        b"dll/win32/kernel32/client/session.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        27 as ::core::ffi::c_int,
    );
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn DosPathToSessionPathA(
    SessionId_param: DWORD,
    InPath: LPSTR,
    OutPath: *mut LPSTR,
) -> DWORD {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"DosPathToSessionPathA\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/client/session.c\0"
            .as_ptr() as *const i8,
        40i32,
    );
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn WTSGetActiveConsoleSessionId() -> DWORD {
    // SharedUserData->ActiveConsoleId: KUSER_SHARED_DATA fixed VA 0x7FFE0000,
    // ActiveConsoleId at offset 0x2D8 (verified, sdk/include/ndk/ketypes.h:592;
    // arch-independent layout). Read at the verified offset rather than mirroring
    // the struct (the "hand offset" hazard) — matches k32_externs SharedUserData_*.
    return core::ptr::read_volatile(
        (crate::k32_externs::K32_USER_SHARED_DATA + 0x2D8) as *const ULONG,
    ) as DWORD;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/session.c
//  * PURPOSE:         Session Support APIs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// DosPathToSessionPathW(IN DWORD SessionID,
//                       IN LPWSTR InPath,
//                       OUT LPWSTR *OutPath)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// DosPathToSessionPathA(IN DWORD SessionId,
//                       IN LPSTR InPath,
//                       OUT LPSTR *OutPath)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WTSGetActiveConsoleSessionId(VOID)
// {
//     return SharedUserData->ActiveConsoleId;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
