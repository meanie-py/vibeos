
spec2def(ks.sys ks.spec ADD_IMPORTLIB)

add_definitions(
    -DBUILDING_KS
    -D_COMDDK_)

# ============================================================================
# ks.sys — Rust cohabitation flip (all 22 .c bodies -> ks-rust)
# ============================================================================
# The Kernel Streaming class library (22 .c, ~19K LOC) is translated to Rust
# (sdk/lib/rust/ks-rust, libks_rust.a; DeepSeek-chat V3 + c2rust-anchored, constant-safeguard
# verified). The .c files are kept on disk per the never-delete rule (listed below, commented).
# ks is an EXPORT driver: spec2def(ks.spec) is preserved and the #[no_mangle] Ks*/Ko*/
# CBaseUnknown* exports + DriverEntry in the rlib satisfy ks.def and take over the link.
# NOT boot-loaded in the standard smoke (the KS/AVStream stack isn't exercised) -> link-clean x3
# is the verification bar. BUILDING_KS / _COMDDK_ preserved.
#
# C bodies replaced by Rust (kept on disk, never deleted):
#   api.c allocators.c bag.c device.c deviceinterface.c driver.c clocks.c connectivity.c
#   event.c filter.c filterfactory.c image.c irp.c methods.c misc.c pin.c property.c
#   topology.c unknown.c worker.c kcom.c swenum.c
set(KS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(KS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ks-target" ks-rust)
set(KS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libks_rust.a")
if(ARCH STREQUAL "i386")
    set(KS_RUST_TARGET "i686-pc-windows-gnu")
    set(KS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(KS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(KS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(KS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(KS_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(KS_RUST_SRCS "${KS_RUST_WORKSPACE_DIR}" ks-rust)
add_custom_command(
    OUTPUT ${KS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KS_RUST_TARGET_DIR}
        RUSTFLAGS=${KS_RUST_RUSTFLAGS}
        cargo build --release --target ${KS_RUST_TARGET}
            -p ks-rust
            --manifest-path ${KS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KS_RUST_TARGET_DIR}/${KS_RUST_TARGET}/release/libks_rust.a
        ${KS_RUST_LIB}
    DEPENDS ${KS_RUST_SRCS} ${KS_RUST_WORKSPACE_DIR}/ks-rust/Cargo.toml
    WORKING_DIRECTORY ${KS_RUST_WORKSPACE_DIR}
    COMMENT "Building ks-rust (${KS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_ks DEPENDS ${KS_RUST_LIB})

# Original C source list (replaced by Rust above; kept for the never-delete record):
# list(APPEND SOURCE
#     api.c allocators.c bag.c device.c deviceinterface.c driver.c clocks.c connectivity.c
#     event.c filter.c filterfactory.c image.c irp.c methods.c misc.c pin.c property.c
#     topology.c unknown.c worker.c kcom.c swenum.c precomp.h)

add_library(ks MODULE
    rust_cohab_stub.c
    ${ks_asm}
    ks.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ks.def)

set_module_type(ks kernelmodedriver ENTRYPOINT 0)

target_link_libraries(ks ${PSEH_LIB})
add_importlibs(ks ntoskrnl hal)
add_dependencies(ks psdk rust_ks)
target_link_libraries(ks ${KS_RUST_LIB})
add_cd_file(TARGET ks DESTINATION reactos/system32/drivers FOR all)
