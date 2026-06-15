#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
// Translated C aliases mutable file-scope globals freely (PEB params, AppCert
// list, CSR messages); &mut/& to `static mut` is intrinsic to the cohab and safe
// in this single-threaded init context.
#![allow(static_mut_refs)]
#![allow(clippy::all)]

//! kernel32.dll — Rust translation of dll/win32/kernel32 (the Win32 base API DLL).
//! staticlib → libkernel32_rust.a, cohab-linked into the cmake kernel32.dll with the
//! translated .c commented out. Imports resolve to ntdll.dll at runtime; FFI decls from
//! ntdll-sys. See docs/engineering/ — kernel32 translation campaign (proc.c first).

// Foundation: k32-internal types/constants, the cohab import surface (Base*/
// Win32/SharedUserData/macros), proc.c's globals, and the aggregated substrate.
pub mod k32_externs;
pub mod k32_globals;
pub mod k32_inlines; // #[no_mangle] RTL FORCEINLINE bodies; NOT globbed by substrate
pub mod k32_types;
pub mod substrate;

// client/proc.c — CreateProcessInternalW + the process-creation path (first flip).
// Body modules land per the parallel hand-translation; each `use crate::substrate::*`.
pub mod proc_getters; // GetProcess*/Set*/Open/Exit/Terminate/Fatal/ReadWriteMem
pub mod proc_helpers; // StuffStdHandle/AppCert/Sxs/BaseProcessStartup/InitCommandLines
pub mod proc_params; // BasePushProcessParameters
pub mod proc_create; // CreateProcessInternalW + CreateProcessW/A + WinExec

// winnls/string/ — NLS/locale/string family (rustconv Sonnet+c2rust). locale.c first.
pub mod string;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
