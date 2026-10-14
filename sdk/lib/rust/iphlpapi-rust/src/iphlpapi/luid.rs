//! iphlpapi.dll interface LUID helpers: `ConvertInterfaceIndexToLuid` and
//! `FreeMibTable`. Both are thin wrappers over the existing interface lookup
//! (`GetIfEntry`) and the process heap.

use crate::prelude::*;
use core::ffi::c_void;

// Windows error codes returned by the NETIO_STATUS-style helpers below.
const LUID_ERROR_FILE_NOT_FOUND: u32 = 2;

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
///
/// NET_LUID layout (64 bits): bits 0..23 Reserved (zero), bits 24..47
/// NetLuidIndex (the interface index), bits 48..63 IfType.
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
    let index = InterfaceIndex as u64 & 0xFF_FFFF;
    *InterfaceLuid = (if_type << 48) | (index << 24);
    NO_ERROR
}
