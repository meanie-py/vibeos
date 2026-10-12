
include_directories(
    BEFORE include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/ndisuio)

# Every translation unit is Rust now (wlanconf-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wlanconf_rust wlanconf-rust "" TRUE)
add_executable(wlanconf
    # wlanconf.c  # replaced by Rust (sdk/lib/rust/wlanconf-rust/src/wlanconf/wlanconf.rs)
    rust_cohab_stub.c
    wlanconf.rc)
add_dependencies(wlanconf rust_wlanconf_rust)
target_link_libraries(wlanconf ${wlanconf_rust_LIB})
set_module_type(wlanconf win32cui UNICODE)
target_link_libraries(wlanconf conutils)
add_importlibs(wlanconf iphlpapi msvcrt kernel32)
add_cd_file(TARGET wlanconf DESTINATION reactos/system32 FOR all)
