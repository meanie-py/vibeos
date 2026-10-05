//! Exercises the shipping probe helpers. Linux supplies a fault-protected copy
//! through process_vm_readv; guest smoke tests exercise the NT implementation.
//! Run through tools/test_kernel_probe.py (no Windows bindgen host-ABI rewrite).
#![allow(dead_code, non_snake_case, non_camel_case_types)]

mod ddk {
    #[repr(C)]
    pub struct _LIST_ENTRY {
        pub Flink: *mut _LIST_ENTRY,
        pub Blink: *mut _LIST_ENTRY,
    }
    /// The NT raise has no handler here; a panic is what `should_panic` sees.
    pub unsafe fn ExRaiseAccessViolation() -> ! {
        panic!("ExRaiseAccessViolation");
    }
}
#[path = "../../sdk/lib/rust/ntkrnl-sys/src/base.rs"]
mod base;
#[path = "../../sdk/lib/rust/ntkrnl-sys/src/probe.rs"]
mod probe;

use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use probe::*;

#[repr(C)]
struct IoVec {
    base: *mut c_void,
    len: usize,
}
extern "C" {
    fn getpid() -> i32;
    fn process_vm_readv(
        pid: i32,
        local: *const IoVec,
        local_count: usize,
        remote: *const IoVec,
        remote_count: usize,
        flags: usize,
    ) -> isize;
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
}

#[no_mangle]
static MmUserProbeAddress: usize = usize::MAX / 2;

#[no_mangle]
unsafe extern "C" fn KmProtectedCopyMemory(
    dst: *mut c_void,
    src: *const c_void,
    len: usize,
) -> i32 {
    let local = IoVec { base: dst, len };
    let remote = IoVec {
        base: src as *mut c_void,
        len,
    };
    (process_vm_readv(getpid(), &local, 1, &remote, 1, 0) == len as isize) as i32
}

static DEREFERENCES: AtomicU32 = AtomicU32::new(0);
#[no_mangle]
unsafe extern "system" fn ObfDereferenceObject(_: *mut c_void) -> u32 {
    DEREFERENCES.fetch_add(1, Ordering::SeqCst);
    0
}

#[test]
fn valid_values_are_really_copied() {
    unsafe {
        let mut value = 17u32;
        assert_eq!(probe_read_user(&value, 1), Ok(17));
        assert_eq!(probe_write_user(&mut value, 42, 1), Ok(()));
        assert_eq!(value, 42);
        assert_eq!(probe_write_ulong(&mut value, 1), Ok(()));
        assert_eq!(value, 42);
    }
}

#[test]
fn inaccessible_user_addresses_return_errors() {
    unsafe {
        let invalid = 0x1000usize as *mut u32;
        assert_eq!(probe_read_user(invalid, 1), Err(STATUS_ACCESS_VIOLATION));
        assert_eq!(
            probe_write_user(invalid, 42, 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert_eq!(probe_write_ulong(invalid, 1), Err(STATUS_ACCESS_VIOLATION));
        assert_eq!(
            probe_write_boolean(invalid.cast(), 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert_eq!(
            probe_write_size_t(invalid.cast(), 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert!(matches!(
            probe_capture_unicode_string_header(invalid.cast(), 1),
            Err(STATUS_ACCESS_VIOLATION)
        ));
        // KernelMode skips the range probe, not fault recovery.
        assert_eq!(probe_read_user(invalid, 0), Err(STATUS_ACCESS_VIOLATION));
        assert_eq!(
            probe_write_user(invalid, 42, 0),
            Err(STATUS_ACCESS_VIOLATION)
        );
    }
}

#[test]
fn readonly_page_rejects_writes_but_allows_reads() {
    unsafe {
        // PROT_READ, MAP_PRIVATE | MAP_ANONYMOUS. Mapping starts zero-filled.
        let page = mmap(core::ptr::null_mut(), 4096, 1, 2 | 0x20, -1, 0);
        assert_ne!(page as usize, usize::MAX);
        assert_eq!(probe_read_user::<u32>(page.cast(), 1), Ok(0));
        assert_eq!(
            probe_write_user::<u32>(page.cast(), 42, 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert_eq!(
            probe_write_ulong(page.cast(), 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert_eq!(munmap(page, 4096), 0);
    }
}

#[test]
fn alignment_range_and_overflow_remain_checked() {
    unsafe {
        assert_eq!(
            probe_read_user::<u32>(0x1001 as _, 1),
            Err(STATUS_DATATYPE_MISALIGNMENT)
        );
        assert_eq!(
            probe_write_user::<u32>(0x1001 as _, 0, 1),
            Err(STATUS_DATATYPE_MISALIGNMENT)
        );
        assert_eq!(
            probe_read_user::<u64>((usize::MAX - 7) as _, 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
        assert_eq!(
            probe_write_user::<u64>((usize::MAX - 7) as _, 0, 1),
            Err(STATUS_ACCESS_VIOLATION)
        );
    }
}

#[test]
fn object_reference_is_released_after_capture_failure() {
    unsafe fn capture_then_return() -> Result<u32, i32> {
        let _guard = ObjRefGuard::new(0x2000 as _);
        probe_read_user(0x1000 as *const u32, 1)
    }
    assert_eq!(
        unsafe { capture_then_return() },
        Err(STATUS_ACCESS_VIOLATION)
    );
    assert_eq!(DEREFERENCES.load(Ordering::SeqCst), 1);
}

// probe_user_range: probe.h's typed-macro range check, which the Rust Probe*
// helpers once skipped (a user pointer at or above MmUserProbeAddress went
// straight through to the kernel's read or write).
#[test]
fn user_range_accepts_a_user_address() {
    let value = 0u64;
    unsafe { probe_user_range(&value) };
}

#[test]
#[should_panic(expected = "ExRaiseAccessViolation")]
fn user_range_rejects_a_kernel_address() {
    unsafe { probe_user_range((usize::MAX / 2) as *const u32) };
}

#[test]
#[should_panic(expected = "ExRaiseAccessViolation")]
fn user_range_rejects_a_value_straddling_the_boundary() {
    unsafe { probe_user_range((usize::MAX / 2 - 3) as *const u64) };
}

#[test]
#[should_panic(expected = "ExRaiseAccessViolation")]
fn user_range_rejects_wraparound() {
    unsafe { probe_user_range((usize::MAX - 1) as *const u32) };
}
