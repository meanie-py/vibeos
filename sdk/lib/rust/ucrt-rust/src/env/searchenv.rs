//! `searchenv.cpp`: find a file in the current directory or along the
//! directories of a `;`-separated environment variable (like `%PATH%`), and
//! return its full path.

use core::ffi::c_int;

use super::{
    calloc_t, errcheck, errcheck_einval, get_errno, invalid, set_errno, EnvChar, HeapPtr, EINVAL, ENOENT, ENOMEM,
    ERANGE, MAX_PATH,
};

/// `_MAX_PATH + 4`: the on-stack first-try buffer.
const LOCAL_PATH_COUNT: usize = MAX_PATH + 4;

unsafe fn common_searchenv_s<C: EnvChar>(
    file_name: *const C,
    environment_variable: *const C,
    result_buffer: *mut C,
    result_count: usize,
) -> c_int {
    if result_buffer.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    if result_count == 0 {
        invalid(EINVAL);
        return EINVAL;
    }
    if file_name.is_null() {
        *result_buffer = C::ch(0);
        invalid(EINVAL);
        return EINVAL;
    }

    // An empty name: empty result, ENOENT (no handler).
    if (*file_name).is(0) {
        *result_buffer = C::ch(0);
        set_errno(ENOENT);
        return ENOENT;
    }

    let mut saved_errno = get_errno();
    let access_result = C::taccess_s(file_name, 0);
    set_errno(saved_errno);

    // Found as given: qualify it.
    if access_result == 0 {
        if C::tfullpath(result_buffer, file_name, result_count).is_null() {
            *result_buffer = C::ch(0);
            return get_errno(); // fullpath set it
        }
        return 0;
    }

    let mut path_string: *mut C = core::ptr::null_mut();
    if errcheck_einval(C::tdupenv_s_crt(&mut path_string, core::ptr::null_mut(), environment_variable)) != 0
        || path_string.is_null()
    {
        *result_buffer = C::ch(0);
        set_errno(ENOENT); // the variable doesn't exist
        return ENOENT;
    }

    let _path_string_cleanup: HeapPtr<C> = HeapPtr(path_string);

    let file_name_length = C::tcslen(file_name);

    let mut local_path_buffer = [C::ch(0); LOCAL_PATH_COUNT];

    let mut path_count = LOCAL_PATH_COUNT;
    let mut path_buffer: *mut C = local_path_buffer.as_mut_ptr();
    if file_name_length >= result_count {
        // Too long for the local buffer: +2 for a separator and the terminator.
        path_count = C::tcslen(path_string) + file_name_length + 2;
        path_buffer = calloc_t::<C>(path_count);
        if path_buffer.is_null() {
            *result_buffer = C::ch(0);
            set_errno(ENOMEM);
            return ENOMEM;
        }
    }

    let local = local_path_buffer.as_mut_ptr();
    let mut path_buffer_cleanup: HeapPtr<C> =
        HeapPtr(if path_buffer == local { core::ptr::null_mut() } else { path_buffer });

    saved_errno = get_errno();
    while !path_string.is_null() {
        let previous_path_string = path_string;
        path_string = C::tgetpath(path_string, path_buffer, path_count.wrapping_sub(file_name_length).wrapping_sub(1));
        if path_string.is_null() && path_buffer == local && get_errno() == ERANGE {
            // Retry once with a buffer sized for this element. NOTE: this
            // second try passes `path_count - file_name_length`, one MORE than
            // the first -- as in the C.
            let required_count = C::tcslen(previous_path_string) + file_name_length + 2;

            // `path_buffer_cleanup = _calloc_crt_t(...)`: the move-assign frees
            // the old (NULL here: the local buffer was in use) pointer first.
            drop(core::mem::replace(&mut path_buffer_cleanup, HeapPtr(calloc_t::<C>(required_count))));
            if path_buffer_cleanup.is_null() {
                *result_buffer = C::ch(0);
                set_errno(ENOMEM);
                return ENOMEM;
            }

            path_count = required_count;
            path_buffer = path_buffer_cleanup.get();

            path_string = C::tgetpath(previous_path_string, path_buffer, path_count.wrapping_sub(file_name_length));
        }

        if path_string.is_null() || (*path_buffer).is(0) {
            *result_buffer = C::ch(0);
            set_errno(ENOENT);
            return ENOENT;
        }

        // Directory, a separator if it lacks one, then the name.
        let mut path_length = C::tcslen(path_buffer);
        let mut path_it = path_buffer.add(path_length);

        let last_character = *path_it.sub(1);
        if !last_character.is(b'/') && !last_character.is(b'\\') && !last_character.is(b':') {
            *path_it = C::ch(b'\\');
            path_it = path_it.add(1);
            path_length += 1;
        }

        errcheck(C::tcscpy_s(path_it, path_count.wrapping_sub(path_length), file_name));

        if C::taccess_s(path_buffer, 0) != 0 {
            continue;
        }

        if path_length + file_name_length + 1 > result_count {
            *result_buffer = C::ch(0);
            set_errno(ERANGE);
            return ERANGE;
        }

        set_errno(saved_errno);

        errcheck(C::tcscpy_s(result_buffer, result_count, path_buffer));
        return 0;
    }

    // Every directory tried; not found.
    *result_buffer = C::ch(0);
    set_errno(ENOENT);
    ENOENT
}

#[no_mangle]
pub unsafe extern "C" fn _searchenv_s(
    file_name: *const u8,
    environment_variable: *const u8,
    result_buffer: *mut u8,
    result_count: usize,
) -> c_int {
    common_searchenv_s(file_name, environment_variable, result_buffer, result_count)
}

#[no_mangle]
pub unsafe extern "C" fn _wsearchenv_s(
    file_name: *const u16,
    environment_variable: *const u16,
    result_buffer: *mut u16,
    result_count: usize,
) -> c_int {
    common_searchenv_s(file_name, environment_variable, result_buffer, result_count)
}

#[no_mangle]
pub unsafe extern "C" fn _searchenv(file_name: *const u8, environment_variable: *const u8, result_buffer: *mut u8) {
    common_searchenv_s(file_name, environment_variable, result_buffer, MAX_PATH);
}

#[no_mangle]
pub unsafe extern "C" fn _wsearchenv(
    file_name: *const u16,
    environment_variable: *const u16,
    result_buffer: *mut u16,
) {
    common_searchenv_s(file_name, environment_variable, result_buffer, MAX_PATH);
}
