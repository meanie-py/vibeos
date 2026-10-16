//! Cargo-driven `classpnp.sys` link (i386 / amd64 / arm64). The ReactOS
//! storage class library (drivers/storage/class/classpnp) — the largest driver
//! after win32k, fully translated to `classpnp-rust` (~166K lines) and loaded
//! at every boot (disk.sys / cdrom.sys are class drivers that import its
//! Class* API). Replaces the cmake `add_library(classpnp MODULE
//! rust_cohab_stub.c)` + `set_module_type(classpnp kernelmodedriver)` +
//! `spec2def(classpnp.sys classpnp.spec)` + `add_importlibs(classpnp ntoskrnl
//! hal)` + `target_link_libraries(… ntoskrnl_vista libcntpr pseh)` cohab.
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
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        o => panic!("classpnp-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    // cnt-rust-lib feature set: amd64/arm64 enable `float` (the forbid-unsafe
    // floatcvt/scanf cabi surface); i386 omits it (no-SSE-illegal by-value f64
    // returns stay C). Mirrors libcntpr.cmake's CNT_RUST_FEATURES + ntdll-cargo.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    // ZERO cmake-built .a — classpnp links a fully cargo-native floor. Each
    // former reuse is gone: liblibcntpr.a (step 8, cnt-rust-lib + the Wave
    // MATH-i386 no-SSE softfloat math cover it), libchkstk.a (clang_rt.builtins
    // provides __chkstk on all 3 arches — cold-link confirms nothing else
    // referenced it), ntoskrnl_vista (→ ntoskrnl-vista-rust sibling below),
    // pseh (→ dummy.c c_shim), cntpr_rust (→ cnt-rust-lib), memcmp
    // (compiler-builtins-mem). The cold-link is the oracle (clean ×3).
    let static_libs: Vec<&str> = vec![];

    let mut linker = Linker::new(target_json)
        .rust_staticlib("classpnp-rust")
        .rust_user_target()
        // cnt-rust-lib (libcntpr's Rust half) built as a cargo-native sibling
        // staticlib instead of reusing cmake's libcntpr_rust.a — exactly the
        // ntdll-cargo pattern. It is now the SOLE libcntpr provider for classpnp
        // (liblibcntpr.a dropped, step 8).
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // ntoskrnl_vista's classpnp-referenced subset (io/ke/po), translated to
        // Rust — replaces the cmake-built libntoskrnl_vista.a for classpnp (the
        // kernels keep the C lib). Matches the C .a's per-arch decoration.
        .rust_lib("ntoskrnl-vista-rust", &[], true)
        // rust_cohab_stub.c provides the REVERSE_BYTES{,_SHORT,_QUAD} byte-swap
        // helpers + the _Analysis_assume_ SAL no-op that classpnp-rust calls as
        // C functions (header macros rustc can't emit as #[no_mangle] bodies).
        // dummy.c provides the two _SEH2_Volatile* globals the C floor references
        // under USE_DUMMY_PSEH (replaces libpseh.a — same as ntdll-cargo). Both
        // header-free TUs → compiled in-cargo via c_shims. memcmp comes from
        // compiler-builtins-mem (build-std), so libmemcmp.a is dropped too.
        .c_shims(&[
            "drivers/storage/class/classpnp/rust_cohab_stub.c",
            "sdk/lib/pseh/dummy.c",
        ]);
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker
        .spec2def("drivers/storage/class/classpnp/classpnp.spec", "classpnp.sys")
        .static_libs(&static_libs)
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .resource_from_rc("drivers/storage/class/classpnp/class.rc", "classpnp_res")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
