
spec2def(input.dll input.spec)

list(APPEND SOURCE
    # input.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/input.rs)
    input.h
    # input_list.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/input_list.rs)
    input_list.h
    # layout_list.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/layout_list.rs)
    layout_list.h
    # locale_list.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/locale_list.rs)
    locale_list.h
    # settings_page.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/settings_page.rs)
    # advanced_settings_page.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/advanced_settings_page.rs)
    # add_dialog.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/add_dialog.rs)
    # edit_dialog.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/edit_dialog.rs)
    # key_settings_dialog.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/key_settings_dialog.rs)
    # key_sequence_dialog.c  # replaced by Rust (sdk/lib/rust/input-rust/src/input/key_sequence_dialog.rs)
    rust_cohab_stub.c
    )

add_rc_deps(input.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/keyboard-shortcuts.ico)

# Every translation unit is Rust now (input-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(input_rust input-rust "" TRUE)
add_library(input MODULE
    ${SOURCE}
    input.rc
    ${CMAKE_CURRENT_BINARY_DIR}/input.def)
add_dependencies(input rust_input_rust)
target_link_libraries(input ${input_rust_LIB})

set_module_type(input win32dll UNICODE)
add_importlibs(input advapi32 user32 comctl32 gdi32 msvcrt kernel32 setupapi)
add_pch(input input.h SOURCE)
add_cd_file(TARGET input DESTINATION reactos/system32 FOR all)
