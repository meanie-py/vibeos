use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

#[no_mangle]
pub unsafe extern "system" fn NetQueryDisplayInformation(
    ServerName: LPCWSTR,
    Level: DWORD,
    Index: DWORD,
    EntriesRequested: DWORD,
    PreferredMaximumLength: DWORD,
    ReturnedEntryCount: LPDWORD,
    SortedBuffer: *mut PVOID,
) -> DWORD {
    let mut ServerNameString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut ServerHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DomainHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DisplayInformation: _DOMAIN_DISPLAY_INFORMATION = 0 as _DOMAIN_DISPLAY_INFORMATION;
    let mut LocalTotalBytesAvailable: u32 = 0;
    let mut LocalTotalBytesReturned: u32 = 0;
    let mut LocalReturnedEntryCount: u32 = 0;
    let mut LocalSortedBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as u32;
    let mut Status: i32 = 0;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/display.c\0"
                as *const u8
                as *const core::ffi::c_char,
            b"NetQueryDisplayInformation\0" as *const u8 as *const core::ffi::c_char,
            42,
            b"NetQueryDisplayInformation(%s, %ld, %ld, %ld, %ld, %p, %p)\n\0" as *const u8
                as *const core::ffi::c_char,
            debugstr_w(ServerName as *const u16),
            Level,
            Index,
            EntriesRequested,
            PreferredMaximumLength,
            ReturnedEntryCount,
            SortedBuffer,
        );
    }

    *ReturnedEntryCount = 0u32;
    *SortedBuffer = core::ptr::null_mut::<core::ffi::c_void>();

    match Level {
        1 => {
            DisplayInformation = DomainDisplayUser;
        }
        2 => {
            DisplayInformation = DomainDisplayMachine;
        }
        3 => {
            DisplayInformation = DomainDisplayGroup;
        }
        _ => {
            return ERROR_INVALID_LEVEL as u32;
        }
    }

    if !ServerName.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ServerNameString), ServerName as PCWSTR);
    }

    Status = SamConnect(
        if !ServerName.is_null() {
            core::ptr::addr_of_mut!(ServerNameString)
        } else {
            core::ptr::null_mut::<_UNICODE_STRING>()
        },
        core::ptr::addr_of_mut!(ServerHandle),
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
    );

    if !(Status >= 0i32) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/display.c\0"
                    as *const u8
                    as *const core::ffi::c_char,
                b"NetQueryDisplayInformation\0" as *const u8 as *const core::ffi::c_char,
                75,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const core::ffi::c_char,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !ServerName.is_null() {
                core::ptr::addr_of_mut!(ServerNameString)
            } else {
                core::ptr::null_mut::<_UNICODE_STRING>()
            },
            DOMAIN_LIST_ACCOUNTS as u32,
            core::ptr::addr_of_mut!(DomainHandle),
        );
        if !(Status >= 0i32) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/display.c\0"
                        as *const u8
                        as *const core::ffi::c_char,
                    b"NetQueryDisplayInformation\0" as *const u8
                        as *const core::ffi::c_char,
                    87,
                    b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            Status = SamQueryDisplayInformation(
                DomainHandle,
                DisplayInformation,
                Index as u32,
                EntriesRequested as u32,
                PreferredMaximumLength as u32,
                core::ptr::addr_of_mut!(LocalTotalBytesAvailable),
                core::ptr::addr_of_mut!(LocalTotalBytesReturned),
                core::ptr::addr_of_mut!(LocalReturnedEntryCount),
                core::ptr::addr_of_mut!(LocalSortedBuffer),
            );
            if !(Status >= 0i32) {
                if (*(__wine_dbch___default)).flags as i32
                    & (1i32 << __WINE_DBCL_ERR as i32)
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/display.c\0"
                            as *const u8
                            as *const core::ffi::c_char,
                        b"NetQueryDisplayInformation\0" as *const u8
                            as *const core::ffi::c_char,
                        104,
                        b"SamQueryDisplayInformation failed (Status %08lx)\n\0"
                            as *const u8
                            as *const core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
            }
        }
    }

    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetGetDisplayInformationIndex(
    ServerName: LPCWSTR,
    Level: DWORD,
    Prefix: LPCWSTR,
    Index: LPDWORD,
) -> DWORD {
    let mut ServerNameString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut PrefixString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DisplayInformation: DOMAIN_DISPLAY_INFORMATION = 0 as DOMAIN_DISPLAY_INFORMATION;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = 0;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default as *const _ as *mut _,
            b"dll/win32/netapi32/display.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"NetGetDisplayInformationIndex\0" as *const u8 as *const core::ffi::c_char,
            138 as core::ffi::c_int,
            b"NetGetDisplayInformationIndex(%s %ld %s %p)\n\0" as *const u8
                as *const core::ffi::c_char,
            debugstr_w(ServerName as *const WCHAR),
            Level as core::ffi::c_long,
            debugstr_w(Prefix as *const WCHAR),
            Index,
        );
    }

    match Level {
        1 => {
            DisplayInformation = DomainDisplayUser;
        }
        2 => {
            DisplayInformation = DomainDisplayMachine;
        }
        3 => {
            DisplayInformation = DomainDisplayGroup;
        }
        _ => return ERROR_INVALID_LEVEL as DWORD,
    }

    if !ServerName.is_null() {
        RtlInitUnicodeString(&raw mut ServerNameString, ServerName as PCWSTR);
    }

    Status = SamConnect(
        if !ServerName.is_null() {
            &raw mut ServerNameString
        } else {
            core::ptr::null_mut::<_UNICODE_STRING>()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
    );

    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default as *const _ as *mut _,
                b"dll/win32/netapi32/display.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"NetGetDisplayInformationIndex\0" as *const u8 as *const core::ffi::c_char,
                168 as core::ffi::c_int,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8
                    as *const core::ffi::c_char,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !ServerName.is_null() {
                &raw mut ServerNameString
            } else {
                core::ptr::null_mut::<_UNICODE_STRING>()
            },
            DOMAIN_LIST_ACCOUNTS as ULONG,
            &raw mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default as *const _ as *mut _,
                    b"dll/win32/netapi32/display.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"NetGetDisplayInformationIndex\0" as *const u8
                        as *const core::ffi::c_char,
                    180 as core::ffi::c_int,
                    b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            RtlInitUnicodeString(&raw mut PrefixString, Prefix as PCWSTR);
            Status = SamGetDisplayEnumerationIndex(
                DomainHandle,
                DisplayInformation,
                &raw mut PrefixString,
                Index as PULONG,
            );
            if !NT_SUCCESS(Status) {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default as *const _ as *mut _,
                        b"dll/win32/netapi32/display.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"NetGetDisplayInformationIndex\0" as *const u8
                            as *const core::ffi::c_char,
                        194 as core::ffi::c_int,
                        b"SamGetDisplayEnumerationIndex failed (Status %08lx)\n\0"
                            as *const u8 as *const core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
            }
        }
    }

    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    ApiStatus
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     NetAPI DLL
//  * LICENSE:     GPL-2.0 (https://spdx.org/licenses/GPL-2.0)
//  * PURPOSE:     SAM service interface code
//  * COPYRIGHT:   Copyright 2017 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "netapi32.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// /* FUNCTIONS *****************************************************************/
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NET_API_STATUS
// WINAPI
// NetQueryDisplayInformation(
//      LPCWSTR ServerName,
//      DWORD Level,
//      DWORD Index,
//      DWORD EntriesRequested,
//      DWORD PreferredMaximumLength,
//      LPDWORD ReturnedEntryCount,
//      PVOID *SortedBuffer)
// {
//     UNICODE_STRING ServerNameString;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     DOMAIN_DISPLAY_INFORMATION DisplayInformation;
//     DWORD LocalTotalBytesAvailable;
//     DWORD LocalTotalBytesReturned;
//     DWORD LocalReturnedEntryCount;
//     PVOID LocalSortedBuffer;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status;
//
//     TRACE("NetQueryDisplayInformation(%s, %ld, %ld, %ld, %ld, %p, %p)\n",
//           debugstr_w(ServerName), Level, Index, EntriesRequested,
//           PreferredMaximumLength, ReturnedEntryCount, SortedBuffer);
//
//     *ReturnedEntryCount = 0;
//     *SortedBuffer = NULL;
//
//     switch (Level)
//     {
//         case 1:
//             DisplayInformation = DomainDisplayUser;
//             break;
//
//         case 2:
//             DisplayInformation = DomainDisplayMachine;
//             break;
//
//         case 3:
//             DisplayInformation = DomainDisplayGroup;
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     if (ServerName != NULL)
//         RtlInitUnicodeString(&ServerNameString, ServerName);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((ServerName != NULL) ? &ServerNameString : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (ServerName != NULL) ? &ServerNameString : NULL,
//                                DOMAIN_LIST_ACCOUNTS,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Query the information */
//     Status = SamQueryDisplayInformation(DomainHandle,
//                                         DisplayInformation,
//                                         Index,
//                                         EntriesRequested,
//                                         PreferredMaximumLength,
//                                         &LocalTotalBytesAvailable,
//                                         &LocalTotalBytesReturned,
//                                         &LocalReturnedEntryCount,
//                                         &LocalSortedBuffer);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamQueryDisplayInformation failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* FIXME */
//
// done:
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetGetDisplayInformationIndex(
//      LPCWSTR ServerName,
//      DWORD Level,
//      LPCWSTR Prefix,
//      LPDWORD Index)
// {
//     UNICODE_STRING ServerNameString, PrefixString;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     DOMAIN_DISPLAY_INFORMATION DisplayInformation;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status;
//
//     TRACE("NetGetDisplayInformationIndex(%s %ld %s %p)\n",
//           debugstr_w(ServerName), Level, debugstr_w(Prefix), Index);
//
//     switch (Level)
//     {
//         case 1:
//             DisplayInformation = DomainDisplayUser;
//             break;
//
//         case 2:
//             DisplayInformation = DomainDisplayMachine;
//             break;
//
//         case 3:
//             DisplayInformation = DomainDisplayGroup;
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     if (ServerName != NULL)
//         RtlInitUnicodeString(&ServerNameString, ServerName);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((ServerName != NULL) ? &ServerNameString : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (ServerName != NULL) ? &ServerNameString : NULL,
//                                DOMAIN_LIST_ACCOUNTS,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     RtlInitUnicodeString(&PrefixString, Prefix);
//
//     /* Get the index */
//     Status = SamGetDisplayEnumerationIndex(DomainHandle,
//                                            DisplayInformation,
//                                            &PrefixString,
//                                            Index);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetDisplayEnumerationIndex failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
