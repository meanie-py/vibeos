//! Per-crate write barrier for iomgr-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate iomgr_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! 55 entries — transcribed 1:1 from the monolith global_barrier.rs iomgr block
//! (IopAutoReboot was the 56th; it relocated to ntoskrnl-rust cmdata.rs — the const
//! CmControlVector table needs its in-crate address, so it's barriered there)
//! (the `iomgr::` path segment dropped since these now live at this crate's root).
//! NOTE: iomgr defines 66 `#[no_mangle]` statics total; only these 56 were in the
//! monolith barrier (the other 10 are non-zero-init / const-safe and were never
//! barrier-listed). Preserve the 56-entry set exactly; do not add the missing 10
//! without evidence they need the barrier.
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches iomgr-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _iomgr_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    // adapter.rs
    write_volatile(addr_of_mut!(crate::adapter::IoAdapterObjectType), read_volatile(addr_of!(crate::adapter::IoAdapterObjectType)));
    write_volatile(addr_of_mut!(crate::adapter::IoDeviceHandlerObjectType), read_volatile(addr_of!(crate::adapter::IoDeviceHandlerObjectType)));
    write_volatile(addr_of_mut!(crate::adapter::IoDeviceHandlerObjectSize), read_volatile(addr_of!(crate::adapter::IoDeviceHandlerObjectSize)));
    // arcname.rs
    write_volatile(addr_of_mut!(crate::arcname::IoArcHalDeviceName), read_volatile(addr_of!(crate::arcname::IoArcHalDeviceName)));
    write_volatile(addr_of_mut!(crate::arcname::IoArcBootDeviceName), read_volatile(addr_of!(crate::arcname::IoArcBootDeviceName)));
    write_volatile(addr_of_mut!(crate::arcname::IoLoaderArcBootDeviceName), read_volatile(addr_of!(crate::arcname::IoLoaderArcBootDeviceName)));
    // controller.rs
    write_volatile(addr_of_mut!(crate::controller::IoControllerObjectType), read_volatile(addr_of!(crate::controller::IoControllerObjectType)));
    // device.rs
    write_volatile(addr_of_mut!(crate::device::ShutdownListHead), read_volatile(addr_of!(crate::device::ShutdownListHead)));
    write_volatile(addr_of_mut!(crate::device::LastChanceShutdownListHead), read_volatile(addr_of!(crate::device::LastChanceShutdownListHead)));
    write_volatile(addr_of_mut!(crate::device::ShutdownListLock), read_volatile(addr_of!(crate::device::ShutdownListLock)));
    // driver.rs
    write_volatile(addr_of_mut!(crate::driver::IopDriverLoadResource), read_volatile(addr_of!(crate::driver::IopDriverLoadResource)));
    write_volatile(addr_of_mut!(crate::driver::DriverReinitListHead), read_volatile(addr_of!(crate::driver::DriverReinitListHead)));
    write_volatile(addr_of_mut!(crate::driver::DriverReinitListLock), read_volatile(addr_of!(crate::driver::DriverReinitListLock)));
    write_volatile(addr_of_mut!(crate::driver::DriverBootReinitListHead), read_volatile(addr_of!(crate::driver::DriverBootReinitListHead)));
    write_volatile(addr_of_mut!(crate::driver::DriverBootReinitListLock), read_volatile(addr_of!(crate::driver::DriverBootReinitListLock)));
    write_volatile(addr_of_mut!(crate::driver::IopHardwareDatabaseKey), read_volatile(addr_of!(crate::driver::IopHardwareDatabaseKey)));
    write_volatile(addr_of_mut!(crate::driver::IoDriverObjectType), read_volatile(addr_of!(crate::driver::IoDriverObjectType)));
    // error.rs
    write_volatile(addr_of_mut!(crate::error::IopTotalLogSize), read_volatile(addr_of!(crate::error::IopTotalLogSize)));
    write_volatile(addr_of_mut!(crate::error::IopErrorLogListHead), read_volatile(addr_of!(crate::error::IopErrorLogListHead)));
    write_volatile(addr_of_mut!(crate::error::IopLogListLock), read_volatile(addr_of!(crate::error::IopLogListLock)));
    write_volatile(addr_of_mut!(crate::error::IopErrorLogWorkItem), read_volatile(addr_of!(crate::error::IopErrorLogWorkItem)));
    write_volatile(addr_of_mut!(crate::error::IopErrorLogObject), read_volatile(addr_of!(crate::error::IopErrorLogObject)));
    // iocomp.rs
    write_volatile(addr_of_mut!(crate::iocomp::IoCompletionType), read_volatile(addr_of!(crate::iocomp::IoCompletionType)));
    write_volatile(addr_of_mut!(crate::iocomp::IoCompletionPacketLookaside), read_volatile(addr_of!(crate::iocomp::IoCompletionPacketLookaside)));
    write_volatile(addr_of_mut!(crate::iocomp::IopCompletionMapping), read_volatile(addr_of!(crate::iocomp::IopCompletionMapping)));
    // iofunc.rs
    write_volatile(addr_of_mut!(crate::iofunc::IoPageReadIrpAllocationFailure), read_volatile(addr_of!(crate::iofunc::IoPageReadIrpAllocationFailure)));
    write_volatile(addr_of_mut!(crate::iofunc::IoPageReadNonPagefileIrpAllocationFailure), read_volatile(addr_of!(crate::iofunc::IoPageReadNonPagefileIrpAllocationFailure)));
    // iomgr.rs
    write_volatile(addr_of_mut!(crate::iomgr::IopTraceLevel), read_volatile(addr_of!(crate::iomgr::IopTraceLevel)));
    write_volatile(addr_of_mut!(crate::iomgr::PnpSystemInit), read_volatile(addr_of!(crate::iomgr::PnpSystemInit)));
    write_volatile(addr_of_mut!(crate::iomgr::IoDeviceObjectType), read_volatile(addr_of!(crate::iomgr::IoDeviceObjectType)));
    write_volatile(addr_of_mut!(crate::iomgr::IoFileObjectType), read_volatile(addr_of!(crate::iomgr::IoFileObjectType)));
    write_volatile(addr_of_mut!(crate::iomgr::IoCountOperations), read_volatile(addr_of!(crate::iomgr::IoCountOperations)));
    write_volatile(addr_of_mut!(crate::iomgr::IoReadOperationCount), read_volatile(addr_of!(crate::iomgr::IoReadOperationCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoReadTransferCount), read_volatile(addr_of!(crate::iomgr::IoReadTransferCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoWriteOperationCount), read_volatile(addr_of!(crate::iomgr::IoWriteOperationCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoWriteTransferCount), read_volatile(addr_of!(crate::iomgr::IoWriteTransferCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoOtherOperationCount), read_volatile(addr_of!(crate::iomgr::IoOtherOperationCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoOtherTransferCount), read_volatile(addr_of!(crate::iomgr::IoOtherTransferCount)));
    write_volatile(addr_of_mut!(crate::iomgr::IoStatisticsLock), read_volatile(addr_of!(crate::iomgr::IoStatisticsLock)));
    // IopAutoReboot relocated to ntoskrnl-rust (cmdata.rs) — barriered there.
    write_volatile(addr_of_mut!(crate::iomgr::IopNumTriageDumpDataBlocks), read_volatile(addr_of!(crate::iomgr::IopNumTriageDumpDataBlocks)));
    write_volatile(addr_of_mut!(crate::iomgr::IopFileMapping), read_volatile(addr_of!(crate::iomgr::IopFileMapping)));
    write_volatile(addr_of_mut!(crate::iomgr::IopSecurityResource), read_volatile(addr_of!(crate::iomgr::IopSecurityResource)));
    write_volatile(addr_of_mut!(crate::iomgr::IopLoaderBlock), read_volatile(addr_of!(crate::iomgr::IopLoaderBlock)));
    // iotimer.rs
    write_volatile(addr_of_mut!(crate::iotimer::IopTimerLock), read_volatile(addr_of!(crate::iotimer::IopTimerLock)));
    write_volatile(addr_of_mut!(crate::iotimer::IopTimerQueueHead), read_volatile(addr_of!(crate::iotimer::IopTimerQueueHead)));
    write_volatile(addr_of_mut!(crate::iotimer::IopTimerDpc), read_volatile(addr_of!(crate::iotimer::IopTimerDpc)));
    write_volatile(addr_of_mut!(crate::iotimer::IopTimer), read_volatile(addr_of!(crate::iotimer::IopTimer)));
    write_volatile(addr_of_mut!(crate::iotimer::IopTimerCount), read_volatile(addr_of!(crate::iotimer::IopTimerCount)));
    // irp.rs
    write_volatile(addr_of_mut!(crate::irp::IopReserveIrpAllocator), read_volatile(addr_of!(crate::irp::IopReserveIrpAllocator)));
    // volume.rs
    write_volatile(addr_of_mut!(crate::volume::IopDatabaseResource), read_volatile(addr_of!(crate::volume::IopDatabaseResource)));
    write_volatile(addr_of_mut!(crate::volume::IopDiskFileSystemQueueHead), read_volatile(addr_of!(crate::volume::IopDiskFileSystemQueueHead)));
    write_volatile(addr_of_mut!(crate::volume::IopNetworkFileSystemQueueHead), read_volatile(addr_of!(crate::volume::IopNetworkFileSystemQueueHead)));
    write_volatile(addr_of_mut!(crate::volume::IopCdRomFileSystemQueueHead), read_volatile(addr_of!(crate::volume::IopCdRomFileSystemQueueHead)));
    write_volatile(addr_of_mut!(crate::volume::IopTapeFileSystemQueueHead), read_volatile(addr_of!(crate::volume::IopTapeFileSystemQueueHead)));
    write_volatile(addr_of_mut!(crate::volume::IopFsNotifyChangeQueueHead), read_volatile(addr_of!(crate::volume::IopFsNotifyChangeQueueHead)));
}
