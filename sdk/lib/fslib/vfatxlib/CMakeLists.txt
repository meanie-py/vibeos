
# fatx.c  # replaced by Rust (sdk/lib/rust/vfatxlib-rust/src/vfatxlib/fatx.rs)
# vfatxlib.c  # replaced by Rust (sdk/lib/rust/vfatxlib-rust/src/vfatxlib/vfatxlib.rs)

# Every translation unit is Rust now (vfatxlib-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(vfatxlib_rust vfatxlib-rust "" TRUE)
add_library(vfatxlib INTERFACE)
add_dependencies(vfatxlib rust_vfatxlib_rust)
target_link_libraries(vfatxlib INTERFACE ${vfatxlib_rust_LIB})
target_link_libraries(vfatxlib INTERFACE chkstk)

if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_link_libraries(vfatxlib INTERFACE -lgcc)
endif()

add_dependencies(vfatxlib psdk)
