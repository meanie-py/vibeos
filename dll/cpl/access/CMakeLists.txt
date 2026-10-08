
spec2def(access.cpl access.spec)

list(APPEND SOURCE
    # access.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/access.rs)
    # display.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/display.rs)
    # general.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/general.rs)
    # keyboard.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/keyboard.rs)
    # mouse.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/mouse.rs)
    # sound.c  # replaced by Rust (sdk/lib/rust/access-rust/src/access/sound.rs)
    rust_cohab_stub.c
    access.h)

add_rc_deps(access.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/applet.ico)

# Every translation unit is Rust now (access-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(access_rust access-rust "" TRUE)
add_library(access MODULE
    ${SOURCE}
    access.rc
    ${CMAKE_CURRENT_BINARY_DIR}/access.def)
add_dependencies(access rust_access_rust)
target_link_libraries(access ${access_rust_LIB})

set_module_type(access cpl UNICODE)
add_importlibs(access user32 gdi32 advapi32 comctl32 msvcrt kernel32)
add_pch(access access.h SOURCE)
add_cd_file(TARGET access DESTINATION reactos/system32 FOR all)
