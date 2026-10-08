
spec2def(wined3dcfg.cpl wined3dcfg.spec)

list(APPEND SOURCE
    # wined3dcfg.c  # replaced by Rust (sdk/lib/rust/wined3dcfg-rust/src/wined3dcfg/wined3dcfg.rs)
    # general.c  # replaced by Rust (sdk/lib/rust/wined3dcfg-rust/src/wined3dcfg/general.rs)
    rust_cohab_stub.c
    wined3dcfg.h)

add_rc_deps(wined3dcfg.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/wined3dcfg.ico)

# Every translation unit is Rust now (wined3dcfg-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wined3dcfg_rust wined3dcfg-rust "" TRUE)
add_library(wined3dcfg MODULE
    ${SOURCE}
    wined3dcfg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wined3dcfg.def)
add_dependencies(wined3dcfg rust_wined3dcfg_rust)
target_link_libraries(wined3dcfg ${wined3dcfg_rust_LIB})

set_module_type(wined3dcfg cpl UNICODE)
add_importlibs(wined3dcfg user32 comctl32 advapi32 msvcrt kernel32)
add_pch(wined3dcfg wined3dcfg.h SOURCE)
add_cd_file(TARGET wined3dcfg DESTINATION reactos/system32 FOR all)
