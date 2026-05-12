//! PIT rollover table — 15 entries spanning 1 ms (1197 ticks of the
//! 1.193182 MHz PIT) through 15 ms (17892 ticks).  Indexed by
//! `HalpLargestClockMS - 1` in HalpInitializeClock and by
//! `HalpNextMSRate - 1` in HalSetTimeIncrement / the PIT clock ISR.
//!
//! Translated from `hal/halx86/generic/clock.c` — pure static data, no
//! functions.  Each entry is `(RollOver, Increment)` where Increment
//! is in 100-ns units; e.g. `(1197, 10032)` = 1197 PIT counts ≈ 1.003 ms.

use ntkrnl_sys::hal_ddk::HALP_ROLLOVER;

#[unsafe(no_mangle)]
pub static HalpRolloverTable: [HALP_ROLLOVER; 15] = [
    HALP_ROLLOVER { RollOver: 1197,  Increment: 10032  }, // 1 ms
    HALP_ROLLOVER { RollOver: 2394,  Increment: 20064  },
    HALP_ROLLOVER { RollOver: 3591,  Increment: 30096  },
    HALP_ROLLOVER { RollOver: 4767,  Increment: 39952  },
    HALP_ROLLOVER { RollOver: 5964,  Increment: 49984  },
    HALP_ROLLOVER { RollOver: 7161,  Increment: 60016  },
    HALP_ROLLOVER { RollOver: 8358,  Increment: 70048  },
    HALP_ROLLOVER { RollOver: 9555,  Increment: 80080  },
    HALP_ROLLOVER { RollOver: 10731, Increment: 89936  },
    HALP_ROLLOVER { RollOver: 11949, Increment: 100144 },
    HALP_ROLLOVER { RollOver: 13125, Increment: 110000 },
    HALP_ROLLOVER { RollOver: 14322, Increment: 120032 },
    HALP_ROLLOVER { RollOver: 15519, Increment: 130064 },
    HALP_ROLLOVER { RollOver: 16695, Increment: 139920 },
    HALP_ROLLOVER { RollOver: 17892, Increment: 149952 }, // 15 ms
];
