##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for rosload 2nd stage loader
## COPYRIGHT:   Copyright 2024 Timo Kreuzer <timo.kreuzer@reactos.org>
##

spec2def(rosload.exe rosload.spec)

list(APPEND ROSLOAD_SOURCE
    include/freeldr.h
    # strsafe_shim.c — variadic RtlStringCbPrintfA out-of-line shim;
    # the Rust freeldr-rust crate can't host a variadic def on stable
    # (same constraint as freeldr_pe.exe / uefildr.efi). Compiled into
    # rosload.exe directly, matching FREELDR_BASE_SOURCE in CMakeLists.txt.
    lib/strsafe_shim.c
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

    if(CMAKE_ASM16_GCC)
        compile_asm_with_gcc(arch/i386/drvmap.S _rosload_drvmap_obj)
        set(ROSLOAD_GCC_OBJECTS ${_rosload_drvmap_obj})
    else()
        list(APPEND ROSLOAD_ASM_SOURCE arch/i386/drvmap.S)
    endif()
    list(APPEND ROSLOAD_ASM_SOURCE
        arch/i386/linux.S)

elseif(ARCH STREQUAL "amd64")

    list(APPEND ROSLOAD_SOURCE
        # ntldr/arch/amd64/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_amd64.rs)
    )

    list(APPEND ROSLOAD_ASM_SOURCE
        arch/amd64/misc.S
        arch/amd64/linux.S
    )

else()
    #TBD
endif()

add_asm_files(rosload_asm ${ROSLOAD_ASM_SOURCE})

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

# Header-dependent C + asm objects the cargo link reuses (strsafe_shim, cmboot,
# the per-arch asm). add_executable(rosload) used to compile them; keep them
# compiled via an OBJECT lib now that the link is cargo-driven. Same
# directory-scoped compile env as the old add_executable. (The i386 16-bit
# drvmap_gcc.obj is produced by compile_asm_with_gcc above and reused directly.)
add_library(rosload_objs OBJECT
    ${ROSLOAD_SOURCE}
    ${rosload_asm})
set_target_properties(rosload_objs PROPERTIES DEFINE_SYMBOL "")
add_dependencies(rosload_objs bugcodes asm xdk)

# cmake targets / files whose .a/.obj the cargo link reuses — DEPENDS so `ninja
# rosload` builds them before invoking cargo. libfreeldr = the freeldr import
# lib (spec2def(freeldr.sys ... ADD_IMPORTLIB) in CMakeLists.txt), which
# rosload's freeldr.sys imports bind against.
set(ROSLOAD_EXTRA_DEPENDS
    rosload_objs rust_freeldr_rosload libfreeldr blcmlib blrtl libcntpr rust_cnt)
if(ARCH STREQUAL "i386")
    list(APPEND ROSLOAD_EXTRA_DEPENDS mini_hal rust_hal_mini
        ${CMAKE_CURRENT_BINARY_DIR}/drvmap_gcc.obj)
endif()

# KIND native → no post-link (rosload's old cmake POST_BUILD was a no-op).
add_cargo_module(rosload rosload-cargo rosload.exe
    KIND native DESTINATION loader EXTRA_DEPENDS ${ROSLOAD_EXTRA_DEPENDS})
