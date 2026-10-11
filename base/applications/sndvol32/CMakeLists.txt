
list(APPEND SOURCE
    # advanced.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/advanced.rs)
    # dialog.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/dialog.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/misc.rs)
    # mixer.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/mixer.rs)
    # sndvol32.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/sndvol32.rs)
    # tray.c  # replaced by Rust (sdk/lib/rust/sndvol32-rust/src/sndvol32/tray.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    sndvol32.h)

add_rc_deps(sndvol32.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/sndvol32.ico)
# Every translation unit is Rust now (sndvol32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sndvol32_rust sndvol32-rust "" TRUE)
add_executable(sndvol32 ${SOURCE} sndvol32.rc)
add_dependencies(sndvol32 rust_sndvol32_rust)
target_link_libraries(sndvol32 ${sndvol32_rust_LIB})
set_module_type(sndvol32 win32gui UNICODE)
add_importlibs(sndvol32 user32 advapi32 gdi32 comctl32 shell32 winmm msvcrt kernel32 ntdll)
add_pch(sndvol32 sndvol32.h SOURCE)
add_cd_file(TARGET sndvol32 DESTINATION reactos/system32 FOR all)
