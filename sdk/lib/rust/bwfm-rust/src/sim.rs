//! A simulated SDHCI with a CYW43455 on its bus, for the host tests.
//!
//! The controller side keeps what the driver depends on: write-1-to-clear
//! status, buffer-ready per block, transfer complete after the last one, and a
//! command timeout when nothing answers. The card side answers CMD5/3/7/52/53
//! the way the SDIO spec says, with the 43455's CIS, function 1's backplane
//! window and clock register, and function 2 as a FIFO that records each CMD53.
//! Time is virtual: it moves only when the driver stalls.

use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;

use sdhci_rust::*;

pub struct Card {
    pub present: bool,
    /// The chip never reports its ALP clock.
    pub alp_never: bool,
    /// Function 2 never becomes ready, as if the firmware did not boot.
    pub fw_never_ready: bool,
    /// One RAM bit reads back as 1 whatever was written (at this address).
    pub stuck_bit_at: Option<u32>,
    pub rca: u16,
    pub selected: bool,
    ocr_polls: u32,
    pub cccr: [u8; 0x100],
    pub fbr_block: [u16; 8],
    pub io_enable: u8,
    cis: Vec<u8>,
    pub clkcsr: u8,
    alp_polls: u32,
    sbaddr: [u8; 3],
    /// Function 1's Broadcom registers 0x10000-0x1001F not modelled otherwise.
    pub f1_regs: [u8; 0x20],
    pub window: u32,
    pub window_writes: u32,
    backplane: HashMap<u32, u32>,
    /// (address, block mode, bytes) of each function 2 CMD53 write.
    pub f2_writes: Vec<(u32, bool, usize)>,
    pub f2_fifo: Vec<u8>,
    /// The firmware's side of function 2: frames queued for the host, the one
    /// being read and how far, and whether its replies carry credit.
    rx_queue: std::collections::VecDeque<Vec<u8>>,
    rx_cur: Vec<u8>,
    rx_pos: usize,
    announced: bool,
    pub credit_frames: bool,
    rx_seq: u8,
    pub desynced: bool,
    /// The CLM as clmload reassembled it, and each chunk's flag.
    pub clm: Vec<u8>,
    pub clm_flags: Vec<u16>,
    pub up: bool,
    pub infra: bool,
    pub event_mask: Vec<u8>,
    /// The access point's side of a join.
    pub refuse_join: bool,
    /// The AP does not broadcast its SSID: only a scan that names it hears it.
    pub ap_hidden: bool,
    /// The firmware puts its own RSN capabilities in the association request
    /// instead of `wpaie`'s, as real firmware may.
    pub fw_rewrites_rsn_caps: Option<u16>,
    /// The RSN element the association request carried.
    assoc_rsn: Vec<u8>,
    /// Ethernet frames the host sent that were not EAPOL.
    pub tx_ethernet: Vec<Vec<u8>>,
    pub wpaie: Vec<u8>,
    pub joined_bssid: Option<[u8; 6]>,
    pub ap_log: Vec<&'static str>,
    pub keys_match: bool,
    wsec: u32,
    ap_ptk: Option<[u8; 48]>,
    pairwise_ok: bool,
    /// Group keys the host installed, (key id, key), in order.
    pub group_keys: Vec<(u8, Vec<u8>)>,
    /// The AP's EAPOL replay counter: message 3 used 2.
    ap_replay: u64,
    last_rekey: Vec<u8>,
}

pub const AP_SSID: &[u8] = b"piconet";
pub const AP_PASS: &[u8] = b"correct horse battery";
pub const AP_BSSID: [u8; 6] = [0x02, 0x11, 0x22, 0x33, 0x44, 0x55];
/// The AP's RSN element: CCMP group and pairwise, PSK, capabilities 0x000C.
const AP_RSN: [u8; 22] = [0x30, 20, 1, 0, 0, 0x0F, 0xAC, 4, 1, 0, 0, 0x0F, 0xAC, 4, 1, 0, 0, 0x0F, 0xAC, 2, 0x0C, 0];
const AP_ANONCE: [u8; 32] = [0xA5; 32];
const AP_GTK: [u8; 16] = [0x47; 16];
// EAPOL-Key frame offsets, from the 802.1X header (wpa-rust's own).
const O_INFO: usize = 5;
const O_REPLAY: usize = 9;
const O_NONCE: usize = 17;
const O_RSC: usize = 65;
const O_MIC: usize = 81;
const O_DATALEN: usize = 97;
const O_DATA: usize = 99;

/// The simulated chip's address.
pub const MAC: [u8; 6] = [0xD8, 0x3A, 0xDD, 0x01, 0x02, 0x03];

/// What the simulated firmware answers `ver` with.
pub const FW_VERSION: &[u8] = b"wl0: Mar 28 2025 version 7.45.265 (28bca26 CY) FWID 01-b677b91b";
/// BCME_UNSUPPORTED: the firmware's answer to an iovar it does not know.
const BCME_UNSUPPORTED: i32 = -23;

const CIS_AT: u32 = 0x1000;
const EROM: u32 = 0x1800_F000;
pub const CR4: u32 = 0x1800_2000;
pub const CR4_WRAP: u32 = 0x1810_2000;
/// Each of the CR4's two banks: 16 blocks of 8 KiB, so 256 KiB of RAM.
const BANKINFO: u32 = 0x0F;

/// A small enumeration ROM in the AI layout (bwfmreg.h `BWFM_DMP_*`).
fn erom() -> Vec<u32> {
    let comp_a = |part: u32| 0x4BF << 20 | part << 8 | 0x1;
    let comp_b = |rev: u32, nsw: u32, nmw: u32| rev << 24 | nsw << 19 | nmw << 14 | 0x1;
    // base | port | slave type | size type (0 = 4K) | ADDRESS
    let addr = |base: u32, stype: u32| base | stype << 6 | 0x5;
    vec![
        comp_a(0x800),
        comp_b(0x2B, 1, 0),
        addr(0x1800_0000, 0),
        addr(0x1810_0000, 2), // slave wrapper
        comp_a(0x83E),
        comp_b(7, 0, 1),
        0x0000_0013, // master port 1
        addr(0x1800_2000, 0),
        addr(0x1810_2000, 3), // master wrapper
        comp_a(0x829),
        comp_b(21, 1, 0),
        addr(0x1800_4000, 0),
        addr(0x1810_4000, 2),
        comp_a(0x135),
        comp_b(0, 0, 0), // no wrappers: skipped
        comp_a(0x827),
        comp_b(1, 1, 0),
        addr(0x1801_2000, 0),
        addr(0x1811_2000, 2),
        0x0000_000F,
    ]
}
const R5_STATE_CMD: u32 = 0x1000;
const R5_FUNCTION_NUMBER: u32 = 0x0200;

impl Card {
    pub fn cyw43455() -> Self {
        let mut cccr = [0u8; 0x100];
        cccr[0] = 0x43; // CCCR 3.00, SDIO 3.00
        cccr[0x13] = 1; // SHS: high speed supported
        cccr[0x09..0x0C].copy_from_slice(&CIS_AT.to_le_bytes()[..3]);
        let mut backplane = HashMap::new();
        backplane.insert(0x1800_0000, 0x1526_4345);
        backplane.insert(0x1800_0004, 0x1000_0000); // chip-common caps: has a PMU
        backplane.insert(0x1800_0604, 24); // PMU rev 24 (save/restore asks the chip)
        backplane.insert(0x1800_00FC, EROM);
        backplane.insert(CR4 + 0x04, 0x11); // one A bank, one B bank
        for (i, w) in erom().iter().enumerate() {
            backplane.insert(EROM + 4 * i as u32, *w);
        }
        Self {
            present: true,
            alp_never: false,
            fw_never_ready: false,
            stuck_bit_at: None,
            rca: 1,
            selected: false,
            ocr_polls: 0,
            cccr,
            fbr_block: [0; 8],
            io_enable: 0,
            // FUNCID (SDIO card), then MANFID: Broadcom 0x02d0, device 0x4345.
            cis: vec![0x21, 0x02, 0x0C, 0x00, 0x20, 0x04, 0xD0, 0x02, 0x45, 0x43, 0xFF],
            clkcsr: 0,
            alp_polls: 0,
            sbaddr: [0; 3],
            f1_regs: [0; 0x20],
            window: 0,
            window_writes: 0,
            backplane,
            f2_writes: Vec::new(),
            f2_fifo: Vec::new(),
            rx_queue: Default::default(),
            rx_cur: Vec::new(),
            rx_pos: 0,
            announced: false,
            credit_frames: true,
            rx_seq: 0,
            desynced: false,
            clm: Vec::new(),
            clm_flags: Vec::new(),
            up: false,
            infra: false,
            event_mask: Vec::new(),
            refuse_join: false,
            ap_hidden: false,
            fw_rewrites_rsn_caps: None,
            assoc_rsn: Vec::new(),
            tx_ethernet: Vec::new(),
            wpaie: Vec::new(),
            joined_bssid: None,
            ap_log: Vec::new(),
            keys_match: false,
            wsec: 0,
            ap_ptk: None,
            pairwise_ok: false,
            group_keys: Vec::new(),
            ap_replay: 2,
            last_rekey: Vec::new(),
        }
    }

    pub fn absent() -> Self {
        Self { present: false, ..Self::cyw43455() }
    }

    fn r52(&mut self, write: bool, func: u8, addr: u32, data: u8) -> Option<u32> {
        if !self.selected {
            return None;
        }
        let v = match func {
            0 => self.f0(write, addr, data),
            1 if self.io_enable & 2 == 0 => return Some(R5_STATE_CMD | R5_FUNCTION_NUMBER),
            1 => self.f1(write, addr, data),
            _ => return Some(R5_STATE_CMD | R5_FUNCTION_NUMBER),
        };
        Some(R5_STATE_CMD | v as u32)
    }

    fn f0(&mut self, write: bool, addr: u32, data: u8) -> u8 {
        let fbr = (addr >> 8) as usize;
        match (write, addr) {
            (false, 0x02) => self.io_enable,
            (false, 0x03) => self.io_enable & (0b11 | if self.firmware_running() { 0b100 } else { 0 }),
            (false, a) if a < 0x100 => self.cccr[a as usize],
            (true, 0x02) => {
                self.io_enable = data;
                data
            }
            (true, a) if a < 0x100 => {
                self.cccr[a as usize] = data;
                data
            }
            (true, a) if (1..8).contains(&fbr) && a & 0xFF == 0x10 => {
                self.fbr_block[fbr] = (self.fbr_block[fbr] & 0xFF00) | data as u16;
                data
            }
            (true, a) if (1..8).contains(&fbr) && a & 0xFF == 0x11 => {
                self.fbr_block[fbr] = (self.fbr_block[fbr] & 0x00FF) | (data as u16) << 8;
                data
            }
            (false, a) if a >= CIS_AT => *self.cis.get((a - CIS_AT) as usize).unwrap_or(&0xFF),
            _ => 0,
        }
    }

    fn f1(&mut self, write: bool, addr: u32, data: u8) -> u8 {
        match addr {
            0x1000A..=0x1000C => {
                if write {
                    self.sbaddr[(addr - 0x1000A) as usize] = data;
                    self.window_writes += 1;
                    let [l, m, h] = self.sbaddr;
                    self.window = (l as u32) << 8 | (m as u32) << 16 | (h as u32) << 24;
                }
                self.sbaddr[(addr - 0x1000A) as usize]
            }
            0x1000E => {
                if write {
                    // A request keeps what is up; dropping every request drops it.
                    let up = if data & 0x1F != 0 { self.clkcsr & 0xC0 } else { 0 };
                    self.clkcsr = (data & 0x3F) | up;
                    self.alp_polls = 0;
                } else if self.clkcsr & 0x18 != 0 && !self.alp_never {
                    self.alp_polls += 1;
                    if self.alp_polls > 2 {
                        self.clkcsr |= if self.clkcsr & 0x10 != 0 { 0xC0 } else { 0x40 };
                    }
                }
                self.clkcsr
            }
            0x10000..=0x1001F => {
                let r = &mut self.f1_regs[(addr - 0x10000) as usize];
                if write {
                    *r = data;
                }
                *r
            }
            _ => 0,
        }
    }

    pub fn word(&self, addr: u32) -> u32 {
        self.backplane.get(&addr).copied().unwrap_or(0)
    }

    /// The CR4 out of reset and not halted, with a reset vector planted: what
    /// "the firmware runs" means here.
    pub fn firmware_running(&self) -> bool {
        !self.fw_never_ready
            && self.word(CR4_WRAP + 0x800) & 1 == 0
            && self.word(CR4_WRAP + 0x408) & 0x20 == 0
            && self.word(CR4_WRAP + 0x408) & 1 != 0
            && self.word(0) != 0
    }

    fn bp_addr(&self, addr: u32) -> u32 {
        assert!(addr & 0x8000 != 0, "a backplane access without the 4-byte flag");
        self.window | (addr & 0x7FFF)
    }

    /// A frame from the firmware: SDPCM headers (credit up to four past the
    /// host's last sequence), `payload`, padded to a word.
    fn queue(&mut self, channel: u8, payload: &[u8]) {
        let len = 12 + payload.len();
        let max = if self.credit_frames { self.rx_seq.wrapping_add(4) } else { 0 };
        let mut f = Vec::with_capacity(len.div_ceil(4) * 4);
        f.extend_from_slice(&(len as u16).to_le_bytes());
        f.extend_from_slice(&(!(len as u16)).to_le_bytes());
        f.extend_from_slice(&[0, channel, 0, 12, 0, max, 0, 0]);
        f.extend_from_slice(payload);
        f.resize(len.div_ceil(4) * 4, 0);
        self.rx_queue.push_back(f);
    }

    /// A reply to an earlier `command` with request id `id`, as a late answer to
    /// one the host gave up on would arrive.
    pub fn queue_stale_reply(&mut self, command: u32, id: u16, value: &[u8]) {
        let mut r = Vec::new();
        r.extend_from_slice(&command.to_le_bytes());
        r.extend_from_slice(&(value.len() as u32).to_le_bytes());
        r.extend_from_slice(&((id as u32) << 16).to_le_bytes());
        r.extend_from_slice(&0u32.to_le_bytes());
        r.extend_from_slice(value);
        self.queue(0, &r);
    }

    /// An event as the firmware sends it: the BCDC data header, an Ethernet
    /// frame of type 0x886C, Broadcom's header, the big-endian event message.
    fn queue_bcm_event(&mut self, event_type: u32, status: u32, data: &[u8]) {
        self.queue_bcm_event_reason(event_type, status, 0, data)
    }

    fn queue_bcm_event_reason(&mut self, event_type: u32, status: u32, reason: u32, data: &[u8]) {
        let mut f = vec![0x20, 0, 0, 0];
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&0x886Cu16.to_be_bytes());
        f.extend_from_slice(&[0x80, 0x01, 0, 0, 0, 0x00, 0x10, 0x18, 0x00, 0x01]);
        f.extend_from_slice(&2u16.to_be_bytes()); // version
        f.extend_from_slice(&0u16.to_be_bytes()); // flags
        for v in [event_type, status, reason, 0, data.len() as u32] {
            f.extend_from_slice(&v.to_be_bytes());
        }
        f.extend_from_slice(&[0; 6 + 16 + 2]); // addr, ifname, ifidx, bsscfgidx
        f.extend_from_slice(data);
        self.queue(1, &f);
    }

    /// One `bwfm_bss_info` (128 bytes), then `ies`.
    fn bss(bssid: [u8; 6], ssid: &[u8], channel: u8, rssi: i16, ies: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; 128];
        r[0..4].copy_from_slice(&109u32.to_le_bytes()); // version
        r[4..8].copy_from_slice(&((128 + ies.len()) as u32).to_le_bytes());
        r[116..118].copy_from_slice(&128u16.to_le_bytes());
        r[120..124].copy_from_slice(&(ies.len() as u32).to_le_bytes());
        r[8..14].copy_from_slice(&bssid);
        r[16..18].copy_from_slice(&0x0411u16.to_le_bytes()); // ESS | privacy | short slot
        r[18] = ssid.len() as u8;
        r[19..19 + ssid.len()].copy_from_slice(ssid);
        r[72..74].copy_from_slice(&(0x1000u16 | channel as u16).to_le_bytes());
        r[78..80].copy_from_slice(&rssi.to_le_bytes());
        r[88] = channel;
        r.extend_from_slice(ies);
        r
    }

    /// What a passive escan hears here: two networks, then the end.
    fn escan_results(&mut self, ap_named: bool) {
        // The piconet entry's IEs: its SSID (empty when hidden), then its RSN element.
        let shown: &[u8] = if ap_named { AP_SSID } else { b"" };
        let ies = [&[0x00, shown.len() as u8][..], shown, &AP_RSN].concat();
        for (bssid, ssid, ch, rssi, ies) in [
            (AP_BSSID, shown, 6, -42, &ies[..]),
            ([0x02, 0x66, 0x77, 0x88, 0x99, 0xAA], &b""[..], 36, -71, &[][..]),
        ] {
            let rec = Self::bss(bssid, ssid, ch, rssi, ies);
            let mut res = Vec::new();
            res.extend_from_slice(&(12 + rec.len() as u32).to_le_bytes()); // buflen
            res.extend_from_slice(&109u32.to_le_bytes()); // version
            res.extend_from_slice(&0x1234u16.to_le_bytes()); // sync id
            res.extend_from_slice(&1u16.to_le_bytes()); // one BSS
            res.extend_from_slice(&rec);
            self.queue_bcm_event(69, 8, &res);
        }
        self.queue_bcm_event(69, 0, &[]);
    }

    fn queue_link_up(&mut self) {
        // E_LINK with the link-up flag: version, flags 1, then the rest as queue_bcm_event.
        let mut f = vec![0x20, 0, 0, 0];
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&AP_BSSID);
        f.extend_from_slice(&0x886Cu16.to_be_bytes());
        f.extend_from_slice(&[0x80, 0x01, 0, 0, 0, 0x00, 0x10, 0x18, 0x00, 0x01]);
        f.extend_from_slice(&2u16.to_be_bytes());
        f.extend_from_slice(&1u16.to_be_bytes());
        for v in [16u32, 0, 0, 0, 0] {
            f.extend_from_slice(&v.to_be_bytes());
        }
        f.extend_from_slice(&[0; 6 + 16 + 2]);
        self.queue(1, &f);
    }

    /// An EAPOL frame from the AP to us, as a data frame.
    fn queue_eapol(&mut self, eapol: &[u8]) {
        let mut f = vec![0x20, 0, 0, 0];
        f.extend_from_slice(&MAC);
        f.extend_from_slice(&AP_BSSID);
        f.extend_from_slice(&[0x88, 0x8E]);
        f.extend_from_slice(eapol);
        self.queue(2, &f);
    }

    /// An EAPOL-Key frame from the authenticator (wpa-rust's test authenticator).
    fn ap_frame(info: u16, replay: u64, nonce: &[u8; 32], rsc: &[u8; 8], data: &[u8], kck: Option<&[u8]>) -> Vec<u8> {
        let mut f = vec![0u8; O_DATA + data.len()];
        f[0] = 2;
        f[1] = 3; // EAPOL-Key
        let n = (f.len() - 4) as u16;
        f[2..4].copy_from_slice(&n.to_be_bytes());
        f[4] = 2; // RSN descriptor
        f[O_INFO..O_INFO + 2].copy_from_slice(&info.to_be_bytes());
        f[7..9].copy_from_slice(&16u16.to_be_bytes());
        f[O_REPLAY..O_REPLAY + 8].copy_from_slice(&replay.to_be_bytes());
        f[O_NONCE..O_NONCE + 32].copy_from_slice(nonce);
        f[O_RSC..O_RSC + 8].copy_from_slice(rsc);
        f[O_DATALEN..O_DATALEN + 2].copy_from_slice(&(data.len() as u16).to_be_bytes());
        f[O_DATA..].copy_from_slice(data);
        if let Some(kck) = kck {
            let m = wpa_rust::HmacSha1::new(kck).mac(&[&f]);
            f[O_MIC..O_MIC + 16].copy_from_slice(&m[..16]);
        }
        f
    }

    /// AES key wrap (RFC 3394).
    fn wrap(kek: &[u8; 16], plain: &[u8]) -> Vec<u8> {
        let n = plain.len() / 8;
        let aes = wpa_rust::Aes128::new(kek);
        let mut a = [0xA6u8; 8];
        let mut r = plain.to_vec();
        for j in 0..6 {
            for i in 1..=n {
                let mut b = [0u8; 16];
                b[..8].copy_from_slice(&a);
                b[8..].copy_from_slice(&r[8 * (i - 1)..8 * i]);
                aes.encrypt(&mut b);
                let t = ((n * j) + i) as u64;
                for (k, x) in t.to_be_bytes().iter().enumerate() {
                    a[k] = b[k] ^ x;
                }
                r[8 * (i - 1)..8 * i].copy_from_slice(&b[8..]);
            }
        }
        let mut out = a.to_vec();
        out.extend_from_slice(&r);
        out
    }

    fn mic_ok(kck: &[u8], f: &[u8]) -> bool {
        let mut z = f.to_vec();
        z[O_MIC..O_MIC + 16].fill(0);
        wpa_rust::HmacSha1::new(kck).mac(&[&z])[..16] == f[O_MIC..O_MIC + 16]
    }

    /// The AP's half of the 4-way handshake.
    fn ap_eapol(&mut self, f: &[u8]) {
        let info = u16::from_be_bytes([f[O_INFO], f[O_INFO + 1]]);
        match info {
            0x010A => {
                let pmk = wpa_rust::psk(AP_PASS, AP_SSID);
                let mut snonce = [0u8; 32];
                snonce.copy_from_slice(&f[O_NONCE..O_NONCE + 32]);
                let (a1, a2) = if AP_BSSID < MAC { (AP_BSSID, MAC) } else { (MAC, AP_BSSID) };
                let (n1, n2) = if AP_ANONCE < snonce { (AP_ANONCE, snonce) } else { (snonce, AP_ANONCE) };
                let mut ptk = [0u8; 48];
                wpa_rust::prf(&pmk, b"Pairwise key expansion", &[&a1, &a2, &n1, &n2], &mut ptk);
                if !Self::mic_ok(&ptk[..16], f) {
                    self.ap_log.push("msg2 bad mic");
                    return;
                }
                // An AP drops a message 2 whose RSN element is not the association
                // request's, byte for byte.
                let n = u16::from_be_bytes([f[O_DATALEN], f[O_DATALEN + 1]]) as usize;
                if f[O_DATA..O_DATA + n] != self.assoc_rsn[..] {
                    self.ap_log.push("msg2 rsn mismatch");
                    return;
                }
                self.ap_log.push("msg2 ok");
                self.ap_ptk = Some(ptk);
                let mut kd = AP_RSN.to_vec();
                kd.extend_from_slice(&[0xDD, 22, 0x00, 0x0F, 0xAC, 0x01, 0x01, 0x00]);
                kd.extend_from_slice(&AP_GTK);
                kd.push(0xDD);
                while kd.len() % 8 != 0 {
                    kd.push(0);
                }
                let kek: [u8; 16] = ptk[16..32].try_into().unwrap_or_default();
                let m3 = Self::ap_frame(0x13CA, 2, &AP_ANONCE, &[0; 8], &Self::wrap(&kek, &kd), Some(&ptk[..16]));
                self.queue_eapol(&m3);
            }
            0x030A => match self.ap_ptk {
                Some(ptk) if Self::mic_ok(&ptk[..16], f) => self.ap_log.push("msg4 ok"),
                _ => self.ap_log.push("msg4 bad"),
            },
            0x0302 => match self.ap_ptk {
                Some(ptk) if Self::mic_ok(&ptk[..16], f) => self.ap_log.push("group msg2 ok"),
                _ => self.ap_log.push("group msg2 bad"),
            },
            _ => self.ap_log.push("unexpected EAPOL"),
        }
    }

    /// The host installs a key: does it match what the AP derived?
    fn install_key(&mut self, k: &[u8]) {
        let len = u32::from_le_bytes([k[4], k[5], k[6], k[7]]) as usize;
        let key = &k[8..8 + len];
        let algo = u32::from_le_bytes([k[112], k[113], k[114], k[115]]);
        if k[156..162] != [0; 6] {
            self.ap_log.push("pairwise key");
            self.pairwise_ok = algo == 4 && k[156..162] == AP_BSSID && self.ap_ptk.is_some_and(|p| key == &p[32..48]);
        } else {
            self.ap_log.push("group key");
            self.group_keys.push((k[0], key.to_vec()));
            self.keys_match = self.pairwise_ok && algo == 4 && key == AP_GTK && k[0] == 1;
        }
    }

    /// The AP rekeys the group: message 1 of the group handshake, carrying `gtk` as key
    /// `id`, wrapped with the pairwise KEK.
    pub fn ap_rekeys(&mut self, gtk: [u8; 16], id: u8) {
        let Some(ptk) = self.ap_ptk else { panic!("a rekey before the 4-way handshake") };
        self.ap_replay += 1;
        let mut kd = vec![0xDD, 22, 0x00, 0x0F, 0xAC, 0x01, id, 0x00];
        kd.extend_from_slice(&gtk);
        let kek: [u8; 16] = ptk[16..32].try_into().unwrap_or_default();
        let g1 = Self::ap_frame(0x1382, self.ap_replay, &[0; 32], &[0; 8], &Self::wrap(&kek, &kd), Some(&ptk[..16]));
        self.queue_eapol(&g1);
        self.last_rekey = g1;
    }

    /// The last rekey message again, unchanged: a replay.
    pub fn ap_replays_rekey(&mut self) {
        let g1 = self.last_rekey.clone();
        self.queue_eapol(&g1);
    }

    /// The AP deauthenticates us, with an 802.11 reason code.
    pub fn ap_deauths(&mut self, reason: u32) {
        self.queue_bcm_event_reason(6, 0, reason, &[]);
    }

    /// An Ethernet frame arriving for the host, as a data frame.
    pub fn queue_ethernet(&mut self, eth: &[u8]) {
        let mut f = vec![0x20, 0, 0, 0];
        f.extend_from_slice(eth);
        self.queue(2, &f);
    }

    pub fn queue_event(&mut self, payload: &[u8]) {
        self.queue(1, payload);
    }

    pub fn drop_pending_rx(&mut self) {
        self.rx_queue.clear();
        self.rx_cur.clear();
        self.announced = true;
    }

    /// The host reads the FIFO: the frame in progress, else the next one, else
    /// zeros. The firmware's first act once running is a credit-only frame.
    fn read_fifo(&mut self, len: usize) -> Vec<u8> {
        if self.firmware_running() && self.io_enable & 0b100 != 0 && !self.announced {
            self.announced = true;
            self.queue(0, &[]);
        }
        let mut out = Vec::with_capacity(len);
        while out.len() < len {
            if self.rx_pos >= self.rx_cur.len() {
                match self.rx_queue.pop_front() {
                    Some(f) => (self.rx_cur, self.rx_pos) = (f, 0),
                    None => break,
                }
            }
            let n = (len - out.len()).min(self.rx_cur.len() - self.rx_pos);
            out.extend_from_slice(&self.rx_cur[self.rx_pos..self.rx_pos + n]);
            self.rx_pos += n;
        }
        out.resize(len, 0);
        out
    }

    /// The host sent a frame: answer a BCDC GET_VAR the way the firmware would.
    fn answer(&mut self, frame: &[u8]) {
        if frame.len() >= 12 + 4 + 14 && frame[5] & 0xF == 2 {
            let at = 12 + 4 + 4 * frame[12 + 3] as usize;
            let eth = &frame[at..u16::from_le_bytes([frame[0], frame[1]]) as usize];
            if eth.len() > 14 && eth[12..14] == [0x88, 0x8E] && eth[..6] == AP_BSSID && eth[6..12] == MAC {
                let body = eth[14..].to_vec();
                self.ap_eapol(&body);
            } else {
                self.tx_ethernet.push(eth.to_vec());
            }
            return;
        }
        if frame.len() < 12 + 16 || frame[5] & 0xF != 0 {
            return;
        }
        // A write shorter than its header's length, or not a whole word, leaves
        // the chip's FIFO out of step: it answers nothing.
        let declared = u16::from_le_bytes([frame[0], frame[1]]) as usize;
        if frame.len() < declared || frame.len() % 4 != 0 {
            self.desynced = true;
            return;
        }
        self.rx_seq = frame[4].wrapping_add(1);
        let dcmd = &frame[frame[7] as usize..];
        let u = |i: usize| u32::from_le_bytes([dcmd[i], dcmd[i + 1], dcmd[i + 2], dcmd[i + 3]]);
        let (command, len, flags) = (u(0), u(4) as usize, u(8));
        let data = &dcmd[16..16 + len.min(dcmd.len() - 16)];
        let name = &data[..data.iter().position(|&c| c == 0).unwrap_or(data.len())];
        let rest = &data[(name.len() + 1).min(data.len())..];
        let (status, mut value) = match (command, name) {
            (262, b"ver") => (0, [FW_VERSION, b"\0"].concat()),
            (262, b"cur_etheraddr") => (0, MAC.to_vec()),
            (262, b"clmload_status") => (0, 0u32.to_le_bytes().to_vec()),
            (263, b"clmload") if rest.len() >= 12 => {
                let flag = u16::from_le_bytes([rest[0], rest[1]]);
                let n = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]) as usize;
                assert_eq!(u16::from_le_bytes([rest[2], rest[3]]), 2, "dload type CLM");
                assert_eq!(n, rest.len() - 12, "chunk length");
                self.clm_flags.push(flag);
                self.clm.extend_from_slice(&rest[12..]);
                (0, Vec::new())
            }
            (263, b"mpc") => (0, Vec::new()),
            (263, b"wpa_auth" | b"auth") => (0, Vec::new()),
            (263, b"wpaie") => {
                self.wpaie = rest.to_vec();
                (0, Vec::new())
            }
            (263, b"wsec") => {
                self.wsec = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]);
                (0, Vec::new())
            }
            (262, b"wsec") => (0, self.wsec.to_le_bytes().to_vec()),
            (262, b"assoc_info") if len < 64 => (-14, Vec::new()), // BCME_BUFTOOSHORT, as the Pi 5's
            (262, b"assoc_info") => {
                // req_len, resp_len: the request's IEs are its SSID and its RSN element.
                let req = 2 + AP_SSID.len() + self.assoc_rsn.len();
                (0, [(req as u32).to_le_bytes(), 0u32.to_le_bytes()].concat())
            }
            (262, b"assoc_req_ies") => (0, [&[0x00, AP_SSID.len() as u8][..], AP_SSID, &self.assoc_rsn].concat()),
            (263, b"join") => {
                assert_eq!(rest.len(), 68, "ext join parameters");
                let ssid_len = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                if self.refuse_join || &rest[4..4 + ssid_len] != AP_SSID {
                    self.queue_bcm_event(0, 1, &[]); // E_SET_SSID, FAIL
                } else {
                    let mut b = [0u8; 6];
                    b.copy_from_slice(&rest[56..62]);
                    self.joined_bssid = Some(b);
                    let mut rsn = self.wpaie.clone();
                    if let (Some(caps), true) = (self.fw_rewrites_rsn_caps, rsn.len() >= 22) {
                        rsn[20..22].copy_from_slice(&caps.to_le_bytes());
                    }
                    self.assoc_rsn = rsn;
                    self.queue_link_up();
                    let m1 = Self::ap_frame(0x008A, 1, &AP_ANONCE, &[0; 8], &[], None);
                    self.queue_eapol(&m1);
                }
                (0, Vec::new())
            }
            (263, b"wsec_key") => {
                self.install_key(rest);
                (0, Vec::new())
            }
            (263, b"event_msgs") => {
                self.event_mask = rest.to_vec();
                (0, Vec::new())
            }
            (263, b"escan") => {
                assert!(rest.len() == 72 || rest.len() == 108, "escan v0 parameters");
                let named = rest.len() == 108 && rest[51] == 0 && &rest[76..76 + rest[72] as usize] == AP_SSID;
                self.escan_results(!self.ap_hidden || named);
                (0, Vec::new())
            }
            (2, _) => {
                self.up = true;
                (0, Vec::new())
            }
            (20, _) => {
                self.infra = data.first() == Some(&1);
                (0, Vec::new())
            }
            _ => (BCME_UNSUPPORTED, Vec::new()),
        };
        // A GET's reply fills the query's length; a SET's carries nothing back.
        if command == 262 {
            value.resize(len.max(value.len()), 0);
        }
        let error = if status != 0 { 1 } else { 0 };
        let mut reply = Vec::new();
        reply.extend_from_slice(&command.to_le_bytes());
        reply.extend_from_slice(&(value.len() as u32).to_le_bytes());
        reply.extend_from_slice(&((flags & 0xFFFF_0000) | error).to_le_bytes());
        reply.extend_from_slice(&(status as u32).to_le_bytes());
        reply.extend_from_slice(&value);
        self.queue(0, &reply);
    }

    fn read53(&mut self, func: u8, addr: u32, len: usize) -> Vec<u8> {
        if func == 2 {
            return self.read_fifo(len);
        }
        assert_eq!(func, 1, "only the backplane and the FIFO are readable here");
        assert_eq!(len % 4, 0);
        let a = self.bp_addr(addr);
        (0..len / 4)
            .flat_map(|i| {
                let at = a + 4 * i as u32;
                let v = match at {
                    // The bank BANKIDX selects.
                    _ if at == CR4 + 0x44 => BANKINFO,
                    _ if Some(at) == self.stuck_bit_at => self.word(at) | 1,
                    _ => self.word(at),
                };
                v.to_le_bytes()
            })
            .collect()
    }

    fn write53(&mut self, func: u8, addr: u32, block_mode: bool, data: &[u8]) {
        match func {
            1 => {
                let a = self.bp_addr(addr);
                for (i, w) in data.chunks(4).enumerate() {
                    let mut b = [0u8; 4];
                    b[..w.len()].copy_from_slice(w);
                    self.backplane.insert(a + 4 * i as u32, u32::from_le_bytes(b));
                }
            }
            2 => {
                self.f2_writes.push((addr, block_mode, data.len()));
                self.f2_fifo.extend_from_slice(data);
                self.answer(data);
            }
            _ => panic!("CMD53 to function {func}"),
        }
    }

    /// A command's response, or None when the card does not answer.
    fn command(&mut self, index: u8, arg: u32) -> Option<u32> {
        if !self.present {
            return (index == 0).then_some(0);
        }
        match index {
            0 => Some(0),
            5 => {
                let base = 2 << 28 | 0x00FF_8000;
                if arg == 0 {
                    return Some(base);
                }
                self.ocr_polls += 1;
                Some(if self.ocr_polls >= 2 { base | 1 << 31 } else { base })
            }
            3 => Some((self.rca as u32) << 16),
            7 if arg >> 16 == self.rca as u32 => {
                self.selected = true;
                Some(0)
            }
            52 => self.r52(arg >> 31 != 0, ((arg >> 28) & 7) as u8, (arg >> 9) & 0x1_FFFF, arg as u8),
            53 if self.selected => Some(R5_STATE_CMD),
            _ => None,
        }
    }
}

struct Xfer {
    read: bool,
    func: u8,
    addr: u32,
    block_mode: bool,
    bs: usize,
    blocks: usize,
    buf: Vec<u8>,
    pos: usize,
}

struct St {
    regs: [u32; 64],
    card: Card,
    xfer: Option<Xfer>,
}

pub struct Sim {
    st: RefCell<St>,
    now: Cell<u64>,
}

impl Sim {
    pub fn new(card: Card) -> Self {
        let mut regs = [0u32; 64];
        regs[REG_CAPS / 4] = 100 << 8 | sdhci_rust::CAPS_HIGH_SPEED; // 100 MHz base clock, high speed
        regs[REG_VERSION / 4] = 2 << 16; // SDHCI 3.00
        Self { st: RefCell::new(St { regs, card, xfer: None }), now: Cell::new(0) }
    }

    pub fn card(&self) -> Ref<'_, Card> {
        Ref::map(self.st.borrow(), |s| &s.card)
    }

    pub fn card_mut(&self) -> std::cell::RefMut<'_, Card> {
        std::cell::RefMut::map(self.st.borrow_mut(), |s| &mut s.card)
    }
}

impl St {
    fn raise(&mut self, bits: u32) {
        self.regs[REG_INT_STATUS / 4] |= bits;
    }

    fn execute(&mut self, cmd: u32) {
        let index = (cmd >> 24) as u8 & 0x3F;
        let flags = (cmd >> 16) as u16 & 0xFF;
        let mode = cmd as u16;
        let arg = self.regs[REG_ARG / 4];
        let Some(resp) = self.card.command(index, arg) else {
            self.raise(INT_ERROR | INT_CMD_TIMEOUT);
            return;
        };
        self.regs[REG_RESP / 4] = resp;
        self.raise(INT_CMD_DONE);
        if flags == RESP_R1B {
            self.raise(INT_XFER_DONE);
        }
        if flags & CMD_DATA == 0 {
            return;
        }
        assert_eq!(index, 53, "data on CMD{index}");
        let block = self.regs[REG_BLOCK / 4];
        let bs = (block & 0xFFF) as usize;
        let blocks = (block >> 16).max(1) as usize;
        let block_mode = arg & 1 << 27 != 0;
        let count = (arg & 0x1FF) as usize;
        // The command's count must describe what the host is about to move.
        if block_mode {
            assert_eq!(count, blocks, "block-mode count");
            assert_eq!(bs, self.card.fbr_block[((arg >> 28) & 7) as usize] as usize, "block size");
            assert_eq!(blocks > 1, mode & XFER_MULTI != 0, "multi-block mode");
        } else {
            assert_eq!((count, blocks), (bs, 1), "byte-mode count");
        }
        let read = mode & XFER_READ != 0;
        assert_eq!(read, arg >> 31 == 0, "transfer direction vs CMD53's R/W flag");
        let (func, addr) = (((arg >> 28) & 7) as u8, (arg >> 9) & 0x1_FFFF);
        let buf = if read { self.card.read53(func, addr, bs * blocks) } else { Vec::new() };
        self.xfer = Some(Xfer { read, func, addr, block_mode, bs, blocks, buf, pos: 0 });
        self.raise(if read { INT_READ_READY } else { INT_WRITE_READY });
    }

    /// The buffer port moved `n` bytes: on a block boundary, ready the next
    /// block or complete the transfer.
    fn advance(&mut self) {
        let Some(x) = self.xfer.as_ref() else { return };
        if x.pos % x.bs != 0 {
            return;
        }
        if x.pos < x.bs * x.blocks {
            let bit = if x.read { INT_READ_READY } else { INT_WRITE_READY };
            self.raise(bit);
            return;
        }
        let Some(x) = self.xfer.take() else { return };
        if !x.read {
            self.card.write53(x.func, x.addr, x.block_mode, &x.buf);
        }
        self.raise(INT_XFER_DONE);
    }
}

impl Platform for Sim {
    fn rd(&self, off: usize) -> u32 {
        let mut st = self.st.borrow_mut();
        match off {
            REG_PRESENT => 0,
            REG_BUFFER => {
                let Some(x) = st.xfer.as_mut().filter(|x| x.read) else { return 0 };
                let end = (x.pos / x.bs + 1) * x.bs;
                let n = 4.min(end - x.pos);
                let mut w = [0u8; 4];
                w[..n].copy_from_slice(&x.buf[x.pos..x.pos + n]);
                x.pos += n;
                st.advance();
                u32::from_le_bytes(w)
            }
            _ => st.regs[off / 4],
        }
    }

    fn wr(&self, off: usize, v: u32) {
        let mut st = self.st.borrow_mut();
        match off {
            REG_INT_STATUS => st.regs[off / 4] &= !v,
            REG_CLOCK => {
                if v & RESET_ALL != 0 {
                    let (caps, version) = (st.regs[REG_CAPS / 4], st.regs[REG_VERSION / 4]);
                    st.regs = [0; 64];
                    st.regs[REG_CAPS / 4] = caps;
                    st.regs[REG_VERSION / 4] = version;
                    st.xfer = None;
                }
                if v & (RESET_CMD | RESET_DAT) != 0 {
                    st.xfer = None;
                }
                let mut c = v & !(RESET_ALL | RESET_CMD | RESET_DAT);
                if c & CLOCK_INTERNAL_ENABLE != 0 {
                    c |= CLOCK_STABLE;
                }
                st.regs[off / 4] = c;
            }
            REG_XFER_CMD => {
                st.regs[off / 4] = v;
                st.execute(v);
            }
            REG_BUFFER => {
                let Some(x) = st.xfer.as_mut().filter(|x| !x.read) else { return };
                let end = (x.pos / x.bs + 1) * x.bs;
                let n = 4.min(end - x.pos);
                x.buf.extend_from_slice(&v.to_le_bytes()[..n]);
                x.pos += n;
                st.advance();
            }
            _ => st.regs[off / 4] = v,
        }
    }

    fn now_us(&self) -> u64 {
        self.now.get()
    }

    fn stall_us(&self, us: u32) {
        self.now.set(self.now.get() + us as u64);
    }
}
