//! ARM3 memory manager subsystem — translated from C.

#[macro_use]
pub mod helpers;
// Tier 1+2
pub mod census;
pub mod commit;
pub mod contmem;
pub mod drvmgmt;
pub mod dynamic;
pub mod hypermap;
pub mod iosup;
pub mod kdbg;
pub mod largepag;
pub mod mmdbg;
pub mod mmsup;
pub mod ncache;
pub mod special;
pub mod syspte;
pub mod vadnode;
pub mod zeropage;
// Tier 3b
pub mod mdlsup;
pub mod mminit;
pub mod pagfault;
pub mod pfnlist;
pub mod procsup;
pub mod section;
pub mod session;
pub mod sysldr;
// virtual.rs — enabled on all 3 arches.
#[path = "virtual.rs"]
pub mod r#virtual;
// Tier 4
pub mod wslist;
