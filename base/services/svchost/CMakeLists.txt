
list(APPEND SOURCE
    # globals.c  # replaced by Rust (sdk/lib/rust/svchost-rust/src/svchost/globals.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/svchost-rust/src/svchost/registry.rs)
    # rpcsrv.c  # replaced by Rust (sdk/lib/rust/svchost-rust/src/svchost/rpcsrv.rs)
    # netbios.c  # replaced by Rust (sdk/lib/rust/svchost-rust/src/svchost/netbios.rs)
    # svchost.c  # replaced by Rust (sdk/lib/rust/svchost-rust/src/svchost/svchost.rs)
    rust_cohab_stub.c
    )

list(APPEND PCH_SKIP_SOURCE
    security.cxx)

# Every translation unit is Rust now (svchost-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(svchost_rust svchost-rust "" TRUE)
add_executable(svchost
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    svchost.rc)
add_dependencies(svchost rust_svchost_rust)
target_link_libraries(svchost ${svchost_rust_LIB})

target_link_libraries(svchost uuid)
set_module_type(svchost win32cui UNICODE)
add_delay_importlibs(svchost netapi32 ole32)
add_importlibs(svchost advapi32 rpcrt4 kernel32 ntdll)
add_pch(svchost svchost.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET svchost DESTINATION reactos/system32 FOR all)
