//! sysfb-rust — Rust translation of the ReactOS standalone framebuffer
//! video device driver (`drivers/video/sysfb/sysfb.c`).
//!
//! Compiled as `crate-type = ["staticlib"]` → `libsysfb_rust.a`,
//! linked into the cmake-built kernel-mode PE driver `sysfb.sys`
//! with `sysfb.c` commented out in
//! `drivers/video/sysfb/CMakeLists.txt` — hal-rust / arm64input-rust
//! style cohabitation (not a cargo-driven DLL link).
//!
//! The driver reads framebuffer geometry from the ARC config tree that
//! FreeLDR populates from the UEFI GOP (`\Registry\Machine\HARDWARE\
//! DESCRIPTION\System\MultifunctionAdapter\N\DisplayController\M`,
//! "Configuration Data" = CM_FULL_RESOURCE_DESCRIPTOR whose Memory
//! descriptor gives the framebuffer phys base/size and whose
//! DeviceSpecific descriptor carries a CM_FRAMEBUF_DEVICE_DATA), then
//! creates `\Device\Video0`, writes the `HARDWARE\DEVICEMAP\VIDEO` and
//! `Services\sysfb\Device0` registry entries win32k's
//! EngpUpdateGraphicsDeviceList reads, and services the video IOCTLs
//! that framebuf.dll issues (mode query, MapVideoMemory, etc.).
//!
//! Standalone driver — no videoprt.sys dependency. Built for all three
//! arches (it is the live UEFI framebuffer driver on i386/amd64/arm64),
//! so unlike arm64input-rust this crate is NOT `#[cfg(aarch64)]`-gated.
//!
//! `#[no_mangle] pub unsafe extern "system" fn DriverEntry` is the single
//! export the loader needs; all other functions are internal.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

pub mod sysfb;

// Standalone `#![no_std]` staticlib (cargo builds `-p sysfb-rust`
// directly from the cmake custom_command), so it needs its own panic
// handler — halt loop, mirroring kdcom-rust / arm64input-rust.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// `core` / `compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`. Panic = abort never unwinds, but the symbol
// must resolve. Same stub as kdcom-rust / arm64input-rust / hal-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
