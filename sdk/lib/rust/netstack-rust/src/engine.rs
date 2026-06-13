//! M2 engine: smoltcp Interface + a poll-loop worker thread, proving ICMP ping
//! and a TCP echo server with no TDI yet.
//!
//! Architecture note: for M2 the worker thread is the SOLE owner of the
//! Interface + SocketSet + device, so no lock is needed yet. The shared lock +
//! completion bridge (design doc §"in-kernel runtime model") arrive at M4 when
//! TDI requests start touching the sockets from other threads.

use alloc::vec;
use alloc::vec::Vec;
use core::ptr::null_mut;

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::socket::{icmp, tcp};
use smoltcp::wire::{
    EthernetAddress, HardwareAddress, Icmpv4Packet, Icmpv4Repr, IpAddress, IpCidr, Ipv4Address,
};

use ntkrnl_sys::ddk::{HANDLE, PVOID, PsCreateSystemThread};

use crate::device::{self, VirtioPhy};
use crate::serial::nsprint;
use crate::time::{delay_ms, now};

const ECHO_PORT: u16 = 7777;
const OUR_IP: [u8; 4] = [10, 0, 2, 15];
const GW_IP: [u8; 4] = [10, 0, 2, 2];
const ICMP_IDENT: u16 = 0x022b;
const THREAD_ALL_ACCESS: u32 = 0x001F_03FF;

/// Spawn the netstack poll-loop worker thread. Returns immediately; the engine
/// runs for the life of the system.
pub fn start() {
    let mut handle: HANDLE = null_mut();
    let status = unsafe {
        PsCreateSystemThread(
            &mut handle,
            THREAD_ALL_ACCESS,
            null_mut(),
            null_mut(),
            null_mut(),
            Some(worker),
            null_mut(),
        )
    };
    if status < 0 {
        nsprint!("[NETSTACK] FAIL: PsCreateSystemThread {:#010x}\n", status as u32);
    }
}

unsafe extern "C" fn worker(_context: PVOID) {
    nsprint!("[NETSTACK] M2 engine: starting smoltcp poll loop\n");

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
    iface.update_ip_addrs(|addrs| {
        let _ = addrs.push(IpCidr::new(
            IpAddress::v4(OUR_IP[0], OUR_IP[1], OUR_IP[2], OUR_IP[3]),
            24,
        ));
    });
    let _ = iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(GW_IP[0], GW_IP[1], GW_IP[2], GW_IP[3]));

    let mut sockets = SocketSet::new(Vec::new());
    let tcp_handle = sockets.add(tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0u8; 4096]),
        tcp::SocketBuffer::new(vec![0u8; 4096]),
    ));
    let icmp_handle = sockets.add(icmp::Socket::new(
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 4], vec![0u8; 256]),
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 4], vec![0u8; 256]),
    ));
    sockets
        .get_mut::<icmp::Socket>(icmp_handle)
        .bind(icmp::Endpoint::Ident(ICMP_IDENT))
        .ok();

    let gw = IpAddress::v4(GW_IP[0], GW_IP[1], GW_IP[2], GW_IP[3]);
    let mut ping_sent = false;
    let mut ping_ok = false;
    let mut echo_announced = false;

    loop {
        let ts = now();
        iface.poll(ts, &mut phy, &mut sockets);

        // --- ICMP ping to the slirp gateway (also exercises ARP resolution) ---
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
