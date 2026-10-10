
spec2def(ftfd.dll ftfd.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # enable.c  # replaced by Rust (sdk/lib/rust/ftfd-rust/src/ftfd/enable.rs)
    # font.c  # replaced by Rust (sdk/lib/rust/ftfd-rust/src/ftfd/font.rs)
    # glyph.c  # replaced by Rust (sdk/lib/rust/ftfd-rust/src/ftfd/glyph.rs)
    # rosglue.c  # replaced by Rust (sdk/lib/rust/ftfd-rust/src/ftfd/rosglue.rs)
    )

list(APPEND PCH_SKIP_SOURCE
    # sprintf.c  # replaced by Rust (sdk/lib/rust/ftfd-rust/src/ftfd/sprintf.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    )

# Every translation unit is Rust now (ftfd-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ftfd_rust ftfd-rust "" TRUE)
add_library(ftfd MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    freetype.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ftfd.def)
add_dependencies(ftfd rust_ftfd_rust)
target_link_libraries(ftfd ${ftfd_rust_LIB})
add_pch(ftfd ftfd.h "${PCH_SKIP_SOURCE}")

set_module_type(ftfd kerneldll ENTRYPOINT FtfdEnableDriver 12)
target_link_libraries(ftfd freetype libcntpr setjmp ${PSEH_LIB})
add_importlibs(ftfd win32k)
add_dependencies(ftfd psdk)
add_cd_file(TARGET ftfd DESTINATION reactos/system32 FOR all)
