use crate::prelude::*;
use core::ffi::c_void;
use crate::{wcschr, wcscpy, wcslen};
#[no_mangle]
pub unsafe extern "C" fn env_init(unicode: BOOL, modif: BOOL) -> core::ffi::c_int {
    if unicode == 0 && MSVCRT___initenv.is_null() {
        let environ_strings: *mut core::ffi::c_char = GetEnvironmentStringsA();
        let mut count: core::ffi::c_int = 1;
        let mut len: core::ffi::c_int = 1;
        let mut i: core::ffi::c_int = 0;
        let mut ptr: *mut core::ffi::c_char;

        ptr = environ_strings;
        while *ptr != 0 {
            if *ptr != b'=' as core::ffi::c_char {
                count += 1;
            }
            len += strlen(ptr) as core::ffi::c_int + 1;
            ptr = ptr.add((strlen(ptr) as usize) + 1 as usize);
        }
        // C: `count * sizeof(*MSVCRT___initenv)`. `sizeof` does NOT evaluate its operand, so
        // dereferencing a null pointer there is legal C — but `size_of_val(&*p)` DOES form a
        // reference, and the `if` above just established this pointer is null. Same size, and
        // the spelling the realloc sites in this file already use.
        MSVCRT___initenv = malloc(
            count as usize * core::mem::size_of::<*mut core::ffi::c_char>() + len as usize,
        ) as *mut *mut core::ffi::c_char;
        if MSVCRT___initenv.is_null() {
            FreeEnvironmentStringsA(environ_strings);
            return -1;
        }

        memcpy(
            core::ptr::addr_of_mut!(*MSVCRT___initenv.add(count as usize)) as *mut core::ffi::c_void,
            environ_strings as *const core::ffi::c_void,
            len as usize,
        );
        ptr = MSVCRT___initenv.add(count as usize) as *mut core::ffi::c_char;
        while *ptr != 0 {
            if *ptr != b'=' as core::ffi::c_char {
                *MSVCRT___initenv.add(i as usize) = ptr;
                i += 1;
            }
            ptr = ptr.add((strlen(ptr) as usize) + 1 as usize);
        }
        *MSVCRT___initenv.add(i as usize) = core::ptr::null_mut();
        FreeEnvironmentStringsA(environ_strings);

        MSVCRT__environ = MSVCRT___initenv;
    }

    if unicode == 0 && modif != 0 && MSVCRT__environ == MSVCRT___initenv {
        let mut i: core::ffi::c_int = 0;

        while !(*MSVCRT___initenv.add(i as usize)).is_null() {
            i += 1;
        }
        MSVCRT__environ = malloc(
            (i + 1) as usize * core::mem::size_of::<*mut core::ffi::c_char>(),
        ) as *mut *mut core::ffi::c_char;
        if MSVCRT__environ.is_null() {
            return -1;
        }
        i = 0;
        while !(*MSVCRT___initenv.add(i as usize)).is_null() {
            *MSVCRT__environ.add(i as usize) = strdup(*MSVCRT___initenv.add(i as usize));
            i += 1;
        }
        *MSVCRT__environ.add(i as usize) = core::ptr::null_mut();
    }

    if unicode != 0 && MSVCRT___winitenv.is_null() {
        let wenviron_strings: *mut wchar_t = GetEnvironmentStringsW();
        let mut count: core::ffi::c_int = 1;
        let mut len: core::ffi::c_int = 1;
        let mut i: core::ffi::c_int = 0;
        let mut wptr: *mut wchar_t;

        wptr = wenviron_strings;
        while *wptr != 0 {
            if *wptr != b'=' as wchar_t {
                count += 1;
            }
            len += wcslen(wptr) as core::ffi::c_int + 1;
            wptr = wptr.add((wcslen(wptr) as usize) + 1 as usize);
        }
        // Same as `env_init`'s narrow half: C's `sizeof(*p)` never evaluates `p`, so the C is
        // fine dereferencing a pointer the `if` above proved null; `size_of_val(&*p)` is not.
        MSVCRT___winitenv = malloc(
            count as usize * core::mem::size_of::<*mut wchar_t>()
                + len as usize * core::mem::size_of::<wchar_t>(),
        ) as *mut *mut wchar_t;
        if MSVCRT___winitenv.is_null() {
            FreeEnvironmentStringsW(wenviron_strings);
            return -1;
        }

        memcpy(
            core::ptr::addr_of_mut!(*MSVCRT___winitenv.add(count as usize)) as *mut core::ffi::c_void,
            wenviron_strings as *const core::ffi::c_void,
            len as usize * core::mem::size_of::<wchar_t>(),
        );
        wptr = MSVCRT___winitenv.add(count as usize) as *mut wchar_t;
        while *wptr != 0 {
            if *wptr != b'=' as wchar_t {
                *MSVCRT___winitenv.add(i as usize) = wptr;
                i += 1;
            }
            wptr = wptr.add((wcslen(wptr) as usize) + 1 as usize);
        }
        *MSVCRT___winitenv.add(i as usize) = core::ptr::null_mut();
        FreeEnvironmentStringsW(wenviron_strings);

        MSVCRT__wenviron = MSVCRT___winitenv;
    }

    if unicode != 0 && modif != 0 && MSVCRT__wenviron == MSVCRT___winitenv {
        let mut i: core::ffi::c_int = 0;

        while !(*MSVCRT___winitenv.add(i as usize)).is_null() {
            i += 1;
        }
        MSVCRT__wenviron = malloc(
            (i + 1) as usize * core::mem::size_of::<*mut wchar_t>(),
        ) as *mut *mut wchar_t;
        if MSVCRT__wenviron.is_null() {
            return -1;
        }
        i = 0;
        while !(*MSVCRT___winitenv.add(i as usize)).is_null() {
            *MSVCRT__wenviron.add(i as usize) = wcsdup(*MSVCRT___winitenv.add(i as usize));
            i += 1;
        }
        *MSVCRT__wenviron.add(i as usize) = core::ptr::null_mut();
    }

    0
}
unsafe extern "C" fn env_get_index(name: *const core::ffi::c_char) -> core::ffi::c_int {
    let mut i: core::ffi::c_int;
    let len: core::ffi::c_int;

    len = strlen(name) as core::ffi::c_int;
    i = 0;
    while !(*MSVCRT__environ.offset(i as isize)).is_null() {
        if strnicmp(name, *MSVCRT__environ.offset(i as isize), len as usize) == 0
            && *(*MSVCRT__environ.offset(i as isize)).offset(len as isize) == b'=' as core::ffi::c_char
        {
            return i;
        }
        i += 1;
    }
    i
}
unsafe extern "C" fn wenv_get_index(name: *const wchar_t) -> i32 {
    let mut i: i32;
    let len: i32;

    len = wcslen(name) as i32;
    i = 0;
    while !(*MSVCRT__wenviron.offset(i as isize)).is_null() {
        if wcsnicmp(name, *MSVCRT__wenviron.offset(i as isize), len as usize) == 0
            && *(*MSVCRT__wenviron.offset(i as isize)).offset(len as isize) == '=' as wchar_t
        {
            return i;
        }
        i += 1;
    }
    i
}
unsafe extern "C" fn env_set(env: *mut *mut core::ffi::c_char, wenv: *mut *mut wchar_t) -> core::ffi::c_int {
    let weq = wcschr(*wenv, '=' as wchar_t);
    let eq = strchr(*env, '=' as core::ffi::c_int);
    let mut idx: core::ffi::c_int;

    *weq = 0;
    if SetEnvironmentVariableW(*wenv, if *weq.offset(1) != 0 { weq.offset(1) } else { core::ptr::null_mut() }) == 0
        && GetLastError() != ERROR_ENVVAR_NOT_FOUND {
        return -1;
    }

    if env_init((FALSE) as i32, (TRUE) as i32) != 0 { return -1; }
    *eq = 0;
    idx = env_get_index(*env);
    *eq = '=' as core::ffi::c_char;
    if *eq.offset(1) == 0 {
        free((*MSVCRT__environ.offset(idx as isize)).cast());
        while !(*MSVCRT__environ.offset(idx as isize)).is_null() {
            *MSVCRT__environ.offset(idx as isize) = *MSVCRT__environ.offset((idx + 1) as isize);
            idx += 1;
        }
    } else if !(*MSVCRT__environ.offset(idx as isize)).is_null() {
        free((*MSVCRT__environ.offset(idx as isize)).cast());
        *MSVCRT__environ.offset(idx as isize) = *env;
        *env = core::ptr::null_mut();
    } else {
        let new_env = realloc(MSVCRT__environ as *mut core::ffi::c_void, ((idx + 2) as usize) * core::mem::size_of::<*mut core::ffi::c_char>()) as *mut *mut core::ffi::c_char;
        if new_env.is_null() { return -1; }
        MSVCRT__environ = new_env;
        *MSVCRT__environ.offset(idx as isize) = *env;
        *MSVCRT__environ.offset((idx + 1) as isize) = core::ptr::null_mut();
        *env = core::ptr::null_mut();
    }

    if MSVCRT__wenviron.is_null() { return 0; }
    if MSVCRT__wenviron == MSVCRT___winitenv {
        if env_init((TRUE) as i32, (TRUE) as i32) != 0 { return -1; }
    }
    idx = wenv_get_index(*wenv);
    *weq = '=' as wchar_t;
    if *weq.offset(1) == 0 {
        free((*MSVCRT__wenviron.offset(idx as isize)).cast());
        while !(*MSVCRT__wenviron.offset(idx as isize)).is_null() {
            *MSVCRT__wenviron.offset(idx as isize) = *MSVCRT__wenviron.offset((idx + 1) as isize);
            idx += 1;
        }
    } else if !(*MSVCRT__wenviron.offset(idx as isize)).is_null() {
        free((*MSVCRT__wenviron.offset(idx as isize)).cast());
        *MSVCRT__wenviron.offset(idx as isize) = *wenv;
        *wenv = core::ptr::null_mut();
    } else {
        let new_env = realloc(MSVCRT__wenviron as *mut core::ffi::c_void, ((idx + 2) as usize) * core::mem::size_of::<*mut wchar_t>()) as *mut *mut wchar_t;
        if new_env.is_null() { return -1; }
        MSVCRT__wenviron = new_env;
        *MSVCRT__wenviron.offset(idx as isize) = *wenv;
        *MSVCRT__wenviron.offset((idx + 1) as isize) = core::ptr::null_mut();
        *wenv = core::ptr::null_mut();
    }
    0
}
unsafe extern "C" fn getenv_helper(name: *const core::ffi::c_char) -> *mut core::ffi::c_char {
    let idx: core::ffi::c_int;

    if name.is_null() {
        return core::ptr::null_mut();
    }

    idx = env_get_index(name);
    if (*MSVCRT__environ.offset(idx as isize)).is_null() {
        return core::ptr::null_mut();
    }
    strchr(*MSVCRT__environ.offset(idx as isize), b'=' as core::ffi::c_int).offset(1)
}

#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const core::ffi::c_char) -> *mut core::ffi::c_char {
    let ret: *mut core::ffi::c_char;

    if MSVCRT_CHECK_PMT!(!name.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    _lock(_ENV_LOCK);
    ret = getenv_helper(name);
    _unlock(_ENV_LOCK);
    ret
}
unsafe extern "C" fn wgetenv_helper(name: *const wchar_t) -> *mut wchar_t {
    let idx: i32;

    if name.is_null() {
        return core::ptr::null_mut();
    }
    if env_init((TRUE) as i32, (FALSE) as i32) != 0 {
        return core::ptr::null_mut();
    }

    idx = wenv_get_index(name);
    if (*MSVCRT__wenviron.offset(idx as isize)).is_null() {
        return core::ptr::null_mut();
    }
    wcschr(*MSVCRT__wenviron.offset(idx as isize), ('=' as i32) as u16).offset(1)
}

#[no_mangle]
pub unsafe extern "C" fn _wgetenv(name: *const wchar_t) -> *mut wchar_t {
    let ret: *mut wchar_t;

    if MSVCRT_CHECK_PMT!(!name.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    _lock(_ENV_LOCK);
    ret = wgetenv_helper(name);
    _unlock(_ENV_LOCK);
    ret
}
unsafe extern "C" fn putenv_helper(name: *const core::ffi::c_char, val: *const core::ffi::c_char, eq: *const core::ffi::c_char) -> core::ffi::c_int {
    let mut wenv: *mut wchar_t;
    let mut env: *mut core::ffi::c_char;
    let mut r: core::ffi::c_int;

    if !eq.is_null() {
        env = strdup(name);
        if env.is_null() {
            return -1;
        }
    } else {
        let name_len = strlen(name) as core::ffi::c_int;

        r = strlen(val) as core::ffi::c_int;
        env = malloc((name_len + r + 2) as usize) as *mut core::ffi::c_char;
        if env.is_null() {
            return -1;
        }
        memcpy(env as *mut core::ffi::c_void, name as *const core::ffi::c_void, name_len as usize);
        *env.offset(name_len as isize) = b'=' as core::ffi::c_char;
        strcpy(env.offset((name_len + 1) as isize), val);
    }

    wenv = msvcrt_wstrdupa(env);
    if wenv.is_null() {
        free(env as *mut core::ffi::c_void);
        return -1;
    }

    _lock(_ENV_LOCK);
    r = env_set(core::ptr::addr_of_mut!(env), core::ptr::addr_of_mut!(wenv));
    _unlock(_ENV_LOCK);
    free(env as *mut core::ffi::c_void);
    free(wenv as *mut core::ffi::c_void);
    r
}
unsafe extern "C" fn msvcrt_astrdupw(wstr: *const wchar_t) -> *mut i8 {
    let len: core::ffi::c_uint = WideCharToMultiByte(CP_ACP, 0, wstr, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as core::ffi::c_uint;
    let str = malloc(len as usize * core::mem::size_of::<i8>()) as *mut i8;

    if str.is_null() {
        return core::ptr::null_mut();
    }
    WideCharToMultiByte(CP_ACP, 0, wstr, -1, str, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
    str
}
unsafe extern "C" fn wputenv_helper(name: *const wchar_t, val: *const wchar_t, eq: *const wchar_t) -> core::ffi::c_int {
    let mut wenv: *mut wchar_t;
    let env: *mut core::ffi::c_char;
    let mut r: core::ffi::c_int;

    _lock(_ENV_LOCK);
    r = env_init((TRUE) as i32, (TRUE) as i32);
    _unlock(_ENV_LOCK);
    if r != 0 {
        return -1;
    }

    if !eq.is_null() {
        wenv = wcsdup(name);
        if wenv.is_null() {
            return -1;
        }
    } else {
        let name_len = wcslen(name) as core::ffi::c_int;

        r = wcslen(val) as core::ffi::c_int;
        wenv = malloc(((name_len + r + 2) as usize) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        if wenv.is_null() {
            return -1;
        }
        memcpy(wenv as *mut core::ffi::c_void, name as *const core::ffi::c_void, (name_len as usize) * core::mem::size_of::<wchar_t>());
        *wenv.offset(name_len as isize) = '=' as wchar_t;
        wcscpy(wenv.offset((name_len + 1) as isize), val);
    }

    env = msvcrt_astrdupw(wenv);
    if env.is_null() {
        free(wenv as *mut core::ffi::c_void);
        return -1;
    }

    _lock(_ENV_LOCK);
    r = env_set(&mut (env as *mut core::ffi::c_char), &mut wenv);
    _unlock(_ENV_LOCK);
    free(env as *mut core::ffi::c_void);
    free(wenv as *mut core::ffi::c_void);
    r
}

#[no_mangle]
pub unsafe extern "C" fn _putenv(str: *const core::ffi::c_char) -> core::ffi::c_int {
    let eq: *const core::ffi::c_char;

    if str.is_null() || {
        eq = strchr(str, '=' as core::ffi::c_int);
        eq.is_null()
    } {
        return -1;
    }

    putenv_helper(str, core::ptr::null_mut(), eq)
}

#[no_mangle]
pub unsafe extern "C" fn _wputenv(str: *const wchar_t) -> core::ffi::c_int {
    let eq: *const wchar_t;

    if str.is_null() || {
        eq = wcschr(str, '=' as wchar_t);
        eq.is_null()
    } {
        return -1;
    }

    wputenv_helper(str, core::ptr::null_mut(), eq)
}

#[no_mangle]
pub unsafe extern "C" fn _putenv_s(name: *const core::ffi::c_char, value: *const core::ffi::c_char) -> errno_t {
    let mut ret: errno_t = 0;

    if MSVCRT_CHECK_PMT!(!name.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!value.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    if putenv_helper(name, value, core::ptr::null_mut()) < 0 {
        msvcrt_set_errno((GetLastError()) as i32);
        ret = *_errno();
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wputenv_s(name: *const wchar_t, value: *const wchar_t) -> errno_t {
    let mut ret: errno_t = 0;

    if MSVCRT_CHECK_PMT!(!name.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!value.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    if wputenv_helper(name, value, core::ptr::null_mut()) < 0 {
        msvcrt_set_errno((GetLastError()) as i32);
        ret = *_errno();
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn getenv_s(
    ret_len: *mut usize,
    buffer: *mut core::ffi::c_char,
    len: usize,
    varname: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let e: *mut core::ffi::c_char;

    if MSVCRT_CHECK_PMT!(!ret_len.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    *ret_len = 0;
    if MSVCRT_CHECK_PMT!(
        (!buffer.is_null() && len > 0) || (buffer.is_null() && len == 0),
    ) == 0
    {
        return (EINVAL) as i32;
    }
    if !buffer.is_null() {
        *buffer.offset(0) = 0;
    }

    _lock(_ENV_LOCK);
    e = getenv_helper(varname);
    if !e.is_null() {
        *ret_len = strlen(e) + 1;
        if len >= *ret_len {
            strcpy(buffer, e);
        }
    }
    _unlock(_ENV_LOCK);

    if e.is_null() || len == 0 {
        return 0;
    }
    if len < *ret_len {
        return (ERANGE) as i32;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wgetenv_s(
    ret_len: *mut usize,
    buffer: *mut wchar_t,
    len: usize,
    varname: *const wchar_t,
) -> core::ffi::c_int {
    let e: *mut wchar_t;

    if MSVCRT_CHECK_PMT!(!ret_len.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    *ret_len = 0;
    if MSVCRT_CHECK_PMT!(
        (!buffer.is_null() && len > 0) || (buffer.is_null() && len == 0),
    ) == 0
    {
        return (EINVAL) as i32;
    }
    if !buffer.is_null() {
        *buffer.add(0) = 0;
    }

    _lock(_ENV_LOCK);
    e = wgetenv_helper(varname);
    if !e.is_null() {
        *ret_len = wcslen(e) + 1;
        if len >= *ret_len {
            wcscpy(buffer, e);
        }
    }
    _unlock(_ENV_LOCK);

    if e.is_null() || len == 0 {
        return 0;
    }
    if len < *ret_len {
        return (ERANGE) as i32;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_environ(ptr: *mut *mut *mut core::ffi::c_char) {
    *ptr = MSVCRT__environ;
}

#[no_mangle]
pub unsafe extern "C" fn _get_wenviron(ptr: *mut *mut *mut wchar_t) {
    *ptr = MSVCRT__wenviron;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll environment functions
//  *
//  * Copyright 1996,1998 Marcus Meissner
//  * Copyright 1996 Jukka Iivonen
//  * Copyright 1997,2000 Uwe Bonnes
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
// #include "msvcrt.h"
// #include "mtdll.h"
// #include <winnls.h>
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// int env_init(BOOL unicode, BOOL modif)
// {
//     if (!unicode && !MSVCRT___initenv)
//     {
//         char *environ_strings = GetEnvironmentStringsA();
//         int count = 1, len = 1, i = 0; /* keep space for the trailing NULLS */
//         char *ptr;
//
//         for (ptr = environ_strings; *ptr; ptr += strlen(ptr) + 1)
//         {
//             /* Don't count environment variables starting with '=' which are command shell specific */
//             if (*ptr != '=') count++;
//             len += strlen(ptr) + 1;
//         }
//         MSVCRT___initenv = malloc(count * sizeof(*MSVCRT___initenv) + len);
//         if (!MSVCRT___initenv)
//         {
//             FreeEnvironmentStringsA(environ_strings);
//             return -1;
//         }
//
//         memcpy(&MSVCRT___initenv[count], environ_strings, len);
//         for (ptr = (char *)&MSVCRT___initenv[count]; *ptr; ptr += strlen(ptr) + 1)
//         {
//             /* Skip special environment strings set by the command shell */
//             if (*ptr != '=') MSVCRT___initenv[i++] = ptr;
//         }
//         MSVCRT___initenv[i] = NULL;
//         FreeEnvironmentStringsA(environ_strings);
//
//         MSVCRT__environ = MSVCRT___initenv;
//     }
//
//     if (!unicode && modif && MSVCRT__environ == MSVCRT___initenv)
//     {
//         int i = 0;
//
//         while(MSVCRT___initenv[i]) i++;
//         MSVCRT__environ = malloc((i + 1) * sizeof(char *));
//         if (!MSVCRT__environ) return -1;
//         for (i = 0; MSVCRT___initenv[i]; i++)
//             MSVCRT__environ[i] = strdup(MSVCRT___initenv[i]);
//         MSVCRT__environ[i] = NULL;
//     }
//
//     if (unicode && !MSVCRT___winitenv)
//     {
//         wchar_t *wenviron_strings = GetEnvironmentStringsW();
//         int count = 1, len = 1, i = 0; /* keep space for the trailing NULLS */
//         wchar_t *wptr;
//
//         for (wptr = wenviron_strings; *wptr; wptr += wcslen(wptr) + 1)
//         {
//             /* Don't count environment variables starting with '=' which are command shell specific */
//             if (*wptr != '=') count++;
//             len += wcslen(wptr) + 1;
//         }
//         MSVCRT___winitenv = malloc(count * sizeof(*MSVCRT___winitenv) + len * sizeof(wchar_t));
//         if (!MSVCRT___winitenv)
//         {
//             FreeEnvironmentStringsW(wenviron_strings);
//             return -1;
//         }
//
//         memcpy(&MSVCRT___winitenv[count], wenviron_strings, len * sizeof(wchar_t));
//         for (wptr = (wchar_t *)&MSVCRT___winitenv[count]; *wptr; wptr += wcslen(wptr) + 1)
//         {
//             /* Skip special environment strings set by the command shell */
//             if (*wptr != '=') MSVCRT___winitenv[i++] = wptr;
//         }
//         MSVCRT___winitenv[i] = NULL;
//         FreeEnvironmentStringsW(wenviron_strings);
//
//         MSVCRT__wenviron = MSVCRT___winitenv;
//     }
//
//     if (unicode && modif && MSVCRT__wenviron == MSVCRT___winitenv)
//     {
//         int i = 0;
//
//         while(MSVCRT___winitenv[i]) i++;
//         MSVCRT__wenviron = malloc((i + 1) * sizeof(wchar_t *));
//         if (!MSVCRT__wenviron) return -1;
//         for (i = 0; MSVCRT___winitenv[i]; i++)
//             MSVCRT__wenviron[i] = wcsdup(MSVCRT___winitenv[i]);
//         MSVCRT__wenviron[i] = NULL;
//     }
//
//     return 0;
// }
//
// static int env_get_index(const char *name)
// {
//     int i, len;
//
//     len = strlen(name);
//     for (i = 0; MSVCRT__environ[i]; i++)
//     {
//         if (!strnicmp(name, MSVCRT__environ[i], len) && MSVCRT__environ[i][len] == '=')
//             return i;
//     }
//     return i;
// }
//
// static int wenv_get_index(const wchar_t *name)
// {
//     int i, len;
//
//     len = wcslen(name);
//     for (i = 0; MSVCRT__wenviron[i]; i++)
//     {
//         if (!wcsnicmp(name, MSVCRT__wenviron[i], len) && MSVCRT__wenviron[i][len] == '=')
//             return i;
//     }
//     return i;
// }
//
// static int env_set(char **env, wchar_t **wenv)
// {
//     wchar_t *weq = wcschr(*wenv, '=');
//     char *eq = strchr(*env, '=');
//     int idx;
//
//     *weq = 0;
//     if (!SetEnvironmentVariableW(*wenv, weq[1] ? weq + 1 : NULL) &&
//             GetLastError() != ERROR_ENVVAR_NOT_FOUND)
//         return -1;
//
//     if (env_init(FALSE, TRUE)) return -1;
//     *eq = 0;
//     idx = env_get_index(*env);
//     *eq = '=';
//     if (!eq[1])
//     {
//         free(MSVCRT__environ[idx]);
//         for(; MSVCRT__environ[idx]; idx++)
//             MSVCRT__environ[idx] = MSVCRT__environ[idx + 1];
//     }
//     else if (MSVCRT__environ[idx])
//     {
//         free(MSVCRT__environ[idx]);
//         MSVCRT__environ[idx] = *env;
//         *env = NULL;
//     }
//     else
//     {
//         char **new_env = realloc(MSVCRT__environ, (idx + 2) * sizeof(*MSVCRT__environ));
//         if (!new_env) return -1;
//         MSVCRT__environ = new_env;
//         MSVCRT__environ[idx] = *env;
//         MSVCRT__environ[idx + 1] = NULL;
//         *env = NULL;
//     }
//
//     if (!MSVCRT__wenviron) return 0;
//     if (MSVCRT__wenviron == MSVCRT___winitenv)
//         if (env_init(TRUE, TRUE)) return -1;
//     idx = wenv_get_index(*wenv);
//     *weq = '=';
//     if (!weq[1])
//     {
//         free(MSVCRT__wenviron[idx]);
//         for(; MSVCRT__wenviron[idx]; idx++)
//             MSVCRT__wenviron[idx] = MSVCRT__wenviron[idx + 1];
//     }
//     else if (MSVCRT__wenviron[idx])
//     {
//         free(MSVCRT__wenviron[idx]);
//         MSVCRT__wenviron[idx] = *wenv;
//         *wenv = NULL;
//     }
//     else
//     {
//         wchar_t **new_env = realloc(MSVCRT__wenviron, (idx + 2) * sizeof(*MSVCRT__wenviron));
//         if (!new_env) return -1;
//         MSVCRT__wenviron = new_env;
//         MSVCRT__wenviron[idx] = *wenv;
//         MSVCRT__wenviron[idx + 1] = NULL;
//         *wenv = NULL;
//     }
//     return 0;
// }
//
// static char * getenv_helper(const char *name)
// {
//     int idx;
//
//     if (!name) return NULL;
//
//     idx = env_get_index(name);
//     if (!MSVCRT__environ[idx]) return NULL;
//     return strchr(MSVCRT__environ[idx], '=') + 1;
// }
//
// /*********************************************************************
//  *              getenv (MSVCRT.@)
//  */
// char *  getenv(const char *name)
// {
//     char *ret;
//
//     if (!MSVCRT_CHECK_PMT(name != NULL)) return NULL;
//
//     _lock(_ENV_LOCK);
//     ret = getenv_helper(name);
//     _unlock(_ENV_LOCK);
//     return ret;
// }
//
// static wchar_t * wgetenv_helper(const wchar_t *name)
// {
//     int idx;
//
//     if (!name) return NULL;
//     if (env_init(TRUE, FALSE)) return NULL;
//
//     idx = wenv_get_index(name);
//     if (!MSVCRT__wenviron[idx]) return NULL;
//     return wcschr(MSVCRT__wenviron[idx], '=') + 1;
// }
//
// /*********************************************************************
//  *              _wgetenv (MSVCRT.@)
//  */
// wchar_t *  _wgetenv(const wchar_t *name)
// {
//     wchar_t *ret;
//
//     if (!MSVCRT_CHECK_PMT(name != NULL)) return NULL;
//
//     _lock(_ENV_LOCK);
//     ret = wgetenv_helper(name);
//     _unlock(_ENV_LOCK);
//     return ret;
// }
//
// static int putenv_helper(const char *name, const char *val, const char *eq)
// {
//     wchar_t *wenv;
//     char *env;
//     int r;
//
//     if (eq)
//     {
//         env = strdup(name);
//         if (!env) return -1;
//     }
//     else
//     {
//         int name_len = strlen(name);
//
//         r = strlen(val);
//         env = malloc(name_len + r + 2);
//         if (!env) return -1;
//         memcpy(env, name, name_len);
//         env[name_len] = '=';
//         strcpy(env + name_len + 1, val);
//     }
//
//     wenv = msvcrt_wstrdupa(env);
//     if (!wenv)
//     {
//         free(env);
//         return -1;
//     }
//
//     _lock(_ENV_LOCK);
//     r = env_set(&env, &wenv);
//     _unlock(_ENV_LOCK);
//     free(env);
//     free(wenv);
//     return r;
// }
//
// static char *msvcrt_astrdupw(const wchar_t *wstr)
// {
//     const unsigned int len = WideCharToMultiByte(CP_ACP, 0, wstr, -1, NULL, 0, NULL, NULL);
//     char *str = malloc(len * sizeof(char));
//
//     if (!str)
//         return NULL;
//     WideCharToMultiByte(CP_ACP, 0, wstr, -1, str, len, NULL, NULL);
//     return str;
// }
//
//
// static int wputenv_helper(const wchar_t *name, const wchar_t *val, const wchar_t *eq)
// {
//     wchar_t *wenv;
//     char *env;
//     int r;
//
//     _lock(_ENV_LOCK);
//     r = env_init(TRUE, TRUE);
//     _unlock(_ENV_LOCK);
//     if (r) return -1;
//
//     if (eq)
//     {
//         wenv = wcsdup(name);
//         if (!wenv) return -1;
//     }
//     else
//     {
//         int name_len = wcslen(name);
//
//         r = wcslen(val);
//         wenv = malloc((name_len + r + 2) * sizeof(wchar_t));
//         if (!wenv) return -1;
//         memcpy(wenv, name, name_len * sizeof(wchar_t));
//         wenv[name_len] = '=';
//         wcscpy(wenv + name_len + 1, val);
//     }
//
//     env = msvcrt_astrdupw(wenv);
//     if (!env)
//     {
//         free(wenv);
//         return -1;
//     }
//
//     _lock(_ENV_LOCK);
//     r = env_set(&env, &wenv);
//     _unlock(_ENV_LOCK);
//     free(env);
//     free(wenv);
//     return r;
// }
//
// /*********************************************************************
//  *		_putenv (MSVCRT.@)
//  */
// int CDECL _putenv(const char *str)
// {
//     const char *eq;
//
//     TRACE("%s\n", debugstr_a(str));
//
//     if (!str || !(eq = strchr(str, '=')))
//         return -1;
//
//     return putenv_helper(str, NULL, eq);
// }
//
// /*********************************************************************
//  *		_wputenv (MSVCRT.@)
//  */
// int CDECL _wputenv(const wchar_t *str)
// {
//     const wchar_t *eq;
//
//     TRACE("%s\n", debugstr_w(str));
//
//     if (!str || !(eq = wcschr(str, '=')))
//         return -1;
//
//     return wputenv_helper(str, NULL, eq);
// }
//
// /*********************************************************************
//  *		_putenv_s (MSVCRT.@)
//  */
// errno_t CDECL _putenv_s(const char *name, const char *value)
// {
//     errno_t ret = 0;
//
//     TRACE("%s %s\n", debugstr_a(name), debugstr_a(value));
//
//     if (!MSVCRT_CHECK_PMT(name != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(value != NULL)) return EINVAL;
//
//     if (putenv_helper(name, value, NULL) < 0)
//     {
//         msvcrt_set_errno(GetLastError());
//         ret = *_errno();
//     }
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_wputenv_s (MSVCRT.@)
//  */
// errno_t CDECL _wputenv_s(const wchar_t *name, const wchar_t *value)
// {
//     errno_t ret = 0;
//
//     TRACE("%s %s\n", debugstr_w(name), debugstr_w(value));
//
//     if (!MSVCRT_CHECK_PMT(name != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(value != NULL)) return EINVAL;
//
//     if (wputenv_helper(name, value, NULL) < 0)
//     {
//         msvcrt_set_errno(GetLastError());
//         ret = *_errno();
//     }
//
//     return ret;
// }
//
//
// /******************************************************************
//  *		getenv_s (MSVCRT.@)
//  */
// int CDECL getenv_s(size_t *ret_len, char* buffer, size_t len, const char *varname)
// {
//     char *e;
//
//     if (!MSVCRT_CHECK_PMT(ret_len != NULL)) return EINVAL;
//     *ret_len = 0;
//     if (!MSVCRT_CHECK_PMT((buffer && len > 0) || (!buffer && !len))) return EINVAL;
//     if (buffer) buffer[0] = 0;
//
//     _lock(_ENV_LOCK);
//     e = getenv_helper(varname);
//     if (e)
//     {
//         *ret_len = strlen(e) + 1;
//         if (len >= *ret_len) strcpy(buffer, e);
//     }
//     _unlock(_ENV_LOCK);
//
//     if (!e || !len) return 0;
//     if (len < *ret_len) return ERANGE;
//     return 0;
// }
//
// /******************************************************************
//  *		_wgetenv_s (MSVCRT.@)
//  */
// int CDECL _wgetenv_s(size_t *ret_len, wchar_t *buffer, size_t len,
//                      const wchar_t *varname)
// {
//     wchar_t *e;
//
//     if (!MSVCRT_CHECK_PMT(ret_len != NULL)) return EINVAL;
//     *ret_len = 0;
//     if (!MSVCRT_CHECK_PMT((buffer && len > 0) || (!buffer && !len))) return EINVAL;
//     if (buffer) buffer[0] = 0;
//
//     _lock(_ENV_LOCK);
//     e = wgetenv_helper(varname);
//     if (e)
//     {
//         *ret_len = wcslen(e) + 1;
//         if (len >= *ret_len) wcscpy(buffer, e);
//     }
//     _unlock(_ENV_LOCK);
//
//     if (!e || !len) return 0;
//     if (len < *ret_len) return ERANGE;
//     return 0;
// }
//
// /*********************************************************************
//  *		_get_environ (MSVCRT.@)
//  */
// void CDECL _get_environ(char ***ptr)
// {
//     *ptr = MSVCRT__environ;
// }
//
// /*********************************************************************
//  *		_get_wenviron (MSVCRT.@)
//  */
// void CDECL _get_wenviron(wchar_t ***ptr)
// {
//     *ptr = MSVCRT__wenviron;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
