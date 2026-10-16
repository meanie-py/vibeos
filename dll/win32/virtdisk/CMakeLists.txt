spec2def(virtdisk.dll virtdisk.spec ADD_IMPORTLIB)

# Written in Rust (sdk/lib/rust/virtdisk-rust): virtdisk.dll for an OS with no virtual-disk provider.
# The target keeps a stub TU and links the Rust archive.
list(APPEND SOURCE
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/virtdisk_stubs.c)

rust_staticlib_variant(virtdisk_rust virtdisk-rust "" TRUE)
add_library(virtdisk MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/virtdisk.def)
add_dependencies(virtdisk rust_virtdisk_rust)
target_link_libraries(virtdisk ${virtdisk_rust_LIB})

# No DllMain: no per-process state beyond a lock-free static.
set_module_type(virtdisk win32dll ENTRYPOINT 0)
add_importlibs(virtdisk kernel32 ntdll)
add_cd_file(TARGET virtdisk DESTINATION reactos/system32 FOR all)
