
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbehci .c bodies (debug/roothub/usbehci) are replaced by Rust (usbehci-rust,
# libusbehci_rust.a). The .c files are kept on disk per the never-delete rule. guid.c stays
# compiled as a data-only TU (DEFINE_GUID). Bar: link-clean x3 (NOT boot-loaded in smoke).
set(USBEHCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBEHCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbehci-target")
set(USBEHCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbehci_rust.a")
if(ARCH STREQUAL "i386")
    set(USBEHCI_RUST_TARGET "i686-pc-windows-gnu")
    set(USBEHCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBEHCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBEHCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBEHCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBEHCI_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE USBEHCI_RUST_SRCS ${USBEHCI_RUST_WORKSPACE_DIR}/usbehci-rust/src/*.rs)
add_custom_command(
    OUTPUT ${USBEHCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBEHCI_RUST_TARGET_DIR} RUSTFLAGS=${USBEHCI_RUST_RUSTFLAGS} RUSTC_BOOTSTRAP=1
        cargo build --release --target ${USBEHCI_RUST_TARGET} -p usbehci-rust --manifest-path ${USBEHCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBEHCI_RUST_TARGET_DIR}/${USBEHCI_RUST_TARGET}/release/libusbehci_rust.a ${USBEHCI_RUST_LIB}
    DEPENDS ${USBEHCI_RUST_SRCS} ${USBEHCI_RUST_WORKSPACE_DIR}/usbehci-rust/Cargo.toml
    WORKING_DIRECTORY ${USBEHCI_RUST_WORKSPACE_DIR}
    COMMENT "Building usbehci-rust (${USBEHCI_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbehci DEPENDS ${USBEHCI_RUST_LIB})

add_library(usbehci MODULE
    rust_cohab_stub.c
    guid.c
    usbehci.rc)

target_compile_definitions(usbehci
    # HACK: Disabled until USB Stack is reinvestigated.
    PRIVATE NO_KERNEL_LIST_ENTRY_CHECKS)

set_module_type(usbehci kernelmodedriver)
add_dependencies(usbehci rust_usbehci)
target_link_libraries(usbehci ${USBEHCI_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbehci usbport usbd hal ntoskrnl)
add_cd_file(TARGET usbehci DESTINATION reactos/system32/drivers NO_CAB FOR all)
