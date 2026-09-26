
spec2def(msgina.dll msgina.spec)

# msgina C->Rust cohab: the 7 hand-written .c are translated to Rust
# (sdk/lib/rust/msgina-rust → libmsgina_rust.a, linked below). The #[no_mangle]
# symbols in the crate are the sole providers of those functions+exports now that
# the C TUs are commented out (conutils-style cohab; no --allow-multiple-definition
# needed since every symbol was removed from the C side). All 7 .c kept on disk
# (never-delete rule). dimmedwindow.cpp is now Rust too (dimmedwindow.rs); the
# generated msgina_stubs.c is the sole remaining C anchor TU.
list(APPEND SOURCE
    # gui.c      — replaced by Rust (sdk/lib/rust/msgina-rust/src/gui.rs)
    # lsa.c      — replaced by Rust (sdk/lib/rust/msgina-rust/src/lsa.rs)
    # msgina.c   — replaced by Rust (sdk/lib/rust/msgina-rust/src/msgina.rs)
    # shutdown.c — replaced by Rust (sdk/lib/rust/msgina-rust/src/shutdown.rs)
    # stubs.c    — replaced by Rust (sdk/lib/rust/msgina-rust/src/stubs.rs)
    # tui.c      — replaced by Rust (sdk/lib/rust/msgina-rust/src/tui.rs)
    # utils.c    — replaced by Rust (sdk/lib/rust/msgina-rust/src/utils.rs)
    #
    # rust_varargs_shim.c is no longer built: strsafe's printf family is strsafe-rust macros
    # that CALL msvcrt's variadic _snwprintf (retired 2026-09-25; file stays on disk, inert).
    )

list(APPEND PCH_SKIP_SOURCE
    # dimmedwindow.cpp — replaced by Rust (sdk/lib/rust/msgina-rust/src/dimmedwindow.rs)
    ${CMAKE_CURRENT_BINARY_DIR}/msgina_stubs.c)

# Builds libmsgina_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml — the user-mode
# no-SSE rule). Inbound Wlx* dispatch ABI from winlogon-sys; outbound Win32/LSA calls
# resolve at link via the importlibs below.
set(MSGINA_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(MSGINA_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-msgina-target" msgina-rust)
set(MSGINA_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmsgina_rust.a")
if(ARCH STREQUAL "i386")
    set(MSGINA_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MSGINA_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MSGINA_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(MSGINA_RUST_SRCS "${MSGINA_RUST_WORKSPACE_DIR}" msgina-rust)
add_custom_command(
    OUTPUT ${MSGINA_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSGINA_RUST_TARGET_DIR}
        cargo build --release --target ${MSGINA_RUST_TARGET}
            -p msgina-rust
            --manifest-path ${MSGINA_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MSGINA_RUST_TARGET_DIR}/${MSGINA_RUST_TARGET}/release/libmsgina_rust.a
        ${MSGINA_RUST_LIB}
    DEPENDS ${MSGINA_RUST_SRCS}
            ${MSGINA_RUST_WORKSPACE_DIR}/msgina-rust/Cargo.toml
    WORKING_DIRECTORY ${MSGINA_RUST_WORKSPACE_DIR}
    COMMENT "Building msgina-rust (${MSGINA_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_msgina DEPENDS ${MSGINA_RUST_LIB})

add_library(msgina MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    msgina.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msgina.def)

set_module_type(msgina win32dll UNICODE)
target_link_libraries(msgina ${MSGINA_RUST_LIB} wine uuid ${PSEH_LIB} cpprt atl_classes)
add_delay_importlibs(msgina secur32)
add_importlibs(msgina advapi32 user32 gdi32 userenv msvcrt kernel32 ntdll)
add_dependencies(msgina rust_msgina)
add_cd_file(TARGET msgina DESTINATION reactos/system32 FOR all)
