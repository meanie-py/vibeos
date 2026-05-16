//! Cargo-driven `bootvid.dll` link (i386). Manifest for the shared
//! `reactos-link` engine. Boot-video sibling of `hal-cargo` /
//! `kdcom-cargo` build.rs.
//!
//! Mirrors the cmake `bootvid.dll` link line (build-i386/build.ninja,
//! target `drivers/base/bootvid/bootvid.dll`):
//!   entry  `-Wl,-e,___ImageBase`  (ENTRYPOINT 0 sentinel)
//!   base   `--image-base,0x00010000`
//!   shared kerneldll → `-shared`, no `--gc-sections` (keeps all 11
//!          `bootvid.spec` `Vid*` exports without an anchor symbol)
//!   libs   ntoskrnl + hal import libs (self-generated from their
//!          specs; cmake `add_importlibs(bootvid ntoskrnl hal)` — the
//!          VGA path imports MmMapIoSpace/READ_REGISTER_*/DbgPrint from
//!          ntoskrnl and READ/WRITE_PORT_* from hal) + memcmp leaf
//!          static lib (cmake LINK_LIBRARIES).
//!
//! Run `ninja` in build-i386/ first (reactos-link consumes the
//! cmake-built leaf static lib + spec2def host tool from there).

use reactos_link::{Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        .rust_staticlib("bootvid-rust")
        .spec2def("drivers/base/bootvid/bootvid.spec", "bootvid.dll")
        .static_libs(&["sdk/lib/crt/libmemcmp.a"])
        // Chainable multi-import (reactos-link Vec<import_lib>): exactly
        // cmake's `add_importlibs(bootvid ntoskrnl hal)`.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            // ENTRYPOINT 0: cmake passes `-Wl,-e,___ImageBase`; no
            // DllMain (ntoskrnl resolves the Vid* exports by name).
            entry: "___ImageBase".into(),
            image_base: 0x0001_0000,
        });
}
