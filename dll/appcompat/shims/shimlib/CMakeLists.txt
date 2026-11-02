
list(APPEND SOURCE
    # shimlib.c  # replaced by Rust (sdk/lib/rust/shimlib-rust/src/shimlib/shimlib.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    shimlib.h
    # These .inl functions are included so they show up in vs
    setup_shim.inl
    implement_shim.inl)

# Every translation unit is Rust now (shimlib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(shimlib_rust shimlib-rust "" TRUE)
add_library(shimlib ${SOURCE})
add_dependencies(shimlib rust_shimlib_rust)
target_link_libraries(shimlib ${shimlib_rust_LIB})
add_dependencies(shimlib xdk)
