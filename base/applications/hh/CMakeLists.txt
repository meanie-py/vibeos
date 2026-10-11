
add_rc_deps(hh.rc ${CMAKE_CURRENT_SOURCE_DIR}/hh.ico)
# Every translation unit is Rust now (hh-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(hh_rust hh-rust "" TRUE)
add_executable(hh
    # main.c  # replaced by Rust (sdk/lib/rust/hh-rust/src/hh/main.rs)
    rust_cohab_stub.c
    hh.rc)
add_dependencies(hh rust_hh_rust)
target_link_libraries(hh ${hh_rust_LIB})
set_module_type(hh win32gui)
add_importlibs(hh gdi32 user32 msvcrt kernel32)
add_cd_file(TARGET hh DESTINATION reactos FOR all)
