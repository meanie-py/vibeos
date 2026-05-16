##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Common build definitions
## COPYRIGHT:   Copyright 2003 Brian Palmer <brianp@sginet.com>
##              Copyright 2011-2014 Amine Khaldi <amine.khaldi@reactos.org>
##              Copyright 2011-2014 Timo Kreuzer <timo.kreuzer@reactos.org>
##              Copyright 2014 Hervé Poussineau <hpoussin@reactos.org>
##              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
##

if(MSVC)
    # Explicitly use string pooling
    add_compile_options("/GF")
endif()

#spec2def(freeldr_pe.exe freeldr.spec)

include_directories(BEFORE include)
include_directories(
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include
    ${REACTOS_SOURCE_DIR}/sdk/lib/cmlib
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/elf)

add_definitions(-D_NTHAL_ -D_BLDR_ -D_NTSYSTEM_)

list(APPEND FREELDR_ARC_SOURCE
    arcname.c
    arch/arcemul.c
    arch/archwsup.c
    disk/disk.c)
    # disk unit → Rust (freeldr-rust: partition.rs/ramdisk.rs). Cohabitation:
    # .c kept on disk, Rust #[no_mangle] takes over.
    # disk/partition.c  # replaced by Rust (freeldr-rust/src/partition.rs)
    # disk/ramdisk.c    # replaced by Rust (freeldr-rust/src/ramdisk.rs)
    # disk/scsiport.c
    # lib/fs/pxe.c

list(APPEND FREELDR_BOOTLIB_SOURCE
    lib/arcsupp.c
    lib/debug.c
    # lib/peloader.c  # replaced by Rust (sdk/lib/rust/freeldr-rust)
    lib/cache/blocklist.c
    lib/cache/cache.c
    lib/comm/rs232.c
    ## add KD support
    # fs unit → Rust (freeldr-rust: fs.rs/fat.rs/iso.rs). ntfs/ext/btrfs
    # PRUNED entirely (no Rust, no C): livecd boots ISO, ramdisk is FAT,
    # so fs.rs FileSystems[] = [Iso,Fat,Pxe] only; the other 3 are dead
    # weight. Keep .c files on disk (never-delete), just unreferenced.
    # lib/fs/btrfs.c   # pruned (not in fs.rs FileSystems[])
    # lib/fs/ext.c     # pruned
    # lib/fs/fat.c     # replaced by Rust (freeldr-rust/src/fat.rs)
    # lib/fs/fs.c      # replaced by Rust (freeldr-rust/src/fs.rs)
    # lib/fs/iso.c     # replaced by Rust (freeldr-rust/src/iso.rs)
    # lib/fs/ntfs.c    # pruned
    lib/inifile/ini_init.c
    lib/inifile/inifile.c
    lib/inifile/parse.c
    lib/mm/meminit.c
    lib/mm/mm.c
    lib/mm/heap.c)

list(APPEND FREELDR_BOOTMGR_SOURCE
    include/freeldr.h
    settings.c
    # ui unit → Rust (freeldr-rust: ui/tui/tuimenu/minitui/noui/video.rs).
    # ui/directui.c is `#if 0` dead code in C (compiles to nothing) and the
    # Rust directui.rs is deliberately NOT wired — symmetric, kept on disk.
    # ui/gui.c untranslated (already commented). Cohabitation: .c kept on
    # disk, Rust #[no_mangle] takes over.
    ui/directui.c
    # ui/gui.c
    # ui/minitui.c  # replaced by Rust (freeldr-rust/src/minitui.rs)
    # ui/noui.c     # replaced by Rust (freeldr-rust/src/noui.rs)
    # ui/tui.c      # replaced by Rust (freeldr-rust/src/tui.rs)
    # ui/tuimenu.c  # replaced by Rust (freeldr-rust/src/tuimenu.rs)
    # ui/ui.c       # replaced by Rust (freeldr-rust/src/ui.rs)
    # ui/video.c    # replaced by Rust (freeldr-rust/src/video.rs)
    )

list(APPEND FREELDR_NTLDR_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmboot.c)
    # ntldr/conversion.c  # replaced by Rust (freeldr-rust/src/conversion.rs)
    # winldr kernel-handoff unit → Rust (freeldr-rust). cmboot.c kept (it
    # is ntoskrnl/config/cmboot.c — shared kernel src, not a freeldr file;
    # registry.rs externs it). Cohabitation: .c kept on disk.
    # ntldr/registry.c    # replaced by Rust (freeldr-rust/src/registry.rs)
    # ntldr/winldr.c      # replaced by Rust (freeldr-rust/src/winldr.rs)
    # ntldr/wlmemory.c    # replaced by Rust (freeldr-rust/src/wlmemory.rs)
    # ntldr/wlregistry.c  # replaced by Rust (freeldr-rust/src/wlregistry.rs)

if(ARCH STREQUAL "i386")

    list(APPEND FREELDR_COMMON_ASM_SOURCE
        arch/i386/i386trap.S)
        # arch/i386/linux.S

    list(APPEND FREELDR_ARC_SOURCE
        arch/i386/i386bug.c)

    list(APPEND FREELDR_NTLDR_SOURCE
        ntldr/headless.c)
        # ntldr/arch/i386/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_i386.rs)

elseif(ARCH STREQUAL "amd64")
    list(APPEND FREELDR_COMMON_ASM_SOURCE
        arch/amd64/misc.S)
        # arch/amd64/linux.S

    list(APPEND FREELDR_ARC_SOURCE
        arch/i386/i386bug.c)
        # arch/i386/ntoskrnl.c  # replaced by Rust (freeldr-rust/src/ntoskrnl.rs)

    # ntldr/arch/amd64/winldr.c → Rust (freeldr-rust/src/winldr_amd64.rs)
    # (FREELDR_NTLDR_SOURCE amd64 entry removed — Rust takes over)

elseif(ARCH STREQUAL "arm64")
    # ntldr/arch/arm64/winldr.c → Rust (freeldr-rust/src/winldr_arm64.rs)

else()
    #TBD
endif()

list(APPEND FREELDR_BASE_SOURCE
    freeldr.c
    lib/rtl/libsupp.c)
    # ntldr/ntldropts.c → Rust (freeldr-rust/src/ntldropts.rs). Cohabitation.

if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
    # Must be included together with disk/scsiport.c
    list(APPEND FREELDR_BASE_SOURCE
        ${CMAKE_CURRENT_BINARY_DIR}/freeldr.def)
endif()

# ============================================================================
# Rust FreeLDR static archive (sdk/lib/rust/freeldr-rust → libfreeldr_rust.a)
# ============================================================================
# Crown jewel: peloader.c is translated to Rust. Its #[no_mangle] cdecl
# exports (PeLdr*, FrLdrModuleList, PeLdrImportDllLoadCallback) replace
# the C lib/peloader.c (commented out of FREELDR_BOOTLIB_SOURCE above).
# Mirrors the hal-rust integration (hal/halx86/CMakeLists.txt): a single
# staticlib linked into the freeldr common libs; rosload imports PeLdr*
# from freeldr.sys via freeldr.spec (ABI unchanged — still cdecl).
set(FREELDR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(FREELDR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-freeldr-target")
if(ARCH STREQUAL "i386")
    set(FREELDR_RUST_TARGET "i686-pc-windows-gnu")
    # FreeLDR runs before CR4.OSFXSR — no SSE (same as hal-rust/mini_hal).
    set(FREELDR_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(FREELDR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(FREELDR_RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(FREELDR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(FREELDR_RUST_EXTRA_ENV "")
endif()
set(FREELDR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfreeldr_rust.a")
add_custom_command(
    OUTPUT ${FREELDR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FREELDR_RUST_TARGET_DIR} ${FREELDR_RUST_EXTRA_ENV}
        cargo build --release --target ${FREELDR_RUST_TARGET}
            -p freeldr-rust
            --manifest-path ${FREELDR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FREELDR_RUST_TARGET_DIR}/${FREELDR_RUST_TARGET}/release/libfreeldr_rust.a
        ${FREELDR_RUST_LIB}
    DEPENDS ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/Cargo.toml
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/lib.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/ffi.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/peloader.rs
    COMMENT "Building freeldr-rust (libfreeldr_rust.a)"
    VERBATIM)
add_custom_target(rust_freeldr DEPENDS ${FREELDR_RUST_LIB})

# ARM64 uses UEFI only, no legacy PC-AT boot
if(NOT ARCH STREQUAL "arm64")
    include(pcat.cmake)
endif()
# rosload is the 2nd stage PC-AT loader; ARM64 uses UEFI-only boot
if(NOT ARCH STREQUAL "arm64")
    include(rosload.cmake)
endif()
include(uefi.cmake)
