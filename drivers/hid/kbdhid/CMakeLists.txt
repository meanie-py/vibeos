
# kbdhid.c is replaced by Rust (sdk/lib/rust/kbdhid-rust) and kept on disk per the
# never-delete rule. DriverEntry + the IRP/PnP/Power dispatch + the HID read loop come from
# libkbdhid_rust.a; the Io/Ke/Ex/Mm/Rtl entry points stay ntoskrnl imports and the HidP_*
# parser functions stay hidparse imports (add_importlibs ... hidparse). kbdhid is a HID
# client driver, not boot-loaded in the QEMU smoke path (QEMU uses PS/2 via i8042prt), so
# the bar is link-clean x3 + no smoke regression.

set(KBDHID_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(KBDHID_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kbdhid-target")
set(KBDHID_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkbdhid_rust.a")

if(ARCH STREQUAL "i386")
    set(KBDHID_RUST_TARGET "i686-pc-windows-gnu")
    set(KBDHID_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(KBDHID_RUST_TARGET "x86_64-pc-windows-gnu")
    set(KBDHID_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(KBDHID_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(KBDHID_RUST_RUSTFLAGS "")
endif()

rust_crate_sources(KBDHID_RUST_SRCS "${KBDHID_RUST_WORKSPACE_DIR}" kbdhid-rust)

add_custom_command(
    OUTPUT ${KBDHID_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KBDHID_RUST_TARGET_DIR}
        RUSTFLAGS=${KBDHID_RUST_RUSTFLAGS}
        cargo build --release --target ${KBDHID_RUST_TARGET}
            -p kbdhid-rust
            --manifest-path ${KBDHID_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KBDHID_RUST_TARGET_DIR}/${KBDHID_RUST_TARGET}/release/libkbdhid_rust.a
        ${KBDHID_RUST_LIB}
    DEPENDS ${KBDHID_RUST_SRCS} ${KBDHID_RUST_WORKSPACE_DIR}/kbdhid-rust/Cargo.toml
    WORKING_DIRECTORY ${KBDHID_RUST_WORKSPACE_DIR}
    COMMENT "Building kbdhid-rust (${KBDHID_RUST_TARGET})"
    VERBATIM)

add_custom_target(rust_kbdhid DEPENDS ${KBDHID_RUST_LIB})

# replaced by Rust (sdk/lib/rust/kbdhid-rust): kbdhid.c
add_library(kbdhid MODULE rust_cohab_stub.c kbdhid.rc)
set_module_type(kbdhid kernelmodedriver)
add_importlibs(kbdhid ntoskrnl hal hidparse)
add_dependencies(kbdhid rust_kbdhid)
target_link_libraries(kbdhid ${KBDHID_RUST_LIB})
add_cd_file(TARGET kbdhid DESTINATION reactos/system32/drivers NO_CAB FOR all)
