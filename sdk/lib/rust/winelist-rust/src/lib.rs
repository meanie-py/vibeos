//! `sdk/include/wine/list.h`, ported once: the intrusive doubly-linked list, its `list_*`
//! inline helpers and `LIST_ENTRY` (container-of).
//!
//! bindgen cannot emit `static inline` functions, and the port's `headers` stage leaves a
//! platform header to a shared crate, so this header had been re-ported per crate. This
//! crate is that port, taken from rtl-threadpool's; every wine/list.h user takes the struct
//! and functions from here (ole32 keeps its own iteration macros, which bind the cursor
//! per iteration instead of assigning the caller's variable).
//!
//! A consumer island blocklists its own `struct list` and re-exports this one, so a
//! `struct list` field in the consumer's structs and these functions agree on one type.
#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

#[repr(C)]
#[derive(Clone, Copy)]
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
pub unsafe fn list_empty(list: *const list) -> i32 {
    ((*list).next == list as *mut list) as i32
}

/// initialize a list
#[inline]
pub unsafe fn list_init(list: *mut list) {
    (*list).next = list;
    (*list).prev = list;
}

/// count the elements of a list
#[inline]
pub unsafe fn list_count(list: *const list) -> u32 {
    let mut count: u32 = 0;
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
/// the list element. `let obj: *mut T = LIST_ENTRY!(elem, T, field);`
#[macro_export]
macro_rules! LIST_ENTRY {
    ($elem:expr, $ty:ty, $($field:tt)+) => {
        (($elem as *mut u8).wrapping_sub(
            ::core::mem::offset_of!($ty, $($field)+),
        ) as *mut $ty)
    };
}


/// `&elem->field` for a cursor that may be the HEAD's container: C's loops compare that
/// address against the head, and the head's "container" is not an object, so the address is
/// formed with wrapping arithmetic rather than a field projection.
#[doc(hidden)]
#[macro_export]
macro_rules! __list_field {
    ($elem:expr, $ty:ty, $field:ident) => {
        (($elem as *mut u8).wrapping_add(::core::mem::offset_of!($ty, $field)) as *mut $crate::list)
    };
}

/// wine/list.h `LIST_FOR_EACH_ENTRY(elem, list, type, field)`: `elem` walks every `type`
/// whose embedded `field` is on `list`. As in C, `elem` is the CALLER's variable, the loop
/// steps from `elem` itself (`elem->field.next`), and the body must not unlink the current
/// node -- that is what the _SAFE form is for.
///
/// After a loop that runs to the end, `elem` is the HEAD's container, exactly as in C, and
/// callers rely on it: kernel32's add_resource_dir_entry walks to the first entry that sorts
/// after the new one, then `list_add_before(&ent->entry, ..)` -- off the end, that is "before
/// the head", i.e. the tail. A cursor left on the last node inserted one place too early.
///
/// The C `for` increment runs on `continue` too, so it leads the loop instead of trailing
/// the body (ole32-rust's port found that the hard way).
#[macro_export]
macro_rules! LIST_FOR_EACH_ENTRY {
    ($elem:ident, $list:expr, $ty:ty, $field:ident, $body:block) => {{
        let __head: *mut $crate::list = $list;
        $elem = $crate::LIST_ENTRY!((*__head).next, $ty, $field);
        let mut __advance = false;
        loop {
            if __advance {
                $elem = $crate::LIST_ENTRY!(
                    (*$crate::__list_field!($elem, $ty, $field)).next,
                    $ty,
                    $field
                );
            }
            __advance = true;
            if $crate::__list_field!($elem, $ty, $field) == __head {
                break;
            }
            $body
        }
    }};
}

/// wine/list.h `LIST_FOR_EACH_ENTRY_SAFE(cursor, cursor2, list, type, field)`: the next link
/// is read BEFORE the body, so the body may unlink or free `cursor`. Both are the caller's
/// variables and the step is C's `cursor = cursor2` -- so after a full walk `cursor` is the
/// head's container, as in LIST_FOR_EACH_ENTRY.
#[macro_export]
macro_rules! LIST_FOR_EACH_ENTRY_SAFE {
    ($elem:ident, $cursor2:ident, $list:expr, $ty:ty, $field:ident, $body:block) => {{
        let __head: *mut $crate::list = $list;
        $elem = $crate::LIST_ENTRY!((*__head).next, $ty, $field);
        $cursor2 = $crate::LIST_ENTRY!(
            (*$crate::__list_field!($elem, $ty, $field)).next,
            $ty,
            $field
        );
        let mut __advance = false;
        loop {
            // Leads the loop, so a `continue` cannot re-run the same node.
            if __advance {
                $elem = $cursor2;
                $cursor2 = $crate::LIST_ENTRY!(
                    (*$crate::__list_field!($elem, $ty, $field)).next,
                    $ty,
                    $field
                );
            }
            __advance = true;
            if $crate::__list_field!($elem, $ty, $field) == __head {
                break;
            }
            $body
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct Item {
        value: u32,
        entry: list,
    }

    #[test]
    fn add_remove_iterate_and_container_of() {
        unsafe {
            let mut head = list { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };
            let h: *mut list = &mut head;
            list_init(h);
            assert_eq!(list_empty(h), 1);
            let mut a = Item { value: 1, entry: list { next: h, prev: h } };
            let mut b = Item { value: 2, entry: list { next: h, prev: h } };
            list_add_tail(h, &mut a.entry);
            list_add_head(h, &mut b.entry);
            assert_eq!(list_count(h), 2);
            // head order: b then a
            let first = list_head(h);
            assert_eq!((*LIST_ENTRY!(first, Item, entry)).value, 2);
            let second = list_next(h, first);
            assert_eq!((*LIST_ENTRY!(second, Item, entry)).value, 1);
            assert!(list_next(h, second).is_null());
            list_remove(&mut b.entry);
            assert_eq!(list_count(h), 1);
            assert_eq!(list_head(h), &mut a.entry as *mut list);
        }
    }

    #[test]
    fn for_each_runs_its_step_on_continue_and_safe_survives_removal() {
        unsafe {
            let mut head = list { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };
            let h: *mut list = &mut head;
            list_init(h);
            let mut items = [1u32, 2, 3, 4].map(|value| Item { value, entry: list { next: h, prev: h } });
            for it in items.iter_mut() {
                list_add_tail(h, &mut it.entry);
            }
            let mut seen = [0u32; 4];
            let mut n = 0;
            let mut cur: *mut Item = core::ptr::null_mut();
            LIST_FOR_EACH_ENTRY!(cur, h, Item, entry, {
                if (*cur).value == 2 {
                    continue; // must still advance, not spin on 2
                }
                seen[n] = (*cur).value;
                n += 1;
            });
            assert_eq!(&seen[..n], &[1, 3, 4]);
            // As in C, a walk that runs off the end leaves the cursor on the HEAD's container.
            assert_eq!(__list_field!(cur, Item, entry), h);
            let mut next: *mut Item = core::ptr::null_mut();
            LIST_FOR_EACH_ENTRY_SAFE!(cur, next, h, Item, entry, {
                if (*cur).value % 2 == 0 {
                    list_remove(&mut (*cur).entry);
                }
            });
            let _ = next;
            assert_eq!(list_count(h), 2);
            assert_eq!(__list_field!(cur, Item, entry), h);
        }
    }

    #[test]
    fn insert_before_the_cursor_after_a_full_walk_appends() {
        // kernel32's add_resource_dir_entry: walk to the first entry that sorts after the
        // new one; when there is none, `list_add_before(&ent->entry, ..)` is the tail.
        unsafe {
            let mut head = list { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };
            let h: *mut list = &mut head;
            list_init(h);
            let mut items = [1u32, 2, 3].map(|value| Item { value, entry: list { next: h, prev: h } });
            let mut new = Item { value: 9, entry: list { next: h, prev: h } };
            for it in items.iter_mut() {
                list_add_tail(h, &mut it.entry);
            }
            let mut ent: *mut Item = core::ptr::null_mut();
            LIST_FOR_EACH_ENTRY!(ent, h, Item, entry, {
                if (*ent).value < new.value {
                    continue;
                }
                break;
            });
            list_add_before(__list_field!(ent, Item, entry), &mut new.entry);
            assert_eq!((*LIST_ENTRY!(list_tail(h), Item, entry)).value, 9);
            // And an empty list: the cursor is the head's container straight away.
            let mut empty = list { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };
            let e: *mut list = &mut empty;
            list_init(e);
            let mut ran = false;
            LIST_FOR_EACH_ENTRY!(ent, e, Item, entry, {
                ran = true;
            });
            assert!(!ran);
            assert_eq!(__list_field!(ent, Item, entry), e);
        }
    }
}
