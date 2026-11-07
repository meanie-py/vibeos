
# Every translation unit is Rust now (sptilib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(sptilib_rust sptilib-rust "" TRUE)
# sptilib.c  # replaced by Rust (sdk/lib/rust/sptilib-rust/src/sptilib/sptilib.rs)
add_library(sptilib INTERFACE)
add_dependencies(sptilib rust_sptilib_rust)
target_link_libraries(sptilib INTERFACE $<LINK_ONLY:${sptilib_rust_LIB}>)
target_link_libraries(sptilib INTERFACE $<LINK_ONLY:${PSEH_LIB}>)
add_dependencies(sptilib xdk)
target_include_directories(sptilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
