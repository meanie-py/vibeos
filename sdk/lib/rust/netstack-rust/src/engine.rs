//! M2+M3 engine: smoltcp Interface + a poll-loop worker thread.
//!
//! M2 proved ICMP ping + a TCP echo server (static IP). M3 replaces the static
//! IP with a DHCPv4 lease from slirp and resolves a name via DNS. The worker
//! thread is still the SOLE owner of the Interface + SocketSet + device, so no
//! lock is needed yet — the shared lock + completion bridge arrive at M4 with
//! the TDI surface.

use alloc::vec;
use alloc::vec::Vec;
use core::ptr::null_mut;

use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::socket::{dhcpv4, dns, icmp, tcp};
use smoltcp::wire::{
    DnsQueryType, EthernetAddress, HardwareAddress, Icmpv4Packet, Icmpv4Repr, IpAddress, IpCidr,
};

use ntkrnl_sys::ddk::{HANDLE, PVOID, PsCreateSystemThread};

use crate::device::{self, VirtioPhy};
use crate::serial::nsprint;
use crate::time::{delay_ms, now};

/// Set once DHCP has configured the interface — the TDI self-test waits on this
/// before opening sockets (it needs a usable source address).
pub static DHCP_READY: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

const ECHO_PORT: u16 = 7777;

/// Neighbors netstack keeps resolved itself: the gateway and the on-link DNS servers.
const WARM_MAX: usize = 5;
/// Re-ARP period, inside smoltcp's 60s neighbor-entry lifetime.
const WARM_PERIOD_MS: i64 = 30_000;

/// ARP for `targets` by hand, through the device, outside smoltcp.
///
/// smoltcp rate-limits neighbor discovery for the WHOLE interface (one ARP request a
/// second, any address; smoltcp `neighbor.rs`, still so upstream at 0.14). One peer
/// that never answers -- a TCP connect to a guestfwd address that is not configured on
/// this boot, a dead host on a real LAN -- re-ARPs every second and every other new
/// destination waits behind it: 10.0.2.3 was never resolved, so no user-mode DNS
/// lookup ever left the guest. smoltcp fills its neighbor cache from ANY ARP packet
/// addressed to us, request or reply (`interface/ipv4.rs` process_arp), so a request we
/// send ourselves is resolved by the reply without going through the limiter. Per-address
/// limiting inside smoltcp is the real fix (owed, roadmap); this keeps the addresses
/// every program depends on resolvable meanwhile.
fn warm_neighbors<D: smoltcp::phy::Device>(
    phy: &mut D,
    ts: smoltcp::time::Instant,
    mac: [u8; 6],
    our_ip: [u8; 4],
    targets: &[[u8; 4]],
) {
    use smoltcp::phy::TxToken;
    for target in targets {
        let Some(tx) = phy.transmit(ts) else { return };
        tx.consume(42, |buf| {
            buf[0..6].copy_from_slice(&[0xff; 6]); // broadcast
            buf[6..12].copy_from_slice(&mac);
            buf[12..14].copy_from_slice(&[0x08, 0x06]); // ARP
            buf[14..22].copy_from_slice(&[0x00, 0x01, 0x08, 0x00, 6, 4, 0x00, 0x01]); // Ethernet/IPv4, request
            buf[22..28].copy_from_slice(&mac);
            buf[28..32].copy_from_slice(&our_ip);
            buf[32..38].copy_from_slice(&[0; 6]);
            buf[38..42].copy_from_slice(target);
        });
    }
}
const DNS_NAME: &str = "example.com";
const DNS_MAX_ATTEMPTS: u32 = 20;
const ICMP_IDENT: u16 = 0x022b;
const THREAD_ALL_ACCESS: u32 = 0x001F_03FF;

/// Spawn the netstack poll-loop worker thread. Returns immediately; the engine
/// runs for the life of the system.
pub fn start() {
    let mut handle: HANDLE = null_mut();
    // PKSTART_ROUTINE is extern "C" on amd64/arm64 but extern "stdcall" on i386;
    // `worker` is extern "system" (the correct ABI per arch). They share the ABI
    // but are nominally distinct Rust types, so bridge with a transmute.
    let start: ntkrnl_sys::ddk::PKSTART_ROUTINE =
        Some(unsafe { core::mem::transmute(worker as *const ()) });
    let status = unsafe {
        PsCreateSystemThread(
            &mut handle,
            THREAD_ALL_ACCESS,
            null_mut(),
            null_mut(),
            null_mut(),
            start,
            null_mut(),
        )
    };
    if status < 0 {
        nsprint!("[NETSTACK] FAIL: PsCreateSystemThread {:#010x}\n", status as u32);
        // No worker will ever drain the TDI inbox.
        crate::tditrans::mark_engine_dead();
    }
}


/// Runtime-selected backend: virtio-net (paravirtual, default) or e1000 (real
/// emulated NIC, `-device e1000`). Same `phy::Device` surface via delegation,
/// so the poll loop below stays backend-agnostic.
enum AnyPhy {
    Virtio(VirtioPhy),
    E1000(crate::e1000::E1000Phy),
    Genet(crate::genet::GenetPhy),
    Gem(crate::gem::GemPhy),
    Ndis(crate::ndisphy::NdisPhy),
}

impl AnyPhy {
    /// The PHY's link, for a NIC netstack drives itself and knows it of (an NDIS
    /// miniport indicates its own: `ndis::take_media_change`).
    fn link_up(&self) -> Option<bool> {
        match self {
            AnyPhy::Genet(g) => Some(g.dev().link_up()),
            AnyPhy::Gem(g) => Some(g.dev().link_up()),
            AnyPhy::E1000(e) => Some(e.dev().link_up()),
            _ => None,
        }
    }

    fn mac(&self) -> [u8; 6] {
        match self {
            AnyPhy::Virtio(v) => v.mac(),
            AnyPhy::E1000(e) => e.mac(),
            AnyPhy::Genet(g) => g.mac(),
            AnyPhy::Gem(g) => g.mac(),
            AnyPhy::Ndis(n) => n.mac(),
        }
    }
}

enum AnyRxToken {
    Virtio(crate::device::VirtioRxToken),
    E1000(crate::e1000::E1000RxToken),
    Genet(crate::genet::GenetRxToken),
    Gem(crate::gem::GemRxToken),
    Ndis(crate::ndisphy::NdisRxToken),
}

impl smoltcp::phy::RxToken for AnyRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        match self {
            AnyRxToken::Virtio(t) => smoltcp::phy::RxToken::consume(t, f),
            AnyRxToken::E1000(t) => smoltcp::phy::RxToken::consume(t, f),
            AnyRxToken::Genet(t) => smoltcp::phy::RxToken::consume(t, f),
            AnyRxToken::Gem(t) => smoltcp::phy::RxToken::consume(t, f),
            AnyRxToken::Ndis(t) => smoltcp::phy::RxToken::consume(t, f),
        }
    }
}

enum AnyTxToken<'a> {
    Virtio(crate::device::VirtioTxToken<'a>),
    E1000(crate::e1000::E1000TxToken<'a>),
    Genet(crate::genet::GenetTxToken<'a>),
    Gem(crate::gem::GemTxToken<'a>),
    Ndis(crate::ndisphy::NdisTxToken),
}

impl<'a> smoltcp::phy::TxToken for AnyTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self {
            AnyTxToken::Virtio(t) => smoltcp::phy::TxToken::consume(t, len, f),
            AnyTxToken::E1000(t) => smoltcp::phy::TxToken::consume(t, len, f),
            AnyTxToken::Genet(t) => smoltcp::phy::TxToken::consume(t, len, f),
            AnyTxToken::Gem(t) => smoltcp::phy::TxToken::consume(t, len, f),
            AnyTxToken::Ndis(t) => smoltcp::phy::TxToken::consume(t, len, f),
        }
    }
}

impl smoltcp::phy::Device for AnyPhy {
    type RxToken<'a> = AnyRxToken;
    type TxToken<'a> = AnyTxToken<'a>;

    fn receive(
        &mut self,
        ts: smoltcp::time::Instant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self {
            AnyPhy::Virtio(v) => v
                .receive(ts)
                .map(|(r, t)| (AnyRxToken::Virtio(r), AnyTxToken::Virtio(t))),
            AnyPhy::E1000(e) => e
                .receive(ts)
                .map(|(r, t)| (AnyRxToken::E1000(r), AnyTxToken::E1000(t))),
            AnyPhy::Genet(g) => g
                .receive(ts)
                .map(|(r, t)| (AnyRxToken::Genet(r), AnyTxToken::Genet(t))),
            AnyPhy::Gem(g) => g
                .receive(ts)
                .map(|(r, t)| (AnyRxToken::Gem(r), AnyTxToken::Gem(t))),
            AnyPhy::Ndis(n) => n
                .receive(ts)
                .map(|(r, t)| (AnyRxToken::Ndis(r), AnyTxToken::Ndis(t))),
        }
    }

    fn transmit(&mut self, ts: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        match self {
            AnyPhy::Virtio(v) => v.transmit(ts).map(AnyTxToken::Virtio),
            AnyPhy::E1000(e) => e.transmit(ts).map(AnyTxToken::E1000),
            AnyPhy::Genet(g) => g.transmit(ts).map(AnyTxToken::Genet),
            AnyPhy::Gem(g) => g.transmit(ts).map(AnyTxToken::Gem),
            AnyPhy::Ndis(n) => n.transmit(ts).map(AnyTxToken::Ndis),
        }
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        match self {
            AnyPhy::Virtio(v) => v.capabilities(),
            AnyPhy::E1000(e) => e.capabilities(),
            AnyPhy::Genet(g) => g.capabilities(),
            AnyPhy::Gem(g) => g.capabilities(),
            AnyPhy::Ndis(n) => n.capabilities(),
        }
    }
}

/// Connect a platform NIC's PnP interrupt to `ack`, then let the device raise
/// it (`enable`). Null, and the device left masked, when there is no interrupt
/// resource or the connect fails: the worker then polls.
fn connect_platform(
    irq: Option<crate::pnp::PlatformIrq>,
    ack: crate::isr::AckFn,
    ctx: *mut u8,
    enable: impl FnOnce(),
) -> *mut ntkrnl_sys::ddk::KEVENT {
    let Some(irq) = irq else {
        nsprint!("[NETSTACK] M10: the platform NIC has no interrupt resource; polling\n");
        return null_mut();
    };
    let event = unsafe {
        crate::isr::connect_resource(ack, ctx.cast(), irq.vector, irq.irql, irq.affinity as _, irq.latched, irq.shared)
    };
    if !event.is_null() {
        enable();
    }
    event
}

/// One adapter: its device, its smoltcp interface, its DHCP client and lease.
///
/// Each link has an interface of its own, so each holds its own lease and answers
/// ARP and ping on its own address. The stack's own sockets (TDI's, the self-tests',
/// SNTP's) live in one shared set, polled with the PRIMARY link's interface: they
/// leave through it, whatever their destination. The primary's DHCP client is
/// moved into that set with them (a frame is only delivered to sockets in the set
/// its interface is polled with); the others' stay in their link's own set.
struct Link {
    phy: AnyPhy,
    iface: Interface,
    own: SocketSet<'static>,
    dhcp: SocketHandle,
    rx_event: *mut ntkrnl_sys::ddk::KEVENT,
    /// The link state last seen, for a NIC whose PHY netstack reads.
    link_up: Option<bool>,
    lease: Option<Lease>,
    warm: [[u8; 4]; WARM_MAX],
    nwarm: usize,
    warm_next_ms: i64,
}

struct Lease {
    ip: [u8; 4],
    router: Option<smoltcp::wire::Ipv4Address>,
    dns: Vec<IpAddress>,
    ntp: Option<smoltcp::wire::Ipv4Address>,
}

/// netcfg::NAMES indexes: what drives a link.
const NAME_VIRTIO: usize = 1;
const NAME_E1000: usize = 2;
const NAME_GENET: usize = 3;
const NAME_GEM: usize = 4;
const NAME_NDIS: usize = 5;

impl Link {
    // Out of line, as are the other one-time helpers below: the worker runs on a 12 KB
    // i386 kernel stack, and inlined, their by-value smoltcp Interface and its copies
    // stayed in the worker's frame for its whole life (0x1C70 bytes, and a double fault
    // 1 boot in 14 once smoltcp's poll ran on what was left).
    #[inline(never)]
    fn new(slot: usize, mut phy: AnyPhy, rx_event: *mut ntkrnl_sys::ddk::KEVENT, name: usize) -> alloc::boxed::Box<Self> {
        let mac = phy.mac();
        // Published for the M9 management plane (iphlpapi IFEntry queries).
        crate::netcfg::set_mac(slot, mac, name);
        let mut config = Config::new(HardwareAddress::Ethernet(EthernetAddress(mac)));
        config.random_seed = (now().total_millis() as u64) ^ 0x9E37_79B9_7F4A_7C15 ^ ((slot as u64) << 56);
        // No static IP — DHCP assigns it.
        let iface = Interface::new(config, &mut phy, now());
        let mut own = SocketSet::new(Vec::new());
        let mut dhcp = dhcpv4::Socket::new();
        // Ask for the NTP servers too, and keep the packet so option 42 can be read.
        dhcp.set_parameter_request_list(&crate::sntp::DHCP_PARAMETERS);
        dhcp.set_receive_packet_buffer(alloc::boxed::Box::leak(vec![0u8; 1500].into_boxed_slice()));
        let dhcp = own.add(dhcp);
        let m = mac;
        nsprint!(
            "[NETSTACK] adapter {}: {}, MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
            slot, crate::netcfg::NAMES[name], m[0], m[1], m[2], m[3], m[4], m[5]
        );
        let link_up = phy.link_up();
        alloc::boxed::Box::new(Link {
            phy,
            iface,
            own,
            dhcp,
            rx_event,
            link_up,
            lease: None,
            warm: [[0; 4]; WARM_MAX],
            nwarm: 0,
            warm_next_ms: 0,
        })
    }

    /// Move the DHCP client into the shared set (`to_shared`) or back to the link's own.
    #[inline(never)]
    fn move_dhcp(&mut self, shared: &mut SocketSet<'static>, to_shared: bool) {
        let (from, to) = if to_shared { (&mut self.own, shared) } else { (shared, &mut self.own) };
        let s = from.remove(self.dhcp);
        self.dhcp = to.add(s);
    }
}

/// Take a DHCP event off `set`'s client and apply it to `iface` and netcfg's `slot`.
/// None: no event. Some(None): the lease is gone. Some(Some(lease)): a new lease.
#[inline(never)]
fn dhcp_event(
    iface: &mut Interface,
    set: &mut SocketSet<'static>,
    dhcp: SocketHandle,
    slot: usize,
) -> Option<Option<Lease>> {
    match sockets_dhcp(set, dhcp).poll()? {
        dhcpv4::Event::Configured(cfg) => {
            iface.update_ip_addrs(|addrs| {
                addrs.clear();
                let _ = addrs.push(IpCidr::Ipv4(cfg.address));
            });
            match cfg.router {
                Some(router) => {
                    let _ = iface.routes_mut().add_default_ipv4_route(router);
                }
                None => {
                    let _ = iface.routes_mut().remove_default_ipv4_route();
                }
            }
            let o = cfg.address.address().octets();
            nsprint!("[NETSTACK] dhcp lease {}.{}.{}.{} on adapter {}\n", o[0], o[1], o[2], o[3], slot);
            // Publish L3 config for the M9 management plane (ipconfig /
            // GetAdaptersInfo / GetIpAddrTable read it via the MIB queries).
            crate::netcfg::set_ipv4(slot, o, cfg.address.prefix_len(), cfg.router.map(|r| r.octets()));
            let mut dns = [[0u8; 4]; 4];
            let mut ndns = 0;
            for server in cfg.dns_servers.iter().take(dns.len()) {
                dns[ndns] = server.octets();
                ndns += 1;
            }
            crate::netcfg::publish_dns_servers(slot, &dns[..ndns]);
            let ntp = cfg.packet.as_ref().and_then(|p| crate::sntp::Sntp::lease_server(p.options()));
            Some(Some(Lease {
                ip: o,
                router: cfg.router,
                dns: dns[..ndns].iter().map(|o| IpAddress::v4(o[0], o[1], o[2], o[3])).collect(),
                ntp,
            }))
        }
        dhcpv4::Event::Deconfigured => {
            iface.update_ip_addrs(|addrs| addrs.clear());
            let _ = iface.routes_mut().remove_default_ipv4_route();
            crate::netcfg::clear_ipv4(slot);
            Some(None)
        }
    }
}

fn sockets_dhcp<'s>(set: &'s mut SocketSet<'static>, h: SocketHandle) -> &'s mut dhcpv4::Socket<'static> {
    set.get_mut::<dhcpv4::Socket>(h)
}

/// The NIC netstack drives itself: virtio-net, the e1000s, else a platform NIC
/// (the Pi's GENET and GEM), with the event its interrupt sets (null: poll it).
#[inline(never)]
fn native_link(slot: usize) -> Option<alloc::boxed::Box<Link>> {
    let mut platform_irq: Option<crate::pnp::PlatformIrq> = None;
    let (mut phy, name) = match device::bringup() {
        Some(d) => (AnyPhy::Virtio(VirtioPhy::new(d)), NAME_VIRTIO),
        None => match crate::e1000::bringup() {
            Some(d) => (AnyPhy::E1000(crate::e1000::E1000Phy::new(d)), NAME_E1000),
            // No PCI NIC: a platform NIC (pnp.rs) starts moments after
            // DriverEntry when the machine has one.
            None => {
                let nic = crate::pnp::wait()?;
                platform_irq = nic.irq;
                match nic.kind {
                    crate::pnp::NicKind::Genet => {
                        (AnyPhy::Genet(crate::genet::GenetPhy::new(crate::genet::bringup(nic)?)), NAME_GENET)
                    }
                    crate::pnp::NicKind::Gem => (AnyPhy::Gem(crate::gem::GemPhy::new(crate::gem::bringup(nic)?)), NAME_GEM),
                }
            }
        },
    };

    // M10: connect the NIC's interrupt so an arriving frame wakes this worker via
    // an event instead of pure busy-polling: virtio's INTx (x86 and arm64), the
    // GEM's PnP interrupt on the Pi 5. Null on any failure → the loop keeps the `delay_ms`
    // poll fallback, no regression.
    let rx_event: *mut ntkrnl_sys::ddk::KEVENT = match &mut phy {
        AnyPhy::Virtio(v) => {
            v.enable_interrupts();
            let df = crate::pci::find_virtio_net();
            match df.and_then(crate::pci::read_interrupt_line) {
                Some(line) => {
                    let event = unsafe { crate::isr::connect(v.dev_ptr(), line as u32) };
                    // INTx stays masked (pci::enable_bus_master) unless the ISR is in.
                    if let (false, Some(df)) = (event.is_null(), df) {
                        crate::pci::unmask_intx(df);
                    }
                    event
                }
                None => null_mut(),
            }
        }
        // The GEM shares its line with both xHCIs, so each NIC is unmasked only
        // once its ISR is in.
        AnyPhy::Gem(g) => {
            connect_platform(platform_irq, crate::gem::irq_ack, g.dev().irq_ctx(), || g.dev().enable_interrupts())
        }
        AnyPhy::E1000(e) => match e.dev().interrupt_line() {
            Some(line) => {
                let event = unsafe { crate::isr::connect_line(crate::e1000::irq_ack, e.dev().irq_ctx().cast(), line as u32) };
                if !event.is_null() {
                    e.dev().enable_interrupts();
                }
                event
            }
            None => null_mut(),
        },
        AnyPhy::Genet(g) => {
            connect_platform(platform_irq, crate::genet::irq_ack, g.dev().irq_ctx(), || g.dev().enable_interrupts())
        }
        AnyPhy::Ndis(_) => null_mut(),
    };
    Some(Link::new(slot, phy, rx_event, name))
}

unsafe extern "system" fn worker(_context: PVOID) {
    nsprint!("[NETSTACK] M2/M3 engine: starting smoltcp poll loop\n");

    // Adapters: the NIC netstack drives itself (virtio-net, the e1000s, the Pi's
    // GENET and GEM), and whatever NDIS miniport binds (M11), now or minutes into
    // boot (PnP installs a downloaded driver's device late), side by side: the Pi 5's
    // GEM and its Wi-Fi. All expose the same phy::Device surface, so the poll loop is
    // backend-agnostic.
    let mut links: Vec<alloc::boxed::Box<Link>> = Vec::new();
    let ndis_only = crate::dma::NDIS_ONLY.load(core::sync::atomic::Ordering::Acquire);
    if ndis_only {
        nsprint!("[NETSTACK] NETSTACK=NDIS: not driving any NIC directly\n");
    } else if let Some(link) = native_link(0) {
        links.push(link);
    }
    let ndis = crate::ndis::register();
    if links.is_empty() {
        if !ndis {
            nsprint!("[NETSTACK] FAIL: M2 device bringup\n");
            crate::tditrans::mark_engine_dead();
            return;
        }
        // No adapter yet: TDI callers fail fast instead of queueing behind a worker
        // that has nothing to carry them; the engine comes back once one binds.
        crate::tditrans::mark_engine_dead();
        nsprint!("[NETSTACK] no NIC of its own; waiting for an NDIS miniport\n");
    }

    let mut sockets = SocketSet::new(Vec::new());
    let tcp_handle = sockets.add(tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0u8; 4096]),
        tcp::SocketBuffer::new(vec![0u8; 4096]),
    ));
    let icmp_handle = sockets.add(icmp::Socket::new(
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 4], vec![0u8; 256]),
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 4], vec![0u8; 256]),
    ));
    // Outbound TCP client socket — connects out via the slirp guestfwd to a host
    // listener, proving the connect/send/recv path the TDI layer will sit on.
    let client_handle = sockets.add(tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0u8; 4096]),
        tcp::SocketBuffer::new(vec![0u8; 4096]),
    ));
    let dns_handle = sockets.add(dns::Socket::new(
        &[], // the primary lease's servers, set when it binds
        Vec::new(),
    ));
    sockets
        .get_mut::<icmp::Socket>(icmp_handle)
        .bind(icmp::Endpoint::Ident(ICMP_IDENT))
        .ok();
    let mut sntp = crate::sntp::Sntp::new(&mut sockets, now().total_millis() as u64 ^ 0xA076_1D64_78BD_642F);

    // The link whose interface carries the shared set.
    let mut primary = 0usize;
    if let Some(link) = links.first_mut() {
        link.move_dhcp(&mut sockets, true);
    }

    // The boot ping goes to the lease's router and the DNS self-test to the lease's
    // servers: slirp's 10.0.2.2 / 10.0.2.3 under QEMU, the LAN's own on a board.
    let mut ping_sent = false;
    let mut ping_ok = false;
    let mut echo_announced = false;
    let mut dns_started = false;
    let mut dns_attempts = 0u32;
    let mut dns_query = None;
    let mut client_sent = false;
    let mut client_done = false;
    let mut client_attempts = 0u32;
    let mut client_sent_ms: i64 = 0;
    let mut client_sends = 0u32;
    let mut client_connect_ms: i64 = 0;
    // TDI transport state (connections/addresses driven by AFD via tdidev).
    let mut tdi_state = crate::tditrans::TdiState::new();

    loop {
        // An NDIS miniport bound: a link of its own.
        if ndis && links.len() < crate::netcfg::ADAPTER_SLOTS {
            if let Some(mac) = crate::ndis::poll_bound() {
                let slot = links.len();
                links.push(Link::new(slot, AnyPhy::Ndis(crate::ndisphy::NdisPhy::new(mac)), crate::ndis::rx_event(), NAME_NDIS));
                if slot == 0 {
                    links[0].move_dhcp(&mut sockets, true);
                    crate::tditrans::revive_engine();
                }
            }
        }
        if links.is_empty() {
            delay_ms(100);
            continue;
        }
        // A link's media came or went (the cable, the access point): look for a
        // lease again at once (a new network, or none, which hands the stack's
        // sockets to another link), instead of when the old lease would run out.
        let ndis_media = crate::ndis::take_media_change();
        for (i, link) in links.iter_mut().enumerate() {
            let change = match &link.phy {
                AnyPhy::Ndis(_) => ndis_media,
                phy => match (phy.link_up(), link.link_up) {
                    (Some(now), Some(was)) if now != was => Some(now),
                    _ => None,
                },
            };
            let Some(up) = change else { continue };
            link.link_up = Some(up);
            nsprint!("[NETSTACK] adapter {}: media {}\n", i, if up { "connected" } else { "disconnected" });
            let set = if i == primary { &mut sockets } else { &mut link.own };
            sockets_dhcp(set, link.dhcp).reset();
        }

        let ts = now();
        let mut primary_changed = false;
        for (i, link) in links.iter_mut().enumerate() {
            let set = if i == primary { &mut sockets } else { &mut link.own };
            link.iface.poll(ts, &mut link.phy, set);
            if let Some(lease) = dhcp_event(&mut link.iface, set, link.dhcp, i) {
                // A new client reports itself unconfigured once, before any lease.
                if lease.is_none() && link.lease.is_some() {
                    nsprint!("[NETSTACK] dhcp lease lost on adapter {}\n", i);
                }
                link.lease = lease;
                let iface = &link.iface;
                let on_link = |o: [u8; 4]| {
                    iface.ip_addrs().iter().any(|c| c.contains_addr(&IpAddress::v4(o[0], o[1], o[2], o[3])))
                };
                let mut targets = [[0u8; 4]; WARM_MAX];
                let mut n = 0;
                if let Some(lease) = &link.lease {
                    if let Some(router) = lease.router {
                        targets[n] = router.octets();
                        n += 1;
                    }
                    for server in &lease.dns {
                        let IpAddress::Ipv4(v4) = server;
                        let o = v4.octets();
                        if on_link(o) && n < WARM_MAX && !targets[..n].contains(&o) {
                            targets[n] = o;
                            n += 1;
                        }
                    }
                }
                link.warm = targets;
                link.nwarm = n;
                link.warm_next_ms = 0;
                primary_changed |= i == primary;
            }
            // Resolve the gateway and the on-link DNS servers ourselves, now and
            // every WARM_PERIOD_MS (see warm_neighbors).
            if let (Some(lease), true) = (&link.lease, link.nwarm > 0) {
                if (ts.total_millis() as i64) >= link.warm_next_ms {
                    let mac = link.phy.mac();
                    warm_neighbors(&mut link.phy, ts, mac, lease.ip, &link.warm[..link.nwarm]);
                    link.warm_next_ms = ts.total_millis() as i64 + WARM_PERIOD_MS;
                }
            }
        }

        // The primary has no lease and another link has one: move the stack's
        // sockets to it. Connections bound to the old address end, as they would
        // when that network went away.
        if links[primary].lease.is_none() {
            if let Some(j) = links.iter().position(|l| l.lease.is_some()) {
                links[primary].move_dhcp(&mut sockets, false);
                links[j].move_dhcp(&mut sockets, true);
                nsprint!("[NETSTACK] primary adapter: {} ({})\n", j, crate::netcfg::name(j));
                primary = j;
                crate::netcfg::set_primary(j);
                primary_changed = true;
            }
        }
        if primary_changed {
            match &links[primary].lease {
                Some(lease) => {
                    sockets.get_mut::<dns::Socket>(dns_handle).update_servers(&lease.dns);
                    sntp.on_lease(lease.ntp);
                    DHCP_READY.store(true, core::sync::atomic::Ordering::Release);
                }
                None => sockets.get_mut::<dns::Socket>(dns_handle).update_servers(&[]),
            }
        }
        let gw = links[primary].lease.as_ref().and_then(|l| l.router).map(IpAddress::Ipv4);
        let dhcp_configured = links[primary].lease.is_some();
        let Link { iface, .. } = &mut *links[primary];

        // TDI data path: drain AFD's commands + run the completion bridge.
        crate::tditrans::pump(iface, &mut sockets, &mut tdi_state);

        if dhcp_configured {
            sntp.poll(iface, &mut sockets, dns_handle, ts.total_millis() as i64);

            // --- ICMP ping the gateway (also exercises ARP) ---
            {
                let s = sockets.get_mut::<icmp::Socket>(icmp_handle);
                if let (false, true, Some(gw)) = (ping_sent, s.can_send(), gw) {
                    let payload = [0u8; 16];
                    let repr = Icmpv4Repr::EchoRequest {
                        ident: ICMP_IDENT,
                        seq_no: 1,
                        data: &payload,
                    };
                    if let Ok(buf) = s.send(repr.buffer_len(), gw) {
                        let mut pkt = Icmpv4Packet::new_unchecked(buf);
                        repr.emit(&mut pkt, &ChecksumCapabilities::default());
                        ping_sent = true;
                        nsprint!("[NETSTACK] icmp echo request -> {}\n", gw);
                    }
                }
                if ping_sent && !ping_ok && s.can_recv() {
                    if let Ok((payload, _src)) = s.recv() {
                        if let Ok(pkt) = Icmpv4Packet::new_checked(payload) {
                            if let Ok(Icmpv4Repr::EchoReply { .. }) =
                                Icmpv4Repr::parse(&pkt, &ChecksumCapabilities::default())
                            {
                                nsprint!("[NETSTACK] ping gateway OK\n");
                                ping_ok = true;
                            }
                        }
                    }
                }
            }

            // --- DNS: resolve a name via slirp's forwarder ---
            if dns_query.is_none() && !dns_started && dns_attempts < DNS_MAX_ATTEMPTS {
                let q = sockets
                    .get_mut::<dns::Socket>(dns_handle)
                    .start_query(iface.context(), DNS_NAME, DnsQueryType::A);
                match q {
                    Ok(h) => {
                        dns_query = Some(h);
                        dns_attempts += 1;
                    }
                    Err(_) => dns_attempts += 1,
                }
            }
            if let Some(h) = dns_query {
                match sockets.get_mut::<dns::Socket>(dns_handle).get_query_result(h) {
                    Ok(_addrs) => {
                        nsprint!("[NETSTACK] dns OK ({} resolved)\n", DNS_NAME);
                        dns_started = true;
                        dns_query = None;
                    }
                    Err(dns::GetQueryResultError::Pending) => {}
                    Err(_) => {
                        // transient failure — allow a retry until the cap.
                        dns_query = None;
                    }
                }
            }
        }
        // --- Outbound TCP client to the host via slirp guestfwd (10.0.2.100:9999) ---
        // Retries until the host listener is up (the test starts it post-boot).
        // :9999 is THIS consumer's port and nobody else's — the harness gives every
        // independent echo consumer its own guestfwd (tests/qemu/vm.py
        // GUESTFWD_ECHO_PORTS), because a shared one hands slirp two sockets matching
        // the same (addr, port) and one of them silently eats the other's echo. This
        // retry loop re-opens forever, so sharing made it the thief in ~90% of runs.
        //
        // Without that guestfwd (every boot but the `-net` configs) nothing answers ARP
        // for 10.0.2.100, and smoltcp rate-limits neighbor discovery for the WHOLE
        // interface: one ARP request a second, any address. A connect parked in SYN-SENT
        // re-ARPs every second and starves every other new destination; 10.0.2.3 never
        // got ARPed, so every user-mode DNS lookup failed while ping and DHCP worked. So
        // a connect still in SYN-SENT after 3s means there is no guestfwd on this boot,
        // and the client gives up for good. With the guestfwd present slirp answers the
        // SYN at once (SYN-ACK, or RST while the host listener is not up yet), so this
        // never fires there.
        if dhcp_configured && !client_done && !crate::netcfg::on_slirp() {
            client_done = true;
            nsprint!("[NETSTACK] outbound TCP: skipped (the host echo exists only on QEMU's network)\n");
        }
        if dhcp_configured && !client_done {
            let s = sockets.get_mut::<tcp::Socket>(client_handle);
            let now_ms = now().total_millis() as i64;
            if s.state() == tcp::State::SynSent && now_ms - client_connect_ms > 3_000 {
                s.abort();
                client_done = true;
                nsprint!("[NETSTACK] outbound TCP: no echo host on this boot, client stopped\n");
            } else if !s.is_open() {
                let local_port = 49500 + (client_attempts % 4000) as u16;
                let _ = s.connect(
                    iface.context(),
                    (IpAddress::v4(10, 0, 2, 100), 9999),
                    local_port,
                );
                client_attempts += 1;
                client_connect_ms = now_ms;
            } else if s.may_send() && !client_sent {
                if s.send_slice(b"VIBEOS-NETSTACK-OUTBOUND\n").is_ok() {
                    client_sent = true;
                    client_sent_ms = now().total_millis() as i64;
                    client_sends += 1;
                    nsprint!("[NETSTACK] outbound TCP connected -> 10.0.2.100:9999, probe sent\n");
                }
            }
            if s.can_recv() {
                let mut tmp = [0u8; 64];
                if let Ok(n) = s.recv_slice(&mut tmp) {
                    if n > 0 {
                        nsprint!("[NETSTACK] outbound TCP echo received ({} bytes) OK\n", n);
                        client_done = true;
                        s.close();
                    }
                }
            } else if client_sent
                && client_sends < 10
                && (now().total_millis() as i64 - client_sent_ms) > 2500
            {
                // No echo within the window — retry as a fresh connection. The
                // windows-host smoke backend forwards this guestfwd through a single
                // shared chardev whose FIRST guest connection loses its return bytes
                // (later connections echo fine; native's per-connection `cmd:cat`
                // doesn't have this); and on any backend the host echo can be
                // transiently unavailable. abort() (RST -> immediate Closed, no
                // TIME-WAIT) then reconnect; the shared chardev stays up, so a guest
                // reconnect is safe.
                s.abort();
                client_sent = false;
            }
        }

        // --- TCP echo server on :7777 (reachable from host via slirp hostfwd) ---
        {
            let s = sockets.get_mut::<tcp::Socket>(tcp_handle);
            if !s.is_open() {
                if s.listen(ECHO_PORT).is_ok() && !echo_announced {
                    nsprint!("[NETSTACK] tcp echo ready\n");
                    echo_announced = true;
                }
            }
            if s.can_recv() {
                let mut tmp = [0u8; 1024];
                if let Ok(n) = s.recv_slice(&mut tmp) {
                    if n > 0 && s.can_send() {
                        let _ = s.send_slice(&tmp[..n]);
                    }
                }
            }
            // The client hung up and everything it sent has been echoed: close our
            // side too. Left in CLOSE-WAIT the socket stays open forever and never
            // listens again, so each boot answered exactly one connection.
            if s.state() == tcp::State::CloseWait && !s.can_recv() && s.send_queue() == 0 {
                s.close();
            }
        }

        // Sleep until the links' next smoltcp deadline. A pending timer caps the
        // wait at 10ms (TX completion / echo latency); when nothing is pending the
        // RX events (every link's, when every link has one) let us idle far longer,
        // since a frame wakes us instantly.
        let mut pending: Option<i64> = None;
        let mut events: [PVOID; crate::netcfg::ADAPTER_SLOTS] = [null_mut(); crate::netcfg::ADAPTER_SLOTS];
        let mut nevents = 0usize;
        let mut polled = false;
        for (i, link) in links.iter_mut().enumerate() {
            let set = if i == primary { &sockets } else { &link.own };
            if let Some(d) = link.iface.poll_delay(ts, set).map(|d| d.total_millis() as i64) {
                pending = Some(pending.map_or(d, |p| p.min(d)));
            }
            // A GEM frame still queued may be waiting on a TX restart that only a poll
            // performs (gem.rs `reap_tx`); don't let the event wait sit on it.
            // GENET's RX-done is masked by its ISR until this pass has run.
            if matches!(&link.phy, AnyPhy::Gem(g) if g.dev().tx_in_flight())
                || matches!(&link.phy, AnyPhy::Genet(g) if g.dev().rearm_irq())
            {
                pending = Some(1);
            }
            if link.rx_event.is_null() {
                polled = true;
            } else {
                events[nevents] = link.rx_event as PVOID;
                nevents += 1;
            }
        }
        if polled || nevents == 0 {
            delay_ms(pending.map(|d| d.clamp(1, 10)).unwrap_or(10));
        } else {
            // Events = primary RX wake; the timeout is the timer/TX fallback.
            // KeWait* auto-resets the synchronization events.
            let ms = pending.map(|d| d.clamp(1, 10)).unwrap_or(100);
            unsafe {
                let mut t: ntkrnl_sys::ddk::LARGE_INTEGER = core::mem::zeroed();
                t.QuadPart = -(ms * 10_000); // relative wait, 100ns units
                if nevents == 1 {
                    ntkrnl_sys::ddk::KeWaitForSingleObject(events[0], 0, 0, 0, &mut t);
                } else {
                    // WaitAny; up to THREAD_WAIT_OBJECTS (3) objects need no wait blocks.
                    ntkrnl_sys::ddk::KeWaitForMultipleObjects(
                        nevents as u32,
                        events.as_mut_ptr(),
                        1 as _,
                        0 as _, // Executive
                        0,      // KernelMode
                        0,      // not alertable
                        &mut t,
                        null_mut(),
                    );
                }
            }
        }
    }
}
