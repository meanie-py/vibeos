use crate::prelude::*;
// NOT `use msvcrt_sys::*;` as well — prelude.rs already re-exports the whole island, and
// a second glob makes every overlapping name E0659-ambiguous. See file.rs for the long form.
use core::ffi::c_void;

// CRT siblings ALREADY Rust in this crate, imported so rustc checks each call against the
// definition. `env_init` is environ.c's, which is why this module rides the same feature.
use crate::msvcrt::environ::env_init;
use crate::wcslen;

// data.c's wine-debug diagnostics. `macro_rules!`, never a `fn`.

// ---- file-scope preamble (translated) ----
static mut initial_wargv: *mut *mut wchar_t = core::ptr::null_mut();
static mut initial_argc: i32 = 0;
/// The CRT's process-wide state, and every one of these is GLOBAL in `data.c.obj` — the
/// `__argc`/`__argv`/`_environ`/`_pgmptr` family plus the version and mode words that
/// `__p__*` hands out pointers to. The translation emitted all 22 as PRIVATE Rust statics,
/// which is the same defect exit.c's `_aexit_rtn` and locale.c's eight globals arrived with:
/// it compiles, it links, and it silently leaves `data.c.obj` as the only definition of each,
/// dragging that whole object back in and duplicating the 35 functions above. Three files in
/// a row makes this systematic, not incidental — the object's symbol table is the authority
/// on what a TU exports, never the `static` keywords in the source.
#[no_mangle]
pub static mut MSVCRT___argc: i32 = 0;
static mut wargc_expand: i32 = 0;
#[no_mangle]
pub static mut MSVCRT__commode: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__fmode: i32 = 0;
#[no_mangle]
pub static mut MSVCRT__osver: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__osplatform: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__winmajor: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__winminor: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__winver: u32 = 0;
#[no_mangle]
pub static mut MSVCRT___setlc_active: u32 = 0;
#[no_mangle]
pub static mut MSVCRT___unguarded_readlc_active: u32 = 0;
#[no_mangle]
pub static mut MSVCRT__HUGE: f64 = 0.0;
#[no_mangle]
pub static mut MSVCRT___argv: *mut *mut i8 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT___wargv: *mut *mut wchar_t = core::ptr::null_mut();
static mut wargv_expand: *mut *mut wchar_t = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__acmdln: *mut i8 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__wcmdln: *mut wchar_t = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__environ: *mut *mut i8 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__wenviron: *mut *mut wchar_t = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT___initenv: *mut *mut i8 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT___winitenv: *mut *mut wchar_t = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT_app_type: i32 = 0;
#[no_mangle]
pub static mut MSVCRT__pgmptr: *mut i8 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__wpgmptr: *mut wchar_t = core::ptr::null_mut();

/// data.c:221 — `typedef void (CDECL *_INITTERMFUN)(void);`. `Option<..>` because
/// `_initterm` tests each table slot against NULL before calling it, which a bare Rust fn
/// pointer cannot express (it is non-nullable by construction). Same shape bindgen gives
/// `_PVFV` for the identical C typedef.
pub type _INITTERMFUN = Option<unsafe extern "C" fn()>;
/// data.c:222 — `typedef int (CDECL *_INITTERM_E_FN)(void);`, nullable for the same reason.
pub type _INITTERM_E_FN = Option<unsafe extern "C" fn() -> i32>;
unsafe extern "C" fn build_argv(wargv: *mut *mut wchar_t) -> *mut *mut i8 {
    let mut argc: i32;
    let mut p: *mut i8;
    let argv: *mut *mut i8;
    let mut total: u32 = 0;

    argc = 0;
    while !(*wargv.offset(argc as isize)).is_null() {
        total += (WideCharToMultiByte(
            CP_ACP,
            0,
            *wargv.offset(argc as isize),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        argc += 1;
    }

    argv = HeapAlloc(
        GetProcessHeap(),
        0,
        (total as usize + (argc as usize + 1) * core::mem::size_of::<*mut i8>()) as SIZE_T,
    ) as *mut *mut i8;
    p = argv.offset((argc + 1) as isize) as *mut i8;

    argc = 0;
    while !(*wargv.offset(argc as isize)).is_null() {
        let reslen: u32 = WideCharToMultiByte(
            CP_ACP,
            0,
            *wargv.offset(argc as isize),
            -1,
            p,
            total as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as u32;
        *argv.offset(argc as isize) = p;
        p = p.offset(reslen as isize);
        total -= reslen;
        argc += 1;
    }
    *argv.offset(argc as isize) = core::ptr::null_mut();
    argv
}
unsafe extern "C" fn cmdline_to_argv(src: *const wchar_t, ret_argc: *mut i32) -> *mut *mut wchar_t {
    let mut argv: *mut *mut wchar_t;
    let mut arg: *mut wchar_t;
    let mut dst: *mut wchar_t;
    let mut argc: i32;
    let mut in_quotes: i32 = 0;
    let mut bcount: i32 = 0;
    let len: usize = wcslen(src) + 1;

    argc = 2 + (len / 2) as i32;
    argv = HeapAlloc(
        GetProcessHeap(),
        0,
        ((argc as usize) * core::mem::size_of::<*mut wchar_t>()
            + len * core::mem::size_of::<wchar_t>()) as SIZE_T,
    ) as *mut *mut wchar_t;
    arg = (argv.offset(argc as isize)) as *mut wchar_t;
    dst = arg;
    argc = 0;
    let mut src_mut = src;
    while *src_mut != 0 {
        if ((*src_mut == ' ' as wchar_t || *src_mut == '\t' as wchar_t) && in_quotes == 0) {
            while *src_mut == ' ' as wchar_t || *src_mut == '\t' as wchar_t {
                src_mut = src_mut.offset(1);
            }
            if *src_mut == 0 {
                break;
            }
            *dst = 0;
            dst = dst.offset(1);
            *argv.offset(argc as isize) = arg;
            argc += 1;
            arg = dst;
            bcount = 0;
        } else if *src_mut == '\\' as wchar_t {
            *dst = *src_mut;
            dst = dst.offset(1);
            src_mut = src_mut.offset(1);
            bcount += 1;
        } else if *src_mut == '"' as wchar_t {
            if (bcount & 1) == 0 {
                dst = dst.offset(-(bcount / 2) as isize);
                src_mut = src_mut.offset(1);
                if in_quotes != 0 && *src_mut == '"' as wchar_t {
                    *dst = *src_mut;
                    dst = dst.offset(1);
                    src_mut = src_mut.offset(1);
                }
                in_quotes = if in_quotes != 0 { 0 } else { 1 };
            } else {
                dst = dst.offset(-(bcount / 2 + 1) as isize);
                *dst = *src_mut;
                dst = dst.offset(1);
                src_mut = src_mut.offset(1);
            }
            bcount = 0;
        } else {
            *dst = *src_mut;
            dst = dst.offset(1);
            src_mut = src_mut.offset(1);
            bcount = 0;
        }
    }
    *dst = 0;
    *argv.offset(argc as isize) = arg;
    argc += 1;
    *argv.offset(argc as isize) = core::ptr::null_mut();
    *ret_argc = argc;
    argv
}
unsafe extern "C" fn cmdline_to_argvA(src: *const i8, ret_argc: *mut i32) -> *mut *mut i8 {
    let mut argv: *mut *mut i8;
    let mut arg: *mut i8;
    let mut dst: *mut i8;
    let mut argc: i32;
    let mut in_quotes: i32 = 0;
    let mut bcount: i32 = 0;
    let len: i32 = strlen(src) as i32 + 1;

    argc = 2 + len / 2;
    argv = HeapAlloc(
        GetProcessHeap(),
        0,
        (argc as usize * core::mem::size_of::<*mut i8>()
            + len as usize * core::mem::size_of::<CHAR>()) as SIZE_T,
    ) as *mut *mut i8;
    arg = argv.offset(argc as isize) as *mut i8;
    dst = arg;
    argc = 0;

    let mut src = src as *mut i8;
    while *src != 0 {
        if ((*src == b' ' as i8 || *src == b'\t' as i8) && in_quotes == 0) {
            while *src == b' ' as i8 || *src == b'\t' as i8 {
                src = src.offset(1);
            }
            if *src == 0 {
                break;
            }
            *dst = 0;
            dst = dst.offset(1);
            *argv.offset(argc as isize) = arg;
            argc += 1;
            arg = dst;
            bcount = 0;
        } else if *src == b'\\' as i8 {
            *dst = *src;
            dst = dst.offset(1);
            src = src.offset(1);
            bcount += 1;
        } else if *src == b'"' as i8 {
            if (bcount & 1) == 0 {
                dst = dst.offset(-(bcount / 2) as isize);
                src = src.offset(1);
                if in_quotes != 0 && *src == b'"' as i8 {
                    *dst = *src;
                    dst = dst.offset(1);
                    src = src.offset(1);
                }
                in_quotes = if in_quotes != 0 { 0 } else { 1 };
            } else {
                dst = dst.offset(-(bcount / 2 + 1) as isize);
                *dst = *src;
                dst = dst.offset(1);
                src = src.offset(1);
            }
            bcount = 0;
        } else {
            *dst = *src;
            dst = dst.offset(1);
            src = src.offset(1);
            bcount = 0;
        }
    }
    *dst = 0;
    *argv.offset(argc as isize) = arg;
    argc += 1;
    *argv.offset(argc as isize) = core::ptr::null_mut();
    *ret_argc = argc;
    argv
}

#[no_mangle]
pub unsafe extern "C" fn __p___argc() -> *mut i32 {
    core::ptr::addr_of_mut!(MSVCRT___argc)
}

#[no_mangle]
pub unsafe extern "C" fn __p__commode() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT__commode)
}

#[no_mangle]
pub unsafe extern "C" fn __p__pgmptr() -> *mut PCHAR {
    core::ptr::addr_of_mut!(MSVCRT__pgmptr)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wpgmptr() -> *mut *mut wchar_t {
    core::ptr::addr_of_mut!(MSVCRT__wpgmptr)
}
#[no_mangle]
pub unsafe extern "C" fn _get_pgmptr(p: *mut *mut i8) -> i32 {
    if MSVCRT_CHECK_PMT!(!p.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    *p = MSVCRT__pgmptr;
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _get_wpgmptr(p: *mut *mut wchar_t) -> i32 {
    if MSVCRT_CHECK_PMT!(!p.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    *p = MSVCRT__wpgmptr;
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn __p__fmode() -> *mut i32 {
    core::ptr::addr_of_mut!(MSVCRT__fmode)
}
#[no_mangle]
pub unsafe extern "C" fn _set_fmode(mode: i32) -> i32 {
    if MSVCRT_CHECK_PMT!(mode == (_O_TEXT) as i32 || mode == (_O_BINARY) as i32) == 0 {
        return (EINVAL) as i32;
    }

    MSVCRT__fmode = mode;
    0
}
#[no_mangle]
pub unsafe extern "C" fn _get_fmode(mode: *mut i32) -> i32 {
    if MSVCRT_CHECK_PMT!(!mode.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    *mode = MSVCRT__fmode;
    0
}

#[no_mangle]
pub unsafe extern "C" fn __p__osver() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT__osver)
}

#[no_mangle]
pub unsafe extern "C" fn __p__winmajor() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT__winmajor)
}

#[no_mangle]
pub unsafe extern "C" fn __p__winminor() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT__winminor)
}

#[no_mangle]
pub unsafe extern "C" fn __p__winver() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT__winver)
}

#[no_mangle]
pub unsafe extern "C" fn __p__acmdln() -> *mut PCHAR {
    core::ptr::addr_of_mut!(MSVCRT__acmdln)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wcmdln() -> *mut *mut wchar_t {
    core::ptr::addr_of_mut!(MSVCRT__wcmdln)
}
#[no_mangle]
pub unsafe extern "C" fn __p___argv() -> *mut *mut *mut i8 {
    core::ptr::addr_of_mut!(MSVCRT___argv)
}

#[no_mangle]
pub unsafe extern "C" fn __p___wargv() -> *mut *mut *mut wchar_t {
    core::ptr::addr_of_mut!(MSVCRT___wargv)
}

#[no_mangle]
pub unsafe extern "C" fn __p__environ() -> *mut *mut *mut i8 {
    core::ptr::addr_of_mut!(MSVCRT__environ)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wenviron() -> *mut *mut *mut wchar_t {
    core::ptr::addr_of_mut!(MSVCRT__wenviron)
}
#[no_mangle]
pub unsafe extern "C" fn __p___initenv() -> *mut *mut *mut i8 {
    core::ptr::addr_of_mut!(MSVCRT___initenv)
}

#[no_mangle]
pub unsafe extern "C" fn __p___winitenv() -> *mut *mut *mut wchar_t {
    core::ptr::addr_of_mut!(MSVCRT___winitenv)
}

#[no_mangle]
pub unsafe extern "C" fn _get_osplatform(pValue: *mut i32) -> i32 {
    if MSVCRT_CHECK_PMT!(pValue != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *pValue = MSVCRT__osplatform as i32;
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_wstrdupa(str: *const i8) -> *mut wchar_t {
    let len: u32 = (strlen(str) + 1) as u32;
    let wstr: *mut wchar_t = malloc((len as usize) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if wstr.is_null() {
        return core::ptr::null_mut();
    }
    MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, str, len as i32, wstr, len as i32);
    wstr
}

#[no_mangle]
pub unsafe extern "C" fn ___unguarded_readlc_active_add_func() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT___unguarded_readlc_active)
}

#[no_mangle]
pub unsafe extern "C" fn ___setlc_active_func() -> u32 {
    MSVCRT___setlc_active
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_args() {
    let mut osvi: OSVERSIONINFOW = core::mem::zeroed();

    MSVCRT__acmdln = _strdup(GetCommandLineA());
    MSVCRT__wcmdln = _wcsdup(GetCommandLineW());
    initial_wargv = cmdline_to_argv(GetCommandLineW(), &raw mut initial_argc);
    MSVCRT___argc = initial_argc;
    MSVCRT___wargv = initial_wargv;
    MSVCRT___argv = build_argv(initial_wargv);

    osvi.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as u32;
    GetVersionExW(&mut osvi);
    MSVCRT__winver = (osvi.dwMajorVersion << 8) | osvi.dwMinorVersion;
    MSVCRT__winmajor = osvi.dwMajorVersion;
    MSVCRT__winminor = osvi.dwMinorVersion;
    MSVCRT__osver = osvi.dwBuildNumber;
    MSVCRT__osplatform = osvi.dwPlatformId;

    // `#ifdef _CRTDLL` in the C. One archive serves every x86 CRT DLL, so this runs in all
    // of them; only crtdll exports the words (they live in legacy.rs). It was a cfg on a
    // feature that never existed, so crtdll reported a base version of 0.
    #[cfg(all(feature = "dllmain", target_arch = "x86"))]
    {
        use crate::legacy::{CRTDLL__basemajor_dll, CRTDLL__baseminor_dll, CRTDLL__baseversion_dll};
        // kernel32's; the msvcrt-sys island does not carry it.
        extern "system" {
            fn GetVersion() -> u32;
        }
        CRTDLL__baseversion_dll = GetVersion() >> 16;
        CRTDLL__basemajor_dll = CRTDLL__baseversion_dll >> 8;
        CRTDLL__baseminor_dll = CRTDLL__baseversion_dll & 0xff;
    }

    MSVCRT__HUGE = HUGE_VAL;
    MSVCRT___setlc_active = 0;
    MSVCRT___unguarded_readlc_active = 0;

    env_init((FALSE) as i32, (FALSE) as i32);

    MSVCRT__pgmptr = (HeapAlloc(GetProcessHeap(), 0, (MAX_PATH) as SIZE_T)).cast();
    if !MSVCRT__pgmptr.is_null() {
        if GetModuleFileNameA(core::ptr::null_mut(), MSVCRT__pgmptr, MAX_PATH) == 0 {
            *MSVCRT__pgmptr.add(0) = b'\0' as i8;
        } else {
            *MSVCRT__pgmptr.add((MAX_PATH - 1) as usize) = b'\0' as i8;
        }
    }

    MSVCRT__wpgmptr = (HeapAlloc(GetProcessHeap(), 0, (MAX_PATH * core::mem::size_of::<wchar_t>() as u32) as SIZE_T)).cast();
    if !MSVCRT__wpgmptr.is_null() {
        if GetModuleFileNameW(core::ptr::null_mut(), MSVCRT__wpgmptr, MAX_PATH) == 0 {
            *MSVCRT__wpgmptr.add(0) = 0;
        } else {
            *MSVCRT__wpgmptr.add((MAX_PATH - 1) as usize) = 0;
        }
    }
}
/// HAND-WRITTEN. `rustconv` refused its own output here — "model damaged deterministic call
/// envelope (not exactly once: `__RUSTCONV_CALL_0005__`)" — so the body was dropped with
/// `--skip-fn` and the other 39 replayed from cache. This is data.c:471 transcribed.
///
/// Two traps worth naming, because both are silent:
///
/// * `path_len` walks BACKWARDS and is the C's `int`, not a `size_t`. `wcslen(x) - 1` on an
///   empty string is `-1` in C and an underflow PANIC in Rust if computed in `usize`, so the
///   arithmetic stays in `i32` exactly as the C has it.
/// * `len` is declared per outer iteration and only assigned INSIDE the `do`/`while`. If
///   every match is `.` or `..` the `continue` skips the assignment, `len` stays 0, and the
///   `if(!len)` fallback below re-adds the unexpanded pattern. Hoisting `len` or
///   initialising it after the loop would quietly drop that argument.
///
/// Called twice per expansion: once with `argv == NULL` to SIZE the block, once to fill it,
/// so the two passes must agree exactly — which is why the `argv.is_null()` guards wrap only
/// the stores and never the counting.
unsafe extern "C" fn build_expanded_wargv(argc: *mut i32, argv: *mut *mut wchar_t) -> i32 {
    let mut size: i32 = 0;
    let mut args_no: i32;
    let mut path_len: i32;
    let mut is_expandable: BOOL;
    let mut h: HANDLE;

    args_no = 0;
    let mut i: i32 = 0;
    while i < initial_argc {
        let mut data: WIN32_FIND_DATAW = core::mem::zeroed();
        let mut len: i32 = 0;

        is_expandable = FALSE as BOOL;
        path_len = wcslen(*initial_wargv.add(i as usize)) as i32 - 1;
        while path_len >= 0 {
            let c = *(*initial_wargv.add(i as usize)).add(path_len as usize);
            if c == '*' as u16 || c == '?' as u16 {
                is_expandable = TRUE as BOOL;
            } else if c == '\\' as u16 || c == '/' as u16 {
                break;
            }
            path_len -= 1;
        }
        path_len += 1;

        if is_expandable != 0 {
            h = FindFirstFileW(*initial_wargv.add(i as usize), &mut data);
        } else {
            h = INVALID_HANDLE_VALUE;
        }

        if h != INVALID_HANDLE_VALUE {
            loop {
                if !(data.cFileName[0] == '.' as u16
                    && (data.cFileName[1] == 0
                        || (data.cFileName[1] == '.' as u16 && data.cFileName[2] == 0)))
                {
                    len = wcslen(data.cFileName.as_ptr()) as i32 + 1;
                    if !argv.is_null() {
                        *argv.add(args_no as usize) =
                            (argv.add(*argc as usize + 1) as *mut wchar_t).add(size as usize);
                        memcpy(
                            *argv.add(args_no as usize) as *mut core::ffi::c_void,
                            *initial_wargv.add(i as usize) as *const core::ffi::c_void,
                            path_len as usize * core::mem::size_of::<wchar_t>(),
                        );
                        memcpy(
                            (*argv.add(args_no as usize)).add(path_len as usize)
                                as *mut core::ffi::c_void,
                            data.cFileName.as_ptr() as *const core::ffi::c_void,
                            len as usize * core::mem::size_of::<wchar_t>(),
                        );
                    }
                    args_no += 1;
                    size += len + path_len;
                }
                if FindNextFileW(h, &mut data) == 0 {
                    break;
                }
            }
            FindClose(h);
        }

        if len == 0 {
            len = wcslen(*initial_wargv.add(i as usize)) as i32 + 1;
            if !argv.is_null() {
                *argv.add(args_no as usize) =
                    (argv.add(*argc as usize + 1) as *mut wchar_t).add(size as usize);
                memcpy(
                    *argv.add(args_no as usize) as *mut core::ffi::c_void,
                    *initial_wargv.add(i as usize) as *const core::ffi::c_void,
                    len as usize * core::mem::size_of::<wchar_t>(),
                );
            }
            args_no += 1;
            size += len;
        }
        i += 1;
    }

    if !argv.is_null() {
        *argv.add(args_no as usize) = core::ptr::null_mut();
    }
    size *= core::mem::size_of::<wchar_t>() as i32;
    size += (args_no + 1) * core::mem::size_of::<*mut wchar_t>() as i32;
    *argc = args_no;
    size
}

#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_args() {
    HeapFree(GetProcessHeap(), 0, MSVCRT___argv as *mut _);
    HeapFree(GetProcessHeap(), 0, MSVCRT__pgmptr as *mut _);
    HeapFree(GetProcessHeap(), 0, MSVCRT__wpgmptr as *mut _);
    HeapFree(GetProcessHeap(), 0, wargv_expand as *mut _);
}
#[no_mangle]
pub unsafe extern "C" fn __wgetmainargs(
    argc: *mut i32,
    wargv: *mut *mut *mut wchar_t,
    wenvp: *mut *mut *mut wchar_t,
    expand_wildcards: i32,
    new_mode: *mut i32,
) -> i32 {
    let mut expand_wildcards = expand_wildcards;

    if expand_wildcards != 0 {
        HeapFree(GetProcessHeap(), 0, (wargv_expand).cast());
        wargv_expand = (HeapAlloc(
            GetProcessHeap(),
            0,
            build_expanded_wargv(core::ptr::addr_of_mut!(wargc_expand), core::ptr::null_mut())
                as SIZE_T,
        )).cast();
        if !wargv_expand.is_null() {
            build_expanded_wargv(core::ptr::addr_of_mut!(wargc_expand), wargv_expand);

            MSVCRT___argc = wargc_expand;
            MSVCRT___wargv = wargv_expand;
        } else {
            expand_wildcards = 0;
        }
    }
    if expand_wildcards == 0 {
        if MSVCRT___wargv.is_null() || MSVCRT___argc == 0 {
            MSVCRT___wargv = cmdline_to_argv(MSVCRT__wcmdln, core::ptr::addr_of_mut!(MSVCRT___argc));
        }
    }

    env_init((TRUE) as i32, (FALSE) as i32);

    *argc = MSVCRT___argc;
    *wargv = MSVCRT___wargv;
    *wenvp = MSVCRT__wenviron;
    if !new_mode.is_null() {
        _set_new_mode(*new_mode);
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn __getmainargs(
    argc: *mut i32,
    argv: *mut *mut *mut i8,
    envp: *mut *mut *mut i8,
    expand_wildcards: i32,
    new_mode: *mut i32,
) -> i32 {
    let mut expand_wildcards = expand_wildcards;

    if expand_wildcards != 0 {
        HeapFree(GetProcessHeap(), 0, wargv_expand as *mut _);
        wargv_expand = (HeapAlloc(
            GetProcessHeap(),
            0,
            build_expanded_wargv(core::ptr::addr_of_mut!(wargc_expand), core::ptr::null_mut())
                as SIZE_T,
        )).cast();
        if !wargv_expand.is_null() {
            build_expanded_wargv(core::ptr::addr_of_mut!(wargc_expand), wargv_expand);

            MSVCRT___argc = wargc_expand;
            MSVCRT___argv = build_argv(wargv_expand);
        } else {
            expand_wildcards = 0;
        }
    }
    if expand_wildcards == 0 {
        if MSVCRT___argv.is_null() || MSVCRT___argc == 0 {
            MSVCRT___argv = cmdline_to_argvA(MSVCRT__acmdln, core::ptr::addr_of_mut!(MSVCRT___argc));
        }
    }

    *argc = MSVCRT___argc;
    *argv = MSVCRT___argv;
    *envp = MSVCRT__environ;

    if !new_mode.is_null() {
        _set_new_mode(*new_mode);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn __GetMainArgs(
    argc: *mut i32,
    argv: *mut *mut *mut i8,
    envp: *mut *mut *mut i8,
    expand_wildcards: i32,
) {
    let mut new_mode = 0;
    __getmainargs(argc, argv, envp, expand_wildcards, &mut new_mode);
}
#[no_mangle]
pub unsafe extern "C" fn _initterm(start: *mut _INITTERMFUN, end: *mut _INITTERMFUN) {
    let mut current = start;

    while current < end {
        if let Some(f) = *current {
            f();
        }
        current = current.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn _initterm_e(
    mut table: *mut _INITTERM_E_FN,
    end: *mut _INITTERM_E_FN,
) -> i32 {
    let mut res = 0;

    while res == 0 && table < end {
        if let Some(f) = *table {
            res = f();
            if res != 0 {
                TRACE!(
                    "function %p failed: %#x\n",
                    *table as *const (),
                    res
                );
            }
        }
        table = table.offset(1);
    }
    res
}

#[no_mangle]
pub unsafe extern "C" fn __set_app_type(app_type: i32) {
    MSVCRT_app_type = app_type;
}
#[no_mangle]
pub unsafe extern "C" fn _get_winmajor(value: *mut i32) -> i32 {
    if MSVCRT_CHECK_PMT!(value != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *value = MSVCRT__winmajor as i32;
    0
}
#[no_mangle]
pub unsafe extern "C" fn _get_winminor(value: *mut i32) -> i32 {
    if MSVCRT_CHECK_PMT!(value != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *value = MSVCRT__winminor as i32;
    0
}
#[no_mangle]
pub unsafe extern "C" fn _get_osver(value: *mut i32) -> i32 {
    if MSVCRT_CHECK_PMT!(value != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *value = MSVCRT__osver as i32;
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll dll data items
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
//
// #include <fcntl.h>
// #include <math.h>
// #include "msvcrt.h"
// #include <winnls.h>
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// static WCHAR **initial_wargv;
// static int initial_argc;
// int MSVCRT___argc = 0;
// static int wargc_expand;
// unsigned int MSVCRT__commode = 0;
// int MSVCRT__fmode = 0;
// unsigned int MSVCRT__osver = 0;
// unsigned int MSVCRT__osplatform = 0;
// unsigned int MSVCRT__winmajor = 0;
// unsigned int MSVCRT__winminor = 0;
// unsigned int MSVCRT__winver = 0;
// #ifdef _CRTDLL
// unsigned int CRTDLL__basemajor_dll = 0;
// unsigned int CRTDLL__baseminor_dll = 0;
// unsigned int CRTDLL__baseversion_dll = 0;
// unsigned int CRTDLL__cpumode_dll = 1;
// unsigned int CRTDLL__osmode_dll = 1;
// #endif
// unsigned int MSVCRT___setlc_active = 0;
// unsigned int MSVCRT___unguarded_readlc_active = 0;
// double MSVCRT__HUGE = 0;
// char **MSVCRT___argv = NULL;
// wchar_t **MSVCRT___wargv = NULL;
// static wchar_t **wargv_expand;
// char *MSVCRT__acmdln = NULL;
// wchar_t *MSVCRT__wcmdln = NULL;
// char **MSVCRT__environ = NULL;
// wchar_t **MSVCRT__wenviron = NULL;
// char **MSVCRT___initenv = NULL;
// wchar_t **MSVCRT___winitenv = NULL;
// int MSVCRT_app_type = 0;
// char* MSVCRT__pgmptr = NULL;
// WCHAR* MSVCRT__wpgmptr = NULL;
//
// static char **build_argv( WCHAR **wargv )
// {
//     int argc;
//     char *p, **argv;
//     DWORD total = 0;
//
//     for (argc = 0; wargv[argc]; argc++)
//         total += WideCharToMultiByte( CP_ACP, 0, wargv[argc], -1, NULL, 0, NULL, NULL );
//
//     argv = HeapAlloc( GetProcessHeap(), 0, total + (argc + 1) * sizeof(*argv) );
//     p = (char *)(argv + argc + 1);
//     for (argc = 0; wargv[argc]; argc++)
//     {
//         DWORD reslen = WideCharToMultiByte( CP_ACP, 0, wargv[argc], -1, p, total, NULL, NULL );
//         argv[argc] = p;
//         p += reslen;
//         total -= reslen;
//     }
//     argv[argc] = NULL;
//     return argv;
// }
//
// static WCHAR **cmdline_to_argv( const WCHAR *src, int *ret_argc )
// {
//     WCHAR **argv, *arg, *dst;
//     int argc, in_quotes = 0, bcount = 0, len = wcslen(src) + 1;
//
//     argc = 2 + len / 2;
//     argv = HeapAlloc( GetProcessHeap(), 0, argc * sizeof(*argv) + len * sizeof(WCHAR) );
//     arg = dst = (WCHAR *)(argv + argc);
//     argc = 0;
//     while (*src)
//     {
//         if ((*src == ' ' || *src == '\t') && !in_quotes)
//         {
//             /* skip the remaining spaces */
//             while (*src == ' ' || *src == '\t') src++;
//             if (!*src) break;
//             /* close the argument and copy it */
//             *dst++ = 0;
//             argv[argc++] = arg;
//             /* start with a new argument */
//             arg = dst;
//             bcount = 0;
//         }
//         else if (*src == '\\')
//         {
//             *dst++ = *src++;
//             bcount++;
//         }
//         else if (*src == '"')
//         {
//             if ((bcount & 1) == 0)
//             {
//                 /* Preceded by an even number of '\', this is half that
//                  * number of '\', plus a '"' which we discard.
//                  */
//                 dst -= bcount / 2;
//                 src++;
//                 if (in_quotes && *src == '"') *dst++ = *src++;
// #ifdef __REACTOS__
//                 in_quotes = !in_quotes;
// #else
//                 else in_quotes = !in_quotes;
// #endif
//             }
//             else
//             {
//                 /* Preceded by an odd number of '\', this is half that
//                  * number of '\' followed by a '"'
//                  */
//                 dst -= bcount / 2 + 1;
//                 *dst++ = *src++;
//             }
//             bcount = 0;
//         }
//         else  /* a regular character */
//         {
//             *dst++ = *src++;
//             bcount = 0;
//         }
//     }
//     *dst = 0;
//     argv[argc++] = arg;
//     argv[argc] = NULL;
//     *ret_argc = argc;
//     return argv;
// }
//
// #ifdef __REACTOS__
// static CHAR **cmdline_to_argvA( const CHAR *src, int *ret_argc )
// {
//     CHAR **argv, *arg, *dst;
//     int argc, in_quotes = 0, bcount = 0, len = strlen(src) + 1;
//
//     argc = 2 + len / 2;
//     argv = HeapAlloc( GetProcessHeap(), 0, argc * sizeof(*argv) + len * sizeof(CHAR) );
//     arg = dst = (CHAR *)(argv + argc);
//     argc = 0;
//     while (*src)
//     {
//         if ((*src == ' ' || *src == '\t') && !in_quotes)
//         {
//             /* skip the remaining spaces */
//             while (*src == ' ' || *src == '\t') src++;
//             if (!*src) break;
//             /* close the argument and copy it */
//             *dst++ = 0;
//             argv[argc++] = arg;
//             /* start with a new argument */
//             arg = dst;
//             bcount = 0;
//         }
//         else if (*src == '\\')
//         {
//             *dst++ = *src++;
//             bcount++;
//         }
//         else if (*src == '"')
//         {
//             if ((bcount & 1) == 0)
//             {
//                 /* Preceded by an even number of '\', this is half that
//                  * number of '\', plus a '"' which we discard.
//                  */
//                 dst -= bcount / 2;
//                 src++;
//                 if (in_quotes && *src == '"') *dst++ = *src++;
// #ifdef __REACTOS__
//                 in_quotes = !in_quotes;
// #else
//                 else in_quotes = !in_quotes;
// #endif
//             }
//             else
//             {
//                 /* Preceded by an odd number of '\', this is half that
//                  * number of '\' followed by a '"'
//                  */
//                 dst -= bcount / 2 + 1;
//                 *dst++ = *src++;
//             }
//             bcount = 0;
//         }
//         else  /* a regular character */
//         {
//             *dst++ = *src++;
//             bcount = 0;
//         }
//     }
//     *dst = 0;
//     argv[argc++] = arg;
//     argv[argc] = NULL;
//     *ret_argc = argc;
//     return argv;
// }
// #endif
//
// typedef void (CDECL *_INITTERMFUN)(void);
// typedef int (CDECL *_INITTERM_E_FN)(void);
//
// /***********************************************************************
//  *		__p___argc (MSVCRT.@)
//  */
// int*  __p___argc(void) { return &MSVCRT___argc; }
//
// /***********************************************************************
//  *		__p__commode (MSVCRT.@)
//  */
// unsigned int*  __p__commode(void) { return &MSVCRT__commode; }
//
//
// /***********************************************************************
//  *              __p__pgmptr (MSVCRT.@)
//  */
// char**  __p__pgmptr(void) { return &MSVCRT__pgmptr; }
//
// /***********************************************************************
//  *              __p__wpgmptr (MSVCRT.@)
//  */
// WCHAR**  __p__wpgmptr(void) { return &MSVCRT__wpgmptr; }
//
// /***********************************************************************
//  *              _get_pgmptr (MSVCRT.@)
//  */
// int CDECL _get_pgmptr(char** p)
// {
//   if (!MSVCRT_CHECK_PMT(p)) return EINVAL;
//
//   *p = MSVCRT__pgmptr;
//   return 0;
// }
//
// /***********************************************************************
//  *              _get_wpgmptr (MSVCRT.@)
//  */
// int CDECL _get_wpgmptr(WCHAR** p)
// {
//   if (!MSVCRT_CHECK_PMT(p)) return EINVAL;
//   *p = MSVCRT__wpgmptr;
//   return 0;
// }
//
// /***********************************************************************
//  *		__p__fmode (MSVCRT.@)
//  */
// int*  __p__fmode(void) { return &MSVCRT__fmode; }
//
// /***********************************************************************
//  *              _set_fmode (MSVCRT.@)
//  */
// int CDECL _set_fmode(int mode)
// {
//     /* TODO: support _O_WTEXT */
//     if(!MSVCRT_CHECK_PMT(mode==_O_TEXT || mode==_O_BINARY))
//         return EINVAL;
//
//     MSVCRT__fmode = mode;
//     return 0;
// }
//
// /***********************************************************************
//  *              _get_fmode (MSVCRT.@)
//  */
// int CDECL _get_fmode(int *mode)
// {
//     if(!MSVCRT_CHECK_PMT(mode))
//         return EINVAL;
//
//     *mode = MSVCRT__fmode;
//     return 0;
// }
//
// /***********************************************************************
//  *		__p__osver (MSVCRT.@)
//  */
// unsigned int*  __p__osver(void) { return &MSVCRT__osver; }
//
// /***********************************************************************
//  *		__p__winmajor (MSVCRT.@)
//  */
// unsigned int*  __p__winmajor(void) { return &MSVCRT__winmajor; }
//
// /***********************************************************************
//  *		__p__winminor (MSVCRT.@)
//  */
// unsigned int*  __p__winminor(void) { return &MSVCRT__winminor; }
//
// /***********************************************************************
//  *		__p__winver (MSVCRT.@)
//  */
// unsigned int*  __p__winver(void) { return &MSVCRT__winver; }
//
// /*********************************************************************
//  *		__p__acmdln (MSVCRT.@)
//  */
// char**  __p__acmdln(void) { return &MSVCRT__acmdln; }
//
// /*********************************************************************
//  *		__p__wcmdln (MSVCRT.@)
//  */
// wchar_t**  __p__wcmdln(void) { return &MSVCRT__wcmdln; }
//
// /*********************************************************************
//  *		__p___argv (MSVCRT.@)
//  */
// char***  __p___argv(void) { return &MSVCRT___argv; }
//
// /*********************************************************************
//  *		__p___wargv (MSVCRT.@)
//  */
// wchar_t***  __p___wargv(void) { return &MSVCRT___wargv; }
//
// /*********************************************************************
//  *		__p__environ (MSVCRT.@)
//  */
// char***  __p__environ(void)
// {
//   return &MSVCRT__environ;
// }
//
// /*********************************************************************
//  *		__p__wenviron (MSVCRT.@)
//  */
// wchar_t***  __p__wenviron(void)
// {
//   return &MSVCRT__wenviron;
// }
//
// /*********************************************************************
//  *		__p___initenv (MSVCRT.@)
//  */
// char***  __p___initenv(void) { return &MSVCRT___initenv; }
//
// /*********************************************************************
//  *		__p___winitenv (MSVCRT.@)
//  */
// wchar_t***  __p___winitenv(void) { return &MSVCRT___winitenv; }
//
// /*********************************************************************
//  *		_get_osplatform (MSVCRT.@)
//  */
// int CDECL _get_osplatform(int *pValue)
// {
//     if (!MSVCRT_CHECK_PMT(pValue != NULL)) return EINVAL;
//     *pValue = MSVCRT__osplatform;
//     return 0;
// }
//
// /* INTERNAL: Create a wide string from an ascii string */
// wchar_t *msvcrt_wstrdupa(const char *str)
// {
//   const unsigned int len = strlen(str) + 1 ;
//   wchar_t *wstr = malloc(len* sizeof (wchar_t));
//   if (!wstr)
//     return NULL;
//    MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED,str,len,wstr,len);
//   return wstr;
// }
//
// /*********************************************************************
//  *		___unguarded_readlc_active_add_func (MSVCRT.@)
//  */
// unsigned int *  ___unguarded_readlc_active_add_func(void)
// {
//   return &MSVCRT___unguarded_readlc_active;
// }
//
// /*********************************************************************
//  *		___setlc_active_func (MSVCRT.@)
//  */
// unsigned int CDECL ___setlc_active_func(void)
// {
//   return MSVCRT___setlc_active;
// }
//
// /* INTERNAL: Since we can't rely on Winelib startup code calling w/getmainargs,
//  * we initialise data values during DLL loading. When called by a native
//  * program we simply return the data we've already initialised. This also means
//  * you can call multiple times without leaking
//  */
// void msvcrt_init_args(void)
// {
//   OSVERSIONINFOW osvi;
//
//   MSVCRT__acmdln = _strdup( GetCommandLineA() );
//   MSVCRT__wcmdln = _wcsdup( GetCommandLineW() );
//   initial_wargv  = cmdline_to_argv( GetCommandLineW(), &initial_argc );
//   MSVCRT___argc  = initial_argc;
//   MSVCRT___wargv = initial_wargv;
//   MSVCRT___argv  = build_argv( initial_wargv );
//
//   TRACE("got %s, wide = %s argc=%d\n", debugstr_a(MSVCRT__acmdln),
//         debugstr_w(MSVCRT__wcmdln),MSVCRT___argc);
//
//   osvi.dwOSVersionInfoSize = sizeof(OSVERSIONINFOW);
//   GetVersionExW( &osvi );
//   MSVCRT__winver     = (osvi.dwMajorVersion << 8) | osvi.dwMinorVersion;
//   MSVCRT__winmajor   = osvi.dwMajorVersion;
//   MSVCRT__winminor   = osvi.dwMinorVersion;
//   MSVCRT__osver      = osvi.dwBuildNumber;
//   MSVCRT__osplatform = osvi.dwPlatformId;
//   TRACE( "winver %08x winmajor %08x winminor %08x osver %08x\n",
//           MSVCRT__winver, MSVCRT__winmajor, MSVCRT__winminor, MSVCRT__osver);
// #ifdef _CRTDLL
//   CRTDLL__baseversion_dll = (GetVersion() >> 16);
//   CRTDLL__basemajor_dll   = CRTDLL__baseversion_dll >> 8;
//   CRTDLL__baseminor_dll   = CRTDLL__baseversion_dll & 0xff;
// #endif
//
//   MSVCRT__HUGE = HUGE_VAL;
//   MSVCRT___setlc_active = 0;
//   MSVCRT___unguarded_readlc_active = 0;
// #ifndef __REACTOS__
//   MSVCRT__fmode = _O_TEXT;
// #endif
//
//   env_init(FALSE, FALSE);
//
//   MSVCRT__pgmptr = HeapAlloc(GetProcessHeap(), 0, MAX_PATH);
//   if (MSVCRT__pgmptr)
//   {
//     if (!GetModuleFileNameA(core::ptr::null_mut(), MSVCRT__pgmptr, MAX_PATH))
//       MSVCRT__pgmptr[0] = '\0';
//     else
//       MSVCRT__pgmptr[MAX_PATH - 1] = '\0';
//   }
//
//   MSVCRT__wpgmptr = HeapAlloc(GetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR));
//   if (MSVCRT__wpgmptr)
//   {
//     if (!GetModuleFileNameW(core::ptr::null_mut(), MSVCRT__wpgmptr, MAX_PATH))
//       MSVCRT__wpgmptr[0] = '\0';
//     else
//       MSVCRT__wpgmptr[MAX_PATH - 1] = '\0';
//   }
// }
//
// /* INTERNAL: free memory used by args */
// void msvcrt_free_args(void)
// {
//   /* FIXME: more things to free */
//   HeapFree(GetProcessHeap(), 0, MSVCRT___argv);
//   HeapFree(GetProcessHeap(), 0, MSVCRT__pgmptr);
//   HeapFree(GetProcessHeap(), 0, MSVCRT__wpgmptr);
//   HeapFree(GetProcessHeap(), 0, wargv_expand);
// }
//
// static int build_expanded_wargv(int *argc, wchar_t **argv)
// {
//     int i, size=0, args_no=0, path_len;
//     BOOL is_expandable;
//     HANDLE h;
//
//     args_no = 0;
//     for(i=0; i < initial_argc; i++) {
//         WIN32_FIND_DATAW data;
//         int len = 0;
//
//         is_expandable = FALSE;
//         for(path_len = wcslen(initial_wargv[i])-1; path_len>=0; path_len--) {
//             if(initial_wargv[i][path_len]=='*' || initial_wargv[i][path_len]=='?')
//                 is_expandable = TRUE;
//             else if(initial_wargv[i][path_len]=='\\' || initial_wargv[i][path_len]=='/')
//                 break;
//         }
//         path_len++;
//
//         if(is_expandable)
//             h = FindFirstFileW(initial_wargv[i], &data);
//         else
//             h = INVALID_HANDLE_VALUE;
//
//         if(h != INVALID_HANDLE_VALUE) {
//             do {
//                 if(data.cFileName[0]=='.' && (data.cFileName[1]=='\0' ||
//                             (data.cFileName[1]=='.' && data.cFileName[2]=='\0')))
//                     continue;
//
//                 len = wcslen(data.cFileName)+1;
//                 if(argv) {
//                     argv[args_no] = (wchar_t*)(argv+*argc+1)+size;
//                     memcpy(argv[args_no], initial_wargv[i], path_len*sizeof(wchar_t));
//                     memcpy(argv[args_no]+path_len, data.cFileName, len*sizeof(wchar_t));
//                 }
//                 args_no++;
//                 size += len+path_len;
//             }while(FindNextFileW(h, &data));
//             FindClose(h);
//         }
//
//         if(!len) {
//             len = wcslen(initial_wargv[i])+1;
//             if(argv) {
//                 argv[args_no] = (wchar_t*)(argv+*argc+1)+size;
//                 memcpy(argv[args_no], initial_wargv[i], len*sizeof(wchar_t));
//             }
//             args_no++;
//             size += len;
//         }
//     }
//
//     if(argv)
//         argv[args_no] = NULL;
//     size *= sizeof(wchar_t);
//     size += (args_no+1)*sizeof(wchar_t*);
//     *argc = args_no;
//     return size;
// }
//
// /*********************************************************************
//  *		__wgetmainargs (MSVCRT.@)
//  */
// int CDECL __wgetmainargs(int *argc, wchar_t** *wargv, wchar_t** *wenvp,
//                           int expand_wildcards, int *new_mode)
// {
//     TRACE("(%p,%p,%p,%d,%p).\n", argc, wargv, wenvp, expand_wildcards, new_mode);
//
//     if (expand_wildcards) {
//         HeapFree(GetProcessHeap(), 0, wargv_expand);
//         wargv_expand = HeapAlloc(GetProcessHeap(), 0,
//                 build_expanded_wargv(&wargc_expand, NULL));
//         if (wargv_expand) {
//             build_expanded_wargv(&wargc_expand, wargv_expand);
//
//             MSVCRT___argc = wargc_expand;
//             MSVCRT___wargv = wargv_expand;
//         }else {
//             expand_wildcards = 0;
//         }
//     }
//     if (!expand_wildcards) {
// #ifdef __REACTOS__
//         if ((MSVCRT___wargv == NULL) || (MSVCRT___argc == 0))
//         {
//             MSVCRT___wargv = cmdline_to_argv(MSVCRT__wcmdln, &MSVCRT___argc);
//         }
// #else
//         MSVCRT___argc = initial_argc;
//         MSVCRT___wargv = initial_wargv;
// #endif
//     }
//
//     env_init(TRUE, FALSE);
//
//     *argc = MSVCRT___argc;
//     *wargv = MSVCRT___wargv;
//     *wenvp = MSVCRT__wenviron;
//     if (new_mode)
//         _set_new_mode( *new_mode );
//     return 0;
// }
//
// /*********************************************************************
//  *		__getmainargs (MSVCRT.@)
//  */
// int CDECL __getmainargs(int *argc, char** *argv, char** *envp,
//                          int expand_wildcards, int *new_mode)
// {
//     TRACE("(%p,%p,%p,%d,%p).\n", argc, argv, envp, expand_wildcards, new_mode);
//
//     if (expand_wildcards) {
//         HeapFree(GetProcessHeap(), 0, wargv_expand);
//         wargv_expand = HeapAlloc(GetProcessHeap(), 0,
//                 build_expanded_wargv(&wargc_expand, NULL));
//         if (wargv_expand) {
//             build_expanded_wargv(&wargc_expand, wargv_expand);
//
//             MSVCRT___argc = wargc_expand;
//             MSVCRT___argv = build_argv( wargv_expand );
//         }else {
//             expand_wildcards = 0;
//         }
//     }
//     if (!expand_wildcards) {
// #ifdef __REACTOS__
//         if ((MSVCRT___argv == NULL) || (MSVCRT___argc == 0))
//         {
//             MSVCRT___argv = cmdline_to_argvA(MSVCRT__acmdln, &MSVCRT___argc);
//         }
// #else
//         MSVCRT___argc = initial_argc;
//         MSVCRT___argv = build_argv( initial_wargv );
// #endif
//     }
//
//     *argc = MSVCRT___argc;
//     *argv = MSVCRT___argv;
//     *envp = MSVCRT__environ;
//
//     if (new_mode)
//         _set_new_mode( *new_mode );
//     return 0;
// }
//
// #ifdef _CRTDLL
// /*********************************************************************
//  *                  __GetMainArgs  (CRTDLL.@)
//  */
// void CDECL __GetMainArgs( int *argc, char ***argv, char ***envp, int expand_wildcards )
// {
//     int new_mode = 0;
//     __getmainargs( argc, argv, envp, expand_wildcards, &new_mode );
// }
// #endif
//
// /*********************************************************************
//  *		_initterm (MSVCRT.@)
//  */
// void CDECL _initterm(_INITTERMFUN *start,_INITTERMFUN *end)
// {
//   _INITTERMFUN* current = start;
//
//   TRACE("(%p,%p)\n",start,end);
//   while (current<end)
//   {
//     if (*current)
//     {
//       TRACE("Call init function %p\n",*current);
//       (**current)();
//       TRACE("returned\n");
//     }
//     current++;
//   }
// }
//
// /*********************************************************************
//  *  _initterm_e (MSVCRT.@)
//  *
//  * call an array of application initialization functions and report the return value
//  */
// int CDECL _initterm_e(_INITTERM_E_FN *table, _INITTERM_E_FN *end)
// {
//     int res = 0;
//
//     TRACE("(%p, %p)\n", table, end);
//
//     while (!res && table < end) {
//         if (*table) {
//             TRACE("calling %p\n", **table);
//             res = (**table)();
//             if (res)
//                 TRACE("function %p failed: %#x\n", *table, res);
//
//         }
//         table++;
//     }
//     return res;
// }
//
// /*********************************************************************
//  *		__set_app_type (MSVCRT.@)
//  */
// void CDECL __set_app_type(int app_type)
// {
//   TRACE("(%d) %s application\n", app_type, app_type == 2 ? "Gui" : "Console");
//   MSVCRT_app_type = app_type;
// }
//
//
// /*********************************************************************
//  *    _get_winmajor (MSVCRT.@)
//  */
// int CDECL _get_winmajor(int* value)
// {
//     if (!MSVCRT_CHECK_PMT(value != NULL)) return EINVAL;
//     *value = MSVCRT__winmajor;
//     return 0;
// }
//
// /*********************************************************************
//  *    _get_winminor (MSVCRT.@)
//  */
// int CDECL _get_winminor(int* value)
// {
//     if (!MSVCRT_CHECK_PMT(value != NULL)) return EINVAL;
//     *value = MSVCRT__winminor;
//     return 0;
// }
//
// /*********************************************************************
//  *    _get_osver (MSVCRT.@)
//  */
// int CDECL _get_osver(int* value)
// {
//     if (!MSVCRT_CHECK_PMT(value != NULL)) return EINVAL;
//     *value = MSVCRT__osver;
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
