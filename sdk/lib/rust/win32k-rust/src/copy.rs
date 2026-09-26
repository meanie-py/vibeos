use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;

// SafeProbeForRead/Write: NTAPI (extern "system"), Length = SIZE_T (canonical — icm.rs).
extern "system" {
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// The fault-tolerant kernel copy, imported from ntoskrnl (cdecl; ntoskrnl.spec).
// Returns 1 if the copy completed, 0 if it faulted. amd64/arm64: the native-SEH
// shim in ntoskrnl/kmsafecopy.c. i386: the exception-table fault-fixup primitive
// in ntoskrnl-rust ke/i386/safecopy.rs (field manual §9.123). All three arches
// carry the per-boot `[KMSAFECOPY] … (PASS)` oracle proving it actually catches.
extern "C" {
    fn KmProtectedCopyMemory(Dst: PVOID, Src: PVOID, Len: usize) -> i32;
}

const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;

// Callers reach these through the MmCopyFromCaller/MmCopyToCaller macros (= the
// `_`-prefixed names, cdecl). PAGED_CODE()/ASSERT(ExGetPreviousMode()==UserMode)
// are retail no-ops, dropped. extern "C" + #[no_mangle].
//
// WHY THE COPY IS FAULT-TOLERANT AND THE PROBE IS NOT (task #68):
// `SafeProbeFor*` is deliberately validate-ONLY — it range/alignment-checks the
// buffer against MmUserProbeAddress and does not touch the page, because a touch
// fault has no handler under USE_DUMMY_PSEH (see safe_user_io.rs). That leaves a
// hole a plain `RtlCopyMemory` walked straight into: a *partially* mapped user
// buffer passes the range check and then faults mid-copy, so ANY user process,
// 32- or 64-bit, could kill the box by passing one. The probe stays validate-only
// (a touch-test would still be uncaught, and it cannot tell "invalid" from "paged
// out" anyway); the COPY is what becomes fault-tolerant. MmAccessFault still runs
// first, so a valid-but-nonresident page pages in and the copy proceeds — only a
// genuinely bad address turns into STATUS_ACCESS_VIOLATION.
#[no_mangle]
pub unsafe extern "C" fn _MmCopyFromCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForRead(Source as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        if Bytes != 0 && KmProtectedCopyMemory(Target, Source, Bytes as usize) == 0 {
            return STATUS_ACCESS_VIOLATION;
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "C" fn _MmCopyToCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForWrite(Target as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        if Bytes != 0 && KmProtectedCopyMemory(Target, Source, Bytes as usize) == 0 {
            return STATUS_ACCESS_VIOLATION;
        }
    }
    Status
}

/// Fault-tolerant capture of a user buffer that has ALREADY been probed.
///
/// The gap this closes: `SafeProbeFor*` is validate-only by design (see the
/// note above and `safe_user_io.rs`) -- it range/alignment-checks against
/// `MmUserProbeAddress` and never touches the page, because under
/// `USE_DUMMY_PSEH` a touch fault has no handler. The C wrapped probe AND the
/// access in one `_SEH2_TRY`, so a translation that keeps the probe and drops
/// the handler reads as protected and is not: a NULL or merely-unmapped
/// pointer passes the probe and faults on the first real load.
///
/// Measured, not theorised: `NtUserScrollDC` and `NtGdiCreateDIBSection` both
/// BUGCHECK 0x1E this way, at the exact load following their own probe.
///
/// Keep the probe -- it is what rejects a kernel-range pointer, which no copy
/// primitive checks -- and route the ACCESS through here. Returns false if the
/// copy faulted; map that to `STATUS_ACCESS_VIOLATION` and the C's error path.
#[inline]
pub unsafe fn capture_from_user(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    bytes: usize,
) -> bool {
    bytes == 0 || KmProtectedCopyMemory(dst, src as PVOID, bytes) != 0
}

/// The WRITE direction of `capture_from_user`, and just as necessary.
///
/// `SafeProbeForWrite` has exactly the same limitation as the read probe: it
/// checks the range, never the mapping. Storing through a probed-but-unmapped
/// user pointer faults with nothing to catch it, the same as loading from one.
/// Easy to miss when fixing a function, because the reads are at the top and
/// the write-back is at the bottom -- `NtUserScrollDC` still BUGCHECKed 0x1E
/// after both its reads were fixed, moving the fault from +0x180 to +0x1C0.
#[inline]
pub unsafe fn publish_to_user(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    bytes: usize,
) -> bool {
    bytes == 0 || KmProtectedCopyMemory(dst, src as PVOID, bytes) != 0
}

/// Zero a user buffer fault-tolerantly. `RtlZeroMemory` straight into user memory
/// is the same hazard as a raw store: it is a plain `memset` with no fault handler,
/// so a partially-mapped buffer that passed the validate-only probe bugchecks
/// mid-clear. Chunked from a small kernel zero block, so an unbounded caller-supplied
/// size needs no matching kernel allocation.
#[inline]
pub unsafe fn zero_to_user(dst: *mut core::ffi::c_void, bytes: usize) -> bool {
    const ZERO_CHUNK: usize = 256;
    let zeros = [0u8; ZERO_CHUNK];
    let mut done = 0usize;
    while done < bytes {
        let n = core::cmp::min(ZERO_CHUNK, bytes - done);
        if KmProtectedCopyMemory((dst as *mut u8).add(done) as PVOID, zeros.as_ptr() as PVOID, n)
            == 0
        {
            return false;
        }
        done += n;
    }
    true
}

// ---------------------------------------------------------------------------
// TYPED wrappers, for the shapes a byte-oriented helper cannot express.
//
// `capture_from_user`/`publish_to_user` take `*mut c_void` + a byte count,
// which fits `RtlCopyMemory` exactly but not the two other unprotected shapes
// the scanner finds. `core::ptr::copy_nonoverlapping(src, dst, count)` counts
// ELEMENTS, and `core::ptr::write_volatile(p, v)` has no length at all -- both
// carry their size in the pointee type, so the wrapper has to be generic to
// get it. Being generic also means the call sites need no casts, which is what
// makes the rewrite mechanical: the `as *mut _` a cast-taking helper would
// force has no type to infer from.
//
// Same contract as the byte helpers: the probe stays (it is what rejects a
// kernel-range pointer), the ACCESS becomes fault-tolerant, and `false` means
// the copy faulted -- map it to STATUS_ACCESS_VIOLATION and the C's error path.

/// `core::ptr::copy_nonoverlapping(src, dst, count)`, read direction, protected.
/// Note the argument order is dst-first here, matching the other helpers rather
/// than `copy_nonoverlapping`'s src-first.
#[inline]
pub unsafe fn capture_n<T>(dst: *mut T, src: *const T, count: usize) -> bool {
    let bytes = count * core::mem::size_of::<T>();
    bytes == 0 || KmProtectedCopyMemory(dst as PVOID, src as *mut T as PVOID, bytes) != 0
}

/// `core::ptr::copy_nonoverlapping(src, dst, count)`, write direction, protected.
#[inline]
pub unsafe fn publish_n<T>(dst: *mut T, src: *const T, count: usize) -> bool {
    capture_n(dst, src, count)
}

/// `core::ptr::write_volatile(dst, val)` into user memory, protected.
///
/// The store is the whole hazard: a probed-but-unmapped out-parameter is the
/// single commonest write-direction instance of this class, because it looks
/// so much smaller than a struct copy that it reads as safe.
#[inline]
pub unsafe fn write_user<T>(dst: *mut T, val: T) -> bool {
    KmProtectedCopyMemory(
        dst as PVOID,
        core::ptr::addr_of!(val) as *mut T as PVOID,
        core::mem::size_of::<T>(),
    ) != 0
}
