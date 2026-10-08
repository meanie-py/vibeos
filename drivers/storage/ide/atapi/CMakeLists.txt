
# ============================================================================
# atapi.sys — Rust cohabitation flip (atapi.c -> atapi-rust)
# ============================================================================
# The NT4-DDK ATAPI IDE miniport (atapi.c, single TU, since deleted) was translated to Rust
# (sdk/lib/rust/atapi-legacy-rust, kept on disk per the never-delete rule; 5-way parallel
# HAND translation — link-only-verified driver, so hand fidelity beats DeepSeek).
# DriverEntry comes from libatapi_legacy_rust.a; the IDE/IDENTIFY/SCSI-SRB private types
# from the per-driver bindgen substrate (atapi-legacy-rust/src/types_*.rs). atapi.sys is a
# SCSIPORT miniport (imports scsiport) and is NOT in txtsetup -> link-clean ×3 is
# the verification bar. Standard per-arch RUST_TARGET, i386 no-SSE RUSTFLAGS.
# PARKED SUCCESSOR (2026-10-06): upstream's 2026 ATA port driver is translated in
# sdk/lib/rust/atapi-rust (rustconv port) but does not compile yet; until it does, atapi.sys
# keeps building from the NT4-DDK miniport translation, renamed atapi-legacy-rust. The
# cohab CMakeLists for the new crate is drafted in the upstream-sync lane doc's ATA section.
set(ATAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(ATAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-atapi-target" atapi-legacy-rust)
set(ATAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libatapi_legacy_rust.a")
if(ARCH STREQUAL "i386")
    set(ATAPI_RUST_TARGET "i686-pc-windows-gnu")
    set(ATAPI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(ATAPI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(ATAPI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(ATAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(ATAPI_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(ATAPI_RUST_SRCS "${ATAPI_RUST_WORKSPACE_DIR}" atapi-legacy-rust)
add_custom_command(
    OUTPUT ${ATAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ATAPI_RUST_TARGET_DIR}
        RUSTFLAGS=${ATAPI_RUST_RUSTFLAGS}
        cargo build --release --target ${ATAPI_RUST_TARGET}
            -p atapi-legacy-rust
            --manifest-path ${ATAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ATAPI_RUST_TARGET_DIR}/${ATAPI_RUST_TARGET}/release/libatapi_legacy_rust.a
        ${ATAPI_RUST_LIB}
    DEPENDS ${ATAPI_RUST_SRCS}
            ${ATAPI_RUST_WORKSPACE_DIR}/atapi-legacy-rust/Cargo.toml
    WORKING_DIRECTORY ${ATAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building atapi-legacy-rust (${ATAPI_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_atapi DEPENDS ${ATAPI_RUST_LIB})

# C body replaced by Rust; atapi.c/.h DELETED 2026-09-20 (Microsoft NT4 DDK EULA,
# the only non-FLOSS grant in the tree). atapi.sys is built from the Rust alone.
add_library(atapi MODULE
    rust_cohab_stub.c
    atapi.rc)
target_link_libraries(atapi libcntpr)
set_module_type(atapi kernelmodedriver)
add_importlibs(atapi scsiport)
add_dependencies(atapi rust_atapi)
target_link_libraries(atapi ${ATAPI_RUST_LIB})
add_cd_file(TARGET atapi DESTINATION reactos/system32/drivers NO_CAB FOR all)
