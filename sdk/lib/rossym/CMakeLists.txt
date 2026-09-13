
add_definitions(-D_NTSYSTEM_)

# rossym is 100% translated to Rust (sdk/lib/rust/rossym-rust). All ten .c are
# flipped; the kernel `rossym` target keeps only a stub TU and links
# librossym_rust.a (which provides the RosSym* cdecl symbols). The C sources
# stay on disk for the host-tools (non-cross) build path. Mirrors cmlib's cohab.
set(ROSSYM_C_SOURCE
    delete.c
    find.c
    fromfile.c
    frommem.c
    fromraw.c
    getraw.c
    init.c
    initkm.c
    initum.c
    zwfile.c)

if(CMAKE_CROSSCOMPILING)
    # ── kernel: drop all flipped .c, link librossym_rust.a ──────────────────
    add_library(rossym rossym_rust_stub.c)
    add_dependencies(rossym psdk bugcodes)
    target_compile_definitions(rossym INTERFACE __ROS_ROSSYM__)

    set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rossym-rust-target" rossym-rust)
    # Like cmlib-rust (and unlike libcntpr), do NOT pass `-C no-prepopulate-passes`:
    # normal optimization collapses each RosSym* helper to a self-contained body
    # with no precompiled-core deps, which is what keeps the cohab archive safe.
    if(ARCH STREQUAL "i386")
        set(RUST_TARGET "i686-pc-windows-gnu")
        # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0).
        set(ROSSYM_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
    elseif(ARCH STREQUAL "amd64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
        set(ROSSYM_RUST_ENV "RUSTFLAGS=")
    elseif(ARCH STREQUAL "arm64")
        set(RUST_TARGET "aarch64-pc-windows-gnullvm")
        set(ROSSYM_RUST_ENV "RUSTFLAGS=")
    endif()

    set(RUST_ROSSYM_LIB "${CMAKE_CURRENT_BINARY_DIR}/librossym_rust.a")
    file(GLOB_RECURSE ROSSYM_RUST_SOURCES CONFIGURE_DEPENDS
        ${RUST_WORKSPACE_DIR}/rossym-rust/src/*.rs)
    add_custom_command(
        OUTPUT ${RUST_ROSSYM_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${ROSSYM_RUST_ENV}
            cargo build --release --target ${RUST_TARGET}
                -p rossym-rust
                --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RUST_TARGET_DIR}/${RUST_TARGET}/release/librossym_rust.a
            ${RUST_ROSSYM_LIB}
        DEPENDS
            ${ROSSYM_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/rossym-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMENT "Building Rust rossym umbrella (${RUST_TARGET})"
    )
    add_custom_target(rust_rossym DEPENDS ${RUST_ROSSYM_LIB})
    add_dependencies(rossym rust_rossym)
    target_link_libraries(rossym PRIVATE ${RUST_ROSSYM_LIB})
else()
    # ── host-tools (non-cross): keep the C build ────────────────────────────
    add_library(rossym ${ROSSYM_C_SOURCE})
    add_dependencies(rossym psdk bugcodes)
    target_compile_definitions(rossym INTERFACE __ROS_ROSSYM__)
endif()
