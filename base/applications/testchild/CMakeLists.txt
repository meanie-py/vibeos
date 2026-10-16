# Every translation unit is Rust now (testchild-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(testchild_rust testchild-rust "" TRUE)
add_executable(testchild
    # testchild.c  # replaced by Rust (sdk/lib/rust/testchild-rust/src/testchild/testchild.rs)
    rust_cohab_stub.c
    )
add_dependencies(testchild rust_testchild_rust)
target_link_libraries(testchild ${testchild_rust_LIB})
set_module_type(testchild win32cui)
add_importlibs(testchild msvcrt kernel32 ntdll)
add_cd_file(TARGET testchild DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
