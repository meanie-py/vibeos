
# mouhid.c is replaced by Rust (sdk/lib/rust/mouhid-rust) and kept on disk per the
# never-delete rule. DriverEntry + the IRP/PnP/Power dispatch + the HID read loop come from
# libmouhid_rust.a; the Io/Ke/Ex/Mm/Rtl entry points stay ntoskrnl imports and the HidP_*
# parser functions stay hidparse imports (add_importlibs ... hidparse). mouhid is a HID
# client driver, not boot-loaded in the QEMU smoke path (QEMU uses PS/2 via i8042prt), so
# the bar is link-clean x3 + no smoke regression.

set(MOUHID_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(MOUHID_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mouhid-target")
set(MOUHID_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmouhid_rust.a")

if(ARCH STREQUAL "i386")
    set(MOUHID_RUST_TARGET "i686-pc-windows-gnu")
    set(MOUHID_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MOUHID_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MOUHID_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(MOUHID_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MOUHID_RUST_RUSTFLAGS "")
endif()

file(GLOB_RECURSE MOUHID_RUST_SRCS ${MOUHID_RUST_WORKSPACE_DIR}/mouhid-rust/src/*.rs)

add_custom_command(
    OUTPUT ${MOUHID_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MOUHID_RUST_TARGET_DIR}
        RUSTFLAGS=${MOUHID_RUST_RUSTFLAGS}
        RUSTC_BOOTSTRAP=1
        cargo build --release --target ${MOUHID_RUST_TARGET}
            -p mouhid-rust
            --manifest-path ${MOUHID_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MOUHID_RUST_TARGET_DIR}/${MOUHID_RUST_TARGET}/release/libmouhid_rust.a
        ${MOUHID_RUST_LIB}
    DEPENDS ${MOUHID_RUST_SRCS} ${MOUHID_RUST_WORKSPACE_DIR}/mouhid-rust/Cargo.toml
    WORKING_DIRECTORY ${MOUHID_RUST_WORKSPACE_DIR}
    COMMENT "Building mouhid-rust (${MOUHID_RUST_TARGET})"
    VERBATIM)

add_custom_target(rust_mouhid DEPENDS ${MOUHID_RUST_LIB})

# replaced by Rust (sdk/lib/rust/mouhid-rust): mouhid.c
add_library(mouhid MODULE rust_cohab_stub.c mouhid.rc)
set_module_type(mouhid kernelmodedriver)
add_importlibs(mouhid ntoskrnl hal hidparse)
add_dependencies(mouhid rust_mouhid)
target_link_libraries(mouhid ${MOUHID_RUST_LIB})
add_cd_file(TARGET mouhid DESTINATION reactos/system32/drivers FOR all)
