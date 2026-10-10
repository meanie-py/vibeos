
# Every translation unit is Rust now (logoff-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(logoff_rust logoff-rust "" TRUE)
add_executable(logoff
    # logoff.c  # replaced by Rust (sdk/lib/rust/logoff-rust/src/logoff/logoff.rs)
    rust_cohab_stub.c
    logoff.rc)
add_dependencies(logoff rust_logoff_rust)
target_link_libraries(logoff ${logoff_rust_LIB})
set_module_type(logoff win32cui UNICODE)
target_link_libraries(logoff conutils)
add_importlibs(logoff user32 msvcrt kernel32)
add_cd_file(TARGET logoff DESTINATION reactos/system32 FOR all)
