
include_directories(
    ${REACTOS_SOURCE_DIR}/subsystems/win32/csrsrv/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# --- Rust cohabitation (smss-csrss-rust) ----------------------------
# csrss.c is commented out of add_executable() (a single direct .obj
# defining `__main` would block lld from ever pulling the archive —
# --allow-multiple-definition does NOT force archive extraction). The
# Rust staticlib provides `__main`/`CsrpSetDefaultProcessHardErrorMode`;
# csrss_rust_stub.c keeps add_executable() valid. csrss.c stays on disk
# (never delete the C). Mirrors the
# sdk/lib/rtl cmake-driven cargo→.a pattern + the hal-rust/ntoskrnl
# cohabitation. See docs/engineering/smss-csrss-translation-audit.md.
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
# csrss.exe builds smss-csrss-rust with `--no-default-features
# --features csrss_main_only,ntddi_ws03`. This excludes the csrsrv /
# csrlib / smlib / smss / basesrv modules so csrss.exe's archive
# carries ONLY `_main` + `CsrpSetDefaultProcessHardErrorMode`. If we
# linked the full archive, csrss.exe would get its own static copy of
# `_CsrApiPort`/`_CsrApiPortName`/etc. via `#[no_mangle]` definitions
# in csrsrv/init.rs — those would *shadow* the imports from csrsrv.dll
# and break cross-DLL data sharing (winsrv → CsrQueryApiPort →
# csrsrv.dll's `_CsrApiPort` reads 0 because csrss.exe wrote to its
# OWN copy at NtCreatePort time). Keep this lean shim.
add_custom_command(
    OUTPUT ${SMSS_CSRSS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p smss-csrss-rust
            --no-default-features
            --features csrss_main_only,ntddi_ws03
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libsmss_csrss_rust.a
        ${SMSS_CSRSS_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrss.rs
        # csrss.exe statically links the whole smss-csrss-rust archive
        # (the csrsrv stem ships inside csrss.exe AND csrsrv.dll, via
        # two independent cargo target dirs). The crate's whole-tree
        # source tracking lives below — anything touched by a rebuild
        # of -p smss-csrss-rust must invalidate the csrss.exe link.
        # See feedback_rustconv_main_repo_leak (different leak) for
        # the parallel cmake-DEPENDS surprise mode.
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/api.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/init.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/procsup.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/server.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/session.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/thredsup.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/csrsrv/wait.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/inlines.rs
    COMMENT "Building smss-csrss-rust (${RUST_TARGET})"
    VERBATIM)
add_custom_target(smss_csrss_rust_lib DEPENDS ${SMSS_CSRSS_RUST_LIB})
# --------------------------------------------------------------------

# csrss.c — replaced by Rust (sdk/lib/rust/smss-csrss-rust/src/csrss.rs)
add_executable(csrss csrss_rust_stub.c csrss.rc)
set_module_type(csrss nativecui)
target_link_libraries(csrss nt ${SMSS_CSRSS_RUST_LIB})
add_dependencies(csrss smss_csrss_rust_lib)
# Rust .a also pulls libcore/compiler-builtins objects; keep
# --allow-multiple-definition for any intrinsic overlap with `nt`.
target_link_options(csrss PRIVATE "-Wl,--allow-multiple-definition")
add_importlibs(csrss csrsrv ntdll)
if(ARCH STREQUAL "arm64")
    # ARM64: llvm-dlltool converts delay-imports to regular imports, causing
    # all DLL dependencies to load at once. Deep recursive LdrpLoadDll calls
    # overflow the default 1MB/4KB-commit stack. Reserve 2MB, commit 256KB.
    # SizeOfStackCommit is patched to 0x40000 by qemu/mkramdisk.sh.
    target_link_options(csrss PRIVATE "-Wl,--stack,2097152")
endif()
add_dependencies(csrss psdk bugcodes)
add_cd_file(TARGET csrss DESTINATION reactos/system32 FOR all)
