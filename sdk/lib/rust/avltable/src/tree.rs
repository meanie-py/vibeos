//! AVL tree operations — direct port of avlsupp.c.
//!
//! All functions are `pub(crate) unsafe fn` taking raw pointers.
//! The algorithm is standard AVL (Knuth A6-A10) with the Windows-specific
//! sentinel root pattern (BalancedRoot.RightChild = real root).

use core::ffi::c_void;
use crate::types::*;

// ============================================================================
// Helper functions
// ============================================================================

/// Return pointer to user data area past the RtlBalancedLinks header.
#[inline(always)]
pub(crate) unsafe fn node_user_data(node: *mut RtlBalancedLinks) -> *mut c_void {
    (node as *mut u8).add(NODE_HEADER_SIZE) as *mut c_void // nolint: magic-offset (size_of-derived const)
}

/// Check if node is the left child of its parent.
#[inline(always)]
unsafe fn is_left_child(node: *mut RtlBalancedLinks) -> bool {
    (*(*node).parent).left_child == node
}

/// Check if node is the right child of its parent.
#[inline(always)]
unsafe fn is_right_child(node: *mut RtlBalancedLinks) -> bool {
    (*(*node).parent).right_child == node
}

// ============================================================================
// Core AVL operations
// ============================================================================

/// BST search using the table's compare callback.
/// Returns (node_or_parent, search_result).
pub(crate) unsafe fn find_node_or_parent(
    table: *mut RtlAvlTable,
    buffer: *const c_void,
) -> (*mut RtlBalancedLinks, u32) {
    if (*table).number_generic_table_elements == 0 {
        return (core::ptr::null_mut(), TABLE_EMPTY_TREE);
    }

    let mut current = (*table).balanced_root.right_child;
    let compare = (*table).compare_routine.unwrap_unchecked();

    loop {
        let user_data = node_user_data(current);
        let result = compare(table, buffer, user_data);

        if result == GENERIC_LESS_THAN {
            let child = (*current).left_child;
            if !child.is_null() {
                current = child;
            } else {
                return (current, TABLE_INSERT_AS_LEFT);
            }
        } else if result == GENERIC_GREATER_THAN {
            let child = (*current).right_child;
            if !child.is_null() {
                current = child;
            } else {
                return (current, TABLE_INSERT_AS_RIGHT);
            }
        } else {
            return (current, TABLE_FOUND_NODE);
        }
    }
}

/// Single rotation: promote node up one level past its parent.
/// Implements the rotation described in Knuth for AVL trees.
unsafe fn promote_node(node: *mut RtlBalancedLinks) {
    let parent = (*node).parent;
    let super_parent = (*parent).parent;

    if is_left_child(node) {
        // Node is left child → right-rotate parent
        let swap = (*node).right_child;
        (*parent).left_child = swap;
        if !swap.is_null() {
            (*swap).parent = parent;
        }
        (*node).right_child = parent;
    } else {
        // Node is right child → left-rotate parent
        let swap = (*node).left_child;
        (*parent).right_child = swap;
        if !swap.is_null() {
            (*swap).parent = parent;
        }
        (*node).left_child = parent;
    }
    (*parent).parent = node;

    // Update super-parent's child pointer
    if (*super_parent).left_child == parent {
        (*super_parent).left_child = node;
    } else {
        (*super_parent).right_child = node;
    }
    (*node).parent = super_parent;
}

/// AVL rebalance at an unbalanced node.
/// Returns `true` if the height change stops here (deletion case 3 only).
unsafe fn rebalance_node(node: *mut RtlBalancedLinks) -> bool {
    let balance = (*node).balance;

    // Pick the heavy child
    let child = if balance == RIGHT_HEAVY {
        (*node).right_child
    } else {
        (*node).left_child
    };

    // Case 1: child has same balance → single rotation (Knuth A8-A10 Case 1)
    if (*child).balance == balance {
        promote_node(child);
        (*child).balance = BALANCED;
        (*node).balance = BALANCED;
        return false;
    }

    // Case 2: child has opposite balance → double rotation (Knuth A8-A10 Case 2)
    if (*child).balance == -balance {
        let sub_child = if balance == RIGHT_HEAVY {
            (*child).left_child
        } else {
            (*child).right_child
        };

        promote_node(sub_child);
        promote_node(sub_child);

        if (*sub_child).balance == balance {
            (*child).balance = BALANCED;
            (*node).balance = -balance;
        } else if (*sub_child).balance == -balance {
            (*child).balance = balance;
            (*node).balance = BALANCED;
        } else {
            (*child).balance = BALANCED;
            (*node).balance = BALANCED;
        }

        (*sub_child).balance = BALANCED;
        return false;
    }

    // Case 3: child was balanced → single rotation (deletion only)
    promote_node(child);
    (*child).balance = -balance;
    true
}

/// Insert a new node into the tree and rebalance.
/// Implements Knuth A6-A7 walk-up rebalancing.
pub(crate) unsafe fn insert_node(
    table: *mut RtlAvlTable,
    new_node: *mut RtlBalancedLinks,
    mut node_or_parent: *mut RtlBalancedLinks,
    search_result: u32,
) {
    // Initialize the new node
    (*new_node).left_child = core::ptr::null_mut();
    (*new_node).right_child = core::ptr::null_mut();
    (*new_node).balance = BALANCED;

    (*table).number_generic_table_elements += 1;

    if search_result == TABLE_EMPTY_TREE {
        // First node: right child of sentinel root
        (*new_node).parent = &mut (*table).balanced_root;
        (*table).balanced_root.right_child = new_node;
        (*table).depth_of_tree = 1;
        return;
    }

    // Link the new node as left or right child
    if search_result == TABLE_INSERT_AS_LEFT {
        (*node_or_parent).left_child = new_node;
    } else {
        (*node_or_parent).right_child = new_node;
    }
    (*new_node).parent = node_or_parent;

    // Cheat: mark sentinel root left-heavy for loop termination detection
    (*table).balanced_root.balance = LEFT_HEAVY;

    // Walk up and rebalance (Knuth A6-A7)
    let mut child = new_node;
    loop {
        let balance = if is_left_child(child) { LEFT_HEAVY } else { RIGHT_HEAVY };

        if (*node_or_parent).balance == BALANCED {
            // Was balanced, now heavy on one side — height increased, continue up
            (*node_or_parent).balance = balance;
            child = node_or_parent;
            node_or_parent = (*node_or_parent).parent;
        } else if (*node_or_parent).balance != balance {
            // Was heavy on opposite side, now balanced — height unchanged
            (*node_or_parent).balance = BALANCED;
            // Check if the cheat detected a full-tree height increase
            if (*table).balanced_root.balance == BALANCED {
                (*table).depth_of_tree += 1;
            }
            break;
        } else {
            // Same-side heavy → +2 unbalanced, needs rotation
            rebalance_node(node_or_parent);
            break;
        }
    }
}

/// Delete a node from the tree and rebalance.
///
/// `node` is the node to logically delete. If it has two children, a
/// successor/predecessor is physically spliced out and swapped into
/// `node`'s position, leaving `node` fully unlinked for the caller to free.
pub(crate) unsafe fn delete_node(
    table: *mut RtlAvlTable,
    node: *mut RtlBalancedLinks,
) {
    // Find the node to physically remove from the tree structure
    let delete_node;
    if (*node).left_child.is_null() || (*node).right_child.is_null() {
        // At most one child: remove node directly
        delete_node = node;
    } else if (*node).balance >= BALANCED {
        // Two children, right-heavy or balanced: use in-order successor
        let mut dn = (*node).right_child;
        while !(*dn).left_child.is_null() {
            dn = (*dn).left_child;
        }
        delete_node = dn;
    } else {
        // Two children, left-heavy: use in-order predecessor
        let mut dn = (*node).left_child;
        while !(*dn).right_child.is_null() {
            dn = (*dn).right_child;
        }
        delete_node = dn;
    }

    // Get parent of the node being physically removed
    let mut parent_node = (*delete_node).parent;

    // Get the child that will replace delete_node (at most one exists)
    let replacement = if !(*delete_node).left_child.is_null() {
        (*delete_node).left_child
    } else {
        (*delete_node).right_child
    };

    // Which side of parent is delete_node on?
    let mut balance = if is_left_child(delete_node) { LEFT_HEAVY } else { RIGHT_HEAVY };

    // Splice out delete_node, replacing with its child
    if is_left_child(delete_node) {
        (*parent_node).left_child = replacement;
    } else {
        (*parent_node).right_child = replacement;
    }
    if !replacement.is_null() {
        (*replacement).parent = parent_node;
    }

    // Sentinel: mark root balanced for loop termination detection
    (*table).balanced_root.balance = BALANCED;

    // Walk up and rebalance
    loop {
        if (*parent_node).balance == balance {
            // Was heavy on deleted side → now balanced. Height decreased, continue.
            (*parent_node).balance = BALANCED;
        } else if (*parent_node).balance == BALANCED {
            // Was balanced → now heavy on opposite side. Height unchanged, stop.
            (*parent_node).balance = -balance;
            if (*table).balanced_root.balance != BALANCED {
                (*table).depth_of_tree -= 1;
            }
            break;
        } else {
            // Was heavy on opposite side → +2 unbalanced, rebalance needed
            if rebalance_node(parent_node) {
                break;
            }
            // After rebalance, get the promoted node (now parent_node's parent)
            parent_node = (*parent_node).parent;
        }

        // Update balance direction for next level and move up
        balance = if is_right_child(parent_node) { RIGHT_HEAVY } else { LEFT_HEAVY };
        parent_node = (*parent_node).parent;
    }

    // If we didn't directly remove the target node, swap delete_node into its place
    if node == delete_node {
        return;
    }

    // Copy node's tree linkage (parent, children, balance) into delete_node
    *delete_node = *node;

    // Update parent's child pointer to point to delete_node
    if is_left_child(node) {
        (*(*delete_node).parent).left_child = delete_node;
    } else {
        (*(*delete_node).parent).right_child = delete_node;
    }

    // Update children's parent pointers
    if !(*delete_node).left_child.is_null() {
        (*(*delete_node).left_child).parent = delete_node;
    }
    if !(*delete_node).right_child.is_null() {
        (*(*delete_node).right_child).parent = delete_node;
    }
}

/// In-order successor: go right then all-left, or walk up to find
/// the first ancestor where we came from the left subtree.
pub(crate) unsafe fn real_successor(
    links: *mut RtlBalancedLinks,
) -> *mut RtlBalancedLinks {
    let right = (*links).right_child;
    if !right.is_null() {
        let mut child = right;
        while !(*child).left_child.is_null() {
            child = (*child).left_child;
        }
        return child;
    }

    let mut child = links;
    while is_right_child(child) {
        child = (*child).parent;
    }

    if is_left_child(child) {
        (*child).parent
    } else {
        core::ptr::null_mut()
    }
}

/// In-order predecessor: go left then all-right, or walk up to find
/// the first ancestor where we came from the right subtree.
pub(crate) unsafe fn real_predecessor(
    links: *mut RtlBalancedLinks,
) -> *mut RtlBalancedLinks {
    let left = (*links).left_child;
    if !left.is_null() {
        let mut child = left;
        while !(*child).right_child.is_null() {
            child = (*child).right_child;
        }
        return child;
    }

    let mut child = links;
    while is_left_child(child) {
        child = (*child).parent;
    }

    if is_right_child(child) {
        let parent = (*child).parent;
        // Don't return the sentinel root (self-referencing parent)
        if (*parent).parent == parent {
            return core::ptr::null_mut();
        }
        parent
    } else {
        core::ptr::null_mut()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn free(ptr: *mut c_void);
    }

    unsafe extern "system" fn test_compare(
        _table: *mut RtlAvlTable,
        first: *const c_void,
        second: *const c_void,
    ) -> u32 {
        let a = *(first as *const u32);
        let b = *(second as *const u32);
        if a < b {
            GENERIC_LESS_THAN
        } else if a > b {
            GENERIC_GREATER_THAN
        } else {
            GENERIC_EQUAL
        }
    }

    unsafe extern "system" fn test_alloc(
        _table: *mut RtlAvlTable,
        byte_size: u32,
    ) -> *mut c_void {
        malloc(byte_size as usize)
    }

    unsafe extern "system" fn test_free(
        _table: *mut RtlAvlTable,
        buffer: *mut c_void,
    ) {
        free(buffer)
    }

    /// Initialize a table for testing.
    unsafe fn init_table(table: *mut RtlAvlTable) {
        core::ptr::write_bytes(table, 0, 1);
        (*table).balanced_root.parent = &mut (*table).balanced_root;
        (*table).compare_routine = Some(test_compare);
        (*table).allocate_routine = Some(test_alloc);
        (*table).free_routine = Some(test_free);
    }

    /// Insert a u32 key. Returns pointer to user data, and whether it was new.
    unsafe fn insert_key(table: *mut RtlAvlTable, key: u32) -> (*mut c_void, bool) {
        let (node_or_parent, result) =
            find_node_or_parent(table, &key as *const u32 as *const c_void);

        if result == TABLE_FOUND_NODE {
            return (node_user_data(node_or_parent), false);
        }

        let alloc_size = (NODE_HEADER_SIZE + core::mem::size_of::<u32>()) as u32;
        let alloc = test_alloc(core::ptr::null_mut(), alloc_size);
        assert!(!alloc.is_null());
        let new_node = alloc as *mut RtlBalancedLinks;

        core::ptr::write_bytes(new_node, 0, 1);
        insert_node(table, new_node, node_or_parent, result);

        let user_data = node_user_data(new_node);
        *(user_data as *mut u32) = key;

        (user_data, true)
    }

    /// Delete a u32 key. Returns true if found and deleted.
    unsafe fn delete_key(table: *mut RtlAvlTable, key: u32) -> bool {
        let (node, result) =
            find_node_or_parent(table, &key as *const u32 as *const c_void);
        if result != TABLE_FOUND_NODE {
            return false;
        }

        if node == (*table).restart_key {
            (*table).restart_key = real_predecessor(node);
        }

        (*table).delete_count += 1;
        delete_node(table, node);
        (*table).number_generic_table_elements -= 1;
        (*table).which_ordered_element = 0;
        (*table).ordered_pointer = core::ptr::null_mut();

        test_free(core::ptr::null_mut(), node as *mut c_void);
        true
    }

    /// Collect all keys in-order via tree traversal.
    unsafe fn collect_keys(table: *mut RtlAvlTable) -> Vec<u32> {
        let mut keys = Vec::new();
        if (*table).number_generic_table_elements == 0 {
            return keys;
        }

        // Find leftmost
        let mut node = (*table).balanced_root.right_child;
        while !(*node).left_child.is_null() {
            node = (*node).left_child;
        }

        loop {
            let data = node_user_data(node);
            keys.push(*(data as *const u32));
            node = real_successor(node);
            if node.is_null() {
                break;
            }
        }
        keys
    }

    /// Verify AVL invariants: BST ordering, balance factors in {-1,0,1},
    /// parent link consistency, correct element count.
    unsafe fn verify_invariants(table: *mut RtlAvlTable) {
        let root = (*table).balanced_root.right_child;
        if root.is_null() {
            assert_eq!((*table).number_generic_table_elements, 0);
            return;
        }

        // Root's parent must be the sentinel
        assert_eq!(
            (*root).parent,
            &mut (*table).balanced_root as *mut RtlBalancedLinks
        );

        let (count, _height) = verify_subtree(root, &mut (*table).balanced_root);
        assert_eq!(count, (*table).number_generic_table_elements);
    }

    /// Recursively verify a subtree. Returns (node_count, height).
    unsafe fn verify_subtree(
        node: *mut RtlBalancedLinks,
        expected_parent: *mut RtlBalancedLinks,
    ) -> (u32, i32) {
        assert!(!node.is_null());
        assert_eq!((*node).parent, expected_parent, "parent link mismatch");
        assert!(
            (*node).balance >= -1 && (*node).balance <= 1,
            "invalid balance factor: {}",
            (*node).balance
        );

        let (left_count, left_height) = if !(*node).left_child.is_null() {
            verify_subtree((*node).left_child, node)
        } else {
            (0, 0)
        };

        let (right_count, right_height) = if !(*node).right_child.is_null() {
            verify_subtree((*node).right_child, node)
        } else {
            (0, 0)
        };

        // Verify balance factor matches actual heights
        let actual_balance = right_height - left_height;
        assert_eq!(
            (*node).balance as i32, actual_balance,
            "balance factor mismatch: stored={} actual={}",
            (*node).balance, actual_balance
        );

        // Verify BST ordering
        if !(*node).left_child.is_null() {
            let left_key = *(node_user_data((*node).left_child) as *const u32);
            let node_key = *(node_user_data(node) as *const u32);
            assert!(left_key <= node_key, "BST violation: left {} > node {}", left_key, node_key);
        }
        if !(*node).right_child.is_null() {
            let right_key = *(node_user_data((*node).right_child) as *const u32);
            let node_key = *(node_user_data(node) as *const u32);
            assert!(right_key >= node_key, "BST violation: right {} < node {}", right_key, node_key);
        }

        let height = 1 + left_height.max(right_height);
        (1 + left_count + right_count, height)
    }

    #[test]
    fn test_empty_table() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            assert_eq!(table.number_generic_table_elements, 0);
            let (_, result) =
                find_node_or_parent(&mut table, core::ptr::null());
            assert_eq!(result, TABLE_EMPTY_TREE);
            verify_invariants(&mut table);
        }
    }

    #[test]
    fn test_single_insert_lookup_delete() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            let (data, is_new) = insert_key(&mut table, 42);
            assert!(is_new);
            assert_eq!(*(data as *const u32), 42);
            assert_eq!(table.number_generic_table_elements, 1);
            verify_invariants(&mut table);

            // Lookup
            let (node, result) =
                find_node_or_parent(&mut table, &42u32 as *const u32 as *const c_void);
            assert_eq!(result, TABLE_FOUND_NODE);
            assert_eq!(*(node_user_data(node) as *const u32), 42);

            // Delete
            assert!(delete_key(&mut table, 42));
            assert_eq!(table.number_generic_table_elements, 0);
            verify_invariants(&mut table);
        }
    }

    #[test]
    fn test_multiple_inserts() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            let keys = [5, 3, 7, 1, 4, 6, 8, 2, 9, 10];
            for &k in &keys {
                let (_, is_new) = insert_key(&mut table, k);
                assert!(is_new);
                verify_invariants(&mut table);
            }
            assert_eq!(table.number_generic_table_elements, 10);

            // Lookup each
            for &k in &keys {
                let (_, result) =
                    find_node_or_parent(&mut table, &k as *const u32 as *const c_void);
                assert_eq!(result, TABLE_FOUND_NODE);
            }

            // Verify in-order
            let collected = collect_keys(&mut table);
            let mut sorted = keys.to_vec();
            sorted.sort();
            assert_eq!(collected, sorted);

            // Cleanup
            for &k in &keys {
                delete_key(&mut table, k);
            }
        }
    }

    #[test]
    fn test_sorted_insert_worst_case() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            // Ascending order — worst case for BST, triggers all rotation types
            for k in 1..=20u32 {
                insert_key(&mut table, k);
                verify_invariants(&mut table);
            }

            let keys = collect_keys(&mut table);
            assert_eq!(keys, (1..=20).collect::<Vec<_>>());

            // Cleanup
            for k in 1..=20u32 {
                delete_key(&mut table, k);
            }
        }
    }

    #[test]
    fn test_reverse_sorted_insert() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            // Descending order
            for k in (1..=20u32).rev() {
                insert_key(&mut table, k);
                verify_invariants(&mut table);
            }

            let keys = collect_keys(&mut table);
            assert_eq!(keys, (1..=20).collect::<Vec<_>>());

            // Cleanup
            for k in 1..=20u32 {
                delete_key(&mut table, k);
            }
        }
    }

    #[test]
    fn test_random_insert_delete() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            // Deterministic "random" sequence using LCG
            let mut rng: u32 = 12345;
            let mut inserted = BTreeSet::new();

            // Insert 100 unique keys
            while inserted.len() < 100 {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let key = (rng >> 16) % 10000;
                if inserted.insert(key) {
                    let (_, is_new) = insert_key(&mut table, key);
                    assert!(is_new);
                    verify_invariants(&mut table);
                }
            }
            assert_eq!(table.number_generic_table_elements, 100);

            // Delete first 50
            let to_delete: Vec<u32> = inserted.iter().take(50).copied().collect();
            for &k in &to_delete {
                assert!(delete_key(&mut table, k));
                inserted.remove(&k);
                verify_invariants(&mut table);
            }
            assert_eq!(table.number_generic_table_elements, 50);

            // Verify remaining
            let remaining = collect_keys(&mut table);
            let expected: Vec<u32> = inserted.iter().copied().collect();
            assert_eq!(remaining, expected);

            // Cleanup
            for &k in &expected {
                delete_key(&mut table, k);
            }
        }
    }

    #[test]
    fn test_duplicate_insert() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            let (_, is_new) = insert_key(&mut table, 42);
            assert!(is_new);

            // Insert same key again — should find existing
            let (node, result) =
                find_node_or_parent(&mut table, &42u32 as *const u32 as *const c_void);
            assert_eq!(result, TABLE_FOUND_NODE);
            assert_eq!(*(node_user_data(node) as *const u32), 42);

            assert_eq!(table.number_generic_table_elements, 1);

            delete_key(&mut table, 42);
        }
    }

    #[test]
    fn test_enumeration() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            for k in [50, 25, 75, 10, 30, 60, 90u32] {
                insert_key(&mut table, k);
            }

            // Enumerate using successor
            let keys = collect_keys(&mut table);
            assert_eq!(keys, vec![10, 25, 30, 50, 60, 75, 90]);

            // Enumerate using predecessor (reverse)
            let mut rev_keys = Vec::new();
            let mut node = table.balanced_root.right_child;
            while !(*node).right_child.is_null() {
                node = (*node).right_child;
            }
            loop {
                rev_keys.push(*(node_user_data(node) as *const u32));
                node = real_predecessor(node);
                if node.is_null() {
                    break;
                }
            }
            assert_eq!(rev_keys, vec![90, 75, 60, 50, 30, 25, 10]);

            for k in [50, 25, 75, 10, 30, 60, 90u32] {
                delete_key(&mut table, k);
            }
        }
    }

    #[test]
    fn test_delete_all_ascending() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            for k in 1..=15u32 {
                insert_key(&mut table, k);
            }
            verify_invariants(&mut table);

            // Delete in ascending order
            for k in 1..=15u32 {
                assert!(delete_key(&mut table, k));
                verify_invariants(&mut table);
            }
            assert_eq!(table.number_generic_table_elements, 0);
        }
    }

    #[test]
    fn test_delete_all_descending() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            for k in 1..=15u32 {
                insert_key(&mut table, k);
            }

            // Delete in descending order
            for k in (1..=15u32).rev() {
                assert!(delete_key(&mut table, k));
                verify_invariants(&mut table);
            }
            assert_eq!(table.number_generic_table_elements, 0);
        }
    }

    #[test]
    fn test_delete_nonexistent() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            insert_key(&mut table, 42);
            assert!(!delete_key(&mut table, 99));
            assert_eq!(table.number_generic_table_elements, 1);

            delete_key(&mut table, 42);
        }
    }

    #[test]
    fn test_predecessor_successor_edges() {
        unsafe {
            let mut table = core::mem::zeroed::<RtlAvlTable>();
            init_table(&mut table);

            insert_key(&mut table, 10);
            insert_key(&mut table, 20);
            insert_key(&mut table, 30);

            // Find min (10)
            let mut node = table.balanced_root.right_child;
            while !(*node).left_child.is_null() {
                node = (*node).left_child;
            }
            assert_eq!(*(node_user_data(node) as *const u32), 10);

            // Predecessor of min should be null
            assert!(real_predecessor(node).is_null());

            // Find max (30)
            node = table.balanced_root.right_child;
            while !(*node).right_child.is_null() {
                node = (*node).right_child;
            }
            assert_eq!(*(node_user_data(node) as *const u32), 30);

            // Successor of max should be null
            assert!(real_successor(node).is_null());

            for k in [10, 20, 30u32] {
                delete_key(&mut table, k);
            }
        }
    }
}
