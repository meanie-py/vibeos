
spec2def(kdcom.dll kdrosdbg.spec ADD_IMPORTLIB)

# ============================================================================
# Rust kdcom (kdrosdbg.c -> sdk/lib/rust/kdcom-rust -> libkdcom_rust.a)
# ============================================================================
# On the non-MSVC build, _WINKD_=FALSE builds kdcom.dll from the 116-LOC
# kdrosdbg.c stub. It is translated to Rust (kdcom-rust); the C is commented
# out of SOURCE below (kept on disk per the never-delete rule). The 8
# kdrosdbg.spec exports come from libkdcom_rust.a, linked per-arch. Mirrors
# the i8042prt-rust / framebuf-rust / arm64vid-rust cohabitation.

set(KDCOM_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(KDCOM_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kdcom-target")
set(KDCOM_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkdcom_rust.a")

if(ARCH STREQUAL "i386")
    set(KDCOM_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(KDCOM_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(KDCOM_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()

add_custom_command(
    OUTPUT ${KDCOM_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KDCOM_RUST_TARGET_DIR}
        cargo build --release --target ${KDCOM_RUST_TARGET}
            -p kdcom-rust
            --manifest-path ${KDCOM_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KDCOM_RUST_TARGET_DIR}/${KDCOM_RUST_TARGET}/release/libkdcom_rust.a
        ${KDCOM_RUST_LIB}
    DEPENDS ${KDCOM_RUST_WORKSPACE_DIR}/kdcom-rust/Cargo.toml
            ${KDCOM_RUST_WORKSPACE_DIR}/kdcom-rust/src/lib.rs
            ${KDCOM_RUST_WORKSPACE_DIR}/kdcom-rust/src/kdrosdbg.rs
    WORKING_DIRECTORY ${KDCOM_RUST_WORKSPACE_DIR}
    COMMENT "Building kdcom-rust (${KDCOM_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_kdcom DEPENDS ${KDCOM_RUST_LIB})

# kdrosdbg.c replaced by Rust (sdk/lib/rust/kdcom-rust/src/kdrosdbg.rs); kept
# on disk. rust_cohab_stub.c is the placeholder TU for the MODULE.
list(APPEND SOURCE
    rust_cohab_stub.c
    kdrosdbg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/kdcom.def)

add_library(kdcom MODULE ${SOURCE})
set_module_type(kdcom kerneldll ENTRYPOINT 0)
add_importlibs(kdcom ntoskrnl hal)
target_link_libraries(kdcom cportlib ${KDCOM_RUST_LIB})
add_dependencies(kdcom psdk rust_kdcom)
add_cd_file(TARGET kdcom DESTINATION reactos/system32 NO_CAB FOR all)
