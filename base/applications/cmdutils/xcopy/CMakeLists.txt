
# Every translation unit is Rust now (xcopy-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(xcopy_rust xcopy-rust "" TRUE)
add_executable(xcopy
    # xcopy.c  # replaced by Rust (sdk/lib/rust/xcopy-rust/src/xcopy/xcopy.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    xcopy.rc)
add_dependencies(xcopy rust_xcopy_rust)
target_link_libraries(xcopy ${xcopy_rust_LIB})
target_link_libraries(xcopy wine)
set_module_type(xcopy win32cui UNICODE)
add_importlibs(xcopy shell32 user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET xcopy DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(xcopy) # CORE-5743: No ARRAY_SIZE macro
