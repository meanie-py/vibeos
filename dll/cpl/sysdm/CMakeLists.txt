spec2def(sysdm.cpl sysdm.spec)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/udmihelp)

# sysdm.cpl — Rust cohabitation flip (all 12 compiled *.c → sysdm-rust).
#
# The System control panel: the General/Hardware/Advanced/Remote property sheets, the SMBIOS
# vendor-name cleanup that produces the machine string, the environment-variable editor, the
# hardware-profile manager, the user-profile list, the pagefile (virtual memory) editor, and
# the startup/recovery boot.ini editor.
#
# rust_varargs_shim.c is NOT a translation reverted to C: it holds the `va_start` for
# ResourceMessageBox (sysdm's own, sysdm.c:27) and for strsafe.h's StringCchPrintfW, neither of
# which stable Rust can DEFINE. See the file's own header.
#
# Profile decisions (gen_driver_types.sh), all load-bearing:
#   * BLOCKLIST_FUNCS_FROM_CRATE — sysdm is a thin driver of other people's APIs, so
#     hand-writing the surface would be hundreds of prototypes, each of which links and then
#     corrupts silently if it is wrong by one pointer level.
#   * MACRO_SIGNED=0, MEASURED. The FOURTH property-sheet module in a row where the
#     resemblance argument ("dialog module, so signed int control ids") would have lost.
#   * UNBLOCK of DLGPROC/WNDPROC, the PROPSHEET containers, and the TCHAR family — Rust
#     compares fn-pointer ABIs NOMINALLY, and ntdll-sys is built WITHOUT UNICODE.
#   * REAL_UNIONS=1 — PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union.
set(SYSDM_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(SYSDM_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SYSDM_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SYSDM_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(SYSDM_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-sysdm-target" sysdm-rust)
set(SYSDM_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsysdm_rust.a")
rust_crate_sources(SYSDM_RUST_SRCS "${SYSDM_RUST_WORKSPACE_DIR}" sysdm-rust)
add_custom_command(
    OUTPUT ${SYSDM_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SYSDM_RUST_TARGET_DIR}
        cargo build --release --target ${SYSDM_RUST_TARGET}
            -p sysdm-rust
            --manifest-path ${SYSDM_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${SYSDM_RUST_TARGET_DIR}/${SYSDM_RUST_TARGET}/release/libsysdm_rust.a
        ${SYSDM_RUST_LIB}
    DEPENDS ${SYSDM_RUST_SRCS}
            ${SYSDM_RUST_WORKSPACE_DIR}/sysdm-rust/Cargo.toml
    WORKING_DIRECTORY ${SYSDM_RUST_WORKSPACE_DIR}
    COMMENT "Building sysdm-rust (${SYSDM_RUST_TARGET})"
)
add_custom_target(rust_sysdm DEPENDS ${SYSDM_RUST_LIB})

# All 12 compiled *.c → sysdm-rust (src/sysdm/*.rs), kept on disk per the never-delete rule.
# list(APPEND SOURCE
#     advanced.c
#     environment.c
#     general.c
#     hardprof.c
#     hardware.c
#     licence.c
#     performance.c
#     smbios.c
#     startrec.c
#     sysdm.c
#     userprofile.c
#     virtmem.c)

list(APPEND SOURCE
    rust_varargs_shim.c
    precomp.h)

file(GLOB sysdm_rc_deps resources/*.*)
add_rc_deps(sysdm.rc ${sysdm_rc_deps})

add_library(sysdm MODULE
    ${SOURCE}
    sysdm.rc
    ${CMAKE_CURRENT_BINARY_DIR}/sysdm.def)
add_pch(sysdm precomp.h SOURCE)

set_module_type(sysdm cpl UNICODE)
# libsysdm_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(sysdm ${SYSDM_RUST_LIB} udmihelp uuid)
# lld first-match: compiler_builtins in the Rust staticlib and libgcc both carry the 64-bit
# division helpers (__udivdi3/__divdi3). Same code, two copies — take the first.
target_link_options(sysdm PRIVATE LINKER:--allow-multiple-definition)
add_dependencies(sysdm rust_sysdm)
add_importlibs(sysdm advapi32 setupapi user32 gdi32 comctl32 comdlg32 msimg32 shell32 shlwapi ole32 powrprof msvcrt kernel32 ntdll)
add_cd_file(TARGET sysdm DESTINATION reactos/system32 FOR all)
