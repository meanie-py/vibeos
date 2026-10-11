
# Every translation unit is Rust now (label-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(label_rust label-rust "" TRUE)
add_executable(label
    # label.c  # replaced by Rust (sdk/lib/rust/label-rust/src/label/label.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    label.rc)
add_dependencies(label rust_label_rust)
target_link_libraries(label ${label_rust_LIB})
set_module_type(label win32cui UNICODE)
target_link_libraries(label conutils)
add_importlibs(label msvcrt kernel32)
add_cd_file(TARGET label DESTINATION reactos/system32 FOR all)
