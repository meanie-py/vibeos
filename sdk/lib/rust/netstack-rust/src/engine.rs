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

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::socket::{dhcpv4, dns, icmp, tcp};
use smoltcp::wire::{
    DnsQueryType, EthernetAddress, HardwareAddress, Icmpv4Packet, Icmpv4Repr, IpAddress, IpCidr,
};

use ntkrnl_sys::ddk::{HANDLE, PVOID, PsCreateSystemThread};

#[cfg(not(feature = "ndis"))]
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
#[cfg(not(feature = "ndis"))]
enum AnyPhy {
    Virtio(VirtioPhy),
    E1000(crate::e1000::E1000Phy),
    Genet(crate::genet::GenetPhy),
    Gem(crate::gem::GemPhy),
}

#[cfg(not(feature = "ndis"))]
impl AnyPhy {
    fn mac(&self) -> [u8; 6] {
        match self {
            AnyPhy::Virtio(v) => v.mac(),
            AnyPhy::E1000(e) => e.mac(),
            AnyPhy::Genet(g) => g.mac(),
            AnyPhy::Gem(g) => g.mac(),
        }
    }
}

#[cfg(not(feature = "ndis"))]
enum AnyRxToken {
    Virtio(crate::device::VirtioRxToken),
    E1000(crate::e1000::E1000RxToken),
    Genet(crate::genet::GenetRxToken),
    Gem(crate::gem::GemRxToken),
}

#[cfg(not(feature = "ndis"))]
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
        }
    }
}

#[cfg(not(feature = "ndis"))]
enum AnyTxToken<'a> {
    Virtio(crate::device::VirtioTxToken<'a>),
    E1000(crate::e1000::E1000TxToken<'a>),
    Genet(crate::genet::GenetTxToken<'a>),
    Gem(crate::gem::GemTxToken<'a>),
}

#[cfg(not(feature = "ndis"))]
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
        }
    }
}

#[cfg(not(feature = "ndis"))]
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
        }
    }

    fn transmit(&mut self, ts: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        match self {
            AnyPhy::Virtio(v) => v.transmit(ts).map(AnyTxToken::Virtio),
            AnyPhy::E1000(e) => e.transmit(ts).map(AnyTxToken::E1000),
            AnyPhy::Genet(g) => g.transmit(ts).map(AnyTxToken::Genet),
            AnyPhy::Gem(g) => g.transmit(ts).map(AnyTxToken::Gem),
        }
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        match self {
            AnyPhy::Virtio(v) => v.capabilities(),
            AnyPhy::E1000(e) => e.capabilities(),
            AnyPhy::Genet(g) => g.capabilities(),
            AnyPhy::Gem(g) => g.capabilities(),
        }
    }
}

unsafe extern "system" fn worker(_context: PVOID) {
    nsprint!("[NETSTACK] M2/M3 engine: starting smoltcp poll loop\n");

    // Device backend: virtio-net (default) or — with `feature = "ndis"` — an NDIS
    // protocol shim above a real miniport (M11). Both expose the same phy::Device
    // surface, so the poll loop below is backend-agnostic.
    #[cfg(not(feature = "ndis"))]
    let mut phy = match device::bringup() {
        Some(d) => AnyPhy::Virtio(VirtioPhy::new(d)),
        None => match crate::e1000::bringup() {
            Some(d) => AnyPhy::E1000(crate::e1000::E1000Phy::new(d)),
            // No PCI NIC: a platform NIC (pnp.rs) starts moments after
            // DriverEntry when the machine has one.
            None => match crate::pnp::wait().and_then(|nic| match nic.kind {
                crate::pnp::NicKind::Genet => {
                    crate::genet::bringup(nic).map(|d| AnyPhy::Genet(crate::genet::GenetPhy::new(d)))
                }
                crate::pnp::NicKind::Gem => crate::gem::bringup(nic).map(|d| AnyPhy::Gem(crate::gem::GemPhy::new(d))),
            }) {
                Some(phy) => phy,
                None => {
                    nsprint!("[NETSTACK] FAIL: M2 device bringup\n");
                    crate::tditrans::mark_engine_dead();
                    return;
                }
            },
        },
    };
    #[cfg(feature = "ndis")]
    let mut phy = match crate::ndisphy::bringup() {
        Some(p) => p,
        None => {
            nsprint!("[NETSTACK] FAIL: M11 NDIS adapter bringup\n");
            crate::tditrans::mark_engine_dead();
            return;
        }
    };
    let mac = phy.mac();
    // Publish the MAC for the M9 management plane (iphlpapi IFEntry queries).
    crate::netcfg::set_mac(mac);

    // M10 (x86 — i386 + amd64): connect the virtio INTx so an arriving frame wakes
    // this worker via an event instead of pure busy-polling. Null on any failure
    // (or arm64) → the loop keeps the `delay_ms` poll fallback, no regression. The
    // NDIS backend (M11) drives RX from ProtocolReceive instead, so this is gated
    // to the virtio path.
    #[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), not(feature = "ndis")))]
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
        // e1000 v1 is pure-poll: interrupts stay masked (IMC), the poll loop's
        // delay_ms cadence drives RX. No regression vs the no-IRQ fallback path.
        AnyPhy::E1000(_) | AnyPhy::Genet(_) | AnyPhy::Gem(_) => null_mut(),
    };

    let mut config = Config::new(HardwareAddress::Ethernet(EthernetAddress(mac)));
    config.random_seed = (now().total_millis() as u64) ^ 0x9E37_79B9_7F4A_7C15;
    let mut iface = Interface::new(config, &mut phy, now());
    // No static IP — DHCP assigns it.

    let mut sockets = SocketSet::new(Vec::new());
    let dhcp_handle = sockets.add(dhcpv4::Socket::new());
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
        &[], // the lease's servers, set when it binds
        Vec::new(),
    ));
    sockets
        .get_mut::<icmp::Socket>(icmp_handle)
        .bind(icmp::Endpoint::Ident(ICMP_IDENT))
        .ok();

    // The boot ping goes to the lease's router and the DNS self-test to the lease's
    // servers: slirp's 10.0.2.2 / 10.0.2.3 under QEMU, the LAN's own on a board.
    let mut gw: Option<IpAddress> = None;
    let mut dhcp_configured = false;
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
    let mut warm: [[u8; 4]; WARM_MAX] = [[0; 4]; WARM_MAX];
    let mut nwarm = 0usize;
    let mut warm_ip = [0u8; 4];
    let mut warm_next_ms: i64 = 0;
    // TDI transport state (connections/addresses driven by AFD via tdidev).
    let mut tdi_state = crate::tditrans::TdiState::new();

    loop {
        let ts = now();
        iface.poll(ts, &mut phy, &mut sockets);

        // TDI data path: drain AFD's commands + run the completion bridge.
        crate::tditrans::pump(&mut iface, &mut sockets, &mut tdi_state);

        // --- DHCPv4: obtain a lease from slirp, apply it to the interface ---
        // The lease's DNS servers, handed to the DNS socket once `sockets` is free.
        let mut lease_dns: Option<Vec<IpAddress>> = None;
        if let Some(event) = sockets.get_mut::<dhcpv4::Socket>(dhcp_handle).poll() {
            match event {
                dhcpv4::Event::Configured(cfg) => {
                    iface.update_ip_addrs(|addrs| {
                        addrs.clear();
                        let _ = addrs.push(IpCidr::Ipv4(cfg.address));
                    });
                    if let Some(router) = cfg.router {
                        let _ = iface.routes_mut().add_default_ipv4_route(router);
                    }
                    let o = cfg.address.address().octets();
                    nsprint!("[NETSTACK] dhcp lease {}.{}.{}.{}\n", o[0], o[1], o[2], o[3]);
                    // Publish L3 config for the M9 management plane (ipconfig /
                    // GetAdaptersInfo / GetIpAddrTable read it via the MIB queries).
                    crate::netcfg::set_ipv4(
                        o,
                        cfg.address.prefix_len(),
                        cfg.router.map(|r| r.octets()),
                    );
                    let mut dns = [[0u8; 4]; 4];
                    let mut ndns = 0;
                    for server in cfg.dns_servers.iter().take(dns.len()) {
                        dns[ndns] = server.octets();
                        ndns += 1;
                    }
                    crate::netcfg::publish_dns_servers(&dns[..ndns]);
                    gw = cfg.router.map(IpAddress::Ipv4);
                    lease_dns =
                        Some(dns[..ndns].iter().map(|o| IpAddress::v4(o[0], o[1], o[2], o[3])).collect());
                    // Resolve the gateway and the on-link DNS servers ourselves, now and
                    // every WARM_PERIOD_MS (see warm_neighbors).
                    nwarm = 0;
                    if let Some(router) = cfg.router {
                        warm[nwarm] = router.octets();
                        nwarm += 1;
                    }
                    for server in &dns[..ndns] {
                        let on_link = cfg
                            .address
                            .contains_addr(&smoltcp::wire::Ipv4Address::from(*server));
                        if on_link && nwarm < WARM_MAX && !warm[..nwarm].contains(server) {
                            warm[nwarm] = *server;
                            nwarm += 1;
                        }
                    }
                    warm_ip = o;
                    warm_next_ms = 0;
                    dhcp_configured = true;
                    DHCP_READY.store(true, core::sync::atomic::Ordering::Release);
                }
                dhcpv4::Event::Deconfigured => {
                    iface.update_ip_addrs(|addrs| addrs.clear());
                    let _ = iface.routes_mut().remove_default_ipv4_route();
                    dhcp_configured = false;
                }
            }
        }
        if let Some(servers) = lease_dns {
            sockets.get_mut::<dns::Socket>(dns_handle).update_servers(&servers);
        }

        // The rest needs an address — gate on the lease.
        if dhcp_configured && nwarm > 0 && (ts.total_millis() as i64) >= warm_next_ms {
            warm_neighbors(&mut phy, ts, mac, warm_ip, &warm[..nwarm]);
            warm_next_ms = ts.total_millis() as i64 + WARM_PERIOD_MS;
        }
        if dhcp_configured {
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

        // Sleep until smoltcp's next deadline. A pending timer caps the wait at
        // 10ms (TX completion / echo latency); when nothing is pending the RX
        // event (amd64) lets us idle far longer since a frame wakes us instantly.
        let pending = iface.poll_delay(ts, &sockets).map(|d| d.total_millis() as i64);
        #[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), not(feature = "ndis")))]
        {
            if rx_event.is_null() {
                delay_ms(pending.map(|d| d.clamp(1, 10)).unwrap_or(10));
            } else {
                // Event = primary RX wake; the timeout is the timer/TX fallback.
                // KeWaitForSingleObject auto-resets the synchronization event.
                let ms = pending.map(|d| d.clamp(1, 10)).unwrap_or(100);
                unsafe {
                    let mut t: ntkrnl_sys::ddk::LARGE_INTEGER = core::mem::zeroed();
                    t.QuadPart = -(ms * 10_000); // relative wait, 100ns units
                    ntkrnl_sys::ddk::KeWaitForSingleObject(
                        rx_event as ntkrnl_sys::ddk::PVOID,
                        0, // Executive
                        0, // KernelMode
                        0, // not alertable
                        &mut t,
                    );
                }
            }
        }
        #[cfg(any(not(any(target_arch = "x86_64", target_arch = "x86")), feature = "ndis"))]
        delay_ms(pending.map(|d| d.clamp(1, 10)).unwrap_or(10));
    }
}
