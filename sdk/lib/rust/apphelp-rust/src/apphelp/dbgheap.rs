use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const TRACE_ALL_FREE_CALLS: i32 = 1;

#[repr(C)]
pub struct SHIM_ALLOC_ENTRY {
    pub Address: PVOID,
    pub Size: SIZE_T,
    pub Line: i32,
    pub File: *const i8,
    pub Next: PVOID,
    pub Prev: PVOID,
}

pub type PSHIM_ALLOC_ENTRY = *mut SHIM_ALLOC_ENTRY;

pub static mut g_SdbpAllocationLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut g_SdbpAllocationTable: RTL_AVL_TABLE = unsafe { core::mem::zeroed() };
pub static mut g_PrivAllocationHeap: HANDLE = core::ptr::null_mut();
