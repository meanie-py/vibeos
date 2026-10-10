
# Every translation unit is Rust now (extrac32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(extrac32_rust extrac32-rust "" TRUE)
add_executable(extrac32
    # extrac32.c  # replaced by Rust (sdk/lib/rust/extrac32-rust/src/extrac32/extrac32.rs)
    rust_cohab_stub.c
    )
add_dependencies(extrac32 rust_extrac32_rust)
target_link_libraries(extrac32 ${extrac32_rust_LIB})
target_link_libraries(extrac32 wine)
set_module_type(extrac32 win32gui UNICODE)
add_importlibs(extrac32 shell32 setupapi shlwapi user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET extrac32 DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(extrac32) # CORE-5743: No ARRAY_SIZE macro
