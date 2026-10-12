
add_rc_deps(resource.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/regedt32.ico)
# Every translation unit is Rust now (regedt32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(regedt32_rust regedt32-rust "" TRUE)
add_executable(regedt32
    # regedt32.c  # replaced by Rust (sdk/lib/rust/regedt32-rust/src/regedt32/regedt32.rs)
    rust_cohab_stub.c
    resource.rc)
add_dependencies(regedt32 rust_regedt32_rust)
target_link_libraries(regedt32 ${regedt32_rust_LIB})
set_module_type(regedt32 win32gui UNICODE)
add_importlibs(regedt32 shell32 shlwapi msvcrt kernel32)
add_cd_file(TARGET regedt32 DESTINATION reactos/system32 FOR all)
