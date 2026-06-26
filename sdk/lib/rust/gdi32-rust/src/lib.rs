#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(static_mut_refs)]
#![allow(clippy::all)]

//! gdi32.dll — Rust translation of win32ss/gdi/gdi32 (the usermode GDI client API DLL).
//! staticlib → libgdi32_rust.a, cohab-linked into the cmake gdi32.dll with the translated
//! .c commented out. The usermode *client* half of the GDI graphics system; the kernel
//! half (win32k.sys GDI) is already 100% Rust. Most exports marshal args and call into
//! win32k via NtGdi* syscalls (the win32ksys importlib) or queue a GdiAllocBatchCommand.
//! See docs/engineering/gdi32-translation-audit.md for the per-wave plan.

// Foundation: minimal Win32 type aliases (grows as waves land).
pub mod types;

// Wave 1a substrate: the shared gdi32p.h FORCEINLINE helpers (GdiAllocBatchCommand,
// GdiGetDcAttr, GdiGetRgnAttr) + the GDI_HANDLE_* macros + NtCurrentTeb. Called by
// misc/objects bodies; not exported.
pub mod gdihelp;

// main/ — DLL entry. Wave 1a: dllmain (DllMain/GdiProcessSetup, gbLpk global).
pub mod main;

// misc/ — client substrate. Wave 1a: misc (handle-table primitives + GdiHandleTable/
// CurrentProcessId/GDI_BatchLimit globals, escapes, batch-limit, language pack).
pub mod misc;

// objects/ — GDI object & drawing bodies. Wave 0: linedda. Wave 1a: gdiobj.
pub mod objects;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
