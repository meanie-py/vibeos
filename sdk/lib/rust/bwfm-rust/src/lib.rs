//! The Raspberry Pis' CYW43455 Wi-Fi: docs/engineering/pi-wifi-design.md.
//!
//! `proto`, `sdio`, `chip`, `nvram`, `bus` and `scan` are plain code over a `sdhci_rust::Platform`, so the
//! host tests (tools/test_bwfm_proto.py) run them against `sim`'s simulated
//! controller and chip.

#![cfg_attr(not(test), no_std)]

pub mod bus;
pub mod chip;
pub mod nvram;
pub mod proto;
pub mod scan;
pub mod sdio;
#[cfg(test)]
mod sim;

// The driver itself (bwfm.sys): kernel-only, so out of the host tests.
#[cfg(not(test))]
mod driver;
#[cfg(not(test))]
mod print;

#[cfg(not(test))]
const IRP_MJ_MAXIMUM_FUNCTION: usize = 0x1B;

/// # Safety
/// Called once by the I/O manager with a valid driver object.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: ntkrnl_sys::ddk::PDRIVER_OBJECT,
    _registry: ntkrnl_sys::ddk::PUNICODE_STRING,
) -> ntkrnl_sys::ddk::NTSTATUS {
    let driver = &mut *driver;
    for f in driver.MajorFunction[..=IRP_MJ_MAXIMUM_FUNCTION].iter_mut() {
        *f = Some(driver::dispatch);
    }
    (*driver.DriverExtension).AddDevice = Some(driver::add_device);
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
