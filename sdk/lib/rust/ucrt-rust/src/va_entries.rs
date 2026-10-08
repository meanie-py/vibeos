//! ucrtbase's C `...` entry points, as varargs-rust `va_entry!` thunks over their Rust `_v`
//! twins. These were dll/win32/ucrtbase/ucrt_varargs_shim.c (va_start trampolines, plus the
//! six va_arg pullers, which varargs-rust's `rust-pullers` feature now supplies) and the
//! `sprintf` half of printf.c. The `_v` twins take a POINTER to the `va_list` (`&ap` in the
//! C), so each body passes `&mut va`.

use core::ffi::c_void;
use varargs_rust::va_entry;

macro_rules! spawn_entry {
    ($($export:ident => $twin:ident),* $(,)?) => {$(
        // intptr_t name(int mode, ch const *file_name, ch const *arguments, ...)
        va_entry!($export => |a| -> isize {
            let mut va: *mut c_void = a.va(3);
            crate::exec::$twin(a.get(0) as _, a.get(1) as _, a.get(2) as _, &mut va as *mut _ as _) as _
        });
    )*};
}
macro_rules! exec_entry {
    ($($export:ident => $twin:ident),* $(,)?) => {$(
        // intptr_t name(ch const *file_name, ch const *arguments, ...)
        va_entry!($export => |a| -> isize {
            let mut va: *mut c_void = a.va(2);
            crate::exec::$twin(a.get(0) as _, a.get(1) as _, &mut va as *mut _ as _) as _
        });
    )*};
}
spawn_entry! {
    _spawnl => _spawnl_v, _spawnle => _spawnle_v, _spawnlp => _spawnlp_v, _spawnlpe => _spawnlpe_v,
    _wspawnl => _wspawnl_v, _wspawnle => _wspawnle_v, _wspawnlp => _wspawnlp_v, _wspawnlpe => _wspawnlpe_v,
}
exec_entry! {
    _execl => _execl_v, _execle => _execle_v, _execlp => _execlp_v, _execlpe => _execlpe_v,
    _wexecl => _wexecl_v, _wexecle => _wexecle_v, _wexeclp => _wexeclp_v, _wexeclpe => _wexeclpe_v,
}

// int _open(char const *path, int oflag, ...) / _wopen
va_entry!(_open => |a| -> i32 {
    let mut va: *mut c_void = a.va(2);
    crate::lowio_open::_open_v(a.get(0) as _, a.get(1) as _, &mut va as *mut _ as _)
});
va_entry!(_wopen => |a| -> i32 {
    let mut va: *mut c_void = a.va(2);
    crate::lowio_open::_wopen_v(a.get(0) as _, a.get(1) as _, &mut va as *mut _ as _)
});
// int _sopen(char const *path, int oflag, int shflag, ...) / _wsopen
va_entry!(_sopen => |a| -> i32 {
    let mut va: *mut c_void = a.va(3);
    crate::lowio_open::_sopen_v(a.get(0) as _, a.get(1) as _, a.get(2) as _, &mut va as *mut _ as _)
});
va_entry!(_wsopen => |a| -> i32 {
    let mut va: *mut c_void = a.va(3);
    crate::lowio_open::_wsopen_v(a.get(0) as _, a.get(1) as _, a.get(2) as _, &mut va as *mut _ as _)
});

// printf.c: the two functions UCRT headers inline but the legacy CRT's callers link to.
// The module's own __local_stdio_printf_options storage is never set, so the options are
// just _CRT_INTERNAL_PRINTF_LEGACY_VSPRINTF_NULL_TERMINATION.
const LEGACY_VSPRINTF_NULL_TERMINATION: u64 = 0x0001;

// int sprintf(char *buffer, char const *format, ...)
va_entry!(sprintf => |a| -> i32 {
    let r = crate::output::__stdio_common_vsprintf(
        LEGACY_VSPRINTF_NULL_TERMINATION as _, a.get(0) as _, usize::MAX as _, a.get(1) as _,
        core::ptr::null_mut(), a.va(2) as _);
    if r < 0 { -1 } else { r }
});

/// int _vsnprintf(char *buffer, size_t count, char const *format, va_list args)
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf(buffer: *mut i8, count: usize, format: *const i8, args: *mut c_void) -> i32 {
    let r = crate::output::__stdio_common_vsprintf(
        LEGACY_VSPRINTF_NULL_TERMINATION as _, buffer as _, count as _, format as _,
        core::ptr::null_mut(), args as _);
    if r < 0 { -1 } else { r }
}
