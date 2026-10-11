
spec2def(deskadp.dll deskadp.spec)

list(APPEND SOURCE
    # deskadp.c  # replaced by Rust (sdk/lib/rust/deskadp-rust/src/deskadp/deskadp.rs)
    # shxiface.c  # replaced by Rust (sdk/lib/rust/deskadp-rust/src/deskadp/shxiface.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(deskadp.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/adapter.ico)

# Every translation unit is Rust now (deskadp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(deskadp_rust deskadp-rust "" TRUE)
add_library(deskadp MODULE
    ${SOURCE}
    deskadp.rc
    ${CMAKE_CURRENT_BINARY_DIR}/deskadp.def)
add_dependencies(deskadp rust_deskadp_rust)
target_link_libraries(deskadp ${deskadp_rust_LIB})

set_module_type(deskadp win32dll UNICODE)
target_link_libraries(deskadp uuid)
add_importlibs(deskadp user32 gdi32 comctl32 ole32 msvcrt kernel32 ntdll)
add_pch(deskadp precomp.h SOURCE)
add_cd_file(TARGET deskadp DESTINATION reactos/system32 FOR all)
