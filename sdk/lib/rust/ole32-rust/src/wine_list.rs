//! wine/list.h — the intrusive circular doubly-linked list the wine sources use
//! everywhere (`struct list` embedded in the element, `LIST_ENTRY_OF` to recover
//! the container). The C had these as `inline` functions in a header, so rustconv
//! emitted them as `extern` declarations pointing at symbols that don't exist.
//! Translated faithfully from sdk/include/wine/list.h.
//!
//! The list is CIRCULAR and the head is a `struct list` too: an empty list has
//! `head.next == head.prev == &head`. `list_head()`/`list_tail()` return NULL for
//! an empty list, which is what the callers test.
#![allow(clippy::all)]

/// wine/list.h `struct list`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct list {
    pub next: *mut list,
    pub prev: *mut list,
}

impl list {
    pub const fn new() -> Self {
        Self { next: core::ptr::null_mut(), prev: core::ptr::null_mut() }
    }
}
unsafe impl Sync for list {}

/// `list_init(list)` — a list points at itself when empty.
#[inline]
pub unsafe fn list_init(l: *mut list) {
    (*l).next = l;
    (*l).prev = l;
}

/// `list_add_after(elem, to_add)`
#[inline]
pub unsafe fn list_add_after(elem: *mut list, to_add: *mut list) {
    (*to_add).next = (*elem).next;
    (*to_add).prev = elem;
    (*(*elem).next).prev = to_add;
    (*elem).next = to_add;
}

/// `list_add_before(elem, to_add)`
#[inline]
pub unsafe fn list_add_before(elem: *mut list, to_add: *mut list) {
    (*to_add).next = elem;
    (*to_add).prev = (*elem).prev;
    (*(*elem).prev).next = to_add;
    (*elem).prev = to_add;
}

/// `list_add_head(list, elem)`
#[inline]
pub unsafe fn list_add_head(l: *mut list, elem: *mut list) {
    list_add_after(l, elem);
}

/// `list_add_tail(list, elem)` — the head's `prev` is the tail (circular).
#[inline]
pub unsafe fn list_add_tail(l: *mut list, elem: *mut list) {
    list_add_before(l, elem);
}

/// `list_remove(elem)`
#[inline]
pub unsafe fn list_remove(elem: *mut list) {
    (*(*elem).next).prev = (*elem).prev;
    (*(*elem).prev).next = (*elem).next;
}

/// `list_next(list, elem)` — NULL when we've wrapped back to the head.
#[inline]
pub unsafe fn list_next(l: *const list, elem: *const list) -> *mut list {
    let ret = (*elem).next;
    if (*elem).next == l as *mut list { core::ptr::null_mut() } else { ret }
}

/// `list_prev(list, elem)`
#[inline]
pub unsafe fn list_prev(l: *const list, elem: *const list) -> *mut list {
    let ret = (*elem).prev;
    if (*elem).prev == l as *mut list { core::ptr::null_mut() } else { ret }
}

/// `list_head(list)` — first element, or NULL if empty.
#[inline]
pub unsafe fn list_head(l: *const list) -> *mut list {
    list_next(l, l)
}

/// `list_tail(list)` — last element, or NULL if empty.
#[inline]
pub unsafe fn list_tail(l: *const list) -> *mut list {
    list_prev(l, l)
}

/// `list_empty(list)`
#[inline]
pub unsafe fn list_empty(l: *const list) -> c_int {
    ((*l).next == l as *mut list) as c_int
}

/// `list_count(list)`
#[inline]
pub unsafe fn list_count(l: *const list) -> c_uint {
    let mut count: c_uint = 0;
    let mut ptr = (*l).next;
    while ptr != l as *mut list {
        count += 1;
        ptr = (*ptr).next;
    }
    count
}

/// `list_move_tail(dst, src)` — move all of src to the tail of dst; src ends empty.
#[inline]
pub unsafe fn list_move_tail(dst: *mut list, src: *mut list) {
    if list_empty(src) != 0 {
        return;
    }
    (*(*dst).prev).next = (*src).next;
    (*(*src).next).prev = (*dst).prev;
    (*dst).prev = (*src).prev;
    (*(*src).prev).next = dst;
    list_init(src);
}

/// `LIST_ENTRY(ptr, type, field)` — recover the containing struct from an embedded
/// list node. The C macro is `((type *)((char *)(ptr) - offsetof(type, field)))`;
/// in Rust always call this with `core::mem::offset_of!(T, field)` so the offset
/// can't drift (a hand-written number here is the classic magic-offset bug).
#[inline]
pub unsafe fn list_entry<T>(ptr: *const list, offset: usize) -> *mut T {
    (ptr as *const u8).sub(offset) as *mut T
}

use core::ffi::{c_int, c_uint};
