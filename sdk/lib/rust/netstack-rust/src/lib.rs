#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// netstack.sys — smoltcp-based network stack (L4 rewrite).
// Engine bringup order (docs/engineering/smoltcp-netstack-design.md):
//   M1: virtio Hal + VirtIONet raw frame RX/TX
//   M2: smoltcp phy::Device + Interface + poll loop + ICMP/TCP-echo self-test
//   M3: DHCPv4 + DNS
//   M4: TDI dispatch + completion bridge (the AFD-facing surface)
// Self-test progress is reported over serial with the `[NETSTACK]` markers
// tests/qemu/test_net.py asserts on — that contract lives in the test's
// module docstring; change both or neither.

extern crate alloc;

pub mod kalloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
