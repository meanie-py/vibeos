
include_directories(inc)
add_definitions(-DUNICODE -D_UNICODE)

# 8 of the 17 dnslib TUs (addr/dnsaddr/flatbuf/hostent/memory/sablob/straddr/
# string) → Rust (sdk/lib/rust/dnslib-rust, kept on disk per the never-delete
# rule). libdnslib_rust.a provides those symbols; the other 9 .c stay C. 3 arches.
list(APPEND SOURCE
    # addr.c     — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    debug.c
    # dnsaddr.c  — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    dnsutil.c
    # flatbuf.c  — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    # hostent.c  — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    ip6.c
    # memory.c   — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    name.c
    print.c
    record.c
    rrprint.c
    # sablob.c   — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    # straddr.c  — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    # string.c   — replaced by Rust implementation (sdk/lib/rust/dnslib-rust/)
    table.c
    utf8.c)

set(DNSLIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(DNSLIB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dnslib-target")
set(DNSLIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdnslib_rust.a")
if(ARCH STREQUAL "i386")
    set(DNSLIB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DNSLIB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DNSLIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
add_custom_command(
    OUTPUT ${DNSLIB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DNSLIB_RUST_TARGET_DIR}
        cargo build --release --target ${DNSLIB_RUST_TARGET}
            -p dnslib-rust
            --manifest-path ${DNSLIB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${DNSLIB_RUST_TARGET_DIR}/${DNSLIB_RUST_TARGET}/release/libdnslib_rust.a
        ${DNSLIB_RUST_LIB}
    DEPENDS ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/Cargo.toml
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/lib.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/addr.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/dnsaddr.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/flatbuf.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/hostent.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/memory.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/sablob.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/straddr.rs
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/src/dnslib/string.rs
    WORKING_DIRECTORY ${DNSLIB_RUST_WORKSPACE_DIR}
    COMMENT "Building dnslib-rust (${DNSLIB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_dnslib DEPENDS ${DNSLIB_RUST_LIB})

add_library(dnslib ${SOURCE})
add_pch(dnslib inc/precomp.h)
add_dependencies(dnslib psdk rust_dnslib)
target_link_libraries(dnslib ${DNSLIB_RUST_LIB})
