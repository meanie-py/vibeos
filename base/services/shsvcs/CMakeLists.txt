
spec2def(shsvcs.dll shsvcs.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (shsvcs-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(shsvcs_rust shsvcs-rust "" TRUE)
add_library(shsvcs MODULE
    # shsvcs.c  # replaced by Rust (sdk/lib/rust/shsvcs-rust/src/shsvcs/shsvcs.rs)
    # thmsvc.c  # replaced by Rust (sdk/lib/rust/shsvcs-rust/src/shsvcs/thmsvc.rs)
    # thmserver.c  # replaced by Rust (sdk/lib/rust/shsvcs-rust/src/shsvcs/thmserver.rs)
    rust_cohab_stub.c
    shsvcs.rc
    ${CMAKE_CURRENT_BINARY_DIR}/shsvcs_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/shsvcs.def)
add_dependencies(shsvcs rust_shsvcs_rust)
target_link_libraries(shsvcs ${shsvcs_rust_LIB})

set_module_type(shsvcs win32dll UNICODE)
target_link_libraries(shsvcs wine)
add_importlibs(shsvcs uxtheme advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET shsvcs DESTINATION reactos/system32 FOR all)
