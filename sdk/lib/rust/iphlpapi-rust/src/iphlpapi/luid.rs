//! iphlpapi.dll interface LUID helpers: the `ConvertInterface*` family, `if_nametoindex` /
//! `if_indextoname`, `GetIfEntry2` / `GetIfTable2` (MIB_IF_ROW2), `FreeMibTable`, and
//! `GetCurrentThreadCompartmentId`. All are thin wrappers over the existing interface lookup
//! (`GetIfEntry` / `GetIfTable`) and the process heap.
//!
//! NET_LUID encoding (64 bits): bits 0..23 Reserved (zero), bits 24..47 NetLuidIndex (the
//! interface index), bits 48..63 IfType. The LUID is carried as a plain `u64` (ABI-identical to
//! the NET_LUID union).

use crate::prelude::*;
use core::ffi::c_void;

// Windows error codes returned by the helpers below.
const LUID_ERROR_FILE_NOT_FOUND: u32 = 2;
const LUID_ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
const LUID_ERROR_OUTOFMEMORY: u32 = 14;
const LUID_ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const LUID_ERROR_INVALID_NAME: u32 = 123;

// Windows SDK constants (ifdef.h / netioapi.h / winbase.h).
pub const IF_MAX_STRING_SIZE: usize = 256;
pub const IF_MAX_PHYS_ADDRESS_LENGTH: usize = 32;
const NET_IF_COMPARTMENT_ID_PRIMARY: u32 = 1;
const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
const LUID_INDEX_MASK: u64 = 0xFF_FFFF;

// IfType values used by the name table and the derived fields.
const IF_TYPE_ETHERNET_CSMACD: u32 = 6;
const IF_TYPE_PPP: u32 = 23;
const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;
const IF_TYPE_IEEE80211: u32 = 71;
const IF_TYPE_TUNNEL: u32 = 131;

// NET_IF_ACCESS_TYPE / NET_IF_MEDIA_CONNECT_STATE / NDIS_MEDIUM values.
const NET_IF_ACCESS_LOOPBACK: u32 = 1;
const NET_IF_ACCESS_BROADCAST: u32 = 2;
const NET_IF_ACCESS_POINT_TO_POINT: u32 = 3;
const MEDIA_CONNECT_STATE_CONNECTED: u32 = 1;
const MEDIA_CONNECT_STATE_DISCONNECTED: u32 = 2;
const NDIS_MEDIUM_802_3: u32 = 0;
const NDIS_MEDIUM_NATIVE_802_11: u32 = 9;
// MIB_IF_OPERATIONAL / CONNECTED values of IF_OPER_STATUS.
const IF_OPER_STATUS_CONNECTED: u32 = 4;
const IF_OPER_STATUS_OPERATIONAL: u32 = 5;

/// GUID tag placed in Data4 of interface GUIDs derived from a LUID. Interface GUIDs are not
/// stored anywhere in this port, so they are a deterministic encoding of the LUID instead:
/// Data1 = NetLuidIndex, Data2 = IfType, Data3 = 0, Data4 = "VIBELUID". ConvertInterfaceGuidToLuid
/// decodes exactly that form and checks the interface still exists with the same type.
const LUID_GUID_TAG: [u8; 8] = *b"VIBELUID";

/// C: `MIB_IF_ROW2` (netioapi.h). Layout is exact; on 64-bit the size is 1352 bytes.
#[repr(C)]
pub struct MIB_IF_ROW2 {
    pub InterfaceLuid: u64,
    pub InterfaceIndex: u32,
    pub InterfaceGuid: GUID,
    pub Alias: [u16; IF_MAX_STRING_SIZE + 1],
    pub Description: [u16; IF_MAX_STRING_SIZE + 1],
    pub PhysicalAddressLength: u32,
    pub PhysicalAddress: [u8; IF_MAX_PHYS_ADDRESS_LENGTH],
    pub PermanentPhysicalAddress: [u8; IF_MAX_PHYS_ADDRESS_LENGTH],
    pub Mtu: u32,
    pub Type: u32,
    pub TunnelType: u32,
    pub MediaType: u32,
    pub PhysicalMediumType: u32,
    pub AccessType: u32,
    pub DirectionType: u32,
    /// Union of the packed BOOLEAN bitfields (HardwareInterface .. LowPower) — one byte.
    pub InterfaceAndOperStatusFlags: u8,
    pub OperStatus: u32,
    pub AdminStatus: u32,
    pub MediaConnectState: u32,
    pub NetworkGuid: GUID,
    pub ConnectionType: u32,
    pub TransmitLinkSpeed: u64,
    pub ReceiveLinkSpeed: u64,
    pub InOctets: u64,
    pub InUcastPkts: u64,
    pub InNUcastPkts: u64,
    pub InDiscards: u64,
    pub InErrors: u64,
    pub InUnknownProtos: u64,
    pub InUcastOctets: u64,
    pub InMulticastOctets: u64,
    pub InBroadcastOctets: u64,
    pub OutOctets: u64,
    pub OutUcastPkts: u64,
    pub OutNUcastPkts: u64,
    pub OutDiscards: u64,
    pub OutErrors: u64,
    pub OutUcastOctets: u64,
    pub OutMulticastOctets: u64,
    pub OutBroadcastOctets: u64,
    pub OutQLen: u64,
}
pub type PMIB_IF_ROW2 = *mut MIB_IF_ROW2;

/// C: `MIB_IF_TABLE2` — `ULONG NumEntries; MIB_IF_ROW2 Table[1];` (Table is 8-aligned).
#[repr(C)]
pub struct MIB_IF_TABLE2 {
    pub NumEntries: u32,
    pub Table: [MIB_IF_ROW2; 1],
}
pub type PMIB_IF_TABLE2 = *mut MIB_IF_TABLE2;

// Offsets measured from the mingw-w64 SDK headers (llvm-mingw clang, offsetof), identical
// on i686 and x86_64; MIB_IF_ROW2 holds no pointers, so arm64 matches.
const _: () = assert!(core::mem::size_of::<MIB_IF_ROW2>() == 1352);
const _: () = assert!(core::mem::offset_of!(MIB_IF_TABLE2, Table) == 8);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, InterfaceLuid) == 0);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, InterfaceIndex) == 8);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, InterfaceGuid) == 12);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, Alias) == 28);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, Description) == 542);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, PhysicalAddressLength) == 1056);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, PhysicalAddress) == 1060);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, PermanentPhysicalAddress) == 1092);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, Mtu) == 1124);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, Type) == 1128);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, TunnelType) == 1132);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, MediaType) == 1136);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, PhysicalMediumType) == 1140);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, AccessType) == 1144);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, DirectionType) == 1148);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, InterfaceAndOperStatusFlags) == 1152);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, OperStatus) == 1156);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, AdminStatus) == 1160);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, MediaConnectState) == 1164);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, NetworkGuid) == 1168);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, ConnectionType) == 1184);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, TransmitLinkSpeed) == 1192);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, ReceiveLinkSpeed) == 1200);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, InOctets) == 1208);
const _: () = assert!(core::mem::offset_of!(MIB_IF_ROW2, OutQLen) == 1344);

#[inline]
pub(crate) fn luid_index(luid: u64) -> u32 {
    ((luid >> 24) & LUID_INDEX_MASK) as u32
}

#[inline]
fn luid_type(luid: u64) -> u32 {
    ((luid >> 48) & 0xFFFF) as u32
}

#[inline]
pub(crate) fn make_luid(if_type: u32, index: u32) -> u64 {
    ((if_type as u64 & 0xFFFF) << 48) | ((index as u64 & LUID_INDEX_MASK) << 24)
}

/// Looks up the interface row for `index`. None when the index is 0, out of range, or unknown.
pub(crate) unsafe fn lookup_row(index: u32) -> Option<MIB_IFROW> {
    if index == 0 || index as u64 > LUID_INDEX_MASK {
        return None;
    }
    let mut row: MIB_IFROW = core::mem::zeroed();
    row.dwIndex = index;
    if crate::iphlpapi::iphlpapi_main::GetIfEntry(&mut row) == NO_ERROR {
        Some(row)
    } else {
        None
    }
}

/// Name prefix for an IfType (the part before the underscore in "<prefix>_<index>").
fn prefix_for_type(if_type: u32) -> &'static str {
    match if_type {
        IF_TYPE_ETHERNET_CSMACD => "ethernet",
        IF_TYPE_IEEE80211 => "wireless",
        IF_TYPE_SOFTWARE_LOOPBACK => "loopback",
        IF_TYPE_TUNNEL => "tunnel",
        IF_TYPE_PPP => "ppp",
        _ => "other",
    }
}

/// Writes "<prefix>_<decimal index>" as UTF-16 into `buf` and returns the unit count (no NUL).
fn name_units(if_type: u32, index: u32, buf: &mut [u16; 64]) -> usize {
    let mut n = 0usize;
    for &b in prefix_for_type(if_type).as_bytes() {
        buf[n] = b as u16;
        n += 1;
    }
    buf[n] = b'_' as u16;
    n += 1;
    let mut digits = [0u8; 10];
    let mut d = 0usize;
    let mut v = index;
    loop {
        digits[d] = b'0' + (v % 10) as u8;
        d += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while d > 0 {
        d -= 1;
        buf[n] = digits[d] as u16;
        n += 1;
    }
    n
}

/// Widens the ANSI `bDescr` of a MIB_IFROW into `out` (NUL-terminated); returns the unit count.
fn descr_units(row: &MIB_IFROW, out: &mut [u16; IF_MAX_STRING_SIZE + 1]) -> usize {
    let mut i = 0usize;
    while i < row.bDescr.len() && row.bDescr[i] != 0 {
        out[i] = row.bDescr[i] as u16;
        i += 1;
    }
    out[i] = 0;
    i
}

/// Parses "<prefix>_<decimal>" from a NUL-terminated UTF-16 string. Returns the position of the
/// underscore (= prefix length) and the index, which must be in 1..=0xFFFFFF.
unsafe fn split_name_w(name: *const u16) -> Option<(usize, u32)> {
    let mut len = 0usize;
    while *name.add(len) != 0 {
        len += 1;
        if len >= IF_MAX_STRING_SIZE {
            return None;
        }
    }
    let units = core::slice::from_raw_parts(name, len);
    let p = units.iter().position(|&c| c == b'_' as u16)?;
    let digits = &units[p + 1..];
    if digits.is_empty() || digits.len() > 8 {
        return None;
    }
    let mut v: u32 = 0;
    for &c in digits {
        if c < b'0' as u16 || c > b'9' as u16 {
            return None;
        }
        v = v * 10 + (c - b'0' as u16) as u32;
    }
    if v == 0 || v as u64 > LUID_INDEX_MASK {
        return None;
    }
    Some((p, v))
}

/// Fills a MIB_IF_ROW2 from the legacy MIB_IFROW. Fields the legacy row cannot supply are left
/// zero: NetworkGuid, ConnectionType, PhysicalMediumType, the InUcastOctets / InMulticastOctets /
/// InBroadcastOctets counters, TunnelType, DirectionType and the status flag byte.
unsafe fn fill_if_row2(src: &MIB_IFROW, dst: &mut MIB_IF_ROW2) {
    let t = src.dwType;
    let index = src.dwIndex;
    dst.InterfaceIndex = index;
    dst.InterfaceLuid = make_luid(t, index);
    dst.InterfaceGuid = guid_from_luid(t, index);

    let mut d = [0u16; IF_MAX_STRING_SIZE + 1];
    descr_units(src, &mut d);
    core::ptr::copy_nonoverlapping(d.as_ptr(), dst.Alias.as_mut_ptr(), IF_MAX_STRING_SIZE + 1);
    core::ptr::copy_nonoverlapping(
        d.as_ptr(),
        dst.Description.as_mut_ptr(),
        IF_MAX_STRING_SIZE + 1,
    );

    // The legacy row only carries 8 physical-address bytes.
    let plen = (src.dwPhysAddrLen as usize).min(src.bPhysAddr.len());
    dst.PhysicalAddressLength = plen as u32;
    core::ptr::copy_nonoverlapping(src.bPhysAddr.as_ptr(), dst.PhysicalAddress.as_mut_ptr(), plen);
    core::ptr::copy_nonoverlapping(
        src.bPhysAddr.as_ptr(),
        dst.PermanentPhysicalAddress.as_mut_ptr(),
        plen,
    );

    dst.Mtu = src.dwMtu;
    dst.Type = t;
    dst.MediaType = match t {
        IF_TYPE_ETHERNET_CSMACD => NDIS_MEDIUM_802_3,
        IF_TYPE_IEEE80211 => NDIS_MEDIUM_NATIVE_802_11,
        _ => 0,
    };
    dst.AccessType = match t {
        IF_TYPE_SOFTWARE_LOOPBACK => NET_IF_ACCESS_LOOPBACK,
        IF_TYPE_PPP | IF_TYPE_TUNNEL => NET_IF_ACCESS_POINT_TO_POINT,
        _ => NET_IF_ACCESS_BROADCAST,
    };

    dst.OperStatus = src.dwOperStatus;
    dst.AdminStatus = src.dwAdminStatus;
    dst.MediaConnectState = if src.dwOperStatus == IF_OPER_STATUS_CONNECTED
        || src.dwOperStatus == IF_OPER_STATUS_OPERATIONAL
    {
        MEDIA_CONNECT_STATE_CONNECTED
    } else {
        MEDIA_CONNECT_STATE_DISCONNECTED
    };

    dst.TransmitLinkSpeed = src.dwSpeed as u64;
    dst.ReceiveLinkSpeed = src.dwSpeed as u64;
    dst.InOctets = src.dwInOctets as u64;
    dst.InUcastPkts = src.dwInUcastPkts as u64;
    dst.InNUcastPkts = src.dwInNUcastPkts as u64;
    dst.InDiscards = src.dwInDiscards as u64;
    dst.InErrors = src.dwInErrors as u64;
    dst.InUnknownProtos = src.dwInUnknownProtos as u64;
    dst.OutOctets = src.dwOutOctets as u64;
    dst.OutUcastPkts = src.dwOutUcastPkts as u64;
    dst.OutNUcastPkts = src.dwOutNUcastPkts as u64;
    dst.OutDiscards = src.dwOutDiscards as u64;
    dst.OutErrors = src.dwOutErrors as u64;
    dst.OutQLen = src.dwOutQLen as u64;
}

/// Deterministic interface GUID for (IfType, index); see LUID_GUID_TAG.
fn guid_from_luid(if_type: u32, index: u32) -> GUID {
    GUID {
        Data1: index,
        Data2: if_type as u16,
        Data3: 0,
        Data4: LUID_GUID_TAG,
    }
}

/// C: `VOID WINAPI FreeMibTable(PVOID Memory)`. Frees a table allocated by the
/// IP Helper API (the process heap backs all of them). NULL is a no-op.
#[no_mangle]
pub unsafe extern "system" fn FreeMibTable(Memory: *mut c_void) {
    if !Memory.is_null() {
        HeapFree(GetProcessHeap(), 0, Memory);
    }
}

/// C: `NETIO_STATUS ConvertInterfaceIndexToLuid(NET_IFINDEX InterfaceIndex,
/// PNET_LUID InterfaceLuid)`.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceIndexToLuid(
    InterfaceIndex: u32,
    InterfaceLuid: *mut u64,
) -> u32 {
    if InterfaceLuid.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let mut row: MIB_IFROW = core::mem::zeroed();
    row.dwIndex = InterfaceIndex;
    if crate::iphlpapi::iphlpapi_main::GetIfEntry(&mut row) != NO_ERROR {
        *InterfaceLuid = 0;
        return LUID_ERROR_FILE_NOT_FOUND;
    }

    let if_type = row.dwType as u64 & 0xFFFF;
    let index = InterfaceIndex as u64 & LUID_INDEX_MASK;
    *InterfaceLuid = (if_type << 48) | (index << 24);
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceLuidToIndex(const NET_LUID*, NET_IFINDEX*)`.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceLuidToIndex(
    InterfaceLuid: *const u64,
    InterfaceIndex: *mut u32,
) -> u32 {
    if InterfaceLuid.is_null() || InterfaceIndex.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let index = luid_index(*InterfaceLuid);
    if lookup_row(index).is_none() {
        *InterfaceIndex = 0;
        return LUID_ERROR_FILE_NOT_FOUND;
    }
    *InterfaceIndex = index;
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceLuidToNameW(const NET_LUID*, PWSTR, SIZE_T)`.
/// Produces "<prefix>_<index>" (e.g. "ethernet_32768"). `len` counts WCHARs including the NUL.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceLuidToNameW(
    InterfaceLuid: *const u64,
    InterfaceAlias: *mut u16,
    Length: usize,
) -> u32 {
    if InterfaceLuid.is_null() || InterfaceAlias.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let luid = *InterfaceLuid;
    let mut buf = [0u16; 64];
    let n = name_units(luid_type(luid), luid_index(luid), &mut buf);
    if n + 1 > Length {
        return LUID_ERROR_NOT_ENOUGH_MEMORY;
    }
    core::ptr::copy_nonoverlapping(buf.as_ptr(), InterfaceAlias, n);
    *InterfaceAlias.add(n) = 0;
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceLuidToNameA(const NET_LUID*, PSTR, SIZE_T)`.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceLuidToNameA(
    InterfaceLuid: *const u64,
    InterfaceAlias: *mut i8,
    Length: usize,
) -> u32 {
    if InterfaceLuid.is_null() || InterfaceAlias.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let mut wbuf = [0u16; 64];
    let r = ConvertInterfaceLuidToNameW(InterfaceLuid, wbuf.as_mut_ptr(), wbuf.len());
    if r != NO_ERROR {
        return r;
    }
    let mut n = 0usize;
    while wbuf[n] != 0 {
        n += 1;
    }
    if n + 1 > Length {
        return LUID_ERROR_NOT_ENOUGH_MEMORY;
    }
    for i in 0..n {
        *InterfaceAlias.add(i) = wbuf[i] as u8 as i8;
    }
    *InterfaceAlias.add(n) = 0;
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceNameToLuidW(PCWSTR, NET_LUID*)`. Parses "<prefix>_<index>".
/// The prefix must be the one the interface's real IfType maps to, so every name produced by
/// ConvertInterfaceLuidToNameW parses back to the same LUID.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceNameToLuidW(
    InterfaceName: *const u16,
    InterfaceLuid: *mut u64,
) -> u32 {
    if InterfaceLuid.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *InterfaceLuid = 0;
    if InterfaceName.is_null() {
        return LUID_ERROR_INVALID_NAME;
    }
    let (plen, index) = match split_name_w(InterfaceName) {
        Some(v) => v,
        None => return LUID_ERROR_INVALID_NAME,
    };
    let row = match lookup_row(index) {
        Some(r) => r,
        None => return LUID_ERROR_INVALID_NAME,
    };
    let want = prefix_for_type(row.dwType).as_bytes();
    if plen != want.len() {
        return LUID_ERROR_INVALID_NAME;
    }
    for i in 0..plen {
        if *InterfaceName.add(i) != want[i] as u16 {
            return LUID_ERROR_INVALID_NAME;
        }
    }
    *InterfaceLuid = make_luid(row.dwType, index);
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceNameToLuidA(PCSTR, NET_LUID*)`.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceNameToLuidA(
    InterfaceName: *const i8,
    InterfaceLuid: *mut u64,
) -> u32 {
    if InterfaceLuid.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *InterfaceLuid = 0;
    if InterfaceName.is_null() {
        return LUID_ERROR_INVALID_NAME;
    }
    let mut wbuf = [0u16; IF_MAX_STRING_SIZE];
    let mut n = 0usize;
    loop {
        let c = *InterfaceName.add(n);
        if c == 0 {
            break;
        }
        if n + 1 >= IF_MAX_STRING_SIZE {
            return LUID_ERROR_INVALID_NAME;
        }
        wbuf[n] = c as u8 as u16;
        n += 1;
    }
    wbuf[n] = 0;
    ConvertInterfaceNameToLuidW(wbuf.as_ptr(), InterfaceLuid)
}

/// C: `NETIO_STATUS ConvertInterfaceLuidToAlias(const NET_LUID*, PWSTR, SIZE_T)`. The alias is
/// the adapter description (the legacy row's bDescr, widened).
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceLuidToAlias(
    InterfaceLuid: *const u64,
    InterfaceAlias: *mut u16,
    Length: usize,
) -> u32 {
    if InterfaceLuid.is_null() || InterfaceAlias.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let row = match lookup_row(luid_index(*InterfaceLuid)) {
        Some(r) => r,
        None => return LUID_ERROR_FILE_NOT_FOUND,
    };
    let mut d = [0u16; IF_MAX_STRING_SIZE + 1];
    let n = descr_units(&row, &mut d);
    if n + 1 > Length {
        return LUID_ERROR_NOT_ENOUGH_MEMORY;
    }
    core::ptr::copy_nonoverlapping(d.as_ptr(), InterfaceAlias, n + 1);
    NO_ERROR
}

/// C: `NETIO_STATUS ConvertInterfaceLuidToGuid(const NET_LUID*, GUID*)`. Zeroes the output on
/// failure. The GUID is the deterministic encoding described at LUID_GUID_TAG.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceLuidToGuid(
    InterfaceLuid: *const u64,
    InterfaceGuid: *mut GUID,
) -> u32 {
    if InterfaceLuid.is_null() || InterfaceGuid.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let index = luid_index(*InterfaceLuid);
    match lookup_row(index) {
        Some(row) => {
            *InterfaceGuid = guid_from_luid(row.dwType, index);
            NO_ERROR
        }
        None => {
            *InterfaceGuid = GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            };
            LUID_ERROR_FILE_NOT_FOUND
        }
    }
}

/// C: `NETIO_STATUS ConvertInterfaceGuidToLuid(const GUID*, NET_LUID*)`. Inverse of
/// ConvertInterfaceLuidToGuid; any GUID not in that encoding, or naming a missing or retyped
/// interface, is ERROR_INVALID_PARAMETER.
#[no_mangle]
pub unsafe extern "system" fn ConvertInterfaceGuidToLuid(
    InterfaceGuid: *const GUID,
    InterfaceLuid: *mut u64,
) -> u32 {
    if InterfaceGuid.is_null() || InterfaceLuid.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *InterfaceLuid = 0;
    let g = &*InterfaceGuid;
    if g.Data3 != 0 || g.Data4 != LUID_GUID_TAG {
        return ERROR_INVALID_PARAMETER;
    }
    let index = g.Data1;
    let row = match lookup_row(index) {
        Some(r) => r,
        None => return ERROR_INVALID_PARAMETER,
    };
    if row.dwType != g.Data2 as u32 {
        return ERROR_INVALID_PARAMETER;
    }
    *InterfaceLuid = make_luid(row.dwType, index);
    NO_ERROR
}

/// C: `NET_IFINDEX if_nametoindex(PCSTR InterfaceName)`. 0 on failure.
#[no_mangle]
pub unsafe extern "system" fn if_nametoindex(InterfaceName: *const i8) -> u32 {
    let mut luid: u64 = 0;
    if ConvertInterfaceNameToLuidA(InterfaceName, &mut luid) != NO_ERROR {
        return 0;
    }
    let mut index: u32 = 0;
    if ConvertInterfaceLuidToIndex(&luid, &mut index) != NO_ERROR {
        return 0;
    }
    index
}

/// C: `PCHAR if_indextoname(NET_IFINDEX InterfaceIndex, PCHAR InterfaceName)`. The caller's
/// buffer must hold IF_MAX_STRING_SIZE + 1 chars. NULL on failure.
#[no_mangle]
pub unsafe extern "system" fn if_indextoname(InterfaceIndex: u32, InterfaceName: *mut i8) -> *mut i8 {
    if InterfaceName.is_null() {
        return core::ptr::null_mut();
    }
    let mut luid: u64 = 0;
    if ConvertInterfaceIndexToLuid(InterfaceIndex, &mut luid) != NO_ERROR {
        return core::ptr::null_mut();
    }
    if ConvertInterfaceLuidToNameA(&luid, InterfaceName, IF_MAX_STRING_SIZE + 1) != NO_ERROR {
        return core::ptr::null_mut();
    }
    InterfaceName
}

/// C: `NETIO_STATUS GetIfEntry2(PMIB_IF_ROW2 Row)`. Uses InterfaceLuid when set, otherwise
/// InterfaceIndex; the row is fully rewritten on success.
#[no_mangle]
pub unsafe extern "system" fn GetIfEntry2(Row: PMIB_IF_ROW2) -> u32 {
    if Row.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let luid = (*Row).InterfaceLuid;
    let index = (*Row).InterfaceIndex;
    if luid == 0 && index == 0 {
        return ERROR_INVALID_PARAMETER;
    }
    let idx = if luid != 0 { luid_index(luid) } else { index };
    let src = match lookup_row(idx) {
        Some(r) => r,
        None => return LUID_ERROR_FILE_NOT_FOUND,
    };
    let mut out: MIB_IF_ROW2 = core::mem::zeroed();
    fill_if_row2(&src, &mut out);
    core::ptr::write(Row, out);
    NO_ERROR
}

/// C: `NETIO_STATUS GetIfTable2(PMIB_IF_TABLE2* Table)`. Allocates with the process heap;
/// release with FreeMibTable.
#[no_mangle]
pub unsafe extern "system" fn GetIfTable2(Table: *mut PMIB_IF_TABLE2) -> u32 {
    if Table.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *Table = core::ptr::null_mut();

    let heap = GetProcessHeap();

    // Ask GetIfTable for the exact legacy-table size, then fetch it.
    let mut size: u32 = 0;
    let r = crate::iphlpapi::iphlpapi_main::GetIfTable(core::ptr::null_mut(), &mut size, FALSE);
    if r != LUID_ERROR_INSUFFICIENT_BUFFER {
        return r;
    }
    let raw = HeapAlloc(heap, 0, size as SIZE_T) as *mut MIB_IFTABLE;
    if raw.is_null() {
        return LUID_ERROR_OUTOFMEMORY;
    }
    let r = crate::iphlpapi::iphlpapi_main::GetIfTable(raw, &mut size, FALSE);
    if r != NO_ERROR {
        HeapFree(heap, 0, raw as *mut c_void);
        return r;
    }

    let count = (*raw).dwNumEntries as usize;
    let out_bytes = core::mem::size_of::<MIB_IF_TABLE2>()
        + count.saturating_sub(1) * core::mem::size_of::<MIB_IF_ROW2>();
    let out = HeapAlloc(heap, HEAP_ZERO_MEMORY, out_bytes as SIZE_T) as *mut MIB_IF_TABLE2;
    if out.is_null() {
        HeapFree(heap, 0, raw as *mut c_void);
        return LUID_ERROR_OUTOFMEMORY;
    }

    (*out).NumEntries = count as u32;
    let src_rows = core::ptr::addr_of!((*raw).table) as *const MIB_IFROW;
    let dst_rows = core::ptr::addr_of_mut!((*out).Table) as *mut MIB_IF_ROW2;
    for i in 0..count {
        fill_if_row2(&*src_rows.add(i), &mut *dst_rows.add(i));
    }

    HeapFree(heap, 0, raw as *mut c_void);
    *Table = out;
    NO_ERROR
}

/// C: `COMPARTMENT_ID GetCurrentThreadCompartmentId(void)`. Only the primary compartment exists.
#[no_mangle]
pub extern "system" fn GetCurrentThreadCompartmentId() -> u32 {
    NET_IF_COMPARTMENT_ID_PRIMARY
}
