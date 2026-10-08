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

use crate::probe::probe_user_range;

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

/// ProbeForReadLargeInteger — volatile read of a LARGE_INTEGER from user-mode.
#[inline]
pub unsafe fn ProbeForReadLargeInteger(
    addr: *const ddk::_LARGE_INTEGER,
) -> ddk::_LARGE_INTEGER {
    probe_user_range(addr);
    core::ptr::read_volatile(addr)
}

/// ProbeForReadUchar — volatile read of a UCHAR from user-mode.
#[inline]
pub unsafe fn ProbeForReadUchar(addr: *const u8) -> u8 {
    probe_user_range(addr);
    core::ptr::read_volatile(addr)
}

/// ProbeForReadUshort — volatile read of a USHORT from user-mode.
#[inline]
pub unsafe fn ProbeForReadUshort(addr: *const u16) -> u16 {
    probe_user_range(addr);
    core::ptr::read_volatile(addr)
}

/// ProbeForWriteHandle — volatile read/write to validate user-mode handle ptr.
#[inline]
pub unsafe fn ProbeForWriteHandle(ptr: *mut ddk::HANDLE) {
    probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
}

/// ProbeForWriteUlong — volatile read/write to validate user-mode ULONG ptr.
#[inline]
pub unsafe fn ProbeForWriteUlong(ptr: *mut ULONG) {
    probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
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

    // C: the probes are one _SEH2_TRY whose handler returns the exception code.
    // ProbeForRead and ProbeForWrite make the same range/alignment check, so one
    // Result probe serves both; ProbeForWriteUlong is the typed range-and-touch.
    if previous_mode != KernelMode {
        if !buffer.is_null() {
            if let Err(status) = crate::probe::probe_user_buffer(
                buffer,
                buffer_length as usize,
                info.AlignmentQUERY as usize,
                previous_mode,
            ) {
                return status;
            }
        }

        if (flags & (ICIF_FORCE_RETURN_LENGTH_PROBE as ULONG) != 0)
            || !return_length.is_null()
        {
            if let Err(status) = crate::probe::probe_write_generic(return_length, 1, previous_mode) {
                return status;
            }
        }

        if !return_length_ptr.is_null() {
            if let Err(status) = crate::probe::probe_user_buffer(
                return_length_ptr as PVOID,
                core::mem::size_of::<ULONG_PTR>(),
                core::mem::size_of::<ULONG_PTR>(),
                previous_mode,
            ) {
                return status;
            }
        }
    }

    STATUS_SUCCESS
}

// ============================================================================
// ProbeAndCaptureUnicodeString (static inline from sdk/include/reactos/probe.h)
// ============================================================================

/// ProbeAndCaptureUnicodeString — probe and capture a user-mode UNICODE_STRING.
///
/// Allocates pool memory for the buffer copy. The C version catches its own probe
/// exceptions (`_SEH2_EXCEPT`: free the buffer, zero `Dest`, return the code); this
/// kernel has no SEH, so the probes and copies are the `Result` ones from `probe.rs`
/// and the same cleanup runs on `Err`.
#[inline]
pub unsafe fn ProbeAndCaptureUnicodeString(
    dest: ddk::PUNICODE_STRING,
    mode: i8, // KPROCESSOR_MODE
    src: *const ddk::_UNICODE_STRING,
) -> NTSTATUS {
    if mode == KernelMode {
        // KernelMode — trust the pointer, just copy the struct
        core::ptr::copy_nonoverlapping(src, dest, 1);
        return STATUS_SUCCESS;
    }

    let mut buffer: *mut u16 = core::ptr::null_mut();
    let result = (|| -> Result<NTSTATUS, NTSTATUS> {
        crate::probe::safe_copy_from_user(
            dest.cast(),
            src.cast(),
            core::mem::size_of::<ddk::_UNICODE_STRING>(),
            1,
            mode,
        )?;
        if (*dest).Buffer.is_null() {
            (*dest).Length = 0;
            (*dest).MaximumLength = 0;
            return Ok(STATUS_SUCCESS);
        }
        if (*dest).Length == 0 {
            (*dest).MaximumLength = 0;
            (*dest).Buffer = core::ptr::null_mut();
            return Ok(STATUS_SUCCESS);
        }
        let length = (*dest).Length as usize;
        crate::probe::probe_user_buffer((*dest).Buffer.cast(), length, 2, mode)?;

        buffer = ddk::ExAllocatePoolWithTag(
            ddk::_POOL_TYPE_PagedPool,
            (length + core::mem::size_of::<u16>()) as ddk::SIZE_T,
            TAG_USTR,
        ) as *mut u16;
        if buffer.is_null() {
            // C: Status = STATUS_INSUFFICIENT_RESOURCES; _SEH2_LEAVE — no cleanup.
            return Ok(STATUS_INSUFFICIENT_RESOURCES);
        }
        crate::probe::safe_copy_from_user(buffer.cast(), (*dest).Buffer.cast(), length, 2, mode)?;
        *buffer.add(length / core::mem::size_of::<u16>()) = 0;

        (*dest).Buffer = buffer;
        if (*dest).Length % (core::mem::size_of::<u16>() as u16) != 0 {
            (*dest).Length -= 1;
        }
        if (*dest).Length >= 0xFFFE {
            // UNICODE_STRING_MAX_BYTES
            (*dest).MaximumLength = (*dest).Length;
        } else {
            (*dest).MaximumLength = (*dest).Length + core::mem::size_of::<u16>() as u16;
        }
        Ok(STATUS_SUCCESS)
    })();

    match result {
        Ok(status) => status,
        Err(status) => {
            if !buffer.is_null() {
                ddk::ExFreePoolWithTag(buffer.cast(), TAG_USTR);
            }
            (*dest).Length = 0;
            (*dest).MaximumLength = 0;
            (*dest).Buffer = core::ptr::null_mut();
            status
        }
    }
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

/// ProbeForWritePointer — volatile read/write to validate user-mode pointer-sized ptr.
#[inline]
pub unsafe fn ProbeForWritePointer(ptr: *mut PVOID) {
    probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
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
// The rest of rtlfuncs.h's list FORCEINLINEs, C-shaped over ddk types (`base::` has
// ntoskrnl's own Rust-flavoured versions: its own LIST_ENTRY, `bool` returns). The checks are
// the C's: this tree never defines NO_KERNEL_LIST_ENTRY_CHECKS, so a C driver fast-fails on a
// corrupted link and so does this.
// ============================================================================

/// RtlFailFast — `__fastfail(Code)`: `int 0x29` / `brk #0xF003` with the code in ecx / x0,
/// which the kernel turns into KERNEL_SECURITY_CHECK_FAILURE. Never returns.
#[inline]
pub unsafe fn RtlFailFast(code: ULONG) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int 0x29", in("ecx") code, options(noreturn, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xF003", in("x0") code as u64, options(noreturn, nostack));
}

/// FAST_FAIL_CORRUPT_LIST_ENTRY.
const FAST_FAIL_CORRUPT_LIST_ENTRY: ULONG = 3;

/// InitializeListHead — `ListHead->Flink = ListHead->Blink = ListHead;`
#[inline]
pub unsafe fn InitializeListHead(list_head: *mut ddk::_LIST_ENTRY) {
    (*list_head).Flink = list_head;
    (*list_head).Blink = list_head;
}

/// IsListEmpty — `(BOOLEAN)(ListHead->Flink == ListHead)`.
#[inline]
pub unsafe fn IsListEmpty(list_head: *const ddk::_LIST_ENTRY) -> BOOLEAN {
    ((*list_head).Flink as *const ddk::_LIST_ENTRY == list_head) as BOOLEAN
}

/// RemoveEntryList — unlink `entry`; TRUE when the list is now empty.
#[inline]
pub unsafe fn RemoveEntryList(entry: *mut ddk::_LIST_ENTRY) -> BOOLEAN {
    let old_flink = (*entry).Flink;
    let old_blink = (*entry).Blink;
    if (*old_flink).Blink != entry || (*old_blink).Flink != entry {
        RtlFailFast(FAST_FAIL_CORRUPT_LIST_ENTRY);
    }
    (*old_flink).Blink = old_blink;
    (*old_blink).Flink = old_flink;
    (old_flink == old_blink) as BOOLEAN
}

/// RemoveHeadList — unlink and return the first entry (the head itself when empty).
#[inline]
pub unsafe fn RemoveHeadList(list_head: *mut ddk::_LIST_ENTRY) -> *mut ddk::_LIST_ENTRY {
    #[cfg(feature = "dbg")]
    if (*(*list_head).Flink).Blink != list_head || (*(*list_head).Blink).Flink != list_head {
        RtlFailFast(FAST_FAIL_CORRUPT_LIST_ENTRY); // RtlpCheckListEntry
    }
    let entry = (*list_head).Flink;
    let flink = (*entry).Flink;
    if (*entry).Blink != list_head || (*flink).Blink != entry {
        RtlFailFast(FAST_FAIL_CORRUPT_LIST_ENTRY);
    }
    (*list_head).Flink = flink;
    (*flink).Blink = list_head;
    entry
}

/// InsertHeadList — link `entry` in after the head.
#[inline]
pub unsafe fn InsertHeadList(list_head: *mut ddk::_LIST_ENTRY, entry: *mut ddk::_LIST_ENTRY) {
    #[cfg(feature = "dbg")]
    if (*(*list_head).Flink).Blink != list_head || (*(*list_head).Blink).Flink != list_head {
        RtlFailFast(FAST_FAIL_CORRUPT_LIST_ENTRY); // RtlpCheckListEntry
    }
    let old_flink = (*list_head).Flink;
    (*entry).Flink = old_flink;
    (*entry).Blink = list_head;
    if (*old_flink).Blink != list_head {
        RtlFailFast(FAST_FAIL_CORRUPT_LIST_ENTRY);
    }
    (*old_flink).Blink = entry;
    (*list_head).Flink = entry;
}

/// InitializeSListHead — rtlfuncs.h FORCEINLINE: zero the header (64-bit: after checking the
/// 16-byte alignment the interlocked 128-bit compare-exchange needs).
#[inline]
pub unsafe fn InitializeSListHead(slist_head: ddk::PSLIST_HEADER) {
    #[cfg(target_pointer_width = "64")]
    if (slist_head as usize) & 0xf != 0 {
        ddk::ExRaiseStatus(0x8000_0002_u32 as NTSTATUS); // STATUS_DATATYPE_MISALIGNMENT
    }
    core::ptr::write_bytes(slist_head as *mut u8, 0, core::mem::size_of::<ddk::SLIST_HEADER>());
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

/// ProbeForWriteBoolean — volatile read/write to validate user-mode BOOLEAN ptr.
#[inline]
pub unsafe fn ProbeForWriteBoolean(ptr: *mut BOOLEAN) {
    probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
}

// ============================================================================
// ProbeForWriteLangId (probe.h macro wrapping ProbeForWriteGenericType)
// ============================================================================

/// ProbeForWriteLangId — volatile read/write to validate user-mode LANGID ptr.
#[inline]
pub unsafe fn ProbeForWriteLangId(ptr: *mut u16) {
    probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
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

/// PushEntryList — rtlfuncs.h FORCEINLINE: link `entry` in at the head.
/// C: Entry->Next = ListHead->Next; ListHead->Next = Entry;
#[inline]
pub unsafe fn PushEntryList(
    list_head: *mut ddk::_SINGLE_LIST_ENTRY,
    entry: *mut ddk::_SINGLE_LIST_ENTRY,
) {
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}

/// RtlULongAdd — intsafe.h `DEFINE_SAFE_ADD(ULongAdd, ULONG)`: on overflow the output is
/// ULONG_ERROR and the result STATUS_INTEGER_OVERFLOW (INTSAFE_E_ARITHMETIC_OVERFLOW in kernel
/// mode), never a wrapped sum.
#[inline]
pub unsafe fn RtlULongAdd(augend: ULONG, addend: ULONG, output: *mut ULONG) -> NTSTATUS {
    match augend.checked_add(addend) {
        Some(sum) => {
            *output = sum;
            0 // STATUS_SUCCESS
        }
        None => {
            *output = ULONG::MAX; // ULONG_ERROR
            0xC000_0095_u32 as NTSTATUS // STATUS_INTEGER_OVERFLOW
        }
    }
}

// rtlfuncs.h: `#define RtlUlongByteSwap(_x) _byteswap_ulong((_x))` (and the USHORT/ULONGLONG
// twins) -- an intrinsic everywhere but i386, where ntoskrnl exports fastcall functions the
// bindgen glob declares.
/// RtlUshortByteSwap — `_byteswap_ushort`.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub fn RtlUshortByteSwap(source: USHORT) -> USHORT {
    source.swap_bytes()
}
/// RtlUlongByteSwap — `_byteswap_ulong`.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub fn RtlUlongByteSwap(source: ULONG) -> ULONG {
    source.swap_bytes()
}
/// RtlUlonglongByteSwap — `_byteswap_uint64`.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub fn RtlUlonglongByteSwap(source: ULONGLONG) -> ULONGLONG {
    source.swap_bytes()
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

// ============================================================================
// wdm.h / ntdef.h macros a driver calls as functions
// ============================================================================

/// `RtlEqualMemory(D, S, L)` = `(!memcmp((D), (S), (L)))`. Through `RtlCompareMemory`, which
/// ntoskrnl exports, so no Rust byte loop can be folded back into a self-call.
#[inline]
pub unsafe fn RtlEqualMemory(
    destination: *const core::ffi::c_void,
    source: *const core::ffi::c_void,
    length: SIZE_T,
) -> bool {
    ddk::RtlCompareMemory(destination, source, length as _) == length as _
}

/// `PoSetDeviceBusy(IdlePointer)` = `((void)(*(IdlePointer) = 0))`.
#[inline]
pub unsafe fn PoSetDeviceBusy(idle_pointer: *mut ULONG) {
    *idle_pointer = 0;
}

/// `IoInitializeRemoveLock(Lock, Tag, Minutes, HighWater)` =
/// `IoInitializeRemoveLockEx(Lock, Tag, Minutes, HighWater, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoInitializeRemoveLock(
    lock: ddk::PIO_REMOVE_LOCK,
    allocate_tag: ULONG,
    max_locked_minutes: ULONG,
    high_watermark: ULONG,
) {
    ddk::IoInitializeRemoveLockEx(
        lock,
        allocate_tag,
        max_locked_minutes,
        high_watermark,
        core::mem::size_of::<ddk::IO_REMOVE_LOCK>() as ULONG,
    );
}

/// `IoAdjustPagingPathCount(Count, Increment)`: interlocked ++ or -- of `*Count`.
#[inline]
pub unsafe fn IoAdjustPagingPathCount(count: *mut LONG, increment: BOOLEAN) {
    if increment != 0 {
        crate::interlocked::InterlockedIncrement(count);
    } else {
        crate::interlocked::InterlockedDecrement(count);
    }
}

/// `RTL_CONSTANT_OBJECT_ATTRIBUTES(n, a)`: a brace initializer in C, a value here.
#[inline]
pub const fn RTL_CONSTANT_OBJECT_ATTRIBUTES(
    object_name: ddk::PUNICODE_STRING,
    attributes: ULONG,
) -> ddk::OBJECT_ATTRIBUTES {
    ddk::OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<ddk::OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: object_name,
        Attributes: attributes,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    }
}

// ============================================================================
// GetIRPMinorFunctionString (FORCEINLINE from reactos/debug/driverdbg.h)
// ============================================================================

/// GetIRPMinorFunctionString — the PnP minor code's name, for debug prints.
#[inline]
pub fn GetIRPMinorFunctionString(minor_function: UCHAR) -> PCHAR {
    // iotypes.h; private here -- a new pub name in driver_ddk is E0659 in pcix's globs.
    const IRP_MN_QUERY_LEGACY_BUS_INFORMATION: u8 = 0x18;
    let s: *const u8 = match minor_function {
        crate::driver_ddk::IRP_MN_START_DEVICE => b"IRP_MN_START_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_REMOVE_DEVICE => b"IRP_MN_QUERY_REMOVE_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_REMOVE_DEVICE => b"IRP_MN_REMOVE_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_CANCEL_REMOVE_DEVICE => b"IRP_MN_CANCEL_REMOVE_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_STOP_DEVICE => b"IRP_MN_STOP_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_STOP_DEVICE => b"IRP_MN_QUERY_STOP_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_CANCEL_STOP_DEVICE => b"IRP_MN_CANCEL_STOP_DEVICE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_DEVICE_RELATIONS => b"IRP_MN_QUERY_DEVICE_RELATIONS\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_INTERFACE => b"IRP_MN_QUERY_INTERFACE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_CAPABILITIES => b"IRP_MN_QUERY_CAPABILITIES\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_RESOURCES => b"IRP_MN_QUERY_RESOURCES\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_RESOURCE_REQUIREMENTS => b"IRP_MN_QUERY_RESOURCE_REQUIREMENTS\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_DEVICE_TEXT => b"IRP_MN_QUERY_DEVICE_TEXT\0".as_ptr(),
        crate::driver_ddk::IRP_MN_FILTER_RESOURCE_REQUIREMENTS => b"IRP_MN_FILTER_RESOURCE_REQUIREMENTS\0".as_ptr(),
        crate::driver_ddk::IRP_MN_READ_CONFIG => b"IRP_MN_READ_CONFIG\0".as_ptr(),
        crate::driver_ddk::IRP_MN_WRITE_CONFIG => b"IRP_MN_WRITE_CONFIG\0".as_ptr(),
        crate::driver_ddk::IRP_MN_EJECT => b"IRP_MN_EJECT\0".as_ptr(),
        crate::driver_ddk::IRP_MN_SET_LOCK => b"IRP_MN_SET_LOCK\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_ID => b"IRP_MN_QUERY_ID\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_PNP_DEVICE_STATE => b"IRP_MN_QUERY_PNP_DEVICE_STATE\0".as_ptr(),
        crate::driver_ddk::IRP_MN_QUERY_BUS_INFORMATION => b"IRP_MN_QUERY_BUS_INFORMATION\0".as_ptr(),
        crate::driver_ddk::IRP_MN_DEVICE_USAGE_NOTIFICATION => b"IRP_MN_DEVICE_USAGE_NOTIFICATION\0".as_ptr(),
        crate::driver_ddk::IRP_MN_SURPRISE_REMOVAL => b"IRP_MN_SURPRISE_REMOVAL\0".as_ptr(),
        IRP_MN_QUERY_LEGACY_BUS_INFORMATION => b"IRP_MN_QUERY_LEGACY_BUS_INFORMATION\0".as_ptr(),
        _ => b"(unknown)IRP_MN\0".as_ptr(),
    };
    s as PCHAR
}
