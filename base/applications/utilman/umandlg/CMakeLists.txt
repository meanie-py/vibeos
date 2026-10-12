
spec2def(umandlg.dll umandlg.spec)

# Every translation unit is Rust now (umandlg-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(umandlg_rust umandlg-rust "" TRUE)
add_library(umandlg MODULE
    # process.c  # replaced by Rust (sdk/lib/rust/umandlg-rust/src/umandlg/process.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/umandlg-rust/src/umandlg/registry.rs)
    # umandlg.c  # replaced by Rust (sdk/lib/rust/umandlg-rust/src/umandlg/umandlg.rs)
    rust_cohab_stub.c
    umandlg.h
    umandlg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/umandlg.def)
add_dependencies(umandlg rust_umandlg_rust)
target_link_libraries(umandlg ${umandlg_rust_LIB})

set_module_type(umandlg win32dll UNICODE)
add_importlibs(umandlg comdlg32 user32 gdi32 advapi32 shell32 comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET umandlg DESTINATION reactos/system32 FOR all)
