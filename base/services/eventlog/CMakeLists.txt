
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/eventlogrpc.idl)

list(APPEND SOURCE
    # eventlog.c  # replaced by Rust (sdk/lib/rust/eventlog-rust/src/eventlog/eventlog.rs)
    # eventsource.c  # replaced by Rust (sdk/lib/rust/eventlog-rust/src/eventlog/eventsource.rs)
    # logport.c  # replaced by Rust (sdk/lib/rust/eventlog-rust/src/eventlog/logport.rs)
    # rpc.c  # replaced by Rust (sdk/lib/rust/eventlog-rust/src/eventlog/rpc.rs)
    # file.c  # replaced by Rust (sdk/lib/rust/eventlog-rust/src/eventlog/file.rs)
    rust_cohab_stub.c
    eventlog.h
    ${CMAKE_CURRENT_BINARY_DIR}/eventlogrpc_s.c)

# Every translation unit is Rust now (eventlog-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(eventlog_rust eventlog-rust "" TRUE)
add_executable(eventlog ${SOURCE} eventlog.rc)
add_dependencies(eventlog rust_eventlog_rust)
target_link_libraries(eventlog ${eventlog_rust_LIB})
add_pch(eventlog eventlog.h SOURCE)

target_link_libraries(eventlog ${PSEH_LIB})

set_module_type(eventlog win32cui UNICODE)
target_link_libraries(eventlog evtlib)
add_importlibs(eventlog advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_cd_file(TARGET eventlog DESTINATION reactos/system32 FOR all)
