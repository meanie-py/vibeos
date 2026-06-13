#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// netstack.sys — smoltcp-based network stack (L4 rewrite).
// Engine bringup order (docs/engineering/smoltcp-netstack-design.md):
//   M1: virtio Hal + VirtIONet raw frame RX/TX            <-- here
//   M2: smoltcp phy::Device + Interface + poll loop + ICMP/TCP-echo self-test
//   M3: DHCPv4 + DNS
//   M4: TDI dispatch + completion bridge (the AFD-facing surface)
// Self-test progress is reported over serial with the `[NETSTACK]` markers
// tests/qemu/test_net.py asserts on.

extern crate alloc;

pub mod kalloc;
pub mod serial;
pub mod hal;
pub mod pci;
pub mod selftest;

use ntkrnl_sys::ddk::{DRIVER_OBJECT, NTSTATUS, UNICODE_STRING};

/// netstack.sys entry point. Registered as a system-start (Start=1), non-PnP
/// kernel driver, so the kernel calls this unconditionally at boot. For M1 we
/// run the throwaway engine self-test and return success (the driver then sits
/// idle until the TDI surface lands at M4).
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    _DriverObject: *mut DRIVER_OBJECT,
    _RegistryPath: *mut UNICODE_STRING,
) -> NTSTATUS {
    serial::emit(core::format_args!(
        "[NETSTACK] DriverEntry: netstack.sys loaded\n"
    ));
    selftest::run();
    0 // STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::emit(core::format_args!("[NETSTACK] PANIC: {}\n", info));
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
