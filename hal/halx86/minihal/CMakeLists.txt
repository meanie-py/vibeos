
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
    ../include/hal.h
    halinit.c)

if(SARCH STREQUAL "xbox")
    list(APPEND MINI_HAL_SOURCE
        ../generic/beep.c
        ../generic/cmos.c
        ../xbox/clock.c
        ../xbox/reboot.c)
elseif(SARCH STREQUAL "pc98")
    list(APPEND MINI_HAL_SOURCE
        ../pc98/beep.c
        ../pc98/clock.c
        ../pc98/cmos.c
        ../pc98/reboot.c)
else()
    list(APPEND MINI_HAL_SOURCE
        # ../generic/beep.c — Rust takeover
        # ../generic/clock.c — Rust takeover (data-only: HalpRolloverTable)
        # ../generic/cmos.c — Rust takeover
        # ../generic/reboot.c — Rust takeover (minihal feature)
    )
endif()

add_asm_files(mini_hal_asm ../generic/systimer.S)
add_library(mini_hal ${MINI_HAL_SOURCE} ${mini_hal_asm})
target_compile_definitions(mini_hal PRIVATE _BLDR_ _NTSYSTEM_)
add_dependencies(mini_hal psdk bugcodes asm)
add_pch(mini_hal ../include/hal.h MINI_HAL_SOURCE)
