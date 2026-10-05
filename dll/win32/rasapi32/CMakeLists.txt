
spec2def(rasapi32.dll rasapi32.spec ADD_IMPORTLIB)

# rasapi32.dll — Rust cohabitation flip (FULL: the one compiled src *.c → rasapi32-rust).
# The RAS (dial-up) client API. Wine-sourced and almost entirely stubs: 53 of its 58
# functions are `FIXME(); return ERROR_NOT_SUPPORTED;`, all lowered DETERMINISTICALLY for
# $0 — only 5 needed a paid call. rasapi.c is commented out below and kept on disk per
# the never-delete rule.
set(RASAPI32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(RASAPI32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(RASAPI32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(RASAPI32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(RASAPI32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-rasapi32-target" rasapi32-rust)
set(RASAPI32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/librasapi32_rust.a")
rust_crate_sources(RASAPI32_RUST_SRCS "${RASAPI32_RUST_WORKSPACE_DIR}" rasapi32-rust)
add_custom_command(
    OUTPUT ${RASAPI32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RASAPI32_RUST_TARGET_DIR}
        cargo build --release --target ${RASAPI32_RUST_TARGET}
            -p rasapi32-rust
            --manifest-path ${RASAPI32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${RASAPI32_RUST_TARGET_DIR}/${RASAPI32_RUST_TARGET}/release/librasapi32_rust.a
        ${RASAPI32_RUST_LIB}
    DEPENDS ${RASAPI32_RUST_SRCS}
            ${RASAPI32_RUST_WORKSPACE_DIR}/rasapi32-rust/Cargo.toml
    WORKING_DIRECTORY ${RASAPI32_RUST_WORKSPACE_DIR}
    COMMENT "Building rasapi32-rust (${RASAPI32_RUST_TARGET})"
)
add_custom_target(rust_rasapi32 DEPENDS ${RASAPI32_RUST_LIB})

# The only compiled src *.c → rasapi32-rust (src/rasapi32/rasapi.rs), kept on disk per
# the never-delete rule.
#   rasapi.c  → rasapi32-rust
# list(APPEND SOURCE
#     rasapi.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/rasapi32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/rasapi32.def)

add_library(rasapi32 MODULE ${SOURCE})
set_module_type(rasapi32 win32dll)
# librasapi32_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(rasapi32 ${RASAPI32_RUST_LIB} wine)
add_dependencies(rasapi32 rust_rasapi32)
add_importlibs(rasapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET rasapi32 DESTINATION reactos/system32 FOR all)
set_wine_module(rasapi32)
