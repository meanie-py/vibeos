# ============================================================================
# processr.sys — Rust cohabitation flip (processr.c/misc.c/pnp.c -> processr-rust)
# ============================================================================
# The generic CPU driver (processr.c DriverEntry + misc.c ForwardIrpAndForget +
# pnp.c ProcessorPnp/ProcessorAddDevice) is translated to Rust (processr-rust,
# kept on disk per the never-delete rule). DriverEntry + all routines come from
# libprocessr_rust.a. cmake keeps the MODULE link (set_module_type
# kernelmodedriver + add_importlibs(hal ntoskrnl) + .rc + cpu.inf). processr-rust
# depends on ntkrnl-sys; the custom_command mirrors the swenum cohab.
set(PROCESSR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PROCESSR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-processr-target")
set(PROCESSR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libprocessr_rust.a")
if(ARCH STREQUAL "i386")
    set(PROCESSR_RUST_TARGET "i686-pc-windows-gnu")
    set(PROCESSR_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PROCESSR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PROCESSR_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PROCESSR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PROCESSR_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${PROCESSR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PROCESSR_RUST_TARGET_DIR}
        RUSTFLAGS=${PROCESSR_RUST_RUSTFLAGS}
        cargo build --release --target ${PROCESSR_RUST_TARGET}
            -p processr-rust
            --manifest-path ${PROCESSR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PROCESSR_RUST_TARGET_DIR}/${PROCESSR_RUST_TARGET}/release/libprocessr_rust.a
        ${PROCESSR_RUST_LIB}
    DEPENDS ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/Cargo.toml
            ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/src/lib.rs
            ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/src/shared.rs
            ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/src/processr.rs
            ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/src/misc.rs
            ${PROCESSR_RUST_WORKSPACE_DIR}/processr-rust/src/pnp.rs
    WORKING_DIRECTORY ${PROCESSR_RUST_WORKSPACE_DIR}
    COMMENT "Building processr-rust (${PROCESSR_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_processr DEPENDS ${PROCESSR_RUST_LIB})

# add_library(processr MODULE processr.c misc.c pnp.c ...)  # body replaced by Rust; kept on disk
add_library(processr MODULE rust_cohab_stub.c processr.rc)
set_module_type(processr kernelmodedriver)
add_importlibs(processr hal ntoskrnl)
add_dependencies(processr rust_processr)
target_link_libraries(processr ${PROCESSR_RUST_LIB})
add_cd_file(TARGET processr DESTINATION reactos/system32/drivers FOR all)
add_driver_inf(processr cpu.inf)
