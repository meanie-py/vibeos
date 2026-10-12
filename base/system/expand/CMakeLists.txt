
# Every translation unit is Rust now (expand-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(expand_rust expand-rust "" TRUE)
add_executable(expand
    # expand.c  # replaced by Rust (sdk/lib/rust/expand-rust/src/expand/expand.rs)
    rust_varargs_shim.c
    rust_cohab_stub.c
    expand.rc)
add_dependencies(expand rust_expand_rust)
target_link_libraries(expand ${expand_rust_LIB})
set_module_type(expand win32cui)
add_importlibs(expand lz32 setupapi user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET expand DESTINATION reactos/system32 FOR all)
