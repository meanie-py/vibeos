# ============================================================================
# arbiter (PnP resource arbiter) — Rust cohabitation flip (arbiter.c -> arbiter-rust)
# ============================================================================
# arbiter.c is translated to Rust (arbiter-rust, kept on disk per the never-delete
# rule). libarbiter_rust.a replaces the C object; the `arbiter` static lib target
# is preserved (ntoskrnl + pcix.sys link it by name) as a stub TU that PUBLIC-links
# the Rust archive (usbdex-style cohab). arbiter-rust depends on ntkrnl-sys
# (ARBITER_INSTANCE etc live in the ddk bindgen). Built for all 3 arches.
add_definitions(-D_NTSYSTEM_)

set(ARBITER_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(ARBITER_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-arbiter-target" arbiter-rust)
set(ARBITER_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libarbiter_rust.a")
if(ARCH STREQUAL "i386")
    set(ARBITER_RUST_TARGET "i686-pc-windows-gnu")
    set(ARBITER_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(ARBITER_RUST_TARGET "x86_64-pc-windows-gnu")
    set(ARBITER_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(ARBITER_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(ARBITER_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(ARBITER_RUST_SRCS "${ARBITER_RUST_WORKSPACE_DIR}" arbiter-rust)
add_custom_command(
    OUTPUT ${ARBITER_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ARBITER_RUST_TARGET_DIR}
        RUSTFLAGS=${ARBITER_RUST_RUSTFLAGS}
        cargo build --release --target ${ARBITER_RUST_TARGET}
            -p arbiter-rust
            --manifest-path ${ARBITER_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ARBITER_RUST_TARGET_DIR}/${ARBITER_RUST_TARGET}/release/libarbiter_rust.a
        ${ARBITER_RUST_LIB}
    DEPENDS ${ARBITER_RUST_SRCS}
            ${ARBITER_RUST_WORKSPACE_DIR}/arbiter-rust/Cargo.toml
    WORKING_DIRECTORY ${ARBITER_RUST_WORKSPACE_DIR}
    COMMENT "Building arbiter-rust (${ARBITER_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_arbiter DEPENDS ${ARBITER_RUST_LIB})

# add_library(arbiter arbiter.c)  # body replaced by Rust; kept on disk
add_library(arbiter rust_cohab_stub.c)
# upstream publishes arbiter.h through the target (its consumers, e.g. pcix, dropped
# their own include_directories line in the same commit). Mirror that here.
target_include_directories(arbiter INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(arbiter bugcodes xdk rust_arbiter)
target_link_libraries(arbiter PUBLIC ${ARBITER_RUST_LIB})
