
spec2def(clb.dll clb.spec)

# Every translation unit is Rust now (clb-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(clb_rust clb-rust "" TRUE)
add_library(clb MODULE
    # clb.c  # replaced by Rust (sdk/lib/rust/clb-rust/src/clb/clb.rs)
    rust_cohab_stub.c
    clb.rc
    ${CMAKE_CURRENT_BINARY_DIR}/clb.def)
add_dependencies(clb rust_clb_rust)
target_link_libraries(clb ${clb_rust_LIB})

set_module_type(clb win32dll UNICODE)
add_importlibs(clb user32 comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET clb DESTINATION reactos/system32 FOR all)
