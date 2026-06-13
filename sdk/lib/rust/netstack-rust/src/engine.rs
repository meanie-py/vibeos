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

use crate::device::{self, VirtioPhy};
use crate::serial::nsprint;
use crate::time::{delay_ms, now};

const ECHO_PORT: u16 = 7777;
const GW_IP: [u8; 4] = [10, 0, 2, 2];
/// slirp's built-in DNS forwarder.
const DNS_IP: [u8; 4] = [10, 0, 2, 3];
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
    }
}

unsafe extern "system" fn worker(_context: PVOID) {
    nsprint!("[NETSTACK] M2/M3 engine: starting smoltcp poll loop\n");

    let dev = match device::bringup() {
        Some(d) => d,
        None => {
            nsprint!("[NETSTACK] FAIL: M2 device bringup\n");
            return;
        }
    };
    let mut phy = VirtioPhy::new(dev);
    let mac = phy.mac();

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
        &[IpAddress::v4(DNS_IP[0], DNS_IP[1], DNS_IP[2], DNS_IP[3])],
        Vec::new(),
    ));
    sockets
        .get_mut::<icmp::Socket>(icmp_handle)
        .bind(icmp::Endpoint::Ident(ICMP_IDENT))
        .ok();

    let gw = IpAddress::v4(GW_IP[0], GW_IP[1], GW_IP[2], GW_IP[3]);
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

    loop {
        let ts = now();
        iface.poll(ts, &mut phy, &mut sockets);

        // --- DHCPv4: obtain a lease from slirp, apply it to the interface ---
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
                    dhcp_configured = true;
                }
                dhcpv4::Event::Deconfigured => {
                    iface.update_ip_addrs(|addrs| addrs.clear());
                    let _ = iface.routes_mut().remove_default_ipv4_route();
                    dhcp_configured = false;
                }
            }
        }

        // The rest needs an address — gate on the lease.
        if dhcp_configured {
            // --- ICMP ping the gateway (also exercises ARP) ---
            {
                let s = sockets.get_mut::<icmp::Socket>(icmp_handle);
                if !ping_sent && s.can_send() {
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
                        nsprint!("[NETSTACK] icmp echo request -> 10.0.2.2\n");
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
        if dhcp_configured && !client_done {
            let s = sockets.get_mut::<tcp::Socket>(client_handle);
            if !s.is_open() {
                let local_port = 49500 + (client_attempts % 4000) as u16;
                let _ = s.connect(
                    iface.context(),
                    (IpAddress::v4(10, 0, 2, 100), 9999),
                    local_port,
                );
                client_attempts += 1;
            } else if s.may_send() && !client_sent {
                if s.send_slice(b"VIBEOS-NETSTACK-OUTBOUND\n").is_ok() {
                    client_sent = true;
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
        }

        // Sleep until smoltcp's next deadline, capped at 10ms for echo latency.
        let delay = iface
            .poll_delay(ts, &sockets)
            .map(|d| d.total_millis())
            .unwrap_or(10);
        delay_ms(delay.clamp(1, 10) as i64);
    }
}
