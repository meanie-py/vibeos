
list(APPEND SOURCE
    # display.c  # replaced by Rust (sdk/lib/rust/fontview-rust/src/fontview/display.rs)
    # fontview.c  # replaced by Rust (sdk/lib/rust/fontview-rust/src/fontview/fontview.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

add_rc_deps(fontview.rc ${CMAKE_CURRENT_SOURCE_DIR}/ttf.ico)
# Every translation unit is Rust now (fontview-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fontview_rust fontview-rust "" TRUE)
add_executable(fontview ${SOURCE} fontview.rc)
add_dependencies(fontview rust_fontview_rust)
target_link_libraries(fontview ${fontview_rust_LIB})
set_module_type(fontview win32gui UNICODE)
add_importlibs(fontview comdlg32 gdi32 shell32 user32 msvcrt kernel32 advapi32)
add_pch(fontview precomp.h SOURCE)
add_cd_file(TARGET fontview DESTINATION reactos/system32 FOR all)
