##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for UEFI
## COPYRIGHT:   Copyright 2023 Justin Miller <justinmiller100@gmail.com>
##

include_directories(BEFORE
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/edk2
    include
    include/arch/uefi)

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

# ============================================================================
# `uefifreeldr_common` is RETIRED.
# ============================================================================
# It was the freeldr C+asm aggregate library uefildr.efi linked. Every freeldr
# C unit is now Rust (UEFILDR_ARC/BOOTLIB/BOOTMGR/NTLDR are all empty/Rust
# above), and its only remaining content — the per-arch asm (uefiasm.S +
# i386:i386trap.S + amd64:misc.S) — is now assembled DIRECTLY by reactos-link
# from uefildr-cargo's build.rs `.asm()` (those files only ever needed asm.inc,
# resolved via a minimal 2-dir include set, not freeldr's 30-include env). With
# nothing left to compile, the aggregate library is gone: uefildr-cargo links
# libfreeldr_rust_uefi.a + the shared leaf libs + (i386) mini_hal directly, and
# the reactos-link-assembled asm objects replace libuefifreeldr_common.a.
# (BIOS freeldr.sys was pruned, so nothing else consumed this asm.)


# ============================================================================
# uefildr.efi — CARGO-DRIVEN final link (sdk/lib/rust/uefildr-cargo)
# ============================================================================
# The final loader link is driven by cargo (uefildr-cargo's build.rs →
# reactos-link, ModuleKind::EfiApplication) instead of cmake's add_executable.
# reactos-link builds the uefildr-specific parts from source (freeldr-rust body
# via libfreeldr_rust_uefi.a, strsafe_shim.c via c_shims, the per-arch asm via
# asm) and links the shared cmake leaf libs; the old uefifreeldr_common
# aggregate + uefildr_shim OBJECT lib are retired. uefildr exports nothing
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
    libcntpr chkstk
    # blcmlib compiled in-cargo (c_from_compile_db) — file-track its C sources
    # instead of depending on the cmake blcmlib target:
    ${FREELDR_BLCMLIB_SOURCES}
    # blrtl likewise compiled in-cargo (c_from_compile_db C + asm_archive asm) —
    # file-track its sources instead of depending on the cmake blrtl target:
    ${FREELDR_BLRTL_SOURCES}
    # source files reactos-link now compiles directly (file-track so `ninja
    # uefildr` re-invokes cargo when they change — reactos-link's per-input
    # cargo:rerun-if-changed then re-runs the link). uefiasm.S/i386trap.S/misc.S
    # are no longer here: flipped to Rust global_asm! (freeldr-rust/uefiasm.rs +
    # loader_asm.rs), tracked via the BODY_CRATE glob below.
    ${CMAKE_CURRENT_SOURCE_DIR}/lib/strsafe_shim.c
    # freeldr-rust body is now built by uefildr-cargo itself (reactos-link
    # .rust_staticlib) — tracked via add_cargo_module's BODY_CRATE below, not a
    # custom-target .a. cmboot-rust is a path-dep of freeldr-rust → file-track
    # its sources so an edit re-invokes the cargo link.
    ${FREELDR_CMBOOT_RUST_SOURCES}
    # cnt-rust-lib is now built by uefildr-cargo itself (reactos-link .rust_lib);
    # file-track its sources (incl. cnt-rust/cnt-ctype path-deps) so an edit
    # re-invokes the cargo link, instead of depending on the cmake libcntpr_rust.a.
    ${FREELDR_CNT_RUST_SOURCES}
    # cport-rust is now built by uefildr-cargo itself (reactos-link .rust_lib);
    # file-track its sources. libcportlib.a is no longer linked (it bundled a
    # redundant second copy of cport_rust), so the cmake `cportlib` target dep
    # is dropped too.
    ${FREELDR_CPORT_RUST_SOURCES})
if(ARCH STREQUAL "i386")
    # mini_hal stays a dep (systimer.S asm → libmini_hal.a, still reused). Its
    # Rust impl (hal-rust minihal) is now built by uefildr-cargo (reactos-link
    # .rust_lib) — file-track its sources instead of the cmake libhal_rust_mini.a.
    list(APPEND UEFILDR_EXTRA_DEPENDS
        mini_hal ${FREELDR_HAL_MINI_SOURCES})
endif()

# KIND efi → post-link strips .rossym + --strip-all (matches the old
# add_executable POST_BUILD). NO_CD → uefildr.efi is embedded into the efisys
# boot image by boot/boot_images.cmake (which references its path directly),
# not add_cd_file'd. BODY_CRATE freeldr-rust → glob its sources for the rebuild
# trigger (the body is built inside the cargo link now).
add_cargo_module(uefildr uefildr-cargo uefildr.efi
    KIND efi NO_CD BODY_CRATE freeldr-rust EXTRA_DEPENDS ${UEFILDR_EXTRA_DEPENDS})
