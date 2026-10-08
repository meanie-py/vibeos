
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(-D_CRT_DECLARE_NONSTDC_NAMES=1)
spec2def(appwiz.cpl appwiz.spec)

list(APPEND SOURCE
    # addons.c  # replaced by Rust (sdk/lib/rust/appwiz-rust/src/appwiz/addons.rs)
    # appwiz.c  # replaced by Rust (sdk/lib/rust/appwiz-rust/src/appwiz/appwiz.rs)
    # createlink.c  # replaced by Rust (sdk/lib/rust/appwiz-rust/src/appwiz/createlink.rs)
    # removestartmenuitems.c  # replaced by Rust (sdk/lib/rust/appwiz-rust/src/appwiz/removestartmenuitems.rs)
    rust_cohab_stub.c
    appwiz.h)

file(GLOB appwiz_rc_deps resources/*.*)
add_rc_deps(appwiz.rc ${appwiz_rc_deps})

# Every translation unit is Rust now (appwiz-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(appwiz_rust appwiz-rust "" TRUE)
add_library(appwiz MODULE
    ${SOURCE}
    appwiz.rc
    ${CMAKE_CURRENT_BINARY_DIR}/appwiz.def)
add_dependencies(appwiz rust_appwiz_rust)
target_link_libraries(appwiz ${appwiz_rust_LIB})

set_module_type(appwiz cpl UNICODE)
target_link_libraries(appwiz uuid wine)
add_delay_importlibs(appwiz msi)
add_importlibs(appwiz urlmon ole32 comctl32 advapi32 shell32 shlwapi user32 msvcrt kernel32 ntdll)
add_pch(appwiz appwiz.h SOURCE)
add_cd_file(TARGET appwiz DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(appwiz) # CORE-5743: No CONST_VTABLE
