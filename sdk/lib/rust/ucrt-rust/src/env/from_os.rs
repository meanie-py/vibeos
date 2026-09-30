//! `get_environment_from_os.cpp`: copy the OS environment block into a CRT
//! (`malloc`) buffer the caller frees.
//!
//! Both widths start from `GetEnvironmentStringsW`: the narrow form converts
//! through `CP_ACP` itself, because `GetEnvironmentStringsA` answers in the OEM
//! code page. The OS block is released with `FreeEnvironmentStringsW` on every
//! path (`environment_strings_handle`'s destructor), AFTER the copy.

use core::ptr::null_mut;

use super::{
    FreeEnvironmentStringsW, GetEnvironmentStringsW, HeapPtr, __acrt_WideCharToMultiByte, malloc, wcslen, CP_ACP,
};

/// `environment_strings_handle`: `FreeEnvironmentStringsW` at scope exit.
struct EnvironmentStrings(*mut u16);

impl Drop for EnvironmentStrings {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                FreeEnvironmentStringsW(self.0);
            }
        }
    }
}

/// `find_end_of_double_null_terminated_sequence`: one past the second NUL.
unsafe fn find_end_of_double_null_terminated_sequence(first: *const u16) -> *const u16 {
    let mut last = first;
    while *last != 0 {
        last = last.add(wcslen(last) + 1);
    }
    last.add(1)
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_wide_environment_from_os() -> *mut u16 {
    let environment = EnvironmentStrings(GetEnvironmentStringsW());
    if environment.0.is_null() {
        return null_mut();
    }

    let first = environment.0 as *const u16;
    let last = find_end_of_double_null_terminated_sequence(first);

    let required_count = last.offset_from(first) as usize;

    // `_malloc_crt_t(wchar_t, n)`: malloc(n * sizeof(wchar_t)).
    let mut buffer: HeapPtr<u16> = HeapPtr(malloc(required_count * core::mem::size_of::<u16>()).cast());
    if buffer.is_null() {
        return null_mut();
    }

    core::ptr::copy_nonoverlapping(first, buffer.get(), required_count);
    buffer.detach()
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_narrow_environment_from_os() -> *mut u8 {
    let environment = EnvironmentStrings(GetEnvironmentStringsW());
    if environment.0.is_null() {
        return null_mut();
    }

    let first = environment.0 as *const u16;
    let last = find_end_of_double_null_terminated_sequence(first);

    let required_wide_count = last.offset_from(first) as usize;
    let required_narrow_count = __acrt_WideCharToMultiByte(
        CP_ACP,
        0,
        environment.0,
        required_wide_count as i32,
        null_mut(),
        0,
        core::ptr::null(),
        null_mut(),
    ) as usize;

    if required_narrow_count == 0 {
        return null_mut();
    }

    let mut buffer: HeapPtr<u8> = HeapPtr(malloc(required_narrow_count).cast());
    if buffer.is_null() {
        return null_mut();
    }

    let conversion_result = __acrt_WideCharToMultiByte(
        CP_ACP,
        0,
        environment.0,
        required_wide_count as i32,
        buffer.get().cast(),
        required_narrow_count as i32,
        core::ptr::null(),
        null_mut(),
    );

    if conversion_result == 0 {
        return null_mut();
    }

    buffer.detach()
}
