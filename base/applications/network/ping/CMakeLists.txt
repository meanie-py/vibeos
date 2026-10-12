
# Every translation unit is Rust now (ping-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ping_rust ping-rust "" TRUE)
add_executable(ping
    # ping.c  # replaced by Rust (sdk/lib/rust/ping-rust/src/ping/ping.rs)
    rust_cohab_stub.c
    ping.rc)
add_dependencies(ping rust_ping_rust)
target_link_libraries(ping ${ping_rust_LIB})
set_module_type(ping win32cui UNICODE)
target_link_libraries(ping conutils ${PSEH_LIB})
add_importlibs(ping ws2_32 iphlpapi msvcrt kernel32 ntdll)
add_cd_file(TARGET ping DESTINATION reactos/system32 FOR all)
