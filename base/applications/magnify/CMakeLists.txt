
list(APPEND SOURCE
    # magnifier.c  # replaced by Rust (sdk/lib/rust/magnify-rust/src/magnify/magnifier.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/magnify-rust/src/magnify/settings.rs)
    rust_cohab_stub.c
    magnifier.h)

add_rc_deps(magnify.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/magnify.ico)
# Every translation unit is Rust now (magnify-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(magnify_rust magnify-rust "" TRUE)
add_executable(magnify ${SOURCE} magnify.rc)
add_dependencies(magnify rust_magnify_rust)
target_link_libraries(magnify ${magnify_rust_LIB})
set_module_type(magnify win32gui)
add_importlibs(magnify user32 gdi32 comctl32 advapi32 shell32 msvcrt kernel32)
add_pch(magnify magnifier.h SOURCE)
add_cd_file(TARGET magnify DESTINATION reactos/system32 FOR all)
