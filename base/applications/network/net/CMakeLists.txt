
add_definitions(-D__USE_W32_SOCKETS)

list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/main.rs)
    # cmdAccounts.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdAccounts.rs)
    # cmdComputer.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdComputer.rs)
    # cmdConfig.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdConfig.rs)
    # cmdContinue.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdContinue.rs)
    # cmdGroup.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdGroup.rs)
    # cmdHelp.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdHelp.rs)
    # cmdHelpMsg.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdHelpMsg.rs)
    # cmdLocalGroup.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdLocalGroup.rs)
    # cmdPause.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdPause.rs)
    # cmdSession.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdSession.rs)
    # cmdShare.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdShare.rs)
    # cmdStart.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdStart.rs)
    # cmdStatistics.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdStatistics.rs)
    # cmdStop.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdStop.rs)
    # cmdUse.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdUse.rs)
    # cmdUser.c  # replaced by Rust (sdk/lib/rust/net-rust/src/net/cmdUser.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    net.h)

# Every translation unit is Rust now (net-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(net_rust net-rust "" TRUE)
add_executable(net ${SOURCE} net.rc)
add_dependencies(net rust_net_rust)
target_link_libraries(net ${net_rust_LIB})
add_pch(net net.h SOURCE)
set_module_type(net win32cui UNICODE)
target_link_libraries(net conutils)
add_dependencies(net net_msg)
add_importlibs(net advapi32 netapi32 mpr msvcrt kernel32 ntdll)
add_cd_file(TARGET net DESTINATION reactos/system32 FOR all)
