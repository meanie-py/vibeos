
add_definitions(-D__USE_W32_SOCKETS)

list(APPEND SOURCE
    # finger.c  # replaced by Rust (sdk/lib/rust/finger-rust/src/finger/finger.rs)
    # err.c  # replaced by Rust (sdk/lib/rust/finger-rust/src/finger/err.rs)
    # getopt.c  # replaced by Rust (sdk/lib/rust/finger-rust/src/finger/getopt.rs)
    # net.c  # replaced by Rust (sdk/lib/rust/finger-rust/src/finger/net.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

# Every translation unit is Rust now (finger-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(finger_rust finger-rust "" TRUE)
add_executable(finger ${SOURCE} finger.rc)
add_dependencies(finger rust_finger_rust)
target_link_libraries(finger ${finger_rust_LIB})
set_module_type(finger win32cui)
add_importlibs(finger ws2_32 msvcrt kernel32)
add_pch(finger precomp.h SOURCE)
add_cd_file(TARGET finger DESTINATION reactos/system32 FOR all)
