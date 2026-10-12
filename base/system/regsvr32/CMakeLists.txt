
# Every translation unit is Rust now (regsvr32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(regsvr32_rust regsvr32-rust "" TRUE)
add_executable(regsvr32
    # regsvr32.c  # replaced by Rust (sdk/lib/rust/regsvr32-rust/src/regsvr32/regsvr32.rs)
    rust_cohab_stub.c
    regsvr32.rc)
add_dependencies(regsvr32 rust_regsvr32_rust)
target_link_libraries(regsvr32 ${regsvr32_rust_LIB})
set_module_type(regsvr32 win32gui UNICODE)
add_importlibs(regsvr32 user32 shell32 ole32 msvcrt kernel32 ntdll)
add_cd_file(TARGET regsvr32 DESTINATION reactos/system32 FOR all)
