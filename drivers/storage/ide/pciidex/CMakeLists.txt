
spec2def(pciidex.sys pciidex.spec ADD_IMPORTLIB)

# ============================================================================
# pciidex.sys — Rust cohabitation flip (fdo/miniport/pciidex/pdo/power.c -> Rust)
# ============================================================================
# The PCI IDE bus extension is translated to Rust (pciidex-legacy-rust, kept on disk per
# the never-delete rule). DriverEntry + the pciidex.spec PciIdeX* exports come from
# libpciidex_legacy_rust.a; the IDE/PnP DDK types come from the per-driver bindgen substrate
# (pciidex-legacy-rust/src/types_*.rs). cmake keeps the MODULE link (set_module_type +
# spec2def .def + add_importlibs(ntoskrnl hal)). pciidex-rust builds on PLAIN STABLE
# rustc: its three C-variadic entry points (the ntstrsafe wrappers + the exported
# PciIdeXDebugPrint) cannot be DEFINED in stable Rust (`c_variadic`, rust-lang#44930),
# so they are 3-line trampolines in rust_varargs_shim.c that call the fixed-arity `*_v`
# twins in the crate — all logic stays in Rust, and RUSTC_BOOTSTRAP is gone (2026-07-12).
# Standard per-arch RUST_TARGET, i386 no-SSE RUSTFLAGS.
# PARKED SUCCESSOR (2026-10-06): upstream's rewrite (core + chipset/ + software AHCI) is
# translated in sdk/lib/rust/pciidex-rust (rustconv port) but does not compile yet; until it
# does, pciidex.sys keeps building from the pre-rewrite translation, pciidex-legacy-rust.
set(PCIIDEX_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(PCIIDEX_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pciidex-target" pciidex-legacy-rust)
set(PCIIDEX_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpciidex_legacy_rust.a")
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
rust_crate_sources(PCIIDEX_RUST_SRCS "${PCIIDEX_RUST_WORKSPACE_DIR}" pciidex-legacy-rust)
add_custom_command(
    OUTPUT ${PCIIDEX_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PCIIDEX_RUST_TARGET_DIR}
        RUSTFLAGS=${PCIIDEX_RUST_RUSTFLAGS}
        cargo build --release --target ${PCIIDEX_RUST_TARGET}
            -p pciidex-legacy-rust
            --manifest-path ${PCIIDEX_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PCIIDEX_RUST_TARGET_DIR}/${PCIIDEX_RUST_TARGET}/release/libpciidex_legacy_rust.a
        ${PCIIDEX_RUST_LIB}
    DEPENDS ${PCIIDEX_RUST_SRCS}
            ${PCIIDEX_RUST_WORKSPACE_DIR}/pciidex-legacy-rust/Cargo.toml
    WORKING_DIRECTORY ${PCIIDEX_RUST_WORKSPACE_DIR}
    COMMENT "Building pciidex-legacy-rust (${PCIIDEX_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_pciidex DEPENDS ${PCIIDEX_RUST_LIB})

# Upstream (post-fork) rewrote pciidex around a new chipset/ quirks layer
# (chipset/*.c: ahci_generic/ahci_hw/ahci_io/amd/ati/cmd/intel/pata_generic/
# pata_hw/pata_io/pata_legacy/pctech/sil680/svw_pata/svw_sata/toshiba/via) plus
# a new acpi.c, calling back into fdo.c/pciidex.c internals we have translated
# to Rust. There is no Rust twin and no wiring for the new chipset framework in
# pciidex-rust, so — mirroring the atapi SATL precedent — this stays UNBUILT
# dead C on disk as a future translation target rather than being linked in
# half-integrated. See FLAGS in the merge report.

# C bodies replaced by Rust (fdo.c miniport.c pciidex.c pdo.c power.c) — kept on disk.
add_library(pciidex MODULE
    rust_cohab_stub.c
    rust_varargs_shim.c
    pciidex.rc
    ${CMAKE_CURRENT_BINARY_DIR}/pciidex.def)

set_module_type(pciidex kernelmodedriver)
add_importlibs(pciidex ntoskrnl hal)
add_dependencies(pciidex rust_pciidex)
target_link_libraries(pciidex ${PCIIDEX_RUST_LIB})
add_cd_file(TARGET pciidex DESTINATION reactos/system32/drivers NO_CAB FOR all)
