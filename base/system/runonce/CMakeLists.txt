
add_rc_deps(runonce.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/icon.ico)
# Every translation unit is Rust now (runonce-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(runonce_rust runonce-rust "" TRUE)
add_executable(runonce
    # runonce.c  # replaced by Rust (sdk/lib/rust/runonce-rust/src/runonce/runonce.rs)
    rust_cohab_stub.c
    runonce.rc)
add_dependencies(runonce rust_runonce_rust)
target_link_libraries(runonce ${runonce_rust_LIB})
set_module_type(runonce win32gui UNICODE)
add_importlibs(runonce advapi32 user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET runonce DESTINATION reactos/system32 FOR all)
