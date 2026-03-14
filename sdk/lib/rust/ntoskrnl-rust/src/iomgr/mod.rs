pub mod adapter;
// pub mod arcname;  // Stubbed-out IopReassignSystemRoot/IopVerifyDiskSignature — using C
pub mod bootlog;
pub mod controller;
// pub mod device;  // LLVM optimizer bug: eliminates success path after ObCreateObject output-pointer pattern
#[cfg(not(target_arch = "aarch64"))]
pub mod driver;
pub mod error;
pub mod iocomp;
pub mod ioevent;
pub mod iomdl;
// pub mod iomgr;  // LLVM codegen bug: generates ud2 in normal paths
pub mod iotimer;
pub mod iowork;
pub mod irq;
#[cfg(not(target_arch = "aarch64"))]
pub mod ramdisk;  // Only for i386/amd64 (never called, but provides the symbol). ARM64 uses C ramdisk.c
pub mod remlock;
pub mod symlink;
pub mod util;
