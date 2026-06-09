//! wine/list.h intrusive doubly-linked-list helpers.
//!
//! These are `static inline` in wine/list.h, so the DeepSeek transliteration of
//! fci.c left them as undefined `extern` symbols (declared in fci.rs but never
//! defined). Provide them here as the linked symbols fci.rs references. The ABI
//! per helper matches the corresponding fci.rs extern declaration: the four
//! mutators are `extern "C"` (cdecl), `list_empty` is `extern "system"`.

#[repr(C)]
pub struct list {
    next: *mut list,
    prev: *mut list,
}

#[no_mangle]
pub unsafe extern "C" fn list_init(list: *mut list) {
    (*list).next = list;
    (*list).prev = list;
}

#[no_mangle]
pub unsafe extern "C" fn list_add_tail(list: *mut list, elem: *mut list) {
    (*elem).prev = (*list).prev;
    (*elem).next = list;
    (*(*list).prev).next = elem;
    (*list).prev = elem;
}

#[no_mangle]
pub unsafe extern "C" fn list_add_before(elem: *mut list, to_add: *mut list) {
    (*to_add).next = elem;
    (*to_add).prev = (*elem).prev;
    (*(*elem).prev).next = to_add;
    (*elem).prev = to_add;
}

#[no_mangle]
pub unsafe extern "C" fn list_remove(elem: *mut list) {
    (*(*elem).next).prev = (*elem).prev;
    (*(*elem).prev).next = (*elem).next;
}

#[no_mangle]
pub unsafe extern "system" fn list_empty(list: *const list) -> i32 {
    ((*list).next == list as *mut list) as i32
}
