pub mod adapter;
// pub mod arcname;  // Passes amd64 but crashes i386 — needs i386-specific debugging
pub mod bootlog;
pub mod controller;
pub mod debug;
// pub mod device;  // LLVM optimizer bug: eliminates success path after ObCreateObject output-pointer pattern
pub mod deviface;
pub mod driver;
pub mod error;
// pub mod file;  // Crashes both arches — agent-translated NtCreateFile etc. have bugs
pub mod iocomp;
pub mod ioevent;
// pub mod iofunc;  // Nt* syscalls translated but buggy (SMSS crash, DbgPrint failed)
pub mod iomdl;
// pub mod iomgr;  // Testing: crashes/hangs on both i386 and amd64
pub mod iorsrce;
pub mod iotimer;
pub mod iowork;
// pub mod irp;  // 3 bugs found+fixed (NT_ERROR, KTHREAD offset, PacketType), still hangs — more bugs in IopCompleteRequest
pub mod irq;
#[cfg(not(target_arch = "aarch64"))]
pub mod ramdisk;  // ARM64 uses C ramdisk.c (Rust retranslation still crashes)
pub mod rawfs;
pub mod remlock;
pub mod symlink;
pub mod util;
pub mod volume;
