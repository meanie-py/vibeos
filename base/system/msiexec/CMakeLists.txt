
list(APPEND SOURCE
    # msiexec.c  # replaced by Rust (sdk/lib/rust/msiexec-rust/src/msiexec/msiexec.rs)
    # service.c  # replaced by Rust (sdk/lib/rust/msiexec-rust/src/msiexec/service.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/msiexec.ico)
# Every translation unit is Rust now (msiexec-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msiexec_rust msiexec-rust "" TRUE)
add_executable(msiexec ${SOURCE} rsrc.rc)
add_dependencies(msiexec rust_msiexec_rust)
target_link_libraries(msiexec ${msiexec_rust_LIB})
set_module_type(msiexec win32gui)
target_link_libraries(msiexec uuid wine)
add_importlibs(msiexec user32 advapi32 ole32 comctl32 msi msvcrt kernel32 ntdll)
add_pch(msiexec precomp.h SOURCE)
add_cd_file(TARGET msiexec DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(msiexec) # CORE-5743: No ARRAY_SIZE macro
