
# Every translation unit is Rust now (where-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(where_rust where-rust "" TRUE)
add_executable(where
    # where.c  # replaced by Rust (sdk/lib/rust/where-rust/src/where/where.rs)
    rust_cohab_stub.c
    where.rc)
add_dependencies(where rust_where_rust)
target_link_libraries(where ${where_rust_LIB})
set_module_type(where win32cui UNICODE)
target_link_libraries(where conutils)
add_importlibs(where msvcrt kernel32)
add_cd_file(TARGET where DESTINATION reactos/system32 FOR all)
