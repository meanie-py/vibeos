
# ============================================================================
# buslogic.sys — Rust cohabitation flip (BusLogic958.c -> buslogic-rust)
# ============================================================================
# The BusLogic BT-958 SCSI HBA miniport (BusLogic958.c, single TU) is translated to
# Rust (sdk/lib/rust/buslogic-rust, kept on disk per the never-delete rule; rustconv
# opus + c2rust-anchored, then hand-integrated). DriverEntry comes from libbuslogic_rust.a;
# the driver-private structs (BusLogic_*/CCB/mailbox + SCSI SRB/CDB/sense DDK layouts)
# from the per-driver bindgen substrate (buslogic-rust/src/types_*.rs). buslogic.sys is a
# SCSIPORT miniport (imports scsiport) and is NOT on any QEMU boot path (live SCSI/IDE =
# uniata) -> link-clean x3 is the verification bar. Standard per-arch RUST_TARGET, i386 no-SSE.
set(BUSLOGIC_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(BUSLOGIC_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-buslogic-target")
set(BUSLOGIC_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libbuslogic_rust.a")
if(ARCH STREQUAL "i386")
    set(BUSLOGIC_RUST_TARGET "i686-pc-windows-gnu")
    set(BUSLOGIC_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(BUSLOGIC_RUST_TARGET "x86_64-pc-windows-gnu")
    set(BUSLOGIC_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(BUSLOGIC_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(BUSLOGIC_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE BUSLOGIC_RUST_SRCS ${BUSLOGIC_RUST_WORKSPACE_DIR}/buslogic-rust/src/*.rs)
add_custom_command(
    OUTPUT ${BUSLOGIC_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${BUSLOGIC_RUST_TARGET_DIR}
        RUSTC_BOOTSTRAP=1
        RUSTFLAGS=${BUSLOGIC_RUST_RUSTFLAGS}
        cargo build --release --target ${BUSLOGIC_RUST_TARGET}
            -p buslogic-rust
            --manifest-path ${BUSLOGIC_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${BUSLOGIC_RUST_TARGET_DIR}/${BUSLOGIC_RUST_TARGET}/release/libbuslogic_rust.a
        ${BUSLOGIC_RUST_LIB}
    DEPENDS ${BUSLOGIC_RUST_SRCS} ${BUSLOGIC_RUST_WORKSPACE_DIR}/buslogic-rust/Cargo.toml
    WORKING_DIRECTORY ${BUSLOGIC_RUST_WORKSPACE_DIR}
    COMMENT "Building buslogic-rust (${BUSLOGIC_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_buslogic DEPENDS ${BUSLOGIC_RUST_LIB})

# C body replaced by Rust (BusLogic958.c) — kept on disk per the never-delete rule.
add_library(buslogic MODULE
    rust_cohab_stub.c
    BusLogic958.rc)
target_link_libraries(buslogic libcntpr)
set_module_type(buslogic kernelmodedriver)
add_importlibs(buslogic scsiport)
add_dependencies(buslogic rust_buslogic)
# C-cohab object(s) and the Rust rlib can both supply transiently-shared helper symbols.
target_link_options(buslogic PRIVATE -Wl,--allow-multiple-definition)
target_link_libraries(buslogic ${BUSLOGIC_RUST_LIB})
add_cd_file(TARGET buslogic DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(buslogic_reg.inf)
