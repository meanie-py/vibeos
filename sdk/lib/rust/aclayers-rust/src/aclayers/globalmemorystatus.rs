use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS: flag-only macro with no value, nothing to emit.

pub type GLOBALMEMORYSTATUSPROC = unsafe extern "system" fn(lpBuffer: LPMEMORYSTATUS);

// SHIM_NS GlobalMemoryStatus2GB: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(GlobalMemoryStatus2GB);

pub unsafe extern "system" fn APIHook_GlobalMemoryStatus(lpBuffer: LPMEMORYSTATUS) {
    CALL_SHIM!(0, GLOBALMEMORYSTATUSPROC)(lpBuffer);

    if (*lpBuffer).dwTotalPhys > 0x3FFFFFFF {
        (*lpBuffer).dwTotalPhys = 0x3FFFFFFF;
    }

    if (*lpBuffer).dwAvailPhys > 0x3FFFFFFF {
        (*lpBuffer).dwAvailPhys = 0x3FFFFFFF;
    }

    if (*lpBuffer).dwTotalPageFile > 0x7FFFFFFF {
        (*lpBuffer).dwTotalPageFile = 0x7FFFFFFF;
    }

    if (*lpBuffer).dwAvailPageFile > 0x3FFFFFFF {
        (*lpBuffer).dwAvailPageFile = 0x3FFFFFFF;
    }
}

// SHIM_NUM_HOOKS 1, no SHIM_NOTIFY_FN
implement_shim!(GlobalMemoryStatus2GB, 1usize, None, [
    (0, "KERNEL32.DLL", "GlobalMemoryStatus", APIHook_GlobalMemoryStatus),
]);
