
# Every translation unit is Rust now (setup16-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(setup16_rust setup16-rust "" TRUE)
add_executable(setup16
    # main.c  # replaced by Rust (sdk/lib/rust/setup16-rust/src/setup16/main.rs)
    rust_cohab_stub.c
    )
add_dependencies(setup16 rust_setup16_rust)
target_link_libraries(setup16 ${setup16_rust_LIB})
set_module_type(setup16 win32gui UNICODE)
add_importlibs(setup16 user32 gdi32 advapi32 shell32 setupapi msvcrt kernel32)
add_cd_file(TARGET setup16 DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
