
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

# ============================================================================
# pci.sys — cargo-driven final link (sdk/lib/rust/pci-cargo via reactos-link)
# ============================================================================
# pci.c/fdo.c/pdo.c are translated to Rust (sdk/lib/rust/pci-rust, kept on disk
# per the never-delete rule); the Rust #[no_mangle] DriverEntry is the PE entry.
# Replaces add_library(pci MODULE rust_cohab_stub.c pci.rc) + set_module_type +
# add_importlibs(ntoskrnl hal) + the rust_pci custom_command — pci-cargo's
# build.rs (reactos-link, ModuleKind::KernelDll, user-target body + i386
# -sse,-sse2) builds pci-rust and links the ntoskrnl/hal import libs + memcmp.
# Version resource (pci.rc) dropped, as in every driver cargo cut.
add_cargo_module(pci pci-cargo pci.sys KIND kerneldll
    BODY_CRATE pci-rust DESTINATION reactos/system32/drivers)
