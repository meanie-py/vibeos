include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# pagefile.c + sminit.c have no Rust equivalent yet → stay C. The other 7
# smss .c files are replaced by Rust (smss-csrss-rust, smss_exe_only). Keep
# all .c on disk (never delete the C — translation philosophy).
list(APPEND SOURCE
    # crashdmp.c — replaced by Rust (smss-csrss-rust/src/smss/crashdmp.rs)
    pagefile.c
    sminit.c
    # smloop.c   — replaced by Rust (smss-csrss-rust/src/smss/smloop.rs)
    # smsbapi.c  — replaced by Rust (smss-csrss-rust/src/smss/smsbapi.rs)
    # smsessn.c  — replaced by Rust (smss-csrss-rust/src/smss/smsessn.rs)
    # smsubsys.c — replaced by Rust (smss-csrss-rust/src/smss/smsubsys.rs)
    # smutil.c   — replaced by Rust (smss-csrss-rust/src/smss/smutil.rs)
    # smss.c     — replaced by Rust (smss-csrss-rust/src/smss/smss.rs)
    smss_rust_stub.c
    smss.h)

# --- Rust cohabitation (smss-csrss-rust, smss_exe_only) — task #18 --------
# Mirrors the csrss.exe flip. smss.exe builds the crate with
# `--no-default-features --features smss_exe_only,ntddi_ws03`, which includes
# the smss/* + smlib modules and EXCLUDES csrsrv/csrlib/csrss/basesrv so the
# archive carries only smss's `_main` + Smp* helpers (no duplicate _main, no
# stray csrsrv data statics). pagefile.c + sminit.c stay C and provide
# SmpCreatePagingFiles / SmpInit, which the Rust `_main` calls.
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
set(SMSS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsmss_csrss_rust.a")
add_custom_command(
    OUTPUT ${SMSS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p smss-csrss-rust
            --no-default-features
            --features smss_exe_only,ntddi_ws03
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libsmss_csrss_rust.a
        ${SMSS_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/mod.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/crashdmp.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smloop.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smsbapi.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smsessn.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smss.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smsubsys.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/smss/smutil.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/inlines.rs
    COMMENT "Building smss-csrss-rust smss_exe_only (${RUST_TARGET})"
    VERBATIM)
add_custom_target(smss_rust_lib DEPENDS ${SMSS_RUST_LIB})
# --------------------------------------------------------------------------

add_executable(smss WIN32 ${SOURCE} smss.rc)
target_link_libraries(smss nt ${PSEH_LIB} smlib ${SMSS_RUST_LIB})
add_dependencies(smss smss_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(smss PRIVATE "-Wl,--allow-multiple-definition")
set_module_type(smss nativecui)
add_importlibs(smss ntdll)
add_pch(smss smss.h SOURCE)
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(smss_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_cd_file(TARGET smss DESTINATION reactos/system32 FOR all)

# Three runtime bugs cleared while flipping (2026-05-31), all "local per-file
# re-definition shadows the correct ntdll_sys bindgen" footguns:
#   1. smss.rs re-defined SMP_*_FLAG with SCRAMBLED values (SUBSYSTEM=0x02 vs the
#      real 0x08, ASYNC/AUTOCHK/INVALID_PATH all shifted) while smsubsys.rs used
#      the correct bindgen values → SmpExecuteCommand's `& SUBSYSTEM_FLAG` never
#      matched the 0x08 the caller passed → SmpLoadSubSystem never ran → csrss
#      never launched. Fixed: deleted the local consts (use ntdll_sys::nt).
#   2. smutil.rs had SMP_INVALID_PATH=0x08 (real 0x10). Same fix.
#   3. smsessn.rs hand-rolled a TRUNCATED `struct SMP_SUBSYSTEM { Entry,
#      MuSessionId }` → MuSessionId at offset 8, but the real layout has Event/
#      ProcessHandle/ImageType/PortHandle/SbApiPort/ClientId in between (offset
#      36 on i386). CheckDuplicateMuSessionId read the Event HANDLE (0x3D0=976)
#      as MuSessionId, never matched session 0, SmpSbCreateSession returned
#      OBJECT_NAME_NOT_FOUND → smss (critical) terminated → BUGCHECK 0xc000021a.
#      Fixed: deleted the truncated mirror (use bindgen _SMP_SUBSYSTEM).
# After these, all 3 arches boot to `shell ready` with smss.exe in Rust:
# i386-uefi 8/8, amd64-uefi 8/8, arm64 6/6. pagefile.c + sminit.c stay C.
