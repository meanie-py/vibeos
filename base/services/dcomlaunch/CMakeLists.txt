spec2def(dcomlaunch.dll dcomlaunch.spec ADD_IMPORTLIB)

# Every translation unit is Rust now (dcomlaunch-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dcomlaunch_rust dcomlaunch-rust "" TRUE)
add_library(dcomlaunch MODULE
    # dcomlaunch.c  # replaced by Rust (sdk/lib/rust/dcomlaunch-rust/src/dcomlaunch/dcomlaunch.rs)
    # network.c  # replaced by Rust (sdk/lib/rust/dcomlaunch-rust/src/dcomlaunch/network.rs)
    rust_cohab_stub.c
    dcomlaunch.rc
    ${CMAKE_CURRENT_BINARY_DIR}/dcomlaunch.def)
add_dependencies(dcomlaunch rust_dcomlaunch_rust)
target_link_libraries(dcomlaunch ${dcomlaunch_rust_LIB})

set_module_type(dcomlaunch win32dll UNICODE)
add_importlibs(dcomlaunch advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET dcomlaunch DESTINATION reactos/system32 FOR all)
