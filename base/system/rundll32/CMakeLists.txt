
# Every translation unit is Rust now (rundll32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(rundll32_rust rundll32-rust "" TRUE)
add_executable(rundll32
    # rundll32.c  # replaced by Rust (sdk/lib/rust/rundll32-rust/src/rundll32/rundll32.rs)
    rust_cohab_stub.c
    rundll32.rc)
add_dependencies(rundll32 rust_rundll32_rust)
target_link_libraries(rundll32 ${rundll32_rust_LIB})
set_module_type(rundll32 win32gui UNICODE)
add_importlibs(rundll32 user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET rundll32 DESTINATION reactos/system32 FOR all)
