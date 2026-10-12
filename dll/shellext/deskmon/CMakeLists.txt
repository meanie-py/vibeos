
spec2def(deskmon.dll deskmon.spec)

list(APPEND SOURCE
    # deskmon.c  # replaced by Rust (sdk/lib/rust/deskmon-rust/src/deskmon/deskmon.rs)
    # shxiface.c  # replaced by Rust (sdk/lib/rust/deskmon-rust/src/deskmon/shxiface.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(deskmon.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/monitor.ico)

# Every translation unit is Rust now (deskmon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(deskmon_rust deskmon-rust "" TRUE)
add_library(deskmon MODULE
    ${SOURCE}
    deskmon.rc
    ${CMAKE_CURRENT_BINARY_DIR}/deskmon.def)
add_dependencies(deskmon rust_deskmon_rust)
target_link_libraries(deskmon ${deskmon_rust_LIB})

set_module_type(deskmon win32dll UNICODE)
target_link_libraries(deskmon uuid)
add_importlibs(deskmon user32 gdi32 comctl32 ole32 setupapi msvcrt kernel32 ntdll)
add_pch(deskmon precomp.h SOURCE)
add_cd_file(TARGET deskmon DESTINATION reactos/system32 FOR all)
