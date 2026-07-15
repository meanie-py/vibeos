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
pub unsafe extern "system" fn WsTcEntryAllocate() -> PTCATALOG_ENTRY {

    let CatalogEntry: PTCATALOG_ENTRY = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_TCATALOG_ENTRY>() as SIZE_T),
    ) as PTCATALOG_ENTRY;

    if !CatalogEntry.is_null() {
        (*CatalogEntry).RefCount = 1;
    }

    CatalogEntry
}

#[no_mangle]
pub unsafe extern "system" fn WsTcEntryDelete(CatalogEntry: PTCATALOG_ENTRY) {
    if !(*CatalogEntry).Provider.is_null() {
        WsTpDereference((*CatalogEntry).Provider);
        (*CatalogEntry).Provider = core::ptr::null_mut();
    }
    HeapFree(WsSockHeap, 0, CatalogEntry as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn WsTcEntryDereference(CatalogEntry: PTCATALOG_ENTRY) {
    if InterlockedDecrement(&mut (*CatalogEntry).RefCount) == 0 {
        WsTcEntryDelete(CatalogEntry);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsTcEntryInitializeFromRegistry(
    CatalogEntry: PTCATALOG_ENTRY,
    ParentKey: HKEY,
    UniqueId: DWORD,
) -> DWORD {
    let mut CatalogEntryName: [CHAR; 13] = [0; 13];
    let mut RegSize: DWORD = 0;
    let mut RegType: DWORD = REG_BINARY;
    let mut EntryKey: HKEY = core::ptr::null_mut();
    let mut Return: DWORD;
    let mut Buf: LPBYTE;
    let mut index: DWORD;

    sprintf(
        CatalogEntryName.as_mut_ptr(),
        b"%012lu\0".as_ptr() as *const core::ffi::c_char,
        UniqueId,
    );

    Return = RegOpenKeyExA(
        ParentKey,
        CatalogEntryName.as_mut_ptr(),
        0,
        KEY_READ,
        &mut EntryKey,
    ) as DWORD;

    Return = RegQueryValueExA(
        EntryKey,
        b"PackedCatalogItem\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut RegSize,
    ) as DWORD;

    Buf = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, RegSize as SIZE_T) as LPBYTE;
    if Buf.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    Return = RegQueryValueExA(
        EntryKey,
        b"PackedCatalogItem\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        &mut RegType,
        Buf,
        &mut RegSize,
    ) as DWORD;

    memcpy(
        (*CatalogEntry).DllPath.as_mut_ptr() as *mut core::ffi::c_void,
        Buf as *const core::ffi::c_void,
        core::mem::size_of_val(&(*CatalogEntry).DllPath),
    );
    index = core::mem::size_of_val(&(*CatalogEntry).DllPath) as DWORD;
    if index < RegSize {
        memcpy(
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo) as *mut core::ffi::c_void,
            Buf.offset(index as isize) as *const core::ffi::c_void,
            core::mem::size_of::<WSAPROTOCOL_INFOW>(),
        );
        index = index.wrapping_add(core::mem::size_of::<WSAPROTOCOL_INFOW>() as DWORD);
    }
    HeapFree(WsSockHeap, 0, Buf as PVOID);

    RegCloseKey(EntryKey);
    Return
}

#[no_mangle]
pub unsafe extern "system" fn WsTcEntrySetProvider(
    Entry: PTCATALOG_ENTRY,
    Provider: PTPROVIDER,
) {
    InterlockedIncrement(&mut (*Provider).RefCount as *mut i32);
    (*Entry).Provider = Provider;
}
// enforce_import_abi: moved 5 import decl(s) into the extern block their
// exporting DLL's .spec requires: HeapAlloc->system, HeapFree->system, RegCloseKey->system, RegOpenKeyExA->system, RegQueryValueExA->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dcatitem.c
//  * PURPOSE:     Transport Catalog Entry Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* FUNCTIONS *****************************************************************/
//
// PTCATALOG_ENTRY
// __stdcall
// WsTcEntryAllocate(VOID)
// {
//     PTCATALOG_ENTRY CatalogEntry;
//
//     /* Allocate the catalog entry */
//     CatalogEntry = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*CatalogEntry));
//     if (CatalogEntry)
//     {
//         /* Set the default non-null members */
//         CatalogEntry->RefCount = 1;
//     }
//
//     /* Return it */
//     return CatalogEntry;
// }
//
// VOID
// __stdcall
// WsTcEntryDelete(IN PTCATALOG_ENTRY CatalogEntry)
// {
//     /* Check if a provider is loaded */
//     if (CatalogEntry->Provider)
//     {
//         /* Dereference it too */
//         WsTpDereference(CatalogEntry->Provider);
//         CatalogEntry->Provider = NULL;
//     }
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, CatalogEntry);
// }
//
// VOID
// __stdcall
// WsTcEntryDereference(IN PTCATALOG_ENTRY CatalogEntry)
// {
//     /* Dereference and check if it's now 0 */
//     if (!(InterlockedDecrement(&CatalogEntry->RefCount)))
//     {
//         /* We can delete the Provider now */
//         WsTcEntryDelete(CatalogEntry);
//     }
// }
//
// DWORD
// __stdcall
// WsTcEntryInitializeFromRegistry(IN PTCATALOG_ENTRY CatalogEntry,
//                                 IN HKEY ParentKey,
//                                 IN DWORD UniqueId)
// {
//     CHAR CatalogEntryName[13];
//     DWORD RegSize;
//     DWORD RegType = REG_BINARY;
//     HKEY EntryKey;
//     DWORD Return;
//     LPBYTE Buf;
//     DWORD index;
//
//     /* Convert to a 00000xxx string */
//     sprintf(CatalogEntryName, "%0""12""lu", UniqueId);
//
//     /* Open the Entry */
//     Return = RegOpenKeyEx(ParentKey,
//                           CatalogEntryName,
//                           0,
//                           KEY_READ,
//                           &EntryKey);
//
//     /* Get Size of Catalog Entry Structure */
//     Return = RegQueryValueEx(EntryKey,
//                               "PackedCatalogItem",
//                               0,
//                               NULL,
//                               NULL,
//                               &RegSize);
//
//     if (!(Buf = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, RegSize)))
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     /* Read the Whole Catalog Entry Structure */
//     Return = RegQueryValueEx(EntryKey,
//                               "PackedCatalogItem",
//                               0,
//                               &RegType,
//                               Buf,
//                               &RegSize);
//
//
//     memcpy(CatalogEntry->DllPath, Buf, sizeof(CatalogEntry->DllPath));
//     index = sizeof(CatalogEntry->DllPath);
//     if (index < RegSize)
//     {
//         memcpy(&CatalogEntry->ProtocolInfo, &Buf[index], sizeof(WSAPROTOCOL_INFOW));
//         index += sizeof(WSAPROTOCOL_INFOW);
//     }
//     HeapFree(WsSockHeap, 0, Buf);
//
//     /* Done */
//     RegCloseKey(EntryKey);
//     return Return;
// }
//
// VOID
// __stdcall
// WsTcEntrySetProvider(IN PTCATALOG_ENTRY Entry,
//                      IN PTPROVIDER Provider)
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
