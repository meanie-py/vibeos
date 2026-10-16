//! HAL processor helpers — UP-only entry points + the
//! active-processor / default-interrupt-affinity bitmaps the rest of
//! the HAL (halinit / pic.rs / apic) write into.
//!
//! Translated from `hal/halx86/pic/processor.c`.  Tiny leaf: 3 NTAPI
//! functions + 2 globals.  All bodies are one-liners that touch
//! interrupt state via `cli` / `sti` / `hlt`.

use ntkrnl_sys::hal_ddk::{BOOLEAN, KAFFINITY};

// ---------------------------------------------------------------------------
// Globals — referenced by halinit.rs (HalInitializeProcessor adds the
// boot processor's bit to each), pic.rs, apic, sysinfo.rs.  hal_bindgen
// already publishes `extern static mut` decls for both; the Rust
// definitions here resolve those externs at link time.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpActiveProcessors: KAFFINITY = 0;

#[unsafe(no_mangle)]
pub static mut HalpDefaultInterruptAffinity: KAFFINITY = 0;

// ---------------------------------------------------------------------------
// HaliHaltSystem — disable interrupts and stop the processor.  Used as
// the HalPrivateDispatchTable.HalHaltSystem entry (wired by halinit).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HaliHaltSystem() {
    unsafe {
        core::arch::asm!("cli; hlt", options(nomem, nostack));
    }
}

// ---------------------------------------------------------------------------
// HalAllProcessorsStarted — UP always returns TRUE (the boot processor
// IS all the processors).  hal.spec exports this stdcall.
// ---------------------------------------------------------------------------

// (smp APIC build: the MP-aware version in smp_processor.rs takes the symbol;
// hal_pic stays UP so it keeps this stub)
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllProcessorsStarted() -> BOOLEAN {
    1 /* TRUE */
}

// ---------------------------------------------------------------------------
// HalProcessorIdle — kernel idle loop calls this when there's nothing
// runnable.  Enable interrupts (so a timer / IPI / IRQ can wake us)
// then halt until the next interrupt.  hal.spec exports stdcall.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalProcessorIdle() {
    unsafe {
        core::arch::asm!("sti; hlt", options(nomem, nostack));
    }
}
