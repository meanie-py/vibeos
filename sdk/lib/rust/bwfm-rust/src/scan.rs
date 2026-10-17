//! Bringing the interface up and scanning: the event mask, `escan`, and the BSS
//! lists its `E_ESCAN_RESULT` events carry.
//!
//! The firmware scans on its own once asked (`escan` with version 1 parameters,
//! `bwfm_scan_v0`); each network it hears comes back as a PARTIAL result event
//! holding `bwfm_bss_info` records, and a SUCCESS event ends the scan. Event
//! headers are big-endian; the results inside them are the firmware's own
//! little-endian. After OpenBSD's bwfm.c (`bwfm_init`, `bwfm_scan_v0`,
//! `bwfm_rx_event_cb`) and bwfmreg.h, ISC (notice in chip.rs).

use core::cell::Cell;

use sdhci_rust::{Error, Platform};

use crate::bus::{Bus, Frame};
use crate::proto::{self, cmd, event};

/// One bit per event, events 0 to 138 (`BWFM_EVENT_MASK_LEN`).
pub const EVENT_MASK_LEN: usize = 18;
/// The events a station needs (`bwfm_init`, station mode).
pub const STATION_EVENTS: [u32; 8] = [
    event::E_IF,
    event::E_LINK,
    event::E_AUTH,
    event::E_ASSOC,
    event::E_EAPOL_MSG,
    event::E_DEAUTH,
    event::E_DISASSOC,
    event::E_ESCAN_RESULT,
];

pub const E_STATUS_SUCCESS: u32 = 0;
pub const E_STATUS_PARTIAL: u32 = 8;

const ESCAN_REQ_VERSION: u32 = 1;
const ESCAN_ACTION_START: u16 = 1;
const BSSTYPE_ANY: u8 = 2;
const SCANTYPE_PASSIVE: u8 = 1;
/// `bwfm_escan_params_v0`: version, action, sync id, then `bwfm_scan_params_v0`
/// (an SSID, a BSSID, BSS and scan types, four times, the channel count).
pub const ESCAN_PARAMS_LEN: usize = 72;
const ESCAN_RESULTS_HEADER: usize = 12;
/// A `bwfm_bss_info` up to `ie_length`; a record is at least this long.
const BSS_INFO_MIN: usize = 124;
/// How long a scan may take: every 2.4 and 5 GHz channel, passively.
const SCAN_WAIT_US: u64 = 15_000_000;

pub fn event_mask(events: &[u32]) -> [u8; EVENT_MASK_LEN] {
    let mut m = [0u8; EVENT_MASK_LEN];
    for &e in events {
        if (e as usize) < EVENT_MASK_LEN * 8 {
            m[e as usize / 8] |= 1 << (e % 8);
        }
    }
    m
}

/// A passive scan of every channel for any network.
pub fn escan_params(sync_id: u16) -> [u8; ESCAN_PARAMS_LEN] {
    let mut p = [0u8; ESCAN_PARAMS_LEN];
    p[0..4].copy_from_slice(&ESCAN_REQ_VERSION.to_le_bytes());
    p[4..6].copy_from_slice(&ESCAN_ACTION_START.to_le_bytes());
    p[6..8].copy_from_slice(&sync_id.to_le_bytes());
    // scan_params at 8: ssid {len, [32]} zero = any; bssid at 44 = broadcast.
    p[44..50].fill(0xFF);
    p[50] = BSSTYPE_ANY;
    p[51] = SCANTYPE_PASSIVE;
    // nprobes, active, passive and home time: -1, the firmware's defaults.
    p[52..68].fill(0xFF);
    // channel_num at 68: no SSIDs, no channels, so all of them.
    p
}

/// A network the scan heard.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bss {
    pub bssid: [u8; 6],
    pub ssid: [u8; 32],
    pub ssid_len: usize,
    pub channel: u8,
    pub rssi: i16,
    pub capability: u16,
}

impl Bss {
    pub fn ssid(&self) -> &[u8] {
        &self.ssid[..self.ssid_len]
    }
}

/// The BSS records in an escan result's data.
pub fn escan_bss(data: &[u8]) -> impl Iterator<Item = Bss> + '_ {
    let count = if data.len() >= ESCAN_RESULTS_HEADER { u16::from_le_bytes([data[10], data[11]]) } else { 0 };
    let mut at = ESCAN_RESULTS_HEADER;
    (0..count).map_while(move |_| {
        let rec = data.get(at..)?;
        if rec.len() < BSS_INFO_MIN {
            return None;
        }
        let length = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]) as usize;
        if length < BSS_INFO_MIN || length > rec.len() {
            return None;
        }
        at += length;
        let mut b = Bss::default();
        b.bssid.copy_from_slice(&rec[8..14]);
        b.capability = u16::from_le_bytes([rec[16], rec[17]]);
        b.ssid_len = (rec[18] as usize).min(32);
        b.ssid[..b.ssid_len].copy_from_slice(&rec[19..19 + b.ssid_len]);
        let chanspec = u16::from_le_bytes([rec[72], rec[73]]);
        b.rssi = i16::from_le_bytes([rec[78], rec[79]]);
        b.channel = if rec[88] != 0 { rec[88] } else { chanspec as u8 };
        Some(b)
    })
}

/// How a scan ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScanEnd {
    /// SUCCESS: every channel done.
    Complete,
    /// Another final status (`BWFM_E_STATUS_*`).
    Status(u32),
}

/// Pull the events out of a frame: `on(event type, status, data)`.
fn events(f: &Frame, mut on: impl FnMut(u32, u32, &[u8])) {
    if f.channel != proto::CHANNEL_EVENT && f.channel != proto::CHANNEL_DATA {
        return;
    }
    if let Some((_, eth)) = proto::strip_data_header(f.payload) {
        if let Some(e) = proto::parse_event(eth) {
            on(e.event_type, e.status, e.data);
        }
    }
}

impl<P: Platform> Bus<P> {
    fn set_int(&mut self, command: u32, v: u32, each: impl FnMut(Frame)) -> Result<(), Error> {
        self.dcmd(command, true, &[&v.to_le_bytes()], &mut [], each).map(|_| ())
    }

    /// Ask for the station events, bring the interface up in infrastructure
    /// mode (`bwfm_init`, the parts a scan needs).
    pub fn bring_up(&mut self, mut each: impl FnMut(Frame)) -> Result<(), Error> {
        self.set_var(b"mpc", &1u32.to_le_bytes(), &mut each)?;
        self.set_var(b"event_msgs", &event_mask(&STATION_EVENTS), &mut each)?;
        self.set_int(cmd::C_UP, 0, &mut each)?;
        self.set_int(cmd::C_SET_INFRA, 1, &mut each)
    }

    /// Scan every channel, handing each network heard to `found` (a network
    /// heard on several channels or twice comes once per report).
    pub fn scan(&mut self, mut found: impl FnMut(&Bss)) -> Result<ScanEnd, Error> {
        let end: Cell<Option<ScanEnd>> = Cell::new(None);
        let mut on_frame = |f: Frame| {
            events(&f, |kind, status, data| {
                if kind != event::E_ESCAN_RESULT || end.get().is_some() {
                    return;
                }
                match status {
                    E_STATUS_PARTIAL => escan_bss(data).for_each(|b| found(&b)),
                    E_STATUS_SUCCESS => end.set(Some(ScanEnd::Complete)),
                    s => end.set(Some(ScanEnd::Status(s))),
                }
            })
        };
        self.set_var(b"escan", &escan_params(0x1234), &mut on_frame)?;
        let start = self.chip.sdio.hc.p.now_us();
        loop {
            self.poll(&mut on_frame)?;
            if let Some(e) = end.get() {
                return Ok(e);
            }
            if self.chip.sdio.hc.p.now_us().wrapping_sub(start) > SCAN_WAIT_US {
                return Err(Error::Card("the scan never finished", 0));
            }
            self.chip.sdio.hc.p.stall_us(10_000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::Chip;
    use crate::sdio::Sdio;
    use crate::sim::{Card, Sim};

    fn running() -> Bus<Sim> {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        let mut chip = Chip::attach(s).map_err(|(_, e)| e).unwrap();
        let mut fw = vec![0u8; 64];
        fw[..4].copy_from_slice(&0xB83E_F198u32.to_le_bytes());
        chip.load_firmware(&fw, &[0u8; 8]).unwrap();
        chip.start().unwrap();
        Bus::new(chip, Box::leak(Box::default()))
    }

    #[test]
    fn the_mask_sets_one_bit_per_event() {
        let m = event_mask(&[0, 16, 69, 138]);
        assert_eq!((m[0], m[2], m[8], m[17]), (1, 1, 1 << 5, 1 << 2));
        assert_eq!(m.iter().map(|b| b.count_ones()).sum::<u32>(), 4);
    }

    #[test]
    fn escan_v0_parameters_lay_out_as_the_firmware_reads_them() {
        let p = escan_params(0x1234);
        assert_eq!(&p[0..8], &[1, 0, 0, 0, 1, 0, 0x34, 0x12]);
        assert!(p[8..44].iter().all(|&b| b == 0), "any SSID");
        assert_eq!(&p[44..52], &[0xFF; 6].iter().copied().chain([2, 1]).collect::<Vec<_>>()[..]);
        assert_eq!(&p[52..68], &[0xFF; 16]);
        assert_eq!(&p[68..72], &[0; 4], "all channels");
    }

    #[test]
    fn a_scan_reports_each_network_then_ends() {
        let mut bus = running();
        bus.bring_up(|_| {}).unwrap();
        {
            let card = bus.chip.sdio.hc.p.card();
            assert!(card.up && card.infra, "C_UP and SET_INFRA sent");
            assert_eq!(card.event_mask, event_mask(&STATION_EVENTS).to_vec());
        }
        let mut seen = Vec::new();
        assert_eq!(bus.scan(|b| seen.push(*b)).unwrap(), ScanEnd::Complete);
        assert_eq!(seen.len(), 2);
        assert_eq!((seen[0].ssid(), seen[0].channel, seen[0].rssi), (&b"piconet"[..], 6, -42));
        assert_eq!(seen[0].bssid, [0x02, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!((seen[1].ssid(), seen[1].channel, seen[1].rssi), (&b""[..], 36, -71), "a hidden 5 GHz network");
    }

    #[test]
    fn a_truncated_bss_record_ends_the_list_instead_of_reading_past_it() {
        let mut data = vec![0u8; ESCAN_RESULTS_HEADER];
        data[10] = 2; // claims two records
        let mut rec = vec![0u8; BSS_INFO_MIN];
        rec[4..8].copy_from_slice(&(BSS_INFO_MIN as u32).to_le_bytes());
        data.extend_from_slice(&rec);
        data.extend_from_slice(&rec[..40]); // the second is cut short
        assert_eq!(escan_bss(&data).count(), 1);
    }
}
