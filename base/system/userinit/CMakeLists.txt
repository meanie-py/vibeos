
# FULL flip: both userinit .c files replaced by Rust (userinit-rust).
# userinit.c + livecd.c translated by Opus 4.8. userinit.exe is the post-login
# shell launcher (winlogon → userinit → explorer.exe) — the last hop before the
# desktop. Keep both .c on disk (never delete the C — translation philosophy).
list(APPEND SOURCE
    # livecd.c   — replaced by Rust (userinit-rust/src/livecd.rs)
    # userinit.c — replaced by Rust (userinit-rust/src/userinit.rs)
    userinit_rust_stub.c
    userinit.h)

# --- Rust cohabitation (userinit-rust) ------------------------------------
# Builds libuserinit_rust.a and links it into userinit.exe. wWinMain (the CRT
# entry target) + StartShell/IsLiveCD/RunLiveCD/the dialog procs/… now come
# from the archive. FFI substrate is winsrv-sys (broad user32/gdi32/advapi32
# superset). Mirrors the smss.exe / csrss.exe cohab pattern.
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
set(USERINIT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libuserinit_rust.a")
add_custom_command(
    OUTPUT ${USERINIT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p userinit-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libuserinit_rust.a
        ${USERINIT_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/userinit-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/userinit-rust/src/types.rs
        ${RUST_WORKSPACE_DIR}/userinit-rust/src/preamble.rs
        ${RUST_WORKSPACE_DIR}/userinit-rust/src/userinit.rs
        ${RUST_WORKSPACE_DIR}/userinit-rust/src/livecd.rs
        ${RUST_WORKSPACE_DIR}/userinit-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/winsrv-sys/src/lib.rs
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building userinit-rust (${RUST_TARGET}) for userinit.exe"
    VERBATIM)
add_custom_target(userinit_rust_lib DEPENDS ${USERINIT_RUST_LIB})
# --------------------------------------------------------------------------

add_executable(userinit ${SOURCE} userinit.rc)
target_link_libraries(userinit wine ${USERINIT_RUST_LIB})
add_dependencies(userinit userinit_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(userinit PRIVATE "-Wl,--allow-multiple-definition")
set_module_type(userinit win32gui UNICODE)
add_importlibs(userinit user32 gdi32 advapi32 shell32 msvcrt kernel32 ntdll)
add_pch(userinit userinit.h SOURCE)
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(userinit_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_cd_file(TARGET userinit DESTINATION reactos/system32 FOR all)
