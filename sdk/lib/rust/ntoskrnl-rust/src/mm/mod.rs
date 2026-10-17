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
pub mod standby;
// Tier 3b
pub mod section;
pub mod section_pe;
// Section I/O + page-table — translated from cache/section/{io,sptab}.c.
pub mod section_io;
pub mod section_table;
pub mod pte_atomic_inlines;
// Folded back 2026-05-09 from per-subsystem rlibs (pilot validated
// the captures(none) firewall is sufficient).
pub mod addr_space_inlines;
pub mod avl;
pub mod lock_inlines;
pub mod macros;
pub mod pte_class_inlines;
pub mod pte_mut_inlines;
pub mod util_inlines;
pub mod ws_lock_inlines;
