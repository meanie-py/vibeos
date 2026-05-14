//! x86-shared backends — hardware register pokes for PC-AT class
//! hardware: 8259 PIC, LAPIC/IOAPIC, 8237 ISA DMA, CMOS/RTC, PIT,
//! port-I/O PCI config, real-mode BIOS thunks, etc.
//!
//! Compiled for both i386 and amd64 targets.

#[cfg(target_arch = "x86_64")]
pub mod apic_halinit;
#[cfg(target_arch = "x86_64")]
pub mod apictimer;
pub mod beep;
pub mod buildtype;
#[cfg(target_arch = "x86_64")]
pub mod busemul;
#[cfg(target_arch = "x86")]
pub mod bushndlr;
#[cfg(target_arch = "x86")]
pub mod bussupp;
#[cfg(target_arch = "x86")]
pub mod legacy_halpnpdd;
#[cfg(target_arch = "x86")]
pub mod pcibus;
pub mod clock;
pub mod cmos;
pub mod cmosbus;
pub mod display;
pub mod dma;
pub mod drive;
pub mod halinit;
#[cfg(target_arch = "x86")]
pub mod halpcat;
pub mod irql;
pub mod isabus;
pub mod kdpci;
#[cfg(target_arch = "x86_64")]
pub mod madt;
pub mod memory;
pub mod misc;
pub mod nmi;
pub mod pcr;
pub mod pic;
#[cfg(target_arch = "x86")]
pub mod pic_halinit;
// pub mod pic_pic; — compile-clean checkpoint (cascade fix 100→0).  IRQL
// primitives translated correctly (KeGetCurrentIrql / KfRaiseIrql / KfLowerIrql
// from pic.c:309-455).  Integration blocked on:
//   1. KiUserTrap / KiEnterInterruptTrap are FORCEINLINE in
//      ntoskrnl/include/internal/i386/trap_x.h:18,370 — not exported by
//      ntoskrnl.spec — must be inlined manually in Rust (KiUserTrap is one
//      line; KiEnterInterruptTrap touches PCR.NtTib.ExceptionList +
//      KiHandleDebugRegistersOnTrapEntry + KiFillTrapFrameDebug).
//   2. HalpHardwareInterrupt[0-15] (16 IRQ stubs) are defined in pic.c via
//      a `HalpDelayedHardwareInterrupt(N)` macro (~13K bytes total) — need
//      to be ported or kept C-side via partial removal of pic.c.
//   3. SWInterruptHandlerTable[20] / SWInterruptHandlerTable2[3] / Hal*Eisa*
//      / HalpDefaultInterruptAffinity definitions live in pic.c — must move
//      to Rust when pic.c is removed.
// See docs/engineering/handoff-pic_pic-integration.md for the full plan.
pub mod processor;
pub mod reboot;
#[cfg(target_arch = "x86_64")]
pub mod rtctimer;
// profil only on i386 — amd64 has its own LAPIC-timer-based version
// in apic/apictimer.c (will translate later as part of the APIC backend).
#[cfg(target_arch = "x86")]
pub mod portio;
#[cfg(target_arch = "x86")]
pub mod profil;
pub mod spinlock;
pub mod sysbus;
pub mod sysinfo;
#[cfg(target_arch = "x86")]
pub mod timer;
pub mod up;
pub mod usage;
