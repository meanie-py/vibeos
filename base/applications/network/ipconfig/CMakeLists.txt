
# Every translation unit is Rust now (ipconfig-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ipconfig_rust ipconfig-rust "" TRUE)
add_executable(ipconfig
    # ipconfig.c  # replaced by Rust (sdk/lib/rust/ipconfig-rust/src/ipconfig/ipconfig.rs)
    rust_cohab_stub.c
    ipconfig.rc)
add_dependencies(ipconfig rust_ipconfig_rust)
target_link_libraries(ipconfig ${ipconfig_rust_LIB})
set_module_type(ipconfig win32cui UNICODE)
target_link_libraries(ipconfig conutils)
add_importlibs(ipconfig iphlpapi dnsapi dhcpcsvc advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET ipconfig DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
