
# Every translation unit is Rust now (aux_klib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(aux_klib_rust aux_klib-rust "" TRUE)
add_library(aux_klib
    # aux_klib.c  # replaced by Rust (sdk/lib/rust/aux_klib-rust/src/aux_klib/aux_klib.rs)
    rust_cohab_stub.c
    )
add_dependencies(aux_klib rust_aux_klib_rust)
target_link_libraries(aux_klib ${aux_klib_rust_LIB})
target_link_libraries(aux_klib ${PSEH_LIB})
add_importlibs(aux_klib ntoskrnl)
