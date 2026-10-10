
list(APPEND SOURCE
    # console.c  # replaced by Rust (sdk/lib/rust/mmc-rust/src/mmc/console.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/mmc-rust/src/mmc/misc.rs)
    # mmc.c  # replaced by Rust (sdk/lib/rust/mmc-rust/src/mmc/mmc.rs)
    # mscfile.c  # replaced by Rust (sdk/lib/rust/mmc-rust/src/mmc/mscfile.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

add_rc_deps(mmc.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/mmc.ico)
# Every translation unit is Rust now (mmc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmc_rust mmc-rust "" TRUE)
add_executable(mmc ${SOURCE} mmc.rc)
add_dependencies(mmc rust_mmc_rust)
target_link_libraries(mmc ${mmc_rust_LIB})
set_module_type(mmc win32gui UNICODE)
target_link_libraries(mmc uuid)
add_importlibs(mmc user32 gdi32 comdlg32 advapi32 shell32 comctl32 ole32 oleaut32 shlwapi msvcrt kernel32 ntdll)
add_pch(mmc precomp.h SOURCE)
add_cd_file(TARGET mmc DESTINATION reactos/system32 FOR all)
