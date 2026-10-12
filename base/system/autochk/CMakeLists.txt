
# Every translation unit is Rust now (autochk-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(autochk_rust autochk-rust "" TRUE)
add_executable(autochk WIN32
    # autochk.c  # replaced by Rust (sdk/lib/rust/autochk-rust/src/autochk/autochk.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    autochk.rc)
add_dependencies(autochk rust_autochk_rust)
target_link_libraries(autochk ${autochk_rust_LIB})
set_module_type(autochk nativecui)
target_link_libraries(autochk nt vfatlib vfatxlib cdfslib)
add_importlibs(autochk ntdll)
add_cd_file(TARGET autochk DESTINATION reactos/system32 FOR all)
