
# ============================================================================
# Rust ARM64 virtio-input driver
# (sdk/lib/rust/arm64input-rust -> libarm64input_rust.a)
# ============================================================================
# arm64input.c is translated to Rust. Its #[no_mangle] DriverEntry (and all
# internal functions) replace the C arm64input.c (commented out of the
# add_library SOURCES below; the .c is kept on disk per the
# never-delete-a-translation rule). Mirrors the arm64 hal-rust cohabitation
# (hal/halarm64/CMakeLists.txt): one staticlib linked into the cmake-built
# kernelmodedriver. arm64-only (the driver bypasses ACPI/PnP/GIC by nature).

set(ARM64INPUT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(ARM64INPUT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-arm64input-target")
set(ARM64INPUT_RUST_TARGET "aarch64-pc-windows-gnullvm")
set(ARM64INPUT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libarm64input_rust.a")

add_custom_command(
    OUTPUT ${ARM64INPUT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ARM64INPUT_RUST_TARGET_DIR}
        cargo build --release --target ${ARM64INPUT_RUST_TARGET}
            -p arm64input-rust
            --manifest-path ${ARM64INPUT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ARM64INPUT_RUST_TARGET_DIR}/${ARM64INPUT_RUST_TARGET}/release/libarm64input_rust.a
        ${ARM64INPUT_RUST_LIB}
    DEPENDS ${ARM64INPUT_RUST_WORKSPACE_DIR}/arm64input-rust/Cargo.toml
            ${ARM64INPUT_RUST_WORKSPACE_DIR}/arm64input-rust/src/lib.rs
            ${ARM64INPUT_RUST_WORKSPACE_DIR}/arm64input-rust/src/arm64input.rs
    WORKING_DIRECTORY ${ARM64INPUT_RUST_WORKSPACE_DIR}
    COMMENT "Building arm64input-rust (${ARM64INPUT_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_arm64input DEPENDS ${ARM64INPUT_RUST_LIB})

# arm64input.c -- replaced by Rust (sdk/lib/rust/arm64input-rust/src/arm64input.rs)
add_library(arm64input MODULE rust_cohab_stub.c)
set_module_type(arm64input kernelmodedriver)
set_target_properties(arm64input PROPERTIES LINKER_LANGUAGE C)
target_link_libraries(arm64input ${ARM64INPUT_RUST_LIB})
add_importlibs(arm64input ntoskrnl hal)
add_dependencies(arm64input rust_arm64input)
