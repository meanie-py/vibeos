
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(tbs.dll tbs.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # tbs.c  # replaced by Rust (sdk/lib/rust/tbs-rust/src/tbs/tbs.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/tbs_stubs.c)

# Every translation unit is Rust now (tbs-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tbs_rust tbs-rust "" TRUE)
add_library(tbs MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/tbs.def)
add_dependencies(tbs rust_tbs_rust)
target_link_libraries(tbs ${tbs_rust_LIB})

set_module_type(tbs win32dll)
target_link_libraries(tbs wine)
add_importlibs(tbs msvcrt kernel32 ntdll)
add_dependencies(tbs psdk)
add_cd_file(TARGET tbs DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(tbs)
