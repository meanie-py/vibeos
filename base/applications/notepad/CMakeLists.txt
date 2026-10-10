
if(DBG)
    add_definitions(-D_DEBUG=1)
endif()

list(APPEND SOURCE
    # dialog.c  # replaced by Rust (sdk/lib/rust/notepad-rust/src/notepad/dialog.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/notepad-rust/src/notepad/main.rs)
    # printing.c  # replaced by Rust (sdk/lib/rust/notepad-rust/src/notepad/printing.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/notepad-rust/src/notepad/settings.rs)
    # text.c  # replaced by Rust (sdk/lib/rust/notepad-rust/src/notepad/text.rs)
    rust_cohab_stub.c
    notepad.h)

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/notepad.ico)
# Every translation unit is Rust now (notepad-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(notepad_rust notepad-rust "" TRUE)
add_executable(notepad ${SOURCE} rsrc.rc)
add_dependencies(notepad rust_notepad_rust)
target_link_libraries(notepad ${notepad_rust_LIB})
set_module_type(notepad win32gui UNICODE)
add_importlibs(notepad user32 gdi32 comctl32 comdlg32 advapi32 shell32 msvcrt kernel32)
add_pch(notepad notepad.h SOURCE)
add_cd_file(TARGET notepad DESTINATION reactos FOR all)
add_cd_file(TARGET notepad DESTINATION reactos/system32 FOR all)
