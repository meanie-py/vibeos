
# Every translation unit is Rust now (stortest-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(stortest_rust stortest-rust "" TRUE)
add_executable(stortest
    # stortest.c  # replaced by Rust (sdk/lib/rust/stortest-rust/src/stortest/stortest.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    )
add_dependencies(stortest rust_stortest_rust)
target_link_libraries(stortest ${stortest_rust_LIB})
set_module_type(stortest win32cui)
add_importlibs(stortest msvcrt kernel32 ntdll)
add_cd_file(TARGET stortest DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
