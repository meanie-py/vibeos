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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// Adapters netstack can carry at once (its own NIC, an NDIS miniport's, one spare).
pub const ADAPTER_SLOTS: usize = 3;

/// One adapter's published config. The slot index is the MIB interface instance
/// (`if_index` = slot + 1).
struct Slot {
    /// MAC in the low 48 bits (byte 0 in the most-significant position), 0 = no adapter.
    mac: AtomicU64,
    ipv4: AtomicU32,
    mask: AtomicU32,
    gateway: AtomicU32,
    dns0: AtomicU32,
    /// Index into `NAMES`: what drives the adapter, for its MIB description.
    name: AtomicUsize,
}

static SLOT: [Slot; ADAPTER_SLOTS] = [const {
    Slot {
        mac: AtomicU64::new(0),
        ipv4: AtomicU32::new(0),
        mask: AtomicU32::new(0),
        gateway: AtomicU32::new(0),
        dns0: AtomicU32::new(0),
        name: AtomicUsize::new(0),
    }
}; ADAPTER_SLOTS];

/// The slot whose adapter carries the stack's own sockets (TDI, the self-tests):
/// the getters without a slot argument read it.
static PRIMARY: AtomicUsize = AtomicUsize::new(0);

/// What can drive an adapter, as the MIB describes it.
pub const NAMES: [&str; 6] = [
    "netstack smoltcp adapter",
    "netstack smoltcp adapter (virtio-net)",
    "netstack smoltcp adapter (Intel Ethernet)",
    "netstack smoltcp adapter (Broadcom GENET Ethernet)",
    "netstack smoltcp adapter (Cadence GEM Ethernet)",
    "netstack smoltcp adapter (NDIS miniport)",
];

pub fn set_mac(slot: usize, mac: [u8; 6], name: usize) {
    let v = (mac[0] as u64) << 40
        | (mac[1] as u64) << 32
        | (mac[2] as u64) << 24
        | (mac[3] as u64) << 16
        | (mac[4] as u64) << 8
        | (mac[5] as u64);
    SLOT[slot].name.store(name, Ordering::Relaxed);
    SLOT[slot].mac.store(v, Ordering::Release);
}

/// Whether `slot` has an adapter.
pub fn present(slot: usize) -> bool {
    SLOT[slot].mac.load(Ordering::Acquire) != 0
}

/// How many adapters there are (they fill the slots from 0).
pub fn count() -> usize {
    (0..ADAPTER_SLOTS).take_while(|&i| present(i)).count()
}

pub fn name(slot: usize) -> &'static str {
    NAMES[SLOT[slot].name.load(Ordering::Relaxed).min(NAMES.len() - 1)]
}

pub fn mac_of(slot: usize) -> [u8; 6] {
    let v = SLOT[slot].mac.load(Ordering::Relaxed);
    [
        (v >> 40) as u8,
        (v >> 32) as u8,
        (v >> 24) as u8,
        (v >> 16) as u8,
        (v >> 8) as u8,
        v as u8,
    ]
}

pub fn mac() -> [u8; 6] {
    mac_of(primary())
}

pub fn set_primary(slot: usize) {
    PRIMARY.store(slot, Ordering::Release);
}

pub fn primary() -> usize {
    PRIMARY.load(Ordering::Acquire)
}

/// Publish a DHCP-assigned config. `prefix_len` is the CIDR length (0..=32);
/// the netmask + broadcast are derived from it.
pub fn set_ipv4(slot: usize, ip: [u8; 4], prefix_len: u8, gw: Option<[u8; 4]>) {
    let s = &SLOT[slot];
    s.ipv4.store(u32::from_ne_bytes(ip), Ordering::Relaxed);
    s.mask.store(u32::from_ne_bytes(mask_octets(prefix_len)), Ordering::Relaxed);
    s.gateway.store(u32::from_ne_bytes(gw.unwrap_or([0, 0, 0, 0])), Ordering::Relaxed);
}

/// The lease is gone.
pub fn clear_ipv4(slot: usize) {
    let s = &SLOT[slot];
    s.ipv4.store(0, Ordering::Relaxed);
    s.mask.store(0, Ordering::Relaxed);
    s.gateway.store(0, Ordering::Relaxed);
    s.dns0.store(0, Ordering::Relaxed);
}

/// A slot's IPv4 as a network-order `in_addr` ULONG (0 if unconfigured).
pub fn ipv4_of(slot: usize) -> u32 {
    SLOT[slot].ipv4.load(Ordering::Relaxed)
}

pub fn mask_of(slot: usize) -> u32 {
    SLOT[slot].mask.load(Ordering::Relaxed)
}

pub fn gateway_of(slot: usize) -> u32 {
    SLOT[slot].gateway.load(Ordering::Relaxed)
}

/// Broadcast = ip | ~mask, as a network-order `in_addr` ULONG.
pub fn broadcast_of(slot: usize) -> u32 {
    ipv4_of(slot) | !mask_of(slot)
}

/// The primary adapter's IPv4 as a network-order `in_addr` ULONG (0 if unconfigured).
pub fn ipv4_naddr() -> u32 {
    ipv4_of(primary())
}

pub fn broadcast_naddr() -> u32 {
    broadcast_of(primary())
}

/// The primary lease's gateway, if it named one.
pub fn gateway() -> Option<[u8; 4]> {
    match gateway_of(primary()) {
        0 => None,
        v => Some(v.to_ne_bytes()),
    }
}

/// Whether this is QEMU's user-mode (slirp) network, gateway 10.0.2.2: the only
/// network where the harness's host listeners (10.0.2.100 guestfwds) exist.
/// Self-tests that need one run only here. Elsewhere a connect to 10.0.2.100 sits
/// in SYN-SENT, and smoltcp rate-limits neighbor discovery for the whole interface
/// (one ARP a second, any address), so whatever it is ARPing for starves the ARP a
/// reply to a real peer needs.
pub fn on_slirp() -> bool {
    gateway() == Some([10, 0, 2, 2])
}

/// The primary lease's first DNS server, if it named one.
pub fn dns0() -> Option<[u8; 4]> {
    match SLOT[primary()].dns0.load(Ordering::Relaxed) {
        0 => None,
        v => Some(v.to_ne_bytes()),
    }
}

/// True once the primary adapter holds a lease.
pub fn configured() -> bool {
    configured_of(primary())
}

pub fn configured_of(slot: usize) -> bool {
    ipv4_of(slot) != 0
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
/// the network itself works. Runs on the poll worker at PASSIVE, once per lease;
/// each adapter has its own key (`netstack`, then `netstack1`, ...).
pub fn publish_dns_servers(slot: usize, servers: &[[u8; 4]]) {
    use ntkrnl_sys::ddk::{ZwClose, ZwCreateKey, ZwSetValueKey, OBJECT_ATTRIBUTES, UNICODE_STRING};

    if servers.is_empty() {
        return;
    }
    SLOT[slot].dns0.store(u32::from_ne_bytes(servers[0]), Ordering::Relaxed);
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
            if part == "netstack" && slot > 0 {
                wide[len] = (b'0' + slot as u8) as u16;
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
