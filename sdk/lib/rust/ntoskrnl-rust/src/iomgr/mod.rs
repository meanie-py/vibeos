pub mod adapter;
pub mod controller;
// pub mod device;  // LLVM optimizer bug: eliminates success path after ObCreateObject output-pointer pattern
#[cfg(not(target_arch = "aarch64"))]
pub mod driver;
pub mod ioevent;
pub mod iomdl;
// pub mod iomgr;  // LLVM codegen bug: generates ud2 in normal paths
pub mod iotimer;
pub mod iowork;
pub mod remlock;
pub mod symlink;
