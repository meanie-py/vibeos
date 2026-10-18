use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub dwRtoAlgorithm: DWORD,
    pub RtoAlgorithm: TCP_RTO_ALGORITHM,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub dwForwarding: DWORD,
    pub Forwarding: MIB_IPSTATS_FORWARDING,
}

static mut mib2: [UINT; 6] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut mib2System: [UINT; 7] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut mib2IfNumber: [UINT; 8] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut ifTable: PMIB_IFTABLE = ::core::ptr::null::<_MIB_IFTABLE>() as *mut _MIB_IFTABLE;

static mut mib2IfEntryMap: [structToAsnValue; 21] = unsafe {
    [
        structToAsnValue {
            offset: 512 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 600 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIfRowDescr
                    as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 516 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 520 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 524 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 528 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIfRowPhysAddr
                    as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 540 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 544 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyOperStatus
                    as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 548 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 552 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 556 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 560 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 564 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 568 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 572 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 576 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 580 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 584 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 588 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 592 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 596 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2IfEntry: [UINT; 9] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut mib2Ip: [UINT; 7] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    4 as ::core::ffi::c_int as UINT,
];

static mut ipStats: MIB_IPSTATS = _MIB_IPSTATS {
    u: _MIB_IPSTATS__bindgen_ty_1 { dwForwarding: 0 },
    dwDefaultTTL: 0,
    dwInReceives: 0,
    dwInHdrErrors: 0,
    dwInAddrErrors: 0,
    dwForwDatagrams: 0,
    dwInUnknownProtos: 0,
    dwInDiscards: 0,
    dwInDelivers: 0,
    dwOutRequests: 0,
    dwRoutingDiscards: 0,
    dwOutDiscards: 0,
    dwOutNoRoutes: 0,
    dwReasmTimeout: 0,
    dwReasmReqds: 0,
    dwReasmOks: 0,
    dwReasmFails: 0,
    dwFragOks: 0,
    dwFragFails: 0,
    dwFragCreates: 0,
    dwNumIf: 0,
    dwNumAddr: 0,
    dwNumRoutes: 0,
};

static mut mib2IpMap: [structToAsnValue; 23] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 8 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 20 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 24 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 28 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 32 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 36 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 44 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 48 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 52 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 56 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 60 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 64 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 68 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 72 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 76 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 0 as size_t,
            copy: None,
        },
        structToAsnValue {
            offset: 0 as size_t,
            copy: None,
        },
        structToAsnValue {
            offset: 0 as size_t,
            copy: None,
        },
        structToAsnValue {
            offset: 40 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2IpAddr: [UINT; 9] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    4 as ::core::ffi::c_int as UINT,
    20 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut ipAddrTable: PMIB_IPADDRTABLE =
    ::core::ptr::null::<_MIB_IPADDRTABLE>() as *mut _MIB_IPADDRTABLE;

static mut mib2IpAddrMap: [structToAsnValue; 5] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 8 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2IpRoute: [UINT; 9] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    4 as ::core::ffi::c_int as UINT,
    21 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut ipRouteTable: PMIB_IPFORWARDTABLE =
    ::core::ptr::null::<_MIB_IPFORWARDTABLE>() as *mut _MIB_IPFORWARDTABLE;

static mut mib2IpRouteMap: [structToAsnValue; 12] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 36 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 40 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 44 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 48 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 20 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 24 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 28 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 52 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2IpNet: [UINT; 9] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    4 as ::core::ffi::c_int as UINT,
    22 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut ipNetTable: PMIB_IPNETTABLE =
    ::core::ptr::null::<_MIB_IPNETTABLE>() as *mut _MIB_IPNETTABLE;

static mut mib2IpNetMap: [structToAsnValue; 4] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpNetPhysAddr
                    as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 20 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2Icmp: [UINT; 7] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    5 as ::core::ffi::c_int as UINT,
];

static mut icmpStats: MIB_ICMP = _MIB_ICMP {
    stats: _MIBICMPINFO {
        icmpInStats: _MIBICMPSTATS {
            dwMsgs: 0,
            dwErrors: 0,
            dwDestUnreachs: 0,
            dwTimeExcds: 0,
            dwParmProbs: 0,
            dwSrcQuenchs: 0,
            dwRedirects: 0,
            dwEchos: 0,
            dwEchoReps: 0,
            dwTimestamps: 0,
            dwTimestampReps: 0,
            dwAddrMasks: 0,
            dwAddrMaskReps: 0,
        },
        icmpOutStats: _MIBICMPSTATS {
            dwMsgs: 0,
            dwErrors: 0,
            dwDestUnreachs: 0,
            dwTimeExcds: 0,
            dwParmProbs: 0,
            dwSrcQuenchs: 0,
            dwRedirects: 0,
            dwEchos: 0,
            dwEchoReps: 0,
            dwTimestamps: 0,
            dwTimestampReps: 0,
            dwAddrMasks: 0,
            dwAddrMaskReps: 0,
        },
    },
};

static mut mib2IcmpMap: [structToAsnValue; 26] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 8 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 20 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 24 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 28 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 32 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 36 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 40 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 44 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 48 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 52 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 56 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 60 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 64 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 68 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 72 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 76 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 80 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 84 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 88 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 92 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 96 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 100 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2Tcp: [UINT; 7] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
];

static mut tcpStats: MIB_TCPSTATS = _MIB_TCPSTATS {
    u: _MIB_TCPSTATS__bindgen_ty_1 { dwRtoAlgorithm: 0 },
    dwRtoMin: 0,
    dwRtoMax: 0,
    dwMaxConn: 0,
    dwActiveOpens: 0,
    dwPassiveOpens: 0,
    dwAttemptFails: 0,
    dwEstabResets: 0,
    dwCurrEstab: 0,
    dwInSegs: 0,
    dwOutSegs: 0,
    dwRetransSegs: 0,
    dwInErrs: 0,
    dwOutRsts: 0,
    dwNumConns: 0,
};

static mut mib2TcpMap: [structToAsnValue; 15] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 8 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 16 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 20 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 24 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 28 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 32 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 36 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 40 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 44 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 48 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 52 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 56 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2Udp: [UINT; 7] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    7 as ::core::ffi::c_int as UINT,
];

static mut udpStats: MIB_UDPSTATS = _MIB_UDPSTATS {
    dwInDatagrams: 0,
    dwNoPorts: 0,
    dwInErrors: 0,
    dwOutDatagrams: 0,
    dwNumAddrs: 0,
};

static mut mib2UdpMap: [structToAsnValue; 4] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 8 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 12 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut mib2UdpEntry: [UINT; 9] = [
    1 as ::core::ffi::c_int as UINT,
    3 as ::core::ffi::c_int as UINT,
    6 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    2 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
    7 as ::core::ffi::c_int as UINT,
    5 as ::core::ffi::c_int as UINT,
    1 as ::core::ffi::c_int as UINT,
];

static mut udpTable: PMIB_UDPTABLE = ::core::ptr::null::<_MIB_UDPTABLE>() as *mut _MIB_UDPTABLE;

static mut mib2UdpEntryMap: [structToAsnValue; 2] = unsafe {
    [
        structToAsnValue {
            offset: 0 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyIpAddr as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
        structToAsnValue {
            offset: 4 as ::core::ffi::c_ulonglong as LONG as size_t,
            copy: Some(
                copyInt as unsafe extern "C" fn(*mut AsnAny, *mut ::core::ffi::c_void) -> DWORD,
            ),
        },
    ]
};

static mut supportedIDs: [mibImplementation; 10] = [
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 8]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2IfNumber as *mut UINT,
            },
            init: Some(mib2IfNumberInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IfNumberQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: Some(mib2IfNumberCleanup as unsafe extern "C" fn() -> ()),
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2IfEntry as *mut UINT,
            },
            init: None,
            query: Some(
                mib2IfEntryQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: None,
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2Ip as *mut UINT,
            },
            init: Some(mib2IpStatsInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IpStatsQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: None,
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2IpAddr as *mut UINT,
            },
            init: Some(mib2IpAddrInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IpAddrQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: Some(mib2IpAddrCleanup as unsafe extern "C" fn() -> ()),
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2IpRoute as *mut UINT,
            },
            init: Some(mib2IpRouteInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IpRouteQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: Some(mib2IpRouteCleanup as unsafe extern "C" fn() -> ()),
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2IpNet as *mut UINT,
            },
            init: Some(mib2IpNetInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IpNetQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: Some(mib2IpNetCleanup as unsafe extern "C" fn() -> ()),
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2Icmp as *mut UINT,
            },
            init: Some(mib2IcmpInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2IcmpQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: None,
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2Tcp as *mut UINT,
            },
            init: Some(mib2TcpInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2TcpQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: None,
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2Udp as *mut UINT,
            },
            init: Some(mib2UdpInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2UdpQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: None,
        },
        mibImplementation {
            name: AsnObjectIdentifier {
                idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<UINT>() as usize)
                    as UINT,
                ids: &raw mut mib2UdpEntry as *mut UINT,
            },
            init: Some(mib2UdpEntryInit as unsafe extern "C" fn() -> ()),
            query: Some(
                mib2UdpEntryQuery
                    as unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL,
            ),
            cleanup: Some(mib2UdpEntryCleanup as unsafe extern "C" fn() -> ()),
        },
    ];

static mut minSupportedIDLength: UINT = 0;

// NONAMELESSUNION (define, no Rust equivalent needed: named union fields are used directly)
// WINE_DEFAULT_DEBUG_CHANNEL(inetmib1); (debug-channel declaration, no Rust item emitted)

pub type copyValueFunc = Option<unsafe extern "C" fn(*mut AsnAny, *mut core::ffi::c_void) -> DWORD>;

#[repr(C)]
pub struct structToAsnValue {
    pub offset: usize,
    pub copy: copyValueFunc,
}

pub type varqueryfunc =
    Option<unsafe extern "C" fn(BYTE, *mut SnmpVarBind, *mut AsnInteger32) -> BOOL>;

#[repr(C)]
pub struct mibImplementation {
    pub name: AsnObjectIdentifier,
    pub init: Option<unsafe extern "C" fn()>,
    pub query: varqueryfunc,
    pub cleanup: Option<unsafe extern "C" fn()>,
}

#[repr(C)]
pub struct GenericTable {
    pub numEntries: DWORD,
    pub entries: [BYTE; 1],
}

pub type oidToKeyFunc = Option<unsafe extern "C" fn(*mut AsnObjectIdentifier, *mut core::ffi::c_void)>;
pub type compareFunc =
    Option<unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32>;

// DEFINE_OID(x) expands to { sizeof(x)/sizeof(UINT), x }; inlined below.
// Mutable because AsnObjectIdentifier holds raw pointers (not Sync).

unsafe extern "C" fn copyInt(value: *mut AsnAny, src: *mut core::ffi::c_void) -> DWORD {
    (*value).asnType = ASN_INTEGER as BYTE;
    (*value).asnValue.number = *(src as *mut DWORD) as AsnInteger32;
    return SNMP_ERRORSTATUS_NOERROR as DWORD;
}

unsafe extern "C" fn setStringValue(value: *mut AsnAny, type_0: BYTE, len: DWORD, str: *mut BYTE) {
    let mut strValue: AsnAny = AsnAny {
        asnType: 0,
        asnValue: AsnAny__bindgen_ty_1 { number: 0 },
    };
    strValue.asnType = type_0;
    strValue.asnValue.string.stream = str;
    strValue.asnValue.string.length = len as UINT;
    strValue.asnValue.string.dynamic = FALSE as BOOL;
    SnmpUtilAsnAnyCpy(value, core::ptr::addr_of_mut!(strValue));
}

unsafe extern "C" fn mapStructEntryToValue(
    map: *mut structToAsnValue,
    mapLen: UINT,
    record: *mut core::ffi::c_void,
    mut id: UINT,
    pVarBind: *mut SnmpVarBind,
) -> AsnInteger32 {
    /* OIDs are 1-based */
    if id == 0 {
        return (SNMP_ERRORSTATUS_NOSUCHNAME) as AsnInteger32;
    }
    id = id.wrapping_sub(1);
    if id >= mapLen {
        return SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
    }
    if (*map.offset(id as isize)).copy.is_none() {
        return SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
    }
    return ((*map.offset(id as isize))
        .copy
        .unwrap_unchecked())(
        core::ptr::addr_of_mut!((*pVarBind).value),
        (record as *mut BYTE).offset((*map.offset(id as isize)).offset as isize)
            as *mut core::ffi::c_void,
    ) as AsnInteger32;
}

unsafe extern "C" fn copyIpAddr(value: *mut AsnAny, src: *mut core::ffi::c_void) -> DWORD {
    setStringValue(
        value,
        ASN_IPADDRESS as BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
        src as *mut BYTE,
    );
    return SNMP_ERRORSTATUS_NOERROR as DWORD;
}

unsafe extern "C" fn mib2IfNumberInit() {
    let mut size: DWORD = 0;
    let mut ret: DWORD = GetIfTable(
        core::ptr::null_mut::<MIB_IFTABLE>(),
        core::ptr::addr_of_mut!(size),
        (FALSE) as BOOL,
    );

    if ret == ERROR_INSUFFICIENT_BUFFER {
        let mut table: *mut MIB_IFTABLE = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut MIB_IFTABLE;
        if !table.is_null() {
            if GetIfTable(
                table as PMIB_IFTABLE,
                core::ptr::addr_of_mut!(size),
                (FALSE) as BOOL,
            ) == 0
            {
                ifTable = table as PMIB_IFTABLE;
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    table as PVOID,
                );
            }
        }
    }
}

unsafe extern "C" fn mib2IfNumberCleanup() {
    HeapFree(GetProcessHeap(), 0 as DWORD, ifTable as LPVOID);
}

unsafe extern "C" fn mib2IfNumberQuery(
    mut bPduType: BYTE,
    mut pVarBind: *mut SnmpVarBind,
    mut pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut numberOid: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: (::core::mem::size_of::<[UINT; 8]>() as usize)
            .wrapping_div(::core::mem::size_of::<UINT>() as usize) as UINT,
        ids: core::ptr::addr_of_mut!(mib2IfNumber) as *mut UINT,
    };
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType as ::core::ffi::c_int) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            if bPduType as ::core::ffi::c_int == (SNMP_PDU_GET as i32)
                && SnmpUtilOidNCmp(
                    core::ptr::addr_of_mut!((*pVarBind).name),
                    core::ptr::addr_of_mut!(numberOid),
                    numberOid.idLength,
                ) == 0
                || SnmpUtilOidNCmp(
                    core::ptr::addr_of_mut!((*pVarBind).name),
                    core::ptr::addr_of_mut!(numberOid),
                    numberOid.idLength,
                ) < 0
            {
                let mut numIfs: DWORD = if !ifTable.is_null() {
                    (*ifTable).dwNumEntries
                } else {
                    0 as DWORD
                };

                copyInt(
                    core::ptr::addr_of_mut!((*pVarBind).value),
                    core::ptr::addr_of_mut!(numIfs) as *mut ::core::ffi::c_void,
                );
                if bPduType as ::core::ffi::c_int == (SNMP_PDU_GETNEXT as i32) {
                    SnmpUtilOidFree(core::ptr::addr_of_mut!((*pVarBind).name));
                    SnmpUtilOidCpy(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(numberOid),
                    );
                }
                *pErrorStatus = SNMP_ERRORSTATUS_NOERROR as AsnInteger32;
            } else {
                *pErrorStatus = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
                /* Caller deals with OID if bPduType == SNMP_PDU_GETNEXT, so don't
                 * need to set it here.
                 */
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = SNMP_ERRORSTATUS_READONLY as AsnInteger32;
            ret = FALSE as BOOL;
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
        }
    }
    return ret;
}

unsafe extern "C" fn copyOperStatus(value: *mut AsnAny, src: *mut ::core::ffi::c_void) -> DWORD {
    (*value).asnType = ASN_INTEGER as BYTE;
    match (*(src as *mut DWORD)) as i32 {
        MIB_IF_OPER_STATUS_OPERATIONAL => {
            (*value).asnValue.number = MIB_IF_ADMIN_STATUS_UP as AsnInteger32;
        }
        MIB_IF_OPER_STATUS_CONNECTING | MIB_IF_OPER_STATUS_CONNECTED => {
            (*value).asnValue.number = MIB_IF_ADMIN_STATUS_TESTING as AsnInteger32;
        }
        _ => {
            (*value).asnValue.number = MIB_IF_ADMIN_STATUS_DOWN as AsnInteger32;
        }
    }
    return SNMP_ERRORSTATUS_NOERROR as DWORD;
}

unsafe extern "C" fn getItemAndIntegerInstanceFromOid(
    mut oid: *mut AsnObjectIdentifier,
    mut base: *mut AsnObjectIdentifier,
    mut bPduType: BYTE,
    mut item: *mut UINT,
    mut instance: *mut UINT,
) -> AsnInteger32 {
    let mut ret: AsnInteger32 = (SNMP_ERRORSTATUS_NOERROR as i32);

    match (bPduType as ::core::ffi::c_int) as u32 {
        SNMP_PDU_GETNEXT => {
            if SnmpUtilOidNCmp(oid, base, (*base).idLength) < 0 {
                *item = 1;
                *instance = 1;
            } else if !(SnmpUtilOidNCmp(oid, base, (*base).idLength) != 0) {
                if (*oid).idLength == (*base).idLength
                    || (*oid).idLength == (*base).idLength.wrapping_add(1)
                {
                    *instance = 1;
                    if (*oid).idLength == (*base).idLength.wrapping_add(1) {
                        *item = *(*oid).ids.offset((*base).idLength as isize);
                    } else {
                        *item = 1;
                    }
                } else {
                    *item = *(*oid).ids.offset((*base).idLength as isize);
                    *instance = (*(*oid)
                        .ids
                        .offset((*base).idLength.wrapping_add(1) as isize))
                    .wrapping_add(1);
                }
            } else {
                ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            }
        }
        _ => {
            if SnmpUtilOidNCmp(oid, base, (*base).idLength) == 0 {
                if (*oid).idLength == (*base).idLength
                    || (*oid).idLength == (*base).idLength.wrapping_add(1)
                {
                    ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
                } else {
                    *item = *(*oid).ids.offset((*base).idLength as isize);
                    *instance = *(*oid)
                        .ids
                        .offset((*base).idLength.wrapping_add(1) as isize);
                }
            } else {
                ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn getItemFromOid(
    oid: *mut AsnObjectIdentifier,
    base: *mut AsnObjectIdentifier,
    bPduType: BYTE,
    item: *mut UINT,
) -> AsnInteger32 {
    let mut ret: AsnInteger32 = (SNMP_ERRORSTATUS_NOERROR as i32);

    if bPduType as i32 == (SNMP_PDU_GETNEXT as i32) {
        if SnmpUtilOidNCmp(oid, base, (*base).idLength) < 0 {
            *item = 1;
        } else if !(SnmpUtilOidNCmp(oid, base, (*base).idLength) != 0) {
            if (*oid).idLength == (*base).idLength {
                /* The item is missing, assume the first item */
                *item = 1;
            } else {
                *item = (*(*oid).ids.offset((*base).idLength as isize))
                    .wrapping_add(1);
            }
        } else {
            ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    } else {
        if !(SnmpUtilOidNCmp(oid, base, (*base).idLength) != 0) {
            if (*oid).idLength == (*base).idLength {
                /* The item is missing */
                ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            } else {
                *item = *(*oid).ids.offset((*base).idLength as isize);
                if *item == 0 {
                    ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
                }
            }
        } else {
            ret = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn oidToIpAddr(oid: *mut AsnObjectIdentifier) -> DWORD {
    assert!(!oid.is_null() && (*oid).idLength >= 4);
    /* Map the IDs to an IP address in little-endian order */
    return ((*(*oid).ids.offset(3) as BYTE as DWORD) << 24)
        | ((*(*oid).ids.offset(2) as BYTE as DWORD) << 16)
        | ((*(*oid).ids.offset(1) as BYTE as DWORD) << 8)
        | (*(*oid).ids.offset(0) as BYTE as DWORD);
}

unsafe extern "C" fn findValueInTable(
    key: *const ::core::ffi::c_void,
    table: *mut GenericTable,
    tableEntrySize: size_t,
    compare: compareFunc,
) -> UINT {
    let mut index: UINT = 0;
    let mut value: *mut ::core::ffi::c_void;

    value = bsearch(
        key,
        core::ptr::addr_of_mut!((*table).entries) as *mut BYTE as *const ::core::ffi::c_void,
        (*table).numEntries as size_t,
        tableEntrySize,
        compare,
    );
    if !value.is_null() {
        index = ((((value as *mut BYTE).offset_from(core::ptr::addr_of_mut!((*table).entries) as *mut BYTE)
            as ::core::ffi::c_longlong as size_t)
            / tableEntrySize)
            .wrapping_add(1)) as UINT;
    }
    return index;
}

unsafe extern "C" fn findOidInTable(
    oid: *mut AsnObjectIdentifier,
    table: *mut GenericTable,
    tableEntrySize: size_t,
    makeKey: oidToKeyFunc,
    compare: compareFunc,
) -> UINT {
    let mut index: UINT = 0 as UINT;
    let mut key: *mut ::core::ffi::c_void = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        tableEntrySize as SIZE_T,
    ) as *mut ::core::ffi::c_void;

    if !key.is_null() {
        makeKey.unwrap_unchecked()(oid, key);
        index = findValueInTable(key, table, tableEntrySize, compare);
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            key as PVOID,
        );
    }
    return index;
}

unsafe extern "C" fn findNextOidInTable(mut oid: *mut AsnObjectIdentifier, mut table: *mut GenericTable, mut tableEntrySize: size_t, mut makeKey: oidToKeyFunc, mut compare: compareFunc) -> UINT {
    let mut index: UINT = 0;
    let mut key: *mut ::core::ffi::c_void = HeapAlloc(GetProcessHeap(), 0, (tableEntrySize) as SIZE_T) as *mut ::core::ffi::c_void;

    if !key.is_null() {
        (makeKey.unwrap_unchecked())(oid, key);
        index = findValueInTable(key, table, tableEntrySize, compare);
        if index == 0 {
            /* Not found in table.  If it's less than the first entry, return
             * the first index.  Otherwise just return 0 and let the caller
             * handle finding the successor.
             */
            if (compare.unwrap_unchecked())(key, core::ptr::addr_of_mut!((*table).entries) as *mut BYTE as *const ::core::ffi::c_void) < 0 {
                index = 1;
            }
        } else {
            /* Skip any entries that match the same key.  This enumeration will
             * be incomplete, but it's what Windows appears to do if there are
             * multiple entries with the same index in a table, and it avoids
             * an infinite loop.
             */
            index = index.wrapping_add(1);
            while index <= (*table).numEntries && (compare.unwrap_unchecked())(key, (core::ptr::addr_of_mut!((*table).entries) as *mut BYTE).add(tableEntrySize.wrapping_mul(index.wrapping_sub(1) as size_t)) as *mut BYTE as *const ::core::ffi::c_void) == 0 {
                index = index.wrapping_add(1);
            }
        }
        HeapFree(GetProcessHeap(), 0, key as PVOID);
    }
    return index;
}

unsafe extern "C" fn getItemAndInstanceFromTable(
    oid: *mut AsnObjectIdentifier,
    base: *mut AsnObjectIdentifier,
    instanceLen: UINT,
    bPduType: BYTE,
    table: *mut GenericTable,
    tableEntrySize: size_t,
    makeKey: oidToKeyFunc,
    compare: compareFunc,
    item: *mut UINT,
    instance: *mut UINT,
) -> AsnInteger32 {
    let mut ret: AsnInteger32 = SNMP_ERRORSTATUS_NOERROR as AsnInteger32;

    if table.is_null() {
        return SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
    }

    match (bPduType as ::core::ffi::c_int) as u32 {
        SNMP_PDU_GETNEXT => {
            if SnmpUtilOidNCmp(oid, base, (*base).idLength) < 0 {
                /* Return the first item and instance from the table */
                *item = 1 as UINT;
                *instance = 1 as UINT;
            } else if SnmpUtilOidNCmp(oid, base, (*base).idLength) == 0
                && (*oid).idLength
                    < (*base)
                        .idLength
                        .wrapping_add(instanceLen)
                        .wrapping_add(1 as UINT)
            {
                /* Either the table or an item is specified, but the instance is
                 * not.
                 */
                *instance = 1 as UINT;
                if (*oid).idLength >= (*base).idLength.wrapping_add(1 as UINT) {
                    *item = *(*oid).ids.offset((*base).idLength as isize);
                    if *item == 0 {
                        *item = 1 as UINT;
                    }
                } else {
                    *item = 1 as UINT;
                }
            } else if SnmpUtilOidNCmp(oid, base, (*base).idLength) == 0
                && (*oid).idLength
                    == (*base)
                        .idLength
                        .wrapping_add(instanceLen)
                        .wrapping_add(1 as UINT)
            {
                *item = *(*oid).ids.offset((*base).idLength as isize);
                if *item == 0 {
                    *instance = 1 as UINT;
                    *item = 1 as UINT;
                } else {
                    let mut instanceOid: AsnObjectIdentifier = AsnObjectIdentifier {
                        idLength: instanceLen,
                        ids: (*oid)
                            .ids
                            .offset((*base).idLength as isize)
                            .offset(1 as isize),
                    };
                    *instance = findNextOidInTable(
                        core::ptr::addr_of_mut!(instanceOid),
                        table,
                        tableEntrySize,
                        makeKey,
                        compare,
                    );
                    if *instance == 0 || *instance > (*table).numEntries {
                        ret = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
                    }
                }
            } else {
                ret = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
            }
        }
        _ => {
            if SnmpUtilOidNCmp(oid, base, (*base).idLength) == 0
                && (*oid).idLength
                    == (*base)
                        .idLength
                        .wrapping_add(instanceLen)
                        .wrapping_add(1 as UINT)
            {
                *item = *(*oid).ids.offset((*base).idLength as isize);
                if *item == 0 {
                    ret = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
                } else {
                    let mut instanceOid_0: AsnObjectIdentifier = AsnObjectIdentifier {
                        idLength: instanceLen,
                        ids: (*oid)
                            .ids
                            .offset((*base).idLength as isize)
                            .offset(1 as isize),
                    };
                    *instance = findOidInTable(
                        core::ptr::addr_of_mut!(instanceOid_0),
                        table,
                        tableEntrySize,
                        makeKey,
                        compare,
                    );
                    if *instance == 0 {
                        ret = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
                    }
                }
            } else {
                ret = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
            }
        }
    }
    return ret;
}

unsafe extern "C" fn setOidWithItem(dst: *mut AsnObjectIdentifier, base: *mut AsnObjectIdentifier, item: UINT) -> INT {
    let mut id: UINT = 0;
    let mut oid: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: 0,
        ids: core::ptr::null_mut(),
    };
    let mut ret: INT = 0;
    SnmpUtilOidFree(dst);
    ret = SnmpUtilOidCpy(dst, base);
    if ret != 0 {
        oid.idLength = 1;
        oid.ids = core::ptr::addr_of_mut!(id);
        id = item;
        ret = SnmpUtilOidAppend(dst, core::ptr::addr_of_mut!(oid));
    }
    return ret;
}

unsafe extern "C" fn setOidWithItemAndIpAddr(
    dst: *mut AsnObjectIdentifier,
    base: *mut AsnObjectIdentifier,
    item: UINT,
    mut addr: DWORD,
) -> INT {
    let mut id: UINT = 0;
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    let mut oid: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: 0,
        ids: core::ptr::null_mut(),
    };
    let mut ret: INT;
    ret = setOidWithItem(dst, base, item);
    if ret != 0 {
        oid.idLength = 1;
        oid.ids = core::ptr::addr_of_mut!(id);
        ptr = core::ptr::addr_of_mut!(addr) as *mut BYTE;
        while ret != 0
            && ptr
                < (core::ptr::addr_of_mut!(addr) as *mut BYTE)
                    .add(core::mem::size_of::<DWORD>())
        {
            id = *ptr as UINT;
            ret = SnmpUtilOidAppend(dst, core::ptr::addr_of_mut!(oid));
            ptr = ptr.add(1);
        }
    }
    return ret;
}

unsafe extern "C" fn setOidWithItemAndInteger(
    dst: *mut AsnObjectIdentifier,
    base: *mut AsnObjectIdentifier,
    item: UINT,
    mut instance: UINT,
) -> INT {
    let mut oid: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: 0,
        ids: core::ptr::null_mut::<UINT>(),
    };
    let mut ret: INT;
    ret = setOidWithItem(dst, base, item);
    if ret != 0 {
        oid.idLength = 1;
        oid.ids = core::ptr::addr_of_mut!(instance);
        ret = SnmpUtilOidAppend(dst, core::ptr::addr_of_mut!(oid));
    }
    return ret;
}

unsafe extern "C" fn copyIfRowDescr(mut value: *mut AsnAny, mut src: *mut ::core::ffi::c_void) -> DWORD {
    let mut row: PMIB_IFROW = (src as *mut BYTE)
        .sub(core::mem::offset_of!(MIB_IFROW, dwDescrLen)) as PMIB_IFROW;
    let mut ret: DWORD = 0;
    if (*row).dwDescrLen != 0 {
        setStringValue(
            value,
            ASN_OCTETSTRING as BYTE,
            (*row).dwDescrLen,
            core::ptr::addr_of_mut!((*row).bDescr) as *mut BYTE,
        );
        ret = SNMP_ERRORSTATUS_NOERROR as DWORD;
    } else {
        ret = SNMP_ERRORSTATUS_NOSUCHNAME as DWORD;
    }
    return ret;
}

unsafe extern "C" fn copyIfRowPhysAddr(value: *mut AsnAny, src: *mut core::ffi::c_void) -> DWORD {
    let row: PMIB_IFROW = (src as *mut BYTE).sub(core::mem::offset_of!(MIB_IFROW, dwPhysAddrLen)) as PMIB_IFROW;
    let mut ret: DWORD;

    if (*row).dwPhysAddrLen != 0 {
        setStringValue(
            value,
            ASN_OCTETSTRING as BYTE,
            (*row).dwPhysAddrLen,
            core::ptr::addr_of_mut!((*row).bPhysAddr) as *mut BYTE,
        );
        ret = SNMP_ERRORSTATUS_NOERROR as DWORD;
    } else {
        ret = SNMP_ERRORSTATUS_NOSUCHNAME as DWORD;
    }
    return ret;
}

unsafe extern "C" fn mib2IfEntryQuery(
    mut bPduType: BYTE,
    mut pVarBind: *mut SnmpVarBind,
    mut pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut entryOid: AsnObjectIdentifier = DEFINE_OID!(mib2IfEntry);
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            if ifTable.is_null() {
                /* There is no interface present, so let the caller deal
                 * with finding the successor.
                 */
                *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            } else {
                let mut tableIndex: UINT = 0;
                let mut item: UINT = 0;

                *pErrorStatus = getItemAndIntegerInstanceFromOid(
                    core::ptr::addr_of_mut!((*pVarBind).name),
                    core::ptr::addr_of_mut!(entryOid),
                    bPduType,
                    core::ptr::addr_of_mut!(item),
                    core::ptr::addr_of_mut!(tableIndex),
                );
                if *pErrorStatus == 0 {
                    assert!(tableIndex != 0);
                    assert!(item != 0);
                    if tableIndex > (*ifTable).dwNumEntries {
                        *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
                    } else {
                        *pErrorStatus = mapStructEntryToValue(
                            core::ptr::addr_of_mut!(mib2IfEntryMap) as *mut structToAsnValue,
                            DEFINE_SIZEOF!(mib2IfEntryMap) as UINT,
                            (core::ptr::addr_of_mut!((*ifTable).table) as *mut MIB_IFROW)
                                .offset(tableIndex.wrapping_sub(1 as UINT)
                                    as isize)
                                as *mut MIB_IFROW
                                as *mut ::core::ffi::c_void,
                            item,
                            pVarBind,
                        );
                        if bPduType == (SNMP_PDU_GETNEXT as u8) {
                            ret = setOidWithItemAndInteger(
                                core::ptr::addr_of_mut!((*pVarBind).name),
                                core::ptr::addr_of_mut!(entryOid),
                                item,
                                tableIndex,
                            ) as BOOL;
                        }
                    }
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!(
                b"0x%02x: unsupported PDU type\n\0" as *const u8 as *const ::core::ffi::c_char,
                bPduType as ::core::ffi::c_int,
            );
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn mib2IpStatsInit() {
    GetIpStatistics(core::ptr::addr_of_mut!(ipStats));
}

unsafe extern "C" fn mib2IpStatsQuery(
    bPduType: BYTE,
    pVarBind: *mut SnmpVarBind,
    pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2Ip);
    let mut item: UINT = 0;
    let mut ret: BOOL = (TRUE as i32);

    TRACE!(
        "(0x%02x, %s, %p)\n",
        bPduType as i32,
        SnmpUtilOidToA(core::ptr::addr_of_mut!((*pVarBind).name)),
        pErrorStatus
    );

    match (bPduType as i32) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemFromOid(
                core::ptr::addr_of_mut!((*pVarBind).name),
                core::ptr::addr_of_mut!(myOid),
                bPduType,
                core::ptr::addr_of_mut!(item),
            );
            if *pErrorStatus == 0 {
                *pErrorStatus = mapStructEntryToValue(
                    core::ptr::addr_of_mut!(mib2IpMap) as *mut structToAsnValue,
                    DEFINE_SIZEOF!(mib2IpMap) as UINT,
                    core::ptr::addr_of_mut!(ipStats) as *mut core::ffi::c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0 && bPduType as i32 == (SNMP_PDU_GETNEXT as i32) {
                    ret = setOidWithItem(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(myOid),
                        item,
                    ) as BOOL;
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as i32);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn mib2IpAddrInit() {
    let mut size: DWORD = 0;
    let ret: DWORD = GetIpAddrTable(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
        (TRUE) as BOOL,
    );

    if ret == ERROR_INSUFFICIENT_BUFFER {
        let mut table: *mut MIB_IPADDRTABLE = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut MIB_IPADDRTABLE;
        if !table.is_null() {
            if GetIpAddrTable(
                table as PMIB_IPADDRTABLE,
                core::ptr::addr_of_mut!(size),
                (TRUE) as BOOL,
            ) == 0
            {
                ipAddrTable = table as PMIB_IPADDRTABLE;
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    table as PVOID,
                );
            }
        }
    }
}

unsafe extern "C" fn mib2IpAddrCleanup() {
    HeapFree(GetProcessHeap(), 0, ipAddrTable as LPVOID);
}

unsafe extern "C" fn oidToIpAddrRow(oid: *mut AsnObjectIdentifier, dst: *mut core::ffi::c_void) {
    let mut row: *mut MIB_IPADDRROW = dst as *mut MIB_IPADDRROW;
    (*row).dwAddr = oidToIpAddr(oid);
}

unsafe extern "C" fn compareIpAddrRow(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let key: *const MIB_IPADDRROW = a as *const MIB_IPADDRROW;
    let value: *const MIB_IPADDRROW = b as *const MIB_IPADDRROW;

    return (*key).dwAddr.wrapping_sub((*value).dwAddr) as core::ffi::c_int;
}

unsafe extern "C" fn mib2IpAddrQuery(
    mut bPduType: BYTE,
    mut pVarBind: *mut SnmpVarBind,
    mut pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: (::core::mem::size_of::<[UINT; 9]>() as usize)
            .wrapping_div(::core::mem::size_of::<UINT>() as usize) as UINT,
        ids: &raw mut mib2IpAddr as *mut UINT,
    };
    let mut tableIndex: UINT = 0 as UINT;
    let mut item: UINT = 0 as UINT;
    let mut ret: BOOL = TRUE as BOOL;

    TRACE!(
        "(0x%02x, %s, %p)\n",
        bPduType as ::core::ffi::c_int,
        SnmpUtilOidToA(&raw mut (*pVarBind).name),
        pErrorStatus
    );

    match (bPduType as ::core::ffi::c_int) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemAndInstanceFromTable(
                &raw mut (*pVarBind).name,
                &raw mut myOid,
                4 as UINT,
                bPduType,
                ipAddrTable as *mut GenericTable,
                ::core::mem::size_of::<MIB_IPADDRROW>() as size_t,
                Some(
                    oidToIpAddrRow
                        as unsafe extern "C" fn(
                            *mut AsnObjectIdentifier,
                            *mut ::core::ffi::c_void,
                        ),
                ),
                Some(
                    compareIpAddrRow
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
                &raw mut item,
                &raw mut tableIndex,
            );
            if *pErrorStatus == 0 {
                assert!(tableIndex != 0);
                assert!(item != 0);
                *pErrorStatus = mapStructEntryToValue(
                    &raw mut mib2IpAddrMap as *mut structToAsnValue,
                    (::core::mem::size_of::<[structToAsnValue; 5]>() as usize)
                        .wrapping_div(::core::mem::size_of::<structToAsnValue>() as usize)
                        as UINT,
                    (&raw mut (*ipAddrTable).table as *mut MIB_IPADDRROW)
                        .offset(tableIndex.wrapping_sub(1 as UINT) as isize)
                        as *mut MIB_IPADDRROW as *mut ::core::ffi::c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0
                    && bPduType as ::core::ffi::c_int == (SNMP_PDU_GETNEXT as i32)
                {
                    ret = setOidWithItemAndIpAddr(
                        &raw mut (*pVarBind).name,
                        &raw mut myOid,
                        item,
                        (*(&raw mut (*ipAddrTable).table as *mut MIB_IPADDRROW)
                            .offset(tableIndex.wrapping_sub(1 as UINT) as isize))
                        .dwAddr,
                    ) as BOOL;
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = SNMP_ERRORSTATUS_READONLY as AsnInteger32;
            ret = FALSE as BOOL;
        }
        _ => {
            FIXME!(
                "0x%02x: unsupported PDU type\n",
                bPduType as ::core::ffi::c_int
            );
            *pErrorStatus = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
        }
    }
    return ret;
}

unsafe extern "C" fn mib2IpRouteInit() {
    let mut size: DWORD = 0 as DWORD;
    let mut ret: DWORD = GetIpForwardTable(core::ptr::null_mut(), core::ptr::addr_of_mut!(size), TRUE as BOOL);

    if ret == ERROR_INSUFFICIENT_BUFFER as DWORD {
        let mut table: *mut MIB_IPFORWARDTABLE = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut MIB_IPFORWARDTABLE;
        if !table.is_null() {
            if GetIpForwardTable(table as PMIB_IPFORWARDTABLE, core::ptr::addr_of_mut!(size), TRUE as BOOL) == 0 {
                ipRouteTable = table as PMIB_IPFORWARDTABLE;
            } else {
                HeapFree(GetProcessHeap(), 0 as DWORD, table as PVOID);
            }
        }
    }
}

unsafe extern "C" fn mib2IpRouteCleanup() {
    HeapFree(GetProcessHeap(), 0, ipRouteTable as LPVOID);
}

unsafe extern "C" fn oidToIpForwardRow(oid: *mut AsnObjectIdentifier, dst: *mut core::ffi::c_void) {
    let mut row: *mut MIB_IPFORWARDROW = dst as *mut MIB_IPFORWARDROW;
    (*row).dwForwardDest = oidToIpAddr(oid);
}

unsafe extern "C" fn compareIpForwardRow(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let key: *const MIB_IPFORWARDROW = a as *const MIB_IPFORWARDROW;
    let value: *const MIB_IPFORWARDROW = b as *const MIB_IPFORWARDROW;

    return (*key).dwForwardDest.wrapping_sub((*value).dwForwardDest) as core::ffi::c_int;
}

unsafe extern "C" fn mib2IpRouteQuery(
    bPduType: BYTE,
    pVarBind: *mut SnmpVarBind,
    pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2IpRoute);
    let mut tableIndex: UINT = 0;
    let mut item: UINT = 0;
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemAndInstanceFromTable(
                core::ptr::addr_of_mut!((*pVarBind).name),
                core::ptr::addr_of_mut!(myOid),
                4,
                bPduType,
                ipRouteTable as *mut GenericTable,
                core::mem::size_of::<MIB_IPFORWARDROW>() as size_t,
                Some(oidToIpForwardRow),
                Some(compareIpForwardRow),
                core::ptr::addr_of_mut!(item),
                core::ptr::addr_of_mut!(tableIndex),
            );
            if *pErrorStatus == 0 {
                assert!(tableIndex != 0);
                assert!(item != 0);
                *pErrorStatus = mapStructEntryToValue(
                    core::ptr::addr_of_mut!(mib2IpRouteMap) as *mut structToAsnValue,
                    DEFINE_SIZEOF!(mib2IpRouteMap) as UINT,
                    core::ptr::addr_of_mut!(
                        (*(*ipRouteTable).table.as_mut_ptr().add(tableIndex.wrapping_sub(1) as usize))
                    ) as *mut core::ffi::c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0 && bPduType == (SNMP_PDU_GETNEXT as u8) {
                    ret = setOidWithItemAndIpAddr(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(myOid),
                        item,
                        (*(*ipRouteTable).table.as_ptr().add((tableIndex - 1) as usize))
                            .dwForwardDest,
                    );
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn copyIpNetPhysAddr(value: *mut AsnAny, src: *mut core::ffi::c_void) -> DWORD {
    let row: PMIB_IPNETROW = (src as *mut BYTE).offset(-(core::mem::offset_of!(MIB_IPNETROW, dwPhysAddrLen) as isize)) as PMIB_IPNETROW;
    setStringValue(value, ASN_OCTETSTRING as BYTE, (*row).dwPhysAddrLen, core::ptr::addr_of_mut!((*row).bPhysAddr) as *mut BYTE);
    return SNMP_ERRORSTATUS_NOERROR as DWORD;
}

unsafe extern "C" fn mib2IpNetInit() {
    let mut size: DWORD = 0 as DWORD;
    let mut ret: DWORD = GetIpNetTable(
        ::core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
        FALSE as BOOL,
    );

    if ret == ERROR_INSUFFICIENT_BUFFER as DWORD {
        let mut table: *mut MIB_IPNETTABLE = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            size as SIZE_T,
        ) as *mut MIB_IPNETTABLE;
        if !table.is_null() {
            if GetIpNetTable(
                table as PMIB_IPNETTABLE,
                core::ptr::addr_of_mut!(size),
                FALSE as BOOL,
            ) == 0
            {
                ipNetTable = table as PMIB_IPNETTABLE;
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    table as PVOID,
                );
            }
        }
    }
}

unsafe extern "C" fn mib2IpNetCleanup() {
    HeapFree(GetProcessHeap(), 0, ipNetTable as PVOID);
}

unsafe extern "C" fn mib2IpNetQuery(bPduType: BYTE, pVarBind: *mut SnmpVarBind, pErrorStatus: *mut AsnInteger32) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2IpNet);
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            if ipNetTable.is_null() {
                *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            } else {
                let mut tableIndex: UINT = 0;
                let mut item: UINT = 0;

                *pErrorStatus = getItemAndIntegerInstanceFromOid(
                    &raw mut (*pVarBind).name,
                    &raw mut myOid,
                    bPduType,
                    &raw mut item,
                    &raw mut tableIndex,
                );
                if *pErrorStatus == 0 {
                    assert!(tableIndex != 0);
                    assert!(item != 0);
                    if tableIndex > (*ipNetTable).dwNumEntries {
                        *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
                    } else {
                        *pErrorStatus = mapStructEntryToValue(
                            &raw mut mib2IpNetMap as *mut structToAsnValue,
                            DEFINE_SIZEOF!(mib2IpNetMap) as UINT,
                            ipNetTable.offset(tableIndex.wrapping_sub(1) as isize)
                                as *mut _MIB_IPNETTABLE
                                as *mut ::core::ffi::c_void,
                            item,
                            pVarBind,
                        );
                        if *pErrorStatus == 0 && bPduType == (SNMP_PDU_GETNEXT as u8) {
                            ret = setOidWithItemAndInteger(
                                &raw mut (*pVarBind).name,
                                &raw mut myOid,
                                item,
                                tableIndex,
                            );
                        }
                    }
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn mib2IcmpInit() {
    GetIcmpStatistics(core::ptr::addr_of_mut!(icmpStats));
}

unsafe extern "C" fn mib2IcmpQuery(
    bPduType: BYTE,
    pVarBind: *mut SnmpVarBind,
    pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2Icmp);
    let mut item: UINT = 0 as UINT;
    let mut ret: BOOL = TRUE as BOOL;

    TRACE!(
        "(0x%02x, %s, %p)\n",
        bPduType as c_int,
        SnmpUtilOidToA(core::ptr::addr_of_mut!((*pVarBind).name)),
        pErrorStatus
    );

    match (bPduType as c_int) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemFromOid(
                core::ptr::addr_of_mut!((*pVarBind).name),
                core::ptr::addr_of_mut!(myOid),
                bPduType,
                core::ptr::addr_of_mut!(item),
            );
            if *pErrorStatus == 0 {
                *pErrorStatus = mapStructEntryToValue(
                    core::ptr::addr_of_mut!(mib2IcmpMap) as *mut structToAsnValue,
                    DEFINE_SIZEOF!(mib2IcmpMap) as UINT,
                    core::ptr::addr_of_mut!(icmpStats) as *mut c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0 && bPduType as c_int == (SNMP_PDU_GETNEXT as i32) {
                    ret = setOidWithItem(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(myOid),
                        item,
                    ) as BOOL;
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = SNMP_ERRORSTATUS_READONLY as AsnInteger32;
            ret = FALSE as BOOL;
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as c_int);
            *pErrorStatus = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
        }
    }
    return ret;
}

unsafe extern "C" fn mib2TcpInit() {
    GetTcpStatistics(core::ptr::addr_of_mut!(tcpStats));
}

unsafe extern "C" fn mib2TcpQuery(
    mut bPduType: BYTE,
    mut pVarBind: *mut SnmpVarBind,
    mut pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2Tcp);
    let mut item: UINT = 0;
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType as ::core::ffi::c_int) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemFromOid(
                core::ptr::addr_of_mut!((*pVarBind).name),
                core::ptr::addr_of_mut!(myOid),
                bPduType,
                core::ptr::addr_of_mut!(item),
            );
            if *pErrorStatus == 0 {
                *pErrorStatus = mapStructEntryToValue(
                    core::ptr::addr_of_mut!(mib2TcpMap) as *mut structToAsnValue,
                    DEFINE_SIZEOF!(mib2TcpMap) as UINT,
                    core::ptr::addr_of_mut!(tcpStats) as *mut ::core::ffi::c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0 && bPduType as ::core::ffi::c_int == (SNMP_PDU_GETNEXT as i32) {
                    ret = setOidWithItem(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(myOid),
                        item,
                    ) as BOOL;
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn mib2UdpInit() {
    GetUdpStatistics(core::ptr::addr_of_mut!(udpStats));
}

unsafe extern "C" fn mib2UdpQuery(
    bPduType: BYTE,
    pVarBind: *mut SnmpVarBind,
    pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2Udp);
    let mut item: UINT = 0;
    let mut ret: BOOL = (TRUE as i32);

    match (bPduType as i32) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            *pErrorStatus = getItemFromOid(
                core::ptr::addr_of_mut!((*pVarBind).name),
                core::ptr::addr_of_mut!(myOid),
                bPduType,
                core::ptr::addr_of_mut!(item),
            );
            if *pErrorStatus == 0 {
                *pErrorStatus = mapStructEntryToValue(
                    core::ptr::addr_of_mut!(mib2UdpMap) as *mut structToAsnValue,
                    DEFINE_SIZEOF!(mib2UdpMap) as UINT,
                    core::ptr::addr_of_mut!(udpStats) as *mut core::ffi::c_void,
                    item,
                    pVarBind,
                );
                if *pErrorStatus == 0 && bPduType as i32 == (SNMP_PDU_GETNEXT as i32) {
                    ret = setOidWithItem(
                        core::ptr::addr_of_mut!((*pVarBind).name),
                        core::ptr::addr_of_mut!(myOid),
                        item,
                    );
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn mib2UdpEntryInit() {
    let mut size: DWORD = 0;
    let mut ret: DWORD = GetUdpTable(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
        (TRUE) as BOOL,
    );

    if ret == ERROR_INSUFFICIENT_BUFFER {
        let mut table: *mut MIB_UDPTABLE = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut MIB_UDPTABLE;
        if !table.is_null() {
            if GetUdpTable(
                table as PMIB_UDPTABLE,
                core::ptr::addr_of_mut!(size),
                (TRUE) as BOOL,
            ) == 0
            {
                udpTable = table as PMIB_UDPTABLE;
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    table as PVOID,
                );
            }
        }
    }
}

unsafe extern "C" fn mib2UdpEntryCleanup() {
    HeapFree(GetProcessHeap(), 0 as DWORD, udpTable as PVOID);
}

unsafe extern "C" fn oidToUdpRow(oid: *mut AsnObjectIdentifier, dst: *mut core::ffi::c_void) {
    let row: *mut MIB_UDPROW = dst as *mut MIB_UDPROW;

    assert!(!oid.is_null() && (*oid).idLength >= 5);
    (*row).dwLocalAddr = oidToIpAddr(oid);
    (*row).dwLocalPort = *(*oid).ids.offset(4 as isize) as DWORD;
}

unsafe extern "C" fn compareUdpRow(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let key: *const MIB_UDPROW = a as *const MIB_UDPROW;
    let value: *const MIB_UDPROW = b as *const MIB_UDPROW;
    let mut ret: core::ffi::c_int;
    ret = (*key).dwLocalAddr.wrapping_sub((*value).dwLocalAddr) as core::ffi::c_int;
    if ret == 0 {
        ret = (*key).dwLocalPort.wrapping_sub((*value).dwLocalPort) as core::ffi::c_int;
    }
    return ret;
}

unsafe extern "C" fn mib2UdpEntryQuery(
    mut bPduType: BYTE,
    mut pVarBind: *mut SnmpVarBind,
    mut pErrorStatus: *mut AsnInteger32,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2UdpEntry);
    let mut ret: BOOL = (TRUE as i32);
    match (bPduType) as u32 {
        SNMP_PDU_GET | SNMP_PDU_GETNEXT => {
            if udpTable.is_null() {
                *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
            } else {
                let mut tableIndex: UINT = 0;
                let mut item: UINT = 0;

                *pErrorStatus = getItemAndInstanceFromTable(
                    &raw mut (*pVarBind).name,
                    &raw mut myOid,
                    5,
                    bPduType,
                    udpTable as *mut GenericTable,
                    core::mem::size_of::<MIB_UDPROW>() as size_t,
                    Some(oidToUdpRow),
                    Some(compareUdpRow),
                    &raw mut item,
                    &raw mut tableIndex,
                );
                if *pErrorStatus == 0 {
                    assert!(tableIndex != 0);
                    assert!(item != 0);
                    *pErrorStatus = mapStructEntryToValue(
                        &raw mut mib2UdpEntryMap as *mut structToAsnValue,
                        DEFINE_SIZEOF!(mib2UdpEntryMap) as UINT,
                        (&raw mut (*udpTable).table as *mut MIB_UDPROW)
                            .offset(tableIndex.wrapping_sub(1) as isize)
                            as *mut MIB_UDPROW as *mut core::ffi::c_void,
                        item,
                        pVarBind,
                    );
                    if *pErrorStatus == 0 && bPduType == (SNMP_PDU_GETNEXT as u8) {
                        let mut oid: AsnObjectIdentifier = AsnObjectIdentifier {
                            idLength: 0,
                            ids: core::ptr::null_mut(),
                        };

                        ret = setOidWithItemAndIpAddr(
                            &raw mut (*pVarBind).name,
                            &raw mut myOid,
                            item,
                            (*(&raw mut (*udpTable).table as *mut MIB_UDPROW)
                                .offset(tableIndex.wrapping_sub(1) as isize))
                            .dwLocalAddr,
                        );
                        if ret != 0 {
                            oid.idLength = 1;
                            oid.ids = &raw mut (*(&raw mut (*udpTable).table as *mut MIB_UDPROW)
                                .offset(tableIndex.wrapping_sub(1) as isize))
                            .dwLocalPort as *mut UINT;
                            ret = SnmpUtilOidAppend(&raw mut (*pVarBind).name, &raw mut oid);
                        }
                    }
                }
            }
        }
        SNMP_PDU_SET => {
            *pErrorStatus = (SNMP_ERRORSTATUS_READONLY as i32);
            ret = (FALSE as i32);
        }
        _ => {
            FIXME!("0x%02x: unsupported PDU type\n", bPduType as core::ffi::c_uint);
            *pErrorStatus = (SNMP_ERRORSTATUS_NOSUCHNAME as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpExtensionInit(
    dwUptimeReference: DWORD,
    phSubagentTrapEvent: *mut HANDLE,
    pFirstSupportedRegion: *mut AsnObjectIdentifier,
) -> BOOL {
    let mut myOid: AsnObjectIdentifier = DEFINE_OID!(mib2System);
    let mut i: UINT = 0;

    minSupportedIDLength = UINT_MAX;
    i = 0;
    while (i as usize) < ARRAY_SIZE!(supportedIDs) as usize {
        if let Some(init) = supportedIDs[i as usize].init {
            init();
        }
        if supportedIDs[i as usize].name.idLength < minSupportedIDLength {
            minSupportedIDLength = supportedIDs[i as usize].name.idLength;
        }
        i = i.wrapping_add(1);
    }
    *phSubagentTrapEvent = core::ptr::null_mut();
    SnmpUtilOidCpy(pFirstSupportedRegion, core::ptr::addr_of_mut!(myOid));
    return (TRUE) as BOOL;
}

unsafe extern "C" fn cleanup() {
    let mut i: UINT = 0;
    while (i as usize) < (ARRAY_SIZE!(supportedIDs) as usize) {
        if supportedIDs[i as usize].cleanup.is_some() {
            supportedIDs[i as usize]
                .cleanup
                .unwrap_unchecked()();
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn findSupportedQuery(
    ids: *mut UINT,
    idLength: UINT,
    matchingIndex: *mut UINT,
) -> *mut mibImplementation {
    let mut indexHigh: INT = DEFINE_SIZEOF!(supportedIDs) as INT - 1;
    let mut indexLow: INT = 0;
    let mut oid1: AsnObjectIdentifier = AsnObjectIdentifier {
        idLength: idLength,
        ids: ids,
    };

    if idLength == 0 {
        return core::ptr::null_mut();
    }

    while indexLow <= indexHigh {
        let mut cmp: INT;
        let mut i: INT = (indexLow + indexHigh) / 2;
        cmp = SnmpUtilOidNCmp(
            core::ptr::addr_of_mut!(oid1),
            core::ptr::addr_of_mut!((*(core::ptr::addr_of_mut!(supportedIDs) as *mut mibImplementation).offset(i as isize)).name),
            idLength,
        );
        if cmp == 0 {
            *matchingIndex = i as UINT;
            return (core::ptr::addr_of_mut!(supportedIDs) as *mut mibImplementation).offset(i as isize);
        }
        if cmp > 0 {
            indexLow = i + 1;
        } else {
            indexHigh = i - 1;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SnmpExtensionQuery(
    mut bPduType: BYTE,
    mut pVarBindList: *mut SnmpVarBindList,
    mut pErrorStatus: *mut AsnInteger32,
    mut pErrorIndex: *mut AsnInteger32,
) -> BOOL {
    let mut mib2oid: AsnObjectIdentifier = DEFINE_OID!(mib2);
    let mut error: AsnInteger32 = (SNMP_ERRORSTATUS_NOERROR as i32);
    let mut errorIndex: AsnInteger32 = 0;
    let mut i: UINT = 0;
    let mut ret: BOOL = (TRUE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/inetmib1/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SnmpExtensionQuery\0" as *const u8 as *const ::core::ffi::c_char,
            1381 as ::core::ffi::c_int,
            b"(0x%02x, %p, %p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            bPduType as ::core::ffi::c_int,
            pVarBindList,
            pErrorStatus,
            pErrorIndex,
        );
    }
    i = 0;
    while error == 0 && i < (*pVarBindList).len {
        if SnmpUtilOidNCmp(
            &raw mut (*(*pVarBindList).list.offset(i as isize)).name,
            &raw mut mib2oid,
            mib2oid.idLength,
        ) == 0
        {
            let mut impl_0: *mut mibImplementation = ::core::ptr::null_mut::<mibImplementation>();
            let mut len: UINT = 0;
            let mut matchingIndex: UINT = 0;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/inetmib1/main.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"SnmpExtensionQuery\0" as *const u8 as *const ::core::ffi::c_char,
                    1392 as ::core::ffi::c_int,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    SnmpUtilOidToA(
                        &raw mut (*(*pVarBindList).list.offset(i as isize)).name,
                    ),
                );
            }
            len = (*(*pVarBindList).list.offset(i as isize)).name.idLength;
            while len >= minSupportedIDLength && impl_0.is_null() {
                impl_0 = findSupportedQuery(
                    (*(*pVarBindList).list.offset(i as isize)).name.ids,
                    len,
                    &raw mut matchingIndex,
                );
                len = len.wrapping_sub(1);
            }
            if !impl_0.is_null() && (*impl_0).query.is_some() {
                ret = (*impl_0).query.unwrap_unchecked()(
                    bPduType,
                    (*pVarBindList).list.offset(i as isize) as *mut SnmpVarBind,
                    &raw mut error,
                );
            } else {
                error = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
            }
            if error == (SNMP_ERRORSTATUS_NOSUCHNAME as i32)
                && bPduType as ::core::ffi::c_int == (SNMP_PDU_GETNEXT as i32)
            {
                matchingIndex = matchingIndex.wrapping_add(1);
                while error == (SNMP_ERRORSTATUS_NOSUCHNAME as i32)
                    && (matchingIndex as usize) < DEFINE_SIZEOF!(supportedIDs)
                {
                    error = SNMP_ERRORSTATUS_NOERROR as AsnInteger32;
                    impl_0 = (&raw mut supportedIDs as *mut mibImplementation)
                        .offset(matchingIndex as isize)
                        as *mut mibImplementation;
                    if (*impl_0).query.is_some() {
                        ret = (*impl_0).query.unwrap_unchecked()(
                            bPduType,
                            (*pVarBindList).list.offset(i as isize) as *mut SnmpVarBind,
                            &raw mut error,
                        );
                    } else {
                        error = SNMP_ERRORSTATUS_NOSUCHNAME as AsnInteger32;
                    }
                    matchingIndex = matchingIndex.wrapping_add(1);
                }
                if error == (SNMP_ERRORSTATUS_NOSUCHNAME as i32) {
                    SnmpUtilOidFree(&raw mut (*(*pVarBindList).list.offset(i as isize)).name);
                    ret = SnmpUtilOidCpy(
                        &raw mut (*(*pVarBindList).list.offset(i as isize)).name,
                        &raw mut (*(&raw mut supportedIDs as *mut mibImplementation)
                            .offset(matchingIndex.wrapping_sub(1 as UINT) as isize))
                        .name,
                    ) as BOOL;
                    let ref mut fresh15 =
                        *(*(*pVarBindList).list.offset(i as isize)).name.ids.offset(
                            (*(*pVarBindList).list.offset(i as isize))
                                .name
                                .idLength
                                .wrapping_sub(1 as UINT) as isize,
                        );
                    *fresh15 = (*fresh15).wrapping_add(1 as UINT);
                }
            }
            if error != 0 {
                errorIndex = i.wrapping_add(1 as UINT) as AsnInteger32;
            }
        }
        i = i.wrapping_add(1);
    }
    *pErrorStatus = error;
    *pErrorIndex = errorIndex;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    mut hinstDLL: HINSTANCE,
    mut fdwReason: DWORD,
    mut lpvReserved: LPVOID,
) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/inetmib1/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DllMain\0" as *const u8 as *const ::core::ffi::c_char,
            1448 as ::core::ffi::c_int,
            b"(0x%p, %d, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinstDLL,
            fdwReason,
            lpvReserved,
        );
    }
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        DLL_PROCESS_DETACH => {
            if lpvReserved.is_null() {
                cleanup();
            }
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}
