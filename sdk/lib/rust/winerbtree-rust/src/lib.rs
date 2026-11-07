//! `sdk/include/wine/rbtree.h`, ported once: Wine's intrusive red-black tree.
//!
//! Every function in the header is `static inline`, so bindgen emits none of them and the
//! port's `headers` stage leaves a platform header to a shared crate. wined3d is the first
//! translated user; d3d10, d3d11, d3dx9, d3dcompiler, dwrite, dbghelp and jscript include it
//! too. Like `wined3d-h`, the bodies are a macro, [`wine_rb_inlines!`], expanded in each
//! consumer's prelude: the consumer's bindgen island already defines `rb_entry`, `rb_tree` and
//! `rb_compare_func_t`, and those names resolve where the macro expands (only locals and labels
//! are hygienic), so the functions and the consumer's structs agree on one type with no
//! blocklisting.
//!
//! The header's `wine_*` names (`#define wine_rb_put rb_put`, kept "for backwards
//! compatibility" and still what wined3d calls) are re-exported under both spellings, and
//! ReactOS's `wine_rb_clear` (`#ifdef __REACTOS__`) with them.
#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

/// `RB_ENTRY_VALUE(element, type, field)` / `WINE_RB_ENTRY_VALUE`: the containing struct of an
/// embedded `rb_entry`. Wrapping arithmetic, because the header's iteration macros compare the
/// cursor against `RB_ENTRY_VALUE(0, ..)` -- a container-of of NULL.
#[macro_export]
macro_rules! RB_ENTRY_VALUE {
    ($element:expr, $type:ty, $($field:tt)+) => {
        (($element) as *const u8).wrapping_sub(::core::mem::offset_of!($type, $($field)+))
            as *mut $type
    };
}

/// The old name the header keeps (`#define WINE_RB_ENTRY_VALUE RB_ENTRY_VALUE`).
#[macro_export]
macro_rules! WINE_RB_ENTRY_VALUE {
    ($($t:tt)*) => { $crate::RB_ENTRY_VALUE!($($t)*) };
}

/// `RB_FLAG_RED`
pub const RB_FLAG_RED: u32 = 0x1;

/// Expands wine/rbtree.h's functions against the CALLER's `rb_entry`, `rb_tree` and
/// `rb_compare_func_t`, under both the `rb_*` and the `wine_rb_*` names.
#[macro_export]
macro_rules! wine_rb_inlines {
    () => {
        /// `typedef void (rb_traverse_func_t)(struct rb_entry *entry, void *context)`; a
        /// `rb_traverse_func_t *` parameter is this `Option<fn>`, which bindgen also uses.
        pub type rb_traverse_func_t = ::core::option::Option<
            unsafe extern "C" fn(entry: *mut rb_entry, context: *mut ::core::ffi::c_void),
        >;

        #[inline]
        pub unsafe fn rb_is_red(entry: *mut rb_entry) -> ::core::ffi::c_int {
            (!entry.is_null() && ((*entry).flags & $crate::RB_FLAG_RED) != 0) as ::core::ffi::c_int
        }

        #[inline]
        pub unsafe fn rb_rotate_left(tree: *mut rb_tree, e: *mut rb_entry) {
            let right = (*e).right;
            if (*e).parent.is_null() {
                (*tree).root = right;
            } else if (*(*e).parent).left == e {
                (*(*e).parent).left = right;
            } else {
                (*(*e).parent).right = right;
            }
            (*e).right = (*right).left;
            if !(*e).right.is_null() {
                (*(*e).right).parent = e;
            }
            (*right).left = e;
            (*right).parent = (*e).parent;
            (*e).parent = right;
        }

        #[inline]
        pub unsafe fn rb_rotate_right(tree: *mut rb_tree, e: *mut rb_entry) {
            let left = (*e).left;
            if (*e).parent.is_null() {
                (*tree).root = left;
            } else if (*(*e).parent).left == e {
                (*(*e).parent).left = left;
            } else {
                (*(*e).parent).right = left;
            }
            (*e).left = (*left).right;
            if !(*e).left.is_null() {
                (*(*e).left).parent = e;
            }
            (*left).right = e;
            (*left).parent = (*e).parent;
            (*e).parent = left;
        }

        #[inline]
        pub unsafe fn rb_flip_color(entry: *mut rb_entry) {
            (*entry).flags ^= $crate::RB_FLAG_RED;
            (*(*entry).left).flags ^= $crate::RB_FLAG_RED;
            (*(*entry).right).flags ^= $crate::RB_FLAG_RED;
        }

        #[inline]
        pub unsafe fn rb_head(mut iter: *mut rb_entry) -> *mut rb_entry {
            if iter.is_null() {
                return ::core::ptr::null_mut();
            }
            while !(*iter).left.is_null() {
                iter = (*iter).left;
            }
            iter
        }

        #[inline]
        pub unsafe fn rb_tail(mut iter: *mut rb_entry) -> *mut rb_entry {
            if iter.is_null() {
                return ::core::ptr::null_mut();
            }
            while !(*iter).right.is_null() {
                iter = (*iter).right;
            }
            iter
        }

        #[inline]
        pub unsafe fn rb_next(mut iter: *mut rb_entry) -> *mut rb_entry {
            if !(*iter).right.is_null() {
                return rb_head((*iter).right);
            }
            while !(*iter).parent.is_null() && (*(*iter).parent).right == iter {
                iter = (*iter).parent;
            }
            (*iter).parent
        }

        #[inline]
        pub unsafe fn rb_prev(mut iter: *mut rb_entry) -> *mut rb_entry {
            if !(*iter).left.is_null() {
                return rb_tail((*iter).left);
            }
            while !(*iter).parent.is_null() && (*(*iter).parent).left == iter {
                iter = (*iter).parent;
            }
            (*iter).parent
        }

        #[inline]
        pub unsafe fn rb_postorder_head(mut iter: *mut rb_entry) -> *mut rb_entry {
            if iter.is_null() {
                return ::core::ptr::null_mut();
            }
            loop {
                while !(*iter).left.is_null() {
                    iter = (*iter).left;
                }
                if (*iter).right.is_null() {
                    return iter;
                }
                iter = (*iter).right;
            }
        }

        #[inline]
        pub unsafe fn rb_postorder_next(iter: *mut rb_entry) -> *mut rb_entry {
            let parent = (*iter).parent;
            if parent.is_null() {
                return ::core::ptr::null_mut();
            }
            if iter == (*parent).right || (*parent).right.is_null() {
                return parent;
            }
            rb_postorder_head((*parent).right)
        }

        /// `RB_FOR_EACH_DESTRUCTOR(iter, next, tree) callback(iter, context);` -- the next
        /// cursor is taken BEFORE the callback, which may free the entry.
        #[inline]
        pub unsafe fn rb_postorder(
            tree: *mut rb_tree,
            callback: rb_traverse_func_t,
            context: *mut ::core::ffi::c_void,
        ) {
            let mut iter = rb_postorder_head((*tree).root);
            while !iter.is_null() {
                let next = rb_postorder_next(iter);
                (callback.unwrap_unchecked())(iter, context);
                iter = next;
            }
        }

        #[inline]
        pub unsafe fn rb_init(tree: *mut rb_tree, compare: rb_compare_func_t) {
            (*tree).compare = compare;
            (*tree).root = ::core::ptr::null_mut();
        }

        /// `RB_FOR_EACH(iter, tree) callback(iter, context);` (in order; the step reads the
        /// entry after the callback, as the C does).
        #[inline]
        pub unsafe fn rb_for_each_entry(
            tree: *mut rb_tree,
            callback: rb_traverse_func_t,
            context: *mut ::core::ffi::c_void,
        ) {
            let mut iter = rb_head((*tree).root);
            while !iter.is_null() {
                (callback.unwrap_unchecked())(iter, context);
                iter = rb_next(iter);
            }
        }

        #[inline]
        pub unsafe fn rb_destroy(
            tree: *mut rb_tree,
            callback: rb_traverse_func_t,
            context: *mut ::core::ffi::c_void,
        ) {
            // Postorder: the callback will likely free the entry.
            if callback.is_some() {
                rb_postorder(tree, callback, context);
            }
            (*tree).root = ::core::ptr::null_mut();
        }

        #[inline]
        pub unsafe fn rb_get(tree: *const rb_tree, key: *const ::core::ffi::c_void) -> *mut rb_entry {
            let mut entry = (*tree).root;
            while !entry.is_null() {
                let c = ((*tree).compare.unwrap_unchecked())(key, entry);
                if c == 0 {
                    return entry;
                }
                entry = if c < 0 { (*entry).left } else { (*entry).right };
            }
            ::core::ptr::null_mut()
        }

        #[inline]
        pub unsafe fn rb_put(
            tree: *mut rb_tree,
            key: *const ::core::ffi::c_void,
            mut entry: *mut rb_entry,
        ) -> ::core::ffi::c_int {
            let mut iter: *mut *mut rb_entry = ::core::ptr::addr_of_mut!((*tree).root);
            let mut parent = (*tree).root;
            while !(*iter).is_null() {
                parent = *iter;
                let c = ((*tree).compare.unwrap_unchecked())(key, parent);
                if c == 0 {
                    return -1;
                } else if c < 0 {
                    iter = ::core::ptr::addr_of_mut!((*parent).left);
                } else {
                    iter = ::core::ptr::addr_of_mut!((*parent).right);
                }
            }
            (*entry).flags = $crate::RB_FLAG_RED;
            (*entry).parent = parent;
            (*entry).left = ::core::ptr::null_mut();
            (*entry).right = ::core::ptr::null_mut();
            *iter = entry;

            while rb_is_red((*entry).parent) != 0 {
                let p = (*entry).parent;
                let g = (*p).parent;
                if p == (*g).left {
                    if rb_is_red((*g).right) != 0 {
                        rb_flip_color(g);
                        entry = g;
                    } else {
                        if entry == (*p).right {
                            entry = p;
                            rb_rotate_left(tree, entry);
                        }
                        (*(*entry).parent).flags &= !$crate::RB_FLAG_RED;
                        (*(*(*entry).parent).parent).flags |= $crate::RB_FLAG_RED;
                        rb_rotate_right(tree, (*(*entry).parent).parent);
                    }
                } else if rb_is_red((*g).left) != 0 {
                    rb_flip_color(g);
                    entry = g;
                } else {
                    if entry == (*p).left {
                        entry = p;
                        rb_rotate_right(tree, entry);
                    }
                    (*(*entry).parent).flags &= !$crate::RB_FLAG_RED;
                    (*(*(*entry).parent).parent).flags |= $crate::RB_FLAG_RED;
                    rb_rotate_left(tree, (*(*entry).parent).parent);
                }
            }
            (*(*tree).root).flags &= !$crate::RB_FLAG_RED;
            0
        }

        #[inline]
        pub unsafe fn rb_remove(tree: *mut rb_tree, entry: *mut rb_entry) {
            let mut iter: *mut rb_entry;
            let mut child: *mut rb_entry;
            let mut parent: *mut rb_entry;
            let mut w: *mut rb_entry;

            if !(*entry).right.is_null() && !(*entry).left.is_null() {
                iter = (*entry).right;
                while !(*iter).left.is_null() {
                    iter = (*iter).left;
                }
            } else {
                iter = entry;
            }

            child = if !(*iter).left.is_null() { (*iter).left } else { (*iter).right };

            if (*iter).parent.is_null() {
                (*tree).root = child;
            } else if iter == (*(*iter).parent).left {
                (*(*iter).parent).left = child;
            } else {
                (*(*iter).parent).right = child;
            }

            if !child.is_null() {
                (*child).parent = (*iter).parent;
            }
            parent = (*iter).parent;

            let need_fixup = rb_is_red(iter) == 0;

            if entry != iter {
                *iter = *entry;
                if (*iter).parent.is_null() {
                    (*tree).root = iter;
                } else if entry == (*(*iter).parent).left {
                    (*(*iter).parent).left = iter;
                } else {
                    (*(*iter).parent).right = iter;
                }
                if !(*iter).right.is_null() {
                    (*(*iter).right).parent = iter;
                }
                if !(*iter).left.is_null() {
                    (*(*iter).left).parent = iter;
                }
                if parent == entry {
                    parent = iter;
                }
            }

            if need_fixup {
                while !parent.is_null() && rb_is_red(child) == 0 {
                    if child == (*parent).left {
                        w = (*parent).right;
                        if rb_is_red(w) != 0 {
                            (*w).flags &= !$crate::RB_FLAG_RED;
                            (*parent).flags |= $crate::RB_FLAG_RED;
                            rb_rotate_left(tree, parent);
                            w = (*parent).right;
                        }
                        if rb_is_red((*w).left) != 0 || rb_is_red((*w).right) != 0 {
                            if rb_is_red((*w).right) == 0 {
                                (*(*w).left).flags &= !$crate::RB_FLAG_RED;
                                (*w).flags |= $crate::RB_FLAG_RED;
                                rb_rotate_right(tree, w);
                                w = (*parent).right;
                            }
                            (*w).flags = ((*w).flags & !$crate::RB_FLAG_RED)
                                | ((*parent).flags & $crate::RB_FLAG_RED);
                            (*parent).flags &= !$crate::RB_FLAG_RED;
                            if !(*w).right.is_null() {
                                (*(*w).right).flags &= !$crate::RB_FLAG_RED;
                            }
                            rb_rotate_left(tree, parent);
                            child = ::core::ptr::null_mut();
                            break;
                        }
                    } else {
                        w = (*parent).left;
                        if rb_is_red(w) != 0 {
                            (*w).flags &= !$crate::RB_FLAG_RED;
                            (*parent).flags |= $crate::RB_FLAG_RED;
                            rb_rotate_right(tree, parent);
                            w = (*parent).left;
                        }
                        if rb_is_red((*w).left) != 0 || rb_is_red((*w).right) != 0 {
                            if rb_is_red((*w).left) == 0 {
                                (*(*w).right).flags &= !$crate::RB_FLAG_RED;
                                (*w).flags |= $crate::RB_FLAG_RED;
                                rb_rotate_left(tree, w);
                                w = (*parent).left;
                            }
                            (*w).flags = ((*w).flags & !$crate::RB_FLAG_RED)
                                | ((*parent).flags & $crate::RB_FLAG_RED);
                            (*parent).flags &= !$crate::RB_FLAG_RED;
                            if !(*w).left.is_null() {
                                (*(*w).left).flags &= !$crate::RB_FLAG_RED;
                            }
                            rb_rotate_right(tree, parent);
                            child = ::core::ptr::null_mut();
                            break;
                        }
                    }
                    (*w).flags |= $crate::RB_FLAG_RED;
                    child = parent;
                    parent = (*child).parent;
                }
                if !child.is_null() {
                    (*child).flags &= !$crate::RB_FLAG_RED;
                }
            }

            if !(*tree).root.is_null() {
                (*(*tree).root).flags &= !$crate::RB_FLAG_RED;
            }
        }

        #[inline]
        pub unsafe fn rb_remove_key(tree: *mut rb_tree, key: *const ::core::ffi::c_void) {
            let entry = rb_get(tree, key);
            if !entry.is_null() {
                rb_remove(tree, entry);
            }
        }

        #[inline]
        pub unsafe fn rb_replace(tree: *mut rb_tree, dst: *mut rb_entry, src: *mut rb_entry) {
            (*src).parent = (*dst).parent;
            if (*src).parent.is_null() {
                (*tree).root = src;
            } else if (*(*dst).parent).left == dst {
                (*(*dst).parent).left = src;
            } else {
                (*(*dst).parent).right = src;
            }
            (*src).left = (*dst).left;
            if !(*src).left.is_null() {
                (*(*src).left).parent = src;
            }
            (*src).right = (*dst).right;
            if !(*src).right.is_null() {
                (*(*src).right).parent = src;
            }
            (*src).flags = (*dst).flags;
        }

        // "old names for backwards compatibility", and ReactOS's wine_rb_clear.
        #[allow(unused_imports)]
        pub use self::{
            rb_destroy as wine_rb_destroy, rb_destroy as wine_rb_clear,
            rb_for_each_entry as wine_rb_for_each_entry, rb_get as wine_rb_get,
            rb_init as wine_rb_init, rb_put as wine_rb_put, rb_remove as wine_rb_remove,
            rb_remove_key as wine_rb_remove_key, rb_replace as wine_rb_replace,
        };
        #[allow(dead_code)]
        pub type wine_rb_entry = rb_entry;
        #[allow(dead_code)]
        pub type wine_rb_tree = rb_tree;
    };
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::vec::Vec;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct rb_entry {
        pub parent: *mut rb_entry,
        pub left: *mut rb_entry,
        pub right: *mut rb_entry,
        pub flags: core::ffi::c_uint,
    }
    pub type rb_compare_func_t = Option<
        unsafe extern "C" fn(key: *const core::ffi::c_void, entry: *const rb_entry) -> core::ffi::c_int,
    >;
    #[repr(C)]
    pub struct rb_tree {
        pub compare: rb_compare_func_t,
        pub root: *mut rb_entry,
    }
    crate::wine_rb_inlines!();

    #[repr(C)]
    struct Node {
        key: i32,
        entry: rb_entry,
    }

    unsafe extern "C" fn compare(key: *const core::ffi::c_void, entry: *const rb_entry) -> i32 {
        let node = crate::RB_ENTRY_VALUE!(entry, Node, entry);
        let k = *(key as *const i32);
        (k > (*node).key) as i32 - (k < (*node).key) as i32
    }

    /// Red-black invariants: a red node has no red child, every root-leaf path has the same
    /// black count, parents link back, and an in-order walk is sorted. Returns the black height.
    unsafe fn check(e: *mut rb_entry, parent: *mut rb_entry) -> usize {
        if e.is_null() {
            return 1;
        }
        assert_eq!((*e).parent, parent);
        if rb_is_red(e) != 0 {
            assert!(rb_is_red((*e).left) == 0 && rb_is_red((*e).right) == 0);
        }
        let l = check((*e).left, e);
        assert_eq!(l, check((*e).right, e));
        l + (rb_is_red(e) == 0) as usize
    }

    unsafe fn keys(tree: *mut rb_tree) -> Vec<i32> {
        let mut out = Vec::new();
        let mut it = rb_head((*tree).root);
        while !it.is_null() {
            out.push((*crate::WINE_RB_ENTRY_VALUE!(it, Node, entry)).key);
            it = rb_next(it);
        }
        out
    }

    #[test]
    fn insert_remove_keeps_the_invariants_and_the_order() {
        unsafe {
            let mut tree = rb_tree { compare: None, root: core::ptr::null_mut() };
            wine_rb_init(&mut tree, Some(compare));
            let order = [50, 20, 80, 10, 30, 70, 90, 25, 27, 26, 5, 1, 95, 85, 60, 65];
            let mut nodes: Vec<Node> = order
                .iter()
                .map(|&key| Node { key, entry: core::mem::zeroed() })
                .collect();
            for n in nodes.iter_mut() {
                let key = n.key;
                assert_eq!(wine_rb_put(&mut tree, (&key as *const i32).cast(), &mut n.entry), 0);
                assert_eq!(rb_is_red(tree.root), 0);
                check(tree.root, core::ptr::null_mut());
            }
            let dup = 30;
            let mut extra = Node { key: 30, entry: core::mem::zeroed() };
            assert_eq!(rb_put(&mut tree, (&dup as *const i32).cast(), &mut extra.entry), -1);

            let mut sorted = order.to_vec();
            sorted.sort();
            assert_eq!(keys(&mut tree), sorted);
            let k = 27;
            let hit = wine_rb_get(&tree, (&k as *const i32).cast());
            assert_eq!((*crate::RB_ENTRY_VALUE!(hit, Node, entry)).key, 27);

            for &key in &[20, 50, 1, 95, 65, 26] {
                wine_rb_remove_key(&mut tree, (&key as *const i32).cast());
                check(tree.root, core::ptr::null_mut());
                sorted.retain(|&k| k != key);
                assert_eq!(keys(&mut tree), sorted);
            }

            unsafe extern "C" fn count(_: *mut rb_entry, ctx: *mut core::ffi::c_void) {
                *(ctx as *mut usize) += 1;
            }
            let mut n = 0usize;
            wine_rb_destroy(&mut tree, Some(count), (&mut n as *mut usize).cast());
            assert_eq!(n, sorted.len());
            assert!(tree.root.is_null());
        }
    }
}
