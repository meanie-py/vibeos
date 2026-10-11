
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/netlogon.idl)
spec2def(netlogon.dll netlogon.spec ADD_IMPORTLIB)

add_definitions(-DWIN32_LEAN_AND_MEAN=)

list(APPEND SOURCE
    # netlogon.c  # replaced by Rust (sdk/lib/rust/netlogon-rust/src/netlogon/netlogon.rs)
    # rpcserver.c  # replaced by Rust (sdk/lib/rust/netlogon-rust/src/netlogon/rpcserver.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (netlogon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netlogon_rust netlogon-rust "" TRUE)
add_library(netlogon MODULE
    ${SOURCE}
    netlogon.rc
    ${CMAKE_CURRENT_BINARY_DIR}/netlogon_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/netlogon_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/netlogon.def)
add_dependencies(netlogon rust_netlogon_rust)
target_link_libraries(netlogon ${netlogon_rust_LIB})

set_module_type(netlogon win32dll UNICODE)
target_link_libraries(netlogon wine ${PSEH_LIB})
add_importlibs(netlogon advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_pch(netlogon precomp.h SOURCE)
add_cd_file(TARGET netlogon DESTINATION reactos/system32 FOR all)
