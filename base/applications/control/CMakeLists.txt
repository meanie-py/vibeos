
add_rc_deps(control.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/config.ico)
# Every translation unit is Rust now (control-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(control_rust control-rust "" TRUE)
add_executable(control
    # control.c  # replaced by Rust (sdk/lib/rust/control-rust/src/control/control.rs)
    rust_cohab_stub.c
    control.rc)
add_dependencies(control rust_control_rust)
target_link_libraries(control ${control_rust_LIB})
set_module_type(control win32gui UNICODE)
add_delay_importlibs(control ole32)
target_link_libraries(control uuid)
add_importlibs(control advapi32 shell32 shlwapi user32 msvcrt kernel32)
add_cd_file(TARGET control DESTINATION reactos/system32 FOR all)
