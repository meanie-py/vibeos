//! ksecdd glue — c2rust phantom-extern macros + substrate import bridges +
//! cryptlib-private struct layouts not present in the bindgen types.rs.
//! Filled during integration/compile-fix (fltmgr/npfs pattern).
#![allow(unused_imports)]

pub use crate::types::*;

use ntkrnl_sys::ddk::{UNICODE_STRING, PSECURITY_STRING};

// PSSPI_SEC_STRING — `#define PSSPI_SEC_STRING PSECURITY_STRING` (sdk/include/xdk/sspi.h).
// PSECURITY_STRING = *mut UNICODE_STRING (bindgen). Not a typedef, so absent from ddk.
pub type PSSPI_SEC_STRING = PSECURITY_STRING;

// MD5_CTX / MD4_CTX — cryptlib-private structs (sdk/lib/cryptlib/{md5,md4}.h).
// Not emitted by bindgen (driver pulls them from the cryptlib headers directly).
// `in` is a Rust keyword → c2rust renames the field `in_0`; the static initializers
// in crypt.rs use `in_0`, so the field MUST be named `in_0` here.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MD5_CTX {
    pub i: [::core::ffi::c_uint; 2],
    pub buf: [::core::ffi::c_uint; 4],
    pub in_0: [::core::ffi::c_uchar; 64],
    pub digest: [::core::ffi::c_uchar; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MD4_CTX {
    pub buf: [::core::ffi::c_uint; 4],
    pub i: [::core::ffi::c_uint; 2],
    pub in_0: [::core::ffi::c_uchar; 64],
    pub digest: [::core::ffi::c_uchar; 16],
}

// Bare enum-member aliases — bindgen names enum members `_ENUM_Member`; the
// translation references the bare `Member`. Not re-exported by ddk/driver_ddk
// for these classes, so alias them here to the exact bindgen consts.
pub use ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as FileStandardInformation;
pub use ntkrnl_sys::ddk::_FSINFOCLASS_FileFsDeviceInformation as FileFsDeviceInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemPerformanceInformation as SystemPerformanceInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemProcessInformation as SystemProcessInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemProcessorPerformanceInformation as SystemProcessorPerformanceInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemInterruptInformation as SystemInterruptInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemExceptionInformation as SystemExceptionInformation;
pub use ntkrnl_sys::ddk::_SYSTEM_INFORMATION_CLASS_SystemLookasideInformation as SystemLookasideInformation;

// PsGetCurrentProcess — macro forwarding to the exported IoGetCurrentProcess.
// ntkrnl_sys::ps::PsGetCurrentProcess returns the OPAQUE ps::PEPROCESS (`[u8;0]`),
// but crypt.rs dereferences `.UniqueProcessId` / `.Pcb.DirectoryTableBase`, which
// require the full bindgen `_EPROCESS`. Re-declare against the ddk PEPROCESS so
// the field accesses resolve (same `IoGetCurrentProcess` symbol).
#[inline]
pub unsafe fn PsGetCurrentProcess() -> ntkrnl_sys::ddk::PEPROCESS {
    extern "system" {
        #[link_name = "IoGetCurrentProcess"]
        fn IoGetCurrentProcess() -> ntkrnl_sys::ddk::PEPROCESS;
    }
    IoGetCurrentProcess()
}
// RtlCopyMemory / RtlZeroMemory — RtlpfuncS macros routed to the rtl module impls.
pub use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

// KeQueryTickCount / SharedSystemTime — KUSER_SHARED_DATA macros (xdk/<arch>/ke.h).
// The translation took the amd64/arm64 macro form: KeQueryTickCount reads the
// 64-bit SharedTickCount (KI_USER_SHARED_DATA + 0x320); KeQuerySystemTime was
// inlined by the translator as `*dst = *(SharedSystemTime as *mut u64)`, where
// SharedSystemTime = KI_USER_SHARED_DATA + 0x14. KUSER_SHARED_DATA has the same
// layout on all 3 arches, so route both through ntkrnl_sys::mm::KI_USER_SHARED_DATA
// (the per-arch base const). Provided as glue so the symbols resolve.
// KI_USER_SHARED_DATA is ntkrnl_sys::mm::ULONG_PTR (= usize); keep that width so
// the `+ offset` arithmetic and the later `as *mut/* const` casts type-check.
pub const SharedSystemTime: usize =
    ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize + 0x14;

#[inline]
pub unsafe fn KeQueryTickCount(current_count: *mut i64) {
    const SHARED_TICK_COUNT: usize =
        ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize + 0x320;
    *current_count =
        core::ptr::read_volatile(SHARED_TICK_COUNT as *const u64) as i64;
}

// RtlSecureZeroMemory — FORCEINLINE (xdk/rtlfuncs.h), NOT an ntoskrnl export; the
// bindgen ddk decl is a never-linking phantom (trap #3 class). Provide the faithful
// inline; callers import this explicitly to shadow the ddk::* glob.
#[inline]
pub unsafe fn RtlSecureZeroMemory(
    pointer: ntkrnl_sys::ddk::PVOID,
    size: ntkrnl_sys::ddk::SIZE_T,
) -> ntkrnl_sys::ddk::PVOID {
    let mut vptr = pointer as *mut core::ffi::c_char;
    let endptr = (vptr as usize).wrapping_add(size as usize) as *mut core::ffi::c_char;
    while vptr < endptr {
        core::ptr::write_volatile(vptr, 0);
        vptr = vptr.wrapping_add(1);
    }
    pointer
}

// RtlAssert is ntoskrnl's export: it prints the failed assertion and returns. An
// empty local definition here won the link and made every failed ASSERT in this
// driver vanish (scan_export_shadows).
extern "system" {
    pub fn RtlAssert(FailedAssertion: ntkrnl_sys::ddk::PVOID, FileName: ntkrnl_sys::ddk::PVOID, LineNumber: ntkrnl_sys::ddk::ULONG, Message: *mut ntkrnl_sys::ddk::CHAR);
}

// _rotl — MSVC rotate-left intrinsic (crypto). `value.rotate_left` reduces the
// shift mod 32 exactly like the intrinsic.
#[inline]
pub unsafe fn _rotl(value: u32, shift: i32) -> u32 {
    value.rotate_left(shift as u32)
}

// --- compiler intrinsics / debug helpers not provided as symbols ---

// __rdtsc — x86 timestamp counter (entropy source). No arm64 equivalent; the C
// guards it per-arch, but the translation dropped the guard -> cfg-gate here.
// arm64 reads the generic timer CNTVCT_EL0 (the faithful high-res counter analogue).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub unsafe fn __rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
    ((hi as u64) << 32) | (lo as u64)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn __rdtsc() -> u64 {
    let v: u64;
    core::arch::asm!("mrs {}, cntvct_el0", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}

// DbgBreakPoint — a software breakpoint (int3 / brk). Faithful to the ntoskrnl
// export; defined locally so the cohab .sys doesn't depend on its decoration.
#[inline]
pub unsafe fn DbgBreakPoint() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000");
}
