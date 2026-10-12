
# Every translation unit is Rust now (chcp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(chcp_rust chcp-rust "" TRUE)
add_executable(chcp
    # chcp.c  # replaced by Rust (sdk/lib/rust/chcp-rust/src/chcp/chcp.rs)
    rust_cohab_stub.c
    chcp.rc)
add_dependencies(chcp rust_chcp_rust)
target_link_libraries(chcp ${chcp_rust_LIB})
set_module_type(chcp win32cui UNICODE)
target_link_libraries(chcp conutils)
add_importlibs(chcp msvcrt kernel32)
set_target_properties(chcp PROPERTIES SUFFIX ".com")
add_cd_file(TARGET chcp DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
