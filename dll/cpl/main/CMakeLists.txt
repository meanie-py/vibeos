
spec2def(main.cpl main.spec)

list(APPEND SOURCE
    # keyboard.c  # replaced by Rust (sdk/lib/rust/main-rust/src/main/keyboard.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/main-rust/src/main/main.rs)
    # mouse.c  # replaced by Rust (sdk/lib/rust/main-rust/src/main/mouse.rs)
    rust_cohab_stub.c
    main.h)

set(ICONS_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../win32/shell32/res/icons)
file(GLOB main_rc_deps resources/*.* ${ICONS_DIR}/38.ico ${ICONS_DIR}/39.ico)
add_rc_deps(main.rc ${main_rc_deps})

# Every translation unit is Rust now (main-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(main_rust main-rust "" TRUE)
add_library(main MODULE
    ${SOURCE}
    main.rc
    ${CMAKE_CURRENT_BINARY_DIR}/main.def)
add_dependencies(main rust_main_rust)
target_link_libraries(main ${main_rust_LIB})

set_module_type(main cpl UNICODE)
target_link_libraries(main uuid)
add_importlibs(main advapi32 user32 comctl32 devmgr comdlg32 shell32 gdi32 msvcrt kernel32)
add_pch(main main.h SOURCE)
add_cd_file(TARGET main DESTINATION reactos/system32 FOR all)
