//! Shim template framework -- hand port of dll/appcompat/shims/shimlib/{setup_shim.inl,
//! implement_shim.inl} and the logging/registration parts of shimlib.h.
//!
//! The C shims `#define SHIM_NS <Name>` and `#include <setup_shim.inl>`; the preprocessor then
//! pastes `<Name>_` in front of every per-shim symbol (SHIM_OBJ_NAME). Rust has no identifier
//! pasting without a proc-macro, so the namespace is a SCOPE instead: a shim lives in its own
//! module (or, for a one-shim file, at file scope), `setup_shim!` declares the per-namespace
//! statics there, and `SHIM_MSG!`/`CALL_SHIM!`/`implement_shim!` resolve `g_szModuleName`,
//! `g_pAPIHooks`, ... by NAME at the call site (macro_rules does not hygiene-rename items).
//! Consequence: those symbols are crate-private instead of C-global `<NS>_g_pAPIHooks`; nothing
//! outside the shim referenced them (shimlib reaches a shim only through the SHIMREG record).
//!
//! shimlib itself (ShimLib_*, SeiDbgPrint, g_ShimEngDebugLevel) stays C, linked from the
//! `shimlib` static library.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::types::*;
use core::ffi::{c_char, c_int, c_void};

// ---- shimlib.h: shim helper API (plain C calling convention, no WINAPI) -----------------------
extern "C" {
    pub fn ShimLib_ShimMalloc(dwSize: SIZE_T) -> PVOID;
    pub fn ShimLib_ShimFree(pData: PVOID);
    pub fn ShimLib_StringDuplicateA(szString: PCSTR) -> PCSTR;
    pub fn ShimLib_StringNDuplicateA(szString: PCSTR, stringLength: SIZE_T) -> PCSTR;
    pub fn ShimLib_StrAEqualsWNC(szString: PCSTR, wszString: PCWSTR) -> BOOL;
    pub fn ShimLib_Instance() -> HINSTANCE;
    pub fn ShimLib_Init(hInstance: HINSTANCE);
    pub fn ShimLib_Deinit();
    /// `BOOL WINAPIV SeiDbgPrint(SEI_LOG_LEVEL, PCSTR Function, PCSTR Format, ...)`: WINAPIV is
    /// cdecl on every arch, i.e. plain "C".
    pub fn SeiDbgPrint(Level: SEI_LOG_LEVEL, Function: PCSTR, Format: PCSTR, ...) -> BOOL;
}
// WINAPI entry points of shimlib.c.
extern "system" {
    pub fn ShimLib_GetHookAPIs(szCommandLine: LPCSTR, wszShimName: LPCWSTR, pdwHookCount: PDWORD) -> PHOOKAPI;
    pub fn ShimLib_NotifyShims(fdwReason: DWORD, ptr: PVOID) -> BOOL;
}

// SHIM_REASON_* / SHIM_NOTIFY_* come from the bindgen island (shimlib.h).

/// A NUL-terminated `PCSTR` from a string literal.
#[macro_export]
macro_rules! shim_pcstr {
    ($s:literal) => {
        concat!($s, "\0").as_ptr() as $crate::types::PCSTR
    };
}

/// SHIM_HOOK's `function` argument: a name (literal) or an ordinal (`[23]` for `(PCSTR)23`).
#[macro_export]
macro_rules! shim_hook_name {
    ([$ord:expr]) => {
        ($ord as usize) as $crate::types::PCSTR
    };
    ($s:literal) => {
        $crate::shim_pcstr!($s)
    };
}

/// shimlib.h SHIM_MSG / SHIM_FAIL / SHIM_WARN / SHIM_INFO. `fmt` must be a string literal (the
/// NUL is appended here); the C `SeiDbgPrint` variadic is CALLED, never defined, from Rust.
/// Needs `g_szModuleName` (from `setup_shim!`) in scope, exactly as the C needs SHIM_NS.
#[macro_export]
macro_rules! SHIM_LOG {
    ($level:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $crate::types::g_ShimEngDebugLevel != 0 {
            $crate::shim::SeiDbgPrint(
                $crate::types::$level,
                g_szModuleName,
                concat!($fmt, "\0").as_ptr() as $crate::types::PCSTR
                $(, $arg)*
            );
        }
    }};
}
#[macro_export]
macro_rules! SHIM_MSG {
    ($($t:tt)*) => { $crate::SHIM_LOG!(_SEI_LOG_LEVEL_SEI_MSG, $($t)*) };
}
#[macro_export]
macro_rules! SHIM_FAIL {
    ($($t:tt)*) => { $crate::SHIM_LOG!(_SEI_LOG_LEVEL_SEI_FAIL, $($t)*) };
}
#[macro_export]
macro_rules! SHIM_WARN {
    ($($t:tt)*) => { $crate::SHIM_LOG!(_SEI_LOG_LEVEL_SEI_WARN, $($t)*) };
}
#[macro_export]
macro_rules! SHIM_INFO {
    ($($t:tt)*) => { $crate::SHIM_LOG!(_SEI_LOG_LEVEL_SEI_INFO, $($t)*) };
}

/// setup_shim.inl `CALL_SHIM(num, PROCTYPE)`: the original function the engine stored for hook
/// `num`, as `PROCTYPE`. Use it in expression position: `CALL_SHIM!(0, T)(args)`.
#[macro_export]
macro_rules! CALL_SHIM {
    ($num:expr, $proc:ty) => {
        core::mem::transmute::<$crate::types::PVOID, $proc>((*g_pAPIHooks.add($num)).OriginalFunction)
    };
}

/// setup_shim.inl's tail: the per-namespace statics. Invoke once per shim, at the scope the
/// shim's functions live in. (`g_szCommandLine` / `g_pAPIHooks` are filled by GetHookAPIs.)
#[macro_export]
macro_rules! setup_shim {
    ($ns:ident) => {
        pub static mut g_szModuleName: $crate::types::PCSTR =
            concat!(stringify!($ns), "\0").as_ptr() as $crate::types::PCSTR;
        pub static mut g_szCommandLine: $crate::types::PCSTR = core::ptr::null();
        pub static mut g_pAPIHooks: $crate::types::PHOOKAPI = core::ptr::null_mut();
    };
}

/// implement_shim.inl: `GetHookAPIs` plus the SHIMREG record in `.shm$BBB`.
///
/// `implement_shim!(NS, num_hooks, notify, [ (num, "DLL", "Function" | [ordinal], target), ... ])`
/// where `notify` is `Some(Notify)` (SHIM_NOTIFY_FN) or `None`, and `target` is anything that
/// casts to `usize` (a function item, or a PVOID read at GetHookAPIs time like msys2's decoy).
#[macro_export]
macro_rules! implement_shim {
    ($ns:ident, $num_hooks:expr, $notify:expr,
     [ $( ($idx:expr, $dll:tt, $func:tt, $target:expr) ),* $(,)? ]) => {
        pub unsafe extern "system" fn GetHookAPIs(
            fdwReason: $crate::types::DWORD,
            pszCmdLine: $crate::types::PCSTR,
            pdwHookCount: $crate::types::PDWORD,
        ) -> $crate::types::PHOOKAPI {
            if !pszCmdLine.is_null() {
                g_szCommandLine = $crate::shim::ShimLib_StringDuplicateA(pszCmdLine);
            } else {
                g_szCommandLine = b"\0".as_ptr() as $crate::types::PCSTR;
            }
            g_pAPIHooks = $crate::shim::ShimLib_ShimMalloc(
                (core::mem::size_of::<$crate::types::HOOKAPI>() * ($num_hooks as usize))
                    as $crate::types::SIZE_T,
            ) as $crate::types::PHOOKAPI;
            if ($num_hooks as usize) != 0 {
                $crate::prelude::memset(
                    g_pAPIHooks as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<$crate::types::HOOKAPI>() * ($num_hooks as usize),
                );
            }

            // SHIM_NOTIFY_FN
            let notify: Option<unsafe extern "system" fn($crate::types::DWORD, $crate::types::PVOID) -> $crate::types::BOOL> = $notify;
            if let Some(notify_fn) = notify {
                if notify_fn(fdwReason, core::ptr::null_mut()) == 0 {
                    *pdwHookCount = 0;
                    // Do not free the memory here
                    // Divergence from the C: it passed the Notify FUNCTION POINTER to "%s"
                    // (SHIM_FAIL("%s failed during attach\n", SHIM_OBJ_NAME(Notify))), which
                    // prints machine code. Pass the name the message means.
                    $crate::SHIM_FAIL!(
                        "%s failed during attach\n",
                        concat!(stringify!($ns), "_Notify\0").as_ptr() as $crate::types::PCSTR
                    );
                    return core::ptr::null_mut();
                }
            }

            *pdwHookCount = ($num_hooks as usize) as $crate::types::DWORD;
            // SHIM_SETUP_HOOKS
            $(
                {
                    let hook = &mut *g_pAPIHooks.add($idx);
                    hook.LibraryName = $crate::shim_pcstr!($dll);
                    hook.FunctionName = $crate::shim_hook_name!($func);
                    hook.ReplacementFunction = ($target as usize) as $crate::types::PVOID;
                    hook.OriginalFunction = core::ptr::null_mut();
                    hook.Reserved[0] = core::ptr::null_mut();
                    hook.Reserved[1] = core::ptr::null_mut();
                }
            )*
            g_pAPIHooks
        }

        // _SHMALLOC(".shm$BBB"): shimlib.c brackets the table with _shim_start/_shim_end in
        // .shm$AAA/.shm$ZZZ. `static mut` because SHIMREG holds raw pointers (not Sync) and the
        // C record is writable data too.
        #[used]
        #[link_section = ".shm$BBB"]
        pub static mut _shim_fn: $crate::types::SHIMREG = $crate::types::SHIMREG {
            // The island spells these fn-pointer types per arch ("system" in some crates, "C" or
            // "stdcall" in others); Rust compares fn ABIs nominally, and the real ABI of a WINAPI
            // callback is "system", so state it here and transmute into whatever the island says.
            GetHookAPIs: unsafe { core::mem::transmute::<
                Option<unsafe extern "system" fn($crate::types::DWORD, $crate::types::PCSTR, $crate::types::PDWORD) -> $crate::types::PHOOKAPI>,
                $crate::types::_PVGetHookAPIs,
            >(Some(GetHookAPIs)) },
            Notify: unsafe { core::mem::transmute::<
                Option<unsafe extern "system" fn($crate::types::DWORD, $crate::types::PVOID) -> $crate::types::BOOL>,
                $crate::types::_PVNotify,
            >($notify) },
            ShimName: concat!(stringify!($ns), "\0").as_ptr() as $crate::types::PCSTR,
        };
    };
}

// ---- strsafe.h narrow ("A") forms --------------------------------------------------------------
// strsafe-rust ports only the wide family; these shims call a few A forms, so the exact
// strsafe.h bodies (StringCopyWorkerA, StringCatWorkerA, StringLengthWorkerA,
// StringVPrintfWorkerA's tail) are ported here, over the same HRESULT constants.
use strsafe_rust::{HRESULT, STRSAFE_E_INSUFFICIENT_BUFFER, STRSAFE_E_INVALID_PARAMETER, STRSAFE_MAX_CCH};

unsafe fn StringCopyWorkerA(mut pszDest: *mut c_char, mut cchDest: usize, mut pszSrc: *const c_char) -> HRESULT {
    let mut hr: HRESULT = 0;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        while cchDest != 0 && *pszSrc != 0 {
            *pszDest = *pszSrc;
            pszDest = pszDest.add(1);
            pszSrc = pszSrc.add(1);
            cchDest -= 1;
        }
        if cchDest == 0 {
            pszDest = pszDest.sub(1);
            hr = STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest = 0;
    }
    hr
}

unsafe fn StringLengthWorkerA(mut psz: *const c_char, mut cchMax: usize, pcchLength: *mut usize) -> HRESULT {
    let mut hr: HRESULT = 0;
    let cchMaxPrev = cchMax;
    while cchMax != 0 && *psz != 0 {
        psz = psz.add(1);
        cchMax -= 1;
    }
    if cchMax == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcchLength.is_null() {
        if hr >= 0 {
            *pcchLength = cchMaxPrev - cchMax;
        } else {
            *pcchLength = 0;
        }
    }
    hr
}

unsafe fn StringCatWorkerA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    let mut cchDestLength: usize = 0;
    let mut hr = StringLengthWorkerA(pszDest, cchDest, &mut cchDestLength);
    if hr >= 0 {
        hr = StringCopyWorkerA(pszDest.add(cchDestLength), cchDest - cchDestLength, pszSrc);
    }
    hr
}

pub unsafe fn StringCchCopyA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        STRSAFE_E_INVALID_PARAMETER
    } else {
        StringCopyWorkerA(pszDest, cchDest, pszSrc)
    }
}

/// `sizeof(char)` is 1, so the Cb form counts the same units.
pub unsafe fn StringCbCopyA(pszDest: *mut c_char, cbDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cbDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerA(pszDest, cbDest, pszSrc)
}

pub unsafe fn StringCchCatA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCatWorkerA(pszDest, cchDest, pszSrc)
}

pub unsafe fn StringCchLengthA(psz: *const c_char, cchMax: usize, pcchLength: *mut usize) -> HRESULT {
    let hr = if psz.is_null() || cchMax > STRSAFE_MAX_CCH {
        STRSAFE_E_INVALID_PARAMETER
    } else {
        StringLengthWorkerA(psz, cchMax, pcchLength)
    };
    if hr < 0 && !pcchLength.is_null() {
        *pcchLength = 0;
    }
    hr
}

extern "C" {
    /// msvcrt `_snprintf` (`_vsnprintf` with the arguments in place of the `va_list`).
    pub fn _snprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}

/// strsafe.h `StringVPrintfWorkerA`'s tail, given `_snprintf`'s return for `cchDest - 1`.
pub unsafe fn StringVPrintfWorkerA_finish(pszDest: *mut c_char, cchDest: usize, iRet: c_int) -> HRESULT {
    let cchMax = cchDest - 1;
    if iRet < 0 || (iRet as usize) > cchMax {
        *pszDest.add(cchMax) = 0;
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        if (iRet as usize) == cchMax {
            *pszDest.add(cchMax) = 0;
        }
        0
    }
}

/// strsafe.h `StringCbPrintfA(pszDest, cbDest, pszFormat, ...)` as a macro call (Rust cannot
/// define the C variadic). Evaluates to the HRESULT.
#[macro_export]
macro_rules! StringCbPrintfA {
    ($pszDest:expr, $cbDest:expr, $pszFormat:expr $(, $arg:expr)* $(,)?) => {{
        let pszDest: *mut core::ffi::c_char = $pszDest as _;
        let cbDest: usize = $cbDest as usize;
        if cbDest > strsafe_rust::STRSAFE_MAX_CCH {
            strsafe_rust::STRSAFE_E_INVALID_PARAMETER
        } else if cbDest == 0 {
            strsafe_rust::STRSAFE_E_INVALID_PARAMETER
        } else {
            let iRet = $crate::shim::_snprintf(pszDest, cbDest - 1, $pszFormat as _ $(, $arg)*);
            $crate::shim::StringVPrintfWorkerA_finish(pszDest, cbDest, iRet)
        }
    }};
}
