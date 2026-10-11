
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/srvsvc.idl)
spec2def(srvsvc.dll srvsvc.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (srvsvc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(srvsvc_rust srvsvc-rust "" TRUE)
add_library(srvsvc MODULE
    # rpcserver.c  # replaced by Rust (sdk/lib/rust/srvsvc-rust/src/srvsvc/rpcserver.rs)
    # srvsvc.c  # replaced by Rust (sdk/lib/rust/srvsvc-rust/src/srvsvc/srvsvc.rs)
    rust_cohab_stub.c
    srvsvc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/srvsvc_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/srvsvc.def)
add_dependencies(srvsvc rust_srvsvc_rust)
target_link_libraries(srvsvc ${srvsvc_rust_LIB})

set_module_type(srvsvc win32dll UNICODE)
target_link_libraries(srvsvc wine)
add_importlibs(srvsvc advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_cd_file(TARGET srvsvc DESTINATION reactos/system32 FOR all)
