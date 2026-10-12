
# Every translation unit is Rust now (whoami-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(whoami_rust whoami-rust "" TRUE)
add_executable(whoami
    # whoami.c  # replaced by Rust (sdk/lib/rust/whoami-rust/src/whoami/whoami.rs)
    rust_cohab_stub.c
    whoami.rc)
add_dependencies(whoami rust_whoami_rust)
target_link_libraries(whoami ${whoami_rust_LIB})
set_module_type(whoami win32cui UNICODE)
target_link_libraries(whoami conutils)
add_importlibs(whoami advapi32 secur32 msvcrt kernel32)
add_cd_file(TARGET whoami DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
