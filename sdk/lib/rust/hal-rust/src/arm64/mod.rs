//! ARM64 HAL translation modules.

pub mod acpi;
pub mod bits;
pub mod clock;
pub mod dma;
pub mod gic3;
pub mod halstubs;
pub mod iort;
pub mod its;
pub mod smmu;
pub mod pnpdd;
pub mod v2m;
// SMP AP-start via PSCI CPU_ON (smp-bringup-quest.md M5a) — SMP HAL only.
#[cfg(feature = "smp")]
pub mod mpboot;
