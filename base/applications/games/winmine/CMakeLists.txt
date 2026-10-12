
list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/winmine-rust/src/winmine/main.rs)
    # dialog.c  # replaced by Rust (sdk/lib/rust/winmine-rust/src/winmine/dialog.rs)
    rust_cohab_stub.c
    main.h)

file(GLOB winmine_rc_deps rc/*.*)
add_rc_deps(rsrc.rc ${winmine_rc_deps})
# Every translation unit is Rust now (winmine-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winmine_rust winmine-rust "" TRUE)
add_executable(winmine ${SOURCE} rsrc.rc)
add_dependencies(winmine rust_winmine_rust)
target_link_libraries(winmine ${winmine_rust_LIB})
add_pch(winmine main.h SOURCE)
set_module_type(winmine win32gui UNICODE)
target_link_libraries(winmine wine)
add_importlibs(winmine user32 gdi32 advapi32 shell32 winmm msvcrt kernel32 ntdll)
add_cd_file(TARGET winmine DESTINATION reactos/system32 FOR all)
