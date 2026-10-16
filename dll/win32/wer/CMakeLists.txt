spec2def(wer.dll wer.spec ADD_IMPORTLIB)

# Written in Rust (sdk/lib/rust/wer-rust): wer.dll (WER API, no WER service); port of Wine's dlls/wer/main.c.
# The target keeps a stub TU and links the Rust archive.
list(APPEND SOURCE
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/wer_stubs.c)

rust_staticlib_variant(wer_rust wer-rust "" TRUE)
add_library(wer MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/wer.def)
add_dependencies(wer rust_wer_rust)
target_link_libraries(wer ${wer_rust_LIB})

# No DllMain: no per-process state beyond a lock-free static.
set_module_type(wer win32dll ENTRYPOINT 0)
add_importlibs(wer advapi32 kernel32 ntdll)
add_cd_file(TARGET wer DESTINATION reactos/system32 FOR all)
