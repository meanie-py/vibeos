
# Every translation unit is Rust now (nt-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(nt_rust nt-rust "" TRUE)
add_library(nt
    # entry_point.c  # replaced by Rust (sdk/lib/rust/nt-rust/src/nt/entry_point.rs)
    rust_cohab_stub.c
    )
add_dependencies(nt rust_nt_rust)
target_link_libraries(nt ${nt_rust_LIB})
add_dependencies(nt psdk)
