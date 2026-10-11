
# Every translation unit is Rust now (hostname-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(hostname_rust hostname-rust "" TRUE)
add_executable(hostname
    # hostname.c  # replaced by Rust (sdk/lib/rust/hostname-rust/src/hostname/hostname.rs)
    rust_cohab_stub.c
    hostname.rc)
add_dependencies(hostname rust_hostname_rust)
target_link_libraries(hostname ${hostname_rust_LIB})
set_module_type(hostname win32cui UNICODE)
target_link_libraries(hostname conutils)
add_importlibs(hostname msvcrt kernel32)
add_cd_file(TARGET hostname DESTINATION reactos/system32 FOR all)
