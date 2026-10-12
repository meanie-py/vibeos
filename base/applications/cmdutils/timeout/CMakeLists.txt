
# Every translation unit is Rust now (timeout-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(timeout_rust timeout-rust "" TRUE)
add_executable(timeout
    # timeout.c  # replaced by Rust (sdk/lib/rust/timeout-rust/src/timeout/timeout.rs)
    rust_cohab_stub.c
    timeout.rc)
add_dependencies(timeout rust_timeout_rust)
target_link_libraries(timeout ${timeout_rust_LIB})
set_module_type(timeout win32cui UNICODE)
target_link_libraries(timeout conutils)
add_importlibs(timeout msvcrt kernel32)
add_cd_file(TARGET timeout DESTINATION reactos/system32 FOR all)
