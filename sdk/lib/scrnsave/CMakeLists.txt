
add_definitions(-DUNICODE -D_UNICODE)
# Every translation unit is Rust now (scrnsave-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(scrnsave_rust scrnsave-rust "" TRUE)
add_library(scrnsave
    # scrnsave.c  # replaced by Rust (sdk/lib/rust/scrnsave-rust/src/scrnsave/scrnsave.rs)
    rust_cohab_stub.c
    )
add_dependencies(scrnsave rust_scrnsave_rust)
target_link_libraries(scrnsave ${scrnsave_rust_LIB})
add_dependencies(scrnsave psdk)
