use crate::prelude::*;
use core::ffi::c_void;

/// dictionary.c:95 `dictionary_find_internal` — hand-translated: rustconv could not
/// extract this body (its return type `struct dictionary_entry **` split across two
/// lines defeated the extractor), so it emitted an extern decl for a symbol that does
/// not exist.
///
/// Faithful to the C: returns the ADDRESS OF the link that points at the matching
/// entry (`&d->head` or `&p->next`), not the entry — that's what lets the callers
/// unlink in place. NULL when not found.
unsafe fn dictionary_find_internal(
    d: *mut dictionary,
    k: *const core::ffi::c_void,
) -> *mut *mut dictionary_entry {
    let mut ret: *mut *mut dictionary_entry = core::ptr::null_mut();

    // special case for head containing the desired element
    if !(*d).head.is_null() && ((*d).comp.unwrap_unchecked())(k, (*(*d).head).key, (*d).extra) == 0 {
        ret = core::ptr::addr_of_mut!((*d).head);
    }

    let mut p: *mut dictionary_entry = (*d).head;
    while ret.is_null() && !p.is_null() && !(*p).next.is_null() {
        if ((*d).comp.unwrap_unchecked())(k, (*(*p).next).key, (*d).extra) == 0 {
            ret = core::ptr::addr_of_mut!((*p).next);
        }
        p = (*p).next;
    }

    ret
}


// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct dictionary_entry {
    pub key: PVOID,
    pub value: PVOID,
    pub next: *mut dictionary_entry,
}

pub type Pdictionary_entry = *mut dictionary_entry;

#[repr(C)]
pub struct dictionary {
    pub comp: comparefunc,
    pub destroy: destroyfunc,
    pub extra: PVOID,
    pub head: *mut dictionary_entry,
    pub num_entries: UINT,
}

pub type Pdictionary = *mut dictionary;
#[no_mangle]
pub unsafe extern "C" fn dictionary_create(c: comparefunc, d: destroyfunc, extra: *mut core::ffi::c_void) -> *mut dictionary {

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), c, d, extra);

    if c.is_none() {
        return core::ptr::null_mut();
    }

    let ret = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<dictionary>() as usize) as *mut dictionary;

    if !ret.is_null() {
        (*ret).comp = c;
        (*ret).destroy = d;
        (*ret).extra = extra;
        (*ret).head = core::ptr::null_mut();
        (*ret).num_entries = 0;
    }

    TRACE!(b"returning %p\n\0".as_ptr(), ret);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_destroy(d: *mut dictionary) {
    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, d);
    if !d.is_null() {
        let mut p = (*d).head;

        while !p.is_null() {
            let next = (*p).next;

            if (*d).destroy.is_some() {
                ((*d).destroy.unwrap_unchecked())((*p).key, (*p).value, (*d).extra);
            }
            HeapFree(GetProcessHeap(), 0, p as *mut _);
            p = next;
        }
        HeapFree(GetProcessHeap(), 0, d as *mut _);
    }
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_num_entries(d: *mut dictionary) -> UINT {
    if !d.is_null() { (*d).num_entries } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_insert(d: *mut dictionary, k: *const core::ffi::c_void, v: *const core::ffi::c_void) {

    let prior: *mut *mut dictionary_entry;

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), d, k, v);
    if d.is_null() {
        return;
    }
    prior = dictionary_find_internal(d, k);
    if !prior.is_null() {
        if ((*d).destroy).is_some() {
            ((*d).destroy).unwrap_unchecked()((**prior).key, (**prior).value, (*d).extra);
        }
        (**prior).key = k as *mut core::ffi::c_void;
        (**prior).value = v as *mut core::ffi::c_void;
    } else {
        let elem = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<dictionary_entry>() as usize) as *mut dictionary_entry;

        if elem.is_null() {
            return;
        }
        (*elem).key = k as *mut core::ffi::c_void;
        (*elem).value = v as *mut core::ffi::c_void;
        (*elem).next = (*d).head;
        (*d).head = elem;
        (*d).num_entries = (*d).num_entries + 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_find(
    d: *mut dictionary,
    k: *const core::ffi::c_void,
    value: *mut *mut core::ffi::c_void,
) -> BOOL {
    let prior: *mut *mut dictionary_entry;
    let mut ret: BOOL = 0;

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as PCHAR, d, k, value);
    if d.is_null() {
        return 0;
    }
    if value.is_null() {
        return 0;
    }
    prior = dictionary_find_internal(d, k);
    if !prior.is_null() {
        *value = (**prior).value;
        ret = 1;
    }
    TRACE!(b"returning %d (%p)\n\0".as_ptr() as PCHAR, ret, *value);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_remove(d: *mut dictionary, k: *const c_void) {
    let prior: *mut *mut dictionary_entry;
    let temp: *mut dictionary_entry;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, d, k);
    if d.is_null() {
        return;
    }
    prior = dictionary_find_internal(d, k);
    if !prior.is_null() {
        temp = *prior;
        if ((*d).destroy).is_some() {
            ((*d).destroy).unwrap_unchecked()((**prior).key, (**prior).value, (*d).extra);
        }
        *prior = (**prior).next;
        HeapFree(GetProcessHeap(), 0, temp as *mut c_void);
        (*d).num_entries = (*d).num_entries.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_enumerate(
    d: *mut dictionary,
    e: enumeratefunc,
    closure: *mut core::ffi::c_void,
) {
    TRACE!(
        b"(%p, %p, %p)\n\0".as_ptr() as PCHAR,
        d,
        e,
        closure,
    );
    if d.is_null() {
        return;
    }
    if e.is_none() {
        return;
    }
    let mut p = (*d).head;
    while !p.is_null() {
        if e.unwrap_unchecked()((*p).key, (*p).value, (*d).extra, closure) == 0 {
            break;
        }
        p = (*p).next;
    }
}
