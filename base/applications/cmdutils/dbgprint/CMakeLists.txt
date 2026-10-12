
# Every translation unit is Rust now (dbgprint_app-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dbgprint_app_rust dbgprint_app-rust "" TRUE)
add_executable(dbgprint
    # dbgprint.c  # replaced by Rust (sdk/lib/rust/dbgprint_app-rust/src/dbgprint_app/dbgprint.rs)
    rust_cohab_stub.c
    )
add_dependencies(dbgprint rust_dbgprint_app_rust)
target_link_libraries(dbgprint ${dbgprint_app_rust_LIB})
set_module_type(dbgprint win32cui)
add_importlibs(dbgprint msvcrt kernel32)
add_cd_file(TARGET dbgprint DESTINATION reactos/system32 FOR all)
