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

#[no_mangle]
pub unsafe extern "system" fn WsNcEntryAllocate() -> PNSCATALOG_ENTRY {
    let mut CatalogEntry: PNSCATALOG_ENTRY = core::ptr::null_mut();

    CatalogEntry = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_NSCATALOG_ENTRY>() as SIZE_T),
    ) as PNSCATALOG_ENTRY;

    if !CatalogEntry.is_null() {
        (*CatalogEntry).RefCount = 1;
        (*CatalogEntry).Enabled = 1u8;
        (*CatalogEntry).AddressFamily = -1;
    }

    return CatalogEntry;
}

#[no_mangle]
pub unsafe extern "system" fn WsNcEntryDelete(CatalogEntry: PNSCATALOG_ENTRY) {
    if !(*CatalogEntry).Provider.is_null() {
        WsNpDereference((*CatalogEntry).Provider);
        (*CatalogEntry).Provider = core::ptr::null_mut();
    }
    HeapFree(WsSockHeap, 0, CatalogEntry as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn WsNcEntryDereference(CatalogEntry: PNSCATALOG_ENTRY) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*CatalogEntry).RefCount)) == 0 {
        WsNcEntryDelete(CatalogEntry);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsNcEntryInitializeFromRegistry(
    CatalogEntry: PNSCATALOG_ENTRY,
    ParentKey: HKEY,
    UniqueId: ULONG,
) -> INT {
    let mut ErrorCode: INT;
    let mut CatalogEntryName: [CHAR; 13] = [0; 13];
    let mut EntryKey: HKEY = core::ptr::null_mut();
    let mut RegType: ULONG = REG_SZ;
    let mut RegSize: ULONG = MAX_PATH;
    let mut RegValue: ULONG = 0;

    sprintf(
        CatalogEntryName.as_mut_ptr(),
        b"%012i\0".as_ptr() as *const i8,
        UniqueId as i32,
    );

    ErrorCode = RegOpenKeyExA(
        ParentKey,
        CatalogEntryName.as_mut_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(EntryKey),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExW(
        EntryKey,
        win32macros_rust::wide_bytes!(b"L\0i\0b\0r\0a\0r\0y\0P\0a\0t\0h\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!((*CatalogEntry).DllPath) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExW(
        EntryKey,
        win32macros_rust::wide_bytes!(b"D\0i\0s\0p\0l\0a\0y\0S\0t\0r\0i\0n\0g\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    (*CatalogEntry).ProviderName = HeapAlloc(WsSockHeap, 0, RegSize as SIZE_T) as LPWSTR;

    ErrorCode = RegQueryValueExW(
        EntryKey,
        win32macros_rust::wide_bytes!(b"D\0i\0s\0p\0l\0a\0y\0S\0t\0r\0i\0n\0g\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        (*CatalogEntry).ProviderName as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    RegType = REG_BINARY;
    RegSize = core::mem::size_of::<GUID>() as ULONG;
    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"ProviderId\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!((*CatalogEntry).ProviderId) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    RegType = REG_DWORD;
    RegSize = core::mem::size_of::<DWORD>() as ULONG;
    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"AddressFamily\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!((*CatalogEntry).AddressFamily) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"SupportedNamespace\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!((*CatalogEntry).NamespaceId) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"Enabled\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!(RegValue) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    (*CatalogEntry).Enabled = (RegValue != 0) as i32 as BOOLEAN;

    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"Version\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!((*CatalogEntry).Version) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        RegCloseKey(EntryKey);
        return ErrorCode;
    }

    ErrorCode = RegQueryValueExA(
        EntryKey,
        b"StoresServiceClassInfo\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::addr_of_mut!(RegValue) as LPBYTE,
        core::ptr::addr_of_mut!(RegSize),
    ) as INT;

    (*CatalogEntry).StoresServiceClassInfo = (RegValue != 0) as i32 as BOOLEAN;

    RegCloseKey(EntryKey);
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WsNcEntrySetProvider(
    Entry: PNSCATALOG_ENTRY,
    Provider: PNS_PROVIDER,
) {
    InterlockedIncrement(&raw mut (*Provider).RefCount);
    (*Entry).Provider = Provider;
}
// enforce_import_abi: moved 5 import decl(s) into the extern block their
// exporting DLL's .spec requires: HeapAlloc->system, RegCloseKey->system, RegOpenKeyExA->system, RegQueryValueExA->system, RegQueryValueExW->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/nscatent.c
//  * PURPOSE:     Namespace Catalog Entry Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* FUNCTIONS *****************************************************************/
//
// PNSCATALOG_ENTRY
// __stdcall
// WsNcEntryAllocate(VOID)
// {
//     PNSCATALOG_ENTRY CatalogEntry;
//
//     /* Allocate the catalog */
//     CatalogEntry = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*CatalogEntry));
//     if (CatalogEntry)
//     {
//         /* Set the default non-null members */
//         CatalogEntry->RefCount = 1;
//         CatalogEntry->Enabled = TRUE;
//         CatalogEntry->AddressFamily = -1;
//     }
//
//     /* Return it */
//     return CatalogEntry;
// }
//
// VOID
// __stdcall
// WsNcEntryDelete(IN PNSCATALOG_ENTRY CatalogEntry)
// {
//     /* Check if a provider is loaded */
//     if (CatalogEntry->Provider)
//     {
//         /* Dereference it too */
//         WsNpDereference(CatalogEntry->Provider);
//         CatalogEntry->Provider = NULL;
//     }
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, CatalogEntry);
// }
//
// VOID
// __stdcall
// WsNcEntryDereference(IN PNSCATALOG_ENTRY CatalogEntry)
// {
//     /* Dereference and check if it's now 0 */
//     if (!(InterlockedDecrement(&CatalogEntry->RefCount)))
//     {
//         /* We can delete the Provider now */
//         WsNcEntryDelete(CatalogEntry);
//     }
// }
//
// INT
// __stdcall
// WsNcEntryInitializeFromRegistry(IN PNSCATALOG_ENTRY CatalogEntry,
//                                 IN HKEY ParentKey,
//                                 IN ULONG UniqueId)
// {
//     INT ErrorCode;
//     CHAR CatalogEntryName[13];
//     HKEY EntryKey;
//     ULONG RegType = REG_SZ;
//     ULONG RegSize = MAX_PATH;
//     ULONG RegValue;
//
//     /* Convert to a 00000xxx string */
//     sprintf(CatalogEntryName, "%0""12""i", (int)UniqueId);
//
//     /* Open the Entry */
//     ErrorCode = RegOpenKeyEx(ParentKey,
//                              CatalogEntryName,
//                              0,
//                              KEY_READ,
//                              &EntryKey);
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//     /* Read the Library Path */
//     ErrorCode = RegQueryValueExW(EntryKey,
//                                  L"LibraryPath",
//                                  0,
//                                  &RegType,
//                                  (LPBYTE)&CatalogEntry->DllPath,
//                                  &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Query Display String Size*/
//     ErrorCode = RegQueryValueExW(EntryKey,
//                                  L"DisplayString",
//                                  0,
//                                  NULL,
//                                  NULL,
//                                  &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Allocate it */
//     CatalogEntry->ProviderName = (LPWSTR)HeapAlloc(WsSockHeap, 0, RegSize);
//
//     /* Read it */
//     ErrorCode = RegQueryValueExW(EntryKey,
//                                  L"DisplayString",
//                                  0,
//                                  &RegType,
//                                  (LPBYTE)CatalogEntry->ProviderName,
//                                  &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Read the Provider Id */
//     RegType = REG_BINARY;
//     RegSize = sizeof(GUID);
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "ProviderId",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&CatalogEntry->ProviderId,
//                                 &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Read the Address Family */
//     RegType = REG_DWORD;
//     RegSize = sizeof(DWORD);
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "AddressFamily",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&CatalogEntry->AddressFamily,
//                                 &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Read the Namespace Id */
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "SupportedNamespace",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&CatalogEntry->NamespaceId,
//                                 &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Read the Enabled Flag */
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "Enabled",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&RegValue,
//                                 &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     CatalogEntry->Enabled = RegValue != 0;
//
//     /* Read the Version */
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "Version",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&CatalogEntry->Version,
//                                 &RegSize);
//     if (ErrorCode != ERROR_SUCCESS) goto out;
//     /* Read the Support Service Class Info Flag */
//     ErrorCode = RegQueryValueEx(EntryKey,
//                                 "StoresServiceClassInfo",
//                                 0,
//                                 &RegType,
//                                 (LPBYTE)&RegValue,
//                                 &RegSize);
//     CatalogEntry->StoresServiceClassInfo = RegValue != 0;
// out:
//     /* Done */
//     RegCloseKey(EntryKey);
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsNcEntrySetProvider(IN PNSCATALOG_ENTRY Entry,
//                      IN PNS_PROVIDER Provider)
// {
//     /* Reference the provider */
//     InterlockedIncrement(&Provider->RefCount);
//
//     /* Set it */
//     Entry->Provider = Provider;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
