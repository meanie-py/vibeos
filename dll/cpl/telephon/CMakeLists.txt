
spec2def(telephon.cpl telephon.spec)
add_rc_deps(telephon.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/applet.ico)

# Every translation unit is Rust now (telephon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(telephon_rust telephon-rust "" TRUE)
add_library(telephon MODULE
    # telephon.c  # replaced by Rust (sdk/lib/rust/telephon-rust/src/telephon/telephon.rs)
    rust_cohab_stub.c
    telephon.rc
    ${CMAKE_CURRENT_BINARY_DIR}/telephon.def)
add_dependencies(telephon rust_telephon_rust)
target_link_libraries(telephon ${telephon_rust_LIB})

set_module_type(telephon cpl UNICODE)
target_link_libraries(telephon uuid)
add_importlibs(telephon advapi32 user32 comctl32 ole32 shell32 msvcrt kernel32)
add_cd_file(TARGET telephon DESTINATION reactos/system32 FOR all)
