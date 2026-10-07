//! nvme.sys: NVM Express controllers as disks, on every architecture.
//!
//! pci.sys enumerates the controller (class 01/08/02) and the critical device
//! database names this driver for `PCI\CC_010802`. At START it brings the
//! controller up (nvme.rs) and publishes one child PDO per active namespace,
//! which disk.sys binds through `GenDisk`, answering classpnp's SRBs itself
//! (pnp.rs) through the shared SCSI-disk layer (scsidisk-rust). There is no
//! storport or scsiport underneath: ReactOS's storport never completed an
//! SRB, and scsiport finds PCI adapters by its own legacy bus scan, which has
//! no slot resources to assign on arm64.
//!
//! Reads, writes and flushes go straight from the dispatch routine onto one
//! I/O queue, up to 63 commands deep, by DMA to and from the requester's own
//! pages (a bounce buffer only for data no MDL describes or not on a dword),
//! and complete from the DPC of an MSI-X message (admin and I/O queue each
//! its own) or, where none are assigned, the pin-based (INTx) interrupt, with
//! a 10 ms timer behind either (see nvme.rs). One queue pair serves every
//! CPU: per-CPU queues are owed, and matter only once one queue is the limit.
//!
//! There is no C original. Windows' equivalent is stornvme.sys, a storport
//! miniport; ReactOS has none.

#![no_std]
#![allow(non_snake_case)]

mod nvme;
mod pnp;
mod print;

use ntkrnl_sys::ddk::{NTSTATUS, PDRIVER_OBJECT, PUNICODE_STRING};

const IRP_MJ_MAXIMUM_FUNCTION: usize = 0x1B;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, _registry: PUNICODE_STRING) -> NTSTATUS {
    let driver = &mut *driver;
    for f in driver.MajorFunction[..=IRP_MJ_MAXIMUM_FUNCTION].iter_mut() {
        *f = Some(pnp::dispatch);
    }
    (*driver.DriverExtension).AddDevice = Some(pnp::add_device);
    0
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
