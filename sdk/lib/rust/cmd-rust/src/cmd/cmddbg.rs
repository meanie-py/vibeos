use ntdll_sys::nt::*;
use ntdll_sys::nt::tier1::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[allow(non_camel_case_types)]
pub const REDZONE_SIZE: usize = 32;
pub const REDZONE_LEFT: u8 = 0x78;
pub const REDZONE_RIGHT: u8 = 0x87;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct alloc_info {
    pub size: usize,
    pub list_entry: LIST_ENTRY,
    pub file: *const i8,
    pub line: i32,
}

#[allow(non_camel_case_types)]
pub type palloc_info = *mut alloc_info;

#[allow(non_upper_case_globals)]
pub static mut allocations: usize = 0;

#[allow(non_upper_case_globals)]
pub static mut allocated_memory: usize = 0;

#[allow(non_upper_case_globals)]
pub static mut alloc_list_head: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::addr_of_mut!(alloc_list_head) as PLIST_ENTRY,
    Blink: core::ptr::addr_of_mut!(alloc_list_head) as PLIST_ENTRY,
};
