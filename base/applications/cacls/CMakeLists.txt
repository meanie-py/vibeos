
# Every translation unit is Rust now (cacls-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(cacls_rust cacls-rust "" TRUE)
add_executable(cacls
    # cacls.c  # replaced by Rust (sdk/lib/rust/cacls-rust/src/cacls/cacls.rs)
    rust_cohab_stub.c
    cacls.rc)
add_dependencies(cacls rust_cacls_rust)
target_link_libraries(cacls ${cacls_rust_LIB})
set_module_type(cacls win32cui UNICODE)
target_link_libraries(cacls conutils)
add_importlibs(cacls advapi32 msvcrt kernel32)
add_cd_file(TARGET cacls DESTINATION reactos/system32 FOR all)
