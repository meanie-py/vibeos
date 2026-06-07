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

const TAG_ROSSYM: u32 = u32::from_ne_bytes(*b"MYSR");

unsafe extern "system" {
    fn RosSymInit(KmCallbacks: *const ROSSYM_CALLBACKS);
}

unsafe fn RosSymAllocMemKM(Size: usize) -> *mut core::ffi::c_void {
    ExAllocatePoolWithTag(NonPagedPool, Size, TAG_ROSSYM)
}

unsafe fn RosSymFreeMemKM(Area: *mut core::ffi::c_void) {
    ExFreePool(Area);
}
#[no_mangle]
pub unsafe extern "C" fn RosSymInitKernelMode() {
    static KmCallbacks: ROSSYM_CALLBACKS = ROSSYM_CALLBACKS {
        AllocMem: RosSymAllocMemKM,
        FreeMem: RosSymFreeMemKM,
        ReadFile: RosSymZwReadFile,
        SeekFile: RosSymZwSeekFile,
    };
    RosSymInit(&KmCallbacks as *const ROSSYM_CALLBACKS);
}
