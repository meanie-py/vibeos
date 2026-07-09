//! arm64input-rust — Rust translation of the ReactOS ARM64 virtio-MMIO
//! input driver (`drivers/input/arm64input/arm64input.c`).
//!
//! Compiled as `crate-type = ["staticlib"]` → `libarm64input_rust.a`,
//! linked into the cmake-built kernel-mode PE driver `arm64input.sys`
//! with `arm64input.c` commented out in
//! `drivers/input/arm64input/CMakeLists.txt` — hal-rust-style
//! cohabitation (not a cargo-driven DLL link).
//!
//! ARM64 has no i8042/PS-2 and this port does no ACPI/PnP bus
//! enumeration there, so this driver is deliberately self-contained:
//!   * hardcoded QEMU `virt` virtio-MMIO window @ 0x0A000000, scanned
//!     for the virtio-input device (device-id 18);
//!   * a polling system thread draining the virtio used-ring (no
//!     `IoConnectInterrupt` — the ARM64 GIC backend is unimplemented,
//!     so polling is intentional);
//!   * `DriverEntry` directly creates `\Device\KeyboardClass0` and
//!     `\Device\PointerClass0` (+ DOS symlinks), impersonating the
//!     kbdclass/mouclass endpoints win32k opens.
//!
//! `#[no_mangle] pub unsafe extern "system" fn DriverEntry` is the
//! single export the loader needs; all other functions are internal.
//!
//! One C file → one module (rustconv writes it flat per the
//! `arm64input` Target's `src_subdir = src`):
//!   arm64input.c → arm64input

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

#[cfg(target_arch = "aarch64")]
pub mod arm64input;

// Standalone `#![no_std]` staticlib (cargo builds `-p arm64input-rust`
// directly from the cmake custom_command), so it needs its own panic
// handler — halt loop, mirroring kdcom-rust / freeldr-rust. arm64-only
// crate; the aarch64 `wfe` branch is the live one.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// `core` / `compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`. Panic = abort never unwinds, but the symbol
// must resolve. Same stub as kdcom-rust / freeldr-rust / hal-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
