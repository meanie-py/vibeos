//! msvcrt's C `...` entry points, as varargs-rust `va_entry!` thunks over their Rust
//! `v`-twins. These were five C files (dll/win32/msvcrt/rust_{,console_,file_,process_}
//! varargs_shim.c and rust_scanf_shim.c) whose only job was `va_start`, which stable Rust
//! cannot write; the thunk lays the argument list out contiguously instead, so each body
//! below is the C body with `a.get(i)` for fixed argument i and `a.va(n)` for its `va_list`.
//! Generated from those files, one line per function, with the C signature above each;
//! the `_open` family (which reads its optional `pmode` only for `_O_CREAT`) is by hand.
//!
//! The thunks live in this staticlib, which every CRT DLL links ahead of msvcrt_static,
//! so they win first-match over the C copies still compiled into its scanf.c/wcs.c/file.c
//! objects, as the C shims did by being module objects.
#![cfg(target_os = "windows")]

use varargs_rust::va_entry;

const _O_CREAT: i32 = 0x0100;
const _SH_DENYNO: i32 = 0x40;

// int _wsopen(const wchar_t *path, int oflags, int shflags, ...)
va_entry!(_wsopen => |a| -> i32 {
    let (oflags, shflags) = (a.get(1) as i32, a.get(2) as i32);
    let pmode = if oflags & _O_CREAT != 0 { a.get(3) as i32 } else { 0 };
    let mut fd = 0i32;
    if crate::msvcrt::file::_wsopen_dispatch(a.get(0) as _, oflags, shflags, pmode, &mut fd, 0) != 0 { -1 } else { fd }
});
// int _sopen(const char *path, int oflags, int shflags, ...)
va_entry!(_sopen => |a| -> i32 {
    let (oflags, shflags) = (a.get(1) as i32, a.get(2) as i32);
    let pmode = if oflags & _O_CREAT != 0 { a.get(3) as i32 } else { 0 };
    let mut fd = 0i32;
    if crate::msvcrt::file::_sopen_dispatch(a.get(0) as _, oflags, shflags, pmode, &mut fd, 0) != 0 { -1 } else { fd }
});
// int _open(const char *path, int flags, ...): _sopen(path, flags, _SH_DENYNO[, pmode]),
// and _sopen reads pmode only for _O_CREAT, so the dispatch gets 0 otherwise.
va_entry!(_open => |a| -> i32 {
    let flags = a.get(1) as i32;
    let pmode = if flags & _O_CREAT != 0 { a.get(2) as i32 } else { 0 };
    let mut fd = 0i32;
    if crate::msvcrt::file::_sopen_dispatch(a.get(0) as _, flags, _SH_DENYNO, pmode, &mut fd, 0) != 0 { -1 } else { fd }
});
// int _wopen(const wchar_t *path, int flags, ...)
va_entry!(_wopen => |a| -> i32 {
    let flags = a.get(1) as i32;
    let pmode = if flags & _O_CREAT != 0 { a.get(2) as i32 } else { 0 };
    let mut fd = 0i32;
    if crate::msvcrt::file::_wsopen_dispatch(a.get(0) as _, flags, _SH_DENYNO, pmode, &mut fd, 0) != 0 { -1 } else { fd }
});

// ---- rust_console_varargs_shim.c (4) ----
// int _cprintf_l(const char *format, _locale_t locale, ...)
va_entry!(_cprintf_l => |a| -> i32 { crate::msvcrt::console::_vcprintf_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _cprintf(const char *format, ...)
va_entry!(_cprintf => |a| -> i32 { crate::msvcrt::console::_vcprintf(a.get(0) as _, a.va(1) as _) as _ });
// int _cwprintf_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_cwprintf_l => |a| -> i32 { crate::msvcrt::console::_vcwprintf_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _cwprintf(const wchar_t *format, ...)
va_entry!(_cwprintf => |a| -> i32 { crate::msvcrt::console::_vcwprintf(a.get(0) as _, a.va(1) as _) as _ });

// ---- rust_file_varargs_shim.c (16) ----
// int fprintf(FILE* file, const char *format, ...)
va_entry!(fprintf => |a| -> i32 { crate::msvcrt::file::vfprintf(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int fprintf_s(FILE* file, const char *format, ...)
va_entry!(fprintf_s => |a| -> i32 { crate::msvcrt::file::vfprintf_s(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _fprintf_l(FILE* file, const char *format, _locale_t locale, ...)
va_entry!(_fprintf_l => |a| -> i32 { crate::msvcrt::file::_vfprintf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _fprintf_p(FILE* file, const char *format, ...)
va_entry!(_fprintf_p => |a| -> i32 { crate::msvcrt::file::_vfprintf_p_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fprintf_p_l(FILE* file, const char *format, _locale_t locale, ...)
va_entry!(_fprintf_p_l => |a| -> i32 { crate::msvcrt::file::_vfprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _fprintf_s_l(FILE* file, const char *format, _locale_t locale, ...)
va_entry!(_fprintf_s_l => |a| -> i32 { crate::msvcrt::file::_vfprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int fwprintf(FILE* file, const wchar_t *format, ...)
va_entry!(fwprintf => |a| -> i32 { crate::msvcrt::file::vfwprintf(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int fwprintf_s(FILE* file, const wchar_t *format, ...)
va_entry!(fwprintf_s => |a| -> i32 { crate::msvcrt::file::vfwprintf_s(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _fwprintf_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
va_entry!(_fwprintf_l => |a| -> i32 { crate::msvcrt::file::_vfwprintf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _fwprintf_p(FILE* file, const wchar_t *format, ...)
va_entry!(_fwprintf_p => |a| -> i32 { crate::msvcrt::file::_vfwprintf_p_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fwprintf_p_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
va_entry!(_fwprintf_p_l => |a| -> i32 { crate::msvcrt::file::_vfwprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _fwprintf_s_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
va_entry!(_fwprintf_s_l => |a| -> i32 { crate::msvcrt::file::_vfwprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int printf(const char *format, ...)
va_entry!(printf => |a| -> i32 { crate::msvcrt::file::vfprintf(crate::msvcrt::file::__iob_func().add(1) as _, a.get(0) as _, a.va(1) as _) as _ });
// int printf_s(const char *format, ...)
va_entry!(printf_s => |a| -> i32 { crate::msvcrt::file::vprintf_s(a.get(0) as _, a.va(1) as _) as _ });
// int wprintf(const wchar_t *format, ...)
va_entry!(wprintf => |a| -> i32 { crate::msvcrt::file::vwprintf(a.get(0) as _, a.va(1) as _) as _ });
// int wprintf_s(const wchar_t *format, ...)
va_entry!(wprintf_s => |a| -> i32 { crate::msvcrt::file::vwprintf_s(a.get(0) as _, a.va(1) as _) as _ });

// ---- rust_process_varargs_shim.c (16) ----
// intptr_t _wexecl(const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wexecl => |a| -> isize { crate::msvcrt::process::_vwexecl(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _execl(const char *name, const char *arg0, ...)
va_entry!(_execl => |a| -> isize { crate::msvcrt::process::_vexecl(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _wexecle(const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wexecle => |a| -> isize { crate::msvcrt::process::_vwexecle(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _execle(const char *name, const char *arg0, ...)
va_entry!(_execle => |a| -> isize { crate::msvcrt::process::_vexecle(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _wexeclp(const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wexeclp => |a| -> isize { crate::msvcrt::process::_vwexeclp(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _execlp(const char *name, const char *arg0, ...)
va_entry!(_execlp => |a| -> isize { crate::msvcrt::process::_vexeclp(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _wexeclpe(const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wexeclpe => |a| -> isize { crate::msvcrt::process::_vwexeclpe(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _execlpe(const char *name, const char *arg0, ...)
va_entry!(_execlpe => |a| -> isize { crate::msvcrt::process::_vexeclpe(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// intptr_t _wspawnl(int flags, const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wspawnl => |a| -> isize { crate::msvcrt::process::_vwspawnl(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _spawnl(int flags, const char *name, const char *arg0, ...)
va_entry!(_spawnl => |a| -> isize { crate::msvcrt::process::_vspawnl(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _wspawnle(int flags, const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wspawnle => |a| -> isize { crate::msvcrt::process::_vwspawnle(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _spawnle(int flags, const char *name, const char *arg0, ...)
va_entry!(_spawnle => |a| -> isize { crate::msvcrt::process::_vspawnle(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _wspawnlp(int flags, const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wspawnlp => |a| -> isize { crate::msvcrt::process::_vwspawnlp(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _spawnlp(int flags, const char *name, const char *arg0, ...)
va_entry!(_spawnlp => |a| -> isize { crate::msvcrt::process::_vspawnlp(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _wspawnlpe(int flags, const wchar_t *name, const wchar_t *arg0, ...)
va_entry!(_wspawnlpe => |a| -> isize { crate::msvcrt::process::_vwspawnlpe(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// intptr_t _spawnlpe(int flags, const char *name, const char *arg0, ...)
va_entry!(_spawnlpe => |a| -> isize { crate::msvcrt::process::_vspawnlpe(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });

// ---- rust_scanf_shim.c (40) ----
// int fscanf(FILE *file, const char *format, ...)
va_entry!(fscanf => |a| -> i32 { crate::scanf::vfscanf_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fscanf_l(FILE *file, const char *format, _locale_t locale, ...)
va_entry!(_fscanf_l => |a| -> i32 { crate::scanf::vfscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int fscanf_s(FILE *file, const char *format, ...)
va_entry!(fscanf_s => |a| -> i32 { crate::scanf::vfscanf_s_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fscanf_s_l(FILE *file, const char *format, _locale_t locale, ...)
va_entry!(_fscanf_s_l => |a| -> i32 { crate::scanf::vfscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int scanf(const char *format, ...)
va_entry!(scanf => |a| -> i32 { crate::scanf::vfscanf_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _scanf_l(const char *format, _locale_t locale, ...)
va_entry!(_scanf_l => |a| -> i32 { crate::scanf::vfscanf_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int scanf_s(const char *format, ...)
va_entry!(scanf_s => |a| -> i32 { crate::scanf::vfscanf_s_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _scanf_s_l(const char *format, _locale_t locale, ...)
va_entry!(_scanf_s_l => |a| -> i32 { crate::scanf::vfscanf_s_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int fwscanf(FILE *file, const wchar_t *format, ...)
va_entry!(fwscanf => |a| -> i32 { crate::scanf::vfwscanf_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fwscanf_l(FILE *file, const wchar_t *format, _locale_t locale, ...)
va_entry!(_fwscanf_l => |a| -> i32 { crate::scanf::vfwscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int fwscanf_s(FILE *file, const wchar_t *format, ...)
va_entry!(fwscanf_s => |a| -> i32 { crate::scanf::vfwscanf_s_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _fwscanf_s_l(FILE *file, const wchar_t *format, _locale_t locale, ...)
va_entry!(_fwscanf_s_l => |a| -> i32 { crate::scanf::vfwscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int wscanf(const wchar_t *format, ...)
va_entry!(wscanf => |a| -> i32 { crate::scanf::vfwscanf_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _wscanf_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_wscanf_l => |a| -> i32 { crate::scanf::vfwscanf_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int wscanf_s(const wchar_t *format, ...)
va_entry!(wscanf_s => |a| -> i32 { crate::scanf::vfwscanf_s_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _wscanf_s_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_wscanf_s_l => |a| -> i32 { crate::scanf::vfwscanf_s_l(crate::msvcrt::file::__iob_func() as _, a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int sscanf(const char *str, const char *format, ...)
va_entry!(sscanf => |a| -> i32 { crate::scanf::vsscanf_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _sscanf_l(const char *str, const char *format, _locale_t locale, ...)
va_entry!(_sscanf_l => |a| -> i32 { crate::scanf::vsscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int sscanf_s(const char *str, const char *format, ...)
va_entry!(sscanf_s => |a| -> i32 { crate::scanf::vsscanf_s_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _sscanf_s_l(const char *str, const char *format, _locale_t locale, ...)
va_entry!(_sscanf_s_l => |a| -> i32 { crate::scanf::vsscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int swscanf(const wchar_t *str, const wchar_t *format, ...)
va_entry!(swscanf => |a| -> i32 { crate::scanf::vswscanf_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _swscanf_l(const wchar_t *str, const wchar_t *format, _locale_t locale, ...)
va_entry!(_swscanf_l => |a| -> i32 { crate::scanf::vswscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int swscanf_s(const wchar_t *str, const wchar_t *format, ...)
va_entry!(swscanf_s => |a| -> i32 { crate::scanf::vswscanf_s_l(a.get(0) as _, a.get(1) as _, core::ptr::null_mut(), a.va(2) as _) as _ });
// int _swscanf_s_l(const wchar_t *str, const wchar_t *format, _locale_t locale, ...)
va_entry!(_swscanf_s_l => |a| -> i32 { crate::scanf::vswscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _cscanf(const char *format, ...)
va_entry!(_cscanf => |a| -> i32 { crate::scanf::vcscanf_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _cscanf_l(const char *format, _locale_t locale, ...)
va_entry!(_cscanf_l => |a| -> i32 { crate::scanf::vcscanf_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _cscanf_s(const char *format, ...)
va_entry!(_cscanf_s => |a| -> i32 { crate::scanf::vcscanf_s_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _cscanf_s_l(const char *format, _locale_t locale, ...)
va_entry!(_cscanf_s_l => |a| -> i32 { crate::scanf::vcscanf_s_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _cwscanf(const wchar_t *format, ...)
va_entry!(_cwscanf => |a| -> i32 { crate::scanf::vcwscanf_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _cwscanf_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_cwscanf_l => |a| -> i32 { crate::scanf::vcwscanf_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _cwscanf_s(const wchar_t *format, ...)
va_entry!(_cwscanf_s => |a| -> i32 { crate::scanf::vcwscanf_s_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _cwscanf_s_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_cwscanf_s_l => |a| -> i32 { crate::scanf::vcwscanf_s_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _snscanf(const char *input, size_t length, const char *format, ...)
va_entry!(_snscanf => |a| -> i32 { crate::scanf::vsnscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _snscanf_l(const char *input, size_t length, const char *format, _locale_t locale, ...)
va_entry!(_snscanf_l => |a| -> i32 { crate::scanf::vsnscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snscanf_s(const char *input, size_t length, const char *format, ...)
va_entry!(_snscanf_s => |a| -> i32 { crate::scanf::vsnscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _snscanf_s_l(const char *input, size_t length, const char *format, _locale_t locale, ...)
va_entry!(_snscanf_s_l => |a| -> i32 { crate::scanf::vsnscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snwscanf(wchar_t *input, size_t length, const wchar_t *format, ...)
va_entry!(_snwscanf => |a| -> i32 { crate::scanf::vsnwscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _snwscanf_l(wchar_t *input, size_t length, const wchar_t *format, _locale_t locale, ...)
va_entry!(_snwscanf_l => |a| -> i32 { crate::scanf::vsnwscanf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snwscanf_s(wchar_t *input, size_t length, const wchar_t *format, ...)
va_entry!(_snwscanf_s => |a| -> i32 { crate::scanf::vsnwscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _snwscanf_s_l(wchar_t *input, size_t length, const wchar_t *format, _locale_t locale, ...)
va_entry!(_snwscanf_s_l => |a| -> i32 { crate::scanf::vsnwscanf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });

// ---- rust_varargs_shim.c (32) ----
// int sprintf(char *str, const char *format, ...)
va_entry!(sprintf => |a| -> i32 { crate::printf_shells::_vsnprintf(a.get(0) as _, i32::MAX as _, a.get(1) as _, a.va(2) as _) as _ });
// int _sprintf_l(char *str, const char *format, _locale_t locale, ...)
va_entry!(_sprintf_l => |a| -> i32 { crate::printf_shells::_vsnprintf_l(a.get(0) as _, i32::MAX as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _snprintf(char *str, size_t len, const char *format, ...)
va_entry!(_snprintf => |a| -> i32 { crate::printf_shells::_vsnprintf(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _snprintf_l(char *str, size_t count, const char *format, _locale_t locale, ...)
va_entry!(_snprintf_l => |a| -> i32 { crate::printf_shells::_vsnprintf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snprintf_c(char *str, size_t count, const char *format, ...)
va_entry!(_snprintf_c => |a| -> i32 { crate::printf_shells::_vsnprintf_c(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _snprintf_c_l(char *str, size_t count, const char *format, _locale_t locale, ...)
va_entry!(_snprintf_c_l => |a| -> i32 { crate::printf_shells::_vsnprintf_c_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int sprintf_s(char *str, size_t num, const char *format, ...)
va_entry!(sprintf_s => |a| -> i32 { crate::printf_shells::_vsnprintf(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _sprintf_s_l(char *str, size_t count, const char *format, _locale_t locale, ...)
va_entry!(_sprintf_s_l => |a| -> i32 { crate::printf_shells::_vsnprintf_s_l(a.get(0) as _, i32::MAX as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snprintf_s(char *str, size_t len, size_t count, const char *format, ...)
va_entry!(_snprintf_s => |a| -> i32 { crate::printf_shells::_vsnprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, core::ptr::null_mut(), a.va(4) as _) as _ });
// int _snprintf_s_l(char *str, size_t len, size_t count, const char *format, _locale_t locale, ...)
va_entry!(_snprintf_s_l => |a| -> i32 { crate::printf_shells::_vsnprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.get(4) as _, a.va(5) as _) as _ });
// int _sprintf_p(char *buffer, size_t length, const char *format, ...)
va_entry!(_sprintf_p => |a| -> i32 { crate::printf_shells::_vsprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _sprintf_p_l(char *buffer, size_t length, const char *format, _locale_t locale, ...)
va_entry!(_sprintf_p_l => |a| -> i32 { crate::printf_shells::_vsprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _scprintf(const char *format, ...)
va_entry!(_scprintf => |a| -> i32 { crate::printf_shells::_vscprintf(a.get(0) as _, a.va(1) as _) as _ });
// int _scprintf_l(const char *format, _locale_t locale, ...)
va_entry!(_scprintf_l => |a| -> i32 { crate::printf_shells::_vscprintf_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _scprintf_p(const char *format, ...)
va_entry!(_scprintf_p => |a| -> i32 { crate::printf_shells::_vscprintf_p_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _scprintf_p_l(const char *format, _locale_t locale, ...)
va_entry!(_scprintf_p_l => |a| -> i32 { crate::printf_shells::_vscprintf_p_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _swprintf(wchar_t *str, const wchar_t *format, ...)
va_entry!(_swprintf => |a| -> i32 { crate::printf_shells::_vsnwprintf(a.get(0) as _, i32::MAX as _, a.get(1) as _, a.va(2) as _) as _ });
// int __swprintf_l(wchar_t *str, const wchar_t *format, _locale_t locale, ...)
va_entry!(__swprintf_l => |a| -> i32 { crate::printf_shells::_vswprintf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _snwprintf(wchar_t *str, size_t len, const wchar_t *format, ...)
va_entry!(_snwprintf => |a| -> i32 { crate::printf_shells::_vsnwprintf(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _snwprintf_l(wchar_t *str, size_t len, const wchar_t *format, _locale_t locale, ...)
va_entry!(_snwprintf_l => |a| -> i32 { crate::printf_shells::_vsnwprintf_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _swprintf_c(wchar_t *str, size_t len, const wchar_t *format, ...)
va_entry!(_swprintf_c => |a| -> i32 { crate::printf_shells::_vswprintf_c(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _) as _ });
// int _swprintf_c_l(wchar_t *str, size_t len, const wchar_t *format, _locale_t locale, ...)
va_entry!(_swprintf_c_l => |a| -> i32 { crate::printf_shells::_vswprintf_c_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int swprintf_s(wchar_t *str, size_t numberOfElements, const wchar_t *format, ...)
va_entry!(swprintf_s => |a| -> i32 { crate::printf_shells::_vsnwprintf_s(a.get(0) as _, a.get(1) as _, i32::MAX as _, a.get(2) as _, a.va(3) as _) as _ });
// int _swprintf_s_l(wchar_t *str, size_t numberOfElements, const wchar_t *format, _locale_t locale, ...)
va_entry!(_swprintf_s_l => |a| -> i32 { crate::printf_shells::_vsnwprintf_s_l(a.get(0) as _, a.get(1) as _, i32::MAX as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _snwprintf_s(wchar_t *str, size_t len, size_t count, const wchar_t *format, ...)
va_entry!(_snwprintf_s => |a| -> i32 { crate::printf_shells::_vsnwprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, core::ptr::null_mut(), a.va(4) as _) as _ });
// int _snwprintf_s_l(wchar_t *str, size_t len, size_t count, const wchar_t *format, _locale_t locale, ...)
va_entry!(_snwprintf_s_l => |a| -> i32 { crate::printf_shells::_vsnwprintf_s_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.get(4) as _, a.va(5) as _) as _ });
// int _swprintf_p(wchar_t *buffer, size_t length, const wchar_t *format, ...)
va_entry!(_swprintf_p => |a| -> i32 { crate::printf_shells::_vswprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, core::ptr::null_mut(), a.va(3) as _) as _ });
// int _swprintf_p_l(wchar_t *buffer, size_t length, const wchar_t *format, _locale_t locale, ...)
va_entry!(_swprintf_p_l => |a| -> i32 { crate::printf_shells::_vswprintf_p_l(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.get(3) as _, a.va(4) as _) as _ });
// int _scwprintf(const wchar_t *format, ...)
va_entry!(_scwprintf => |a| -> i32 { crate::printf_shells::_vsnwprintf(core::ptr::null_mut(), i32::MAX as _, a.get(0) as _, a.va(1) as _) as _ });
// int _scwprintf_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_scwprintf_l => |a| -> i32 { crate::printf_shells::_vsnwprintf_l(core::ptr::null_mut(), i32::MAX as _, a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
// int _scwprintf_p(const wchar_t *format, ...)
va_entry!(_scwprintf_p => |a| -> i32 { crate::printf_shells::_vscwprintf_p_l(a.get(0) as _, core::ptr::null_mut(), a.va(1) as _) as _ });
// int _scwprintf_p_l(const wchar_t *format, _locale_t locale, ...)
va_entry!(_scwprintf_p_l => |a| -> i32 { crate::printf_shells::_vscwprintf_p_l(a.get(0) as _, a.get(1) as _, a.va(2) as _) as _ });
