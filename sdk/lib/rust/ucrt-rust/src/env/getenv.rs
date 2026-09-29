//! `getenv.cpp`: `getenv`/`_wgetenv`, `getenv_s`/`_wgetenv_s` and
//! `_dupenv_s`/`_wdupenv_s`, all under the environment lock.

use core::ffi::c_int;
use core::ptr::null_mut;

use super::{
    calloc, errcheck, invalid, set_errno, with_environment_lock, EnvChar, EINVAL, ENOMEM, ERANGE, MAX_ENV,
};

/// `common_getenv_nolock`: a pointer INTO the table entry, just past `=`.
/// Asking creates this width's table from the other if only that one exists.
pub(crate) unsafe fn common_getenv_nolock<C: EnvChar>(name: *const C) -> *mut C {
    let environment = C::get_or_create_environment_nolock();
    if environment.is_null() || name.is_null() {
        return null_mut();
    }

    let name_length = C::tcslen(name);

    let mut current = environment;
    while !(*current).is_null() {
        let entry = *current;
        current = current.add(1);

        if C::tcslen(entry) <= name_length {
            continue;
        }
        if !(*entry.add(name_length)).is(b'=') {
            continue;
        }
        if C::tcsnicoll(entry, name, name_length) != 0 {
            continue;
        }
        return entry.add(name_length + 1);
    }

    null_mut()
}

unsafe fn common_getenv<C: EnvChar>(name: *const C) -> *mut C {
    if name.is_null() {
        invalid(EINVAL);
        return null_mut();
    }
    if C::tcsnlen(name, MAX_ENV) >= MAX_ENV {
        invalid(EINVAL);
        return null_mut();
    }

    with_environment_lock(|| common_getenv_nolock(name))
}

#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const u8) -> *mut u8 {
    common_getenv(name)
}

#[no_mangle]
pub unsafe extern "C" fn _wgetenv(name: *const u16) -> *mut u16 {
    common_getenv(name)
}

/// `common_getenv_s_nolock`. Note the validation runs INSIDE the lock (the
/// handler is invoked with the environment lock held), as in the C.
unsafe fn common_getenv_s_nolock<C: EnvChar>(
    required_count: *mut usize,
    buffer: *mut C,
    buffer_count: usize,
    name: *const C,
) -> c_int {
    if required_count.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    *required_count = 0;

    if !((!buffer.is_null() && buffer_count > 0) || (buffer.is_null() && buffer_count == 0)) {
        invalid(EINVAL);
        return EINVAL;
    }

    if !buffer.is_null() {
        *buffer = C::ch(0);
    }

    let value = common_getenv_nolock(name);
    if value.is_null() {
        return 0;
    }

    *required_count = C::tcslen(value) + 1;
    if buffer_count == 0 {
        return 0;
    }

    // Too small: the caller may retry with *required_count.
    if *required_count > buffer_count {
        return ERANGE;
    }

    errcheck(C::tcscpy_s(buffer, buffer_count, value));
    0
}

unsafe fn common_getenv_s<C: EnvChar>(
    required_count: *mut usize,
    buffer: *mut C,
    buffer_count: usize,
    name: *const C,
) -> c_int {
    with_environment_lock(|| common_getenv_s_nolock(required_count, buffer, buffer_count, name))
}

#[no_mangle]
pub unsafe extern "C" fn getenv_s(
    required_count: *mut usize,
    buffer: *mut u8,
    buffer_count: usize,
    name: *const u8,
) -> c_int {
    common_getenv_s(required_count, buffer, buffer_count, name)
}

#[no_mangle]
pub unsafe extern "C" fn _wgetenv_s(
    required_count: *mut usize,
    buffer: *mut u16,
    buffer_count: usize,
    name: *const u16,
) -> c_int {
    common_getenv_s(required_count, buffer, buffer_count, name)
}

/// `common_dupenv_s_nolock`. The copy comes from the PUBLIC `calloc` (in a
/// release build `_calloc_dbg` is `calloc`), so the caller frees it with
/// `free`. A missing variable is success with `*buffer_pointer == NULL`.
unsafe fn common_dupenv_s_nolock<C: EnvChar>(
    buffer_pointer: *mut *mut C,
    buffer_count: *mut usize,
    name: *const C,
) -> c_int {
    if buffer_pointer.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    *buffer_pointer = null_mut();

    if !buffer_count.is_null() {
        *buffer_count = 0;
    }

    if name.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }

    let value = common_getenv_nolock(name);
    if value.is_null() {
        return 0;
    }

    let value_count = C::tcslen(value) + 1;

    *buffer_pointer = calloc(value_count, core::mem::size_of::<C>()).cast();
    // _VALIDATE_RETURN_ERRCODE_NOEXC: errno only.
    if (*buffer_pointer).is_null() {
        set_errno(ENOMEM);
        return ENOMEM;
    }

    errcheck(C::tcscpy_s(*buffer_pointer, value_count, value));
    if !buffer_count.is_null() {
        *buffer_count = value_count;
    }

    0
}

unsafe fn common_dupenv_s<C: EnvChar>(buffer_pointer: *mut *mut C, buffer_count: *mut usize, name: *const C) -> c_int {
    with_environment_lock(|| common_dupenv_s_nolock(buffer_pointer, buffer_count, name))
}

#[no_mangle]
pub unsafe extern "C" fn _dupenv_s(buffer_pointer: *mut *mut u8, buffer_count: *mut usize, name: *const u8) -> c_int {
    common_dupenv_s(buffer_pointer, buffer_count, name)
}

#[no_mangle]
pub unsafe extern "C" fn _wdupenv_s(
    buffer_pointer: *mut *mut u16,
    buffer_count: *mut usize,
    name: *const u16,
) -> c_int {
    common_dupenv_s(buffer_pointer, buffer_count, name)
}
