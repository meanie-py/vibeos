//! A tiny xorshift for shuffling.
//!
//! Self-contained rather than the CRT's `rand()`: the library is `no_std`, and
//! a shuffle that silently depends on whoever last called `srand()` is the kind
//! of shared global state that makes two games in one process interfere.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::sys;

static STATE: AtomicU32 = AtomicU32::new(0);

/// xorshift32. Full period over the non-zero words, which is plenty for
/// dealing cards and costs three shifts.
pub fn next() -> u32 {
    let mut x = STATE.load(Ordering::Relaxed);
    if x == 0 {
        // Seed lazily on first use, so the seed reflects when the player
        // actually started a game rather than when the module loaded.
        x = unsafe { sys::GetTickCount() } | 1;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    STATE.store(x, Ordering::Relaxed);
    x
}
