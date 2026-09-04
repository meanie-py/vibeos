
# ============================================================================
# partmgr.sys — Rust cohabitation flip (partition.c/partmgr.c/utils.c -> partmgr-rust)
# ============================================================================
# The partition manager (partition.c PartitionCreateDevice + PDO PnP/IOCTL, partmgr.c
# DriverEntry + FDO PnP/power/layout IOCTLs, utils.c ForwardIrpAndForget +
# IssueSyncIoControlRequest) is translated to Rust (sdk/lib/rust/partmgr-rust, kept on
# disk per the never-delete rule). DriverEntry + all routines come from libpartmgr_rust.a.
# guid.c (the DEFINE_GUID data the Rust modules reference as extern statics, and the C
# TU the MODULE link needs) stays C. partmgr IS boot-loaded storage infra, so the bar
# is a clean smoke boot ×3. Driver-private structs come from the gen_driver_types.sh
# bindgen substrate (partmgr-rust/src/types.rs). Mirrors the pci-rust / processr cohab.
set(PARTMGR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PARTMGR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-partmgr-target")
set(PARTMGR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpartmgr_rust.a")
if(ARCH STREQUAL "i386")
    set(PARTMGR_RUST_TARGET "i686-pc-windows-gnu")
    set(PARTMGR_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PARTMGR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PARTMGR_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PARTMGR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PARTMGR_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(PARTMGR_RUST_SRCS "${PARTMGR_RUST_WORKSPACE_DIR}" partmgr-rust)
add_custom_command(
    OUTPUT ${PARTMGR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PARTMGR_RUST_TARGET_DIR}
        RUSTFLAGS=${PARTMGR_RUST_RUSTFLAGS}
        cargo build --release --target ${PARTMGR_RUST_TARGET}
            -p partmgr-rust
            --manifest-path ${PARTMGR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PARTMGR_RUST_TARGET_DIR}/${PARTMGR_RUST_TARGET}/release/libpartmgr_rust.a
        ${PARTMGR_RUST_LIB}
    DEPENDS ${PARTMGR_RUST_SRCS}
            ${PARTMGR_RUST_WORKSPACE_DIR}/partmgr-rust/Cargo.toml
    WORKING_DIRECTORY ${PARTMGR_RUST_WORKSPACE_DIR}
    COMMENT "Building partmgr-rust (${PARTMGR_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_partmgr DEPENDS ${PARTMGR_RUST_LIB})

# partition.c / partmgr.c / utils.c bodies replaced by Rust (partmgr-rust), kept on
# disk per the never-delete rule. guid.c (GUID data + the C TU) stays C.
# list(APPEND SOURCE partition.c partmgr.c utils.c)  # replaced by Rust (partmgr-rust)
list(APPEND PCH_SKIP_SOURCE
    guid.c)

add_library(partmgr MODULE
	${PCH_SKIP_SOURCE}
	partmgr.rc)

set_module_type(partmgr kernelmodedriver)
add_importlibs(partmgr ntoskrnl hal)
add_dependencies(partmgr rust_partmgr)
target_link_libraries(partmgr ${PARTMGR_RUST_LIB})
add_registry_inf(partmgr_reg.inf)
add_cd_file(TARGET partmgr DESTINATION reactos/system32/drivers NO_CAB FOR all)
