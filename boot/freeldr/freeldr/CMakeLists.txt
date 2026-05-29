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

list(APPEND FREELDR_ARC_SOURCE)
    # arcname.c          # replaced by Rust (freeldr-rust/src/arcname.rs)
    # arch/arcemul.c     # replaced by Rust (freeldr-rust/src/arcemul.rs)
    # arch/archwsup.c    # replaced by Rust (freeldr-rust/src/archwsup.rs)
    # disk/disk.c        # empty TU (only #include <freeldr.h>) — removed
    # disk unit → Rust (freeldr-rust: partition.rs/ramdisk.rs). Cohabitation:
    # .c kept on disk, Rust #[no_mangle] takes over.
    # disk/partition.c  # replaced by Rust (freeldr-rust/src/partition.rs)
    # disk/ramdisk.c    # replaced by Rust (freeldr-rust/src/ramdisk.rs)
    # disk/scsiport.c
    # lib/fs/pxe.c

list(APPEND FREELDR_BOOTLIB_SOURCE)
    # lib-pure/cache/inifile units → Rust (freeldr-rust). Prior commits
    # (4a90a07f cache+inifile, 49677d6f lib-pure) wired `pub mod` but left
    # these .c active → cold-link dup. Completing the cohabitation flips
    # (AGENTS option 1; symbols the .c also provided are #[no_mangle]-
    # finished in the matching .rs as the undefined oracle surfaces them).
    # lib/arcsupp.c       # replaced by Rust (freeldr-rust/src/arcsupp.rs)
    # lib/debug.c         # replaced by Rust (freeldr-rust/src/debug.rs)
    # lib/peloader.c      # replaced by Rust (sdk/lib/rust/freeldr-rust)
    # lib/cache/blocklist.c  # replaced by Rust (freeldr-rust/src/blocklist.rs)
    # lib/cache/cache.c   # replaced by Rust (freeldr-rust/src/cache.rs)
    # lib/comm/rs232.c    # replaced by Rust (freeldr-rust/src/rs232.rs)
    ## add KD support
    # fs unit → Rust (freeldr-rust: fs.rs/fat.rs/iso.rs). livecd boots ISO,
    # ramdisk is FAT, so fs.rs FileSystems[] = [Iso,Fat,Pxe] only.
    # btrfs/ext/ntfs read drivers had no Rust + no live caller → DELETED
    # (lib/fs/{btrfs,ext,ntfs}.c + include/fs/{btrfs,ext,ntfs}.h).
    # lib/fs/fat.c     # replaced by Rust (freeldr-rust/src/fat.rs)
    # lib/fs/fs.c      # replaced by Rust (freeldr-rust/src/fs.rs)
    # lib/fs/iso.c     # replaced by Rust (freeldr-rust/src/iso.rs)
    # lib/inifile/ini_init.c  # replaced by Rust (freeldr-rust/src/ini_init.rs)
    # lib/inifile/inifile.c   # replaced by Rust (freeldr-rust/src/inifile.rs)
    # lib/inifile/parse.c     # replaced by Rust (freeldr-rust/src/parse.rs)
    # mm unit → Rust (freeldr-rust: meminit/mm/heap.rs). Prior mm commit
    # (f7322948) wired `pub mod` but left the .c active → cold-link dup.
    # Completing the cohabitation flip (AGENTS option 1). The Ex*/
    # AddMemoryDescriptor symbols heap.c/meminit.c also provided are now
    # #[no_mangle]-finished in heap.rs/meminit.rs.
    # lib/mm/meminit.c  # replaced by Rust (freeldr-rust/src/meminit.rs)
    # lib/mm/mm.c       # replaced by Rust (freeldr-rust/src/mm.rs)
    # lib/mm/heap.c     # replaced by Rust (freeldr-rust/src/heap.rs)

list(APPEND FREELDR_BOOTMGR_SOURCE
    include/freeldr.h
    # settings.c        # replaced by Rust (freeldr-rust/src/settings.rs)
    # ui unit → Rust (freeldr-rust: ui/tui/tuimenu/minitui/noui/video.rs).
    # ui/directui.c (#if 0) and ui/gui.c (never-implemented empty-stub GUI,
    # GuiVtbl only referenced under #if 0 "no GUI mode yet") were dead — deleted.
    # directui.rs kept on disk (deliberately NOT wired).
    # Cohabitation: live .c kept on disk, Rust #[no_mangle] takes over.
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

    # setup/legacy unit → Rust (freeldr-rust: i386bug.rs/headless.rs).
    # list(APPEND FREELDR_ARC_SOURCE
    #     arch/i386/i386bug.c)   # replaced by Rust (freeldr-rust/src/i386bug.rs)
    # list(APPEND FREELDR_NTLDR_SOURCE
    #     ntldr/headless.c)      # replaced by Rust (freeldr-rust/src/headless.rs)
        # ntldr/arch/i386/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_i386.rs)

elseif(ARCH STREQUAL "amd64")
    list(APPEND FREELDR_COMMON_ASM_SOURCE
        arch/amd64/misc.S)
        # arch/amd64/linux.S

    # setup/legacy unit → Rust (freeldr-rust: i386bug.rs).
    # list(APPEND FREELDR_ARC_SOURCE
    #     arch/i386/i386bug.c)  # replaced by Rust (freeldr-rust/src/i386bug.rs)
        # arch/i386/ntoskrnl.c  # replaced by Rust (freeldr-rust/src/ntoskrnl.rs)

    # ntldr/arch/amd64/winldr.c → Rust (freeldr-rust/src/winldr_amd64.rs)
    # (FREELDR_NTLDR_SOURCE amd64 entry removed — Rust takes over)

elseif(ARCH STREQUAL "arm64")
    # ntldr/arch/arm64/winldr.c → Rust (freeldr-rust/src/winldr_arm64.rs)

else()
    #TBD
endif()

list(APPEND FREELDR_BASE_SOURCE
    # freeldr.c          # replaced by Rust (freeldr-rust/src/freeldr.rs)
    lib/strsafe_shim.c)
    # lib/strsafe_shim.c — variadic RtlStringCbPrintfA out-of-line shim. The
    # Rust freeldr-rust crate cannot host the variadic def on stable
    # (c_variadic still unstable for the DEF side); rustconv's degraded
    # stub silently left %u/%s unformatted in boot paths. Real shim
    # forwards to ntstrsafe.h's static-inline RtlStringCbVPrintfA.
    # lib/rtl/libsupp.c → Rust (freeldr-rust/src/libsupp.rs). Completing
    # the cohabitation flip (49677d6f wired pub mod, left .c active).
    # lib/rtl/libsupp.c   # replaced by Rust (freeldr-rust/src/libsupp.rs)
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

# Second archive for the UEFI link (uefildr.efi). Mirrors hal-rust's
# libhal_rust_mini.a: same crate, `--features uefi`, separate
# CARGO_TARGET_DIR (cargo can't share a target dir across feature sets).
# Gates out the BIOS-only PC-AT machine entry (machpc MachInit, pchw
# StallExecutionProcessor, vidfb_pc_fallback) so they don't dup the
# UEFI-active C (uefisetup.c/vidfb.c/uefildr.c).
set(FREELDR_RUST_UEFI_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-freeldr-uefi-target")
set(FREELDR_RUST_UEFI_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfreeldr_rust_uefi.a")
add_custom_command(
    OUTPUT ${FREELDR_RUST_UEFI_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FREELDR_RUST_UEFI_TARGET_DIR} ${FREELDR_RUST_EXTRA_ENV}
        cargo build --release --target ${FREELDR_RUST_TARGET}
            -p freeldr-rust --features uefi
            --manifest-path ${FREELDR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FREELDR_RUST_UEFI_TARGET_DIR}/${FREELDR_RUST_TARGET}/release/libfreeldr_rust.a
        ${FREELDR_RUST_UEFI_LIB}
    DEPENDS ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/Cargo.toml
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/lib.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/ffi.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/peloader.rs
    COMMENT "Building freeldr-rust UEFI variant (libfreeldr_rust_uefi.a, --features uefi)"
    VERBATIM)
add_custom_target(rust_freeldr_uefi DEPENDS ${FREELDR_RUST_UEFI_LIB})

# Third archive for the rosload.exe link (2nd-stage loader). Same crate,
# --features rosload, separate CARGO_TARGET_DIR. Selects bootmgr/custom/
# options/oslist/winldr/wlmemory/wlregistry/registry/setupldr/inffile/
# headless/conversion/libsupp/miscboot/linuxboot/drivemap/ntoskrnl/winldr_arch
# and gates out the BIOS/UEFI hardware code (machpc/pchw/tui/ueficon/etc).
# rosload imports those services from freeldr.sys via add_importlibs(rosload
# freeldr). See lib.rs `feature = "rosload"` cfg gates for the module split.
set(FREELDR_RUST_ROSLOAD_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-freeldr-rosload-target")
set(FREELDR_RUST_ROSLOAD_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfreeldr_rust_rosload.a")
add_custom_command(
    OUTPUT ${FREELDR_RUST_ROSLOAD_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FREELDR_RUST_ROSLOAD_TARGET_DIR} ${FREELDR_RUST_EXTRA_ENV}
        cargo build --release --target ${FREELDR_RUST_TARGET}
            -p freeldr-rust --features rosload
            --manifest-path ${FREELDR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FREELDR_RUST_ROSLOAD_TARGET_DIR}/${FREELDR_RUST_TARGET}/release/libfreeldr_rust.a
        ${FREELDR_RUST_ROSLOAD_LIB}
    DEPENDS ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/Cargo.toml
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/lib.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/ffi.rs
            ${FREELDR_RUST_WORKSPACE_DIR}/freeldr-rust/src/peloader.rs
    COMMENT "Building freeldr-rust rosload variant (libfreeldr_rust_rosload.a, --features rosload)"
    VERBATIM)
add_custom_target(rust_freeldr_rosload DEPENDS ${FREELDR_RUST_ROSLOAD_LIB})

# ARM64 uses UEFI only, no legacy PC-AT boot
if(NOT ARCH STREQUAL "arm64")
    include(pcat.cmake)
endif()
# rosload is the 2nd stage PC-AT loader; ARM64 uses UEFI-only boot
if(NOT ARCH STREQUAL "arm64")
    include(rosload.cmake)
endif()
include(uefi.cmake)
