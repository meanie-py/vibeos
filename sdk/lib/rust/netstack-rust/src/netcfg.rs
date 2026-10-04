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

/// Publish the lease's DNS servers where the user-mode resolver looks for them.
///
/// dnsapi takes its server list from iphlpapi's GetNetworkParams, which enumerates
/// `Tcpip\Parameters\Interfaces\*` and reads each interface's `NameServer` or
/// `DhcpNameServer` (comma-separated, split by resinfo's EnumNameServers). On
/// Windows dhcpcsvc writes `DhcpNameServer` when it binds a lease; here netstack's
/// own DHCP client owns the lease (dhcpcsvc's DISCOVERs never see a reply), so
/// unless netstack writes it, every name lookup fails with "host not found" while
/// the network itself works. Runs on the poll worker at PASSIVE, once per lease.
pub fn publish_dns_servers(servers: &[[u8; 4]]) {
    use ntkrnl_sys::ddk::{ZwClose, ZwCreateKey, ZwSetValueKey, OBJECT_ATTRIBUTES, UNICODE_STRING};

    if servers.is_empty() {
        return;
    }
    // "a.b.c.d,a.b.c.d" as UTF-16, NUL-terminated (REG_SZ).
    let mut text = [0u16; 128];
    let mut n = 0usize;
    for (i, ip) in servers.iter().enumerate() {
        if i > 0 && n < text.len() - 1 {
            text[n] = b',' as u16;
            n += 1;
        }
        for (j, octet) in ip.iter().enumerate() {
            if j > 0 && n < text.len() - 1 {
                text[n] = b'.' as u16;
                n += 1;
            }
            let mut digits = [0u8; 3];
            let mut v = *octet;
            let mut len = 0;
            loop {
                digits[len] = b'0' + v % 10;
                v /= 10;
                len += 1;
                if v == 0 {
                    break;
                }
            }
            while len > 0 && n < text.len() - 1 {
                len -= 1;
                text[n] = digits[len] as u16;
                n += 1;
            }
        }
    }
    let data_bytes = ((n + 1) * 2) as u32;

    // Create each level in turn: the hives need not carry Interfaces yet.
    const PATH: [&str; 4] = [
        "\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Services\\Tcpip",
        "Parameters",
        "Interfaces",
        "netstack",
    ];
    const KEY_ALL_ACCESS: u32 = 0xF003F;
    const OBJ_CASE_INSENSITIVE: u32 = 0x40;
    const OBJ_KERNEL_HANDLE: u32 = 0x200;
    const REG_SZ: u32 = 1;

    unsafe {
        let mut parent: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut wide = [0u16; 96];
        for part in PATH {
            let mut len = 0usize;
            for b in part.bytes() {
                wide[len] = b as u16;
                len += 1;
            }
            let mut name = UNICODE_STRING {
                Length: (len * 2) as u16,
                MaximumLength: (len * 2) as u16,
                Buffer: wide.as_mut_ptr(),
            };
            let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
            oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
            oa.RootDirectory = parent;
            oa.ObjectName = &mut name;
            oa.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
            let mut key = core::ptr::null_mut();
            let st = ZwCreateKey(&mut key, KEY_ALL_ACCESS, &mut oa, 0, core::ptr::null_mut(), 0, core::ptr::null_mut());
            if !parent.is_null() {
                ZwClose(parent);
            }
            if st < 0 {
                crate::serial::nsprint!("[NETSTACK] dns: ZwCreateKey {} failed {:#x}\n", part, st);
                return;
            }
            parent = key;
        }
        let value: [u16; 14] = [
            b'D' as u16, b'h' as u16, b'c' as u16, b'p' as u16, b'N' as u16, b'a' as u16,
            b'm' as u16, b'e' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16,
            b'e' as u16, b'r' as u16,
        ];
        let mut vname = UNICODE_STRING {
            Length: (value.len() * 2) as u16,
            MaximumLength: (value.len() * 2) as u16,
            Buffer: value.as_ptr() as *mut u16,
        };
        let st = ZwSetValueKey(parent, &mut vname, 0, REG_SZ, text.as_mut_ptr() as *mut _, data_bytes);
        ZwClose(parent);
        if st < 0 {
            crate::serial::nsprint!("[NETSTACK] dns: ZwSetValueKey failed {:#x}\n", st);
        } else {
            crate::serial::nsprint!("[NETSTACK] dns servers published ({} entries)\n", servers.len());
        }
    }
}
