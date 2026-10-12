
# Every translation unit is Rust now (doskey-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(doskey_rust doskey-rust "" TRUE)
add_executable(doskey
    # doskey.c  # replaced by Rust (sdk/lib/rust/doskey-rust/src/doskey/doskey.rs)
    rust_cohab_stub.c
    doskey.rc)
add_dependencies(doskey rust_doskey_rust)
target_link_libraries(doskey ${doskey_rust_LIB})
set_module_type(doskey win32cui UNICODE)
add_importlibs(doskey user32 msvcrt kernel32)
add_cd_file(TARGET doskey DESTINATION reactos/system32 FOR all)
