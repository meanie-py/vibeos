
# ============================================================================
# vioblk.sys — virtio-blk SCSIPORT miniport, written in Rust (sdk/lib/rust/vioblk-rust).
# ============================================================================
# There is no C body: before this the x86 kernel had no virtio-blk driver at all. The
# module is a thin link of libvioblk_rust.a (its #[no_mangle] DriverEntry) against
# scsiport. Boot-start (vioblk_reg.inf) and bound to PCI\VEN_1AF4&DEV_1001/1042 through
# the CriticalDeviceDatabase (boot/bootdata/hivesys.inf).
set(VIOBLK_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(VIOBLK_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-vioblk-target" vioblk-rust)
set(VIOBLK_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libvioblk_rust.a")
if(ARCH STREQUAL "i386")
    set(VIOBLK_RUST_TARGET "i686-pc-windows-gnu")
    set(VIOBLK_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(VIOBLK_RUST_TARGET "x86_64-pc-windows-gnu")
    set(VIOBLK_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(VIOBLK_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(VIOBLK_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(VIOBLK_RUST_SRCS "${VIOBLK_RUST_WORKSPACE_DIR}" vioblk-rust)
add_custom_command(
    OUTPUT ${VIOBLK_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${VIOBLK_RUST_TARGET_DIR}
        RUSTFLAGS=${VIOBLK_RUST_RUSTFLAGS}
        cargo build --release --target ${VIOBLK_RUST_TARGET}
            -p vioblk-rust
            --manifest-path ${VIOBLK_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${VIOBLK_RUST_TARGET_DIR}/${VIOBLK_RUST_TARGET}/release/libvioblk_rust.a
        ${VIOBLK_RUST_LIB}
    DEPENDS ${VIOBLK_RUST_SRCS} ${VIOBLK_RUST_WORKSPACE_DIR}/vioblk-rust/Cargo.toml
    WORKING_DIRECTORY ${VIOBLK_RUST_WORKSPACE_DIR}
    COMMENT "Building vioblk-rust (${VIOBLK_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_vioblk DEPENDS ${VIOBLK_RUST_LIB})

add_library(vioblk MODULE
    rust_stub.c
    vioblk.rc)
target_link_libraries(vioblk libcntpr)
set_module_type(vioblk kernelmodedriver)
add_importlibs(vioblk scsiport hal ntoskrnl)
add_dependencies(vioblk rust_vioblk)
target_link_libraries(vioblk ${VIOBLK_RUST_LIB})
add_cd_file(TARGET vioblk DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(vioblk_reg.inf)
