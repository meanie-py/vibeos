
# Every translation unit is Rust now (sptilib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sptilib_rust sptilib-rust "" TRUE)
add_library(sptilib STATIC
    # sptilib.c  # replaced by Rust (sdk/lib/rust/sptilib-rust/src/sptilib/sptilib.rs)
    rust_cohab_stub.c
    )
add_dependencies(sptilib rust_sptilib_rust)
target_link_libraries(sptilib PRIVATE ${sptilib_rust_LIB})
target_link_libraries(sptilib PRIVATE ${PSEH_LIB})
add_dependencies(sptilib xdk)
target_include_directories(sptilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
