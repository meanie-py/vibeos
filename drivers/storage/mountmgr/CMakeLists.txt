
# ============================================================================
# mountmgr.sys — Rust cohabitation flip (all 7 C sources -> sdk/lib/rust/mountmgr-rust)
# ============================================================================
# Every mountmgr C source (database/device/mountmgr/notify/point/symlink/uniqueid)
# is translated to Rust (mountmgr-rust, kept on disk per the never-delete rule).
# The PE DriverEntry comes from libmountmgr_rust.a; cmake keeps the MODULE link
# (set_module_type kernelmodedriver + add_importlibs(ntoskrnl hal)). mountmgr
# exports nothing but DriverEntry (no .spec) — DriverEntry-only cohab, like disk.
# Boot-start WDM driver. Mirrors disk-rust.
set(MOUNTMGR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(MOUNTMGR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mountmgr-target")
set(MOUNTMGR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmountmgr_rust.a")
if(ARCH STREQUAL "i386")
    set(MOUNTMGR_RUST_TARGET "i686-pc-windows-gnu")
    set(MOUNTMGR_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MOUNTMGR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MOUNTMGR_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(MOUNTMGR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MOUNTMGR_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(MOUNTMGR_RUST_SRCS "${MOUNTMGR_RUST_WORKSPACE_DIR}" mountmgr-rust)
add_custom_command(
    OUTPUT ${MOUNTMGR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MOUNTMGR_RUST_TARGET_DIR}
        RUSTFLAGS=${MOUNTMGR_RUST_RUSTFLAGS}
        cargo build --release --target ${MOUNTMGR_RUST_TARGET}
            -p mountmgr-rust
            --manifest-path ${MOUNTMGR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MOUNTMGR_RUST_TARGET_DIR}/${MOUNTMGR_RUST_TARGET}/release/libmountmgr_rust.a
        ${MOUNTMGR_RUST_LIB}
    DEPENDS ${MOUNTMGR_RUST_SRCS}
            ${MOUNTMGR_RUST_WORKSPACE_DIR}/mountmgr-rust/Cargo.toml
    WORKING_DIRECTORY ${MOUNTMGR_RUST_WORKSPACE_DIR}
    COMMENT "Building mountmgr-rust (${MOUNTMGR_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_mountmgr DEPENDS ${MOUNTMGR_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c
    mntmgr.h)
#   database.c device.c mountmgr.c notify.c point.c symlink.c uniqueid.c  # all replaced by Rust (mountmgr-rust); kept on disk

add_library(mountmgr MODULE ${SOURCE} mountmgr.rc)
add_dependencies(mountmgr rust_mountmgr)
target_link_libraries(mountmgr ${MOUNTMGR_RUST_LIB} wdmguid ioevent memcmp)
set_module_type(mountmgr kernelmodedriver)
add_importlibs(mountmgr ntoskrnl hal)
add_cd_file(TARGET mountmgr DESTINATION reactos/system32/drivers NO_CAB FOR all)
