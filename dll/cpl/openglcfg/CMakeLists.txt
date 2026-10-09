
spec2def(openglcfg.cpl openglcfg.spec)

list(APPEND SOURCE
    # openglcfg.c  # replaced by Rust (sdk/lib/rust/openglcfg-rust/src/openglcfg/openglcfg.rs)
    # general.c  # replaced by Rust (sdk/lib/rust/openglcfg-rust/src/openglcfg/general.rs)
    rust_cohab_stub.c
    openglcfg.h)

add_rc_deps(openglcfg.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/openglcfg.ico)

# Every translation unit is Rust now (openglcfg-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(openglcfg_rust openglcfg-rust "" TRUE)
add_library(openglcfg MODULE
    ${SOURCE}
    openglcfg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/openglcfg.def)
add_dependencies(openglcfg rust_openglcfg_rust)
target_link_libraries(openglcfg ${openglcfg_rust_LIB})

set_module_type(openglcfg cpl UNICODE)
add_importlibs(openglcfg user32 comctl32 advapi32 msvcrt kernel32)
add_pch(openglcfg openglcfg.h SOURCE)
add_cd_file(TARGET openglcfg DESTINATION reactos/system32 FOR all)
