
# Every translation unit is Rust now (runas-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(runas_rust runas-rust "" TRUE)
add_executable(runas
    # runas.c  # replaced by Rust (sdk/lib/rust/runas-rust/src/runas/runas.rs)
    rust_cohab_stub.c
    runas.rc)
add_dependencies(runas rust_runas_rust)
target_link_libraries(runas ${runas_rust_LIB})
set_module_type(runas win32cui UNICODE)
target_link_libraries(runas conutils)
add_importlibs(runas advapi32 msvcrt kernel32)
add_cd_file(TARGET runas DESTINATION reactos/system32 FOR all)
