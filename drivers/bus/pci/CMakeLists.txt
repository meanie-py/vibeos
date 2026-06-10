
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

# ============================================================================
# pci.sys — Rust cohabitation flip (pci.c/fdo.c/pdo.c -> pci-rust)
# ============================================================================
# The whole PCI bus driver (pci.c DriverEntry + device-ID string builders, fdo.c
# FDO PnP/power + bus enumeration, pdo.c PDO PnP/power + BUS_INTERFACE_STANDARD
# callbacks) is translated to Rust (sdk/lib/rust/pci-rust, kept on disk per the
# never-delete rule). DriverEntry + all routines come from libpci_rust.a. cmake
# keeps the MODULE link (set_module_type kernelmodedriver + add_importlibs(ntoskrnl
# hal) + .rc). pci-rust depends only on ntkrnl-sys; the custom_command mirrors the
# processr cohab.
set(PCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pci-target")
set(PCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpci_rust.a")
if(ARCH STREQUAL "i386")
    set(PCI_RUST_TARGET "i686-pc-windows-gnu")
    set(PCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PCI_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${PCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PCI_RUST_TARGET_DIR}
        RUSTFLAGS=${PCI_RUST_RUSTFLAGS}
        cargo build --release --target ${PCI_RUST_TARGET}
            -p pci-rust
            --manifest-path ${PCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PCI_RUST_TARGET_DIR}/${PCI_RUST_TARGET}/release/libpci_rust.a
        ${PCI_RUST_LIB}
    DEPENDS ${PCI_RUST_WORKSPACE_DIR}/pci-rust/Cargo.toml
            ${PCI_RUST_WORKSPACE_DIR}/pci-rust/src/lib.rs
            ${PCI_RUST_WORKSPACE_DIR}/pci-rust/src/shared.rs
            ${PCI_RUST_WORKSPACE_DIR}/pci-rust/src/pci.rs
            ${PCI_RUST_WORKSPACE_DIR}/pci-rust/src/fdo.rs
            ${PCI_RUST_WORKSPACE_DIR}/pci-rust/src/pdo.rs
    WORKING_DIRECTORY ${PCI_RUST_WORKSPACE_DIR}
    COMMENT "Building pci-rust (${PCI_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_pci DEPENDS ${PCI_RUST_LIB})

# list(APPEND SOURCE fdo.c pci.c pdo.c pci.h)  # body replaced by Rust; kept on disk
add_library(pci MODULE rust_cohab_stub.c pci.rc)
set_module_type(pci kernelmodedriver)
add_importlibs(pci ntoskrnl hal)
add_dependencies(pci rust_pci)
target_link_libraries(pci ${PCI_RUST_LIB})
add_cd_file(TARGET pci DESTINATION reactos/system32/drivers NO_CAB FOR all)
