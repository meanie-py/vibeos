use crate::prelude::*;
use core::ffi::c_void;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub s_b1: UCHAR,
    pub s_b2: UCHAR,
    pub s_b3: UCHAR,
    pub s_b4: UCHAR,
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SEPARATOR: ::core::ffi::c_char = '-' as i32 as ::core::ffi::c_char;

#[no_mangle]
pub static mut _CRT_glob: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

// #define WIN32_NO_STATUS: valueless macro, no Rust constant emitted.
// #define _INC_WINDOWS: valueless macro, no Rust constant emitted.

// Prototypes of DoFormatMessage, PrintEntries, DisplayArpEntries, Addhost,
// Deletehost and Usage are forward declarations of functions defined in arp.c.
// They are translated as function definitions in their own passes, so no
// declarations are emitted here.
#[no_mangle]
pub unsafe extern "C" fn DoFormatMessage() -> DWORD {
    let mut lpMsgBuf: LPTSTR = core::ptr::null_mut();
    let mut RetVal: DWORD = 0;

    let ErrorCode: DWORD = GetLastError();

    if ErrorCode != ERROR_SUCCESS {
        RetVal = FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER |
            FORMAT_MESSAGE_FROM_SYSTEM |
            FORMAT_MESSAGE_IGNORE_INSERTS,
            core::ptr::null_mut(),
            ErrorCode,
            (MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT)) as _, /* Default language */
            core::ptr::addr_of_mut!(lpMsgBuf) as LPTSTR,
            0,
            core::ptr::null_mut(),
        );

        if RetVal != 0 {
            _tprintf(_T!("%s").as_ptr() as *const core::ffi::c_char, lpMsgBuf);
            LocalFree(lpMsgBuf as HLOCAL);
            /* return number of TCHAR's stored in output buffer
             * excluding '\0' - as FormatMessage does*/
            return RetVal;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn PrintMessage(dwMessage: DWORD) {
    let mut lpMsgBuf: LPTSTR = core::ptr::null_mut();
    let mut RetVal: DWORD;
    RetVal = FormatMessage(
        FORMAT_MESSAGE_ALLOCATE_BUFFER |
        FORMAT_MESSAGE_FROM_HMODULE |
        FORMAT_MESSAGE_IGNORE_INSERTS,
        GetModuleHandleW(core::ptr::null()) as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPTSTR,
        0,
        core::ptr::null_mut(),
    );
    if RetVal != 0 {
        // arp is ANSI: `_tprintf(_T("%s"), ..)` is printf with a narrow format (a UTF-16
        // "%s" reads as the one-byte string "%" and prints nothing).
        printf(b"%s\0".as_ptr() as *const ::core::ffi::c_char, lpMsgBuf);
        LocalFree(lpMsgBuf as HLOCAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintMessageV_v(dwMessage: DWORD, ap_list: *mut ::core::ffi::c_void) {
    let mut lpMsgBuf: LPTSTR = core::ptr::null_mut();
    let mut args: va_list = *(ap_list as *mut va_list);
    let mut RetVal: DWORD = 0;



    RetVal = FormatMessage(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE,
        GetModuleHandleW(core::ptr::null_mut()) as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPTSTR,
        0,
        core::ptr::addr_of_mut!(args),
    );


    if RetVal != 0 {
        // arp is ANSI: `_tprintf(_T("%s"), ..)` is printf with a narrow format (a UTF-16
        // "%s" reads as the one-byte string "%" and prints nothing).
        printf(b"%s\0".as_ptr() as *const ::core::ffi::c_char, lpMsgBuf);
        LocalFree(lpMsgBuf as HLOCAL);
    }
}


#[no_mangle]
pub unsafe extern "C" fn PrintEntries(pIpAddRow: PMIB_IPNETROW) -> DWORD {
    let mut inaddr: IN_ADDR = in_addr {
        S_un: in_addr__bindgen_ty_1 {
            S_un_b: in_addr__bindgen_ty_1__bindgen_ty_1 {
                s_b1: 0,
                s_b2: 0,
                s_b3: 0,
                s_b4: 0,
            },
        },
    };
    let mut cMacAddr: [TCHAR; 20] = [0; 20];

    /* print IP addresses */
    inaddr.S_un.S_addr = (*pIpAddRow).dwAddr as ULONG;
    _tprintf(_T!("  %-22s").as_ptr() as *const TCHAR, inet_ntoa(inaddr as in_addr));

    /* print MAC address */
    _stprintf(
        core::ptr::addr_of_mut!(cMacAddr) as *mut TCHAR,
        _T!("%02x-%02x-%02x-%02x-%02x-%02x").as_ptr() as *const TCHAR,
        (*pIpAddRow).bPhysAddr[0] as ::core::ffi::c_int,
        (*pIpAddRow).bPhysAddr[1] as ::core::ffi::c_int,
        (*pIpAddRow).bPhysAddr[2] as ::core::ffi::c_int,
        (*pIpAddRow).bPhysAddr[3] as ::core::ffi::c_int,
        (*pIpAddRow).bPhysAddr[4] as ::core::ffi::c_int,
        (*pIpAddRow).bPhysAddr[5] as ::core::ffi::c_int,
    );
    _tprintf(_T!("%-22s").as_ptr() as *const TCHAR, core::ptr::addr_of_mut!(cMacAddr) as *mut TCHAR);

    /* print cache type */
    match (*pIpAddRow).__bindgen_anon_1.dwType {
        MIB_IPNET_TYPE_DYNAMIC => {
            PrintMessage(MSG_ARP_DYNAMIC);
        }
        MIB_IPNET_TYPE_STATIC => {
            PrintMessage(MSG_ARP_STATIC);
        }
        MIB_IPNET_TYPE_INVALID => {
            PrintMessage(MSG_ARP_INVALID);
        }
        MIB_IPNET_TYPE_OTHER => {
            PrintMessage(MSG_ARP_OTHER);
        }
        _ => {}
    }
    _putts(_T!("").as_ptr() as *const TCHAR);
    return NO_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayArpEntries(pszInetAddr: PTCHAR, pszIfAddr: PTCHAR) -> DWORD {
    let mut i: DWORD = 0;
    let mut k: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut pIpNetTable: PMIB_IPNETTABLE = core::ptr::null_mut();
    let mut pIpAddrTable: PMIB_IPADDRTABLE = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut inaddr: in_addr = core::mem::zeroed();
    let mut inaddr2: in_addr = core::mem::zeroed();
    let mut pszIpAddr: PTCHAR = core::ptr::null_mut();
    let mut szIntIpAddr: [TCHAR; 20] = [0; 20];
    let mut dwError: DWORD = NO_ERROR;

    'cleanup: {
        /* retrieve the IP-to-physical address mapping table */

        /* get table size */
        GetIpNetTable(pIpNetTable, core::ptr::addr_of_mut!(Size), 0);

        /* allocate memory for ARP address table */
        pIpNetTable = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PMIB_IPNETTABLE;
        if pIpNetTable.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        ZeroMemory!(pIpNetTable as *mut core::ffi::c_void, core::mem::size_of::<MIB_IPNETTABLE>());

        dwError = GetIpNetTable(pIpNetTable, core::ptr::addr_of_mut!(Size), (TRUE) as BOOL);
        if dwError != NO_ERROR {
            _tprintf(_T!("GetIpNetTable failed: %lu\n\0").as_ptr() as *const core::ffi::c_char, dwError);
            DoFormatMessage();
            break 'cleanup;
        }

        /* check there are entries in the table */
        if (*pIpNetTable).dwNumEntries == 0 {
            PrintMessage(MSG_ARP_NO_ENTRIES);
            break 'cleanup;
        }

        /* Retrieve the interface-to-ip address mapping
         * table to get the IP address for adapter */

        /* get table size */
        Size = 0;
        GetIpAddrTable(pIpAddrTable, core::ptr::addr_of_mut!(Size), 0);

        pIpAddrTable = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PMIB_IPADDRTABLE;
        if pIpAddrTable.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        ZeroMemory!(pIpAddrTable as *mut core::ffi::c_void, core::mem::size_of::<MIB_IPADDRTABLE>());

        dwError = GetIpAddrTable(pIpAddrTable, core::ptr::addr_of_mut!(Size), (TRUE) as BOOL);
        if dwError != NO_ERROR {
            _tprintf(_T!("GetIpAddrTable failed: %lu\n\0").as_ptr() as *const core::ffi::c_char, dwError);
            DoFormatMessage();
            break 'cleanup;
        }

        k = 0;
        while k < (*pIpAddrTable).dwNumEntries {
            if (*(core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                .offset(0 as isize))
                .dwIndex
                == (*(core::ptr::addr_of_mut!((*pIpAddrTable).table) as *mut MIB_IPADDRROW)
                    .offset(k as isize))
                .dwIndex
            {
                //printf("debug print: pIpAddrTable->table[?].dwIndex = %lx\n", pIpNetTable->table[k].dwIndex);
                inaddr2.S_un.S_addr = (*(core::ptr::addr_of_mut!((*pIpAddrTable).table) as *mut MIB_IPADDRROW)
                    .offset(k as isize))
                .dwAddr as ULONG;
                pszIpAddr = inet_ntoa(inaddr2) as PTCHAR;
                strcpy(core::ptr::addr_of_mut!(szIntIpAddr) as *mut TCHAR, pszIpAddr);
            }
            k = k.wrapping_add(1);
        }

        /* Count relevant ARP entries */
        dwCount = 0;
        i = 0;
        while i < (*pIpNetTable).dwNumEntries {
            /* if the user has supplied their own internet address *
             * only count the arp entry which matches that */
            if !pszInetAddr.is_null() {
                inaddr.S_un.S_addr = (*(core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                    .offset(i as isize))
                .dwAddr as ULONG;
                pszIpAddr = inet_ntoa(inaddr) as PTCHAR;

                /* check if it matches, count it */
                if strcmp(pszIpAddr as *const TCHAR, pszInetAddr as *const TCHAR) == 0 {
                    dwCount = dwCount.wrapping_add(1);
                }
            } else {
                /* if an address is not supplied, count all entries */
                dwCount = dwCount.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        /* Print message and leave if there are no relevant ARP entries */
        if dwCount == 0 {
            PrintMessage(MSG_ARP_NO_ENTRIES);
            break 'cleanup;
        }

        /* print header, including interface IP address and index number */
        PrintMessageV(
            MSG_ARP_INTERFACE,
            core::ptr::addr_of_mut!(szIntIpAddr) as *mut TCHAR,
            (*(core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                .offset(0 as isize))
            .dwIndex,
        );

        /* go through all ARP entries */
        i = 0;
        while i < (*pIpNetTable).dwNumEntries {
            /* if the user has supplied their own internet address *
             * only print the arp entry which matches that */
            if !pszInetAddr.is_null() {
                inaddr.S_un.S_addr = (*(core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                    .offset(i as isize))
                .dwAddr as ULONG;
                pszIpAddr = inet_ntoa(inaddr) as PTCHAR;

                /* check if it matches, print it */
                if strcmp(pszIpAddr as *const TCHAR, pszInetAddr as *const TCHAR) == 0 {
                    PrintEntries((core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                        .offset(i as isize));
                }
            } else {
                /* if an address is not supplied, print all entries */
                PrintEntries((core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut MIB_IPNETROW)
                    .offset(i as isize));
            }
            i = i.wrapping_add(1);
        }
    }

    // cleanup:
    if !pIpNetTable.is_null() {
        HeapFree(GetProcessHeap(), 0, pIpNetTable as PVOID);
    }
    if !pIpAddrTable.is_null() {
        HeapFree(GetProcessHeap(), 0, pIpAddrTable as PVOID);
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn Addhost(pszInetAddr: PTCHAR, pszEthAddr: PTCHAR, pszIfAddr: PTCHAR) -> DWORD {
    let mut pAddHost: PMIB_IPNETROW = core::ptr::null_mut();
    let mut pIpNetTable: PMIB_IPNETTABLE = core::ptr::null_mut();
    let mut dwIpAddr: DWORD = 0;
    let mut Size: ULONG = 0;
    let mut i: INT = 0;
    let mut val: INT = 0;
    let mut c: INT = 0;
    let mut dwError: DWORD = NO_ERROR;

    /* error checking */

    /* check IP address */
    if pszInetAddr.is_null() {
        Usage();
        return ERROR_INVALID_PARAMETER;
    }

    dwIpAddr = inet_addr(pszInetAddr as *const i8);
    if dwIpAddr == INADDR_NONE {
        PrintMessageV(MSG_ARP_BAD_IP_ADDRESS, pszInetAddr);
        return ERROR_INVALID_PARAMETER;
    }

    /* check MAC address */
    if strlen(pszEthAddr as *const i8) != 17 {
        PrintMessageV(MSG_ARP_BAD_ARGUMENT, pszEthAddr);
        return ERROR_INVALID_PARAMETER;
    }

    i = 0;
    while i < 17 {
        'body: {
            if (*pszEthAddr.offset(i as isize) as INT) == (SEPARATOR) as _ {
                break 'body;
            }

            if !(isxdigit(*pszEthAddr.offset(i as isize) as INT) != 0) {
                PrintMessageV(MSG_ARP_BAD_ARGUMENT, pszEthAddr);
                return ERROR_INVALID_PARAMETER;
            }
        }
        i += 1;
    }

    /* We need the IpNetTable to get the adapter index */
    /* Return required buffer size */
    GetIpNetTable(pIpNetTable, core::ptr::addr_of_mut!(Size), 0);

    /* allocate memory for ARP address table */
    pIpNetTable = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PMIB_IPNETTABLE;

    'cleanup: {
        if pIpNetTable.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        core::ptr::write_bytes(pIpNetTable as *mut u8, 0, core::mem::size_of::<MIB_IPNETTABLE>());

        dwError = GetIpNetTable(pIpNetTable, core::ptr::addr_of_mut!(Size), (TRUE) as BOOL);
        if dwError != NO_ERROR {
            _tprintf(b"GetIpNetTable failed: %lu\n\0".as_ptr() as *const i8, dwError);
            DoFormatMessage();
            break 'cleanup;
        }

        /* reserve memory on heap and zero */
        pAddHost = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MIB_IPNETROW>() as SIZE_T) as PMIB_IPNETROW;
        if pAddHost.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        core::ptr::write_bytes(pAddHost as *mut u8, 0, core::mem::size_of::<MIB_IPNETROW>());

        /* set dwIndex field to the index of a local IP address to
         * indicate the network on which the ARP entry applies */
        if !pszIfAddr.is_null() {
            if sscanf(
                pszIfAddr as *const i8,
                b"%lx\0".as_ptr() as *const i8,
                core::ptr::addr_of_mut!((*pAddHost).dwIndex),
            ) == EOF
            {
                break 'cleanup;
            }
        } else {
            //printf("debug print: pIpNetTable->table[0].dwIndex = %lx\n", pIpNetTable->table[0].dwIndex);
            /* needs testing. I get the correct index on my machine, but need others
             * to test their card index.  Any problems and we can use GetAdaptersInfo instead */
            (*pAddHost).dwIndex = (*(*pIpNetTable).table.as_ptr().add(0 as usize)).dwIndex;
        }

        /* Set MAC address to 6 bytes (typical) */
        (*pAddHost).dwPhysAddrLen = 6;

        /* Encode bPhysAddr into correct byte array */
        i = 0;
        while i < 6 {
            val = 0;
            c = toupper(*pszEthAddr.offset((i * 3) as isize) as INT);
            c = c - (if isdigit(c) != 0 {
                '0' as INT
            } else {
                'A' as INT - 10
            });
            val += c;
            val = val << 4;
            c = toupper(
                *pszEthAddr.offset((i * 3 + 1) as isize) as INT,
            );
            c = c - (if isdigit(c) != 0 {
                '0' as INT
            } else {
                'A' as INT - 10
            });
            val += c;
            (*pAddHost).bPhysAddr[i as usize] = val as BYTE;
            i += 1;
        }

        /* copy converted IP address */
        (*pAddHost).dwAddr = dwIpAddr;

        /* set type to static */
        (*pAddHost).__bindgen_anon_1.dwType = MIB_IPNET_TYPE_STATIC;

        /* Add the ARP entry */
        dwError = SetIpNetEntry(pAddHost);
        if dwError != NO_ERROR {
            DoFormatMessage();
            break 'cleanup;
        }
    }

    /* cleanup: */
    if !pIpNetTable.is_null() {
        HeapFree(GetProcessHeap(), 0, pIpNetTable as PVOID);
    }
    if !pAddHost.is_null() {
        HeapFree(GetProcessHeap(), 0, pAddHost as PVOID);
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn Deletehost(pszInetAddr: PTCHAR, pszIfAddr: PTCHAR) -> DWORD {
    let mut pDelHost: PMIB_IPNETROW = core::ptr::null_mut();
    let mut pIpNetTable: PMIB_IPNETTABLE = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut dwIpAddr: DWORD = 0;
    let mut bFlushTable: BOOL = (FALSE) as i32;
    let mut dwError: DWORD = NO_ERROR;

    /* error checking */

    /* check IP address */
    if pszInetAddr.is_null() {
        Usage();
        return ERROR_INVALID_PARAMETER;
    }

    /* if wildcard is given, set flag to delete all hosts */
    if strncmp(pszInetAddr as *const i8, b"*\0".as_ptr() as *const i8, 1 as usize) == 0 {
        bFlushTable = (TRUE) as i32;
    } else {
        dwIpAddr = inet_addr(pszInetAddr as *const i8) as DWORD;
        if dwIpAddr == INADDR_NONE {
            PrintMessageV(MSG_ARP_BAD_IP_ADDRESS, pszInetAddr);
            return ERROR_INVALID_PARAMETER;
        }
    }

    'cleanup: {
        /* We need the IpNetTable to get the adapter index */
        /* Return required buffer size */
        GetIpNetTable(core::ptr::null_mut(), core::ptr::addr_of_mut!(Size), 0);

        /* allocate memory for ARP address table */
        pIpNetTable = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PMIB_IPNETTABLE;
        if pIpNetTable.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        ZeroMemory!(pIpNetTable as PVOID, core::mem::size_of::<_MIB_IPNETTABLE>() as SIZE_T);

        dwError = GetIpNetTable(pIpNetTable, core::ptr::addr_of_mut!(Size), (TRUE) as BOOL);
        if dwError != NO_ERROR {
            _tprintf(b"GetIpNetTable failed: %lu\n\0".as_ptr() as *const i8, dwError);
            DoFormatMessage();
            break 'cleanup;
        }

        /* reserve memory on heap and zero */
        pDelHost = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MIB_IPNETROW>() as SIZE_T) as *mut MIB_IPNETROW as PMIB_IPNETROW;
        if pDelHost.is_null() {
            PrintMessage(MSG_ARP_NO_MEMORY);
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        ZeroMemory!(pDelHost as PVOID, core::mem::size_of::<MIB_IPNETROW>() as SIZE_T);

        /* set dwIndex field to the index of a local IP address to
         * indicate the network on which the ARP entry applies */
        if !pszIfAddr.is_null() {
            if sscanf(pszIfAddr as *const i8, b"%lx\0".as_ptr() as *const i8, core::ptr::addr_of_mut!((*pDelHost).dwIndex)) == EOF {
                break 'cleanup;
            }
        } else {
            /* needs testing. I get the correct index on my machine, but need others
             * to test their card index. Any problems and we can use GetAdaptersInfo instead */
            (*pDelHost).dwIndex = (*(*pIpNetTable).table.as_ptr().add(0 as usize)).dwIndex;
        }

        if (bFlushTable) as u32 != FALSE {
            /* delete arp cache */
            dwError = FlushIpNetTable((*pDelHost).dwIndex);
            if dwError != NO_ERROR {
                DoFormatMessage();
                break 'cleanup;
            }
        } else {
            /* copy converted IP address */
            (*pDelHost).dwAddr = dwIpAddr;

            /* Delete the ARP entry */
            dwError = DeleteIpNetEntry(pDelHost);
            if dwError != NO_ERROR {
                DoFormatMessage();
                break 'cleanup;
            }
        }
    }

    /* cleanup: */
    if !pIpNetTable.is_null() {
        HeapFree(GetProcessHeap(), 0, pIpNetTable as PVOID);
    }
    if !pDelHost.is_null() {
        HeapFree(GetProcessHeap(), 0, pDelHost as PVOID);
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn Usage() {
    PrintMessage(MSG_ARP_SYNTAX);
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: INT, argv: *mut *mut i8) -> INT {
    let mut dwError: DWORD = NO_ERROR;

    if (argc < 2) || (argc > 5) {
        Usage();
        return (EXIT_FAILURE) as INT;
    }

    if *(*argv.offset(1 as isize)).offset(0 as isize) != b'-' as i8 {
        Usage();
        return (EXIT_SUCCESS) as INT;
    }

    match *(*argv.offset(1 as isize)).offset(1 as isize) as u8 {
        b'a' | b'g' => {
            if argc == 2 {
                dwError = DisplayArpEntries(core::ptr::null_mut(), core::ptr::null_mut());
            } else if argc == 3 {
                dwError = DisplayArpEntries(*argv.offset(2 as isize), core::ptr::null_mut());
            } else if (argc == 4)
                && (strcmp(*argv.offset(2 as isize), b"-N\0".as_ptr() as *const i8) == 0)
            {
                dwError = DisplayArpEntries(core::ptr::null_mut(), *argv.offset(3 as isize));
            } else if (argc == 5)
                && (strcmp(*argv.offset(3 as isize), b"-N\0".as_ptr() as *const i8) == 0)
            {
                dwError = DisplayArpEntries(*argv.offset(2 as isize), *argv.offset(4 as isize));
            } else {
                Usage();
                dwError = ERROR_INVALID_PARAMETER;
            }
        }

        b'd' => {
            if argc == 3 {
                dwError = Deletehost(*argv.offset(2 as isize), core::ptr::null_mut());
            } else if argc == 4 {
                dwError = Deletehost(*argv.offset(2 as isize), *argv.offset(3 as isize));
            } else {
                Usage();
                dwError = ERROR_INVALID_PARAMETER;
            }
        }

        b's' => {
            if argc == 4 {
                dwError = Addhost(
                    *argv.offset(2 as isize),
                    *argv.offset(3 as isize),
                    core::ptr::null_mut(),
                );
            } else if argc == 5 {
                dwError = Addhost(
                    *argv.offset(2 as isize),
                    *argv.offset(3 as isize),
                    *argv.offset(4 as isize),
                );
            } else {
                Usage();
                dwError = ERROR_INVALID_PARAMETER;
            }
        }

        _ => {
            Usage();
            dwError = ERROR_INVALID_PARAMETER;
        }
    }

    if dwError == NO_ERROR {
        (EXIT_SUCCESS as i32)
    } else {
        (EXIT_FAILURE as i32)
    }
}
