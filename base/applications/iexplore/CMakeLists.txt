
add_rc_deps(iexplore.rc
    ${CMAKE_CURRENT_SOURCE_DIR}/iexplore.ico
    ${CMAKE_CURRENT_SOURCE_DIR}/iexplore.inf)
# Every translation unit is Rust now (iexplore-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(iexplore_rust iexplore-rust "" TRUE)
add_executable(iexplore
    # main.c  # replaced by Rust (sdk/lib/rust/iexplore-rust/src/iexplore/main.rs)
    rust_cohab_stub.c
    iexplore.rc)
add_dependencies(iexplore rust_iexplore_rust)
target_link_libraries(iexplore ${iexplore_rust_LIB})
target_link_libraries(iexplore wine)
set_module_type(iexplore win32gui UNICODE)
add_delay_importlibs(iexplore advpack version user32)
add_importlibs(iexplore ieframe msvcrt kernel32 ntdll)
add_cd_file(TARGET iexplore DESTINATION reactos FOR all)
