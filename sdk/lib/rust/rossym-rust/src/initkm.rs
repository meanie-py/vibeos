use crate::types::*;
use crate::init::RosSymInit;
use crate::zwfile::{RosSymZwReadFile, RosSymZwSeekFile};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
// (all imports from ddk moved to local definitions)
use core::ffi::c_void;

extern "system" {
    fn ExFreePool(P: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
pub const TAG_ROSSYM: u32 = 0x5253594D; // 'MYSR' in little-endian

unsafe extern "C" fn RosSymAllocMemKM(Size: usize) -> *mut core::ffi::c_void {
    ExAllocatePoolWithTag(NonPagedPool, Size, TAG_ROSSYM)
}

unsafe extern "C" fn RosSymFreeMemKM(Area: *mut core::ffi::c_void) {
    ExFreePool(Area);
}
#[no_mangle]
pub unsafe extern "C" fn RosSymInitKernelMode() {
    static mut KmCallbacks: ROSSYM_CALLBACKS = ROSSYM_CALLBACKS {
        AllocMemProc: Some(RosSymAllocMemKM),
        FreeMemProc: Some(RosSymFreeMemKM),
        ReadFileProc: Some(RosSymZwReadFile),
        SeekFileProc: Some(RosSymZwSeekFile),
        MemGetProc: None,
    };
    RosSymInit(core::ptr::addr_of_mut!(KmCallbacks));
}
