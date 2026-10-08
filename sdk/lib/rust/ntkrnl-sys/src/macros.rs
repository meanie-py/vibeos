//! Inline functions replacing C macros and FORCEINLINE functions.
//!
//! These are C preprocessor macros or `FORCEINLINE` functions that bindgen
//! cannot capture. Implemented as `#[inline]` Rust functions.

use crate::base::*;
use crate::constants::*;
use crate::ddk;

// ============================================================================
// Object header macros (from obtypes.h)
// ============================================================================

/// OBJECT_TO_OBJECT_HEADER(o) — get OBJECT_HEADER from object body pointer.
/// C: `CONTAINING_RECORD(o, OBJECT_HEADER, Body)`
#[inline]
pub unsafe fn OBJECT_TO_OBJECT_HEADER(o: PVOID) -> ddk::POBJECT_HEADER {
    // Use wrapping_sub instead of sub to avoid generating `getelementptr inbounds`
    // with a negative offset. LLVM treats `inbounds` GEP with negative offsets as UB
    // if it can't prove the result stays within the allocation, which causes it to
    // insert `unreachable` and eliminate subsequent code (ud2 crash on i386).
    // wrapping_sub generates a plain GEP without `inbounds`, preventing this.
    (o as *mut u8).wrapping_sub(core::mem::offset_of!(ddk::_OBJECT_HEADER, Body))
        as ddk::POBJECT_HEADER
}

/// OBJECT_HEADER_TO_NAME_INFO(h) — get name info from header, or null.
/// C: `(h->NameInfoOffset ? (PCHAR)h - h->NameInfoOffset : NULL)`
#[inline]
pub unsafe fn OBJECT_HEADER_TO_NAME_INFO(
    h: ddk::POBJECT_HEADER,
) -> ddk::POBJECT_HEADER_NAME_INFO {
    let offset = (*h).NameInfoOffset;
    if offset == 0 {
        core::ptr::null_mut()
    } else {
        (h as *mut u8).sub(offset as usize) as ddk::POBJECT_HEADER_NAME_INFO
    }
}

/// OBJECT_HEADER_TO_HANDLE_INFO(h)
#[inline]
pub unsafe fn OBJECT_HEADER_TO_HANDLE_INFO(
    h: ddk::POBJECT_HEADER,
) -> ddk::POBJECT_HEADER_HANDLE_INFO {
    let offset = (*h).HandleInfoOffset;
    if offset == 0 {
        core::ptr::null_mut()
    } else {
        (h as *mut u8).sub(offset as usize) as ddk::POBJECT_HEADER_HANDLE_INFO
    }
}

/// OBJECT_HEADER_TO_QUOTA_INFO(h)
#[inline]
pub unsafe fn OBJECT_HEADER_TO_QUOTA_INFO(
    h: ddk::POBJECT_HEADER,
) -> ddk::POBJECT_HEADER_QUOTA_INFO {
    let offset = (*h).QuotaInfoOffset;
    if offset == 0 {
        core::ptr::null_mut()
    } else {
        (h as *mut u8).sub(offset as usize) as ddk::POBJECT_HEADER_QUOTA_INFO
    }
}

/// OBJECT_HEADER_TO_CREATOR_INFO(h)
#[inline]
pub unsafe fn OBJECT_HEADER_TO_CREATOR_INFO(
    h: ddk::POBJECT_HEADER,
) -> ddk::POBJECT_HEADER_CREATOR_INFO {
    if (*h).Flags & OB_FLAG_CREATOR_INFO == 0 {
        core::ptr::null_mut()
    } else {
        (h as *mut u8).sub(core::mem::size_of::<ddk::_OBJECT_HEADER_CREATOR_INFO>())
            as ddk::POBJECT_HEADER_CREATOR_INFO
    }
}

/// OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(h)
#[inline]
pub unsafe fn OBJECT_HEADER_TO_EXCLUSIVE_PROCESS(
    h: ddk::POBJECT_HEADER,
) -> ddk::PEPROCESS {
    if (*h).Flags & OB_FLAG_EXCLUSIVE as u8 == 0 {
        core::ptr::null_mut()
    } else {
        let qi = OBJECT_HEADER_TO_QUOTA_INFO(h);
        if qi.is_null() {
            core::ptr::null_mut()
        } else {
            (*qi).ExclusiveProcess
        }
    }
}

// ============================================================================
// Handle entry macros (from ob.h)
// ============================================================================

/// ObpGetHandleObject — extract POBJECT_HEADER from a handle table entry.
/// C: `(POBJECT_HEADER)((ULONG_PTR)x->Object & ~OBJ_HANDLE_ATTRIBUTES)`
#[inline]
pub unsafe fn ObpGetHandleObject(
    entry: ddk::PHANDLE_TABLE_ENTRY,
) -> ddk::POBJECT_HEADER {
    let obj_ptr = (*entry).__bindgen_anon_1.Object as usize;
    (obj_ptr & !(OBJ_HANDLE_ATTRIBUTES as usize)) as ddk::POBJECT_HEADER
}

// ============================================================================
// Kernel handle macros (from ob.h)
// ============================================================================

#[cfg(target_pointer_width = "64")]
const KERNEL_HANDLE_FLAG: usize = 0xFFFFFFFF80000000;

#[cfg(target_pointer_width = "32")]
const KERNEL_HANDLE_FLAG: usize = 0x80000000;

/// ObpIsKernelHandle — check if handle is a kernel handle.
#[inline]
pub unsafe fn ObpIsKernelHandle(Handle: ddk::HANDLE, ProcessorMode: i8) -> BOOLEAN {
    if ((Handle as usize) & KERNEL_HANDLE_FLAG) == KERNEL_HANDLE_FLAG
        && ProcessorMode == KernelMode
        && Handle != NtCurrentProcess()
        && Handle != NtCurrentThread()
    {
        TRUE
    } else {
        FALSE
    }
}

/// ObKernelHandleToHandle — strip kernel flag from handle.
#[inline]
pub fn ObKernelHandleToHandle(Handle: ddk::HANDLE) -> ddk::HANDLE {
    ((Handle as usize) & !KERNEL_HANDLE_FLAG) as ddk::HANDLE
}

/// ObMarkHandleAsKernelHandle — set kernel flag on handle.
#[inline]
pub fn ObMarkHandleAsKernelHandle(Handle: ddk::HANDLE) -> ddk::HANDLE {
    ((Handle as usize) | KERNEL_HANDLE_FLAG) as ddk::HANDLE
}

// ============================================================================
// Handle pseudo-values (from pstypes.h)
// ============================================================================

/// NtCurrentProcess() — pseudo-handle for current process (-1).
#[inline]
pub fn NtCurrentProcess() -> ddk::HANDLE {
    (-1isize) as ddk::HANDLE
}

/// NtCurrentThread() — pseudo-handle for current thread (-2).
#[inline]
pub fn NtCurrentThread() -> ddk::HANDLE {
    (-2isize) as ddk::HANDLE
}

/// HandleToLong — cast handle to LONG.
#[inline]
pub fn HandleToLong(h: ddk::HANDLE) -> LONG {
    h as isize as LONG
}

// ============================================================================
// Probe macros (from probe.h)
// ============================================================================

/// ProbeForWriteHandle — validate user-mode handle pointer: probe.h's range
/// check (`crate::probe::probe_user_range`), then the touch-test write-back.
/// This copy used to skip the range check entirely.
#[inline]
pub unsafe fn ProbeForWriteHandle(Ptr: *mut ddk::HANDLE) {
    crate::probe::probe_user_range(Ptr);
    core::ptr::write_volatile(Ptr, core::ptr::read_volatile(Ptr));
}

// ============================================================================
// Mode helpers (from ketypes.h)
// ============================================================================

extern "system" {
    /// KeGetPreviousMode — get previous processor mode.
    pub fn KeGetPreviousMode() -> i8;
}

/// ExGetPreviousMode — alias for KeGetPreviousMode.
#[inline]
pub unsafe fn ExGetPreviousMode() -> i8 {
    KeGetPreviousMode()
}

// ============================================================================
// Extern declarations for kernel globals
// ============================================================================

extern "system" {
    /// PsProcessType — the process object type.
    pub static mut PsProcessType: ddk::POBJECT_TYPE;

    /// ObpSymbolicLinkObjectType — symbolic link object type (internal).
    pub static mut ObpSymbolicLinkObjectType: ddk::POBJECT_TYPE;

    /// KdDebuggerEnabled — TRUE if kernel debugger is active.
    pub static mut KdDebuggerEnabled: BOOLEAN;
}
// Note: PsInitialSystemProcess is in mm.rs, DbgBreakPoint is in ke.rs

// ============================================================================
// OB subsystem extern globals
// ============================================================================

extern "C" {
    // Object types
    pub static mut ObpDirectoryObjectType: ddk::POBJECT_TYPE;
    pub static mut ObpTypeDirectoryObject: ddk::POBJECT_DIRECTORY;

    // Root directory and device maps
    pub static mut ObpRootDirectoryObject: ddk::POBJECT_DIRECTORY;
    pub static mut ObSystemDeviceMap: ddk::PDEVICE_MAP;
    pub static mut ObpDeviceMapLock: ddk::_KGUARDED_MUTEX;
    pub static mut ObpLUIDDeviceMapsEnabled: ULONG;

    // Handle table
    pub static mut ObpKernelHandleTable: ddk::PHANDLE_TABLE;

    // Lookaside lists
    pub static mut ObpNameBufferLookasideList: ddk::_GENERAL_LOOKASIDE;
    pub static mut ObpCreateInfoLookasideList: ddk::_GENERAL_LOOKASIDE;
    pub static mut ExSystemLookasideListHead: crate::base::LIST_ENTRY;

    // Reaper
    pub static mut ObpReaperList: *mut core::ffi::c_void;
    pub static mut ObpReaperWorkItem: ddk::WORK_QUEUE_ITEM;

    // Short name for DosDevices
    pub static mut ObpDosDevicesShortName: ddk::_UNICODE_STRING;
    pub static mut ObpDosDevicesShortNamePrefix: ddk::_UNICODE_STRING;
    pub static mut ObpDosDevicesShortNameRoot: ddk::_UNICODE_STRING;

    // Security SIDs
    pub static mut SeWorldSid: ddk::PSID;
    pub static mut SeLocalSystemSid: ddk::PSID;
    pub static mut SeAliasAdminsSid: ddk::PSID;
    pub static mut SeRestrictedSid: ddk::PSID;

    // Security privileges
    pub static mut SeTcbPrivilege: ddk::LUID;
    pub static mut SeCreatePermanentPrivilege: ddk::LUID;
    pub static mut SeSecurityPrivilege: ddk::LUID;
}
