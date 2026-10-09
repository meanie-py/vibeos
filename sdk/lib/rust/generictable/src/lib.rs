//! RTL Generic Table (Splay Tree) — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/generictable.c` (613 lines).
//! Thirteen exported functions for a splay-tree-based generic table with
//! NT-style callback allocation/comparison.
//! All architectures.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

use core::ffi::c_void;

// ============================================================================
// Constants
// ============================================================================

const PAGE_SIZE: usize = 0x1000;

// RTL_GENERIC_COMPARE_RESULTS
const GENERIC_LESS_THAN: u32 = 0;
const GENERIC_GREATER_THAN: u32 = 1;
const GENERIC_EQUAL: u32 = 2;

// TABLE_SEARCH_RESULT
const TABLE_EMPTY_TREE: u32 = 0;
const TABLE_FOUND_NODE: u32 = 1;
const TABLE_INSERT_AS_LEFT: u32 = 2;
const TABLE_INSERT_AS_RIGHT: u32 = 3;

// ============================================================================
// Types
// ============================================================================

/// RTL_SPLAY_LINKS — matches splaytree crate's SplayLinks.
#[repr(C)]
pub struct SplayLinks {
    pub parent: *mut SplayLinks,
    pub left_child: *mut SplayLinks,
    pub right_child: *mut SplayLinks,
}

/// LIST_ENTRY — doubly-linked list node.
#[repr(C)]
pub struct ListEntry {
    pub flink: *mut ListEntry,
    pub blink: *mut ListEntry,
}

/// TABLE_ENTRY_HEADER — internal node layout.
#[repr(C)]
pub struct TableEntryHeader {
    pub splay_links: SplayLinks,
    pub list_entry: ListEntry,
    pub user_data: i64, // LONGLONG UserData — start of user data region
}

/// Callback type: compare two user data buffers.
pub type CompareRoutine = unsafe extern "system" fn(
    table: *mut RtlGenericTable,
    first: *const c_void,
    second: *const c_void,
) -> u32;

/// Callback type: allocate memory for a new entry.
pub type AllocateRoutine = unsafe extern "system" fn(
    table: *mut RtlGenericTable,
    byte_size: u32,
) -> *mut c_void;

/// Callback type: free memory for a deleted entry.
pub type FreeRoutine = unsafe extern "system" fn(
    table: *mut RtlGenericTable,
    buffer: *mut c_void,
);

/// RTL_GENERIC_TABLE — splay-tree-based generic table.
#[repr(C)]
pub struct RtlGenericTable {
    pub table_root: *mut SplayLinks,
    pub insert_order_list: ListEntry,
    pub ordered_pointer: *mut ListEntry,
    pub which_ordered_element: u32,
    pub number_generic_table_elements: u32,
    pub compare_routine: CompareRoutine,
    pub allocate_routine: AllocateRoutine,
    pub free_routine: FreeRoutine,
    pub table_context: *mut c_void,
}

// ============================================================================
// SplayLinks helpers
// ============================================================================

#[inline]
unsafe fn sl_parent(n: *mut SplayLinks) -> *mut SplayLinks { (*n).parent }
#[inline]
unsafe fn sl_left(n: *mut SplayLinks) -> *mut SplayLinks { (*n).left_child }
#[inline]
unsafe fn sl_right(n: *mut SplayLinks) -> *mut SplayLinks { (*n).right_child }
#[inline]
unsafe fn sl_is_root(n: *mut SplayLinks) -> bool { (*n).parent == n }
#[inline]
unsafe fn sl_init(n: *mut SplayLinks) {
    (*n).parent = n;
    (*n).left_child = core::ptr::null_mut();
    (*n).right_child = core::ptr::null_mut();
}
#[inline]
unsafe fn sl_insert_left(parent: *mut SplayLinks, child: *mut SplayLinks) {
    (*parent).left_child = child;
    (*child).parent = parent;
}
#[inline]
unsafe fn sl_insert_right(parent: *mut SplayLinks, child: *mut SplayLinks) {
    (*parent).right_child = child;
    (*child).parent = parent;
}

// ============================================================================
// LIST_ENTRY helpers
// ============================================================================

#[inline]
unsafe fn list_init(head: *mut ListEntry) {
    (*head).flink = head;
    (*head).blink = head;
}

#[inline]
unsafe fn list_insert_tail(head: *mut ListEntry, entry: *mut ListEntry) {
    (*entry).flink = head;
    (*entry).blink = (*head).blink;
    (*(*head).blink).flink = entry;
    (*head).blink = entry;
}

#[inline]
unsafe fn list_remove(entry: *mut ListEntry) {
    (*(*entry).flink).blink = (*entry).blink;
    (*(*entry).blink).flink = (*entry).flink;
}

// ============================================================================
// Node ↔ user data conversion
// ============================================================================

/// Get pointer to user data from a SplayLinks (TABLE_ENTRY_HEADER) pointer.
#[inline]
unsafe fn node_to_user_data(node: *mut SplayLinks) -> *mut c_void {
    &mut (*(node as *mut TableEntryHeader)).user_data as *mut i64 as *mut c_void
}

/// Get pointer to ListEntry from a SplayLinks (TABLE_ENTRY_HEADER) pointer.
#[inline]
unsafe fn node_to_list_entry(node: *mut SplayLinks) -> *mut ListEntry {
    &mut (*(node as *mut TableEntryHeader)).list_entry
}

/// Get pointer to SplayLinks from a ListEntry.
#[inline]
unsafe fn list_entry_to_node(entry: *mut ListEntry) -> *mut SplayLinks {
    // ListEntry is at offset size_of::<SplayLinks>() in TableEntryHeader
    (entry as *mut u8).sub(core::mem::size_of::<SplayLinks>()) as *mut SplayLinks
}

/// Get user data pointer from a ListEntry.
#[inline]
unsafe fn list_entry_to_user_data(entry: *mut ListEntry) -> *mut c_void {
    let node = list_entry_to_node(entry);
    node_to_user_data(node)
}

/// User data field offset in TABLE_ENTRY_HEADER — the C's
/// `FIELD_OFFSET(TABLE_ENTRY_HEADER, UserData)`. It is NOT the sum of the two
/// link structs: `UserData` is a LONGLONG, 8-aligned on every target, so on
/// i386 it sits at 24, not 20. Sizing nodes by the sum allocated 4 bytes short
/// and every insert wrote past its pool block (i386 BUGCHECK 0x19 in
/// RtlSplayTreeKM); 64-bit was unaffected only because both come to 40 there.
#[inline]
fn user_data_offset() -> u32 {
    core::mem::offset_of!(TableEntryHeader, user_data) as u32
}

// ============================================================================
// Extern splaytree functions (resolved at link time)
// ============================================================================

#[cfg(not(test))]
extern "system" {
    fn RtlSplay(links: *mut SplayLinks) -> *mut SplayLinks;
    fn RtlDelete(links: *mut SplayLinks) -> *mut SplayLinks;
    fn RtlRealSuccessor(links: *mut SplayLinks) -> *mut SplayLinks;
    fn RtlSubtreePredecessor(links: *mut SplayLinks) -> *mut SplayLinks;
}

// In test mode, forward to the splaytree crate's pub Rust functions (added as a
// dev-dependency). Its `SplayLinks` is layout-identical (#[repr(C)], three
// pointers), so the raw pointers cast directly — exactly the ABI compatibility
// that lets the kernel link generictable against splaytree's symbols. This makes
// the functional model tests exercise the real splay algorithm.
#[cfg(test)]
unsafe fn RtlSplay(l: *mut SplayLinks) -> *mut SplayLinks {
    splaytree::splay(l as *mut splaytree::SplayLinks) as *mut SplayLinks
}
#[cfg(test)]
unsafe fn RtlDelete(l: *mut SplayLinks) -> *mut SplayLinks {
    splaytree::delete(l as *mut splaytree::SplayLinks) as *mut SplayLinks
}
#[cfg(test)]
unsafe fn RtlRealSuccessor(l: *mut SplayLinks) -> *mut SplayLinks {
    splaytree::real_successor(l as *mut splaytree::SplayLinks) as *mut SplayLinks
}
#[cfg(test)]
unsafe fn RtlSubtreePredecessor(l: *mut SplayLinks) -> *mut SplayLinks {
    splaytree::subtree_predecessor(l as *mut splaytree::SplayLinks) as *mut SplayLinks
}

// ============================================================================
// Private functions
// ============================================================================

unsafe fn find_node_or_parent(
    table: *mut RtlGenericTable,
    buffer: *const c_void,
    node_or_parent: *mut *mut SplayLinks,
) -> u32 {

    // Empty tree check
    if (*table).table_root.is_null() {
        return TABLE_EMPTY_TREE;
    }

    let mut current = (*table).table_root;

    // Guard against corrupt root pointer (DUMMY_PSEH can't catch faults)
    if (current as usize) < PAGE_SIZE {
        (*table).table_root = core::ptr::null_mut();
        return TABLE_EMPTY_TREE;
    }

    loop {
        let result = ((*table).compare_routine)(table, buffer, node_to_user_data(current));

        if result == GENERIC_LESS_THAN {
            let child = sl_left(current);
            if !child.is_null() {
                current = child;
            } else {
                *node_or_parent = current;
                return TABLE_INSERT_AS_LEFT;
            }
        } else if result == GENERIC_GREATER_THAN {
            let child = sl_right(current);
            if !child.is_null() {
                current = child;
            } else {
                *node_or_parent = current;
                return TABLE_INSERT_AS_RIGHT;
            }
        } else {
            // GenericEqual
            *node_or_parent = current;
            return TABLE_FOUND_NODE;
        }
    }
}

// ============================================================================
// Public functions (exported as FFI)
// ============================================================================

pub unsafe fn generic_table_init(
    table: *mut RtlGenericTable,
    compare: CompareRoutine,
    allocate: AllocateRoutine,
    free: FreeRoutine,
    context: *mut c_void,
) {
    list_init(&mut (*table).insert_order_list);
    (*table).table_root = core::ptr::null_mut();
    (*table).number_generic_table_elements = 0;
    (*table).which_ordered_element = 0;
    (*table).ordered_pointer = &mut (*table).insert_order_list;
    (*table).compare_routine = compare;
    (*table).allocate_routine = allocate;
    (*table).free_routine = free;
    (*table).table_context = context;
}

pub unsafe fn generic_table_insert_full(
    table: *mut RtlGenericTable,
    buffer: *const c_void,
    buffer_size: u32,
    new_element: *mut u8,
    node_or_parent: *mut SplayLinks,
    search_result: u32,
) -> *mut c_void {
    let new_node;

    if search_result != TABLE_FOUND_NODE {
        // Allocate a new node
        let alloc_size = buffer_size + user_data_offset();
        let raw = ((*table).allocate_routine)(table, alloc_size);
        if raw.is_null() {
            if !new_element.is_null() {
                *new_element = 0;
            }
            return core::ptr::null_mut();
        }

        new_node = raw as *mut SplayLinks;
        sl_init(new_node);
        list_insert_tail(
            &mut (*table).insert_order_list,
            node_to_list_entry(new_node),
        );

        (*table).number_generic_table_elements += 1;

        if search_result == TABLE_EMPTY_TREE {
            (*table).table_root = new_node;
        } else if search_result == TABLE_INSERT_AS_LEFT {
            sl_insert_left(node_or_parent, new_node);
        } else {
            sl_insert_right(node_or_parent, new_node);
        }

        // Copy user data
        core::ptr::copy_nonoverlapping(
            buffer as *const u8,
            node_to_user_data(new_node) as *mut u8,
            buffer_size as usize,
        );
    } else {
        new_node = node_or_parent;
    }

    // Splay the tree
    (*table).table_root = RtlSplay(new_node);

    if !new_element.is_null() {
        *new_element = if search_result != TABLE_FOUND_NODE { 1 } else { 0 };
    }

    node_to_user_data(new_node)
}

pub unsafe fn generic_table_insert(
    table: *mut RtlGenericTable,
    buffer: *const c_void,
    buffer_size: u32,
    new_element: *mut u8,
) -> *mut c_void {
    let mut node_or_parent: *mut SplayLinks = core::ptr::null_mut();
    let result = find_node_or_parent(table, buffer, &mut node_or_parent);
    generic_table_insert_full(table, buffer, buffer_size, new_element, node_or_parent, result)
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlInitializeGenericTable(
        table: *mut RtlGenericTable,
        compare_routine: CompareRoutine,
        allocate_routine: AllocateRoutine,
        free_routine: FreeRoutine,
        table_context: *mut c_void,
    ) {
        generic_table_init(table, compare_routine, allocate_routine, free_routine, table_context);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlInsertElementGenericTable(
        table: *mut RtlGenericTable,
        buffer: *const c_void,
        buffer_size: u32,
        new_element: *mut u8,
    ) -> *mut c_void {
        generic_table_insert(table, buffer, buffer_size, new_element)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlInsertElementGenericTableFull(
        table: *mut RtlGenericTable,
        buffer: *const c_void,
        buffer_size: u32,
        new_element: *mut u8,
        node_or_parent: *mut c_void,
        search_result: u32,
    ) -> *mut c_void {
        generic_table_insert_full(
            table, buffer, buffer_size, new_element,
            node_or_parent as *mut SplayLinks, search_result,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlIsGenericTableEmpty(
        table: *mut RtlGenericTable,
    ) -> u8 {
        if (*table).table_root.is_null() { 1 } else { 0 }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlNumberGenericTableElements(
        table: *mut RtlGenericTable,
    ) -> u32 {
        (*table).number_generic_table_elements
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLookupElementGenericTable(
        table: *mut RtlGenericTable,
        buffer: *const c_void,
    ) -> *mut c_void {
        let mut node_or_parent: *mut SplayLinks = core::ptr::null_mut();
        let mut search_result: u32 = 0;
        RtlLookupElementGenericTableFull(
            table,
            buffer,
            &mut node_or_parent as *mut _ as *mut *mut c_void,
            &mut search_result,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLookupElementGenericTableFull(
        table: *mut RtlGenericTable,
        buffer: *const c_void,
        node_or_parent: *mut *mut c_void,
        search_result: *mut u32,
    ) -> *mut c_void {
        *search_result = find_node_or_parent(
            table,
            buffer,
            node_or_parent as *mut *mut SplayLinks,
        );

        if *search_result == TABLE_EMPTY_TREE || *search_result != TABLE_FOUND_NODE {
            return core::ptr::null_mut();
        }

        (*table).table_root = RtlSplay(*(node_or_parent as *mut *mut SplayLinks));
        node_to_user_data(*(node_or_parent as *mut *mut SplayLinks))
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDeleteElementGenericTable(
        table: *mut RtlGenericTable,
        buffer: *const c_void,
    ) -> u8 {
        let mut node_or_parent: *mut SplayLinks = core::ptr::null_mut();
        let result = find_node_or_parent(table, buffer, &mut node_or_parent);
        if result != TABLE_FOUND_NODE {
            return 0;
        }

        (*table).table_root = RtlDelete(node_or_parent);
        list_remove(node_to_list_entry(node_or_parent));

        (*table).number_generic_table_elements -= 1;
        (*table).which_ordered_element = 0;
        (*table).ordered_pointer = &mut (*table).insert_order_list;

        ((*table).free_routine)(table, node_or_parent as *mut c_void);
        1
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnumerateGenericTable(
        table: *mut RtlGenericTable,
        restart: u8,
    ) -> *mut c_void {
        if (*table).table_root.is_null() {
            return core::ptr::null_mut();
        }

        let found_node;

        if restart != 0 {
            // Find leftmost node
            let mut node = (*table).table_root;
            while !sl_left(node).is_null() {
                node = sl_left(node);
            }
            (*table).table_root = RtlSplay(node);
            found_node = node;
        } else {
            let node = RtlRealSuccessor((*table).table_root);
            if !node.is_null() {
                (*table).table_root = RtlSplay(node);
            }
            found_node = node;
        }

        if found_node.is_null() {
            core::ptr::null_mut()
        } else {
            node_to_user_data(found_node)
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnumerateGenericTableWithoutSplaying(
        table: *mut RtlGenericTable,
        restart_key: *mut *mut c_void,
    ) -> *mut c_void {
        if (*table).table_root.is_null() {
            return core::ptr::null_mut();
        }

        let found_node;

        if (*restart_key).is_null() {
            // Find leftmost node
            let mut node = (*table).table_root;
            while !sl_left(node).is_null() {
                node = sl_left(node);
            }
            *restart_key = node as *mut c_void;
            found_node = node;
        } else {
            let node = RtlRealSuccessor(*restart_key as *mut SplayLinks);
            if !node.is_null() {
                *restart_key = node as *mut c_void;
            }
            found_node = node;
        }

        if found_node.is_null() {
            core::ptr::null_mut()
        } else {
            node_to_user_data(found_node)
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetElementGenericTable(
        table: *mut RtlGenericTable,
        i: u32,
    ) -> *mut c_void {
        let next_i = i.wrapping_add(1);
        let element_count = (*table).number_generic_table_elements;

        if i == u32::MAX || next_i > element_count {
            return core::ptr::null_mut();
        }

        let mut ordered_node = (*table).ordered_pointer;
        let ordered_element = (*table).which_ordered_element;

        if next_i == ordered_element {
            return list_entry_to_user_data(ordered_node);
        }

        if ordered_element > next_i {
            if next_i > ordered_element / 2 {
                // Walk backwards from current position
                let mut delta = ordered_element - next_i;
                while delta > 0 {
                    ordered_node = (*ordered_node).blink;
                    delta -= 1;
                }
            } else {
                // Walk forwards from list head
                ordered_node = &mut (*table).insert_order_list;
                let mut count = next_i;
                while count > 0 {
                    ordered_node = (*ordered_node).flink;
                    count -= 1;
                }
            }
        } else {
            let delta_up = next_i - ordered_element;
            let delta_down = (element_count - next_i) + 1;

            if delta_up <= delta_down {
                let mut d = delta_up;
                while d > 0 {
                    ordered_node = (*ordered_node).blink;
                    d -= 1;
                }
            } else {
                ordered_node = &mut (*table).insert_order_list;
                let mut d = delta_down;
                while d > 0 {
                    ordered_node = (*ordered_node).blink;
                    d -= 1;
                }
            }
        }

        (*table).ordered_pointer = ordered_node;
        (*table).which_ordered_element = next_i;

        list_entry_to_user_data(ordered_node)
    }

    /// RtlEnumerateGenericTableLikeADirectory — UNIMPLEMENTED stub.
    /// Note: This function takes RTL_AVL_TABLE (not RTL_GENERIC_TABLE) in the
    /// original C code. It's an unimplemented stub placed in the wrong file.
    #[no_mangle]
    pub unsafe extern "system" fn RtlEnumerateGenericTableLikeADirectory(
        _table: *mut c_void,
        _match_function: *mut c_void,
        _match_data: *mut c_void,
        _next_flag: u32,
        _restart_key: *mut *mut c_void,
        _delete_count: *mut u32,
        _buffer: *mut c_void,
    ) -> *mut c_void {
        core::ptr::null_mut()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_sizes() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(core::mem::size_of::<SplayLinks>(), 24);
            assert_eq!(core::mem::size_of::<ListEntry>(), 16);
            assert_eq!(user_data_offset(), 40);
        }
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(core::mem::size_of::<SplayLinks>(), 12);
            assert_eq!(core::mem::size_of::<ListEntry>(), 8);
            // LONGLONG UserData is 8-aligned under the Windows i386 ABI (and our
            // i686-kernel target), leaving a 4-byte gap after ListEntry: 24, not
            // 20. A SysV i686 host aligns i64 to 4, so derive it rather than pin.
            let a = core::mem::align_of::<i64>();
            assert_eq!(user_data_offset() as usize, (20 + a - 1) & !(a - 1));
        }
    }

    #[test]
    fn test_list_operations() {
        unsafe {
            let mut head = ListEntry {
                flink: core::ptr::null_mut(),
                blink: core::ptr::null_mut(),
            };
            let head_ptr = &mut head as *mut ListEntry;
            list_init(head_ptr);
            assert_eq!((*head_ptr).flink, head_ptr);
            assert_eq!((*head_ptr).blink, head_ptr);

            let mut entry1 = ListEntry {
                flink: core::ptr::null_mut(),
                blink: core::ptr::null_mut(),
            };
            let entry1_ptr = &mut entry1 as *mut ListEntry;
            list_insert_tail(head_ptr, entry1_ptr);
            assert_eq!((*head_ptr).flink, entry1_ptr);
            assert_eq!((*head_ptr).blink, entry1_ptr);
            assert_eq!((*entry1_ptr).flink, head_ptr);
            assert_eq!((*entry1_ptr).blink, head_ptr);

            let mut entry2 = ListEntry {
                flink: core::ptr::null_mut(),
                blink: core::ptr::null_mut(),
            };
            let entry2_ptr = &mut entry2 as *mut ListEntry;
            list_insert_tail(head_ptr, entry2_ptr);
            // Order: head <-> entry1 <-> entry2 <-> head
            assert_eq!((*head_ptr).flink, entry1_ptr);
            assert_eq!((*head_ptr).blink, entry2_ptr);
            assert_eq!((*entry1_ptr).flink, entry2_ptr);
            assert_eq!((*entry2_ptr).blink, entry1_ptr);

            list_remove(entry1_ptr);
            // Order: head <-> entry2 <-> head
            assert_eq!((*head_ptr).flink, entry2_ptr);
            assert_eq!((*head_ptr).blink, entry2_ptr);
        }
    }

    #[test]
    fn test_node_to_user_data_offset() {
        // The node is sized by the offset node_to_user_data() writes at, so
        // the two must agree; a sum of the link sizes does not on i386.
        let mut header: TableEntryHeader = unsafe { core::mem::zeroed() };
        let base = &mut header as *mut TableEntryHeader as usize;
        let data = unsafe { node_to_user_data(&mut header.splay_links) } as usize;
        assert_eq!(data - base, user_data_offset() as usize);
    }
}

// ============================================================================
// Functional model tests — the existing `tests` cover struct layout + the list
// helpers but never exercise the actual table algorithm. These drive the
// algorithm layer (generic_table_insert / find_node_or_parent / RtlDelete /
// RtlRealSuccessor — the FFI shims live under cfg(not(test))) against a sorted
// BTreeSet reference, and crucially verify the splay tree's BST in-order
// invariant after every operation. Real NT-style callbacks back the table;
// elements are 4-byte u32 keys stored in the user-data region.
// ============================================================================
#[cfg(test)]
mod model_tests {
    use super::*;
    use std::alloc::{alloc_zeroed, dealloc, Layout};
    use std::collections::BTreeSet;

    const KEY_SIZE: u32 = 4; // a u32 key in the user-data region

    fn node_layout() -> Layout {
        let size = user_data_offset() as usize + KEY_SIZE as usize;
        Layout::from_size_align(size, core::mem::align_of::<TableEntryHeader>()).unwrap()
    }

    unsafe extern "system" fn t_alloc(_t: *mut RtlGenericTable, byte_size: u32) -> *mut c_void {
        assert_eq!(byte_size, user_data_offset() + KEY_SIZE);
        alloc_zeroed(node_layout()) as *mut c_void
    }

    unsafe extern "system" fn t_free(_t: *mut RtlGenericTable, buf: *mut c_void) {
        dealloc(buf as *mut u8, node_layout());
    }

    unsafe extern "system" fn t_compare(
        _t: *mut RtlGenericTable,
        a: *const c_void,
        b: *const c_void,
    ) -> u32 {
        let (ka, kb) = (*(a as *const u32), *(b as *const u32));
        match ka.cmp(&kb) {
            core::cmp::Ordering::Less => GENERIC_LESS_THAN,
            core::cmp::Ordering::Greater => GENERIC_GREATER_THAN,
            core::cmp::Ordering::Equal => GENERIC_EQUAL,
        }
    }

    fn make() -> Box<RtlGenericTable> {
        // Valid fn-pointer fields up front (no mem::zeroed UB), then init.
        let mut t = Box::new(RtlGenericTable {
            table_root: core::ptr::null_mut(),
            insert_order_list: ListEntry {
                flink: core::ptr::null_mut(),
                blink: core::ptr::null_mut(),
            },
            ordered_pointer: core::ptr::null_mut(),
            which_ordered_element: 0,
            number_generic_table_elements: 0,
            compare_routine: t_compare,
            allocate_routine: t_alloc,
            free_routine: t_free,
            table_context: core::ptr::null_mut(),
        });
        unsafe {
            generic_table_init(&mut *t, t_compare, t_alloc, t_free, core::ptr::null_mut());
        }
        t
    }

    unsafe fn insert(t: *mut RtlGenericTable, key: u32) -> bool {
        let mut new_flag: u8 = 0;
        let buf = key;
        let p = generic_table_insert(
            t,
            &buf as *const u32 as *const c_void,
            KEY_SIZE,
            &mut new_flag,
        );
        assert!(!p.is_null(), "insert returned null");
        assert_eq!(*(p as *const u32), key, "returned user data holds the key");
        new_flag != 0
    }

    unsafe fn lookup(t: *mut RtlGenericTable, key: u32) -> bool {
        let buf = key;
        let mut node: *mut SplayLinks = core::ptr::null_mut();
        if find_node_or_parent(t, &buf as *const u32 as *const c_void, &mut node) == TABLE_FOUND_NODE
        {
            assert_eq!(*(node_to_user_data(node) as *const u32), key);
            true
        } else {
            false
        }
    }

    // Mirror RtlDeleteElementGenericTable (FFI shim is cfg(not(test))).
    unsafe fn delete(t: *mut RtlGenericTable, key: u32) -> bool {
        let buf = key;
        let mut node: *mut SplayLinks = core::ptr::null_mut();
        if find_node_or_parent(t, &buf as *const u32 as *const c_void, &mut node) != TABLE_FOUND_NODE
        {
            return false;
        }
        (*t).table_root = RtlDelete(node);
        list_remove(node_to_list_entry(node));
        (*t).number_generic_table_elements -= 1;
        (*t).which_ordered_element = 0;
        (*t).ordered_pointer = &mut (*t).insert_order_list;
        t_free(t, node as *mut c_void);
        true
    }

    // In-order traversal (leftmost + RtlRealSuccessor) → keys in sorted order
    // iff the splay tree is a valid BST.
    unsafe fn in_order(t: *mut RtlGenericTable) -> Vec<u32> {
        let mut out = Vec::new();
        if (*t).table_root.is_null() {
            return out;
        }
        let mut node = (*t).table_root;
        while !sl_left(node).is_null() {
            node = sl_left(node);
        }
        while !node.is_null() {
            out.push(*(node_to_user_data(node) as *const u32));
            node = RtlRealSuccessor(node);
        }
        out
    }

    unsafe fn check(t: *mut RtlGenericTable, model: &BTreeSet<u32>) {
        assert_eq!((*t).number_generic_table_elements as usize, model.len(), "count");
        let sorted: Vec<u32> = model.iter().copied().collect();
        assert_eq!(in_order(t), sorted, "in-order traversal must equal the sorted model");
        for &k in model {
            assert!(lookup(t, k), "present key {k} must be found");
        }
    }

    unsafe fn drain(t: *mut RtlGenericTable, model: &mut BTreeSet<u32>) {
        let keys: Vec<u32> = model.iter().copied().collect();
        for k in keys {
            assert!(delete(t, k), "delete present {k}");
            model.remove(&k);
            check(t, model);
        }
        assert!((*t).table_root.is_null(), "drained tree root null");
    }

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u32) -> u32 {
            (self.next() % n as u64) as u32
        }
    }

    #[test]
    fn empty_and_single() {
        unsafe {
            let mut t = make();
            let p = &mut *t as *mut RtlGenericTable;
            let mut model = BTreeSet::new();
            check(p, &model);
            assert!(!lookup(p, 7));
            assert!(!delete(p, 7));

            assert!(insert(p, 7));
            assert!(!insert(p, 7)); // duplicate rejected
            model.insert(7);
            check(p, &model);
            drain(p, &mut model);
        }
    }

    #[test]
    fn ascending_then_descending() {
        unsafe {
            for order in 0..2 {
                let mut t = make();
                let p = &mut *t as *mut RtlGenericTable;
                let mut model = BTreeSet::new();
                let seq: Vec<u32> = if order == 0 {
                    (1..=150).collect()
                } else {
                    (1..=150).rev().collect()
                };
                for k in seq {
                    assert!(insert(p, k));
                    model.insert(k);
                }
                check(p, &model);
                drain(p, &mut model);
            }
        }
    }

    #[test]
    fn random_unique_insert() {
        unsafe {
            let mut t = make();
            let p = &mut *t as *mut RtlGenericTable;
            let mut model = BTreeSet::new();
            let mut rng = Rng(0xCAFE_F00D);
            for _ in 0..1500 {
                let k = rng.below(500) + 1;
                assert_eq!(insert(p, k), !model.contains(&k));
                model.insert(k);
            }
            check(p, &model);
            drain(p, &mut model);
        }
    }

    #[test]
    fn random_insert_delete_stress() {
        unsafe {
            let mut t = make();
            let p = &mut *t as *mut RtlGenericTable;
            let mut model = BTreeSet::new();
            let mut rng = Rng(0xBADC_0FFEE);
            for step in 0..4000 {
                let k = rng.below(200) + 1;
                if rng.below(2) == 0 {
                    assert_eq!(insert(p, k), !model.contains(&k));
                    model.insert(k);
                } else {
                    assert_eq!(delete(p, k), model.remove(&k));
                }
                if step % 250 == 0 {
                    check(p, &model);
                }
            }
            check(p, &model);
            drain(p, &mut model);
        }
    }
}
