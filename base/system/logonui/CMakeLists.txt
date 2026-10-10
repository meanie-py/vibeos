
# Every translation unit is Rust now (logonui-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(logonui_rust logonui-rust "" TRUE)
add_executable(logonui
    # logonui.c  # replaced by Rust (sdk/lib/rust/logonui-rust/src/logonui/logonui.rs)
    # NT5design.c  # replaced by Rust (sdk/lib/rust/logonui-rust/src/logonui/NT5design.rs)
    rust_cohab_stub.c
    logonui.rc)
add_dependencies(logonui rust_logonui_rust)
target_link_libraries(logonui ${logonui_rust_LIB})
set_module_type(logonui win32gui UNICODE)
add_importlibs(logonui gdi32 user32 msimg32 shell32 msvcrt kernel32)
add_cd_file(TARGET logonui DESTINATION reactos/system32 FOR all)
