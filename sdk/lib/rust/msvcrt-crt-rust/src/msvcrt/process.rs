use crate::prelude::*;
use core::ffi::c_void;
use crate::{wcscat, wcscpy, wcslen};
use crate::msvcrt::file::{
    _close, _dup, _get_osfhandle, _pipe, _wfdopen, fclose, msvcrt_create_io_inherit_block,
    STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};

// ---- the substrate this file stands on ----
//
// Everything below used to be a local `extern` block or a local `const`, each flagged
// `// TODO(hoist)`, because the lane that wrote this file owned neither msvcrt-sys nor
// prelude.rs. They are all derived now and the locals are gone:
//
//   msvcrt-sys (bindgen, all 3 arches):  GetEnvironmentVariableW,
//                                        CREATE_UNICODE_ENVIRONMENT,
//                                        STARTUPINFOW / PROCESS_INFORMATION
//   prelude.rs:                          realloc, _exit, intptr_t
//
// `CREATE_UNICODE_ENVIRONMENT` is the one worth naming: it was absent from the island not
// because nobody asked for it but because the allowlist said `CREATE_[A-Z]+`, which matches
// CREATE_NEW and drops every multi-word member of the family. `msvcrt_spawn` ORs it into
// the creation flags of EVERY spawn, so a hand-written value that had been wrong would have
// been wrong for all of them, silently. Same shape as LOCALE_SMONTHNAME1 (§9.113).
//
// NOT hoisted, deliberately: `get_aw_cp`, `convert_acp_utf8_to_wcs` and `HIWORD` below.
// Each mirrors the C's own `static inline` or macro, so a private per-TU copy is the
// FAITHFUL shape rather than a shortcut.

/// `dll/win32/msvcrt/msvcrt.h:422` — `static inline UINT get_aw_cp(void)`. Duplicated from
/// `crate::prelude::get_aw_cp` (private there, and prelude.rs is off-limits to this lane
/// right now) rather than hoisted to `pub`: the C original is itself a per-TU `static
/// inline`, so a private copy per translation unit is the faithful shape, not a shortcut.
/// At `_MSVCR_VER=0` (this build) it is unconditionally `CP_ACP` — see prelude.rs's own
/// copy for the collapsed `#if _MSVCR_VER>=140` reasoning.
#[inline]
fn get_aw_cp() -> UINT {
    CP_ACP
}

/// `dll/win32/msvcrt/msvcrt.h:430` — `static inline int convert_acp_utf8_to_wcs(...)`.
/// Same per-TU-copy reasoning as `get_aw_cp` above.
#[inline]
unsafe fn convert_acp_utf8_to_wcs(str_: *const i8, wstr: *mut wchar_t, len: i32) -> i32 {
    MultiByteToWideChar(get_aw_cp(), MB_PRECOMPOSED, str_, -1, wstr, len)
}

/// `sdk/include/psdk/minwindef.h:190` — `#define HIWORD(l) ((WORD)(((DWORD_PTR)(l) >> 16) &
/// 0xffff))`. A preprocessor macro, not an ABI symbol, so there is no substrate gap to
/// hoist: every TU that wants it gets its own copy, same as winsrv-rust's
/// (`consrv/frontends/gui/helpers.rs`).
#[inline]
#[allow(non_snake_case)]
fn HIWORD(l: usize) -> u16 {
    ((l >> 16) & 0xFFFF) as u16
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct popen_handle {
    pub f: *mut FILE,
    pub proc: HANDLE,
}

pub type Ppopen_handle = *mut popen_handle;

pub static mut popen_handles: *mut popen_handle = core::ptr::null_mut();
pub static mut popen_handles_size: DWORD = 0;
unsafe extern "C" fn msvcrt_search_executable(name: *const wchar_t, fullname: *mut wchar_t, use_path: i32) {
    static suffix: [[wchar_t; 5]; 4] = [
        [b'.' as u16, b'c' as u16, b'o' as u16, b'm' as u16, 0],
        [b'.' as u16, b'e' as u16, b'x' as u16, b'e' as u16, 0],
        [b'.' as u16, b'b' as u16, b'a' as u16, b't' as u16, 0],
        [b'.' as u16, b'c' as u16, b'm' as u16, b'd' as u16, 0],
    ];

    let mut buffer: [wchar_t; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut env: *const wchar_t;
    let mut p: *const wchar_t;
    let mut end: *const wchar_t;
    let mut i: u32;
    let mut name_len: u32;
    let mut path_len: u32;
    let mut extension: i32 = 1;

    *fullname = 0;

    end = name.offset((MAX_PATH - 1) as isize);
    p = name;
    while p < end {
        if *p == 0 {
            break;
        }
        p = p.offset(1);
    }
    name_len = p.offset_from(name) as u32;

    memcpy(
        buffer.as_mut_ptr() as *mut _,
        name as *const _,
        (name_len as usize) * core::mem::size_of::<wchar_t>(),
    );
    buffer[name_len as usize] = 0;

    if GetFileAttributesW(buffer.as_ptr()) != INVALID_FILE_ATTRIBUTES {
        wcscpy(fullname, buffer.as_ptr());
        return;
    }

    p = p.offset(-1);
    while p >= name {
        if *p == b'\\' as u16 || *p == b'/' as u16 || *p == b':' as u16 || *p == b'.' as u16 {
            break;
        }
        p = p.offset(-1);
    }

    if (p < name || *p != b'.' as u16) && name_len <= MAX_PATH - 5 {
        i = 0;
        while i < 4 {
            memcpy(
                buffer.as_mut_ptr().offset(name_len as isize) as *mut _,
                suffix[i as usize].as_ptr() as *const _,
                5 * core::mem::size_of::<wchar_t>(),
            );
            if GetFileAttributesW(buffer.as_ptr()) != INVALID_FILE_ATTRIBUTES {
                wcscpy(fullname, buffer.as_ptr());
                return;
            }
            i += 1;
        }
        extension = 0;
    }

    if use_path == 0 {
        return;
    }
    // Ten bytes, not nine: a `u16` NUL terminator is TWO zero bytes. A Rust byte string is
    // not NUL-terminated for you, so `b"P\0A\0T\0H\0\0"` gives `_wgetenv` a key whose final
    // code unit is half read from the next object in rodata — PATH search then silently
    // matches nothing.
    env = _wgetenv(win32macros_rust::wide_bytes!(b"P\0A\0T\0H\0\0\0") as *const wchar_t);
    if env.is_null() {
        return;
    }

    loop {
        p = env;
        while *p != 0 && *p != b';' as u16 {
            p = p.offset(1);
        }
        if p == env {
            return;
        }

        path_len = p.offset_from(env) as u32;
        if path_len + name_len <= MAX_PATH - 2 {
            memcpy(
                buffer.as_mut_ptr() as *mut _,
                env as *const _,
                (path_len as usize) * core::mem::size_of::<wchar_t>(),
            );
            if buffer[path_len as usize] != b'/' as u16 && buffer[path_len as usize] != b'\\' as u16 {
                buffer[path_len as usize] = b'\\' as u16;
                path_len += 1;
                buffer[path_len as usize] = 0;
            } else {
                buffer[path_len as usize] = 0;
            }

            wcscat(buffer.as_mut_ptr(), name);
            if GetFileAttributesW(buffer.as_ptr()) != INVALID_FILE_ATTRIBUTES {
                wcscpy(fullname, buffer.as_ptr());
                return;
            }
        }

        if extension == 0 && path_len + name_len <= MAX_PATH - 5 {
            i = 0;
            while i < 4 {
                memcpy(
                    buffer.as_mut_ptr().offset((path_len + name_len) as isize) as *mut _,
                    suffix[i as usize].as_ptr() as *const _,
                    5 * core::mem::size_of::<wchar_t>(),
                );
                if GetFileAttributesW(buffer.as_ptr()) != INVALID_FILE_ATTRIBUTES {
                    wcscpy(fullname, buffer.as_ptr());
                    return;
                }
                i += 1;
            }
        }

        env = if *p != 0 { p.offset(1) } else { p };

        if 1 == 0 {
            break;
        }
    }
}

// ============================================================================
// dll/win32/msvcrt/process.c:135-186 — `msvcrt_spawn` (file-static helper, NOT exported).
// Hand-written (`/home/curi/vibeos-rust-staging/wave20/spawn-wpopen.rs`): the paid
// pipeline could not get this body past the deterministic call-envelope audit. Pasted in
// source order (right after `msvcrt_search_executable`, before `msvcrt_argvtos`, exactly
// as process.c has it) and reviewed/repaired to compile — see the flags-comparison casts
// and `SIZE_T`-vs-`usize` fixes below, none of which change the logic, all of which the
// original hand-written draft got wrong in ways that only show up on a real compiler.
// ============================================================================
unsafe extern "C" fn msvcrt_spawn(
    flags: i32,
    exe: *const wchar_t,
    cmdline: *mut wchar_t,
    env: *mut wchar_t,
    use_path: i32,
) -> intptr_t {
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut fullname: [wchar_t; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut create_flags: DWORD = CREATE_UNICODE_ENVIRONMENT;

    // if ((unsigned)flags > _P_DETACH) { *_errno() = EINVAL; return -1; }
    if (flags as u32) > _P_DETACH {
        *_errno() = EINVAL as i32;
        return -1;
    }

    msvcrt_search_executable(exe, fullname.as_mut_ptr(), use_path);

    // memset(&si, 0, sizeof(si)); si.cb = sizeof(si);
    si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    msvcrt_create_io_inherit_block(&mut si.cbReserved2, &mut si.lpReserved2);
    if flags as u32 == _P_DETACH {
        create_flags |= DETACHED_PROCESS;
    }
    if CreateProcessW(
        fullname.as_ptr(),
        cmdline,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TRUE as BOOL,
        create_flags,
        env as LPVOID,
        core::ptr::null(),
        &mut si,
        &mut pi,
    ) == 0
    {
        msvcrt_set_errno(GetLastError() as i32);
        free(si.lpReserved2 as *mut c_void);
        return -1;
    }

    free(si.lpReserved2 as *mut c_void);

    // switch(flags) { case _P_WAIT: ...; case _P_DETACH: ...; /* fall through */
    // case _P_NOWAIT: case _P_NOWAITO: ...; case _P_OVERLAY: _exit(0); }
    if flags as u32 == _P_WAIT {
        WaitForSingleObject(pi.hProcess, INFINITE);
        GetExitCodeProcess(pi.hProcess, &mut pi.dwProcessId);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        return pi.dwProcessId as intptr_t;
    } else if flags as u32 == _P_DETACH || flags as u32 == _P_NOWAIT || flags as u32 == _P_NOWAITO {
        if flags as u32 == _P_DETACH {
            CloseHandle(pi.hProcess);
            pi.hProcess = core::ptr::null_mut();
        }
        CloseHandle(pi.hThread);
        return pi.hProcess as intptr_t;
    } else if flags as u32 == _P_OVERLAY {
        _exit(0);
    }
    -1 /* can't reach here */
}

unsafe extern "C" fn msvcrt_argvtos(arg: *const *const wchar_t, delim: wchar_t) -> *mut wchar_t {
    let mut a: *const *const wchar_t;
    let mut size: i32;
    let mut p: *mut wchar_t;
    let ret: *mut wchar_t;

    if arg.is_null() {
        return core::ptr::null_mut();
    }

    a = arg;
    size = 0;
    while !(*a).is_null() {
        size += wcslen(*a) as i32 + 1;
        a = a.offset(1);
    }

    let ret = malloc(((size + 1) as usize) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    a = arg;
    p = ret;
    while !(*a).is_null() {
        let len = wcslen(*a);
        memcpy(p as *mut core::ffi::c_void, *a as *const core::ffi::c_void, len * core::mem::size_of::<wchar_t>());
        p = p.offset(len as isize);
        *p = delim;
        p = p.offset(1);
        a = a.offset(1);
    }
    if delim != 0 && p > ret {
        *p.offset(-(1 as isize)) = 0;
    } else {
        *p = 0;
    }
    ret
}
unsafe extern "C" fn msvcrt_argvtos_aw(arg: *const *const i8, env: u32) -> *mut wchar_t {
    let cp: u32 = if env != 0 {
        CP_ACP
    } else {
        get_aw_cp()
    };
    let mut a: *const *const i8;
    let mut len: u32;
    let mut p: *mut wchar_t;
    let ret: *mut wchar_t;

    if arg.is_null() {
        return core::ptr::null_mut();
    }

    a = arg;
    len = 0;
    while !(*a).is_null() {
        len += MultiByteToWideChar(
            cp,
            0,
            *a,
            -1,
            core::ptr::null_mut(),
            0,
        ) as u32;
        a = a.offset(1);
    }

    ret = malloc(((len + 1) * core::mem::size_of::<wchar_t>() as u32) as usize) as *mut wchar_t;
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    a = arg;
    p = ret;
    while !(*a).is_null() {
        p = p.offset(MultiByteToWideChar(
            cp,
            0,
            *a,
            strlen(*a) as i32,
            p,
            (len as isize - p.offset_from(ret)) as i32,
        ) as isize);
        *p = if env != 0 { 0 } else { ' ' as wchar_t };
        p = p.offset(1);
        a = a.offset(1);
    }
    if env == 0 && p > ret {
        *p.offset(-1) = 0;
    } else {
        *p = 0;
    }
    ret
}
unsafe extern "C" fn msvcrt_valisttos(arg0: *const wchar_t, mut alist: va_list, delim: wchar_t) -> *mut wchar_t {
    let mut size: u32 = 0;
    let mut pos: u32 = 0;
    let mut arg: *const wchar_t;
    let mut new: *mut wchar_t;
    let mut ret: *mut wchar_t = core::ptr::null_mut();

    arg = arg0;
    while !arg.is_null() {
        let len: u32 = wcslen(arg) as u32 + 1;
        if pos + len >= size {
            size = core::cmp::max(256, size * 2);
            size = core::cmp::max(size, pos + len + 1);
            new = realloc(ret as *mut core::ffi::c_void, (size as usize) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
            if new.is_null() {
                free(ret as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }
            ret = new;
        }
        wcscpy(ret.add(pos as usize), arg);
        pos += len;
        *ret.add((pos - 1) as usize) = delim;

        // `arg = va_arg(alist, wchar_t *)` (process.c:285) — `alist` is a plain pointer
        // cursor on every Windows target we build, so advancing our OWN local copy here
        // does not disturb whatever copy the caller holds. See `crate::printf::va_next`.
        arg = crate::printf::va_next::<*const wchar_t>(&mut alist);
    }
    if pos != 0 {
        if delim != 0 {
            *ret.add((pos - 1) as usize) = 0;
        } else {
            *ret.add(pos as usize) = 0;
        }
    }
    ret
}
unsafe extern "C" fn msvcrt_valisttos_aw(arg0: *const i8, mut alist: va_list, delim: wchar_t) -> *mut wchar_t {
    let mut size: u32 = 0;
    let mut pos: u32 = 0;
    let mut arg: *const i8;
    let mut new: *mut wchar_t;
    let mut ret: *mut wchar_t = core::ptr::null_mut();

    arg = arg0;
    while !arg.is_null() {
        let len: u32 = convert_acp_utf8_to_wcs(arg, core::ptr::null_mut(), 0) as u32;
        if pos + len >= size {
            size = core::cmp::max(256, size * 2);
            size = core::cmp::max(size, pos + len + 1);
            new = realloc(ret as *mut core::ffi::c_void, (size as usize) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
            if new.is_null() {
                free(ret as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }
            ret = new;
        }
        pos += convert_acp_utf8_to_wcs(arg, ret.add(pos as usize), (size - pos) as i32) as u32;
        *ret.add((pos - 1) as usize) = delim;
        // See the comment in `msvcrt_valisttos` above — same by-value cursor reasoning.
        arg = crate::printf::va_next::<*const i8>(&mut alist);
    }
    if pos != 0 {
        if delim != 0 {
            *ret.add((pos - 1) as usize) = 0;
        } else {
            *ret.add(pos as usize) = 0;
        }
    }
    ret
}
unsafe extern "C" fn msvcrt_get_comspec() -> *mut wchar_t {
    let mut ret: *mut wchar_t;
    let mut len: u32;

    len = GetEnvironmentVariableW(win32macros_rust::wide_bytes!(b"C\0O\0M\0S\0P\0E\0C\0\0\0") as *const wchar_t, core::ptr::null_mut(), 0);
    if len == 0 {
        len = 4;
    }
    ret = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<wchar_t>()) as SIZE_T) as *mut wchar_t;
    if !ret.is_null() {
        if GetEnvironmentVariableW(win32macros_rust::wide_bytes!(b"C\0O\0M\0S\0P\0E\0C\0\0\0") as *const wchar_t, ret, len) == 0 {
            wcscpy(ret, win32macros_rust::wide_bytes!(b"c\0m\0d\0\0\0") as *const wchar_t);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _cwait(status: *mut i32, pid: isize, action: i32) -> isize {
    let hPid = pid as HANDLE;
    let doserrno: i32;

    if WaitForSingleObject(hPid, INFINITE) == 0 {
        if !status.is_null() {
            let mut stat: u32 = 0;
            GetExitCodeProcess(hPid, &mut stat);
            *status = stat as i32;
        }
        return pid;
    }
    doserrno = GetLastError() as i32;

    if doserrno == ERROR_INVALID_HANDLE as i32 {
        *_errno() = ECHILD as i32;
        *__doserrno() = doserrno as u32;
    } else {
        msvcrt_set_errno(doserrno);
    }

    if !status.is_null() {
        *status = -1 as i32;
        *status as isize
    } else {
        -1 as isize
    }
}

// ============================================================================
// process.c:390-598 — the eight `_exec*` WINAPIV variadic exports (name/arg0/... form).
//
// Stable Rust cannot DEFINE a C-variadic function (`c_variadic` is nightly-only; this
// project is pinned to stable — AGENTS.md). So the real exports (`_wexecl`, `_execl`, ...)
// live as `va_start` trampolines in `dll/win32/msvcrt/rust_process_varargs_shim.c`, each
// handing its ALREADY-STARTED `va_list` cursor to the Rust twin below. RECEIVING a started
// cursor needs no `c_variadic` — a `va_list` is a plain pointer on every Windows target we
// build (`crate::prelude::va_list`) — only DEFINING the `...` shell is out of reach.
//
// Twin naming: real MSVCRT spells "the va_list version of X" by inserting a `v` right
// after X's leading underscore (`vfprintf`/`_vsnprintf`/...), so `_wexecl`'s twin is
// `_vwexecl`, `_spawnlpe`'s is `_vspawnlpe`. None of the sixteen twin names collide with a
// real msvcrt.spec export (checked). Calling convention: plain `extern "C"` — on i686 that
// is cdecl, matching `WINAPIV`; on amd64/arm64 there is only one convention, so the
// question does not arise. The `WINAPIV`-ness lives entirely in the C trampolines, which
// keep the real prototype.
//
// Double-walk families (`_execle`, `_execlpe`, `_wexecle`, `_wexeclpe`, and the four
// `_spawn` counterparts further down): process.c calls `va_start` on the SAME parameter
// TWICE — once to build the command line via `msvcrt_valisttos[_aw]`, once more to skip
// past the argument list and read `envp`. On every target we build, `va_start(ap, arg0)`
// always resets `ap` to the FIXED address right after `arg0`'s slot — it cannot depend on
// what a previous walk consumed, because `msvcrt_valisttos[_aw]` receives its `alist`
// parameter BY VALUE (a plain pointer copy) and only ever advances that local copy, never
// the caller's `ap`. So the C's two `va_start` calls are two independent walks from the
// identical starting point, and handing the twin a SINGLE `va_list` snapshot is
// sufficient: the twin takes its own `let mut ap = valist` copy per walk below, exactly
// reproducing the effect of the C's two `va_start`s. This equivalence is specific to the
// flat-pointer Windows `va_list` model we build for — it would NOT hold on an ABI where
// `va_list` is itself a mutated-in-place struct (SysV x86-64's form).
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _vwexecl(name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> intptr_t {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);
    let ret = msvcrt_spawn(_P_OVERLAY as i32, name, args_str, core::ptr::null_mut(), 0);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vexecl(name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args_str = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args_str, core::ptr::null_mut(), 0);

    free(nameW as *mut c_void);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwexecle(name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> isize {
    let args = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut wchar_t>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const wchar_t>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos(envp, 0);
    }

    let ret = msvcrt_spawn(_P_OVERLAY as i32, name, args, envs, 0);

    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vexecle(name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut i8>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const i8>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos_aw(envp, TRUE);
    }

    let ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, envs, 0);

    free(nameW as *mut c_void);
    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwexeclp(name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> intptr_t {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);
    let ret = msvcrt_spawn(_P_OVERLAY as i32, name, args_str, core::ptr::null_mut(), 1);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vexeclp(name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args_w = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args_w, core::ptr::null_mut(), 1);

    free(nameW as *mut c_void);
    free(args_w as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwexeclpe(name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> intptr_t {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut wchar_t>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const wchar_t>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos(envp, 0);
    }

    let ret = msvcrt_spawn(_P_OVERLAY as i32, name, args_str, envs, 1);

    free(args_str as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vexeclpe(name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut i8>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const i8>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos_aw(envp, TRUE);
    }

    let ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, envs, 1);

    free(nameW as *mut c_void);
    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wexecv(name: *const wchar_t, argv: *const *const wchar_t) -> isize {
    _wspawnve(_P_OVERLAY as i32, name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _execv(name: *const i8, argv: *const *const i8) -> isize {
    _spawnve(_P_OVERLAY as i32, name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _wexecve(
    name: *const wchar_t,
    argv: *const *const wchar_t,
    envv: *const *const wchar_t,
) -> isize {
    _wspawnve(_P_OVERLAY as i32, name, argv, envv)
}

#[no_mangle]
pub unsafe extern "C" fn _execve(name: *const i8, argv: *const *const i8, envv: *const *const i8) -> isize {
    _spawnve(_P_OVERLAY as i32, name, argv, envv)
}

#[no_mangle]
pub unsafe extern "C" fn _wexecvpe(
    name: *const wchar_t,
    argv: *const *const wchar_t,
    envv: *const *const wchar_t,
) -> isize {
    _wspawnvpe(_P_OVERLAY as i32, name, argv, envv)
}

#[no_mangle]
pub unsafe extern "C" fn _execvpe(
    name: *const i8,
    argv: *const *const i8,
    envv: *const *const i8,
) -> isize {
    _spawnvpe(_P_OVERLAY as i32, name, argv, envv)
}

#[no_mangle]
pub unsafe extern "C" fn _wexecvp(name: *const wchar_t, argv: *mut *const wchar_t) -> intptr_t {
    _wexecvpe(name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _execvp(name: *const i8, argv: *mut *const i8) -> intptr_t {
    _execvpe(name, argv, core::ptr::null())
}

// ============================================================================
// process.c:685-892 — the eight `_spawn*` WINAPIV variadic exports (flags/name/arg0/...
// form). Same language limit, same trampoline split, same twin-naming and double-walk
// reasoning as the `_exec*` family above — see that block header. The only difference is
// the extra leading `flags: i32` parameter, threaded straight through to `msvcrt_spawn`.
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _vwspawnl(flags: i32, name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> isize {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);
    let ret = msvcrt_spawn(flags, name, args_str, core::ptr::null_mut(), 0);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vspawnl(flags: i32, name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args_str = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let ret = msvcrt_spawn(flags, nameW, args_str, core::ptr::null_mut(), 0);

    free(nameW as *mut c_void);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwspawnle(flags: i32, name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> isize {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut wchar_t>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const wchar_t>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos(envp, 0);
    }

    let ret = msvcrt_spawn(flags, name, args_str, envs, 0);

    free(args_str as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vspawnle(flags: i32, name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut i8>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const i8>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos_aw(envp, TRUE);
    }

    let ret = msvcrt_spawn(flags, nameW, args, envs, 0);

    free(nameW as *mut c_void);
    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwspawnlp(flags: i32, name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> isize {
    let args_str = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);
    let ret = msvcrt_spawn(flags, name, args_str, core::ptr::null_mut(), 1);
    free(args_str as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vspawnlp(flags: i32, name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1;
    }

    let args_ = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let ret = msvcrt_spawn(flags, nameW, args_, core::ptr::null_mut(), 1);

    free(nameW as *mut c_void);
    free(args_ as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vwspawnlpe(flags: i32, name: *const wchar_t, arg0: *const wchar_t, valist: va_list) -> isize {
    let args = msvcrt_valisttos(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut wchar_t>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const wchar_t>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos(envp, 0);
    }

    let ret = msvcrt_spawn(flags, name, args, envs, 1);

    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vspawnlpe(flags: i32, name: *const i8, arg0: *const i8, valist: va_list) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1 as isize;
    }

    let args_str = msvcrt_valisttos_aw(arg0, valist, ' ' as wchar_t);

    let mut ap = valist;
    while !crate::printf::va_next::<*mut i8>(&mut ap).is_null() {}
    let envp = crate::printf::va_next::<*const *const i8>(&mut ap);
    let mut envs: *mut wchar_t = core::ptr::null_mut();
    if !envp.is_null() {
        envs = msvcrt_argvtos_aw(envp, TRUE);
    }

    let ret = msvcrt_spawn(flags, nameW, args_str, envs, 1);

    free(nameW as *mut c_void);
    free(args_str as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _spawnve(
    flags: i32,
    name: *const i8,
    argv: *const *const i8,
    envv: *const *const i8,
) -> isize {
    let nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -1 as isize;
    }

    let args = msvcrt_argvtos_aw(argv, msvcrt_sys::FALSE);
    let envs = msvcrt_argvtos_aw(envv, TRUE);

    let ret = msvcrt_spawn(flags, nameW, args, envs, 0);

    free(nameW as *mut c_void);
    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wspawnve(
    flags: i32,
    name: *const wchar_t,
    argv: *const *const wchar_t,
    envv: *const *const wchar_t,
) -> isize {
    let args: *mut wchar_t = msvcrt_argvtos(argv, ' ' as wchar_t);
    let envs: *mut wchar_t = msvcrt_argvtos(envv, 0);

    let ret: isize = msvcrt_spawn(flags, name, args, envs, 0);

    free(args as *mut core::ffi::c_void);
    free(envs as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _spawnv(
    flags: i32,
    name: *const i8,
    argv: *mut *const i8,
) -> intptr_t {
    _spawnve(flags, name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _wspawnv(
    flags: i32,
    name: *const wchar_t,
    argv: *mut *const wchar_t,
) -> intptr_t {
    _wspawnve(flags, name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _spawnvpe(
    flags: i32,
    name: *const i8,
    argv: *const *const i8,
    envv: *const *const i8,
) -> isize {
    let mut nameW: *mut wchar_t;
    let mut args: *mut wchar_t;
    let mut envs: *mut wchar_t;
    let ret: isize;

    nameW = wstrdupa_utf8(name);
    if nameW.is_null() {
        return -(1 as isize);
    }

    args = msvcrt_argvtos_aw(argv, msvcrt_sys::FALSE);
    envs = msvcrt_argvtos_aw(envv, TRUE);

    ret = msvcrt_spawn(flags, nameW, args, envs, 1);

    free(nameW as *mut c_void);
    free(args as *mut c_void);
    free(envs as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wspawnvpe(
    flags: i32,
    name: *const wchar_t,
    argv: *const *const wchar_t,
    envv: *const *const wchar_t,
) -> isize {
    let args: *mut wchar_t = msvcrt_argvtos(argv, ' ' as wchar_t);
    let envs: *mut wchar_t = msvcrt_argvtos(envv, 0);

    let ret: isize = msvcrt_spawn(flags, name, args, envs, 1);

    free(args as *mut core::ffi::c_void);
    free(envs as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _spawnvp(
    flags: i32,
    name: *const i8,
    argv: *mut *const i8,
) -> intptr_t {
    _spawnvpe(flags, name, argv, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn _wspawnvp(
    flags: i32,
    name: *const wchar_t,
    argv: *mut *const wchar_t,
) -> intptr_t {
    _wspawnvpe(flags, name, argv, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_popen_data() {
    free(popen_handles as *mut c_void);
}

// ============================================================================
// dll/win32/msvcrt/process.c:1044-1180 — `_wpopen` (MSVCRT.@), exported.
// Hand-written (`/home/curi/vibeos-rust-staging/wave20/spawn-wpopen.rs`): the paid
// pipeline could not get this body past the deterministic call-envelope audit. Pasted in
// source order (right after `msvcrt_free_popen_data`, before `_popen`, exactly as
// process.c has it) and reviewed/repaired to compile. Uses the file's OWN preamble statics
// `popen_handles`/`popen_handles_size` and the `popen_handle` struct declared at the top
// of this file — not redeclared here.
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn _wpopen(command: *const wchar_t, mode: *const wchar_t) -> *mut FILE {
    let mut comspec: *mut wchar_t;
    let mut fullcmd: *mut wchar_t;
    let mut fullname: [wchar_t; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut textmode: i32 = 0;
    let mut fds: [i32; 2] = [0; 2];
    let fd_parent: i32;
    let mut fd_child: i32;
    let mut container: *mut popen_handle;
    let mut pi: PROCESS_INFORMATION;
    let mut si: STARTUPINFOW;
    let mut read_pipe: BOOL = TRUE as BOOL;
    let mut p: *const wchar_t;
    let mut len: u32;
    let ret: *mut FILE;
    let mut i: DWORD;
    let r: BOOL;

    if command.is_null() || mode.is_null() {
        return core::ptr::null_mut();
    }

    _get_fmode(&mut textmode);
    textmode &= (_O_BINARY | _O_TEXT) as i32;
    textmode |= _O_NOINHERIT as i32;

    p = mode;
    while *p != 0 {
        if *p == b'W' as u16 || *p == b'w' as u16 {
            read_pipe = msvcrt_sys::FALSE as BOOL;
        } else if *p == b'B' as u16 || *p == b'b' as u16 {
            textmode |= _O_BINARY as i32;
            textmode &= !(_O_TEXT as i32);
        } else if *p == b'T' as u16 || *p == b't' as u16 {
            textmode |= _O_TEXT as i32;
            textmode &= !(_O_BINARY as i32);
        }
        p = p.offset(1);
    }
    if _pipe(fds.as_mut_ptr(), 0, textmode) == -1 {
        return core::ptr::null_mut();
    }

    if read_pipe != 0 {
        fd_parent = fds[0];
        fd_child = _dup(fds[1]);
        _close(fds[1]);
    } else {
        fd_parent = fds[1];
        fd_child = _dup(fds[0]);
        _close(fds[0]);
    }
    if fd_child == -1 {
        _close(fd_parent);
        return core::ptr::null_mut();
    }
    ret = _wfdopen(fd_parent, mode);
    if ret.is_null() {
        _close(fd_child);
        return core::ptr::null_mut();
    }

    _lock(_POPEN_LOCK as i32);

    'error: {
        i = 0;
        while i < popen_handles_size {
            if (*popen_handles.offset(i as isize)).f.is_null() {
                break;
            }
            i += 1;
        }
        if i == popen_handles_size {
            i = if popen_handles_size != 0 { popen_handles_size * 2 } else { 8 };
            container = realloc(
                popen_handles as *mut c_void,
                (i as usize) * core::mem::size_of::<popen_handle>(),
            ) as *mut popen_handle;
            if container.is_null() {
                break 'error;
            }

            popen_handles = container;
            container = popen_handles.offset(popen_handles_size as isize);
            memset(
                container as *mut c_void,
                0,
                ((i - popen_handles_size) as usize) * core::mem::size_of::<popen_handle>(),
            );
            popen_handles_size = i;
        } else {
            container = popen_handles.offset(i as isize);
        }

        comspec = msvcrt_get_comspec();
        if comspec.is_null() {
            break 'error;
        }
        len = (wcslen(comspec) + wcslen(command) + 5) as u32;

        fullcmd = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<wchar_t>()) as SIZE_T)
            as *mut wchar_t;
        if fullcmd.is_null() {
            HeapFree(GetProcessHeap(), 0, comspec as *mut c_void);
            break 'error;
        }

        wcscpy(fullcmd, comspec);
        // L" /c " — correctly interleaved UTF-16LE (4 chars + NUL = 5 `u16` = 10 bytes),
        // NOT the buggy pattern `_wsystem` (this file, below) used to have: plain ASCII
        // bytes cast to `*const wchar_t` pairs ' ' with '/' into one garbage code unit and
        // never actually NUL-terminates before running off the array.
        wcscat(fullcmd, win32macros_rust::wide_bytes!(b" \0/\0c\0 \0\0\0") as *const wchar_t);
        wcscat(fullcmd, command);
        msvcrt_search_executable(comspec, fullname.as_mut_ptr(), 1);

        si = core::mem::zeroed();
        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        si.dwFlags = STARTF_USESTDHANDLES;
        if read_pipe != 0 {
            si.hStdInput = _get_osfhandle(STDIN_FILENO) as HANDLE;
            si.hStdOutput = _get_osfhandle(fd_child) as HANDLE;
        } else {
            si.hStdInput = _get_osfhandle(fd_child) as HANDLE;
            si.hStdOutput = _get_osfhandle(STDOUT_FILENO) as HANDLE;
        }
        si.hStdError = _get_osfhandle(STDERR_FILENO) as HANDLE;

        pi = core::mem::zeroed();
        r = CreateProcessW(
            fullname.as_ptr(),
            fullcmd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            TRUE as BOOL,
            0,
            core::ptr::null_mut(),
            core::ptr::null(),
            &mut si,
            &mut pi,
        );
        HeapFree(GetProcessHeap(), 0, comspec as *mut c_void);
        HeapFree(GetProcessHeap(), 0, fullcmd as *mut c_void);
        if r == 0 {
            msvcrt_set_errno(GetLastError() as i32);
            break 'error;
        }
        CloseHandle(pi.hThread);
        _close(fd_child);
        (*container).proc = pi.hProcess;
        (*container).f = ret;
        _unlock(_POPEN_LOCK as i32);
        return ret;
    }

    _unlock(_POPEN_LOCK as i32);
    _close(fd_child);
    fclose(ret);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn _popen(command: LPCSTR, mode: LPCSTR) -> *mut FILE {
    let mut ret: *mut FILE;
    let mut cmdW: *mut wchar_t;
    let mut modeW: *mut wchar_t;

    if command.is_null() || mode.is_null() {
        return core::ptr::null_mut();
    }

    cmdW = wstrdupa_utf8(command);
    if cmdW.is_null() {
        return core::ptr::null_mut();
    }
    modeW = msvcrt_wstrdupa(mode);
    if modeW.is_null() {
        free(cmdW as *mut c_void);
        return core::ptr::null_mut();
    }

    ret = _wpopen(cmdW, modeW);

    free(cmdW as *mut c_void);
    free(modeW as *mut c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _pclose(file: *mut FILE) -> i32 {
    let mut h: HANDLE;
    let mut i: DWORD;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return -1;
    }

    _lock(_POPEN_LOCK as i32);
    i = 0;
    while i < popen_handles_size {
        if (*popen_handles.offset(i as isize)).f == file {
            break;
        }
        i += 1;
    }
    if i == popen_handles_size {
        _unlock(_POPEN_LOCK as i32);
        *_errno() = EBADF as i32;
        return -1;
    }

    h = (*popen_handles.offset(i as isize)).proc;
    (*popen_handles.offset(i as isize)).f = core::ptr::null_mut();
    _unlock(_POPEN_LOCK as i32);

    fclose(file);
    if WaitForSingleObject(h, INFINITE) == WAIT_FAILED || GetExitCodeProcess(h, &mut i) == 0 {
        msvcrt_set_errno(GetLastError() as i32);
        CloseHandle(h);
        return -1;
    }

    CloseHandle(h);
    return i as i32;
}

#[cfg(any())] // `#ifndef __REACTOS__` in process.c (process.c:1248-1312) — never compiled
              // here, and the object symbol table proves it: no arch's process.c.obj
              // defines `_wsystem`.
#[no_mangle]
pub unsafe extern "C" fn _wsystem(cmd: *const wchar_t) -> i32 {
    let mut res: i32;
    let mut comspec: *mut wchar_t;
    let mut fullcmd: *mut wchar_t;
    let mut len: u32;

    comspec = msvcrt_get_comspec();

    if cmd.is_null() {
        if comspec.is_null() {
            *_errno() = ENOENT;
            return 0;
        }
        HeapFree(GetProcessHeap(), 0, comspec);
        return 1;
    }

    if comspec.is_null() {
        return -1;
    }

    len = wcslen(comspec) + wcslen(cmd) + 5;

    fullcmd = HeapAlloc(GetProcessHeap(), 0, len.wrapping_mul(core::mem::size_of::<wchar_t>() as u32));
    if fullcmd.is_null() {
        HeapFree(GetProcessHeap(), 0, comspec);
        return -1;
    }

    wcscpy(fullcmd, comspec);
    // `L" /c "` is UTF-16, so every character needs its interleaved zero byte. The plain
    // ASCII form cast to `*const wchar_t` pairs ' ' with '/' into one garbage code unit.
    wcscat(fullcmd, win32macros_rust::wide_bytes!(b" \0/\0c\0 \0\0\0") as *const wchar_t);
    wcscat(fullcmd, cmd);

    res = msvcrt_spawn(_P_WAIT, comspec, fullcmd, core::ptr::null_mut(), 1);

    HeapFree(GetProcessHeap(), 0, comspec);
    HeapFree(GetProcessHeap(), 0, fullcmd);
    return res;
}

#[cfg(any())] // `#ifndef __REACTOS__` in process.c (process.c:1248-1312) — never compiled
              // here, and the object symbol table proves it: no arch's process.c.obj
              // defines `system`.
#[no_mangle]
pub unsafe extern "C" fn system(cmd: LPCSTR) -> i32 {
    let mut res: i32 = -1;
    let mut cmdW: *mut wchar_t;

    if cmd.is_null() {
        return _wsystem(core::ptr::null());
    }

    cmdW = wstrdupa_utf8(cmd);
    if !cmdW.is_null() {
        res = _wsystem(cmdW);
        free(cmdW);
    }
    return res;
}

#[no_mangle]
pub unsafe extern "C" fn _loaddll(dllname: *const i8) -> isize {
    let mut dllnameW: *mut wchar_t = core::ptr::null_mut();
    let ret: isize;

    if !dllname.is_null() {
        dllnameW = wstrdupa_utf8(dllname);
        if dllnameW.is_null() {
            return 0;
        }
    }
    ret = LoadLibraryW(dllnameW) as isize;
    free(dllnameW as *mut c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _unloaddll(dll: isize) -> i32 {
    if FreeLibrary(dll as HMODULE) != 0 {
        return 0;
    } else {
        let err = GetLastError() as i32;
        msvcrt_set_errno(err);
        return err;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _getdllprocaddr(
    dll: isize,
    name: *const i8,
    ordinal: i32,
) -> *mut core::ffi::c_void {
    if !name.is_null() {
        if ordinal != -1 {
            return core::ptr::null_mut();
        }
        // `FARPROC` is `Option<unsafe extern "stdcall" fn() -> INT_PTR>` (bindgen), not a
        // raw pointer — a Win32 `NULL` return is `None`, not a null fn pointer, so this
        // has to unwrap the option rather than `as`-cast it directly.
        return match GetProcAddress(dll as HMODULE, name) {
            Some(f) => f as *mut c_void,
            None => core::ptr::null_mut(),
        };
    }
    if HIWORD(ordinal as usize) != 0 {
        return core::ptr::null_mut();
    }
    return match GetProcAddress(dll as HMODULE, ordinal as usize as LPCSTR) {
        Some(f) => f as *mut c_void,
        None => core::ptr::null_mut(),
    };
}

#[no_mangle]
pub unsafe extern "C" fn _getpid() -> i32 {
    GetCurrentProcessId() as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll spawn/exec functions
//  *
//  * Copyright 1996,1998 Marcus Meissner
//  * Copyright 1996 Jukka Iivonen
//  * Copyright 1997,2000 Uwe Bonnes
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2007 Hans Leidekker
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
//  * FIXME:
//  * -File handles need some special handling. Sometimes children get
//  *  open file handles, sometimes not. The docs are confusing
//  * -No check for maximum path/argument/environment size is done
//  */
//
// #include <fcntl.h>
// #include <io.h>
// #include <process.h>
// #include <stdarg.h>
//
// #include "msvcrt.h"
// #include <winnls.h>
// #include "mtdll.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// static void msvcrt_search_executable(const wchar_t *name, wchar_t *fullname, int use_path)
// {
//   static const wchar_t suffix[][5] =
//     {L".com", L".exe", L".bat", L".cmd"};
//
//   wchar_t buffer[MAX_PATH];
//   const wchar_t *env, *p, *end;
//   unsigned int i, name_len, path_len;
//   int extension = 1;
//
//   *fullname = '\0';
// #ifndef __REACTOS__
//   msvcrt_set_errno(ERROR_FILE_NOT_FOUND);
// #endif
//
//   end = name + MAX_PATH - 1;
//   for(p = name; p < end; p++)
//       if(!*p) break;
//   name_len = p - name;
//
//   /* FIXME extra-long names are silently truncated */
//   memcpy(buffer, name, name_len * sizeof(wchar_t));
//   buffer[name_len] = '\0';
//
//   /* try current dir first */
//   if (GetFileAttributesW(buffer) != INVALID_FILE_ATTRIBUTES)
//   {
//     wcscpy(fullname, buffer);
//     return;
//   }
//
//   for (p--; p >= name; p--)
//     if (*p == '\\' || *p == '/' || *p == ':' || *p == '.') break;
//
//   /* if there's no extension, try some well-known extensions */
//   if ((p < name || *p != '.') && name_len <= MAX_PATH - 5)
//   {
//     for (i = 0; i < 4; i++)
//     {
//       memcpy(buffer + name_len, suffix[i], 5 * sizeof(wchar_t));
//       if (GetFileAttributesW(buffer) != INVALID_FILE_ATTRIBUTES)
//       {
//         wcscpy(fullname, buffer);
//         return;
//       }
//     }
//     extension = 0;
//   }
//
//   if (!use_path || !(env = _wgetenv(L"PATH"))) return;
//
//   /* now try search path */
//   do
//   {
//     p = env;
//     while (*p && *p != ';') p++;
//     if (p == env) return;
//
//     path_len = p - env;
//     if (path_len + name_len <= MAX_PATH - 2)
//     {
//       memcpy(buffer, env, path_len * sizeof(wchar_t));
//       if (buffer[path_len] != '/' && buffer[path_len] != '\\')
//       {
//         buffer[path_len++] = '\\';
//         buffer[path_len] = '\0';
//       }
//       else buffer[path_len] = '\0';
//
//       wcscat(buffer, name);
//       if (GetFileAttributesW(buffer) != INVALID_FILE_ATTRIBUTES)
//       {
//         wcscpy(fullname, buffer);
//         return;
//       }
//     }
//     /* again, if there's no extension, try some well-known extensions */
//     if (!extension && path_len + name_len <= MAX_PATH - 5)
//     {
//       for (i = 0; i < 4; i++)
//       {
//         memcpy(buffer + path_len + name_len, suffix[i], 5 * sizeof(wchar_t));
//         if (GetFileAttributesW(buffer) != INVALID_FILE_ATTRIBUTES)
//         {
//           wcscpy(fullname, buffer);
//           return;
//         }
//       }
//     }
//     env = *p ? p + 1 : p;
//   } while(1);
// }
//
// static intptr_t msvcrt_spawn(int flags, const wchar_t* exe, wchar_t* cmdline,
//                                     wchar_t* env, int use_path)
// {
//   STARTUPINFOW si;
//   PROCESS_INFORMATION pi;
//   wchar_t fullname[MAX_PATH];
//   DWORD create_flags = CREATE_UNICODE_ENVIRONMENT;
//
//   TRACE("%x %s %s %s %d\n", flags, debugstr_w(exe), debugstr_w(cmdline), debugstr_w(env), use_path);
//
//   if ((unsigned)flags > _P_DETACH)
//   {
//     *_errno() = EINVAL;
//     return -1;
//   }
//
//   msvcrt_search_executable(exe, fullname, use_path);
//
//   memset(&si, 0, sizeof(si));
//   si.cb = sizeof(si);
//   msvcrt_create_io_inherit_block(&si.cbReserved2, &si.lpReserved2);
//   if (flags == _P_DETACH) create_flags |= DETACHED_PROCESS;
//   if (!CreateProcessW(fullname, cmdline, NULL, NULL, TRUE,
//                       create_flags, env, NULL, &si, &pi))
//   {
//     msvcrt_set_errno(GetLastError());
//     free(si.lpReserved2);
//     return -1;
//   }
//
//   free(si.lpReserved2);
//   switch(flags)
//   {
//   case _P_WAIT:
//     WaitForSingleObject(pi.hProcess, INFINITE);
//     GetExitCodeProcess(pi.hProcess,&pi.dwProcessId);
//     CloseHandle(pi.hProcess);
//     CloseHandle(pi.hThread);
//     return pi.dwProcessId;
//   case _P_DETACH:
//     CloseHandle(pi.hProcess);
//     pi.hProcess = 0;
//     /* fall through */
//   case _P_NOWAIT:
//   case _P_NOWAITO:
//     CloseHandle(pi.hThread);
//     return (intptr_t)pi.hProcess;
//   case  _P_OVERLAY:
//     _exit(0);
//   }
//   return -1; /* can't reach here */
// }
//
// /* INTERNAL: Convert wide argv list to a single 'delim'-separated wide string, with an
//  * extra '\0' to terminate it.
//  */
// static wchar_t* msvcrt_argvtos(const wchar_t* const* arg, wchar_t delim)
// {
//   const wchar_t* const* a;
//   int size;
//   wchar_t* p;
//   wchar_t* ret;
//
//   if (!arg)
//   {
//       /* Return NULL for an empty environment list */
//       return NULL;
//   }
//
//   /* get length */
//   a = arg;
//   size = 0;
//   while (*a)
//   {
//     size += wcslen(*a) + 1;
//     a++;
//   }
//
//   ret = malloc((size + 1) * sizeof(wchar_t));
//   if (!ret)
//     return NULL;
//
//   /* fill string */
//   a = arg;
//   p = ret;
//   while (*a)
//   {
//     int len = wcslen(*a);
//     memcpy(p,*a,len * sizeof(wchar_t));
//     p += len;
//     *p++ = delim;
//     a++;
//   }
//   if (delim && p > ret) p[-1] = 0;
//   else *p = 0;
//   return ret;
// }
//
// /* INTERNAL: Convert ansi argv list to a single wide string, use ' ' or '\0'
//  * as separator depending on env argument.
//  */
// static wchar_t *msvcrt_argvtos_aw(const char * const *arg, BOOL env)
// {
//   UINT cp = env ? CP_ACP : get_aw_cp();
//   const char * const *a;
//   unsigned int len;
//   wchar_t *p, *ret;
//
//   if (!arg)
//   {
//       /* Return NULL for an empty environment list */
//       return NULL;
//   }
//
//   /* get length */
//   a = arg;
//   len = 0;
//   while (*a)
//   {
//     len += MultiByteToWideChar(cp, 0, *a, -1, NULL, 0);
//     a++;
//   }
//
//   ret = malloc((len + 1) * sizeof(wchar_t));
//   if (!ret)
//     return NULL;
//
//   /* fill string */
//   a = arg;
//   p = ret;
//   while (*a)
//   {
//     p += MultiByteToWideChar(cp, 0, *a, strlen(*a), p, len - (p - ret));
//     *p++ = env ? 0 : ' ';
//     a++;
//   }
//   if (!env && p > ret) p[-1] = 0;
//   else *p = 0;
//   return ret;
// }
//
// /* INTERNAL: Convert wide va_list to a single 'delim'-separated wide string, with an
//  * extra '\0' to terminate it.
//  */
// static wchar_t *msvcrt_valisttos(const wchar_t *arg0, va_list alist, wchar_t delim)
// {
//     unsigned int size = 0, pos = 0;
//     const wchar_t *arg;
//     wchar_t *new, *ret = NULL;
//
//     for (arg = arg0; arg; arg = va_arg( alist, wchar_t * ))
//     {
//         unsigned int len = wcslen( arg ) + 1;
//         if (pos + len >= size)
//         {
//             size = max( 256, size * 2 );
//             size = max( size, pos + len + 1 );
//             if (!(new = realloc( ret, size * sizeof(wchar_t) )))
//             {
//                 free( ret );
//                 return NULL;
//             }
//             ret = new;
//         }
//         wcscpy( ret + pos, arg );
//         pos += len;
//         ret[pos - 1] = delim;
//     }
//     if (pos)
//     {
//         if (delim) ret[pos - 1] = 0;
//         else ret[pos] = 0;
//     }
//     return ret;
// }
//
// /* INTERNAL: Convert ansi va_list to a single 'delim'-separated wide string, with an
//  * extra '\0' to terminate it.
//  */
// static wchar_t *msvcrt_valisttos_aw(const char *arg0, va_list alist, wchar_t delim)
// {
//     unsigned int size = 0, pos = 0;
//     const char *arg;
//     wchar_t *new, *ret = NULL;
//
//     for (arg = arg0; arg; arg = va_arg( alist, char * ))
//     {
//         unsigned int len = convert_acp_utf8_to_wcs( arg, NULL, 0 );
//         if (pos + len >= size)
//         {
//             size = max( 256, size * 2 );
//             size = max( size, pos + len + 1 );
//             if (!(new = realloc( ret, size * sizeof(wchar_t) )))
//             {
//                 free( ret );
//                 return NULL;
//             }
//             ret = new;
//         }
//         pos += convert_acp_utf8_to_wcs( arg, ret + pos, size - pos );
//         ret[pos - 1] = delim;
//     }
//     if (pos)
//     {
//         if (delim) ret[pos - 1] = 0;
//         else ret[pos] = 0;
//     }
//     return ret;
// }
//
// /* INTERNAL: retrieve COMSPEC environment variable */
// static wchar_t *msvcrt_get_comspec(void)
// {
//   wchar_t *ret;
//   unsigned int len;
//
//   if (!(len = GetEnvironmentVariableW(L"COMSPEC", NULL, 0))) len = 4;
//   if ((ret = HeapAlloc(GetProcessHeap(), 0, len * sizeof(wchar_t))))
//   {
//     if (!GetEnvironmentVariableW(L"COMSPEC", ret, len)) wcscpy(ret, L"cmd");
//   }
//   return ret;
// }
//
// /*********************************************************************
//  *		_cwait (MSVCRT.@)
//  */
// intptr_t CDECL _cwait(int *status, intptr_t pid, int action)
// {
//   HANDLE hPid = (HANDLE)pid;
//   int doserrno;
//
//   if (!WaitForSingleObject(hPid, INFINITE))
//   {
//     if (status)
//     {
//       DWORD stat;
//       GetExitCodeProcess(hPid, &stat);
//       *status = (int)stat;
//     }
//     return pid;
//   }
//   doserrno = GetLastError();
//
//   if (doserrno == ERROR_INVALID_HANDLE)
//   {
//     *_errno() =  ECHILD;
//     *__doserrno() = doserrno;
//   }
//   else
//     msvcrt_set_errno(doserrno);
//
//   return status ? *status = -1 : -1;
// }
//
// /*********************************************************************
//  *      _wexecl (MSVCRT.@)
//  *
//  * Unicode version of _execl
//  */
// intptr_t __cdecl _wexecl(const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, name, args, NULL, 0);
//
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *		_execl (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t __cdecl _execl(const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, NULL, 0);
//
//   free(nameW);
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wexecle (MSVCRT.@)
//  *
//  * Unicode version of _execle
//  */
// intptr_t __cdecl _wexecle(const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args, *envs = NULL;
//   const wchar_t * const *envp;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, wchar_t * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const wchar_t * const * );
//   if (envp) envs = msvcrt_argvtos(envp, 0);
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, name, args, envs, 0);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_execle (MSVCRT.@)
//  */
// intptr_t __cdecl _execle(const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args, *envs = NULL;
//   const char * const *envp;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, char * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const char * const * );
//   if (envp) envs = msvcrt_argvtos_aw(envp, TRUE);
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, envs, 0);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wexeclp (MSVCRT.@)
//  *
//  * Unicode version of _execlp
//  */
// intptr_t __cdecl _wexeclp(const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, name, args, NULL, 1);
//
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *		_execlp (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t __cdecl _execlp(const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, NULL, 1);
//
//   free(nameW);
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wexeclpe (MSVCRT.@)
//  *
//  * Unicode version of _execlpe
//  */
// intptr_t __cdecl _wexeclpe(const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args, *envs = NULL;
//   const wchar_t * const *envp;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, wchar_t * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const wchar_t * const * );
//   if (envp) envs = msvcrt_argvtos(envp, 0);
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, name, args, envs, 1);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_execlpe (MSVCRT.@)
//  */
// intptr_t __cdecl _execlpe(const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args, *envs = NULL;
//   const char * const *envp;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, char * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const char * const * );
//   if (envp) envs = msvcrt_argvtos_aw(envp, TRUE);
//   va_end(ap);
//
//   ret = msvcrt_spawn(_P_OVERLAY as i32, nameW, args, envs, 1);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wexecv (MSVCRT.@)
//  *
//  * Unicode version of _execv
//  */
// intptr_t CDECL _wexecv(const wchar_t* name, const wchar_t* const* argv)
// {
//   return _wspawnve(_P_OVERLAY as i32, name, argv, NULL);
// }
//
// /*********************************************************************
//  *		_execv (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _execv(const char* name, const char* const* argv)
// {
//   return _spawnve(_P_OVERLAY as i32, name, argv, NULL);
// }
//
// /*********************************************************************
//  *      _wexecve (MSVCRT.@)
//  *
//  * Unicode version of _execve
//  */
// intptr_t CDECL _wexecve(const wchar_t* name, const wchar_t* const* argv, const wchar_t* const* envv)
// {
//   return _wspawnve(_P_OVERLAY as i32, name, argv, envv);
// }
//
// /*********************************************************************
//  *		_execve (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _execve(const char* name, const char* const* argv, const char* const* envv)
// {
//   return _spawnve(_P_OVERLAY as i32, name, argv, envv);
// }
//
// /*********************************************************************
//  *      _wexecvpe (MSVCRT.@)
//  *
//  * Unicode version of _execvpe
//  */
// intptr_t CDECL _wexecvpe(const wchar_t* name, const wchar_t* const* argv, const wchar_t* const* envv)
// {
//   return _wspawnvpe(_P_OVERLAY as i32, name, argv, envv);
// }
//
// /*********************************************************************
//  *		_execvpe (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _execvpe(const char* name, const char* const* argv, const char* const* envv)
// {
//   return _spawnvpe(_P_OVERLAY as i32, name, argv, envv);
// }
//
// /*********************************************************************
//  *      _wexecvp (MSVCRT.@)
//  *
//  * Unicode version of _execvp
//  */
// intptr_t CDECL _wexecvp(const wchar_t* name, const wchar_t* const* argv)
// {
//   return _wexecvpe(name, argv, NULL);
// }
//
// /*********************************************************************
//  *		_execvp (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _execvp(const char* name, const char* const* argv)
// {
//   return _execvpe(name, argv, NULL);
// }
//
// /*********************************************************************
//  *      _wspawnl (MSVCRT.@)
//  *
//  * Unicode version of _spawnl
//  */
// intptr_t __cdecl _wspawnl(int flags, const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, name, args, NULL, 0);
//
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnl (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t __cdecl _spawnl(int flags, const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, nameW, args, NULL, 0);
//
//   free(nameW);
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wspawnle (MSVCRT.@)
//  *
//  * Unicode version of _spawnle
//  */
// intptr_t __cdecl _wspawnle(int flags, const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args, *envs = NULL;
//   const wchar_t * const *envp;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, wchar_t * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const wchar_t * const * );
//   if (envp) envs = msvcrt_argvtos(envp, 0);
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, name, args, envs, 0);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnle (MSVCRT.@)
//  */
// intptr_t __cdecl _spawnle(int flags, const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args, *envs = NULL;
//   const char * const *envp;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, char * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const char * const * );
//   if (envp) envs = msvcrt_argvtos_aw(envp, TRUE);
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, nameW, args, envs, 0);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wspawnlp (MSVCRT.@)
//  *
//  * Unicode version of _spawnlp
//  */
// intptr_t __cdecl _wspawnlp(int flags, const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, name, args, NULL, 1);
//
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnlp (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t __cdecl _spawnlp(int flags, const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, nameW, args, NULL, 1);
//
//   free(nameW);
//   free(args);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wspawnlpe (MSVCRT.@)
//  *
//  * Unicode version of _spawnlpe
//  */
// intptr_t __cdecl _wspawnlpe(int flags, const wchar_t* name, const wchar_t* arg0, ...)
// {
//   va_list ap;
//   wchar_t *args, *envs = NULL;
//   const wchar_t * const *envp;
//   intptr_t ret;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, wchar_t * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const wchar_t * const * );
//   if (envp) envs = msvcrt_argvtos(envp, 0);
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, name, args, envs, 1);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnlpe (MSVCRT.@)
//  */
// intptr_t __cdecl _spawnlpe(int flags, const char* name, const char* arg0, ...)
// {
//   va_list ap;
//   wchar_t *nameW, *args, *envs = NULL;
//   const char * const *envp;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   va_start(ap, arg0);
//   args = msvcrt_valisttos_aw(arg0, ap, ' ');
//   va_end(ap);
//
//   va_start(ap, arg0);
//   while (va_arg( ap, char * ) != NULL) /*nothing*/;
//   envp = va_arg( ap, const char * const * );
//   if (envp) envs = msvcrt_argvtos_aw(envp, TRUE);
//   va_end(ap);
//
//   ret = msvcrt_spawn(flags, nameW, args, envs, 1);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnve (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _spawnve(int flags, const char* name, const char* const* argv,
//                                const char* const* envv)
// {
//   wchar_t *nameW, *args, *envs;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   args = msvcrt_argvtos_aw(argv, FALSE);
//   envs = msvcrt_argvtos_aw(envv, TRUE);
//
//   ret = msvcrt_spawn(flags, nameW, args, envs, 0);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wspawnve (MSVCRT.@)
//  *
//  * Unicode version of _spawnve
//  */
// intptr_t CDECL _wspawnve(int flags, const wchar_t* name, const wchar_t* const* argv,
//                                 const wchar_t* const* envv)
// {
//   wchar_t *args, *envs;
//   intptr_t ret;
//
//   args = msvcrt_argvtos(argv, ' ');
//   envs = msvcrt_argvtos(envv, 0);
//
//   ret = msvcrt_spawn(flags, name, args, envs, 0);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnv (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _spawnv(int flags, const char* name, const char* const* argv)
// {
//   return _spawnve(flags, name, argv, NULL);
// }
//
// /*********************************************************************
//  *      _wspawnv (MSVCRT.@)
//  *
//  * Unicode version of _spawnv
//  */
// intptr_t CDECL _wspawnv(int flags, const wchar_t* name, const wchar_t* const* argv)
// {
//   return _wspawnve(flags, name, argv, NULL);
// }
//
// /*********************************************************************
//  *		_spawnvpe (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _spawnvpe(int flags, const char* name, const char* const* argv,
//                                 const char* const* envv)
// {
//   wchar_t *nameW, *args, *envs;
//   intptr_t ret;
//
//   if (!(nameW = wstrdupa_utf8(name))) return -1;
//
//   args = msvcrt_argvtos_aw(argv, FALSE);
//   envs = msvcrt_argvtos_aw(envv, TRUE);
//
//   ret = msvcrt_spawn(flags, nameW, args, envs, 1);
//
//   free(nameW);
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *      _wspawnvpe (MSVCRT.@)
//  *
//  * Unicode version of _spawnvpe
//  */
// intptr_t CDECL _wspawnvpe(int flags, const wchar_t* name, const wchar_t* const* argv,
//                                  const wchar_t* const* envv)
// {
//   wchar_t *args, *envs;
//   intptr_t ret;
//
//   args = msvcrt_argvtos(argv, ' ');
//   envs = msvcrt_argvtos(envv, 0);
//
//   ret = msvcrt_spawn(flags, name, args, envs, 1);
//
//   free(args);
//   free(envs);
//   return ret;
// }
//
// /*********************************************************************
//  *		_spawnvp (MSVCRT.@)
//  *
//  * Like on Windows, this function does not handle arguments with spaces
//  * or double-quotes.
//  */
// intptr_t CDECL _spawnvp(int flags, const char* name, const char* const* argv)
// {
//   return _spawnvpe(flags, name, argv, NULL);
// }
//
// /*********************************************************************
//  *      _wspawnvp (MSVCRT.@)
//  *
//  * Unicode version of _spawnvp
//  */
// intptr_t CDECL _wspawnvp(int flags, const wchar_t* name, const wchar_t* const* argv)
// {
//   return _wspawnvpe(flags, name, argv, NULL);
// }
//
// static struct popen_handle {
//     FILE *f;
//     HANDLE proc;
// } *popen_handles;
// static DWORD popen_handles_size;
//
// void msvcrt_free_popen_data(void)
// {
//     free(popen_handles);
// }
//
// /*********************************************************************
//  *		_wpopen (MSVCRT.@)
//  *
//  * Unicode version of _popen
//  */
// FILE*  _wpopen(const wchar_t* command, const wchar_t* mode)
// {
//     wchar_t *comspec, *fullcmd, fullname[MAX_PATH];
//     int textmode, fds[2], fd_parent, fd_child;
//     struct popen_handle *container;
//     PROCESS_INFORMATION pi;
//     BOOL read_pipe = TRUE;
//     const wchar_t *p;
//     unsigned int len;
//     STARTUPINFOW si;
//     FILE *ret;
//     DWORD i;
//     BOOL r;
//
//     TRACE("(command=%s, mode=%s)\n", debugstr_w(command), debugstr_w(mode));
//
//     if (!command || !mode)
//         return NULL;
//
//     _get_fmode(&textmode);
//     textmode &= _O_BINARY | _O_TEXT;
//     textmode |= _O_NOINHERIT;
//     for (p = mode; *p; p++)
//     {
//         switch (*p)
//         {
//         case 'W':
//         case 'w':
//             read_pipe = FALSE;
//             break;
//         case 'B':
//         case 'b':
//             textmode |= _O_BINARY;
//             textmode &= ~_O_TEXT;
//             break;
//         case 'T':
//         case 't':
//             textmode |= _O_TEXT;
//             textmode &= ~_O_BINARY;
//             break;
//         }
//     }
//     if (_pipe(fds, 0, textmode) == -1)
//         return NULL;
//
//     if (read_pipe)
//     {
//         fd_parent = fds[0];
//         fd_child = _dup(fds[1]);
//         _close(fds[1]);
//     }
//     else
//     {
//         fd_parent = fds[1];
//         fd_child = _dup(fds[0]);
//         _close(fds[0]);
//     }
//     if (fd_child == -1)
//     {
//         _close(fd_parent);
//         return NULL;
//     }
//     ret = _wfdopen(fd_parent, mode);
//     if (!ret)
//     {
//         _close(fd_child);
//         return NULL;
//     }
//
//     _lock(_POPEN_LOCK);
//     for (i = 0; i < popen_handles_size; i++)
//     {
//         if (!popen_handles[i].f)
//             break;
//     }
//     if (i == popen_handles_size)
//     {
//         i = (popen_handles_size ? popen_handles_size * 2 : 8);
//         container = realloc(popen_handles, i * sizeof(*container));
//         if (!container) goto error;
//
//         popen_handles = container;
//         container = popen_handles + popen_handles_size;
//         memset(container, 0, (i - popen_handles_size) * sizeof(*container));
//         popen_handles_size = i;
//     }
//     else container = popen_handles + i;
//
//     if (!(comspec = msvcrt_get_comspec())) goto error;
//     len = wcslen(comspec) + wcslen(command) + 5;
//
//     if (!(fullcmd = HeapAlloc(GetProcessHeap(), 0, len * sizeof(wchar_t))))
//     {
//         HeapFree(GetProcessHeap(), 0, comspec);
//         goto error;
//     }
//
//     wcscpy(fullcmd, comspec);
//     wcscat(fullcmd, L" /c ");
//     wcscat(fullcmd, command);
//     msvcrt_search_executable(comspec, fullname, 1);
//
//     memset(&si, 0, sizeof(si));
//     si.cb = sizeof(si);
//     si.dwFlags = STARTF_USESTDHANDLES;
//     if (read_pipe)
//     {
//         si.hStdInput = (HANDLE)_get_osfhandle(STDIN_FILENO);
//         si.hStdOutput = (HANDLE)_get_osfhandle(fd_child);
//     }
//     else
//     {
//         si.hStdInput = (HANDLE)_get_osfhandle(fd_child);
//         si.hStdOutput = (HANDLE)_get_osfhandle(STDOUT_FILENO);
//     }
//     si.hStdError = (HANDLE)_get_osfhandle(STDERR_FILENO);
//     r = CreateProcessW(fullname, fullcmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi);
//     HeapFree(GetProcessHeap(), 0, comspec);
//     HeapFree(GetProcessHeap(), 0, fullcmd);
//     if (!r)
//     {
//         msvcrt_set_errno(GetLastError());
//         goto error;
//     }
//     CloseHandle(pi.hThread);
//     _close(fd_child);
//     container->proc = pi.hProcess;
//     container->f = ret;
//     _unlock(_POPEN_LOCK);
//     return ret;
//
// error:
//     _unlock(_POPEN_LOCK);
//     _close(fd_child);
//     fclose(ret);
//     return NULL;
// }
//
// /*********************************************************************
//  *      _popen (MSVCRT.@)
//  */
// FILE*  _popen(const char* command, const char* mode)
// {
//   FILE *ret;
//   wchar_t *cmdW, *modeW;
//
//   TRACE("(command=%s, mode=%s)\n", debugstr_a(command), debugstr_a(mode));
//
//   if (!command || !mode)
//     return NULL;
//
//   if (!(cmdW = wstrdupa_utf8(command))) return NULL;
//   if (!(modeW = msvcrt_wstrdupa(mode)))
//   {
//     free(cmdW);
//     return NULL;
//   }
//
//   ret = _wpopen(cmdW, modeW);
//
//   free(cmdW);
//   free(modeW);
//   return ret;
// }
//
// /*********************************************************************
//  *		_pclose (MSVCRT.@)
//  */
// int CDECL _pclose(FILE* file)
// {
//   HANDLE h;
//   DWORD i;
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return -1;
//
//   _lock(_POPEN_LOCK);
//   for(i=0; i<popen_handles_size; i++)
//   {
//     if (popen_handles[i].f == file)
//       break;
//   }
//   if(i == popen_handles_size)
//   {
//     _unlock(_POPEN_LOCK);
//     *_errno() = EBADF;
//     return -1;
//   }
//
//   h = popen_handles[i].proc;
//   popen_handles[i].f = NULL;
//   _unlock(_POPEN_LOCK);
//
//   fclose(file);
//   if(WaitForSingleObject(h, INFINITE)==WAIT_FAILED || !GetExitCodeProcess(h, &i))
//   {
//     msvcrt_set_errno(GetLastError());
//     CloseHandle(h);
//     return -1;
//   }
//
//   CloseHandle(h);
//   return i;
// }
//
// #ifndef __REACTOS__
// /*********************************************************************
//  *      _wsystem (MSVCRT.@)
//  *
//  * Unicode version of system
//  */
// int CDECL _wsystem(const wchar_t* cmd)
// {
//   int res;
//   wchar_t *comspec, *fullcmd;
//   unsigned int len;
//
//   comspec = msvcrt_get_comspec();
//
//   if (cmd == NULL)
//   {
//     if (comspec == NULL)
//     {
//         *_errno() = ENOENT;
//         return 0;
//     }
//     HeapFree(GetProcessHeap(), 0, comspec);
//     return 1;
//   }
//
//   if (comspec == NULL)
//     return -1;
//
//   len = wcslen(comspec) + wcslen(cmd) + 5;
//
//   if (!(fullcmd = HeapAlloc(GetProcessHeap(), 0, len * sizeof(wchar_t))))
//   {
//     HeapFree(GetProcessHeap(), 0, comspec);
//     return -1;
//   }
//   wcscpy(fullcmd, comspec);
//   wcscat(fullcmd, L" /c ");
//   wcscat(fullcmd, cmd);
//
//   res = msvcrt_spawn(_P_WAIT, comspec, fullcmd, NULL, 1);
//
//   HeapFree(GetProcessHeap(), 0, comspec);
//   HeapFree(GetProcessHeap(), 0, fullcmd);
//   return res;
// }
//
// /*********************************************************************
//  *		system (MSVCRT.@)
//  */
// int CDECL system(const char* cmd)
// {
//   int res = -1;
//   wchar_t *cmdW;
//
//   if (cmd == NULL)
//     return _wsystem(NULL);
//
//   if ((cmdW = wstrdupa_utf8(cmd)))
//   {
//     res = _wsystem(cmdW);
//     free(cmdW);
//   }
//   return res;
// }
// #endif
//
// /*********************************************************************
//  *		_loaddll (MSVCRT.@)
//  */
// intptr_t CDECL _loaddll(const char* dllname)
// {
//     wchar_t *dllnameW = NULL;
//     intptr_t ret;
//
//     if (dllname && !(dllnameW = wstrdupa_utf8(dllname))) return 0;
//     ret = (intptr_t)LoadLibraryW(dllnameW);
//     free(dllnameW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_unloaddll (MSVCRT.@)
//  */
// int CDECL _unloaddll(intptr_t dll)
// {
//   if (FreeLibrary((HMODULE)dll))
//     return 0;
//   else
//   {
//     int err = GetLastError();
//     msvcrt_set_errno(err);
//     return err;
//   }
// }
//
// /*********************************************************************
//  *		_getdllprocaddr (MSVCRT.@)
//  */
// void *  _getdllprocaddr(intptr_t dll, const char *name, int ordinal)
// {
//     if (name)
//     {
//         if (ordinal != -1) return NULL;
//         return GetProcAddress( (HMODULE)dll, name );
//     }
//     if (HIWORD(ordinal)) return NULL;
//     return GetProcAddress( (HMODULE)dll, (LPCSTR)(ULONG_PTR)ordinal );
// }
//
// /*********************************************************************
//  *              _getpid (MSVCRT.@)
//  */
// int CDECL _getpid(void)
// {
//     return GetCurrentProcessId();
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
