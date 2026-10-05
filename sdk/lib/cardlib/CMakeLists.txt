# cardlib — the playing-card rendering library used by sol and spider.
#
# The implementation is Rust (sdk/lib/rust/cardlib-rust), exposed through the
# plain C ABI in cardlib.h, so C and C++ callers link it unchanged. Built the
# same way as the other user-mode Rust staticlibs in the tree (see
# dll/cpl/desk): a builtin target triple on the pinned stable toolchain, no
# -Z build-std and no custom target json on any architecture.

set(CARDLIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(CARDLIB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(CARDLIB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(CARDLIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
else()
    message(FATAL_ERROR "cardlib: unsupported ARCH ${ARCH}")
endif()

rust_storage_target(CARDLIB_RUST_TARGET_DIR
    "${CMAKE_CURRENT_BINARY_DIR}/rust-cardlib-target" cardlib-rust)
set(CARDLIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcardlib_rust.a")
rust_crate_sources(CARDLIB_RUST_SRCS "${CARDLIB_RUST_WORKSPACE_DIR}" cardlib-rust)

add_custom_command(
    OUTPUT ${CARDLIB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CARDLIB_RUST_TARGET_DIR}
        cargo build --release --target ${CARDLIB_RUST_TARGET}
            -p cardlib-rust
            --manifest-path ${CARDLIB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${CARDLIB_RUST_TARGET_DIR}/${CARDLIB_RUST_TARGET}/release/libcardlib_rust.a
        ${CARDLIB_RUST_LIB}
    DEPENDS ${CARDLIB_RUST_SRCS}
            ${CARDLIB_RUST_WORKSPACE_DIR}/cardlib-rust/Cargo.toml
    WORKING_DIRECTORY ${CARDLIB_RUST_WORKSPACE_DIR}
    COMMENT "Building cardlib-rust (${CARDLIB_RUST_TARGET})"
)
add_custom_target(rust_cardlib DEPENDS ${CARDLIB_RUST_LIB})

# An INTERFACE target so `target_link_libraries(sol cardlib)` keeps working and
# carries the header search path with it.
add_library(cardlib INTERFACE)
target_include_directories(cardlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(cardlib INTERFACE ${CARDLIB_RUST_LIB})
add_dependencies(cardlib rust_cardlib psdk)
