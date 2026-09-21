
# FULL flip (task #45): font.c + settings.c → Rust (concfg-rust crate). concfg
# is a SHARED leaf static lib — linked by BOTH winsrv.dll (via consrv) AND
# dll/cpl/console/console.dll (the Console CPL applet). So the Rust symbols come
# from a dedicated concfg-rust crate built here as libconcfg_rust.a and handed
# to every consumer via this target's INTERFACE. winsrv-rust depends on
# concfg-rust as an rlib for consrv's direct (`use crate::concfg::...`) calls.
# Keep the .c on disk (never delete the C — translation philosophy).
set(CONCFG_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(CONCFG_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target" concfg-rust)
if(ARCH STREQUAL "i386")
    set(CONCFG_RUST_TARGET "i686-pc-windows-gnu")
    set(CONCFG_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(CONCFG_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CONCFG_RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(CONCFG_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CONCFG_RUST_EXTRA_ENV "")
endif()
set(CONCFG_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libconcfg_rust.a")
file(GLOB_RECURSE CONCFG_RUST_SOURCES CONFIGURE_DEPENDS
    "${CONCFG_RUST_WORKSPACE_DIR}/concfg-rust/src/*.rs")
add_custom_command(
    OUTPUT ${CONCFG_RUST_LIB}
    # `cargo rustc --crate-type staticlib --features standalone`: the crate is an
    # rlib by default (winsrv-rust dep); override to a staticlib here and turn on
    # the panic_handler/eh_personality the standalone artifact needs.
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CONCFG_RUST_TARGET_DIR} ${CONCFG_RUST_EXTRA_ENV}
        cargo rustc --release --target ${CONCFG_RUST_TARGET}
            -p concfg-rust --crate-type staticlib --features standalone
            --manifest-path ${CONCFG_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CONCFG_RUST_TARGET_DIR}/${CONCFG_RUST_TARGET}/release/libconcfg_rust.a
        ${CONCFG_RUST_LIB}
    DEPENDS ${CONCFG_RUST_SOURCES}
        ${CONCFG_RUST_WORKSPACE_DIR}/concfg-rust/Cargo.toml
        ${CONCFG_RUST_WORKSPACE_DIR}/winsrv-sys/src/lib.rs
    COMMENT "Building concfg-rust staticlib (${CONCFG_RUST_TARGET})"
    VERBATIM)
add_custom_target(concfg_rust_lib DEPENDS ${CONCFG_RUST_LIB})

list(APPEND SOURCE
    # font.c     — replaced by Rust (concfg-rust/src/font.rs)
    # settings.c — replaced by Rust (concfg-rust/src/settings.rs)
    concfg_rust_stub.c
    precomp.h)

add_library(concfg ${SOURCE})
# upstream publishes concfg.h through the target; its consumers dropped their own
# include_directories line in the same commit, so this has to be here now.
target_include_directories(concfg INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(concfg_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
add_dependencies(concfg concfg_rust_lib xdk)
# Hand the Rust symbols to BOTH consumers (winsrv.dll + console.dll) via the
# static lib's INTERFACE. --allow-multiple-definition covers the libcore/
# compiler-builtins overlap (and the winsrv-rust rlib's own copy of these fns).
target_link_libraries(concfg INTERFACE ${CONCFG_RUST_LIB})
target_link_options(concfg INTERFACE "-Wl,--allow-multiple-definition")
