#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// i8042prt.sys — Rust translation of the PS/2 (i8042) port driver
// (drivers/input/i8042prt/*.c). crate-type=staticlib → libi8042prt_rust.a,
// linked into the cmake-built i8042prt.sys with the C sources commented out
// (kept on disk per never-delete rule). Standalone kernel-mode driver
// depending on ntkrnl-sys + dmilib; mirrors arm64input-rust cohabitation.
// Built for all 3 arches.
//
// One C file → one module (flat, rustconv per-file convention):
//   i8042prt.c createclose.c hwhacks.c keyboard.c misc.c mouse.c
//   pnp.c ps2pp.c readwrite.c registry.c
//
// WIP — the per-file rustconv (opus 4.8) translations live on disk in src/
// but are NOT yet integrated: they need the i8042prt.h private-header module
// (PORT_KEYBOARD_EXTENSION / PORT_MOUSE_EXTENSION / I8042_* etc.) plus the
// import-recipe pass that kbdclass+mouclass went through (~499 raw cargo
// errors at first translate). Modules are commented out so the workspace
// compiles; uncomment + fix file-by-file, same recipe as the class drivers.
// The .rs are kept (never delete a paid translation) — they are the starting
// point, not dead weight.
//
pub mod i8042prt_h;
pub(crate) use i8042prt_h::*;

pub mod i8042prt;
pub mod createclose;
pub mod dmi;
pub mod hwhacks;
pub mod keyboard;
pub mod misc;
pub mod mouse;
pub mod pnp;
pub mod ps2pp;
pub mod readwrite;
pub mod registry;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
