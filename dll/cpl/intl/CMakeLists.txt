
spec2def(intl.cpl intl.spec)

# intl.cpl — Rust cohabitation flip (all 12 compiled *.c → intl-rust).
#
# The International control panel: the General/Numbers/Currency/Time/Date/Sort/Languages/
# Advanced property sheets, the locale + keyboard-layout installer that drives intl.inf, and
# the sample-formatting helpers the pages preview with.
#
# Profile decisions (gen_driver_types.sh), all load-bearing:
#   * BLOCKLIST_FUNCS_FROM_CRATE — intl is a thin driver of other people's APIs
#     (user32/comctl32/advapi32/setupapi/shlwapi/shell32, plus the locale/geo/calendar and .inf
#     families), so BLOCKLIST_FUNCS=1 meant 87 prototypes to hand-write, and a hand-written
#     prototype wrong by one pointer level links and then corrupts silently. profile_derive
#     defaults to 1 and does not know the module's shape.
#   * winuser.h + winbase.h allowlisted. The derive adds a header when it OWNS a `case` label,
#     and intl's window messages resolve through the -sys glob instead — so neither made the
#     list even though every page proc calls GetDlgItem/SendDlgItemMessageW/CheckDlgButton.
#     Types from a glob are not prototypes.
#   * MACRO_SIGNED=0, MEASURED: 97 errors unsigned against 617 signed, one cargo check each.
#     That is the FIFTH property-sheet module in a row where the resemblance argument ("dialog
#     module, so signed int control ids") would have lost.
#   * UNBLOCK of DLGPROC, the PROPSHEET containers and the TCHAR family — Rust compares
#     fn-pointer ABIs NOMINALLY and ntdll-sys spells each in the arch's literal convention;
#     TCHAR because ntdll-sys is built WITHOUT UNICODE, so its glob says `TCHAR = c_char`.
#   * REAL_UNIONS=1 — PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union and the C
#     FLATTENS them (`psh.nStartPage`, `psp.pszTemplate`).
set(INTL_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(INTL_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(INTL_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(INTL_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(INTL_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-intl-target" intl-rust)
set(INTL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libintl_rust.a")
rust_crate_sources(INTL_RUST_SRCS "${INTL_RUST_WORKSPACE_DIR}" intl-rust)
add_custom_command(
    OUTPUT ${INTL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${INTL_RUST_TARGET_DIR}
        cargo build --release --target ${INTL_RUST_TARGET}
            -p intl-rust
            --manifest-path ${INTL_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${INTL_RUST_TARGET_DIR}/${INTL_RUST_TARGET}/release/libintl_rust.a
        ${INTL_RUST_LIB}
    DEPENDS ${INTL_RUST_SRCS}
            ${INTL_RUST_WORKSPACE_DIR}/intl-rust/Cargo.toml
    WORKING_DIRECTORY ${INTL_RUST_WORKSPACE_DIR}
    COMMENT "Building intl-rust (${INTL_RUST_TARGET})"
)
add_custom_target(rust_intl DEPENDS ${INTL_RUST_LIB})

# All 12 compiled *.c → intl-rust (src/intl/*.rs), kept on disk per the never-delete rule.
# list(APPEND SOURCE
#     currency.c
#     date.c
#     generalp.c
#     intl.c
#     inplocale.c
#     numbers.c
#     time.c
#     misc.c
#     kblayouts.c
#     languages.c
#     advanced.c
#     sort.c)

list(APPEND SOURCE
    intl.h)

add_rc_deps(intl.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/applet.ico)

add_library(intl MODULE
    ${SOURCE}
    intl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/intl.def)

set_module_type(intl cpl UNICODE)
# libintl_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(intl ${INTL_RUST_LIB})
add_dependencies(intl rust_intl)
add_importlibs(intl user32 comctl32 advapi32 setupapi shlwapi shell32 msvcrt kernel32 ntdll)
add_pch(intl intl.h SOURCE)
add_cd_file(TARGET intl DESTINATION reactos/system32 FOR all)
