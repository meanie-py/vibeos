spec2def(wuauserv.dll wuauserv.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (wuauserv-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wuauserv_rust wuauserv-rust "" TRUE)
add_library(wuauserv MODULE
    # wuauserv.c  # replaced by Rust (sdk/lib/rust/wuauserv-rust/src/wuauserv/wuauserv.rs)
    rust_cohab_stub.c
    wuauserv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wuauserv.def)
add_dependencies(wuauserv rust_wuauserv_rust)
target_link_libraries(wuauserv ${wuauserv_rust_LIB})

set_module_type(wuauserv win32dll UNICODE)
add_importlibs(wuauserv advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wuauserv DESTINATION reactos/system32 FOR all)
