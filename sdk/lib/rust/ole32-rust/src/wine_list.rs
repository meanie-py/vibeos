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

/// wine/list.h `LIST_FOR_EACH_ENTRY(elem, list, type, field)` — iterate a list,
/// yielding a `*mut type` for each embedded node.
///
/// Faithful to the C: it does NOT cache `next` before running the body, so (exactly as
/// in C) the body must not free or unlink the current element. Use the _SAFE variant
/// below when it does — that distinction is the whole reason wine ships both, and
/// silently "improving" the unsafe one into the safe one would paper over a real bug.
///
/// The container-of step goes through `offset_of!`, never a hand-written number.
#[macro_export]
macro_rules! LIST_FOR_EACH_ENTRY {
    ($elem:ident, $list:expr, $ty:ty, $field:ident, $body:block) => {{
        let head: *mut $crate::wine_list::list = $list;
        let mut cur = (*head).next;
        // The C `for` increment runs on `continue` as well, so it leads the loop instead of
        // trailing `$body`: after the body, a `continue` skipped it and re-ran the same node.
        let mut advance = false;
        loop {
            if advance {
                cur = (*cur).next;
            }
            advance = true;
            if cur == head {
                break;
            }
            #[allow(unused_variables)]
            let $elem: *mut $ty =
                $crate::wine_list::list_entry::<$ty>(cur, core::mem::offset_of!($ty, $field));
            $body
        }
    }};
}

/// wine/list.h `LIST_FOR_EACH_ENTRY_SAFE(cursor, cursor2, type, field)` — caches the
/// next link BEFORE the body, so the body may unlink/free the current element.
///
/// Two call shapes, because the C macro is `LIST_FOR_EACH_ENTRY_SAFE(cursor, cursor2,
/// type, field)`: `cursor2` is the caller-supplied scratch variable the C uses to stash
/// the next link before running the body. We cache `next` inside the macro instead, so
/// that parameter has nothing to bind to — but translations still pass it, so accept and
/// ignore it rather than making every call site drop an argument.
#[macro_export]
macro_rules! LIST_FOR_EACH_ENTRY_SAFE {
    ($elem:ident, $cursor2:ident, $list:expr, $ty:ty, $field:ident, $body:block) => {
        $crate::LIST_FOR_EACH_ENTRY_SAFE!($elem, $list, $ty, $field, $body)
    };
    ($elem:ident, $list:expr, $ty:ty, $field:ident, $body:block) => {{
        let head: *mut $crate::wine_list::list = $list;
        let mut cur = (*head).next;
        while cur != head {
            let next = (*cur).next;
            #[allow(unused_variables)]
            let $elem: *mut $ty =
                $crate::wine_list::list_entry::<$ty>(cur, core::mem::offset_of!($ty, $field));
            // Advance BEFORE the body (next is already cached), so a `continue` in it cannot
            // skip the step and re-run the same node.
            cur = next;
            $body
        }
    }};
}
