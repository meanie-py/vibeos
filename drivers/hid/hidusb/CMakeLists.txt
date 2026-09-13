
# hidusb.c replaced by Rust (sdk/lib/rust/hidusb-rust) and kept on disk per the never-delete
# rule. The USB HID minidriver body — DriverEntry/AddDevice, async URB build/dispatch
# (Hid_DispatchUrb), the work-item pipe/port RESET recovery, the multi-step descriptor chain,
# and the read-report completion path — comes from libhidusb_rust.a. It calls
# HidRegisterMinidriver (hidclass import) + USBD_CreateConfigurationRequestEx /
# USBD_ParseConfigurationDescriptorEx (usbd import); Io/Ke/Ex/Mm/Rtl stay ntoskrnl imports.
# Not boot-loaded in the QEMU smoke path (no USB HID) — bar is link-clean x3 + no regression.

set(HIDUSB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(HIDUSB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-hidusb-target" hidusb-rust)
set(HIDUSB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libhidusb_rust.a")

if(ARCH STREQUAL "i386")
    set(HIDUSB_RUST_TARGET "i686-pc-windows-gnu")
    set(HIDUSB_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(HIDUSB_RUST_TARGET "x86_64-pc-windows-gnu")
    set(HIDUSB_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(HIDUSB_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(HIDUSB_RUST_RUSTFLAGS "")
endif()

rust_crate_sources(HIDUSB_RUST_SRCS "${HIDUSB_RUST_WORKSPACE_DIR}" hidusb-rust)

add_custom_command(
    OUTPUT ${HIDUSB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${HIDUSB_RUST_TARGET_DIR}
        RUSTFLAGS=${HIDUSB_RUST_RUSTFLAGS}
        cargo build --release --target ${HIDUSB_RUST_TARGET}
            -p hidusb-rust
            --manifest-path ${HIDUSB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${HIDUSB_RUST_TARGET_DIR}/${HIDUSB_RUST_TARGET}/release/libhidusb_rust.a
        ${HIDUSB_RUST_LIB}
    DEPENDS ${HIDUSB_RUST_SRCS} ${HIDUSB_RUST_WORKSPACE_DIR}/hidusb-rust/Cargo.toml
    WORKING_DIRECTORY ${HIDUSB_RUST_WORKSPACE_DIR}
    COMMENT "Building hidusb-rust (${HIDUSB_RUST_TARGET})"
    VERBATIM)

add_custom_target(rust_hidusb DEPENDS ${HIDUSB_RUST_LIB})

# replaced by Rust (sdk/lib/rust/hidusb-rust): hidusb.c
add_library(hidusb MODULE rust_cohab_stub.c hidusb.rc)
set_module_type(hidusb kernelmodedriver)
add_importlibs(hidusb hidclass ntoskrnl usbd hal)
add_dependencies(hidusb rust_hidusb)
target_link_libraries(hidusb ${HIDUSB_RUST_LIB})
add_cd_file(TARGET hidusb DESTINATION reactos/system32/drivers NO_CAB FOR all)
