
# Every translation unit is Rust now (apisets-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(apisets_rust apisets-rust "" TRUE)
# apisets.c  # replaced by Rust (sdk/lib/rust/apisets-rust/src/apisets/apisets.rs)
# Generated file:
# apisets.table.c  # replaced by Rust (sdk/lib/rust/apisets-rust/src/apisets/apisets_table/mod.rs, generated: port data)
add_library(apisets INTERFACE)
add_dependencies(apisets rust_apisets_rust)
target_link_libraries(apisets INTERFACE ${apisets_rust_LIB})

target_include_directories(apisets INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(apisets psdk)
