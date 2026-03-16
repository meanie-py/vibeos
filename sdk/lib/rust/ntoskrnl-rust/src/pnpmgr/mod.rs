// Phase 1: leaf files with no inter-pnpmgr dependencies — smoke tested
pub mod pnputil;
pub mod pnpirp;

// Disabled: runtime bugs when enabled
// pnpnotify: Rust version disabled on i386 due to undiagnosed runtime crash
// (BUGCHECK 0x1E STATUS_IN_PAGE_ERROR at KeSynchronizeExecution during scsiport init).
// Root cause: not calling convention or code logic — pure Rust wrapper around C impl works,
// full Rust code crashes. Crash is in unrelated KeSynchronizeExecution, suggesting subtle
// interaction with kernel image layout or BSS/data section handling on i386.
// amd64 and arm64 use the Rust version.
#[cfg(not(target_arch = "x86"))]
pub mod pnpnotify;
pub mod pnpdma;
// pub mod devnode;
// pub mod plugplay;
// pub mod pnpinit;
// pub mod pnpmap;
// pub mod pnpmgr;
// pub mod pnpreport;
// pub mod pnpres;
// pub mod pnproot;
// pub mod devaction;
// pub mod arb;
