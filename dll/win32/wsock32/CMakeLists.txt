
spec2def(wsock32.dll wsock32.spec)

# wsock32.dll — Rust cohabitation flip (FULL: the single compiled src *.c → wsock32-rust).
# Nearly all of wsock32 is `.spec` forwarders to mswsock/ws2_32 that spec2def turns into
# thunks; the only real C was a DllMain returning TRUE, which rustconv lowered
# deterministically (zero paid API calls). stubs.c is commented out below and kept on disk
# per the never-delete rule.
set(WSOCK32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WSOCK32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WSOCK32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WSOCK32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WSOCK32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wsock32-target" wsock32-rust)
set(WSOCK32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwsock32_rust.a")
rust_crate_sources(WSOCK32_RUST_SRCS "${WSOCK32_RUST_WORKSPACE_DIR}" wsock32-rust)
add_custom_command(
    OUTPUT ${WSOCK32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WSOCK32_RUST_TARGET_DIR}
        cargo build --release --target ${WSOCK32_RUST_TARGET}
            -p wsock32-rust
            --manifest-path ${WSOCK32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WSOCK32_RUST_TARGET_DIR}/${WSOCK32_RUST_TARGET}/release/libwsock32_rust.a
        ${WSOCK32_RUST_LIB}
    DEPENDS ${WSOCK32_RUST_SRCS}
            ${WSOCK32_RUST_WORKSPACE_DIR}/wsock32-rust/Cargo.toml
    WORKING_DIRECTORY ${WSOCK32_RUST_WORKSPACE_DIR}
    COMMENT "Building wsock32-rust (${WSOCK32_RUST_TARGET})"
)
add_custom_target(rust_wsock32 DEPENDS ${WSOCK32_RUST_LIB})

# The only compiled src *.c → wsock32-rust (src/wsock32/stubs.rs).
#   stubs.c  → wsock32-rust
add_library(wsock32 MODULE
    # stubs.c
    wsock32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wsock32.def)
set_module_type(wsock32 win32dll UNICODE)
# libwsock32_rust.a FIRST so its #[no_mangle] DllMain wins.
target_link_libraries(wsock32 ${WSOCK32_RUST_LIB})
add_dependencies(wsock32 rust_wsock32)
add_importlibs(wsock32 mswsock ws2_32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wsock32 DESTINATION reactos/system32 FOR all)
