use crate::types::*;
use crate::glue::*;
use core::ffi::c_void;

// stdcall kernel32/advapi32 imports -> extern "system" (i386 `_Name@N` decoration).
extern "system" {
    fn HeapAlloc(heap: HANDLE, flags: DWORD, size: usize) -> *mut core::ffi::c_void;
    fn HeapFree(heap: HANDLE, flags: DWORD, ptr: *mut core::ffi::c_void) -> BOOL;
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: *mut HKEY) -> LONG;
    fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG;
    fn RegCloseKey(hKey: HKEY) -> LONG;
    fn ExpandEnvironmentStringsW(lpSrc: PWSTR, lpDst: PWSTR, nSize: u32) -> u32;
    fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    fn LoadLibraryW(lpLibFileName: PWSTR) -> HMODULE;
}
// cdecl C-runtime (msvcrt). ERR/TRACE are shadowed by the glue macros (dead decls).
// HKEY_LOCAL_MACHINE is a glue const (winreg.h macro, not an import).
extern "C" {
    fn ERR(fmt: *const u8, ...);
    fn TRACE(fmt: *const u8, ...);
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    fn wcscat(dest: *mut u16, src: *const u16) -> *mut u16;
}

// ---- file-scope preamble (translated) ----
pub static mut HelperDLLDatabaseLock: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut HelperDLLDatabaseListHead: LIST_ENTRY = LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
#[no_mangle]
pub unsafe extern "C" fn SockGetTdiName(
    AddressFamily: PINT,
    SocketType: PINT,
    Protocol: PINT,
    Group: GROUP,
    Flags: DWORD,
    TransportName: PUNICODE_STRING,
    HelperDllContext: *mut PVOID,
    HelperDllData: *mut PHELPER_DATA,
    Events: PDWORD,
) -> INT {
    let mut HelperData: PHELPER_DATA = core::ptr::null_mut();
    let mut Transports: PWSTR = core::ptr::null_mut();
    let mut Transport: PWSTR;
    let mut Mapping: PWINSOCK_MAPPING = core::ptr::null_mut();
    let mut Helpers: PLIST_ENTRY;
    let mut Status: INT;

    TRACE!(
        b"AddressFamily %p, SocketType %p, Protocol %p, Group %u, Flags %lx, TransportName %p, HelperDllContext %p, HelperDllData %p, Events %p\n\0".as_ptr() as *const i8,
        AddressFamily,
        SocketType,
        Protocol,
        Group,
        Flags,
        TransportName,
        HelperDllContext,
        HelperDllData,
        Events,
    );

    Helpers = SockHelpersListHead.Flink;
    while Helpers != core::ptr::addr_of_mut!(SockHelpersListHead) {
        HelperData = (Helpers as *mut u8).offset(-(core::mem::offset_of!(_HELPER_DATA, Helpers) as isize)) as PHELPER_DATA;

        if SockIsTripleInMapping(
            (*HelperData).Mapping,
            *AddressFamily,
            *SocketType,
            *Protocol,
        ) != 0
        {
            if (*HelperData).WSHOpenSocket2.is_none() {
                ((*HelperData).WSHOpenSocket.unwrap())(
                    AddressFamily,
                    SocketType,
                    Protocol,
                    TransportName,
                    HelperDllContext as *mut c_void,
                    Events,
                );
            } else {
                ((*HelperData).WSHOpenSocket2.unwrap())(
                    AddressFamily,
                    SocketType,
                    Protocol,
                    Group,
                    Flags,
                    TransportName,
                    HelperDllContext,
                    Events,
                );
            }

            *HelperDllData = HelperData;
            return NO_ERROR as INT;
        }

        Helpers = (*Helpers).Flink;
    }

    Status = SockLoadTransportList(core::ptr::addr_of_mut!(Transports));

    if Status != 0 {
        WARN!(b"Can't get transport list\n\0".as_ptr() as *const i8);
        return Status;
    }

    Transport = Transports;
    while *Transports != 0 {
        TRACE!(b"Transport: %S\n\0".as_ptr() as *const i8, Transports);

        Status = SockLoadTransportMapping(Transport, core::ptr::addr_of_mut!(Mapping));

        if Status != 0 {
            ERR!(b"Can't get mapping for %S\n\0".as_ptr() as *const i8, Transports);
            HeapFree(GlobalHeap, 0, Transports as *mut core::ffi::c_void);
            return Status;
        }

        if SockIsTripleInMapping(Mapping, *AddressFamily, *SocketType, *Protocol) != 0 {
            Status = SockLoadHelperDll(Transport, Mapping, core::ptr::addr_of_mut!(HelperData));

            if Status != 0 {
                ERR!(b"Can't load helper DLL for Transport %S.\n\0".as_ptr() as *const i8, Transport);
                HeapFree(GlobalHeap, 0, Transports as *mut core::ffi::c_void);
                HeapFree(GlobalHeap, 0, Mapping as *mut core::ffi::c_void);
                return Status;
            }

            if (*HelperData).WSHOpenSocket2.is_none() {
                ((*HelperData).WSHOpenSocket.unwrap())(
                    AddressFamily,
                    SocketType,
                    Protocol,
                    TransportName,
                    HelperDllContext as *mut c_void,
                    Events,
                );
            } else {
                ((*HelperData).WSHOpenSocket2.unwrap())(
                    AddressFamily,
                    SocketType,
                    Protocol,
                    Group,
                    Flags,
                    TransportName,
                    HelperDllContext,
                    Events,
                );
            }

            *HelperDllData = HelperData;
            return NO_ERROR as INT;
        }

        HeapFree(GlobalHeap, 0, Mapping as *mut core::ffi::c_void);
        Transport = Transport.offset(wcslen(Transport) as isize + 1);
    }

    HeapFree(GlobalHeap, 0, Transports as *mut core::ffi::c_void);
    return WSAEINVAL as INT;
}
#[no_mangle]
pub unsafe extern "C" fn SockLoadTransportMapping(
    TransportName: PWSTR,
    Mapping: *mut PWINSOCK_MAPPING,
) -> INT {

    TRACE!(b"TransportName %ws\n\0".as_ptr(), TransportName);

    let TransportKey = HeapAlloc(GlobalHeap, 0, (54 + wcslen(TransportName)) * core::mem::size_of::<WCHAR>()) as PWSTR;

    if TransportKey.is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr());
        return WSAEINVAL as INT;
    }

    wcscpy(TransportKey, b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0\0".as_ptr() as *const u16);
    wcscat(TransportKey, TransportName);
    wcscat(TransportKey, b"\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\\\0W\0i\0n\0s\0o\0c\0k\0\0".as_ptr() as *const u16);

    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let Status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, TransportKey as LPCWSTR, 0, KEY_READ, &mut KeyHandle);

    HeapFree(GlobalHeap, 0, TransportKey as *mut core::ffi::c_void);

    if Status != 0 {
        ERR!(b"Error reading transport mapping registry\n\0".as_ptr());
        return WSAEINVAL as INT;
    }

    let mut MappingSize: ULONG = 0;
    let Status = RegQueryValueExW(KeyHandle, b"M\0a\0p\0p\0i\0n\0g\0\0".as_ptr() as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), &mut MappingSize as PDWORD);

    if Status != 0 {
        ERR!(b"Error reading transport mapping registry\n\0".as_ptr());
        return WSAEINVAL as INT;
    }

    *Mapping = HeapAlloc(GlobalHeap, 0, MappingSize as usize) as PWINSOCK_MAPPING;

    if (*Mapping).is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr());
        return WSAEINVAL as INT;
    }

    let Status = RegQueryValueExW(KeyHandle, b"M\0a\0p\0p\0i\0n\0g\0\0".as_ptr() as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), *Mapping as LPBYTE, &mut MappingSize as PDWORD);

    if Status != 0 {
        ERR!(b"Error reading transport mapping registry\n\0".as_ptr());
        HeapFree(GlobalHeap, 0, *Mapping as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    RegCloseKey(KeyHandle);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn SockLoadTransportList(
    TransportList: *mut PWSTR
) -> INT {
    let mut TransportListSize: u32 = 0;
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut Status: i32;

    TRACE!(b"Called\n\0".as_ptr() as *const i8);

    Status = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0W\0i\0n\0s\0o\0c\0k\0\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0".as_ptr() as PWSTR,
        0,
        KEY_READ,
        &mut KeyHandle
    );

    if Status != 0 {
        ERR!(b"Error reading transport list registry\n\0".as_ptr() as *const i8);
        return WSAEINVAL as INT;
    }

    Status = RegQueryValueExW(
        KeyHandle,
        b"T\0r\0a\0n\0s\0p\0o\0r\0t\0s\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut TransportListSize
    );

    if Status != 0 {
        ERR!(b"Error reading transport list registry\n\0".as_ptr() as *const i8);
        return WSAEINVAL as INT;
    }

    *TransportList = HeapAlloc(GlobalHeap, 0, TransportListSize as usize) as PWSTR;

    if (*TransportList).is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
        return WSAEINVAL as INT;
    }

    Status = RegQueryValueExW(
        KeyHandle,
        b"T\0r\0a\0n\0s\0p\0o\0r\0t\0s\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        *TransportList as LPBYTE,
        &mut TransportListSize
    );

    if Status != 0 {
        ERR!(b"Error reading transport list registry\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, *TransportList as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    RegCloseKey(KeyHandle);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn SockLoadHelperDll(
    TransportName: PWSTR,
    Mapping: PWINSOCK_MAPPING,
    HelperDllData: *mut PHELPER_DATA,
) -> INT {

    let HelperData: PHELPER_DATA = HeapAlloc(
        GlobalHeap,
        0,
        core::mem::size_of::<HELPER_DATA>() + (wcslen(TransportName) + 1) * core::mem::size_of::<u16>(),
    ) as PHELPER_DATA;

    if HelperData.is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
        return WSAEINVAL as INT;
    }

    let HelperKey: PWSTR = HeapAlloc(
        GlobalHeap,
        0,
        (54 + wcslen(TransportName)) * core::mem::size_of::<u16>(),
    ) as PWSTR;

    if HelperKey.is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    wcscpy(HelperKey, b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0\0\0".as_ptr() as PWSTR);
    wcscat(HelperKey, TransportName);
    wcscat(HelperKey, b"\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\\\0W\0i\0n\0s\0o\0c\0k\0\0\0".as_ptr() as PWSTR);

    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut Status: i32 = RegOpenKeyExW(HKEY_LOCAL_MACHINE, HelperKey, 0, KEY_READ, &mut KeyHandle);

    HeapFree(GlobalHeap, 0, HelperKey as *mut core::ffi::c_void);

    if Status != 0 {
        ERR!(b"Error reading helper DLL parameters\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    let mut DataSize: u32 = core::mem::size_of::<u32>() as u32;
    (*HelperData).MinWSAddressLength = 16;
    RegQueryValueExW(
        KeyHandle,
        b"M\0i\0n\0S\0o\0c\0k\0a\0d\0d\0r\0L\0e\0n\0g\0t\0h\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*HelperData).MinWSAddressLength) as LPBYTE,
        &mut DataSize,
    );
    DataSize = core::mem::size_of::<u32>() as u32;
    (*HelperData).MaxWSAddressLength = 16;
    RegQueryValueExW(
        KeyHandle,
        b"M\0a\0x\0S\0o\0c\0k\0a\0d\0d\0r\0L\0e\0n\0g\0t\0h\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*HelperData).MaxWSAddressLength) as LPBYTE,
        &mut DataSize,
    );

    (*HelperData).MinTDIAddressLength = (*HelperData).MinWSAddressLength + 6;
    (*HelperData).MaxTDIAddressLength = (*HelperData).MaxWSAddressLength + 6;

    DataSize = core::mem::size_of::<u32>() as u32;
    (*HelperData).UseDelayedAcceptance = 0u8;
    RegQueryValueExW(
        KeyHandle,
        b"U\0s\0e\0D\0e\0l\0a\0y\0e\0d\0A\0c\0c\0e\0p\0t\0a\0n\0c\0e\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*HelperData).UseDelayedAcceptance) as LPBYTE,
        &mut DataSize,
    );

    let HelperDllName: PWSTR = HeapAlloc(GlobalHeap, 0, 512) as PWSTR;

    if HelperDllName.is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    let FullHelperDllName: PWSTR = HeapAlloc(GlobalHeap, 0, 512) as PWSTR;

    if FullHelperDllName.is_null() {
        ERR!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, HelperDllName as *mut core::ffi::c_void);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    DataSize = 512;
    Status = RegQueryValueExW(
        KeyHandle,
        b"H\0e\0l\0p\0e\0r\0D\0l\0l\0N\0a\0m\0e\0\0\0".as_ptr() as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        HelperDllName as LPBYTE,
        &mut DataSize,
    );

    if Status != 0 {
        ERR!(b"Error reading helper DLL parameters\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, FullHelperDllName as *mut core::ffi::c_void);
        HeapFree(GlobalHeap, 0, HelperDllName as *mut core::ffi::c_void);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    ExpandEnvironmentStringsW(HelperDllName, FullHelperDllName, 256);

    (*HelperData).hInstance = LoadLibraryW(FullHelperDllName) as HANDLE;

    HeapFree(GlobalHeap, 0, HelperDllName as *mut core::ffi::c_void);
    HeapFree(GlobalHeap, 0, FullHelperDllName as *mut core::ffi::c_void);

    if (*HelperData).hInstance.is_null() {
        ERR!(b"Error loading helper DLL\n\0".as_ptr() as *const i8);
        HeapFree(GlobalHeap, 0, HelperData as *mut core::ffi::c_void);
        return WSAEINVAL as INT;
    }

    RegCloseKey(KeyHandle);

    (*HelperData).WSHOpenSocket = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHOpenSocket\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHOpenSocket2 = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHOpenSocket2\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHJoinLeaf = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHJoinLeaf\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHNotify = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHNotify\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHGetSocketInformation = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHGetSocketInformation\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHSetSocketInformation = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHSetSocketInformation\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHGetSockaddrType = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHGetSockaddrType\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHGetWildcardSockaddr = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHGetWildcardSockaddr\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHGetBroadcastSockaddr = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHGetBroadcastSockaddr\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHAddressToString = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHAddressToString\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHStringToAddress = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHStringToAddress\0".as_ptr() as *const i8,
    ));
    (*HelperData).WSHIoctl = core::mem::transmute(GetProcAddress(
        (*HelperData).hInstance as HMODULE,
        b"WSHIoctl\0".as_ptr() as *const i8,
    ));

    (*HelperData).Mapping = Mapping;
    wcscpy((*HelperData).TransportName.as_mut_ptr(), TransportName);

    (*HelperData).RefCount = 1;

    InsertHeadList(&mut SockHelpersListHead, &mut (*HelperData).Helpers);

    *HelperDllData = HelperData;
    0
}
#[no_mangle]
pub unsafe extern "C" fn SockIsTripleInMapping(
    Mapping: PWINSOCK_MAPPING,
    AddressFamily: INT,
    SocketType: INT,
    Protocol: INT,
) -> BOOL {
    let mut Row: ULONG;

    TRACE!(b"Called, Mapping rows = %d\n\0".as_ptr() as PCHAR, (*Mapping).Rows);

    Row = 0;
    while Row < (*Mapping).Rows {
        TRACE!(
            b"Examining: row %d: AF %d type %d proto %d\n\0".as_ptr() as PCHAR,
            Row,
            (*Mapping).Mapping[Row as usize].AddressFamily as INT,
            (*Mapping).Mapping[Row as usize].SocketType as INT,
            (*Mapping).Mapping[Row as usize].Protocol as INT,
        );

        if ((*Mapping).Mapping[Row as usize].AddressFamily as INT == AddressFamily)
            && ((*Mapping).Mapping[Row as usize].SocketType as INT == SocketType)
            && ((*Mapping).Mapping[Row as usize].Protocol as INT == Protocol)
        {
            TRACE!(b"Found\n\0".as_ptr() as PCHAR);
            return 1;
        }
        Row += 1;
    }
    WARN!(b"Not found\n\0".as_ptr() as PCHAR);
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver DLL
//  * FILE:        dll/win32/msafd/misc/helpers.c
//  * PURPOSE:     Helper DLL management
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *				Alex Ionescu (alex@relsoft.net)
//  * REVISIONS:
//  *   CSH 01/09-2000 Created
//  *	 Alex 16/07/2004 - Complete Rewrite
//  */
//
// #include <msafd.h>
//
// #include <winreg.h>
//
// CRITICAL_SECTION HelperDLLDatabaseLock;
// LIST_ENTRY HelperDLLDatabaseListHead;
//
//
// INT
// SockGetTdiName(
//     PINT AddressFamily,
//     PINT SocketType,
//     PINT Protocol,
//     GROUP Group,
//     DWORD Flags,
//     PUNICODE_STRING TransportName,
//     PVOID *HelperDllContext,
//     PHELPER_DATA *HelperDllData,
//     PDWORD Events)
// {
//     PHELPER_DATA        HelperData;
//     PWSTR               Transports;
//     PWSTR               Transport;
//     PWINSOCK_MAPPING	Mapping;
//     PLIST_ENTRY	        Helpers;
//     INT                 Status;
//
//     TRACE("AddressFamily %p, SocketType %p, Protocol %p, Group %u, Flags %lx, TransportName %p, HelperDllContext %p, HelperDllData %p, Events %p\n",
//         AddressFamily, SocketType, Protocol, Group, Flags, TransportName, HelperDllContext, HelperDllData, Events);
//
//     /* Check in our Current Loaded Helpers */
//     for (Helpers = SockHelpersListHead.Flink;
//          Helpers != &SockHelpersListHead;
//          Helpers = Helpers->Flink ) {
//
//         HelperData = CONTAINING_RECORD(Helpers, HELPER_DATA, Helpers);
//
//         /* See if this Mapping works for us */
//         if (SockIsTripleInMapping (HelperData->Mapping,
//                                    *AddressFamily,
//                                    *SocketType,
//                                    *Protocol)) {
//
//             /* Call the Helper Dll function get the Transport Name */
//             if (HelperData->WSHOpenSocket2 == NULL ) {
//
//                 /* DLL Doesn't support WSHOpenSocket2, call the old one */
//                 HelperData->WSHOpenSocket(AddressFamily,
//                                           SocketType,
//                                           Protocol,
//                                           TransportName,
//                                           HelperDllContext,
//                                           Events
//                                           );
//             } else {
//                 HelperData->WSHOpenSocket2(AddressFamily,
//                                            SocketType,
//                                            Protocol,
//                                            Group,
//                                            Flags,
//                                            TransportName,
//                                            HelperDllContext,
//                                            Events
//                                            );
//             }
//
//             /* Return the Helper Pointers */
//             *HelperDllData = HelperData;
//             return NO_ERROR;
//         }
//     }
//
//     /* Get the Transports available */
//     Status = SockLoadTransportList(&Transports);
//
//     /* Check for error */
//     if (Status) {
//         WARN("Can't get transport list\n");
//         return Status;
//     }
//
//     /* Loop through each transport until we find one that can satisfy us */
//     for (Transport = Transports;
//          *Transports != 0;
//          Transport += wcslen(Transport) + 1) {
//         TRACE("Transport: %S\n", Transports);
//
//         /* See what mapping this Transport supports */
//         Status = SockLoadTransportMapping(Transport, &Mapping);
//
//         /* Check for error */
//         if (Status) {
//             ERR("Can't get mapping for %S\n", Transports);
//             HeapFree(GlobalHeap, 0, Transports);
//             return Status;
//         }
//
//         /* See if this Mapping works for us */
//         if (SockIsTripleInMapping(Mapping, *AddressFamily, *SocketType, *Protocol)) {
//
//             /* It does, so load the DLL associated with it */
//             Status = SockLoadHelperDll(Transport, Mapping, &HelperData);
//
//             /* Check for error */
//             if (Status) {
//                 ERR("Can't load helper DLL for Transport %S.\n", Transport);
//                 HeapFree(GlobalHeap, 0, Transports);
//                 HeapFree(GlobalHeap, 0, Mapping);
//                 return Status;
//             }
//
//             /* Call the Helper Dll function get the Transport Name */
//             if (HelperData->WSHOpenSocket2 == NULL) {
//                 /* DLL Doesn't support WSHOpenSocket2, call the old one */
//                 HelperData->WSHOpenSocket(AddressFamily,
//                                           SocketType,
//                                           Protocol,
//                                           TransportName,
//                                           HelperDllContext,
//                                           Events
//                                           );
//             } else {
//                 HelperData->WSHOpenSocket2(AddressFamily,
//                                            SocketType,
//                                            Protocol,
//                                            Group,
//                                            Flags,
//                                            TransportName,
//                                            HelperDllContext,
//                                            Events
//                                            );
//             }
//
//             /* Return the Helper Pointers */
//             *HelperDllData = HelperData;
//             /* We actually cache these ... the can't be freed yet */
//             /*HeapFree(GlobalHeap, 0, Transports);*/
//             /*HeapFree(GlobalHeap, 0, Mapping);*/
//             return NO_ERROR;
//         }
//
//         HeapFree(GlobalHeap, 0, Mapping);
//     }
//     HeapFree(GlobalHeap, 0, Transports);
//     return WSAEINVAL;
// }
//
// INT
// SockLoadTransportMapping(
//     PWSTR TransportName,
//     PWINSOCK_MAPPING *Mapping)
// {
//     PWSTR               TransportKey;
//     HKEY                KeyHandle;
//     ULONG               MappingSize;
//     LONG                Status;
//
//     TRACE("TransportName %ws\n", TransportName);
//
//     /* Allocate a Buffer */
//     TransportKey = HeapAlloc(GlobalHeap, 0, (54 + wcslen(TransportName)) * sizeof(WCHAR));
//
//     /* Check for error */
//     if (TransportKey == NULL) {
//         ERR("Buffer allocation failed\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Generate the right key name */
//     wcscpy(TransportKey, L"System\\CurrentControlSet\\Services\\");
//     wcscat(TransportKey, TransportName);
//     wcscat(TransportKey, L"\\Parameters\\Winsock");
//
//     /* Open the Key */
//     Status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, TransportKey, 0, KEY_READ, &KeyHandle);
//
//     /* We don't need the Transport Key anymore */
//     HeapFree(GlobalHeap, 0, TransportKey);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport mapping registry\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Find out how much space we need for the Mapping */
//     Status = RegQueryValueExW(KeyHandle, L"Mapping", NULL, NULL, NULL, &MappingSize);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport mapping registry\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Allocate Memory for the Mapping */
//     *Mapping = HeapAlloc(GlobalHeap, 0, MappingSize);
//
//     /* Check for error */
//     if (*Mapping == NULL) {
//         ERR("Buffer allocation failed\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Read the Mapping */
//     Status = RegQueryValueExW(KeyHandle, L"Mapping", NULL, NULL, (LPBYTE)*Mapping, &MappingSize);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport mapping registry\n");
//         HeapFree(GlobalHeap, 0, *Mapping);
//         return WSAEINVAL as INT;
//     }
//
//     /* Close key and return */
//     RegCloseKey(KeyHandle);
//     return 0;
// }
//
// INT
// SockLoadTransportList(
//     PWSTR *TransportList)
// {
//     ULONG	TransportListSize;
//     HKEY	KeyHandle;
//     LONG	Status;
//
//     TRACE("Called\n");
//
//     /* Open the Transports Key */
//     Status = RegOpenKeyExW (HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Services\\Winsock\\Parameters",
//                             0,
//                             KEY_READ,
//                             &KeyHandle);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport list registry\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Get the Transport List Size */
//     Status = RegQueryValueExW(KeyHandle,
//                               L"Transports",
//                               NULL,
//                               NULL,
//                               NULL,
//                               &TransportListSize);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport list registry\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Allocate Memory for the Transport List */
//     *TransportList = HeapAlloc(GlobalHeap, 0, TransportListSize);
//
//     /* Check for error */
//     if (*TransportList == NULL) {
//         ERR("Buffer allocation failed\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Get the Transports */
//     Status = RegQueryValueExW (KeyHandle,
//                                L"Transports",
//                                NULL,
//                                NULL,
//                                (LPBYTE)*TransportList,
//                                &TransportListSize);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading transport list registry\n");
//         HeapFree(GlobalHeap, 0, *TransportList);
//         return WSAEINVAL as INT;
//     }
//
//     /* Close key and return */
//     RegCloseKey(KeyHandle);
//     return 0;
// }
//
// INT
// SockLoadHelperDll(
//     PWSTR TransportName,
//     PWINSOCK_MAPPING Mapping,
//     PHELPER_DATA *HelperDllData)
// {
//     PHELPER_DATA        HelperData;
//     PWSTR               HelperDllName;
//     PWSTR               FullHelperDllName;
//     PWSTR               HelperKey;
//     HKEY                KeyHandle;
//     ULONG               DataSize;
//     LONG                Status;
//
//     /* Allocate space for the Helper Structure and TransportName */
//     HelperData = HeapAlloc(GlobalHeap, 0, sizeof(*HelperData) + (wcslen(TransportName) + 1) * sizeof(WCHAR));
//
//     /* Check for error */
//     if (HelperData == NULL) {
//         ERR("Buffer allocation failed\n");
//         return WSAEINVAL as INT;
//     }
//
//     /* Allocate Space for the Helper DLL Key */
//     HelperKey = HeapAlloc(GlobalHeap, 0, (54 + wcslen(TransportName)) * sizeof(WCHAR));
//
//     /* Check for error */
//     if (HelperKey == NULL) {
//         ERR("Buffer allocation failed\n");
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     /* Generate the right key name */
//     wcscpy(HelperKey, L"System\\CurrentControlSet\\Services\\");
//     wcscat(HelperKey, TransportName);
//     wcscat(HelperKey, L"\\Parameters\\Winsock");
//
//     /* Open the Key */
//     Status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, HelperKey, 0, KEY_READ, &KeyHandle);
//
//     HeapFree(GlobalHeap, 0, HelperKey);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading helper DLL parameters\n");
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     /* Read Size of SockAddr Structures */
//     DataSize = sizeof(HelperData->MinWSAddressLength);
//     HelperData->MinWSAddressLength = 16;
//     RegQueryValueExW (KeyHandle,
//                       L"MinSockaddrLength",
//                       NULL,
//                       NULL,
//                       (LPBYTE)&HelperData->MinWSAddressLength,
//                       &DataSize);
//     DataSize = sizeof(HelperData->MinWSAddressLength);
//     HelperData->MaxWSAddressLength = 16;
//     RegQueryValueExW (KeyHandle,
//                       L"MaxSockaddrLength",
//                       NULL,
//                       NULL,
//                       (LPBYTE)&HelperData->MaxWSAddressLength,
//                       &DataSize);
//
//     /* Size of TDI Structures */
//     HelperData->MinTDIAddressLength = HelperData->MinWSAddressLength + 6;
//     HelperData->MaxTDIAddressLength = HelperData->MaxWSAddressLength + 6;
//
//     /* Read Delayed Acceptance Setting */
//     DataSize = sizeof(DWORD);
//     HelperData->UseDelayedAcceptance = FALSE;
//     RegQueryValueExW (KeyHandle,
//                       L"UseDelayedAcceptance",
//                       NULL,
//                       NULL,
//                       (LPBYTE)&HelperData->UseDelayedAcceptance,
//                       &DataSize);
//
//     /* Allocate Space for the Helper DLL Names */
//     HelperDllName = HeapAlloc(GlobalHeap, 0, 512);
//
//     /* Check for error */
//     if (HelperDllName == NULL) {
//         ERR("Buffer allocation failed\n");
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     FullHelperDllName = HeapAlloc(GlobalHeap, 0, 512);
//
//     /* Check for error */
//     if (FullHelperDllName == NULL) {
//         ERR("Buffer allocation failed\n");
//         HeapFree(GlobalHeap, 0, HelperDllName);
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     /* Get the name of the Helper DLL*/
//     DataSize = 512;
//     Status = RegQueryValueExW (KeyHandle,
//                                L"HelperDllName",
//                                NULL,
//                                NULL,
//                                (LPBYTE)HelperDllName,
//                                &DataSize);
//
//     /* Check for error */
//     if (Status) {
//         ERR("Error reading helper DLL parameters\n");
//         HeapFree(GlobalHeap, 0, FullHelperDllName);
//         HeapFree(GlobalHeap, 0, HelperDllName);
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     /* Get the Full name, expanding Environment Strings */
//     ExpandEnvironmentStringsW (HelperDllName,
//                                FullHelperDllName,
//                                256);
//
//     /* Load the DLL */
//     HelperData->hInstance = LoadLibraryW(FullHelperDllName);
//
//     HeapFree(GlobalHeap, 0, HelperDllName);
//     HeapFree(GlobalHeap, 0, FullHelperDllName);
//
//     if (HelperData->hInstance == NULL) {
//         ERR("Error loading helper DLL\n");
//         HeapFree(GlobalHeap, 0, HelperData);
//         return WSAEINVAL as INT;
//     }
//
//     /* Close Key */
//     RegCloseKey(KeyHandle);
//
//     /* Get the Pointers to the Helper Routines */
//     HelperData->WSHOpenSocket =	(PWSH_OPEN_SOCKET)
// 									GetProcAddress(HelperData->hInstance,
// 									"WSHOpenSocket");
//     HelperData->WSHOpenSocket2 = (PWSH_OPEN_SOCKET2)
// 									GetProcAddress(HelperData->hInstance,
// 									"WSHOpenSocket2");
//     HelperData->WSHJoinLeaf = (PWSH_JOIN_LEAF)
// 								GetProcAddress(HelperData->hInstance,
// 								"WSHJoinLeaf");
//     HelperData->WSHNotify = (PWSH_NOTIFY)
// 								GetProcAddress(HelperData->hInstance, "WSHNotify");
//     HelperData->WSHGetSocketInformation = (PWSH_GET_SOCKET_INFORMATION)
// 											GetProcAddress(HelperData->hInstance,
// 											"WSHGetSocketInformation");
//     HelperData->WSHSetSocketInformation = (PWSH_SET_SOCKET_INFORMATION)
// 											GetProcAddress(HelperData->hInstance,
// 											"WSHSetSocketInformation");
//     HelperData->WSHGetSockaddrType = (PWSH_GET_SOCKADDR_TYPE)
// 										GetProcAddress(HelperData->hInstance,
// 										"WSHGetSockaddrType");
//     HelperData->WSHGetWildcardSockaddr = (PWSH_GET_WILDCARD_SOCKADDR)
// 											GetProcAddress(HelperData->hInstance,
// 											"WSHGetWildcardSockaddr");
//     HelperData->WSHGetBroadcastSockaddr = (PWSH_GET_BROADCAST_SOCKADDR)
// 											GetProcAddress(HelperData->hInstance,
// 											"WSHGetBroadcastSockaddr");
//     HelperData->WSHAddressToString = (PWSH_ADDRESS_TO_STRING)
// 										GetProcAddress(HelperData->hInstance,
// 										"WSHAddressToString");
//     HelperData->WSHStringToAddress = (PWSH_STRING_TO_ADDRESS)
// 										GetProcAddress(HelperData->hInstance,
// 										"WSHStringToAddress");
//     HelperData->WSHIoctl = (PWSH_IOCTL)
// 							GetProcAddress(HelperData->hInstance,
// 							"WSHIoctl");
//
//     /* Save the Mapping Structure and transport name */
//     HelperData->Mapping = Mapping;
//     wcscpy(HelperData->TransportName, TransportName);
//
//     /* Increment Reference Count */
//     HelperData->RefCount = 1;
//
//     /* Add it to our list */
//     InsertHeadList(&SockHelpersListHead, &HelperData->Helpers);
//
//     /* Return Pointers */
//     *HelperDllData = HelperData;
//     return 0;
// }
//
// BOOL
// SockIsTripleInMapping(
//     PWINSOCK_MAPPING Mapping,
//     INT AddressFamily,
//     INT SocketType,
//     INT Protocol)
// {
//     /* The Windows version returns more detailed information on which of the 3 parameters failed...we should do this later */
//     ULONG    Row;
//
//     TRACE("Called, Mapping rows = %d\n", Mapping->Rows);
//
//     /* Loop through Mapping to Find a matching one */
//     for (Row = 0; Row < Mapping->Rows; Row++) {
//         TRACE("Examining: row %d: AF %d type %d proto %d\n",
// 				Row,
// 				(INT)Mapping->Mapping[Row].AddressFamily,
// 				(INT)Mapping->Mapping[Row].SocketType,
// 				(INT)Mapping->Mapping[Row].Protocol);
//
//         /* Check of all three values Match */
//         if (((INT)Mapping->Mapping[Row].AddressFamily == AddressFamily) &&
//             ((INT)Mapping->Mapping[Row].SocketType == SocketType) &&
//             ((INT)Mapping->Mapping[Row].Protocol == Protocol)) {
//             TRACE("Found\n");
//             return TRUE;
//         }
//     }
//     WARN("Not found\n");
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
