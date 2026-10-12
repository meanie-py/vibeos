
spec2def(newdev.dll newdev.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # newdev.c  # replaced by Rust (sdk/lib/rust/newdev-rust/src/newdev/newdev.rs)
    # stubs.c  # replaced by Rust (sdk/lib/rust/newdev-rust/src/newdev/stubs.rs)
    # wizard.c  # replaced by Rust (sdk/lib/rust/newdev-rust/src/newdev/wizard.rs)
    rust_cohab_stub.c
    newdev_private.h
    ${CMAKE_CURRENT_BINARY_DIR}/newdev_stubs.c)

# Every translation unit is Rust now (newdev-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(newdev_rust newdev-rust "" TRUE)
add_library(newdev MODULE
    ${SOURCE}
    newdev.rc
    ${CMAKE_CURRENT_BINARY_DIR}/newdev.def)
add_dependencies(newdev rust_newdev_rust)
target_link_libraries(newdev ${newdev_rust_LIB})

set_module_type(newdev win32dll UNICODE)
target_link_libraries(newdev wine)
add_importlibs(newdev gdi32 comctl32 setupapi advapi32 user32 shell32 shlwapi ole32 msvcrt kernel32 ntdll)
add_pch(newdev newdev_private.h SOURCE)
add_cd_file(TARGET newdev DESTINATION reactos/system32 FOR all)
