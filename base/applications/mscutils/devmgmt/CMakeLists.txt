
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/dll/devmgr)

# Every translation unit is Rust now (devmgmt-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(devmgmt_rust devmgmt-rust "" TRUE)
add_executable(devmgmt
    # devmgmt.c  # replaced by Rust (sdk/lib/rust/devmgmt-rust/src/devmgmt/devmgmt.rs)
    rust_cohab_stub.c
    devmgmt.rc)
add_dependencies(devmgmt rust_devmgmt_rust)
target_link_libraries(devmgmt ${devmgmt_rust_LIB})
set_module_type(devmgmt win32gui UNICODE)
add_importlibs(devmgmt devmgr msvcrt kernel32)
add_cd_file(TARGET devmgmt DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
