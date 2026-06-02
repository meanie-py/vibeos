
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/winreg.idl)

# PARTIAL Rust flip: GINA-independent leaf files move to Rust one at a
# time (winlogon-rust). The GINA core (wlx.c), SAS state machine (sas.c),
# main (winlogon.c) and RPC (rpcserver.c) stay C. Flipped .c stay on disk
# (never delete the C — translation philosophy), commented out here.
list(APPEND SOURCE
    # environment.c — replaced by Rust (winlogon-rust/src/environment.rs)
    notify.c
    rpcserver.c
    sas.c
    # screensaver.c — replaced by Rust (winlogon-rust/src/screensaver.rs)
    # security.c — replaced by Rust (winlogon-rust/src/security.rs)
    # setup.c    — replaced by Rust (winlogon-rust/src/setup.rs)
    winlogon_rust_stub.c
    # shutdown.c — replaced by Rust (winlogon-rust/src/shutdown.rs)
    winlogon.c
    wlx.c
    winlogon.h
    ${CMAKE_CURRENT_BINARY_DIR}/winreg_s.c)

# --- Rust cohabitation (winlogon-rust) ------------------------------------
# Builds libwinlogon_rust.a and links it into winlogon.exe. The flipped
# leaf functions (GetSetupType/RunSetup) come from the archive; the still-C
# core calls them and owns the shared WLSession global the leaves read.
# FFI substrate is the self-contained winlogon-sys (Win32 + GINA + ndk +
# the WLSESSION/GINA layout, size-asserted x3). Mirrors userinit.exe.
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
endif()
set(WINLOGON_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwinlogon_rust.a")
add_custom_command(
    OUTPUT ${WINLOGON_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p winlogon-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libwinlogon_rust.a
        ${WINLOGON_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/preamble.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/setup.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/environment.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/shutdown.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/screensaver.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/src/security.rs
        ${RUST_WORKSPACE_DIR}/winlogon-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/winlogon-sys/src/lib.rs
    COMMENT "Building winlogon-rust (${RUST_TARGET}) for winlogon.exe"
    VERBATIM)
add_custom_target(winlogon_rust_lib DEPENDS ${WINLOGON_RUST_LIB})
# --------------------------------------------------------------------------

add_rc_deps(winlogon.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/winlogon.ico)
add_executable(winlogon ${SOURCE} winlogon.rc)
target_link_libraries(winlogon wine ${PSEH_LIB} ${WINLOGON_RUST_LIB})
add_dependencies(winlogon winlogon_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(winlogon PRIVATE "-Wl,--allow-multiple-definition")
set_module_type(winlogon win32gui)
add_importlibs(winlogon gdi32 user32 advapi32 userenv secur32 rpcrt4 mpr msvcrt kernel32 ntdll)
add_pch(winlogon winlogon.h SOURCE)
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(winlogon_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_cd_file(TARGET winlogon DESTINATION reactos/system32 FOR all)
