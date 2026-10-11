
# Every translation unit is Rust now (taskkill-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(taskkill_rust taskkill-rust "" TRUE)
add_executable(taskkill
    # taskkill.c  # replaced by Rust (sdk/lib/rust/taskkill-rust/src/taskkill/taskkill.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    taskkill.rc)
add_dependencies(taskkill rust_taskkill_rust)
target_link_libraries(taskkill ${taskkill_rust_LIB})
target_link_libraries(taskkill wine)
set_module_type(taskkill win32cui UNICODE)
add_importlibs(taskkill user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET taskkill DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(taskkill) # CORE-5743: No ARRAY_SIZE macro
