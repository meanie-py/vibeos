
spec2def(ncpa.cpl ncpa.spec)

# Every translation unit is Rust now (ncpa-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ncpa_rust ncpa-rust "" TRUE)
add_library(ncpa MODULE
    # ncpa.c  # replaced by Rust (sdk/lib/rust/ncpa-rust/src/ncpa/ncpa.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/ncpa.def)
add_dependencies(ncpa rust_ncpa_rust)
target_link_libraries(ncpa ${ncpa_rust_LIB})

set_module_type(ncpa cpl UNICODE)
add_importlibs(ncpa user32 advapi32 shell32 msvcrt kernel32)
add_cd_file(TARGET ncpa DESTINATION reactos/system32 FOR all)
