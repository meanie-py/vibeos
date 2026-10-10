//! msys2.c -- amd64 only (CMakeLists: `if(ARCH STREQUAL "amd64") list(APPEND SOURCE msys2.c)`).
//! Hand-written: the per-function translator never saw this TU.
use crate::prelude::*;
use core::ffi::c_void;

// compat_undoc.h ReactOS_ShimData, amd64 layout (the `#ifdef _M_AMD64` decoy member is present).
// The bindgen island carries no copy: compat_undoc.h is outside the other TUs' include closure.
#[repr(C)]
pub struct ReactOS_ShimData {
    pub dwReserved1: [DWORD; 130],
    pub dwSize: DWORD,
    pub dwMagic: DWORD,
    pub dwReserved2: [DWORD; 242],
    pub dwRosProcessCompatVersion: DWORD,
    pub RtlGetCurrentDirectory_U_RtlpMsysDecoy: PVOID,
}

pub const REACTOS_SHIMDATA_MAGIC: DWORD = 0xAC0DEDAB;

static mut RtlGetCurrentDirectory_U_RtlpMsysDecoy: PVOID = core::ptr::null_mut();

// SHIM_NS MsysDecoy: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(MsysDecoy);

pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    if fdwReason == SHIM_NOTIFY_ATTACH {
        let pShimData: *mut ReactOS_ShimData = (*ntdll_sys::nt::NtCurrentPeb()).pShimData as *mut ReactOS_ShimData;
        if !pShimData.is_null() && (*pShimData).dwMagic == REACTOS_SHIMDATA_MAGIC {
            /* Grab the private function from ntdll */
            RtlGetCurrentDirectory_U_RtlpMsysDecoy = (*pShimData).RtlGetCurrentDirectory_U_RtlpMsysDecoy;
            SHIM_MSG!(
                "RtlGetCurrentDirectory_U_RtlpMsysDecoy=%p\n",
                RtlGetCurrentDirectory_U_RtlpMsysDecoy
            );
            return (!RtlGetCurrentDirectory_U_RtlpMsysDecoy.is_null()) as BOOL;
        }
        SHIM_FAIL!("Invalid pShimData @ %p\n", pShimData);
        /* Returning false here will not register the hooks */
        return FALSE as BOOL;
    }
    return TRUE as BOOL;
}

// SHIM_NOTIFY_FN = SHIM_OBJ_NAME(Notify); SHIM_NUM_HOOKS 1. The replacement is the PVOID the
// Notify above fetched from ntdll's shim data, read when GetHookAPIs fills the table.
implement_shim!(MsysDecoy, 1usize, Some(Notify), [
    (0, "NTDLL.DLL", "RtlGetCurrentDirectory_U", RtlGetCurrentDirectory_U_RtlpMsysDecoy),
]);
