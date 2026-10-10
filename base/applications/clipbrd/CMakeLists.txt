
list(APPEND SOURCE
    # clipbrd.c  # replaced by Rust (sdk/lib/rust/clipbrd-rust/src/clipbrd/clipbrd.rs)
    # cliputils.c  # replaced by Rust (sdk/lib/rust/clipbrd-rust/src/clipbrd/cliputils.rs)
    # fileutils.c  # replaced by Rust (sdk/lib/rust/clipbrd-rust/src/clipbrd/fileutils.rs)
    # scrollutils.c  # replaced by Rust (sdk/lib/rust/clipbrd-rust/src/clipbrd/scrollutils.rs)
    # winutils.c  # replaced by Rust (sdk/lib/rust/clipbrd-rust/src/clipbrd/winutils.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (clipbrd-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(clipbrd_rust clipbrd-rust "" TRUE)
add_executable(clipbrd ${SOURCE} clipbrd.rc)
add_dependencies(clipbrd rust_clipbrd_rust)
target_link_libraries(clipbrd ${clipbrd_rust_LIB})
add_pch(clipbrd precomp.h SOURCE)
set_module_type(clipbrd win32gui UNICODE)
add_importlibs(clipbrd hhctrl user32 gdi32 comdlg32 shell32 msvcrt kernel32)
add_cd_file(TARGET clipbrd DESTINATION reactos/system32 FOR all)
