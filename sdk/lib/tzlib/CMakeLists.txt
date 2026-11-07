
# Every translation unit is Rust now (tzlib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(tzlib_rust tzlib-rust "" TRUE)
# tzlib.c  # replaced by Rust (sdk/lib/rust/tzlib-rust/src/tzlib/tzlib.rs)
add_library(tzlib INTERFACE)
add_dependencies(tzlib rust_tzlib_rust)
target_link_libraries(tzlib INTERFACE ${tzlib_rust_LIB})
add_dependencies(tzlib xdk)
target_include_directories(tzlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
