
# fdo.c / hidclass.c / pdo.c / guid.c replaced by Rust (sdk/lib/rust/hidclass-rust) and kept
# on disk per the never-delete rule. The HID FDO/PDO class-driver body — DllInitialize,
# HidRegisterMinidriver (exported, hidusb links it), per-collection PDO creation, the FDO/PDO
# PnP descriptor-fetch state machine, the IO_WORKITEM read-completion path — comes from
# libhidclass_rust.a. spec2def still generates hidclass.def (the .spec exports, satisfied by
# the Rust #[no_mangle] symbols) + the import lib hidusb consumes. Io/Ke/Ex/Mm/Rtl stay
# ntoskrnl imports; HidP_* stay hidparse imports. ENTRYPOINT 0 (DLL-style, no DriverEntry).
# Not boot-loaded in the QEMU smoke path (PS/2 input) — bar is link-clean x3 + no regression.

spec2def(hidclass.sys hidclass.spec ADD_IMPORTLIB)

set(HIDCLASS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(HIDCLASS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-hidclass-target")
set(HIDCLASS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libhidclass_rust.a")

if(ARCH STREQUAL "i386")
    set(HIDCLASS_RUST_TARGET "i686-pc-windows-gnu")
    set(HIDCLASS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(HIDCLASS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(HIDCLASS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(HIDCLASS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(HIDCLASS_RUST_RUSTFLAGS "")
endif()

rust_crate_sources(HIDCLASS_RUST_SRCS "${HIDCLASS_RUST_WORKSPACE_DIR}" hidclass-rust)

add_custom_command(
    OUTPUT ${HIDCLASS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${HIDCLASS_RUST_TARGET_DIR}
        RUSTFLAGS=${HIDCLASS_RUST_RUSTFLAGS}
        cargo build --release --target ${HIDCLASS_RUST_TARGET}
            -p hidclass-rust
            --manifest-path ${HIDCLASS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${HIDCLASS_RUST_TARGET_DIR}/${HIDCLASS_RUST_TARGET}/release/libhidclass_rust.a
        ${HIDCLASS_RUST_LIB}
    DEPENDS ${HIDCLASS_RUST_SRCS} ${HIDCLASS_RUST_WORKSPACE_DIR}/hidclass-rust/Cargo.toml
    WORKING_DIRECTORY ${HIDCLASS_RUST_WORKSPACE_DIR}
    COMMENT "Building hidclass-rust (${HIDCLASS_RUST_TARGET})"
    VERBATIM)

add_custom_target(rust_hidclass DEPENDS ${HIDCLASS_RUST_LIB})

# replaced by Rust (sdk/lib/rust/hidclass-rust): fdo.c hidclass.c pdo.c guid.c
add_library(hidclass MODULE
    rust_cohab_stub.c
    hidclass.rc
    ${CMAKE_CURRENT_BINARY_DIR}/hidclass.def)

set_module_type(hidclass kernelmodedriver ENTRYPOINT 0)
add_importlibs(hidclass ntoskrnl hidparse hal)
add_dependencies(hidclass rust_hidclass)
target_link_libraries(hidclass ${HIDCLASS_RUST_LIB})
add_cd_file(TARGET hidclass DESTINATION reactos/system32/drivers NO_CAB FOR all)
