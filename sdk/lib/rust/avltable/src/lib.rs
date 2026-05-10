//! RTL AVL Generic Table — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/avltable.c` (326 lines) + `avlsupp.c` (395 lines).
//! Fully C-compatible: same ABI, same struct layouts, same exported symbols.
//!
//! - `types.rs`: `#[repr(C)]` structs, enums, callback type aliases
//! - `tree.rs`: AVL operations (find, insert, delete, rotate, traverse)
//! - `lib.rs`: 12 FFI entry points with `extern "system"` (= NTAPI)

#![cfg_attr(not(test), no_std)]

pub mod types;
pub mod tree;

#[cfg(not(test))]
mod ffi {
    use core::ffi::c_void;
    use crate::types::*;
    use crate::tree;

    #[no_mangle]
    pub unsafe extern "system" fn RtlInitializeGenericTableAvl(
        table: *mut RtlAvlTable,
        compare_routine: Option<AvlCompareRoutine>,
        allocate_routine: Option<AvlAllocateRoutine>,
        free_routine: Option<AvlFreeRoutine>,
        table_context: *mut c_void,
    ) {
        core::ptr::write_bytes(table, 0, 1);
        (*table).balanced_root.parent = &mut (*table).balanced_root;
        (*table).compare_routine = compare_routine;
        (*table).allocate_routine = allocate_routine;
        (*table).free_routine = free_routine;
        (*table).table_context = table_context;
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlInsertElementGenericTableFullAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
        buffer_size: u32,
        new_element: *mut u8, // PBOOLEAN, OPTIONAL
        node_or_parent: *mut c_void, // PVOID (actually *mut RtlBalancedLinks)
        search_result: u32,          // TABLE_SEARCH_RESULT
    ) -> *mut c_void {
        let node_or_parent = node_or_parent as *mut RtlBalancedLinks;

        if search_result != TABLE_FOUND_NODE {
            // Allocate: header + user data
            let alloc_size = buffer_size + NODE_HEADER_SIZE as u32;
            let new_node = ((*table).allocate_routine.unwrap_unchecked())(
                table, alloc_size,
            ) as *mut RtlBalancedLinks;

            if new_node.is_null() {
                if !new_element.is_null() {
                    *new_element = 0;
                }
                return core::ptr::null_mut();
            }

            let user_data = tree::node_user_data(new_node);

            // Zero the links header and insert
            core::ptr::write_bytes(new_node, 0, 1);
            tree::insert_node(table, new_node, node_or_parent, search_result);

            // Copy user buffer
            core::ptr::copy_nonoverlapping(
                buffer as *const u8,
                user_data as *mut u8,
                buffer_size as usize,
            );

            if !new_element.is_null() {
                *new_element = 1;
            }
            user_data
        } else {
            // Already exists
            if !new_element.is_null() {
                *new_element = 0;
            }
            tree::node_user_data(node_or_parent)
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlInsertElementGenericTableAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
        buffer_size: u32,
        new_element: *mut u8, // PBOOLEAN, OPTIONAL
    ) -> *mut c_void {
        let (node_or_parent, result) = tree::find_node_or_parent(table, buffer);
        RtlInsertElementGenericTableFullAvl(
            table,
            buffer,
            buffer_size,
            new_element,
            node_or_parent as *mut c_void,
            result,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDeleteElementGenericTableAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
    ) -> u8 {
        let (node, result) = tree::find_node_or_parent(table, buffer);
        if result != TABLE_FOUND_NODE {
            return 0;
        }

        // Update restart key if it points to the deleted node
        if node == (*table).restart_key {
            (*table).restart_key = tree::real_predecessor(node);
        }

        (*table).delete_count += 1;
        tree::delete_node(table, node);
        (*table).number_generic_table_elements -= 1;

        // Reset ordered enumeration state
        (*table).which_ordered_element = 0;
        (*table).ordered_pointer = core::ptr::null_mut();

        // Free the node
        ((*table).free_routine.unwrap_unchecked())(table, node as *mut c_void);

        1 // TRUE
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLookupElementGenericTableFullAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
        out_node_or_parent: *mut *mut c_void,
        out_search_result: *mut u32,
    ) -> *mut c_void {
        let (node, result) = tree::find_node_or_parent(table, buffer);
        *out_search_result = result;
        *out_node_or_parent = node as *mut c_void;

        if result != TABLE_FOUND_NODE {
            core::ptr::null_mut()
        } else {
            tree::node_user_data(node)
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLookupElementGenericTableAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
    ) -> *mut c_void {
        let mut node_or_parent: *mut c_void = core::ptr::null_mut();
        let mut result: u32 = 0;
        RtlLookupElementGenericTableFullAvl(
            table,
            buffer,
            &mut node_or_parent,
            &mut result,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLookupFirstMatchingElementGenericTableAvl(
        table: *mut RtlAvlTable,
        buffer: *const c_void,
        restart_key: *mut *mut c_void,
    ) -> *mut c_void {
        *restart_key = core::ptr::null_mut();

        let (mut node, result) = tree::find_node_or_parent(table, buffer);
        if result != TABLE_FOUND_NODE {
            return core::ptr::null_mut();
        }

        // Walk predecessors while they compare equal
        let compare = (*table).compare_routine.unwrap_unchecked();
        loop {
            let prev = tree::real_predecessor(node);
            if prev.is_null() || (*prev).parent == prev {
                break;
            }
            let prev_data = tree::node_user_data(prev);
            if compare(table, buffer, prev_data) != GENERIC_EQUAL {
                break;
            }
            node = prev;
        }

        *restart_key = node as *mut c_void;
        tree::node_user_data(node)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnumerateGenericTableWithoutSplayingAvl(
        table: *mut RtlAvlTable,
        restart_key: *mut *mut c_void,
    ) -> *mut c_void {
        if (*table).number_generic_table_elements == 0 {
            return core::ptr::null_mut();
        }

        let current;
        if (*restart_key).is_null() {
            // Find leftmost node (smallest)
            let mut node = (*table).balanced_root.right_child;
            while !(*node).left_child.is_null() {
                node = (*node).left_child;
            }
            current = node;
            *restart_key = current as *mut c_void;
        } else {
            // Get successor of current position
            let node = *restart_key as *mut RtlBalancedLinks;
            current = tree::real_successor(node);
            if !current.is_null() {
                *restart_key = current as *mut c_void;
            }
        }

        if !current.is_null() {
            tree::node_user_data(current)
        } else {
            core::ptr::null_mut()
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnumerateGenericTableAvl(
        table: *mut RtlAvlTable,
        restart: u8, // BOOLEAN
    ) -> *mut c_void {
        if restart != 0 {
            (*table).restart_key = core::ptr::null_mut();
        }
        RtlEnumerateGenericTableWithoutSplayingAvl(
            table,
            &mut (*table).restart_key as *mut *mut RtlBalancedLinks as *mut *mut c_void,
        )
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlGetElementGenericTableAvl(
        _table: *mut RtlAvlTable,
        _i: u32,
    ) -> *mut c_void {
        // Unimplemented, matches C version
        core::ptr::null_mut()
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlIsGenericTableEmptyAvl(
        table: *mut RtlAvlTable,
    ) -> u8 {
        ((*table).number_generic_table_elements == 0) as u8
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlNumberGenericTableElementsAvl(
        table: *mut RtlAvlTable,
    ) -> u32 {
        (*table).number_generic_table_elements
    }
}
