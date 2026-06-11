
remove_definitions(-D_WIN32_WINNT=0x502)

# ============================================================================
# cdrom.sys — Rust cohabitation flip (11 of 13 .c -> cdrom-rust)
# ============================================================================
# The CD-ROM KMDF class driver is translated to Rust (sdk/lib/rust/cdrom-rust,
# kept on disk per the never-delete rule). 11 .c bodies (aacs/autorun/cdrom/common/
# init/ioctl/mmc/pnppower/scratch/sense/zpodd) are replaced by libcdrom_rust.a;
# data.c (the CDROM data tables the Rust modules reference) + guid.c (GUID data
# instantiation) stay C. cdrom IS boot-loaded on the LiveCD (txtsetup cdrom.sys=1,
# CriticalDeviceDatabase GenCdRom), so a finished cdrom is smoke-exercised on
# i386/amd64; arm64 boots a ramdisk so its bar is a clean link.
#
# cdrom is the fleet's only KMDF driver: every Wdf* call in the Rust dispatches
# through the kmdf-sys substrate (WdfFunctions[] table), and the C wdfdriverentry
# stub pulled in by `kmdfdriver` (FxDriverEntry) binds the framework via
# WdfVersionBind before invoking this crate's #[no_mangle] DriverEntry.
#
# RUSTC_BOOTSTRAP=1 + i386 softfloat RUSTFLAGS mirror the pcix/pci-rust cohab so the
# standard-target staticlib build is self-sufficient (kernel mode = no SSE).
set(CDROM_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(CDROM_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-cdrom-target")
set(CDROM_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcdrom_rust.a")
if(ARCH STREQUAL "i386")
    set(CDROM_RUST_TARGET "i686-pc-windows-gnu")
    set(CDROM_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(CDROM_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CDROM_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(CDROM_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CDROM_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE CDROM_RUST_SRCS ${CDROM_RUST_WORKSPACE_DIR}/cdrom-rust/src/*.rs)
add_custom_command(
    OUTPUT ${CDROM_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CDROM_RUST_TARGET_DIR}
        RUSTFLAGS=${CDROM_RUST_RUSTFLAGS}
        RUSTC_BOOTSTRAP=1
        cargo build --release --target ${CDROM_RUST_TARGET}
            -p cdrom-rust
            --manifest-path ${CDROM_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CDROM_RUST_TARGET_DIR}/${CDROM_RUST_TARGET}/release/libcdrom_rust.a
        ${CDROM_RUST_LIB}
    DEPENDS ${CDROM_RUST_SRCS}
            ${CDROM_RUST_WORKSPACE_DIR}/cdrom-rust/Cargo.toml
    WORKING_DIRECTORY ${CDROM_RUST_WORKSPACE_DIR}
    COMMENT "Building cdrom-rust (${CDROM_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_cdrom DEPENDS ${CDROM_RUST_LIB})

# 11 .c bodies replaced by Rust (cdrom-rust), kept on disk per the never-delete rule:
#   aacs.c autorun.c cdrom.c common.c init.c ioctl.c mmc.c pnppower.c scratch.c sense.c zpodd.c
# data.c (CDROM data tables) + guid.c (GUID data) stay C:
list(APPEND SOURCE
    data.c
    guid.c
    cdrom.h)

add_library(cdrom MODULE ${SOURCE} cdrom.rc)
set_module_type(cdrom kmdfdriver)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(cdrom PRIVATE -Wno-format -Wno-unused-variable -Wno-pointer-sign)
    target_compile_options(cdrom PRIVATE -Wno-unknown-pragmas -Wno-incompatible-pointer-types -Wno-switch)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(cdrom PRIVATE -Wno-enum-conversion -Wno-tautological-constant-compare)
endif()

target_compile_definitions(cdrom PRIVATE DEBUG_USE_KDPRINT
    WDF_DEVICE_NO_WDMSEC_H _WIN32_WINNT=0x603 NTDDI_VERSION=0x06030000)

target_link_libraries(cdrom ntoskrnl_vista libcntpr ${PSEH_LIB})
add_dependencies(cdrom rust_cdrom)
target_link_libraries(cdrom ${CDROM_RUST_LIB})
add_importlibs(cdrom wdfldr ntoskrnl hal)
# add_pch(cdrom cdrom.h SOURCE)
add_cd_file(TARGET cdrom DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(cdrom_reg.inf)
add_driver_inf(cdrom cdrom.inf)
