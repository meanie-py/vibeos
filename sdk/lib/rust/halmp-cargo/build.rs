//! Cargo-driven `halmp.dll` link — the SMP HAL, paired with `ntkrnlmp.exe`.
//!
//! Mirrors `ntoskrnl-cargo` → `ntkrnlmp-cargo`: the same body built with the
//! MP cfg, shipped as a second binary rather than swapped in behind a cmake
//! cache var. That is what lets ONE media boot either flavor — freeldr's
//! `/HAL=halmp.dll` (livecd.ini `[LiveCD_SMP]`) instead of a full reconfigure +
//! HAL relink, which used to cost 9-12 minutes per flavor switch and made an
//! MP gate leg unaffordable.
//!
//! The whole recipe lives in the shared `hal-link` crate — see
//! `hal-link/src/lib.rs`.

fn main() {
    hal_link::link(hal_link::Flavor::Mp);
}
