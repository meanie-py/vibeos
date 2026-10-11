
PROJECT(more)

# Every translation unit is Rust now (more-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(more_rust more-rust "" TRUE)
add_executable(more
    # more.c  # replaced by Rust (sdk/lib/rust/more-rust/src/more/more.rs)
    rust_cohab_stub.c
    more.rc)
add_dependencies(more rust_more_rust)
target_link_libraries(more ${more_rust_LIB})
set_module_type(more win32cui UNICODE)
target_link_libraries(more conutils)
add_importlibs(more advapi32 msvcrt kernel32)
set_target_properties(more PROPERTIES SUFFIX ".com")
add_cd_file(TARGET more DESTINATION reactos/system32 FOR all)
