
# Every translation unit is Rust now (netstat-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netstat_rust netstat-rust "" TRUE)
add_executable(netstat
    # netstat.c  # replaced by Rust (sdk/lib/rust/netstat-rust/src/netstat/netstat.rs)
    rust_cohab_stub.c
    netstat.rc)
add_dependencies(netstat rust_netstat_rust)
target_link_libraries(netstat ${netstat_rust_LIB})
set_module_type(netstat win32cui UNICODE)
target_link_libraries(netstat conutils)
add_importlibs(netstat ws2_32 snmpapi iphlpapi msvcrt kernel32)
add_cd_file(TARGET netstat DESTINATION reactos/system32 FOR all)
