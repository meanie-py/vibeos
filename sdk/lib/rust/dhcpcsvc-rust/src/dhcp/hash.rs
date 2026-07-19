use crate::prelude::*;
use core::ffi::c_void;


// ---- new_hash ----
pub unsafe fn new_hash() -> *mut hash_table {
    let rv = new_hash_table((DEFAULT_HASH_SIZE) as i32);
    if rv.is_null() {
        return rv;
    }
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*rv).buckets[0]) as *mut u8,
        0,
        DEFAULT_HASH_SIZE as usize * core::mem::size_of::<*mut hash_bucket>(),
    );
    rv
}

// ---- do_hash ----
unsafe fn do_hash(name: *const u8, len: i32, size: i32) -> i32 {
    let mut accum: i32 = 0;
    let mut s = name;
    let mut i = len;
    while i != 0 {
        i -= 1;
        accum += *s as i32;
        s = s.offset(1);
        while accum > 255 {
            accum = (accum & 255) + (accum >> 8);
        }
    }
    accum % size
}

// ---- add_hash ----
#[no_mangle]
pub unsafe extern "C" fn add_hash(
    table: *mut hash_table,
    name: *mut u8,
    len: i32,
    pointer: *mut u8,
) {
    if table.is_null() {
        return;
    }

    let mut len = len;
    if len == 0 {
        len = strlen(name as *const i8) as i32;
    }

    let hashno = do_hash(name, len, (*table).hash_count);
    let bp: *mut hash_bucket = new_hash_bucket();

    if bp.is_null() {
        warn(
            b"Can't add %s to hash table.\0".as_ptr() as *const i8,
            name as *const i8,
        );
        return;
    }

    (*bp).name = name;
    (*bp).value = pointer;
    (*bp).next = (*table).buckets[hashno as usize];
    (*bp).len = len;
    (*table).buckets[hashno as usize] = bp;
}

// ---- delete_hash_entry ----
unsafe fn delete_hash_entry(table: *mut hash_table, name: *const u8, len: i32) {
    let mut hashno: i32;
    let mut bp: *mut hash_bucket;
    let mut pbp: *mut hash_bucket = core::ptr::null_mut();

    if table.is_null() {
        return;
    }
    let mut len = len;
    if len == 0 {
        len = strlen(name as *const i8) as i32;
    }

    hashno = do_hash(name, len, (*table).hash_count);

    bp = (*table).buckets[hashno as usize];
    while !bp.is_null() {
        if ((*bp).len == 0 && strcmp((*bp).name as *const i8, name as *const i8) == 0) ||
            ((*bp).len == len && memcmp((*bp).name as *const core::ffi::c_void, name as *const core::ffi::c_void, len as usize) == 0)
        {
            if !pbp.is_null() {
                (*pbp).next = (*bp).next;
            } else {
                (*table).buckets[hashno as usize] = (*bp).next;
            }
            free_hash_bucket(bp, "delete_hash_entry\0".as_ptr() as *mut i8);
            break;
        }
        pbp = bp;
        bp = (*bp).next;
    }
}
