
list(APPEND HAL_GENERIC_SOURCE
    # generic/beep.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/beep.rs)
    # generic/cmos.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/cmos.rs)
    # generic/display.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/display.rs)
    # generic/dma.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/dma.rs)
    # generic/drive.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/drive.rs)
    # generic/halinit.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halinit.rs)
    # generic/kdpci.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/kdpci.rs)
    # generic/memory.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/memory.rs)
    # generic/misc.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/misc.rs)
    # generic/nmi.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/nmi.rs)
    # generic/pic.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pic.rs)
    # generic/reboot.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/reboot.rs)
    # generic/sysinfo.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/sysinfo.rs)
    # generic/usage.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/usage.rs)
    # generic/x86bios.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/x86bios.rs)
    )

if(ARCH STREQUAL "i386")
    # generic/portio.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/portio.rs)
    # generic/bios.c + v86.S removed: pre-Vista V86-mode INT10 BIOS path, dead
    # under UEFI. The fast486/x86bios emulation engine has been retired.
endif()

# On amd64 every generic .c is now Rust (hal-rust) and there's no
# generic asm — lib_hal_generic would be an empty OBJECT lib, which
# cmake rejects.  Drop in a placeholder .c so the target stays alive
# for add_hal(COMPONENTS ... generic ...).  i386 still has bios.c.
if(NOT HAL_GENERIC_SOURCE AND NOT HAL_GENERIC_ASM_SOURCE)
    set(HAL_GENERIC_STUB ${CMAKE_CURRENT_BINARY_DIR}/lib_hal_generic_empty.c)
    file(WRITE ${HAL_GENERIC_STUB} "/* placeholder: HAL generic sources are in hal-rust */\n")
    list(APPEND HAL_GENERIC_SOURCE ${HAL_GENERIC_STUB})
endif()

add_asm_files(lib_hal_generic_asm ${HAL_GENERIC_ASM_SOURCE})
add_library(lib_hal_generic OBJECT ${HAL_GENERIC_SOURCE} ${lib_hal_generic_asm})
add_dependencies(lib_hal_generic asm)
target_link_libraries(lib_hal_generic PRIVATE fast486)
