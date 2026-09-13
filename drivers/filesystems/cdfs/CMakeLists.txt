
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

# ============================================================================
# cdfs.sys — Rust cohabitation flip (27 of 29 .c -> cdfs-rust)
# ============================================================================
# The ISO9660 CD filesystem is translated to Rust (sdk/lib/rust/cdfs-rust, kept on
# disk per the never-delete rule). DriverEntry + all dispatch/support routines come
# from libcdfs_rust.a. cdfs IS boot-loaded (cdfs.sys=1 in txtsetup; the LiveCD
# ISO9660 read-stack leg ISO->cdrom->classpnp->cdfs), so the bar is a clean smoke
# boot x3. Driver-private structs (FCB/VCB/CCB/IRP_CONTEXT + the ISO9660 on-disk raw
# structs from cd.h + the CD-ROM DDK types from ntddcdrm.h) come from the
# gen_driver_types.sh bindgen substrate (cdfs-rust/src/types.rs). Mirrors fs_rec-rust.
#
# Two .c stay C (never translated -> no .rs counterpart): cdprocssrc.c (forces
# out-of-line emission of the cdprocs.h FORCEINLINEs) and filobsup.c (file-object
# decode/set helpers the Rust calls via extern).
set(CDFS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(CDFS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-cdfs-target" cdfs-rust)
set(CDFS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcdfs_rust.a")
if(ARCH STREQUAL "i386")
    set(CDFS_RUST_TARGET "i686-pc-windows-gnu")
    set(CDFS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(CDFS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CDFS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(CDFS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CDFS_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(CDFS_RUST_SRCS "${CDFS_RUST_WORKSPACE_DIR}" cdfs-rust)
add_custom_command(
    OUTPUT ${CDFS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CDFS_RUST_TARGET_DIR}
        RUSTFLAGS=${CDFS_RUST_RUSTFLAGS}
        cargo build --release --target ${CDFS_RUST_TARGET}
            -p cdfs-rust
            --manifest-path ${CDFS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CDFS_RUST_TARGET_DIR}/${CDFS_RUST_TARGET}/release/libcdfs_rust.a
        ${CDFS_RUST_LIB}
    DEPENDS ${CDFS_RUST_SRCS}
            ${CDFS_RUST_WORKSPACE_DIR}/cdfs-rust/Cargo.toml
    WORKING_DIRECTORY ${CDFS_RUST_WORKSPACE_DIR}
    COMMENT "Building cdfs-rust (${CDFS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_cdfs DEPENDS ${CDFS_RUST_LIB})

# 27 .c bodies replaced by Rust (cdfs-rust), kept on disk per the never-delete rule:
#   allocsup cachesup cddata cdinit cleanup close create devctrl deviosup dirctrl
#   dirsup fileinfo fsctrl fspdisp lockctrl namesup pathsup pnp prefxsup read
#   resrcsup shutdown strucsup verfysup volinfo workque write
# list(APPEND SOURCE allocsup.c cachesup.c cddata.c cdinit.c cleanup.c close.c
#   create.c devctrl.c deviosup.c dirctrl.c dirsup.c fileinfo.c fsctrl.c fspdisp.c
#   lockctrl.c namesup.c pathsup.c prefxsup.c read.c resrcsup.c shutdown.c strucsup.c
#   verfysup.c volinfo.c workque.c write.c)
list(APPEND SOURCE
    cdprocssrc.c
    filobsup.c)

add_library(cdfs MODULE ${SOURCE} cdfs.rc)
set_module_type(cdfs kernelmodedriver)
target_link_libraries(cdfs ${PSEH_LIB} memcmp)
add_importlibs(cdfs ntoskrnl hal)
add_dependencies(cdfs rust_cdfs)
target_link_libraries(cdfs ${CDFS_RUST_LIB})

add_cd_file(TARGET cdfs DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(cdfs_reg.inf)
