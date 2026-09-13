
include_directories(..)

# ramdisk.c replaced by Rust (ramdisk-rust). Pure WDM RAM-disk class driver -- no miniport /
# no ISR / no DPC / no HW register I/O. ramdisk.c kept on disk (never-delete). Bar: link-clean x3.
set(RAMDISK_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(RAMDISK_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ramdisk-target" ramdisk-rust)
set(RAMDISK_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libramdisk_rust.a")
if(ARCH STREQUAL "i386")
    set(RAMDISK_RUST_TARGET "i686-pc-windows-gnu")
    set(RAMDISK_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(RAMDISK_RUST_TARGET "x86_64-pc-windows-gnu")
    set(RAMDISK_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(RAMDISK_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RAMDISK_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(RAMDISK_RUST_SRCS "${RAMDISK_RUST_WORKSPACE_DIR}" ramdisk-rust)
add_custom_command(
    OUTPUT ${RAMDISK_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RAMDISK_RUST_TARGET_DIR} RUSTFLAGS=${RAMDISK_RUST_RUSTFLAGS}
        cargo build --release --target ${RAMDISK_RUST_TARGET} -p ramdisk-rust --manifest-path ${RAMDISK_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RAMDISK_RUST_TARGET_DIR}/${RAMDISK_RUST_TARGET}/release/libramdisk_rust.a ${RAMDISK_RUST_LIB}
    DEPENDS ${RAMDISK_RUST_SRCS} ${RAMDISK_RUST_WORKSPACE_DIR}/ramdisk-rust/Cargo.toml
    WORKING_DIRECTORY ${RAMDISK_RUST_WORKSPACE_DIR}
    COMMENT "Building ramdisk-rust (${RAMDISK_RUST_TARGET})" VERBATIM)
add_custom_target(rust_ramdisk DEPENDS ${RAMDISK_RUST_LIB})

add_library(ramdisk MODULE rust_cohab_stub.c ramdisk.rc)
set_module_type(ramdisk kernelmodedriver)
add_importlibs(ramdisk ntoskrnl hal)
add_dependencies(ramdisk rust_ramdisk)
target_link_libraries(ramdisk ${RAMDISK_RUST_LIB})
add_cd_file(TARGET ramdisk DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_driver_inf(ramdisk ramdisk.inf)
