
include_directories(
    BEFORE include
    ../tdihelpers/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

# ============================================================================
# afd.sys — Rust cohabitation flip (10 of 11 afd/*.c -> afd-rust)
# ============================================================================
# The Ancillary Function Driver (usermode Winsock -> TDI bridge) is translated to
# Rust (sdk/lib/rust/afd-rust, kept on disk per the never-delete rule). DriverEntry
# + all dispatch/support routines come from libafd_rust.a. afd is NOT boot-loaded
# (Start=auto), so the bar is cohab link-clean x3 + a winsock round-trip, not a boot
# marker. Driver-private structs (AFD_FCB/ENDPOINT/DEVICE_EXTENSION + the TDI/shared
# IOCTL structs) come from the gen_driver_types.sh bindgen substrate
# (afd-rust/src/types.rs). The TDI helpers come from the shared `tdihelpers` library
# (../tdihelpers), which netio.sys links too, and resolve as extern "C". event.c is NOT in this build (dead in upstream: its
# AfdRegisterEventHandlers is called nowhere) — event.rs is kept but not a module.
set(AFD_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(AFD_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-afd-target" afd-rust)
set(AFD_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libafd_rust.a")
if(ARCH STREQUAL "i386")
    set(AFD_RUST_TARGET "i686-pc-windows-gnu")
    set(AFD_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(AFD_RUST_TARGET "x86_64-pc-windows-gnu")
    set(AFD_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(AFD_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(AFD_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(AFD_RUST_SRCS "${AFD_RUST_WORKSPACE_DIR}" afd-rust)
add_custom_command(
    OUTPUT ${AFD_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${AFD_RUST_TARGET_DIR}
        RUSTFLAGS=${AFD_RUST_RUSTFLAGS}
        cargo build --release --target ${AFD_RUST_TARGET}
            -p afd-rust
            --manifest-path ${AFD_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${AFD_RUST_TARGET_DIR}/${AFD_RUST_TARGET}/release/libafd_rust.a
        ${AFD_RUST_LIB}
    DEPENDS ${AFD_RUST_SRCS}
            ${AFD_RUST_WORKSPACE_DIR}/afd-rust/Cargo.toml
    WORKING_DIRECTORY ${AFD_RUST_WORKSPACE_DIR}
    COMMENT "Building afd-rust (${AFD_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_afd DEPENDS ${AFD_RUST_LIB})

# 10 afd/*.c bodies replaced by Rust (afd-rust), kept on disk per the never-delete rule:
#   bind connect context info listen lock main read select write
# list(APPEND SOURCE afd/bind.c afd/connect.c afd/context.c afd/info.c afd/listen.c
#   afd/lock.c afd/main.c afd/read.c afd/select.c afd/write.c)
list(APPEND SOURCE
    include/afd.h)

add_library(afd MODULE ${SOURCE} afd.rc)
target_link_libraries(afd tdihelpers ${PSEH_LIB} memcmp)
set_module_type(afd kernelmodedriver)
add_importlibs(afd ntoskrnl hal)
add_pch(afd include/afd.h SOURCE)
add_dependencies(afd rust_afd)
target_link_libraries(afd ${AFD_RUST_LIB})
add_cd_file(TARGET afd DESTINATION reactos/system32/drivers FOR all)
add_registry_inf(afd_reg.inf)
