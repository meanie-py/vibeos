
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/sam.idl)
spec2def(samsrv.dll samsrv.spec ADD_IMPORTLIB)

# samr RPC server stub: C->Rust. The widl-generated sam_s.c is REPLACED by the
# `sam-stub` crate (sdk/lib/rust/sam-stub -> libsam_stub.a, linked below), which
# transliterates the same MIDL static graph to Rust against midl-rt and exports
# `samr_v1_0_s_ifspec`. The server routines the stub dispatches to (Samr*)
# resolve at link from the samsrv bodies below (cohab; they flip to Rust
# file-by-file).
#
# sam_s.c is kept on disk (never-delete) and compiled -- but NOT linked -- by the
# `samsrv_c` OBJECT lib below: that records its exact per-arch compile command in
# compile_commands.json so sam-stub's build.rs (generate_rpc_stub_rust) can read
# the precise symbol decorations (i386 _Samr*@N). Mirrors the lsasrv_c pattern.
list(APPEND SOURCE
    alias.c
    database.c
    display.c
    domain.c
    group.c
    registry.c
    samrpc.c
    samsrv.c
    security.c
    setup.c
    user.c
    utils.c
    samsrv.h
    ${CMAKE_CURRENT_BINARY_DIR}/samsrv_stubs.c)

# Decoration-only OBJECT lib: compiles sam_s.c (flag-exact, UNICODE like the
# module) so generate_rpc_stub_rust can replay the cc command. Never linked.
add_library(samsrv_c OBJECT ${CMAKE_CURRENT_BINARY_DIR}/sam_s.c samsrv.h)
target_compile_definitions(samsrv_c PRIVATE UNICODE _UNICODE)
add_pch(samsrv_c samsrv.h "")

# --- Rust cohabitation (sam-stub) -----------------------------------------
# Build libsam_stub.a from the workspace crate (STANDARD user-mode per-arch
# gnu targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml). The
# crate's build.rs runs widl + reads decorations from build-<arch> -- so it
# DEPENDS on native-widl (the host tool) and samsrv_c (forces the rpc files +
# compile_commands entry to exist).
set(SAM_STUB_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(SAM_STUB_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-sam-stub-target")
set(SAM_STUB_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsam_stub.a")
if(ARCH STREQUAL "i386")
    set(SAM_STUB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SAM_STUB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SAM_STUB_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
file(GLOB_RECURSE SAM_STUB_SRCS "${SAM_STUB_WORKSPACE_DIR}/sam-stub/src/*.rs")
add_custom_command(
    OUTPUT ${SAM_STUB_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SAM_STUB_TARGET_DIR}
        cargo build --release --target ${SAM_STUB_RUST_TARGET}
            -p sam-stub
            --manifest-path ${SAM_STUB_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SAM_STUB_TARGET_DIR}/${SAM_STUB_RUST_TARGET}/release/libsam_stub.a
        ${SAM_STUB_LIB}
    DEPENDS ${SAM_STUB_SRCS}
            ${SAM_STUB_WORKSPACE_DIR}/sam-stub/Cargo.toml
            ${SAM_STUB_WORKSPACE_DIR}/sam-stub/build.rs
            native-widl
            samsrv_c
    WORKING_DIRECTORY ${SAM_STUB_WORKSPACE_DIR}
    COMMENT "Building sam-stub (${SAM_STUB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_sam_stub DEPENDS ${SAM_STUB_LIB})

# Build libsamsrv_rust.a — the translated samsrv .c bodies (cohab). #[no_mangle]
# symbols take over from the commented-out C TUs. FFI substrate = samsrv-sys +
# lsasrv-sys + ntdll-sys (built transitively).
set(SAMSRV_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-samsrv-target")
set(SAMSRV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsamsrv_rust.a")
file(GLOB_RECURSE SAMSRV_RUST_SRCS
    "${SAM_STUB_WORKSPACE_DIR}/samsrv-rust/src/*.rs"
    "${SAM_STUB_WORKSPACE_DIR}/samsrv-sys/src/*.rs")
add_custom_command(
    OUTPUT ${SAMSRV_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SAMSRV_RUST_TARGET_DIR}
        cargo build --release --target ${SAM_STUB_RUST_TARGET}
            -p samsrv-rust
            --manifest-path ${SAM_STUB_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SAMSRV_RUST_TARGET_DIR}/${SAM_STUB_RUST_TARGET}/release/libsamsrv_rust.a
        ${SAMSRV_RUST_LIB}
    DEPENDS ${SAMSRV_RUST_SRCS}
            ${SAM_STUB_WORKSPACE_DIR}/samsrv-rust/Cargo.toml
            ${SAM_STUB_WORKSPACE_DIR}/samsrv-sys/Cargo.toml
    WORKING_DIRECTORY ${SAM_STUB_WORKSPACE_DIR}
    COMMENT "Building samsrv-rust (${SAM_STUB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_samsrv DEPENDS ${SAMSRV_RUST_LIB})
# --------------------------------------------------------------------------

add_library(samsrv MODULE
    ${SOURCE}
    samsrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/samsrv.def)

set_module_type(samsrv win32dll UNICODE)
# --allow-multiple-definition: sam-stub is a no_std staticlib carrying its own
# panic_handler/eh_personality (lld first-match wins; the stub never panics).
target_link_libraries(samsrv ${SAMSRV_RUST_LIB} ${SAM_STUB_LIB} wine ${PSEH_LIB})
target_link_options(samsrv PRIVATE LINKER:--allow-multiple-definition)
add_importlibs(samsrv rpcrt4 advapi32 msvcrt kernel32 ntdll)
add_pch(samsrv samsrv.h SOURCE)
add_dependencies(samsrv rust_sam_stub rust_samsrv)
add_cd_file(TARGET samsrv DESTINATION reactos/system32 FOR all)
