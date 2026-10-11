
add_message_headers(UNICODE arp_msg.mc)
# Every translation unit is Rust now (arp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(arp_rust arp-rust "" TRUE)
add_executable(arp
    # arp.c  # replaced by Rust (sdk/lib/rust/arp-rust/src/arp/arp.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    arp.rc)
add_dependencies(arp rust_arp_rust)
target_link_libraries(arp ${arp_rust_LIB})
set_module_type(arp win32cui)
add_dependencies(arp arp_msg)
add_importlibs(arp iphlpapi ws2_32 shlwapi msvcrt kernel32)
add_cd_file(TARGET arp DESTINATION reactos/system32 FOR all)
