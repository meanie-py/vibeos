
add_definitions(
    -DUNICODE -D_UNICODE
    -DDEBUG_NT4=1)

include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound)

list(APPEND SOURCE
    # detect.c  # replaced by Rust (sdk/lib/rust/mment4-rust/src/mment4/detect.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/mment4-rust/src/mment4/registry.rs)
    # control.c  # replaced by Rust (sdk/lib/rust/mment4-rust/src/mment4/control.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (mment4-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mment4_rust mment4-rust "" TRUE)
add_library(mment4 ${SOURCE})
add_dependencies(mment4 rust_mment4_rust)
target_link_libraries(mment4 ${mment4_rust_LIB})
add_pch(mment4 precomp.h SOURCE)
add_dependencies(mment4 psdk)
