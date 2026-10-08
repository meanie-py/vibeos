
spec2def(console.dll console.spec)

list(APPEND SOURCE
    # console.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/console.rs)
    # options.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/options.rs)
    # font.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/font.rs)
    # layout.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/layout.rs)
    # colors.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/colors.rs)
    # utils.c  # replaced by Rust (sdk/lib/rust/console-rust/src/console/utils.rs)
    rust_cohab_stub.c
    console.h)

add_rc_deps(console.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/terminal.ico)

# Every translation unit is Rust now (console-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(console_rust console-rust "" TRUE)
add_library(console MODULE
    ${SOURCE}
    console.rc
    ${CMAKE_CURRENT_BINARY_DIR}/console.def)
add_dependencies(console rust_console_rust)
target_link_libraries(console ${console_rust_LIB})
add_pch(console console.h SOURCE)

set_module_type(console cpl UNICODE)
set_target_properties(console PROPERTIES SUFFIX ".dll")

target_link_libraries(console concfg)
add_importlibs(console advapi32 user32 gdi32 comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET console DESTINATION reactos/system32 FOR all)
