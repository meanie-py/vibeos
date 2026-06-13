//! Cargo-driven `classpnp.sys` link (i386 / amd64 / arm64). The ReactOS
//! storage class library (drivers/storage/class/classpnp) — the largest driver
//! after win32k, fully translated to `classpnp-rust` (~166K lines) and loaded
//! at every boot (disk.sys / cdrom.sys are class drivers that import its
//! Class* API). Replaces the cmake `add_library(classpnp MODULE
//! rust_cohab_stub.c)` + `set_module_type(classpnp kernelmodedriver)` +
//! `spec2def(classpnp.sys classpnp.spec)` + `add_importlibs(classpnp ntoskrnl
//! hal)` + `target_link_libraries(… ntoskrnl_vista libcntpr pseh)` cohab.
//! Version resource dropped (as in every driver cargo cut).
//!
//! native:5.01, -shared, --exclude-all-symbols + 0x1000 align; entry
//! _DriverEntry@8 / DriverEntry; base 0x10000 (x86) / 0x7FFB00000000 (arm64);
//! EXPORTS the Class* API via the spec2def-generated classpnp.def; imports
//! ntoskrnl + hal; reuses the C floor ntoskrnl_vista / libcntpr / pseh /
//! chkstk (i386+arm64 only — amd64 takes __chkstk from clang_rt.builtins) /
//! cntpr_rust / memcmp, C-first in the link group. classpnp-rust builds with
//! the standard user target + i386 -sse,-sse2 (.rust_user_target()). The cmake
//! `spec2def(classpnp.sys classpnp.spec ADD_IMPORTLIB)` STAYS (consumers link
//! the libclasspnp import lib independently).
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses cmake's
//! ntoskrnl_vista/libcntpr/pseh/chkstk/cntpr_rust/memcmp archives + spec2def).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base, chkstk): (&str, &str, u64, bool) =
        match arch.as_str() {
            "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000, true),
            "x86_64" => ("x86_64-kernel.json", "DriverEntry", 0x0001_0000, false),
            "aarch64" => ("aarch64-kernel.json", "DriverEntry", 0x7FFB_0000_0000, true),
            o => panic!("classpnp-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
        };

    // C floor reused from cmake, in the cmake link-line order (C-first; the
    // engine emits libclasspnp_rust.a after this group).
    let mut static_libs: Vec<&str> = vec![
        "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista.a",
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/pseh/libpseh.a",
    ];
    if chkstk {
        static_libs.push("sdk/lib/crt/libchkstk.a");
    }
    static_libs.push("sdk/lib/crt/libcntpr_rust.a");
    static_libs.push("sdk/lib/crt/libmemcmp.a");

    let mut linker = Linker::new(target_json)
        .rust_staticlib("classpnp-rust")
        .rust_user_target()
        // rust_cohab_stub.c provides the REVERSE_BYTES{,_SHORT,_QUAD} byte-swap
        // helpers + the _Analysis_assume_ SAL no-op that classpnp-rust calls as
        // C functions (header macros rustc can't emit as #[no_mangle] bodies).
        // Header-free TU → compiled in-cargo via c_shims (was a source of the
        // cmake classpnp MODULE; the cargo cut drops the MODULE so compile it
        // here). Per-arch cdecl decoration matches (i386 _REVERSE_BYTES /
        // __Analysis_assume_).
        .c_shims(&["drivers/storage/class/classpnp/rust_cohab_stub.c"]);
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker
        .spec2def("drivers/storage/class/classpnp/classpnp.spec", "classpnp.sys")
        .static_libs(&static_libs)
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
