
list(APPEND SOURCE
    # keyboard.c  # replaced by Rust (sdk/lib/rust/osk-rust/src/osk/keyboard/mod.rs, generated: port data)
    # main.c  # replaced by Rust (sdk/lib/rust/osk-rust/src/osk/main.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/osk-rust/src/osk/settings.rs)
    rust_cohab_stub.c
    precomp.h)

file(GLOB osk_rc_deps res/*.*)
add_rc_deps(osk.rc ${osk_rc_deps})
# Every translation unit is Rust now (osk-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(osk_rust osk-rust "" TRUE)
add_executable(osk ${SOURCE} osk.rc)
add_dependencies(osk rust_osk_rust)
target_link_libraries(osk ${osk_rust_LIB})
set_module_type(osk win32gui UNICODE)
add_importlibs(osk comdlg32 winmm shell32 user32 gdi32 advapi32 comctl32 msvcrt kernel32 ntdll)
add_pch(osk precomp.h SOURCE)
add_cd_file(TARGET osk DESTINATION reactos/system32 FOR all)
