# inflib: the cross-compiled `inflib` target (infcore/infget/infput/infros{gen,get,put})
# is translated to Rust (sdk/lib/rust/inflib-rust). The kernel/setup `inflib` keeps only a
# stub TU and links libinflib_rust.a. The host `inflibhost` target (infhost*.c, Linux libc)
# is unchanged. Mirrors rossym/cmlib/evtlib cohab.

if(CMAKE_CROSSCOMPILING)
    # ── kernel/setup: drop the 6 flipped .c, link libinflib_rust.a ──────────
    add_library(inflib inflib_rust_stub.c)
    # upstream publishes infros.h through the target; its consumers dropped their own
    # include_directories line in the same commit, so this has to live here now.
    target_include_directories(inflib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
    add_dependencies(inflib psdk)

    set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/inflib-rust-target" inflib-rust)
    if(ARCH STREQUAL "i386")
        set(RUST_TARGET "i686-pc-windows-gnu")
        # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0).
        set(INFLIB_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
    elseif(ARCH STREQUAL "amd64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
        set(INFLIB_RUST_ENV "RUSTFLAGS=")
    elseif(ARCH STREQUAL "arm64")
        set(RUST_TARGET "aarch64-pc-windows-gnullvm")
        set(INFLIB_RUST_ENV "RUSTFLAGS=")
    endif()

    set(RUST_INFLIB_LIB "${CMAKE_CURRENT_BINARY_DIR}/libinflib_rust.a")
    file(GLOB_RECURSE INFLIB_RUST_SOURCES CONFIGURE_DEPENDS
        ${RUST_WORKSPACE_DIR}/inflib-rust/src/*.rs)
    add_custom_command(
        OUTPUT ${RUST_INFLIB_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${INFLIB_RUST_ENV}
            cargo build --release --target ${RUST_TARGET}
                -p inflib-rust
                --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libinflib_rust.a
            ${RUST_INFLIB_LIB}
        DEPENDS
            ${INFLIB_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/inflib-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMENT "Building Rust inflib (${RUST_TARGET})"
    )
    add_custom_target(rust_inflib DEPENDS ${RUST_INFLIB_LIB})
    add_dependencies(inflib rust_inflib)
    target_link_libraries(inflib PRIVATE ${RUST_INFLIB_LIB})
else()
    # ── host-tools (non-cross): unchanged inflibhost C build ────────────────
    list(APPEND SOURCE
        infcore.c
        infget.c
        infput.c
        infhostgen.c
        infhostget.c
        infhostput.c
        infhostrtl.c)

    add_definitions(-D__NO_CTYPE_INLINES -DINFLIB_HOST -D_CRT_SECURE_NO_WARNINGS)
    add_library(inflibhost STATIC ${SOURCE})
    target_include_directories(inflibhost INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

    target_compile_options(inflibhost PRIVATE -fshort-wchar -Wpointer-arith -Wwrite-strings)

    target_link_libraries(inflibhost PRIVATE host_includes)
endif()
