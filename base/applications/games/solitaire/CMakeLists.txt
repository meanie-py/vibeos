
list(APPEND SOURCE
    # solcreate.cpp  # replaced by Rust (sdk/lib/rust/sol-rust/src/sol/solcreate.rs)
    # solgame.cpp  # replaced by Rust (sdk/lib/rust/sol-rust/src/sol/solgame.rs)
    # solundo.cpp  # replaced by Rust (sdk/lib/rust/sol-rust/src/sol/solundo.rs)
    # solitaire.cpp  # replaced by Rust (sdk/lib/rust/sol-rust/src/sol/solitaire.rs)
    rust_cohab_stub.cpp
    solitaire.h)

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/solitaire.ico)
# Every translation unit is Rust now (sol-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sol_rust sol-rust "" TRUE)
add_executable(sol ${SOURCE} rsrc.rc)
add_dependencies(sol rust_sol_rust)
target_link_libraries(sol ${sol_rust_LIB})
target_link_libraries(sol cardlib)
set_module_type(sol win32gui UNICODE)
add_importlibs(sol advapi32 comctl32 shell32 user32 gdi32 msvcrt kernel32 ntdll)
add_pch(sol solitaire.h SOURCE)
add_cd_file(TARGET sol DESTINATION reactos/system32 FOR all)
