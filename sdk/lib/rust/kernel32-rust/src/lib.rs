#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
// Translated C aliases mutable file-scope globals freely (PEB params, AppCert
// list, CSR messages); &mut/& to `static mut` is intrinsic to the cohab and safe
// in this single-threaded init context.
#![allow(clippy::all)]

//! kernel32.dll — Rust translation of dll/win32/kernel32 (the Win32 base API DLL).
//! staticlib → libkernel32_rust.a, cohab-linked into the cmake kernel32.dll with the
//! translated .c commented out. Imports resolve to ntdll.dll at runtime; FFI decls from
//! ntdll-sys. See docs/engineering/ — kernel32 translation campaign (proc.c first).

// Fold-proof Wine TRACE/FIXME/WARN/ERR shims (§9.99) — see winetrace-shim.
extern crate winetrace_shim;

// Crate-wide `w!("...")` — compile-time UTF-16, NUL-terminated wide string literal
// (Wine/DDK idiom). All call sites pass ASCII; `.as_ptr()` yields the LPCWSTR.
// Defined at crate root (before the mods) so every descendant module sees bare `w!`.
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

// Crate-wide const UTF-16 builder for `static FOO: [u16; N] = to_w("...");` sites.
pub const fn to_w<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

// Foundation: k32-internal types/constants, the cohab import surface (Base*/
// Win32/SharedUserData/macros), proc.c's globals, and the aggregated substrate.
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod k32_externs;
pub mod k32_globals;
pub mod k32_inlines; // #[no_mangle] RTL FORCEINLINE bodies; NOT globbed by substrate
pub mod k32_types;
pub mod prelude;
pub mod substrate;

// Interlocked* intrinsics kernel32 owns as real symbols (i386 32-bit family +
// the pointer ExchangePointer wrapper). See module doc.
pub mod interlocked;

// ntstrsafe.h FORCEINLINE string routines (no export) referenced by the C
// translation as extern symbols → faithful Rust bodies. See module doc.
pub mod ntstrsafe;

// mountmgr.h MOUNTMGR_IS_*_VOLUME_NAME macros referenced as extern fns → real defs.
pub mod mountmgr;

// client/proc.c — CreateProcessInternalW + the process-creation path (first flip).
// Body modules land per the parallel hand-translation; each `use crate::substrate::*`.
pub mod proc_getters; // GetProcess*/Set*/Open/Exit/Terminate/Fatal/ReadWriteMem
pub mod proc_helpers; // StuffStdHandle/AppCert/Sxs/BaseProcessStartup/InitCommandLines
pub mod proc_params; // BasePushProcessParameters
pub mod proc_create; // CreateProcessInternalW + CreateProcessW/A + WinExec

// winnls/string/ — NLS/locale/string family (rustconv Sonnet+c2rust). locale.c first.
pub mod string;

// wine/ — Wine-derived helpers (muldiv, ...).
pub mod wine;

// client/ — the client-side Win32 API surface (version/handle/atom/environ/path/...).
pub mod client;

// client/file/ — the file-I/O API family.
pub mod file;

// client/console/ — console client APIs, ported to Rust (rustconv c2rust) via the
// winsrv-sys FFI bridge (console/cffi.rs re-exports the CSR CONSOLE_API_MESSAGE family).
pub mod console;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
