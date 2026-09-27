
# usbxhci is written in Rust from the start (usbxhci-rust, libusbxhci_rust.a); there is no C
# implementation. rust_stub.c only gives add_library a translation unit.
set(USBXHCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(USBXHCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbxhci-target" usbxhci-rust)
set(USBXHCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbxhci_rust.a")
if(ARCH STREQUAL "i386")
    set(USBXHCI_RUST_TARGET "i686-pc-windows-gnu")
    set(USBXHCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBXHCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBXHCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBXHCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBXHCI_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBXHCI_RUST_SRCS "${USBXHCI_RUST_WORKSPACE_DIR}" usbxhci-rust)
add_custom_command(
    OUTPUT ${USBXHCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBXHCI_RUST_TARGET_DIR} RUSTFLAGS=${USBXHCI_RUST_RUSTFLAGS}
        cargo build --release --target ${USBXHCI_RUST_TARGET} -p usbxhci-rust --manifest-path ${USBXHCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBXHCI_RUST_TARGET_DIR}/${USBXHCI_RUST_TARGET}/release/libusbxhci_rust.a ${USBXHCI_RUST_LIB}
    DEPENDS ${USBXHCI_RUST_SRCS} ${USBXHCI_RUST_WORKSPACE_DIR}/usbxhci-rust/Cargo.toml
    WORKING_DIRECTORY ${USBXHCI_RUST_WORKSPACE_DIR}
    COMMENT "Building usbxhci-rust (${USBXHCI_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbxhci DEPENDS ${USBXHCI_RUST_LIB})

add_library(usbxhci MODULE
    rust_stub.c
    usbxhci.rc)

set_module_type(usbxhci kernelmodedriver)
add_dependencies(usbxhci rust_usbxhci)
target_link_libraries(usbxhci ${USBXHCI_RUST_LIB})
add_importlibs(usbxhci usbport hal ntoskrnl)
add_cd_file(TARGET usbxhci DESTINATION reactos/system32/drivers NO_CAB FOR all)
