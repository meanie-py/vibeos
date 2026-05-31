# ============================================================================
# Rust framebuffer video driver
# (sdk/lib/rust/arm64vid-rust -> libarm64vid_rust.a)
# ============================================================================
# arm64vid.c is translated to Rust. Its #[no_mangle] DriverEntry (and all
# internal functions) replace the C arm64vid.c (commented out of the
# add_library SOURCES below; the .c is kept on disk per the
# never-delete-a-translation rule). Mirrors the arm64input-rust cohabitation
# (drivers/input/arm64input/CMakeLists.txt): one staticlib linked into the
# cmake-built kernelmodedriver.
#
# Unlike arm64input (arm64-only), this driver is the live GOP/linear
# framebuffer video device on ALL THREE UEFI arches, so the Rust target is
# selected per-arch (mirroring hal/halx86/CMakeLists.txt).

set(ARM64VID_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(ARM64VID_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-arm64vid-target")
if(ARCH STREQUAL "i386")
    set(ARM64VID_RUST_TARGET "i686-pc-windows-gnu")
    # i386: disable SSE — same constraint as kernel-rust / hal-rust (no SSE
    # in early kernel-mode paths).
    set(ARM64VID_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(ARM64VID_RUST_TARGET "x86_64-pc-windows-gnu")
    set(ARM64VID_RUST_EXTRA_ENV "")
else()
    set(ARM64VID_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(ARM64VID_RUST_EXTRA_ENV "")
endif()
set(ARM64VID_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libarm64vid_rust.a")

add_custom_command(
    OUTPUT ${ARM64VID_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ARM64VID_RUST_TARGET_DIR} ${ARM64VID_RUST_EXTRA_ENV}
        cargo build --release --target ${ARM64VID_RUST_TARGET}
            -p arm64vid-rust
            --manifest-path ${ARM64VID_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ARM64VID_RUST_TARGET_DIR}/${ARM64VID_RUST_TARGET}/release/libarm64vid_rust.a
        ${ARM64VID_RUST_LIB}
    DEPENDS ${ARM64VID_RUST_WORKSPACE_DIR}/arm64vid-rust/Cargo.toml
            ${ARM64VID_RUST_WORKSPACE_DIR}/arm64vid-rust/src/lib.rs
            ${ARM64VID_RUST_WORKSPACE_DIR}/arm64vid-rust/src/arm64vid.rs
    WORKING_DIRECTORY ${ARM64VID_RUST_WORKSPACE_DIR}
    COMMENT "Building arm64vid-rust (${ARM64VID_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_arm64vid DEPENDS ${ARM64VID_RUST_LIB})

# arm64vid.c -- replaced by Rust (sdk/lib/rust/arm64vid-rust/src/arm64vid.rs)
add_library(arm64vid MODULE rust_cohab_stub.c)
set_module_type(arm64vid kernelmodedriver)
set_target_properties(arm64vid PROPERTIES LINKER_LANGUAGE C)
target_link_libraries(arm64vid ${ARM64VID_RUST_LIB})
add_importlibs(arm64vid ntoskrnl hal)
add_dependencies(arm64vid rust_arm64vid)
add_cd_file(TARGET arm64vid DESTINATION reactos/system32/drivers FOR all)
