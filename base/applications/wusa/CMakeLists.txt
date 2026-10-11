
add_rc_deps(resource.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/setup.ico)
# Every translation unit is Rust now (wusa-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wusa_rust wusa-rust "" TRUE)
add_executable(wusa
    # wusa.c  # replaced by Rust (sdk/lib/rust/wusa-rust/src/wusa/wusa.rs)
    rust_cohab_stub.c
    resource.rc)
add_dependencies(wusa rust_wusa_rust)
target_link_libraries(wusa ${wusa_rust_LIB})
set_module_type(wusa win32gui UNICODE)
add_importlibs(wusa shell32 msvcrt kernel32)
add_cd_file(TARGET wusa DESTINATION reactos/system32 FOR all)
