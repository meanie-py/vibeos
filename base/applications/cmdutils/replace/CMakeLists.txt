
# Every translation unit is Rust now (replace-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(replace_rust replace-rust "" TRUE)
add_executable(replace
    # replace.c  # replaced by Rust (sdk/lib/rust/replace-rust/src/replace/replace.rs)
    # util.c  # replaced by Rust (sdk/lib/rust/replace-rust/src/replace/util.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    replace.rc)
add_dependencies(replace rust_replace_rust)
target_link_libraries(replace ${replace_rust_LIB})
set_module_type(replace win32cui UNICODE)
target_link_libraries(replace conutils)
add_importlibs(replace msvcrt kernel32)
add_cd_file(TARGET replace DESTINATION reactos/system32 FOR all)
