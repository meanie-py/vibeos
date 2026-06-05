##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for rosload 2nd stage loader
## COPYRIGHT:   Copyright 2024 Timo Kreuzer <timo.kreuzer@reactos.org>
##

spec2def(rosload.exe rosload.spec)

list(APPEND ROSLOAD_SOURCE
    include/freeldr.h
    # strsafe_shim.c (variadic RtlStringCbPrintfA shim) is now compiled directly
    # by reactos-link via rosload-cargo's .c_shims() — a self-contained leaf, no
    # freeldr compile env needed. rosload_objs below is now ONLY the per-arch asm.
    # lib/strsafe_shim.c
    # Cohabitation: each .c below is replaced by its Rust translation in
    # libfreeldr_rust_rosload.a (--features rosload). .c kept on disk.
    # bootmgr.c          # replaced by Rust (freeldr-rust/src/bootmgr.rs)
    # custom.c           # replaced by Rust (freeldr-rust/src/custom.rs)
    # linuxboot.c        # replaced by Rust (freeldr-rust/src/linuxboot.rs)
    # miscboot.c         # replaced by Rust (freeldr-rust/src/miscboot.rs)
    # options.c          # replaced by Rust (freeldr-rust/src/options.rs)
    # oslist.c           # replaced by Rust (freeldr-rust/src/oslist.rs)
    # lib/rtl/libsupp.c  # replaced by Rust (freeldr-rust/src/libsupp.rs)
    # cmboot.c → Rust (sdk/lib/rust/cmboot-rust), linked via freeldr-rust's
    # `rosload` feature (cmboot_shims supplies the loader-local globals). Same
    # shared CM boot-init crate the kernel and uefildr link.
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmboot.c
    # ntldr/conversion.c # replaced by Rust (freeldr-rust/src/conversion.rs)
    # ntldr/inffile.c    # replaced by Rust (freeldr-rust/src/inffile.rs)
    # ntldr/registry.c   # replaced by Rust (freeldr-rust/src/registry.rs)
    # ntldr/setupldr.c   # replaced by Rust (freeldr-rust/src/setupldr.rs)
    # ntldr/winldr.c     # replaced by Rust (freeldr-rust/src/winldr.rs)
    # ntldr/wlmemory.c   # replaced by Rust (freeldr-rust/src/wlmemory.rs)
    # ntldr/wlregistry.c # replaced by Rust (freeldr-rust/src/wlregistry.rs)
)

if(ARCH STREQUAL "i386")

    list(APPEND ROSLOAD_SOURCE
        # arch/i386/halstub.c        # replaced by Rust (freeldr-rust/src/halstub.rs)
        # arch/i386/ntoskrnl.c       # replaced by Rust (freeldr-rust/src/ntoskrnl.rs)
        # disk/scsiport.c — DELETED: SCSI emulation (ScsiPort* exports for
        #   ntbootdd.sys SCSI/RAID boot). Unused on our targets; exports
        #   removed from rosload.spec too.
        # ntldr/arch/i386/winldr.c   # replaced by Rust (freeldr-rust/src/winldr_i386.rs)
        # ntldr/headless.c           # replaced by Rust (freeldr-rust/src/headless.rs)
    )

    # drvmap.S is 16-bit (.code16) — clang can't assemble it, so it stays a
    # gcc-built object (reused directly by rosload-cargo). linux.S → reactos-link
    # (rosload-cargo .asm()); it needs only asm.inc + pcbios.h (REGS_*).
    if(CMAKE_ASM16_GCC)
        compile_asm_with_gcc(arch/i386/drvmap.S _rosload_drvmap_obj)
        set(ROSLOAD_GCC_OBJECTS ${_rosload_drvmap_obj})
    else()
        list(APPEND ROSLOAD_ASM_SOURCE arch/i386/drvmap.S)
    endif()
    # arch/i386/linux.S → reactos-link

elseif(ARCH STREQUAL "amd64")

    list(APPEND ROSLOAD_SOURCE
        # ntldr/arch/amd64/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_amd64.rs)
    )
    # arch/amd64/misc.S + arch/amd64/linux.S → reactos-link (rosload-cargo .asm())

else()
    #TBD
endif()

# rosload's asm (linux.S/misc.S) is assembled by reactos-link from raw .S now;
# only the i386 16-bit drvmap stays a gcc-built object. With ROSLOAD_SOURCE all
# Rust + the asm moved, the old `rosload_objs` OBJECT lib is retired.
if(ROSLOAD_GCC_OBJECTS)
    set_source_files_properties(${ROSLOAD_GCC_OBJECTS} PROPERTIES GENERATED TRUE EXTERNAL_OBJECT TRUE)
endif()

# ============================================================================
# rosload.exe — CARGO-DRIVEN final link (sdk/lib/rust/rosload-cargo)
# ============================================================================
# The final 2nd-stage-loader link is driven by cargo (rosload-cargo's build.rs
# → reactos-link, ModuleKind::NativeExe) instead of add_executable. The cargo
# build REUSES libfreeldr_rust_rosload.a + the leaf libs + the freeldr import
# lib (add_importlibs(rosload freeldr) → libfreeldr.a) + the rosload_objs
# objects below; cmake's own link is retired. The i386 force-undefined HAL
# dispatch-table pulls (-u _HalDispatchTable/_HalPrivateDispatchTable) now live
# in rosload-cargo's build.rs (Linker::force_undefined).

# `rosload_objs` is RETIRED. It used to compile rosload's C+asm into reused
# .objs, but strsafe_shim.c + cmboot are gone (Rust / reactos-link c_shims) and
# the per-arch asm (linux.S/misc.S) is now assembled by reactos-link from raw
# .S in rosload-cargo's build.rs (.asm() — they need only asm.inc + pcbios.h).
# The sole remaining cmake-built input is the i386 16-bit drvmap_gcc.obj
# (compile_asm_with_gcc above; clang can't .code16), reused directly.

# cmake targets / files whose .a/.obj the cargo link reuses — DEPENDS so `ninja
# rosload` builds them before invoking cargo. libfreeldr = the freeldr import
# lib (spec2def(freeldr.sys ... ADD_IMPORTLIB) in CMakeLists.txt), which
# rosload's freeldr.sys imports bind against.
#
# As in uefi.cmake: a DEPENDS on a custom-TARGET (rust_freeldr_rosload, rust_cnt)
# is order-only and does NOT relink rosload when that .a's content changes — so
# also DEPENDS on the .a FILE PATHS (each a custom_command OUTPUT, file-tracked
# by ninja). reactos-link's per-static_lib cargo:rerun-if-changed then re-runs
# the actual link. Library targets (libfreeldr, blcmlib, …) already file-track.
set(ROSLOAD_EXTRA_DEPENDS
    libfreeldr blcmlib blrtl libcntpr
    # source files reactos-link compiles directly (file-track for ninja):
    ${CMAKE_CURRENT_SOURCE_DIR}/lib/strsafe_shim.c
    # custom-target-produced rust .a link inputs — depend on the FILES:
    rust_freeldr_rosload ${FREELDR_RUST_ROSLOAD_LIB}
    rust_cnt ${CMAKE_BINARY_DIR}/sdk/lib/crt/libcntpr_rust.a)
if(ARCH STREQUAL "i386")
    list(APPEND ROSLOAD_EXTRA_DEPENDS
        mini_hal rust_hal_mini ${CMAKE_BINARY_DIR}/hal/halx86/libhal_rust_mini.a
        ${CMAKE_CURRENT_BINARY_DIR}/drvmap_gcc.obj)
endif()
# (linux.S + amd64 misc.S were flipped to Rust global_asm! in freeldr-rust;
#  tracked via FREELDR_RUST_SOURCES. Only the 16-bit drvmap.obj stays cmake.)

# KIND native → no post-link (rosload's old cmake POST_BUILD was a no-op).
add_cargo_module(rosload rosload-cargo rosload.exe
    KIND native DESTINATION loader EXTRA_DEPENDS ${ROSLOAD_EXTRA_DEPENDS})
