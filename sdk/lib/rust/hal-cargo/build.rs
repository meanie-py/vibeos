//! Cargo-driven `hal.dll` link — the UNIPROCESSOR HAL, paired with
//! `ntoskrnl.exe`.
//!
//! The whole recipe lives in the shared `hal-link` crate so this cell and
//! `halmp-cargo` (halmp.dll, the `smp`-feature twin) cannot drift: they differ
//! by exactly one cargo feature and the output name. See `hal-link/src/lib.rs`
//! for the per-arch link inputs, the CRT floor, and the boot-time `/HAL=`
//! selection mechanism.

fn main() {
    hal_link::link(hal_link::Flavor::Up);
}
