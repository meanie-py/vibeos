use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szHexChar: [u16; 17] = [
    0x30u16, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
    0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
    0x00,
];
unsafe extern "C" fn AnsiToUnicode(NarrowString: PSTR) -> PWSTR {
    let WideLen = MultiByteToWideChar(0, 0, NarrowString, -1, core::ptr::null_mut(), 0);
    if WideLen == 0 {
        return core::ptr::null_mut();
    }

    let WideString = HeapAlloc(
        GetProcessHeap(),
        0,
        ((WideLen as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;

    if WideString.is_null() {
        return core::ptr::null_mut();
    }

    MultiByteToWideChar(0, 0, NarrowString, -1, WideString, WideLen);

    WideString
}
unsafe extern "C" fn ParseIpv4Address(address_string: PCSTR, p_address: PIN_ADDR) -> u8 {
    let mut p_terminator: *const i8 = core::ptr::null();
    let status = RtlIpv4StringToAddressA(
        address_string,
        1u8,
        &raw mut p_terminator,
        p_address,
    );
    if status >= 0 && !p_terminator.is_null() && *p_terminator == 0 {
        return 1u8;
    }
    0u8
}
unsafe extern "C" fn ParseIpv6Address(AddressString: PCSTR, pAddress: PIN6_ADDR) -> u8 {
    let mut pTerminator: PCSTR = core::ptr::null();
    let Status = RtlIpv6StringToAddressA(AddressString, &mut pTerminator, pAddress);
    if NT_SUCCESS(Status) && !pTerminator.is_null() && *pTerminator == 0 {
        return 1u8;
    }
    0u8
}
unsafe extern "C" fn AddIpv4HostEntries(pszHostName: PWSTR, pAddress: PIN_ADDR) {
    let mut ARecord: _DNS_RECORDW = core::mem::zeroed();
    ZeroMemory(
        &mut ARecord as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<_DNS_RECORDW>(),
    );
    ARecord.pName = pszHostName;
    ARecord.wType = (DNS_TYPE_A) as u16;
    ARecord.wDataLength = core::mem::size_of::<_DNS_A_DATA>() as u16;
    ARecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
    ARecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
    ARecord.dwTtl = 86400;
    ARecord.Data.A.as_mut().IpAddress = (*pAddress).S_un.S_addr;

    let mut szReverseName: [u16; 32] = [0u16; 32];
    let format_wide: [u16; 23] = [
        b'%' as u16,
        b'u' as u16,
        b'.' as u16,
        b'u' as u16,
        b'.' as u16,
        b'u' as u16,
        b'.' as u16,
        b'u' as u16,
        b'.' as u16,
        b'i' as u16,
        b'n' as u16,
        b'-' as u16,
        b'a' as u16,
        b'd' as u16,
        b'd' as u16,
        b'r' as u16,
        b'.' as u16,
        b'a' as u16,
        b'r' as u16,
        b'p' as u16,
        b'a' as u16,
        b'.' as u16,
        0u16,
    ];
    swprintf(
        szReverseName.as_mut_ptr(),
        format_wide.as_ptr(),
        (*pAddress).S_un.S_un_b.s_b4 as u32,
        (*pAddress).S_un.S_un_b.s_b3 as u32,
        (*pAddress).S_un.S_un_b.s_b2 as u32,
        (*pAddress).S_un.S_un_b.s_b1 as u32,
    );

    let mut PtrRecord: _DNS_RECORDW = core::mem::zeroed();
    ZeroMemory(
        &mut PtrRecord as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<_DNS_RECORDW>(),
    );
    PtrRecord.pName = szReverseName.as_mut_ptr();
    PtrRecord.wType = (DNS_TYPE_PTR) as u16;
    PtrRecord.wDataLength = core::mem::size_of::<_DNS_PTR_DATA>() as u16;
    PtrRecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
    PtrRecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
    PtrRecord.dwTtl = 86400;
    PtrRecord.Data.PTR.as_mut().pNameHost = pszHostName;

    DnsIntCacheAddEntry(&mut ARecord as *mut _DNS_RECORDW, 1);
    DnsIntCacheAddEntry(&mut PtrRecord as *mut _DNS_RECORDW, 1);
}
unsafe extern "C" fn AddIpv6HostEntries(pszHostName: PWSTR, pAddress: PIN6_ADDR) {
    let mut AAAARecord: DNS_RECORDW = core::mem::zeroed();
    ZeroMemory(
        &mut AAAARecord as *mut _ as _,
        core::mem::size_of::<DNS_RECORDW>(),
    );

    AAAARecord.pName = pszHostName;
    AAAARecord.wType = (DNS_TYPE_AAAA) as u16;
    AAAARecord.wDataLength = core::mem::size_of::<DNS_AAAA_DATA>() as u16;
    AAAARecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
    AAAARecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
    AAAARecord.dwTtl = 86400;

    CopyMemory(
        core::ptr::addr_of_mut!(AAAARecord.Data.AAAA.as_mut().Ip6Address) as *mut _,
        core::ptr::addr_of!((*pAddress).u.Byte) as *const _,
        core::mem::size_of::<IN6_ADDR>(),
    );

    let mut szReverseName: [u16; 80] = [0u16; 80];
    ZeroMemory(
        szReverseName.as_mut_ptr() as *mut _,
        core::mem::size_of::<[u16; 80]>(),
    );

    let mut i = 0u32;
    let mut j: u32;
    let mut k: u32;
    while i < 16 {
        j = 4u32.wrapping_mul(i);
        k = 15u32.wrapping_sub(i);
        szReverseName[j as usize] =
            szHexChar[((*pAddress).u.Byte[k as usize] & 0xF) as usize];
        szReverseName[j as usize + 1] = b'.' as u16;
        szReverseName[j as usize + 2] =
            szHexChar[(((*pAddress).u.Byte[k as usize] >> 4) & 0xF) as usize];
        szReverseName[j as usize + 3] = b'.' as u16;
        i = i.wrapping_add(1);
    }

    static IP6_ARPA: [u16; 11] = [
        0x0069u16, 0x0070, 0x0036, 0x002e,
        0x0061, 0x0072, 0x0070, 0x0061,
        0x002e, 0x0000, 0u16,
    ];
    wcscat(szReverseName.as_mut_ptr(), IP6_ARPA.as_ptr());

    let mut PtrRecord: DNS_RECORDW = core::mem::zeroed();
    ZeroMemory(
        &mut PtrRecord as *mut _ as _,
        core::mem::size_of::<DNS_RECORDW>(),
    );

    PtrRecord.pName = szReverseName.as_mut_ptr();
    PtrRecord.wType = (DNS_TYPE_PTR) as u16;
    PtrRecord.wDataLength = core::mem::size_of::<DNS_PTR_DATA>() as u16;
    PtrRecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
    PtrRecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
    PtrRecord.dwTtl = 86400;
    PtrRecord.Data.PTR.as_mut().pNameHost = pszHostName;

    DnsIntCacheAddEntry(core::ptr::addr_of_mut!(AAAARecord), 1u8);
    DnsIntCacheAddEntry(core::ptr::addr_of_mut!(PtrRecord), 1u8);
}
unsafe extern "C" fn OpenHostsFile() -> *mut FILE {
    let process_heap = GetProcessHeap();
    let max_path = MAX_PATH as usize;
    let wide_char_size = core::mem::size_of::<u16>();
    let buffer_size = max_path * wide_char_size;
    let expanded_path: PWSTR = HeapAlloc(process_heap, 0, (buffer_size) as SIZE_T) as PWSTR;
    if expanded_path.is_null() {
        return core::ptr::null_mut();
    }

    let mut database_key: HKEY = core::mem::zeroed();
    let dw_error: u32 = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        [
            0x53u16, 0x79, 0x73, 0x74, 0x65, 0x6d, 0x5c, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6e, 0x74,
            0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x53, 0x65, 0x74, 0x5c, 0x53, 0x65, 0x72,
            0x76, 0x69, 0x63, 0x65, 0x73, 0x5c, 0x54, 0x63, 0x70, 0x69, 0x70, 0x5c, 0x50, 0x61,
            0x72, 0x61, 0x6d, 0x65, 0x74, 0x65, 0x72, 0x73, 0x00,
        ]
        .as_ptr(),
        0,
        KEY_READ,
        &mut database_key,
    )) as u32;

    let database_path: PWSTR;
    let mut reg_size: u32 = 0;
    // Divergence from the C: with Tcpip\Parameters present (a configured NIC) but no
    // DatabasePath value (nothing in our hives sets one), the C either bailed out with no
    // hosts file at all or expanded a zero-byte block as the directory -- so the hosts file
    // vanished from name resolution once the NIC was configured. Any failure falls back to
    // the default directory, as in mswsock's OpenNetworkDatabase.
    let mut have_path = false;
    if dw_error == ERROR_SUCCESS {
        let dw_size: u32 = (RegQueryValueExW(
            database_key,
            w!("DatabasePath"),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut reg_size,
        )) as u32;

        if dw_size == ERROR_SUCCESS && reg_size as usize >= core::mem::size_of::<u16>() {
            // One WCHAR more than the value, for a terminator the registry need not store.
            database_path = HeapAlloc(
                process_heap,
                0,
                (reg_size as usize + core::mem::size_of::<u16>()) as SIZE_T,
            ) as PWSTR;
            if database_path.is_null() {
                HeapFree(process_heap, 0, expanded_path as *mut core::ffi::c_void);
                RegCloseKey(database_key);
                return core::ptr::null_mut();
            }

            let dw_error2: u32 = (RegQueryValueExW(
                database_key,
                w!("DatabasePath"),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                database_path as *mut u8,
                &mut reg_size,
            )) as u32;

            if dw_error2 == ERROR_SUCCESS {
                *database_path.add(reg_size as usize / core::mem::size_of::<u16>()) = 0;
                let n = ExpandEnvironmentStringsW(database_path, expanded_path, MAX_PATH as u32);
                have_path = n != 0 && n <= MAX_PATH as u32 && *expanded_path != 0;
            }
            HeapFree(process_heap, 0, database_path as *mut core::ffi::c_void);
        }

        RegCloseKey(database_key);
    }
    if !have_path {
        GetSystemDirectoryW(expanded_path, MAX_PATH as u32);

        let mut string_length: usize = 0;
        StringCchLengthW(expanded_path, (MAX_PATH as u64) as usize, &mut string_length);
        if *(expanded_path as *const u16).add(string_length - 1) != 0x5c_u16 {
            StringCchCatW(
                expanded_path,
                (MAX_PATH as u64) as usize,
                w!("\\"),
            );
        }

        StringCchCatW(
            expanded_path,
            (MAX_PATH as u64) as usize,
            w!("drivers\\etc\\"),
        );
    }

    let mut string_length2: usize = 0;
    StringCchLengthW(expanded_path, (MAX_PATH as u64) as usize, &mut string_length2);
    if *(expanded_path as *const u16).add(string_length2 - 1) != 0x5c_u16 {
        StringCchCatW(
            expanded_path,
            (MAX_PATH as u64) as usize,
            w!("\\"),
        );
    }

    StringCchCatW(
        expanded_path,
        (MAX_PATH as u64) as usize,
        w!("hosts"),
    );

    let hosts_file: *mut FILE = _wfopen(
        expanded_path,
        w!("r"),
    );

    HeapFree(process_heap, 0, expanded_path as *mut core::ffi::c_void);

    hosts_file
}
#[no_mangle]
pub unsafe extern "C" fn ReadHostsFile() -> i32 {
    let mut szLineBuffer: [u8; 512] = [0; 512];
    let mut pHostFile: *mut FILE = core::ptr::null_mut();
    let mut Ptr: *mut u8;
    let mut NameStart: *mut u8;
    let mut NameEnd: *mut u8;
    let mut AddressStart: *mut u8;
    let mut AddressEnd: *mut u8;
    let mut Ipv4Address: in_addr = core::mem::zeroed();
    let mut Ipv6Address: in6_addr = core::mem::zeroed();
    let mut pszHostName: PWSTR;

    pHostFile = OpenHostsFile();
    if pHostFile.is_null() {
        return 0;
    }

    loop {
        if fgets(szLineBuffer.as_mut_ptr() as *mut i8, 512, pHostFile).is_null() {
            break;
        }

        NameStart = core::ptr::null_mut();
        NameEnd = core::ptr::null_mut();
        AddressStart = core::ptr::null_mut();
        AddressEnd = core::ptr::null_mut();

        Ptr = szLineBuffer.as_mut_ptr();

        // Search for the start of the ip address
        loop {
            if *Ptr == 0 || *Ptr == b'#' {
                break;
            }
            if isspace(*Ptr as i32) == 0 {
                AddressStart = Ptr;
                Ptr = Ptr.add(1);
                break;
            }
            Ptr = Ptr.add(1);
        }

        // Search for the end of the ip address
        loop {
            if *Ptr == 0 || *Ptr == b'#' {
                break;
            }
            if isspace(*Ptr as i32) != 0 {
                AddressEnd = Ptr;
                Ptr = Ptr.add(1);
                break;
            }
            Ptr = Ptr.add(1);
        }

        // Search for the start of the name
        loop {
            if *Ptr == 0 || *Ptr == b'#' {
                break;
            }
            if isspace(*Ptr as i32) == 0 {
                NameStart = Ptr;
                Ptr = Ptr.add(1);
                break;
            }
            Ptr = Ptr.add(1);
        }

        // Search for the end of the name
        loop {
            if *Ptr == 0 || *Ptr == b'#' {
                break;
            }
            if isspace(*Ptr as i32) != 0 {
                NameEnd = Ptr;
                break;
            }
            Ptr = Ptr.add(1);
        }

        if AddressStart.is_null() || AddressEnd.is_null() ||
           NameStart.is_null() || NameEnd.is_null() {
            continue;
        }

        *AddressEnd = 0;
        *NameEnd = 0;

        // DPRINT("%s ==> %s\n", NameStart, AddressStart);
        // DPRINT("IPv4: %s\n", AddressStart);  // will be emitted inside branches if needed

        if ParseIpv4Address(AddressStart as PCSTR, &mut Ipv4Address) != 0 {
            // DPRINT("IPv4: %s\n", AddressStart);
            pszHostName = AnsiToUnicode(NameStart as *mut c_char);
            if !pszHostName.is_null() {
                AddIpv4HostEntries(pszHostName, &mut Ipv4Address as PIN_ADDR);
                HeapFree(GetProcessHeap(), 0, pszHostName as *mut core::ffi::c_void);
            }
        } else if ParseIpv6Address(AddressStart as PCSTR, &mut Ipv6Address) != 0 {
            // DPRINT("IPv6: %s\n", AddressStart);
            pszHostName = AnsiToUnicode(NameStart as *mut c_char);
            if !pszHostName.is_null() {
                AddIpv6HostEntries(pszHostName, &mut Ipv6Address as PIN6_ADDR);
                HeapFree(GetProcessHeap(), 0, pszHostName as *mut core::ffi::c_void);
            }
        }
    }

    fclose(pHostFile);
    return 1;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS system libraries
//  * FILE:        base/services/dnsrslvr/hostsfile.c
//  * PURPOSE:     HOSTS file routines
//  * PROGRAMERS:  Art Yerkes
//  *              Eric Kohl
//  */
//
// #include "precomp.h"
// #include <inaddr.h>
// #include <in6addr.h>
//
//
// #define NDEBUG
// #include <debug.h>
//
// static WCHAR szHexChar[] = L"0123456789abcdef";
//
// static
// PWSTR
// AnsiToUnicode(
//     PSTR NarrowString)
// {
//     PWSTR WideString;
//     int WideLen;
//
//     WideLen = MultiByteToWideChar(CP_ACP,
//                                   0,
//                                   NarrowString,
//                                   -1,
//                                   NULL,
//                                   0);
//     if (WideLen == 0)
//         return NULL;
//
//     WideString = HeapAlloc(GetProcessHeap(),
//                            0,
//                            WideLen * sizeof(WCHAR));
//     if (WideString == NULL)
//         return NULL;
//
//     MultiByteToWideChar(CP_ACP,
//                         0,
//                         NarrowString,
//                         -1,
//                         WideString,
//                         WideLen);
//
//     return WideString;
// }
//
//
// static
// BOOL
// ParseIpv4Address(
//      PCSTR AddressString,
//      PIN_ADDR pAddress)
// {
//     PCSTR pTerminator = NULL;
//     NTSTATUS Status;
//
//     Status = RtlIpv4StringToAddressA(AddressString,
//                                      TRUE,
//                                      &pTerminator,
//                                      pAddress);
//     if (NT_SUCCESS(Status) && pTerminator != NULL && *pTerminator == '\0')
//         return TRUE;
//
//     return FALSE;
// }
//
//
// static
// BOOL
// ParseIpv6Address(
//      LPCSTR AddressString,
//      PIN6_ADDR pAddress)
// {
//     PCSTR pTerminator = NULL;
//     NTSTATUS Status;
//
//     Status = RtlIpv6StringToAddressA(AddressString,
//                                      &pTerminator,
//                                      pAddress);
//     if (NT_SUCCESS(Status) && pTerminator != NULL && *pTerminator == '\0')
//         return TRUE;
//
//     return FALSE;
// }
//
//
// static
// VOID
// AddIpv4HostEntries(
//     PWSTR pszHostName,
//     PIN_ADDR pAddress)
// {
//     DNS_RECORDW ARecord, PtrRecord;
//     WCHAR szReverseName[32];
//
//     /* Prepare the A record */
//     ZeroMemory(&ARecord, sizeof(DNS_RECORDW));
//
//     ARecord.pName = pszHostName;
//     ARecord.wType = DNS_TYPE_A;
//     ARecord.wDataLength = sizeof(DNS_A_DATA);
//     ARecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
//     ARecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
//     ARecord.dwTtl = 86400;
//
//     ARecord.Data.A.as_mut().IpAddress = pAddress->S_un.S_addr;
//
//     /* Prepare the PTR record */
//     swprintf(szReverseName,
//              L"%u.%u.%u.%u.in-addr.arpa.",
//              pAddress->S_un.S_un_b.s_b4,
//              pAddress->S_un.S_un_b.s_b3,
//              pAddress->S_un.S_un_b.s_b2,
//              pAddress->S_un.S_un_b.s_b1);
//
//     ZeroMemory(&PtrRecord, sizeof(DNS_RECORDW));
//
//     PtrRecord.pName = szReverseName;
//     PtrRecord.wType = DNS_TYPE_PTR;
//     PtrRecord.wDataLength = sizeof(DNS_PTR_DATA);
//     PtrRecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
//     PtrRecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
//     PtrRecord.dwTtl = 86400;
//
//     PtrRecord.Data.PTR.as_mut().pNameHost = pszHostName;
//
//     DnsIntCacheAddEntry(core::ptr::addr_of_mut!(ARecord), TRUE);
//     DnsIntCacheAddEntry(core::ptr::addr_of_mut!(PtrRecord), TRUE);
// }
//
//
// static
// VOID
// AddIpv6HostEntries(
//     PWSTR pszHostName,
//     PIN6_ADDR pAddress)
// {
//     DNS_RECORDW AAAARecord, PtrRecord;
//     WCHAR szReverseName[80];
//     DWORD i, j, k;
//
//     /* Prepare the AAAA record */
//     ZeroMemory(&AAAARecord, sizeof(DNS_RECORDW));
//
//     AAAARecord.pName = pszHostName;
//     AAAARecord.wType = DNS_TYPE_AAAA;
//     AAAARecord.wDataLength = sizeof(DNS_AAAA_DATA);
//     AAAARecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
//     AAAARecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
//     AAAARecord.dwTtl = 86400;
//
//     CopyMemory(&AAAARecord.Data.AAAA.as_mut().Ip6Address,
//                &pAddress->u.Byte,
//                sizeof(IN6_ADDR));
//
//     /* Prepare the PTR record */
//     ZeroMemory(szReverseName, sizeof(szReverseName));
//
//     for (i = 0; i < sizeof(IN6_ADDR); i++)
//     {
//         j = 4 * i;
//         k = sizeof(IN6_ADDR) - 1 - i;
//         szReverseName[j] = szHexChar[pAddress->u.Byte[k] & 0xF];
//         szReverseName[j + 1] = L'.';
//         szReverseName[j + 2] = szHexChar[(pAddress->u.Byte[k] >> 4) & 0xF];
//         szReverseName[j + 3] = L'.';
//     }
//     wcscat(szReverseName, L"ip6.arpa.");
//
//     ZeroMemory(&PtrRecord, sizeof(DNS_RECORDW));
//
//     PtrRecord.pName = szReverseName;
//     PtrRecord.wType = DNS_TYPE_PTR;
//     PtrRecord.wDataLength = sizeof(DNS_PTR_DATA);
//     PtrRecord.Flags.S.as_mut().set_Section(DnsSectionAnswer);
//     PtrRecord.Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
//     PtrRecord.dwTtl = 86400;
//
//     PtrRecord.Data.PTR.as_mut().pNameHost = pszHostName;
//
//     DnsIntCacheAddEntry(core::ptr::addr_of_mut!(AAAARecord), TRUE);
//     DnsIntCacheAddEntry(core::ptr::addr_of_mut!(PtrRecord), TRUE);
// }
//
//
// static
// FILE *
// OpenHostsFile(VOID)
// {
//     PWSTR ExpandedPath;
//     PWSTR DatabasePath;
//     HKEY DatabaseKey;
//     DWORD RegSize = 0;
//     size_t StringLength;
//     FILE *pHostsFile;
//     DWORD dwError;
//
//     ExpandedPath = HeapAlloc(GetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR));
//     if (ExpandedPath == NULL)
//         return NULL;
//
//     /* Open the database path key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services\\Tcpip\\Parameters",
//                             0,
//                             KEY_READ,
//                             &DatabaseKey);
//     if (dwError == ERROR_SUCCESS)
//     {
//         /* Read the actual path */
//         RegQueryValueExW(DatabaseKey,
//                          L"DatabasePath",
//                          NULL,
//                          NULL,
//                          NULL,
//                          &RegSize);
//
//         DatabasePath = HeapAlloc(GetProcessHeap(), 0, RegSize);
//         if (DatabasePath == NULL)
//         {
//             HeapFree(GetProcessHeap(), 0, ExpandedPath);
//             RegCloseKey(DatabaseKey);
//             return NULL;
//         }
//
//         /* Read the actual path */
//         dwError = RegQueryValueExW(DatabaseKey,
//                                    L"DatabasePath",
//                                    NULL,
//                                    NULL,
//                                    (LPBYTE)DatabasePath,
//                                    &RegSize);
//
//         /* Close the key */
//         RegCloseKey(DatabaseKey);
//
//         if (dwError != ERROR_SUCCESS)
//         {
//             HeapFree(GetProcessHeap(), 0, DatabasePath);
//             HeapFree(GetProcessHeap(), 0, ExpandedPath);
//             return NULL;
//         }
//
//         /* Expand the name */
//         ExpandEnvironmentStringsW(DatabasePath, ExpandedPath, MAX_PATH);
//
//         HeapFree(GetProcessHeap(), 0, DatabasePath);
//     }
//     else
//     {
//         /* Use defalt path */
//         GetSystemDirectoryW(ExpandedPath, MAX_PATH);
//
//         StringCchLengthW(ExpandedPath, MAX_PATH, &StringLength);
//         if (ExpandedPath[StringLength - 1] != L'\\')
//         {
//             /* It isn't, so add it ourselves */
//             StringCchCatW(ExpandedPath, MAX_PATH, L"\\");
//         }
//
//         StringCchCatW(ExpandedPath, MAX_PATH, L"drivers\\etc\\");
//     }
//
//     /* Make sure that the path is backslash-terminated */
//     StringCchLengthW(ExpandedPath, MAX_PATH, &StringLength);
//     if (ExpandedPath[StringLength - 1] != L'\\')
//     {
//         /* It isn't, so add it ourselves */
//         StringCchCatW(ExpandedPath, MAX_PATH, L"\\");
//     }
//
//     /* Add the database name */
//     StringCchCatW(ExpandedPath, MAX_PATH, L"hosts");
//
//     /* Open the hosts file */
//     pHostsFile = _wfopen(ExpandedPath, L"r");
//
//     HeapFree(GetProcessHeap(), 0, ExpandedPath);
//
//     return pHostsFile;
// }
//
//
// BOOL
// ReadHostsFile(VOID)
// {
//     CHAR szLineBuffer[512];
//     FILE *pHostFile = NULL;
//     CHAR *Ptr, *NameStart, *NameEnd, *AddressStart, *AddressEnd;
//     struct in_addr Ipv4Address;
//     struct in6_addr Ipv6Address;
//     PWSTR pszHostName;
//
//     pHostFile = OpenHostsFile();
//     if (pHostFile == NULL)
//         return FALSE;
//
//     for (;;)
//     {
//         /* Read a line */
//         if (fgets(szLineBuffer, sizeof(szLineBuffer), pHostFile) == NULL)
//             break;
//
//         NameStart = NameEnd = NULL;
//         AddressStart = AddressEnd = NULL;
//
//         /* Search for the start of the ip address */
//         Ptr = szLineBuffer;
//         for (;;)
//         {
//             if (*Ptr == 0 || *Ptr == '#')
//                 break;
//
//             if (!isspace(*Ptr))
//             {
//                 AddressStart = Ptr;
//                 Ptr = Ptr + 1;
//                 break;
//             }
//
//             Ptr = Ptr + 1;
//         }
//
//         /* Search for the end of the ip address */
//         for (;;)
//         {
//             if (*Ptr == 0 || *Ptr == '#')
//                 break;
//
//             if (isspace(*Ptr))
//             {
//                 AddressEnd = Ptr;
//                 Ptr = Ptr + 1;
//                 break;
//             }
//
//             Ptr = Ptr + 1;
//         }
//
//         /* Search for the start of the name */
//         for (;;)
//         {
//             if (*Ptr == 0 || *Ptr == '#')
//                 break;
//
//             if (!isspace(*Ptr))
//             {
//                 NameStart = Ptr;
//                 Ptr = Ptr + 1;
//                 break;
//             }
//
//             Ptr = Ptr + 1;
//         }
//
//         /* Search for the end of the name */
//         for (;;)
//         {
//             if (*Ptr == 0 || *Ptr == '#')
//                 break;
//
//             if (isspace(*Ptr))
//             {
//                 NameEnd = Ptr;
//                 break;
//             }
//
//             Ptr = Ptr + 1;
//         }
//
//         if (AddressStart == NULL || AddressEnd == NULL ||
//             NameStart == NULL || NameEnd == NULL)
//             continue;
//
//         *AddressEnd = 0;
//         *NameEnd = 0;
//
//         DPRINT("%s ==> %s\n", NameStart, AddressStart);
//
//         if (ParseIpv4Address(AddressStart, &Ipv4Address))
//         {
//             DPRINT("IPv4: %s\n", AddressStart);
//
//             pszHostName = AnsiToUnicode(NameStart as *mut c_char);
//             if (pszHostName != NULL)
//             {
//                 AddIpv4HostEntries(pszHostName, &Ipv4Address);
//                 HeapFree(GetProcessHeap(), 0, pszHostName);
//             }
//         }
//         else if (ParseIpv6Address(AddressStart, &Ipv6Address))
//         {
//             DPRINT("IPv6: %s\n", AddressStart);
//
//             pszHostName = AnsiToUnicode(NameStart as *mut c_char);
//             if (pszHostName != NULL)
//             {
//                 AddIpv6HostEntries(pszHostName, &Ipv6Address);
//                 HeapFree(GetProcessHeap(), 0, pszHostName);
//             }
//         }
//     }
//
//     fclose(pHostFile);
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
