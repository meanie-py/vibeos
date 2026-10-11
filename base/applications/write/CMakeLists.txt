
# Every translation unit is Rust now (write-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(write_rust write-rust "" TRUE)
add_executable(write
    # write.c  # replaced by Rust (sdk/lib/rust/write-rust/src/write/write.rs)
    rust_cohab_stub.c
    rsrc.rc)
add_dependencies(write rust_write_rust)
target_link_libraries(write ${write_rust_LIB})
set_module_type(write win32gui UNICODE)
add_importlibs(write user32 gdi32 msvcrt kernel32)
add_cd_file(TARGET write DESTINATION reactos/system32 FOR all)
