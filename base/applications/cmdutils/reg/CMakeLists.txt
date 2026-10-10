
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

# Every translation unit is Rust now (reg-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(reg_rust reg-rust "" TRUE)
add_executable(reg
    # add.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/add.rs)
    # copy.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/copy.rs)
    # delete.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/delete.rs)
    # export.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/export.rs)
    # import.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/import.rs)
    # query.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/query.rs)
    # reg.c  # replaced by Rust (sdk/lib/rust/reg-rust/src/reg/reg.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    reg.rc)
add_dependencies(reg rust_reg_rust)
target_link_libraries(reg ${reg_rust_LIB})
set_module_type(reg win32cui UNICODE)
target_compile_definitions(reg PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)
target_link_libraries(reg wine oldnames)
add_importlibs(reg advapi32 advapi32_vista user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET reg DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(reg) # CORE-5743: No ARRAY_SIZE macro
