
include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# The usbport .c bodies (debug/device/endpoint/iface/ioctl/iso/pnp/power/queue/roothub/
# trfsplit/urb/usb2/usbport) are replaced by Rust (usbport-rust, libusbport_rust.a). The
# .c files are kept on disk per the never-delete rule. guid.c stays compiled as a data-only
# TU (DEFINE_GUID). The two driver exports USBPORT_GetHciMn / USBPORT_RegisterUSBPortDriver
# are #[no_mangle] extern "system" in the rlib; spec2def(usbport.spec) emits usbport.def so
# the linker exports them. Bar: link-clean x3 (NOT boot-loaded in smoke).
spec2def(usbport.sys usbport.spec ADD_IMPORTLIB)

set(USBPORT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(USBPORT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbport-target" usbport-rust)
set(USBPORT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbport_rust.a")
if(ARCH STREQUAL "i386")
    set(USBPORT_RUST_TARGET "i686-pc-windows-gnu")
    set(USBPORT_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBPORT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBPORT_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBPORT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBPORT_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBPORT_RUST_SRCS "${USBPORT_RUST_WORKSPACE_DIR}" usbport-rust)
add_custom_command(
    OUTPUT ${USBPORT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBPORT_RUST_TARGET_DIR} RUSTFLAGS=${USBPORT_RUST_RUSTFLAGS}
        cargo build --release --target ${USBPORT_RUST_TARGET} -p usbport-rust --manifest-path ${USBPORT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBPORT_RUST_TARGET_DIR}/${USBPORT_RUST_TARGET}/release/libusbport_rust.a ${USBPORT_RUST_LIB}
    DEPENDS ${USBPORT_RUST_SRCS} ${USBPORT_RUST_WORKSPACE_DIR}/usbport-rust/Cargo.toml
    WORKING_DIRECTORY ${USBPORT_RUST_WORKSPACE_DIR}
    COMMENT "Building usbport-rust (${USBPORT_RUST_TARGET})" VERBATIM)
add_custom_target(rust_usbport DEPENDS ${USBPORT_RUST_LIB})

# All usbport .c bodies are now Rust (usbport-rust). guid.c stays as the data-only TU
# (DEFINE_GUID via initguid.h — instantiates the interface/bus GUID symbols).
list(APPEND PCH_SKIP_SOURCE
    guid.c)

add_library(usbport MODULE
    rust_cohab_stub.c
    rust_varargs_shim.c
    ${PCH_SKIP_SOURCE}
    usbport.rc
    ${CMAKE_CURRENT_BINARY_DIR}/usbport.def)

target_compile_definitions(usbport
    # HACK: Disabled until USB Stack is reinvestigated.
    PRIVATE NO_KERNEL_LIST_ENTRY_CHECKS)

set_module_type(usbport kernelmodedriver)
add_dependencies(usbport rust_usbport)
target_link_libraries(usbport ${USBPORT_RUST_LIB} ${PSEH_LIB})
add_importlibs(usbport ntoskrnl hal)
add_cd_file(TARGET usbport DESTINATION reactos/system32/drivers NO_CAB FOR all)
