
list(APPEND SOURCE
    # gui.c  # replaced by Rust (sdk/lib/rust/shutdown-rust/src/shutdown/gui.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/shutdown-rust/src/shutdown/misc.rs)
    # shutdown.c  # replaced by Rust (sdk/lib/rust/shutdown-rust/src/shutdown/shutdown.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (shutdown-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(shutdown_rust shutdown-rust "" TRUE)
add_executable(shutdown ${SOURCE} shutdown.rc)
add_dependencies(shutdown rust_shutdown_rust)
target_link_libraries(shutdown ${shutdown_rust_LIB})
add_pch(shutdown precomp.h SOURCE)
set_module_type(shutdown win32cui UNICODE)
target_link_libraries(shutdown conutils)
add_importlibs(shutdown advapi32 powrprof user32 msvcrt kernel32)
add_cd_file(TARGET shutdown DESTINATION reactos/system32 FOR all)
