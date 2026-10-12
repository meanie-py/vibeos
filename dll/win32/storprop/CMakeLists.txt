
spec2def(storprop.dll storprop.spec)

list(APPEND SOURCE
    # disk.c  # replaced by Rust (sdk/lib/rust/storprop-rust/src/storprop/disk.rs)
    # dvd.c  # replaced by Rust (sdk/lib/rust/storprop-rust/src/storprop/dvd.rs)
    # ide.c  # replaced by Rust (sdk/lib/rust/storprop-rust/src/storprop/ide.rs)
    # storprop.c  # replaced by Rust (sdk/lib/rust/storprop-rust/src/storprop/storprop.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (storprop-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(storprop_rust storprop-rust "" TRUE)
add_library(storprop MODULE
    ${SOURCE}
    storprop.rc
    ${CMAKE_CURRENT_BINARY_DIR}/storprop.def)
add_dependencies(storprop rust_storprop_rust)
target_link_libraries(storprop ${storprop_rust_LIB})

set_module_type(storprop win32dll UNICODE)
add_importlibs(storprop setupapi comctl32 advapi32 user32 msvcrt kernel32 ntdll)
add_pch(storprop precomp.h SOURCE)
add_cd_file(TARGET storprop DESTINATION reactos/system32 FOR all)
