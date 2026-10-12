
spec2def(winfax.dll winfax.spec)

list(APPEND SOURCE
    # winfax.c  # replaced by Rust (sdk/lib/rust/winfax-rust/src/winfax/winfax.rs)
    rust_cohab_stub.c
    winfax.rc
    ${CMAKE_CURRENT_BINARY_DIR}/winfax.def)

# Every translation unit is Rust now (winfax-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winfax_rust winfax-rust "" TRUE)
add_library(winfax MODULE ${SOURCE})
add_dependencies(winfax rust_winfax_rust)
target_link_libraries(winfax ${winfax_rust_LIB})
set_module_type(winfax win32dll)
add_importlibs(winfax msvcrt kernel32 ntdll)
add_cd_file(TARGET winfax DESTINATION reactos/system32 FOR all)
