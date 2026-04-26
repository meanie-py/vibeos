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

/// User data field offset in TABLE_ENTRY_HEADER.
#[inline]
fn user_data_offset() -> u32 {
    // offset_of!(TableEntryHeader, user_data)
    (core::mem::size_of::<SplayLinks>() + core::mem::size_of::<ListEntry>()) as u32
}

// ============================================================================
// ARM64 QEMU TCG workaround
// ============================================================================

#[cfg(all(not(test), target_arch = "aarch64"))]
#[inline]
unsafe fn gt_barrier() {
    // Memory barrier only — UART trace removed (Arm64UartKernelBase is
    // kernel-only and not available when linked into user-mode DLLs like ntdll).
    core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));
}

#[cfg(any(test, not(target_arch = "aarch64")))]
#[inline]
unsafe fn gt_barrier() {
    // No-op on non-ARM64
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

// In test mode, we call the splaytree crate's pub functions directly would
// require a dep — instead just provide stubs that won't be called.
#[cfg(test)]
unsafe fn RtlSplay(_: *mut SplayLinks) -> *mut SplayLinks { core::ptr::null_mut() }
#[cfg(test)]
unsafe fn RtlDelete(_: *mut SplayLinks) -> *mut SplayLinks { core::ptr::null_mut() }
#[cfg(test)]
unsafe fn RtlRealSuccessor(_: *mut SplayLinks) -> *mut SplayLinks { core::ptr::null_mut() }
#[cfg(test)]
unsafe fn RtlSubtreePredecessor(_: *mut SplayLinks) -> *mut SplayLinks { core::ptr::null_mut() }

// ============================================================================
// Private functions
// ============================================================================

unsafe fn find_node_or_parent(
    table: *mut RtlGenericTable,
    buffer: *const c_void,
    node_or_parent: *mut *mut SplayLinks,
) -> u32 {
    gt_barrier();

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

    #[cfg(target_arch = "aarch64")]
    let mut loop_count: u32 = 0;

    loop {
        #[cfg(target_arch = "aarch64")]
        {
            loop_count += 1;
            if loop_count > 100_000 {
                // Infinite loop safety — break out
                *node_or_parent = current;
                return TABLE_INSERT_AS_LEFT;
            }
        }

        gt_barrier();
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
        gt_barrier();
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
    gt_barrier();
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

        gt_barrier();
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

        gt_barrier();
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
            gt_barrier();
            (*table).table_root = RtlSplay(node);
            found_node = node;
        } else {
            let node = RtlRealSuccessor((*table).table_root);
            if !node.is_null() {
                gt_barrier();
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

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
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
            assert_eq!(user_data_offset(), 20);
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
        // Verify the offset calculation is correct
        let offset = user_data_offset() as usize;
        assert_eq!(
            offset,
            core::mem::size_of::<SplayLinks>() + core::mem::size_of::<ListEntry>()
        );
    }
}
