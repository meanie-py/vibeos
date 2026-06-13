//! M1 engine bring-up self-test: prove raw virtio-net frame RX/TX end to end,
//! with no smoltcp and no TDI yet.
//!
//! We send a broadcast ARP request for the slirp gateway (10.0.2.2) and poll
//! for the reply. A reply means: our frame crossed the TX virtqueue, slirp
//! processed it, and its answer crossed the RX virtqueue back to us — the full
//! device path both directions. The gateway's MAC (52:55:0a:00:02:02 under
//! slirp) is logged as proof.

use virtio_drivers::device::net::TxBuffer;

use crate::device;
use crate::serial::nsprint;

extern "system" {
    fn KeStallExecutionProcessor(MicroSeconds: u32);
}

/// Our self-assigned bring-up IP (slirp's DHCP pool starts at .15).
const OUR_IP: [u8; 4] = [10, 0, 2, 15];
/// The slirp gateway we ARP for.
const GW_IP: [u8; 4] = [10, 0, 2, 2];

pub fn run() {
    nsprint!("[NETSTACK] M1 self-test: probing for virtio-net\n");

    let mut vnet = match device::bringup() {
        Some(v) => v,
        None => {
            nsprint!("[NETSTACK] FAIL: virtio-net bringup (find/transport/init)\n");
            return;
        }
    };

    let mac = vnet.mac_address();
    nsprint!(
        "[NETSTACK] mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    let frame = build_arp_request(mac);
    if vnet.send(TxBuffer::from(&frame)).is_err() {
        nsprint!("[NETSTACK] FAIL: tx of arp request\n");
        return;
    }
    nsprint!("[NETSTACK] arp request sent (who-has 10.0.2.2)\n");

    // Poll for the reply: ~5000 * 100us = up to 500ms. slirp answers instantly.
    for _ in 0..5000 {
        if vnet.can_recv() {
            if let Ok(rx) = vnet.receive() {
                if let Some(gw_mac) = parse_arp_reply(rx.packet()) {
                    nsprint!(
                        "[NETSTACK] arp gateway 10.0.2.2 is-at \
                         {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
                        gw_mac[0], gw_mac[1], gw_mac[2], gw_mac[3], gw_mac[4], gw_mac[5]
                    );
                    nsprint!("[NETSTACK] M1 raw frame RX/TX OK\n");
                    let _ = vnet.recycle_rx_buffer(rx);
                    return;
                }
                let _ = vnet.recycle_rx_buffer(rx);
            }
        }
        unsafe { KeStallExecutionProcessor(100) };
    }
    nsprint!("[NETSTACK] FAIL: no arp reply within timeout\n");
}

/// Build a padded (60-byte) broadcast ARP request frame.
fn build_arp_request(src_mac: [u8; 6]) -> [u8; 60] {
    let mut f = [0u8; 60];
    // Ethernet header.
    f[0..6].copy_from_slice(&[0xFF; 6]); // dst = broadcast
    f[6..12].copy_from_slice(&src_mac); // src
    f[12..14].copy_from_slice(&0x0806u16.to_be_bytes()); // ethertype = ARP
    // ARP payload.
    f[14..16].copy_from_slice(&0x0001u16.to_be_bytes()); // htype = ethernet
    f[16..18].copy_from_slice(&0x0800u16.to_be_bytes()); // ptype = IPv4
    f[18] = 6; // hlen
    f[19] = 4; // plen
    f[20..22].copy_from_slice(&0x0001u16.to_be_bytes()); // oper = request
    f[22..28].copy_from_slice(&src_mac); // sender hw addr
    f[28..32].copy_from_slice(&OUR_IP); // sender proto addr
    // target hw addr stays zero
    f[38..42].copy_from_slice(&GW_IP); // target proto addr
    f
}

/// If `pkt` is an ARP reply for the gateway, return the sender's MAC.
fn parse_arp_reply(pkt: &[u8]) -> Option<[u8; 6]> {
    if pkt.len() < 42 {
        return None;
    }
    let ethertype = u16::from_be_bytes([pkt[12], pkt[13]]);
    if ethertype != 0x0806 {
        return None;
    }
    let oper = u16::from_be_bytes([pkt[20], pkt[21]]);
    if oper != 0x0002 {
        return None; // not a reply
    }
    // sender proto addr (bytes 28..32) should be the gateway.
    if pkt[28..32] != GW_IP {
        return None;
    }
    let mut mac = [0u8; 6];
    mac.copy_from_slice(&pkt[22..28]); // sender hw addr
    Some(mac)
}
