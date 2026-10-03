//! `startup/assert.cpp` -- `_assert` and `_wassert`: report a failed
//! assertion, then abort (or, from the message box, let the user retry or
//! ignore).
//!
//! Details that are silent if wrong:
//! * Where the report goes: `_set_error_mode(_REPORT_ERRMODE)`; `_OUT_TO_STDERR`
//!   -> stderr; `_OUT_TO_DEFAULT` -> stderr only for a CONSOLE app
//!   (`_query_app_type() == _crt_console_app`); anything else (including
//!   `_OUT_TO_MSGBOX` and a default-mode GUI/unknown app) -> message box.
//! * stderr, wide only: first try the console directly -- stderr handle valid,
//!   `FILE_TYPE_CHAR`, `swprintf` into a 576-wchar buffer, `WriteConsoleW` --
//!   and abort if the write succeeded. Otherwise (and always for `_assert`):
//!   give stderr `_IONBF` if it has no buffer of any kind yet (so no stream
//!   buffer is allocated now), `fprintf`/`fwprintf` the
//!   "Assertion failed: %Ts, file %Ts, line %d\n" line, `fflush`, `abort`.
//!   The printf options word is the header's `__local_stdio_printf_options`
//!   storage, which nothing in ucrtbase ever sets: 0.
//! * The `%d` receives the `unsigned` line number, and the message box's
//!   `_itoa_s`/`_itow_s` takes it as an `int`: a line above INT_MAX prints
//!   negative in both, as in the C.
//! * Message box text (576 chars): "Assertion failed!", the program (the
//!   module containing `_ReturnAddress()`, or the EXE if that is ucrtbase
//!   itself or unknown; if the name fails, "<program name unknown>"), the
//!   file, the line, the expression, and the two fixed help paragraphs. The
//!   line-fitting arithmetic uses `program_intro_count`/`file_intro_count`,
//!   which COUNT THE NUL (10 and 7), exactly as Microsoft wrote it; the
//!   unsigned `size_t` wrap-arounds in the file-name trimming are kept
//!   (`wrapping_*`). Every `_ERRCHECK`'d string call that fails is
//!   `_invoke_watson`.
//! * Box result: Abort -> `raise(SIGABRT)` then `_exit(3)` (NOT `abort()`, so
//!   no Watson); Retry -> `__debugbreak()` and RETURN to the caller; Ignore ->
//!   return; anything else -> `abort()`.
//! * `_ReturnAddress()` has no stable-Rust intrinsic, so `_assert`/`_wassert`
//!   are naked entry stubs that append their own return address as a fourth
//!   argument and hand over to the Rust body.

use core::ffi::{c_char, c_int, c_uint, c_void};

use super::exit::_exit;
use super::CRT_SIGABRT;
use crate::locale::CrtLocalePointers;
use crate::stdio_layout::{flags, StreamData};

/// `MAXLINELEN`: max length for a line in the message box.
const MAXLINELEN: usize = 64;
/// `ASSERTBUFSZ`: nine lines.
const ASSERTBUFSZ: usize = MAXLINELEN * 9;
/// `_MAX_PATH` / `MAX_PATH`.
const MAX_PATH: usize = 260;

/// `_REPORT_ERRMODE` (`stdlib.h`).
const REPORT_ERRMODE: c_int = 3;
/// `_OUT_TO_DEFAULT`.
const OUT_TO_DEFAULT: c_int = 0;
/// `_OUT_TO_STDERR`.
const OUT_TO_STDERR: c_int = 1;
/// `_crt_console_app` (`corecrt_startup.h`).
const CRT_CONSOLE_APP: c_int = 1;
/// `_IONBF` (`stdio.h`).
const CRT_IONBF: c_int = 0x0004;
/// stderr's `__acrt_iob_func` index.
const STDERR_INDEX: c_uint = 2;

/// `STD_ERROR_HANDLE` ((DWORD)-12).
const STD_ERROR_HANDLE: u32 = -12i32 as u32;
const INVALID_HANDLE_VALUE: *mut c_void = usize::MAX as *mut c_void;
const FILE_TYPE_CHAR: u32 = 2;

/// `GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | ..._FROM_ADDRESS`.
const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x2;
const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;

/// `MB_TASKMODAL | MB_ICONHAND | MB_ABORTRETRYIGNORE | MB_SETFOREGROUND`
/// (0x2000 | 0x10 | 0x2 | 0x10000 = 0x12012, as the compiled C passes).
const MB_ABORTRETRYIGNORE: c_uint = 0x0000_0002;
const MB_ICONHAND: c_uint = 0x0000_0010;
const MB_TASKMODAL: c_uint = 0x0000_2000;
const MB_SETFOREGROUND: c_uint = 0x0001_0000;
const IDABORT: c_int = 3;
const IDRETRY: c_int = 4;
const IDIGNORE: c_int = 5;

extern "C" {
    fn _set_error_mode(new_error_mode: c_int) -> c_int;
    fn _query_app_type() -> c_int;
    fn raise(signum: c_int) -> c_int;
    fn abort() -> !;
    fn __acrt_iob_func(id: c_uint) -> *mut c_void;
    fn setvbuf(public_stream: *mut c_void, buffer: *mut c_char, kind: c_int, buffer_size_in_bytes: usize) -> c_int;
    fn fflush(public_stream: *mut c_void) -> c_int;
    fn __stdio_common_vfprintf(
        options: u64,
        stream: *mut c_void,
        format: *const u8,
        locale: *mut CrtLocalePointers,
        arglist: *mut u8,
    ) -> c_int;
    fn __stdio_common_vfwprintf(
        options: u64,
        stream: *mut c_void,
        format: *const u16,
        locale: *mut CrtLocalePointers,
        arglist: *mut u8,
    ) -> c_int;
    fn __stdio_common_vswprintf(
        options: u64,
        buffer: *mut u16,
        buffer_count: usize,
        format: *const u16,
        locale: *mut CrtLocalePointers,
        arglist: *mut u8,
    ) -> c_int;

    fn strcpy_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn strcat_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn strncat_s(dst: *mut u8, size: usize, src: *const u8, count: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn wcscat_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn wcsncat_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int;
    fn wcslen(s: *const u16) -> usize;
    fn _itoa_s(value: c_int, buffer: *mut u8, size_in_chars: usize, radix: c_int) -> c_int;
    fn _itow_s(value: c_int, buffer: *mut u16, size_in_words: usize, radix: c_int) -> c_int;
    fn memcpy_s(dst: *mut c_void, dst_size: usize, src: *const c_void, src_size: usize) -> c_int;
    fn _invoke_watson(
        expression: *const u16,
        function_name: *const u16,
        file_name: *const u16,
        line_number: u32,
        reserved: usize,
    ) -> !;

    /// `DWORD __cdecl __acrt_GetModuleFileNameA(HMODULE, char*, DWORD)`.
    fn __acrt_GetModuleFileNameA(module: *mut c_void, filename: *mut c_char, size: u32) -> u32;
    /// `int __cdecl __acrt_show_narrow_message_box(char const*, char const*, unsigned)`.
    fn __acrt_show_narrow_message_box(text: *const c_char, caption: *const c_char, type_: c_uint) -> c_int;
    /// `int __cdecl __acrt_show_wide_message_box(wchar_t const*, wchar_t const*, unsigned)`.
    fn __acrt_show_wide_message_box(text: *const u16, caption: *const u16, type_: c_uint) -> c_int;

    /// The linker's image-base symbol: this DLL's own base (CRTDLL).
    static __ImageBase: u8;
}

extern "system" {
    fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut c_void) -> c_int;
    fn GetModuleFileNameW(module: *mut c_void, filename: *mut u16, size: u32) -> u32;
    fn GetStdHandle(id: u32) -> *mut c_void;
    fn GetFileType(handle: *mut c_void) -> u32;
    fn WriteConsoleW(
        console: *mut c_void,
        buffer: *const c_void,
        chars_to_write: u32,
        chars_written: *mut u32,
        reserved: *mut c_void,
    ) -> c_int;
}

/// `_ERRCHECK(e)`: Watson on any nonzero errno_t.
#[inline(always)]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// `__debugbreak()`, emitted inline as the intrinsic is.
#[inline(always)]
unsafe fn debugbreak() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000");
}

/// `_GENERATE_TCHAR_STRING_FUNCTIONS(name, "...")`: the narrow and wide
/// literal (NUL-terminated), plus `name_length` (without the NUL) and
/// `name_count` (with it) as the C derives them from the narrow literal.
macro_rules! tchar_string {
    ($narrow:ident, $wide:ident, $length:ident, $count:ident, $s:literal) => {
        const $narrow: &[u8] = concat!($s, "\0").as_bytes();
        const $wide: &[u16] = super::w!($s);
        #[allow(dead_code)]
        const $length: usize = $narrow.len() - 1;
        #[allow(dead_code)]
        const $count: usize = $narrow.len();
    };
}

tchar_string!(ASSERT_FORMAT_N, ASSERT_FORMAT_W, ASSERT_FORMAT_LENGTH, ASSERT_FORMAT_COUNT, "Assertion failed: %Ts, file %Ts, line %d\n");
tchar_string!(BANNER_TEXT_N, BANNER_TEXT_W, BANNER_TEXT_LENGTH, BANNER_TEXT_COUNT, "Microsoft Visual C++ Runtime Library");
tchar_string!(BOX_INTRO_N, BOX_INTRO_W, BOX_INTRO_LENGTH, BOX_INTRO_COUNT, "Assertion failed!");
tchar_string!(PROGRAM_INTRO_N, PROGRAM_INTRO_W, PROGRAM_INTRO_LENGTH, PROGRAM_INTRO_COUNT, "Program: ");
tchar_string!(FILE_INTRO_N, FILE_INTRO_W, FILE_INTRO_LENGTH, FILE_INTRO_COUNT, "File: ");
tchar_string!(LINE_INTRO_N, LINE_INTRO_W, LINE_INTRO_LENGTH, LINE_INTRO_COUNT, "Line: ");
tchar_string!(EXPRESSION_INTRO_N, EXPRESSION_INTRO_W, EXPRESSION_INTRO_LENGTH, EXPRESSION_INTRO_COUNT, "Expression: ");
tchar_string!(
    INFO_INTRO_N,
    INFO_INTRO_W,
    INFO_INTRO_LENGTH,
    INFO_INTRO_COUNT,
    "For information on how your program can cause an assertion\nfailure, see the Visual C++ documentation on asserts"
);
tchar_string!(
    HELP_INTRO_N,
    HELP_INTRO_W,
    HELP_INTRO_LENGTH,
    HELP_INTRO_COUNT,
    "(Press Retry to debug the application - JIT must be enabled)"
);
tchar_string!(DOT_DOT_DOT_N, DOT_DOT_DOT_W, DOT_DOT_DOT_LENGTH, DOT_DOT_DOT_COUNT, "...");
tchar_string!(NEWLINE_N, NEWLINE_W, NEWLINE_LENGTH, NEWLINE_COUNT, "\n");
tchar_string!(DOUBLE_NEWLINE_N, DOUBLE_NEWLINE_W, DOUBLE_NEWLINE_LENGTH, DOUBLE_NEWLINE_COUNT, "\n\n");
tchar_string!(
    PROGRAM_NAME_UNKNOWN_TEXT_N,
    PROGRAM_NAME_UNKNOWN_TEXT_W,
    PROGRAM_NAME_UNKNOWN_TEXT_LENGTH,
    PROGRAM_NAME_UNKNOWN_TEXT_COUNT,
    "<program name unknown>"
);

/// `__crt_char_traits<Character>`: the operations assert.cpp's templates
/// call, narrow and wide.
trait AssertChar: Copy + 'static {
    const ZERO: Self;
    /// `get_<name>(Character())`.
    fn pick(narrow: &'static [u8], wide: &'static [u16]) -> *const Self;
    /// `'\\'` / `'/'` compare.
    fn is_separator(self) -> bool;
    unsafe fn tcscpy_s(dst: *mut Self, n: usize, src: *const Self) -> c_int;
    unsafe fn tcscat_s(dst: *mut Self, n: usize, src: *const Self) -> c_int;
    unsafe fn tcsncat_s(dst: *mut Self, n: usize, src: *const Self, count: usize) -> c_int;
    unsafe fn tcslen(s: *const Self) -> usize;
    unsafe fn itot_s(value: c_int, dst: *mut Self, n: usize, radix: c_int) -> c_int;
    unsafe fn get_module_file_name(module: *mut c_void, dst: *mut Self, n: u32) -> u32;
    unsafe fn show_message_box(text: *const Self, caption: *const Self, type_: c_uint) -> c_int;
    /// `traits::ftprintf(stream, format, ...)` with the args already laid out
    /// as a va_list.
    unsafe fn vftprintf(stream: *mut c_void, format: *const Self, arglist: *mut u8) -> c_int;
    /// `common_assert_to_stderr_direct`.
    unsafe fn assert_to_stderr_direct(expression: *const Self, file_name: *const Self, line_number: c_uint);
}

impl AssertChar for u8 {
    const ZERO: Self = 0;
    fn pick(narrow: &'static [u8], _wide: &'static [u16]) -> *const u8 {
        narrow.as_ptr()
    }
    fn is_separator(self) -> bool {
        self == b'\\' || self == b'/'
    }
    unsafe fn tcscpy_s(dst: *mut u8, n: usize, src: *const u8) -> c_int {
        strcpy_s(dst, n, src)
    }
    unsafe fn tcscat_s(dst: *mut u8, n: usize, src: *const u8) -> c_int {
        strcat_s(dst, n, src)
    }
    unsafe fn tcsncat_s(dst: *mut u8, n: usize, src: *const u8, count: usize) -> c_int {
        strncat_s(dst, n, src, count)
    }
    unsafe fn tcslen(s: *const u8) -> usize {
        strlen(s as *const c_char)
    }
    unsafe fn itot_s(value: c_int, dst: *mut u8, n: usize, radix: c_int) -> c_int {
        _itoa_s(value, dst, n, radix)
    }
    unsafe fn get_module_file_name(module: *mut c_void, dst: *mut u8, n: u32) -> u32 {
        __acrt_GetModuleFileNameA(module, dst as *mut c_char, n)
    }
    unsafe fn show_message_box(text: *const u8, caption: *const u8, type_: c_uint) -> c_int {
        __acrt_show_narrow_message_box(text as *const c_char, caption as *const c_char, type_)
    }
    unsafe fn vftprintf(stream: *mut c_void, format: *const u8, arglist: *mut u8) -> c_int {
        __stdio_common_vfprintf(0, stream, format, core::ptr::null_mut(), arglist)
    }
    unsafe fn assert_to_stderr_direct(_expression: *const u8, _file_name: *const u8, _line_number: c_uint) {
        // No action for narrow strings.
    }
}

impl AssertChar for u16 {
    const ZERO: Self = 0;
    fn pick(_narrow: &'static [u8], wide: &'static [u16]) -> *const u16 {
        wide.as_ptr()
    }
    fn is_separator(self) -> bool {
        self == b'\\' as u16 || self == b'/' as u16
    }
    unsafe fn tcscpy_s(dst: *mut u16, n: usize, src: *const u16) -> c_int {
        wcscpy_s(dst, n, src)
    }
    unsafe fn tcscat_s(dst: *mut u16, n: usize, src: *const u16) -> c_int {
        wcscat_s(dst, n, src)
    }
    unsafe fn tcsncat_s(dst: *mut u16, n: usize, src: *const u16, count: usize) -> c_int {
        wcsncat_s(dst, n, src, count)
    }
    unsafe fn tcslen(s: *const u16) -> usize {
        wcslen(s)
    }
    unsafe fn itot_s(value: c_int, dst: *mut u16, n: usize, radix: c_int) -> c_int {
        _itow_s(value, dst, n, radix)
    }
    unsafe fn get_module_file_name(module: *mut c_void, dst: *mut u16, n: u32) -> u32 {
        GetModuleFileNameW(module, dst, n)
    }
    unsafe fn show_message_box(text: *const u16, caption: *const u16, type_: c_uint) -> c_int {
        __acrt_show_wide_message_box(text, caption, type_)
    }
    unsafe fn vftprintf(stream: *mut c_void, format: *const u16, arglist: *mut u8) -> c_int {
        __stdio_common_vfwprintf(0, stream, format, core::ptr::null_mut(), arglist)
    }
    unsafe fn assert_to_stderr_direct(expression: *const u16, file_name: *const u16, line_number: c_uint) {
        let stderr_handle = GetStdHandle(STD_ERROR_HANDLE);
        if stderr_handle == INVALID_HANDLE_VALUE || stderr_handle.is_null() {
            return;
        }

        if GetFileType(stderr_handle) != FILE_TYPE_CHAR {
            return;
        }

        let mut assert_buffer = [0u16; ASSERTBUFSZ];
        // swprintf(buffer, count, format, ...) -> _vswprintf_c_l: a negative
        // result is reported as -1.
        let mut args = assert_args(expression, file_name, line_number);
        let r = __stdio_common_vswprintf(
            0,
            assert_buffer.as_mut_ptr(),
            ASSERTBUFSZ,
            ASSERT_FORMAT_W.as_ptr(),
            core::ptr::null_mut(),
            args.as_mut_ptr() as *mut u8,
        );
        if r < 0 {
            return;
        }

        let assert_buffer_length = wcslen(assert_buffer.as_ptr()) as u32;
        let mut characters_written: u32 = 0;
        if WriteConsoleW(
            stderr_handle,
            assert_buffer.as_ptr() as *const c_void,
            assert_buffer_length,
            &mut characters_written,
            core::ptr::null_mut(),
        ) == 0
        {
            return;
        }

        abort();
    }
}

/// The `(expression, file_name, line_number)` varargs as a Windows va_list:
/// one pointer-sized slot per argument on all three targets (the `unsigned`
/// line occupies the low half of its slot on 64-bit, as a promoted vararg
/// does).
#[inline(always)]
fn assert_args<C>(expression: *const C, file_name: *const C, line_number: c_uint) -> [usize; 3] {
    [expression as usize, file_name as usize, line_number as usize]
}

/// `__crt_stdio_stream(stderr).has_any_buffer()`.
unsafe fn stream_has_any_buffer(stream: *mut c_void) -> bool {
    let flag_word = core::ptr::addr_of!((*(stream as *const StreamData)).flags).read_volatile();
    flag_word & (flags::STREAM_BUFFER_CRT | flags::STREAM_BUFFER_USER | flags::STREAM_BUFFER_NONE) != 0
}

/// `common_assert_to_stderr`: never returns.
unsafe fn assert_to_stderr<C: AssertChar>(expression: *const C, file_name: *const C, line_number: c_uint) -> ! {
    // Straight to the console if we can (wide only; aborts on success).
    C::assert_to_stderr_direct(expression, file_name, line_number);

    // No buffer yet: make stderr unbuffered rather than allocate one now.
    if !stream_has_any_buffer(__acrt_iob_func(STDERR_INDEX)) {
        setvbuf(__acrt_iob_func(STDERR_INDEX), core::ptr::null_mut(), CRT_IONBF, 0);
    }

    let mut args = assert_args(expression, file_name, line_number);
    C::vftprintf(
        __acrt_iob_func(STDERR_INDEX),
        C::pick(ASSERT_FORMAT_N, ASSERT_FORMAT_W),
        args.as_mut_ptr() as *mut u8,
    );
    fflush(__acrt_iob_func(STDERR_INDEX));
    abort()
}

/// `common_assert_to_message_box_build_string`.
unsafe fn build_message_box_string<C: AssertChar>(
    assert_buffer: *mut C,
    assert_buffer_count: usize,
    expression: *const C,
    file_name: *const C,
    line_number: c_uint,
    return_address: *mut c_void,
) {
    let s = |n: &'static [u8], w: &'static [u16]| C::pick(n, w);

    // Line 1: box introduction.
    errcheck(C::tcscpy_s(assert_buffer, assert_buffer_count, s(BOX_INTRO_N, BOX_INTRO_W)));
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOUBLE_NEWLINE_N, DOUBLE_NEWLINE_W)));

    // Line 2: program.
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(PROGRAM_INTRO_N, PROGRAM_INTRO_W)));

    let mut program_name = [C::ZERO; MAX_PATH + 1];

    let mut asserting_module: *mut c_void = core::ptr::null_mut();
    if GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        return_address as *const u16,
        &mut asserting_module,
    ) == 0
    {
        asserting_module = core::ptr::null_mut();
    }

    // CRTDLL: an assert from inside ucrtbase is reported as the EXE's.
    if asserting_module as *const u8 == core::ptr::addr_of!(__ImageBase) {
        asserting_module = core::ptr::null_mut();
    }

    if C::get_module_file_name(asserting_module, program_name.as_mut_ptr(), program_name.len() as u32) == 0 {
        errcheck(C::tcscpy_s(
            program_name.as_mut_ptr(),
            program_name.len(),
            s(PROGRAM_NAME_UNKNOWN_TEXT_N, PROGRAM_NAME_UNKNOWN_TEXT_W),
        ));
    }

    let mut pch_prog = program_name.as_mut_ptr();
    // program_intro_COUNT (with its NUL), as Microsoft wrote it.
    let program_line = PROGRAM_INTRO_COUNT + C::tcslen(program_name.as_ptr()) + NEWLINE_LENGTH;
    if program_line > MAXLINELEN {
        pch_prog = pch_prog.add(program_line - MAXLINELEN);
        // Overwrite only the first three characters with the ellipsis.
        let used = pch_prog.offset_from(program_name.as_ptr()) as usize;
        errcheck(memcpy_s(
            pch_prog as *mut c_void,
            core::mem::size_of::<C>() * ((MAX_PATH + 1) - used),
            s(DOT_DOT_DOT_N, DOT_DOT_DOT_W) as *const c_void,
            core::mem::size_of::<C>() * DOT_DOT_DOT_LENGTH,
        ));
    }

    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, pch_prog));
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(NEWLINE_N, NEWLINE_W)));

    // Line 3: file.
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(FILE_INTRO_N, FILE_INTRO_W)));

    if FILE_INTRO_COUNT + C::tcslen(file_name) + NEWLINE_LENGTH > MAXLINELEN {
        let ffn = MAXLINELEN - FILE_INTRO_COUNT - NEWLINE_LENGTH;

        // p: distance from the end back to the last path separator (or len).
        let pch = file_name;
        let len = C::tcslen(file_name);
        let mut p: usize = 1;
        while !(*pch.add(len - p)).is_separator() && p < len {
            p += 1;
        }

        // Up to 2/3 of the line for the path, the rest for the file name.
        if (ffn - ffn / 3) < (len - p) && ffn / 3 > p {
            // The path is too long: first part of the path, full file name.
            errcheck(C::tcsncat_s(
                assert_buffer,
                assert_buffer_count,
                pch,
                ffn.wrapping_sub(DOT_DOT_DOT_LENGTH).wrapping_sub(p),
            ));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOT_DOT_DOT_N, DOT_DOT_DOT_W)));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, pch.add(len - p)));
        } else if ffn - ffn / 3 > len - p {
            // The file name is too long: full path, first and last part of
            // the file name.
            p /= 2;
            errcheck(C::tcsncat_s(
                assert_buffer,
                assert_buffer_count,
                pch,
                ffn.wrapping_sub(DOT_DOT_DOT_LENGTH).wrapping_sub(p),
            ));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOT_DOT_DOT_N, DOT_DOT_DOT_W)));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, pch.add(len - p)));
        } else {
            // Both too long.
            errcheck(C::tcsncat_s(assert_buffer, assert_buffer_count, pch, ffn - ffn / 3 - DOT_DOT_DOT_LENGTH));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOT_DOT_DOT_N, DOT_DOT_DOT_W)));
            errcheck(C::tcsncat_s(assert_buffer, assert_buffer_count, pch.add(len - p), ffn / 6 - 1));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOT_DOT_DOT_N, DOT_DOT_DOT_W)));
            errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, pch.add(len - (ffn / 3 - ffn / 6 - 2))));
        }
    } else {
        errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, file_name));
    }

    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(NEWLINE_N, NEWLINE_W)));

    // Line 4: line number (as an int).
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(LINE_INTRO_N, LINE_INTRO_W)));
    let used = C::tcslen(assert_buffer);
    errcheck(C::itot_s(line_number as c_int, assert_buffer.add(used), assert_buffer_count - used, 10));
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOUBLE_NEWLINE_N, DOUBLE_NEWLINE_W)));

    // Line 5: expression, truncated to what the fixed tail leaves room for.
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(EXPRESSION_INTRO_N, EXPRESSION_INTRO_W)));

    let characters_used =
        C::tcslen(assert_buffer) + 2 * DOUBLE_NEWLINE_LENGTH + INFO_INTRO_LENGTH + HELP_INTRO_COUNT;

    if characters_used + C::tcslen(expression) > assert_buffer_count {
        let characters_to_write = assert_buffer_count.wrapping_sub(characters_used + DOT_DOT_DOT_LENGTH);
        errcheck(C::tcsncat_s(assert_buffer, assert_buffer_count, expression, characters_to_write));
        errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOT_DOT_DOT_N, DOT_DOT_DOT_W)));
    } else {
        errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, expression));
    }

    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOUBLE_NEWLINE_N, DOUBLE_NEWLINE_W)));

    // Info line.
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(INFO_INTRO_N, INFO_INTRO_W)));
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(DOUBLE_NEWLINE_N, DOUBLE_NEWLINE_W)));

    // Help line.
    errcheck(C::tcscat_s(assert_buffer, assert_buffer_count, s(HELP_INTRO_N, HELP_INTRO_W)));
}

/// `common_assert_to_message_box`.
unsafe fn assert_to_message_box<C: AssertChar>(
    expression: *const C,
    file_name: *const C,
    line_number: c_uint,
    return_address: *mut c_void,
) {
    let mut assert_buffer = [C::ZERO; ASSERTBUFSZ];
    build_message_box_string(
        assert_buffer.as_mut_ptr(),
        ASSERTBUFSZ,
        expression,
        file_name,
        line_number,
        return_address,
    );

    let action = C::show_message_box(
        assert_buffer.as_ptr(),
        C::pick(BANNER_TEXT_N, BANNER_TEXT_W),
        MB_TASKMODAL | MB_ICONHAND | MB_ABORTRETRYIGNORE | MB_SETFOREGROUND,
    );

    match action {
        IDABORT => {
            // A user SIGABRT handler may return; then exit at once -- NOT
            // abort(), the user already had the chance to debug.
            raise(CRT_SIGABRT);
            _exit(3)
        }
        IDRETRY => {
            // Break into the debugger, then return to the caller.
            debugbreak();
        }
        IDIGNORE => {}
        _ => abort(),
    }
}

/// `common_assert`.
unsafe fn common_assert<C: AssertChar>(
    expression: *const C,
    file_name: *const C,
    line_number: c_uint,
    return_address: *mut c_void,
) {
    let current_error_mode = _set_error_mode(REPORT_ERRMODE);
    if current_error_mode == OUT_TO_STDERR {
        assert_to_stderr(expression, file_name, line_number);
    }

    if current_error_mode == OUT_TO_DEFAULT && _query_app_type() == CRT_CONSOLE_APP {
        assert_to_stderr(expression, file_name, line_number);
    }

    assert_to_message_box(expression, file_name, line_number, return_address);
}

/// `_assert`'s body; `return_address` is the stub's `_ReturnAddress()`.
unsafe extern "C" fn assert_with_return_address(
    expression: *const u8,
    file_name: *const u8,
    line_number: c_uint,
    return_address: *mut c_void,
) {
    common_assert(expression, file_name, line_number, return_address);
}

/// `_wassert`'s body.
unsafe extern "C" fn wassert_with_return_address(
    expression: *const u16,
    file_name: *const u16,
    line_number: c_uint,
    return_address: *mut c_void,
) {
    common_assert(expression, file_name, line_number, return_address);
}

/// Defines a naked `extern "C"` entry that calls `$body(a, b, c,
/// return_address)`: amd64 loads its return address into the fourth argument
/// register and tail-jumps; arm64 copies the link register; i386 re-pushes the
/// three stack arguments beneath its return address and calls.
macro_rules! entry_with_return_address {
    ($(#[$m:meta])* $name:ident($a:ident: $at:ty, $b:ident: $bt:ty, $c:ident: $ct:ty) => $body:ident) => {
        $(#[$m])*
        #[no_mangle]
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name($a: $at, $b: $bt, $c: $ct) {
            #[cfg(target_arch = "x86_64")]
            core::arch::naked_asm!("mov r9, [rsp]", "jmp {body}", body = sym $body);
            #[cfg(target_arch = "aarch64")]
            core::arch::naked_asm!("mov x3, x30", "b {body}", body = sym $body);
            #[cfg(target_arch = "x86")]
            core::arch::naked_asm!(
                "mov eax, [esp]",
                "push eax",
                "push dword ptr [esp + 16]",
                "push dword ptr [esp + 16]",
                "push dword ptr [esp + 16]",
                "call {body}",
                "add esp, 16",
                "ret",
                body = sym $body,
            );
        }
    };
}

entry_with_return_address! {
    /// `_assert`: report a failed narrow assertion.
    _assert(expression: *const c_char, file_name: *const c_char, line_number: c_uint) => assert_with_return_address
}

entry_with_return_address! {
    /// `_wassert`: report a failed wide assertion.
    _wassert(expression: *const u16, file_name: *const u16, line_number: c_uint) => wassert_with_return_address
}
