
list(APPEND HAL_GENERIC_SOURCE
    # generic/beep.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/beep.rs)
    generic/cmos.c
    # generic/display.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/display.rs)
    generic/dma.c
    # generic/drive.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/drive.rs)
    # generic/halinit.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halinit.rs)
    generic/kdpci.c
    generic/memory.c
    generic/misc.c
    # generic/nmi.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/nmi.rs)
    # generic/pic.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pic.rs)
    # generic/reboot.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/reboot.rs)
    # generic/sysinfo.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/sysinfo.rs)
    generic/usage.c
    generic/x86bios.c)

if(ARCH STREQUAL "i386")
    list(APPEND HAL_GENERIC_SOURCE
        generic/bios.c)
        # generic/portio.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/portio.rs)

    list(APPEND HAL_GENERIC_ASM_SOURCE
        generic/v86.S)
endif()

add_asm_files(lib_hal_generic_asm ${HAL_GENERIC_ASM_SOURCE})
add_library(lib_hal_generic OBJECT ${HAL_GENERIC_SOURCE} ${lib_hal_generic_asm})
add_dependencies(lib_hal_generic asm)
