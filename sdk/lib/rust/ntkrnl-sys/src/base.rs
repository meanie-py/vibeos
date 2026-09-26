//! Fundamental Windows kernel types.
//!
//! Derived from `sdk/include/xdk/ntbasedef.h`, `sdk/include/psdk/ntdef.h`,
//! and `sdk/include/xdk/winnt_old.h`.

use core::ffi::c_void;

// ============================================================================
// Scalar types
// ============================================================================

pub type NTSTATUS = i32;
pub type PVOID = *mut c_void;
pub type PCVOID = *const c_void;
pub type ULONG = u32;
pub type LONG = i32;
pub type USHORT = u16;
pub type SHORT = i16;
pub type UCHAR = u8;
pub type CHAR = i8;
pub type BOOLEAN = u8;
pub type WCHAR = u16;
pub type KIRQL = u8;
pub type SIZE_T = usize;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;
pub type ULONGLONG = u64;
pub type LONGLONG = i64;

/// PFN_NUMBER — page frame number (pointer-width unsigned).
pub type PFN_NUMBER = ULONG_PTR;

/// PFN_COUNT — page count (always 32-bit).
pub type PFN_COUNT = ULONG;

// Pointer types
pub type PULONG = *mut ULONG;
pub type PLONG = *mut LONG;
pub type PUSHORT = *mut USHORT;
pub type PUCHAR = *mut UCHAR;
pub type PSIZE_T = *mut SIZE_T;
pub type PULONG_PTR = *mut ULONG_PTR;
pub type PBOOLEAN = *mut BOOLEAN;
pub type PWSTR = *mut WCHAR;

// ============================================================================
// Status codes and constants
// ============================================================================

pub const STATUS_SUCCESS: NTSTATUS = 0;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as i32;
pub const STATUS_ALREADY_COMMITTED: NTSTATUS = 0xC0000021_u32 as i32;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017_u32 as i32;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002_u32 as i32;
// 0x8... is a WARNING-severity status, not 0xC... . ARM3/virtual.rs carried
// 0xC000009B here, which is STATUS_DFS_EXIT_PATH_FOUND -- one past
// STATUS_INSUFFICIENT_RESOURCES above, and a plausible-looking neighbour.
pub const STATUS_PARTIAL_COPY: NTSTATUS = 0x8000000D_u32 as i32;

#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// PCHAR — pointer to 8-bit character.
pub type PCHAR = *mut CHAR;
/// PWCHAR — pointer to wide character.
pub type PWCHAR = *mut WCHAR;
/// PCWSTR — const pointer to wide string.
pub type PCWSTR = *const WCHAR;

pub const TRUE: BOOLEAN = 1;
pub const FALSE: BOOLEAN = 0;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SHIFT: u32 = 12;

#[inline]
pub fn PAGE_ALIGN(va: usize) -> usize {
    va & !(PAGE_SIZE - 1)
}

#[inline]
pub fn BYTES_TO_PAGES(size: usize) -> usize {
    (size + PAGE_SIZE - 1) >> PAGE_SHIFT as usize
}

#[inline]
pub fn ROUND_TO_PAGES(size: usize) -> usize {
    (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

// Size constants
pub const _1KB: usize = 0x400;
pub const _1MB: usize = 0x100000;
pub const _1GB: usize = 0x40000000;

// IRQL levels
pub const PASSIVE_LEVEL: KIRQL = 0;
pub const APC_LEVEL: KIRQL = 1;
pub const DISPATCH_LEVEL: KIRQL = 2;

// Tag for untagged allocations
pub const TAG_NONE: ULONG = u32::from_le_bytes(*b"None");

// Protected pool flag (bit 31 of tag)
pub const PROTECTED_POOL: ULONG = 0x80000000;

// Pool type modifier flags (from ntdef.h / extypes.h)
pub const POOL_RAISE_IF_ALLOCATION_FAILURE: ULONG = 16;
pub const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ULONG = 8;

// Memory allocation flags
pub const MEM_COMMIT: ULONG = 0x1000;
pub const MEM_RESERVE: ULONG = 0x2000;
pub const MEM_RELEASE: ULONG = 0x8000;
pub const PAGE_READWRITE: ULONG = 0x04;

// ============================================================================
// LIST_ENTRY — doubly-linked list node (unified with ddk::_LIST_ENTRY)
// ============================================================================

pub use crate::ddk::_LIST_ENTRY as LIST_ENTRY;
pub type PLIST_ENTRY = *mut LIST_ENTRY;

impl crate::ddk::_LIST_ENTRY {
    pub const fn new() -> Self {
        Self {
            Flink: core::ptr::null_mut(),
            Blink: core::ptr::null_mut(),
        }
    }
}

/// Initialize a list head to point to itself.
#[inline]
pub unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}

/// Check if a list is empty.
#[inline]
pub unsafe fn IsListEmpty(head: *const LIST_ENTRY) -> bool {
    (*head).Flink == head as *mut LIST_ENTRY
}

/// Insert entry at the head of the list.
#[inline]
pub unsafe fn InsertHeadList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let flink = (*head).Flink;
    (*entry).Flink = flink;
    (*entry).Blink = head;
    (*flink).Blink = entry;
    (*head).Flink = entry;
}

/// Insert entry at the tail of the list.
#[inline]
pub unsafe fn InsertTailList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;
}

/// Remove entry from its list.
#[inline]
pub unsafe fn RemoveEntryList(entry: *mut LIST_ENTRY) -> bool {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
    flink == blink
}

/// Remove and return the entry at the head of a list.
#[inline]
pub unsafe fn RemoveHeadList(list_head: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    let entry = (*list_head).Flink;
    let flink = (*entry).Flink;
    (*list_head).Flink = flink;
    (*flink).Blink = list_head;
    entry
}

/// Remove and return the entry at the tail of a list.
#[inline]
pub unsafe fn RemoveTailList(list_head: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    let entry = (*list_head).Blink;
    let blink = (*entry).Blink;
    (*list_head).Blink = blink;
    (*blink).Flink = list_head;
    entry
}

/// CONTAINING_RECORD — get struct pointer from field pointer.
/// Usage: `CONTAINING_RECORD!(entry_ptr, StructType, field_name)`
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $field:ident) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $field)) as *mut $type
    };
}

// ============================================================================
// SINGLE_LIST_ENTRY
// ============================================================================

#[repr(C)]
pub struct SINGLE_LIST_ENTRY {
    pub Next: *mut SINGLE_LIST_ENTRY,
}

pub type PSINGLE_LIST_ENTRY = *mut SINGLE_LIST_ENTRY;

// ============================================================================
// SLIST_HEADER — interlocked singly-linked list header
// ============================================================================

/// On 64-bit: 16-byte aligned, contains packed Depth/Sequence/NextEntry.
/// On 32-bit: 8 bytes (ULONGLONG).
/// We represent this as an opaque aligned byte array since the exact layout
/// is handled by the SList intrinsics (RtlInterlockedPopEntrySList, etc.).
#[cfg(target_pointer_width = "64")]
#[repr(C, align(16))]
pub struct SLIST_HEADER {
    pub _opaque: [u64; 2],
}

#[cfg(target_pointer_width = "32")]
#[repr(C, align(8))]
pub struct SLIST_HEADER {
    pub _opaque: [u32; 2],
}

pub type PSLIST_HEADER = *mut SLIST_HEADER;

impl SLIST_HEADER {
    pub const fn zeroed() -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            Self { _opaque: [0; 2] }
        }
        #[cfg(target_pointer_width = "32")]
        {
            Self { _opaque: [0; 2] }
        }
    }
}

// ============================================================================
// LARGE_INTEGER
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub union LARGE_INTEGER {
    pub s: LARGE_INTEGER_PARTS,
    pub QuadPart: LONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LARGE_INTEGER_PARTS {
    pub LowPart: ULONG,
    pub HighPart: LONG,
}

// ============================================================================
// UNICODE_STRING
// ============================================================================

#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PWSTR,
}

// ============================================================================
// SLIST_ENTRY
// ============================================================================

#[cfg(target_pointer_width = "64")]
#[repr(C, align(16))]
pub struct SLIST_ENTRY {
    pub Next: *mut SLIST_ENTRY,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub struct SLIST_ENTRY {
    pub Next: *mut SLIST_ENTRY,
}

pub type PSLIST_ENTRY = *mut SLIST_ENTRY;

// ============================================================================
// Bugcheck codes used by the pool allocator
// ============================================================================

pub const BAD_POOL_HEADER: ULONG = 0x19;
pub const BAD_POOL_CALLER: ULONG = 0xC2;
pub const MUST_SUCCEED_POOL_EMPTY: ULONG = 0x41;
pub const DRIVER_CORRUPTED_EXPOOL: ULONG = 0xC5;

// ============================================================================
// Lock queue numbers
// ============================================================================

/// KSPIN_LOCK_QUEUE_NUMBER values used by pool allocator.
pub const LockQueueNonPagedPoolLock: ULONG = 6;
pub const LockQueueMmNonPagedPoolLock: ULONG = 15;

// ============================================================================
// Compile-time alignment assertions (size assertions in size_check_*.rs)
// ============================================================================

const _: () = {
    #[cfg(target_pointer_width = "64")]
    assert!(core::mem::align_of::<SLIST_HEADER>() == 16);

    #[cfg(target_pointer_width = "32")]
    assert!(core::mem::align_of::<SLIST_HEADER>() == 8);
};
