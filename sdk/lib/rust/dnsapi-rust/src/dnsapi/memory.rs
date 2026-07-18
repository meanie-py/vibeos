use crate::prelude::*;
use core::ffi::c_void;


// ---- DnsApiFree ----
#[no_mangle]
pub unsafe extern "system" fn DnsApiFree(Data: *mut core::ffi::c_void) {
    RtlFreeHeap(RtlGetProcessHeap(), 0u32, Data);
}

// ---- DnsApiAlloc ----
#[no_mangle]
pub unsafe extern "system" fn DnsApiAlloc(Size: u32) -> *mut core::ffi::c_void {
    RtlAllocateHeap(RtlGetProcessHeap(), 0u32, Size as usize)
}

// ---- DnsQueryConfigAllocEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsQueryConfigAllocEx(
    Config: DNS_CONFIG_TYPE,
    pBuffer: PVOID,
    pBufferLength: PDWORD,
) -> PVOID {
    core::ptr::null_mut()
}

// ---- DnsFree ----
#[no_mangle]
pub unsafe extern "system" fn DnsFree(Data: PVOID, FreeType: DNS_FREE_TYPE) {
    match FreeType {
        _DNS_FREE_TYPE_DnsFreeFlat => {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Data);
        }
        _DNS_FREE_TYPE_DnsFreeRecordList => {
            DnsIntFreeRecordList(Data as *mut _);
        }
        _DNS_FREE_TYPE_DnsFreeParsedMessageFields => {
            // assert(FALSE); not implemented
        }
        _ => {}
    }
}
