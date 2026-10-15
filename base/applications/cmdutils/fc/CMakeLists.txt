
# Every translation unit is Rust now (fc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fc_rust fc-rust "" TRUE)
add_executable(fc
    # fc.c  # replaced by Rust (sdk/lib/rust/fc-rust/src/fc/fc.rs)
    # texta.c  # replaced by Rust (sdk/lib/rust/fc-rust/src/fc/texta.rs)
    # textw.c  # replaced by Rust (sdk/lib/rust/fc-rust/src/fc/textw.rs)
    rust_cohab_stub.c
    fc.rc)
add_dependencies(fc rust_fc_rust)
target_link_libraries(fc ${fc_rust_LIB})
set_module_type(fc win32cui UNICODE)
target_link_libraries(fc conutils)
add_importlibs(fc shlwapi msvcrt kernel32)
add_cd_file(TARGET fc DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
