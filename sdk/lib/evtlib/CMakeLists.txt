
# evtlib (EventLog file library) is translated to Rust (sdk/lib/rust/evtlib-rust).
# The single evtlib.c is flipped; the kernel `evtlib` target keeps only a stub TU
# and links libevtlib_rust.a (which provides the Elf* symbols). The C source stays
# on disk for the host-tools (non-cross) build path. Mirrors rossym/cmlib cohab.

if(CMAKE_CROSSCOMPILING)
    # ── kernel: drop evtlib.c, link libevtlib_rust.a ────────────────────────
    add_library(evtlib evtlib_rust_stub.c)
    add_dependencies(evtlib xdk)
    # upstream publishes evtlib.h through the target (its consumers dropped their
    # own include_directories line in the same commit). Mirror that on both branches.
    target_include_directories(evtlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

    set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/evtlib-rust-target" evtlib-rust)
    if(ARCH STREQUAL "i386")
        set(RUST_TARGET "i686-pc-windows-gnu")
        # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0).
        set(EVTLIB_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
    elseif(ARCH STREQUAL "amd64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
        set(EVTLIB_RUST_ENV "RUSTFLAGS=")
    elseif(ARCH STREQUAL "arm64")
        set(RUST_TARGET "aarch64-pc-windows-gnullvm")
        set(EVTLIB_RUST_ENV "RUSTFLAGS=")
    endif()

    set(RUST_EVTLIB_LIB "${CMAKE_CURRENT_BINARY_DIR}/libevtlib_rust.a")
    file(GLOB_RECURSE EVTLIB_RUST_SOURCES CONFIGURE_DEPENDS
        ${RUST_WORKSPACE_DIR}/evtlib-rust/src/*.rs)
    add_custom_command(
        OUTPUT ${RUST_EVTLIB_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${EVTLIB_RUST_ENV}
            cargo build --release --target ${RUST_TARGET}
                -p evtlib-rust
                --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libevtlib_rust.a
            ${RUST_EVTLIB_LIB}
        DEPENDS
            ${EVTLIB_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/evtlib-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMENT "Building Rust evtlib (${RUST_TARGET})"
    )
    add_custom_target(rust_evtlib DEPENDS ${RUST_EVTLIB_LIB})
    add_dependencies(evtlib rust_evtlib)
    target_link_libraries(evtlib PRIVATE ${RUST_EVTLIB_LIB})
else()
    # ── host-tools (non-cross): keep the C build ────────────────────────────
    add_library(evtlib evtlib.c)
    add_dependencies(evtlib xdk)
    target_include_directories(evtlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
endif()
