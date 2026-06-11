
spec2def(usbd.sys usbd.spec ADD_IMPORTLIB)

# ============================================================================
# usbd.sys — Rust cohabitation flip (usbd.c -> sdk/lib/rust/usbd-rust)
# ============================================================================
# usbd.c (the USBD library driver) is translated to Rust (usbd-rust, kept on
# disk per the never-delete rule). DriverEntry + the usbd.spec USBD_* exports
# come from libusbd_rust.a; the USB DDK types come from usb-sys (bindgen of
# usbdi.h/usbdlib.h). cmake keeps the MODULE link (set_module_type + spec2def
# .def + add_importlibs(ntoskrnl hal)). usbd-rust depends on ntkrnl-sys +
# usb-sys, so the custom_command mirrors the cmlib cohab (standard per-arch
# RUST_TARGET, i386 no-SSE RUSTFLAGS).
set(USBD_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBD_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbd-target")
set(USBD_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbd_rust.a")
if(ARCH STREQUAL "i386")
    set(USBD_RUST_TARGET "i686-pc-windows-gnu")
    set(USBD_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBD_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBD_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBD_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBD_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${USBD_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBD_RUST_TARGET_DIR}
        RUSTFLAGS=${USBD_RUST_RUSTFLAGS}
        cargo build --release --target ${USBD_RUST_TARGET}
            -p usbd-rust
            --manifest-path ${USBD_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBD_RUST_TARGET_DIR}/${USBD_RUST_TARGET}/release/libusbd_rust.a
        ${USBD_RUST_LIB}
    DEPENDS ${USBD_RUST_WORKSPACE_DIR}/usbd-rust/Cargo.toml
            ${USBD_RUST_WORKSPACE_DIR}/usbd-rust/src/lib.rs
            ${USBD_RUST_WORKSPACE_DIR}/usbd-rust/src/usbd.rs
            ${USBD_RUST_WORKSPACE_DIR}/usb-sys/src/lib.rs
    WORKING_DIRECTORY ${USBD_RUST_WORKSPACE_DIR}
    COMMENT "Building usbd-rust (${USBD_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_usbd DEPENDS ${USBD_RUST_LIB})

# add_library(usbd MODULE usbd.c ...)  # body replaced by Rust; kept on disk
add_library(usbd MODULE
    rust_cohab_stub.c
    usbd.rc
    ${CMAKE_CURRENT_BINARY_DIR}/usbd.def)

set_module_type(usbd kernelmodedriver)
add_importlibs(usbd ntoskrnl hal)
add_dependencies(usbd rust_usbd)
target_link_libraries(usbd ${USBD_RUST_LIB})
add_cd_file(TARGET usbd DESTINATION reactos/system32/drivers NO_CAB FOR all)
