
# ============================================================================
# npfs.sys — Rust cohabitation flip (the 20 .c bodies -> npfs-rust)
# ============================================================================
# The Named Pipe filesystem (DriverEntry + IRP dispatch + fast-I/O + the VCB/DCB/FCB/CCB node
# graph, NP_DATA_QUEUE engine, pipe state machine, wait queue, client security contexts) is
# translated to Rust (sdk/lib/rust/npfs-rust, kept on disk per the never-delete rule).
# DriverEntry + all routines come from libnpfs_rust.a. npfs has no GUID data TU, so a tiny
# rust_cohab_stub.c is the only C source the MODULE link needs. PSEH2 is dead under
# USE_DUMMY_PSEH → the Rust has no SEH and the MODULE no longer links ${PSEH_LIB}. npfs is NOT
# boot-loaded (no 'x' in txtsetup.sif), so the bar is a clean cold link ×3 + no smoke
# regression. Driver-private node structs (NP_VCB/DCB/FCB/CCB/...) are defined in-crate
# (npfs-rust/src/types.rs); kernel/Io/Ke/Ex/Se/FsRtl/Rtl types resolve to the single
# ntkrnl-sys ddk identity. Mirrors the mup-rust / fs_rec-rust / msfs-rust cohab.
set(NPFS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(NPFS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-npfs-target" npfs-rust)
set(NPFS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libnpfs_rust.a")
if(ARCH STREQUAL "i386")
    set(NPFS_RUST_TARGET "i686-pc-windows-gnu")
    set(NPFS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(NPFS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(NPFS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(NPFS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(NPFS_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(NPFS_RUST_SRCS "${NPFS_RUST_WORKSPACE_DIR}" npfs-rust)
add_custom_command(
    OUTPUT ${NPFS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NPFS_RUST_TARGET_DIR}
        RUSTFLAGS=${NPFS_RUST_RUSTFLAGS}
        cargo build --release --target ${NPFS_RUST_TARGET}
            -p npfs-rust
            --manifest-path ${NPFS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${NPFS_RUST_TARGET_DIR}/${NPFS_RUST_TARGET}/release/libnpfs_rust.a
        ${NPFS_RUST_LIB}
    DEPENDS ${NPFS_RUST_SRCS}
            ${NPFS_RUST_WORKSPACE_DIR}/npfs-rust/Cargo.toml
    WORKING_DIRECTORY ${NPFS_RUST_WORKSPACE_DIR}
    COMMENT "Building npfs-rust (${NPFS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_npfs DEPENDS ${NPFS_RUST_LIB})

# All 20 .c bodies replaced by Rust (npfs-rust), kept on disk per the never-delete rule:
#   cleanup.c close.c create.c datasup.c fileinfo.c fileobsup.c flushbuf.c fsctrl.c main.c
#   prefxsup.c read.c readsup.c secursup.c seinfo.c statesup.c strucsup.c volinfo.c waitsup.c
#   write.c writesup.c
# list(APPEND SOURCE cleanup.c close.c create.c datasup.c fileinfo.c fileobsup.c flushbuf.c
#     fsctrl.c main.c prefxsup.c read.c readsup.c secursup.c seinfo.c statesup.c strucsup.c
#     volinfo.c waitsup.c write.c writesup.c npfs.h)

add_library(npfs MODULE rust_cohab_stub.c npfs.rc)
set_module_type(npfs kernelmodedriver)
add_importlibs(npfs ntoskrnl hal)
add_dependencies(npfs rust_npfs)
target_link_libraries(npfs ${NPFS_RUST_LIB})
add_cd_file(TARGET npfs DESTINATION reactos/system32/drivers FOR all)
add_registry_inf(npfs_reg.inf)
