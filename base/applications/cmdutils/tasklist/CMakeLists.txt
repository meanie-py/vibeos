
# Every translation unit is Rust now (tasklist-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tasklist_rust tasklist-rust "" TRUE)
add_executable(tasklist
    # tasklist.c  # replaced by Rust (sdk/lib/rust/tasklist-rust/src/tasklist/tasklist.rs)
    rust_cohab_stub.c
    tasklist.rc)
add_dependencies(tasklist rust_tasklist_rust)
target_link_libraries(tasklist ${tasklist_rust_LIB})
set_module_type(tasklist win32cui UNICODE)
target_link_libraries(tasklist conutils)
add_importlibs(tasklist msvcrt kernel32 ntdll)
add_cd_file(TARGET tasklist DESTINATION reactos/system32 FOR all)
