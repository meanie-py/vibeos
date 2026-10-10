
file(GLOB mplay32_rc_deps resources/*.*)
add_rc_deps(mplay32.rc ${mplay32_rc_deps})
# Every translation unit is Rust now (mplay32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mplay32_rust mplay32-rust "" TRUE)
add_executable(mplay32
    # mplay32.c  # replaced by Rust (sdk/lib/rust/mplay32-rust/src/mplay32/mplay32.rs)
    rust_cohab_stub.c
    mplay32.rc)
add_dependencies(mplay32 rust_mplay32_rust)
target_link_libraries(mplay32 ${mplay32_rust_LIB})
set_module_type(mplay32 win32gui UNICODE)
add_importlibs(mplay32 advapi32 comctl32 comdlg32 user32 gdi32 winmm shell32 msvcrt kernel32)
add_cd_file(TARGET mplay32 DESTINATION reactos/system32 FOR all)
