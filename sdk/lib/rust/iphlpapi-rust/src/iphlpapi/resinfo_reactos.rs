use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _NAME_SERVER_LIST_PRIVATE {
    pub NumServers: u32,
    pub pCurrent: *mut IP_ADDR_STRING,
}
pub type NAME_SERVER_LIST_PRIVATE = _NAME_SERVER_LIST_PRIVATE;
pub type PNAME_SERVER_LIST_PRIVATE = *mut _NAME_SERVER_LIST_PRIVATE;
// C (resinfo_reactos.c:34): `typedef VOID (*ENUM_INTERFACE_CALLBACK)(HKEY, LPWSTR, PVOID)`
// — no WINAPI/__stdcall, so cdecl. Must be extern "C": on i386 `extern "system"` is stdcall,
// which disagrees on who pops the arguments (amd64/arm64 would hide the bug — they alias).
pub type ENUM_INTERFACE_CALLBACK = Option<
    unsafe extern "C" fn(ChildKeyHandle: HKEY, ChildKeyName: *mut u16, CallbackContext: PVOID),
>;
#[no_mangle]
pub unsafe extern "C" fn QueryNameServer(
    hInterface: HKEY,
    NameServerKey: LPCWSTR,
    OutNameServer: *mut LPWSTR,
) -> LSTATUS {
    let mut dwLength: u32 = 0;
    let mut dwType: u32 = 0;
    let mut status: LSTATUS;

    // First call to retrieve required size
    status = RegQueryValueExW(
        hInterface,
        NameServerKey,
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut dwLength,
    );
    if status != (ERROR_SUCCESS as i32) {
        return status;
    }

    // Add space for terminating null
    dwLength += 2; // sizeof(WCHAR) == 2

    // Allocate buffer
    let name_server = HeapAlloc(GetProcessHeap(), 0, (dwLength as SIZE_T)) as LPWSTR;
    if name_server.is_null() {
        return (ERROR_OUTOFMEMORY as i32);
    }

    // Second call to retrieve actual value
    status = RegQueryValueExW(
        hInterface,
        NameServerKey,
        core::ptr::null_mut(),
        &mut dwType,
        name_server as LPBYTE,
        &mut dwLength,
    );
    if status != (ERROR_SUCCESS as i32) || dwType != REG_SZ {
        HeapFree(GetProcessHeap(), 0, name_server as *mut core::ffi::c_void);
        return status;
    }

    // Overwrite the terminator (dwLength already includes the null from RegQueryValueExW,
    // so this sets one character past it, which is safe but redundant)
    let index = (dwLength as usize) / 2; // divide by sizeof(WCHAR)
    *name_server.add(index) = 0u16; // L'\0'

    // Store result
    *OutNameServer = name_server;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn EnumNameServers(
    hInterface: HKEY,
    InterfaceName: LPWSTR,
    ServerCallbackContext: PVOID,
    // iphlpapi_private.h: `EnumNameServersFunc` = cdecl, nullable (bindgen Option).
    CallbackRoutine: EnumNameServersFunc,
) -> i32 {
    let mut status: i32;
    let mut name_server: *mut u16 = core::ptr::null_mut();
    let mut buffer = [0u16; 50];
    let mut start: *mut u16;
    let mut comma: *mut u16;
    let mut length: u32;

    let nameserver_key: [u16; 11] = [78, 97, 109, 101, 83, 101, 114, 118, 101, 114, 0];
    let dhcpnameserver_key: [u16; 15] = [68, 104, 99, 112, 78, 97, 109, 101, 83, 101, 114, 118, 101, 114, 0];

    status = QueryNameServer((hInterface as *mut HKEY__), nameserver_key.as_ptr(), core::ptr::addr_of_mut!(name_server));
    if status != 0 {
        status = QueryNameServer((hInterface as *mut HKEY__), dhcpnameserver_key.as_ptr(), core::ptr::addr_of_mut!(name_server));
        if status != 0 {
            return status;
        }
    }

    start = name_server;

    loop {
        comma = wcschr(start as *const u16, ',' as u16) as *mut u16;

        if !comma.is_null() {
            length = (comma as *const u16).offset_from(start as *const u16) as u32;

            core::ptr::copy_nonoverlapping(start, buffer.as_mut_ptr(), length as usize);
            buffer[length as usize] = 0;
            if let Some(cb) = CallbackRoutine {
                cb(InterfaceName, buffer.as_mut_ptr(), ServerCallbackContext);
            }

            start = comma.add(1);
        } else {
            if let Some(cb) = CallbackRoutine {
                cb(InterfaceName, start, ServerCallbackContext);
            }
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, name_server as *mut core::ffi::c_void);
    0
}
#[no_mangle]
pub unsafe extern "C" fn EnumInterfaces(
    CallbackRoutine: ENUM_INTERFACE_CALLBACK,
    InterfaceCallbackContext: PVOID,
) -> LSTATUS {
    const INTERFACES_KEY: [u16; 62] = [
        0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
        0x005C,
        0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043,
        0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065,
        0x0074,
        0x005C,
        0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073,
        0x005C,
        0x0054, 0x0063, 0x0070, 0x0069, 0x0070,
        0x005C,
        0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065,
        0x0072, 0x0073,
        0x005C,
        0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063,
        0x0065, 0x0073,
        0x0000,
    ];

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hInterface: HKEY = core::ptr::null_mut();
    let mut Status: LSTATUS;
    let mut NumInterfaces: u32 = 0;
    let mut InterfaceNameLen: u32 = 0;
    let mut Index: u32;
    let mut Length: u32;
    let mut InterfaceName: LPWSTR;

    Status = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        INTERFACES_KEY.as_ptr() as LPWSTR,
        0,
        KEY_READ,
        &mut hKey as *mut HKEY,
    );

    if Status != (ERROR_SUCCESS as i32) {
        return Status;
    }

    Status = RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut NumInterfaces as *mut u32,
        &mut InterfaceNameLen as *mut u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != (ERROR_SUCCESS as i32) {
        RegCloseKey(hKey);
        return Status;
    }

    InterfaceNameLen += 1;

    InterfaceName = HeapAlloc(
        GetProcessHeap(),
        0,
        (InterfaceNameLen as usize * 2) as SIZE_T,
    ) as LPWSTR;

    if InterfaceName.is_null() {
        RegCloseKey(hKey);
        return (ERROR_OUTOFMEMORY as i32);
    }

    for Index in 0..NumInterfaces {
        Length = InterfaceNameLen;
        Status = RegEnumKeyExW(
            hKey,
            Index,
            InterfaceName,
            &mut Length as *mut u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if Status == (ERROR_SUCCESS as i32) {
            *InterfaceName.wrapping_add(Length as usize) = 0;

            Status = RegOpenKeyExW(
                hKey,
                InterfaceName,
                0,
                KEY_READ,
                &mut hInterface as *mut HKEY,
            );

            if Status == (ERROR_SUCCESS as i32) {
                if let Some(cb) = CallbackRoutine {
                    cb(hInterface, InterfaceName, InterfaceCallbackContext);
                }
                RegCloseKey(hInterface);
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, InterfaceName as *mut core::ffi::c_void);
    RegCloseKey(hKey);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CountNameServerCallback(
    _interface_name: LPWSTR,
    _server: LPWSTR,
    callback_context: PVOID,
) {
    let data = callback_context as *mut NAME_SERVER_LIST_PRIVATE;
    (*data).NumServers += 1;
}
#[no_mangle]
pub unsafe extern "C" fn CountServerCallbackTrampoline(
    ChildKeyHandle: HKEY,
    ChildKeyName: LPWSTR,
    CallbackContext: *mut core::ffi::c_void,
) {
    EnumNameServers(
        ChildKeyHandle,
        ChildKeyName,
        CallbackContext,
        Some(CountNameServerCallback),
    );
}
#[no_mangle]
pub unsafe extern "C" fn CountNameServers(PrivateData: PNAME_SERVER_LIST_PRIVATE) -> LSTATUS {
    EnumInterfaces(Some(CountServerCallbackTrampoline), PrivateData as PVOID)
}
#[no_mangle]
pub unsafe extern "C" fn CreateNameServerListCallback(
    _interface_name: *mut u16,
    server: *mut u16,
    callback_context: *mut core::ffi::c_void,
) {
    let data = callback_context as *mut _NAME_SERVER_LIST_PRIVATE;
    if WideCharToMultiByte(
        0u32,
        0,
        server as *const u16,
        -1i32,
        (*(*data).pCurrent).IpAddress.String.as_mut_ptr(),
        16i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) != 0
    {
        let old_cur = (*data).pCurrent;
        let new_cur = old_cur.add(1);
        (*old_cur).Next = new_cur;
        (*data).pCurrent = new_cur;
        (*data).NumServers += 1;
    } else {
        (*(*data).pCurrent).IpAddress.String[0] = (b'\0' as i8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateNameServerListCallbackTrampoline(
    ChildKeyHandle: HKEY,
    ChildKeyName: LPWSTR,
    CallbackContext: PVOID,
) {
    unsafe {
        EnumNameServers(ChildKeyHandle, ChildKeyName, CallbackContext, Some(CreateNameServerListCallback));
    }
}
#[no_mangle]
pub unsafe extern "C" fn MakeNameServerList(PrivateData: PNAME_SERVER_LIST_PRIVATE) -> LSTATUS {
    EnumInterfaces(Some(CreateNameServerListCallbackTrampoline), PrivateData as PVOID)
}
#[no_mangle]
pub unsafe extern "C" fn getResInfo() -> *mut _IPHLP_RES_INFO {
    let mut private_enum: _NAME_SERVER_LIST_PRIVATE = core::mem::zeroed();
    let mut dns_list: *mut IP_ADDR_STRING = core::ptr::null_mut();
    let mut status: LSTATUS;
    let res_info: *mut _IPHLP_RES_INFO;

    private_enum.NumServers = 0;
    status = CountNameServers(&raw mut private_enum);

    if status != (ERROR_SUCCESS as i32) {
        return core::ptr::null_mut();
    }

    if private_enum.NumServers != 0 {
        let alloc_size = (private_enum.NumServers as usize) * core::mem::size_of::<IP_ADDR_STRING>();
        dns_list = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ((alloc_size) as SIZE_T)) as *mut IP_ADDR_STRING;
        if dns_list.is_null() {
            return core::ptr::null_mut();
        }
    }

    res_info = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ((core::mem::size_of::<_IPHLP_RES_INFO>()) as SIZE_T)) as *mut _IPHLP_RES_INFO;
    if res_info.is_null() {
        if !dns_list.is_null() {
            HeapFree(GetProcessHeap(), 0, dns_list as *mut core::ffi::c_void);
        }
        return core::ptr::null_mut();
    }

    if private_enum.NumServers != 0 {
        private_enum.NumServers = 0;
        private_enum.pCurrent = dns_list;
        MakeNameServerList(&raw mut private_enum);
        (*res_info).DnsList = dns_list;
        (*res_info).riCount = private_enum.NumServers;
    }

    res_info
}
#[no_mangle]
pub unsafe extern "C" fn disposeResInfo(InfoPtr: *mut IPHLP_RES_INFO) {
    HeapFree(GetProcessHeap(), 0u32, (*InfoPtr).DnsList as *mut core::ffi::c_void);
    RtlFreeHeap(GetProcessHeap(), 0u32, InfoPtr as *mut core::ffi::c_void);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * iphlpapi dll implementation -- Setting and storing route information
//  *
//  * These are stubs for functions that set routing information on the target
//  * operating system.  They are grouped here because their implementation will
//  * vary widely by operating system.
//  *
//  * Copyright (C) 2004 Art Yerkes
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include <config.h>
// #include "iphlpapi_private.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi);
//
// typedef struct _NAME_SERVER_LIST_PRIVATE {
//     UINT NumServers;
//     IP_ADDR_STRING * pCurrent;
// } NAME_SERVER_LIST_PRIVATE, *PNAME_SERVER_LIST_PRIVATE;
//
// typedef VOID (*ENUM_INTERFACE_CALLBACK)(
//     HKEY ChildKeyHandle,
//     LPWSTR ChildKeyName,
//     PVOID CallbackContext);
//
// LSTATUS
// QueryNameServer(
//      HKEY hInterface,
//      LPCWSTR NameServerKey,
//      LPWSTR * OutNameServer)
// {
//     DWORD dwLength, dwType;
//     LPWSTR NameServer;
//     LSTATUS Status;
//
//     /* query ns */
//     dwLength = 0;
//     Status = RegQueryValueExW(hInterface, NameServerKey, NULL, &dwType, NULL, &dwLength);
//
//     if (Status != ERROR_SUCCESS)
//     {
//         /* failed to retrieve size */
//         TRACE!("Status %x\n", Status);
//         return Status;
//     }
//
//     /* add terminating null */
//     dwLength += sizeof(WCHAR);
//
//     /* allocate name server */
//     NameServer = HeapAlloc(GetProcessHeap(), 0, dwLength);
//
//     if (!NameServer)
//     {
//         /* no memory */
//         return ERROR_OUTOFMEMORY;
//     }
//
//     /* query ns */
//     Status = RegQueryValueExW(hInterface, NameServerKey, NULL, &dwType, (LPBYTE)NameServer, &dwLength);
//
//     if (Status != ERROR_SUCCESS || dwType != REG_SZ)
//     {
//         /* failed to retrieve ns */
//         HeapFree(GetProcessHeap(), 0, NameServer);
//         return Status;
//     }
//
//     /* null terminate it */
//     NameServer[dwLength / sizeof(WCHAR)] = L'\0';
//
//     /* store result */
//     *OutNameServer = NameServer;
//
//     return STATUS_SUCCESS;
// }
//
//
// LSTATUS
// EnumNameServers(
//      HKEY hInterface,
//      LPWSTR InterfaceName,
//     PVOID ServerCallbackContext,
//     EnumNameServersFunc CallbackRoutine)
// {
//     LSTATUS Status;
//     LPWSTR NameServer;
//     WCHAR Buffer[50];
//     DWORD Length;
//     LPWSTR Start, Comma;
//
//     /* query static assigned name server */
//     Status = QueryNameServer(hInterface, L"NameServer", &NameServer);
//     if (Status != ERROR_SUCCESS)
//     {
//         /* query dynamic assigned name server */
//         Status = QueryNameServer(hInterface, L"DhcpNameServer", &NameServer);
//
//         if (Status != ERROR_SUCCESS)
//         {
//             /* failed to retrieve name servers */
//             return Status;
//         }
//     }
//
//     /* enumerate all name servers, terminated by comma */
//     Start = NameServer;
//
//     do
//     {
//         /* find next terminator */
//         Comma = wcschr(Start, L',');
//
//         if (Comma)
//         {
//             /* calculate length */
//             Length = Comma - Start;
//
//             /* copy name server */
//             RtlMoveMemory(Buffer, Start, Length * sizeof(WCHAR));
//
//             /* null terminate it */
//             Buffer[Length] = L'\0';
//
//             /* perform callback */
//             CallbackRoutine(InterfaceName, Buffer, ServerCallbackContext);
//
//         }
//         else
//         {
//             /* perform callback */
//             CallbackRoutine(InterfaceName, Start, ServerCallbackContext);
//
//             /* last entry */
//             break;
//         }
//
//         /* increment offset */
//         Start = Comma + 1;
//
//     }while(TRUE);
//
//     /* free name server string */
//     HeapFree(GetProcessHeap(), 0, NameServer);
//
//     /* done */
//     return ERROR_SUCCESS;
// }
//
// LSTATUS
// EnumInterfaces(
//     ENUM_INTERFACE_CALLBACK CallbackRoutine,
//     PVOID InterfaceCallbackContext)
// {
//     HKEY hKey, hInterface;
//     LSTATUS Status;
//     DWORD NumInterfaces, InterfaceNameLen, Index, Length;
//     LPWSTR InterfaceName;
//
//     /* first open interface key */
//     Status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces", 0, KEY_READ, &hKey);
//
//     /* check for success */
//     if (Status != ERROR_SUCCESS)
//     {
//         /* failed to open interface key */
//         return Status;
//     }
//
//     /* now get maximum interface name length and number of interfaces */
//     Status = RegQueryInfoKeyW(hKey, NULL, NULL, NULL, &NumInterfaces, &InterfaceNameLen, NULL, NULL, NULL, NULL, NULL, NULL);
//     if (Status != ERROR_SUCCESS)
//     {
//         /* failed to get key info */
//         RegCloseKey(hKey);
//         return Status;
//     }
//
//     /* RegQueryInfoKey does not include terminating null */
//     InterfaceNameLen++;
//
//     /* allocate interface name */
//     InterfaceName = (LPWSTR) HeapAlloc(GetProcessHeap(), 0, InterfaceNameLen * sizeof(WCHAR));
//
//     if (!InterfaceName)
//     {
//         /* no memory */
//         RegCloseKey(hKey);
//         return ERROR_OUTOFMEMORY;
//     }
//
//     /* no enumerate all interfaces */
//     for(Index = 0; Index < NumInterfaces; Index++)
//     {
//         /* query interface name */
//         Length = InterfaceNameLen;
//         Status = RegEnumKeyExW(hKey, Index, InterfaceName, &Length, NULL, NULL, NULL, NULL);
//
//         if (Status == ERROR_SUCCESS)
//         {
//             /* make sure it is null terminated */
//             InterfaceName[Length] = L'\0';
//
//             /* now open child key */
//             Status = RegOpenKeyExW(hKey, InterfaceName, 0, KEY_READ, &hInterface);
//
//             if (Status == ERROR_SUCCESS)
//             {
//                 /* perform enumeration callback */
//                 CallbackRoutine(hInterface, InterfaceName, InterfaceCallbackContext);
//
//                 /* close interface key */
//                 RegCloseKey(hInterface);
//             }
//         }
//     }
//
//     /* free interface name */
//     HeapFree(GetProcessHeap(), 0, InterfaceName);
//
//     /* close root interface key */
//     RegCloseKey(hKey);
//
//     /* done */
//     return Status;
// }
//
// VOID
// CountNameServerCallback(
//      LPWSTR InterfaceName,
//      LPWSTR Server,
//      PVOID CallbackContext)
// {
//     /* get context */
//     PNAME_SERVER_LIST_PRIVATE Data = (PNAME_SERVER_LIST_PRIVATE)CallbackContext;
//
//     /* increment server count */
//     Data->NumServers++;
// }
//
// VOID
// CountServerCallbackTrampoline(
//     HKEY ChildKeyHandle,
//     LPWSTR ChildKeyName,
//     PVOID CallbackContext)
// {
//     EnumNameServers(ChildKeyHandle, ChildKeyName, CallbackContext, CountNameServerCallback);
// }
//
// LSTATUS
// CountNameServers(
//      PNAME_SERVER_LIST_PRIVATE PrivateData )
// {
//     return EnumInterfaces(CountServerCallbackTrampoline, (PVOID)PrivateData);
// }
//
// VOID
// CreateNameServerListCallback(
//      LPWSTR InterfaceName,
//      LPWSTR Server,
//      PVOID CallbackContext)
// {
//     /* get context */
//     PNAME_SERVER_LIST_PRIVATE Data = (PNAME_SERVER_LIST_PRIVATE)CallbackContext;
//
//     /* convert to ansi ns string */
//     if (WideCharToMultiByte(CP_ACP, 0, Server, -1, Data->pCurrent->IpAddress.String, 16, NULL, NULL))
//     {
//         /* store offset to next name server struct */
//         Data->pCurrent->Next = (struct _IP_ADDR_STRING*)(Data->pCurrent + 1);
//
//         /* move to next entry */
//         Data->pCurrent = Data->pCurrent->Next;
//
//         /* increment server count */
//         Data->NumServers++;
//     }
//     else
//     {
//         /* failed to convert dns server */
//         Data->pCurrent->IpAddress.String[0] = '\0';
//     }
// }
//
// VOID
// CreateNameServerListCallbackTrampoline(
//     HKEY ChildKeyHandle,
//     LPWSTR ChildKeyName,
//     PVOID CallbackContext)
// {
//     EnumNameServers(ChildKeyHandle, ChildKeyName, CallbackContext, Some(CreateNameServerListCallback));
// }
//
// LSTATUS
// MakeNameServerList(
//     PNAME_SERVER_LIST_PRIVATE PrivateData )
// {
//     return EnumInterfaces(CreateNameServerListCallbackTrampoline, (PVOID)PrivateData);
// }
//
// PIPHLP_RES_INFO
// getResInfo()
// {
//     NAME_SERVER_LIST_PRIVATE PrivateNSEnum;
//     PIPHLP_RES_INFO ResInfo;
//     IP_ADDR_STRING * DnsList = NULL;
//     LSTATUS Status;
//
//     PrivateNSEnum.NumServers = 0;
//
//     /* count name servers */
//     Status = CountNameServers(&PrivateNSEnum);
//
//     if (Status != ERROR_SUCCESS)
//     {
//         /* failed to enumerate name servers */
//         return NULL;
//     }
//
//     /* are there any servers */
//     if (PrivateNSEnum.NumServers)
//     {
//         /* allocate dns servers */
//         DnsList = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PrivateNSEnum.NumServers * sizeof(IP_ADDR_STRING));
//
//         if (!DnsList)
//         {
//             /* no memory */
//             return NULL;
//         }
//     }
//
//     /* allocate private struct */
//     ResInfo = (PIPHLP_RES_INFO)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(IPHLP_RES_INFO));
//
//     if(!ResInfo)
//     {
//         /* no memory */
//         if (DnsList)
//         {
//             /* free dns list */
//             HeapFree( GetProcessHeap(), 0, DnsList);
//         }
//         return NULL;
//     }
//
//     /* are there any servers */
//     if (PrivateNSEnum.NumServers)
//     {
//         /* initialize enumeration context */
//         PrivateNSEnum.NumServers = 0;
//         PrivateNSEnum.pCurrent = DnsList;
//
//         /* enumerate servers */
//         MakeNameServerList( &PrivateNSEnum );
//
//         /* store result */
//         ResInfo->DnsList = DnsList;
//         ResInfo->riCount = PrivateNSEnum.NumServers;
//     }
//
//     /* done */
//     return ResInfo;
// }
//
// VOID disposeResInfo( PIPHLP_RES_INFO InfoPtr )
// {
//     HeapFree(GetProcessHeap(), 0, InfoPtr->DnsList);
//     RtlFreeHeap( GetProcessHeap(), 0, InfoPtr );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
