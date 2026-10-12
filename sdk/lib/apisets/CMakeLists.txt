
# Every translation unit is Rust now (apisets-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(apisets_rust apisets-rust "" TRUE)
add_library(apisets
    # apisets.c  # replaced by Rust (sdk/lib/rust/apisets-rust/src/apisets/apisets.rs)
    apisets.h
    apisetsp.h

    # Generated file:
    # apisets.table.c  # replaced by Rust (sdk/lib/rust/apisets-rust/src/apisets/apisets_table/mod.rs, generated: port data)
    rust_cohab_stub.c
    )
add_dependencies(apisets rust_apisets_rust)
target_link_libraries(apisets ${apisets_rust_LIB})

target_include_directories(apisets INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_definitions(apisets PRIVATE _NTSYSTEM_)
add_dependencies(apisets psdk)
