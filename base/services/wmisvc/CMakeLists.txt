spec2def(wmisvc.dll wmisvc.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (wmisvc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wmisvc_rust wmisvc-rust "" TRUE)
add_library(wmisvc MODULE
    # wmisvc.c  # replaced by Rust (sdk/lib/rust/wmisvc-rust/src/wmisvc/wmisvc.rs)
    rust_cohab_stub.c
    wmisvc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wmisvc.def)
add_dependencies(wmisvc rust_wmisvc_rust)
target_link_libraries(wmisvc ${wmisvc_rust_LIB})

set_module_type(wmisvc win32dll UNICODE)
add_importlibs(wmisvc advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wmisvc DESTINATION reactos/system32/wbem FOR all)
