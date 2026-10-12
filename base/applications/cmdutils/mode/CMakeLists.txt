
# Every translation unit is Rust now (mode-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mode_rust mode-rust "" TRUE)
add_executable(mode
    # mode.c  # replaced by Rust (sdk/lib/rust/mode-rust/src/mode/mode.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    mode.rc)
add_dependencies(mode rust_mode_rust)
target_link_libraries(mode ${mode_rust_LIB})
set_module_type(mode win32cui UNICODE)
target_link_libraries(mode conutils)
add_importlibs(mode user32 msvcrt kernel32)
set_target_properties(mode PROPERTIES SUFFIX ".com")
add_cd_file(TARGET mode DESTINATION reactos/system32 FOR all)
