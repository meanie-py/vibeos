//! Cargo-driven `framebuf.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. The GDI framebuffer display driver, loaded by
//! win32k on every UEFI GOP/linear-framebuffer boot (all three arches).
//!
//! Mirrors the cmake `framebuf.dll` link
//! (set_module_type framebuf kerneldll ENTRYPOINT DrvEnableDriver 12):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align, no `--gc-sections`.
//!   entry  `DrvEnableDriver@12` — i386 `_DrvEnableDriver@12`; amd64/arm64
//!          `DrvEnableDriver` (win32k's GDI loader calls the PE entry point;
//!          the driver exports nothing).
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!   libs   win32k import lib (cmake `add_importlibs(framebuf win32k)`; the
//!          Eng*/PALOBJ_*/DDI surface comes from win32k.sys) + libcntpr
//!          (cmake `target_link_libraries(framebuf … libcntpr)`: the C/asm
//!          cntpr `liblibcntpr.a` + its Rust half `libcntpr_rust.a`).
//!
//! Unlike the kernelmodedrivers, framebuf does NOT import ntoskrnl/hal — only
//! win32k (which itself imports ntoskrnl).
//! Cohab body built with the user
//! target -sse,-sse2; the kernel softfloat json here is correct for a
//! kernel-mode display driver.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses cmake's
//! liblibcntpr.a/libcntpr_rust.a + the spec2def host tool + win32k.spec).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DrvEnableDriver@12", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DrvEnableDriver", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DrvEnableDriver", 0x7FFB_0000_0000),
        other => panic!("framebuf-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    // cnt-rust-lib `float` feature: i386 omits it (no-SSE softfloat kernel),
    // amd64/arm64 enable it — same gating as ntoskrnl-cargo/classpnp.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        .rust_staticlib("framebuf-rust")
        // ZERO cmake .a: both cmake cntpr .a dropped — cnt-rust-lib (cargo-native
        // sibling) is the sole libcntpr provider, like classpnp/ntoskrnl-cargo.
        // cold-link confirms framebuf needs no C residue (no c_from_compile_db).
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        .import_lib("win32ss/win32k.spec", "win32k.sys", "win32k")
        .resource_from_rc("win32ss/drivers/displays/framebuf/framebuf.rc", "framebuf_res")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
