use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
static copyright: &[u8] = b"$Id: tree.c,v 1.10 1997/05/09 08:14:57 mellon Exp $ Copyright (c) 1995, 1996, 1997 The Internet Software Consortium.  All rights reserved.\n\0";

// ---- cons ----
pub unsafe extern "C" fn cons(car: caddr_t, cdr: pair) -> pair {
    let foo = dmalloc(core::mem::size_of::<_pair>(), b"cons\0".as_ptr() as *const i8) as pair;
    if foo.is_null() {
        error(b"no memory for cons.\0".as_ptr() as *const i8);
    }
    (*foo).car = car;
    (*foo).cdr = cdr;
    foo
}

// ---- do_data_copy ----
unsafe fn do_data_copy(bufix: *mut i32, bufp: *mut *mut u8, bufcount: *mut i32, data: *mut u8, len: i32) {
    let mut space = *bufcount - *bufix;
    if space > len {
        space = len;
    }
    if space > 0 {
        core::ptr::copy_nonoverlapping(data as *const u8, (*bufp).offset(*bufix as isize), space as usize);
    }
    *bufix += len;
}

// ---- tree_evaluate_recurse ----
unsafe fn tree_evaluate_recurse(
    bufix: *mut i32,
    bufp: *mut *mut u8,
    bufcount: *mut i32,
    tree: *mut _tree,
) -> TIME {
    let limit: i32;
    let t1: TIME;
    let t2: TIME;

    match (*tree).op {
        TREE_CONCAT => {
            t1 = tree_evaluate_recurse(bufix, bufp, bufcount, (*tree).__bindgen_anon_1.concat.left);
            t2 = tree_evaluate_recurse(bufix, bufp, bufcount, (*tree).__bindgen_anon_1.concat.right);
            if t1 > t2 {
                return t2;
            }
            return t1;
        }
        TREE_HOST_LOOKUP => {
            return do_host_lookup(bufix, bufp, bufcount, (*tree).__bindgen_anon_1.host_lookup.host);
        }
        TREE_CONST => {
            do_data_copy(
                bufix,
                bufp,
                bufcount,
                (*tree).__bindgen_anon_1.const_val.data,
                (*tree).__bindgen_anon_1.const_val.len,
            );
            t1 = MAX_TIME;
            return t1;
        }
        TREE_LIMIT => {
            limit = *bufix + (*tree).__bindgen_anon_1.limit.limit;
            t1 = tree_evaluate_recurse(bufix, bufp, bufcount, (*tree).__bindgen_anon_1.limit.tree);
            *bufix = limit;
            return t1;
        }
        _ => {
            warn("Bad node id in tree: %d.");
            t1 = MAX_TIME;
            return t1;
        }
    }
}

// ---- tree_evaluate ----
unsafe fn tree_evaluate(tree_cache: *mut _tree_cache) -> i32 {
    let mut bp: *mut u8 = (*tree_cache).value;
    let mut bc: i32 = (*tree_cache).buf_size;
    let mut bufix: i32 = 0;

    // If there's no tree associated with this cache, it evaluates
    // to a constant and that was detected at startup.
    if (*tree_cache).tree.is_null() {
        return 1;
    }

    // Try to evaluate the tree without allocating more memory...
    (*tree_cache).timeout = tree_evaluate_recurse(&mut bufix, &mut bp, &mut bc, (*tree_cache).tree);

    // No additional allocation needed?
    if bufix <= bc {
        (*tree_cache).len = bufix;
        return 1;
    }

    // If we can't allocate more memory, return with what we have (maybe nothing).
    bp = dmalloc(bufix as usize, b"tree_evaluate\0".as_ptr() as *const i8) as *mut u8;
    if bp.is_null() {
        return 0;
    }

    // Record the change in conditions...
    bc = bufix;
    bufix = 0;

    // Note that the size of the result shouldn't change on the
    // second call to tree_evaluate_recurse, since we haven't
    // changed the ``current'' time.
    tree_evaluate_recurse(&mut bufix, &mut bp, &mut bc, (*tree_cache).tree);

    // Free the old buffer if needed, then store the new buffer
    // location and size and return.
    if !(*tree_cache).value.is_null() {
        dfree((*tree_cache).value, b"tree_evaluate\0".as_ptr() as *const i8);
    }
    (*tree_cache).value = bp;
    (*tree_cache).len = bufix;
    (*tree_cache).buf_size = bc;
    1
}
