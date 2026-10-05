add_definitions(-D_WIN32)
spec2def(desk.cpl desk.spec)

# desk.cpl — Rust cohabitation flip (all 19 compiled *.c → desk-rust).
#
# The Display control panel: the Background/Screen Saver/Appearance/Settings/Desktop-items
# property sheets, the classic-and-visual theme reader and applier, the monitor-selection and
# preview custom controls, the GDI+ wallpaper loader, and the CDevSettings IDataObject that
# carries the desk.cpl <-> display-driver shell-extension protocol.
#
# guid.c stays C. It is an `initguid.h` translation unit whose whole job is EMITTING the CLSIDs
# as data — zero functions, so there is nothing to translate. Same call as ole32 and aclui.
#
# rust_varargs_shim.c is no longer built (retired 2026-09-25: the strsafe printf family is strsafe-rust macros that CALL
# msvcrt's variadic _snwprintf, which stable Rust can do; the file stays on disk, inert).
#
# Profile decisions (gen_driver_types.sh), all load-bearing:
#   * BLOCKLIST_FUNCS_FROM_CRATE — desk is a thin driver of other people's APIs
#     (user32/gdi32/comctl32/shell32/setupapi/uxtheme/gdiplus), so hand-writing the surface
#     would be several hundred prototypes, each of which links and then corrupts silently if it
#     is wrong by one pointer level.
#   * MACRO_SIGNED=0, MEASURED: 1,517 errors unsigned against 1,820 signed. desk is the third
#     property-sheet module where the resemblance argument ("dialog module, so signed int
#     control ids") would have lost.
#   * UNBLOCK of DLGPROC, the PROPSHEET containers, and the TCHAR family. The callbacks because
#     Rust compares fn-pointer ABIs NOMINALLY and ntdll-sys spells each one in the arch's
#     literal convention; TCHAR because ntdll-sys is built WITHOUT UNICODE, so inheriting its
#     `TCHAR = c_char` made desk.h's own `TCHAR szPath[MAX_PATH]` structs half-sized — a layout
#     bug that compiles clean.
#   * REAL_UNIONS=1 — PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union.
#   * The widl-GENERATED objidl.h/wtypes.h are allowlisted, because `--allowlist-file` matches
#     where an item is DEFINED and objbase.h merely #includes them: without that the island
#     carried IDataObject but none of the DVASPECT_*/TYMED_*/DATADIR_* its call sites pass.
set(DESK_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(DESK_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DESK_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DESK_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(DESK_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-desk-target" desk-rust)
set(DESK_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdesk_rust.a")
rust_crate_sources(DESK_RUST_SRCS "${DESK_RUST_WORKSPACE_DIR}" desk-rust)
add_custom_command(
    OUTPUT ${DESK_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DESK_RUST_TARGET_DIR}
        cargo build --release --target ${DESK_RUST_TARGET}
            -p desk-rust
            --manifest-path ${DESK_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${DESK_RUST_TARGET_DIR}/${DESK_RUST_TARGET}/release/libdesk_rust.a
        ${DESK_RUST_LIB}
    DEPENDS ${DESK_RUST_SRCS}
            ${DESK_RUST_WORKSPACE_DIR}/desk-rust/Cargo.toml
    WORKING_DIRECTORY ${DESK_RUST_WORKSPACE_DIR}
    COMMENT "Building desk-rust (${DESK_RUST_TARGET})"
)
add_custom_target(rust_desk DEPENDS ${DESK_RUST_LIB})

# All 19 compiled *.c → desk-rust (src/desk/*.rs), kept on disk per the never-delete rule.
# list(APPEND SOURCE
#     advmon.c
#     appearance.c
#     background.c
#     classinst.c
#     desk.c
#     desktop.c
#     devsett.c
#     dibitmap.c
#     misc.c
#     preview.c
#     screensaver.c
#     advappdlg.c
#     effappdlg.c
#     settings.c
#     monslctl.c
#     general.c
#     draw.c
#     theme.c
#     muireg.c)

list(APPEND PCH_SKIP_SOURCE
    guid.c)

file(GLOB desk_rc_deps resources/*.*)
add_rc_deps(desk.rc ${desk_rc_deps})

add_library(desk MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    desk.rc
    ${CMAKE_CURRENT_BINARY_DIR}/desk.def)

set_module_type(desk cpl UNICODE)
# libdesk_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(desk ${DESK_RUST_LIB} uuid)
add_dependencies(desk rust_desk)
add_importlibs(desk user32 advapi32 gdi32 comctl32 comdlg32 ole32 setupapi shell32 shlwapi uxtheme gdiplus msvcrt kernel32 ntdll)
add_pch(desk desk.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET desk DESTINATION reactos/system32 FOR all)
