pub mod adapter;
pub mod arcname;  // amd64 4/4; i386 blocked by pre-existing device.rs ud2 crash (not arcname)
pub mod bootlog;
pub mod controller;
pub mod debug;
pub mod device;
pub mod deviface;
pub mod driver;
pub mod error;
pub mod file;
pub mod iocomp;
pub mod io_tables;  // 8 Iop*Operation* lookup tables (replaces io_tables.c)
pub mod ioevent;
pub mod iofunc;  // Constants fixed, truncated (missing NtSetInformationFile+), C provides missing functions
pub mod iomdl;
pub mod iomgr;
pub mod iorsrce;
pub mod iotimer;
pub mod iowork;
pub mod irp;
pub mod irq;
pub mod ramdisk;
pub mod rawfs;
pub mod remlock;
pub mod symlink;
pub mod util;
pub mod volume;

// Folded back 2026-05-09 from per-subsystem rlibs (pilot validated
// the captures(none) firewall is sufficient).
pub mod inlines;
