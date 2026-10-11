
# Every translation unit is Rust now (dwnl-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dwnl_rust dwnl-rust "" TRUE)
add_executable(dwnl
    # dwnl.c  # replaced by Rust (sdk/lib/rust/dwnl-rust/src/dwnl/dwnl.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    dwnl.rc)
add_dependencies(dwnl rust_dwnl_rust)
target_link_libraries(dwnl ${dwnl_rust_LIB})
set_module_type(dwnl win32cui UNICODE)
target_link_libraries(dwnl conutils uuid)
add_importlibs(dwnl urlmon wininet msvcrt kernel32)
add_cd_file(TARGET dwnl DESTINATION reactos/system32 FOR all)
