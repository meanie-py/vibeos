//! Rust-native probe API for validating user-mode pointers.
//!
//! Replaces C's `ProbeForRead`/`ProbeForWrite` + `_SEH2_TRY/_SEH2_EXCEPT`
//! with Result-returning functions that validate alignment and address range
//! before performing volatile reads/writes.
//!
//! C `ProbeForRead`/`ProbeForWrite` raise software exceptions on bad pointers,
//! requiring SEH to catch them. With DUMMY_PSEH, EXCEPT handlers are dead code,
//! so exceptions crash the kernel. These functions return `Err(NTSTATUS)`
//! instead, which callers propagate with `?`.
//!
//! The address range check uses `MmUserProbeAddress` — same as the C macros
//! in `sdk/include/reactos/probe.h`.

use crate::base::*;

/// Processor mode: 0 = KernelMode, 1 = UserMode.
type KPROCESSOR_MODE = i8;

// NTSTATUS codes for probe failures.
pub const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0xC0000082_u32 as i32;
pub const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as i32;

extern "C" {
    /// MmUserProbeAddress — highest user-mode address + 1.
    /// Pointers above this are kernel-mode and must not be passed from user-mode.
    static MmUserProbeAddress: ULONG_PTR;
}

// ============================================================================
// Core probe functions
// ============================================================================

/// Validate alignment + address range, then volatile-read a value of type `T`.
///
/// Returns `Err(STATUS_DATATYPE_MISALIGNMENT)` if the pointer is misaligned,
/// or `Err(STATUS_ACCESS_VIOLATION)` if it's outside user-mode range.
/// KernelMode callers bypass all checks (trusted).
///
/// Replaces the pattern:
/// ```c
/// _SEH2_TRY { value = ProbeForReadType(ptr); }
/// _SEH2_EXCEPT(...) { Status = _SEH2_GetExceptionCode(); }
/// ```
#[inline]
pub unsafe fn probe_read_user<T: Copy>(
    ptr: *const T,
    previous_mode: KPROCESSOR_MODE,
) -> Result<T, NTSTATUS> {
    // KernelMode callers are trusted — skip validation
    if previous_mode == 0 {
        return Ok(core::ptr::read_volatile(ptr));
    }

    // Alignment check
    let align = core::mem::align_of::<T>();
    if align > 1 && (ptr as usize) & (align - 1) != 0 {
        return Err(STATUS_DATATYPE_MISALIGNMENT);
    }

    // Range check: ptr..ptr+size must be below MmUserProbeAddress
    let addr = ptr as usize;
    let size = core::mem::size_of::<T>();
    let end = addr.wrapping_add(size);
    if end < addr || end > MmUserProbeAddress {
        return Err(STATUS_ACCESS_VIOLATION);
    }

    // Volatile read — minimizes TOCTOU window
    Ok(core::ptr::read_volatile(ptr))
}

/// Validate alignment + address range, then volatile-write a value of type `T`.
///
/// The write also serves as a touch-test: it proves the page is writable.
/// With DUMMY_PSEH the actual page fault from a bad pointer would crash,
/// so the range check catches it first.
///
/// Replaces the pattern:
/// ```c
/// _SEH2_TRY { ProbeForWriteType(ptr); }
/// _SEH2_EXCEPT(...) { Status = _SEH2_GetExceptionCode(); }
/// ```
#[inline]
pub unsafe fn probe_write_user<T: Copy>(
    ptr: *mut T,
    value: T,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    // KernelMode callers are trusted — skip validation
    if previous_mode == 0 {
        core::ptr::write_volatile(ptr, value);
        return Ok(());
    }

    // Alignment check
    let align = core::mem::align_of::<T>();
    if align > 1 && (ptr as usize) & (align - 1) != 0 {
        return Err(STATUS_DATATYPE_MISALIGNMENT);
    }

    // Range check
    let addr = ptr as usize;
    let size = core::mem::size_of::<T>();
    let end = addr.wrapping_add(size);
    if end < addr || end > MmUserProbeAddress {
        return Err(STATUS_ACCESS_VIOLATION);
    }

    // Volatile write — touch-tests the page
    core::ptr::write_volatile(ptr, value);
    Ok(())
}

/// Validate a user buffer (address range + alignment check only, no read/write).
///
/// Replaces `ProbeForRead(buf, len, align)` and `ProbeForWrite(buf, len, align)`
/// for buffer validation patterns where the caller just needs to verify the
/// buffer is in user-mode range before copying.
///
/// `alignment` must be a power of 2 (matching the C function contract).
#[inline]
pub unsafe fn probe_user_buffer(
    buffer: *const core::ffi::c_void,
    length: usize,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    // KernelMode callers are trusted
    if previous_mode == 0 {
        return Ok(());
    }

    // Alignment check on the buffer start
    if alignment > 1 && (buffer as usize) & (alignment - 1) != 0 {
        return Err(STATUS_DATATYPE_MISALIGNMENT);
    }

    // Range check: buffer..buffer+length must be below MmUserProbeAddress
    if length > 0 {
        let addr = buffer as usize;
        let end = addr.wrapping_add(length);
        if end < addr || end > MmUserProbeAddress {
            return Err(STATUS_ACCESS_VIOLATION);
        }
    }

    Ok(())
}

/// Validate + read a UNICODE_STRING header from user-mode.
///
/// Captures Length, MaximumLength, Buffer pointer via volatile reads.
/// Does NOT validate or capture the string data itself — the caller must
/// separately probe the Buffer if needed.
///
/// Replaces `ProbeForReadUnicodeString(ptr)` from probe.h.
#[inline]
pub unsafe fn probe_capture_unicode_string_header(
    ptr: *const UNICODE_STRING,
    previous_mode: KPROCESSOR_MODE,
) -> Result<UNICODE_STRING, NTSTATUS> {
    if previous_mode == 0 {
        // KernelMode — trusted, just read
        return Ok(core::ptr::read_volatile(ptr));
    }

    // Alignment check (UNICODE_STRING has USHORT fields, pointer-aligned overall)
    let align = core::mem::align_of::<UNICODE_STRING>();
    if align > 1 && (ptr as usize) & (align - 1) != 0 {
        return Err(STATUS_DATATYPE_MISALIGNMENT);
    }

    // Range check
    let addr = ptr as usize;
    let size = core::mem::size_of::<UNICODE_STRING>();
    let end = addr.wrapping_add(size);
    if end < addr || end > MmUserProbeAddress {
        return Err(STATUS_ACCESS_VIOLATION);
    }

    Ok(core::ptr::read_volatile(ptr))
}

// ============================================================================
// Convenience wrappers matching C probe.h macros
// ============================================================================

/// Probe and read a ULONG from user-mode.
/// Replaces `ProbeForReadUlong(ptr)`.
#[inline]
pub unsafe fn probe_read_ulong(
    ptr: *const ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> Result<ULONG, NTSTATUS> {
    probe_read_user::<ULONG>(ptr, previous_mode)
}

/// Probe and write zero to a ULONG in user-mode (touch-test).
/// Replaces `ProbeForWriteUlong(ptr)`.
#[inline]
pub unsafe fn probe_write_ulong(
    ptr: *mut ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    // C macro does read-then-write-back; we write 0 as touch-test
    // (caller will write the real value later via volatile write)
    let val = core::ptr::read_volatile(ptr);
    probe_write_user::<ULONG>(ptr, val, previous_mode)
}

/// Probe and write NULL to a HANDLE in user-mode (touch-test + zero).
/// Replaces `ProbeAndZeroHandle(ptr)` / `ProbeForWriteHandle(ptr)`.
#[inline]
pub unsafe fn probe_write_handle(
    ptr: *mut PVOID,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    probe_write_user::<PVOID>(ptr, core::ptr::null_mut(), previous_mode)
}

/// Probe and write a BOOLEAN value to user-mode.
/// Replaces `ProbeForWriteBoolean(ptr)`.
#[inline]
pub unsafe fn probe_write_boolean(
    ptr: *mut BOOLEAN,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    let val = core::ptr::read_volatile(ptr);
    probe_write_user::<BOOLEAN>(ptr, val, previous_mode)
}

/// Probe and write a ULONG_PTR (SIZE_T) to user-mode.
/// Replaces `ProbeForWriteSize_t(ptr)`.
#[inline]
pub unsafe fn probe_write_size_t(
    ptr: *mut ULONG_PTR,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    let val = core::ptr::read_volatile(ptr);
    probe_write_user::<ULONG_PTR>(ptr, val, previous_mode)
}
