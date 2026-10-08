
spec2def(mprapi.dll mprapi.spec)

# mprapi.dll — Rust cohabitation flip (FULL: the single compiled src *.c → mprapi-rust).
# The RAS/router admin API is almost entirely stubs; rustconv lowered one of the three
# functions deterministically for $0. mprapi.c is commented out below and kept on disk per
# the never-delete rule.
set(MPRAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(MPRAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MPRAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MPRAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(MPRAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mprapi-target" mprapi-rust)
set(MPRAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmprapi_rust.a")
rust_crate_sources(MPRAPI_RUST_SRCS "${MPRAPI_RUST_WORKSPACE_DIR}" mprapi-rust)
add_custom_command(
    OUTPUT ${MPRAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MPRAPI_RUST_TARGET_DIR}
        cargo build --release --target ${MPRAPI_RUST_TARGET}
            -p mprapi-rust
            --manifest-path ${MPRAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${MPRAPI_RUST_TARGET_DIR}/${MPRAPI_RUST_TARGET}/release/libmprapi_rust.a
        ${MPRAPI_RUST_LIB}
    DEPENDS ${MPRAPI_RUST_SRCS}
            ${MPRAPI_RUST_WORKSPACE_DIR}/mprapi-rust/Cargo.toml
    WORKING_DIRECTORY ${MPRAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building mprapi-rust (${MPRAPI_RUST_TARGET})"
)
add_custom_target(rust_mprapi DEPENDS ${MPRAPI_RUST_LIB})

# The only compiled src *.c → mprapi-rust (src/mprapi/mprapi.rs), kept on disk per the
# never-delete rule.
#   mprapi.c  → mprapi-rust
# list(APPEND SOURCE
#     mprapi.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/mprapi_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/mprapi.def)

add_library(mprapi MODULE ${SOURCE})
set_module_type(mprapi win32dll)
# libmprapi_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(mprapi ${MPRAPI_RUST_LIB} wine)
add_dependencies(mprapi rust_mprapi)
add_importlibs(mprapi msvcrt kernel32 ntdll)
add_cd_file(TARGET mprapi DESTINATION reactos/system32 FOR all)
set_wine_module(mprapi)
