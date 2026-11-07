
# FIXFIX: You guys should move this header in include/reactos/libs/iphlpapi!
include_directories(${REACTOS_SOURCE_DIR}/dll/win32/iphlpapi)
# enum.c  # replaced by Rust (sdk/lib/rust/tdilib-rust/src/tdilib/enum.rs)
# handle.c  # replaced by Rust (sdk/lib/rust/tdilib-rust/src/tdilib/handle.rs)

# Every translation unit is Rust now (tdilib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(tdilib_rust tdilib-rust "" TRUE)
add_library(tdilib INTERFACE)
add_dependencies(tdilib rust_tdilib_rust)
target_link_libraries(tdilib INTERFACE ${tdilib_rust_LIB})
add_dependencies(tdilib psdk)
target_include_directories(tdilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
