
spec2def(netsh.exe netsh.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # alias.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/alias.rs)
    # context.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/context.rs)
    # help.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/help.rs)
    # helper.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/helper.rs)
    # interpreter.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/interpreter.rs)
    # netsh.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/netsh.rs)
    # wmi.c  # replaced by Rust (sdk/lib/rust/netsh-rust/src/netsh/wmi.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h
    ${CMAKE_CURRENT_BINARY_DIR}/netsh_stubs.c)

# Every translation unit is Rust now (netsh-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netsh_rust netsh-rust "" TRUE)
add_executable(netsh ${SOURCE} netsh.rc ${CMAKE_CURRENT_BINARY_DIR}/netsh.def)
add_dependencies(netsh rust_netsh_rust)
target_link_libraries(netsh ${netsh_rust_LIB})
add_pch(netsh precomp.h SOURCE)

set_target_properties(netsh
    PROPERTIES
    ENABLE_EXPORTS TRUE
    DEFINE_SYMBOL "")

set_module_type(netsh win32cui UNICODE)
target_link_libraries(netsh conutils)
add_importlibs(netsh ole32 oleaut32 iphlpapi advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET netsh DESTINATION reactos/system32 FOR all)
