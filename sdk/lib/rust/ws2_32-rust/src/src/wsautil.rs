use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;

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

pub const WINSOCK_ROOT: [::core::ffi::c_char; 54] = unsafe {
    ::core::mem::transmute::<[u8; 54], [::core::ffi::c_char; 54]>(
        *b"System\\CurrentControlSet\\Services\\WinSock2\\Parameters\0",
    )
};

#[no_mangle]
pub unsafe extern "system" fn WsOpenRegistryRoot() -> HKEY {
    let mut WinsockRootKey: HKEY = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut CreateDisposition: u32 = 0;

    ErrorCode = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        WINSOCK_ROOT.as_ptr() as *const i8,
        0,
        MAXIMUM_ALLOWED,
        &mut WinsockRootKey,
    );

    if ErrorCode == ERROR_FILE_NOT_FOUND as i32 {
        ErrorCode = RegCreateKeyExA(
            HKEY_LOCAL_MACHINE,
            WINSOCK_ROOT.as_ptr() as *const i8,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            &mut WinsockRootKey,
            &mut CreateDisposition,
        );
    } else if ErrorCode == ERROR_SUCCESS as i32 {
        CreateDisposition = REG_OPENED_EXISTING_KEY;
    }

    if ErrorCode != ERROR_SUCCESS as i32 {
        return core::ptr::null_mut();
    }

    if CreateDisposition == REG_CREATED_NEW_KEY {
        RegSetValueExA(
            WinsockRootKey,
            b"WinSock_Registry_Version\0".as_ptr() as *const i8,
            0,
            REG_SZ,
            b"2.2\0".as_ptr(),
            4,
        );
    }

    return WinsockRootKey;
}

#[no_mangle]
pub unsafe extern "system" fn WsCheckCatalogState(Event: HANDLE) -> BOOL {
    let Return: DWORD;

    Return = WaitForSingleObject(Event, 0);

    if Return == WAIT_OBJECT_0 {
        return (TRUE as i32);
    }

    return (FALSE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WsApiProlog(
    Process: *mut PWSPROCESS,
    Thread: *mut PWSTHREAD,
) -> i32 {
    let mut ErrorCode: i32 = WSANOTINITIALISED as i32;

    *Process = WsGetProcess();
    if !(*Process).is_null() {
        ErrorCode = WsThreadGetCurrentThread(*Process, Thread) as i32;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WsSlowProlog() -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();

    return WsApiProlog(&mut Process, &mut Thread);
}

#[no_mangle]
pub unsafe extern "system" fn WsSlowPrologTid(ThreadId: *mut LPWSATHREADID) -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);

    if ErrorCode == ERROR_SUCCESS as i32 {
        *ThreadId = core::ptr::addr_of_mut!((*Thread).WahThreadId);
    }

    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn WsSetupCatalogProtection(
    CatalogKey: HKEY,
    CatalogEvent: HANDLE,
    UniqueId: LPDWORD,
) -> INT {

    let mut ErrorCode: INT;
    let mut RegistryKey: HKEY = unsafe { core::mem::zeroed() };
    let mut NewUniqueId: DWORD = 0;
    let mut KeyBuffer: [i8; 32] = [0; 32];
    let mut RegType: DWORD = REG_DWORD;
    let mut RegSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    loop {
        ErrorCode = RegQueryValueExA(
            CatalogKey,
            b"Serial_Access_Num\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(NewUniqueId) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        ) as INT;

        if ErrorCode != ERROR_SUCCESS as INT {
            ErrorCode = WSASYSCALLFAILURE as INT;
            break;
        }

        sprintf(
            KeyBuffer.as_mut_ptr(),
            b"%8.8lX\0".as_ptr() as *const i8,
            NewUniqueId,
        );

        ErrorCode = RegOpenKeyExA(
            CatalogKey,
            KeyBuffer.as_ptr() as LPCSTR,
            0,
            MAXIMUM_ALLOWED as REGSAM,
            core::ptr::addr_of_mut!(RegistryKey),
        ) as INT;

        if ErrorCode == ERROR_FILE_NOT_FOUND as INT || ErrorCode == ERROR_KEY_DELETED as INT {
            ErrorCode = ERROR_SUCCESS as INT;
            *UniqueId = NewUniqueId;
            break;
        } else if ErrorCode != ERROR_SUCCESS as INT {
            ErrorCode = WSASYSCALLFAILURE as INT;
            break;
        }

        ErrorCode = RegCloseKey(RegistryKey) as INT;
        ErrorCode = WSANO_RECOVERY as INT;

        if WaitForSingleObject(CatalogEvent, 180 * 1000) != WAIT_OBJECT_0 {
            break;
        }
    }

    ErrorCode
}
#[no_mangle]
pub unsafe extern "system" fn MapUnicodeProtocolInfoToAnsi(
    UnicodeInfo: LPWSAPROTOCOL_INFOW,
    AnsiInfo: LPWSAPROTOCOL_INFOA,
) -> i32 {
    let mut ReturnValue: i32;

    core::ptr::copy_nonoverlapping(
        UnicodeInfo as *const u8,
        AnsiInfo as *mut u8,
        116,
    );

    ReturnValue = WideCharToMultiByte(
        CP_ACP,
        0,
        (core::ptr::addr_of_mut!((*UnicodeInfo).szProtocol) as *const u16 as *mut u16),
        -1,
        core::ptr::addr_of_mut!((*AnsiInfo).szProtocol) as *mut i8,
        core::mem::size_of::<[i8; 256]>() as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    if ReturnValue == 0 {
        return (WSASYSCALLFAILURE as i32);
    }

    return (ERROR_SUCCESS as i32);
}

const CP_ACP: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn WEP() -> () {
    return;
}
// enforce_import_abi: moved 7 import decl(s) into the extern block their
// exporting DLL's .spec requires: RegCloseKey->system, RegCreateKeyExA->system, RegOpenKeyExA->system, RegQueryValueExA->system, RegSetValueExA->system, WaitForSingleObject->system, WideCharToMultiByte->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/wsautil.c
//  * PURPOSE:     Winsock Utility Functions
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* FUNCTIONS *****************************************************************/
//
// HKEY
// __stdcall
// WsOpenRegistryRoot(VOID)
// {
//     HKEY WinsockRootKey;
//     INT ErrorCode;
//     ULONG CreateDisposition;
//
//     /* Open Registry Key */
//     ErrorCode = RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                              WINSOCK_ROOT,
//                              0,
//                              MAXIMUM_ALLOWED,
//                              &WinsockRootKey);
//
//     /* Check if it wasn't found */
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* Create it */
//         ErrorCode = RegCreateKeyEx(HKEY_LOCAL_MACHINE,
//                                    WINSOCK_ROOT,
//                                    0,
//                                    NULL,
//                                    REG_OPTION_NON_VOLATILE,
//                                    KEY_ALL_ACCESS,
//                                    NULL,
//                                    &WinsockRootKey,
//                                    &CreateDisposition);
//     }
//     else if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* It already exists */
//         CreateDisposition = REG_OPENED_EXISTING_KEY;
//     }
//
//     /* Check for failure */
//     if (ErrorCode != ERROR_SUCCESS) return NULL;
//
//     /* Check if we had to create a new key */
//     if (CreateDisposition == REG_CREATED_NEW_KEY)
//     {
//         /* Write the Winsock Version */
//         RegSetValueEx(WinsockRootKey,
//                       "WinSock_Registry_Version",
//                       0,
//                       REG_SZ,
//                       (BYTE*)"2.2",
//                       4);
//     }
//     else
//     {
//         /* Read the Winsock Version */
//     }
//
//     /* Return the key */
//     return WinsockRootKey;
// }
//
// BOOL
// __stdcall
// WsCheckCatalogState(IN HANDLE Event)
// {
//     DWORD Return;
//
//     /* Wait for the object */
//     Return = WaitForSingleObject(Event, 0);
//
//     /* Check for the value */
//     if (Return == WAIT_OBJECT_0) return TRUE;
//
//     /* If it timedout or anything else, return false */
//     return FALSE;
// }
//
// INT
// __stdcall
// WsApiProlog(OUT PWSPROCESS *Process,
//             OUT PWSTHREAD *Thread)
// {
//     INT ErrorCode = WSANOTINITIALISED;
//
//     /* Try to get the current process */
//     if ((*Process = WsGetProcess()))
//     {
//         /* And the current thread */
//         ErrorCode = WsThreadGetCurrentThread(*Process, Thread);
//     }
//
//     /* Return init status */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WsSlowProlog(VOID)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//
//     /* Call the prolog */
//     return WsApiProlog(&Process, &Thread);
// }
//
// INT
// __stdcall
// WsSlowPrologTid(OUT LPWSATHREADID *ThreadId)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//
//     /* Call the prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Return the Thread ID */
//         *ThreadId = &Thread->WahThreadId;
//     }
//
//     /* Return status */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WsSetupCatalogProtection(IN HKEY CatalogKey,
//                          IN HANDLE CatalogEvent,
//                          OUT LPDWORD UniqueId)
// {
//     INT ErrorCode;
//     HKEY RegistryKey;
//     DWORD NewUniqueId;
//     CHAR KeyBuffer[32];
//     DWORD RegType = REG_DWORD;
//     DWORD RegSize = sizeof(DWORD);
//
//     /* Start loop */
//     do
//     {
//
//         /* Read the current ID */
//         ErrorCode = RegQueryValueEx(CatalogKey,
//                                     "Serial_Access_Num",
//                                     0,
//                                     &RegType,
//                                     (LPBYTE)&NewUniqueId,
//                                     &RegSize);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Critical failure */
//             ErrorCode = WSASYSCALLFAILURE;
//             break;
//         }
//
//         /* Try to open it for writing */
//         sprintf(KeyBuffer, "%8.8lX", NewUniqueId);
//         ErrorCode = RegOpenKeyEx(CatalogKey,
//                                  KeyBuffer,
//                                  0,
//                                  MAXIMUM_ALLOWED,
//                                  &RegistryKey);
//
//         /* If the key doesn't exist or is being delete, that's ok for us */
//         if ((ErrorCode == ERROR_FILE_NOT_FOUND) ||
//             (ErrorCode == ERROR_KEY_DELETED))
//         {
//             /* Set success and return the new ID */
//             ErrorCode = ERROR_SUCCESS;
//             *UniqueId = NewUniqueId;
//             break;
//         }
//         else if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Any other failure is bad */
//             ErrorCode = WSASYSCALLFAILURE;
//             break;
//         }
//
//         /* If we could actually open the key, someone is using it :/ */
//         ErrorCode = RegCloseKey(RegistryKey);
//
//         /* In case we break out prematurely */
//         ErrorCode = WSANO_RECOVERY;
//
//         /* Keep looping until they let go of the registry writing */
//     } while (!WaitForSingleObject(CatalogEvent, 180 * 1000));
//
//     /* Return error code */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// MapUnicodeProtocolInfoToAnsi(IN LPWSAPROTOCOL_INFOW UnicodeInfo,
//                              OUT LPWSAPROTOCOL_INFOA AnsiInfo)
// {
//     INT ReturnValue;
//
//     /* Copy all the data that doesn't need converting */
//     RtlCopyMemory(AnsiInfo,
//                   UnicodeInfo,
//                   FIELD_OFFSET(WSAPROTOCOL_INFOA, szProtocol));
//
//     /* Now convert the protocol string */
//     ReturnValue = WideCharToMultiByte(CP_ACP,
//                                       0,
//                                       UnicodeInfo->szProtocol,
//                                       -1,
//                                       AnsiInfo->szProtocol,
//                                       sizeof(AnsiInfo->szProtocol),
//                                       NULL,
//                                       NULL);
//     if (!ReturnValue) return WSASYSCALLFAILURE;
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// __stdcall
// WEP(VOID)
// {
//     return;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
