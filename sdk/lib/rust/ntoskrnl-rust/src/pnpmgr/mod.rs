// Phase 1: leaf files with no inter-pnpmgr dependencies — smoke tested
pub mod pnputil;
pub mod pnpirp;

// Disabled: runtime bugs when enabled
// All remaining pnpmgr files pass amd64 but crash i386 — systematic issue
// pub mod pnpnotify;
// pub mod pnpmap;
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
