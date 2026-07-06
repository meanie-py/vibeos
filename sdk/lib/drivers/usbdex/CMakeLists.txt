
add_definitions(-D_NTSYSTEM_)

# ============================================================================
# usbdex — Rust cohabitation flip (usbdex.c -> sdk/lib/rust/usbdex-rust)
# ============================================================================
# usbdex.c (the "UsbdEx lib") is translated to Rust (usbdex-rust, kept on disk
# per the never-delete rule). The 7 USBD_* symbols (plain cdecl — usbdlib.h has
# no NTAPI on them) come from libusbdex_rust.a, linked PUBLIC into the usbdex
# target so every USB consumer picks up the Rust impl. Mirrors cportlib's
# static-lib cohabitation.
set(USBDEX_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(USBDEX_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbdex-target")
set(USBDEX_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbdex_rust.a")
if(ARCH STREQUAL "i386")
    set(USBDEX_RUST_TARGET "i686-pc-windows-gnu")
    set(USBDEX_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBDEX_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBDEX_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBDEX_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBDEX_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE USBDEX_RUST_SRCS ${USBDEX_RUST_WORKSPACE_DIR}/usbdex-rust/src/*.rs)
add_custom_command(
    OUTPUT ${USBDEX_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBDEX_RUST_TARGET_DIR}
        RUSTFLAGS=${USBDEX_RUST_RUSTFLAGS}
        cargo build --release --target ${USBDEX_RUST_TARGET}
            -p usbdex-rust
            --manifest-path ${USBDEX_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBDEX_RUST_TARGET_DIR}/${USBDEX_RUST_TARGET}/release/libusbdex_rust.a
        ${USBDEX_RUST_LIB}
    DEPENDS ${USBDEX_RUST_SRCS}
            ${USBDEX_RUST_WORKSPACE_DIR}/usbdex-rust/Cargo.toml
    WORKING_DIRECTORY ${USBDEX_RUST_WORKSPACE_DIR}
    COMMENT "Building usbdex-rust (${USBDEX_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_usbdex DEPENDS ${USBDEX_RUST_LIB})

# add_library(usbdex usbdex.c)  # replaced by Rust (usbdex-rust); kept on disk
add_library(usbdex rust_cohab_stub.c)
add_dependencies(usbdex xdk rust_usbdex)
target_link_libraries(usbdex PUBLIC ${USBDEX_RUST_LIB})
