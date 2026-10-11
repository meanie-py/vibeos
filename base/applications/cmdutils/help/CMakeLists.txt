
# Every translation unit is Rust now (cmd_help-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(cmd_help_rust cmd_help-rust "" TRUE)
add_executable(cmd_help
    # help.c  # replaced by Rust (sdk/lib/rust/cmd_help-rust/src/cmd_help/help.rs)
    rust_cohab_stub.c
    help.rc)
add_dependencies(cmd_help rust_cmd_help_rust)
target_link_libraries(cmd_help ${cmd_help_rust_LIB})
set_module_type(cmd_help win32cui UNICODE)
target_link_libraries(cmd_help conutils)
add_importlibs(cmd_help msvcrt kernel32)
set_target_properties(cmd_help PROPERTIES OUTPUT_NAME "help")
add_cd_file(TARGET cmd_help DESTINATION reactos/system32 FOR all)
