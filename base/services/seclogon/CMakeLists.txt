
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/seclogon.idl)
spec2def(seclogon.dll seclogon.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # rpcserver.c  # replaced by Rust (sdk/lib/rust/seclogon-rust/src/seclogon/rpcserver.rs)
    # seclogon.c  # replaced by Rust (sdk/lib/rust/seclogon-rust/src/seclogon/seclogon.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (seclogon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(seclogon_rust seclogon-rust "" TRUE)
add_library(seclogon MODULE
    ${SOURCE}
    seclogon.rc
    ${CMAKE_CURRENT_BINARY_DIR}/seclogon_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/seclogon_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/seclogon.def)
add_dependencies(seclogon rust_seclogon_rust)
target_link_libraries(seclogon ${seclogon_rust_LIB})

set_module_type(seclogon win32dll UNICODE)
target_link_libraries(seclogon wine ${PSEH_LIB})
add_importlibs(seclogon userenv advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_pch(seclogon precomp.h SOURCE)
add_cd_file(TARGET seclogon DESTINATION reactos/system32 FOR all)
