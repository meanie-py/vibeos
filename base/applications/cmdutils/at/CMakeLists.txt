
# Every translation unit is Rust now (at-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(at_rust at-rust "" TRUE)
add_executable(at
    # at.c  # replaced by Rust (sdk/lib/rust/at-rust/src/at/at.rs)
    rust_cohab_stub.c
    at.rc)
add_dependencies(at rust_at_rust)
target_link_libraries(at ${at_rust_LIB})
set_module_type(at win32cui UNICODE)
target_link_libraries(at conutils)
add_importlibs(at netapi32 msvcrt kernel32)
add_cd_file(TARGET at DESTINATION reactos/system32 FOR all)
