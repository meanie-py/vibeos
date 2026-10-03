//! `environment_initialization.cpp`: the four environment tables, their
//! creation from the OS block, teardown, and the lazy get-or-create that keeps
//! the narrow and wide tables in step.

use core::ffi::c_int;
use core::ptr::{addr_of_mut, null_mut};

use super::{calloc_t, errcheck, free, EnvChar, HeapPtr};

// ── the tables ───────────────────────────────────────────────────────────────

/// The environment handed to `main`: never modified after startup (the first
/// change clones it; see setenv.rs). A ucrtbase export in its own right.
#[no_mangle]
pub static mut __dcrt_initial_narrow_environment: *mut *mut u8 = null_mut();
#[no_mangle]
pub static mut __dcrt_initial_wide_environment: *mut *mut u16 = null_mut();

/// `dual_state_global<char**> _environ_table`: one pointer (no state
/// isolation). Publicly documented name; `_environ` is `*__p__environ()`.
#[no_mangle]
pub static mut _environ_table: *mut *mut u8 = null_mut();
#[no_mangle]
pub static mut _wenviron_table: *mut *mut u16 = null_mut();

#[no_mangle]
pub unsafe extern "C" fn __p__environ() -> *mut *mut *mut u8 {
    addr_of_mut!(_environ_table)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wenviron() -> *mut *mut *mut u16 {
    addr_of_mut!(_wenviron_table)
}

// ── create / free (no global state) ──────────────────────────────────────────

/// `count_variables_in_environment_block`: strings NOT starting with `=`.
unsafe fn count_variables_in_environment_block<C: EnvChar>(environment_block: *mut C) -> usize {
    let mut count = 0usize;
    let mut it = environment_block;
    while !(*it).is(0) {
        if !(*it).is(b'=') {
            count += 1;
        }
        it = it.add(C::tcslen(it) + 1);
    }
    count
}

/// `free_environment`: each string, then the array. NULL is a no-op.
pub(crate) unsafe fn free_environment<C: EnvChar>(environment: *mut *mut C) {
    if environment.is_null() {
        return;
    }
    let mut it = environment;
    while !(*it).is_null() {
        free((*it).cast());
        it = it.add(1);
    }
    free(environment.cast());
}

/// `create_environment`: an array of copies of every `name=value` string in
/// the double-NUL-terminated block, skipping the `=X:` drive strings.
unsafe fn create_environment<C: EnvChar>(environment_block: *mut C) -> *mut *mut C {
    let variable_count = count_variables_in_environment_block(environment_block);

    let mut environment: HeapPtr<*mut C> = HeapPtr(calloc_t::<*mut C>(variable_count + 1));
    if environment.is_null() {
        return null_mut();
    }

    let mut source_it = environment_block;
    let mut result_it = environment.get();

    while !(*source_it).is(0) {
        let required_count = C::tcslen(source_it) + 1;

        // Don't copy drive letter settings, which start with '=':
        if !(*source_it).is(b'=') {
            let mut variable: HeapPtr<C> = HeapPtr(calloc_t::<C>(required_count));
            if variable.is_null() {
                free_environment(environment.detach());
                return null_mut();
            }
            errcheck(C::tcscpy_s(variable.get(), required_count, source_it));
            *result_it = variable.detach();
            result_it = result_it.add(1);
        }

        source_it = source_it.add(required_count);
    }

    // Already NULL-terminated by the calloc.
    environment.detach()
}

// ── initialize / uninitialize ────────────────────────────────────────────────

/// `common_initialize_environment_nolock`: 0 on success (or if the table
/// already exists), -1 on failure.
unsafe fn common_initialize_environment_nolock<C: EnvChar>() -> c_int {
    // Only ever once; later changes go through setenv.
    if !(*C::table()).is_null() {
        return 0;
    }

    C::pre_initialize();

    let os_environment: HeapPtr<C> = HeapPtr(C::get_environment_from_os());
    if os_environment.is_null() {
        return -1;
    }

    let mut crt_environment: HeapPtr<*mut C> = HeapPtr(create_environment(os_environment.get()));
    if crt_environment.is_null() {
        return -1;
    }

    *C::initial() = crt_environment.get();
    // `dual_state_global::initialize`.
    *C::table() = crt_environment.detach();
    0
}

#[no_mangle]
pub unsafe extern "C" fn _initialize_narrow_environment() -> c_int {
    common_initialize_environment_nolock::<u8>()
}

#[no_mangle]
pub unsafe extern "C" fn _initialize_wide_environment() -> c_int {
    common_initialize_environment_nolock::<u16>()
}

/// `uninitialize_environment_internal`: frees the current table unless it IS
/// the initial one (which is freed separately right after).
unsafe fn uninitialize_environment_internal<C: EnvChar>(environment: *mut *mut *mut C) {
    if *environment == *C::initial() {
        return;
    }
    free_environment(*environment);
}

/// Frees both tables and both initial tables. Nothing is set back to NULL,
/// exactly as in the C (this runs at CRT teardown).
#[no_mangle]
pub unsafe extern "C" fn __dcrt_uninitialize_environments_nolock() {
    uninitialize_environment_internal::<u8>(addr_of_mut!(_environ_table));
    uninitialize_environment_internal::<u16>(addr_of_mut!(_wenviron_table));

    free_environment(*addr_of_mut!(__dcrt_initial_narrow_environment));
    free_environment(*addr_of_mut!(__dcrt_initial_wide_environment));
}

// ── get-or-create ────────────────────────────────────────────────────────────

/// `initialize_environment_by_cloning_nolock`: build this width's table from
/// the other one, converting each string through `CP_ACP`.
unsafe fn initialize_environment_by_cloning_nolock<C: EnvChar>() -> c_int {
    let other_environment = *<C::Other as EnvChar>::table();
    if other_environment.is_null() {
        return -1;
    }

    let mut it = other_environment;
    while !(*it).is_null() {
        let required_count = <C::Other as EnvChar>::required_transform_count(*it);
        if required_count == 0 {
            return -1;
        }

        let mut buffer: HeapPtr<C> = HeapPtr(calloc_t::<C>(required_count));
        if buffer.is_null() {
            return -1;
        }

        // C::Other::Other is C: the transform writes this width.
        let actual_count = transform_into::<C>(*it, buffer.get(), required_count);
        if actual_count == 0 {
            return -1;
        }

        // A failed set is ignored; carry on with the rest.
        let _ = C::set_variable_in_environment_nolock(buffer.detach(), 0);
        it = it.add(1);
    }
    0
}

/// `__crt_transform_string(CP_ACP, other_string, buffer, count)` with the
/// destination typed as `C` (the trait's own signature types it as
/// `Other::Other`, which is `C` for both impls).
unsafe fn transform_into<C: EnvChar>(s: *const C::Other, buffer: *mut C, count: usize) -> usize {
    if C::WIDE {
        <u8 as EnvChar>::transform_string(s.cast(), buffer.cast(), count)
    } else {
        <u16 as EnvChar>::transform_string(s.cast(), buffer.cast(), count)
    }
}

/// `common_get_or_create_environment_nolock`.
pub(crate) unsafe fn common_get_or_create_environment_nolock<C: EnvChar>() -> *mut *mut C {
    let existing_environment = *C::table();
    if !existing_environment.is_null() {
        return existing_environment;
    }

    // Only create this table when the other one already exists.
    let other_environment = *<C::Other as EnvChar>::table();
    if other_environment.is_null() {
        return null_mut();
    }

    if common_initialize_environment_nolock::<C>() != 0 && initialize_environment_by_cloning_nolock::<C>() != 0 {
        return null_mut();
    }

    *C::table()
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_or_create_narrow_environment_nolock() -> *mut *mut u8 {
    common_get_or_create_environment_nolock::<u8>()
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_or_create_wide_environment_nolock() -> *mut *mut u16 {
    common_get_or_create_environment_nolock::<u16>()
}

unsafe fn common_get_initial_environment<C: EnvChar>() -> *mut *mut C {
    let initial_environment = C::initial();
    if (*initial_environment).is_null() {
        *initial_environment = common_get_or_create_environment_nolock::<C>();
    }
    *initial_environment
}

#[no_mangle]
pub unsafe extern "C" fn _get_initial_narrow_environment() -> *mut *mut u8 {
    common_get_initial_environment::<u8>()
}

#[no_mangle]
pub unsafe extern "C" fn _get_initial_wide_environment() -> *mut *mut u16 {
    common_get_initial_environment::<u16>()
}
