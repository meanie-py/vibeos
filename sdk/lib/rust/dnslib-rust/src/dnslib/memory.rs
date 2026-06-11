use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "system" {
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;
    fn LocalAlloc(uFlags: u32, uBytes: usize) -> *mut c_void;
}

pub type PDNS_ALLOC_FUNCTION = Option<unsafe extern "system" fn(Size: usize) -> *mut c_void>;
pub type PDNS_FREE_FUNCTION = Option<unsafe extern "system" fn(Buffer: *mut c_void)>;
pub static mut pDnsAllocFunction: PDNS_ALLOC_FUNCTION = None;
pub static mut pDnsFreeFunction: PDNS_FREE_FUNCTION = None;

const LMEM_ZEROINIT: u32 = 0x0040;

#[no_mangle]
pub unsafe extern "system" fn Dns_Free(Address: *mut c_void) {
    if let Some(dns_free) = pDnsFreeFunction {
        dns_free(Address);
    } else {
        LocalFree(Address);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Dns_AllocZero(Size: usize) -> *mut c_void {
    if let Some(alloc_fn) = pDnsAllocFunction {
        let Buffer = alloc_fn(Size);
        if !Buffer.is_null() {
            core::ptr::write_bytes(Buffer, 0, Size);
        }
        Buffer
    } else {
        LocalAlloc(LMEM_ZEROINIT, Size)
    }
}
