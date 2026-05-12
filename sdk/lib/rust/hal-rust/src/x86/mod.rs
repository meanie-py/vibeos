//! x86-shared backends — hardware register pokes for PC-AT class
//! hardware: 8259 PIC, LAPIC/IOAPIC, 8237 ISA DMA, CMOS/RTC, PIT,
//! port-I/O PCI config, real-mode BIOS thunks, etc.
//!
//! Compiled for both i386 and amd64 targets.

#[cfg(target_arch = "x86_64")]
pub mod apic_halinit;
pub mod beep;
pub mod buildtype;
#[cfg(target_arch = "x86_64")]
pub mod busemul;
pub mod clock;
pub mod cmos;
pub mod cmosbus;
pub mod display;
pub mod drive;
pub mod halinit;
#[cfg(target_arch = "x86")]
pub mod halpcat;
pub mod irql;
pub mod memory;
pub mod misc;
pub mod nmi;
pub mod pcr;
pub mod pic;
#[cfg(target_arch = "x86")]
pub mod pic_halinit;
pub mod processor;
pub mod reboot;
// profil only on i386 — amd64 has its own LAPIC-timer-based version
// in apic/apictimer.c (will translate later as part of the APIC backend).
#[cfg(target_arch = "x86")]
pub mod portio;
#[cfg(target_arch = "x86")]
pub mod profil;
pub mod spinlock;
pub mod sysbus;
pub mod sysinfo;
pub mod up;
pub mod usage;
