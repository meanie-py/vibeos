
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbohci .c bodies (roothub/usbohci) are replaced by Rust (usbohci-rust,
# libusbohci_rust.a). The .c files are kept on disk per the never-delete rule. guid.c stays
# compiled as a data-only TU (DEFINE_GUID). Bar: link-clean x3 (NOT boot-loaded in smoke).
set(USBOHCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBOHCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbohci-target")
set(USBOHCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbohci_rust.a")
if(ARCH STREQUAL "i386")
    set(USBOHCI_RUST_TARGET "i686-pc-windows-gnu")
    set(USBOHCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBOHCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBOHCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBOHCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBOHCI_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE USBOHCI_RUST_SRCS ${USBOHCI_RUST_WORKSPACE_DIR}/usbohci-rust/src/*.rs)
add_custom_command(
    OUTPUT ${USBOHCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBOHCI_RUST_TARGET_DIR} RUSTFLAGS=${USBOHCI_RUST_RUSTFLAGS}
        cargo build --release --target ${USBOHCI_RUST_TARGET} -p usbohci-rust --manifest-path ${USBOHCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBOHCI_RUST_TARGET_DIR}/${USBOHCI_RUST_TARGET}/release/libusbohci_rust.a ${USBOHCI_RUST_LIB}
    DEPENDS ${USBOHCI_RUST_SRCS} ${USBOHCI_RUST_WORKSPACE_DIR}/usbohci-rust/Cargo.toml
    WORKING_DIRECTORY ${USBOHCI_RUST_WORKSPACE_DIR}
    COMMENT "Building usbohci-rust (${USBOHCI_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbohci DEPENDS ${USBOHCI_RUST_LIB})

add_library(usbohci MODULE
    rust_cohab_stub.c
    guid.c
    usbohci.rc)

target_compile_definitions(usbohci
    # HACK: Disabled until USB Stack is reinvestigated.
    PRIVATE NO_KERNEL_LIST_ENTRY_CHECKS)

set_module_type(usbohci kernelmodedriver)
add_dependencies(usbohci rust_usbohci)
target_link_libraries(usbohci ${USBOHCI_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbohci usbport usbd hal ntoskrnl)
add_cd_file(TARGET usbohci DESTINATION reactos/system32/drivers NO_CAB FOR all)
