
list(APPEND SOURCE
    # dmilib.c  # replaced by Rust (sdk/lib/rust/dmilib-rust/src/dmilib/dmilib.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (dmilib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dmilib_rust dmilib-rust "" TRUE)
add_library(dmilib STATIC ${SOURCE})
add_dependencies(dmilib rust_dmilib_rust)
target_link_libraries(dmilib ${dmilib_rust_LIB})
add_pch(dmilib precomp.h SOURCE)
add_dependencies(dmilib psdk)
target_include_directories(dmilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
