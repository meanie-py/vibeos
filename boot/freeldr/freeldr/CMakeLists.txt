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

#spec2def(freeldr_pe.exe freeldr.spec)

include_directories(BEFORE include)

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

list(APPEND FREELDR_NTLDR_SOURCE)
    # cmboot.c → Rust (sdk/lib/rust/cmboot-rust), linked via freeldr-rust's
    # `uefi` feature. It is ntoskrnl/config/cmboot.c — "shared by both the
    # kernel and the bootloader" per its own header — so the kernel and the
    # loaders now link the SAME shared Rust crate (cmboot-rust) instead of
    # compiling the C three times. cmboot_shims (freeldr-rust) supplies the
    # loader-local InitSafeBootMode/CMTRACE/IS_NULL_TERMINATED.
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmboot.c
    # ntldr/conversion.c  # replaced by Rust (freeldr-rust/src/conversion.rs)
    # winldr kernel-handoff unit → Rust (freeldr-rust).
    # ntldr/registry.c    # replaced by Rust (freeldr-rust/src/registry.rs)
    # ntldr/winldr.c      # replaced by Rust (freeldr-rust/src/winldr.rs)
    # ntldr/wlmemory.c    # replaced by Rust (freeldr-rust/src/wlmemory.rs)
    # ntldr/wlregistry.c  # replaced by Rust (freeldr-rust/src/wlregistry.rs)

if(ARCH STREQUAL "i386")

    # i386trap.S now assembled by reactos-link from uefildr-cargo's .asm()
    # (BIOS freeldr.sys was pruned, so uefildr was the only consumer).
    list(APPEND FREELDR_COMMON_ASM_SOURCE)
        # arch/i386/i386trap.S
        # arch/i386/linux.S

    # setup/legacy unit → Rust (freeldr-rust: i386bug.rs/headless.rs).
    # list(APPEND FREELDR_ARC_SOURCE
    #     arch/i386/i386bug.c)   # replaced by Rust (freeldr-rust/src/i386bug.rs)
    # list(APPEND FREELDR_NTLDR_SOURCE
    #     ntldr/headless.c)      # replaced by Rust (freeldr-rust/src/headless.rs)
        # ntldr/arch/i386/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_i386.rs)

elseif(ARCH STREQUAL "amd64")
    # misc.S now assembled by reactos-link from uefildr-cargo's .asm().
    list(APPEND FREELDR_COMMON_ASM_SOURCE)
        # arch/amd64/misc.S
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
# Rust FreeLDR body (sdk/lib/rust/freeldr-rust) — built by the loaders' cargo
# ============================================================================
# Crown jewel: peloader.c (and the rest of freeldr) is translated to Rust. The
# freeldr-rust body is no longer built here by cmake custom_commands: the loader
# cargo links (uefildr-cargo / rosload-cargo, below in uefi.cmake / rosload.cmake)
# build it DIRECTLY via reactos-link `.rust_staticlib("freeldr-rust")` with the
# per-loader feature set (uefi,uefildr-rust-entry / rosload) and `.rust_user_target()`
# (standard target + i386 -sse,-sse2, exactly as the retired custom_commands did).
# This kept only ONE source of the freeldr-rust .a per loader and removed three
# `cargo build -p freeldr-rust` custom_commands (default/uefi/rosload variants;
# the default `libfreeldr_rust.a` had no consumers since the BIOS prune).
#
# We still glob the sources for the loaders' EXTRA_DEPENDS so a `.rs` edit
# re-invokes the (otherwise opaque) cargo link. freeldr-rust's own src is
# tracked by add_cargo_module's `BODY_CRATE freeldr-rust`; cmboot-rust (a
# path-dep) is tracked via FREELDR_CMBOOT_RUST_SOURCES below.
set(FREELDR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
file(GLOB_RECURSE FREELDR_CMBOOT_RUST_SOURCES CONFIGURE_DEPENDS
    ${FREELDR_RUST_WORKSPACE_DIR}/cmboot-rust/src/*.rs)
# cnt-rust cohabitation twin (libcntpr_rust.a) is now built by the loaders'
# own cargo link (reactos-link .rust_lib("cnt-rust-lib")), not reused from the
# cmake libcntpr.cmake copy. Glob its three crates (cnt-rust-lib + the
# cnt-rust/cnt-ctype path-deps) so a `.rs` edit re-invokes the loader link.
file(GLOB_RECURSE FREELDR_CNT_RUST_SOURCES CONFIGURE_DEPENDS
    ${FREELDR_RUST_WORKSPACE_DIR}/cnt-rust/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/cnt-rust-lib/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/cnt-ctype/src/*.rs)
# blcmlib (CM hive boot lib) is now compiled in-cargo by the loaders'
# reactos-link c_from_compile_db (compile_commands.json replay), not reused as
# libblcmlib.a. file-track its C sources so an edit re-invokes the cargo link.
file(GLOB FREELDR_BLCMLIB_SOURCES CONFIGURE_DEPENDS
    ${CMAKE_SOURCE_DIR}/sdk/lib/cmlib/*.c)
# blrtl (boot-loader RTL, boot/rtl/blrtl.cmake) is the blrtl-rust crate (a
# reactos-link .rust_lib sibling: the kernel's RTL rlibs plus nlsboot), the C still
# compiled in-cargo via c_from_compile_db("blrtl", …), and the per-arch asm via
# .asm_archive("blrtl_asm", …) — no reused libblrtl.a. file-track all three so an
# edit re-invokes the cargo link. The C/asm set is blrtl.cmake's (NOT a glob — the
# rtl dir holds far more than blrtl compiles), listed regardless of arch.
file(GLOB_RECURSE FREELDR_BLRTL_SOURCES CONFIGURE_DEPENDS
    ${FREELDR_RUST_WORKSPACE_DIR}/blrtl-rust/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/bitmap/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/compress/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/crc32/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/largeint/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/random/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/byteswap/src/*.rs
    ${FREELDR_RUST_WORKSPACE_DIR}/mem/src/*.rs)
list(APPEND FREELDR_BLRTL_SOURCES
    ${FREELDR_RUST_WORKSPACE_DIR}/blrtl-rust/Cargo.toml
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/image.c
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/unicode.c
    # per-arch asm (.asm_archive): i386 debug_asm/rtlmem/rtlswap, amd64 debug_asm.
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/i386/debug_asm.S
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/i386/rtlmem.s
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/i386/rtlswap.S
    ${CMAKE_SOURCE_DIR}/sdk/lib/rtl/amd64/debug_asm.S)
# cport-rust (NS16550 serial, sdk/lib/cportlib/CMakeLists.txt) + hal-rust's
# minihal variant (hal/halx86/CMakeLists.txt, i386-only) are now DIRECT cargo
# siblings of the loader link (reactos-link .rust_lib()), not reused from the
# cmake-built libcport_rust.a / libhal_rust_mini.a. Glob their crate sources so
# a `.rs` edit re-invokes the (opaque) cargo link. (The cmake `cportlib` /
# `mini_hal` targets stay deps for the C/asm leftovers — the cport cohab stub
# and mini_hal's systimer.S asm — which the loaders still reuse.)
file(GLOB_RECURSE FREELDR_CPORT_RUST_SOURCES CONFIGURE_DEPENDS
    ${FREELDR_RUST_WORKSPACE_DIR}/cport-rust/src/*.rs)
file(GLOB_RECURSE FREELDR_HAL_MINI_SOURCES CONFIGURE_DEPENDS
    ${FREELDR_RUST_WORKSPACE_DIR}/hal-rust/src/*.rs)

# BIOS prune: the legacy PC-AT BIOS loader (freeldr.sys = frldr16 + freeldr_pe,
# built by pcat.cmake) is gone — x86 boots UEFI-only via uefildr.efi. rosload
# still imports freeldr.sys services (under UEFI those resolve at runtime
# against uefildr.efi, which the PE loader registers as "freeldr.sys" and which
# exports the same freeldr.spec symbols). The freeldr import LIBRARY that
# satisfies those imports is now self-hosted by reactos-link from freeldr.spec
# (rosload-cargo's .import_lib()), so the cmake spec2def(freeldr.sys freeldr.spec
# ADD_IMPORTLIB) is retired — nothing else referenced the libfreeldr target.
# rosload is the 2nd stage loader (loads ntoskrnl); ARM64 uses UEFI-only boot
if(NOT ARCH STREQUAL "arm64")
    include(rosload.cmake)
endif()
include(uefi.cmake)
# The legacy BIOS loader, freeldr.sys (restored 2026-10-05; amd64 only).
if(ARCH STREQUAL "amd64")
    include(pcat.cmake)
endif()
