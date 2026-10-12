
spec2def(serialui.dll serialui.spec)

list(APPEND SOURCE
    # serialui.c  # replaced by Rust (sdk/lib/rust/serialui-rust/src/serialui/serialui.rs)
    rust_cohab_stub.c
    serialui.rc
    ${CMAKE_CURRENT_BINARY_DIR}/serialui.def)

# Every translation unit is Rust now (serialui-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(serialui_rust serialui-rust "" TRUE)
add_library(serialui MODULE ${SOURCE})
add_dependencies(serialui rust_serialui_rust)
target_link_libraries(serialui ${serialui_rust_LIB})
set_module_type(serialui win32dll UNICODE)
add_importlibs(serialui user32 shlwapi msvcrt kernel32)
add_cd_file(TARGET serialui DESTINATION reactos/system32 FOR all)
