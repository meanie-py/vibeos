
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

spec2def(basesrv.dll basesrv.spec ADD_IMPORTLIB)

# --- Rust cohabitation (smss-csrss-rust, basesrv_dll_only) — task #18 -----
# PARTIAL flip: appcompat/init/nls/proc/sndsntry → Rust; dosdev.c + vdm.c stay
# C (no Rust translation yet) and keep defining the VDM/DOS helpers the Rust
# dispatch table extern-refs. Mirrors the csrss.exe cohab pattern (own cargo
# target dir; the 5 flipped .c files are removed from SOURCE so lld pulls their
# symbols from the archive; basesrv_rust_stub.c keeps add_library valid).
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
set(BASESRV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsmss_csrss_rust.a")
add_custom_command(
    OUTPUT ${BASESRV_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p smss-csrss-rust
            --no-default-features
            --features basesrv_dll_only,ntddi_ws03
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libsmss_csrss_rust.a
        ${BASESRV_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/mod.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/appcompat.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/init.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/nls.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/proc.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/src/basesrv/sndsntry.rs
        ${RUST_WORKSPACE_DIR}/smss-csrss-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building smss-csrss-rust basesrv_dll_only (${RUST_TARGET})"
    VERBATIM)
add_custom_target(basesrv_rust_lib DEPENDS ${BASESRV_RUST_LIB})
# -------------------------------------------------------------------------

list(APPEND SOURCE
    # appcompat.c — replaced by Rust (smss-csrss-rust/src/basesrv/appcompat.rs)
    dosdev.c
    # init.c      — replaced by Rust (smss-csrss-rust/src/basesrv/init.rs)
    # proc.c      — replaced by Rust (smss-csrss-rust/src/basesrv/proc.rs)
    # sndsntry.c  — replaced by Rust (smss-csrss-rust/src/basesrv/sndsntry.rs)
    vdm.c
    # nls.c       — replaced by Rust (smss-csrss-rust/src/basesrv/nls.rs)
    basesrv_rust_stub.c
    basesrv.h)

add_library(basesrv MODULE
    ${SOURCE}
    basesrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/basesrv.def)

set_module_type(basesrv win32dll UNICODE ENTRYPOINT DllMain 12)
#############################################
## HACK FOR MSVC COMPILATION WITH win32dll ##
set_subsystem(basesrv console)
################# END  HACK #################

target_link_libraries(basesrv ${PSEH_LIB} ${BASESRV_RUST_LIB})
add_dependencies(basesrv basesrv_rust_lib)
# Rust .a pulls libcore/compiler-builtins objects; allow intrinsic overlap.
target_link_options(basesrv PRIVATE "-Wl,--allow-multiple-definition")
add_importlibs(basesrv csrsrv ntdll)
add_pch(basesrv basesrv.h SOURCE)
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(basesrv_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_dependencies(basesrv bugcodes xdk)
add_cd_file(TARGET basesrv DESTINATION reactos/system32 FOR all)

# Cohabitation flip deferred (task #18). 2026-05-26 debug session
# uncovered:
#   1. `CSR_SERVER_DLL_FULL` Rust mirror was missing the Length/Name/
#      ServerHandle/ServerId/Unknown preamble — corrupted ApiBase
#      writes. FIXED. The auxiliary `CSR_SERVER_DLL_STRUCT` (a
#      2-field fake) was *also* used to read/write ProcessHeap +
#      SharedSection at wrong offsets. FIXED — heap now comes from
#      RtlGetProcessHeap() (TEB inline) and SharedSection from the
#      proper struct offset (0x3C with CSR_DBG=on).
#   2. `BaseServerApiNameTable` was `#[cfg(feature = "CSR_DBG")]`-
#      gated, so it didn't compile by default — but the C build has
#      DBG=1 → CSR_DBG on → NameTable slot is real. FIXED.
#   3. `BASE_STATIC_SERVER_DATA` Rust mirror had completely wrong field
#      order (TermsrvClientTimeZoneId first vs WindowsDirectory first in
#      C) and was missing WindowsMajorVersion / NlsUserInfo /
#      tziTermsrvClientTimeZone / ktTermsrvClientBias / etc, with opaque
#      [u8;N] blobs for SysInfo/TimeOfDay. FIXED 2026-05-31: rebuilt
#      #[repr(C)] in exact C field order using the real ntdll-sys nested
#      types (UNICODE_STRING / SYSTEM_BASIC_INFORMATION /
#      SYSTEM_TIMEOFDAY_INFORMATION / NLS_USER_INFO / TIME_ZONE_INFORMATION
#      / KSYSTEM_TIME). i386 size-asserted == 0x1AC8 (matches base.h
#      `#if _M_IX86`); 64-bit correct by construction. 3 arches cargo-green.
# The Rust source improvements (CSR_SERVER_DLL layout, no_mangle on data
# globals, NameTable export, BASE_STATIC_SERVER_DATA layout) ARE landed.
# Remaining to land the cmake flip: (a) dosdev.c + vdm.c have no Rust
# equivalent yet → must stay C in a PARTIAL cohab; (b) add the per-binary
# cargo build (--features basesrv_dll_only,ntddi_ws03) + basesrv_rust_stub.c
# + comment the 5 translated .c files; (c) boot-test to confirm the
# ServerDllInitialization hang is cleared by the layout repair.
