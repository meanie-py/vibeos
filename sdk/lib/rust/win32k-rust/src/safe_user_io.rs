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
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as NTSTATUS;

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

// ---------------------------------------------------------------------------
// Result-returning replacements for the RAISING probes, for crate-internal use.
//
// C win32k wraps `ProbeForRead`/`ProbeForWrite` (and the probe.h typed macros
// `ProbeForReadUnicodeString`, `ProbeForReadLargeString`, ...) in
// `_SEH2_TRY { .. } _SEH2_EXCEPT { handler }`. This kernel has no SEH, so a
// raise there is an unhandled exception (BUGCHECK 0x1E) where the C would have
// run its handler. These return `Err(status)` instead, so each site can run the
// C handler's body explicitly.
//
// Why not `ntkrnl_sys::probe::{probe_user_buffer, safe_copy_from_user, ...}`:
// those read `MmUserProbeAddress` through a plain extern, which a separate
// driver cannot link (a DATA import; arm64 failed with `undefined symbol:
// MmUserProbeAddress`, see accelerator.rs). The range check here goes through
// the per-arch binding above; the copy goes through `safe_copy_raw`
// (KmProtectedCopyMemory, a function export), which is fault-tolerant.
//
// The range check is unconditional (no PreviousMode), exactly like the C
// ProbeForRead/ProbeForWrite exports and the probe.h typed macros.
// ---------------------------------------------------------------------------

#[inline]
fn nt_result(st: NTSTATUS) -> Result<(), NTSTATUS> {
    if st < 0 { Err(st) } else { Ok(()) }
}

/// `ProbeForRead(Address, Length, Alignment)` without the raise.
#[inline]
pub(crate) unsafe fn probe_for_read(
    address: *const c_void,
    length: usize,
    alignment: u32,
) -> Result<(), NTSTATUS> {
    nt_result(safep_validate_user_range(address, length as SIZE_T, alignment as ULONG))
}

/// `ProbeForWrite(Address, Length, Alignment)` without the raise. Range +
/// alignment only: the C export's write touch-test is replaced by the
/// fault-tolerant copy that every converted site does afterwards.
#[inline]
pub(crate) unsafe fn probe_for_write(
    address: *mut c_void,
    length: usize,
    alignment: u32,
) -> Result<(), NTSTATUS> {
    nt_result(safep_validate_user_range(address as *const c_void, length as SIZE_T, alignment as ULONG))
}

/// `ProbeForRead(src, len, align); RtlCopyMemory(dst, src, len);` inside one
/// `_SEH2_TRY`: range check, then a copy that survives an unmapped page.
#[inline]
pub(crate) unsafe fn copy_from_user(
    dst: *mut c_void,
    src: *const c_void,
    length: usize,
    alignment: u32,
) -> Result<(), NTSTATUS> {
    probe_for_read(src, length, alignment)?;
    ntkrnl_sys::probe::safe_copy_raw(dst, src, length)
}

/// `ProbeForWrite(dst, len, align); RtlCopyMemory(dst, src, len);` inside one
/// `_SEH2_TRY`.
#[inline]
pub(crate) unsafe fn copy_to_user(
    dst: *mut c_void,
    src: *const c_void,
    length: usize,
    alignment: u32,
) -> Result<(), NTSTATUS> {
    probe_for_write(dst, length, alignment)?;
    ntkrnl_sys::probe::safe_copy_raw(dst, src, length)
}

/// `ProbeForReadGenericType(Ptr, T, Default)` (probe.h) — the typed macros
/// `ProbeForReadUnicodeString`, `ProbeForReadLargeString`, `ProbeForReadUint`
/// .. — capture a `T` by value. Range check only (the macro checks no
/// alignment), and on failure `Err` where the macro raises.
#[inline]
pub(crate) unsafe fn capture_user<T: Copy>(ptr: *const T) -> Result<T, NTSTATUS> {
    let mut out = core::mem::MaybeUninit::<T>::uninit();
    copy_from_user(
        out.as_mut_ptr() as *mut c_void,
        ptr as *const c_void,
        core::mem::size_of::<T>(),
        1,
    )?;
    // Only a complete copy (Ok) initializes T.
    Ok(out.assume_init())
}

/// `ProbeAndCaptureUnicodeString` (sdk/include/reactos/probe.h, static
/// __inline) with its `_SEH2_TRY`/`_SEH2_EXCEPT` made real: a bad header or
/// buffer pointer returns the fault status with `*Dest` zeroed and nothing
/// allocated, exactly the C handler. Free the result with
/// `ReleaseCapturedUnicodeString` (pool tag 'RTSU').
///
/// One divergence, deliberately: on pool exhaustion the C `_SEH2_LEAVE`s with
/// `Dest->Buffer` still holding the CALLER'S user pointer, and callers that
/// `ReleaseCapturedUnicodeString` on their exit path then ExFreePool a user
/// address. Here `*Dest` is zeroed on that path too.
pub(crate) unsafe fn probe_and_capture_unicode_string(
    Dest: PUNICODE_STRING,
    CurrentMode: i8,
    UnsafeSrc: *const _UNICODE_STRING,
) -> NTSTATUS {
    if CurrentMode == 0 {
        // KernelMode: trust the caller, plain structure copy (no allocation).
        *Dest = *UnsafeSrc;
        return STATUS_SUCCESS;
    }

    let mut src: _UNICODE_STRING = match capture_user(UnsafeSrc) {
        Ok(v) => v,
        Err(st) => {
            (*Dest).Length = 0;
            (*Dest).MaximumLength = 0;
            (*Dest).Buffer = core::ptr::null_mut();
            return st;
        }
    };

    if src.Buffer.is_null() {
        /* Sanitize structure */
        src.Length = 0;
        src.MaximumLength = 0;
        *Dest = src;
        return STATUS_SUCCESS;
    }
    if src.Length == 0 {
        /* Sanitize structure */
        src.MaximumLength = 0;
        src.Buffer = core::ptr::null_mut();
        *Dest = src;
        return STATUS_SUCCESS;
    }

    // ProbeForRead(Dest->Buffer, Dest->Length, sizeof(WCHAR)) -- before the
    // allocation, as in the C.
    let st = safep_validate_user_range(
        src.Buffer as *const c_void,
        src.Length as SIZE_T,
        core::mem::size_of::<u16>() as ULONG,
    );
    if st < 0 {
        (*Dest).Length = 0;
        (*Dest).MaximumLength = 0;
        (*Dest).Buffer = core::ptr::null_mut();
        return st;
    }

    let tag = u32::from_le_bytes(*b"RTSU");
    let buffer = ntkrnl_sys::ddk::ExAllocatePoolWithTag(
        PagedPool as _,
        (src.Length as usize + core::mem::size_of::<u16>()) as _,
        tag,
    ) as *mut u16;
    if buffer.is_null() {
        (*Dest).Length = 0;
        (*Dest).MaximumLength = 0;
        (*Dest).Buffer = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Copy it */
    if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
        buffer as *mut c_void,
        src.Buffer as *const c_void,
        src.Length as usize,
    ) {
        /* Free allocated resources and zero the destination string */
        ntkrnl_sys::ddk::ExFreePoolWithTag(buffer as *mut c_void, tag);
        (*Dest).Length = 0;
        (*Dest).MaximumLength = 0;
        (*Dest).Buffer = core::ptr::null_mut();
        return st;
    }
    *buffer.add(src.Length as usize / core::mem::size_of::<u16>()) = 0;

    /* Set it as the buffer */
    src.Buffer = buffer;
    if src.Length % 2 != 0 {
        src.Length -= 1;
    }
    // UNICODE_STRING_MAX_BYTES = 0xFFFE
    src.MaximumLength = if src.Length as u32 >= 0xFFFE {
        src.Length
    } else {
        src.Length + core::mem::size_of::<u16>() as u16
    };
    *Dest = src;
    STATUS_SUCCESS
}
