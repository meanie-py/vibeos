//! Rust-native probe API for validating user-mode pointers.
//!
//! Replaces C's `ProbeForRead`/`ProbeForWrite` + `_SEH2_TRY/_SEH2_EXCEPT`
//! with Result-returning functions that validate alignment and address range
//! before performing fault-protected copies.
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

/// MmUserProbeAddress — highest user-mode address + 1. Pointers at or above it
/// are kernel-mode and must not be passed from user mode.
///
/// A data export of ntoskrnl, so how it binds depends on who links this crate
/// (feature `import_mm_user_probe_address`, see Cargo.toml).
#[cfg(not(feature = "import_mm_user_probe_address"))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    extern "C" {
        static MmUserProbeAddress: ULONG_PTR;
    }
    MmUserProbeAddress
}

/// Driver side: i386/arm64 bind through raw-dylib `+verbatim` (the symbol is the
/// value); amd64 binds `__imp_` and dereferences. Same as win32k's safe_user_io.
#[cfg(all(feature = "import_mm_user_probe_address", any(target_arch = "x86", target_arch = "aarch64")))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    #[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
    extern "C" {
        static MmUserProbeAddress: ULONG_PTR;
    }
    MmUserProbeAddress
}

#[cfg(all(feature = "import_mm_user_probe_address", not(any(target_arch = "x86", target_arch = "aarch64"))))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    extern "C" {
        #[link_name = "__imp_MmUserProbeAddress"]
        static MmUserProbeAddress_imp: *const ULONG_PTR;
    }
    *MmUserProbeAddress_imp
}

/// The range test every typed probe.h macro makes before it touches `Ptr`:
///
/// ```c
/// if ((ULONG_PTR)(Ptr) + sizeof(Type) - 1 < (ULONG_PTR)(Ptr) ||
///     (ULONG_PTR)(Ptr) + sizeof(Type) - 1 >= (ULONG_PTR)MmUserProbeAddress)
///     ExRaiseAccessViolation();
/// ```
///
/// The inline::common and macros.rs Probe* helpers shipped without it, so a
/// user-mode caller could hand a syscall a KERNEL address and have the kernel
/// read or write it. That stayed hidden partly because C shims
/// (io_rust_shims.c) carried the check and won the amd64/arm64 link by order for
/// two of the exported names; tools/link_guard.py surfaced it.
/// No alignment check: the typed macros have none.
#[inline]
pub unsafe fn probe_user_range<T>(ptr: *const T) {
    let first = ptr as ULONG_PTR;
    let last = first.wrapping_add(core::mem::size_of::<T>() - 1);
    if last < first || last >= mm_user_probe_address() {
        crate::ddk::ExRaiseAccessViolation();
    }
}

// ============================================================================
// Core probe functions
// ============================================================================

/// Validate alignment + address range, then fault-protected-copy a value of type `T`.
///
/// Returns `Err(STATUS_DATATYPE_MISALIGNMENT)` if the pointer is misaligned,
/// or `Err(STATUS_ACCESS_VIOLATION)` if it's outside user-mode range.
/// KernelMode callers bypass range/alignment checks, but retain fault protection.
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
    let mut value = core::mem::MaybeUninit::<T>::uninit();
    safe_copy_from_user(
        value.as_mut_ptr().cast(),
        ptr.cast(),
        core::mem::size_of::<T>(),
        core::mem::align_of::<T>(),
        previous_mode,
    )?;
    // Only a complete copy initializes T. A fault may have copied a prefix;
    // that prefix must never be interpreted as a value on the error path.
    Ok(value.assume_init())
}

/// Validate alignment + address range, then fault-protected-write a value of type `T`.
///
/// Unmapped and read-only pages return STATUS_ACCESS_VIOLATION. Range checks
/// alone cannot establish that a page is accessible; the copy catches faults.
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
    safe_copy_to_user(
        ptr.cast(),
        core::ptr::addr_of!(value).cast(),
        core::mem::size_of::<T>(),
        core::mem::align_of::<T>(),
        previous_mode,
    )
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
        if end < addr || end > mm_user_probe_address() {
            return Err(STATUS_ACCESS_VIOLATION);
        }
    }

    Ok(())
}

/// Validate a user buffer the kernel will WRITE: `ProbeForWrite(buf, len, align)`.
///
/// Range and alignment as `probe_user_buffer`, then — unlike it — touch every page with a
/// fault-protected read and write-back, as the exported ProbeForWrite does. A buffer that is
/// in range but unmapped or read-only (a NULL buffer with a non-zero length) fails here with
/// STATUS_ACCESS_VIOLATION, before the request reaches a driver. Translating ProbeForWrite to
/// the range-only `probe_user_buffer` let NtReadFile build an IRP for a NULL buffer.
///
/// A zero `length` checks nothing, exactly like ProbeForWrite (alignment included).
#[inline]
pub unsafe fn probe_user_buffer_write(
    buffer: *const core::ffi::c_void,
    length: usize,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    if previous_mode == 0 || length == 0 {
        return Ok(());
    }
    probe_user_buffer(buffer, length, alignment, previous_mode)?;
    let page = 0x1000usize;
    let last = (buffer as usize + length - 1) & !(page - 1);
    let mut current = buffer as usize;
    loop {
        let mut byte = 0u8;
        fault_tolerant_copy(core::ptr::addr_of_mut!(byte).cast(), current as *const core::ffi::c_void, 1)?;
        fault_tolerant_copy(current as *mut core::ffi::c_void, core::ptr::addr_of!(byte).cast(), 1)?;
        if current & !(page - 1) == last {
            return Ok(());
        }
        current = (current & !(page - 1)) + page;
    }
}

/// Validate + read a UNICODE_STRING header from user-mode.
///
/// Captures Length, MaximumLength, Buffer pointer via a protected copy.
/// Does NOT validate or capture the string data itself — the caller must
/// separately probe the Buffer if needed.
///
/// Replaces `ProbeForReadUnicodeString(ptr)` from probe.h.
#[inline]
pub unsafe fn probe_capture_unicode_string_header(
    ptr: *const UNICODE_STRING,
    previous_mode: KPROCESSOR_MODE,
) -> Result<UNICODE_STRING, NTSTATUS> {
    let mut value = core::mem::MaybeUninit::<UNICODE_STRING>::uninit();
    safe_copy_from_user(
        value.as_mut_ptr().cast(), ptr.cast(),
        core::mem::size_of::<UNICODE_STRING>(),
        core::mem::align_of::<UNICODE_STRING>(), previous_mode,
    )?;
    Ok(value.assume_init())
}

// ============================================================================
// Write-back helpers for SEH Pattern B (write computed result to user pointer)
// ============================================================================

/// Validate alignment + address range, then fault-protected-write a value of type `T`.
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
    // (the caller writes the real value later through another protected copy)
    let val = probe_read_user(ptr, previous_mode)?;
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

/// Faithful replacement for `ProbeForReadGenericType(Ptr, T, Default)`.
///
/// **The caller states the alignment the C states** — pass 1 for the typed
/// macros (`ProbeForReadLargeInteger`, …), which check none, and the C's
/// literal argument (usually `sizeof(ULONG)` = 4) when replacing an explicit
/// `ProbeForRead(p, len, align)`. It is a parameter rather than `align_of::<T>()`
/// precisely because those two differ: `align_of::<LARGE_INTEGER>()` is 8.
///
/// The typed C macro has no alignment check at all:
///
/// ```c
/// #define ProbeForReadGenericType(Ptr, Type, Default)                        \
///     (((ULONG_PTR)(Ptr) + sizeof(Type) - 1 < (ULONG_PTR)(Ptr) ||            \
///      (ULONG_PTR)(Ptr) + sizeof(Type) - 1 >= (ULONG_PTR)MmUserProbeAddress) \
///          ? ExRaiseAccessViolation(), Default                               \
///          : *(const volatile Type *)(Ptr))
/// ```
///
/// Use this, not `probe_read_user`, when replacing one of the typed macros
/// (`ProbeForReadLargeInteger`, …). `probe_read_user` derives alignment from
/// `align_of::<T>()`, which is 8 for `LARGE_INTEGER` and `PVOID` on 64-bit, so
/// it REJECTS addresses upstream accepts. Tightening a probe is a behaviour
/// change, not a hardening: it turns a working call into
/// `STATUS_DATATYPE_MISALIGNMENT`. The LPC translation already shipped that bug
/// once by passing `align_of::<UNICODE_STRING>()` where
/// `ProbeForReadUnicodeString` is documented range-only.
#[inline]
pub unsafe fn probe_read_generic<T: Copy>(
    ptr: *const T,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<T, NTSTATUS> {
    let mut value = core::mem::MaybeUninit::<T>::uninit();
    safe_copy_from_user(
        value.as_mut_ptr().cast(),
        ptr.cast(),
        core::mem::size_of::<T>(),
        alignment,
        previous_mode,
    )?;
    Ok(value.assume_init())
}

/// Faithful replacement for `ProbeForWriteGenericType(Ptr, T)` —
/// `ProbeForWriteHandle`, `ProbeForWritePointer`, `ProbeForWriteUlong`, …
///
/// Two things it gets right that `probe_write_handle` and friends above do not:
///
/// 1. **It PRESERVES the caller's value.** The C macro's touch-test is
///    `*(volatile Type *)(Ptr) = *(volatile Type *)(Ptr)` — a read-modify-
///    write-back. `probe_write_handle` writes `NULL` instead, which is
///    `ProbeAndZeroHandle`'s contract, a DIFFERENT macro. Zeroing an OUT
///    parameter that the C leaves intact is observable to any caller that
///    fails later and reads its own variable back.
/// 2. **Range check only, no alignment check** — see `probe_read_generic`.
///
/// The write-back is what proves the page is WRITABLE; a range check alone
/// cannot, which is the whole reason the C macro stores rather than just
/// comparing.
#[inline]
pub unsafe fn probe_write_generic<T: Copy>(
    ptr: *mut T,
    alignment: usize,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    let mut value = core::mem::MaybeUninit::<T>::uninit();
    safe_copy_from_user(
        value.as_mut_ptr().cast(),
        ptr.cast(),
        core::mem::size_of::<T>(),
        alignment,
        previous_mode,
    )?;
    safe_copy_to_user(
        ptr.cast(),
        value.as_ptr().cast(),
        core::mem::size_of::<T>(),
        alignment,
        previous_mode,
    )
}

/// Probe and write a BOOLEAN value to user-mode.
/// Replaces `ProbeForWriteBoolean(ptr)`.
#[inline]
pub unsafe fn probe_write_boolean(
    ptr: *mut BOOLEAN,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    let val = probe_read_user(ptr, previous_mode)?;
    probe_write_user::<BOOLEAN>(ptr, val, previous_mode)
}

/// Probe and write a ULONG_PTR (SIZE_T) to user-mode.
/// Replaces `ProbeForWriteSize_t(ptr)`.
#[inline]
pub unsafe fn probe_write_size_t(
    ptr: *mut ULONG_PTR,
    previous_mode: KPROCESSOR_MODE,
) -> Result<(), NTSTATUS> {
    let val = probe_read_user(ptr, previous_mode)?;
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
// i386: NOT the C shim — the extable fault-fixup primitive in
// ntoskrnl-rust ke/i386/safecopy.rs exports the same `KmProtectedCopyMemory`
// symbol/ABI. Same contract (1 = copied, 0 = faulted), same MmAccessFault-first
// semantics; no SEH involved at all. See that file for why.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
extern "C" {
    fn KmProtectedCopyMemory(
        dst: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        len: usize,
    ) -> i32;
}

// ntoskrnl-rust ARM3/mdlsup.rs: MmProbeAndLockPages returning its failure status
// instead of raising it. The raising NT export is a bugcheck wherever no SEH frame
// catches it, which is every Rust caller; `scan_raising_probes` gates the raising
// name. Exported by ntoskrnl.spec, so drivers outside the kernel image link it too.
extern "system" {
    pub fn MmTryProbeAndLockPages(
        Mdl: crate::ddk::PMDL,
        AccessMode: KPROCESSOR_MODE,
        Operation: crate::ddk::_LOCK_OPERATION,
    ) -> NTSTATUS;
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
/// **i386: PROTECTED (2026-07-24) — but by fault fixup, not SEH.** i386 took
/// option (b) of the two closes this comment used to list, because option (a)
/// (frame-based kernel SEH) is still broken there — see task #18 and the
/// 2026-07-17 spike. `ntoskrnl-rust ke/i386/safecopy.rs` exports the same
/// `KmProtectedCopyMemory` symbol as a `rep movsb` whose address is the single
/// key of a Linux-style exception table; `KiTrap0EHandler` consults it on a
/// kernel-mode fault MmAccessFault declined and redirects `Eip` to a fixup pad.
/// No exception dispatch, no registration chain, no unwinder. Same
/// MmAccessFault-first semantics as amd64/arm64 (a valid-but-nonresident page is
/// paged in and the copy proceeds), and the same per-boot `[KMSAFECOPY]` oracle.
///
/// The rejected non-fix stays rejected on every arch: an `MmIsAddressValid`
/// per-page pre-check cannot distinguish "invalid" from "valid but paged out", so
/// on a hot path (e.g. the IRP completion copy-back) it would spuriously fail
/// legitimate I/O with STATUS_ACCESS_VIOLATION under memory pressure — a
/// subtly-wrong degradation far worse than the crash it papers over.
///
/// `length` must be > 0 (callers early-return on 0).
#[inline]
unsafe fn fault_tolerant_copy(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    length: usize,
) -> Result<(), NTSTATUS> {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
    {
        if KmProtectedCopyMemory(dst, src, length) == 0 {
            return Err(STATUS_ACCESS_VIOLATION);
        }
        Ok(())
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
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

/// Store one value through a caller's pointer that was range-probed earlier: the
/// C's `_SEH2_TRY { *Ptr = Value; } _SEH2_EXCEPT { Status = code; }`. No probe and
/// no alignment check (the probe already ran, with the C's alignment); the store
/// itself survives an unmapped page.
#[inline]
pub unsafe fn write_back<T: Copy>(dst: *mut T, value: T) -> Result<(), NTSTATUS> {
    safe_copy_raw(
        dst.cast(),
        core::ptr::addr_of!(value).cast(),
        core::mem::size_of::<T>(),
    )
}

/// Load one value through a caller's pointer that was range-probed earlier: the
/// C's `_SEH2_TRY { Value = *Ptr; } _SEH2_EXCEPT { Status = code; }`. The mirror
/// of `write_back`: no probe and no alignment check, but the load survives a page
/// another thread unmapped after the probe. Each call is ONE fetch, so a value
/// that sizes or validates something must be read once and the copy used after.
#[inline]
pub unsafe fn read_back<T: Copy>(src: *const T) -> Result<T, NTSTATUS> {
    let mut value = core::mem::MaybeUninit::<T>::uninit();
    safe_copy_raw(
        value.as_mut_ptr().cast(),
        src.cast(),
        core::mem::size_of::<T>(),
    )?;
    Ok(value.assume_init())
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
    fn ExAllocatePoolWithTag(PoolType: i32, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
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

// ============================================================================
// Output bounce: query syscalls that fill a caller's buffer in place
// ============================================================================

const TAG_OUTPUT_BOUNCE: ULONG = u32::from_le_bytes(*b"PbOb");

/// A pool stand-in for a user-mode caller's output buffer.
///
/// The NtQuery* syscalls fill their output buffer IN PLACE inside one `_SEH2_TRY`,
/// often under a lock, so a user page that goes bad mid-query was the handler's
/// problem. With no SEH it is a bugcheck. Running the query against a pool copy
/// and copying it out once at the end covers every write in the body at once,
/// including the ones made by helpers (`RtlCopySidAndAttributesArray`, ...).
///
/// The buffer is copied IN first, so bytes the query leaves alone come back
/// unchanged. A class that stores a pointer INTO the buffer (a UNICODE_STRING
/// whose text follows it, a SID array) must move it to the caller's address with
/// [`OutputBounce::rebase`] before [`OutputBounce::finish`].
///
/// Kernel-mode callers (and empty buffers) are passed straight through.
pub struct OutputBounce {
    pool: PVOID,
    user: PVOID,
    length: usize,
}

impl OutputBounce {
    pub unsafe fn new(user: PVOID, length: usize, previous_mode: KPROCESSOR_MODE) -> Result<Self, NTSTATUS> {
        if previous_mode == 0 || length == 0 || user.is_null() {
            return Ok(Self { pool: core::ptr::null_mut(), user, length: 0 });
        }
        let pool = ExAllocatePoolWithTag(1 /* PagedPool */, length, TAG_OUTPUT_BOUNCE);
        if pool.is_null() {
            return Err(STATUS_INSUFFICIENT_RESOURCES);
        }
        if let Err(status) = fault_tolerant_copy(pool, user, length) {
            ExFreePoolWithTag(pool, TAG_OUTPUT_BOUNCE);
            return Err(status);
        }
        Ok(Self { pool, user, length })
    }

    /// The buffer the query should fill.
    pub fn buffer(&self) -> PVOID {
        if self.pool.is_null() { self.user } else { self.pool }
    }

    /// The mode to pass to a probe or write INTO the buffer: kernel once it is pool.
    pub fn mode(&self, previous_mode: KPROCESSOR_MODE) -> KPROCESSOR_MODE {
        if self.pool.is_null() { previous_mode } else { 0 }
    }

    /// Whether `size` bytes at `p` lie inside the pool copy.
    pub fn contains(&self, p: *const core::ffi::c_void, size: usize) -> bool {
        let base = self.pool as usize;
        let p = p as usize;
        !self.pool.is_null() && p >= base && p.wrapping_add(size) <= base + self.length && p.wrapping_add(size) >= p
    }

    /// Move a pointer that points into the pool copy to the same offset in the
    /// caller's buffer. Anything else is left alone.
    pub unsafe fn rebase<T>(&self, field: *mut *mut T) {
        if self.contains(field as *const _, core::mem::size_of::<*mut T>()) && self.contains(*field as *const _, 0) {
            *field = ((*field as usize) - (self.pool as usize) + (self.user as usize)) as *mut T;
        }
    }

    /// Copy the result out and release the pool copy. `Err` is a fault on the
    /// caller's buffer: the C handler's exception code.
    pub unsafe fn finish(mut self) -> Result<(), NTSTATUS> {
        if self.pool.is_null() {
            return Ok(());
        }
        let result = fault_tolerant_copy(self.user, self.pool, self.length);
        ExFreePoolWithTag(self.pool, TAG_OUTPUT_BOUNCE);
        self.pool = core::ptr::null_mut();
        result
    }
}

impl Drop for OutputBounce {
    fn drop(&mut self) {
        if !self.pool.is_null() {
            unsafe { ExFreePoolWithTag(self.pool, TAG_OUTPUT_BOUNCE) };
        }
    }
}
