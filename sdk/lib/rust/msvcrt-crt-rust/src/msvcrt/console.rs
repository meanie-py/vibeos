use crate::prelude::*;
// NOT `use msvcrt_sys::*;` as well. prelude.rs already re-exports the whole island
// (`pub use msvcrt_sys::*`), so a second glob imports every name TWICE from two different
// roots and every use of an overlapping one is E0659 "ambiguous" — this file arrived with
// the second glob and ten `WEOF`/`FALSE` uses failed exactly that way. The prelude also
// re-types a few island constants to the shape their uses need, and an explicit item beats
// a glob, so going through the prelude keeps those choices. One door.

// The printf engine, already Rust — the same set file.rs drives. `_cvcprintf`/`_vcwprintf`
// hand their va_list straight to it.
use crate::printf::{arg_clbk_valist, pf_printf_a, pf_printf_w};
// CRT siblings ALREADY Rust in this crate. IMPORTED, not re-declared `extern`: both would
// link, but only the import makes rustc check the signature against the definition.
use crate::wcslen;

// console.c's TRACE/WARN are wine-debug macros the translation drops. A `macro_rules!`
// no-op, never an out-of-line `fn` — an out-of-line no-op is an optimization barrier that
// has uncloaked latent ABI mismatches into `ud2` before now.
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `_CONIO_LOCK` comes from the msvcrt-sys island (mtdll.h), not from a literal here —
// see the note in exit.rs for the incident that made these two derived rather than typed.

// console.c:33-34 — `#define LOCK_CONSOLE _lock(_CONIO_LOCK)` / `UNLOCK_CONSOLE _unlock(...)`.
// `macro_rules!` rather than a helper `fn`, for the reason given in exit.rs.
macro_rules! LOCK_CONSOLE {
    () => {
        _lock(_CONIO_LOCK as core::ffi::c_int)
    };
}
macro_rules! UNLOCK_CONSOLE {
    () => {
        _unlock(_CONIO_LOCK as core::ffi::c_int)
    };
}

pub const NORMAL_CHAR: u32 = 0;
pub const ALT_CHAR: u32 = 1;
pub const CTRL_CHAR: u32 = 2;
pub const SHIFT_CHAR: u32 = 3;

static mut MSVCRT_console_in: HANDLE = core::ptr::null_mut();
static mut MSVCRT_console_out: HANDLE = core::ptr::null_mut();
static mut __MSVCRT_console_buffer: core::ffi::c_int = EOF;
static mut __MSVCRT_console_buffer_w: wchar_t = WEOF as wchar_t;

#[repr(C)]
pub struct enh_map_entry {
    pub vk: core::ffi::c_ushort,
    pub ch: [[core::ffi::c_uchar; 2]; 4],
}

pub static enh_map: [enh_map_entry; 10] = [
    enh_map_entry { vk: 0x47, ch: [[0xE0, 0x47], [0x00, 0x97], [0xE0, 0x77], [0xE0, 0x47]] },
    enh_map_entry { vk: 0x48, ch: [[0xE0, 0x48], [0x00, 0x98], [0xE0, 0x8D], [0xE0, 0x48]] },
    enh_map_entry { vk: 0x49, ch: [[0xE0, 0x49], [0x00, 0x99], [0xE0, 0x86], [0xE0, 0x49]] },
    enh_map_entry { vk: 0x4B, ch: [[0xE0, 0x4B], [0x00, 0x9B], [0xE0, 0x73], [0xE0, 0x4B]] },
    enh_map_entry { vk: 0x4D, ch: [[0xE0, 0x4D], [0x00, 0x9D], [0xE0, 0x74], [0xE0, 0x4D]] },
    enh_map_entry { vk: 0x4F, ch: [[0xE0, 0x4F], [0x00, 0x9F], [0xE0, 0x75], [0xE0, 0x4F]] },
    enh_map_entry { vk: 0x50, ch: [[0xE0, 0x50], [0x00, 0xA0], [0xE0, 0x91], [0xE0, 0x50]] },
    enh_map_entry { vk: 0x51, ch: [[0xE0, 0x51], [0x00, 0xA1], [0xE0, 0x76], [0xE0, 0x51]] },
    enh_map_entry { vk: 0x52, ch: [[0xE0, 0x52], [0x00, 0xA2], [0xE0, 0x92], [0xE0, 0x52]] },
    enh_map_entry { vk: 0x53, ch: [[0xE0, 0x53], [0x00, 0xA3], [0xE0, 0x93], [0xE0, 0x53]] },
];
unsafe extern "C" fn msvcrt_input_console() -> HANDLE {
    if MSVCRT_console_in.is_null() {
        MSVCRT_console_in = CreateFileA(
            b"CONIN$\0".as_ptr() as PCHAR,
            GENERIC_WRITE | GENERIC_READ,
            FILE_SHARE_WRITE | FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if MSVCRT_console_in == INVALID_HANDLE_VALUE {
        }
    }
    MSVCRT_console_in
}
unsafe extern "C" fn msvcrt_output_console() -> HANDLE {
    if MSVCRT_console_out.is_null() {
        MSVCRT_console_out = CreateFileA(
            b"CONOUT$\0".as_ptr() as PCHAR,
            GENERIC_WRITE,
            FILE_SHARE_WRITE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if MSVCRT_console_out == INVALID_HANDLE_VALUE {
        }
    }
    MSVCRT_console_out
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_console() {
    CloseHandle(MSVCRT_console_in);
    CloseHandle(MSVCRT_console_out);
}

#[no_mangle]
pub unsafe extern "C" fn _cputs(str: *const core::ffi::c_char) -> core::ffi::c_int {
    let mut count: DWORD = 0;
    let len: core::ffi::c_int;
    let mut retval: core::ffi::c_int = -1;

    if MSVCRT_CHECK_PMT!(str != core::ptr::null()) == 0 {
        return -1;
    }
    len = strlen(str) as core::ffi::c_int;

    LOCK_CONSOLE!();
    if WriteConsoleA(
        msvcrt_output_console(),
        str as *const _,
        len as DWORD,
        &mut count,
        core::ptr::null_mut(),
    ) != 0
        && count == len as DWORD
    {
        retval = 0;
    }
    UNLOCK_CONSOLE!();
    retval
}

#[no_mangle]
pub unsafe extern "C" fn _cputws(str: *const wchar_t) -> i32 {
    let mut count: DWORD = 0;
    let len: i32;
    let mut retval: i32 = -1;

    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return -1;
    }
    len = wcslen(str) as i32;

    LOCK_CONSOLE!();
    if WriteConsoleW(
        msvcrt_output_console(),
        str as *const core::ffi::c_void,
        len as DWORD,
        &mut count,
        core::ptr::null_mut(),
    ) != 0
        && count == len as DWORD
    {
        retval = 0;
    }
    UNLOCK_CONSOLE!();
    retval
}
unsafe extern "C" fn handle_enhanced_keys(ir: *mut INPUT_RECORD, ch1: *mut core::ffi::c_uchar, ch2: *mut core::ffi::c_uchar) -> BOOL {
    let mut i: core::ffi::c_int = 0;
    while (i as usize) < ARRAY_SIZE!(enh_map) {
        if (*ir).Event.KeyEvent.wVirtualScanCode == enh_map[i as usize].vk {
            let idx: core::ffi::c_uint;

            if (*ir).Event.KeyEvent.dwControlKeyState & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0 {
                idx = ALT_CHAR;
            } else if (*ir).Event.KeyEvent.dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0 {
                idx = CTRL_CHAR;
            } else if (*ir).Event.KeyEvent.dwControlKeyState & SHIFT_PRESSED != 0 {
                idx = SHIFT_CHAR;
            } else {
                idx = NORMAL_CHAR;
            }

            *ch1 = enh_map[i as usize].ch[idx as usize][0];
            *ch2 = enh_map[i as usize].ch[idx as usize][1];
            return TRUE as BOOL;
        }
        i += 1;
    }

    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn _getch_nolock() -> core::ffi::c_int {
    let mut retval: core::ffi::c_int = EOF;

    if __MSVCRT_console_buffer != EOF {
        retval = __MSVCRT_console_buffer;
        __MSVCRT_console_buffer = EOF;
    } else {
        let mut ir: INPUT_RECORD = core::mem::zeroed();
        let mut count: DWORD = 0;
        let mut mode: DWORD = 0;

        GetConsoleMode(msvcrt_input_console(), &mut mode);
        if mode != 0 {
            SetConsoleMode(msvcrt_input_console(), 0);
        }

        loop {
            if ReadConsoleInputA(msvcrt_input_console(), &mut ir, 1, &mut count) != 0 {
                if ir.EventType == KEY_EVENT as u16
                    && ir.Event.KeyEvent.bKeyDown != 0
                {
                    let mut ch1: core::ffi::c_uchar = 0;
                    let mut ch2: core::ffi::c_uchar = 0;

                    if ir.Event.KeyEvent.uChar.AsciiChar != 0 {
                        retval = ir.Event.KeyEvent.uChar.AsciiChar as core::ffi::c_int;
                        break;
                    }

                    if handle_enhanced_keys(&mut ir, &mut ch1, &mut ch2) != 0 {
                        retval = ch1 as core::ffi::c_int;
                        __MSVCRT_console_buffer = ch2 as core::ffi::c_int;
                        break;
                    }
                }
            } else {
                break;
            }

            if 1 == 0 {
                break;
            }
        }

        if mode != 0 {
            SetConsoleMode(msvcrt_input_console(), mode);
        }
    }

    retval
}

#[no_mangle]
pub unsafe extern "C" fn _getch() -> core::ffi::c_int {
    let ret: core::ffi::c_int;

    LOCK_CONSOLE!();
    ret = _getch_nolock();
    UNLOCK_CONSOLE!();
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn _getwch_nolock() -> wchar_t {
    let mut retval: wchar_t = WEOF;

    if __MSVCRT_console_buffer_w != WEOF {
        retval = __MSVCRT_console_buffer_w;
        __MSVCRT_console_buffer_w = WEOF;
    } else {
        let mut ir: INPUT_RECORD = core::mem::zeroed();
        let mut count: DWORD = 0;
        let mut mode: DWORD = 0;

        GetConsoleMode(msvcrt_input_console(), &mut mode);
        if mode != 0 {
            SetConsoleMode(msvcrt_input_console(), 0);
        }

        loop {
            if ReadConsoleInputW(msvcrt_input_console(), &mut ir, 1, &mut count) != 0 {
                if ir.EventType as DWORD == KEY_EVENT && ir.Event.KeyEvent.bKeyDown != 0 {
                    let mut ch1: core::ffi::c_uchar = 0;
                    let mut ch2: core::ffi::c_uchar = 0;

                    if ir.Event.KeyEvent.uChar.UnicodeChar != 0 {
                        retval = ir.Event.KeyEvent.uChar.UnicodeChar as wchar_t;
                        break;
                    }

                    if handle_enhanced_keys(&mut ir, &mut ch1, &mut ch2) != 0 {
                        retval = ch1 as wchar_t;
                        __MSVCRT_console_buffer_w = ch2 as wchar_t;
                        break;
                    }
                }
            } else {
                break;
            }

            if !(1 != 0) {
                break;
            }
        }

        if mode != 0 {
            SetConsoleMode(msvcrt_input_console(), mode);
        }
    }
    retval
}

#[no_mangle]
pub unsafe extern "C" fn _getwch() -> wchar_t {
    let ret: wchar_t;

    LOCK_CONSOLE!();
    ret = _getwch_nolock();
    UNLOCK_CONSOLE!();
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _putch_nolock(mut c: c_int) -> c_int {
    let mut count: DWORD = 0;
    if WriteConsoleA(
        msvcrt_output_console(),
        core::ptr::addr_of_mut!(c) as *const _,
        1,
        &mut count,
        core::ptr::null_mut(),
    ) != 0
        && count == 1
    {
        return c;
    }
    EOF
}

#[no_mangle]
pub unsafe extern "C" fn _putch(mut c: c_int) -> c_int {
    LOCK_CONSOLE!();
    c = _putch_nolock(c);
    UNLOCK_CONSOLE!();
    return c;
}

#[no_mangle]
pub unsafe extern "C" fn _putwch_nolock(c: wchar_t) -> wchar_t {
    let mut count: DWORD = 0;
    let mut c = c;
    if WriteConsoleW(
        msvcrt_output_console(),
        core::ptr::addr_of_mut!(c) as *const _,
        1,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) != 0
        && count == 1
    {
        return c;
    }
    WEOF
}

#[no_mangle]
pub unsafe extern "C" fn _putwch(mut c: wchar_t) -> wchar_t {
    LOCK_CONSOLE!();
    c = _putwch_nolock(c);
    UNLOCK_CONSOLE!();
    return c;
}

#[no_mangle]
pub unsafe extern "C" fn _getche_nolock() -> core::ffi::c_int {
    let mut retval: core::ffi::c_int;
    retval = _getch_nolock();
    if retval != EOF {
        retval = _putch_nolock(retval);
    }
    retval
}

#[no_mangle]
pub unsafe extern "C" fn _getche() -> core::ffi::c_int {
    let ret: core::ffi::c_int;

    LOCK_CONSOLE!();
    ret = _getche_nolock();
    UNLOCK_CONSOLE!();
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _getwche_nolock() -> wchar_t {
    let wch: wchar_t;
    wch = _getch_nolock() as wchar_t;
    if wch == WEOF as wchar_t {
        return wch;
    }
    _putwch_nolock(wch)
}

#[no_mangle]
pub unsafe extern "C" fn _getwche() -> wchar_t {
    let ret: wchar_t;

    LOCK_CONSOLE!();
    ret = _getwche_nolock();
    UNLOCK_CONSOLE!();
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _cgets(str: *mut c_char) -> *mut c_char {
    let mut buf = str.offset(2);
    let mut got: DWORD = 0;
    let mut conmode: DWORD = 0;

    *str.offset(1) = 0;
    LOCK_CONSOLE!();
    GetConsoleMode(msvcrt_input_console(), &mut conmode);
    SetConsoleMode(msvcrt_input_console(), ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);

    if ReadConsoleA(msvcrt_input_console(), buf as *mut _, *str.offset(0) as DWORD, &mut got, core::ptr::null_mut()) != 0 {
        if *buf.offset((got - 2) as isize) == b'\r' as c_char {
            *buf.offset((got - 2) as isize) = 0;
            *str.offset(1) = (got - 2) as c_char;
        } else if got == 1 && *buf.offset((got - 1) as isize) == b'\n' as c_char {
            *buf.offset(0) = 0;
            *str.offset(1) = 0;
        } else if got == *str.offset(0) as DWORD && *buf.offset((got - 1) as isize) == b'\r' as c_char {
            *buf.offset((got - 1) as isize) = 0;
            *str.offset(1) = (got - 1) as c_char;
        } else {
            *str.offset(1) = got as c_char;
        }
    } else {
        buf = core::ptr::null_mut();
    }
    SetConsoleMode(msvcrt_input_console(), conmode);
    UNLOCK_CONSOLE!();
    buf
}

#[no_mangle]
pub unsafe extern "C" fn _ungetch_nolock(c: core::ffi::c_int) -> core::ffi::c_int {
    let mut retval: core::ffi::c_int = EOF;
    if c != EOF && __MSVCRT_console_buffer == EOF {
        __MSVCRT_console_buffer = c;
        retval = c;
    }
    retval
}

#[no_mangle]
pub unsafe extern "C" fn _ungetch(mut c: core::ffi::c_int) -> core::ffi::c_int {
    LOCK_CONSOLE!();
    c = _ungetch_nolock(c);
    UNLOCK_CONSOLE!();
    c
}
#[no_mangle]
pub unsafe extern "C" fn _ungetwch_nolock(c: wchar_t) -> wchar_t {
    let mut retval: wchar_t = WEOF;
    if c != WEOF && __MSVCRT_console_buffer_w == WEOF {
        __MSVCRT_console_buffer_w = c;
        retval = c;
    }
    retval
}

#[no_mangle]
pub unsafe extern "C" fn _ungetwch(mut c: wchar_t) -> wchar_t {
    LOCK_CONSOLE!();
    c = _ungetwch_nolock(c);
    UNLOCK_CONSOLE!();
    c
}

#[no_mangle]
pub unsafe extern "C" fn _kbhit() -> core::ffi::c_int {
    let mut retval = 0;

    LOCK_CONSOLE!();
    if __MSVCRT_console_buffer != EOF {
        retval = 1;
    } else {
        let mut ir: *mut INPUT_RECORD = core::ptr::null_mut();
        let mut count: DWORD = 0;
        let mut i: DWORD;

        GetNumberOfConsoleInputEvents(msvcrt_input_console(), &mut count);

        if count != 0 && {
            ir = malloc(
                count as usize * core::mem::size_of::<INPUT_RECORD>(),
            ) as *mut INPUT_RECORD;
            !ir.is_null()
        } && PeekConsoleInputA(msvcrt_input_console(), ir, count, &mut count) != 0
        {
            i = 0;
            while i < count {
                if (*ir.offset(i as isize)).EventType == KEY_EVENT as WORD
                    && (*ir.offset(i as isize)).Event.KeyEvent.bKeyDown != 0
                    && (*ir.offset(i as isize)).Event.KeyEvent.uChar.AsciiChar != 0
                {
                    retval = 1;
                    break;
                }
                i += 1;
            }
        }
        free(ir as *mut _);
    }
    UNLOCK_CONSOLE!();
    retval
}
unsafe extern "C" fn puts_clbk_console_a(ctx: *mut core::ffi::c_void, mut len: i32, str_: *const u8) -> i32 {
    LOCK_CONSOLE!();
    if WriteConsoleA(msvcrt_output_console(), str_ as *const core::ffi::c_void, len as u32, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        len = -1;
    }
    UNLOCK_CONSOLE!();
    len
}
unsafe extern "C" fn puts_clbk_console_w(ctx: *mut core::ffi::c_void, mut len: i32, str: *const wchar_t) -> i32 {
    LOCK_CONSOLE!();
    if WriteConsoleW(msvcrt_output_console(), str as *const core::ffi::c_void, len as u32, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        len = -1;
    }
    UNLOCK_CONSOLE!();
    len
}

#[no_mangle]
pub unsafe extern "C" fn _vcprintf_l(format: *const core::ffi::c_char, locale: _locale_t, mut valist: va_list) -> core::ffi::c_int {
    pf_printf_a(puts_clbk_console_a, core::ptr::null_mut(), format as *const u8, locale as *mut core::ffi::c_void, 0, arg_clbk_valist, core::ptr::null_mut(), core::ptr::addr_of_mut!(valist) as *mut *mut u8)
}

#[no_mangle]
pub unsafe extern "C" fn _vcprintf(format: *const core::ffi::c_char, mut valist: va_list) -> core::ffi::c_int {
    pf_printf_a(
        puts_clbk_console_a,
        core::ptr::null_mut(),
        format as *const u8,
        core::ptr::null_mut(),
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(valist) as *mut *mut u8,
    )
}



#[no_mangle]
pub unsafe extern "C" fn _vcwprintf_l(format: *const wchar_t, locale: _locale_t, valist: va_list) -> core::ffi::c_int {
    let mut valist = valist;
    pf_printf_w(puts_clbk_console_w, core::ptr::null_mut(), format, locale as *mut core::ffi::c_void, 0, arg_clbk_valist, core::ptr::null_mut(), core::ptr::addr_of_mut!(valist) as *mut *mut u8)
}

#[no_mangle]
pub unsafe extern "C" fn _vcwprintf(format: *const wchar_t, valist: va_list) -> core::ffi::c_int {
    let mut valist = valist;
    pf_printf_w(
        puts_clbk_console_w,
        core::ptr::null_mut(),
        format,
        core::ptr::null_mut(),
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(valist) as *mut *mut u8,
    )
}

// `_cprintf`, `_cprintf_l`, `_cwprintf` and `_cwprintf_l` are NOT here, and their absence is
// a language limit rather than a translation gap: they are WINAPIV `...` entry points, and
// stable Rust cannot DEFINE a C-variadic function (`c_variadic` is nightly; this project is
// pinned to stable). They live in dll/win32/msvcrt/rust_console_varargs_shim.c as four
// `va_start` trampolines over the `_vcprintf`/`_vcprintf_l`/`_vcwprintf`/`_vcwprintf_l` twins
// above, which ARE Rust — RECEIVING an already-started `va_list` is fine on every Windows
// target we build. The shim is compiled into the msvcrt MODULE, so its objects resolve ahead
// of console.o and first-match still reaches Rust for the twins.
//
// This is the fifth instance of the same split (wcs.c, scanf.c, file.c, process.c), so the
// count of console.c exports living in Rust is 25 of 29, with the other 4 being these.

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll console functions
//  *
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * Note: init and free don't need MT locking since they are called at DLL
//  * (de)attachment time, which is synchronised for us
//  */
//
// #include "msvcrt.h"
// #include "winnls.h"
// #include "wincon.h"
// #include "mtdll.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// /* MT */
// #define LOCK_CONSOLE   _lock(_CONIO_LOCK)
// #define UNLOCK_CONSOLE _unlock(_CONIO_LOCK)
//
// static HANDLE MSVCRT_console_in;
// static HANDLE MSVCRT_console_out;
// static int __MSVCRT_console_buffer = EOF;
// static wchar_t __MSVCRT_console_buffer_w = WEOF;
//
// /* INTERNAL: Initialise console handles, _CONIO_LOCK must be held */
// static HANDLE msvcrt_input_console(void)
// {
//   if (!MSVCRT_console_in)
//   {
//     MSVCRT_console_in = CreateFileA("CONIN$", GENERIC_WRITE|GENERIC_READ,
//                                     FILE_SHARE_WRITE|FILE_SHARE_READ,
//                                     NULL, OPEN_EXISTING, 0, NULL);
//     if (MSVCRT_console_in == INVALID_HANDLE_VALUE)
//       WARN("Input console handle initialization failed!\n");
//   }
//   return MSVCRT_console_in;
// }
//
// static HANDLE msvcrt_output_console(void)
// {
//   if (!MSVCRT_console_out)
//   {
//     MSVCRT_console_out = CreateFileA("CONOUT$", GENERIC_WRITE, FILE_SHARE_WRITE,
//                                      NULL, OPEN_EXISTING, 0, NULL);
//     if (MSVCRT_console_out == INVALID_HANDLE_VALUE)
//       WARN("Output console handle initialization failed!\n");
//   }
//   return MSVCRT_console_out;
// }
//
// /* INTERNAL: Free console handles */
// void msvcrt_free_console(void)
// {
//   TRACE(":Closing console handles\n");
//   CloseHandle(MSVCRT_console_in);
//   CloseHandle(MSVCRT_console_out);
// }
//
// /*********************************************************************
//  *		_cputs (MSVCRT.@)
//  */
// int CDECL _cputs(const char* str)
// {
//   DWORD count;
//   int len, retval = -1;
//
//   if (!MSVCRT_CHECK_PMT(str != NULL)) return -1;
//   len = strlen(str);
//
//   LOCK_CONSOLE;
//   if (WriteConsoleA(msvcrt_output_console(), str, len, &count, NULL)
//       && count == len)
//     retval = 0;
//   UNLOCK_CONSOLE;
//   return retval;
// }
//
// /*********************************************************************
//  *		_cputws (MSVCRT.@)
//  */
// int CDECL _cputws(const wchar_t* str)
// {
//   DWORD count;
//   int len, retval = -1;
//
//   if (!MSVCRT_CHECK_PMT(str != NULL)) return -1;
//   len = wcslen(str);
//
//   LOCK_CONSOLE;
//   if (WriteConsoleW(msvcrt_output_console(), str, len, &count, NULL)
//       && count == len)
//     retval = 0;
//   UNLOCK_CONSOLE;
//   return retval;
// }
//
// #define NORMAL_CHAR     0
// #define ALT_CHAR        1
// #define CTRL_CHAR       2
// #define SHIFT_CHAR      3
//
// static const struct {unsigned short vk; unsigned char ch[4][2];} enh_map[] = {
//     {0x47, {{0xE0, 0x47}, {0x00, 0x97}, {0xE0, 0x77}, {0xE0, 0x47}}},
//     {0x48, {{0xE0, 0x48}, {0x00, 0x98}, {0xE0, 0x8D}, {0xE0, 0x48}}},
//     {0x49, {{0xE0, 0x49}, {0x00, 0x99}, {0xE0, 0x86}, {0xE0, 0x49}}},
//     {0x4B, {{0xE0, 0x4B}, {0x00, 0x9B}, {0xE0, 0x73}, {0xE0, 0x4B}}},
//     {0x4D, {{0xE0, 0x4D}, {0x00, 0x9D}, {0xE0, 0x74}, {0xE0, 0x4D}}},
//     {0x4F, {{0xE0, 0x4F}, {0x00, 0x9F}, {0xE0, 0x75}, {0xE0, 0x4F}}},
//     {0x50, {{0xE0, 0x50}, {0x00, 0xA0}, {0xE0, 0x91}, {0xE0, 0x50}}},
//     {0x51, {{0xE0, 0x51}, {0x00, 0xA1}, {0xE0, 0x76}, {0xE0, 0x51}}},
//     {0x52, {{0xE0, 0x52}, {0x00, 0xA2}, {0xE0, 0x92}, {0xE0, 0x52}}},
//     {0x53, {{0xE0, 0x53}, {0x00, 0xA3}, {0xE0, 0x93}, {0xE0, 0x53}}},
// };
//
// static BOOL handle_enhanced_keys(INPUT_RECORD *ir, unsigned char *ch1, unsigned char *ch2)
// {
//     int i;
//
//     for (i = 0; i < ARRAY_SIZE(enh_map); i++)
//     {
//         if (ir->Event.KeyEvent.wVirtualScanCode == enh_map[i].vk)
//         {
//             unsigned idx;
//
//             if (ir->Event.KeyEvent.dwControlKeyState & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED))
//                 idx = ALT_CHAR;
//             else if (ir->Event.KeyEvent.dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) )
//                 idx = CTRL_CHAR;
//             else if (ir->Event.KeyEvent.dwControlKeyState & SHIFT_PRESSED)
//                 idx = SHIFT_CHAR;
//             else
//                 idx = NORMAL_CHAR;
//
//             *ch1 = enh_map[i].ch[idx][0];
//             *ch2 = enh_map[i].ch[idx][1];
//             return TRUE;
//         }
//     }
//
//     WARN("Unmapped char keyState=%lx vk=%x\n",
//             ir->Event.KeyEvent.dwControlKeyState, ir->Event.KeyEvent.wVirtualScanCode);
//     return FALSE;
// }
//
// /*********************************************************************
//  *		_getch_nolock (MSVCR80.@)
//  */
// int CDECL _getch_nolock(void)
// {
//   int retval = EOF;
//
//   if (__MSVCRT_console_buffer != EOF)
//   {
//     retval = __MSVCRT_console_buffer;
//     __MSVCRT_console_buffer = EOF;
//   }
//   else
//   {
//     INPUT_RECORD ir;
//     DWORD count;
//     DWORD mode = 0;
//
//     GetConsoleMode(msvcrt_input_console(), &mode);
//     if(mode)
//       SetConsoleMode(msvcrt_input_console(), 0);
//
//     do {
//       if (ReadConsoleInputA(msvcrt_input_console(), &ir, 1, &count))
//       {
//         /* Only interested in ASCII chars */
//         if (ir.EventType == KEY_EVENT &&
//             ir.Event.KeyEvent.bKeyDown)
//         {
//             unsigned char ch1, ch2;
//
//             if (ir.Event.KeyEvent.uChar.AsciiChar)
//             {
//                 retval = ir.Event.KeyEvent.uChar.AsciiChar;
//                 break;
//             }
//
//             if (handle_enhanced_keys(&ir, &ch1, &ch2))
//             {
//                 retval = ch1;
//                 __MSVCRT_console_buffer = ch2;
//                 break;
//             }
//         }
//       }
//       else
//         break;
//     } while(1);
//     if (mode)
//       SetConsoleMode(msvcrt_input_console(), mode);
//   }
//   return retval;
// }
//
// /*********************************************************************
//  *		_getch (MSVCRT.@)
//  */
// int CDECL _getch(void)
// {
//     int ret;
//
//     LOCK_CONSOLE;
//     ret = _getch_nolock();
//     UNLOCK_CONSOLE;
//     return ret;
// }
//
// /*********************************************************************
//  *		_getwch_nolock (MSVCR80.@)
//  */
// wchar_t CDECL _getwch_nolock(void)
// {
//     wchar_t retval = WEOF;
//
//     if (__MSVCRT_console_buffer_w != WEOF)
//     {
//         retval = __MSVCRT_console_buffer_w;
//         __MSVCRT_console_buffer_w = WEOF;
//     }
//     else
//     {
//         INPUT_RECORD ir;
//         DWORD count;
//         DWORD mode = 0;
//
//         GetConsoleMode(msvcrt_input_console(), &mode);
//         if(mode)
//             SetConsoleMode(msvcrt_input_console(), 0);
//
//         do {
//             if (ReadConsoleInputW(msvcrt_input_console(), &ir, 1, &count))
//             {
//                 /* Only interested in ASCII chars */
//                 if (ir.EventType == KEY_EVENT &&
//                         ir.Event.KeyEvent.bKeyDown)
//                 {
//                     unsigned char ch1, ch2;
//
//                     if (ir.Event.KeyEvent.uChar.UnicodeChar)
//                     {
//                         retval = ir.Event.KeyEvent.uChar.UnicodeChar;
//                         break;
//                     }
//
//                     if (handle_enhanced_keys(&ir, &ch1, &ch2))
//                     {
//                         retval = ch1;
//                         __MSVCRT_console_buffer_w = ch2;
//                         break;
//                     }
//                 }
//             }
//             else
//                 break;
//         } while(1);
//         if (mode)
//             SetConsoleMode(msvcrt_input_console(), mode);
//     }
//     return retval;
// }
//
// /*********************************************************************
//  *              _getwch (MSVCRT.@)
//  */
// wchar_t CDECL _getwch(void)
// {
//     wchar_t ret;
//
//     LOCK_CONSOLE;
//     ret = _getwch_nolock();
//     UNLOCK_CONSOLE;
//     return ret;
// }
//
// /*********************************************************************
//  *		_putch_nolock (MSVCR80.@)
//  */
// int CDECL _putch_nolock(int c)
// {
//   DWORD count;
//   if (WriteConsoleA(msvcrt_output_console(), &c, 1, &count, NULL) && count == 1)
//     return c;
//   return EOF;
// }
//
// /*********************************************************************
//  *		_putch (MSVCRT.@)
//  */
// int CDECL _putch(int c)
// {
//     LOCK_CONSOLE;
//     c = _putch_nolock(c);
//     UNLOCK_CONSOLE;
//     return c;
// }
//
// /*********************************************************************
//  *		_putwch_nolock (MSVCR80.@)
//  */
// wchar_t CDECL _putwch_nolock(wchar_t c)
// {
//     DWORD count;
//     if (WriteConsoleW(msvcrt_output_console(), &c, 1, &count, NULL) && count==1)
//         return c;
//     return WEOF;
// }
//
// /*********************************************************************
//  *		_putwch (MSVCRT.@)
//  */
// wchar_t CDECL _putwch(wchar_t c)
// {
//     LOCK_CONSOLE;
//     c = _putwch_nolock(c);
//     UNLOCK_CONSOLE;
//     return c;
// }
//
// /*********************************************************************
//  *		_getche_nolock (MSVCR80.@)
//  */
// int CDECL _getche_nolock(void)
// {
//   int retval;
//   retval = _getch_nolock();
//   if (retval != EOF)
//     retval = _putch_nolock(retval);
//   return retval;
// }
//
// /*********************************************************************
//  *		_getche (MSVCRT.@)
//  */
// int CDECL _getche(void)
// {
//     int ret;
//
//     LOCK_CONSOLE;
//     ret = _getche_nolock();
//     UNLOCK_CONSOLE;
//     return ret;
// }
//
// /*********************************************************************
//  *              _getwche_nolock (MSVCR80.@)
//  */
// wchar_t CDECL _getwche_nolock(void)
// {
//     wchar_t wch;
//     wch = _getch_nolock();
//     if (wch == WEOF)
//         return wch;
//     return _putwch_nolock(wch);
// }
//
// /*********************************************************************
//  *              _getwche (MSVCRT.@)
//  */
// wchar_t CDECL _getwche(void)
// {
//     wchar_t ret;
//
//     LOCK_CONSOLE;
//     ret = _getwche_nolock();
//     UNLOCK_CONSOLE;
//     return ret;
// }
//
// /*********************************************************************
//  *		_cgets (MSVCRT.@)
//  */
// char*  _cgets(char* str)
// {
//   char *buf = str + 2;
//   DWORD got;
//   DWORD conmode = 0;
//
//   TRACE("(%p)\n", str);
//   str[1] = 0; /* Length */
//   LOCK_CONSOLE;
//   GetConsoleMode(msvcrt_input_console(), &conmode);
//   SetConsoleMode(msvcrt_input_console(), ENABLE_LINE_INPUT|ENABLE_ECHO_INPUT|ENABLE_PROCESSED_INPUT);
//
//   if(ReadConsoleA(msvcrt_input_console(), buf, str[0], &got, NULL)) {
//     if(buf[got-2] == '\r') {
//       buf[got-2] = 0;
//       str[1] = got-2;
//     }
//     else if(got == 1 && buf[got-1] == '\n') {
//       buf[0] = 0;
//       str[1] = 0;
//     }
//     else if(got == str[0] && buf[got-1] == '\r') {
//       buf[got-1] = 0;
//       str[1] = got-1;
//     }
//     else
//       str[1] = got;
//   }
//   else
//     buf = NULL;
//   SetConsoleMode(msvcrt_input_console(), conmode);
//   UNLOCK_CONSOLE;
//   return buf;
// }
//
// /*********************************************************************
//  *		_ungetch_nolock (MSVCRT.@)
//  */
// int CDECL _ungetch_nolock(int c)
// {
//   int retval = EOF;
//   if (c != EOF && __MSVCRT_console_buffer == EOF)
//     retval = __MSVCRT_console_buffer = c;
//   return retval;
// }
//
// /*********************************************************************
//  *		_ungetch (MSVCRT.@)
//  */
// int CDECL _ungetch(int c)
// {
//     LOCK_CONSOLE;
//     c = _ungetch_nolock(c);
//     UNLOCK_CONSOLE;
//     return c;
// }
//
// /*********************************************************************
//  *              _ungetwch_nolock (MSVCR80.@)
//  */
// wchar_t CDECL _ungetwch_nolock(wchar_t c)
// {
//     wchar_t retval = WEOF;
//     if (c != WEOF && __MSVCRT_console_buffer_w == WEOF)
//         retval = __MSVCRT_console_buffer_w = c;
//     return retval;
// }
//
// /*********************************************************************
//  *              _ungetwch (MSVCRT.@)
//  */
// wchar_t CDECL _ungetwch(wchar_t c)
// {
//     LOCK_CONSOLE;
//     c = _ungetwch_nolock(c);
//     UNLOCK_CONSOLE;
//     return c;
// }
//
// /*********************************************************************
//  *		_kbhit (MSVCRT.@)
//  */
// int CDECL _kbhit(void)
// {
//   int retval = 0;
//
//   LOCK_CONSOLE;
//   if (__MSVCRT_console_buffer != EOF)
//     retval = 1;
//   else
//   {
//     /* FIXME: There has to be a faster way than this in Win32.. */
//     INPUT_RECORD *ir = NULL;
//     DWORD count = 0, i;
//
//     GetNumberOfConsoleInputEvents(msvcrt_input_console(), &count);
//
//     if (count && (ir = malloc(count * sizeof(INPUT_RECORD))) &&
//         PeekConsoleInputA(msvcrt_input_console(), ir, count, &count))
//       for(i = 0; i < count; i++)
//       {
//         if (ir[i].EventType == KEY_EVENT &&
//             ir[i].Event.KeyEvent.bKeyDown &&
//             ir[i].Event.KeyEvent.uChar.AsciiChar)
//         {
//           retval = 1;
//           break;
//         }
//       }
//     free(ir);
//   }
//   UNLOCK_CONSOLE;
//   return retval;
// }
//
// static int puts_clbk_console_a(void *ctx, int len, const char *str)
// {
//     LOCK_CONSOLE;
//     if(!WriteConsoleA(msvcrt_output_console(), str, len, NULL, NULL))
//         len = -1;
//     UNLOCK_CONSOLE;
//     return len;
// }
//
// static int puts_clbk_console_w(void *ctx, int len, const wchar_t *str)
// {
//     LOCK_CONSOLE;
//     if(!WriteConsoleW(msvcrt_output_console(), str, len, NULL, NULL))
//         len = -1;
//     UNLOCK_CONSOLE;
//     return len;
// }
//
// /*********************************************************************
//  *		_vcprintf_l (MSVCRT.@)
//  */
// int CDECL _vcprintf_l(const char* format, _locale_t locale, va_list valist)
// {
//     return pf_printf_a(puts_clbk_console_a, NULL, format, locale, 0, arg_clbk_valist, NULL, &valist);
// }
//
// /*********************************************************************
//  *		_vcprintf (MSVCRT.@)
//  */
// int CDECL _vcprintf(const char* format, va_list valist)
// {
//     return pf_printf_a(puts_clbk_console_a, NULL, format, NULL, 0, arg_clbk_valist, NULL, &valist);
// }
//
// /*********************************************************************
//  *		_cprintf_l (MSVCRT.@)
//  */
// int __cdecl _cprintf_l(const char* format, _locale_t locale, ...)
// {
//     int retval;
//     va_list valist;
//
//     va_start(valist, locale);
//     retval = _vcprintf_l(format, locale, valist);
//     va_end(valist);
//
//     return retval;
// }
//
// /*********************************************************************
//  *		_cprintf (MSVCRT.@)
//  */
// int __cdecl _cprintf(const char* format, ...)
// {
//   int retval;
//   va_list valist;
//
//   va_start( valist, format );
//   retval = _vcprintf(format, valist);
//   va_end(valist);
//
//   return retval;
// }
//
// /*********************************************************************
//  *		_vcwprintf_l (MSVCRT.@)
//  */
// int CDECL _vcwprintf_l(const wchar_t* format, _locale_t locale, va_list valist)
// {
//     return pf_printf_w(puts_clbk_console_w, NULL, format, locale, 0, arg_clbk_valist, NULL, &valist);
// }
//
// /*********************************************************************
//  *		_vcwprintf (MSVCRT.@)
//  */
// int CDECL _vcwprintf(const wchar_t* format, va_list valist)
// {
//     return pf_printf_w(puts_clbk_console_w, NULL, format, NULL, 0, arg_clbk_valist, NULL, &valist);
// }
//
// /*********************************************************************
//  *		_cwprintf_l (MSVCRT.@)
//  */
// int __cdecl _cwprintf_l(const wchar_t* format, _locale_t locale, ...)
// {
//     int retval;
//     va_list valist;
//
//     va_start(valist, locale);
//     retval = _vcwprintf_l(format, locale, valist);
//     va_end(valist);
//
//     return retval;
// }
//
// /*********************************************************************
//  *		_cwprintf (MSVCRT.@)
//  */
// int __cdecl _cwprintf(const wchar_t* format, ...)
// {
//   int retval;
//   va_list valist;
//
//   va_start( valist, format );
//   retval = _vcwprintf(format, valist);
//   va_end(valist);
//
//   return retval;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
