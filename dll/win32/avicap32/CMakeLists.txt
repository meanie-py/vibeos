
spec2def(avicap32.dll avicap32.spec ADD_IMPORTLIB)

include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)

# Every translation unit is Rust now (avicap32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(avicap32_rust avicap32-rust "" TRUE)
add_library(avicap32 MODULE
    # avicap32.c  # replaced by Rust (sdk/lib/rust/avicap32-rust/src/avicap32/avicap32.rs)
    rust_cohab_stub.c
    avicap32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/avicap32.def)
add_dependencies(avicap32 rust_avicap32_rust)
target_link_libraries(avicap32 ${avicap32_rust_LIB})

set_module_type(avicap32 win32dll UNICODE)
target_link_libraries(avicap32 wine)
add_importlibs(avicap32 advapi32 user32 version msvcrt kernel32 ntdll)
add_cd_file(TARGET avicap32 DESTINATION reactos/system32 FOR all)
