
add_definitions(-DUNICODE -D_UNICODE)
# Every translation unit is Rust now (scrnsave-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(scrnsave_rust scrnsave-rust "" TRUE)
# scrnsave.c  # replaced by Rust (sdk/lib/rust/scrnsave-rust/src/scrnsave/scrnsave.rs)
add_library(scrnsave INTERFACE)
add_dependencies(scrnsave rust_scrnsave_rust)
target_link_libraries(scrnsave INTERFACE ${scrnsave_rust_LIB})
add_dependencies(scrnsave psdk)
