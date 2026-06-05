##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for UEFI
## COPYRIGHT:   Copyright 2023 Justin Miller <justinmiller100@gmail.com>
##

include_directories(BEFORE
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/edk2
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr/include
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr/include/arch/uefi)

list(APPEND UEFILDR_ARC_SOURCE
    ${FREELDR_ARC_SOURCE}
    # arch/vidfb.c    # replaced by Rust (freeldr-rust/src/vidfb.rs)
    # arch/vgafont.c  # replaced by Rust (freeldr-rust/src/vgafont.rs)
)
    # uefi-hw unit → Rust (freeldr-rust: ueficon/uefidisk/uefihw/uefimem/
    # uefivid.rs + uefisetup/uefi_stubs/uefiutil.rs under feature="uefi").
    # Cohabitation: .c kept on disk, Rust #[no_mangle] takes over.
    # arch/uefi/ueficon.c   # replaced by Rust (freeldr-rust/src/ueficon.rs)
    # arch/uefi/uefidisk.c  # replaced by Rust (freeldr-rust/src/uefidisk.rs)
    # arch/uefi/uefihw.c    # replaced by Rust (freeldr-rust/src/uefihw.rs)
    # arch/uefi/uefimem.c   # replaced by Rust (freeldr-rust/src/uefimem.rs)
    # arch/uefi/uefivid.c   # replaced by Rust (freeldr-rust/src/uefivid.rs)
    # arch/uefi/stubs.c     # replaced by Rust (freeldr-rust/src/uefi_stubs.rs)
    # arch/uefi/uefisetup.c # replaced by Rust (freeldr-rust/src/uefisetup.rs)
    # arch/uefi/uefiutil.c  # replaced by Rust (freeldr-rust/src/uefiutil.rs)
    # arch/uefi/uefildr.c   # NOT commented — owns uefildr.exe binary entry.
    #                       # The Rust translation lives in freeldr-rust/src/
    #                       # uefildr.rs, compiled but with #[no_mangle] gated
    #                       # behind off-by-default `uefildr-rust-entry`
    #                       # feature pending a future cohabitation flip.

if(ARCH STREQUAL "i386")
    # arch/i386/i386idt.c  # replaced by Rust (freeldr-rust/src/i386idt.rs)
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/i386/uefiasm.S
        arch/i386/i386trap.S)
elseif(ARCH STREQUAL "amd64")
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/amd64/uefiasm.S)
elseif(ARCH STREQUAL "arm64")
    list(APPEND UEFILDR_ARC_SOURCE
        # arch/arm64/debug.c   # replaced by Rust (freeldr-rust/src/arm64_debug.rs)
    )
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/arm64/uefiasm.S)
else()
    #TBD
endif()

list(APPEND UEFILDR_BOOTMGR_SOURCE
    ${FREELDR_BOOTMGR_SOURCE}
    # custom.c   — replaced by Rust (sdk/lib/rust/freeldr-rust/src/custom.rs)
    # options.c — replaced by Rust (sdk/lib/rust/freeldr-rust/src/options.rs)
    # oslist.c  — replaced by Rust (sdk/lib/rust/freeldr-rust/src/oslist.rs)
    # NOTE: rosload.cmake keeps its own C copies of these (rosload.exe does
    # not link libfreeldr_rust.a; cohabitation flip is a future task).
)

add_asm_files(uefifreeldr_common_asm ${FREELDR_COMMON_ASM_SOURCE} ${UEFILDR_COMMON_ASM_SOURCE})

add_library(uefifreeldr_common
    ${uefifreeldr_common_asm}
    ${UEFILDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${UEFILDR_BOOTMGR_SOURCE}
    ${FREELDR_NTLDR_SOURCE})

target_compile_definitions(uefifreeldr_common PRIVATE _FRLDRLIB_ UEFIBOOT)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Prevent using SSE (no support in freeldr) - x86/x64 only
    if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
        target_compile_options(uefifreeldr_common PUBLIC -mno-sse)
    endif()
endif()

set(PCH_SOURCE
    ${UEFILDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${UEFILDR_BOOTMGR_SOURCE}
    ${FREELDR_NTLDR_SOURCE})

add_pch(uefifreeldr_common include/arch/uefi/uefildr.h PCH_SOURCE)
add_dependencies(uefifreeldr_common bugcodes asm xdk)

# Rust peloader (replaces lib/peloader.c — see boot/freeldr/freeldr/CMakeLists.txt)
target_link_libraries(uefifreeldr_common ${FREELDR_RUST_UEFI_LIB})
add_dependencies(uefifreeldr_common rust_freeldr_uefi)

## GCC builds need this extra thing for some reason...
if(ARCH STREQUAL "i386" AND NOT MSVC)
    target_link_libraries(uefifreeldr_common mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(uefifreeldr_common rust_hal_mini)
endif()


# ============================================================================
# uefildr.efi — CARGO-DRIVEN final link (sdk/lib/rust/uefildr-cargo)
# ============================================================================
# The final loader link is driven by cargo (uefildr-cargo's build.rs →
# reactos-link, ModuleKind::EfiApplication) instead of cmake's add_executable.
# The cargo build REUSES the cmake-built libuefifreeldr_common.a +
# libfreeldr_rust_uefi.a + the leaf libs + the uefildr_shim object below;
# cmake's own loader link is retired. uefildr exports nothing
# (--exclude-all-symbols); the freeldr import lib rosload consumes still comes
# from CMakeLists.txt's `spec2def(freeldr.sys freeldr.spec ADD_IMPORTLIB)`, and
# rosload's freeldr.sys imports bind at runtime against uefildr's internal
# table (not PE exports), so no spec2def(uefildr.exe) / .def is needed here.

# strsafe_shim.c (the variadic RtlStringCb* shim Rust stable can't host) is now
# compiled directly by reactos-link via uefildr-cargo's .c_shims() — it is a
# self-contained leaf (zero #includes; it self-defines its types + externs only
# libcntpr's _vsnprintf), so it never needed freeldr's compile env. The old
# `uefildr_shim` OBJECT lib is retired; -mno-sse moves to the build.rs c_shim_flags.

# cmake targets whose .a/.obj the cargo link reuses — DEPENDS so `ninja
# uefildr` builds them before invoking cargo (an opaque link).
#
# CONTENT tracking, not just ordering: a DEPENDS on an add_library TARGET
# (uefifreeldr_common, cportlib, blcmlib, …) auto-tracks its output FILE, so a
# change relinks uefildr. But a DEPENDS on a custom-TARGET (rust_freeldr_uefi,
# rust_cnt, …) is only an order-only/phony edge — ninja will NOT relink uefildr
# when that target's .a content changes (it just ensures it built once). That
# left a real hole: editing freeldr-rust rebuilt libfreeldr_rust_uefi.a but
# `ninja uefildr` said "no work to do", silently booting a stale loader. Fix:
# also DEPENDS on the .a FILE PATHS produced by those custom targets (each is a
# custom_command OUTPUT, so ninja file-tracks it and relinks on mtime change).
# reactos-link already emits cargo:rerun-if-changed for each static_lib, so once
# ninja re-invokes cargo, the link actually re-runs.
set(UEFILDR_EXTRA_DEPENDS
    uefifreeldr_common
    cportlib blcmlib blrtl libcntpr chkstk
    # custom-target-produced rust .a link inputs — depend on the FILES:
    rust_freeldr_uefi ${FREELDR_RUST_UEFI_LIB}
    rust_cnt ${CMAKE_BINARY_DIR}/sdk/lib/crt/libcntpr_rust.a
    ${CMAKE_BINARY_DIR}/sdk/lib/cportlib/libcport_rust.a)
if(ARCH STREQUAL "i386")
    list(APPEND UEFILDR_EXTRA_DEPENDS
        mini_hal rust_hal_mini ${CMAKE_BINARY_DIR}/hal/halx86/libhal_rust_mini.a)
endif()

# KIND efi → post-link strips .rossym + --strip-all (matches the old
# add_executable POST_BUILD). NO_CD → uefildr.efi is embedded into the efisys
# boot image by boot/boot_images.cmake (which references its path directly),
# not add_cd_file'd.
add_cargo_module(uefildr uefildr-cargo uefildr.efi
    KIND efi NO_CD EXTRA_DEPENDS ${UEFILDR_EXTRA_DEPENDS})
