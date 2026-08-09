
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbuhci .c bodies (roothub/usbuhci) are replaced by Rust (usbuhci-rust,
# libusbuhci_rust.a). The .c files are kept on disk per the never-delete rule. guid.c stays
# compiled as a data-only TU (DEFINE_GUID). Bar: link-clean x3 (NOT boot-loaded in smoke).
set(USBUHCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBUHCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbuhci-target")
set(USBUHCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbuhci_rust.a")
if(ARCH STREQUAL "i386")
    set(USBUHCI_RUST_TARGET "i686-pc-windows-gnu")
    set(USBUHCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBUHCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBUHCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBUHCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBUHCI_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBUHCI_RUST_SRCS "${USBUHCI_RUST_WORKSPACE_DIR}" usbuhci-rust)
add_custom_command(
    OUTPUT ${USBUHCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBUHCI_RUST_TARGET_DIR} RUSTFLAGS=${USBUHCI_RUST_RUSTFLAGS}
        cargo build --release --target ${USBUHCI_RUST_TARGET} -p usbuhci-rust --manifest-path ${USBUHCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBUHCI_RUST_TARGET_DIR}/${USBUHCI_RUST_TARGET}/release/libusbuhci_rust.a ${USBUHCI_RUST_LIB}
    DEPENDS ${USBUHCI_RUST_SRCS} ${USBUHCI_RUST_WORKSPACE_DIR}/usbuhci-rust/Cargo.toml
    WORKING_DIRECTORY ${USBUHCI_RUST_WORKSPACE_DIR}
    COMMENT "Building usbuhci-rust (${USBUHCI_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbuhci DEPENDS ${USBUHCI_RUST_LIB})

add_library(usbuhci MODULE
    rust_cohab_stub.c
    guid.c
    usbuhci.rc)

target_compile_definitions(usbuhci
    # HACK: Disabled until USB Stack is reinvestigated.
    PRIVATE NO_KERNEL_LIST_ENTRY_CHECKS)

set_module_type(usbuhci kernelmodedriver)
add_dependencies(usbuhci rust_usbuhci)
target_link_libraries(usbuhci ${USBUHCI_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbuhci usbport usbd hal ntoskrnl)
add_cd_file(TARGET usbuhci DESTINATION reactos/system32/drivers NO_CAB FOR all)
