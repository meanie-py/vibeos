
# Every translation unit is Rust now (comp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(comp_rust comp-rust "" TRUE)
add_executable(comp
    # comp.c  # replaced by Rust (sdk/lib/rust/comp-rust/src/comp/comp.rs)
    rust_cohab_stub.c
    comp.rc)
add_dependencies(comp rust_comp_rust)
target_link_libraries(comp ${comp_rust_LIB})
set_module_type(comp win32cui UNICODE)
target_link_libraries(comp conutils)
add_importlibs(comp msvcrt kernel32)
add_cd_file(TARGET comp DESTINATION reactos/system32 FOR all)
