# Every translation unit is Rust now (testgui-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(testgui_rust testgui-rust "" TRUE)
add_executable(testgui WIN32
    # testgui.c  # replaced by Rust (sdk/lib/rust/testgui-rust/src/testgui/testgui.rs)
    rust_cohab_stub.c
    )
add_dependencies(testgui rust_testgui_rust)
target_link_libraries(testgui ${testgui_rust_LIB})
set_module_type(testgui win32gui UNICODE)
add_importlibs(testgui user32 gdi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET testgui DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
