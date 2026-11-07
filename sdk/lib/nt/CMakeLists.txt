
# Every translation unit is Rust now (nt-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(nt_rust nt-rust "" TRUE)
# entry_point.c  # replaced by Rust (sdk/lib/rust/nt-rust/src/nt/entry_point.rs)
add_library(nt INTERFACE)
add_dependencies(nt rust_nt_rust)
target_link_libraries(nt INTERFACE ${nt_rust_LIB})
add_dependencies(nt psdk)
