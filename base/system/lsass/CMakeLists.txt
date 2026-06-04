
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# FULL flip: lsass.c replaced by Rust (lsass-rust). lsass.exe is the Local
# Security Authority Subsystem host on the security boot spine (smss/wininit →
# lsass → lsasrv/samsrv init; winlogon → LsaLogonUser authenticates against it).
# Keep lsass.c on disk (never delete the C — translation philosophy).
# lsass_rust_stub.c keeps the static-lib link anchored (CMake needs a TU it
# always compiles so the linker extracts the wWinMain archive member).
list(APPEND SOURCE
    # lsass.c — replaced by Rust (lsass-rust/src/lsass.rs)
    lsass_rust_stub.c)

# --- Rust cohabitation (lsass-rust) ---------------------------------------
# Builds liblsass_rust.a (wWinMain) and links it into lsass.exe. FFI substrate
# is ntdll-sys (Nt*/Rtl*); LsapInitLsa/ServiceInit (lsasrv) + SamIInitialize
# (samsrv) resolve against the import libs below. Mirrors userinit / csrss cohab.
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
set(LSASS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/liblsass_rust.a")
add_custom_command(
    OUTPUT ${LSASS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p lsass-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/liblsass_rust.a
        ${LSASS_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/lsass-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/lsass-rust/src/lsass.rs
        ${RUST_WORKSPACE_DIR}/lsass-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building lsass-rust (${RUST_TARGET}) for lsass.exe"
    VERBATIM)
add_custom_target(lsass_rust_lib DEPENDS ${LSASS_RUST_LIB})
# --------------------------------------------------------------------------

add_executable(lsass ${SOURCE} lsass.rc)
target_link_libraries(lsass ${LSASS_RUST_LIB})
add_dependencies(lsass lsass_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(lsass PRIVATE "-Wl,--allow-multiple-definition")
set_module_type(lsass win32gui UNICODE)
add_importlibs(lsass advapi32 lsasrv samsrv msvcrt kernel32 ntdll)
# The Rust-cohab stub is self-contained.
set_source_files_properties(lsass_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_cd_file(TARGET lsass DESTINATION reactos/system32 FOR all)
