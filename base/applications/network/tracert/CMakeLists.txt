
# tracert.cpp is replaced by Rust (sdk/lib/rust/tracert-rust/src/tracert/tracert.rs, `rustconv
# port`); the C++ stays on disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tracert_rust tracert-rust "" TRUE)
add_executable(tracert rust_cohab_stub.c tracert.rc)
add_dependencies(tracert rust_tracert_rust)
target_link_libraries(tracert ${tracert_rust_LIB})
set_module_type(tracert win32cui UNICODE)
target_link_libraries(tracert conutils)
add_importlibs(tracert ws2_32 iphlpapi msvcrt kernel32)
add_cd_file(TARGET tracert DESTINATION reactos/system32 FOR all)
