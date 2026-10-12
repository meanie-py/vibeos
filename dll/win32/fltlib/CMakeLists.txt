
add_definitions(-D__WINESRC__)
spec2def(fltlib.dll fltlib.spec ADD_IMPORTLIB)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/fltmgr
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    includes)

list(APPEND SOURCE
    # fltlib.c  # replaced by Rust (sdk/lib/rust/fltlib-rust/src/fltlib/fltlib.rs)
    # message.c  # replaced by Rust (sdk/lib/rust/fltlib-rust/src/fltlib/message.rs)
    # stubs.c  # replaced by Rust (sdk/lib/rust/fltlib-rust/src/fltlib/stubs.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/fltlib.def)

# Every translation unit is Rust now (fltlib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fltlib_rust fltlib-rust "" TRUE)
add_library(fltlib MODULE ${SOURCE} rsrc.rc)
add_dependencies(fltlib rust_fltlib_rust)
target_link_libraries(fltlib ${fltlib_rust_LIB})
set_module_type(fltlib win32dll)
target_link_libraries(fltlib wine)
target_compile_definitions(fltlib PRIVATE NTDDI_VERSION=NTDDI_WS03SP1)
add_importlibs(fltlib msvcrt kernel32 ntdll)
add_cd_file(TARGET fltlib DESTINATION reactos/system32 FOR all)
