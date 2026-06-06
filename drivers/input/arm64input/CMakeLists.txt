
# ============================================================================
# arm64input.sys — cargo-driven final link (sdk/lib/rust/arm64input-cargo via reactos-link)
# ============================================================================
# arm64-only virtio-input driver. arm64input.c is fully translated to Rust
# (sdk/lib/rust/arm64input-rust, kept on disk per the never-delete rule); its
# DriverEntry is the PE entry point. The final arm64input.sys link is no longer
# cmake's add_library(MODULE) + rust_cohab_stub.c + custom_command scaffolding —
# it is driven by cargo (arm64input-cargo's build.rs → reactos-link,
# ModuleKind::KernelDll, entry=DriverEntry, no exports). add_cargo_module builds
# arm64input-rust + links it with the ntoskrnl/hal import libs into the shipping
# PE and applies the kernel-dll post-link. NO_CD mirrors the original (the C
# target registered no add_cd_file; mkramdisk picks the .sys up by path for the
# arm64 ramdisk). Only added for ARCH==arm64 (drivers/input/CMakeLists.txt).
add_cargo_module(arm64input arm64input-cargo arm64input.sys KIND kerneldll
    BODY_CRATE arm64input-rust NO_CD)
