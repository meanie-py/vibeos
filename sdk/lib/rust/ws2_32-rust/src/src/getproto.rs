use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dthread::WsThreadGetProtoBuffer;


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

// No file-scope declarations to translate from this preamble
#[no_mangle]
pub unsafe extern "system" fn GetProtoOpenNetworkDatabase(Name: PCHAR) -> HANDLE {

    let mut ErrorCode: INT;
    let mut DatabaseKey: HKEY = core::ptr::null_mut();
    let mut RegType: DWORD = 0;
    let mut RegSize: DWORD = 0;
    let ret: HANDLE;

    let ExpandedPath = HeapAlloc(WsSockHeap, 0, MAX_PATH as SIZE_T) as PCHAR;
    if ExpandedPath.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    ErrorCode = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        b"System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0".as_ptr() as LPCSTR,
        0,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(DatabaseKey),
    ) as INT;

    // Divergence from the C: with Tcpip\Parameters present (a configured NIC) but no
    // DatabasePath value (nothing in our hives sets one), it expanded an uninitialised
    // zero-byte block as the directory, so getprotobyname() depended on whether the NIC had
    // been configured yet. Any failure falls back to the default directory, as in mswsock's
    // OpenNetworkDatabase.
    let mut have_path = false;
    if ErrorCode == (NO_ERROR as i32) {
        ErrorCode = RegQueryValueExA(
            DatabaseKey,
            b"DatabasePath\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegSize),
        ) as INT;

        if ErrorCode == (NO_ERROR as i32) && RegSize != 0 {
            // One byte more than the value, for a terminator the registry need not store.
            let DatabasePath = HeapAlloc(WsSockHeap, 0, (RegSize + 1) as SIZE_T) as PCHAR;
            if DatabasePath.is_null() {
                RegCloseKey(DatabaseKey);
                HeapFree(WsSockHeap, 0, ExpandedPath as PVOID);
                return INVALID_HANDLE_VALUE;
            }

            ErrorCode = RegQueryValueExA(
                DatabaseKey,
                b"DatabasePath\0".as_ptr() as LPCSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(RegType),
                DatabasePath as LPBYTE,
                core::ptr::addr_of_mut!(RegSize),
            ) as INT;
            if ErrorCode == (NO_ERROR as i32) {
                *DatabasePath.add(RegSize as usize) = 0;
                let n = ExpandEnvironmentStringsA(
                    DatabasePath as LPCSTR,
                    ExpandedPath as LPSTR,
                    MAX_PATH as DWORD,
                );
                have_path = n != 0 && n <= MAX_PATH as DWORD && *ExpandedPath != 0;
            }

            HeapFree(WsSockHeap, 0, DatabasePath as PVOID);
        }

        RegCloseKey(DatabaseKey);
    }
    if !have_path {
        GetSystemDirectoryA(ExpandedPath as LPSTR, MAX_PATH as UINT);
        if *ExpandedPath.offset(
            (strlen(ExpandedPath as *const core::ffi::c_char) as core::ffi::c_ulonglong)
                .wrapping_sub(1) as isize,
        ) as core::ffi::c_int
            != '\\' as i32
        {
            strncat(
                ExpandedPath as *mut core::ffi::c_char,
                b"\\\0".as_ptr() as *const core::ffi::c_char,
                MAX_PATH as size_t,
            );
        }
        strncat(
            ExpandedPath as *mut core::ffi::c_char,
            b"DRIVERS\\ETC\\\0".as_ptr() as *const core::ffi::c_char,
            MAX_PATH as size_t,
        );
    }

    if *ExpandedPath.offset(
        (strlen(ExpandedPath as *const core::ffi::c_char) as core::ffi::c_ulonglong)
            .wrapping_sub(1) as isize,
    ) as core::ffi::c_int
        != '\\' as i32
    {
        strncat(
            ExpandedPath as *mut core::ffi::c_char,
            b"\\\0".as_ptr() as *const core::ffi::c_char,
            MAX_PATH as size_t,
        );
    }

    strncat(
        ExpandedPath as *mut core::ffi::c_char,
        Name as *const core::ffi::c_char,
        MAX_PATH as size_t,
    );

    ret = CreateFileA(
        ExpandedPath as LPCSTR,
        FILE_READ_ACCESS as DWORD,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    );

    HeapFree(WsSockHeap, 0, ExpandedPath as PVOID);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn GetProtoPatternMatch(mut Buffer: PCHAR, Lookup: PCHAR) -> PCHAR {
    let mut ScanChar: CHAR;
    loop {
        ScanChar = *Buffer;
        if ScanChar == 0 {
            break;
        }
        if !strchr(Lookup as *const i8, ScanChar as i32).is_null() {
            return Buffer;
        }
        Buffer = Buffer.offset(1);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn GetProtoGetNextEnt(
    mut DbHandle: HANDLE,
    mut Buffer: PWSPROTO_BUFFER,
) -> PPROTOENT {
    let mut Read: DWORD = 0;
    let mut n: LONG;
    let mut p: PCHAR;
    let mut p1: PCHAR;
    let mut Entry: PCHAR;
    let mut Aliases: *mut PCHAR;
    let mut ReturnedProtoent: PPROTOENT = core::ptr::null_mut();

    n = SetFilePointer(DbHandle, 0, core::ptr::null_mut(), FILE_CURRENT) as LONG;

    loop {
        if ReadFile(
            DbHandle,
            core::ptr::addr_of_mut!((*Buffer).LineBuffer) as LPVOID,
            (core::mem::size_of::<[CHAR; 512]>() - 1) as DWORD,
            core::ptr::addr_of_mut!(Read),
            core::ptr::null_mut(),
        ) == 0
        {
            return core::ptr::null_mut();
        }

        p1 = core::ptr::addr_of_mut!((*Buffer).LineBuffer) as PCHAR;
        (*Buffer).LineBuffer[Read as usize] = 0;
        p = strchr(core::ptr::addr_of_mut!((*Buffer).LineBuffer) as *const i8, '\n' as i32) as PCHAR;

        if p.is_null() {
            return core::ptr::null_mut();
        }

        n += (p.offset_from(p1) as LONG) + 1;

        SetFilePointer(DbHandle, n, core::ptr::null_mut(), FILE_BEGIN);

        *p = 0;

        if *p1 as i32 == '#' as i32 {
            continue;
        }

        Entry = GetProtoPatternMatch(p1, b"#\n\0".as_ptr() as PCHAR);
        if Entry.is_null() {
            continue;
        }
        *Entry = 0;

        (*Buffer).Protoent.p_name = p1 as *mut i8;

        Entry = GetProtoPatternMatch(p1, b" \t\0".as_ptr() as PCHAR);
        if Entry.is_null() {
            continue;
        }
        let fresh0 = Entry;
        Entry = Entry.offset(1);
        *fresh0 = 0;

        while *Entry as i32 == ' ' as i32 || *Entry as i32 == '\t' as i32 {
            Entry = Entry.offset(1);
        }

        p1 = GetProtoPatternMatch(Entry, b" \t\0".as_ptr() as PCHAR);
        if !p1.is_null() {
            let fresh1 = p1;
            p1 = p1.offset(1);
            *fresh1 = 0;
        }

        (*Buffer).Protoent.p_proto = atoi(Entry as *const i8) as i16;

        (*Buffer).Protoent.p_aliases = core::ptr::addr_of_mut!((*Buffer).Aliases) as *mut *mut i8;
        Aliases = (*Buffer).Protoent.p_aliases as *mut PCHAR;

        if !p1.is_null() {
            Entry = p1;

            while !Entry.is_null() && *Entry as i32 != 0 {
                while *Entry as i32 == ' ' as i32 || *Entry as i32 == '\t' as i32 {
                    Entry = Entry.offset(1);
                }

                if Aliases < (*Buffer).Protoent.p_aliases.offset((MAXALIASES - 1) as isize) as *mut PCHAR {
                    let fresh2 = Aliases;
                    Aliases = Aliases.offset(1);
                    *fresh2 = Entry;
                }

                Entry = GetProtoPatternMatch(Entry, b" \t\0".as_ptr() as PCHAR);
                if !Entry.is_null() {
                    let fresh3 = Entry;
                    Entry = Entry.offset(1);
                    *fresh3 = 0;
                }
            }
        }

        *Aliases = core::ptr::null_mut();

        ReturnedProtoent = core::ptr::addr_of_mut!((*Buffer).Protoent) as PPROTOENT;
        break;
    }

    return ReturnedProtoent;
}

#[no_mangle]
pub unsafe extern "system" fn getprotobynumber(number: INT) -> LPPROTOENT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Protoent: PPROTOENT;
    let mut GetProtoBuffer: PVOID;
    let mut DbHandle: HANDLE;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return core::ptr::null_mut();
    }

    GetProtoBuffer = WsThreadGetProtoBuffer(Thread) as PVOID;
    if GetProtoBuffer.is_null() {
        SetLastError(WSANO_DATA as DWORD);
        return core::ptr::null_mut();
    }

    DbHandle = GetProtoOpenNetworkDatabase(b"protocol\0".as_ptr() as PCHAR);
    if DbHandle == INVALID_HANDLE_VALUE {
        SetLastError(WSANO_DATA as DWORD);
        return core::ptr::null_mut();
    }

    loop {
        Protoent = GetProtoGetNextEnt(DbHandle, GetProtoBuffer as PWSPROTO_BUFFER);
        if Protoent.is_null() {
            break;
        }
        if (*Protoent).p_proto as INT == number {
            break;
        }
    }

    CloseHandle(DbHandle);

    if Protoent.is_null() {
        SetLastError(WSANO_DATA as DWORD);
    }

    Protoent
}

#[no_mangle]
pub unsafe extern "system" fn getprotobyname(name: *const CHAR) -> LPPROTOENT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Protoent: PPROTOENT;
    let mut GetProtoBuffer: PVOID;
    let mut DbHandle: HANDLE;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return core::ptr::null_mut();
    }

    GetProtoBuffer = WsThreadGetProtoBuffer(Thread) as PVOID;
    if GetProtoBuffer.is_null() {
        SetLastError(WSANO_DATA as DWORD);
        return core::ptr::null_mut();
    }

    DbHandle = GetProtoOpenNetworkDatabase(b"protocol\0".as_ptr() as PCHAR);
    if DbHandle == INVALID_HANDLE_VALUE {
        SetLastError(WSANO_DATA as DWORD);
        return core::ptr::null_mut();
    }

    loop {
        Protoent = GetProtoGetNextEnt(DbHandle, GetProtoBuffer as PWSPROTO_BUFFER);
        if Protoent.is_null() {
            break;
        }
        if _stricmp((*Protoent).p_name, name) == 0 {
            break;
        }
    }

    CloseHandle(DbHandle);

    if Protoent.is_null() {
        SetLastError(WSANO_DATA as DWORD);
    }

    Protoent
}
// enforce_import_abi: moved 3 import decl(s) into the extern block their
// exporting DLL's .spec requires: _stricmp->C, strlen->C, strncat->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/getproto.c
//  * PURPOSE:     GetProtoByY Functions.
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
// /* FUNCTIONS *****************************************************************/
//
// HANDLE
// __stdcall
// GetProtoOpenNetworkDatabase(PCHAR Name)
// {
//     PCHAR ExpandedPath;
//     PCHAR DatabasePath;
//     INT ErrorCode;
//     HKEY DatabaseKey;
//     DWORD RegType;
//     DWORD RegSize = 0;
//     HANDLE ret;
//
//     ExpandedPath = HeapAlloc(WsSockHeap, 0, MAX_PATH);
//     if (!ExpandedPath)
//         return INVALID_HANDLE_VALUE;
//
//     /* Open the database path key */
//     ErrorCode = RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                              "System\\CurrentControlSet\\Services\\Tcpip\\Parameters",
//                              0,
//                              KEY_READ,
//                              &DatabaseKey);
//     if (ErrorCode == NO_ERROR)
//     {
//         /* Read the actual path */
//         ErrorCode = RegQueryValueEx(DatabaseKey,
//                                     "DatabasePath",
//                                     NULL,
//                                     &RegType,
//                                     NULL,
//                                     &RegSize);
//
//         DatabasePath = HeapAlloc(WsSockHeap, 0, RegSize);
//         if (!DatabasePath)
//         {
//             HeapFree(WsSockHeap, 0, ExpandedPath);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         /* Read the actual path */
//         ErrorCode = RegQueryValueEx(DatabaseKey,
//                                     "DatabasePath",
//                                     NULL,
//                                     &RegType,
//                                     (LPBYTE)DatabasePath,
//                                     &RegSize);
//
//
//         /* Close the key */
//         RegCloseKey(DatabaseKey);
//
//         /* Expand the name */
//         ExpandEnvironmentStrings(DatabasePath, ExpandedPath, MAX_PATH);
//
//         HeapFree(WsSockHeap, 0, DatabasePath);
//     }
//     else
//     {
//         /* Use default path */
//         GetSystemDirectory(ExpandedPath, MAX_PATH);
//         if (ExpandedPath[strlen(ExpandedPath) - 1] != '\\')
//         {
//             /* It isn't, so add it ourselves */
//             strncat(ExpandedPath, "\\", MAX_PATH);
//         }
//         strncat(ExpandedPath, "DRIVERS\\ETC\\", MAX_PATH);
//     }
//
//     /* Make sure that the path is backslash-terminated */
//     if (ExpandedPath[strlen(ExpandedPath) - 1] != '\\')
//     {
//         /* It isn't, so add it ourselves */
//         strncat(ExpandedPath, "\\", MAX_PATH);
//     }
//
//     /* Add the database name */
//     strncat(ExpandedPath, Name, MAX_PATH);
//
//     /* Return a handle to the file */
//     ret = CreateFile(ExpandedPath,
//                       FILE_READ_ACCESS,
//                       0,
//                       NULL,
//                       OPEN_EXISTING,
//                       FILE_ATTRIBUTE_NORMAL,
//                       NULL);
//
//     HeapFree(WsSockHeap, 0, ExpandedPath);
//     return ret;
// }
//
// PCHAR
// __stdcall
// GetProtoPatternMatch(IN PCHAR Buffer,
//                      IN PCHAR Lookup)
// {
//     CHAR ScanChar;
//
//     /* Loop as long as we have data */
//     while ((ScanChar = *Buffer))
//     {
//         /* Check for a match and return its pointer if found */
//         if (strchr(Lookup, ScanChar)) return Buffer;
//
//         /* Keep going */
//         Buffer++;
//     }
//
//     /* Nothing found */
//     return NULL;
// }
//
// PPROTOENT
// __stdcall
// GetProtoGetNextEnt(IN HANDLE DbHandle,
//                    IN PWSPROTO_BUFFER Buffer)
// {
//     DWORD Read;
//     LONG n;
//     PCHAR p, p1, Entry, *Aliases;
//     PPROTOENT ReturnedProtoent;
//
//     /* Find out where we currently are in the file */
//     n = SetFilePointer(DbHandle, 0, 0, FILE_CURRENT);
//
//     while (TRUE)
//     {
//         /* Read 512 bytes */
//         if (!ReadFile(DbHandle,
//                       Buffer->LineBuffer,
//                       sizeof(Buffer->LineBuffer) - 1,
//                       &Read,
//                       NULL)) return NULL;
//
//         /* Find out where the line ends */
//         p1 = Buffer->LineBuffer;
//         Buffer->LineBuffer[Read] = ANSI_NULL;
//         p = strchr(Buffer->LineBuffer, '\n');
//
//         /* Bail out if the file is parsed */
//         if (!p) return NULL;
//
//         /* Calculate our new position */
//         n += (LONG)(p - p1) + 1;
//
//         /* Make it so we read from there next time */
//         SetFilePointer(DbHandle, n, 0, FILE_BEGIN);
//
//         /* Null-terminate the buffer so it only contains this line */
//         *p = ANSI_NULL;
//
//         /* If this line is a comment, skip it */
//         if (*p1 == '#') continue;
//
//         /* Get the entry in this line and null-terminate it */
//         Entry = GetProtoPatternMatch(p1, "#\n");
//         if (!Entry) continue;
//         *Entry = ANSI_NULL;
//
//         /* Start with the name */
//         Buffer->Protoent.p_name = p1;
//
//         /* Get the first tab and null-terminate */
//         Entry = GetProtoPatternMatch(p1, " \t");
//         if (!Entry) continue;
//         *Entry++ = ANSI_NULL;
//
//         /* Handle remaining tabs or spaces */
//         while (*Entry == ' ' || *Entry == '\t') Entry++;
//
//         /* Now move our read pointer */
//         p1 = GetProtoPatternMatch(Entry, " \t");
//         if (p1) *p1++ = ANSI_NULL;
//
//         /* This is where the address is */
//         Buffer->Protoent.p_proto = (short)atoi(Entry);
//
//         /* Setup the alias buffer */
//         Buffer->Protoent.p_aliases = Buffer->Aliases;
//         Aliases = Buffer->Protoent.p_aliases;
//
//         /* Check if the pointer is stil valid */
//         if (p1)
//         {
//             /* The first entry is here */
//             Entry = p1;
//
//             /* Loop while there are non-null entries */
//             while (Entry && *Entry)
//             {
//                 /* Handle tabs and spaces */
//                 while (*Entry == ' ' || *Entry == '\t') Entry++;
//
//                 /* Make sure we don't go over the buffer */
//                 if (Aliases < &Buffer->Protoent.p_aliases[MAXALIASES - 1])
//                 {
//                     /* Write the alias */
//                     *Aliases++ = Entry;
//                 }
//
//                 /* Get to the next entry */
//                 Entry = GetProtoPatternMatch(Entry, " \t");
//                 if (Entry) *Entry++ = ANSI_NULL;
//             }
//         }
//
//         /* Terminate the list */
//         *Aliases = NULL;
//
//         /* Return to caller */
//         ReturnedProtoent = &Buffer->Protoent;
//         break;
//     }
//
//     /* Return whatever we got */
//     return ReturnedProtoent;
// }
//
// /*
//  * @implemented
//  */
// LPPROTOENT
// __stdcall
// getprotobynumber(IN INT number)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PPROTOENT Protoent;
//     PVOID GetProtoBuffer;
//     HANDLE DbHandle;
//     DPRINT("getprotobynumber: %lx\n", number);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Get our buffer */
//     GetProtoBuffer = WsThreadGetProtoBuffer(Thread);
//     if (!GetProtoBuffer)
//     {
//         /* Fail */
//         SetLastError(WSANO_DATA);
//         return NULL;
//     }
//
//     /* Open the network database */
//     DbHandle = GetProtoOpenNetworkDatabase("protocol");
//     if (DbHandle == INVALID_HANDLE_VALUE)
//     {
//         /* Couldn't open the DB; fail */
//         SetLastError(WSANO_DATA);
//         return NULL;
//     }
//
//     /* Start the scan loop */
//     while (TRUE)
//     {
//         /* Get a protoent entry */
//         Protoent = GetProtoGetNextEnt(DbHandle, GetProtoBuffer);
//
//         /* Break if we didn't get any new one */
//         if (!Protoent) break;
//
//         /* Break if we have a match */
//         if (Protoent->p_proto == number) break;
//     }
//
//     /* Close the network database */
//     CloseHandle(DbHandle);
//
//     /* Set error if we don't have a protoent */
//     if (!Protoent) SetLastError(WSANO_DATA);
//
//     /* Return it */
//     return Protoent;
// }
//
// /*
//  * @implemented
//  */
// LPPROTOENT
// __stdcall
// getprotobyname(IN CONST CHAR  *name)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PPROTOENT Protoent;
//     PVOID GetProtoBuffer;
//     HANDLE DbHandle;
//     DPRINT("getprotobyname: %s\n", name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Get our buffer */
//     GetProtoBuffer = WsThreadGetProtoBuffer(Thread);
//     if (!GetProtoBuffer)
//     {
//         /* Fail */
//         SetLastError(WSANO_DATA);
//         return NULL;
//     }
//
//     /* Open the network database */
//     DbHandle = GetProtoOpenNetworkDatabase("protocol");
//     if (DbHandle == INVALID_HANDLE_VALUE)
//     {
//         /* Couldn't open the DB; fail */
//         SetLastError(WSANO_DATA);
//         return NULL;
//     }
//
//     /* Start the scan loop */
//     while (TRUE)
//     {
//         /* Get a protoent entry */
//         Protoent = GetProtoGetNextEnt(DbHandle, GetProtoBuffer);
//
//         /* Break if we didn't get any new one */
//         if (!Protoent) break;
//
//         /* Break if we have a match */
//         if (!_stricmp(Protoent->p_name, name)) break;
//     }
//
//     /* Close the network database */
//     CloseHandle(DbHandle);
//
//     /* Set error if we don't have a protoent */
//     if (!Protoent) SetLastError(WSANO_DATA);
//
//     /* Return it */
//     return Protoent;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
