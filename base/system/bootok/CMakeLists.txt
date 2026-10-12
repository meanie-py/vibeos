
# Every translation unit is Rust now (bootok-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(bootok_rust bootok-rust "" TRUE)
add_executable(bootok
    # bootok.c  # replaced by Rust (sdk/lib/rust/bootok-rust/src/bootok/bootok.rs)
    rust_cohab_stub.c
    bootok.rc)
add_dependencies(bootok rust_bootok_rust)
target_link_libraries(bootok ${bootok_rust_LIB})
set_module_type(bootok win32cui UNICODE)
add_importlibs(bootok advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET bootok DESTINATION reactos/system32 FOR all)
