
# ============================================================================
# fs_rec.sys — Rust cohabitation flip (all 11 .c -> fs_rec-rust)
# ============================================================================
# The filesystem recognizer (fs_rec.c DriverEntry + FsRecRegisterFs/FsRecLoadFileSystem
# + dispatch; blockdev.c device-I/O helpers; one sniffer per FS type:
# fat/fatx/cdfs/ntfs/udfs/ext/btrfs/reiserfs/ffs.c) is translated to Rust
# (sdk/lib/rust/fs_rec-rust, kept on disk per the never-delete rule). DriverEntry + all
# routines come from libfs_rec_rust.a. fs_rec has no GUID data TU, so a tiny
# rust_cohab_stub.c is the only C source the MODULE link needs. fs_rec IS boot-loaded
# (Group "Boot File System", registers via IoRegisterFileSystem), so the bar is a clean
# smoke boot ×3. Driver-private structs (PACKED_BOOT_SECTOR/BIOS_PARAMETER_BLOCK/
# DEVICE_EXTENSION + per-FS superblocks) come from the gen_driver_types.sh bindgen
# substrate (fs_rec-rust/src/types.rs). Mirrors the partmgr-rust / swenum-rust cohab.
set(FS_REC_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(FS_REC_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-fs_rec-target" fs_rec-rust)
set(FS_REC_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfs_rec_rust.a")
if(ARCH STREQUAL "i386")
    set(FS_REC_RUST_TARGET "i686-pc-windows-gnu")
    set(FS_REC_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(FS_REC_RUST_TARGET "x86_64-pc-windows-gnu")
    set(FS_REC_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(FS_REC_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(FS_REC_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(FS_REC_RUST_SRCS "${FS_REC_RUST_WORKSPACE_DIR}" fs_rec-rust)
add_custom_command(
    OUTPUT ${FS_REC_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FS_REC_RUST_TARGET_DIR}
        RUSTFLAGS=${FS_REC_RUST_RUSTFLAGS}
        cargo build --release --target ${FS_REC_RUST_TARGET}
            -p fs_rec-rust
            --manifest-path ${FS_REC_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FS_REC_RUST_TARGET_DIR}/${FS_REC_RUST_TARGET}/release/libfs_rec_rust.a
        ${FS_REC_RUST_LIB}
    DEPENDS ${FS_REC_RUST_SRCS}
            ${FS_REC_RUST_WORKSPACE_DIR}/fs_rec-rust/Cargo.toml
    WORKING_DIRECTORY ${FS_REC_RUST_WORKSPACE_DIR}
    COMMENT "Building fs_rec-rust (${FS_REC_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_fs_rec DEPENDS ${FS_REC_RUST_LIB})

# All 11 .c bodies replaced by Rust (fs_rec-rust), kept on disk per the never-delete rule:
#   blockdev.c fs_rec.c fat.c fatx.c cdfs.c ntfs.c udfs.c ext.c btrfs.c reiserfs.c ffs.c
# list(APPEND SOURCE blockdev.c btrfs.c cdfs.c ext.c fat.c fatx.c ffs.c fs_rec.c ntfs.c reiserfs.c udfs.c fs_rec.h)

add_library(fs_rec MODULE rust_cohab_stub.c fs_rec.rc)
set_module_type(fs_rec kernelmodedriver)
add_importlibs(fs_rec ntoskrnl hal)
add_dependencies(fs_rec rust_fs_rec)
target_link_libraries(fs_rec ${FS_REC_RUST_LIB})
add_cd_file(TARGET fs_rec DESTINATION reactos/system32/drivers FOR all)
