#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! gdi32.dll — Rust translation of win32ss/gdi/gdi32 (the usermode GDI client API DLL).
//! staticlib → libgdi32_rust.a, cohab-linked into the cmake gdi32.dll with the translated
//! .c commented out. The usermode *client* half of the GDI graphics system; the kernel
//! half (win32k.sys GDI) is already 100% Rust. Most exports marshal args and call into
//! win32k via NtGdi* syscalls (the win32ksys importlib) or queue a GdiAllocBatchCommand.
//! See docs/engineering/gdi32-translation-audit.md for the per-wave plan.

// Foundation: minimal Win32 type aliases (grows as waves land).
// Emit a fn twice: `extern "fastcall"` on i386, `extern "C"` elsewhere.
// FASTCALL only exists as a distinct ABI on x86; rustc treats "fastcall" as "C"
// on other arches today and makes it a HARD ERROR on newer toolchains
// (unsupported_calling_conventions). Same pattern as ex-rust's fastcall_export!.
macro_rules! fastcall_fn {
    ($(#[$attr:meta])* $vis:vis unsafe fn $name:ident($($params:tt)*) $(-> $ret:ty)? $body:block) => {
        $(#[$attr])*
        #[cfg(target_arch = "x86")]
        $vis unsafe extern "fastcall" fn $name($($params)*) $(-> $ret)? $body

        $(#[$attr])*
        #[cfg(not(target_arch = "x86"))]
        $vis unsafe extern "C" fn $name($($params)*) $(-> $ret)? $body
    };
}

pub mod types;

// Wave 1a substrate: the shared gdi32p.h FORCEINLINE helpers (GdiAllocBatchCommand,
// GdiGetDcAttr, GdiGetRgnAttr) + the GDI_HANDLE_* macros + NtCurrentTeb. Called by
// misc/objects bodies; not exported.
pub mod gdihelp;

// WoW64 DC_ATTR layout shim (M7): the 64-bit kernel's DC_ATTR mirrored on the
// 32-bit compiler, plus the dcattr_f! field-projection macro below.
pub mod dcattr64;

/// Project a field of a shared DC_ATTR as a correctly-typed `*mut` pointer,
/// honoring the WoW64 wide (64-bit) layout when `g_GdiWow64` is set. Works
/// for reads, writes, compound assigns, and sub-field access alike:
/// `(*dcattr_f!(pdcattr, ulDirty_)) |= DC_BRUSH_DIRTY;`
/// Wide pointer/handle slots project their meaningful low half (see
/// dcattr64.rs for the zero-upper-half invariant).
#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! dcattr_f {
    ($p:expr, $f:ident) => {{
        let __pdcattr = $p;
        if $crate::misc::misc::g_GdiWow64 != 0 {
            core::ptr::addr_of_mut!(
                (*(__pdcattr as *mut $crate::dcattr64::DC_ATTR64)).$f
            )
        } else {
            core::ptr::addr_of_mut!((*__pdcattr).$f)
        }
    }};
}
#[cfg(not(target_arch = "x86"))]
#[macro_export]
macro_rules! dcattr_f {
    ($p:expr, $f:ident) => {
        core::ptr::addr_of_mut!((*$p).$f)
    };
}

// main/ — DLL entry. Wave 1a: dllmain (DllMain/GdiProcessSetup, gbLpk global).
pub mod main;

// misc/ — client substrate. Wave 1a: misc (handle-table primitives + GdiHandleTable/
// CurrentProcessId/GDI_BatchLimit globals, escapes, batch-limit, language pack).
pub mod misc;

// objects/ — GDI object & drawing bodies. Wave 0: linedda. Wave 1a: gdiobj.
pub mod objects;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
