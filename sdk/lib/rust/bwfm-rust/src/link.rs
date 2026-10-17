//! The joined link, once the 4-way handshake is done: received traffic out to the
//! network stack, the AP's group rekeys answered, and the link's loss noticed.
//!
//! One `Bus::service` call reads what the chip has queued and deals with all of it. The
//! driver's worker calls it in a loop between sends; nothing here waits.

use core::cell::RefCell;

use sdhci_rust::{Error, Platform};

use crate::bus::{Bus, Frame};
use crate::join::{Answer, Station, ETHERTYPE_EAPOL};
use crate::net;
use crate::proto::{self, cmd, event};

/// EAPOL frames held between the read that brought them and their answer. A rekey is
/// one frame; a second is a retransmission.
const EAPOL_HELD: usize = 2;
const EAPOL_BODY_MAX: usize = 256;

/// What a `service` call found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Serviced {
    /// Nothing was queued.
    Idle,
    /// Frames were read and dealt with: there may be more.
    Busy,
    /// The firmware reported the link gone (the event, and its reason or status).
    Lost(u32, u32),
}

/// The link's counters, for the log.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinkLog {
    /// Ethernet frames handed to the stack.
    pub delivered: u32,
    /// Group keys installed after the join.
    pub rekeys: u32,
    /// EAPOL frames the supplicant refused, and the last reason.
    pub rejected: u32,
    pub last_reject: Option<wpa_rust::Error>,
}

struct Pending {
    eapol: [[u8; EAPOL_BODY_MAX]; EAPOL_HELD],
    len: [usize; EAPOL_HELD],
    n: usize,
    lost: Option<(u32, u32)>,
    delivered: u32,
}

impl Pending {
    /// Sort one received frame: traffic goes straight to `deliver`; EAPOL and the
    /// events that end the link are kept for `service`.
    fn take(&mut self, f: &Frame, deliver: &mut impl FnMut(&[u8])) {
        if let Some(eth) = net::ethernet(f) {
            if u16::from_be_bytes([eth[12], eth[13]]) == ETHERTYPE_EAPOL {
                if self.n < EAPOL_HELD {
                    let body = &eth[14..];
                    let k = body.len().min(EAPOL_BODY_MAX);
                    self.eapol[self.n][..k].copy_from_slice(&body[..k]);
                    self.len[self.n] = k;
                    self.n += 1;
                }
            } else {
                deliver(eth);
                self.delivered += 1;
            }
            return;
        }
        if f.channel != proto::CHANNEL_EVENT && f.channel != proto::CHANNEL_DATA {
            return;
        }
        let Some((_, eth)) = proto::strip_data_header(f.payload) else { return };
        let Some(e) = proto::parse_event(eth) else { return };
        match e.event_type {
            event::E_LINK if e.flags & 1 == 0 => self.lost = Some((e.event_type, e.reason)),
            event::E_DEAUTH | event::E_DEAUTH_IND | event::E_DISASSOC | event::E_DISASSOC_IND => {
                self.lost = Some((e.event_type, e.reason))
            }
            _ => {}
        }
    }
}

impl<P: Platform> Bus<P> {
    /// Read everything the chip has queued: Ethernet frames go to `deliver` (the stack's
    /// receive path), EAPOL frames to the station's supplicant, whose answers and keys go
    /// back down. Frames that arrive while an answer is being sent are dealt with too.
    pub fn service(&mut self, st: &mut Station, log: &mut LinkLog, deliver: impl FnMut(&[u8])) -> Result<Serviced, Error> {
        self.sorting(st, log, deliver, |bus, each| bus.poll(each).map(|p| p.frames))
    }

    /// Send one of the stack's frames. What the chip hands back while it goes out
    /// (traffic, a rekey, the link's end) is dealt with as `service` would.
    pub fn send_traffic(
        &mut self,
        st: &mut Station,
        log: &mut LinkLog,
        eth: &[u8],
        deliver: impl FnMut(&[u8]),
    ) -> Result<Serviced, Error> {
        self.sorting(st, log, deliver, |bus, each| bus.send_ethernet(eth, each).map(|()| 0))
    }

    /// Leave the network: the firmware drops its association, if it holds one, before
    /// the next join. Refusal means it held none.
    pub fn leave(&mut self) -> Result<(), Error> {
        self.dcmd(cmd::C_DISASSOC, true, &[], &mut [], |_| {}).map(|_| ())
    }

    /// The rate the firmware transmits at, in bits per second.
    pub fn rate(&mut self) -> Result<u64, Error> {
        let mut r = [0u8; 4];
        self.dcmd(cmd::C_GET_RATE, false, &[&[0; 4]], &mut r, |_| {})?;
        // Units of 500 kb/s.
        Ok(u32::from_le_bytes(r) as u64 * 500_000)
    }

    /// Run `io` with every frame it reads sorted (traffic to `deliver`, EAPOL and the
    /// link's end kept), then answer the EAPOL. `io` returns how many frames it read.
    fn sorting(
        &mut self,
        st: &mut Station,
        log: &mut LinkLog,
        mut deliver: impl FnMut(&[u8]),
        io: impl FnOnce(&mut Self, &mut dyn FnMut(Frame)) -> Result<u32, Error>,
    ) -> Result<Serviced, Error> {
        let pending = RefCell::new(Pending {
            eapol: [[0; EAPOL_BODY_MAX]; EAPOL_HELD],
            len: [0; EAPOL_HELD],
            n: 0,
            lost: None,
            delivered: 0,
        });
        let mut each = |f: Frame| pending.borrow_mut().take(&f, &mut deliver);
        let frames = io(self, &mut each)?;
        let mut frame = [0u8; EAPOL_BODY_MAX];
        loop {
            let len = {
                let mut p = pending.borrow_mut();
                if p.n == 0 {
                    break;
                }
                let len = p.len[0];
                frame[..len].copy_from_slice(&p.eapol[0][..len]);
                p.eapol.copy_within(1.., 0);
                p.len.copy_within(1.., 0);
                p.n -= 1;
                len
            };
            match self.answer_eapol(st, &frame[..len], &mut each)? {
                Answer::Rejected(e) => {
                    log.rejected += 1;
                    log.last_reject = Some(e);
                }
                Answer::Answered { group, .. } => log.rekeys += group as u32,
            }
        }
        let p = pending.into_inner();
        log.delivered += p.delivered;
        Ok(match p.lost {
            Some((e, r)) => Serviced::Lost(e, r),
            None if frames > 0 => Serviced::Busy,
            None => Serviced::Idle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Buffers;
    use crate::chip::Chip;
    use crate::join::{JoinEnd, JoinLog, WifiConfig};
    use crate::scan::Bss;
    use crate::sdio::Sdio;
    use crate::sim::{Card, Sim, AP_PASS, AP_SSID, MAC};

    fn joined() -> (Bus<Sim>, Station) {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        let mut chip = Chip::attach(s).map_err(|(_, e)| e).unwrap();
        let mut fw = vec![0u8; 64];
        fw[..4].copy_from_slice(&0xB83E_F198u32.to_le_bytes());
        chip.load_firmware(&fw, &[0u8; 8]).unwrap();
        chip.start().unwrap();
        let mut bus = Bus::new(chip, Box::leak(Box::<Buffers>::default()));
        bus.read_mac(|_| {}).unwrap();
        bus.bring_up(|_| {}).unwrap();
        let mut best: Option<Bss> = None;
        bus.scan(|b| {
            if b.ssid() == AP_SSID {
                best = Some(*b);
            }
        })
        .unwrap();
        let text = [b"ssid=".as_slice(), AP_SSID, b"\npsk=", AP_PASS, b"\n"].concat();
        let cfg = WifiConfig::parse(&text).unwrap();
        let (end, st) = bus.join_station(&cfg, &best.unwrap(), &mut JoinLog::default(), |_| {}).unwrap();
        assert_eq!(end, JoinEnd::Up);
        (bus, st.expect("a station once the link is up"))
    }

    fn ping_reply() -> Vec<u8> {
        let mut f = MAC.to_vec();
        f.extend_from_slice(&[0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x00]);
        f.extend_from_slice(&[0x45; 84]);
        f
    }

    #[test]
    fn traffic_reaches_the_stack_and_an_idle_chip_says_so() {
        let (mut bus, mut st) = joined();
        let mut log = LinkLog::default();
        assert_eq!(bus.service(&mut st, &mut log, |_| panic!("nothing queued")).unwrap(), Serviced::Idle);
        let f = ping_reply();
        bus.chip.sdio.hc.p.card_mut().queue_ethernet(&f);
        let mut got = Vec::new();
        assert_eq!(bus.service(&mut st, &mut log, |e| got.push(e.to_vec())).unwrap(), Serviced::Busy);
        assert_eq!(got, vec![f]);
        assert_eq!(log.delivered, 1);
    }

    #[test]
    fn a_group_rekey_is_answered_and_its_key_installed() {
        let (mut bus, mut st) = joined();
        let mut log = LinkLog::default();
        bus.chip.sdio.hc.p.card_mut().ap_rekeys([0x52; 16], 2);
        let mut got = 0;
        bus.service(&mut st, &mut log, |_| got += 1).unwrap();
        assert_eq!((log.rekeys, log.rejected, got), (1, 0, 0), "EAPOL is not traffic");
        let card = bus.chip.sdio.hc.p.card();
        assert_eq!(card.ap_log[card.ap_log.len() - 2..], ["group msg2 ok", "group key"], "the answer, then the key");
        assert_eq!(card.group_keys.last(), Some(&(2, vec![0x52; 16])), "the new GTK, under its key id");
    }

    #[test]
    fn a_replayed_rekey_is_refused_and_nothing_goes_in() {
        let (mut bus, mut st) = joined();
        let mut log = LinkLog::default();
        bus.chip.sdio.hc.p.card_mut().ap_rekeys([0x52; 16], 2);
        bus.service(&mut st, &mut log, |_| {}).unwrap();
        let keys = bus.chip.sdio.hc.p.card().group_keys.len();
        bus.chip.sdio.hc.p.card_mut().ap_replays_rekey();
        bus.service(&mut st, &mut log, |_| {}).unwrap();
        assert_eq!((log.rekeys, log.rejected, log.last_reject), (1, 1, Some(wpa_rust::Error::Replayed)));
        assert_eq!(bus.chip.sdio.hc.p.card().group_keys.len(), keys);
    }

    #[test]
    fn the_stacks_frame_goes_out_and_the_link_stays() {
        let (mut bus, mut st) = joined();
        let mut log = LinkLog::default();
        let mut f = vec![0xFF; 6];
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&[0x08, 0x06]);
        f.extend_from_slice(&[0u8; 28]);
        assert_eq!(bus.send_traffic(&mut st, &mut log, &f, |_| {}).unwrap(), Serviced::Idle);
        assert_eq!(bus.chip.sdio.hc.p.card().tx_ethernet.last(), Some(&f));
    }

    #[test]
    fn the_ap_leaving_ends_the_link_and_traffic_before_it_still_arrives() {
        let (mut bus, mut st) = joined();
        let mut log = LinkLog::default();
        let f = ping_reply();
        bus.chip.sdio.hc.p.card_mut().queue_ethernet(&f);
        bus.chip.sdio.hc.p.card_mut().ap_deauths(7);
        let mut got = 0;
        let r = bus.service(&mut st, &mut log, |_| got += 1).unwrap();
        assert_eq!((r, got), (Serviced::Lost(event::E_DEAUTH_IND, 7), 1));
    }
}
