
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(ndfapi.dll ndfapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # ndfapi_main.c  # replaced by Rust (sdk/lib/rust/ndfapi-rust/src/ndfapi/ndfapi_main.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/ndfapi_stubs.c)

# Every translation unit is Rust now (ndfapi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ndfapi_rust ndfapi-rust "" TRUE)
add_library(ndfapi MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/ndfapi.def)
add_dependencies(ndfapi rust_ndfapi_rust)
target_link_libraries(ndfapi ${ndfapi_rust_LIB})

set_module_type(ndfapi win32dll)
target_link_libraries(ndfapi wine wine_dll_register wine_dll_canunload)
add_importlibs(ndfapi msvcrt kernel32 ntdll)
add_dependencies(ndfapi psdk)
add_cd_file(TARGET ndfapi DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(ndfapi)
