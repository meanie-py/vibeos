
# Every translation unit is Rust now (findstr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(findstr_rust findstr-rust "" TRUE)
add_executable(findstr
    # findstr.c  # replaced by Rust (sdk/lib/rust/findstr-rust/src/findstr/findstr.rs)
    rust_cohab_stub.c
    findstr.rc)
add_dependencies(findstr rust_findstr_rust)
target_link_libraries(findstr ${findstr_rust_LIB})
set_module_type(findstr win32cui)
add_importlibs(findstr user32 msvcrt kernel32)
add_cd_file(TARGET findstr DESTINATION reactos/system32 FOR all)
