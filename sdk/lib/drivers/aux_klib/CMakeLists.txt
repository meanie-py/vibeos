
# Every translation unit is Rust now (aux_klib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(aux_klib_rust aux_klib-rust "" TRUE)
# aux_klib.c  # replaced by Rust (sdk/lib/rust/aux_klib-rust/src/aux_klib/aux_klib.rs)
add_library(aux_klib INTERFACE)
add_dependencies(aux_klib rust_aux_klib_rust)
target_link_libraries(aux_klib INTERFACE ${aux_klib_rust_LIB})
target_link_libraries(aux_klib INTERFACE ${PSEH_LIB})
target_link_libraries(aux_klib INTERFACE libntoskrnl)
