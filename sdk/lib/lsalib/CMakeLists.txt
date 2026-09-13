
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# lsalib (user-mode variant, consumed by secur32) stays C — kept on disk.
add_library(lsalib lsa.c)
add_dependencies(lsalib xdk)

# ============================================================================
# ntlsalib (the kernel _NTSYSTEM_ variant, linked into ntoskrnl) — Rust cohab
# ============================================================================
# lsa.c is translated to Rust (lsalib-rust, kept on disk per the never-delete
# rule). liblsalib_rust.a replaces the C object for the kernel variant; the
# `ntlsalib` static lib target is preserved (ntoskrnl links it by name) as a stub
# TU that PUBLIC-links the Rust archive (usbdex-style cohab). lsalib-rust depends
# on ntkrnl-sys + lsass-sys (the LSA LPC message substrate). Built for all 3 arches.
set(NTLSALIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(NTLSALIB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-lsalib-target" lsalib-rust)
set(NTLSALIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/liblsalib_rust.a")
if(ARCH STREQUAL "i386")
    set(NTLSALIB_RUST_TARGET "i686-pc-windows-gnu")
    set(NTLSALIB_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(NTLSALIB_RUST_TARGET "x86_64-pc-windows-gnu")
    set(NTLSALIB_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(NTLSALIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(NTLSALIB_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(LSALIB_RUST_SRCS "${NTLSALIB_RUST_WORKSPACE_DIR}" lsalib-rust)
add_custom_command(
    OUTPUT ${NTLSALIB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NTLSALIB_RUST_TARGET_DIR}
        RUSTFLAGS=${NTLSALIB_RUST_RUSTFLAGS}
        cargo build --release --target ${NTLSALIB_RUST_TARGET}
            -p lsalib-rust
            --manifest-path ${NTLSALIB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${NTLSALIB_RUST_TARGET_DIR}/${NTLSALIB_RUST_TARGET}/release/liblsalib_rust.a
        ${NTLSALIB_RUST_LIB}
    DEPENDS ${LSALIB_RUST_SRCS}
            ${NTLSALIB_RUST_WORKSPACE_DIR}/lsalib-rust/Cargo.toml
    WORKING_DIRECTORY ${NTLSALIB_RUST_WORKSPACE_DIR}
    COMMENT "Building lsalib-rust (${NTLSALIB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_ntlsalib DEPENDS ${NTLSALIB_RUST_LIB})

# add_library(ntlsalib lsa.c) + _NTSYSTEM_  # body replaced by Rust; kept on disk
add_library(ntlsalib rust_cohab_stub.c)
add_dependencies(ntlsalib xdk rust_ntlsalib)
target_link_libraries(ntlsalib PUBLIC ${NTLSALIB_RUST_LIB})
