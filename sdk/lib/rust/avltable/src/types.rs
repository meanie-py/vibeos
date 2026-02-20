//! C-ABI types for RTL AVL Generic Table.
//!
//! All structs use `#[repr(C)]` to match the Windows SDK layout exactly.
//! Callback types use `extern "system"` which maps to `__stdcall` on i386
//! and `__cdecl` on amd64/arm64 — matching NTAPI.

use core::ffi::c_void;

// --- Compare results (RTL_GENERIC_COMPARE_RESULTS enum values) ---
pub const GENERIC_LESS_THAN: u32 = 0;
pub const GENERIC_GREATER_THAN: u32 = 1;
pub const GENERIC_EQUAL: u32 = 2;

// --- Table search results (TABLE_SEARCH_RESULT enum values) ---
pub const TABLE_EMPTY_TREE: u32 = 0;
pub const TABLE_FOUND_NODE: u32 = 1;
pub const TABLE_INSERT_AS_LEFT: u32 = 2;
pub const TABLE_INSERT_AS_RIGHT: u32 = 3;

// --- AVL balance factors (RTL_AVL_BALANCE_FACTOR) ---
pub const LEFT_HEAVY: i8 = -1;
pub const BALANCED: i8 = 0;
pub const RIGHT_HEAVY: i8 = 1;

// --- Callback types (NTAPI = extern "system") ---

pub type AvlCompareRoutine = unsafe extern "system" fn(
    table: *mut RtlAvlTable,
    first: *const c_void,
    second: *const c_void,
) -> u32;

pub type AvlAllocateRoutine = unsafe extern "system" fn(
    table: *mut RtlAvlTable,
    byte_size: u32,
) -> *mut c_void;

pub type AvlFreeRoutine = unsafe extern "system" fn(
    table: *mut RtlAvlTable,
    buffer: *mut c_void,
);

// --- RTL_BALANCED_LINKS (AVL tree node header) ---

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RtlBalancedLinks {
    pub parent: *mut RtlBalancedLinks,
    pub left_child: *mut RtlBalancedLinks,
    pub right_child: *mut RtlBalancedLinks,
    pub balance: i8,
    pub reserved: [u8; 3],
}

// --- RTL_AVL_TABLE ---

#[repr(C)]
pub struct RtlAvlTable {
    pub balanced_root: RtlBalancedLinks,
    pub ordered_pointer: *mut c_void,
    pub which_ordered_element: u32,
    pub number_generic_table_elements: u32,
    pub depth_of_tree: u32,
    pub restart_key: *mut RtlBalancedLinks,
    pub delete_count: u32,
    pub compare_routine: Option<AvlCompareRoutine>,
    pub allocate_routine: Option<AvlAllocateRoutine>,
    pub free_routine: Option<AvlFreeRoutine>,
    pub table_context: *mut c_void,
}

// --- TABLE_ENTRY_HEADER (internal, for offset calculation) ---

#[repr(C)]
#[allow(dead_code)]
struct TableEntryHeader {
    _balanced_links: RtlBalancedLinks,
    _user_data: i64,
}

/// Offset from node start to user data area.
/// Matches `FIELD_OFFSET(TABLE_ENTRY_HEADER, UserData)` in avlsupp.c.
pub const NODE_HEADER_SIZE: usize = core::mem::size_of::<RtlBalancedLinks>();

// --- Compile-time size assertions ---

// Verify no padding between BalancedLinks and UserData
const _: () = assert!(
    core::mem::size_of::<TableEntryHeader>()
        == core::mem::size_of::<RtlBalancedLinks>() + core::mem::size_of::<i64>()
);

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<RtlBalancedLinks>() == 32);
    assert!(core::mem::size_of::<RtlAvlTable>() == 104);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<RtlBalancedLinks>() == 16);
    assert!(core::mem::size_of::<RtlAvlTable>() == 56);
};
