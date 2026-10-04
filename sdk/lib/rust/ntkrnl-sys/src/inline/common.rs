//! Common inline functions shared across subsystems.
//!
//! Probe macros, buffer checks, object attributes, etc.

use crate::base::*;
use crate::constants::*;
use crate::ddk;

// ============================================================================
// CONTAINING_RECORD typed helper
// ============================================================================

/// CONTAINING_RECORD_typed — subtract field offset to get containing struct pointer.
/// Type-safe version for inline Rust code.
#[inline]
pub unsafe fn CONTAINING_RECORD_typed<T>(address: PVOID, field_offset: usize) -> *mut T {
    (address as *mut u8).sub(field_offset) as *mut T
}

// ============================================================================
// Extern declarations for kernel functions not in bindgen
// ============================================================================

extern "system" {
    fn ProbeForRead(Address: PVOID, Length: ULONG_PTR, Alignment: ULONG);
    fn ProbeForWrite(Address: PVOID, Length: ULONG_PTR, Alignment: ULONG);
}

// ============================================================================
// ICIF flags for DefaultQueryInfoBufferCheck / DefaultSetInfoBufferCheck
// ============================================================================

pub const ICIF_QUERY: u16 = 0x1;
pub const ICIF_SET: u16 = 0x2;
pub const ICIF_QUERY_SIZE_VARIABLE: u16 = 0x4;
pub const ICIF_SET_SIZE_VARIABLE: u16 = 0x8;
pub const ICIF_PROBE_READ_U16: u16 = 0x10;
pub const ICIF_FORCE_RETURN_LENGTH_PROBE: u16 = 0x20;

// Pool tag 'USTR' in little-endian byte order
const TAG_USTR: u32 = u32::from_le_bytes(*b"USTR");

// ============================================================================
// InitializeObjectAttributes (macro from ndk/obtypes.h)
// ============================================================================

/// InitializeObjectAttributes — fill in an OBJECT_ATTRIBUTES struct.
#[inline]
pub unsafe fn InitializeObjectAttributes(
    p: *mut ddk::_OBJECT_ATTRIBUTES,
    n: ddk::PUNICODE_STRING,
    a: ULONG,
    r: ddk::HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<ddk::_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

// ============================================================================
// ExInitializeResource (macro = ExInitializeResourceLite)
// ============================================================================

/// ExInitializeResource — alias for ExInitializeResourceLite.
#[inline]
pub unsafe fn ExInitializeResource(resource: ddk::PERESOURCE) -> NTSTATUS {
    ddk::ExInitializeResourceLite(resource)
}

// ============================================================================
// PspClearCrossThreadFlag (macro from ps_x.h)
// ============================================================================

/// PspClearCrossThreadFlag — atomically clear a cross-thread flag.
/// C: `InterlockedAnd((PLONG)&Thread->CrossThreadFlags, ~Flag)`
#[inline]
pub unsafe fn PspClearCrossThreadFlag(thread: ddk::PETHREAD, flag: ULONG) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let ptr = core::ptr::addr_of_mut!((*thread).__bindgen_anon_7.CrossThreadFlags) as *const AtomicI32;
    (*ptr).fetch_and(!(flag as i32), Ordering::SeqCst);
}

// ============================================================================
// Probe macros (from sdk/include/reactos/probe.h)
// ============================================================================
//
// Every typed probe is `ProbeForReadGenericType` / `ProbeForWriteGenericType`:
// raise an access violation unless the whole object lies below
// `MmUserProbeAddress` (and the span does not wrap), then touch it. These used to
// be bare volatile accesses with NO range check, so every syscall whose probe
// resolved here (NtProtectVirtualMemory's OldAccessProtection, the config/po/dbgk
// out-params, ...) wrote through a kernel pointer handed in from user mode.

extern "system" {
    // ex-rust's definition returns `()`; don't trust a `-> !` it doesn't promise.
    #[link_name = "ExRaiseAccessViolation"]
    fn probe_raise_access_violation();
}

extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}

/// probe.h's test: `Ptr + sizeof(Type) - 1` wraps or reaches `MmUserProbeAddress`.
#[inline]
unsafe fn probe_outside_user<T>(ptr: *const T) -> bool {
    let first = ptr as ULONG_PTR;
    let last = first.wrapping_add(core::mem::size_of::<T>() - 1);
    last < first || last >= MmUserProbeAddress
}

/// `ProbeForReadGenericType(Ptr, T, Default)`; `Default` is all-zero for every user.
#[inline]
unsafe fn probe_read_generic_type<T: Copy>(ptr: *const T) -> T {
    if probe_outside_user(ptr) {
        probe_raise_access_violation();
        return core::mem::zeroed();
    }
    core::ptr::read_volatile(ptr)
}

/// `ProbeForWriteGenericType(Ptr, T)`: the touch writes the value back unchanged.
#[inline]
unsafe fn probe_write_generic_type<T: Copy>(ptr: *mut T) {
    if probe_outside_user(ptr) {
        probe_raise_access_violation();
        return;
    }
    core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr));
}

/// ProbeForReadLargeInteger — probe.h `ProbeForReadGenericType(.., LARGE_INTEGER, ..)`.
#[inline]
pub unsafe fn ProbeForReadLargeInteger(
    addr: *const ddk::_LARGE_INTEGER,
) -> ddk::_LARGE_INTEGER {
    probe_read_generic_type(addr)
}

/// ProbeForReadUchar — probe.h `ProbeForReadGenericType(Ptr, UCHAR, 0)`.
#[inline]
pub unsafe fn ProbeForReadUchar(addr: *const u8) -> u8 {
    probe_read_generic_type(addr)
}

/// ProbeForReadUshort — probe.h `ProbeForReadGenericType(Ptr, USHORT, 0)`.
#[inline]
pub unsafe fn ProbeForReadUshort(addr: *const u16) -> u16 {
    probe_read_generic_type(addr)
}

/// ProbeForWriteHandle — probe.h `ProbeForWriteGenericType(Ptr, HANDLE)`.
#[inline]
pub unsafe fn ProbeForWriteHandle(ptr: *mut ddk::HANDLE) {
    probe_write_generic_type(ptr)
}

/// ProbeForWriteUlong — probe.h `ProbeForWriteGenericType(Ptr, ULONG)`.
#[inline]
pub unsafe fn ProbeForWriteUlong(ptr: *mut ULONG) {
    probe_write_generic_type(ptr)
}

// ============================================================================
// DefaultSetInfoBufferCheck (static inline from ntoskrnl/include/internal/probe.h)
// ============================================================================

/// DefaultSetInfoBufferCheck — validate info class and probe buffer for SET operations.
///
/// Return probe failures directly; callers capture the contents with protected copies.
#[inline]
pub unsafe fn DefaultSetInfoBufferCheck(
    class: ULONG,
    class_list: *const ddk::_INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    previous_mode: i8, // KPROCESSOR_MODE
) -> NTSTATUS {
    if class >= class_list_entries {
        return STATUS_INVALID_INFO_CLASS;
    }

    let info = &*class_list.add(class as usize);

    if info.Flags & ICIF_SET == 0 {
        return STATUS_INVALID_INFO_CLASS;
    }

    if info.RequiredSizeSET > 0 && buffer_length != info.RequiredSizeSET as ULONG {
        if info.Flags & ICIF_SET_SIZE_VARIABLE == 0 {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    }

    if previous_mode != KernelMode {
        if let Err(status) = crate::probe::probe_user_buffer(
            buffer, buffer_length as usize, info.AlignmentSET as usize, previous_mode,
        ) {
            return status;
        }
    }

    STATUS_SUCCESS
}

// ============================================================================
// DefaultQueryInfoBufferCheck (static inline from ntoskrnl/include/internal/probe.h)
// ============================================================================

/// DefaultQueryInfoBufferCheck — validate info class and probe buffer for QUERY operations.
///
/// With DUMMY_PSEH, the SEH handlers are dead code; probing may fault.
#[inline]
pub unsafe fn DefaultQueryInfoBufferCheck(
    class: ULONG,
    class_list: *const ddk::_INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    flags: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    return_length: *mut ULONG,
    return_length_ptr: *mut ULONG_PTR,
    previous_mode: i8, // KPROCESSOR_MODE
) -> NTSTATUS {
    if class >= class_list_entries {
        return STATUS_INVALID_INFO_CLASS;
    }

    let info = &*class_list.add(class as usize);

    if info.Flags & ICIF_QUERY == 0 {
        return STATUS_INVALID_INFO_CLASS;
    }

    if info.RequiredSizeQUERY > 0 && buffer_length != info.RequiredSizeQUERY as ULONG {
        if info.Flags & ICIF_QUERY_SIZE_VARIABLE == 0 {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    }

    if previous_mode != KernelMode {
        if !buffer.is_null() {
            if flags & ICIF_PROBE_READ != 0 {
                ProbeForRead(
                    buffer,
                    buffer_length as ULONG_PTR,
                    info.AlignmentQUERY as ULONG,
                );
            } else {
                ProbeForWrite(
                    buffer,
                    buffer_length as ULONG_PTR,
                    info.AlignmentQUERY as ULONG,
                );
            }
        }

        if (flags & (ICIF_FORCE_RETURN_LENGTH_PROBE as ULONG) != 0)
            || !return_length.is_null()
        {
            ProbeForWriteUlong(return_length);
        }

        if !return_length_ptr.is_null() {
            ProbeForWrite(
                return_length_ptr as PVOID,
                core::mem::size_of::<ULONG_PTR>() as ULONG_PTR,
                core::mem::size_of::<ULONG_PTR>() as ULONG,
            );
        }
    }

    STATUS_SUCCESS
}

// ============================================================================
// ProbeAndCaptureUnicodeString (static inline from sdk/include/reactos/probe.h)
// ============================================================================

/// ProbeAndCaptureUnicodeString — probe and capture a user-mode UNICODE_STRING.
///
/// Allocates pool memory for the buffer copy. With DUMMY_PSEH, no exception handling.
#[inline]
pub unsafe fn ProbeAndCaptureUnicodeString(
    dest: ddk::PUNICODE_STRING,
    mode: i8, // KPROCESSOR_MODE
    src: *const ddk::_UNICODE_STRING,
) -> NTSTATUS {
    if mode != KernelMode {
        // Probe and copy the UNICODE_STRING header
        ProbeForRead(
            src as PVOID,
            core::mem::size_of::<ddk::_UNICODE_STRING>() as ULONG_PTR,
            1,
        );
        core::ptr::copy_nonoverlapping(src, dest, 1);

        if !(*dest).Buffer.is_null() {
            if (*dest).Length != 0 {
                // Probe the buffer
                ProbeForRead(
                    (*dest).Buffer as PVOID,
                    (*dest).Length as ULONG_PTR,
                    core::mem::size_of::<u16>() as ULONG,
                );

                // Allocate and copy
                let alloc_size = (*dest).Length as ULONG_PTR + core::mem::size_of::<u16>() as ULONG_PTR;
                let buffer = ddk::ExAllocatePoolWithTag(
                    ddk::_POOL_TYPE_PagedPool,
                    alloc_size as ddk::SIZE_T,
                    TAG_USTR,
                ) as *mut u16;

                if buffer.is_null() {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }

                core::ptr::copy_nonoverlapping(
                    (*dest).Buffer,
                    buffer,
                    (*dest).Length as usize / core::mem::size_of::<u16>(),
                );
                // Null-terminate
                *buffer.add((*dest).Length as usize / core::mem::size_of::<u16>()) = 0;

                (*dest).Buffer = buffer;

                // Adjust Length/MaximumLength
                if (*dest).Length % (core::mem::size_of::<u16>() as u16) != 0 {
                    (*dest).Length -= 1;
                }
                if (*dest).Length >= 0xFFFE {
                    // UNICODE_STRING_MAX_BYTES
                    (*dest).MaximumLength = (*dest).Length;
                } else {
                    (*dest).MaximumLength =
                        (*dest).Length + core::mem::size_of::<u16>() as u16;
                }
            } else {
                (*dest).MaximumLength = 0;
                (*dest).Buffer = core::ptr::null_mut();
            }
        } else {
            (*dest).Length = 0;
            (*dest).MaximumLength = 0;
        }
    } else {
        // KernelMode — trust the pointer, just copy the struct
        core::ptr::copy_nonoverlapping(src, dest, 1);
    }

    STATUS_SUCCESS
}

// ============================================================================
// ReleaseCapturedUnicodeString (static inline from sdk/include/reactos/probe.h)
// ============================================================================

/// ReleaseCapturedUnicodeString — free captured unicode string buffer.
#[inline]
pub unsafe fn ReleaseCapturedUnicodeString(
    str: ddk::PUNICODE_STRING,
    mode: i8, // KPROCESSOR_MODE
) {
    if mode != KernelMode && !(*str).Buffer.is_null() {
        ddk::ExFreePoolWithTag((*str).Buffer as PVOID, TAG_USTR);
    }

    (*str).Length = 0;
    (*str).MaximumLength = 0;
    (*str).Buffer = core::ptr::null_mut();
}

// ============================================================================
// FsRtlEnterFileSystem / FsRtlExitFileSystem (macros from fltkernel.h)
// ============================================================================

/// FsRtlEnterFileSystem — disable APCs (KeEnterCriticalRegion).
#[inline]
pub unsafe fn FsRtlEnterFileSystem() {
    ddk::KeEnterCriticalRegion();
}

/// FsRtlExitFileSystem — re-enable APCs (KeLeaveCriticalRegion).
#[inline]
pub unsafe fn FsRtlExitFileSystem() {
    ddk::KeLeaveCriticalRegion();
}

// ============================================================================
// CcBugCheck (macro from cc.h)
// ============================================================================

/// CcBugCheck — bugcheck with CACHE_MANAGER code.
/// C: `KeBugCheckEx(CACHE_MANAGER, 0, A, B, C)`
#[inline]
pub unsafe fn CcBugCheck(a: ULONG, b: ULONG, c: ULONG) {
    ddk::KeBugCheckEx(
        CACHE_MANAGER,
        0 as ddk::ULONG_PTR,
        a as ddk::ULONG_PTR,
        b as ddk::ULONG_PTR,
        c as ddk::ULONG_PTR,
    );
}

// ============================================================================
// RTL Splay Link macros (from ntdef.h / rtltypes.h)
// ============================================================================

/// RtlInitializeSplayLinks — initialize splay links (Parent=self, children=NULL).
/// C: `Links->Parent = Links; Links->LeftChild = Links->RightChild = NULL;`
#[inline]
pub unsafe fn RtlInitializeSplayLinks(links: ddk::PRTL_SPLAY_LINKS) {
    (*links).Parent = links;
    (*links).LeftChild = core::ptr::null_mut();
    (*links).RightChild = core::ptr::null_mut();
}

/// RtlInsertAsLeftChild — insert ChildLinks as the left child of ParentLinks.
/// C: `ParentLinks->LeftChild = ChildLinks; ChildLinks->Parent = ParentLinks;`
#[inline]
pub unsafe fn RtlInsertAsLeftChild(
    parent: ddk::PRTL_SPLAY_LINKS,
    child: ddk::PRTL_SPLAY_LINKS,
) {
    (*parent).LeftChild = child;
    (*child).Parent = parent;
}

/// RtlInsertAsRightChild — insert ChildLinks as the right child of ParentLinks.
/// C: `ParentLinks->RightChild = ChildLinks; ChildLinks->Parent = ParentLinks;`
#[inline]
pub unsafe fn RtlInsertAsRightChild(
    parent: ddk::PRTL_SPLAY_LINKS,
    child: ddk::PRTL_SPLAY_LINKS,
) {
    (*parent).RightChild = child;
    (*child).Parent = parent;
}

/// RtlIsLeftChild — check if Links is the left child of its parent.
/// C: `(RtlLeftChild(RtlParent(Links)) == (PRTL_SPLAY_LINKS)(Links))`
#[inline]
pub unsafe fn RtlIsLeftChild(links: ddk::PRTL_SPLAY_LINKS) -> BOOLEAN {
    if (*(*links).Parent).LeftChild == links { 1 } else { 0 }
}

/// RtlParent — return the parent of a splay link node.
/// C: `(PRTL_SPLAY_LINKS)(Links)->Parent`
#[inline]
pub unsafe fn RtlParent(links: ddk::PRTL_SPLAY_LINKS) -> ddk::PRTL_SPLAY_LINKS {
    (*links).Parent
}

// ============================================================================
// RtlLongLongAdd (FORCEINLINE from intsafe.h)
// ============================================================================

// INTSAFE_E_ARITHMETIC_OVERFLOW = HRESULT(0x80070216)
const INTSAFE_E_ARITHMETIC_OVERFLOW: i32 = 0x80070216_u32 as i32;

/// RtlLongLongAdd — checked 64-bit addition.
/// C: returns S_OK (0) on success, INTSAFE_E_ARITHMETIC_OVERFLOW on overflow.
#[inline]
pub unsafe fn RtlLongLongAdd(
    augend: i64,
    addend: i64,
    result: *mut i64,
) -> i32 {
    match augend.checked_add(addend) {
        Some(sum) => {
            *result = sum;
            0 // S_OK
        }
        None => {
            *result = 0; // LONGLONG_ERROR = 0
            INTSAFE_E_ARITHMETIC_OVERFLOW
        }
    }
}

// ============================================================================
// ProbeForWritePointer (macro from probe.h)
// ============================================================================

/// ProbeForWritePointer — probe.h `ProbeForWriteGenericType(Ptr, PVOID)`.
#[inline]
pub unsafe fn ProbeForWritePointer(ptr: *mut PVOID) {
    probe_write_generic_type(ptr)
}

// ============================================================================
// RtlInitEmptyAnsiString (FORCEINLINE from rtlfuncs.h)
// ============================================================================

/// RtlInitEmptyAnsiString — initialize an empty ANSI_STRING with a buffer.
/// C: `Length = 0; MaximumLength = MaximumLength; Buffer = Buffer;`
#[inline]
pub unsafe fn RtlInitEmptyAnsiString(
    ansi_string: ddk::PANSI_STRING,
    buffer: *mut i8,
    maximum_length: u16,
) {
    (*ansi_string).Length = 0;
    (*ansi_string).MaximumLength = maximum_length;
    (*ansi_string).Buffer = buffer;
}

// ============================================================================
// InsertTailList (FORCEINLINE from wdm.h)
// ============================================================================

/// InsertTailList — insert entry at the tail of a doubly-linked list.
/// C: `Blink = ListHead->Blink; Entry->Flink = ListHead; Entry->Blink = Blink;
///     Blink->Flink = Entry; ListHead->Blink = Entry;`
#[inline]
pub unsafe fn InsertTailList(
    list_head: *mut ddk::_LIST_ENTRY,
    entry: *mut ddk::_LIST_ENTRY,
) {
    let blink = (*list_head).Blink;
    (*entry).Flink = list_head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*list_head).Blink = entry;
}

// ============================================================================
// PsGetCurrentThread (FORCEINLINE from psfuncs.h)
// ============================================================================

/// PsGetCurrentThread — return current thread as PETHREAD.
/// C: `return (PETHREAD)KeGetCurrentThread();`
/// Uses mm::KeGetCurrentThread (inline asm) since ddk::KeGetCurrentThread is
/// FORCEINLINE on amd64 and not available in bindgen.
#[inline]
pub unsafe fn PsGetCurrentThread() -> ddk::PETHREAD {
    crate::mm::KeGetCurrentThread() as ddk::PETHREAD
}

// ============================================================================
// RtlInitEmptyUnicodeString (FORCEINLINE from rtlfuncs.h)
// ============================================================================

/// RtlInitEmptyUnicodeString — initialize an empty UNICODE_STRING with a buffer.
/// C: `Length = 0; MaximumLength = BufferSize; Buffer = buf;`
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    unicode_string: ddk::PUNICODE_STRING,
    buffer: *mut u16,
    buffer_size: u16,
) {
    (*unicode_string).Length = 0;
    (*unicode_string).MaximumLength = buffer_size;
    (*unicode_string).Buffer = buffer;
}

// ============================================================================
// ExceptionRecord32To64 (static __inline from windbgkd.h)
// ============================================================================

/// ExceptionRecord32To64 — widen a 32-bit exception record to 64-bit.
#[inline]
pub unsafe fn ExceptionRecord32To64(
    ex32: *const ddk::_EXCEPTION_RECORD32,
    ex64: *mut ddk::_EXCEPTION_RECORD64,
) {
    (*ex64).ExceptionCode = (*ex32).ExceptionCode;
    (*ex64).ExceptionFlags = (*ex32).ExceptionFlags;
    (*ex64).ExceptionRecord = (*ex32).ExceptionRecord as u64;
    (*ex64).ExceptionAddress = (*ex32).ExceptionAddress as u64;
    (*ex64).NumberParameters = (*ex32).NumberParameters;
    let n = (*ex32).NumberParameters as usize;
    let max = EXCEPTION_MAXIMUM_PARAMETERS as usize;
    let count = if n < max { n } else { max };
    for i in 0..count {
        (*ex64).ExceptionInformation[i] = (*ex32).ExceptionInformation[i] as u64;
    }
}

// ============================================================================
// ProbeForWriteBoolean (probe.h macro wrapping ProbeForWriteGenericType)
// ============================================================================

/// ProbeForWriteBoolean — probe.h `ProbeForWriteGenericType(Ptr, BOOLEAN)`.
#[inline]
pub unsafe fn ProbeForWriteBoolean(ptr: *mut BOOLEAN) {
    probe_write_generic_type(ptr)
}

// ============================================================================
// ProbeForWriteLangId (probe.h macro wrapping ProbeForWriteGenericType)
// ============================================================================

/// ProbeForWriteLangId — probe.h `ProbeForWriteGenericType(Ptr, LANGID)`.
#[inline]
pub unsafe fn ProbeForWriteLangId(ptr: *mut u16) {
    probe_write_generic_type(ptr)
}

// ============================================================================
// PopEntryList (rtlfuncs.h FORCEINLINE)
// ============================================================================

/// PopEntryList — pop first entry from a singly-linked list.
/// C: if (ListHead->Next != NULL) { ListHead->Next = FirstEntry->Next; } return FirstEntry;
#[inline]
pub unsafe fn PopEntryList(
    list_head: *mut ddk::_SINGLE_LIST_ENTRY,
) -> *mut ddk::_SINGLE_LIST_ENTRY {
    let first = (*list_head).Next;
    if !first.is_null() {
        (*list_head).Next = (*first).Next;
    }
    first
}

// ============================================================================
// InterlockedExchangeAdd64 (intrinsic _InterlockedExchangeAdd64)
// ============================================================================

/// InterlockedExchangeAdd64 — atomic 64-bit add, return old value.
#[inline]
pub unsafe fn InterlockedExchangeAdd64(addend: *mut i64, value: i64) -> i64 {
    use core::sync::atomic::{AtomicI64, Ordering};
    let atomic = &*(addend as *const AtomicI64);
    atomic.fetch_add(value, Ordering::SeqCst)
}

// ============================================================================
// Interlocked functions (compiler intrinsics → atomics)
// ============================================================================

/// InterlockedDecrementUL — atomic decrement of ULONG, return new value.
/// C: `return (ULONG)InterlockedDecrement((PLONG)Addend);`
#[inline]
pub unsafe fn InterlockedDecrementUL(addend: *mut ULONG) -> ULONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(addend as *const AtomicI32);
    let old = atomic.fetch_sub(1, Ordering::SeqCst);
    (old - 1) as ULONG
}

/// InterlockedCompareExchangePointer — atomic CAS for pointer-sized values.
/// C: `return _InterlockedCompareExchangePointer(Destination, Exchange, Comparand);`
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    destination: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(destination as *const AtomicUsize);
    match atomic.compare_exchange(
        comparand as usize,
        exchange as usize,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(old) => old as PVOID,
        Err(old) => old as PVOID,
    }
}

/// InterlockedCompareExchange — atomic CAS for LONG.
/// C: `return _InterlockedCompareExchange(Destination, Exchange, Comparand);`
#[inline]
pub unsafe fn InterlockedCompareExchange(
    destination: *mut i32,
    exchange: i32,
    comparand: i32,
) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(destination as *const AtomicI32);
    match atomic.compare_exchange(
        comparand,
        exchange,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// ============================================================================
// RtlStringCbCopyA (ntstrsafe.h static inline)
// ============================================================================

const NTSTRSAFE_MAX_CCH: usize = 2147483647; // INT_MAX

/// RtlStringCopyWorkerA — bounded string copy (inline Rust implementation).
/// Copies up to cchDest-1 chars from pszSrc, NUL-terminates.
#[inline]
unsafe fn rtl_string_copy_worker_a(
    psz_dest: *mut u8,
    cch_dest: usize,
    psz_src: *const u8,
) -> NTSTATUS {
    if cch_dest == 0 {
        return crate::constants::STATUS_INVALID_PARAMETER;
    }
    let mut i: usize = 0;
    while i < cch_dest - 1 {
        let ch = *psz_src.add(i);
        if ch == 0 {
            break;
        }
        *psz_dest.add(i) = ch;
        i += 1;
    }
    *psz_dest.add(i) = 0;
    if i < cch_dest - 1 || *psz_src.add(i) == 0 {
        0 // STATUS_SUCCESS
    } else {
        crate::constants::STATUS_BUFFER_OVERFLOW
    }
}

/// RtlStringCbCopyA — safe string copy with byte-count limit.
#[inline]
pub unsafe fn RtlStringCbCopyA(
    psz_dest: *mut u8,
    cb_dest: usize,
    psz_src: *const u8,
) -> NTSTATUS {
    let cch_dest = cb_dest;
    if cch_dest > NTSTRSAFE_MAX_CCH {
        if cch_dest > 0 {
            *psz_dest = 0;
        }
        return crate::constants::STATUS_INVALID_PARAMETER;
    }
    rtl_string_copy_worker_a(psz_dest, cch_dest, psz_src)
}
