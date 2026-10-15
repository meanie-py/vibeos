spec2def(icuuc.dll icuuc.spec ADD_IMPORTLIB)

# Written in Rust from the start (sdk/lib/rust/icuuc-rust): there is no C to translate,
# because Wine has no ICU. The target keeps a stub TU and links the Rust archive.
list(APPEND SOURCE
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/icuuc_stubs.c)

rust_staticlib_variant(icuuc_rust icuuc-rust "" TRUE)
add_library(icuuc MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/icuuc.def)
add_dependencies(icuuc rust_icuuc_rust)
target_link_libraries(icuuc ${icuuc_rust_LIB})

# No DllMain: the converters keep no per-process state.
set_module_type(icuuc win32dll ENTRYPOINT 0)
add_importlibs(icuuc kernel32 ntdll)
add_cd_file(TARGET icuuc DESTINATION reactos/system32 FOR all)
