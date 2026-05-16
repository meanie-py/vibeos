//! x86-shared backends — hardware register pokes for PC-AT class
//! hardware: 8259 PIC, LAPIC/IOAPIC, 8237 ISA DMA, CMOS/RTC, PIT,
//! port-I/O PCI config, real-mode BIOS thunks, etc.
//!
//! Compiled for both i386 and amd64 targets.

// FreeLDR `--features minihal` keep-set mirrors the C
// hal/halx86/minihal/CMakeLists.txt MINI_HAL_SOURCE list (beep, bushndlr,
// bussupp, clock, cmos, cmosbus, drive, isabus, misc, nmi, pcibus,
// reboot, spinlock, timer, usage) + loader essentials (minihal_stubs ←
// halinit, pic_pic ← pic minihal stubs, irql, portio, buildtype, up,
// pcr, dma per-item-gated). Every other module is full-HAL and is
// `#[cfg(not(feature = "minihal"))]` so its kernel deps stay out of
// libhal_rust_mini.a. feedback_freeldr_clean_build_regression
#[cfg(target_arch = "x86_64")]
pub mod apic;
#[cfg(target_arch = "x86_64")]
pub mod apic_halinit;
#[cfg(target_arch = "x86_64")]
pub mod apictimer;
pub mod beep;
#[cfg(all(target_arch = "x86", not(feature = "minihal")))]
pub mod bios;
pub mod buildtype;
#[cfg(target_arch = "x86_64")]
pub mod busemul;
#[cfg(target_arch = "x86")]
pub mod bushndlr;
#[cfg(target_arch = "x86")]
pub mod bussupp;
pub mod clock;
pub mod cmos;
pub mod cmosbus;
#[cfg(not(feature = "minihal"))]
pub mod display;
pub mod dma;
pub mod drive;
#[cfg(target_arch = "x86_64")]
pub mod halacpi;
#[cfg(not(feature = "minihal"))]
pub mod halinit;
#[cfg(all(target_arch = "x86", not(feature = "minihal")))]
pub mod halpcat;
#[cfg(target_arch = "x86_64")]
pub mod halpnpdd;
pub mod irql;
pub mod isabus;
#[cfg(not(feature = "minihal"))]
pub mod kdpci;
#[cfg(all(target_arch = "x86", not(feature = "minihal")))]
pub mod legacy_halpnpdd;
#[cfg(target_arch = "x86_64")]
pub mod madt;
#[cfg(not(feature = "minihal"))]
pub mod memory;
#[cfg(target_arch = "x86")]
pub mod minihal_stubs;
pub mod misc;
pub mod nmi;
pub mod pcibus;
#[cfg(target_arch = "x86")]
pub mod pcidata;
pub mod pcr;
pub mod pic;
#[cfg(all(target_arch = "x86", not(feature = "minihal")))]
pub mod pic_halinit;
#[cfg(target_arch = "x86")]
pub mod pic_pic;
#[cfg(not(feature = "minihal"))]
pub mod processor;
pub mod reboot;
#[cfg(target_arch = "x86_64")]
pub mod rtctimer;
// profil only on i386 — amd64 has its own LAPIC-timer-based version
// in apic/apictimer.c (will translate later as part of the APIC backend).
#[cfg(target_arch = "x86")]
pub mod portio;
#[cfg(all(target_arch = "x86", not(feature = "minihal")))]
pub mod profil;
pub mod spinlock;
#[cfg(not(feature = "minihal"))]
pub mod sysbus;
#[cfg(not(feature = "minihal"))]
pub mod sysinfo;
#[cfg(target_arch = "x86")]
pub mod timer;
#[cfg(target_arch = "x86_64")]
pub mod tsc;
pub mod up;
pub mod usage;
// x86bios was UNCONDITIONAL (i386 AND amd64) — only gate out of minihal,
// do NOT add target_arch (amd64 hal.dll needs x86BiosCall/AllocateBuffer).
#[cfg(not(feature = "minihal"))]
pub mod x86bios;
