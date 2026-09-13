
include_directories(inc)
add_definitions(-DUNICODE -D_UNICODE)

# NOTE: dnslib is NOT currently built — add_subdirectory(dnslib) is commented
# out in sdk/lib/CMakeLists.txt ("Nothing links to this lib."). This file is
# dormant; the cohab below is ready for if/when a consumer re-enables it.
#
# dnslib is effectively 100% Rust. The 8 FUNCTIONAL TUs (addr/dnsaddr/flatbuf/
# hostent/memory/sablob/straddr/string) → Rust (sdk/lib/rust/dnslib-rust);
# libdnslib_rust.a (linked below) provides their symbols. The other 9 .c
# (debug/dnsutil/ip6/name/print/record/rrprint/table/utf8) are EMPTY ReactOS
# placeholder stubs (header comment + #include precomp.h, 0 functions/symbols) —
# not real C debt — so they are dropped from the build too. ALL 17 .c kept on
# disk per the never-delete rule. dnslib_rust_stub.c is the anchor TU
# (add_library needs ≥1 source); it defines no symbols. See its header for why.
# 3 arches.
list(APPEND SOURCE
    # The 8 functional .c → Rust (sdk/lib/rust/dnslib-rust/):
    #   addr.c dnsaddr.c flatbuf.c hostent.c memory.c sablob.c straddr.c string.c
    # The 9 empty placeholder .c (0 symbols), dropped:
    #   debug.c dnsutil.c ip6.c name.c print.c record.c rrprint.c table.c utf8.c
    dnslib_rust_stub.c)

set(DNSLIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(DNSLIB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dnslib-target" dnslib-rust)
set(DNSLIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdnslib_rust.a")
if(ARCH STREQUAL "i386")
    set(DNSLIB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DNSLIB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DNSLIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(DNSLIB_RUST_SRCS "${DNSLIB_RUST_WORKSPACE_DIR}" dnslib-rust)
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
    DEPENDS ${DNSLIB_RUST_SRCS}
            ${DNSLIB_RUST_WORKSPACE_DIR}/dnslib-rust/Cargo.toml
    WORKING_DIRECTORY ${DNSLIB_RUST_WORKSPACE_DIR}
    COMMENT "Building dnslib-rust (${DNSLIB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_dnslib DEPENDS ${DNSLIB_RUST_LIB})

add_library(dnslib ${SOURCE})
add_pch(dnslib inc/precomp.h)
add_dependencies(dnslib psdk rust_dnslib)
target_link_libraries(dnslib ${DNSLIB_RUST_LIB})
