
list(APPEND HAL_PIC_ASM_SOURCE
    generic/systimer.S
    generic/trap.S
    pic/pic.S)

list(APPEND HAL_PIC_SOURCE
    # generic/clock.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/clock.rs)
    # generic/profil.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/profil.rs)
    generic/timer.c
    # pic/halinit.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pic_halinit.rs)
    # pic/irql.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/irql.rs)
    pic/pic.c
    # pic/processor.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/processor.rs)
    )

add_asm_files(lib_hal_pic_asm ${HAL_PIC_ASM_SOURCE})
add_library(lib_hal_pic OBJECT ${HAL_PIC_SOURCE} ${lib_hal_pic_asm})
add_dependencies(lib_hal_pic asm)
