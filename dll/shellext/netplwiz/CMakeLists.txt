
spec2def(netplwiz.dll netplwiz.spec)

list(APPEND SOURCE
    # netplwiz.c  # replaced by Rust (sdk/lib/rust/netplwiz-rust/src/netplwiz/netplwiz.rs)
    # SHDisconnectNetDrives.c  # replaced by Rust (sdk/lib/rust/netplwiz-rust/src/netplwiz/SHDisconnectNetDrives.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (netplwiz-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netplwiz_rust netplwiz-rust "" TRUE)
add_library(netplwiz MODULE
    ${SOURCE}
    netplwiz.rc
    ${CMAKE_CURRENT_BINARY_DIR}/netplwiz_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/netplwiz.def)
add_dependencies(netplwiz rust_netplwiz_rust)
target_link_libraries(netplwiz ${netplwiz_rust_LIB})

set_module_type(netplwiz win32dll UNICODE)
target_link_libraries(netplwiz wine)
add_importlibs(netplwiz comctl32 mpr ole32 shell32 user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET netplwiz DESTINATION reactos/system32 FOR all)
