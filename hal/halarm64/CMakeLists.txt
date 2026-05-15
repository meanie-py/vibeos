
spec2def(hal.dll ../hal.spec ADD_IMPORTLIB)

add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    generic
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# ============================================================================
# Rust HAL static archive (sdk/lib/rust/hal-rust -> libhal_rust.a)
# ============================================================================

set(HAL_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(HAL_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-hal-target")
set(HAL_RUST_TARGET "aarch64-pc-windows-gnullvm")
set(HAL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libhal_rust.a")

add_custom_command(
    OUTPUT ${HAL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${HAL_RUST_TARGET_DIR}
        cargo build --release --target ${HAL_RUST_TARGET}
            -p hal-rust
            --manifest-path ${HAL_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${HAL_RUST_TARGET_DIR}/${HAL_RUST_TARGET}/release/libhal_rust.a
        ${HAL_RUST_LIB}
    DEPENDS ${HAL_RUST_WORKSPACE_DIR}/hal-rust/Cargo.toml
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/lib.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/arm64/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/arm64/halstubs.rs
    WORKING_DIRECTORY ${HAL_RUST_WORKSPACE_DIR}
    COMMENT "Building hal-rust (${HAL_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_hal_arm64 DEPENDS ${HAL_RUST_LIB})

list(APPEND SOURCES
    # generic/halstubs.c -- replaced by Rust (sdk/lib/rust/hal-rust/src/arm64/halstubs.rs)
    ${CMAKE_CURRENT_BINARY_DIR}/hal.def)

add_library(hal MODULE ${SOURCES})

set_module_type(hal kerneldll ENTRYPOINT HalInitSystem 8)
set_target_properties(hal PROPERTIES C_STANDARD 23 LINKER_LANGUAGE C)
target_link_libraries(hal ${HAL_RUST_LIB})
add_importlibs(hal ntoskrnl)
add_dependencies(hal psdk rust_hal_arm64)
add_cd_file(TARGET hal DESTINATION reactos/system32 FOR all)
