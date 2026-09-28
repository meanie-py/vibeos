//! `new_handler.cpp` and `new_mode.cpp`: the `_PNH` new handler (stored
//! pointer-encoded) and the `_newmode` flag that makes `malloc` call it.

use super::{invalid_einval, __acrt_lock, __acrt_unlock, ACRT_HEAP_LOCK};
use core::ffi::{c_int, c_void};
use core::sync::atomic::{AtomicI32, Ordering};

/// `typedef int (__CRTDECL* _PNH)(size_t)` (`new.h`); NULL is `None`.
pub type Pnh = Option<unsafe extern "C" fn(usize) -> c_int>;

// `__security_cookie` (crt-startup-rust's gs.rs), the key of every
// `__crt_fast_encode_pointer`/`__crt_fast_decode_pointer`. Same declaration
// as invalid_parameter.rs.
extern "C" {
    static mut __security_cookie: usize;
}

/// `__crt_fast_encode_pointer` == `__crt_fast_decode_pointer`: XOR with the
/// cookie (`sdk/lib/vcruntime/inc/internal_shared.h`).
#[inline(always)]
unsafe fn fast_code(ptr: usize) -> usize {
    ptr ^ core::ptr::addr_of!(__security_cookie).read()
}

/// `static dual_state_global<_PNH> __acrt_new_handler` -- file-static in the
/// C (`_ZL18__acrt_new_handler`), so nothing outside this file reads it by
/// name. One state slot (`state_index_count == 1`), holding the ENCODED
/// pointer as a plain word.
static mut ACRT_NEW_HANDLER: usize = 0;

/// `static dual_state_global<long> __acrt_global_new_mode` -- file-static too.
static ACRT_GLOBAL_NEW_MODE: AtomicI32 = AtomicI32::new(0);

#[inline(always)]
unsafe fn pnh_from_word(word: usize) -> Pnh {
    // SAFETY: `Option<extern fn>` is one pointer with NULL as `None`.
    core::mem::transmute::<usize, Pnh>(word)
}

#[inline(always)]
fn pnh_to_word(pnh: Pnh) -> usize {
    pnh.map_or(0, |f| f as usize)
}

/// `void __acrt_initialize_new_handler(void* encoded_null)` -- called by
/// `internal/initialization.cpp` with `__crt_fast_encode_pointer(nullptr)`;
/// stored as given (it is already encoded).
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_new_handler(encoded_null: *mut c_void) {
    core::ptr::addr_of_mut!(ACRT_NEW_HANDLER).write(encoded_null as usize);
}

/// `_PNH _set_new_handler(_PNH)`: under `__acrt_heap_lock`, returns the old
/// (decoded) handler and stores the new one encoded. `new.h` declares it in a
/// C block, so it is the plain name, not mangled.
#[no_mangle]
pub unsafe extern "C" fn _set_new_handler(new_handler: Pnh) -> Pnh {
    __acrt_lock(ACRT_HEAP_LOCK);
    let slot = core::ptr::addr_of_mut!(ACRT_NEW_HANDLER);
    let result = fast_code(slot.read());
    slot.write(fast_code(pnh_to_word(new_handler)));
    __acrt_unlock(ACRT_HEAP_LOCK);
    pnh_from_word(result)
}

/// `_PNH _set_new_handler(int)` -- the C++ overload taking a literal 0,
/// `_Z16_set_new_handleri` (not in ucrtbase.spec, no caller in the tree;
/// defined because new_handler.cpp defines it). `i` is `int` on every target,
/// and rustc adds i386's `_` itself, giving `__Z16_set_new_handleri` there as
/// the C object has.
///
/// DIVERGENCE, deliberately: the C passes `__crt_fast_encode_pointer(nullptr)`
/// (the COOKIE) to `_set_new_handler`, which encodes it AGAIN and stores 0 --
/// which `_query_new_handler` then decodes to the cookie, a non-NULL garbage
/// "handler" that the next failed `new`/`malloc` would call. Its own comment
/// says the overload "Sets the new handler to nullptr", so we pass NULL.
#[export_name = "_Z16_set_new_handleri"]
pub unsafe extern "C" fn set_new_handler_int(_new_handler: c_int) -> Pnh {
    _set_new_handler(None)
}

/// `_PNH _query_new_handler()`: the decoded handler, read under the lock.
#[no_mangle]
pub unsafe extern "C" fn _query_new_handler() -> Pnh {
    __acrt_lock(ACRT_HEAP_LOCK);
    let result = fast_code(core::ptr::addr_of!(ACRT_NEW_HANDLER).read());
    __acrt_unlock(ACRT_HEAP_LOCK);
    pnh_from_word(result)
}

/// `int _callnewh(size_t)`: 1 if a handler is installed and returned
/// nonzero, else 0. The handler may throw `std::bad_alloc` in C++; it passes
/// through this frame exactly as through the C one (no cleanup here).
#[no_mangle]
pub unsafe extern "C" fn _callnewh(size: usize) -> c_int {
    let pnh = _query_new_handler();
    match pnh {
        None => 0,
        Some(f) => {
            if f(size) == 0 {
                0
            } else {
                1
            }
        }
    }
}

/// `int _set_new_mode(int)`: only 0 and 1 are valid (`_VALIDATE_RETURN`,
/// EINVAL, -1); returns the previous mode (`_InterlockedExchange`).
#[no_mangle]
pub unsafe extern "C" fn _set_new_mode(mode: c_int) -> c_int {
    if !(mode == 0 || mode == 1) {
        invalid_einval();
        return -1;
    }
    ACRT_GLOBAL_NEW_MODE.swap(mode, Ordering::SeqCst)
}

/// `int _query_new_mode()` (`__crt_interlocked_read`, a plain read here).
#[no_mangle]
pub unsafe extern "C" fn _query_new_mode() -> c_int {
    ACRT_GLOBAL_NEW_MODE.load(Ordering::Relaxed)
}
