
list(APPEND SOURCE
    # udmihelp.c  # replaced by Rust (sdk/lib/rust/udmihelp-rust/src/udmihelp/udmihelp.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (udmihelp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(udmihelp_rust udmihelp-rust "" TRUE)
add_library(udmihelp STATIC ${SOURCE})
add_dependencies(udmihelp rust_udmihelp_rust)
target_link_libraries(udmihelp ${udmihelp_rust_LIB})
add_pch(udmihelp precomp.h SOURCE)
target_include_directories(udmihelp INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(udmihelp dmilib)
add_dependencies(udmihelp psdk)
