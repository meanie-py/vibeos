
# Every translation unit is Rust now (subst-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(subst_rust subst-rust "" TRUE)
add_executable(subst
    # subst.c  # replaced by Rust (sdk/lib/rust/subst-rust/src/subst/subst.rs)
    rust_cohab_stub.c
    subst.rc)
add_dependencies(subst rust_subst_rust)
target_link_libraries(subst ${subst_rust_LIB})
set_module_type(subst win32cui UNICODE)
target_link_libraries(subst conutils)
add_importlibs(subst msvcrt kernel32)
add_cd_file(TARGET subst DESTINATION reactos/system32 FOR all)
