//! MM-flavored AVL tree algorithms.
//!
//! Direct port of `sdk/lib/rtl/avlsupp.c` + the MM-specific helpers from
//! `ntoskrnl/mm/ARM3/miavl.h`. The C code lives in `mm_rust_shims.c` as a
//! `#include avlsupp.c` trick that compiles the Rtl algorithms with all
//! `Rtl*` helpers `#define`-renamed to `Mi*` flavors that know how to pack
//! the balance factor into the low 2 bits of the parent pointer.
//!
//! The Mm flavor differs from Rtl in two ways:
//!   1. `Balance` is a 2-bit signed field packed into the low bits of
//!      `_MMADDRESS_NODE.u1.Parent` (the parent pointer is 4-/8-byte
//!      aligned so the low 2 bits are free), instead of a separate
//!      `i8 Balance` field as in `_RTL_BALANCED_LINKS`.
//!   2. The compare routine inlines a `StartingVpn`/`EndingVpn` range
//!      check directly against the node, instead of going through a
//!      callback like the Rtl version.
//!
//! Same augment+isolated-rlib pattern as every other `*-inlines` /
//! `*-macros` sibling: workspace `lto = false` plus the crate boundary
//! defuses cross-CGU captures(none) propagation.

#![no_std]

use ntkrnl_sys::ddk::{
    _MMADDRESS_NODE, _MM_AVL_TABLE, PMMADDRESS_NODE, PMM_AVL_TABLE,
};

// 2-bit signed Balance values (RTL_AVL_BALANCE_FACTOR enum).
const BALANCED:    i8 =  0;
const RIGHT_HEAVY: i8 =  1;
const LEFT_HEAVY:  i8 = -1;
// (UNBALANCED = -2 exists in the C enum but is never written.)

// TABLE_SEARCH_RESULT values (from ntkrnl_sys::ddk).
const TABLE_EMPTY_TREE:        u32 = 0;
const TABLE_FOUND_NODE:        u32 = 1;
const TABLE_INSERT_AS_LEFT:    u32 = 2;
const TABLE_INSERT_AS_RIGHT:   u32 = 3;

// === Helpers (Mm-flavored equivalents of miavl.h's Mi* FORCEINLINEs) ===

#[inline]
unsafe fn parent(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    (((*node).u1.Parent as usize) & !0b11usize) as PMMADDRESS_NODE
}

#[inline]
unsafe fn set_parent(node: PMMADDRESS_NODE, p: PMMADDRESS_NODE) {
    // Preserve the existing 2-bit Balance encoded in the low bits.
    let balance_bits = ((*node).u1.Parent as usize) & 0b11usize;
    (*node).u1.Parent = ((p as usize) | balance_bits) as *mut _MMADDRESS_NODE;
}

#[inline]
unsafe fn balance(node: PMMADDRESS_NODE) -> i8 {
    // The 2-bit signed value is stored in the low 2 bits of Parent.
    // Sign-extend by shifting into i32's sign bit and arithmetic-right-shifting back.
    let bits = ((*node).u1.Parent as usize) & 0b11usize;
    (((bits as i32) << 30) >> 30) as i8
}

#[inline]
unsafe fn set_balance(node: PMMADDRESS_NODE, b: i8) {
    let parent_only = ((*node).u1.Parent as usize) & !0b11usize;
    let new_bits = (b as u8 as usize) & 0b11usize;
    (*node).u1.Parent = (parent_only | new_bits) as *mut _MMADDRESS_NODE;
}

#[inline]
unsafe fn left_child(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    (*node).LeftChild
}

#[inline]
unsafe fn right_child(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    (*node).RightChild
}

#[inline]
unsafe fn is_left_child(node: PMMADDRESS_NODE) -> bool {
    left_child(parent(node)) == node
}

#[inline]
unsafe fn is_right_child(node: PMMADDRESS_NODE) -> bool {
    right_child(parent(node)) == node
}

#[inline]
unsafe fn insert_as_left_child(p: PMMADDRESS_NODE, node: PMMADDRESS_NODE) {
    (*p).LeftChild = node;
    set_parent(node, p);
}

#[inline]
unsafe fn insert_as_right_child(p: PMMADDRESS_NODE, node: PMMADDRESS_NODE) {
    (*p).RightChild = node;
    set_parent(node, p);
}

#[inline]
unsafe fn copy_node_data(dst: PMMADDRESS_NODE, src: PMMADDRESS_NODE) {
    // MiCopyAvlNodeData copies the Parent (with embedded balance) and both children.
    // It deliberately does NOT copy StartingVpn/EndingVpn — those stay with the
    // node being relinked.
    (*dst).u1.Parent = (*src).u1.Parent;
    (*dst).LeftChild = (*src).LeftChild;
    (*dst).RightChild = (*src).RightChild;
}

/// MiAvlCompareRoutine: return -1 (less), 0 (equal), 1 (greater) comparing
/// `buffer` (a StartingVpn key) against `node`'s [StartingVpn, EndingVpn]
/// range. This is the Mm-specific inline comparison; the Rtl version uses
/// a user callback.
#[inline]
unsafe fn compare(node: PMMADDRESS_NODE, buffer_starting_vpn: usize) -> i32 {
    let starting = (*node).StartingVpn as usize;
    let ending = (*node).EndingVpn as usize;
    if buffer_starting_vpn < starting {
        -1
    } else if buffer_starting_vpn <= ending {
        0
    } else {
        1
    }
}

// === MM_AVL_TABLE bitfield accessors ===
//
// `MM_AVL_TABLE` packs `DepthOfTree` (5 bits) and `NumberGenericTableElements`
// (the rest, 56 bits on 64-bit) into one `_bitfield_1: [u8; 8]`. We need
// pointer-based access since callers pass `*mut MM_AVL_TABLE`. Use the
// bindgen-generated `_raw` helpers.

#[inline]
unsafe fn count(table: PMM_AVL_TABLE) -> usize {
    _MM_AVL_TABLE::NumberGenericTableElements_raw(table) as usize
}

#[inline]
unsafe fn set_count(table: PMM_AVL_TABLE, n: usize) {
    _MM_AVL_TABLE::set_NumberGenericTableElements_raw(table, n as _);
}

#[inline]
unsafe fn depth(table: PMM_AVL_TABLE) -> usize {
    _MM_AVL_TABLE::DepthOfTree_raw(table) as usize
}

#[inline]
unsafe fn set_depth(table: PMM_AVL_TABLE, d: usize) {
    _MM_AVL_TABLE::set_DepthOfTree_raw(table, d as _);
}

#[inline]
unsafe fn root_node(table: PMM_AVL_TABLE) -> PMMADDRESS_NODE {
    core::ptr::addr_of_mut!((*table).BalancedRoot)
}

// === Internal algorithms (avlsupp.c FORCEINLINEs) ===

unsafe fn promote_node(node: PMMADDRESS_NODE) {
    // Mirror RtlpPromoteAvlTreeNode — single rotation (Knuth A8-A10 Case 1/3).
    let parent_node = parent(node);
    let super_parent = parent(parent_node);

    // Pick which slots get rotated based on whether `node` is parent's left or right.
    let (swap1, swap2): (*mut PMMADDRESS_NODE, *mut PMMADDRESS_NODE) = if is_left_child(node) {
        (
            core::ptr::addr_of_mut!((*parent_node).LeftChild),
            core::ptr::addr_of_mut!((*node).RightChild),
        )
    } else {
        (
            core::ptr::addr_of_mut!((*parent_node).RightChild),
            core::ptr::addr_of_mut!((*node).LeftChild),
        )
    };

    // Rotate.
    *swap1 = *swap2;
    if !(*swap1).is_null() {
        set_parent(*swap1, parent_node);
    }
    *swap2 = parent_node;
    set_parent(parent_node, node);

    // Update super-parent's child link to point at `node`.
    let super_swap = if left_child(super_parent) == parent_node {
        core::ptr::addr_of_mut!((*super_parent).LeftChild)
    } else {
        core::ptr::addr_of_mut!((*super_parent).RightChild)
    };
    *super_swap = node;
    set_parent(node, super_parent);
}

/// Returns true if rebalance terminates a delete propagation (Case 3).
unsafe fn rebalance_node(node: PMMADDRESS_NODE) -> bool {
    let bal = balance(node);
    let child = if bal == RIGHT_HEAVY {
        right_child(node)
    } else {
        left_child(node)
    };

    let child_bal = balance(child);
    if child_bal == bal {
        // Case 1: child has same heaviness — single rotation.
        promote_node(child);
        set_balance(child, BALANCED);
        set_balance(node, BALANCED);
        return false;
    }

    if child_bal == -bal {
        // Case 2: child has opposite heaviness — double rotation.
        let sub_child = if bal == RIGHT_HEAVY {
            left_child(child)
        } else {
            right_child(child)
        };
        promote_node(sub_child);
        promote_node(sub_child);

        let sub_bal = balance(sub_child);
        if sub_bal == bal {
            set_balance(child, BALANCED);
            set_balance(node, -bal);
        } else if sub_bal == -bal {
            set_balance(child, bal);
            set_balance(node, BALANCED);
        } else {
            set_balance(child, BALANCED);
            set_balance(node, BALANCED);
        }
        set_balance(sub_child, BALANCED);
        return false;
    }

    // Case 3: child was balanced — single rotation; only happens on delete,
    // and signals "stop the delete propagation".
    promote_node(child);
    set_balance(child, -bal);
    true
}

// === Public exports — three algorithms (Find / Insert / Delete) and five
// trivial accessors used directly from the C side. All use stdcall on i386
// (NTAPI), cdecl on amd64/arm64. The C trampolines in mm_rust_shims.c
// (RtlpFindAvlTableNodeOrParent / RtlpInsertAvlTreeNode /
// RtlpDeleteAvlTreeNode / RtlRightChildAvl / RtlLeftChildAvl /
// RtlParentAvl / RtlIsLeftChildAvl / RtlIsRightChildAvl) one-line-delegate
// to these. ===

#[no_mangle]
pub unsafe extern "system" fn RtlRightChildAvl(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    right_child(node)
}

#[no_mangle]
pub unsafe extern "system" fn RtlLeftChildAvl(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    left_child(node)
}

#[no_mangle]
pub unsafe extern "system" fn RtlParentAvl(node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    parent(node)
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsRightChildAvl(node: PMMADDRESS_NODE) -> u8 {
    if is_right_child(node) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsLeftChildAvl(node: PMMADDRESS_NODE) -> u8 {
    if is_left_child(node) { 1 } else { 0 }
}

/// `RtlpFindAvlTableNodeOrParent` (Mm flavor).
///
/// Walks the tree from the right child of `BalancedRoot` (the dummy root)
/// using the Mm-specific compare. On exit:
///   - `TABLE_EMPTY_TREE` (0) if the tree was empty
///   - `TABLE_FOUND_NODE` (1) if a node with overlapping VPN range was found
///   - `TABLE_INSERT_AS_LEFT` (2) / `TABLE_INSERT_AS_RIGHT` (3) for the
///     node where the new entry should be inserted
///
/// `*node_or_parent` receives the found/parent node on non-empty results.
#[no_mangle]
pub unsafe extern "system" fn RtlpFindAvlTableNodeOrParent(
    table: PMM_AVL_TABLE,
    buffer: *mut core::ffi::c_void,
    node_or_parent: *mut PMMADDRESS_NODE,
) -> u32 {
    if count(table) == 0 {
        return TABLE_EMPTY_TREE;
    }
    let key = buffer as usize;
    let mut current = right_child(root_node(table));

    loop {
        let cmp = compare(current, key);
        if cmp < 0 {
            let lc = left_child(current);
            if !lc.is_null() {
                current = lc;
            } else {
                core::ptr::write_volatile(node_or_parent, current);
                return TABLE_INSERT_AS_LEFT;
            }
        } else if cmp > 0 {
            let rc = right_child(current);
            if !rc.is_null() {
                current = rc;
            } else {
                core::ptr::write_volatile(node_or_parent, current);
                return TABLE_INSERT_AS_RIGHT;
            }
        } else {
            core::ptr::write_volatile(node_or_parent, current);
            return TABLE_FOUND_NODE;
        }
    }
}

/// `RtlpInsertAvlTreeNode` (Mm flavor).
///
/// Inserts `new_node` into the tree at the position discovered by a prior
/// `Find` call. `node_or_parent` is the parent returned by `Find`;
/// `search_result` is the disposition (`TABLE_INSERT_AS_LEFT`,
/// `TABLE_INSERT_AS_RIGHT`, or `TABLE_EMPTY_TREE`).
///
/// If `search_result == TABLE_FOUND_NODE`, the call returns early with no
/// tree mutation — duplicate insertion would corrupt the AVL tree and
/// cause deadlocks. (The duplicate-insertion guard moved here from the
/// historical C trampoline in `mm_rust_shims.c`; the original DPRINT1 was
/// the diagnostic value, but `debug_assert!` here covers the same hot
/// path in debug builds.)
///
/// Walks back up the tree updating balance factors, rebalancing where
/// needed, and increments `DepthOfTree` if the insert grew the tree.
#[no_mangle]
pub unsafe extern "system" fn RtlpInsertAvlTreeNode(
    table: PMM_AVL_TABLE,
    new_node: PMMADDRESS_NODE,
    mut node_or_parent: PMMADDRESS_NODE,
    search_result: u32,
) {
    if search_result == TABLE_FOUND_NODE {
        ntkrnl_sys::ASSERT!(false, "RtlpInsertAvlTreeNode: duplicate insertion blocked");
        return;
    }

    (*new_node).LeftChild = core::ptr::null_mut();
    (*new_node).RightChild = core::ptr::null_mut();
    set_balance(new_node, BALANCED);

    set_count(table, count(table) + 1);

    if search_result == TABLE_EMPTY_TREE {
        insert_as_right_child(root_node(table), new_node);
        ntkrnl_sys::ASSERT!(depth(table) == 0);
        set_depth(table, 1);
        return;
    }

    if search_result == TABLE_INSERT_AS_LEFT {
        insert_as_left_child(node_or_parent, new_node);
    } else {
        insert_as_right_child(node_or_parent, new_node);
    }

    // Cheat: pre-set BalancedRoot to LEFT_HEAVY so the depth-grew check at the
    // top of the tree falls out of the same branch as a regular balanced parent.
    set_balance(root_node(table), LEFT_HEAVY);

    let mut new_node = new_node;
    loop {
        let bal = if is_left_child(new_node) { LEFT_HEAVY } else { RIGHT_HEAVY };

        let parent_bal = balance(node_or_parent);
        if parent_bal == BALANCED {
            // Parent was balanced — now leaning toward `new_node`'s side.
            set_balance(node_or_parent, bal);
            new_node = node_or_parent;
            node_or_parent = parent(node_or_parent);
        } else if parent_bal != bal {
            // Parent leaned the other way — insertion balanced it.
            set_balance(node_or_parent, BALANCED);
            // The "depth grew" signal: BalancedRoot was set to LEFT_HEAVY at the
            // top of this loop; if it's still LEFT_HEAVY we never touched it,
            // meaning we walked all the way up and the tree depth grew.
            if balance(root_node(table)) == BALANCED {
                set_depth(table, depth(table) + 1);
            }
            break;
        } else {
            // Parent already leaned toward `new_node`'s side — needs rebalance.
            rebalance_node(node_or_parent);
            break;
        }
    }
}

/// `RtlpDeleteAvlTreeNode` (Mm flavor).
///
/// Removes `node` from the tree, swapping in a successor (or predecessor)
/// if `node` has two children, and walking back up to update balance
/// factors / decrement `DepthOfTree` if the tree shrank.
#[no_mangle]
pub unsafe extern "system" fn RtlpDeleteAvlTreeNode(
    table: PMM_AVL_TABLE,
    node: PMMADDRESS_NODE,
) {
    // Pick the actual node to physically unlink. If `node` has at most one
    // child, unlink `node` itself; otherwise pick the in-order successor (or
    // predecessor, depending on which side is heavier) — that node is
    // guaranteed to have at most one child.
    let mut delete_node: PMMADDRESS_NODE = core::ptr::null_mut();
    if left_child(node).is_null() || right_child(node).is_null() {
        delete_node = node;
    }
    if delete_node.is_null() && balance(node) >= BALANCED {
        // Right side heavier or balanced — pick in-order successor.
        delete_node = right_child(node);
        while !left_child(delete_node).is_null() {
            delete_node = left_child(delete_node);
        }
    } else if delete_node.is_null() {
        // Left side heavier — pick in-order predecessor.
        delete_node = left_child(node);
        while !right_child(delete_node).is_null() {
            delete_node = right_child(delete_node);
        }
    }

    let mut parent_node = parent(delete_node);

    // Pick the side of `delete_node` that has a child (or neither).
    let node1: *mut PMMADDRESS_NODE = if !left_child(delete_node).is_null() {
        core::ptr::addr_of_mut!((*delete_node).LeftChild)
    } else {
        core::ptr::addr_of_mut!((*delete_node).RightChild)
    };

    // Pick the parent's slot that points at `delete_node`.
    let node2: *mut PMMADDRESS_NODE = if is_left_child(delete_node) {
        core::ptr::addr_of_mut!((*parent_node).LeftChild)
    } else {
        core::ptr::addr_of_mut!((*parent_node).RightChild)
    };

    let mut bal = if is_left_child(delete_node) { LEFT_HEAVY } else { RIGHT_HEAVY };

    // Splice `delete_node` out: parent's child slot now points at delete_node's
    // child (which may be NULL).
    *node2 = *node1;
    if !(*node1).is_null() {
        set_parent(*node1, parent_node);
    }

    // Cheat: same loop trick as Insert. Pre-set BalancedRoot to BALANCED so the
    // "tree level was lost" detection falls out at the top.
    set_balance(root_node(table), BALANCED);

    loop {
        let parent_bal = balance(parent_node);
        if parent_bal == bal {
            // Parent leaned toward the side we shrank — now balanced.
            set_balance(parent_node, BALANCED);
        } else if parent_bal == BALANCED {
            // Parent was balanced — now leans away from the side we shrank.
            set_balance(parent_node, -bal);
            if balance(root_node(table)) != BALANCED {
                // We walked up to BalancedRoot and it changed — depth shrank.
                set_depth(table, depth(table) - 1);
            }
            break;
        } else {
            // Parent already leaned away — rebalance. If rebalance returns
            // true (Case 3), the propagation terminates immediately.
            if rebalance_node(parent_node) {
                break;
            }
            parent_node = parent(parent_node);
        }

        bal = if is_right_child(parent_node) { RIGHT_HEAVY } else { LEFT_HEAVY };
        parent_node = parent(parent_node);
    }

    // If we physically removed `node` itself, we're done.
    if node == delete_node {
        return;
    }

    // Otherwise, splice `delete_node` (now unlinked) into `node`'s position.
    copy_node_data(delete_node, node);

    let parent_of_delete = parent(delete_node);
    let slot: *mut PMMADDRESS_NODE = if is_left_child(node) {
        core::ptr::addr_of_mut!((*parent_of_delete).LeftChild)
    } else {
        core::ptr::addr_of_mut!((*parent_of_delete).RightChild)
    };
    *slot = delete_node;

    if !left_child(delete_node).is_null() {
        set_parent(left_child(delete_node), delete_node);
    }
    if !right_child(delete_node).is_null() {
        set_parent(right_child(delete_node), delete_node);
    }
}
