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
pub mod time;
pub mod hal;
pub mod pci;
pub mod device;
pub mod selftest;
pub mod engine;
pub mod tdi;
pub mod tditrans;
pub mod tdidev;
pub mod tditest;
pub mod tdidgtest;

use ntkrnl_sys::ddk::{DRIVER_OBJECT, NTSTATUS, UNICODE_STRING};

/// netstack.sys entry point. Registered as a system-start (Start=1), non-PnP
/// kernel driver, so the kernel calls this unconditionally at boot. For M1 we
/// run the throwaway engine self-test and return success (the driver then sits
/// idle until the TDI surface lands at M4).
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: *mut DRIVER_OBJECT,
    _RegistryPath: *mut UNICODE_STRING,
) -> NTSTATUS {
    serial::emit(core::format_args!(
        "[NETSTACK] DriverEntry: netstack.sys loaded\n"
    ));
    // M1: blocking ARP round-trip self-test (creates + drops its own device).
    selftest::run();
    // M2/M3: spawn the persistent smoltcp poll-loop engine (DHCP/ping/echo/DNS).
    engine::start();
    // M4: create the TDI transport devices + dispatch so AFD (winsock) can drive
    // the stack. netstack owns \Device\Tcp etc.
    tdidev::init(DriverObject);
    // M4 verification: a kernel TDI client (tditest::spawn) that drives our own
    // \Device\Tcp via real IRPs (the exact path AFD uses) — open address +
    // connection objects (EA buffers), associate, connect to the slirp echo,
    // send, receive. Re-enabled after fixing the inbox synchronization (the
    // prior KeAcquireSpinLockForDpc is a no-op in this tree → unsynchronized
    // Vec race; now uses IrqlGuard/DISPATCH_LEVEL = real UP mutual exclusion).
    tditest::spawn();
    // M5 verification: the UDP/datagram analogue — a kernel TDI client that drives
    // our own \Device\Udp via real IRPs (the path AFD uses for sendto/recvfrom and
    // the usermode DHCP/DNS services): open an address object, register a
    // TDI_EVENT_RECEIVE_DATAGRAM handler, send a DNS query datagram to slirp's DNS
    // forwarder, receive the reply via the datagram upcall.
    tdidgtest::spawn();
    0 // STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::emit(core::format_args!("[NETSTACK] PANIC: {}\n", info));
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
