# cryptlib (LibTomCrypt-derived crypto: aes/des/md4/md5/mvAesAlg/rc4/sha1/util) is
# translated to Rust (sdk/lib/rust/cryptlib-rust). The kernel `cryptlib` target keeps
# only a stub TU and links libcryptlib_rust.a (consumed by ksecdd.sys). The C sources
# stay on disk for the host-tools (non-cross) build path. Mirrors rossym/cmlib cohab.

if(CMAKE_CROSSCOMPILING)
    # ── kernel: drop all 8 flipped .c, link libcryptlib_rust.a ──────────────
    add_library(cryptlib cryptlib_rust_stub.c)
    add_dependencies(cryptlib xdk)

    set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cryptlib-rust-target" cryptlib-rust)
    if(ARCH STREQUAL "i386")
        set(RUST_TARGET "i686-pc-windows-gnu")
        # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0).
        set(CRYPTLIB_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
    elseif(ARCH STREQUAL "amd64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
        set(CRYPTLIB_RUST_ENV "RUSTFLAGS=")
    elseif(ARCH STREQUAL "arm64")
        set(RUST_TARGET "aarch64-pc-windows-gnullvm")
        set(CRYPTLIB_RUST_ENV "RUSTFLAGS=")
    endif()

    set(RUST_CRYPTLIB_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcryptlib_rust.a")
    file(GLOB_RECURSE CRYPTLIB_RUST_SOURCES CONFIGURE_DEPENDS
        ${RUST_WORKSPACE_DIR}/cryptlib-rust/src/*.rs)
    add_custom_command(
        OUTPUT ${RUST_CRYPTLIB_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${CRYPTLIB_RUST_ENV}
            cargo build --release --target ${RUST_TARGET}
                -p cryptlib-rust
                --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libcryptlib_rust.a
            ${RUST_CRYPTLIB_LIB}
        DEPENDS
            ${CRYPTLIB_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/cryptlib-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMENT "Building Rust cryptlib (${RUST_TARGET})"
    )
    add_custom_target(rust_cryptlib DEPENDS ${RUST_CRYPTLIB_LIB})
    add_dependencies(cryptlib rust_cryptlib)
    target_link_libraries(cryptlib PRIVATE ${RUST_CRYPTLIB_LIB})
else()
    # ── host-tools (non-cross): keep the C build ────────────────────────────
    list(APPEND SOURCE
        aes.c
        des.c
        md4.c
        md5.c
        mvAesAlg.c
        rc4.c
        sha1.c
        util.c)

    add_library(cryptlib ${SOURCE})
    add_dependencies(cryptlib xdk)
endif()
