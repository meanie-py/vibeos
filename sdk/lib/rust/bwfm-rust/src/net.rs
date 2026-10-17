//! Ethernet frames over the chip: what a network stack hands the driver and gets back.
//!
//! The firmware is FullMAC: it takes and gives plain 802.3 frames (destination,
//! source, ethertype, payload) behind the 4-byte BCDC data header, and does the
//! 802.11 framing and the encryption itself once the keys are in. Received data
//! frames also carry the firmware's events (ethertype 0x886C); those are not
//! traffic.

use sdhci_rust::{Error, Platform};

use crate::bus::{Bus, Frame};
use crate::proto;

/// The longest Ethernet frame (no FCS) a stack may hand over: 1500 bytes of payload.
pub const ETH_FRAME_MAX: usize = 1514;

/// The Ethernet frame inside a received data frame, or None for anything else
/// (control replies, events, a frame too short to be Ethernet).
pub fn ethernet<'a>(f: &Frame<'a>) -> Option<&'a [u8]> {
    if f.channel != proto::CHANNEL_DATA {
        return None;
    }
    let (_, eth) = proto::strip_data_header(f.payload)?;
    if eth.len() < 14 || u16::from_be_bytes([eth[12], eth[13]]) == proto::ETHERTYPE_LINK_CTL {
        return None;
    }
    Some(eth)
}

/// Frames the stack has handed over, waiting for the worker that owns the bus. The
/// stack's send path may run at DISPATCH_LEVEL and must not wait on the SDIO bus, so
/// it only copies the frame in; the worker copies it out and sends it. A full ring
/// refuses the frame, and the stack's protocol retries as it would a lost one.
pub struct TxRing<const N: usize> {
    slots: [[u8; ETH_FRAME_MAX]; N],
    len: [u16; N],
    head: usize,
    count: usize,
}

impl<const N: usize> TxRing<N> {
    pub const fn new() -> Self {
        Self { slots: [[0; ETH_FRAME_MAX]; N], len: [0; N], head: 0, count: 0 }
    }

    /// Queue a copy of `eth`. False when the ring is full or the frame is no Ethernet frame.
    pub fn push(&mut self, eth: &[u8]) -> bool {
        if self.count == N || eth.len() < 14 || eth.len() > ETH_FRAME_MAX {
            return false;
        }
        let at = (self.head + self.count) % N;
        self.slots[at][..eth.len()].copy_from_slice(eth);
        self.len[at] = eth.len() as u16;
        self.count += 1;
        true
    }

    /// The oldest frame, copied into `out`; its length.
    pub fn pop(&mut self, out: &mut [u8; ETH_FRAME_MAX]) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let n = self.len[self.head] as usize;
        out[..n].copy_from_slice(&self.slots[self.head][..n]);
        self.head = (self.head + 1) % N;
        self.count -= 1;
        Some(n)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

impl<const N: usize> Default for TxRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Platform> Bus<P> {
    /// Send a whole Ethernet frame as the stack built it; its source address
    /// should be the chip's (`read_mac`).
    pub fn send_ethernet(&mut self, eth: &[u8], each: impl FnMut(Frame)) -> Result<(), Error> {
        if eth.len() < 14 || eth.len() > ETH_FRAME_MAX {
            return Err(Error::Card("not an Ethernet frame", eth.len() as u32));
        }
        let mut dst = [0u8; 6];
        dst.copy_from_slice(&eth[..6]);
        let ethertype = u16::from_be_bytes([eth[12], eth[13]]);
        self.send_data(&dst, ethertype, &eth[14..], each)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Buffers;
    use crate::chip::Chip;
    use crate::sdio::Sdio;
    use crate::sim::{Card, Sim, MAC};

    fn running() -> Bus<Sim> {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        let mut chip = Chip::attach(s).map_err(|(_, e)| e).unwrap();
        let mut fw = vec![0u8; 64];
        fw[..4].copy_from_slice(&0xB83E_F198u32.to_le_bytes());
        chip.load_firmware(&fw, &[0u8; 8]).unwrap();
        chip.start().unwrap();
        let mut bus = Bus::new(chip, Box::leak(Box::<Buffers>::default()));
        bus.read_mac(|_| {}).unwrap();
        bus
    }

    fn arp_request() -> Vec<u8> {
        let mut f = vec![0xFF; 6];
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&[0x08, 0x06]);
        f.extend_from_slice(&[0x00, 0x01, 0x08, 0x00, 6, 4, 0, 1]);
        f.extend_from_slice(&[0u8; 20]);
        f
    }

    #[test]
    fn a_stacks_frame_goes_out_whole_behind_the_data_header() {
        let mut bus = running();
        let f = arp_request();
        bus.send_ethernet(&f, |_| {}).unwrap();
        let card = bus.chip.sdio.hc.p.card();
        assert_eq!(card.tx_ethernet, vec![f], "the firmware got the frame as sent");
    }

    #[test]
    fn received_traffic_comes_out_and_events_do_not() {
        let mut bus = running();
        let mut reply = MAC.to_vec();
        reply.extend_from_slice(&[0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x06]);
        reply.extend_from_slice(&[0u8; 28]);
        bus.chip.sdio.hc.p.card_mut().queue_ethernet(&reply);
        bus.chip.sdio.hc.p.card_mut().queue_event(b"not traffic");
        let mut got = Vec::new();
        bus.poll(|f| {
            if let Some(eth) = ethernet(&f) {
                got.push(eth.to_vec());
            }
        })
        .unwrap();
        assert_eq!(got, vec![reply]);
    }

    #[test]
    fn the_ring_hands_frames_out_in_order_and_refuses_when_full() {
        let mut r: Box<TxRing<2>> = Box::default();
        let mut out = [0u8; ETH_FRAME_MAX];
        assert_eq!(r.pop(&mut out), None);
        let (a, b, c) = (vec![1u8; 60], vec![2u8; ETH_FRAME_MAX], vec![3u8; 14]);
        assert!(r.push(&a) && r.push(&b));
        assert!(!r.push(&c), "two slots");
        assert_eq!(r.pop(&mut out), Some(60));
        assert_eq!(out[..60], a[..]);
        assert!(r.push(&c), "a slot came free; the ring wraps");
        assert_eq!(r.pop(&mut out), Some(ETH_FRAME_MAX));
        assert_eq!(out[..], b[..]);
        assert_eq!(r.pop(&mut out), Some(14));
        assert_eq!((r.pop(&mut out), r.len()), (None, 0));
        assert!(!r.push(&[0u8; 13]) && !r.push(&vec![0u8; ETH_FRAME_MAX + 1]), "not Ethernet frames");
    }

    #[test]
    fn a_frame_too_long_for_ethernet_is_refused_not_sent() {
        let mut bus = running();
        assert!(bus.send_ethernet(&vec![0u8; ETH_FRAME_MAX + 1], |_| {}).is_err());
        assert!(bus.chip.sdio.hc.p.card().tx_ethernet.is_empty());
    }
}
