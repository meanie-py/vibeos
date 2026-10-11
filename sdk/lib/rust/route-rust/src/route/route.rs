// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_08F79C40983C: [u16; 7] = [0x0064u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_09C488271038: [u16; 4] = [0x0061u16, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_0B4242BA6CFE: [u16; 5] = [0x006Du16, 0x0061, 0x0073, 0x006B, 0x0000];
pub static RUSTCONV_WSTR_128F490EEE24: [u16; 10] = [0x0032u16, 0x0035, 0x0035, 0x002E, 0x0030, 0x002E, 0x0030, 0x002E, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_5A9BC6E8FCA1: [u16; 10] = [0x0032u16, 0x0034, 0x0030, 0x002E, 0x0030, 0x002E, 0x0030, 0x002E, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_64F21C85D58A: [u16; 10] = [0x0031u16, 0x0032, 0x0037, 0x002E, 0x0030, 0x002E, 0x0030, 0x002E, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_74C59F8927AE: [u16; 13] = [
    0x0025u16, 0x0073, 0x002C, 0x0025, 0x0073, 0x002C, 0x0025, 0x0073, 0x002C, 0x0025,
    0x006C, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_85C12618D8FB: [u16; 16] = [
    0x0032u16, 0x0035, 0x0035, 0x002E, 0x0032, 0x0035, 0x0035, 0x002E, 0x0032, 0x0035,
    0x0035, 0x002E, 0x0032, 0x0035, 0x0035, 0x0000,
];
pub static RUSTCONV_WSTR_929E1A6DA5A0: [u16; 10] = [0x0032u16, 0x0032, 0x0034, 0x002E, 0x0030, 0x002E, 0x0030, 0x002E, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9EE4EE67ED95: [u16; 2] = [0x0066u16, 0x0000];
pub static RUSTCONV_WSTR_AE1738016435: [u16; 68] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0054, 0x0063, 0x0070, 0x0069, 0x0070, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073,
    0x005C, 0x0050, 0x0065, 0x0072, 0x0073, 0x0069, 0x0073, 0x0074, 0x0065, 0x006E,
    0x0074, 0x0052, 0x006F, 0x0075, 0x0074, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_BAC643AA9F1F: [u16; 4] = [0x0025u16, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_BB31F4088066: [u16; 7] = [0x006Du16, 0x0065, 0x0074, 0x0072, 0x0069, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_CA2E16EFA58A: [u16; 31] = [
    0x0025u16, 0x0030, 0x0032, 0x0058, 0x0020, 0x0025, 0x0030, 0x0032, 0x0058, 0x0020,
    0x0025, 0x0030, 0x0032, 0x0058, 0x0020, 0x0025, 0x0030, 0x0032, 0x0058, 0x0020,
    0x0025, 0x0030, 0x0032, 0x0058, 0x0020, 0x0025, 0x0030, 0x0032, 0x0058, 0x0020,
    0x0000,
];
pub static RUSTCONV_WSTR_D3FE97979D0F: [u16; 2] = [0x0070u16, 0x0000];
pub static RUSTCONV_WSTR_DACCBBBCFACF: [u16; 6] = [0x0070u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const IPBUF: usize = 17;

pub const DELETE_FLAG: i32 = 0x1;

pub const PERSISTENT_FLAG: i32 = 0x2;

unsafe extern "C" fn MatchWildcard(Text: PWSTR, Pattern: PWSTR) -> BOOL {
    let mut TextLength: size_t = 0;
    let mut PatternLength: size_t = 0;
    let mut TextIndex: size_t = 0 as size_t;
    let mut PatternIndex: size_t = 0 as size_t;
    let mut StartIndex: size_t = (0 as size_t).wrapping_sub(1 as size_t);
    let mut MatchIndex: size_t = 0 as size_t;

    if Pattern.is_null() {
        return TRUE as BOOL;
    }

    TextLength = wcslen(Text as *const wchar_t) as size_t;
    PatternLength = wcslen(Pattern as *const wchar_t) as size_t;

    while TextIndex < TextLength {
        if (PatternIndex < PatternLength)
            && ((*Pattern.add(PatternIndex) == 0x003Fu16)
                || (towlower(*Pattern.add(PatternIndex) as wint_t)
                    == towlower(*Text.add(TextIndex) as wint_t)))
        {
            TextIndex = TextIndex.wrapping_add(1);
            PatternIndex = PatternIndex.wrapping_add(1);
        } else if (PatternIndex < PatternLength)
            && (*Pattern.add(PatternIndex) == 0x002Au16)
        {
            StartIndex = PatternIndex;
            MatchIndex = TextIndex;
            PatternIndex = PatternIndex.wrapping_add(1);
        } else if StartIndex != (0 as size_t).wrapping_sub(1 as size_t) {
            PatternIndex = StartIndex.wrapping_add(1 as size_t);
            MatchIndex = MatchIndex.wrapping_add(1);
            TextIndex = MatchIndex;
        } else {
            return FALSE as BOOL;
        }
    }

    while (PatternIndex < PatternLength)
        && (*Pattern.add(PatternIndex) == 0x002Au16)
    {
        PatternIndex = PatternIndex.wrapping_add(1);
    }

    return (PatternIndex == PatternLength) as BOOL;
}

unsafe extern "C" fn ParsePersistentRouteValue(
    RouteValue: PWSTR,
    Destination: *mut PWSTR,
    Netmask: *mut PWSTR,
    Gateway: *mut PWSTR,
    Metric: *mut PWSTR,
) {
    let mut Ptr: PWSTR = core::ptr::null_mut();
    let mut DestPtr: PWSTR = core::ptr::null_mut();

    if !Destination.is_null() {
        *Destination = core::ptr::null_mut();
    }

    if !Netmask.is_null() {
        *Netmask = core::ptr::null_mut();
    }

    if !Gateway.is_null() {
        *Gateway = core::ptr::null_mut();
    }

    if !Metric.is_null() {
        *Metric = core::ptr::null_mut();
    }

    DestPtr = RouteValue;
    if !Destination.is_null() {
        *Destination = DestPtr;
    }

    Ptr = wcschr(DestPtr as *const u16, 0x002Cu16) as PWSTR;
    if Ptr.is_null() {
        return;
    }

    *Ptr = 0x0000u16;
    Ptr = Ptr.offset(1);

    DestPtr = Ptr;
    if !Netmask.is_null() {
        *Netmask = DestPtr;
    }

    Ptr = wcschr(DestPtr as *const u16, 0x002Cu16) as PWSTR;
    if Ptr.is_null() {
        return;
    }

    *Ptr = 0x0000u16;
    Ptr = Ptr.offset(1);

    DestPtr = Ptr;
    if !Gateway.is_null() {
        *Gateway = DestPtr;
    }

    Ptr = wcschr(DestPtr as *const u16, 0x002Cu16) as PWSTR;
    if Ptr.is_null() {
        return;
    }

    *Ptr = 0x0000u16;
    Ptr = Ptr.offset(1);

    if !Metric.is_null() {
        *Metric = Ptr;
    }
}

unsafe extern "C" fn ParseCmdLine(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
    mut start: ::core::ffi::c_int,
    mut Destination: *mut PWSTR,
    mut Netmask: *mut PWSTR,
    mut Gateway: *mut PWSTR,
    mut Metric: *mut PWSTR,
) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    if !Destination.is_null() {
        *Destination = ::core::ptr::null_mut::<WCHAR>();
    }
    if !Netmask.is_null() {
        *Netmask = ::core::ptr::null_mut::<WCHAR>();
    }
    if !Gateway.is_null() {
        *Gateway = ::core::ptr::null_mut::<WCHAR>();
    }
    if !Metric.is_null() {
        *Metric = ::core::ptr::null_mut::<WCHAR>();
    }
    if argc > start {
        if !Destination.is_null() {
            *Destination = *argv.offset(start as isize) as PWSTR;
        }
    } else {
        return FALSE;
    }
    i = start + 1;
    while i < argc {
        if _wcsicmp(
            *argv.offset(i as isize),
            RUSTCONV_WSTR_0B4242BA6CFE.as_ptr(),
        ) == 0
        {
            i += 1;
            if i >= argc {
                return FALSE;
            }
            if !Netmask.is_null() {
                *Netmask = *argv.offset(i as isize) as PWSTR;
            }
        } else if _wcsicmp(
            *argv.offset(i as isize),
            RUSTCONV_WSTR_BB31F4088066.as_ptr(),
        ) == 0
        {
            i += 1;
            if i >= argc {
                return FALSE;
            }
            if !Metric.is_null() {
                *Metric = *argv.offset(i as isize) as PWSTR;
            }
        } else {
            if !Gateway.is_null() {
                *Gateway = *argv.offset(i as isize) as PWSTR;
            }
        }
        i += 1;
    }
    return TRUE;
}

unsafe extern "C" fn PrintMacAddress(Mac: PBYTE, Buffer: PWSTR) {
    swprintf(Buffer, RUSTCONV_WSTR_CA2E16EFA58A.as_ptr() as *const u16,
        *Mac.offset(0 as isize) as i32, *Mac.offset(1 as isize) as i32, *Mac.offset(2 as isize) as i32, *Mac.offset(3 as isize) as i32, *Mac.offset(4 as isize) as i32, *Mac.offset(5 as isize) as i32);
}

unsafe extern "C" fn PrintPersistentRoutes(mut Filter: PWSTR) -> DWORD {
    let mut szBuffer: [WCHAR; 64] = [0; 64];
    let mut dwIndex: DWORD = 0 as DWORD;
    let mut dwBufferSize: DWORD;
    let mut EntriesFound: BOOL = FALSE as BOOL;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: DWORD = ERROR_SUCCESS as DWORD;
    let mut Destination: PWSTR = core::ptr::null_mut();
    let mut Netmask: PWSTR = core::ptr::null_mut();
    let mut Gateway: PWSTR = core::ptr::null_mut();
    let mut Metric: PWSTR = core::ptr::null_mut();

    ConResPrintf(StdOut, IDS_SEPARATOR as UINT);
    ConResPrintf(StdOut, IDS_PERSISTENT_ROUTES as UINT);

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_AE1738016435.as_ptr(),
        0 as DWORD,
        KEY_READ as REGSAM,
        &raw mut hKey,
    ) as DWORD;
    if Error == ERROR_SUCCESS as DWORD {
        loop {
            dwBufferSize = 64 as DWORD;

            Error = RegEnumValueW(
                hKey,
                dwIndex,
                &raw mut szBuffer as LPWSTR,
                &raw mut dwBufferSize,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            if Error != ERROR_SUCCESS as DWORD {
                if Error == ERROR_NO_MORE_ITEMS as DWORD {
                    Error = ERROR_SUCCESS as DWORD;
                }

                break;
            }

            ParsePersistentRouteValue(
                &raw mut szBuffer as PWSTR,
                &raw mut Destination,
                &raw mut Netmask,
                &raw mut Gateway,
                &raw mut Metric,
            );

            if MatchWildcard(Destination, Filter) != 0 {
                if EntriesFound == FALSE as BOOL {
                    ConResPrintf(StdOut, IDS_PERSISTENT_HEADER as UINT);
                }
                ConResPrintf(StdOut, IDS_PERSISTENT_ENTRY as UINT, Destination, Netmask, Gateway, Metric);
                EntriesFound = TRUE as BOOL;
            }

            dwIndex = dwIndex.wrapping_add(1);
        }

        RegCloseKey(hKey);
    }

    if EntriesFound == FALSE as BOOL {
        ConResPrintf(StdOut, IDS_NONE as UINT);
    }

    return Error;
}

unsafe extern "C" fn FormatIPv4Address(
    mut pBuffer: PWCHAR,
    mut UnicastAddress: PIP_ADAPTER_UNICAST_ADDRESS,
    mut IfIndex: DWORD,
) {
    let mut Ptr: PIP_ADAPTER_UNICAST_ADDRESS = UnicastAddress;

    while !Ptr.is_null() {
        if (*(*Ptr).Address.lpSockaddr).sa_family as ::core::ffi::c_int == AF_INET as ::core::ffi::c_int {
            let mut si: *mut sockaddr_in = (*Ptr).Address.lpSockaddr as *mut sockaddr_in;
            // DWORD dwStrLen = IPV4_ADDR_STRING_MAX_LEN;
            // WSAAddressToStringW(Ptr->Address.lpSockaddr, (DWORD)Ptr->Address.iSockaddrLength, NULL, pBuffer, &dwStrLen);
            // (void)InetNtopW(AF_INET, &(si->sin_addr), pBuffer, IPV4_ADDR_STRING_MAX_LEN);
            RtlIpv4AddressToStringW(core::ptr::addr_of_mut!((*si).sin_addr), pBuffer);
            return;
        }

        Ptr = (*Ptr).Next as PIP_ADAPTER_UNICAST_ADDRESS;
    }

    swprintf(pBuffer, RUSTCONV_WSTR_BAC643AA9F1F.as_ptr(), IfIndex);
}

unsafe extern "C" fn GetFirstIPv4AddressFromIndex(
    mut pBuffer: PWSTR,
    mut pAdapterAddresses: PIP_ADAPTER_ADDRESSES,
    mut IfIndex: DWORD,
) {
    let mut Ptr: PIP_ADAPTER_ADDRESSES = pAdapterAddresses;
    while !Ptr.is_null() {
        if (*Ptr).__bindgen_anon_1.__bindgen_anon_1.IfIndex == IfIndex {
            FormatIPv4Address(
                pBuffer as PWCHAR,
                (*Ptr).FirstUnicastAddress,
                (*Ptr).__bindgen_anon_1.__bindgen_anon_1.IfIndex,
            );
            return;
        }
        Ptr = (*Ptr).Next as PIP_ADAPTER_ADDRESSES;
    }
    swprintf(pBuffer, RUSTCONV_WSTR_BAC643AA9F1F.as_ptr(), IfIndex);
}

unsafe extern "C" fn CompareAdapters(elem1: *const ::core::ffi::c_void, elem2: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    return (*(elem2 as PIP_ADAPTER_ADDRESSES))


        .__bindgen_anon_1.__bindgen_anon_1.IfIndex
        .wrapping_sub(
            (*(elem1 as PIP_ADAPTER_ADDRESSES))


                .__bindgen_anon_1.__bindgen_anon_1.IfIndex,
        ) as core::ffi::c_int;
}

unsafe extern "C" fn PrintActiveRoutes(mut DestinationPattern: PWSTR) -> DWORD {
    let mut IpForwardTable: PMIB_IPFORWARDTABLE = core::ptr::null_mut();
    let mut pAdapterAddresses: PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();
    let mut Ptr: PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();
    let mut pAdapterSortArray: *mut PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut adaptOutBufLen: ULONG = 15000;
    let mut Destination: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Gateway: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Netmask: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Interface: [WCHAR; IPBUF] = [0; IPBUF];
    let mut i: ::core::ffi::c_uint = 0;
    let mut AdapterCount: ::core::ffi::c_uint = 0;
    let mut AdapterIndex: ::core::ffi::c_uint = 0;
    let mut EntriesFound: BOOL = 0;
    let mut Flags: ULONG = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

    'body: {
        /* set required buffer size */
        pAdapterAddresses = malloc(adaptOutBufLen as size_t) as PIP_ADAPTER_ADDRESSES;
        if pAdapterAddresses.is_null() {
            Error = ERROR_NOT_ENOUGH_MEMORY;
            break 'body;
        }

        if GetAdaptersAddresses(
            AF_INET,
            Flags,
            core::ptr::null_mut(),
            pAdapterAddresses,
            core::ptr::addr_of_mut!(adaptOutBufLen),
        ) == ERROR_BUFFER_OVERFLOW
        {
            free(pAdapterAddresses as *mut ::core::ffi::c_void);
            pAdapterAddresses = malloc(adaptOutBufLen as size_t) as PIP_ADAPTER_ADDRESSES;
            if pAdapterAddresses.is_null() {
                Error = ERROR_NOT_ENOUGH_MEMORY;
                break 'body;
            }
        }

        if (GetIpForwardTable(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Size),
            TRUE,
        )) == ERROR_INSUFFICIENT_BUFFER
        {
            IpForwardTable = malloc(Size as size_t) as PMIB_IPFORWARDTABLE;
            if IpForwardTable.is_null() {
                Error = ERROR_NOT_ENOUGH_MEMORY;
                break 'body;
            }
        }

        Error = GetAdaptersAddresses(
            AF_INET,
            Flags,
            core::ptr::null_mut(),
            pAdapterAddresses,
            core::ptr::addr_of_mut!(adaptOutBufLen),
        );
        if Error == NO_ERROR
            && {
                Error = GetIpForwardTable(
                    IpForwardTable,
                    core::ptr::addr_of_mut!(Size),
                    TRUE,
                );
                Error == NO_ERROR
            }
        {
            ConResPrintf(StdOut, IDS_SEPARATOR);
            ConResPrintf(StdOut, IDS_INTERFACE_LIST);

            AdapterCount = 0;
            Ptr = pAdapterAddresses;
            while !Ptr.is_null() {
                AdapterCount = AdapterCount.wrapping_add(1);
                Ptr = (*Ptr).Next as PIP_ADAPTER_ADDRESSES;
            }

            pAdapterSortArray = malloc(
                (AdapterCount as size_t)
                    .wrapping_mul(::core::mem::size_of::<PVOID>() as size_t),
            ) as *mut PIP_ADAPTER_ADDRESSES;
            if pAdapterSortArray.is_null() {
                Error = ERROR_NOT_ENOUGH_MEMORY;
                break 'body;
            }

            AdapterIndex = 0;
            Ptr = pAdapterAddresses;
            while !Ptr.is_null() {
                *pAdapterSortArray.offset(AdapterIndex as isize) = Ptr;
                AdapterIndex = AdapterIndex.wrapping_add(1);
                Ptr = (*Ptr).Next as PIP_ADAPTER_ADDRESSES;
            }

            qsort(
                pAdapterSortArray as *mut ::core::ffi::c_void,
                AdapterCount as size_t,
                ::core::mem::size_of::<PVOID>() as size_t,
                Some(CompareAdapters as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int),
            );

            AdapterIndex = 0;
            while AdapterIndex < AdapterCount {
                Ptr = *pAdapterSortArray.offset(AdapterIndex as isize);
                if (*Ptr).IfType == IF_TYPE_ETHERNET_CSMACD {
                    let mut PhysicalAddress: [WCHAR; 20] = [0; 20];
                    PrintMacAddress(
                        core::ptr::addr_of_mut!((*Ptr).PhysicalAddress) as PBYTE,
                        core::ptr::addr_of_mut!(PhysicalAddress) as PWSTR,
                    );
                    ConResPrintf(
                        StdOut,
                        IDS_ETHERNET_ENTRY,
                        (*Ptr).__bindgen_anon_1.__bindgen_anon_1.IfIndex,
                        core::ptr::addr_of_mut!(PhysicalAddress) as *mut WCHAR,
                        (*Ptr).Description,
                    );
                } else {
                    ConResPrintf(
                        StdOut,
                        IDS_INTERFACE_ENTRY,
                        (*Ptr).__bindgen_anon_1.__bindgen_anon_1.IfIndex,
                        (*Ptr).Description,
                    );
                }
                AdapterIndex = AdapterIndex.wrapping_add(1);
            }
            ConResPrintf(StdOut, IDS_SEPARATOR);

            ConResPrintf(StdOut, IDS_IPV4_ROUTE_TABLE);
            ConResPrintf(StdOut, IDS_SEPARATOR);
            ConResPrintf(StdOut, IDS_ACTIVE_ROUTES);
            EntriesFound = FALSE;
            i = 0;
            while (i as DWORD) < (*IpForwardTable).dwNumEntries {
                let row: *mut MIB_IPFORWARDROW = (core::ptr::addr_of_mut!((*IpForwardTable).table)
                    as *mut MIB_IPFORWARDROW)
                    .offset(i as isize);

                mbstowcs(
                    core::ptr::addr_of_mut!(Destination) as *mut wchar_t,
                    inet_ntoa(*(&raw const (*row).dwForwardDest as *const in_addr)),
                    IPBUF as size_t,
                );
                mbstowcs(
                    core::ptr::addr_of_mut!(Netmask) as *mut wchar_t,
                    inet_ntoa(*(&raw const (*row).dwForwardMask as *const in_addr)),
                    IPBUF as size_t,
                );
                mbstowcs(
                    core::ptr::addr_of_mut!(Gateway) as *mut wchar_t,
                    inet_ntoa(*(&raw const (*row).dwForwardNextHop as *const in_addr)),
                    IPBUF as size_t,
                );
                GetFirstIPv4AddressFromIndex(
                    core::ptr::addr_of_mut!(Interface) as PWSTR,
                    pAdapterAddresses,
                    (*row).dwForwardIfIndex,
                );

                if MatchWildcard(
                    core::ptr::addr_of_mut!(Destination) as PWSTR,
                    DestinationPattern,
                ) != 0
                {
                    if EntriesFound == FALSE {
                        ConResPrintf(StdOut, IDS_ROUTES_HEADER);
                    }
                    ConResPrintf(
                        StdOut,
                        IDS_ROUTES_ENTRY,
                        core::ptr::addr_of_mut!(Destination) as *mut WCHAR,
                        core::ptr::addr_of_mut!(Netmask) as *mut WCHAR,
                        core::ptr::addr_of_mut!(Gateway) as *mut WCHAR,
                        core::ptr::addr_of_mut!(Interface) as *mut WCHAR,
                        (*row).dwForwardMetric1,
                    );
                    EntriesFound = TRUE;
                }
                i = i.wrapping_add(1);
            }

            if DestinationPattern.is_null() {
                i = 0;
                while (i as DWORD) < (*IpForwardTable).dwNumEntries {
                    let row: *mut MIB_IPFORWARDROW = (core::ptr::addr_of_mut!((*IpForwardTable).table)
                        as *mut MIB_IPFORWARDROW)
                        .offset(i as isize);
                    if (*row).dwForwardDest == 0 {
                        mbstowcs(
                            core::ptr::addr_of_mut!(Gateway) as *mut wchar_t,
                            inet_ntoa(*(&raw const (*row).dwForwardNextHop as *const in_addr)),
                            IPBUF as size_t,
                        );
                        ConResPrintf(
                            StdOut,
                            IDS_DEFAULT_GATEWAY,
                            core::ptr::addr_of_mut!(Gateway) as *mut WCHAR,
                        );
                    }
                    i = i.wrapping_add(1);
                }
            } else if EntriesFound == FALSE {
                ConResPrintf(StdOut, IDS_NONE);
            }
        }
    }

    // Error:
    if !pAdapterSortArray.is_null() {
        free(pAdapterSortArray as *mut ::core::ffi::c_void);
    }
    if !pAdapterAddresses.is_null() {
        free(pAdapterAddresses as *mut ::core::ffi::c_void);
    }
    if !IpForwardTable.is_null() {
        free(IpForwardTable as *mut ::core::ffi::c_void);
    }

    if Error != ERROR_SUCCESS {
        ConResPrintf(StdErr, IDS_ROUTE_ENUM_ERROR);
    }

    return Error;
}

unsafe extern "C" fn PrintRoutes(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
    mut start: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut DestinationPattern: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut Error: DWORD = 0;
    if argc > start {
        ParseCmdLine(
            argc,
            argv,
            start,
            core::ptr::addr_of_mut!(DestinationPattern),
            core::ptr::null_mut::<PWSTR>(),
            core::ptr::null_mut::<PWSTR>(),
            core::ptr::null_mut::<PWSTR>(),
        );
    }
    Error = PrintActiveRoutes(DestinationPattern);
    if Error == ERROR_SUCCESS {
        Error = PrintPersistentRoutes(DestinationPattern);
    }
    return if Error == ERROR_SUCCESS {
        0
    } else {
        2
    };
}

unsafe extern "C" fn ConvertAddCmdLine(
    mut RowToAdd: PMIB_IPFORWARDROW,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
    mut start: ::core::ffi::c_int,
) -> BOOL {
    let mut i: ::core::ffi::c_int;
    let mut addr: [::core::ffi::c_char; 16] = [0 as ::core::ffi::c_char; 16];

    if argc > start {
        wcstombs((addr.as_mut_ptr()).cast(), *argv.offset(start as isize), 16);
        (*RowToAdd).dwForwardDest = inet_addr(addr.as_ptr()) as DWORD;
    } else {
        return FALSE;
    }

    i = start + 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_0B4242BA6CFE.as_ptr()) == 0 {
            i += 1;
            if i >= argc {
                return FALSE;
            }
            wcstombs((addr.as_mut_ptr()).cast(), *argv.offset(i as isize), 16);
            (*RowToAdd).dwForwardMask = inet_addr(addr.as_ptr()) as DWORD;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_BB31F4088066.as_ptr()) == 0 {
            i += 1;
            if i >= argc {
                return FALSE;
            }
            (*RowToAdd).dwForwardMetric1 = _wtoi(*argv.offset(i as isize)) as DWORD;
        } else {
            wcstombs((addr.as_mut_ptr()).cast(), *argv.offset(i as isize), 16);
            (*RowToAdd).dwForwardNextHop = inet_addr(addr.as_ptr()) as DWORD;
        }
        i += 1;
    }

    /* Restrict metric value to range 1 to 9999 */
    if (*RowToAdd).dwForwardMetric1 == 0 as DWORD {
        (*RowToAdd).dwForwardMetric1 = 1 as DWORD;
    } else if (*RowToAdd).dwForwardMetric1 > 9999 as DWORD {
        (*RowToAdd).dwForwardMetric1 = 9999 as DWORD;
    }

    return TRUE;
}

unsafe extern "C" fn CreatePersistentIpForwardEntry(mut RowToAdd: PMIB_IPFORWARDROW) -> DWORD {
    let mut Destination: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Gateway: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Netmask: [WCHAR; IPBUF] = [0; IPBUF];
    let mut ValueName: [WCHAR; 64] = [0; 64];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: DWORD = ERROR_SUCCESS;

    mbstowcs(
        &raw mut Destination as *mut wchar_t,
        inet_ntoa(*(&raw mut (*RowToAdd).dwForwardDest as *mut in_addr)),
        IPBUF as size_t,
    );
    mbstowcs(
        &raw mut Netmask as *mut wchar_t,
        inet_ntoa(*(&raw mut (*RowToAdd).dwForwardMask as *mut in_addr)),
        IPBUF as size_t,
    );
    mbstowcs(
        &raw mut Gateway as *mut wchar_t,
        inet_ntoa(*(&raw mut (*RowToAdd).dwForwardNextHop as *mut in_addr)),
        IPBUF as size_t,
    );

    swprintf(
        &raw mut ValueName as *mut wchar_t,
        RUSTCONV_WSTR_74C59F8927AE.as_ptr() as *const wchar_t,
        &raw mut Destination as *mut WCHAR,
        &raw mut Netmask as *mut WCHAR,
        &raw mut Gateway as *mut WCHAR,
        (*RowToAdd).dwForwardMetric1,
    );

    Error = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_AE1738016435.as_ptr() as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut::<WCHAR>(),
        REG_OPTION_NON_VOLATILE as DWORD,
        KEY_WRITE as REGSAM,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        &raw mut hKey,
        core::ptr::null_mut::<DWORD>(),
    ) as DWORD;
    if Error == ERROR_SUCCESS as DWORD {
        Error = RegSetValueEx(
            hKey,
            &raw mut ValueName as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            REG_SZ as DWORD,
            RUSTCONV_WSTR_96A296D224F2.as_ptr() as *const u16 as LPBYTE as *const BYTE,
            core::mem::size_of::<WCHAR>() as DWORD,
        ) as DWORD;

        RegCloseKey(hKey);
    }

    return Error;
}

unsafe extern "C" fn DeleteCustomRoutes() -> DWORD {
    let mut Destination: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Netmask: [WCHAR; IPBUF] = [0; IPBUF];
    let mut IpForwardTable: PMIB_IPFORWARDTABLE = ::core::ptr::null_mut::<_MIB_IPFORWARDTABLE>();
    let mut Size: ULONG = 0 as ULONG;
    let mut Error: DWORD = ERROR_SUCCESS as DWORD;
    let mut i: ULONG;

    'done: {
        if (GetIpForwardTable(
            ::core::ptr::null_mut::<_MIB_IPFORWARDTABLE>(),
            &raw mut Size,
            TRUE as BOOL,
        )) == ERROR_INSUFFICIENT_BUFFER as DWORD
        {
            IpForwardTable = malloc(Size as size_t) as PMIB_IPFORWARDTABLE;
            if IpForwardTable.is_null() {
                Error = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'done;
            }
        }

        Error = GetIpForwardTable(
            IpForwardTable,
            &raw mut Size,
            TRUE as BOOL,
        );
        if Error != ERROR_SUCCESS as DWORD {
            break 'done;
        }

        i = 0 as ULONG;
        while i < (*IpForwardTable).dwNumEntries {
            mbstowcs(
                &raw mut Destination as *mut wchar_t,
                inet_ntoa(*(&raw mut (*(&raw mut (*IpForwardTable).table as *mut MIB_IPFORWARDROW)
                        .offset(i as isize))
                    .dwForwardDest as *mut in_addr)),
                IPBUF as size_t,
            );
            mbstowcs(
                &raw mut Netmask as *mut wchar_t,
                inet_ntoa(*(&raw mut (*(&raw mut (*IpForwardTable).table as *mut MIB_IPFORWARDROW)
                        .offset(i as isize))
                    .dwForwardMask as *mut in_addr)),
                IPBUF as size_t,
            );

            if (wcscmp(
                &raw mut Netmask as *mut WCHAR,
                RUSTCONV_WSTR_85C12618D8FB.as_ptr(),
            ) != 0)
                && ((wcscmp(
                    &raw mut Destination as *mut WCHAR,
                    RUSTCONV_WSTR_64F21C85D58A.as_ptr(),
                ) != 0)
                    || (wcscmp(
                        &raw mut Netmask as *mut WCHAR,
                        RUSTCONV_WSTR_128F490EEE24.as_ptr(),
                    ) != 0))
                && ((wcscmp(
                    &raw mut Destination as *mut WCHAR,
                    RUSTCONV_WSTR_929E1A6DA5A0.as_ptr(),
                ) != 0)
                    || (wcscmp(
                        &raw mut Netmask as *mut WCHAR,
                        RUSTCONV_WSTR_5A9BC6E8FCA1.as_ptr(),
                    ) != 0))
            {
                Error = DeleteIpForwardEntry(
                    (&raw mut (*IpForwardTable).table as *mut MIB_IPFORWARDROW)
                        .offset(i as isize) as PMIB_IPFORWARDROW,
                );
                if Error != ERROR_SUCCESS as DWORD {
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    }

    if !IpForwardTable.is_null() {
        free(IpForwardTable as *mut ::core::ffi::c_void);
    }

    return Error;
}

unsafe extern "C" fn AddRoute(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
    mut start: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut RowToAdd: MIB_IPFORWARDROW = core::mem::zeroed();
    let mut Error: DWORD;

    if (argc <= start) || ConvertAddCmdLine(&raw mut RowToAdd, argc, argv, start) == 0 {
        return 1;
    }

    if flags & (DELETE_FLAG as i32) != 0 {
        DeleteCustomRoutes();
    }

    if flags & (PERSISTENT_FLAG as i32) != 0 {
        Error = CreatePersistentIpForwardEntry(&raw mut RowToAdd);
    } else {
        Error = CreateIpForwardEntry(&raw mut RowToAdd);
    }
    if Error != ERROR_SUCCESS {
        ConResPrintf(&raw mut csStdErr, IDS_ROUTE_ADD_ERROR);
        return 2;
    }

    return 0;
}

unsafe extern "C" fn DeleteActiveRoutes(mut DestinationPattern: PWSTR) -> DWORD {
    let mut Destination: [WCHAR; IPBUF] = [0; IPBUF];
    let mut Netmask: [WCHAR; IPBUF] = [0; IPBUF];
    let mut IpForwardTable: PMIB_IPFORWARDTABLE = ::core::ptr::null_mut::<_MIB_IPFORWARDTABLE>();
    let mut Size: ULONG = 0 as ULONG;
    let mut Error: DWORD = ERROR_SUCCESS as DWORD;
    let mut i: ULONG;

    'done: {
        if GetIpForwardTable(
            ::core::ptr::null_mut::<_MIB_IPFORWARDTABLE>(),
            &raw mut Size,
            TRUE as BOOL,
        ) == ERROR_INSUFFICIENT_BUFFER as DWORD
        {
            IpForwardTable = malloc(Size as size_t) as PMIB_IPFORWARDTABLE;
            if IpForwardTable.is_null() {
                Error = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'done;
            }
        }

        Error = GetIpForwardTable(IpForwardTable, &raw mut Size, TRUE as BOOL);
        if Error != ERROR_SUCCESS as DWORD {
            break 'done;
        }

        i = 0 as ULONG;
        while i < (*IpForwardTable).dwNumEntries {
            mbstowcs(
                &raw mut Destination as *mut wchar_t,
                inet_ntoa(
                    *(&raw mut (*(&raw mut (*IpForwardTable).table
                        as *mut MIB_IPFORWARDROW)
                        .offset(i as isize))
                    .dwForwardDest as *mut in_addr),
                ),
                IPBUF as size_t,
            );
            mbstowcs(
                &raw mut Netmask as *mut wchar_t,
                inet_ntoa(
                    *(&raw mut (*(&raw mut (*IpForwardTable).table
                        as *mut MIB_IPFORWARDROW)
                        .offset(i as isize))
                    .dwForwardMask as *mut in_addr),
                ),
                IPBUF as size_t,
            );

            if MatchWildcard(&raw mut Destination as PWSTR, DestinationPattern) != 0 {
                Error = DeleteIpForwardEntry(
                    (&raw mut (*IpForwardTable).table as *mut MIB_IPFORWARDROW)
                        .offset(i as isize) as PMIB_IPFORWARDROW,
                );
                if Error != ERROR_SUCCESS as DWORD {
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    }

    if !IpForwardTable.is_null() {
        free(IpForwardTable as *mut ::core::ffi::c_void);
    }

    return Error;
}

unsafe extern "C" fn DeletePersistentRoutes(mut DestinationPattern: PWSTR) -> DWORD {
    let mut szBuffer: [WCHAR; 64] = [0; 64];
    let mut dwIndex: DWORD = 0;
    let mut dwBufferSize: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut Destination: PWSTR = core::ptr::null_mut();

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_AE1738016435.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;
    if Error == ERROR_SUCCESS {
        loop {
            dwBufferSize = 64;

            Error = RegEnumValueW(
                hKey,
                dwIndex,
                core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
                core::ptr::addr_of_mut!(dwBufferSize),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            if Error != ERROR_SUCCESS {
                if Error == ERROR_NO_MORE_ITEMS {
                    Error = ERROR_SUCCESS;
                }

                break;
            }

            ParsePersistentRouteValue(
                core::ptr::addr_of_mut!(szBuffer) as PWSTR,
                core::ptr::addr_of_mut!(Destination),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if MatchWildcard(Destination, DestinationPattern) != 0 {
                Error = RegDeleteValueW(hKey, core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR) as DWORD;
                if Error != ERROR_SUCCESS {
                    break;
                }
            }

            dwIndex = dwIndex.wrapping_add(1);
        }

        RegCloseKey(hKey);
    }

    return Error;
}

unsafe extern "C" fn DeleteRoutes(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
    mut start: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut DestinationPattern: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut Error: DWORD = 0;
    if argc <= start
        || ParseCmdLine(
            argc,
            argv,
            start,
            &raw mut DestinationPattern,
            ::core::ptr::null_mut::<PWSTR>(),
            ::core::ptr::null_mut::<PWSTR>(),
            ::core::ptr::null_mut::<PWSTR>(),
        ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (flags & DELETE_FLAG as ::core::ffi::c_int) != 0 {
        DeleteCustomRoutes();
    }
    Error = DeleteActiveRoutes(DestinationPattern);
    if Error == ERROR_SUCCESS as DWORD {
        Error = DeletePersistentRoutes(DestinationPattern);
    }
    if Error != ERROR_SUCCESS as DWORD {
        ConResPrintf(&raw mut csStdErr, IDS_ROUTE_DEL_ERROR as UINT);
        return 2 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: ::core::ffi::c_int,
    argv: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    i = 1 as ::core::ffi::c_int;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Du16
            || *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
        {
            if _wcsicmp(
                (*argv.offset(i as isize)).offset(1 as isize),
                RUSTCONV_WSTR_9EE4EE67ED95.as_ptr(),
            ) == 0
            {
                flags |= DELETE_FLAG as ::core::ffi::c_int;
            } else if _wcsicmp(
                (*argv.offset(i as isize)).offset(1 as isize),
                RUSTCONV_WSTR_D3FE97979D0F.as_ptr(),
            ) == 0
            {
                flags |= PERSISTENT_FLAG as ::core::ffi::c_int;
            } else {
                ret = 1 as ::core::ffi::c_int;
                break;
            }
        } else {
            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_DACCBBBCFACF.as_ptr()) == 0 {
                ret = PrintRoutes(
                    argc,
                    argv,
                    i + 1 as ::core::ffi::c_int,
                    flags,
                );
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_09C488271038.as_ptr()) == 0 {
                ret = AddRoute(
                    argc,
                    argv,
                    i + 1 as ::core::ffi::c_int,
                    flags,
                );
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_08F79C40983C.as_ptr()) == 0 {
                ret = DeleteRoutes(
                    argc,
                    argv,
                    i + 1 as ::core::ffi::c_int,
                    flags,
                );
            } else {
                ret = 1 as ::core::ffi::c_int;
            }

            break;
        }
        i += 1;
    }

    if argc == 1 as ::core::ffi::c_int {
        ret = 1 as ::core::ffi::c_int;
    }

    if ret == 1 as ::core::ffi::c_int {
        ConResPrintf(
            StdErr,
            IDS_USAGE1 as UINT,
        );
//        ConResPrintf(StdErr, IDS_USAGE2);
        ConResPrintf(
            StdErr,
            IDS_USAGE3 as UINT,
        );
    }

    return if ret == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
