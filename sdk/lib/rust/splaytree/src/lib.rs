//! RTL Splay Tree — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/splaytree.c` (796 lines).
//! Seven exported functions: RtlSplay, RtlDelete, RtlDeleteNoSplay,
//! RtlRealPredecessor, RtlRealSuccessor, RtlSubtreePredecessor, RtlSubtreeSuccessor.
//! All architectures.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// Types
// ============================================================================

/// RTL_SPLAY_LINKS — 3 pointers (Parent, LeftChild, RightChild).
#[repr(C)]
pub struct SplayLinks {
    pub parent: *mut SplayLinks,
    pub left_child: *mut SplayLinks,
    pub right_child: *mut SplayLinks,
}

// ============================================================================
// Node accessor helpers
// ============================================================================

#[inline]
unsafe fn parent(n: *mut SplayLinks) -> *mut SplayLinks {
    (*n).parent
}
#[inline]
unsafe fn left(n: *mut SplayLinks) -> *mut SplayLinks {
    (*n).left_child
}
#[inline]
unsafe fn right(n: *mut SplayLinks) -> *mut SplayLinks {
    (*n).right_child
}
#[inline]
unsafe fn set_parent(n: *mut SplayLinks, p: *mut SplayLinks) {
    (*n).parent = p;
}
#[inline]
unsafe fn set_left(n: *mut SplayLinks, c: *mut SplayLinks) {
    (*n).left_child = c;
}
#[inline]
unsafe fn set_right(n: *mut SplayLinks, c: *mut SplayLinks) {
    (*n).right_child = c;
}
#[inline]
unsafe fn is_root(n: *mut SplayLinks) -> bool {
    parent(n) == n
}
#[inline]
unsafe fn is_left_child(n: *mut SplayLinks) -> bool {
    left(parent(n)) == n
}
#[inline]
unsafe fn is_right_child(n: *mut SplayLinks) -> bool {
    right(parent(n)) == n
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Fix up child→parent back-links after a swap.
unsafe fn fixup_child_links(links: *mut SplayLinks, root: bool, was_left_child: bool) {
    if !left(links).is_null() {
        // Re-link left child's parent to us
        set_parent(left(links), links);
        // Redundant set_left but matches C: RtlInsertAsLeftChild sets both
    }
    if !right(links).is_null() {
        set_parent(right(links), links);
    }
    if !root {
        if was_left_child {
            set_left(parent(links), links);
        } else {
            set_right(parent(links), links);
        }
    }
}

/// Swap two nodes in the tree, fixing all parent/child pointers.
unsafe fn swap_splay_links(mut link_a: *mut SplayLinks, mut link_b: *mut SplayLinks) {
    // Normalize: A should be the potential parent/root
    if parent(link_a) == link_b || is_root(link_b) {
        let tmp = link_a;
        link_a = link_b;
        link_b = tmp;
    }

    // Save original states
    let ta = SplayLinks {
        parent: parent(link_a),
        left_child: left(link_a),
        right_child: right(link_a),
    };
    let tb = SplayLinks {
        parent: parent(link_b),
        left_child: left(link_b),
        right_child: right(link_b),
    };
    let root_a = is_root(link_a);
    let left_a = is_left_child(link_a);
    let left_b = is_left_child(link_b);

    // Swap the node contents
    (*link_b).parent = ta.parent;
    (*link_b).left_child = ta.left_child;
    (*link_b).right_child = ta.right_child;
    (*link_a).parent = tb.parent;
    (*link_a).left_child = tb.left_child;
    (*link_a).right_child = tb.right_child;

    // Special case: A was parent of B
    if tb.parent == link_a {
        if !root_a {
            if left_a {
                set_left(ta.parent, link_b);
            } else {
                set_right(ta.parent, link_b);
            }
        }
        if left_b {
            set_left(link_b, link_a);
        } else {
            set_right(link_b, link_a);
        }
    }

    // When A was B's parent, the special case above already wired A in under B
    // (and repointed A's old parent at B). A's recorded parent (tb.parent) is
    // A's *own* address, so letting fixup relink A into "its parent" runs
    // set_left/right(A, A) and self-corrupts A's child pointer (orphaning A's
    // real child). Treat A as not-needing-a-parent-relink in that case. (Bug:
    // two-children delete where the predecessor is n's direct left child.)
    let a_was_parent_of_b = tb.parent == link_a;
    fixup_child_links(link_a, a_was_parent_of_b, left_b);
    fixup_child_links(link_b, root_a, left_a);

    // A was root: B becomes root (parent = self)
    if root_a {
        set_parent(link_b, link_b);
    }
}

// ============================================================================
// Core algorithm
// ============================================================================

/// Splay a node to the root of the tree.
pub unsafe fn splay(links: *mut SplayLinks) -> *mut SplayLinks {
    let n = links;

    while !is_root(n) {
        let p = parent(n);
        let g = parent(p);

        if is_left_child(n) {
            if is_left_child(p) {
                // Case 1: N=left(P), P=left(G) — zig-zig
                set_left(p, right(n));
                set_left(g, right(p));
                if !left(p).is_null() { set_parent(left(p), p); }
                if !left(g).is_null() { set_parent(left(g), g); }

                if is_root(g) {
                    set_parent(n, n);
                } else {
                    set_parent(n, parent(g));
                    if is_left_child(g) {
                        set_left(parent(g), n);
                    } else {
                        set_right(parent(g), n);
                    }
                }
                set_right(n, p);
                set_parent(p, n);
                set_right(p, g);
                set_parent(g, p);
            } else if is_right_child(p) {
                // Case 3: N=left(P), P=right(G) — zig-zag
                set_right(g, left(n));
                set_left(p, right(n));
                if !right(g).is_null() { set_parent(right(g), g); }
                if !left(p).is_null() { set_parent(left(p), p); }

                if is_root(g) {
                    set_parent(n, n);
                } else {
                    set_parent(n, parent(g));
                    if is_left_child(g) {
                        set_left(parent(g), n);
                    } else {
                        set_right(parent(g), n);
                    }
                }
                set_left(n, g);
                set_parent(g, n);
                set_right(n, p);
                set_parent(p, n);
            } else {
                // P is root — simple zig
                set_left(p, right(n));
                if !left(p).is_null() { set_parent(left(p), p); }
                set_parent(n, n);
                set_right(n, p);
                set_parent(p, n);
            }
        } else {
            // N is right child of P
            if is_right_child(p) {
                // Case 2: N=right(P), P=right(G) — zag-zag
                set_right(g, left(p));
                set_right(p, left(n));
                if !right(g).is_null() { set_parent(right(g), g); }
                if !right(p).is_null() { set_parent(right(p), p); }

                if is_root(g) {
                    set_parent(n, n);
                } else {
                    set_parent(n, parent(g));
                    if is_left_child(g) {
                        set_left(parent(g), n);
                    } else {
                        set_right(parent(g), n);
                    }
                }
                set_left(n, p);
                set_parent(p, n);
                set_left(p, g);
                set_parent(g, p);
            } else if is_left_child(p) {
                // Case 4: N=right(P), P=left(G) — zag-zig
                set_right(p, left(n));
                set_left(g, right(n));
                if !right(p).is_null() { set_parent(right(p), p); }
                if !left(g).is_null() { set_parent(left(g), g); }

                if is_root(g) {
                    set_parent(n, n);
                } else {
                    set_parent(n, parent(g));
                    if is_left_child(g) {
                        set_left(parent(g), n);
                    } else {
                        set_right(parent(g), n);
                    }
                }
                set_left(n, p);
                set_parent(g, n);
                set_right(n, g);
                set_parent(p, n);
            } else {
                // P is root — simple zag
                set_right(p, left(n));
                if !right(p).is_null() { set_parent(right(p), p); }
                set_parent(n, n);
                set_left(n, p);
                set_parent(p, n);
            }
        }
    }

    n
}

/// Get the in-order predecessor (left subtree's rightmost, or walk up).
pub unsafe fn real_predecessor(links: *mut SplayLinks) -> *mut SplayLinks {
    let mut child = left(links);
    if !child.is_null() {
        while !right(child).is_null() {
            child = right(child);
        }
        return child;
    }

    child = links;
    while is_left_child(child) {
        child = parent(child);
    }
    if is_right_child(child) {
        parent(child)
    } else {
        core::ptr::null_mut()
    }
}

/// Get the in-order successor (right subtree's leftmost, or walk up).
pub unsafe fn real_successor(links: *mut SplayLinks) -> *mut SplayLinks {
    let mut child = right(links);
    if !child.is_null() {
        while !left(child).is_null() {
            child = left(child);
        }
        return child;
    }

    child = links;
    while is_right_child(child) {
        child = parent(child);
    }
    if is_left_child(child) {
        parent(child)
    } else {
        core::ptr::null_mut()
    }
}

/// Get the leftmost node of the left subtree (subtree predecessor).
pub unsafe fn subtree_predecessor(links: *mut SplayLinks) -> *mut SplayLinks {
    let mut child = left(links);
    if child.is_null() {
        return core::ptr::null_mut();
    }
    while !right(child).is_null() {
        child = right(child);
    }
    child
}

/// Get the rightmost node of the right subtree (subtree successor).
pub unsafe fn subtree_successor(links: *mut SplayLinks) -> *mut SplayLinks {
    let mut child = right(links);
    if child.is_null() {
        return core::ptr::null_mut();
    }
    while !left(child).is_null() {
        child = left(child);
    }
    child
}

/// Delete a node and splay the parent to root. Returns the new root (or null if tree empty).
pub unsafe fn delete(links: *mut SplayLinks) -> *mut SplayLinks {
    let n = links;

    // If node has two children, swap with predecessor
    if !left(n).is_null() && !right(n).is_null() {
        let sp = subtree_predecessor(n);
        swap_splay_links(sp, n);
    }

    // No children — just unlink
    if left(n).is_null() && right(n).is_null() {
        if is_root(n) {
            return core::ptr::null_mut();
        }
        let p = parent(n);
        if is_left_child(n) {
            set_left(p, core::ptr::null_mut());
        } else {
            set_right(p, core::ptr::null_mut());
        }
        return splay(p);
    }

    // One child — link child to parent
    let child = if !left(n).is_null() { left(n) } else { right(n) };

    if is_root(n) {
        set_parent(child, child);
        return child;
    }

    let p = parent(n);
    if is_left_child(n) {
        set_left(p, child);
    } else {
        set_right(p, child);
    }
    set_parent(child, p);
    splay(p)
}

/// Delete a node without splaying. Updates *root if the root changes.
pub unsafe fn delete_no_splay(links: *mut SplayLinks, root: *mut *mut SplayLinks) {
    let n = links;

    if !left(n).is_null() && !right(n).is_null() {
        let sp = subtree_predecessor(n);
        if is_root(n) {
            *root = sp;
        }
        swap_splay_links(sp, n);
    }

    if left(n).is_null() && right(n).is_null() {
        if is_root(n) {
            *root = core::ptr::null_mut();
            return;
        }
        let p = parent(n);
        if is_left_child(n) {
            set_left(p, core::ptr::null_mut());
        } else {
            set_right(p, core::ptr::null_mut());
        }
        return;
    }

    let child = if !left(n).is_null() { left(n) } else { right(n) };

    if is_root(n) {
        set_parent(child, child);
        *root = child;
        return;
    }

    let p = parent(n);
    if is_left_child(n) {
        set_left(p, child);
    } else {
        set_right(p, child);
    }
    set_parent(child, p);
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlSplay(links: *mut SplayLinks) -> *mut SplayLinks {
        splay(links)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDelete(links: *mut SplayLinks) -> *mut SplayLinks {
        delete(links)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDeleteNoSplay(
        links: *mut SplayLinks,
        root: *mut *mut SplayLinks,
    ) {
        delete_no_splay(links, root);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlRealPredecessor(
        links: *mut SplayLinks,
    ) -> *mut SplayLinks {
        real_predecessor(links)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlRealSuccessor(
        links: *mut SplayLinks,
    ) -> *mut SplayLinks {
        real_successor(links)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSubtreePredecessor(
        links: *mut SplayLinks,
    ) -> *mut SplayLinks {
        subtree_predecessor(links)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSubtreeSuccessor(
        links: *mut SplayLinks,
    ) -> *mut SplayLinks {
        subtree_successor(links)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    struct TestTree {
        nodes: Vec<Box<SplayLinks>>,
    }

    impl TestTree {
        fn new() -> Self {
            TestTree { nodes: Vec::new() }
        }

        fn alloc_node(&mut self) -> *mut SplayLinks {
            let mut node = Box::new(SplayLinks {
                parent: core::ptr::null_mut(),
                left_child: core::ptr::null_mut(),
                right_child: core::ptr::null_mut(),
            });
            let ptr = &mut *node as *mut SplayLinks;
            // Initialize as root (parent = self)
            node.parent = ptr;
            self.nodes.push(node);
            ptr
        }

        /// Insert as left child, return inserted node.
        unsafe fn insert_left(&mut self, parent: *mut SplayLinks) -> *mut SplayLinks {
            let child = self.alloc_node();
            (*parent).left_child = child;
            (*child).parent = parent;
            child
        }

        /// Insert as right child, return inserted node.
        unsafe fn insert_right(&mut self, parent: *mut SplayLinks) -> *mut SplayLinks {
            let child = self.alloc_node();
            (*parent).right_child = child;
            (*child).parent = parent;
            child
        }
    }

    /// Collect in-order traversal of the tree.
    unsafe fn inorder(root: *mut SplayLinks) -> Vec<*mut SplayLinks> {
        let mut result = Vec::new();
        if root.is_null() {
            return result;
        }
        // Find leftmost
        let mut node = root;
        while !left(node).is_null() {
            node = left(node);
        }
        loop {
            result.push(node);
            let next = real_successor(node);
            if next.is_null() {
                break;
            }
            node = next;
        }
        result
    }

    /// Verify parent-child link consistency.
    unsafe fn verify_links(root: *mut SplayLinks) {
        if root.is_null() {
            return;
        }
        assert!(is_root(root), "Root's parent should be itself");
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let l = left(node);
            let r = right(node);
            if !l.is_null() {
                assert_eq!(parent(l), node, "Left child's parent mismatch");
                stack.push(l);
            }
            if !r.is_null() {
                assert_eq!(parent(r), node, "Right child's parent mismatch");
                stack.push(r);
            }
        }
    }

    #[test]
    fn test_single_node_splay() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let result = splay(root);
            assert_eq!(result, root);
            assert!(is_root(root));
        }
    }

    #[test]
    fn test_left_child_splay() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let child = tree.insert_left(root);
            let new_root = splay(child);
            assert_eq!(new_root, child);
            assert!(is_root(child));
            assert_eq!(right(child), root);
            assert_eq!(parent(root), child);
            verify_links(new_root);
        }
    }

    #[test]
    fn test_right_child_splay() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let child = tree.insert_right(root);
            let new_root = splay(child);
            assert_eq!(new_root, child);
            assert!(is_root(child));
            assert_eq!(left(child), root);
            verify_links(new_root);
        }
    }

    #[test]
    fn test_zig_zig_splay() {
        // Build: root -> left(A) -> left(B)
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let a = tree.insert_left(root);
            let b = tree.insert_left(a);
            let new_root = splay(b);
            assert_eq!(new_root, b);
            assert!(is_root(b));
            verify_links(new_root);
        }
    }

    #[test]
    fn test_zig_zag_splay() {
        // Build: root -> right(A) -> left(B)
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let a = tree.insert_right(root);
            let b = tree.insert_left(a);
            let new_root = splay(b);
            assert_eq!(new_root, b);
            assert!(is_root(b));
            verify_links(new_root);
        }
    }

    #[test]
    fn test_predecessor_successor() {
        // Build a manual BST: B(root) -> A(left), C(right)
        let mut tree = TestTree::new();
        let b = tree.alloc_node();
        unsafe {
            let a = tree.insert_left(b);
            let c = tree.insert_right(b);

            assert_eq!(real_predecessor(b), a);
            assert_eq!(real_successor(b), c);
            assert!(real_predecessor(a).is_null());
            assert_eq!(real_successor(a), b);
            assert_eq!(real_predecessor(c), b);
            assert!(real_successor(c).is_null());
        }
    }

    #[test]
    fn test_subtree_predecessor_successor() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let _l = tree.insert_left(root);
            let _r = tree.insert_right(root);

            assert_eq!(subtree_predecessor(root), _l);
            assert_eq!(subtree_successor(root), _r);

            // Leaf nodes have null subtree pred/succ
            assert!(subtree_predecessor(_l).is_null());
            assert!(subtree_successor(_r).is_null());
        }
    }

    #[test]
    fn test_delete_leaf() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let child = tree.insert_left(root);
            let new_root = delete(child);
            assert_eq!(new_root, root);
            assert!(is_root(root));
            assert!(left(root).is_null());
            verify_links(new_root);
        }
    }

    #[test]
    fn test_delete_root_only() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let new_root = delete(root);
            assert!(new_root.is_null());
        }
    }

    #[test]
    fn test_delete_root_with_one_child() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let child = tree.insert_left(root);
            let new_root = delete(root);
            assert_eq!(new_root, child);
            assert!(is_root(child));
            verify_links(new_root);
        }
    }

    #[test]
    fn test_delete_no_splay_leaf() {
        let mut tree = TestTree::new();
        let root = tree.alloc_node();
        unsafe {
            let child = tree.insert_right(root);
            let mut root_ptr = root;
            delete_no_splay(child, &mut root_ptr);
            assert_eq!(root_ptr, root);
            assert!(right(root).is_null());
        }
    }

    #[test]
    fn test_inorder_traversal() {
        // Build: B(root) -> A(left), D(right) -> C(D.left)
        let mut tree = TestTree::new();
        let b = tree.alloc_node();
        unsafe {
            let a = tree.insert_left(b);
            let d = tree.insert_right(b);
            let c = tree.insert_left(d);

            let order = inorder(b);
            assert_eq!(order.len(), 4);
            assert_eq!(order[0], a);
            assert_eq!(order[1], b);
            assert_eq!(order[2], c);
            assert_eq!(order[3], d);
        }
    }

    #[test]
    fn test_splay_preserves_order() {
        // Build: D(root) -> B(left) -> A(B.left), C(B.right)
        //                 -> F(right) -> E(F.left), G(F.right)
        let mut tree = TestTree::new();
        let d = tree.alloc_node();
        unsafe {
            let b = tree.insert_left(d);
            let _a = tree.insert_left(b);
            let _c = tree.insert_right(b);
            let f = tree.insert_right(d);
            let _e = tree.insert_left(f);
            let _g = tree.insert_right(f);

            let order_before = inorder(d);
            assert_eq!(order_before.len(), 7);

            // Splay node A to root
            let new_root = splay(_a);
            verify_links(new_root);

            let order_after = inorder(new_root);
            assert_eq!(order_after.len(), 7);
            // In-order traversal should be preserved
            assert_eq!(order_before, order_after);
        }
    }

    #[test]
    fn test_delete_two_children() {
        // The two-children delete path (subtree_predecessor + swap_splay_links)
        // was previously untested. Build a balanced BST (in-order A..G) and
        // delete the root D, which has two children.
        //         D
        //       /   \
        //      B     F
        //     / \   / \
        //    A  C  E   G
        let mut tree = TestTree::new();
        let d = tree.alloc_node();
        unsafe {
            let b = tree.insert_left(d);
            let a = tree.insert_left(b);
            let c = tree.insert_right(b);
            let f = tree.insert_right(d);
            let e = tree.insert_left(f);
            let g = tree.insert_right(f);

            assert_eq!(inorder(d), vec![a, b, c, d, e, f, g]);

            let new_root = delete(d);
            verify_links(new_root);
            // D is gone; the in-order of the remaining nodes is preserved.
            assert_eq!(inorder(new_root), vec![a, b, c, e, f, g]);
        }
    }

    #[test]
    fn test_delete_two_children_pred_is_left_child() {
        // Two-children delete where the predecessor is n's DIRECT left child
        // (the "A was parent of B" branch of swap_splay_links). In-order A B D F.
        //       D
        //      / \
        //     B   F
        //    /
        //   A
        let mut tree = TestTree::new();
        let d = tree.alloc_node();
        unsafe {
            let b = tree.insert_left(d);
            let a = tree.insert_left(b);
            let f = tree.insert_right(d);
            assert_eq!(inorder(d), vec![a, b, d, f]);

            let new_root = delete(d);
            verify_links(new_root);
            assert_eq!(inorder(new_root), vec![a, b, f]);
        }
    }

    #[test]
    fn random_insert_delete_model() {
        // Drive splay/delete as a keyed BST against a BTreeMap reference, with a
        // full link + in-order check after EVERY operation. Nodes are never
        // freed/reused, so any corruption is a genuine link bug (not memory
        // reuse). This is the first test to exercise the two-children delete
        // path (swap_splay_links) under random structures.
        use std::collections::BTreeMap;
        use std::collections::HashMap;

        // SplitMix64 — deterministic, dependency-free.
        struct Rng(u64);
        impl Rng {
            fn next(&mut self) -> u64 {
                self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = self.0;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^ (z >> 31)
            }
        }

        let mut tree = TestTree::new();
        let mut rng = Rng(0x5EED);
        let mut model: BTreeMap<u32, *mut SplayLinks> = BTreeMap::new();
        let mut keyof: HashMap<usize, u32> = HashMap::new();
        let mut root: *mut SplayLinks = core::ptr::null_mut();

        unsafe {
            for step in 0..4000usize {
                let k = (rng.next() % 64) as u32;
                if rng.next() & 1 == 0 {
                    // insert if absent
                    if model.contains_key(&k) {
                        continue;
                    }
                    let node = tree.alloc_node(); // parent = self (root marker)
                    keyof.insert(node as usize, k);
                    if root.is_null() {
                        root = node;
                    } else {
                        let mut cur = root;
                        loop {
                            let ck = keyof[&(cur as usize)];
                            if k < ck {
                                if left(cur).is_null() {
                                    (*cur).left_child = node;
                                    (*node).parent = cur;
                                    break;
                                }
                                cur = left(cur);
                            } else {
                                if right(cur).is_null() {
                                    (*cur).right_child = node;
                                    (*node).parent = cur;
                                    break;
                                }
                                cur = right(cur);
                            }
                        }
                        root = splay(node);
                    }
                    model.insert(k, node);
                } else if let Some(&node) = model.get(&k) {
                    root = delete(node);
                    model.remove(&k);
                }

                verify_links(root);
                let got: Vec<u32> = inorder(root).iter().map(|&n| keyof[&(n as usize)]).collect();
                let want: Vec<u32> = model.keys().copied().collect();
                assert_eq!(got, want, "mismatch after step {step} (key {k})");
            }
        }
    }
}
