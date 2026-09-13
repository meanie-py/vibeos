
# ============================================================================
# msfs.sys — Rust cohabitation flip (msfs.c/create.c/finfo.c/fsctrl.c/msfssup.c/rw.c
#            -> msfs-rust)
# ============================================================================
# The Mailslot filesystem (DriverEntry + IRP dispatch + the IO_CSQ cancel-safe queue and
# timeout DPC) is translated to Rust (sdk/lib/rust/msfs-rust, kept on disk per the
# never-delete rule). DriverEntry + all routines come from libmsfs_rust.a. msfs has no GUID
# data TU, so a tiny rust_cohab_stub.c is the only C source the MODULE link needs. msfs is
# NOT boot-loaded (no 'x' in txtsetup.sif → not on the SetupLdr load path), so the bar is a
# clean cold link ×3 + no smoke regression. Driver-private node structs (MSFS_DEVICE_EXTENSION/
# FCB/CCB/MESSAGE/DPC_CTX) are defined in-crate (msfs-rust/src/types.rs); kernel/Io/Ke/Rtl
# types resolve to the single ntkrnl-sys ddk identity. Mirrors the mup-rust / fs_rec-rust cohab.
set(MSFS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(MSFS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-msfs-target" msfs-rust)
set(MSFS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmsfs_rust.a")
if(ARCH STREQUAL "i386")
    set(MSFS_RUST_TARGET "i686-pc-windows-gnu")
    set(MSFS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MSFS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MSFS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(MSFS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MSFS_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(MSFS_RUST_SRCS "${MSFS_RUST_WORKSPACE_DIR}" msfs-rust)
add_custom_command(
    OUTPUT ${MSFS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSFS_RUST_TARGET_DIR}
        RUSTFLAGS=${MSFS_RUST_RUSTFLAGS}
        cargo build --release --target ${MSFS_RUST_TARGET}
            -p msfs-rust
            --manifest-path ${MSFS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MSFS_RUST_TARGET_DIR}/${MSFS_RUST_TARGET}/release/libmsfs_rust.a
        ${MSFS_RUST_LIB}
    DEPENDS ${MSFS_RUST_SRCS}
            ${MSFS_RUST_WORKSPACE_DIR}/msfs-rust/Cargo.toml
    WORKING_DIRECTORY ${MSFS_RUST_WORKSPACE_DIR}
    COMMENT "Building msfs-rust (${MSFS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_msfs DEPENDS ${MSFS_RUST_LIB})

# All six .c bodies replaced by Rust (msfs-rust), kept on disk per the never-delete rule:
#   create.c finfo.c fsctrl.c msfs.c msfssup.c rw.c
# list(APPEND SOURCE create.c finfo.c fsctrl.c msfs.c msfssup.c rw.c msfs.h)

add_library(msfs MODULE rust_cohab_stub.c msfs.rc)
set_module_type(msfs kernelmodedriver)
add_importlibs(msfs ntoskrnl hal)
add_dependencies(msfs rust_msfs)
target_link_libraries(msfs ${MSFS_RUST_LIB})
add_cd_file(TARGET msfs DESTINATION reactos/system32/drivers FOR all)
add_registry_inf(msfs_reg.inf)
