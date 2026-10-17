# ============================================================================
# framebuf.dll — cargo-driven final link (sdk/lib/rust/framebuf-cargo via reactos-link)
# ============================================================================
# enable.c/palette.c/pointer.c/screen.c/surface.c (+ dd/ddenable) are translated
# to Rust (sdk/lib/rust/framebuf-rust, kept on disk per the never-delete rule);
# the Rust #[no_mangle] DrvEnableDriver is the PE entry point. The final
# framebuf.dll link is no longer cmake's add_library(MODULE) + rust_cohab_stub.c
# + the per-arch custom_command — it is driven by cargo (framebuf-cargo's
# build.rs → reactos-link, ModuleKind::KernelDll, entry=DrvEnableDriver@12, no
# exports — win32k's GDI loader calls the PE entry point).
#
# framebuf is the GDI display driver for every UEFI GOP/linear-framebuffer boot
# on all three arches. add_cargo_module builds framebuf-rust + cnt-rust-lib
# (cargo-native, the sole libcntpr provider) + the win32k import lib (the
# Eng*/PALOBJ_*/DDI surface) into the shipping PE, applies the kernel-dll
# post-link and registers it on the boot media. ZERO cmake .a: liblibcntpr.a +
# libcntpr_rust.a both dropped (cold-link needs no C residue). Unlike the
# kernelmodedrivers it imports win32k, NOT ntoskrnl/hal.
# The version resource: cmake compiles it in this OBJECT library, whose windres
# command the cargo link replays.
add_cargo_module(framebuf framebuf-cargo framebuf.dll KIND kerneldll
    BODY_CRATE framebuf-rust DESTINATION reactos/system32
    EXTRA_DEPENDS psdk)
