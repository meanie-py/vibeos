//! `putenv.cpp`: `_putenv`/`_wputenv` (one `name=value` option) and
//! `_putenv_s`/`_wputenv_s` (name and value apart). Each change goes into
//! this width's table (creating it from the other if needed, and mirroring
//! into the OS), then -- if the other width's table exists -- into that one
//! too, converted through `CP_ACP`.

use core::ffi::c_int;
use core::ptr::null_mut;

use super::init::{_environ_table, _wenviron_table};
use super::{
    calloc_t, get_errno, invalid, set_errno, with_environment_lock, EnvChar, HeapPtr, EILSEQ, EINVAL, MAX_ENV,
};

/// `other_environment_exists(Character())`.
unsafe fn other_environment_exists<C: EnvChar>() -> bool {
    !(*<C::Other as EnvChar>::table()).is_null()
}

/// `compute_required_transform_buffer_count`: `Other`-width units for
/// `name=value` (each count includes a terminator; one of them becomes the
/// `=`). A conversion failure sets EILSEQ and returns 0 (`false`).
unsafe fn compute_required_transform_buffer_count<C: EnvChar>(name: *const C, value: *const C) -> usize {
    let name_count_required = C::required_transform_count(name);
    if name_count_required == 0 {
        set_errno(EILSEQ);
        return 0;
    }

    if value.is_null() {
        return name_count_required;
    }

    let value_count_required = C::required_transform_count(value);
    if value_count_required == 0 {
        set_errno(EILSEQ);
        return 0;
    }

    name_count_required + value_count_required
}

/// `create_environment_string`: a fresh `name=value` (or a copy of `name`
/// when `value` is NULL), or NULL.
unsafe fn create_environment_string<C: EnvChar>(name: *const C, value: *const C) -> *mut C {
    if !value.is_null() {
        let name_length = C::tcsnlen(name, MAX_ENV);
        let value_length = C::tcsnlen(value, MAX_ENV);

        if name_length >= MAX_ENV {
            invalid(EINVAL);
            return null_mut();
        }
        if value_length >= MAX_ENV {
            invalid(EINVAL);
            return null_mut();
        }

        // One for the '=' and one for the terminator.
        let buffer_count = name_length + 1 + value_length + 1;

        let mut buffer: HeapPtr<C> = HeapPtr(calloc_t::<C>(buffer_count));
        if buffer.is_null() {
            return null_mut();
        }

        // Both copies' results are ignored in the C.
        let _ = C::tcscpy_s(buffer.get(), buffer_count, name);
        *buffer.get().add(name_length) = C::ch(b'=');
        let _ = C::tcscpy_s(buffer.get().add(name_length + 1), value_length + 1, value);

        buffer.detach()
    } else {
        let equal_sign_it = C::tcschr(name, b'=');
        if !equal_sign_it.is_null() {
            if equal_sign_it.offset_from(name) as usize >= MAX_ENV {
                invalid(EINVAL);
                return null_mut();
            }
            if C::tcsnlen(equal_sign_it.add(1), MAX_ENV) >= MAX_ENV {
                invalid(EINVAL);
                return null_mut();
            }
        }

        let buffer_count = C::tcslen(name) + 1;

        let mut buffer: HeapPtr<C> = HeapPtr(calloc_t::<C>(buffer_count));
        if buffer.is_null() {
            return null_mut();
        }

        let _ = C::tcscpy_s(buffer.get(), buffer_count, name);

        buffer.detach()
    }
}

/// `set_variable_in_other_environment`: convert and apply to the other table
/// (not a top-level call: the OS was already updated).
unsafe fn set_variable_in_other_environment<C: EnvChar>(name: *const C, value: *const C) -> bool {
    let buffer_count = compute_required_transform_buffer_count(name, value);

    let mut buffer: HeapPtr<C::Other> = HeapPtr(calloc_t::<C::Other>(buffer_count));
    if buffer.is_null() {
        return false;
    }

    let name_written_count = C::transform_string(name, buffer.get(), buffer_count);
    if name_written_count == 0 {
        set_errno(EILSEQ);
        return false;
    }

    if !value.is_null() {
        // Overwrite the terminator with an '=':
        *buffer.get().add(name_written_count - 1) = <C::Other as EnvChar>::ch(b'=');

        let value_written_count = C::transform_string(
            value,
            buffer.get().add(name_written_count),
            buffer_count.wrapping_sub(name_written_count),
        );
        if value_written_count == 0 {
            set_errno(EILSEQ);
            return false;
        }
    }

    <C::Other as EnvChar>::set_variable_in_environment_nolock(buffer.detach(), 0) == 0
}

/// `common_putenv_nolock`. 0 on success, -1 on failure.
unsafe fn common_putenv_nolock<C: EnvChar>(name: *const C, value: *const C) -> c_int {
    // Neither table: -1, errno untouched, and BEFORE the name check.
    if (*core::ptr::addr_of_mut!(_environ_table)).is_null() && (*core::ptr::addr_of_mut!(_wenviron_table)).is_null()
    {
        return -1;
    }

    if name.is_null() {
        invalid(EINVAL);
        return -1;
    }

    let mut new_option: HeapPtr<C> = HeapPtr(create_environment_string(name, value));
    if new_option.is_null() {
        return -1;
    }

    if C::set_variable_in_environment_nolock(new_option.detach(), 1) != 0 {
        return -1;
    }

    if !other_environment_exists::<C>() {
        return 0;
    }

    if !set_variable_in_other_environment(name, value) {
        return -1;
    }

    0
}

unsafe fn common_putenv<C: EnvChar>(name: *const C, value: *const C) -> c_int {
    with_environment_lock(|| common_putenv_nolock(name, value))
}

#[no_mangle]
pub unsafe extern "C" fn _putenv(option: *const u8) -> c_int {
    common_putenv(option, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _wputenv(option: *const u16) -> c_int {
    common_putenv(option, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _putenv_s(name: *const u8, value: *const u8) -> c_int {
    if value.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    if common_putenv(name, value) == 0 {
        0
    } else {
        get_errno()
    }
}

#[no_mangle]
pub unsafe extern "C" fn _wputenv_s(name: *const u16, value: *const u16) -> c_int {
    if value.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    if common_putenv(name, value) == 0 {
        0
    } else {
        get_errno()
    }
}
