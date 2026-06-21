//! Shared network-config snapshot (M9 management plane).
//!
//! The poll worker is the sole owner of the smoltcp `Interface`, but the iphlpapi
//! MIB queries (`IOCTL_TCP_QUERY_INFORMATION_EX` → ipconfig / GetAdaptersInfo /
//! GetIpAddrTable) arrive as **METHOD_NEITHER** IOCTLs whose `Irp->UserBuffer` is a
//! raw user-mode pointer valid ONLY in the calling thread — so they must be
//! answered synchronously in the dispatch thread, which cannot touch the worker's
//! iface. The worker therefore publishes the current L2/L3 config here (lock-free
//! atomics; aligned loads/stores don't tear) for the dispatch to read.
//!
//! Addresses are stored as their network-order octets reinterpreted with
//! `from_ne_bytes`, so the recovered `u32` matches the on-wire `in_addr` layout
//! the MIB structs expect (octet `a` at the lowest address).

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// MAC in the low 48 bits (byte 0 in the most-significant position), 0 = unknown.
static MAC: AtomicU64 = AtomicU64::new(0);
static IPV4: AtomicU32 = AtomicU32::new(0);
static MASK: AtomicU32 = AtomicU32::new(0);
static GATEWAY: AtomicU32 = AtomicU32::new(0);

pub fn set_mac(mac: [u8; 6]) {
    let v = (mac[0] as u64) << 40
        | (mac[1] as u64) << 32
        | (mac[2] as u64) << 24
        | (mac[3] as u64) << 16
        | (mac[4] as u64) << 8
        | (mac[5] as u64);
    MAC.store(v, Ordering::Relaxed);
}

pub fn mac() -> [u8; 6] {
    let v = MAC.load(Ordering::Relaxed);
    [
        (v >> 40) as u8,
        (v >> 32) as u8,
        (v >> 24) as u8,
        (v >> 16) as u8,
        (v >> 8) as u8,
        v as u8,
    ]
}

/// Publish the DHCP-assigned config. `prefix_len` is the CIDR length (0..=32);
/// the netmask + broadcast are derived from it.
pub fn set_ipv4(ip: [u8; 4], prefix_len: u8, gw: Option<[u8; 4]>) {
    IPV4.store(u32::from_ne_bytes(ip), Ordering::Relaxed);
    MASK.store(u32::from_ne_bytes(mask_octets(prefix_len)), Ordering::Relaxed);
    GATEWAY.store(u32::from_ne_bytes(gw.unwrap_or([0, 0, 0, 0])), Ordering::Relaxed);
}

/// The interface IPv4 as a network-order `in_addr` ULONG (0 if unconfigured).
pub fn ipv4_naddr() -> u32 {
    IPV4.load(Ordering::Relaxed)
}

pub fn mask_naddr() -> u32 {
    MASK.load(Ordering::Relaxed)
}

pub fn gateway_naddr() -> u32 {
    GATEWAY.load(Ordering::Relaxed)
}

/// Broadcast = ip | ~mask, as a network-order `in_addr` ULONG.
pub fn broadcast_naddr() -> u32 {
    let ip = IPV4.load(Ordering::Relaxed);
    let mask = MASK.load(Ordering::Relaxed);
    ip | !mask
}

/// True once a DHCP lease has been published.
pub fn configured() -> bool {
    IPV4.load(Ordering::Relaxed) != 0
}

/// CIDR prefix length -> network-order mask octets (e.g. 24 -> [255,255,255,0]).
fn mask_octets(prefix_len: u8) -> [u8; 4] {
    let p = prefix_len.min(32);
    let bits: u32 = if p == 0 { 0 } else { 0xFFFF_FFFFu32 << (32 - p as u32) };
    // `bits` is host-order with the network prefix in the high bits; emit
    // big-endian so octet order is a.b.c.d (network order).
    bits.to_be_bytes()
}
