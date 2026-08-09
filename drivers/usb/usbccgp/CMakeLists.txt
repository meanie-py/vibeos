
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbccgp .c bodies are replaced by Rust (usbccgp-rust). .c files kept on disk (never-delete).
# Bar: link-clean x3 (NOT boot-loaded in smoke -- boots from CD/ramdisk, not USB).
set(USBCCGP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBCCGP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbccgp-target")
set(USBCCGP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbccgp_rust.a")
if(ARCH STREQUAL "i386")
    set(USBCCGP_RUST_TARGET "i686-pc-windows-gnu")
    set(USBCCGP_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBCCGP_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBCCGP_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBCCGP_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBCCGP_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBCCGP_RUST_SRCS "${USBCCGP_RUST_WORKSPACE_DIR}" usbccgp-rust)
add_custom_command(
    OUTPUT ${USBCCGP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBCCGP_RUST_TARGET_DIR} RUSTFLAGS=${USBCCGP_RUST_RUSTFLAGS}
        cargo build --release --target ${USBCCGP_RUST_TARGET} -p usbccgp-rust --manifest-path ${USBCCGP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBCCGP_RUST_TARGET_DIR}/${USBCCGP_RUST_TARGET}/release/libusbccgp_rust.a ${USBCCGP_RUST_LIB}
    DEPENDS ${USBCCGP_RUST_SRCS} ${USBCCGP_RUST_WORKSPACE_DIR}/usbccgp-rust/Cargo.toml
    WORKING_DIRECTORY ${USBCCGP_RUST_WORKSPACE_DIR}
    COMMENT "Building usbccgp-rust (${USBCCGP_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbccgp DEPENDS ${USBCCGP_RUST_LIB})

add_library(usbccgp MODULE rust_cohab_stub.c usbccgp.rc)
set_module_type(usbccgp kernelmodedriver)
add_dependencies(usbccgp rust_usbccgp)
target_link_libraries(usbccgp ${USBCCGP_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbccgp ntoskrnl hal usbd)
add_cd_file(TARGET usbccgp DESTINATION reactos/system32/drivers NO_CAB FOR all)
