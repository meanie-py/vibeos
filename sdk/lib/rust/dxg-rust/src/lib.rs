//! dxg-rust — Rust translation of win32ss/reactx/dxg (dxg.sys), the legacy
//! DirectDraw/Direct3D kernel-mode thunk layer. Cohab staticlib → libdxg_rust.a,
//! linked into the cmake dxg.sys MODULE (uniata pattern). The translated C files
//! are commented out in win32ss/reactx/dxg/CMakeLists.txt and the `#[no_mangle]`
//! Rust symbols take over.
//!
//! FFI: `win32k_sys::win32k::*` (DirectDraw layouts / Eng* / win32k internals)
//! + `ntkrnl_sys::ddk::*` (Ex/Ke/Mm/Ob kernel APIs). dxg-private types
//! (DXENG_FUNCTIONS table, DD handle-manager structs, ObjType_* tags) live in
//! `types` — they are NOT in win32k-sys.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens, unused_mut, unused_variables, unused_assignments, unused_unsafe)]

extern crate ntkrnl_sys;

// dxg-private types/constants (from dxg_int.h). Flat re-export so the translated
// modules see DXENG_FUNCTIONS / DD_ENTRY / ObjType_* / DxEGShDevData_* directly.
pub mod types;
pub use types::*;

// Pool tags (win32ss/reactx/dxg/tags.h).
pub mod tags;
pub use tags::*;

// Translated modules (rustconv per-file). Flat layout, one `pub mod` per .c.
pub mod driver_main;
pub mod ddhmg;
pub mod d3d;
pub mod dd;
pub mod ddraw;
pub mod ddsurf;
pub mod dxgallocator;
pub mod eng;
pub mod historic;

// dxg_driver.h — the 75 DXG dispatch-table stubs + the gaDxgFuncs[] DRVFN table.
pub mod dxg_driver;

// DirectDraw driver-info GUID data (was DEFINE_GUID in main.c via initguid.h);
// #[no_mangle] statics that satisfy win32k-sys's extern GUID_* declarations.
mod guids;

// ============================================================================
// Panic handler — dxg.sys is its own kernel-mode module (separate from
// ntoskrnl.exe), so the Rust body needs its own panic handler + eh_personality,
// exactly like hal-rust. Only compiled when dxg-rust is the panic-owning crate
// (i.e. the staticlib build); a `no_panic_handler` feature lets a future
// umbrella opt out if it ever links dxg-rust alongside another handler.
// ============================================================================
#[cfg(not(feature = "no_panic_handler"))]
#[panic_handler]
fn dxg_panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(feature = "no_panic_handler"))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
