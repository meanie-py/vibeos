//! `dll/win32/msvcrt/reactos/system.cpp` — `system` and `_wsystem`, 2 exports.
//!
//! The only `.cpp` compiled into msvcrt_static, and it was carried for months as "the C++
//! substrate lane". It is not one. There is no RTTI here, no vtable, no exception object, no
//! C++ ABI anywhere on the boundary — both exports are `extern "C"`, and `throw()` is an
//! empty exception specification, not a use of exceptions. The whole C++-ness is a
//! `template <typename Character>` instantiated at exactly `char` and `wchar_t` and resolved
//! at compile time, which is a Rust generic over a trait, and every callee it needs
//! (`getenv`/`_wgetenv`, `_spawnve`/`_wspawnve`, `_spawnvpe`/`_wspawnvpe`,
//! `_access_s`/`_waccess_s`, `errno`) has been Rust since earlier waves.
//!
//! Not to be confused with process.rs's `system`/`_wsystem`, which are Wine's different
//! implementation and stay `#[cfg(any())]` there: process.c wraps them in `#ifndef
//! __REACTOS__`, so the C compiled into this DLL is THIS file's. Two live definitions of
//! `system` in one archive would be resolved by member order, which is the roulette the
//! cohab rules exist to prevent.

use crate::msvcrt::environ::{_wgetenv, getenv};
use crate::msvcrt::errno::_errno;
use crate::msvcrt::file::{_access_s, _waccess_s};
use crate::msvcrt::process::{_spawnve, _spawnvpe, _wspawnve, _wspawnvpe};
use crate::prelude::*;

// The C spells these as function-local `static Character const name[] = { 'C', 'O', ... }`
// inside the template, one instantiation per character type. Written out element-by-element
// there for the same reason they are here: `L"COMSPEC"` in a template body would not
// instantiate at `char`.
static COMSPEC_A: [c_char; 8] = [
    b'C' as c_char,
    b'O' as c_char,
    b'M' as c_char,
    b'P' as c_char,
    b'S' as c_char,
    b'E' as c_char,
    b'C' as c_char,
    0,
];
static CMD_EXE_A: [c_char; 8] = [
    b'c' as c_char,
    b'm' as c_char,
    b'd' as c_char,
    b'.' as c_char,
    b'e' as c_char,
    b'x' as c_char,
    b'e' as c_char,
    0,
];
static SLASH_C_A: [c_char; 3] = [b'/' as c_char, b'c' as c_char, 0];

static COMSPEC_W: [wchar_t; 8] = [
    b'C' as wchar_t,
    b'O' as wchar_t,
    b'M' as wchar_t,
    b'P' as wchar_t,
    b'S' as wchar_t,
    b'E' as wchar_t,
    b'C' as wchar_t,
    0,
];
static CMD_EXE_W: [wchar_t; 8] = [
    b'c' as wchar_t,
    b'm' as wchar_t,
    b'd' as wchar_t,
    b'.' as wchar_t,
    b'e' as wchar_t,
    b'x' as wchar_t,
    b'e' as wchar_t,
    0,
];
static SLASH_C_W: [wchar_t; 3] = [b'/' as wchar_t, b'c' as wchar_t, 0];

/// The template parameter, as a trait. Each method stands for one `_t*` overload pair in the
/// C++ — `_tgetenv`, `_taccess_s`, `_tspawnve`, `_tspawnvpe` — which exist there only to give
/// the template a single name to call. Overload resolution at compile time becomes trait
/// dispatch at compile time; neither survives into the object.
trait SystemChar: Copy + 'static {
    fn comspec_name() -> *const Self;
    fn cmd_exe() -> *const Self;
    fn slash_c() -> *const Self;
    unsafe fn getenv(name: *const Self) -> *mut Self;
    unsafe fn access_s(filename: *const Self, mode: c_int) -> c_int;
    unsafe fn spawnve(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t;
    unsafe fn spawnvpe(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t;
}

impl SystemChar for c_char {
    fn comspec_name() -> *const Self {
        COMSPEC_A.as_ptr()
    }
    fn cmd_exe() -> *const Self {
        CMD_EXE_A.as_ptr()
    }
    fn slash_c() -> *const Self {
        SLASH_C_A.as_ptr()
    }
    unsafe fn getenv(name: *const Self) -> *mut Self {
        getenv(name)
    }
    unsafe fn access_s(filename: *const Self, mode: c_int) -> c_int {
        _access_s(filename, mode)
    }
    unsafe fn spawnve(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t {
        _spawnve(flags, name, argv, envv)
    }
    unsafe fn spawnvpe(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t {
        _spawnvpe(flags, name, argv, envv)
    }
}

impl SystemChar for wchar_t {
    fn comspec_name() -> *const Self {
        COMSPEC_W.as_ptr()
    }
    fn cmd_exe() -> *const Self {
        CMD_EXE_W.as_ptr()
    }
    fn slash_c() -> *const Self {
        SLASH_C_W.as_ptr()
    }
    unsafe fn getenv(name: *const Self) -> *mut Self {
        _wgetenv(name)
    }
    unsafe fn access_s(filename: *const Self, mode: c_int) -> c_int {
        _waccess_s(filename, mode)
    }
    unsafe fn spawnve(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t {
        _wspawnve(flags, name, argv, envv)
    }
    unsafe fn spawnvpe(
        flags: c_int,
        name: *const Self,
        argv: *const *const Self,
        envv: *const *const Self,
    ) -> intptr_t {
        _wspawnvpe(flags, name, argv, envv)
    }
}

unsafe fn common_system<C: SystemChar>(command: *const C) -> c_int {
    let comspec_value: *const C = C::getenv(C::comspec_name());

    // If the command is null, return TRUE only if %COMSPEC% is set and the file
    // to which it points exists.
    if command.is_null() {
        if comspec_value.is_null() {
            return 0;
        }

        return (C::access_s(comspec_value, 0) == 0) as c_int;
    }

    let mut arguments: [*const C; 4] = [comspec_value, C::slash_c(), command, core::ptr::null()];

    if !comspec_value.is_null() {
        let saved_errno: c_int = *_errno();
        *_errno() = 0;

        let result: c_int = C::spawnve(
            _P_WAIT as c_int,
            arguments[0],
            arguments.as_ptr(),
            core::ptr::null(),
        ) as c_int;
        if result != -1 {
            *_errno() = saved_errno;
            return result;
        }

        if *_errno() != ENOENT as c_int && *_errno() != EACCES as c_int {
            return result;
        }

        // If the error wasn't one of those two errors, try again with cmd.exe...
        *_errno() = saved_errno;
    }

    arguments[0] = C::cmd_exe();
    C::spawnvpe(
        _P_WAIT as c_int,
        arguments[0],
        arguments.as_ptr(),
        core::ptr::null(),
    ) as c_int

    // The C has an unreachable `return 0;` after that tail return. Dropped rather than
    // transcribed: Rust would reject it as unreachable, and a statement no execution can
    // reach carries no behaviour to preserve.
}

#[no_mangle]
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    common_system(command)
}

#[no_mangle]
pub unsafe extern "C" fn _wsystem(command: *const wchar_t) -> c_int {
    common_system(command)
}
