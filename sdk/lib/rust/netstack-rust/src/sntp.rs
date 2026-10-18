//! The system time from the network (SNTP, RFC 4330), once a lease is up.
//!
//! A board without a battery-backed clock (both Pis) boots at the HAL's fallback
//! date, so wall-clock time is only right once something sets it. The server is the
//! lease's NTP server (DHCP option 42) if it names one, else `pool.ntp.org` through
//! the lease's DNS. Synced again every `RESYNC_MS`; a failure retries with backoff.

mod wire;

use alloc::vec;

use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::{dns, udp};
use smoltcp::wire::{DhcpOption, DnsQueryType, IpAddress, Ipv4Address};

use ntkrnl_sys::ddk::{RtlTimeToTimeFields, ZwSetSystemTime, LARGE_INTEGER, TIME_FIELDS};

use crate::serial::nsprint;

/// DHCP option 42, Network Time Protocol Servers.
pub const OPT_NTP_SERVERS: u8 = 42;
/// What the DHCP client asks the server for: smoltcp's default (mask, router, DNS)
/// and the NTP servers.
pub static DHCP_PARAMETERS: [u8; 4] = [1, 3, 6, OPT_NTP_SERVERS];

const POOL: &str = "pool.ntp.org";
const LOCAL_PORT: u16 = 49123;
const REPLY_WAIT_MS: i64 = 3_000;
const RETRY_MIN_MS: i64 = 5_000;
const RETRY_MAX_MS: i64 = 300_000;
const RESYNC_MS: i64 = 6 * 3600 * 1000;
/// Failures logged before the retries go quiet.
const LOUD_FAILURES: u32 = 3;

enum State {
    Idle,
    Resolving(dns::QueryHandle),
    Waiting { server: Ipv4Address, sent_ms: i64, nonce: u64 },
}

pub struct Sntp {
    udp: SocketHandle,
    state: State,
    lease_server: Option<Ipv4Address>,
    next_ms: i64,
    failures: u32,
    seed: u64,
}

impl Sntp {
    pub fn new(sockets: &mut SocketSet<'static>, seed: u64) -> Self {
        let mut s = udp::Socket::new(
            udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 2], vec![0u8; 2 * wire::LEN]),
            udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 2], vec![0u8; 2 * wire::LEN]),
        );
        let _ = s.bind(LOCAL_PORT);
        Sntp { udp: sockets.add(s), state: State::Idle, lease_server: None, next_ms: 0, failures: 0, seed: seed | 1 }
    }

    /// The first NTP server a DHCP packet's option 42 names.
    pub fn lease_server<'p>(mut options: impl Iterator<Item = DhcpOption<'p>>) -> Option<Ipv4Address> {
        let d = options.find(|o| o.kind == OPT_NTP_SERVERS && o.data.len() >= 4)?.data;
        Some(Ipv4Address::new(d[0], d[1], d[2], d[3]))
    }

    /// A new lease: sync now, from `server` (its option 42) if it named one.
    pub fn on_lease(&mut self, server: Option<Ipv4Address>) {
        self.lease_server = server;
        self.state = State::Idle;
        self.next_ms = 0;
        self.failures = 0;
    }

    /// Advance the exchange; called from the poll loop while the lease is held.
    pub fn poll(&mut self, iface: &mut Interface, sockets: &mut SocketSet<'static>, dns: SocketHandle, now_ms: i64) {
        match self.state {
            State::Idle if now_ms >= self.next_ms => match self.lease_server {
                Some(server) => self.send(sockets, server, now_ms),
                None => match sockets.get_mut::<dns::Socket>(dns).start_query(iface.context(), POOL, DnsQueryType::A) {
                    Ok(q) => self.state = State::Resolving(q),
                    Err(_) => self.fail(now_ms, "no DNS query slot", None, None),
                },
            },
            State::Idle => {}
            State::Resolving(q) => match sockets.get_mut::<dns::Socket>(dns).get_query_result(q) {
                Ok(addrs) => match addrs.iter().find_map(|a| match a {
                    IpAddress::Ipv4(v4) => Some(*v4),
                    #[allow(unreachable_patterns)]
                    _ => None,
                }) {
                    Some(server) => self.send(sockets, server, now_ms),
                    None => self.fail(now_ms, "pool.ntp.org has no IPv4 address", None, None),
                },
                Err(dns::GetQueryResultError::Pending) => return, // still resolving
                Err(_) => self.fail(now_ms, "pool.ntp.org did not resolve", None, None),
            },
            State::Waiting { server, sent_ms, nonce } => {
                let s = sockets.get_mut::<udp::Socket>(self.udp);
                let mut buf = [0u8; wire::LEN];
                while let Ok((n, meta)) = s.recv_slice(&mut buf) {
                    if meta.endpoint.addr != IpAddress::Ipv4(server) || meta.endpoint.port != wire::PORT {
                        continue;
                    }
                    match wire::reply(&buf[..n], nonce) {
                        Ok(t) => return self.set(server, t + ((now_ms - sent_ms) as u64 * 10_000) / 2, now_ms),
                        Err(wire::Reject::NotOurs) => continue,
                        Err(why) => return self.fail(now_ms, "reply refused", Some(server), Some(why)),
                    }
                }
                if now_ms - sent_ms > REPLY_WAIT_MS {
                    self.fail(now_ms, "no reply", Some(server), None);
                }
            }
        }
    }

    fn send(&mut self, sockets: &mut SocketSet<'static>, server: Ipv4Address, now_ms: i64) {
        // xorshift64: a nonce an off-path sender cannot guess from the last one's timing.
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        let nonce = self.seed;
        let s = sockets.get_mut::<udp::Socket>(self.udp);
        while s.recv().is_ok() {} // a late reply to an abandoned request
        match s.send_slice(&wire::request(nonce), (IpAddress::Ipv4(server), wire::PORT)) {
            Ok(()) => self.state = State::Waiting { server, sent_ms: now_ms, nonce },
            Err(_) => self.fail(now_ms, "send failed", Some(server), None),
        }
    }

    fn set(&mut self, server: Ipv4Address, nt: u64, now_ms: i64) {
        let mut t: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        let mut was: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        t.QuadPart = nt as i64;
        let status = unsafe { ZwSetSystemTime(&mut t, &mut was) };
        if status < 0 {
            nsprint!("[NETSTACK] sntp: setting the time failed {:#x}\n", status as u32);
            self.state = State::Idle;
            self.next_ms = now_ms + RETRY_MAX_MS;
            return;
        }
        let mut f: TIME_FIELDS = unsafe { core::mem::zeroed() };
        unsafe { RtlTimeToTimeFields(&mut t, &mut f) };
        let off = (nt as i64 - unsafe { was.QuadPart }) / 10_000_000;
        let o = server.octets();
        nsprint!(
            "[NETSTACK] sntp: time set from {}.{}.{}.{}: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC (clock was {}s off)\n",
            o[0], o[1], o[2], o[3], f.Year, f.Month, f.Day, f.Hour, f.Minute, f.Second, off
        );
        self.state = State::Idle;
        self.failures = 0;
        self.next_ms = now_ms + RESYNC_MS;
    }

    fn fail(&mut self, now_ms: i64, what: &str, server: Option<Ipv4Address>, why: Option<wire::Reject>) {
        self.failures += 1;
        if self.failures <= LOUD_FAILURES {
            match (server, why) {
                (Some(s), Some(why)) => {
                    let o = s.octets();
                    nsprint!("[NETSTACK] sntp: {} from {}.{}.{}.{} ({:?})\n", what, o[0], o[1], o[2], o[3], why)
                }
                (Some(s), None) => {
                    let o = s.octets();
                    nsprint!("[NETSTACK] sntp: {} from {}.{}.{}.{}\n", what, o[0], o[1], o[2], o[3])
                }
                _ => nsprint!("[NETSTACK] sntp: {}\n", what),
            }
        }
        self.state = State::Idle;
        let shift = (self.failures - 1).min(6);
        self.next_ms = now_ms + (RETRY_MIN_MS << shift).min(RETRY_MAX_MS);
    }
}
