
add_rc_deps(kbswitch.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/kbswitch.ico)
# Every translation unit is Rust now (kbswitch-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(kbswitch_rust kbswitch-rust "" TRUE)
add_executable(kbswitch
    # kbswitch.c  # replaced by Rust (sdk/lib/rust/kbswitch-rust/src/kbswitch/kbswitch.rs)
    # imemenu.c  # replaced by Rust (sdk/lib/rust/kbswitch-rust/src/kbswitch/imemenu.rs)
    rust_cohab_stub.c
    kbswitch.rc)
add_dependencies(kbswitch rust_kbswitch_rust)
target_link_libraries(kbswitch ${kbswitch_rust_LIB})
set_module_type(kbswitch win32gui UNICODE)
target_link_libraries(kbswitch wine)
add_importlibs(kbswitch advapi32 imm32 user32 shell32 shlwapi gdi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET kbswitch DESTINATION reactos/system32 FOR all)

add_subdirectory(indicdll)
add_dependencies(kbswitch indicdll)
