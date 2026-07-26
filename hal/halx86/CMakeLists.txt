
add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# ============================================================================
# hal.dll is CARGO-DRIVEN (sdk/lib/rust/hal-cargo) — see the link block below.
# ============================================================================
# The HAL is fully translated to Rust (sdk/lib/rust/hal-rust). The runtime
# hal.dll link, and hal-rust itself, are now built by the cargo link
# (add_cargo_kernel_dll → reactos-link, ModuleKind::KernelDll); FreeLDR's
# loader-time mini-HAL Rust is built by the loaders' own cargo links
# (.rust_lib("hal-rust", ["minihal"])). So the former cmake scaffolding —
# the `rust_hal` / `rust_hal_mini` custom_commands (built hal-rust on the
# standard user target), the `add_hal` function, and the per-flavor
# lib_hal_<component> OBJECT libs (generic/acpi/apic/legacy/pic.cmake) — is
# all retired: nothing consumed it after the cutover + the hal-mini flip.
# (Those component .cmake files remain on disk as cohabitation history.)
#
# `mini_hal` (minihal/, the systimer.S asm half of FreeLDR's mini-HAL) stays —
# the freeldr loaders still reuse libmini_hal.a.

if(ARCH STREQUAL "i386")
    add_subdirectory(minihal)

    # i386 hal.dll link inputs the cargo build reuses beyond the common leaf
    # libs: libchkstk.a + the cnt-rust cohabitation twin libcntpr_rust.a (the
    # cnt-rust MATH-TIERC flip moved strnlen/wcsnlen into the Rust twin, so the
    # C liblibcntpr.a references them undefined — same as amd64); the self-hosted
    # pic-flavor asm (file-track so an edit re-invokes the opaque cargo link);
    # and the checked-in pic/halpic.rc resource.
    # File-track the self-hosted flavor asm so an edit re-invokes the opaque
    # cargo link. The DEFAULT i386 flavor is now APIC (apic/apictrap.S +
    # apic/tsccal.S); the legacy PIC escape hatch (-DVIBEOS_PIC_HAL=ON) assembles
    # a different set (generic/systimer.S + generic/trap.S + pic/pic.S) — track
    # THAT set instead, matching hal-cargo/build.rs's VIBEOS_PIC_HAL asm switch.
    if(VIBEOS_PIC_HAL)
        set(_HAL_I386_FLAVOR_ASM
            ${CMAKE_CURRENT_SOURCE_DIR}/generic/systimer.S
            ${CMAKE_CURRENT_SOURCE_DIR}/generic/trap.S
            ${CMAKE_CURRENT_SOURCE_DIR}/pic/pic.S)
    else()
        set(_HAL_I386_FLAVOR_ASM
            ${CMAKE_CURRENT_SOURCE_DIR}/apic/apictrap.S
            ${CMAKE_CURRENT_SOURCE_DIR}/apic/tsccal.S)
    endif()
    set(HAL_CARGO_EXTRA_DEPENDS
        chkstk
        rust_cnt ${CMAKE_BINARY_DIR}/sdk/lib/crt/libcntpr_rust.a
        ${_HAL_I386_FLAVOR_ASM}
        ${CMAKE_SOURCE_DIR}/sdk/lib/rust/hal-cargo/data/hal-i386.rc.res)

elseif(ARCH STREQUAL "amd64")

    # amd64 hal.dll link adds the cnt-rust cohabitation twin (libcntpr_rust.a,
    # custom target rust_cnt) and the apic-flavor asm; no chkstk, no resource.
    set(HAL_CARGO_EXTRA_DEPENDS
        rust_cnt ${CMAKE_BINARY_DIR}/sdk/lib/crt/libcntpr_rust.a
        ${CMAKE_CURRENT_SOURCE_DIR}/apic/apictrap.S
        ${CMAKE_CURRENT_SOURCE_DIR}/apic/tsccal.S)

endif()

# ============================================================================
# hal.dll — CARGO-DRIVEN link (sdk/lib/rust/hal-cargo)
# ============================================================================
# The final hal.dll link is driven by cargo (hal-cargo's build.rs → reactos-link,
# ModuleKind::KernelDll). The cargo build reuses the cmake-built leaf libs
# (cntpr/setjmp/arbiter/pseh/memcmp + the per-arch extras above), self-hosts the
# HAL asm + the ntoskrnl import lib + (i386) the resource, and builds hal-rust
# itself (BODY_CRATE, on the i686/x86_64-kernel target). There is exactly ONE
# hal.dll (the halacpi/halapic/halaacpi names are install aliases).
# spec2def(... ADD_IMPORTLIB) stays — ntoskrnl/ntkrnlmp import hal's exports.
spec2def(hal.dll ../hal.spec ADD_IMPORTLIB)

# Common leaf-lib targets (file-tracked via the target) + the spec inputs +
# asm/psdk/bugcodes/xdk so the generated asm includes (ks*.inc / sysfuncs.h)
# exist before the cargo link assembles the HAL .S.
list(APPEND HAL_CARGO_EXTRA_DEPENDS
    libcntpr setjmp arbiter ${PSEH_LIB} memcmp
    asm psdk bugcodes xdk
    ${CMAKE_CURRENT_SOURCE_DIR}/../hal.spec
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ntoskrnl.spec)

add_cargo_kernel_dll(hal hal-cargo hal.dll
    BODY_CRATE hal-rust DESTINATION reactos/system32
    EXTRA_DEPENDS ${HAL_CARGO_EXTRA_DEPENDS})

# ============================================================================
# halmp.dll — the SMP HAL twin (hal-rust `smp` feature)
# ============================================================================
# Same recipe, same inputs, one cargo feature apart: halmp-cargo's build.rs is
# three lines over the SHARED `hal-link` crate that hal-cargo also drives, so
# the two flavors cannot drift. It pairs with ntkrnlmp.exe (HalpBuildType drops
# PRCB_BUILD_UNIPROCESSOR, so the x86 MISMATCHED_HAL 0x79 handshake accepts the
# MP kernel) and freeldr selects it per boot entry — livecd.ini `[LiveCD_SMP]`
# passes `/HAL=halmp.dll`, and winldr.c registers whatever file it loads under
# the name "hal.dll" in the loader block, so every import still resolves.
#
# Shipping both is what makes an MP gate leg affordable: switching flavors used
# to mean a cmake reconfigure + full HAL relink (measured 9m25s i386 / 12m22s
# amd64 warm) because the flavor was a build-dir-global cache var. Now it is one
# extra boot off the same media, and the mis-pair class (0x79 on x86, silent
# 1-cpu on arm64) is unreachable by construction.
#
# NO spec2def here: halmp.dll exports the identical hal.spec surface, and its
# .def is self-hosted by the cargo link. The cmake ADD_IMPORTLIB above is what
# ntoskrnl/ntkrnlmp link against, and there is only ever one of those.
add_cargo_kernel_dll(halmp halmp-cargo halmp.dll
    BODY_CRATE hal-rust DESTINATION reactos/system32
    EXTRA_DEPENDS ${HAL_CARGO_EXTRA_DEPENDS})
