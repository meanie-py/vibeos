//! `setenv.cpp`: add, replace or remove one `name=value` string in a table,
//! and mirror the change into the OS environment.

use core::ffi::c_int;
use core::ptr::null_mut;

use super::init::{_environ_table, _wenviron_table};
use super::{abort, calloc_t, errcheck, free, set_errno, EnvChar, HeapPtr, _recalloc, EILSEQ, EINVAL};

/// `copy_environment`: a deep copy. Allocation failure is `abort()`, not an
/// error return.
unsafe fn copy_environment<C: EnvChar>(old_environment: *mut *mut C) -> *mut *mut C {
    if old_environment.is_null() {
        return null_mut();
    }

    let mut entry_count = 0usize;
    let mut it = old_environment;
    while !(*it).is_null() {
        entry_count += 1;
        it = it.add(1);
    }

    let mut new_environment: HeapPtr<*mut C> = HeapPtr(calloc_t::<*mut C>(entry_count + 1));
    if new_environment.is_null() {
        abort();
    }

    let mut old_it = old_environment;
    let mut new_it = new_environment.get();
    while !(*old_it).is_null() {
        let required_count = C::tcslen(*old_it) + 1;
        *new_it = calloc_t::<C>(required_count);
        if (*new_it).is_null() {
            abort();
        }
        errcheck(C::tcscpy_s(*new_it, required_count, *old_it));
        old_it = old_it.add(1);
        new_it = new_it.add(1);
    }

    new_environment.detach()
}

/// `ensure_current_environment_is_not_initial_environment_nolock`.
unsafe fn ensure_current_environment_is_not_initial_environment_nolock<C: EnvChar>() {
    if *C::table() == *C::initial() {
        *C::table() = copy_environment(*C::table());
    }
}

/// `find_in_environment_nolock`: the index of the entry whose first `length`
/// characters collate equal to `name` AND whose next character is `=` or NUL;
/// otherwise MINUS the entry count (so 0 is ambiguous: found at 0, or empty).
unsafe fn find_in_environment_nolock<C: EnvChar>(name: *const C, length: usize) -> isize {
    let environment = *C::table();

    let mut it = environment;
    while !(*it).is_null() {
        if C::tcsnicoll(name, *it, length) != 0 {
            it = it.add(1);
            continue;
        }
        let next = *(*it).add(length);
        if !next.is(b'=') && !next.is(0) {
            it = it.add(1);
            continue;
        }
        return it.offset_from(environment);
    }

    -it.offset_from(environment)
}

/// `common_set_variable_in_environment_nolock`. Takes ownership of `option`
/// (frees it on every path that does not store it). 0 on success, -1 on
/// failure.
unsafe fn common_set_variable_in_environment_nolock<C: EnvChar>(option: *mut C, is_top_level_call: c_int) -> c_int {
    // _VALIDATE_RETURN_NOEXC: errno only, no handler.
    if option.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    let mut owned_option: HeapPtr<C> = HeapPtr(option);

    let equal_sign = C::tcschr(option, b'=');
    if equal_sign.is_null() || equal_sign == option {
        set_errno(EINVAL);
        return -1;
    }

    // "NAME=" (nothing after the '=') removes.
    let is_removal = (*equal_sign.add(1)).is(0);

    // Never modify the environment that was passed to main.
    ensure_current_environment_is_not_initial_environment_nolock::<C>();

    if (*C::table()).is_null() {
        if is_top_level_call != 0 && !(*<C::Other as EnvChar>::table()).is_null() {
            if C::get_or_create_environment_nolock().is_null() {
                set_errno(EINVAL);
                return -1;
            }
            // get_or_create may have made the current table the initial one.
            ensure_current_environment_is_not_initial_environment_nolock::<C>();
        } else {
            // Nothing to remove from a table that does not exist.
            if is_removal {
                return 0;
            }

            // Create BOTH tables, empty, whichever is missing.
            if (*core::ptr::addr_of_mut!(_environ_table)).is_null() {
                *core::ptr::addr_of_mut!(_environ_table) = calloc_t::<*mut u8>(1);
            }
            if (*core::ptr::addr_of_mut!(_environ_table)).is_null() {
                return -1;
            }
            if (*core::ptr::addr_of_mut!(_wenviron_table)).is_null() {
                *core::ptr::addr_of_mut!(_wenviron_table) = calloc_t::<*mut u16>(1);
            }
            if (*core::ptr::addr_of_mut!(_wenviron_table)).is_null() {
                return -1;
            }
        }
    }

    let environment = *C::table();
    if environment.is_null() {
        // _ASSERTE(("CRT logic error in setenv", 0)) -- a no-op in release.
        return -1;
    }

    let option_index = find_in_environment_nolock(option, equal_sign.offset_from(option) as usize);

    if option_index >= 0 && !(*environment).is_null() {
        free((*environment.add(option_index as usize)).cast());

        if is_removal {
            // Shift everything after it down one slot.
            let mut i = option_index as usize;
            while !(*environment.add(i)).is_null() {
                *environment.add(i) = *environment.add(i + 1);
                i += 1;
            }

            // Shrink to `i` slots (the count remaining). A failure is harmless
            // and keeps the old block.
            let new_environment: *mut *mut C = _recalloc(environment.cast(), i, core::mem::size_of::<*mut C>()).cast();
            if !new_environment.is_null() {
                *C::table() = new_environment;
            }
        } else {
            *environment.add(option_index as usize) = owned_option.detach();
        }
    } else {
        if is_removal {
            // Removing a variable that does not exist: a no-op, and the OS is
            // not told either.
            return 0;
        }

        let environment_count = (-option_index) as usize;
        if environment_count.wrapping_add(2) < environment_count {
            return -1;
        }
        if environment_count + 2 >= usize::MAX / core::mem::size_of::<*mut C>() {
            return -1;
        }

        let new_environment: *mut *mut C =
            _recalloc(environment.cast(), environment_count + 2, core::mem::size_of::<*mut C>()).cast();
        if new_environment.is_null() {
            return -1;
        }

        *new_environment.add(environment_count) = owned_option.detach();
        *new_environment.add(environment_count + 1) = null_mut();

        *C::table() = new_environment;
    }

    // Mirror into the OS. Only the top-level call does it: the OS keeps its
    // own narrow and wide views in step. Allocation failure here is success.
    if is_top_level_call != 0 {
        let count = C::tcslen(option) + 2;
        let buffer: HeapPtr<C> = HeapPtr(calloc_t::<C>(count));
        if buffer.is_null() {
            return 0;
        }

        let name = buffer.get();
        errcheck(C::tcscpy_s(name, count, option));

        let value = name.offset(equal_sign.offset_from(option) + 1);
        *value.sub(1) = C::ch(0); // Overwrite the '=' with a terminator.

        let value_arg = if is_removal { core::ptr::null() } else { value as *const C };
        if C::set_environment_variable(name, value_arg) == 0 {
            set_errno(EILSEQ);
            return -1;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_set_variable_in_narrow_environment_nolock(
    option: *mut u8,
    is_top_level_call: c_int,
) -> c_int {
    common_set_variable_in_environment_nolock(option, is_top_level_call)
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_set_variable_in_wide_environment_nolock(
    option: *mut u16,
    is_top_level_call: c_int,
) -> c_int {
    common_set_variable_in_environment_nolock(option, is_top_level_call)
}
