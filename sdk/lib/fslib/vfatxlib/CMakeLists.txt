
list(APPEND SOURCE
    # fatx.c  # replaced by Rust (sdk/lib/rust/vfatxlib-rust/src/vfatxlib/fatx.rs)
    # vfatxlib.c  # replaced by Rust (sdk/lib/rust/vfatxlib-rust/src/vfatxlib/vfatxlib.rs)
    rust_cohab_stub.c
    vfatxlib.h)

# Every translation unit is Rust now (vfatxlib-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(vfatxlib_rust vfatxlib-rust "" TRUE)
add_library(vfatxlib STATIC ${SOURCE})
add_dependencies(vfatxlib rust_vfatxlib_rust)
target_link_libraries(vfatxlib ${vfatxlib_rust_LIB})
add_pch(vfatxlib vfatxlib.h SOURCE)
target_link_libraries(vfatxlib chkstk)

if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_link_libraries(vfatxlib -lgcc)
endif()

add_dependencies(vfatxlib psdk)
