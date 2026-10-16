
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(wevtapi.dll wevtapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/wevtapi-rust/src/wevtapi/main.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/wevtapi_stubs.c)

# Every translation unit is Rust now (wevtapi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wevtapi_rust wevtapi-rust "" TRUE)
add_library(wevtapi MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/wevtapi.def)
add_dependencies(wevtapi rust_wevtapi_rust)
target_link_libraries(wevtapi ${wevtapi_rust_LIB})

set_module_type(wevtapi win32dll)
target_link_libraries(wevtapi wine)
add_importlibs(wevtapi msvcrt kernel32 ntdll)
add_dependencies(wevtapi psdk)
add_cd_file(TARGET wevtapi DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(wevtapi)
