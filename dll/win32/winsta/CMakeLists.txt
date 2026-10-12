
spec2def(winsta.dll winsta.spec)

include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)

list(APPEND SOURCE
    # logon.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/logon.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/main.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/misc.rs)
    # query.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/query.rs)
    # security.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/security.rs)
    # server.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/server.rs)
    # ws.c  # replaced by Rust (sdk/lib/rust/winsta-rust/src/winsta/ws.rs)
    rust_cohab_stub.c
    winsta.h)

# Every translation unit is Rust now (winsta-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winsta_rust winsta-rust "" TRUE)
add_library(winsta MODULE
    ${SOURCE}
    winsta.rc
    ${CMAKE_CURRENT_BINARY_DIR}/winsta.def)
add_dependencies(winsta rust_winsta_rust)
target_link_libraries(winsta ${winsta_rust_LIB})

set_module_type(winsta win32dll)
target_link_libraries(winsta wine)
add_importlibs(winsta msvcrt kernel32 ntdll)
add_pch(winsta winsta.h SOURCE)
add_cd_file(TARGET winsta DESTINATION reactos/system32 FOR all)
