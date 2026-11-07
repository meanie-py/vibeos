
# dmilib.c  # replaced by Rust (sdk/lib/rust/dmilib-rust/src/dmilib/dmilib.rs)

# Every translation unit is Rust now (dmilib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(dmilib_rust dmilib-rust "" TRUE)
add_library(dmilib INTERFACE)
add_dependencies(dmilib rust_dmilib_rust)
target_link_libraries(dmilib INTERFACE ${dmilib_rust_LIB})
add_dependencies(dmilib psdk)
target_include_directories(dmilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
