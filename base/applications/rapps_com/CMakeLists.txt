project(rapps)

# Every translation unit is Rust now (rapps_com-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(rapps_com_rust rapps_com-rust "" TRUE)
add_executable(rapps.com
    # main.c  # replaced by Rust (sdk/lib/rust/rapps_com-rust/src/rapps_com/main.rs)
    rust_cohab_stub.c
    rapps.rc)
add_dependencies(rapps.com rust_rapps_com_rust)
target_link_libraries(rapps.com ${rapps_com_rust_LIB})
set_module_type(rapps.com win32cui UNICODE)
set_target_properties(rapps.com PROPERTIES SUFFIX "")
add_importlibs(rapps.com msvcrt kernel32 ntdll)
add_cd_file(TARGET rapps.com DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
