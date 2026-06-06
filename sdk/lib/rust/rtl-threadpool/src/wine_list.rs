//! Faithful Rust port of `sdk/include/wine/list.h` — the intrusive
//! doubly-linked list used throughout the Wine thread-pool code.
//!
//! Mirrors the C `struct list { struct list *next, *prev; }` and the
//! `list_*` inline helpers exactly (raw-pointer semantics, `list_empty`
//! returns `int`, getters return a raw `*mut list`). The container-of
//! pattern (`LIST_ENTRY`) is provided by the [`list_entry!`] macro.

use ntkrnl_sys::base::*;

#[repr(C)]
pub struct list {
    pub next: *mut list,
    pub prev: *mut list,
}

/// add an element after the specified one
#[inline]
pub unsafe fn list_add_after(elem: *mut list, to_add: *mut list) {
    (*to_add).next = (*elem).next;
    (*to_add).prev = elem;
    (*(*elem).next).prev = to_add;
    (*elem).next = to_add;
}

/// add an element before the specified one
#[inline]
pub unsafe fn list_add_before(elem: *mut list, to_add: *mut list) {
    (*to_add).next = elem;
    (*to_add).prev = (*elem).prev;
    (*(*elem).prev).next = to_add;
    (*elem).prev = to_add;
}

/// add element at the head of the list
#[inline]
pub unsafe fn list_add_head(list: *mut list, elem: *mut list) {
    list_add_after(list, elem);
}

/// add element at the tail of the list
#[inline]
pub unsafe fn list_add_tail(list: *mut list, elem: *mut list) {
    list_add_before(list, elem);
}

/// remove an element from its list
#[inline]
pub unsafe fn list_remove(elem: *mut list) {
    (*(*elem).next).prev = (*elem).prev;
    (*(*elem).prev).next = (*elem).next;
}

/// get the next element
#[inline]
pub unsafe fn list_next(list: *const list, elem: *const list) -> *mut list {
    let ret = (*elem).next;
    if (*elem).next == list as *mut list {
        return core::ptr::null_mut();
    }
    ret
}

/// get the previous element
#[inline]
pub unsafe fn list_prev(list: *const list, elem: *const list) -> *mut list {
    let ret = (*elem).prev;
    if (*elem).prev == list as *mut list {
        return core::ptr::null_mut();
    }
    ret
}

/// get the first element
#[inline]
pub unsafe fn list_head(list: *const list) -> *mut list {
    list_next(list, list)
}

/// get the last element
#[inline]
pub unsafe fn list_tail(list: *const list) -> *mut list {
    list_prev(list, list)
}

/// check if a list is empty
#[inline]
pub unsafe fn list_empty(list: *const list) -> INT {
    ((*list).next == list as *mut list) as INT
}

/// initialize a list
#[inline]
pub unsafe fn list_init(list: *mut list) {
    (*list).next = list;
    (*list).prev = list;
}

/// count the elements of a list
#[inline]
pub unsafe fn list_count(list: *const list) -> UINT {
    let mut count: UINT = 0;
    let mut ptr = (*list).next;
    while ptr != list as *mut list {
        count += 1;
        ptr = (*ptr).next;
    }
    count
}

/// move all elements from src to the tail of dst
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

/// move all elements from src to the head of dst
#[inline]
pub unsafe fn list_move_head(dst: *mut list, src: *mut list) {
    if list_empty(src) != 0 {
        return;
    }
    (*(*dst).next).prev = (*src).prev;
    (*(*src).prev).next = (*dst).next;
    (*dst).next = (*src).next;
    (*(*src).next).prev = dst;
    list_init(src);
}

/// `LIST_ENTRY(elem, type, field)` — pointer to the object containing
/// the list element. `let obj: *mut T = list_entry!(elem, T, field);`
#[macro_export]
macro_rules! list_entry {
    ($elem:expr, $ty:ty, $field:ident) => {
        (($elem as *mut u8).wrapping_sub(
            ::core::mem::offset_of!($ty, $field),
        ) as *mut $ty)
    };
}
