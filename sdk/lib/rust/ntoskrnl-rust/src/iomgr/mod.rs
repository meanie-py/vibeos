pub mod adapter;
// pub mod arcname;  // Passes amd64 but crashes i386 — needs i386-specific debugging
pub mod bootlog;
pub mod controller;
pub mod debug;
// pub mod device;  // LLVM optimizer bug: eliminates success path after ObCreateObject output-pointer pattern
pub mod deviface;
#[cfg(not(target_arch = "aarch64"))]
pub mod driver;
pub mod error;
// pub mod file;  // Crashes both arches — agent-translated NtCreateFile etc. have bugs
pub mod iocomp;
pub mod ioevent;
// pub mod iofunc;  // Nt* syscalls translated but buggy (SMSS crash, DbgPrint failed)
pub mod iomdl;
// pub mod iomgr;  // Retranslated but still hangs at mountmgr.sys on amd64
pub mod iorsrce;
pub mod iotimer;
pub mod iowork;
// pub mod irp;  // Causes i386+amd64 boot hang (BUGCHECK 0x1E in KiInsertQueueApc, STATUS_IN_PAGE_ERROR at elevated IRQL)
pub mod irq;
#[cfg(not(target_arch = "aarch64"))]
pub mod ramdisk;  // ARM64 uses C ramdisk.c (Rust retranslation still crashes)
pub mod rawfs;
pub mod remlock;
pub mod symlink;
pub mod util;
pub mod volume;
