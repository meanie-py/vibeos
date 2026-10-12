
# Every translation unit is Rust now (mshta-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mshta_rust mshta-rust "" TRUE)
add_executable(mshta
    # main.c  # replaced by Rust (sdk/lib/rust/mshta-rust/src/mshta/main.rs)
    rust_cohab_stub.c
    mshta.rc)
add_dependencies(mshta rust_mshta_rust)
target_link_libraries(mshta ${mshta_rust_LIB})
target_link_libraries(mshta)
set_module_type(mshta win32gui)
add_importlibs(mshta msvcrt mshtml kernel32 ntdll)
add_cd_file(TARGET mshta DESTINATION reactos/system32 FOR all)
