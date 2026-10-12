
# Every translation unit is Rust now (conime-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(conime_rust conime-rust "" TRUE)
add_executable(conime WIN32
    # conime.c  # replaced by Rust (sdk/lib/rust/conime-rust/src/conime/conime.rs)
    rust_cohab_stub.c
    conime.rc)
add_dependencies(conime rust_conime_rust)
target_link_libraries(conime ${conime_rust_LIB})
set_module_type(conime win32gui UNICODE)
target_link_libraries(conime wine ${PSEH_LIB})
add_importlibs(conime imm32 advapi32 user32 gdi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET conime DESTINATION reactos/system32 FOR all)
