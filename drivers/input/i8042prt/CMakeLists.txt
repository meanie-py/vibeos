
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/dmilib)

# ============================================================================
# Rust i8042prt PS/2 port driver
# (sdk/lib/rust/i8042prt-rust -> libi8042prt_rust.a)
# ============================================================================
# All 10 C sources are translated to Rust. Their #[no_mangle] DriverEntry and
# internal functions replace the C (commented out of SOURCE below; the .c files
# are kept on disk per the never-delete-a-translation rule). guid.c stays C
# (poclass GUID definitions only). Mirrors arm64input-rust / hal-rust
# cohabitation: one staticlib linked into the cmake-built kernelmodedriver,
# per-arch RUST_TARGET.

set(I8042PRT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(I8042PRT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-i8042prt-target")
set(I8042PRT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libi8042prt_rust.a")

if(ARCH STREQUAL "i386")
    set(I8042PRT_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(I8042PRT_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(I8042PRT_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()

add_custom_command(
    OUTPUT ${I8042PRT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${I8042PRT_RUST_TARGET_DIR}
        cargo build --release --target ${I8042PRT_RUST_TARGET}
            -p i8042prt-rust
            --manifest-path ${I8042PRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${I8042PRT_RUST_TARGET_DIR}/${I8042PRT_RUST_TARGET}/release/libi8042prt_rust.a
        ${I8042PRT_RUST_LIB}
    DEPENDS ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/Cargo.toml
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/lib.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/i8042prt_h.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/createclose.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/hwhacks.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/i8042prt.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/keyboard.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/misc.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/mouse.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/pnp.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/ps2pp.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/readwrite.rs
            ${I8042PRT_RUST_WORKSPACE_DIR}/i8042prt-rust/src/registry.rs
    WORKING_DIRECTORY ${I8042PRT_RUST_WORKSPACE_DIR}
    COMMENT "Building i8042prt-rust (${I8042PRT_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_i8042prt DEPENDS ${I8042PRT_RUST_LIB})

# C sources replaced by Rust (sdk/lib/rust/i8042prt-rust/src/*.rs):
#   createclose.c hwhacks.c i8042prt.c keyboard.c misc.c
#   mouse.c pnp.c ps2pp.c readwrite.c registry.c
# (kept on disk per the never-delete-a-translation rule).
# guid.c stays C — it only emits the poclass GUIDs via initguid.h.
add_library(i8042prt MODULE
    rust_cohab_stub.c
    guid.c
    i8042prt.rc)

set_module_type(i8042prt kernelmodedriver)
target_link_libraries(i8042prt dmilib ${I8042PRT_RUST_LIB})
add_importlibs(i8042prt ntoskrnl hal)
add_dependencies(i8042prt rust_i8042prt)
add_cd_file(TARGET i8042prt DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(i8042prt_reg.inf)
