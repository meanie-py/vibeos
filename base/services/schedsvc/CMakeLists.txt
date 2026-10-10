
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/atsvc.idl)
spec2def(schedsvc.dll schedsvc.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (schedsvc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(schedsvc_rust schedsvc-rust "" TRUE)
add_library(schedsvc MODULE
    # job.c  # replaced by Rust (sdk/lib/rust/schedsvc-rust/src/schedsvc/job.rs)
    # rpcserver.c  # replaced by Rust (sdk/lib/rust/schedsvc-rust/src/schedsvc/rpcserver.rs)
    # schedsvc.c  # replaced by Rust (sdk/lib/rust/schedsvc-rust/src/schedsvc/schedsvc.rs)
    rust_cohab_stub.c
    schedsvc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/atsvc_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/schedsvc.def)
add_dependencies(schedsvc rust_schedsvc_rust)
target_link_libraries(schedsvc ${schedsvc_rust_LIB})

set_module_type(schedsvc win32dll UNICODE)
target_link_libraries(schedsvc wine)
add_importlibs(schedsvc advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_cd_file(TARGET schedsvc DESTINATION reactos/system32 FOR all)
