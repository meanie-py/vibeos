
list(APPEND SOURCE
    # about.c  # replaced by Rust (sdk/lib/rust/charmap-rust/src/charmap/about.rs)
    # charmap.c  # replaced by Rust (sdk/lib/rust/charmap-rust/src/charmap/charmap.rs)
    # lrgcell.c  # replaced by Rust (sdk/lib/rust/charmap-rust/src/charmap/lrgcell.rs)
    # map.c  # replaced by Rust (sdk/lib/rust/charmap-rust/src/charmap/map.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/charmap-rust/src/charmap/settings.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(charmap.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/charmap.ico)
# Every translation unit is Rust now (charmap-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(charmap_rust charmap-rust "" TRUE)
add_executable(charmap ${SOURCE} charmap.rc)
add_dependencies(charmap rust_charmap_rust)
target_link_libraries(charmap ${charmap_rust_LIB})
set_module_type(charmap win32gui UNICODE)
add_importlibs(charmap getuname advapi32 user32 gdi32 comctl32 msvcrt kernel32)
add_pch(charmap precomp.h SOURCE)
add_cd_file(TARGET charmap DESTINATION reactos/system32 FOR all)
