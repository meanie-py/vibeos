
file(GLOB logon_rc_deps res/*.*)
add_rc_deps(logon.rc ${logon_rc_deps})
# Every translation unit is Rust now (logon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(logon_rust logon-rust "" TRUE)
add_executable(logon
    # logon.c  # replaced by Rust (sdk/lib/rust/logon-rust/src/logon/logon.rs)
    rust_cohab_stub.c
    logon.rc)
add_dependencies(logon rust_logon_rust)
target_link_libraries(logon ${logon_rust_LIB})
set_module_type(logon win32gui UNICODE)
set_target_properties(logon PROPERTIES SUFFIX ".scr")
target_link_libraries(logon scrnsave)
add_importlibs(logon user32 gdi32 msvcrt kernel32)
add_cd_file(TARGET logon DESTINATION reactos/system32 FOR all)
