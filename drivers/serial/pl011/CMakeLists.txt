# ============================================================================
# pl011.sys — Rust cohabitation flip (pl011.c -> sdk/lib/rust/pl011-rust)
# ============================================================================
# pl011.c (the minimal ARM64 PL011 UART driver) is translated to Rust
# (pl011-rust, kept on disk per the never-delete rule). DriverEntry + the
# Pl011* dispatch routines come from libpl011_rust.a. cmake keeps the MODULE
# link (set_module_type kernelmodedriver + add_importlibs(ntoskrnl hal)).
# pl011-rust depends on ntkrnl-sys; the custom_command mirrors the swenum cohab
# (standard per-arch RUST_TARGET). pl011 is arm64-only (see ../CMakeLists.txt),
# so only the aarch64 branch is exercised, but the per-arch set is kept for
# parity with the other driver cohabs.
set(PL011_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PL011_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pl011-target")
set(PL011_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpl011_rust.a")
if(ARCH STREQUAL "i386")
    set(PL011_RUST_TARGET "i686-pc-windows-gnu")
    set(PL011_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PL011_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PL011_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PL011_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PL011_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(PL011_RUST_SRCS "${PL011_RUST_WORKSPACE_DIR}" pl011-rust)
add_custom_command(
    OUTPUT ${PL011_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PL011_RUST_TARGET_DIR}
        RUSTFLAGS=${PL011_RUST_RUSTFLAGS}
        cargo build --release --target ${PL011_RUST_TARGET}
            -p pl011-rust
            --manifest-path ${PL011_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PL011_RUST_TARGET_DIR}/${PL011_RUST_TARGET}/release/libpl011_rust.a
        ${PL011_RUST_LIB}
    DEPENDS ${PL011_RUST_SRCS}
            ${PL011_RUST_WORKSPACE_DIR}/pl011-rust/Cargo.toml
    WORKING_DIRECTORY ${PL011_RUST_WORKSPACE_DIR}
    COMMENT "Building pl011-rust (${PL011_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_pl011 DEPENDS ${PL011_RUST_LIB})

# add_library(pl011 MODULE pl011.c)  # body replaced by Rust; kept on disk
add_library(pl011 MODULE rust_cohab_stub.c)
set_module_type(pl011 kernelmodedriver)
add_importlibs(pl011 ntoskrnl hal)
add_dependencies(pl011 rust_pl011)
target_link_libraries(pl011 ${PL011_RUST_LIB})
add_cd_file(TARGET pl011 DESTINATION reactos/system32/drivers NO_CAB FOR all)
