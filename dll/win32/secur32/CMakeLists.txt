
add_definitions(-D__SECUR32__)
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)
spec2def(secur32.dll secur32.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # dllmain.c — replaced by Rust (secur32-rust)
    # lsalpc.c — replaced by Rust (secur32-rust)
    # sspi.c — replaced by Rust (secur32-rust)
    # stubs.c — replaced by Rust (secur32-rust)
    # thunks.c — replaced by Rust (secur32-rust)
    # wrapper.c — replaced by Rust (secur32-rust)
    # wine/base64_codec.c — replaced by Rust (secur32-rust)
    # wine/dispatcher.c — replaced by Rust (secur32-rust)
    # wine/hmac_md5.c — replaced by Rust (secur32-rust)
    # wine/kerberos.c — replaced by Rust (secur32-rust)
    # wine/negotiate.c — replaced by Rust (secur32-rust)
    # wine/ntlm.c — replaced by Rust (secur32-rust)
    # wine/util.c — replaced by Rust (secur32-rust)
    precomp.h)

# --- Rust cohabitation (secur32-rust) -------------------------------------
# Build libsecur32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml). The
# #[no_mangle] bodies replace the commented-out .c TUs above; lld first-match
# (--allow-multiple-definition) lets the Rust staticlib's panic_handler/
# eh_personality coexist with whatever the C cohab pulls in.
set(SECUR32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(SECUR32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-secur32-target" secur32-rust)
set(SECUR32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsecur32_rust.a")
if(ARCH STREQUAL "i386")
    set(SECUR32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SECUR32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SECUR32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(SECUR32_RUST_SRCS "${SECUR32_RUST_WORKSPACE_DIR}" secur32-rust)
add_custom_command(
    OUTPUT ${SECUR32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SECUR32_RUST_TARGET_DIR}
        cargo build --release --target ${SECUR32_RUST_TARGET}
            -p secur32-rust
            --manifest-path ${SECUR32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SECUR32_RUST_TARGET_DIR}/${SECUR32_RUST_TARGET}/release/libsecur32_rust.a
        ${SECUR32_RUST_LIB}
    DEPENDS ${SECUR32_RUST_SRCS}
            ${SECUR32_RUST_WORKSPACE_DIR}/secur32-rust/Cargo.toml
    WORKING_DIRECTORY ${SECUR32_RUST_WORKSPACE_DIR}
    COMMENT "Building secur32-rust (${SECUR32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_secur32 DEPENDS ${SECUR32_RUST_LIB})
# --------------------------------------------------------------------------

add_library(secur32 MODULE
    ${SOURCE}
    secur32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/secur32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/secur32.def)

set_module_type(secur32 win32dll UNICODE)
# SECUR32_RUST_LIB first so its #[no_mangle] bodies win over any lingering C.
target_link_libraries(secur32 ${SECUR32_RUST_LIB} lsalib wine)
add_importlibs(secur32 advapi32 netapi32 msvcrt kernel32 ntdll)
add_pch(secur32 precomp.h SOURCE)
add_dependencies(secur32 rust_secur32)
add_cd_file(TARGET secur32 DESTINATION reactos/system32 FOR all)
