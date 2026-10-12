
spec2def(hid.dll hid.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # hid.c  # replaced by Rust (sdk/lib/rust/hid-rust/src/hid/hid.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

# Every translation unit is Rust now (hid-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(hid_rust hid-rust "" TRUE)
add_library(hid MODULE
    ${SOURCE}
    hid.rc
    ${CMAKE_CURRENT_BINARY_DIR}/hid.def)
add_dependencies(hid rust_hid_rust)
target_link_libraries(hid ${hid_rust_LIB})
add_pch(hid precomp.h SOURCE)

set_module_type(hid win32dll UNICODE)
add_importlibs(hid msvcrt kernel32 ntdll)
# hid.c includes the parser's hidp.h, which the C static lib used to export as its interface.
target_include_directories(hid PRIVATE ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/hidparser)

# The HID parser is hidparser-rust, built for user mode (its `user` panic personality).
rust_staticlib_variant(hid_hidparser_rust hidparser-rust "user" FALSE)
add_dependencies(hid rust_hid_hidparser_rust)
target_link_libraries(hid ${hid_hidparser_rust_LIB})
add_cd_file(TARGET hid DESTINATION reactos/system32 FOR all)
