use crate::prelude::*;
// NOT `use msvcrt_sys::*;` as well. prelude.rs already re-exports the whole island
// (`pub use msvcrt_sys::*`), so a second glob imports every name TWICE from two different
// roots and every use of an overlapping one is E0659 "ambiguous" — this file arrived with
// the second glob and ten `WEOF`/`FALSE` uses failed exactly that way. The prelude also
// re-types a few island constants to the shape their uses need, and an explicit item beats
// a glob, so going through the prelude keeps those choices. One door.

// CRT siblings ALREADY Rust in this crate, imported so rustc checks the signatures.
// `_cputs` is console.c's, which `abort` calls; it rides the same `conio-exit` feature as
// this module, so the import cannot dangle.
use crate::msvcrt::console::_cputs;


extern "C" {
    // THIS tree's `sdk/include/crt/stdio.h:92,100` — `#define _iob __iob_func()` and
    // `#define stderr (&_iob[2])`, so exit.c's `fwprintf(stderr, ...)` is a CALL, not a
    // reference to an array. Declared `extern` rather than imported from `file.rs` (where
    // the Rust body lives) because that module rides the `stdio` feature and this one rides
    // `conio-exit`: an import would couple the two gates for a symbol that is exported from
    // the DLL anyway. Same reasoning, and same declaration, as `scanf.rs`.
    fn __iob_func() -> *mut FILE;
}
macro_rules! stderr {
    () => {
        __iob_func().add(2)
    };
}
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `_EXIT_LOCK1` is NOT transcribed here — it comes from the msvcrt-sys island (mtdll.h),
// because this file is why it now does. The translation arrived with `const _EXIT_LOCK1 = 8`,
// which is `_CONIO_LOCK`'s value from two lines earlier in the same table: a constant
// PERMUTED within its own header rather than invented, so the seal's closed-SET invariant
// was satisfied and nothing downstream could object — `_lock(8)` compiles, links and runs.
// The effect would have been `_cexit`/`_onexit`/`atexit` serializing on the CONSOLE lock,
// racing each other while deadlocking against console.c's `LOCK_CONSOLE`. The island now
// carries both names, so the value is derived and this class cannot recur here.

// exit.c:32-33 — `#define LOCK_EXIT _lock(_EXIT_LOCK1)` / `UNLOCK_EXIT _unlock(...)`.
// `macro_rules!`, not a helper `fn`: an out-of-line wrapper for a C macro is an optimization
// barrier that has uncloaked latent ABI mismatches into `ud2` before now.
macro_rules! LOCK_EXIT {
    () => {
        _lock(_EXIT_LOCK1 as core::ffi::c_int)
    };
}
macro_rules! UNLOCK_EXIT {
    () => {
        _unlock(_EXIT_LOCK1 as core::ffi::c_int)
    };
}

pub static mut purecall_handler: _purecall_handler = None;

pub static mut MSVCRT_atexit_table: _onexit_table_t = unsafe { core::mem::zeroed() };

pub type _tls_callback_type = ::core::option::Option<
    unsafe extern "system" fn(arg1: *mut core::ffi::c_void, arg2: ULONG, arg3: *mut core::ffi::c_void),
>;

pub static mut tls_atexit_callback: _tls_callback_type = None;

pub static mut MSVCRT_onexit_cs_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { core::ptr::addr_of!(MSVCRT_onexit_cs) as *mut _ },
    ProcessLocksList: _LIST_ENTRY {
        Flink: unsafe { core::ptr::addr_of!(MSVCRT_onexit_cs_debug.ProcessLocksList) as *mut _ },
        Blink: unsafe { core::ptr::addr_of!(MSVCRT_onexit_cs_debug.ProcessLocksList) as *mut _ },
    },
    EntryCount: 0,
    ContentionCount: 0,
    // The C's trailing `{ (DWORD_PTR)(__FILE__ ": MSVCRT_onexit_cs") }` initializes the
    // FIRST member of winnt.h's anonymous union, `WineDebugString` — NOT the flat
    // `Flags`/`CreatorBackTraceIndexHigh`/`Identifier`/`Spare` scalars the translation
    // invented, none of which exist as siblings at this level. Only the derived layout says
    // so; the invented shape would have compiled and put a string pointer into a flags word.
    //
    // Zeroed rather than carrying the label, and this is the file's one deliberate
    // divergence from the C: Rust has no stable way to cast a pointer to an integer during
    // CONST evaluation, so `"..." as DWORD_PTR` in a static initializer is rejected outright
    // (it is not a lint — there is no spelling that works). The field is a debugger-only
    // annotation: this critical section is statically initialized, so ntdll never runs the
    // path that would allocate or read a debug record for it, and nothing in the OS branches
    // on the value. Losing the string costs a name in a debugger and nothing else.
    __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
};

pub static mut MSVCRT_onexit_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { core::ptr::addr_of!(MSVCRT_onexit_cs_debug) as *mut _ },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

pub static mut MSVCRT_abort_behavior: core::ffi::c_uint = _WRITE_ABORT_MSG | _CALL_REPORTFAULT;

pub static mut MSVCRT_error_mode: core::ffi::c_int = _OUT_TO_DEFAULT as core::ffi::c_int;

/// `_aexit_rtn` is NOT file-static, which is the whole reason this needs `#[no_mangle]`:
/// `exit.c.obj` exports it as a global `D` and `msvcrt.spec:321` re-exports it from the DLL
/// (`@ extern _aexit_rtn`). The translation arrived without the attribute, i.e. as a private
/// Rust static — which compiles, links, and silently leaves the C `exit.c.obj` as the only
/// provider of the name, dragging that object back in and duplicating every other symbol in
/// this file. `llvm-nm --defined-only` on the object is what caught it; the source's `static`
/// keywords are not the authority on what a TU exports.
#[no_mangle]
pub static mut _aexit_rtn: Option<unsafe extern "C" fn(core::ffi::c_int)> = unsafe {
    // C: `void (*CDECL _aexit_rtn)(int) = _exit;`. In C the slot's type is plainly
    // `void(*)(int)` and `noreturn` is an ATTRIBUTE on `_exit`, not part of its type, so the
    // assignment needs no conversion. Rust puts divergence IN the type, so `fn(c_int) -> !`
    // and `fn(c_int)` are different types and no `as` cast bridges them.
    //
    // The slot stays `fn(c_int)` — the honest C type — rather than being widened to `-> !`,
    // because it is EXPORTED and writable: a caller may install a function that returns, and
    // typing the slot as diverging would make that well-defined-in-C case UB here.
    core::mem::transmute::<unsafe extern "C" fn(core::ffi::c_int) -> !, _>(_exit)
};
unsafe extern "C" fn initialize_onexit_table(table: *mut _onexit_table_t) -> core::ffi::c_int {
    if table.is_null() {
        return -1;
    }

    if (*table)._first == (*table)._end {
        (*table)._first = core::ptr::null_mut();
        (*table)._end = core::ptr::null_mut();
        (*table)._last = core::ptr::null_mut();
    }
    0
}
unsafe extern "C" fn register_onexit_function(table: *mut _onexit_table_t, func: _onexit_t) -> i32 {
    if table.is_null() {
        return -1;
    }

    EnterCriticalSection(&raw mut MSVCRT_onexit_cs);
    if (*table)._first.is_null() {
        (*table)._first = calloc(32, core::mem::size_of::<*mut core::ffi::c_void>() as _) as *mut _PVFV;
        if (*table)._first.is_null() {
            LeaveCriticalSection(&raw mut MSVCRT_onexit_cs);
            return -1;
        }
        (*table)._last = (*table)._first;
        (*table)._end = (*table)._first.add(32 as usize);
    }

    /* grow if full */
    if (*table)._last == (*table)._end {
        let len = (*table)._end.offset_from((*table)._first) as i32;
        let tmp = realloc((*table)._first as *mut core::ffi::c_void, (2 * len) as usize * core::mem::size_of::<*mut core::ffi::c_void>()) as *mut _PVFV;
        if tmp.is_null() {
            LeaveCriticalSection(&raw mut MSVCRT_onexit_cs);
            return -1;
        }
        (*table)._first = tmp;
        (*table)._end = (*table)._first.add((2 * len) as usize);
        (*table)._last = (*table)._first.add(len as usize);
    }

    *(*table)._last = core::mem::transmute::<_onexit_t, _PVFV>(func);
    (*table)._last = (*table)._last.add(1);
    LeaveCriticalSection(&raw mut MSVCRT_onexit_cs);
    0
}
unsafe extern "C" fn execute_onexit_table(table: *mut _onexit_table_t) -> core::ffi::c_int {
    let mut copy: _onexit_table_t = core::mem::zeroed();
    let mut func: *mut _PVFV;

    if table.is_null() {
        return -1;
    }

    EnterCriticalSection(&raw mut MSVCRT_onexit_cs);
    if (*table)._first.is_null() || (*table)._first >= (*table)._last {
        LeaveCriticalSection(&raw mut MSVCRT_onexit_cs);
        return 0;
    }
    copy._first = (*table)._first;
    copy._last = (*table)._last;
    copy._end = (*table)._end;
    memset(table as *mut core::ffi::c_void, 0, core::mem::size_of::<_onexit_table_t>());
    initialize_onexit_table(table);
    LeaveCriticalSection(&raw mut MSVCRT_onexit_cs);

    func = copy._last.offset(-(1 as isize));
    while func >= copy._first {
        if !(*func).is_none() {
            if let Some(f) = *func {
                f();
            }
        }
        func = func.offset(-1);
    }

    free(copy._first as *mut core::ffi::c_void);
    0
}
unsafe extern "C" fn call_atexit() {
    if let Some(cb) = tls_atexit_callback {
        cb(core::ptr::null_mut(), DLL_PROCESS_DETACH, core::ptr::null_mut());
    }
    execute_onexit_table(core::ptr::addr_of_mut!(MSVCRT_atexit_table));
}

#[no_mangle]
pub unsafe extern "C" fn __dllonexit(
    func: _onexit_t,
    start: *mut *mut _onexit_t,
    end: *mut *mut _onexit_t,
) -> _onexit_t {
    let tmp: *mut _onexit_t;
    let mut len: core::ffi::c_int;

    if start.is_null()
        || (*start).is_null()
        || end.is_null()
        || (*end).is_null()
    {
        FIXME!("bad table\n");
        return None;
    }

    len = (*end).offset_from(*start) as core::ffi::c_int;

    len += 1;
    if len <= 0 {
        return None;
    }

    tmp = realloc(
        *start as *mut core::ffi::c_void,
        len as usize * core::mem::size_of::<_onexit_t>(),
    ) as *mut _onexit_t;
    if tmp.is_null() {
        return None;
    }
    *start = tmp;
    *end = tmp.offset(len as isize);
    *tmp.offset((len - 1) as isize) = func;
    func
}

/// `-> !`, not `-> ()`. `stdlib.h:229` declares this `__declspec(noreturn) void _exit(int)`,
/// so every C caller is compiled assuming control never comes back — and this crate's own
/// `prelude.rs` and `process.rs` both DECLARE it `-> !` for that reason. It arrived defined
/// as returning `()`, which is a declaration/definition ABI split the compiler cannot see
/// across a `#[no_mangle]` boundary: `scan_extern_arity` caught it as
/// `[RETVOID] declared as returning a value, defined as returning nothing`.
///
/// The island types `ExitProcess` as `-> !` (winbase.h's DECLSPEC_NORETURN), so the call
/// itself is the diverging tail -- no `unreachable_unchecked` needed after it.
#[no_mangle]
pub unsafe extern "C" fn _exit(exitcode: i32) -> ! {
    // TRACE("(%d)\n", exitcode);
    ExitProcess(exitcode as u32);
}
unsafe extern "C" fn DoMessageBoxW(lead: *const wchar_t, message: *const wchar_t) {
    let mut msgbox: MSGBOXPARAMSW = core::mem::zeroed();
    let mut text: [wchar_t; 2048] = [0; 2048];
    let ret: INT;

    _snwprintf(
        text.as_mut_ptr(),
        (ARRAY_SIZE!(text)) as usize,
        wstr!("%ls\n\nProgram: %ls\n%ls\n\nPress OK to exit the program, or Cancel to start the Wine debugger.\n"),
        lead,
        MSVCRT__wpgmptr,
        message,
    );

    msgbox.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as _;
    msgbox.hwndOwner = GetActiveWindow();
    msgbox.hInstance = core::ptr::null_mut();
    msgbox.lpszText = text.as_ptr();
    msgbox.lpszCaption = wstr!("Wine C++ Runtime Library");
    msgbox.dwStyle = MB_OKCANCEL | MB_ICONERROR;
    msgbox.lpszIcon = core::ptr::null_mut();
    msgbox.dwContextHelpId = 0;
    msgbox.lpfnMsgBoxCallback = None;
    msgbox.dwLanguageId = LANG_NEUTRAL;

    ret = MessageBoxIndirectW(&mut msgbox);
    if ret == IDCANCEL as INT {
        DebugBreak();
    }
}
unsafe extern "C" fn DoMessageBox(lead: *const core::ffi::c_char, message: *const core::ffi::c_char) {
    let mut leadW: [wchar_t; 1024] = core::mem::zeroed();
    let mut messageW: [wchar_t; 1024] = core::mem::zeroed();

    mbstowcs(leadW.as_mut_ptr(), lead, 1024);
    mbstowcs(messageW.as_mut_ptr(), message, 1024);

    DoMessageBoxW(leadW.as_ptr(), messageW.as_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn _amsg_exit(errnum: core::ffi::c_int) {
    if (MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int)
        || ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2))
    {
        let mut text: [core::ffi::c_char; 32 as usize] =
            [0; 32 as usize];
        sprintf(
            text.as_mut_ptr(),
            b"Error: R60%d\0".as_ptr() as *const core::ffi::c_char,
            errnum,
        );
        DoMessageBox(
            b"Runtime error!\0".as_ptr() as *const core::ffi::c_char,
            text.as_ptr(),
        );
    } else {
        _cprintf(
            b"\nruntime error R60%d\n\0".as_ptr() as *const core::ffi::c_char,
            errnum,
        );
    }
    // C: `_aexit_rtn(255)`. The pointer is exported and writable, so a caller can NULL it and
    // the C would jump to 0; `unwrap_unchecked` reproduces that rather than converting it into
    // a Rust panic the original never had.
    _aexit_rtn.unwrap_unchecked()(255);
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    if MSVCRT_abort_behavior & _WRITE_ABORT_MSG != 0 {
        if (MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int)
            || ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2))
        {
            DoMessageBox(
                b"Runtime error!\0".as_ptr() as *const core::ffi::c_char,
                b"abnormal program termination\0".as_ptr() as *const core::ffi::c_char,
            );
        } else {
            _cputs(b"\nabnormal program termination\n\0".as_ptr() as *const core::ffi::c_char);
        }
    }
    raise(SIGABRT as core::ffi::c_int);
    _exit(3);
}

#[no_mangle]
pub unsafe extern "C" fn _wassert(str: *const wchar_t, file: *const wchar_t, line: core::ffi::c_uint) {
    ERR!("(%S,%S,%d)\n", str, file, line);
    if (MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int)
        || ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2))
    {
        let mut text: [wchar_t; 2048 as usize] =
            [0; 2048 as usize];
        // `size_of_val`, NOT `ARRAY_SIZE!` — and that is deliberate. The C says
        // `_snwprintf(text, sizeof(text), ...)` where `text` is `wchar_t[2048]`, so it passes
        // 4096 for a parameter measured in CHARACTERS. That is an upstream Wine bug (the
        // sibling DoMessageBoxW three functions up gets it right with `ARRAY_SIZE`), but it is
        // the behaviour being ported: `_snwprintf` will happily write past the array on a long
        // enough expression. Translating it to 2048 would be a silent divergence, so the bug
        // is reproduced and named here instead.
        _snwprintf(
            text.as_mut_ptr(),
            core::mem::size_of_val(&text),
            wstr!("File: %ls\nLine: %d\n\nExpression: \"%ls\""),
            file,
            line,
            str,
        );
        DoMessageBoxW(wstr!("Assertion failed!"), text.as_ptr());
    } else {
        fwprintf(
            stderr!(),
            wstr!("Assertion failed: %ls, file %ls, line %d\n\n"),
            str,
            file,
            line,
        );
    }

    raise(SIGABRT as core::ffi::c_int);
    _exit(3);
}

#[no_mangle]
pub unsafe extern "C" fn _assert(str: *const core::ffi::c_char, file: *const core::ffi::c_char, line: core::ffi::c_uint) {
    let mut strW: [wchar_t; 1024] = [0; 1024];
    let mut fileW: [wchar_t; 1024] = [0; 1024];

    mbstowcs(strW.as_mut_ptr(), str, 1024);
    mbstowcs(fileW.as_mut_ptr(), file, 1024);

    _wassert(strW.as_ptr(), fileW.as_ptr(), line);
}

#[no_mangle]
pub unsafe extern "C" fn _c_exit() {
    // TRACE("(void)\n");
}

#[no_mangle]
pub unsafe extern "C" fn _cexit() {
    LOCK_EXIT!();
    call_atexit();
    UNLOCK_EXIT!();
}

#[no_mangle]
pub unsafe extern "C" fn _onexit(func: _onexit_t) -> _onexit_t {
    if func.is_none() {
        return None;
    }

    LOCK_EXIT!();
    register_onexit_function(core::ptr::addr_of_mut!(MSVCRT_atexit_table), func);
    UNLOCK_EXIT!();

    func
}

#[no_mangle]
pub unsafe extern "C" fn exit(exitcode: i32) {
    let hmscoree: HMODULE;
    let pCorExitProcess: Option<unsafe extern "system" fn(i32)>;

    _cexit();

    hmscoree = GetModuleHandleW(wstr!("mscoree"));

    if !hmscoree.is_null() {
        pCorExitProcess = core::mem::transmute::<_, Option<unsafe extern "system" fn(i32)>>(
            GetProcAddress(hmscoree, b"CorExitProcess\0".as_ptr() as *const i8),
        );

        if let Some(func) = pCorExitProcess {
            func(exitcode);
        }
    }

    ExitProcess(exitcode as u32);
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT_atexit(func: Option<unsafe extern "C" fn()>) -> core::ffi::c_int {
    if _onexit(core::mem::transmute::<_, _onexit_t>(func)) == core::mem::transmute::<_, _onexit_t>(func) {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _register_thread_local_exe_atexit_callback(callback: _tls_callback_type) {
    tls_atexit_callback = callback;
}

#[no_mangle]
pub unsafe extern "C" fn _set_error_mode(mode: core::ffi::c_int) -> core::ffi::c_int {
    let old = MSVCRT_error_mode;
    if _REPORT_ERRMODE as core::ffi::c_int != mode {
        MSVCRT_error_mode = mode;
    }
    old
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll exit functions
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
//  */
// #include <conio.h>
// #include <process.h>
// #include <signal.h>
// #include <stdio.h>
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "winuser.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// /* MT */
// #define LOCK_EXIT   _lock(_EXIT_LOCK1)
// #define UNLOCK_EXIT _unlock(_EXIT_LOCK1)
//
// static _purecall_handler purecall_handler = NULL;
//
// static _onexit_table_t MSVCRT_atexit_table;
//
// typedef void (__stdcall *_tls_callback_type)(void*,ULONG,void*);
// static _tls_callback_type tls_atexit_callback;
//
// static CRITICAL_SECTION MSVCRT_onexit_cs;
// static CRITICAL_SECTION_DEBUG MSVCRT_onexit_cs_debug =
// {
//     0, 0, &MSVCRT_onexit_cs,
//     { &MSVCRT_onexit_cs_debug.ProcessLocksList, &MSVCRT_onexit_cs_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": MSVCRT_onexit_cs") }
// };
// static CRITICAL_SECTION MSVCRT_onexit_cs = { &MSVCRT_onexit_cs_debug, -1, 0, 0, 0, 0 };
//
// extern int MSVCRT_app_type;
// extern wchar_t *MSVCRT__wpgmptr;
//
// #if _MSVCR_VER > 0 || defined(_DEBUG)
// static unsigned int MSVCRT_abort_behavior =  _WRITE_ABORT_MSG | _CALL_REPORTFAULT;
// #endif
//
// static int MSVCRT_error_mode = _OUT_TO_DEFAULT;
//
// void (* _aexit_rtn)(int) = _exit;
//
// static int initialize_onexit_table(_onexit_table_t *table)
// {
//     if (!table)
//         return -1;
//
//     if (table->_first == table->_end)
//         table->_last = table->_end = table->_first = NULL;
//     return 0;
// }
//
// static int register_onexit_function(_onexit_table_t *table, _onexit_t func)
// {
//     if (!table)
//         return -1;
//
//     EnterCriticalSection(&MSVCRT_onexit_cs);
//     if (!table->_first)
//     {
//         table->_first = calloc(32, sizeof(void *));
//         if (!table->_first)
//         {
//             WARN("failed to allocate initial table.\n");
//             LeaveCriticalSection(&MSVCRT_onexit_cs);
//             return -1;
//         }
//         table->_last = table->_first;
//         table->_end = table->_first + 32;
//     }
//
//     /* grow if full */
//     if (table->_last == table->_end)
//     {
//         int len = table->_end - table->_first;
//         _PVFV *tmp = realloc(table->_first, 2 * len * sizeof(void *));
//         if (!tmp)
//         {
//             WARN("failed to grow table.\n");
//             LeaveCriticalSection(&MSVCRT_onexit_cs);
//             return -1;
//         }
//         table->_first = tmp;
//         table->_end = table->_first + 2 * len;
//         table->_last = table->_first + len;
//     }
//
//     *table->_last = (_PVFV)func;
//     table->_last++;
//     LeaveCriticalSection(&MSVCRT_onexit_cs);
//     return 0;
// }
//
// static int execute_onexit_table(_onexit_table_t *table)
// {
//     _onexit_table_t copy;
//     _PVFV *func;
//
//     if (!table)
//         return -1;
//
//     EnterCriticalSection(&MSVCRT_onexit_cs);
//     if (!table->_first || table->_first >= table->_last)
//     {
//         LeaveCriticalSection(&MSVCRT_onexit_cs);
//         return 0;
//     }
//     copy._first = table->_first;
//     copy._last  = table->_last;
//     copy._end   = table->_end;
//     memset(table, 0, sizeof(*table));
//     initialize_onexit_table(table);
//     LeaveCriticalSection(&MSVCRT_onexit_cs);
//
//     for (func = copy._last - 1; func >= copy._first; func--)
//     {
//         if (*func)
//            (*func)();
//     }
//
//     free(copy._first);
//     return 0;
// }
//
// static void call_atexit(void)
// {
//     /* Note: should only be called with the exit lock held */
//     if (tls_atexit_callback) tls_atexit_callback(NULL, DLL_PROCESS_DETACH, NULL);
//     execute_onexit_table(&MSVCRT_atexit_table);
// }
//
// /*********************************************************************
//  *		__dllonexit (MSVCRT.@)
//  */
// _onexit_t CDECL __dllonexit(_onexit_t func, _onexit_t **start, _onexit_t **end)
// {
//   _onexit_t *tmp;
//   int len;
//
//   TRACE("(%p,%p,%p)\n", func, start, end);
//
//   if (!start || !*start || !end || !*end)
//   {
//    FIXME("bad table\n");
//    return NULL;
//   }
//
//   len = (*end - *start);
//
//   TRACE("table start %p-%p, %d entries\n", *start, *end, len);
//
//   if (++len <= 0)
//     return NULL;
//
//   tmp = realloc(*start, len * sizeof(*tmp));
//   if (!tmp)
//     return NULL;
//   *start = tmp;
//   *end = tmp + len;
//   tmp[len - 1] = func;
//   TRACE("new table start %p-%p, %d entries\n", *start, *end, len);
//   return func;
// }
//
// /*********************************************************************
//  *		_exit (MSVCRT.@)
//  */
// void CDECL _exit(int exitcode)
// {
//   TRACE("(%d)\n", exitcode);
//   ExitProcess(exitcode);
// }
//
// /* Print out an error message with an option to debug */
// static void DoMessageBoxW(const wchar_t *lead, const wchar_t *message)
// {
//   MSGBOXPARAMSW msgbox;
//   wchar_t text[2048];
//   INT ret;
//
//   _snwprintf(text, ARRAY_SIZE(text), L"%ls\n\nProgram: %ls\n%ls\n\n"
//           L"Press OK to exit the program, or Cancel to start the Wine debugger.\n",
//           lead, MSVCRT__wpgmptr, message);
//
//   msgbox.cbSize = sizeof(msgbox);
//   msgbox.hwndOwner = GetActiveWindow();
//   msgbox.hInstance = 0;
//   msgbox.lpszText = text;
//   msgbox.lpszCaption = L"Wine C++ Runtime Library";
//   msgbox.dwStyle = MB_OKCANCEL|MB_ICONERROR;
//   msgbox.lpszIcon = NULL;
//   msgbox.dwContextHelpId = 0;
//   msgbox.lpfnMsgBoxCallback = NULL;
//   msgbox.dwLanguageId = LANG_NEUTRAL;
//
//   ret = MessageBoxIndirectW(&msgbox);
//   if (ret == IDCANCEL)
//     DebugBreak();
// }
//
// static void DoMessageBox(const char *lead, const char *message)
// {
//   wchar_t leadW[1024], messageW[1024];
//
//   mbstowcs(leadW, lead, 1024);
//   mbstowcs(messageW, message, 1024);
//
//   DoMessageBoxW(leadW, messageW);
// }
//
// /*********************************************************************
//  *		_amsg_exit (MSVCRT.@)
//  */
// void CDECL _amsg_exit(int errnum)
// {
//   TRACE("(%d)\n", errnum);
//
//   if ((MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int) ||
//      ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2)))
//   {
//     char text[32];
//     sprintf(text, "Error: R60%d",errnum);
//     DoMessageBox("Runtime error!", text);
//   }
//   else
//     _cprintf("\nruntime error R60%d\n",errnum);
//   _aexit_rtn(255);
// }
//
// /*********************************************************************
//  *		abort (MSVCRT.@)
//  */
// void CDECL abort(void)
// {
//   TRACE("()\n");
//
// #if (_MSVCR_VER > 0 && _MSVCR_VER < 100) || _MSVCR_VER == 120 || defined(_DEBUG)
//   if (MSVCRT_abort_behavior & _WRITE_ABORT_MSG)
//   {
//     if ((MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int) ||
//        ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2)))
//     {
//       DoMessageBox("Runtime error!", "abnormal program termination");
//     }
//     else
//       _cputs("\nabnormal program termination\n");
//   }
// #endif
//   raise(SIGABRT as core::ffi::c_int);
//   /* in case raise() returns */
//   _exit(3);
// }
//
//
// /*********************************************************************
//  *              _wassert (MSVCRT.@)
//  */
// void DECLSPEC_NORETURN CDECL _wassert(const wchar_t* str, const wchar_t* file, unsigned int line)
// {
//   ERR("(%s,%s,%d)\n", debugstr_w(str), debugstr_w(file), line);
//
//   if ((MSVCRT_error_mode == _OUT_TO_MSGBOX as core::ffi::c_int) ||
//      ((MSVCRT_error_mode == _OUT_TO_DEFAULT as core::ffi::c_int) && (MSVCRT_app_type == 2)))
//   {
//     wchar_t text[2048];
//     _snwprintf(text, sizeof(text), L"File: %ls\nLine: %d\n\nExpression: \"%ls\"", file, line, str);
//     DoMessageBoxW(L"Assertion failed!", text);
//   }
//   else
//     fwprintf(stderr, L"Assertion failed: %ls, file %ls, line %d\n\n", str, file, line);
//
//   raise(SIGABRT as core::ffi::c_int);
//   _exit(3);
// }
//
// /*********************************************************************
//  *		_assert (MSVCRT.@)
//  */
// void DECLSPEC_NORETURN CDECL _assert(const char* str, const char* file, unsigned int line)
// {
//     wchar_t strW[1024], fileW[1024];
//
//     mbstowcs(strW, str, 1024);
//     mbstowcs(fileW, file, 1024);
//
//     _wassert(strW, fileW, line);
// }
//
// /*********************************************************************
//  *		_c_exit (MSVCRT.@)
//  */
// void CDECL _c_exit(void)
// {
//   TRACE("(void)\n");
//   /* All cleanup is done on DLL detach; Return to caller */
// }
//
// /*********************************************************************
//  *		_cexit (MSVCRT.@)
//  */
// void CDECL _cexit(void)
// {
//   TRACE("(void)\n");
//   LOCK_EXIT;
//   call_atexit();
//   UNLOCK_EXIT;
// }
//
// /*********************************************************************
//  *		_onexit (MSVCRT.@)
//  */
// _onexit_t CDECL _onexit(_onexit_t func)
// {
//   TRACE("(%p)\n",func);
//
//   if (!func)
//     return NULL;
//
//   LOCK_EXIT;
//   register_onexit_function(&MSVCRT_atexit_table, func);
//   UNLOCK_EXIT;
//
//   return func;
// }
//
// /*********************************************************************
//  *		exit (MSVCRT.@)
//  */
// void CDECL exit(int exitcode)
// {
//   HMODULE hmscoree;
//   void (WINAPI *pCorExitProcess)(int);
//
//   TRACE("(%d)\n",exitcode);
//   _cexit();
//
//   hmscoree = GetModuleHandleW(L"mscoree");
//
//   if (hmscoree)
//   {
//     pCorExitProcess = (void*)GetProcAddress(hmscoree, "CorExitProcess");
//
//     if (pCorExitProcess)
//       pCorExitProcess(exitcode);
//   }
//
//   ExitProcess(exitcode);
// }
//
// /*********************************************************************
//  *		atexit (MSVCRT.@)
//  */
// int CDECL MSVCRT_atexit(void (__cdecl *func)(void))
// {
//   TRACE("(%p)\n", func);
//   return _onexit((_onexit_t)func) == (_onexit_t)func ? 0 : -1;
// }
//
//
// /*********************************************************************
//  *		_register_thread_local_exe_atexit_callback (UCRTBASE.@)
//  */
// void CDECL _register_thread_local_exe_atexit_callback(_tls_callback_type callback)
// {
//     TRACE("(%p)\n", callback);
//     tls_atexit_callback = callback;
// }
//
//
//
// /*********************************************************************
//  *		_purecall (MSVCRT.@)
//  */
// void CDECL _purecall(void)
// {
//   TRACE("(void)\n");
//
//   if(purecall_handler)
//       purecall_handler();
//   _amsg_exit( 25 );
// }
//
// /******************************************************************************
//  *		_set_error_mode (MSVCRT.@)
//  *
//  * Set the error mode, which describes where the C run-time writes error messages.
//  *
//  * PARAMS
//  *   mode - the new error mode
//  *
//  * RETURNS
//  *   The old error mode.
//  *
//  */
// int CDECL _set_error_mode(int mode)
// {
//
//   const int old = MSVCRT_error_mode;
//   if ( _REPORT_ERRMODE != mode ) {
//     MSVCRT_error_mode = mode;
//   }
//   return old;
// }
//

// ---- hand-written: the body --skip-fn dropped ------------------------------------------
//
// rustconv refused this one because the model called the sealed `purecall_handler()`
// occurrence with an argument the C does not pass (`wants 0, got 1`) — i.e. it damaged the
// single dispatch this function exists to perform. Transcribed from dll/win32/msvcrt/exit.c.

/// `exit.c` — `void CDECL _purecall(void)`. The compiler plants a call to this in a vtable
/// slot for a pure virtual, so reaching it means a pure virtual was called during
/// construction/destruction. Runs the installed handler if there is one, then `_amsg_exit(25)`
/// — which does NOT return, so there is nothing after it.
#[no_mangle]
pub unsafe extern "C" fn _purecall() {
    TRACE!("(void)\n");

    if let Some(handler) = purecall_handler {
        handler();
    }
    _amsg_exit(25);
}

// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
