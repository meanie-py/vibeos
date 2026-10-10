
list(APPEND SOURCE
    # osinfo.c  # replaced by Rust (sdk/lib/rust/winver-rust/src/winver/osinfo.rs)
    # winver.c  # replaced by Rust (sdk/lib/rust/winver-rust/src/winver/winver.rs)
    rust_cohab_stub.c
    winver_p.h)

# Every translation unit is Rust now (winver-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winver_rust winver-rust "" TRUE)
add_executable(winver ${SOURCE} winver.rc)
add_dependencies(winver rust_winver_rust)
target_link_libraries(winver ${winver_rust_LIB})
add_pch(winver winver_p.h SOURCE)
set_module_type(winver win32gui UNICODE)
add_importlibs(winver advapi32 user32 shell32 comctl32 msvcrt kernel32)
add_cd_file(TARGET winver DESTINATION reactos/system32 FOR all)
