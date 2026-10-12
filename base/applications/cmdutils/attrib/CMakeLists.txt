
# Every translation unit is Rust now (attrib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(attrib_rust attrib-rust "" TRUE)
add_executable(attrib
    # attrib.c  # replaced by Rust (sdk/lib/rust/attrib-rust/src/attrib/attrib.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    attrib.rc)
add_dependencies(attrib rust_attrib_rust)
target_link_libraries(attrib ${attrib_rust_LIB})
set_module_type(attrib win32cui UNICODE)
target_link_libraries(attrib conutils)
add_importlibs(attrib msvcrt kernel32)
add_cd_file(TARGET attrib DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
