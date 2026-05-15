//! x86-shared backends — hardware register pokes for PC-AT class
//! hardware: 8259 PIC, LAPIC/IOAPIC, 8237 ISA DMA, CMOS/RTC, PIT,
//! port-I/O PCI config, real-mode BIOS thunks, etc.
//!
//! Compiled for both i386 and amd64 targets.

#[cfg(target_arch = "x86_64")]
pub mod apic;
#[cfg(target_arch = "x86_64")]
pub mod apic_halinit;
#[cfg(target_arch = "x86_64")]
pub mod apictimer;
pub mod beep;
#[cfg(target_arch = "x86")]
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
pub mod display;
pub mod dma;
pub mod drive;
#[cfg(target_arch = "x86_64")]
pub mod halacpi;
pub mod halinit;
#[cfg(target_arch = "x86")]
pub mod halpcat;
#[cfg(target_arch = "x86_64")]
pub mod halpnpdd;
pub mod irql;
pub mod isabus;
pub mod kdpci;
#[cfg(target_arch = "x86")]
pub mod legacy_halpnpdd;
#[cfg(target_arch = "x86_64")]
pub mod madt;
pub mod memory;
#[cfg(target_arch = "x86")]
pub mod minihal_stubs;
pub mod misc;
pub mod nmi;
pub mod pcibus;
pub mod pcr;
pub mod pic;
#[cfg(target_arch = "x86")]
pub mod pic_halinit;
#[cfg(target_arch = "x86")]
pub mod pic_pic;
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
#[cfg(target_arch = "x86_64")]
pub mod tsc;
pub mod up;
pub mod usage;
pub mod x86bios;
