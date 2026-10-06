//! nvme.sys: NVM Express controllers as disks, on every architecture.
//!
//! pci.sys enumerates the controller (class 01/08/02) and the critical device
//! database names this driver for `PCI\CC_010802`. At START it brings the
//! controller up (nvme.rs) and publishes one child PDO per active namespace,
//! which disk.sys binds through `GenDisk`, answering classpnp's SRBs itself
//! from a worker thread (pnp.rs) through the shared SCSI-disk layer
//! (scsidisk-rust). There is no storport or scsiport underneath: ReactOS's
//! storport never completed an SRB, and scsiport finds PCI adapters by its
//! own legacy bus scan, which has no slot resources to assign on arm64.
//!
//! One command at a time, through a bounce buffer, completed by the
//! controller's pin-based (INTx) interrupt with a 10 ms poll behind it (see
//! nvme.rs): the simplest controller that works on QEMU, the OptiPlex's drive
//! and a Graviton's EBS volumes alike. MSI-X (which Graviton will want) and
//! deeper queues come next.
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
