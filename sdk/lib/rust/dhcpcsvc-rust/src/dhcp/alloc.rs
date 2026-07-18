use crate::prelude::*;
use core::ffi::c_void;


// ---- new_string_list ----
unsafe fn new_string_list(size: usize) -> *mut string_list {
    let rval = calloc(1, core::mem::size_of::<string_list>() + size) as *mut string_list;
    if !rval.is_null() {
        (*rval).string = ((rval as *mut u8).add(core::mem::size_of::<string_list>()) as *mut u8) as *mut i8;
    }
    rval
}

// ---- new_hash_table ----
pub unsafe fn new_hash_table(count: i32) -> *mut hash_table {
    let rval = calloc(
        1,
        (core::mem::size_of::<hash_table>()
            - DEFAULT_HASH_SIZE as usize * core::mem::size_of::<*mut hash_bucket>())
            + count as usize * core::mem::size_of::<*mut hash_bucket>(),
    ) as *mut hash_table;
    if rval.is_null() {
        return core::ptr::null_mut();
    }
    (*rval).hash_count = count;
    rval
}

// ---- new_hash_bucket ----
pub unsafe fn new_hash_bucket() -> *mut hash_bucket {
    let rval = calloc(1, core::mem::size_of::<hash_bucket>()) as *mut hash_bucket;
    rval
}

// ---- dfree ----
#[no_mangle]
pub unsafe extern "C" fn dfree(ptr: *mut core::ffi::c_void, name: *const i8) {
    if ptr.is_null() {
        warning(
            b"dfree %s: free on null pointer.\0".as_ptr() as *const i8,
            name,
        );
        return;
    }
    free(ptr);
}

// ---- free_hash_bucket ----
pub unsafe fn free_hash_bucket(ptr: *mut hash_bucket, name: *mut i8) {
    dfree(ptr as *mut core::ffi::c_void, name as *const i8);
}
