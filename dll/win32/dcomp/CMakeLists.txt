
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(dcomp.dll dcomp.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # device.c  # replaced by Rust (sdk/lib/rust/dcomp-rust/src/dcomp/device.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/dcomp_stubs.c)

# Every translation unit is Rust now (dcomp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dcomp_rust dcomp-rust "" TRUE)
add_library(dcomp MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/dcomp.def)
add_dependencies(dcomp rust_dcomp_rust)
target_link_libraries(dcomp ${dcomp_rust_LIB})

set_module_type(dcomp win32dll)
target_link_libraries(dcomp wine wine_dll_canunload)
add_importlibs(dcomp msvcrt kernel32 ntdll)
add_dependencies(dcomp psdk)
add_cd_file(TARGET dcomp DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(dcomp)
