//! ke/amd64 subsystem — translated from C.

pub mod context;
pub mod cpu;
pub mod except;
pub mod freeze;
pub mod interrupt;
pub mod ipi;
pub mod irql;
pub mod kiinit;
pub mod krnlinit;
pub mod mproc;
pub mod spinlock;
pub mod stubs;
pub mod thrdini;
pub mod traphandler;
// pub mod usercall; // Using C version — Rust version BSODs amd64 boot (verified 2026-05-01).
                       // Audit's "return-address corruption" excuse is current, not stale.
pub mod xstate;
