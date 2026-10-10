
PROJECT(progman)

list(APPEND SOURCE
    # dialog.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/dialog.rs)
    # group.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/group.rs)
    # grpfile.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/grpfile.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/main.rs)
    # program.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/program.rs)
    # string.c  # replaced by Rust (sdk/lib/rust/progman-rust/src/progman/string.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    progman.h)

# Every translation unit is Rust now (progman-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(progman_rust progman-rust "" TRUE)
add_executable(progman ${SOURCE} progman.rc)
add_dependencies(progman rust_progman_rust)
target_link_libraries(progman ${progman_rust_LIB})
set_module_type(progman win32gui UNICODE)
add_importlibs(progman advapi32 user32 gdi32 comctl32 comdlg32 shell32 msvcrt kernel32)
add_pch(progman progman.h SOURCE)
add_cd_file(TARGET progman DESTINATION reactos/system32 FOR all)
