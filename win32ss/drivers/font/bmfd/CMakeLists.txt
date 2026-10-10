
list(APPEND SOURCE
    # enable.c  # replaced by Rust (sdk/lib/rust/bmfd-rust/src/bmfd/enable.rs)
    # font.c  # replaced by Rust (sdk/lib/rust/bmfd-rust/src/bmfd/font.rs)
    # glyph.c  # replaced by Rust (sdk/lib/rust/bmfd-rust/src/bmfd/glyph.rs)
    rust_varargs_shim.c
    rust_cohab_stub.c
    bmfd.h)

# Every translation unit is Rust now (bmfd-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(bmfd_rust bmfd-rust "" TRUE)
add_library(bmfd MODULE ${SOURCE})
add_dependencies(bmfd rust_bmfd_rust)
target_link_libraries(bmfd ${bmfd_rust_LIB})
set_module_type(bmfd kerneldll ENTRYPOINT BmfdEnableDriver 12)
target_link_libraries(bmfd libcntpr)
add_importlibs(bmfd win32k)
add_pch(bmfd bmfd.h SOURCE)
add_dependencies(bmfd psdk)
