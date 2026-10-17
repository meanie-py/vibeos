
add_definitions(
    -DUNICODE -D_UNICODE)
if(DBG)
    add_compile_definitions(NDEBUG)
endif()

include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound)

list(APPEND SOURCE
    # controls.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/controls.rs)
    # filter.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/filter.rs)
    # midi.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/midi.rs)
    # mixer.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/mixer.rs)
    # sup.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/sup.rs)
    # wave.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/wave.rs)
    # topology.c  # replaced by Rust (sdk/lib/rust/mmixer-rust/src/mmixer/topology.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (mmixer-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmixer_rust mmixer-rust "" TRUE)
add_library(mmixer STATIC ${SOURCE})
add_dependencies(mmixer rust_mmixer_rust)
target_link_libraries(mmixer ${mmixer_rust_LIB})
add_pch(mmixer precomp.h SOURCE)
add_dependencies(mmixer bugcodes xdk)
target_include_directories(mmixer INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
