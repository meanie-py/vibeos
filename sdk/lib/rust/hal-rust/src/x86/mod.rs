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
// APIC interrupt-model backend (LAPIC/IOAPIC/TPR). The two interrupt flavors are
// written as cfg expressions throughout this crate (a build script used to emit
// them as `hal_apic`/`hal_pic`, which kept hal-rust out of the artifact store):
//   APIC: any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic"))
//   PIC:  all(target_arch = "x86", not(feature = "apic"))
// amd64 is always APIC; i386 is APIC with the `apic` feature (i386-apic-hal lane).
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
pub mod apic;
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
pub mod apic_halinit;
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
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
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
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
#[cfg(all(all(target_arch = "x86", not(feature = "apic")), not(feature = "minihal")))]
pub mod pic_halinit;
#[cfg(all(target_arch = "x86", not(feature = "apic")))]
pub mod pic_pic;
#[cfg(not(feature = "minihal"))]
pub mod processor;
pub mod reboot;
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
pub mod rtctimer;
// portio: i386 port-I/O PCI config — needed by BOTH flavors (bus/IO, not
// interrupt-controller), so it stays keyed on the arch, not the flavor.
#[cfg(target_arch = "x86")]
pub mod portio;
// profil: RTC-based profiling for the PIC flavor. The APIC flavor profiles via
// the LAPIC timer (apictimer), so this is hal_pic-only.
#[cfg(all(all(target_arch = "x86", not(feature = "apic")), not(feature = "minihal")))]
pub mod profil;
pub mod spinlock;
#[cfg(not(feature = "minihal"))]
pub mod sysbus;
#[cfg(not(feature = "minihal"))]
pub mod sysinfo;
// timer: the 8254 PIT system clock — PIC flavor only (and the FreeLDR mini-HAL,
// which is always hal_pic). The APIC flavor's clock is the RTC (rtctimer).
#[cfg(all(target_arch = "x86", not(feature = "apic")))]
pub mod timer;
#[cfg(any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")))]
pub mod tsc;
// SMP AP-start + real-IPI backend (smp-bringup-quest.md M2/M5). Keyed on the
// APIC flavor, not the arch: amd64 is always hal_apic, and i386 gets it on the
// (now default) APIC HAL — SMP requires the LAPIC/IOAPIC/MADT the PIC flavor
// lacks, so hal_pic stays UP. The colliding UP stubs in up.rs/processor.rs are
// cfg'd out under this same condition (their stub gate flips to hal_pic).
#[cfg(all(feature = "smp", any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")), not(feature = "minihal")))]
pub mod smp_processor;
#[cfg(all(feature = "smp", any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")), not(feature = "minihal")))]
pub(crate) mod trampoline;
pub mod up;
pub mod usage;
// x86bios / bios (INT10 + V86 real-mode BIOS emulation via fast486) removed:
// pre-Vista platform support, dead under UEFI. The only consumer (videoprt
// INT10) was stubbed and the legacy video miniports were dropped.
