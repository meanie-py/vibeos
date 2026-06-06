# ============================================================================
# arm64vid.sys — cargo-driven final link (sdk/lib/rust/arm64vid-cargo via reactos-link)
# ============================================================================
# Despite the name, arm64vid is the LIVE GOP/linear-framebuffer video driver on
# all three UEFI arches (it renders the desktop). arm64vid.c is fully translated
# to Rust (sdk/lib/rust/arm64vid-rust, kept on disk per the never-delete rule);
# its DriverEntry is the PE entry point. The final arm64vid.sys link is no longer
# cmake's add_library(MODULE) + the rust_cohab_stub.c + per-arch custom_command
# scaffolding — it is driven by cargo (arm64vid-cargo's build.rs → reactos-link,
# ModuleKind::KernelDll, entry=DriverEntry, no exports). add_cargo_module builds
# arm64vid-rust, links it with the ntoskrnl/hal import libs into the shipping PE,
# applies the kernel-dll post-link and registers it on the boot media.
add_cargo_module(arm64vid arm64vid-cargo arm64vid.sys KIND kerneldll
    BODY_CRATE arm64vid-rust DESTINATION reactos/system32/drivers)
