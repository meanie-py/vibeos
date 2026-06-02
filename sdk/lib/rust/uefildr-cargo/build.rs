//! Cargo-driven `uefildr.efi` link (i386). Manifest for the shared
//! `reactos-link` engine. UEFI sibling of `kdcom-cargo`'s build.rs.
//!
//! Mirrors the cmake `uefildr.efi` link line (from build-i386/build.ninja,
//! target `boot/freeldr/freeldr/uefildr.efi`):
//!   entry      `-Wl,-entry,_EfiEntry`        (set_entrypoint EfiEntry)
//!   subsystem  `efi_application:5.01`         (set_subsystem 10)
//!   align      file/section 0x200             (UEFI app)
//!   gc         `--gc-sections`, NOT -shared   (it's an exe)
//!   base       auto (`--enable-auto-image-base`, no explicit base on i386)
//!   exports    freeldr.spec → uefildr.def     (rosload imports these at
//!              runtime against uefildr.efi, registered as "freeldr.sys")
//!
//! Inputs are the cmake-built artifacts (run `ninja uefildr` in build-i386/
//! first). The big freeldr C aggregate `libuefifreeldr_common.a`, the
//! prebuilt `strsafe_shim.c.obj` (the variadic RtlStringCb* shim, which
//! Rust stable can't host), and `libfreeldr_rust_uefi.a` (the freeldr-rust
//! body + the `_EfiEntry` entry, built by cmake `--features
//! uefi,uefildr-rust-entry`) plus the leaf libs, in the build.ninja order.
//! reactos-link wraps them in --start-group/--end-group, so the cmake
//! link's duplicate-listing trick to satisfy circular refs is unneeded.

use reactos_link::{Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        // No .rust_staticlib(): reuse the cmake-built libfreeldr_rust_uefi.a
        // (identical codegen + carries the uefi/uefildr-rust-entry features).
        //
        // NO .spec2def(): cmake's uefildr.efi link (rules.ninja
        // C_EXECUTABLE_LINKER__uefildr) does NOT pass uefildr.def to the
        // linker — the EFI app links `--exclude-all-symbols` and exports
        // nothing. The .def/freeldr.spec is only for the freeldr import
        // library that rosload consumes, not for uefildr's own exports.
        // Passing it here would force every freeldr.spec export as a
        // required symbol, and the UEFI build genuinely lacks a few
        // (`PxeCallApi`, `DriveMapMapDrivesInSection` — gated out of the
        // Rust uefi feature, defined only in the C BIOS libfreeldr.a),
        // producing `<root>: undefined symbol`. Matching cmake = no .def.
        .static_libs(&[
            // strsafe_shim.c.obj — the C variadic shim, compiled into the
            // cmake uefildr.dir (header-dependent, so reused prebuilt rather
            // than recompiled as a header-free c_shim).
            "boot/freeldr/freeldr/CMakeFiles/uefildr.dir/lib/strsafe_shim.c.obj",
            "hal/halx86/minihal/libmini_hal.a",
            "hal/halx86/libhal_rust_mini.a",
            "boot/freeldr/freeldr/libuefifreeldr_common.a",
            "boot/freeldr/freeldr/libfreeldr_rust_uefi.a",
            "sdk/lib/cportlib/libcportlib.a",
            "sdk/lib/cmlib/libblcmlib.a",
            "boot/rtl/libblrtl.a",
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/crt/libchkstk.a",
            "sdk/lib/crt/libcntpr_rust.a",
        ])
        .link(ModuleKind::EfiApplication {
            // i386 cdecl decoration: EfiEntry → _EfiEntry. Provided by
            // libfreeldr_rust_uefi.a (freeldr-rust/src/uefildr.rs,
            // #[no_mangle] under --features uefildr-rust-entry).
            entry: "_EfiEntry".into(),
        });
}
