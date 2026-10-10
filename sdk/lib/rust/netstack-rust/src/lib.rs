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
pub mod dma;
pub mod hal;
pub mod pci;
pub mod device;
pub mod e1000;
// A NIC only ACPI describes (the Pi 4's GENET): netstack binds to it as its PnP
// function driver (pnp.rs) and drives it directly (genet.rs).
pub mod pnp;
pub mod gem;
pub mod genet;
// M10: interrupt-driven RX (NIC interrupt -> ISR -> DPC -> event): virtio's INTx
// on x86, and the platform NICs' interrupt resource from PnP on arm64.
pub mod isr;
// M11: real-NIC backend via an NDIS protocol shim (off by default — virtio is the
// shipping path). Selecting it swaps VirtioPhy -> NdisPhy in engine.rs.
#[cfg(feature = "ndis")]
pub mod ndis;
#[cfg(feature = "ndis")]
pub mod ndisphy;
pub mod netcfg;
pub mod selftest;
pub mod engine;
pub mod tdi;
pub mod tditrans;
pub mod tdidev;
pub mod tditest;
pub mod tdidgtest;
pub mod tdisrvtest;
pub mod icmptest;
pub mod settest;

use ntkrnl_sys::ddk::{DRIVER_OBJECT, NTSTATUS, PVOID, RtlWriteRegistryValue, UNICODE_STRING};

/// Compile-time ASCII -> UTF-16 (NUL-terminated) for the registry path/value
/// names RtlWriteRegistryValue takes as PCWSTR.
const fn wide<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}

/// Publish whether a virtio-net NIC is present so usermode can gate net-only
/// work on it. winlogon reads `HKLM\SYSTEM\CurrentControlSet\Services\netstack`
/// value `NicPresent` and only spawns `nettest.exe` when it is 1. Plain smoke
/// boots have no NIC → flag 0 → nettest is never spawned, keeping it off
/// explorer's timing-critical csrss/win32k handoff (the `nettest_explorer_race`
/// flake). The net-test configs (amd64-uefi-net / arm64-net) add the NIC → flag
/// 1 → the `[NETTEST]` proof still runs. The signal is intrinsic to the machine
/// (the NIC itself), so it needs no per-config media/boot-arg divergence — plain
/// smoke and net boot the SAME image. Written synchronously here in DriverEntry
/// (a cheap read-only PCI scan) so the value is committed long before winlogon
/// (a much later usermode launch) ever reads it.
unsafe fn publish_nic_presence() {
    const RTL_REGISTRY_SERVICES: u32 = 1;
    const REG_DWORD: u32 = 4;
    static KEY: [u16; 9] = wide(b"netstack\0");
    static VALUE: [u16; 11] = wide(b"NicPresent\0");
    let present: u32 = (pci::find_virtio_net().is_some() || e1000::find_e1000().is_some()) as u32;
    serial::emit(core::format_args!(
        "[NETSTACK] NicPresent={} (virtio-net or e1000 {})\n",
        present,
        if present == 1 { "found" } else { "absent" }
    ));
    let _ = RtlWriteRegistryValue(
        RTL_REGISTRY_SERVICES,
        KEY.as_ptr(),
        VALUE.as_ptr(),
        REG_DWORD,
        &present as *const u32 as PVOID,
        core::mem::size_of::<u32>() as u32,
    );
}

/// netstack.sys entry point. Registered as a system-start (Start=1), non-PnP
/// kernel driver, so the kernel calls this unconditionally at boot. For M1 we
/// run the throwaway engine self-test and return success (the driver then sits
/// idle until the TDI surface lands at M4).
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: *mut DRIVER_OBJECT,
    RegistryPath: *mut UNICODE_STRING,
) -> NTSTATUS {
    dma::read_parameters(RegistryPath);
    serial::emit(core::format_args!(
        "[NETSTACK] DriverEntry: netstack.sys loaded\n"
    ));
    // Publish NIC presence for usermode (winlogon gates the nettest.exe spawn on
    // it — see publish_nic_presence). Done first, before the self-test/engine
    // touch the device, so the flag is committed regardless of what follows.
    publish_nic_presence();
    // A platform NIC arrives as a PnP device after DriverEntry returns; register
    // AddDevice before anything can send us one.
    pnp::init(DriverObject);
    // M1: blocking ARP round-trip self-test (creates + drops its own device).
    // virtio-only — with an e1000 (or no NIC) it would just print a FAIL that
    // trips serial grep patterns; the engine below proves e1000 RX/TX instead.
    if pci::find_virtio_net().is_some() {
        selftest::run();
    }
    // M2/M3: spawn the persistent smoltcp poll-loop engine (DHCP/ping/echo/DNS).
    engine::start();
    // M4: create the TDI transport devices + dispatch so AFD (winsock) can drive
    // the stack. netstack owns \Device\Tcp etc.
    tdidev::init(DriverObject);
    // M4 verification: a kernel TDI client (tditest::spawn) that drives our own
    // \Device\Tcp via real IRPs (the exact path AFD uses) — open address +
    // connection objects (EA buffers), associate, connect to the slirp echo,
    // send, receive. The inbox is guarded by a real KSPIN_LOCK (KeAcquireSpinLock)
    // — cross-CPU mutual exclusion that holds under -smp 2, not the old
    // IrqlGuard/DISPATCH raise that only worked on a single vCPU (§9.55).
    tditest::spawn();
    // M5 verification: the UDP/datagram analogue — a kernel TDI client that drives
    // our own \Device\Udp via real IRPs (the path AFD uses for sendto/recvfrom and
    // the usermode DHCP/DNS services): open an address object, register a
    // TDI_EVENT_RECEIVE_DATAGRAM handler, send a DNS query datagram to slirp's DNS
    // forwarder, receive the reply via the datagram upcall.
    tdidgtest::spawn();
    // M8 verification: the inbound/server analogue of tditest — a kernel TDI
    // server that binds :8888, posts TDI_LISTEN (pends), and on a host connect
    // accepts + echoes. Exercises the M8 listen/accept dispatch + completion
    // bridge over the exact IRP path AfdListenSocket/AfdAccept use.
    tdisrvtest::spawn();
    // M9 verification: drive the ICMP echo (ping) IOCTL on \Device\Ip — opens the
    // device, issues IOCTL_ICMP_ECHO_REQUEST to the gateway, validates the
    // ICMP_ECHO_REPLY. Proves the path usermode ping/IcmpSendEcho2 ride on.
    icmptest::spawn();
    // M9 item 4 verification: drive IOCTL_TCP_SET_INFORMATION_EX on \Device\Tcp —
    // program a default route (the path iphlpapi's createIpForwardEntry / the
    // usermode `route` command and dhcpcsvc ride on) and assert the ack.
    settest::spawn();
    0 // STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
