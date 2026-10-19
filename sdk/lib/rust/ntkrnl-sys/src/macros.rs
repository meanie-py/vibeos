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

// Windows 8 handle reference biasing. Creating a handle charges the object OBP_HANDLE_BIAS
// references: one the handle holds, and BIAS-1 cached in the entry, which
// ObReferenceObjectByHandle draws down instead of incrementing PointerCount and which
// closing the handle returns. So PointerCount reads 65537 for two handles plus a pointer
// reference on x64 (65 on x86), as ObjectBasicInformation shows on Windows (kmtest ObHandle,
// MmSection, ZwMapViewOfSection). The cache lives where Windows keeps it: on 64-bit in the
// object pointer's top 16 bits (kernel pointers are canonical, so those bits are otherwise
// all ones; stored inverted, so a plain pointer means 0 cached); on x86 in GrantedAccess bits
// 26-30, above ObpAccessProtectCloseBit (25). Entries are read through ObpGetHandleObject
// and ObpHandleGrantedAccess, which strip it.
#[cfg(target_pointer_width = "64")]
pub const OBP_HANDLE_BIAS: usize = 0x8000;
#[cfg(target_pointer_width = "32")]
pub const OBP_HANDLE_BIAS: usize = 32;
#[cfg(target_pointer_width = "64")]
const OBP_HANDLE_REFS_SHIFT: u32 = 48;
#[cfg(target_pointer_width = "32")]
const OBP_HANDLE_REFS_SHIFT: u32 = 26;
#[cfg(target_pointer_width = "32")]
const OBP_HANDLE_REFS_MASK: u32 = 0x1F << OBP_HANDLE_REFS_SHIFT;

/// ObpGetHandleObject — extract POBJECT_HEADER from a handle table entry.
/// C: `(POBJECT_HEADER)((ULONG_PTR)x->Object & ~OBJ_HANDLE_ATTRIBUTES)`; on 64-bit the top
/// 16 bits are restored too (the cached-reference count, above).
#[inline]
pub unsafe fn ObpGetHandleObject(
    entry: ddk::PHANDLE_TABLE_ENTRY,
) -> ddk::POBJECT_HEADER {
    let obj_ptr = (*entry).__bindgen_anon_1.Object as usize;
    if obj_ptr == 0 {
        return core::ptr::null_mut();
    }
    #[cfg(target_pointer_width = "64")]
    let obj_ptr = obj_ptr | (0xFFFFusize << OBP_HANDLE_REFS_SHIFT);
    (obj_ptr & !(OBJ_HANDLE_ATTRIBUTES as usize)) as ddk::POBJECT_HEADER
}

/// The entry's GrantedAccess without the x86 cached-reference bits.
#[inline]
pub unsafe fn ObpHandleGrantedAccess(entry: ddk::PHANDLE_TABLE_ENTRY) -> u32 {
    let access = (*entry).__bindgen_anon_2.GrantedAccess;
    #[cfg(target_pointer_width = "32")]
    let access = access & !OBP_HANDLE_REFS_MASK;
    access
}

/// References cached in the entry (see OBP_HANDLE_BIAS). The entry must be locked or new.
#[inline]
pub unsafe fn ObpHandleCachedRefs(entry: ddk::PHANDLE_TABLE_ENTRY) -> usize {
    #[cfg(target_pointer_width = "64")]
    {
        !(((*entry).__bindgen_anon_1.Object as usize >> OBP_HANDLE_REFS_SHIFT) as u16) as usize
    }
    #[cfg(target_pointer_width = "32")]
    {
        (((*entry).__bindgen_anon_2.GrantedAccess & OBP_HANDLE_REFS_MASK) >> OBP_HANDLE_REFS_SHIFT)
            as usize
    }
}

/// Store the cached-reference count. On 64-bit only the top 16 bits are written, so the lock
/// bit ExUnlockHandleTableEntry ORs into the low dword is not raced. Locked or new entries only.
#[inline]
pub unsafe fn ObpSetHandleCachedRefs(entry: ddk::PHANDLE_TABLE_ENTRY, refs: usize) {
    #[cfg(target_pointer_width = "64")]
    {
        let top = (core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.Object) as *mut u16).add(3);
        core::ptr::write_volatile(top, !(refs as u16));
    }
    #[cfg(target_pointer_width = "32")]
    {
        let access = (*entry).__bindgen_anon_2.GrantedAccess & !OBP_HANDLE_REFS_MASK;
        (*entry).__bindgen_anon_2.GrantedAccess =
            access | (((refs as u32) << OBP_HANDLE_REFS_SHIFT) & OBP_HANDLE_REFS_MASK);
    }
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
