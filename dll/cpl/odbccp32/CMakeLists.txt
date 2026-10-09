
spec2def(odbccp32i.cpl odbccp32.spec)

# Every translation unit is Rust now (odbccp32i-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(odbccp32i_rust odbccp32i-rust "" TRUE)
add_library(odbccp32i MODULE
    # odbccp32.c  # replaced by Rust (sdk/lib/rust/odbccp32i-rust/src/odbccp32i/odbccp32.rs)
    rust_cohab_stub.c
    odbccp32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/odbccp32i.def)
add_dependencies(odbccp32i rust_odbccp32i_rust)
target_link_libraries(odbccp32i ${odbccp32i_rust_LIB})

set_module_type(odbccp32i cpl UNICODE)
add_importlibs(odbccp32i user32 comctl32 msvcrt kernel32)
add_cd_file(TARGET odbccp32i DESTINATION reactos/system32 FOR all)
