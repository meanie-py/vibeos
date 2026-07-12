//! ole32 Lane B — the ORPC/marshaling cluster (marshal.c, stubmanager.c,
//! rpc.c, ftmarshal.c, git.c), translated as one unit: the five TUs
//! inter-call by symbol and share the compobj_private.h layouts mirrored in
//! `types` (which must stay layout-identical to `super::compobj::types` —
//! the lanes were translated against the same C header; integration keeps
//! both mirrors and they interoperate only through raw pointers across the
//! FFI symbol boundary, exactly like the C TUs did).
//!
//! Ledger: ~/vibeos-rust-staging/ole32-lane-b/MANIFEST.md.

pub mod types;
pub mod marshal;
pub mod stubmanager;
pub mod rpc;
pub mod ftmarshal;
pub mod git;
