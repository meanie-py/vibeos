
# Every translation unit is Rust now (lodctr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(lodctr_rust lodctr-rust "" TRUE)
add_executable(lodctr
    # lodctr_main.c  # replaced by Rust (sdk/lib/rust/lodctr-rust/src/lodctr/lodctr_main.rs)
    rust_cohab_stub.c
    )
add_dependencies(lodctr rust_lodctr_rust)
target_link_libraries(lodctr ${lodctr_rust_LIB})
set_module_type(lodctr win32cui UNICODE)
add_importlibs(lodctr loadperf msvcrt kernel32)
add_cd_file(TARGET lodctr DESTINATION reactos/system32 FOR all)
