
# Every translation unit is Rust now (find-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(find_rust find-rust "" TRUE)
add_executable(find
    # find.c  # replaced by Rust (sdk/lib/rust/find-rust/src/find/find.rs)
    rust_cohab_stub.c
    find.rc)
add_dependencies(find rust_find_rust)
target_link_libraries(find ${find_rust_LIB})
set_module_type(find win32cui UNICODE)
target_link_libraries(find conutils)
add_importlibs(find msvcrt kernel32)
add_cd_file(TARGET find DESTINATION reactos/system32 FOR all)
