
# Every translation unit is Rust now (tzlib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tzlib_rust tzlib-rust "" TRUE)
add_library(tzlib STATIC
    # tzlib.c  # replaced by Rust (sdk/lib/rust/tzlib-rust/src/tzlib/tzlib.rs)
    rust_cohab_stub.c
    )
add_dependencies(tzlib rust_tzlib_rust)
target_link_libraries(tzlib ${tzlib_rust_LIB})
add_dependencies(tzlib xdk)
target_include_directories(tzlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
