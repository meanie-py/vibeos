
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/sptilib)

# The 9 usbstor .c bodies (descriptor/disk/error/fdo/misc/pdo/queue/scsi/usbstor + guid) are
# replaced by Rust (usbstor-rust). USB Bulk-Only mass-storage class driver -- bridges USBDI/URB
# to the SCSI/disk class. .c files kept on disk (never-delete). Bar: link-clean x3 (NOT
# boot-loaded in smoke -- boots from CD/ramdisk, not USB).
set(USBSTOR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(USBSTOR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbstor-target" usbstor-rust)
set(USBSTOR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbstor_rust.a")
if(ARCH STREQUAL "i386")
    set(USBSTOR_RUST_TARGET "i686-pc-windows-gnu")
    set(USBSTOR_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBSTOR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBSTOR_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBSTOR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBSTOR_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBSTOR_RUST_SRCS "${USBSTOR_RUST_WORKSPACE_DIR}" usbstor-rust)
add_custom_command(
    OUTPUT ${USBSTOR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBSTOR_RUST_TARGET_DIR} RUSTFLAGS=${USBSTOR_RUST_RUSTFLAGS}
        cargo build --release --target ${USBSTOR_RUST_TARGET} -p usbstor-rust --manifest-path ${USBSTOR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBSTOR_RUST_TARGET_DIR}/${USBSTOR_RUST_TARGET}/release/libusbstor_rust.a ${USBSTOR_RUST_LIB}
    DEPENDS ${USBSTOR_RUST_SRCS} ${USBSTOR_RUST_WORKSPACE_DIR}/usbstor-rust/Cargo.toml
    WORKING_DIRECTORY ${USBSTOR_RUST_WORKSPACE_DIR}
    COMMENT "Building usbstor-rust (${USBSTOR_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbstor DEPENDS ${USBSTOR_RUST_LIB})

add_library(usbstor MODULE rust_cohab_stub.c rust_varargs_shim.c usbstor.rc)
set_module_type(usbstor kernelmodedriver)
add_dependencies(usbstor rust_usbstor)
# Rust lib FIRST so lld's single-pass static-lib resolution sees its reference to
# SptiHandleScsiPassthru BEFORE sptilib is processed; sptilib + PSEH after.
target_link_libraries(usbstor ${USBSTOR_RUST_LIB} sptilib ${PSEH_LIB})
add_importlibs(usbstor ntoskrnl hal usbd)
add_cd_file(TARGET usbstor DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_driver_inf(usbstor usbstor.inf)
