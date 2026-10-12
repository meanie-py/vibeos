
# Every translation unit is Rust now (dllhost-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dllhost_rust dllhost-rust "" TRUE)
add_executable(dllhost
    # dllhost.c  # replaced by Rust (sdk/lib/rust/dllhost-rust/src/dllhost/dllhost.rs)
    rust_cohab_stub.c
    dllhost.rc)
add_dependencies(dllhost rust_dllhost_rust)
target_link_libraries(dllhost ${dllhost_rust_LIB})
set_module_type(dllhost win32gui UNICODE)
add_importlibs(dllhost msvcrt kernel32 ntdll)
add_cd_file(TARGET dllhost DESTINATION reactos/system32 FOR all)
