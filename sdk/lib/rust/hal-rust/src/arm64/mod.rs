//! ARM64 HAL translation modules.

pub mod clock;
pub mod dma;
pub mod halstubs;
// SMP AP-start via PSCI CPU_ON (smp-bringup-quest.md M5a) — SMP HAL only.
#[cfg(feature = "smp")]
pub mod mpboot;
