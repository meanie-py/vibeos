
# Every translation unit is Rust now (mofcomp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mofcomp_rust mofcomp-rust "" TRUE)
add_executable(mofcomp
    # mofcomp.c  # replaced by Rust (sdk/lib/rust/mofcomp-rust/src/mofcomp/mofcomp.rs)
    rust_cohab_stub.c
    )
add_dependencies(mofcomp rust_mofcomp_rust)
target_link_libraries(mofcomp ${mofcomp_rust_LIB})
target_link_libraries(mofcomp wine)
set_module_type(mofcomp win32cui UNICODE)
add_importlibs(mofcomp msvcrt kernel32 ntdll)
add_cd_file(TARGET mofcomp DESTINATION reactos/system32/wbem FOR all)
