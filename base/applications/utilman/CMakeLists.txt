
list(APPEND SOURCE
    # utilman.c  # replaced by Rust (sdk/lib/rust/utilman-rust/src/utilman/utilman.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(utilman.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/utilman.ico)
# Every translation unit is Rust now (utilman-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(utilman_rust utilman-rust "" TRUE)
add_executable(utilman ${SOURCE} utilman.rc)
add_dependencies(utilman rust_utilman_rust)
target_link_libraries(utilman ${utilman_rust_LIB})
set_module_type(utilman win32gui UNICODE)
add_importlibs(utilman user32 msvcrt kernel32 ntdll)
add_pch(utilman precomp.h SOURCE)
add_cd_file(TARGET utilman DESTINATION reactos/system32 FOR all)
add_subdirectory(umandlg)
