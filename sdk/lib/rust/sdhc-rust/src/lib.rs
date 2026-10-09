//! sdhc.sys: the Raspberry Pi 4's SD card slot (EMMC2, an SDHCI 3.0 host) as a
//! disk.
//!
//! The arm64 HAL's ACPI PnP root publishes the controller as `ACPI\BRCME88C`
//! (hal-rust/src/arm64/pnpdd.rs) and the critical device database names this
//! driver for it. At START it initialises the card (host.rs) and publishes one
//! child PDO that disk.sys binds through `GenDisk`, answering classpnp's SRBs
//! itself (scsi.rs) from a worker thread (pnp.rs). There is no storport or
//! scsiport underneath: the controller is polled PIO, so the whole port layer
//! is one queue and one thread.
//!
//! There is no C original. Windows splits this between sdbus.sys and
//! sdstor.sys, and ReactOS has neither.
//!
//! Any SDHCI on PCI (class 08/05/01) binds here too, so QEMU tests it with no
//! board: `VIBEOS_QEMU_EXTRA_ARGS="-device sdhci-pci,sd-spec-version=3,addr=0x1f
//! -drive if=none,id=sd0,format=raw,file=<image> -device sd-card,drive=sd0"`.
//! Pin the slot (`addr=`): a device that takes an earlier one renumbers the boot
//! disk and the firmware falls back to PXE. On Windows-host QEMU the image path
//! is a Windows one with forward slashes. QEMU's SDHCI completes every command
//! before the driver could sleep, so the interrupt path (irq.rs) only runs on a
//! board; the Pi 5 logs `first interrupt serviced`.

#![no_std]
#![allow(non_snake_case)]

mod host;
mod irq;
mod pnp;
mod print;
mod scsi;

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
