
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/svcctl.idl)

# services.exe (SCM) Rust cohabitation flip (services-rust), wave by wave.
# Each flipped .c is commented out below; libservices_rust.a provides the
# symbols (lld first-match via --allow-multiple-definition). The .c stay on
# disk (never delete the C — translation philosophy). services_rust_stub.c
# keeps a TU always compiled so the linker extracts the archive members.
list(APPEND SOURCE
    # config.c     — replaced by Rust (services-rust/src/config.rs)
    # controlset.c — replaced by Rust (services-rust/src/controlset.rs)
    # database.c   — replaced by Rust (services-rust/src/database.rs)
    # driver.c     — replaced by Rust (services-rust/src/driver.rs)
    # groupdb.c    — replaced by Rust (services-rust/src/groupdb.rs)
    # lock.c       — replaced by Rust (services-rust/src/lock.rs)
    services_rust_stub.c
    # rpcserver.c  — replaced by Rust (services-rust/src/rpcserver.rs)
    # security.c   — replaced by Rust (services-rust/src/security.rs)
    services.c
    services.h
    ${CMAKE_CURRENT_BINARY_DIR}/svcctl_s.c)

# --- Rust cohabitation (services-rust) ------------------------------------
# Builds libservices_rust.a and links it into services.exe. FFI substrate is
# winsrv-sys (kernel32 Heap + Win32 base + error consts over ntdll-sys);
# services-private + svcctl RPC types live in services-rust/src/types.rs; the
# internal Scm* database-lock helpers stay C (database.c). Mirrors winsrv /
# lsass cohab.
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
set(SERVICES_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libservices_rust.a")
add_custom_command(
    OUTPUT ${SERVICES_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p services-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libservices_rust.a
        ${SERVICES_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/services-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/types.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/lock.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/controlset.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/security.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/groupdb.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/driver.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/config.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/database.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/rpcserver.rs
        ${RUST_WORKSPACE_DIR}/services-rust/src/svcctl.rs
        ${RUST_WORKSPACE_DIR}/services-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/winsrv-sys/src/lib.rs
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building services-rust (${RUST_TARGET}) for services.exe"
    VERBATIM)
add_custom_target(services_rust_lib DEPENDS ${SERVICES_RUST_LIB})
# --------------------------------------------------------------------------

add_executable(services ${SOURCE} services.rc)
target_link_libraries(services ${PSEH_LIB} ${SERVICES_RUST_LIB})
add_dependencies(services services_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(services PRIVATE "-Wl,--allow-multiple-definition")

set_module_type(services win32gui UNICODE)
add_importlibs(services userenv user32 advapi32 advapi32_vista rpcrt4 msvcrt kernel32 ntdll)
add_pch(services services.h SOURCE)
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(services_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_cd_file(TARGET services DESTINATION reactos/system32 FOR all)
