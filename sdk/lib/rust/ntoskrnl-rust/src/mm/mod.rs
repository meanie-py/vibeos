//! mm subsystem — translated from C.

// Tier 1+2
pub mod balance;
pub mod freelist;
pub mod marea;
pub mod mmfault;
pub mod mminit;
pub mod pagefile;
pub mod region;
pub mod rmap;
pub mod shutdown;
// Tier 3b
pub mod section;
