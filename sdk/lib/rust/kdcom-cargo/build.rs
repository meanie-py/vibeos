//! Cargo-driven `kdcom.dll` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there. KD-COM sibling
//! of `hal-cargo`'s `build.rs`.
//!
//! Mirrors the cmake `kdcom.dll` link line (from build-i386/build.ninja,
//! target `drivers/base/kdrosdbg/kdcom.dll`):
//!   entry  `-Wl,-e,___ImageBase`  (ENTRYPOINT 0 sentinel)
//!   base   `--image-base,0x00010000`
//!   shared kerneldll → `-shared`, no `--gc-sections` (keeps all 8
//!          `kdrosdbg.spec` exports without an anchor symbol)
//!   libs   ntoskrnl import lib (self-generated from ntoskrnl.spec,
//!          hal-cargo precedent — only `DbgPrint` is referenced) +
//!          cportlib + memcmp leaf static libs (cmake LINK_LIBRARIES;
//!          unreferenced by the stub but mirrored for parity).
//!
//! The C `kdrosdbg` target also imports `hal`, but the stub references
//! zero hal symbols, so no hal import lib is needed (and `import_lib`
//! is single-valued today — ntoskrnl is the only one that matters).
//!
//! Run `ninja` in build-i386/ first (reactos-link consumes cmake-built
//! leaf static libs from there).

use reactos_link::{Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        .rust_staticlib("kdcom-rust")
        .spec2def("drivers/base/kdrosdbg/kdrosdbg.spec", "kdcom.dll")
        .static_libs(&[
            "sdk/lib/cportlib/libcportlib.a",
            "sdk/lib/crt/libmemcmp.a",
        ])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .link(ModuleKind::KernelDll {
            // ENTRYPOINT 0: cmake passes `-Wl,-e,___ImageBase`; the DLL
            // has no DllMain (ntoskrnl resolves the 8 spec exports by
            // name). `___ImageBase` is the lld-defined image-base symbol
            // (C `_` prefix → triple underscore on i686 GNU).
            entry: "___ImageBase".into(),
            image_base: 0x0001_0000,
        });
}
