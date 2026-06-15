// Translated from win32ss/safe_user_io.c — SEH-free user-buffer validation.
//
// Pass-1 probe-site lowering shim: replaces the `_SEH2_TRY { ProbeFor*(...) }
// _SEH2_EXCEPT { Status = code }` idiom with a status-returning, validate-only
// check. Unlike the DDK ProbeForRead/ProbeForWrite macros these do NOT raise a
// software exception and do NOT touch-test the page (the touch in
// ntkrnl_sys::probe::probe_user_buffer is exactly what this must avoid) — under
// DUMMY_PSEH a raised exception or fault would crash the kernel uncaught.
//
// Identical range/alignment arithmetic to the DDK macros, minus exception/touch.
use win32k_sys::win32k::*;
use core::ffi::c_void;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002_u32 as NTSTATUS;
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;

// MmUserProbeAddress — highest user-mode address + 1 (DATA export from ntoskrnl).
// A plain extern is unsatisfiable for a data import; per-arch like class.rs:
// i386/arm64 raw-dylib +verbatim (symbol IS the value); amd64 binds __imp_ + deref.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_MmUserProbeAddress"]
    static MmUserProbeAddress_imp: *const ULONG_PTR;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    MmUserProbeAddress
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    *MmUserProbeAddress_imp
}

// Validate that [Address, Address+Length) lies entirely below MmUserProbeAddress
// and satisfies the requested alignment, without raising and without touching
// the page.
#[inline]
unsafe fn safep_validate_user_range(
    address: *const c_void,
    length: SIZE_T,
    alignment: ULONG,
) -> NTSTATUS {
    /* DDK ProbeFor* is a no-op for zero length */
    if length == 0 {
        return STATUS_SUCCESS;
    }

    /* Alignment is a byte count and must be a power of two (DDK contract) */
    if alignment > 1
        && (address as ULONG_PTR) & (alignment as ULONG_PTR - 1) != 0
    {
        return STATUS_DATATYPE_MISALIGNMENT;
    }

    let start = address as ULONG_PTR;
    let last = start.wrapping_add(length as ULONG_PTR).wrapping_sub(1);

    /* Address-space wrap, or any byte at/above the user probe boundary */
    if last < start || last >= mm_user_probe_address() {
        return STATUS_ACCESS_VIOLATION;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SafeProbeForRead(
    address: *const c_void,
    length: SIZE_T,
    alignment: ULONG,
) -> NTSTATUS {
    safep_validate_user_range(address, length, alignment)
}

#[no_mangle]
pub unsafe extern "system" fn SafeProbeForWrite(
    address: *mut c_void,
    length: SIZE_T,
    alignment: ULONG,
) -> NTSTATUS {
    safep_validate_user_range(address as *const c_void, length, alignment)
}
