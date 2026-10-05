
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys/lsass)

add_rpc_files(server
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dssetup.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/lsa.idl)
spec2def(lsasrv.dll lsasrv.spec ADD_IMPORTLIB)

# lsarpc RPC server stub: C->Rust. The widl-generated lsa_s.c is REPLACED by the
# `lsa-stub` crate (sdk/lib/rust/lsa-stub -> liblsa_stub.a, linked below), which
# transliterates the same MIDL static graph to Rust against midl-rt and exports
# `lsarpc_v0_0_s_ifspec` (referenced by lsarpc.c:60). The server routines the
# stub dispatches to (Lsar*) resolve at link from the C bodies below (cohab; they
# flip to Rust file-by-file in later milestones). dssetup_s.c stays C for now.
#
# lsa_s.c is kept on disk (never-delete) and compiled -- but NOT linked -- by the
# `lsasrv_c` OBJECT lib below: that records its exact per-arch compile command in
# compile_commands.json so lsa-stub's build.rs (generate_rpc_stub_rust) can read
# the precise symbol decorations (i386 _Lsar*@N). Mirrors the services_c pattern.
list(APPEND SOURCE
    # authpackage.c — replaced by Rust (lsasrv-rust/src/lsasrv/authpackage.rs)
    # authport.c    — replaced by Rust (lsasrv-rust/src/lsasrv/authport.rs)
    # database.c   — replaced by Rust (lsasrv-rust/src/lsasrv/database.rs)
    # dssetup.c  — replaced by Rust (sdk/lib/rust/lsasrv-rust/src/dssetup.rs)
    # lookup.c      — replaced by Rust (lsasrv-rust/src/lsasrv/lookup.rs)
    # lsarpc.c      — replaced by Rust (lsasrv-rust/src/lsasrv/lsarpc.rs)
    # lsasrv.c      — replaced by Rust (lsasrv-rust/src/lsasrv/lsasrv.rs)
    # notify.c      — replaced by Rust (lsasrv-rust/src/lsasrv/notify.rs)
    # policy.c      — replaced by Rust (lsasrv-rust/src/lsasrv/policy.rs)
    # privileges.c  — replaced by Rust (lsasrv-rust/src/lsasrv/privileges.rs)
    # registry.c     — replaced by Rust (lsasrv-rust/src/lsasrv/registry.rs)
    # security.c     — replaced by Rust (lsasrv-rust/src/lsasrv/security.rs)
    # service.c     — replaced by Rust (lsasrv-rust/src/lsasrv/service.rs)
    # session.c     — replaced by Rust (lsasrv-rust/src/lsasrv/session.rs)
    # srm.c         — replaced by Rust (lsasrv-rust/src/lsasrv/srm.rs)
    # utils.c        — replaced by Rust (lsasrv-rust/src/lsasrv/utils.rs)
    lsasrv.h
    ${CMAKE_CURRENT_BINARY_DIR}/dssetup_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/lsasrv_stubs.c)

# Decoration-only OBJECT lib: compiles lsa_s.c (flag-exact, UNICODE like the
# module) so generate_rpc_stub_rust can replay the cc command. Never linked.
add_library(lsasrv_c OBJECT ${CMAKE_CURRENT_BINARY_DIR}/lsa_s.c lsasrv.h)
target_compile_definitions(lsasrv_c PRIVATE UNICODE _UNICODE)
add_pch(lsasrv_c lsasrv.h "")

# --- Rust cohabitation (lsa-stub) -----------------------------------------
# Build liblsa_stub.a from the workspace crate (STANDARD user-mode per-arch
# gnu targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml). The
# crate's build.rs runs widl + reads decorations from build-<arch> -- so it
# DEPENDS on native-widl (the host tool) and lsasrv_c (forces the rpc files +
# compile_commands entry to exist).
set(LSA_STUB_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(LSA_STUB_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-lsa-stub-target" lsa-stub)
set(LSA_STUB_LIB "${CMAKE_CURRENT_BINARY_DIR}/liblsa_stub.a")
if(ARCH STREQUAL "i386")
    set(LSA_STUB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(LSA_STUB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(LSA_STUB_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
file(GLOB_RECURSE LSA_STUB_SRCS "${LSA_STUB_WORKSPACE_DIR}/lsa-stub/src/*.rs")
add_custom_command(
    OUTPUT ${LSA_STUB_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${LSA_STUB_TARGET_DIR}
        cargo build --release --target ${LSA_STUB_RUST_TARGET}
            -p lsa-stub
            --manifest-path ${LSA_STUB_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LSA_STUB_TARGET_DIR}/${LSA_STUB_RUST_TARGET}/release/liblsa_stub.a
        ${LSA_STUB_LIB}
    DEPENDS ${LSA_STUB_SRCS}
            ${LSA_STUB_WORKSPACE_DIR}/lsa-stub/Cargo.toml
            ${LSA_STUB_WORKSPACE_DIR}/lsa-stub/build.rs
            native-widl
            lsasrv_c
    WORKING_DIRECTORY ${LSA_STUB_WORKSPACE_DIR}
    COMMENT "Building lsa-stub (${LSA_STUB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_lsa_stub DEPENDS ${LSA_STUB_LIB})

# Build liblsasrv_rust.a — the translated lsasrv .c bodies (cohab; dssetup.rs is
# the first). #[no_mangle] symbols take over from the commented-out C TUs. FFI
# substrate = lsasrv-sys (bindgen, builds transitively).
rust_storage_target(LSASRV_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-lsasrv-target" lsasrv-rust)
set(LSASRV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/liblsasrv_rust.a")
rust_crate_sources(LSASRV_RUST_SRCS "${LSA_STUB_WORKSPACE_DIR}" lsasrv-rust)
add_custom_command(
    OUTPUT ${LSASRV_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${LSASRV_RUST_TARGET_DIR}
        cargo build --release --target ${LSA_STUB_RUST_TARGET}
            -p lsasrv-rust
            --manifest-path ${LSA_STUB_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LSASRV_RUST_TARGET_DIR}/${LSA_STUB_RUST_TARGET}/release/liblsasrv_rust.a
        ${LSASRV_RUST_LIB}
    DEPENDS ${LSASRV_RUST_SRCS}
            ${LSA_STUB_WORKSPACE_DIR}/lsasrv-rust/Cargo.toml
            ${LSA_STUB_WORKSPACE_DIR}/lsasrv-sys/Cargo.toml
    WORKING_DIRECTORY ${LSA_STUB_WORKSPACE_DIR}
    COMMENT "Building lsasrv-rust (${LSA_STUB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_lsasrv DEPENDS ${LSASRV_RUST_LIB})
# --------------------------------------------------------------------------

add_library(lsasrv MODULE
    ${SOURCE}
    lsasrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/lsasrv.def)

set_module_type(lsasrv win32dll UNICODE)
# lsa-stub is a no_std staticlib carrying its own panic_handler/eh_personality;
# link_guard allows that panic-runtime twin (the stub never panics).
target_link_libraries(lsasrv ${LSASRV_RUST_LIB} ${LSA_STUB_LIB} wine ${PSEH_LIB})
add_importlibs(lsasrv samsrv rpcrt4 msvcrt kernel32 advapi32 ntdll)
add_pch(lsasrv lsasrv.h SOURCE)
add_dependencies(lsasrv rust_lsa_stub rust_lsasrv)
add_cd_file(TARGET lsasrv DESTINATION reactos/system32 FOR all)
