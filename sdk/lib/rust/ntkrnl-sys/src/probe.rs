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
pub const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002_u32 as i32;
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
// Write-back helpers for SEH Pattern B (write computed result to user pointer)
// ============================================================================

/// Validate alignment + address range, then volatile-write a value of type `T`.
///
/// This is an alias for `probe_write_user` — the name makes the intent clearer
/// in syscall implementations where a computed result is written back to the caller.
///
/// Replaces Pattern B:
/// ```c
/// _SEH2_TRY { *OutputPtr = ComputedValue; }
/// _SEH2_EXCEPT(ExSystemExceptionFilter()) { Status = _SEH2_GetExceptionCode(); }
/// ```
///
/// Usage: `write_user(output_ptr, computed_value, previous_mode)?;`
#[inline]
pub unsafe fn write_user<T: Copy>(
    ptr: *mut T,
    value: T,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    probe_write_user(ptr, value, previous_mode)
}

/// Write a value to a user-mode pointer, ignoring any validation failure.
///
/// Replaces the pattern where the EXCEPT handler is `(void)0` (silently swallow):
/// ```c
/// _SEH2_TRY { *OutputPtr = ComputedValue; }
/// _SEH2_EXCEPT(ExSystemExceptionFilter()) { (void)0; }
/// ```
///
/// Used in timer.c where write-back failure is non-fatal.
#[inline]
pub unsafe fn write_user_nofail<T: Copy>(
    ptr: *mut T,
    value: T,
    previous_mode: KPROCESSOR_MODE,
) {
    let _ = probe_write_user(ptr, value, previous_mode);
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

// ============================================================================
// Safe copy to/from user-mode
// ============================================================================

// amd64 + arm64: the fault-tolerant copy is a native-SEH C shim
// (ntoskrnl/kmsafecopy.c), the validated table-based-SEH path (seh_poc.c PASS +
// __C_specific_handler in the link). Returns 1 on success, 0 if the copy faulted.
// arm64 added 2026-07-17: the arm64 kernel-mode HARDWARE-fault SEH path was
// validated end-to-end (KmSafeCopySelfTest PASS after the .xdata save_next decoder
// fix, §9.69/§9.93) — a real page fault in a callee now unwinds into the __except.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
extern "C" {
    fn KmProtectedCopyMemory(
        dst: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        len: usize,
    ) -> i32;
}

/// Copy `length` bytes, returning `Err(STATUS_ACCESS_VIOLATION)` instead of
/// BUGCHECKing when the source or destination faults.
///
/// This is the primitive the DUMMY_PSEH world lost: in C the capture memcpy sat
/// inside `_SEH2_TRY`, but under `USE_DUMMY_PSEH` that handler is dead and the
/// Rust translations dropped it, so a hostile pointer crashed the kernel (the
/// memcpy+0x40 cluster: ObSymbolicLink / ZwWaitForMultipleObjects / IoReadWrite).
///
/// **amd64**: routes through the native-SEH shim `KmProtectedCopyMemory`
/// (ntoskrnl/kmsafecopy.c). A genuine hardware-fault catch — and because the
/// fault dispatches through `MmAccessFault` FIRST, a valid-but-nonresident page
/// is simply paged in and the copy proceeds; only a truly invalid address reaches
/// the `__except`. Validated 2026-07-12: ObSymbolicLink + ZwWaitForMultipleObjects
/// went CRASH -> OK.
///
/// **arm64: PROTECTED (2026-07-17)** — routes through `KmProtectedCopyMemory` like
/// amd64. The arm64 kernel-mode hardware-fault SEH path was validated end-to-end
/// after the `.xdata` save_next decoder fix (§9.69/§9.93): `KmSafeCopySelfTest`
/// now catches a genuine page fault at an unmapped VA (`invalid_caught=1`), where
/// before it wedged the boot. Same MmAccessFault-first semantics as amd64.
///
/// **i386: NO PROTECTION — this is a plain copy (status quo).** Stated
/// plainly rather than faked, per the fail-loud rule. The obvious stand-in,
/// an `MmIsAddressValid` per-page pre-check, is NOT a valid substitute: it cannot
/// distinguish "invalid" from "valid but paged out", so on a hot path (e.g. the
/// IRP completion copy-back) it would spuriously fail legitimate I/O with
/// STATUS_ACCESS_VIOLATION under memory pressure — a subtly-wrong degradation far
/// worse than the crash it papers over. Closing the i386 gap needs either
/// (a) validating clang's frame-based i386 kernel SEH the way seh_poc.c validated
/// amd64/arm64, or (b) a Linux-extable-style fault-fixup copy primitive the
/// page-fault handler consults. OWED; the i386 equivalents stay quarantined.
///
/// `length` must be > 0 (callers early-return on 0).
#[inline]
unsafe fn fault_tolerant_copy(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    length: usize,
) -> Result<(), NTSTATUS> {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        if KmProtectedCopyMemory(dst, src, length) == 0 {
            return Err(STATUS_ACCESS_VIOLATION);
        }
        Ok(())
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, length);
        Ok(())
    }
}

/// Fault-tolerant copy with NO probing — the direct replacement for a bare
/// `_SEH2_TRY { RtlCopyMemory(..) } _SEH2_EXCEPT { .. }` at sites where the C did
/// no probe because both pointers are already-validated/kernel-owned, yet the copy
/// still had to survive a bad one (e.g. `IopCompleteRequest`'s buffered copy-back
/// to `Irp->UserBuffer`, irp.c:292 — the caller's buffer may be NULL/hostile).
///
/// See `fault_tolerant_copy` for the per-arch protection guarantee.
#[inline]
pub unsafe fn safe_copy_raw(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    length: usize,
) -> Result<(), NTSTATUS> {
    if length == 0 {
        return Ok(());
    }
    fault_tolerant_copy(dst, src, length)
}

/// Validate a user-mode source buffer, then fault-tolerantly copy `length` bytes
/// to a kernel buffer.
///
/// Replaces the pattern:
/// ```c
/// _SEH2_TRY {
///     ProbeForRead(UserBuffer, Length, Alignment);
///     RtlCopyMemory(KernelBuffer, UserBuffer, Length);
/// } _SEH2_EXCEPT(...) { Status = _SEH2_GetExceptionCode(); }
/// ```
///
/// The range/alignment probe is skipped for KernelMode callers (a kernel-range
/// source is legitimate), but the copy itself is ALWAYS fault-protected — the C
/// `_SEH2_TRY` wrapped it unconditionally, and kernel-mode callers (e.g. the
/// kmtest drivers) still pass hostile pointers that must return an error rather
/// than crash.
#[inline]
pub unsafe fn safe_copy_from_user(
    kernel_dst: *mut core::ffi::c_void,
    user_src: *const core::ffi::c_void,
    length: usize,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    if length == 0 {
        return Ok(());
    }
    probe_user_buffer(user_src, length, alignment, previous_mode)?;
    fault_tolerant_copy(kernel_dst, user_src, length)
}

/// Validate a user-mode destination buffer, then fault-tolerantly copy `length`
/// bytes from a kernel buffer.
///
/// Replaces the pattern:
/// ```c
/// _SEH2_TRY {
///     ProbeForWrite(UserBuffer, Length, Alignment);
///     RtlCopyMemory(UserBuffer, KernelBuffer, Length);
/// } _SEH2_EXCEPT(...) { Status = _SEH2_GetExceptionCode(); }
/// ```
///
/// Copy is always fault-protected (see `safe_copy_from_user`).
#[inline]
pub unsafe fn safe_copy_to_user(
    user_dst: *mut core::ffi::c_void,
    kernel_src: *const core::ffi::c_void,
    length: usize,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    if length == 0 {
        return Ok(());
    }
    probe_user_buffer(user_dst, length, alignment, previous_mode)?;
    fault_tolerant_copy(user_dst, kernel_src, length)
}

// ============================================================================
// RAII guards for SEH cleanup patterns
// ============================================================================
//
// These replace _SEH2_EXCEPT handlers that call Free/Dereference/Release.
// The guard's Drop impl runs on any exit path (normal return, ? early return).
//
// Classifier data: 27 ObDereferenceObject, 20 ExFreePool*, 14 IopCleanup,
// 2 IoFreeIrp, 1 KeReleaseMutex across 52 CLEANUP handlers kernel-wide.

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfDereferenceObject(Object: PVOID) -> ULONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ObfDereferenceObject(Object: PVOID) -> ULONG;
}
extern "system" {
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

/// RAII guard that calls `ObDereferenceObject` on drop.
///
/// Replaces the pattern:
/// ```c
/// _SEH2_EXCEPT(...) { ObDereferenceObject(Object); ... }
/// ```
///
/// Use `core::mem::forget(guard)` to prevent the dereference (e.g., on success
/// when ownership is transferred to caller).
pub struct ObjRefGuard {
    object: PVOID,
}

impl ObjRefGuard {
    /// Create a guard that will dereference `object` when dropped.
    #[inline]
    pub unsafe fn new(object: PVOID) -> Self {
        Self { object }
    }

    /// Consume the guard without calling ObDereferenceObject.
    /// Use when ownership is successfully transferred.
    #[inline]
    pub fn disarm(self) {
        core::mem::forget(self);
    }
}

impl Drop for ObjRefGuard {
    fn drop(&mut self) {
        if !self.object.is_null() {
            unsafe { ObfDereferenceObject(self.object); }
        }
    }
}

/// RAII guard that calls `ExFreePoolWithTag` on drop.
///
/// Replaces the pattern:
/// ```c
/// _SEH2_EXCEPT(...) { ExFreePoolWithTag(Buffer, Tag); ... }
/// ```
pub struct PoolAllocGuard {
    ptr: PVOID,
    tag: ULONG,
}

impl PoolAllocGuard {
    /// Create a guard that will free `ptr` with `tag` when dropped.
    #[inline]
    pub unsafe fn new(ptr: PVOID, tag: ULONG) -> Self {
        Self { ptr, tag }
    }

    /// Consume the guard without freeing the pool allocation.
    /// Use when ownership is successfully transferred.
    #[inline]
    pub fn disarm(self) {
        core::mem::forget(self);
    }
}

impl Drop for PoolAllocGuard {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ExFreePoolWithTag(self.ptr, self.tag); }
        }
    }
}
