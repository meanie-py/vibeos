//! iphlpapi.dll Vista+ `*2` network tables (netioapi.h): `GetIpForwardTable2`, `GetIpNetTable2`,
//! `GetIpNetEntry2`, `ResolveIpNetEntry2`, `GetUnicastIpAddressTable`, `GetIpInterfaceEntry`,
//! `GetBestRoute2`, the `Notify*Change` registrations and `SetCurrentThreadCompartmentId`.
//!
//! The legacy TDI stack underneath this port is IPv4-only, so every table is SYNTHESIZED from the
//! legacy paths (`GetIpForwardTable` / `GetIpNetTable` / `GetIpAddrTable` / `GetIfEntry`) and
//! only AF_INET rows ever exist. For AF_INET6 the tables are honestly empty and the single-row
//! lookups say ERROR_NOT_FOUND; AF_UNSPEC therefore equals AF_INET.
//!
//! All tables are `HeapAlloc`ed on the process heap and released with `FreeMibTable`.
//!
//! Layouts were measured with the llvm-mingw clang (offsetof/sizeof) on x86_64 and i686; none of
//! these rows contains a pointer, so the numbers are identical on both (and on arm64). The
//! structs are force-aligned to 8 because they hold a NET_LUID (u64), and Rust gives u64 only
//! 4-byte alignment on some 32-bit targets.

use crate::iphlpapi::luid::{lookup_row, luid_index, make_luid};
use crate::prelude::*;
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Win32 error codes (winerror.h).
const E_NOT_SUPPORTED: u32 = 50;
const E_INVALID_PARAMETER: u32 = 87;
const E_INSUFFICIENT_BUFFER: u32 = 122;
const E_NO_DATA: u32 = 232;
const E_NOT_FOUND: u32 = 1168;
const E_NOT_ENOUGH_MEMORY: u32 = 8;

const FAM_UNSPEC: u16 = 0;
const FAM_INET: u16 = 2;
const FAM_INET6: u16 = 23;

const HEAP_ZERO_MEMORY: u32 = 0x8;
const INFINITE_LIFETIME: u32 = 0xFFFF_FFFF;

// IfType values (ipifcons.h).
const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;

// NL_NEIGHBOR_STATE (nldef.h).
const NLNS_UNREACHABLE: u32 = 0;
const NLNS_INCOMPLETE: u32 = 1;
const NLNS_STALE: u32 = 4;
const NLNS_REACHABLE: u32 = 5;
const NLNS_PERMANENT: u32 = 6;

// MIB_IPNET_TYPE (legacy ipmib.h).
const MIB_IPNET_TYPE_INVALID: u32 = 2;
const MIB_IPNET_TYPE_DYNAMIC: u32 = 3;
const MIB_IPNET_TYPE_STATIC: u32 = 4;

// NL_PREFIX_ORIGIN / NL_SUFFIX_ORIGIN / NL_DAD_STATE (nldef.h).
const NLPO_OTHER: u32 = 0;
const NLPO_WELL_KNOWN: u32 = 2;
const NLSO_OTHER: u32 = 0;
const NLSO_WELL_KNOWN: u32 = 2;
const NLDS_PREFERRED: u32 = 4;

// NL_LINK_LOCAL_ADDRESS_BEHAVIOR / NL_ROUTER_DISCOVERY_BEHAVIOR.
const LINK_LOCAL_DELAYED: u32 = 2;
const ROUTER_DISCOVERY_ENABLED: u32 = 1;

// MIB_NOTIFICATION_TYPE (netioapi.h).
const MIB_INITIAL_NOTIFICATION: u32 = 3;

/// SOCKADDR_INET (ws2ipdef.h) — union of sockaddr_in (16) / sockaddr_in6 (28), align 4.
/// `w[0]` is `sin_addr` for IPv4 (network order) and `sin6_flowinfo` for IPv6.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SockAddrInet {
    pub si_family: u16,
    pub port: u16,
    pub w: [u32; 6],
}

impl SockAddrInet {
    const ZERO: SockAddrInet = SockAddrInet { si_family: 0, port: 0, w: [0; 6] };
    fn v4(addr: u32) -> SockAddrInet {
        SockAddrInet { si_family: FAM_INET, port: 0, w: [addr, 0, 0, 0, 0, 0] }
    }
}

/// IP_ADDRESS_PREFIX (netioapi.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpAddressPrefix {
    pub Prefix: SockAddrInet,
    pub PrefixLength: u8,
}

/// C: `MIB_IPFORWARD_ROW2` (netioapi.h).
#[repr(C, align(8))]
pub struct MIB_IPFORWARD_ROW2 {
    pub InterfaceLuid: u64,
    pub InterfaceIndex: u32,
    pub DestinationPrefix: IpAddressPrefix,
    pub NextHop: SockAddrInet,
    pub SitePrefixLength: u8,
    pub ValidLifetime: u32,
    pub PreferredLifetime: u32,
    pub Metric: u32,
    pub Protocol: u32,
    pub Loopback: u8,
    pub AutoconfigureAddress: u8,
    pub Publish: u8,
    pub Immortal: u8,
    pub Age: u32,
    pub Origin: u32,
}

/// C: `MIB_IPFORWARD_TABLE2` — `ULONG NumEntries; MIB_IPFORWARD_ROW2 Table[1];`.
#[repr(C, align(8))]
pub struct MIB_IPFORWARD_TABLE2 {
    pub NumEntries: u32,
    pub Table: [MIB_IPFORWARD_ROW2; 1],
}

/// C: `MIB_IPNET_ROW2` (netioapi.h).
#[repr(C, align(8))]
pub struct MIB_IPNET_ROW2 {
    pub Address: SockAddrInet,
    pub InterfaceIndex: u32,
    pub InterfaceLuid: u64,
    pub PhysicalAddress: [u8; 32],
    pub PhysicalAddressLength: u32,
    pub State: u32,
    /// Union of the BOOLEAN bitfields IsRouter / IsUnreachable.
    pub Flags: u8,
    /// Union of LastReachable / LastUnreachable.
    pub ReachabilityTime: u32,
}

/// C: `MIB_IPNET_TABLE2`.
#[repr(C, align(8))]
pub struct MIB_IPNET_TABLE2 {
    pub NumEntries: u32,
    pub Table: [MIB_IPNET_ROW2; 1],
}

/// C: `MIB_UNICASTIPADDRESS_ROW` (netioapi.h).
#[repr(C, align(8))]
pub struct MIB_UNICASTIPADDRESS_ROW {
    pub Address: SockAddrInet,
    pub InterfaceLuid: u64,
    pub InterfaceIndex: u32,
    pub PrefixOrigin: u32,
    pub SuffixOrigin: u32,
    pub ValidLifetime: u32,
    pub PreferredLifetime: u32,
    pub OnLinkPrefixLength: u8,
    pub SkipAsSource: u8,
    pub DadState: u32,
    pub ScopeId: u32,
    pub CreationTimeStamp: i64,
}

/// C: `MIB_UNICASTIPADDRESS_TABLE`.
#[repr(C, align(8))]
pub struct MIB_UNICASTIPADDRESS_TABLE {
    pub NumEntries: u32,
    pub Table: [MIB_UNICASTIPADDRESS_ROW; 1],
}

/// C: `MIB_IPINTERFACE_ROW` (netioapi.h).
#[repr(C, align(8))]
pub struct MIB_IPINTERFACE_ROW {
    pub Family: u16,
    pub InterfaceLuid: u64,
    pub InterfaceIndex: u32,
    pub MaxReassemblySize: u32,
    pub InterfaceIdentifier: u64,
    pub MinRouterAdvertisementInterval: u32,
    pub MaxRouterAdvertisementInterval: u32,
    pub AdvertisingEnabled: u8,
    pub ForwardingEnabled: u8,
    pub WeakHostSend: u8,
    pub WeakHostReceive: u8,
    pub UseAutomaticMetric: u8,
    pub UseNeighborUnreachabilityDetection: u8,
    pub ManagedAddressConfigurationSupported: u8,
    pub OtherStatefulConfigurationSupported: u8,
    pub AdvertiseDefaultRoute: u8,
    pub RouterDiscoveryBehavior: u32,
    pub DadTransmits: u32,
    pub BaseReachableTime: u32,
    pub RetransmitTime: u32,
    pub PathMtuDiscoveryTimeout: u32,
    pub LinkLocalAddressBehavior: u32,
    pub LinkLocalAddressTimeout: u32,
    pub ZoneIndices: [u32; 16],
    pub SitePrefixLength: u32,
    pub Metric: u32,
    pub NlMtu: u32,
    pub Connected: u8,
    pub SupportsWakeUpPatterns: u8,
    pub SupportsNeighborDiscovery: u8,
    pub SupportsRouterDiscovery: u8,
    pub ReachableTime: u32,
    pub TransmitOffload: u8,
    pub ReceiveOffload: u8,
    pub DisableDefaultRoutes: u8,
}

// Sizes / offsets measured from the mingw-w64 SDK headers (llvm-mingw clang) on x86_64 AND
// i686 — identical on both; no pointers in any of these rows, so arm64 matches.
const _: () = assert!(core::mem::size_of::<SockAddrInet>() == 28);
const _: () = assert!(core::mem::size_of::<IpAddressPrefix>() == 32);

const _: () = assert!(core::mem::size_of::<MIB_IPFORWARD_ROW2>() == 104);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, InterfaceIndex) == 8);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, DestinationPrefix) == 12);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, NextHop) == 44);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, SitePrefixLength) == 72);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, ValidLifetime) == 76);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, PreferredLifetime) == 80);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, Metric) == 84);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, Protocol) == 88);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, Loopback) == 92);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, Age) == 96);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_ROW2, Origin) == 100);
const _: () = assert!(core::mem::size_of::<MIB_IPFORWARD_TABLE2>() == 112);
const _: () = assert!(core::mem::offset_of!(MIB_IPFORWARD_TABLE2, Table) == 8);

const _: () = assert!(core::mem::size_of::<MIB_IPNET_ROW2>() == 88);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, InterfaceIndex) == 28);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, InterfaceLuid) == 32);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, PhysicalAddress) == 40);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, PhysicalAddressLength) == 72);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, State) == 76);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, Flags) == 80);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_ROW2, ReachabilityTime) == 84);
const _: () = assert!(core::mem::size_of::<MIB_IPNET_TABLE2>() == 96);
const _: () = assert!(core::mem::offset_of!(MIB_IPNET_TABLE2, Table) == 8);

const _: () = assert!(core::mem::size_of::<MIB_UNICASTIPADDRESS_ROW>() == 80);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, InterfaceLuid) == 32);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, InterfaceIndex) == 40);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, PrefixOrigin) == 44);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, ValidLifetime) == 52);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, OnLinkPrefixLength) == 60);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, SkipAsSource) == 61);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, DadState) == 64);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, ScopeId) == 68);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_ROW, CreationTimeStamp) == 72);
const _: () = assert!(core::mem::size_of::<MIB_UNICASTIPADDRESS_TABLE>() == 88);
const _: () = assert!(core::mem::offset_of!(MIB_UNICASTIPADDRESS_TABLE, Table) == 8);

const _: () = assert!(core::mem::size_of::<MIB_IPINTERFACE_ROW>() == 168);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, InterfaceLuid) == 8);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, InterfaceIndex) == 16);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, MaxReassemblySize) == 20);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, InterfaceIdentifier) == 24);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, MinRouterAdvertisementInterval) == 32);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, AdvertisingEnabled) == 40);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, RouterDiscoveryBehavior) == 52);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, DadTransmits) == 56);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, LinkLocalAddressBehavior) == 72);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, ZoneIndices) == 80);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, SitePrefixLength) == 144);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, Metric) == 148);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, NlMtu) == 152);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, Connected) == 156);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, ReachableTime) == 160);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, TransmitOffload) == 164);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, ReceiveOffload) == 165);
const _: () = assert!(core::mem::offset_of!(MIB_IPINTERFACE_ROW, DisableDefaultRoutes) == 166);

extern "system" {
    fn QueueUserWorkItem(
        function: unsafe extern "system" fn(*mut c_void) -> u32,
        context: *mut c_void,
        flags: u32,
    ) -> BOOL;
}

// ---------------------------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------------------------

#[inline]
fn family_ok(family: u16) -> bool {
    family == FAM_UNSPEC || family == FAM_INET || family == FAM_INET6
}

/// Allocates a zeroed `{ ULONG NumEntries; ROW Table[count]; }` block (rows start at offset 8)
/// on the process heap — what FreeMibTable frees.
unsafe fn alloc_table(count: usize, row_size: usize) -> *mut u8 {
    let bytes = 8usize + count.saturating_mul(row_size);
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, bytes as SIZE_T) as *mut u8
}

/// Runs a legacy "size probe then fetch" IP Helper call and returns the heap block.
/// `call(buf, &mut size)` is e.g. `GetIpForwardTable(buf, size, FALSE)`.
unsafe fn fetch_legacy<T, F: FnMut(*mut T, *mut u32) -> u32>(mut call: F) -> Result<*mut T, u32> {
    let heap = GetProcessHeap();
    let mut size: u32 = 0;
    let r = call(core::ptr::null_mut(), &mut size);
    if r != E_INSUFFICIENT_BUFFER {
        return Err(r);
    }
    for _ in 0..4 {
        let buf = HeapAlloc(heap, HEAP_ZERO_MEMORY, (size as usize).max(16) as SIZE_T) as *mut T;
        if buf.is_null() {
            return Err(E_NOT_ENOUGH_MEMORY);
        }
        let r = call(buf, &mut size);
        if r == NO_ERROR {
            return Ok(buf);
        }
        HeapFree(heap, 0, buf as *mut c_void);
        if r != E_INSUFFICIENT_BUFFER {
            return Err(r);
        }
    }
    Err(E_INSUFFICIENT_BUFFER)
}

/// Legacy "no rows" outcomes are an empty table, not a failure.
fn empty_ok(e: u32) -> bool {
    e == E_NO_DATA || e == E_NOT_FOUND
}

unsafe fn luid_for_index(index: u32) -> u64 {
    match lookup_row(index) {
        Some(r) => make_luid(r.dwType, index),
        None => make_luid(0, index),
    }
}

// ---------------------------------------------------------------------------------------------
// Forward (route) table.
// ---------------------------------------------------------------------------------------------

unsafe fn fill_forward_row2(src: &MIB_IPFORWARDROW, dst: *mut MIB_IPFORWARD_ROW2) {
    let mut row: MIB_IPFORWARD_ROW2 = core::mem::zeroed();
    row.InterfaceIndex = src.dwForwardIfIndex;
    row.InterfaceLuid = luid_for_index(src.dwForwardIfIndex);
    row.DestinationPrefix.Prefix = SockAddrInet::v4(src.dwForwardDest);
    row.DestinationPrefix.PrefixLength = src.dwForwardMask.count_ones() as u8;
    row.NextHop = SockAddrInet::v4(src.dwForwardNextHop);
    row.ValidLifetime = INFINITE_LIFETIME;
    row.PreferredLifetime = INFINITE_LIFETIME;
    row.Metric = src.dwForwardMetric1;
    // The legacy MIB_IPFORWARD_PROTO values are the NL_ROUTE_PROTOCOL values.
    row.Protocol = src.__bindgen_anon_2.bindgen_union_field;
    row.Loopback = (luid_type_of(row.InterfaceLuid) == IF_TYPE_SOFTWARE_LOOPBACK) as u8;
    row.Age = src.dwForwardAge;
    row.Origin = 0; // NlroManual — the legacy table does not record who created a route.
    core::ptr::write(dst, row);
}

/// IPv4 netmask for a prefix length, in network byte order as stored in the legacy rows.
#[inline]
fn prefix_mask(plen: u32) -> u32 {
    match plen {
        0 => 0,
        1..=31 => (u32::MAX << (32 - plen)).to_be(),
        _ => u32::MAX,
    }
}

#[inline]
fn luid_type_of(luid: u64) -> u32 {
    ((luid >> 48) & 0xFFFF) as u32
}

/// Builds the IPv4 MIB_IPFORWARD_TABLE2 (empty for IPv6). Caller frees with FreeMibTable.
unsafe fn build_forward_table2(family: u16) -> Result<*mut MIB_IPFORWARD_TABLE2, u32> {
    let legacy = if family == FAM_INET6 {
        None
    } else {
        match fetch_legacy::<MIB_IPFORWARDTABLE, _>(|p, s| {
            crate::iphlpapi::iphlpapi_main::GetIpForwardTable(p, s, FALSE)
        }) {
            Ok(t) => Some(t),
            Err(e) if empty_ok(e) => None,
            Err(e) => return Err(e),
        }
    };
    let count = match legacy {
        Some(t) => (*t).dwNumEntries as usize,
        None => 0,
    };
    let out = alloc_table(count, core::mem::size_of::<MIB_IPFORWARD_ROW2>()) as *mut MIB_IPFORWARD_TABLE2;
    if out.is_null() {
        if let Some(t) = legacy {
            HeapFree(GetProcessHeap(), 0, t as *mut c_void);
        }
        return Err(E_NOT_ENOUGH_MEMORY);
    }
    (*out).NumEntries = count as u32;
    if let Some(t) = legacy {
        let src = core::ptr::addr_of!((*t).table) as *const MIB_IPFORWARDROW;
        let dst = core::ptr::addr_of_mut!((*out).Table) as *mut MIB_IPFORWARD_ROW2;
        for i in 0..count {
            fill_forward_row2(&*src.add(i), dst.add(i));
        }
        HeapFree(GetProcessHeap(), 0, t as *mut c_void);
    }
    Ok(out)
}

/// C: `NETIO_STATUS GetIpForwardTable2(ADDRESS_FAMILY Family, PMIB_IPFORWARD_TABLE2 *Table)`.
/// IPv4 rows come from the legacy route table; IPv6 yields an empty table (no v6 stack).
#[no_mangle]
pub unsafe extern "system" fn GetIpForwardTable2(
    Family: u16, // ADDRESS_FAMILY: a USHORT, whose upper register bits x64 callers leave undefined
    Table: *mut *mut MIB_IPFORWARD_TABLE2,
) -> u32 {
    if Table.is_null() || !family_ok(Family) {
        return E_INVALID_PARAMETER;
    }
    *Table = core::ptr::null_mut();
    match build_forward_table2(Family) {
        Ok(t) => {
            *Table = t;
            NO_ERROR
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------------------------
// Neighbour (ARP) table.
// ---------------------------------------------------------------------------------------------

/// Legacy MIB_IPNET_TYPE -> NL_NEIGHBOR_STATE. Dynamic entries are reachable, static ones
/// permanent, "invalid" ones unreachable; "other" has no counterpart so it is reported Stale
/// (known to exist, not confirmed reachable).
fn state_from_legacy(t: u32) -> u32 {
    match t {
        MIB_IPNET_TYPE_DYNAMIC => NLNS_REACHABLE,
        MIB_IPNET_TYPE_STATIC => NLNS_PERMANENT,
        MIB_IPNET_TYPE_INVALID => NLNS_UNREACHABLE,
        _ => NLNS_STALE,
    }
}

unsafe fn fill_net_row2(src: &MIB_IPNETROW, dst: *mut MIB_IPNET_ROW2) {
    let mut row: MIB_IPNET_ROW2 = core::mem::zeroed();
    row.Address = SockAddrInet::v4(src.dwAddr);
    row.InterfaceIndex = src.dwIndex;
    row.InterfaceLuid = luid_for_index(src.dwIndex);
    let n = (src.dwPhysAddrLen as usize).min(src.bPhysAddr.len());
    core::ptr::copy_nonoverlapping(src.bPhysAddr.as_ptr(), row.PhysicalAddress.as_mut_ptr(), n);
    row.PhysicalAddressLength = n as u32;
    row.State = state_from_legacy(src.__bindgen_anon_1.bindgen_union_field);
    // IsRouter / IsUnreachable and the reachability timestamps are not tracked by the legacy ARP
    // table; they stay zero.
    core::ptr::write(dst, row);
}

unsafe fn build_net_table2(family: u16) -> Result<*mut MIB_IPNET_TABLE2, u32> {
    let legacy = if family == FAM_INET6 {
        None
    } else {
        match fetch_legacy::<MIB_IPNETTABLE, _>(|p, s| {
            crate::iphlpapi::iphlpapi_main::GetIpNetTable(p, s, FALSE)
        }) {
            Ok(t) => Some(t),
            Err(e) if empty_ok(e) => None,
            Err(e) => return Err(e),
        }
    };
    let count = match legacy {
        Some(t) => (*t).dwNumEntries as usize,
        None => 0,
    };
    let out = alloc_table(count, core::mem::size_of::<MIB_IPNET_ROW2>()) as *mut MIB_IPNET_TABLE2;
    if out.is_null() {
        if let Some(t) = legacy {
            HeapFree(GetProcessHeap(), 0, t as *mut c_void);
        }
        return Err(E_NOT_ENOUGH_MEMORY);
    }
    (*out).NumEntries = count as u32;
    if let Some(t) = legacy {
        let src = core::ptr::addr_of!((*t).table) as *const MIB_IPNETROW;
        let dst = core::ptr::addr_of_mut!((*out).Table) as *mut MIB_IPNET_ROW2;
        for i in 0..count {
            fill_net_row2(&*src.add(i), dst.add(i));
        }
        HeapFree(GetProcessHeap(), 0, t as *mut c_void);
    }
    Ok(out)
}

/// C: `NETIO_STATUS GetIpNetTable2(ADDRESS_FAMILY Family, PMIB_IPNET_TABLE2 *Table)`.
#[no_mangle]
pub unsafe extern "system" fn GetIpNetTable2(
    Family: u16, // ADDRESS_FAMILY: a USHORT, whose upper register bits x64 callers leave undefined
    Table: *mut *mut MIB_IPNET_TABLE2,
) -> u32 {
    if Table.is_null() || !family_ok(Family) {
        return E_INVALID_PARAMETER;
    }
    *Table = core::ptr::null_mut();
    match build_net_table2(Family) {
        Ok(t) => {
            *Table = t;
            NO_ERROR
        }
        Err(e) => e,
    }
}

/// Validates the key of a MIB_IPNET_ROW2 lookup. Returns (is_v6, interface index) or an error.
/// Windows requires a valid Address family and either InterfaceLuid or InterfaceIndex.
unsafe fn net_row_key(row: *const MIB_IPNET_ROW2) -> Result<(bool, u32), u32> {
    let fam = (*row).Address.si_family;
    if fam != FAM_INET && fam != FAM_INET6 {
        return Err(E_INVALID_PARAMETER);
    }
    let luid = (*row).InterfaceLuid;
    let index = (*row).InterfaceIndex;
    if luid == 0 && index == 0 {
        return Err(E_INVALID_PARAMETER);
    }
    let idx = if luid != 0 { luid_index(luid) } else { index };
    Ok((fam == FAM_INET6, idx))
}

/// Looks up the neighbour-cache entry for (Address, idx). Returns a copy.
unsafe fn find_net_entry(addr: u32, idx: u32) -> Result<Option<MIB_IPNET_ROW2>, u32> {
    let t = build_net_table2(FAM_INET)?;
    let rows = core::ptr::addr_of!((*t).Table) as *const MIB_IPNET_ROW2;
    let mut found = None;
    for i in 0..(*t).NumEntries as usize {
        let r = &*rows.add(i);
        if r.InterfaceIndex == idx && r.Address.w[0] == addr {
            let mut copy: MIB_IPNET_ROW2 = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(r as *const MIB_IPNET_ROW2, &mut copy, 1);
            found = Some(copy);
            break;
        }
    }
    HeapFree(GetProcessHeap(), 0, t as *mut c_void);
    Ok(found)
}

/// C: `NETIO_STATUS GetIpNetEntry2(PMIB_IPNET_ROW2 Row)`. Looks the (Address, interface) key up
/// in the ARP cache and rewrites the row; ERROR_NOT_FOUND when absent (always so for IPv6).
#[no_mangle]
pub unsafe extern "system" fn GetIpNetEntry2(Row: *mut MIB_IPNET_ROW2) -> u32 {
    if Row.is_null() {
        return E_INVALID_PARAMETER;
    }
    let (v6, idx) = match net_row_key(Row) {
        Ok(k) => k,
        Err(e) => return e,
    };
    if v6 {
        return E_NOT_FOUND;
    }
    match find_net_entry((*Row).Address.w[0], idx) {
        Ok(Some(found)) => {
            core::ptr::write(Row, found);
            NO_ERROR
        }
        Ok(None) => E_NOT_FOUND,
        Err(e) => e,
    }
}

/// C: `NETIO_STATUS ResolveIpNetEntry2(PMIB_IPNET_ROW2 Row, const SOCKADDR_INET *SourceAddress)`.
/// A usable cache hit is returned as is; otherwise an ARP request is sent (via SendARP) and the
/// resolved MAC is stored with State = Reachable. IPv6 (neighbour discovery) is not supported.
#[no_mangle]
pub unsafe extern "system" fn ResolveIpNetEntry2(
    Row: *mut MIB_IPNET_ROW2,
    SourceAddress: *const SockAddrInet,
) -> u32 {
    if Row.is_null() {
        return E_INVALID_PARAMETER;
    }
    let (v6, idx) = match net_row_key(Row) {
        Ok(k) => k,
        Err(e) => return e,
    };
    if v6 {
        return E_NOT_SUPPORTED;
    }
    if lookup_row(idx).is_none() {
        return E_NOT_FOUND;
    }
    let dest = (*Row).Address.w[0];
    if let Ok(Some(found)) = find_net_entry(dest, idx) {
        if found.State != NLNS_UNREACHABLE && found.State != NLNS_INCOMPLETE {
            core::ptr::write(Row, found);
            return NO_ERROR;
        }
    }
    let src = if !SourceAddress.is_null() && (*SourceAddress).si_family == FAM_INET {
        (*SourceAddress).w[0]
    } else {
        0
    };
    let mut mac = [0u32; 2];
    let mut len: u32 = 8;
    let r = crate::iphlpapi::iphlpapi_main::SendARP(dest, src, mac.as_mut_ptr(), &mut len);
    if r != NO_ERROR {
        return r;
    }
    let mut out: MIB_IPNET_ROW2 = core::mem::zeroed();
    out.Address = SockAddrInet::v4(dest);
    out.InterfaceIndex = idx;
    out.InterfaceLuid = luid_for_index(idx);
    let n = (len as usize).min(8);
    core::ptr::copy_nonoverlapping(mac.as_ptr() as *const u8, out.PhysicalAddress.as_mut_ptr(), n);
    out.PhysicalAddressLength = n as u32;
    out.State = NLNS_REACHABLE;
    core::ptr::write(Row, out);
    NO_ERROR
}

// ---------------------------------------------------------------------------------------------
// Unicast address table.
// ---------------------------------------------------------------------------------------------

unsafe fn fill_unicast_row(src: &MIB_IPADDRROW, dst: *mut MIB_UNICASTIPADDRESS_ROW) {
    let mut row: MIB_UNICASTIPADDRESS_ROW = core::mem::zeroed();
    row.Address = SockAddrInet::v4(src.dwAddr);
    row.InterfaceIndex = src.dwIndex;
    row.InterfaceLuid = luid_for_index(src.dwIndex);
    // The legacy address table does not say how an address was configured (DHCP / manual /
    // link-local), so origins are "Other" — except loopback, which is well known.
    let loopback = src.dwAddr.to_ne_bytes()[0] == 127; // stored in network order
    row.PrefixOrigin = if loopback { NLPO_WELL_KNOWN } else { NLPO_OTHER };
    row.SuffixOrigin = if loopback { NLSO_WELL_KNOWN } else { NLSO_OTHER };
    row.ValidLifetime = INFINITE_LIFETIME;
    row.PreferredLifetime = INFINITE_LIFETIME;
    row.OnLinkPrefixLength = src.dwMask.count_ones() as u8;
    row.DadState = NLDS_PREFERRED;
    core::ptr::write(dst, row);
}

unsafe fn build_unicast_table(family: u16) -> Result<*mut MIB_UNICASTIPADDRESS_TABLE, u32> {
    let legacy = if family == FAM_INET6 {
        None
    } else {
        match fetch_legacy::<MIB_IPADDRTABLE, _>(|p, s| {
            crate::iphlpapi::iphlpapi_main::GetIpAddrTable(p, s, FALSE)
        }) {
            Ok(t) => Some(t),
            Err(e) if empty_ok(e) => None,
            Err(e) => return Err(e),
        }
    };
    // Unconfigured (0.0.0.0) interface entries are not addresses.
    let (src, n) = match legacy {
        Some(t) => (core::ptr::addr_of!((*t).table) as *const MIB_IPADDRROW, (*t).dwNumEntries as usize),
        None => (core::ptr::null(), 0),
    };
    let mut count = 0usize;
    for i in 0..n {
        if (*src.add(i)).dwAddr != 0 {
            count += 1;
        }
    }
    let out = alloc_table(count, core::mem::size_of::<MIB_UNICASTIPADDRESS_ROW>())
        as *mut MIB_UNICASTIPADDRESS_TABLE;
    if out.is_null() {
        if let Some(t) = legacy {
            HeapFree(GetProcessHeap(), 0, t as *mut c_void);
        }
        return Err(E_NOT_ENOUGH_MEMORY);
    }
    (*out).NumEntries = count as u32;
    let dst = core::ptr::addr_of_mut!((*out).Table) as *mut MIB_UNICASTIPADDRESS_ROW;
    let mut o = 0usize;
    for i in 0..n {
        let r = &*src.add(i);
        if r.dwAddr != 0 {
            fill_unicast_row(r, dst.add(o));
            o += 1;
        }
    }
    if let Some(t) = legacy {
        HeapFree(GetProcessHeap(), 0, t as *mut c_void);
    }
    Ok(out)
}

/// C: `NETIO_STATUS GetUnicastIpAddressTable(ADDRESS_FAMILY Family,
/// PMIB_UNICASTIPADDRESS_TABLE *Table)`. IPv4 addresses from GetIpAddrTable; IPv6 is empty.
#[no_mangle]
pub unsafe extern "system" fn GetUnicastIpAddressTable(
    Family: u16, // ADDRESS_FAMILY: a USHORT, whose upper register bits x64 callers leave undefined
    Table: *mut *mut MIB_UNICASTIPADDRESS_TABLE,
) -> u32 {
    if Table.is_null() || !family_ok(Family) {
        return E_INVALID_PARAMETER;
    }
    *Table = core::ptr::null_mut();
    match build_unicast_table(Family) {
        Ok(t) => {
            *Table = t;
            NO_ERROR
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------------------------
// IP interface entry.
// ---------------------------------------------------------------------------------------------

/// C: `NETIO_STATUS GetIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row)`. The key is
/// (Family, InterfaceLuid | InterfaceIndex). IPv4 only; IPv6 reports ERROR_NOT_FOUND. The legacy
/// stack exposes only the MTU and link state; the neighbour-discovery / DAD parameters are the
/// Windows defaults (as Wine reports), and Metric stays 0 with UseAutomaticMetric set.
#[no_mangle]
pub unsafe extern "system" fn GetIpInterfaceEntry(Row: *mut MIB_IPINTERFACE_ROW) -> u32 {
    if Row.is_null() {
        return E_INVALID_PARAMETER;
    }
    let fam = (*Row).Family;
    if fam != FAM_INET && fam != FAM_INET6 {
        return E_INVALID_PARAMETER;
    }
    let luid = (*Row).InterfaceLuid;
    let idx = if luid != 0 { luid_index(luid) } else { (*Row).InterfaceIndex };
    if idx == 0 {
        return E_NOT_FOUND;
    }
    if fam == FAM_INET6 {
        return E_NOT_FOUND;
    }
    let src = match lookup_row(idx) {
        Some(r) => r,
        None => return E_NOT_FOUND,
    };
    let mut row: MIB_IPINTERFACE_ROW = core::mem::zeroed();
    row.Family = fam;
    row.InterfaceLuid = make_luid(src.dwType, idx);
    row.InterfaceIndex = idx;
    row.MinRouterAdvertisementInterval = 200;
    row.MaxRouterAdvertisementInterval = 600;
    row.UseAutomaticMetric = 1;
    row.UseNeighborUnreachabilityDetection = 1;
    row.RouterDiscoveryBehavior = ROUTER_DISCOVERY_ENABLED;
    row.DadTransmits = 3;
    row.BaseReachableTime = 30000;
    row.RetransmitTime = 1000;
    row.PathMtuDiscoveryTimeout = 600000;
    row.LinkLocalAddressBehavior = LINK_LOCAL_DELAYED;
    row.NlMtu = src.dwMtu;
    row.Connected = (src.dwOperStatus == 4 || src.dwOperStatus == 5) as u8;
    row.SupportsNeighborDiscovery = 1;
    row.SupportsRouterDiscovery = 1;
    row.ReachableTime = 30000;
    core::ptr::write(Row, row);
    NO_ERROR
}

// ---------------------------------------------------------------------------------------------
// GetBestRoute2.
// ---------------------------------------------------------------------------------------------

/// C: `NETIO_STATUS GetBestRoute2(NET_LUID *InterfaceLuid, NET_IFINDEX InterfaceIndex,
/// const SOCKADDR_INET *SourceAddress, const SOCKADDR_INET *DestinationAddress,
/// ULONG AddressSortOptions, PMIB_IPFORWARD_ROW2 BestRoute, SOCKADDR_INET *BestSourceAddress)`.
///
/// Longest-prefix match over the synthesized IPv4 forward table, ties broken by lowest metric,
/// restricted to the interface named by InterfaceLuid / InterfaceIndex or by SourceAddress's
/// owning interface. The best source address is SourceAddress if given, else the interface's
/// address in the route's subnet (falling back to its first address). The kernel-side address
/// sort options and the interface metric are not modelled. IPv6: ERROR_NOT_FOUND (no v6 routes).
#[no_mangle]
pub unsafe extern "system" fn GetBestRoute2(
    InterfaceLuid: *const u64,
    InterfaceIndex: u32,
    SourceAddress: *const SockAddrInet,
    DestinationAddress: *const SockAddrInet,
    AddressSortOptions: u32,
    BestRoute: *mut MIB_IPFORWARD_ROW2,
    BestSourceAddress: *mut SockAddrInet,
) -> u32 {
    if DestinationAddress.is_null() || BestRoute.is_null() || BestSourceAddress.is_null() {
        return E_INVALID_PARAMETER;
    }
    core::ptr::write_bytes(BestRoute, 0, 1);
    core::ptr::write_bytes(BestSourceAddress, 0, 1);

    let fam = (*DestinationAddress).si_family;
    if fam != FAM_INET && fam != FAM_INET6 {
        return E_INVALID_PARAMETER;
    }
    if fam == FAM_INET6 {
        return E_NOT_FOUND;
    }
    let dst = (*DestinationAddress).w[0];

    // A source of the wrong family or of INADDR_ANY means "no source constraint".
    let src = if SourceAddress.is_null()
        || (*SourceAddress).si_family != FAM_INET
        || (*SourceAddress).w[0] == 0
    {
        None
    } else {
        Some((*SourceAddress).w[0])
    };

    let mut index = InterfaceIndex;
    if !InterfaceLuid.is_null() && *InterfaceLuid != 0 {
        index = luid_index(*InterfaceLuid);
    }

    let uni = match build_unicast_table(FAM_INET) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let fwd = match build_forward_table2(FAM_INET) {
        Ok(t) => t,
        Err(e) => {
            HeapFree(GetProcessHeap(), 0, uni as *mut c_void);
            return e;
        }
    };
    let urows = core::ptr::addr_of!((*uni).Table) as *const MIB_UNICASTIPADDRESS_ROW;
    let frows = core::ptr::addr_of!((*fwd).Table) as *const MIB_IPFORWARD_ROW2;
    let un = (*uni).NumEntries as usize;
    let fnum = (*fwd).NumEntries as usize;

    let mut ret = NO_ERROR;
    'done: {
        if index != 0 && !(0..un).any(|i| (*urows.add(i)).InterfaceIndex == index) {
            // Unknown (or address-less) interface.
            ret = if !InterfaceLuid.is_null() { E_NOT_FOUND } else { 2 /* ERROR_FILE_NOT_FOUND */ };
            break 'done;
        }
        if let Some(s) = src {
            match (0..un).find(|&i| (*urows.add(i)).Address.w[0] == s) {
                Some(i) => index = (*urows.add(i)).InterfaceIndex,
                None => {
                    ret = E_NOT_FOUND;
                    break 'done;
                }
            }
        }

        let mut best: Option<usize> = None;
        let mut best_len: i32 = -1;
        let mut best_metric = u32::MAX;
        for i in 0..fnum {
            let r = &*frows.add(i);
            if index != 0 && r.InterfaceIndex != index {
                continue;
            }
            let plen = r.DestinationPrefix.PrefixLength as u32;
            let mask = prefix_mask(plen);
            if (dst & mask) != (r.DestinationPrefix.Prefix.w[0] & mask) {
                continue;
            }
            let better = (plen as i32) > best_len
                || ((plen as i32) == best_len && r.Metric < best_metric);
            if better {
                best = Some(i);
                best_len = plen as i32;
                best_metric = r.Metric;
            }
        }
        let bi = match best {
            Some(i) => i,
            None => {
                ret = E_NOT_FOUND;
                break 'done;
            }
        };
        let route = &*frows.add(bi);

        // Source address: the caller's, else the route interface's address that shares the
        // subnet of the next hop (or destination if on-link), else its first address.
        let chosen = match src {
            Some(s) => Some(s),
            None => {
                let target = if route.NextHop.w[0] != 0 { route.NextHop.w[0] } else { dst };
                let mut first = None;
                let mut subnet = None;
                for i in 0..un {
                    let u = &*urows.add(i);
                    if u.InterfaceIndex != route.InterfaceIndex {
                        continue;
                    }
                    if first.is_none() {
                        first = Some(u.Address.w[0]);
                    }
                    let l = u.OnLinkPrefixLength as u32;
                    let m = prefix_mask(l);
                    if subnet.is_none() && (u.Address.w[0] & m) == (target & m) {
                        subnet = Some(u.Address.w[0]);
                    }
                }
                subnet.or(first)
            }
        };
        let chosen = match chosen {
            Some(c) => c,
            None => {
                ret = E_NOT_FOUND;
                break 'done;
            }
        };
        core::ptr::copy_nonoverlapping(route as *const MIB_IPFORWARD_ROW2, BestRoute, 1);
        *BestSourceAddress = SockAddrInet::v4(chosen);
    }
    HeapFree(GetProcessHeap(), 0, uni as *mut c_void);
    HeapFree(GetProcessHeap(), 0, fwd as *mut c_void);
    ret
}

// ---------------------------------------------------------------------------------------------
// Change notification.
// ---------------------------------------------------------------------------------------------

const NOTIFY_MAGIC: u32 = 0x4D4E_4F54; // "MNOT"

/// A change-notification registration. The handle returned to the caller is a pointer to this.
/// `refs` counts the caller's handle plus a queued initial-notification work item.
#[repr(C)]
struct NotifyReg {
    magic: u32,
    refs: AtomicU32,
    cancelled: AtomicBool,
    callback: usize,
    context: *mut c_void,
}

unsafe fn notify_release(reg: *mut NotifyReg) {
    if (*reg).refs.fetch_sub(1, Ordering::AcqRel) == 1 {
        (*reg).magic = 0;
        HeapFree(GetProcessHeap(), 0, reg as *mut c_void);
    }
}

/// Pool work item delivering the InitialNotification, as Windows does.
unsafe extern "system" fn notify_initial_worker(ctx: *mut c_void) -> u32 {
    let reg = ctx as *mut NotifyReg;
    if !(*reg).cancelled.load(Ordering::Acquire) {
        // PIPINTERFACE_CHANGE_CALLBACK / PIPFORWARD_CHANGE_CALLBACK:
        // VOID (CALLBACK)(PVOID CallerContext, PMIB_xxx Row, MIB_NOTIFICATION_TYPE Type)
        let cb: unsafe extern "system" fn(*mut c_void, *mut c_void, u32) =
            core::mem::transmute((*reg).callback);
        cb((*reg).context, core::ptr::null_mut(), MIB_INITIAL_NOTIFICATION);
    }
    notify_release(reg);
    0
}

unsafe fn register_notify(
    family: u32,
    callback: usize,
    context: *mut c_void,
    initial: u8,
    handle: *mut *mut c_void,
) -> u32 {
    if callback == 0 || handle.is_null() || family > 0xFFFF || !family_ok(family as u16) {
        return E_INVALID_PARAMETER;
    }
    let reg = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<NotifyReg>() as SIZE_T)
        as *mut NotifyReg;
    if reg.is_null() {
        return E_NOT_ENOUGH_MEMORY;
    }
    (*reg).magic = NOTIFY_MAGIC;
    (*reg).refs = AtomicU32::new(if initial != 0 { 2 } else { 1 });
    (*reg).cancelled = AtomicBool::new(false);
    (*reg).callback = callback;
    (*reg).context = context;
    // The handle is published before the callback can run, so it may already cancel itself.
    *handle = reg as *mut c_void;
    if initial != 0 && QueueUserWorkItem(notify_initial_worker, reg as *mut c_void, 0) == 0 {
        let e = GetLastError();
        *handle = core::ptr::null_mut();
        HeapFree(GetProcessHeap(), 0, reg as *mut c_void);
        return if e != 0 { e } else { E_NOT_ENOUGH_MEMORY };
    }
    NO_ERROR
}

/// C: `NETIO_STATUS NotifyIpInterfaceChange(ADDRESS_FAMILY Family,
/// PIPINTERFACE_CHANGE_CALLBACK Callback, PVOID CallerContext, BOOLEAN InitialNotification,
/// HANDLE *NotificationHandle)`.
///
/// Returns a real registration handle and, if requested, delivers the initial notification
/// (Row = NULL, MibInitialNotification) from a thread-pool work item like Windows. DIVERGENCE:
/// the legacy stack has no change-event source, so later add/delete/parameter notifications are
/// never delivered.
#[no_mangle]
pub unsafe extern "system" fn NotifyIpInterfaceChange(
    Family: u16, // ADDRESS_FAMILY: a USHORT, whose upper register bits x64 callers leave undefined
    Callback: usize,
    CallerContext: *mut c_void,
    InitialNotification: u8,
    NotificationHandle: *mut *mut c_void,
) -> u32 {
    register_notify(Family as u32, Callback, CallerContext, InitialNotification, NotificationHandle)
}

/// C: `NETIO_STATUS NotifyRouteChange2(ADDRESS_FAMILY Family, PIPFORWARD_CHANGE_CALLBACK Callback,
/// PVOID CallerContext, BOOLEAN InitialNotification, HANDLE *NotificationHandle)`.
/// Same contract and the same divergence as NotifyIpInterfaceChange: only the initial
/// notification is ever delivered.
#[no_mangle]
pub unsafe extern "system" fn NotifyRouteChange2(
    Family: u16, // ADDRESS_FAMILY: a USHORT, whose upper register bits x64 callers leave undefined
    Callback: usize,
    CallerContext: *mut c_void,
    InitialNotification: u8,
    NotificationHandle: *mut *mut c_void,
) -> u32 {
    register_notify(Family as u32, Callback, CallerContext, InitialNotification, NotificationHandle)
}

/// C: `NETIO_STATUS CancelMibChangeNotify2(HANDLE NotificationHandle)`. Stops further callbacks
/// and drops the registration. A callback already running (or a not-yet-run initial
/// notification, which is then skipped) is not waited for. The handle is invalid afterwards.
#[no_mangle]
pub unsafe extern "system" fn CancelMibChangeNotify2(NotificationHandle: *mut c_void) -> u32 {
    if NotificationHandle.is_null() {
        return E_INVALID_PARAMETER;
    }
    let reg = NotificationHandle as *mut NotifyReg;
    if (*reg).magic != NOTIFY_MAGIC {
        return E_INVALID_PARAMETER;
    }
    (*reg).cancelled.store(true, Ordering::Release);
    notify_release(reg);
    NO_ERROR
}

// ---------------------------------------------------------------------------------------------
// Compartments.
// ---------------------------------------------------------------------------------------------

/// C: `NETIO_STATUS SetCurrentThreadCompartmentId(NET_IF_COMPARTMENT_ID CompartmentId)`.
/// Compartment 1 (NET_IF_COMPARTMENT_ID_PRIMARY, the Windows default) is the only compartment
/// this system has: selecting it succeeds, anything else (0 = unspecified, or a nonexistent
/// compartment) is ERROR_INVALID_PARAMETER. GetCurrentThreadCompartmentId always reports 1.
#[no_mangle]
pub extern "system" fn SetCurrentThreadCompartmentId(CompartmentId: u32) -> u32 {
    if CompartmentId == 1 {
        NO_ERROR
    } else {
        E_INVALID_PARAMETER
    }
}

// The initial-notification worker casts the callback as
// `extern "system" fn(PVOID ctx, PMIB_xxx Row, MIB_NOTIFICATION_TYPE)`, which is exactly
// PUNICAST_IPADDRESS_CHANGE_CALLBACK (stdcall on i386), so it is shared.

/// C: `NETIO_STATUS NotifyUnicastIpAddressChange(ADDRESS_FAMILY Family,
/// PUNICAST_IPADDRESS_CHANGE_CALLBACK Callback, PVOID CallerContext, BOOLEAN InitialNotification,
/// HANDLE *NotificationHandle)`.  Same contract and divergence as NotifyIpInterfaceChange (real
/// cancellable handle, initial notification from a pool item, later changes never delivered).
/// DEVIATION from Wine: Wine returns a NULL handle and calls the callback inline; here the
/// handle works with CancelMibChangeNotify2.
#[no_mangle]
pub unsafe extern "system" fn NotifyUnicastIpAddressChange(
    Family: u16,
    Callback: usize,
    CallerContext: *mut c_void,
    InitialNotification: u8,
    NotificationHandle: *mut *mut c_void,
) -> u32 {
    register_notify(Family as u32, Callback, CallerContext, InitialNotification, NotificationHandle)
}

/// C: `VOID InitializeIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row)`: prepares a row for
/// SetIpInterfaceEntry, where "all ones" means "leave this setting unchanged". Measured on
/// Windows 11 (amd64): the whole row, padding included, becomes 0xFF, then Family,
/// InterfaceLuid, InterfaceIndex, BaseReachableTime, RetransmitTime, PathMtuDiscoveryTimeout
/// and NlMtu are zeroed. (Microsoft's remarks say only "other fields ... zero", which is
/// not what it does.) No NULL check: Windows has none either.
#[no_mangle]
pub unsafe extern "system" fn InitializeIpInterfaceEntry(Row: *mut MIB_IPINTERFACE_ROW) {
    core::ptr::write_bytes(Row as *mut u8, 0xFF, core::mem::size_of::<MIB_IPINTERFACE_ROW>());
    (*Row).Family = 0; // AF_UNSPEC
    (*Row).InterfaceLuid = 0;
    (*Row).InterfaceIndex = 0;
    (*Row).BaseReachableTime = 0;
    (*Row).RetransmitTime = 0;
    (*Row).PathMtuDiscoveryTimeout = 0;
    (*Row).NlMtu = 0;
}
