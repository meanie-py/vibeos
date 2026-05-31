
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

spec2def(csrsrv.dll csrsrv.spec ADD_IMPORTLIB)

# --- Rust cohabitation (smss-csrss-rust) ----------------------------
# All 7 csrsrv .c are commented out of add_library() (a direct .obj
# defining a spec export would block lld from pulling the archive —
# --allow-multiple-definition does NOT force archive extraction). The
# Rust staticlib provides the 35 csrsrv.spec exports; csrsrv_rust_stub.c
# keeps add_library(MODULE) valid. The .c stay on disk (never delete
# the C). Mirrors the csrss.exe cohabitation + the sdk/lib/rtl
# cmake-driven cargo→.a pattern.
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
set(SMSS_CSRSS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsmss_csrss_rust.a")
# csrsrv.dll keeps the kitchen-sink default-features build (which now
# resolves to csrsrv_dll + csrlib + smlib + ntddi_ws03 per Cargo.toml,
# the same union as before the per-binary feature refactor). Changing
# this changes cgu partitioning and risks
# feedback_workspace_member_cgu_shift / captures(none)-ud2 mode bugs,
# so we leave it alone for now.
add_custom_command(
    OUTPUT ${SMSS_CSRSS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p smss-csrss-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libsmss_csrss_rust.a
        ${SMSS_CSRSS_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/api.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/init.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/procsup.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/server.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/session.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/thredsup.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/wait.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building smss-csrss-rust (${RUST_TARGET}) for csrsrv.dll"
    VERBATIM)
add_custom_target(csrsrv_rust_lib DEPENDS ${SMSS_CSRSS_RUST_LIB})
# --------------------------------------------------------------------

list(APPEND SOURCE
    csrsrv_rust_stub.c
    # api.c       — replaced by Rust (smss-csrss-rust/src/csrsrv/api.rs)
    # init.c      — replaced by Rust (smss-csrss-rust/src/csrsrv/init.rs)
    # procsup.c   — replaced by Rust (smss-csrss-rust/src/csrsrv/procsup.rs)
    # server.c    — replaced by Rust (smss-csrss-rust/src/csrsrv/server.rs)
    # session.c   — replaced by Rust (smss-csrss-rust/src/csrsrv/session.rs)
    # thredsup.c  — replaced by Rust (smss-csrss-rust/src/csrsrv/thredsup.rs)
    # wait.c      — replaced by Rust (smss-csrss-rust/src/csrsrv/wait.rs)
    srv.h)

add_library(csrsrv MODULE
    ${SOURCE}
    csrsrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/csrsrv.def)

set_module_type(csrsrv nativedll)
# smlib (C) is still built (its cmake flip is deferred to Phase 3 smss,
# which also links the .a); the Rust .a also carries Rust smlib —
# --allow-multiple-definition covers the overlap (lld first-match).
target_link_libraries(csrsrv smlib ${PSEH_LIB} ${SMSS_CSRSS_RUST_LIB})
add_dependencies(csrsrv csrsrv_rust_lib)
target_link_options(csrsrv PRIVATE "-Wl,--allow-multiple-definition")
add_importlibs(csrsrv ntdll)
add_pch(csrsrv srv.h SOURCE)
add_dependencies(csrsrv psdk bugcodes)
add_cd_file(TARGET csrsrv DESTINATION reactos/system32 FOR all)
