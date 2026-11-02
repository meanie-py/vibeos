
# FIXFIX: You guys should move this header in include/reactos/libs/iphlpapi!
include_directories(${REACTOS_SOURCE_DIR}/dll/win32/iphlpapi)
list(APPEND SOURCE
    # enum.c  # replaced by Rust (sdk/lib/rust/tdilib-rust/src/tdilib/enum.rs)
    # handle.c  # replaced by Rust (sdk/lib/rust/tdilib-rust/src/tdilib/handle.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (tdilib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tdilib_rust tdilib-rust "" TRUE)
add_library(tdilib STATIC ${SOURCE})
add_dependencies(tdilib rust_tdilib_rust)
target_link_libraries(tdilib ${tdilib_rust_LIB})
add_pch(tdilib precomp.h SOURCE)
add_dependencies(tdilib psdk)
target_include_directories(tdilib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
