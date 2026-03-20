pub mod adapter;
// pub mod arcname;  // Passes amd64 but crashes i386 — needs i386-specific debugging
pub mod bootlog;
pub mod controller;
pub mod debug;
pub mod device;
pub mod deviface;
pub mod driver;
pub mod error;
pub mod file;
pub mod iocomp;
pub mod ioevent;
// pub mod iofunc;  // PE binary layout issue: adding to Rust lib hangs ksecdd.sys even with C functions winning
pub mod iomdl;
// pub mod iomgr;  // Testing: crashes/hangs on both i386 and amd64
pub mod iorsrce;
pub mod iotimer;
pub mod iowork;
// pub mod irp;  // 4 bugs fixed (NT_ERROR, KTHREAD, PacketType, IsIrpSynchronous), still hangs — IopCompleteRequest verified correct, bug elsewhere in irp.rs
pub mod irq;
#[cfg(not(target_arch = "aarch64"))]
pub mod ramdisk;  // ARM64: binary layout change causes 0x6B crash (not a translation bug)
pub mod rawfs;
pub mod remlock;
pub mod symlink;
pub mod util;
pub mod volume;
