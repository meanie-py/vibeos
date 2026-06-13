//! Cargo-driven `usbd.sys` link (i386 / amd64 / arm64). The ReactOS USB
//! driver-support library (drivers/usb/usbd), fully translated to `usbd-rust`
//! (USB DDK types via usb-sys bindgen). Replaces the cmake
//! `add_library(usbd MODULE rust_cohab_stub.c …)` +
//! `set_module_type(usbd kernelmodedriver)` + `spec2def(usbd.sys usbd.spec)` +
//! `add_importlibs(usbd ntoskrnl hal)` cohab. Version resource dropped.
//!
//! native:5.01, -shared, --exclude-all-symbols + 0x1000 align; entry
//! _DriverEntry@8 / DriverEntry; base 0x10000 (x86) / 0x7FFB00000000 (arm64);
//! EXPORTS the USBD_* API via the spec2def-generated usbd.def (usbport etc.
//! import it); imports ntoskrnl + hal; reuses libmemcmp.a. usbd-rust builds
//! with the standard user target + i386 -sse,-sse2 (.rust_user_target()).
//! The cmake `spec2def(usbd.sys usbd.spec ADD_IMPORTLIB)` STAYS (the libusbd
//! import lib its consumers link is independent of this target).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-kernel.json", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-kernel.json", "DriverEntry", 0x7FFB_0000_0000),
        o => panic!("usbd-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    let mut linker = Linker::new(target_json)
        .rust_staticlib("usbd-rust")
        .rust_user_target();
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker
        .spec2def("drivers/usb/usbd/usbd.spec", "usbd.sys")
        // memcmp comes from compiler-builtins-mem (build-std); libmemcmp.a dropped.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
