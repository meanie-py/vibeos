#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! msgina.dll — Rust translation of dll/win32/msgina (the GINA logon-UI DLL).
//! staticlib → libmsgina_rust.a, cohab-linked into the cmake msgina.dll with the
//! translated .c commented out. Inbound Wlx* dispatch ABI from winlogon-sys; outbound
//! Win32 calls declared in msgina_externs and resolved at link via the cmake importlibs.
//! See docs/engineering/userland-translation-audit-part2.md.

// Crate-wide `w!("...")` — compile-time UTF-16, NUL-terminated wide string literal.
#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        const __B: &[u8] = $s.as_bytes();
        const __N: usize = __B.len();
        static __W: [u16; __N + 1] = {
            let mut out = [0u16; __N + 1];
            let mut i = 0;
            while i < __N {
                out[i] = __B[i] as u16;
                i += 1;
            }
            out
        };
        &__W
    }};
}

// Wine debug-channel logging macros (ERR!/WARN!/TRACE!/FIXME!). The post-processor
// rewrites DeepSeek's `ERR(...)` call-syntax to `ERR!(...)`; c2rust-anchored bodies
// instead emit the equivalent expansion inline. Both are gated by the per-file
// `__wine_dbch___default` channel flags, which default to `WINE_DBG_DEFAULT_FLAGS`
// (0 normally; 0xFF under the `wine_debug` cargo feature). When off this is a single
// flag-test; when on, it logs through libwine's `ros_dbg_log`. Defined before the
// mods so every translated module sees them. `__wine_dbch___default` resolves at the
// call site (each module's c2rust preamble defines it). Args forward verbatim to the
// variadic logger (the C printf format + values, already lowered by the translation).
macro_rules! __msgina_wine_log {
    ($cls:path, $($arg:tt)*) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let __ch = __wine_dbch___default;
            if (*__ch).flags as u32 & (1u32 << ($cls as u32)) != 0 {
                $crate::prelude::ros_dbg_log(
                    $cls,
                    __ch,
                    concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char,
                    b"\0".as_ptr() as *const core::ffi::c_char,
                    line!() as core::ffi::c_int,
                    $($arg)*
                );
            }
        }
    }};
}
macro_rules! ERR   { ($($a:tt)*) => { __msgina_wine_log!($crate::prelude::__WINE_DBCL_ERR,   $($a)*) }; }
macro_rules! WARN  { ($($a:tt)*) => { __msgina_wine_log!($crate::prelude::__WINE_DBCL_WARN,  $($a)*) }; }
macro_rules! TRACE { ($($a:tt)*) => { __msgina_wine_log!($crate::prelude::__WINE_DBCL_TRACE, $($a)*) }; }
macro_rules! FIXME { ($($a:tt)*) => { __msgina_wine_log!($crate::prelude::__WINE_DBCL_FIXME, $($a)*) }; }

// ---- Win32 helper macros (windef.h/winuser.h) the translation uses. Defined
// before the mods so every module sees them (macro_rules! resolves textually). ----
macro_rules! LOWORD { ($x:expr) => { (($x as usize) & 0xffff) as $crate::prelude::WORD } }
macro_rules! HIWORD { ($x:expr) => { ((($x as usize) >> 16) & 0xffff) as $crate::prelude::WORD } }
macro_rules! MAKEINTRESOURCEW { ($i:expr) => { (($i as u16) as usize) as $crate::prelude::PWSTR } }
// The translation carries the source's `MAKINTRESOURCEW` typo verbatim.
macro_rules! MAKINTRESOURCEW { ($i:expr) => { (($i as u16) as usize) as $crate::prelude::PWSTR } }
macro_rules! GET_X_LPARAM { ($lp:expr) => { (($lp as usize & 0xffff) as i16) as core::ffi::c_int } }
macro_rules! GET_Y_LPARAM { ($lp:expr) => { ((($lp as usize >> 16) & 0xffff) as i16) as core::ffi::c_int } }
macro_rules! PRIMARYLANGID { ($l:expr) => { (($l as usize) & 0x3ff) as $crate::prelude::WORD } }
macro_rules! UNREFERENCED_PARAMETER { ($x:expr) => { { let _ = &$x; } } }
// UNIMPLEMENTED() — DPRINT1-class no-op in this translation.
macro_rules! UNIMPLEMENTED { () => { {} } }
// A/W function-name macros (winuser.h/wingdi.h): the translation uses the generic
// names; map them to the W-suffixed link symbols in crate::msgina_externs.
macro_rules! SetWindowText { ($($a:expr),* $(,)?) => { $crate::msgina_externs::SetWindowTextW($($a),*) } }
macro_rules! GetObject { ($($a:expr),* $(,)?) => { $crate::msgina_externs::GetObjectW($($a),*) } }
macro_rules! GetVersionEx { ($($a:expr),* $(,)?) => { $crate::msgina_externs::GetVersionExW($($a),*) } }
// RtlCopyMemory(dst, src, len) → copy_nonoverlapping(src, dst, count).
macro_rules! RtlCopyMemory {
    ($d:expr, $s:expr, $n:expr $(,)?) => {
        core::ptr::copy_nonoverlapping($s as *const u8, $d as *mut u8, $n as usize)
    };
}
// powrprof.h IS_PWR_{HIBERNATE,SUSPEND}_ALLOWED(pcaps) — msgina does NOT link powrprof,
// so these are stubbed to 0/FALSE. TODO: this disables the hibernate/suspend shutdown
// options in the GINA shutdown dialog; revisit when powrprof is available.
macro_rules! IS_PWR_HIBERNATE_ALLOWED { ($caps:expr) => {{ let _ = &$caps; 0i32 }} }
macro_rules! IS_PWR_SUSPEND_ALLOWED { ($caps:expr) => {{ let _ = &$caps; 0i32 }} }

// strsafe.h's VARIADIC printf entry points.
//
// These were macros that DISCARDED the count argument and forwarded to user32's `wsprintfW`,
// which bounds to 1024 wchars internally regardless of what the caller's buffer is. That is not
// an approximation of strsafe, it is the opposite of it: `shutdown.rs`'s log-off string formats
// a resource-loaded format plus the user name into a `WCHAR[256]` on the stack, so a long user
// name wrote up to 768 wchars past the end of a stack array where the C truncated and returned
// STRSAFE_E_INSUFFICIENT_BUFFER. (The same site had a second bug the macro hid: `UserName` is a
// `[WCHAR; 256]` and was passed to the variadic BY VALUE, so `%ws` read the first pointer-width
// bytes of the name text as an address. C's array-to-pointer decay does not exist in Rust.)
//
// strsafe.h's printf entry points are VARIADIC, and stable Rust can CALL a C-variadic though
// it cannot define one; so these are strsafe-rust macros that call msvcrt's `_snwprintf` in
// place, with strsafe's bounds and HRESULTs. The C trampoline that used to define them
// (rust_varargs_shim.c) retired 2026-09-25. Call sites say `StringCchPrintfW!(...)`.
pub use strsafe_rust::{StringCbPrintfW, StringCchPrintfW};

// Foundation: cross-DLL Win32/LSA out-call decls, GINA-specific types, and the
// universal prelude the per-file translations expect in scope.
pub mod msgina_consts;
pub mod msgina_externs;
pub mod msgina_types;
pub mod prelude;
pub mod resources;
pub mod strsafe;

// ---- translated body modules (added per-file as each .c is flipped) ----
pub mod dimmedwindow;
pub mod gui;
pub mod lsa;
pub mod msgina;
pub mod shutdown;
pub mod stubs;
pub mod tui;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
