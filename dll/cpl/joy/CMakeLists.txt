
spec2def(joy.cpl joy.spec)
add_rc_deps(joy.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/applet.ico)

# Every translation unit is Rust now (joy-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(joy_rust joy-rust "" TRUE)
add_library(joy MODULE
    # joy.c  # replaced by Rust (sdk/lib/rust/joy-rust/src/joy/joy.rs)
    rust_cohab_stub.c
    joy.rc
    ${CMAKE_CURRENT_BINARY_DIR}/joy.def)
add_dependencies(joy rust_joy_rust)
target_link_libraries(joy ${joy_rust_LIB})

set_module_type(joy cpl UNICODE)
target_link_libraries(joy uuid)
add_importlibs(joy user32 comctl32 ole32 shell32 msvcrt kernel32 ntdll)
add_cd_file(TARGET joy DESTINATION reactos/system32 FOR all)
