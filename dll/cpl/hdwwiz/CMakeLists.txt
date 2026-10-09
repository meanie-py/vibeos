
spec2def(hdwwiz.cpl hdwwiz.spec)
file(GLOB hdwwiz_rc_deps resources/*.*)
add_rc_deps(hdwwiz.rc ${hdwwiz_rc_deps})

# Every translation unit is Rust now (hdwwiz-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(hdwwiz_rust hdwwiz-rust "" TRUE)
add_library(hdwwiz MODULE
    # hdwwiz.c  # replaced by Rust (sdk/lib/rust/hdwwiz-rust/src/hdwwiz/hdwwiz.rs)
    rust_cohab_stub.c
    hdwwiz.rc
    ${CMAKE_CURRENT_BINARY_DIR}/hdwwiz.def)
add_dependencies(hdwwiz rust_hdwwiz_rust)
target_link_libraries(hdwwiz ${hdwwiz_rust_LIB})

set_module_type(hdwwiz cpl UNICODE)
add_importlibs(hdwwiz setupapi advapi32 comctl32 rpcrt4 user32 gdi32 devmgr msvcrt kernel32 ntdll)
add_cd_file(TARGET hdwwiz DESTINATION reactos/system32 FOR all)
