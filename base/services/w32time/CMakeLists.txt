
spec2def(w32time.dll w32time.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (w32time-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(w32time_rust w32time-rust "" TRUE)
add_library(w32time MODULE
    # w32time.c  # replaced by Rust (sdk/lib/rust/w32time-rust/src/w32time/w32time.rs)
    # ntpclient.c  # replaced by Rust (sdk/lib/rust/w32time-rust/src/w32time/ntpclient.rs)
    # register.c  # replaced by Rust (sdk/lib/rust/w32time-rust/src/w32time/register.rs)
    rust_cohab_stub.c
    w32time.rc
    ${CMAKE_CURRENT_BINARY_DIR}/w32time.def)
add_dependencies(w32time rust_w32time_rust)
target_link_libraries(w32time ${w32time_rust_LIB})

set_module_type(w32time win32dll UNICODE)

add_importlibs(w32time ws2_32 advapi32 msvcrt user32 kernel32 ntdll)
add_cd_file(TARGET w32time DESTINATION reactos/system32 FOR all)
