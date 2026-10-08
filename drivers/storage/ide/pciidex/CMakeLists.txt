
spec2def(pciidex.sys pciidex.spec ADD_IMPORTLIB)

# ============================================================================
# pciidex.sys — Rust cohabitation flip (upstream's 2026 rewrite -> pciidex-rust)
# ============================================================================
# The PCI IDE bus extension as upstream rewrote it after the fork: the controller/channel
# FDO+PDO core (fdo/pdo/pciidex/miniport/power/acpi.c) plus the chipset/ layer (per-vendor
# PATA quirks and a software AHCI HBA driver, ahci_generic/ahci_hw/ahci_io). All of it is
# translated to Rust in sdk/lib/rust/pciidex-rust (rustconv port, sealed DeepSeek). The
# translation of the PRE-rewrite pciidex is parked as pciidex-legacy-rust (never-delete).
# The pciidex.spec exports (PciIdeXInitialize/GetBusData/SetBusData/GetBusLocation and the
# varargs PciIdeXDebugPrint) come from the Rust #[no_mangle] symbols, except that
# PciIdeXDebugPrint is DEFINED by a 3-line trampoline in rust_varargs_shim.c (stable Rust
# cannot define a C-variadic fn) that calls PciIdeXDebugPrint_v in the crate.
set(PCIIDEX_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(PCIIDEX_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pciidex-target" pciidex-rust)
set(PCIIDEX_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpciidex_rust.a")
if(ARCH STREQUAL "i386")
    set(PCIIDEX_RUST_TARGET "i686-pc-windows-gnu")
    set(PCIIDEX_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PCIIDEX_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PCIIDEX_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PCIIDEX_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PCIIDEX_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(PCIIDEX_RUST_SRCS "${PCIIDEX_RUST_WORKSPACE_DIR}" pciidex-rust)
add_custom_command(
    OUTPUT ${PCIIDEX_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PCIIDEX_RUST_TARGET_DIR}
        RUSTFLAGS=${PCIIDEX_RUST_RUSTFLAGS}
        cargo build --release --target ${PCIIDEX_RUST_TARGET}
            -p pciidex-rust
            --manifest-path ${PCIIDEX_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PCIIDEX_RUST_TARGET_DIR}/${PCIIDEX_RUST_TARGET}/release/libpciidex_rust.a
        ${PCIIDEX_RUST_LIB}
    DEPENDS ${PCIIDEX_RUST_SRCS}
            ${PCIIDEX_RUST_WORKSPACE_DIR}/pciidex-rust/Cargo.toml
    WORKING_DIRECTORY ${PCIIDEX_RUST_WORKSPACE_DIR}
    COMMENT "Building pciidex-rust (${PCIIDEX_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_pciidex DEPENDS ${PCIIDEX_RUST_LIB})

# C bodies replaced by Rust — kept on disk as the translation reference.
# list(APPEND SOURCE
#     chipset/ahci_generic.c
#     chipset/ahci_hw.c
#     chipset/ahci_io.c
#     chipset/amd.c
#     chipset/ati.c
#     chipset/cmd.c
#     chipset/intel.c
#     chipset/pata_generic.c
#     chipset/pata_hw.c
#     chipset/pata_io.c
#     chipset/pctech.c
#     chipset/sil680.c
#     chipset/svw_pata.c
#     chipset/svw_sata.c
#     chipset/toshiba.c
#     chipset/via.c
#     acpi.c
#     fdo.c
#     miniport.c
#     pciidex.c
#     pdo.c
#     power.c)
# i386 only, under -DATA_DETECT_LEGACY_DEVICES (in the crate: `#[cfg(target_arch = "x86")]`):
# if(ARCH STREQUAL "i386")
# list(APPEND SOURCE chipset/pata_legacy.c)
# endif()

# ${SOURCE} is empty while the list above stays commented; it is spelled here so
# `profile_derive --flipped-sources` can re-audit the C the Rust replaced.
add_library(pciidex MODULE
    ${SOURCE}
    rust_cohab_stub.c
    rust_varargs_shim.c
    pciidex.rc
    ${CMAKE_CURRENT_BINARY_DIR}/pciidex.def)

set_module_type(pciidex kernelmodedriver)
add_importlibs(pciidex ntoskrnl hal)
add_dependencies(pciidex rust_pciidex)
target_link_libraries(pciidex ${PCIIDEX_RUST_LIB})
add_cd_file(TARGET pciidex DESTINATION reactos/system32/drivers NO_CAB FOR all)
