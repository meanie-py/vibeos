
add_definitions(
    -DUNICODE -D_UNICODE)
if(DBG)
    add_compile_definitions(NDEBUG)
endif()

include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound)

list(APPEND SOURCE
    # capabilities.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/capabilities.rs)
    # devicelist.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/devicelist.rs)
    # deviceinstance.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/deviceinstance.rs)
    # functiontable.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/functiontable.rs)
    # mmewrap.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/mmewrap.rs)
    # reentrancy.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/reentrancy.rs)
    # utility.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/utility.rs)
    # kernel.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/kernel.rs)
    # thread.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/thread.rs)
    # auxiliary/auxMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/auxMessage.rs)
    # midi/midMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/midMessage.rs)
    # midi/modMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/modMessage.rs)
    # mixer/mxdMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/mxdMessage.rs)
    # wave/widMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/widMessage.rs)
    # wave/wodMessage.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/wodMessage.rs)
    # wave/format.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/format.rs)
    # wave/header.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/header.rs)
    # wave/streaming.c  # replaced by Rust (sdk/lib/rust/mmebuddy-rust/src/mmebuddy/streaming.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (mmebuddy-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmebuddy_rust mmebuddy-rust "" TRUE)
add_library(mmebuddy ${SOURCE})
add_dependencies(mmebuddy rust_mmebuddy_rust)
target_link_libraries(mmebuddy ${mmebuddy_rust_LIB})
add_pch(mmebuddy precomp.h SOURCE)
add_dependencies(mmebuddy psdk)
