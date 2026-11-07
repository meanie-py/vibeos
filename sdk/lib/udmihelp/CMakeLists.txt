
# udmihelp.c  # replaced by Rust (sdk/lib/rust/udmihelp-rust/src/udmihelp/udmihelp.rs)

# Every translation unit is Rust now (udmihelp-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(udmihelp_rust udmihelp-rust "" TRUE)
add_library(udmihelp INTERFACE)
add_dependencies(udmihelp rust_udmihelp_rust)
target_link_libraries(udmihelp INTERFACE ${udmihelp_rust_LIB})
target_include_directories(udmihelp INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(udmihelp INTERFACE dmilib)
add_dependencies(udmihelp psdk)
