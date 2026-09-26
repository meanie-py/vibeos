
# All built console-util TUs (outstream/pager/screen/stream/utils) → Rust
# (sdk/lib/rust/conutils-rust, kept on disk per the never-delete rule).
# libconutils_rust.a replaces the C objects; conutils is a stub TU that links the
# Rust archive (lsalib-style cohab). instream.c was already not built. 3 arches.
list(APPEND SOURCE
    # instream.c   — not built (pre-existing)
    # outstream.c  — replaced by Rust implementation (sdk/lib/rust/conutils-rust/)
    # pager.c      — replaced by Rust implementation (sdk/lib/rust/conutils-rust/)
    # screen.c     — replaced by Rust implementation (sdk/lib/rust/conutils-rust/)
    # stream.c     — replaced by Rust implementation (sdk/lib/rust/conutils-rust/)
    # utils.c      — replaced by Rust implementation (sdk/lib/rust/conutils-rust/)
    # conutils.h
    rust_cohab_stub.c
    # C variadic forwarders (ConPrintf/ConResPrintf/&c.) — Rust can't define C
    # variadics, so the `...`->va_list bridges to conutils-rust's *V cores live
    # here (were in outstream.c, no longer built).
    conutils_variadic_shims.c
    )

set(CONUTILS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(CONUTILS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-conutils-target" conutils-rust)
set(CONUTILS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libconutils_rust.a")
if(ARCH STREQUAL "i386")
    set(CONUTILS_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(CONUTILS_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(CONUTILS_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(CONUTILS_RUST_SRCS "${CONUTILS_RUST_WORKSPACE_DIR}" conutils-rust)
add_custom_command(
    OUTPUT ${CONUTILS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CONUTILS_RUST_TARGET_DIR}
        cargo build --release --target ${CONUTILS_RUST_TARGET}
            -p conutils-rust
            --manifest-path ${CONUTILS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CONUTILS_RUST_TARGET_DIR}/${CONUTILS_RUST_TARGET}/release/libconutils_rust.a
        ${CONUTILS_RUST_LIB}
    DEPENDS ${CONUTILS_RUST_SRCS}
            ${CONUTILS_RUST_WORKSPACE_DIR}/conutils-rust/Cargo.toml
    WORKING_DIRECTORY ${CONUTILS_RUST_WORKSPACE_DIR}
    COMMENT "Building conutils-rust (${CONUTILS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_conutils DEPENDS ${CONUTILS_RUST_LIB})

add_library(conutils STATIC ${SOURCE})
# add_pch(conutils conutils.h SOURCE)
add_dependencies(conutils xdk rust_conutils)
target_include_directories(conutils INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(conutils ${PSEH_LIB} ${CONUTILS_RUST_LIB})
add_importlibs(conutils msvcrt kernel32)
