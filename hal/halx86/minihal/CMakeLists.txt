
# We need to define this here, because the target definitions are not
# passed to CL, when preprocessing the asm files on MSVC builds.
# See CORE-19847
add_definitions(-D_MINIHAL_)

list(APPEND MINI_HAL_SOURCE
    # ../legacy/bus/bushndlr.c — Rust takeover (minihal feature)
    # ../legacy/bus/cmosbus.c — Rust takeover (hal-rust mini feature)
    # ../legacy/bus/isabus.c — Rust takeover
    # ../legacy/bus/pcibus.c — Rust takeover (minihal feature)
    # ../legacy/bussupp.c — Rust takeover (minihal feature)
    # ../generic/dma.c — entire file is #ifndef _MINIHAL_; compiles to nothing
    # ../generic/drive.c — Rust takeover
    # ../generic/misc.c — Rust takeover (minihal feature)
    # ../generic/nmi.c — Rust takeover
    # ../generic/spinlock.c — Rust takeover (minihal feature)
    # ../generic/timer.c — Rust takeover (minihal feature)
    # ../generic/usage.c — Rust takeover (minihal feature)
    # ../pic/pic.c — only a forward decl of HalpEndSoftwareInterrupt under MINIHAL
    # halinit.c — disabled (kept on disk); the 7 mini-HAL init stubs
    #   (HalpInitProcessor, HalpInitPhase0/1, HalpSetupAcpiPhase0,
    #   HalpInitializePICs, HalpGetDmaAdapter, HalpBiosDisplayReset) now live in
    #   sdk/lib/rust/hal-rust/src/x86/minihal_stubs.rs (cfg feature=minihal).
    ../include/hal.h
    )

# PC SARCH only — xbox / pc-98 SARCH removed.
# All beep / clock / cmos / reboot / init-stub symbols are provided by
# libhal_rust_mini.a — the only C contribution to mini_hal is the
# ../generic/systimer.S assembly file.

# systimer.S is the ONLY C-side contribution left to mini_hal; without it the target
# has no compilable source and cmake cannot even pick a linker language. Upstream
# restructured minihal and dropped this line in the merge.
add_asm_files(mini_hal_asm ../generic/systimer.S)
add_library(mini_hal ${MINI_HAL_SOURCE} ${mini_hal_asm})
add_pch(mini_hal ../include/hal.h MINI_HAL_SOURCE)
target_compile_definitions(mini_hal PRIVATE _BLDR_ _NTSYSTEM_)
add_dependencies(mini_hal psdk bugcodes asm)
