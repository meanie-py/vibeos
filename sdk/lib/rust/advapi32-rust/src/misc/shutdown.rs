use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(advapi) - debug channel macro, no Rust equivalent

#[no_mangle]
pub unsafe extern "system" fn AbortSystemShutdownA(lpMachineName: LPCSTR) -> BOOL {
    let mut MachineNameA: ANSI_STRING = core::mem::zeroed();
    let mut MachineNameW: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;
    let rv: BOOL;

    RtlInitAnsiString(&mut MachineNameA, lpMachineName as LPSTR);
    Status = RtlAnsiStringToUnicodeString(&mut MachineNameW, &mut MachineNameA, 1u8);
    if STATUS_SUCCESS != Status {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    rv = AbortSystemShutdownW(MachineNameW.Buffer);
    RtlFreeUnicodeString(&mut MachineNameW);
    SetLastError(ERROR_SUCCESS);
    rv
}

#[no_mangle]
pub unsafe extern "system" fn AbortSystemShutdownW(lpMachineName: LPCWSTR) -> BOOL {
    let dwError: DWORD = rpc_try(|| BaseAbortSystemShutdown(lpMachineName as PREGISTRY_SERVER_NAME)).unwrap_or_else(|code| RtlNtStatusToDosError(code as _) as _);

    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn InitiateSystemShutdownA(
    lpMachineName: LPSTR,
    lpMessage: LPSTR,
    dwTimeout: DWORD,
    bForceAppsClosed: BOOL,
    bRebootAfterShutdown: BOOL,
) -> BOOL {
    let mut MachineNameA: ANSI_STRING = core::mem::zeroed();
    let mut MessageA: ANSI_STRING = core::mem::zeroed();
    let mut MachineNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut MessageW: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let res: BOOL;

    MachineNameW.Buffer = core::ptr::null_mut();
    MessageW.Buffer = core::ptr::null_mut();

    if !lpMachineName.is_null() {
        RtlInitAnsiString(&mut MachineNameA, lpMachineName);
        Status = RtlAnsiStringToUnicodeString(&mut MachineNameW, &mut MachineNameA, 1u8);
        if STATUS_SUCCESS != Status {
            if !MachineNameW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MachineNameW);
            }

            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    }

    if !lpMessage.is_null() {
        RtlInitAnsiString(&mut MessageA, lpMessage);
        Status = RtlAnsiStringToUnicodeString(&mut MessageW, &mut MessageA, 1u8);
        if STATUS_SUCCESS != Status {
            if !MessageW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MessageW);
            }

            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    }

    res = InitiateSystemShutdownW(
        MachineNameW.Buffer,
        MessageW.Buffer,
        dwTimeout,
        bForceAppsClosed,
        bRebootAfterShutdown,
    );

    if !lpMachineName.is_null() {
        RtlFreeUnicodeString(&mut MachineNameW);
    }

    if !lpMessage.is_null() {
        RtlFreeUnicodeString(&mut MessageW);
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn InitiateSystemShutdownW(
    lpMachineName: LPWSTR,
    lpMessage: LPWSTR,
    dwTimeout: DWORD,
    bForceAppsClosed: BOOL,
    bRebootAfterShutdown: BOOL,
) -> BOOL {
    let mut Message: _UNICODE_STRING = core::mem::zeroed();
    let dwError: DWORD;

    RtlInitUnicodeString(&mut Message, lpMessage);

    dwError = rpc_try(|| BaseInitiateSystemShutdown(
        lpMachineName as PREGISTRY_SERVER_NAME,
        &mut Message as *mut _UNICODE_STRING as PRPC_UNICODE_STRING,
        dwTimeout,
        bForceAppsClosed as BOOLEAN,
        bRebootAfterShutdown as BOOLEAN,
    )).unwrap_or_else(|code| RtlNtStatusToDosError(code as _) as _);

    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn InitiateSystemShutdownExA(
    lpMachineName: LPSTR,
    lpMessage: LPSTR,
    dwTimeout: DWORD,
    bForceAppsClosed: BOOL,
    bRebootAfterShutdown: BOOL,
    dwReason: DWORD,
) -> BOOL {
    let mut MachineNameA: ANSI_STRING = core::mem::zeroed();
    let mut MessageA: ANSI_STRING = core::mem::zeroed();
    let mut MachineNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut MessageW: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    MachineNameW.Buffer = core::ptr::null_mut();
    MessageW.Buffer = core::ptr::null_mut();

    if !lpMachineName.is_null() {
        RtlInitAnsiString(&mut MachineNameA, lpMachineName);
        let s = RtlAnsiStringToUnicodeString(&mut MachineNameW, &mut MachineNameA, 1u8);
        if STATUS_SUCCESS != s {
            if !MachineNameW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MachineNameW);
            }
            SetLastError(RtlNtStatusToDosError(s));
            return 0;
        }
    }

    if !lpMessage.is_null() {
        RtlInitAnsiString(&mut MessageA, lpMessage);
        let s = RtlAnsiStringToUnicodeString(&mut MessageW, &mut MessageA, 1u8);
        if STATUS_SUCCESS != s {
            if !MessageW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MessageW);
            }
            SetLastError(RtlNtStatusToDosError(s));
            return 0;
        }
    }

    let res = InitiateSystemShutdownExW(
        MachineNameW.Buffer,
        MessageW.Buffer,
        dwTimeout,
        bForceAppsClosed,
        bRebootAfterShutdown,
        dwReason,
    );

    if !lpMachineName.is_null() {
        RtlFreeUnicodeString(&mut MachineNameW);
    }

    if !lpMessage.is_null() {
        RtlFreeUnicodeString(&mut MessageW);
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn InitiateSystemShutdownExW(
    lpMachineName: LPWSTR,
    lpMessage: LPWSTR,
    dwTimeout: DWORD,
    bForceAppsClosed: BOOL,
    bRebootAfterShutdown: BOOL,
    dwReason: DWORD,
) -> BOOL {
    let mut Message: _UNICODE_STRING = core::mem::zeroed();
    let dwError: DWORD;

    RtlInitUnicodeString(&mut Message, lpMessage);

    dwError = rpc_try(|| BaseInitiateSystemShutdownEx(
        lpMachineName as PREGISTRY_SERVER_NAME,
        core::ptr::addr_of_mut!(Message) as PRPC_UNICODE_STRING,
        dwTimeout,
        bForceAppsClosed as BOOLEAN,
        bRebootAfterShutdown as BOOLEAN,
        dwReason,
    )).unwrap_or_else(|code| code as _);

    if dwError != ERROR_SUCCESS {
        TRACE!(b"BaseInitiateSystemShutdownEx() failed (Error %lu)\n\0".as_ptr() as LPCSTR, dwError);
        SetLastError(dwError);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn InitiateShutdownW(
    lpMachineName: LPWSTR,
    lpMessage: LPWSTR,
    dwGracePeriod: DWORD,
    dwShutdownFlags: DWORD,
    dwReason: DWORD,
) -> DWORD {
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn InitiateShutdownA(
    lpMachineName: LPSTR,
    lpMessage: LPSTR,
    dwGracePeriod: DWORD,
    dwShutdownFlags: DWORD,
    dwReason: DWORD,
) -> DWORD {
    let mut MachineNameA: ANSI_STRING = core::mem::zeroed();
    let mut MessageA: ANSI_STRING = core::mem::zeroed();
    let mut MachineNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut MessageW: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;
    let LastError: INT;
    let rv: DWORD;

    MachineNameW.Buffer = core::ptr::null_mut();
    MessageW.Buffer = core::ptr::null_mut();

    if !lpMachineName.is_null() {
        RtlInitAnsiString(&mut MachineNameA, lpMachineName);
        let Status = RtlAnsiStringToUnicodeString(&mut MachineNameW, &mut MachineNameA, 1u8);
        if STATUS_SUCCESS != Status {
            if !MachineNameW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MachineNameW);
            }

            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    }

    if !lpMessage.is_null() {
        RtlInitAnsiString(&mut MessageA, lpMessage);
        let Status = RtlAnsiStringToUnicodeString(&mut MessageW, &mut MessageA, 1u8);
        if STATUS_SUCCESS != Status {
            if !MessageW.Buffer.is_null() {
                RtlFreeUnicodeString(&mut MessageW);
            }

            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    }

    let _ = Status;

    rv = InitiateShutdownW(
        MachineNameW.Buffer,
        MessageW.Buffer,
        dwGracePeriod,
        dwShutdownFlags,
        dwReason,
    );
    LastError = GetLastError() as INT;

    if !lpMachineName.is_null() {
        RtlFreeUnicodeString(&mut MachineNameW);
    }

    if !lpMessage.is_null() {
        RtlFreeUnicodeString(&mut MessageW);
    }

    SetLastError(LastError as DWORD);
    rv
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS system libraries
//  * FILE:        dll/win32/advapi32/misc/shutdown.c
//  * PURPOSE:     System shutdown functions
//  * PROGRAMMER:  Lee Schroeder <spaceseel at gmail dot com>
//  *              Emanuele Aliberti
//  */
//
// #include <advapi32.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /**********************************************************************
//  *      AbortSystemShutdownA
//  *
//  * see AbortSystemShutdownW
//  */
// BOOL WINAPI
// AbortSystemShutdownA(LPCSTR lpMachineName)
// {
//     ANSI_STRING MachineNameA;
//     UNICODE_STRING MachineNameW;
//     NTSTATUS Status;
//     BOOL rv;
//
//     RtlInitAnsiString(&MachineNameA, (LPSTR)lpMachineName);
//     Status = RtlAnsiStringToUnicodeString(&MachineNameW, &MachineNameA, TRUE);
//     if (STATUS_SUCCESS != Status)
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     rv = AbortSystemShutdownW(MachineNameW.Buffer);
//     RtlFreeUnicodeString(&MachineNameW);
//     SetLastError(ERROR_SUCCESS);
//     return rv;
// }
//
//
// /**********************************************************************
//  *      AbortSystemShutdownW
//  *
//  * @implemented
//  */
// BOOL WINAPI
// AbortSystemShutdownW(LPCWSTR lpMachineName)
// {
//     DWORD dwError;
//
//     RpcTryExcept
//     {
//         dwError = BaseAbortSystemShutdown((PREGISTRY_SERVER_NAME)lpMachineName);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwError = RtlNtStatusToDosError(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         TRACE("BaseAbortSystemShutdown() failed (Error %lu)\n", dwError);
//         SetLastError(dwError);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *      InitiateSystemShutdownA
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// InitiateSystemShutdownA(LPSTR lpMachineName,
//                         LPSTR lpMessage,
//                         DWORD dwTimeout,
//                         BOOL bForceAppsClosed,
//                         BOOL bRebootAfterShutdown)
// {
//     ANSI_STRING MachineNameA, MessageA;
//     UNICODE_STRING MachineNameW, MessageW;
//     NTSTATUS Status;
//     BOOL res;
//
//     MachineNameW.Buffer = NULL;
//     MessageW.Buffer = NULL;
//
//     if (lpMachineName)
//     {
//         RtlInitAnsiString(&MachineNameA, lpMachineName);
//         Status = RtlAnsiStringToUnicodeString(&MachineNameW, &MachineNameA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if(MachineNameW.Buffer)
//                 RtlFreeUnicodeString(&MachineNameW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     if (lpMessage)
//     {
//         RtlInitAnsiString(&MessageA, lpMessage);
//         Status = RtlAnsiStringToUnicodeString(&MessageW, &MessageA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if (MessageW.Buffer)
//                 RtlFreeUnicodeString(&MessageW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     res = InitiateSystemShutdownW(MachineNameW.Buffer,
//                                   MessageW.Buffer,
//                                   dwTimeout,
//                                   bForceAppsClosed,
//                                   bRebootAfterShutdown);
//
//     /* Clear the values of both strings */
//     if (lpMachineName)
//         RtlFreeUnicodeString(&MachineNameW);
//
//     if (lpMessage)
//         RtlFreeUnicodeString(&MessageW);
//
//     return res;
// }
//
//
// /**********************************************************************
//  *      InitiateSystemShutdownW
//  *
//  * @implemented
//  */
// BOOL WINAPI
// InitiateSystemShutdownW(LPWSTR lpMachineName,
//                         LPWSTR lpMessage,
//                         DWORD dwTimeout,
//                         BOOL bForceAppsClosed,
//                         BOOL bRebootAfterShutdown)
// {
//     UNICODE_STRING Message;
//     DWORD dwError;
//
//     RtlInitUnicodeString(&Message, lpMessage);
//
//     RpcTryExcept
//     {
//         dwError = BaseInitiateSystemShutdown((PREGISTRY_SERVER_NAME)lpMachineName,
//                                              (PRPC_UNICODE_STRING)&Message,
//                                              dwTimeout,
//                                              bForceAppsClosed,
//                                              bRebootAfterShutdown);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwError = RtlNtStatusToDosError(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         TRACE("BaseInitiateSystemShutdown() failed (Error %lu)\n", dwError);
//         SetLastError(dwError);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * InitiateSystemShutdownExA [ADVAPI32.@]
//  *
//  * see InitiateSystemShutdownExW
//  */
// BOOL WINAPI
// InitiateSystemShutdownExA(LPSTR lpMachineName,
//                           LPSTR lpMessage,
//                           DWORD dwTimeout,
//                           BOOL bForceAppsClosed,
//                           BOOL bRebootAfterShutdown,
//                           DWORD dwReason)
// {
//     ANSI_STRING MachineNameA, MessageA;
//     UNICODE_STRING MachineNameW, MessageW;
//     NTSTATUS Status;
//     BOOL res;
//
//     MachineNameW.Buffer = NULL;
//     MessageW.Buffer = NULL;
//
//     if (lpMachineName)
//     {
//         RtlInitAnsiString(&MachineNameA, lpMachineName);
//         Status = RtlAnsiStringToUnicodeString(&MachineNameW, &MachineNameA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if(MachineNameW.Buffer)
//                 RtlFreeUnicodeString(&MachineNameW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     if (lpMessage)
//     {
//         RtlInitAnsiString(&MessageA, lpMessage);
//         Status = RtlAnsiStringToUnicodeString(&MessageW, &MessageA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if (MessageW.Buffer)
//                 RtlFreeUnicodeString(&MessageW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     res = InitiateSystemShutdownExW(MachineNameW.Buffer,
//                                     MessageW.Buffer,
//                                     dwTimeout,
//                                     bForceAppsClosed,
//                                     bRebootAfterShutdown,
//                                     dwReason);
//
//     /* Clear the values of both strings */
//     if (lpMachineName)
//         RtlFreeUnicodeString(&MachineNameW);
//
//     if (lpMessage)
//         RtlFreeUnicodeString(&MessageW);
//
//     return res;
// }
//
//
// /******************************************************************************
//  * InitiateSystemShutdownExW [ADVAPI32.@]
//  *
//  * @unimplemented
//  */
// BOOL WINAPI
// InitiateSystemShutdownExW(LPWSTR lpMachineName,
//                           LPWSTR lpMessage,
//                           DWORD dwTimeout,
//                           BOOL bForceAppsClosed,
//                           BOOL bRebootAfterShutdown,
//                           DWORD dwReason)
// {
//     UNICODE_STRING Message;
//     DWORD dwError;
//
//     RtlInitUnicodeString(&Message, lpMessage);
//
//     RpcTryExcept
//     {
//         dwError = BaseInitiateSystemShutdownEx((PREGISTRY_SERVER_NAME)lpMachineName,
//                                                (PRPC_UNICODE_STRING)&Message,
//                                                dwTimeout,
//                                                bForceAppsClosed,
//                                                bRebootAfterShutdown,
//                                                dwReason);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwError = RpcExceptionCode();
//     }
//     RpcEndExcept;
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         TRACE("BaseInitiateSystemShutdownEx() failed (Error %lu)\n", dwError);
//         SetLastError(dwError);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * InitiateShutdownW [ADVAPI32.@]
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// InitiateShutdownW(LPWSTR lpMachineName,
//                   LPWSTR lpMessage,
//                   DWORD dwGracePeriod,
//                   DWORD dwShutdownFlags,
//                   DWORD dwReason)
// {
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// /******************************************************************************
//  * InitiateShutdownA [ADVAPI32.@]
//  *
//  * see InitiateShutdownW
//  */
// DWORD WINAPI
// InitiateShutdownA(LPSTR lpMachineName,
//                   LPSTR lpMessage,
//                   DWORD dwGracePeriod,
//                   DWORD dwShutdownFlags,
//                   DWORD dwReason)
// {
//     ANSI_STRING MachineNameA, MessageA;
//     UNICODE_STRING MachineNameW, MessageW;
//     NTSTATUS Status;
//     INT LastError;
//     DWORD rv;
//
//     MachineNameW.Buffer = NULL;
//     MessageW.Buffer = NULL;
//
//     if (lpMachineName)
//     {
//         RtlInitAnsiString(&MachineNameA, lpMachineName);
//         Status = RtlAnsiStringToUnicodeString(&MachineNameW, &MachineNameA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if(MachineNameW.Buffer)
//                 RtlFreeUnicodeString(&MachineNameW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     if (lpMessage)
//     {
//         RtlInitAnsiString(&MessageA, lpMessage);
//         Status = RtlAnsiStringToUnicodeString(&MessageW, &MessageA, TRUE);
//         if (STATUS_SUCCESS != Status)
//         {
//             if (MessageW.Buffer)
//                 RtlFreeUnicodeString(&MessageW);
//
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     rv = InitiateShutdownW(MachineNameW.Buffer,
//                            MessageW.Buffer,
//                            dwGracePeriod,
//                            dwShutdownFlags,
//                            dwReason);
//     LastError = GetLastError();
//
//     /* Clear the values of both strings */
//     if (lpMachineName)
//         RtlFreeUnicodeString(&MachineNameW);
//
//     if (lpMessage)
//         RtlFreeUnicodeString(&MessageW);
//
//     SetLastError(LastError);
//     return rv;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
