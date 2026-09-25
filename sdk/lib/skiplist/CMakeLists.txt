
# The library
add_definitions(-DSKIPLIST_LEVELS=16)

# skiplist.c → Rust (sdk/lib/rust/skiplist-rust, kept on disk per the never-delete
# rule). libskiplist_rust.a replaces the C object; the skiplist16 target is a stub
# TU that PUBLIC-links the Rust archive (lsalib-style cohab). Built for all 3 arches.
set(SKIPLIST_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(SKIPLIST_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-skiplist-target" skiplist-rust)
set(SKIPLIST_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libskiplist_rust.a")
if(ARCH STREQUAL "i386")
    set(SKIPLIST_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SKIPLIST_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SKIPLIST_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(SKIPLIST_RUST_SRCS "${SKIPLIST_RUST_WORKSPACE_DIR}" skiplist-rust)
add_custom_command(
    OUTPUT ${SKIPLIST_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SKIPLIST_RUST_TARGET_DIR}
        cargo build --release --target ${SKIPLIST_RUST_TARGET}
            -p skiplist-rust
            --manifest-path ${SKIPLIST_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SKIPLIST_RUST_TARGET_DIR}/${SKIPLIST_RUST_TARGET}/release/libskiplist_rust.a
        ${SKIPLIST_RUST_LIB}
    DEPENDS ${SKIPLIST_RUST_SRCS}
            ${SKIPLIST_RUST_WORKSPACE_DIR}/skiplist-rust/Cargo.toml
    WORKING_DIRECTORY ${SKIPLIST_RUST_WORKSPACE_DIR}
    COMMENT "Building skiplist-rust (${SKIPLIST_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_skiplist DEPENDS ${SKIPLIST_RUST_LIB})

# add_library(skiplist16 skiplist.c)  # body replaced by Rust; kept on disk
add_library(skiplist16 rust_cohab_stub.c)
# upstream publishes skiplist.h through the target; its consumers dropped their own
# include_directories line in the same commit, so this has to live here now.
target_include_directories(skiplist16 INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(skiplist16 psdk rust_skiplist)
target_link_libraries(skiplist16 PUBLIC ${SKIPLIST_RUST_LIB})

# The Test program
add_executable(skiplist_test skiplist_test.c)
target_link_libraries(skiplist_test skiplist16)
set_module_type(skiplist_test win32cui)
add_importlibs(skiplist_test msvcrt kernel32 ntdll)
