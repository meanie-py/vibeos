
# Every translation unit is Rust now (sort-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sort_rust sort-rust "" TRUE)
add_executable(sort
    # sort.c  # replaced by Rust (sdk/lib/rust/sort-rust/src/sort/sort.rs)
    rust_cohab_stub.c
    sort.rc)
add_dependencies(sort rust_sort_rust)
target_link_libraries(sort ${sort_rust_LIB})
set_module_type(sort win32cui)
add_importlibs(sort msvcrt kernel32)
add_cd_file(TARGET sort DESTINATION reactos/system32 FOR all)
