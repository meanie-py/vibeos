
spec2def(ifmon.dll ifmon.spec)

list(APPEND SOURCE
    # guid.c  # replaced by Rust (sdk/lib/rust/ifmon-rust/src/ifmon/guid.rs)
    # ifmon.c  # replaced by Rust (sdk/lib/rust/ifmon-rust/src/ifmon/ifmon.rs)
    ifmon.rc
    # interface.c  # replaced by Rust (sdk/lib/rust/ifmon-rust/src/ifmon/interface.rs)
    # ip.c  # replaced by Rust (sdk/lib/rust/ifmon-rust/src/ifmon/ip.rs)
    # winsock.c  # replaced by Rust (sdk/lib/rust/ifmon-rust/src/ifmon/winsock.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/ifmon.def)

# Every translation unit is Rust now (ifmon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ifmon_rust ifmon-rust "" TRUE)
add_library(ifmon MODULE ${SOURCE})
add_dependencies(ifmon rust_ifmon_rust)
target_link_libraries(ifmon ${ifmon_rust_LIB})
set_module_type(ifmon win32dll UNICODE)
target_link_libraries(ifmon uuid)
add_importlibs(ifmon netsh ole32 iphlpapi msvcrt kernel32 ntdll)
add_cd_file(TARGET ifmon DESTINATION reactos/system32 FOR all)
