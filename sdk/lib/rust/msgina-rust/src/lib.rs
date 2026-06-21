#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(static_mut_refs)]
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

// Foundation: cross-DLL Win32/LSA out-call decls, GINA-specific types, and the
// universal prelude the per-file translations expect in scope.
pub mod msgina_externs;
pub mod msgina_types;
pub mod prelude;
pub mod resources;

// ---- translated body modules (added per-file as each .c is flipped) ----
pub mod lsa;
pub mod stubs;
pub mod tui;
pub mod utils;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
