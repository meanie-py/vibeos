include_directories(BEFORE ${CMAKE_CURRENT_SOURCE_DIR})

#add_definitions(-D_DEBUG)

# ============================================================================
# uniata.sys — 100% Rust; the UniATA C is GONE (2026-09-20)
# ============================================================================
# The UniATA miniport is translated to Rust (uniata-rust): DriverEntry + the
# SCSIPORT HwXxx miniport machinery + the PCI/AHCI/SATA register stacks + the
# chip-detect tables all come from libuniata_rust.a, and the driver-private DDK
# types (HW_DEVICE_EXTENSION/HW_CHANNEL/HW_LU_EXTENSION/ATA_REQ/IORES + the
# IDENTIFY/SRB/CDB layouts) from the per-driver bindgen substrate that is now
# checked in as uniata-rust/src/types_{i386,amd64,arm64}.rs.
#
# The 9 .cpp bodies and the 16 headers they needed were DELETED, not just
# flipped: every one carried `Copyright (c) Alexandr A. Telyatnikov` (or
# VorontSOFT, or Microsoft for ntddscsi.h) with the BSD warranty DISCLAIMER and
# no "Redistribution and use ... are permitted" clause, i.e. no grant at all --
# and id_ata.cpp's own header says the skeleton came from the NT4 DDK's
# ATAPI.SYS. Same call the owner made for the atapi C the same day: the Rust
# stays, the unlicensed C does not. See the PROVENANCE note at the top of
# sdk/lib/rust/uniata-rust/src/lib.rs. This is the ONE place the never-delete
# rule is overridden, and only for the C.
#
# ros_glue/ros_glue.cpp went with them. It existed solely as the data home for
# three mutable ULONGs (MajorVersion/MinorVersion/BuildNumber); they are now
# `#[no_mangle] pub static mut` in id_ata_part3.rs with ros_glue's own 5.2.0
# initialisers. That also retired the stdafx.h PCH, which had no C++ TU left to
# serve, and the `inc/` include path with it.
#
# rust_varargs_shim.c stays: it is ours, and it holds the ONE thing stable Rust
# cannot do -- DEFINE the C-variadic _PrintNtConsole entry point (`va_start`).
# It trampolines into uniata-rust's _PrintNtConsole_v (which keeps the 512-byte
# buffer + the HalDisplayString call) and forwards the va_list into ntoskrnl's
# _vsnprintf, so the crate builds on PLAIN STABLE rustc: no bootstrap escape
# hatch, no nightly feature gate. Standard per-arch RUST_TARGET, i386 no-SSE
# RUSTFLAGS. First C++-tier proof (rustconv tree-sitter-cpp).
list(APPEND SOURCE
    rust_varargs_shim.c)

set(UNIATA_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(UNIATA_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-uniata-target" uniata-rust)
set(UNIATA_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libuniata_rust.a")
if(ARCH STREQUAL "i386")
    set(UNIATA_RUST_TARGET "i686-pc-windows-gnu")
    set(UNIATA_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(UNIATA_RUST_TARGET "x86_64-pc-windows-gnu")
    set(UNIATA_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(UNIATA_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(UNIATA_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(UNIATA_RUST_SRCS "${UNIATA_RUST_WORKSPACE_DIR}" uniata-rust)
add_custom_command(
    OUTPUT ${UNIATA_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${UNIATA_RUST_TARGET_DIR}
        RUSTFLAGS=${UNIATA_RUST_RUSTFLAGS}
        cargo build --release --target ${UNIATA_RUST_TARGET}
            -p uniata-rust
            --manifest-path ${UNIATA_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${UNIATA_RUST_TARGET_DIR}/${UNIATA_RUST_TARGET}/release/libuniata_rust.a
        ${UNIATA_RUST_LIB}
    DEPENDS ${UNIATA_RUST_SRCS}
            ${UNIATA_RUST_WORKSPACE_DIR}/uniata-rust/Cargo.toml
    WORKING_DIRECTORY ${UNIATA_RUST_WORKSPACE_DIR}
    COMMENT "Building uniata-rust (${UNIATA_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_uniata DEPENDS ${UNIATA_RUST_LIB})

add_library(uniata MODULE
    rust_cohab_stub.cpp
    ${SOURCE}
    idedma.rc)

if(MSVC)
    # Disable warning C4267: '=': conversion from 'size_t' to 'USHORT', possible loss of data
    # Disable warning C4838: conversion from 'int' to 'ULONG' requires a narrowing conversion
    target_compile_options(uniata PRIVATE /wd4267 /wd4838)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(uniata PRIVATE -Wno-narrowing -Wno-unused-but-set-variable)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(uniata PRIVATE -Wno-unused-const-variable)
endif()

set_module_type(uniata kernelmodedriver)
add_importlibs(uniata scsiport ntoskrnl hal)
add_dependencies(uniata rust_uniata)
target_link_libraries(uniata ${UNIATA_RUST_LIB})
add_cd_file(TARGET uniata DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(uniata_reg.inf)
add_driver_inf(uniata uniata_comm.inf)
