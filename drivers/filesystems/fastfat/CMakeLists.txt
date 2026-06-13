
# ============================================================================
# fastfat.sys — Rust cohabitation flip (all 34 .c -> fastfat-rust)
# ============================================================================
# The FAT filesystem driver (fatinit.c DriverEntry, FSD dispatch + FSP queue,
# create/close/cleanup/read/write, allocsup/dirsup/easup/strucsup/cachesup/
# deviosup support layers, fsctrl mount/verify, fileinfo/volinfo, lockctrl,
# pnp/shutdown, splay/name/time/verify helpers) is translated to Rust
# (sdk/lib/rust/fastfat-rust, kept on disk per the never-delete rule).
# DriverEntry + all routines come from libfastfat_rust.a. fastfat has no GUID
# data TU, so a tiny rust_cohab_stub.c is the only C source the MODULE link
# needs. fastfat IS the live system volume FS on the arm64 smoke (FAT32
# ramdisk), so the bar there is a clean boot; x86 boots from cdfs LiveCD so
# the bar is a clean link + boot. Driver-private structs (VCB/FCB/CCB/
# IRP_CONTEXT/PACKED_DIRENT/... 53 structs) come from the gen_driver_types.sh
# bindgen substrate (fastfat-rust/src/types_<arch>.rs, 0x502 version profile).
# Mirrors the fs_rec-rust / partmgr-rust cohab.
set(FASTFAT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(FASTFAT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-fastfat-target")
set(FASTFAT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfastfat_rust.a")
if(ARCH STREQUAL "i386")
    set(FASTFAT_RUST_TARGET "i686-pc-windows-gnu")
    set(FASTFAT_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(FASTFAT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(FASTFAT_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(FASTFAT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(FASTFAT_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE FASTFAT_RUST_SRCS ${FASTFAT_RUST_WORKSPACE_DIR}/fastfat-rust/src/*.rs)
add_custom_command(
    OUTPUT ${FASTFAT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FASTFAT_RUST_TARGET_DIR}
        RUSTFLAGS=${FASTFAT_RUST_RUSTFLAGS}
        RUSTC_BOOTSTRAP=1
        cargo build --release --target ${FASTFAT_RUST_TARGET}
            -p fastfat-rust
            --manifest-path ${FASTFAT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FASTFAT_RUST_TARGET_DIR}/${FASTFAT_RUST_TARGET}/release/libfastfat_rust.a
        ${FASTFAT_RUST_LIB}
    DEPENDS ${FASTFAT_RUST_SRCS}
            ${FASTFAT_RUST_WORKSPACE_DIR}/fastfat-rust/Cargo.toml
    WORKING_DIRECTORY ${FASTFAT_RUST_WORKSPACE_DIR}
    COMMENT "Building fastfat-rust (${FASTFAT_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_fastfat DEPENDS ${FASTFAT_RUST_LIB})

# All 34 .c bodies replaced by Rust (fastfat-rust), kept on disk per the
# never-delete rule:
# list(APPEND SOURCE
#     acchksup.c allocsup.c cachesup.c cleanup.c close.c create.c devctrl.c
#     deviosup.c dirctrl.c dirsup.c dumpsup.c ea.c easup.c fatdata.c fatinit.c
#     fatprocssrc.c fileinfo.c filobsup.c flush.c fsctrl.c fspdisp.c lockctrl.c
#     namesup.c pnp.c read.c resrcsup.c shutdown.c splaysup.c strucsup.c
#     timesup.c verfysup.c volinfo.c workque.c write.c fatprocs.h)

if(ARCH STREQUAL "arm64")
    add_definitions(-DARM64_UART_TRACE_ENABLE)
endif()
add_library(fastfat MODULE rust_cohab_stub.c rust_cohab_forceinline.c rust_cohab_fat.c fastfat.rc)
# i386 NTAPI = stdcall. The Rust crate is uniformly `extern "system"` (stdcall on
# i386), so every cohab-provided helper must emit the `@N` stdcall decoration to
# match. -mrtd makes clang default all non-variadic C functions to stdcall while
# leaving variadics (DebugTrace/DbgPrint/KdPrintEx) cdecl — exactly mirroring
# Rust's `extern "system"` variadic→cdecl fallback. No-op on amd64/arm64 (no
# decoration), so the working amd64 link is unaffected.
if(ARCH STREQUAL "i386")
    set_source_files_properties(
        rust_cohab_stub.c rust_cohab_forceinline.c rust_cohab_fat.c
        PROPERTIES COMPILE_OPTIONS "-mrtd")
endif()
set_module_type(fastfat kernelmodedriver)
add_importlibs(fastfat ntoskrnl hal)
add_dependencies(fastfat rust_fastfat)
target_link_libraries(fastfat ${FASTFAT_RUST_LIB})
add_cd_file(TARGET fastfat DESTINATION reactos/system32/drivers NO_CAB FOR all)
