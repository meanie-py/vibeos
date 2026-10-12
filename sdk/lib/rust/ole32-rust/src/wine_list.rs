//! wine/list.h — the intrusive circular doubly-linked list the wine sources use
//! everywhere (`struct list` embedded in the element, `LIST_ENTRY_OF` to recover
//! the container). The C had these as `inline` functions in a header, so rustconv
//! emitted them as `extern` declarations pointing at symbols that don't exist. The struct
//! and functions are winelist-rust's; ole32 keeps its own iteration macros, which bind the
//! cursor per iteration rather than assigning the caller's variable.
//!
//! The list is CIRCULAR and the head is a `struct list` too: an empty list has
//! `head.next == head.prev == &head`. `list_head()`/`list_tail()` return NULL for
//! an empty list, which is what the callers test.
#![allow(clippy::all)]

// The struct and its inline helpers: the shared port.
pub use winelist_rust::{
    list, list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty,
    list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail,
};

/// `LIST_ENTRY(ptr, type, field)` — recover the containing struct from an embedded
/// list node. The C macro is `((type *)((char *)(ptr) - offsetof(type, field)))`;
/// in Rust always call this with `core::mem::offset_of!(T, field)` so the offset
/// can't drift (a hand-written number here is the classic magic-offset bug).
#[inline]
pub unsafe fn list_entry<T>(ptr: *const list, offset: usize) -> *mut T {
    (ptr as *const u8).sub(offset) as *mut T
}


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
