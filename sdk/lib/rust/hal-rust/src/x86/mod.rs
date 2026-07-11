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
// halacpi: ACPI HAL identity + table walker. Was x86_64-only; un-gated to
// x86 too for the i386 ACPI-HAL backport (task #4 — gives i386 ACPI\PNP0303
// PS/2 enumeration). Keeps PIC interrupt delivery (pic*/timer/portio stay).
#[cfg(not(feature = "minihal"))]
pub mod halacpi;
#[cfg(not(feature = "minihal"))]
pub mod halinit;
// halpcat: i386 PIC-HAL identity. Fully superseded by halacpi.rs on i386
// (every one of its 8 symbols is redefined there with ACPI-aware bodies —
// `comm -23` of the two is empty). Disabled on i386 now that halacpi is
// linked, mirroring the C model where legacy/halpcat.c and acpi/halacpi.c
// are never linked into the same HAL flavor. File kept (never-delete).
#[cfg(any())]
pub mod halpcat;
#[cfg(not(feature = "minihal"))]
pub mod halpnpdd;
pub mod irql;
pub mod isabus;
#[cfg(not(feature = "minihal"))]
pub mod kdpci;
// legacy_halpnpdd: PCI_HAL\PNP0A03 PnP root DD (non-ACPI). Clean swap-pair
// with halpnpdd (ACPI_HAL\PNP0C08) — identical symbol surface, different
// reported device IDs. Disabled on i386 in favour of halpnpdd so the HAL
// root advertises ACPI_HAL → PnP loads acpi.sys as root bus → DSDT walk →
// ACPI\PNP0303. File kept (never-delete). Task #4.
#[cfg(any())]
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
// SMP AP-start + real-IPI backend (smp-bringup-quest.md M2). amd64-only —
// i386 runs the PIC HAL (no LAPIC/MADT); its SMP story is M5. The colliding
// UP stubs in up.rs/processor.rs are cfg'd out under this same condition.
#[cfg(all(feature = "smp", target_arch = "x86_64", not(feature = "minihal")))]
pub mod smp_processor;
#[cfg(all(feature = "smp", target_arch = "x86_64", not(feature = "minihal")))]
pub(crate) mod trampoline;
pub mod up;
pub mod usage;
// x86bios / bios (INT10 + V86 real-mode BIOS emulation via fast486) removed:
// pre-Vista platform support, dead under UEFI. The only consumer (videoprt
// INT10) was stubbed and the legacy video miniports were dropped.
