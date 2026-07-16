use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::wsautil::WsOpenRegistryRoot;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut HostnameGuid: GUID = _GUID {
    Data1: 0x2a800 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut AddressGuid: GUID = _GUID {
    Data1: 0x2a801 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut HostAddrByNameGuid: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IANAGuid: GUID = _GUID {
    Data1: 0x2a802 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut InetHostName: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut Ipv6Guid: GUID = _GUID {
    Data1: ((0x9 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | 53 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x1c as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut WsRasInitialized: BOOLEAN = 0;

pub static mut WsRasDllHandle: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub static mut WsRasHelperLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub static mut lpfnWSAttemptAutodialAddr: PWS_ATTEMPT_AUTODIAL_ADDR = None;

pub static mut lpfnWSAttemptAutodialName: PWS_ATTEMPT_AUTODIAL_NAME = None;

pub static mut lpfnWSNoteSuccessfulHostentLookup: PWS_NOTE_SUCCESSFUL_HOSTENT_LOOKUP = None;

pub type PWS_ATTEMPT_AUTODIAL_ADDR = Option<unsafe extern "system" fn(
    Name: *const SOCKADDR,
    NameLength: i32,
) -> i32>;

pub type PWS_ATTEMPT_AUTODIAL_NAME = Option<unsafe extern "system" fn(
    lpqsRestrictions: LPWSAQUERYSETW,
) -> i32>;

pub type PWS_NOTE_SUCCESSFUL_HOSTENT_LOOKUP = Option<unsafe extern "system" fn(
    Name: *const i8,
    Address: u32,
)>;

#[no_mangle]
pub unsafe extern "system" fn WsRasInitializeAutodial() {
    InitializeCriticalSection((core::ptr::addr_of_mut!(WsRasHelperLock) as *mut c_void));
}

#[no_mangle]
pub unsafe extern "system" fn WsRasUninitializeAutodial() {
    EnterCriticalSection(core::ptr::addr_of_mut!(WsRasHelperLock));
    if !WsRasDllHandle.is_null() {
        FreeLibrary((WsRasDllHandle as *mut c_void));
    }
    WsRasDllHandle = core::ptr::null_mut();
    LeaveCriticalSection(core::ptr::addr_of_mut!(WsRasHelperLock));
    DeleteCriticalSection(core::ptr::addr_of_mut!(WsRasHelperLock));
}

#[no_mangle]
pub unsafe extern "system" fn WsRasLoadHelperDll() -> i32 {
    let mut HelperPath: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut WinsockKey: HKEY;
    let mut ErrorCode: i32;
    let mut RegType: u32 = REG_SZ;
    let mut RegSize: u32 = MAX_PATH;

    EnterCriticalSection(core::ptr::addr_of_mut!(WsRasHelperLock));

    if WsRasInitialized == 0 {
        WinsockKey = WsOpenRegistryRoot();
        if !WinsockKey.is_null() {
            ErrorCode = RegQueryValueExA(
                WinsockKey,
                b"AutodialDLL\0".as_ptr() as *const i8,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(RegType),
                HelperPath.as_mut_ptr() as LPBYTE,
                core::ptr::addr_of_mut!(RegSize),
            );
            RegCloseKey(WinsockKey);

            if ErrorCode == ERROR_SUCCESS as i32 {
                WsRasDllHandle = LoadLibraryA(HelperPath.as_ptr());
            }
        }

        if WsRasDllHandle.is_null() {
            WsRasDllHandle = LoadLibraryA(b"rasadhlp.dll\0".as_ptr() as *const i8);
        }

        if !WsRasDllHandle.is_null() {
            lpfnWSAttemptAutodialAddr = core::mem::transmute(GetProcAddress(
                WsRasDllHandle,
                b"WSAttemptAutodialAddr\0".as_ptr() as *const i8,
            ));
            lpfnWSAttemptAutodialName = core::mem::transmute(GetProcAddress(
                WsRasDllHandle,
                b"WSAttemptAutodialName\0".as_ptr() as *const i8,
            ));
            lpfnWSNoteSuccessfulHostentLookup = core::mem::transmute(GetProcAddress(
                WsRasDllHandle,
                b"WSNoteSuccessfulHostentLookup\0".as_ptr() as *const i8,
            ));
        }

        WsRasInitialized = 1u8;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsRasHelperLock));

    WsRasInitialized as i32
}

#[no_mangle]
pub unsafe extern "system" fn WSAttemptAutodialAddr(
    Name: *const SOCKADDR,
    NameLength: INT,
) -> BOOL {
    if WsRasLoadHelperDll() == 0 || lpfnWSAttemptAutodialAddr.is_none() {
        return (FALSE as i32);
    }
    return lpfnWSAttemptAutodialAddr.unwrap_unchecked()(Name, NameLength);
}

#[no_mangle]
pub unsafe extern "system" fn WSAttemptAutodialName(lpqsRestrictions: LPWSAQUERYSETW) -> BOOL {
    if WsRasLoadHelperDll() == 0 || lpfnWSAttemptAutodialName.is_none() {
        return 0;
    }
    return lpfnWSAttemptAutodialName.unwrap_unchecked()(lpqsRestrictions);
}

#[no_mangle]
pub unsafe extern "system" fn WSNoteSuccessfulHostentLookup(
    Name: *const CHAR,
    Address: ULONG,
) {
    if WsRasLoadHelperDll() == 0 || lpfnWSNoteSuccessfulHostentLookup.is_none() {
        return;
    }
    lpfnWSNoteSuccessfulHostentLookup.unwrap_unchecked()(Name, Address);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/rasdial.c
//  * PURPOSE:     RAS Auto-Dial Support
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// typedef BOOL
// (__stdcall *PWS_ATTEMPT_AUTODIAL_ADDR)(
//     IN CONST SOCKADDR  *Name,
//     IN INT NameLength
// );
//
// typedef BOOL
// (__stdcall *PWS_ATTEMPT_AUTODIAL_NAME)(IN CONST LPWSAQUERYSETW lpqsRestrictions);
//
// typedef VOID
// (__stdcall *PWS_NOTE_SUCCESSFUL_HOSTENT_LOOKUP)(
//     IN CONST CHAR  *Name,
//     IN CONST ULONG Address
// );
//
// BOOLEAN WsRasInitialized;
// HINSTANCE WsRasDllHandle;
// CRITICAL_SECTION WsRasHelperLock;
// PWS_ATTEMPT_AUTODIAL_ADDR lpfnWSAttemptAutodialAddr;
// PWS_ATTEMPT_AUTODIAL_NAME lpfnWSAttemptAutodialName;
// PWS_NOTE_SUCCESSFUL_HOSTENT_LOOKUP lpfnWSNoteSuccessfulHostentLookup;
//
// #define WsRasLock()         EnterCriticalSection(&WsRasHelperLock)
// #define WsRasUnlock()       LeaveCriticalSection(&WsRasHelperLock)
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// __stdcall
// WsRasInitializeAutodial(VOID)
// {
//     /* Initialize the autodial lock */
//     InitializeCriticalSection(&WsRasHelperLock);
// }
//
// VOID
// __stdcall
// WsRasUninitializeAutodial(VOID)
// {
//     /* Acquire lock */
//     WsRasLock();
//
//     /* Free the library if it's loaded */
//     if (WsRasDllHandle) FreeLibrary(WsRasDllHandle);
//     WsRasDllHandle = NULL;
//
//     /* Release and delete lock */
//     WsRasUnlock();
//     DeleteCriticalSection(&WsRasHelperLock);
// }
//
// INT
// __stdcall
// WsRasLoadHelperDll(VOID)
// {
//     CHAR HelperPath[MAX_PATH];
//     HKEY WinsockKey;
//     INT ErrorCode;
//     DWORD RegType = REG_SZ;
//     DWORD RegSize = MAX_PATH;
//
//     /* Acquire the lock */
//     WsRasLock();
//
//     /* Check if we were already initialized */
//     if (!WsRasInitialized)
//     {
//         /* Open the registry root key */
//         WinsockKey = WsOpenRegistryRoot();
//         if (WinsockKey)
//         {
//             /* Read the helper's location */
//             ErrorCode = RegQueryValueEx(WinsockKey,
//                                         "AutodialDLL",
//                                         0,
//                                         &RegType,
//                                         (LPBYTE)&HelperPath,
//                                         &RegSize);
//             RegCloseKey(WinsockKey);
//
//             /* Make sure we read the path */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Now load it */
//                 WsRasDllHandle = LoadLibrary(HelperPath);
//             }
//         }
//
//         /* Check if we weren't able to load it and load the default */
//         if (!WsRasDllHandle) WsRasDllHandle = LoadLibrary("rasadhlp.dll");
//
//         /* Check again if we loaded it */
//         if (WsRasDllHandle)
//         {
//             /* Get function pointers */
//             lpfnWSAttemptAutodialAddr =
//                 (PVOID)GetProcAddress(WsRasDllHandle,
//                                       "WSAttemptAutodialAddr");
//             lpfnWSAttemptAutodialName =
//                 (PVOID)GetProcAddress(WsRasDllHandle,
//                                       "WSAttemptAutodialName");
//             lpfnWSNoteSuccessfulHostentLookup =
//                 (PVOID)GetProcAddress(WsRasDllHandle,
//                                       "WSNoteSuccessfulHostentLookup");
//         }
//
//         /* Mark us as loaded */
//         WsRasInitialized = TRUE;
//     }
//
//     /* Release lock */
//     WsRasUnlock();
//
//     /* Return status */
//     return WsRasInitialized;
// }
//
// BOOL
// __stdcall
// WSAttemptAutodialAddr(IN CONST SOCKADDR  *Name,
//                       IN INT NameLength)
// {
//     /* Load the helper DLL and make sure it exports this routine */
//     if (!(WsRasLoadHelperDll()) || !(lpfnWSAttemptAutodialAddr)) return FALSE;
//
//     /* Call the function in the helper */
//     return lpfnWSAttemptAutodialAddr(Name, NameLength);
// }
//
// BOOL
// __stdcall
// WSAttemptAutodialName(IN CONST LPWSAQUERYSETW lpqsRestrictions)
// {
//     /* Load the helper DLL and make sure it exports this routine */
//     if (!(WsRasLoadHelperDll()) || !(lpfnWSAttemptAutodialName)) return FALSE;
//
//     /* Call the function in the helper */
//     return lpfnWSAttemptAutodialName(lpqsRestrictions);
// }
//
// VOID
// __stdcall
// WSNoteSuccessfulHostentLookup(IN CONST CHAR  *Name,
//                               IN CONST ULONG Address)
// {
//     /* Load the helper DLL and make sure it exports this routine */
//     if (!(WsRasLoadHelperDll()) || !(lpfnWSNoteSuccessfulHostentLookup)) return;
//
//     /* Call the function in the helper */
//     lpfnWSNoteSuccessfulHostentLookup(Name, Address);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
