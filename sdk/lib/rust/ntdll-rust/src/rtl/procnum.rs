//! `NtGetCurrentProcessorNumberEx` (Windows 7+). `ZwGetCurrentProcessorNumberEx` is the same
//! body: ntdll.spec aliases it here, as Windows exports both names at ONE address (the probe
//! checks `GetProcAddress` equality).
//!
//! On Windows this is a system service. Our kernel has none (ntoskrnl/sysfuncs.lst carries
//! only the plain `NtGetCurrentProcessorNumber`), so it is built here over the plain one, as
//! `NtOpenKeyEx` is built over `NtOpenKey` (rtl/openkeyex.rs). We have exactly one processor
//! group, so Group 0 plus the plain service's number is the whole answer — the same
//! reasoning rtl-strings' `RtlGetCurrentProcessorNumberEx` already ships on.
//!
//! Contract, measured black-box on Windows 10.0.26300 (the ntdll-gated-oracle harness):
//!   * returns an NTSTATUS (STATUS_SUCCESS), NOT the processor index that some headers
//!     (phnt) declare: pinned to CPU 1 it still returns 0 and writes Number = 1.
//!   * the argument is mandatory: NULL, the low 64K and kernel addresses give
//!     STATUS_ACCESS_VIOLATION (the kernel's probe), and no alignment is required (an odd
//!     address is written fine).
//!   * writes { Group = 0, Number, Reserved = 0 }.
//! Divergence: the kernel probe also turns an unmapped USER address into
//! STATUS_ACCESS_VIOLATION; without SEH here such a pointer faults in the caller instead.
//! We reject what can be rejected without touching memory: below the lowest user address
//! (0x10000) and, on 64-bit, above the highest one. (On 32-bit a large-address-aware process
//! may legitimately own memory above 2GB, so no upper bound is applied there.)

#![allow(non_snake_case)]

#[repr(C)]
pub struct PROCESSOR_NUMBER {
    pub Group: u16,
    pub Number: u8,
    pub Reserved: u8,
}

const STATUS_SUCCESS: i32 = 0;
const STATUS_ACCESS_VIOLATION: i32 = 0xC000_0005_u32 as i32;

/// MM_LOWEST_USER_ADDRESS: the first 64K of every address space is never mapped.
const LOWEST_USER_ADDRESS: usize = 0x1_0000;
/// MM_HIGHEST_USER_ADDRESS on 64-bit Windows (and ours): 0x7FFF'FFFE'FFFF.
#[cfg(target_pointer_width = "64")]
const HIGHEST_USER_ADDRESS: usize = 0x7FFF_FFFE_FFFF;

extern "system" {
    fn NtGetCurrentProcessorNumber() -> u32;
}

#[inline]
fn user_writable_range(p: usize) -> bool {
    let size = core::mem::size_of::<PROCESSOR_NUMBER>();
    if p < LOWEST_USER_ADDRESS {
        return false;
    }
    #[cfg(target_pointer_width = "64")]
    if p > HIGHEST_USER_ADDRESS + 1 - size {
        return false;
    }
    let _ = size;
    true
}

#[no_mangle]
pub unsafe extern "system" fn NtGetCurrentProcessorNumberEx(ProcessorNumber: *mut PROCESSOR_NUMBER) -> i32 {
    if !user_writable_range(ProcessorNumber as usize) {
        return STATUS_ACCESS_VIOLATION;
    }
    let value = PROCESSOR_NUMBER {
        Group: 0,
        Number: NtGetCurrentProcessorNumber() as u8,
        Reserved: 0,
    };
    // No alignment is promised (Windows writes an odd address happily).
    core::ptr::write_unaligned(ProcessorNumber, value);
    STATUS_SUCCESS
}
