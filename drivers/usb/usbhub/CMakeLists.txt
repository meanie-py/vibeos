
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbhub .c bodies are replaced by Rust (usbhub-rust). .c files kept on disk (never-delete).
# Bar: link-clean x3 (NOT boot-loaded in smoke -- boots from CD/ramdisk, not USB).
set(USBHUB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBHUB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbhub-target")
set(USBHUB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbhub_rust.a")
if(ARCH STREQUAL "i386")
    set(USBHUB_RUST_TARGET "i686-pc-windows-gnu")
    set(USBHUB_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBHUB_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBHUB_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBHUB_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBHUB_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE USBHUB_RUST_SRCS ${USBHUB_RUST_WORKSPACE_DIR}/usbhub-rust/src/*.rs)
add_custom_command(
    OUTPUT ${USBHUB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBHUB_RUST_TARGET_DIR} RUSTFLAGS=${USBHUB_RUST_RUSTFLAGS} RUSTC_BOOTSTRAP=1
        cargo build --release --target ${USBHUB_RUST_TARGET} -p usbhub-rust --manifest-path ${USBHUB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBHUB_RUST_TARGET_DIR}/${USBHUB_RUST_TARGET}/release/libusbhub_rust.a ${USBHUB_RUST_LIB}
    DEPENDS ${USBHUB_RUST_SRCS} ${USBHUB_RUST_WORKSPACE_DIR}/usbhub-rust/Cargo.toml
    WORKING_DIRECTORY ${USBHUB_RUST_WORKSPACE_DIR}
    COMMENT "Building usbhub-rust (${USBHUB_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbhub DEPENDS ${USBHUB_RUST_LIB})

add_library(usbhub MODULE rust_cohab_stub.c usbhub.rc)
set_module_type(usbhub kernelmodedriver)
add_dependencies(usbhub rust_usbhub)
target_link_libraries(usbhub ${USBHUB_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbhub ntoskrnl hal wmilib usbd)
add_cd_file(TARGET usbhub DESTINATION reactos/system32/drivers NO_CAB FOR all)
