
# ============================================================================
# cportlib — Rust cohabitation flip (cport.c -> sdk/lib/rust/cport-rust)
# ============================================================================
# cport.c (the NS16550 serial library) is translated to Rust (cport-rust,
# kept on disk per the never-delete rule). The Cp* NTAPI exports come from
# libcport_rust.a, linked PUBLIC into the cportlib target so every consumer
# picks up the Rust impl: cmake consumers (ntoskrnl / ntkrnlmp / kdgdb) get it
# via propagation; the cargo-driven consumers (kdcom/uefildr/rosload) add
# libcport_rust.a to their own build.rs static_libs (they reference
# libcportlib.a by path, which rust_cohab_stub.c keeps present).
set(CPORT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(CPORT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-cport-target")
set(CPORT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcport_rust.a")
if(ARCH STREQUAL "i386")
    set(CPORT_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(CPORT_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(CPORT_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(CPORT_RUST_SRCS "${CPORT_RUST_WORKSPACE_DIR}" cport-rust)
add_custom_command(
    OUTPUT ${CPORT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CPORT_RUST_TARGET_DIR}
        cargo build --release --target ${CPORT_RUST_TARGET}
            -p cport-rust
            --manifest-path ${CPORT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CPORT_RUST_TARGET_DIR}/${CPORT_RUST_TARGET}/release/libcport_rust.a
        ${CPORT_RUST_LIB}
    DEPENDS ${CPORT_RUST_SRCS}
            ${CPORT_RUST_WORKSPACE_DIR}/cport-rust/Cargo.toml
    WORKING_DIRECTORY ${CPORT_RUST_WORKSPACE_DIR}
    COMMENT "Building cport-rust (${CPORT_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_cport DEPENDS ${CPORT_RUST_LIB})

# add_library(cportlib cport.c)  # replaced by Rust (cport-rust); kept on disk
add_library(cportlib rust_cohab_stub.c)
add_dependencies(cportlib xdk rust_cport)
target_link_libraries(cportlib PUBLIC ${CPORT_RUST_LIB})
