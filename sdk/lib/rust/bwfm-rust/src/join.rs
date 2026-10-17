//! Joining a WPA2-Personal network: the firmware associates, the host runs the
//! 4-way handshake (`sup_wpa 0`) with wpa-rust's `Supplicant`, and the keys go
//! back down to the firmware.
//!
//! The firmware puts our RSN element (`wpaie`) in its association request, so the
//! supplicant repeats those exact bytes in message 2. EAPOL frames travel as
//! ordinary data frames (ethertype 0x888E) on the data channel. The pairwise key is
//! installed only after message 4 has gone out, since the key it confirms must not
//! encrypt it. After OpenBSD's bwfm.c (`bwfm_connect`, `bwfm_set_key_cb`) and
//! bwfmreg.h, ISC (notice in chip.rs).

use core::cell::RefCell;

use sdhci_rust::{Error, Platform};
use wpa_rust::{psk, Reply, Sha1, Supplicant, MAX_REPLY};

use crate::bus::{Bus, Frame};
use crate::proto::{self, cmd, event};
use crate::scan::Bss;

pub const ETHERTYPE_EAPOL: u16 = 0x888E;

const WPA_AUTH_WPA2_PSK: u32 = 1 << 7;
const WSEC_TKIP: u32 = 1 << 1;
const WSEC_AES: u32 = 1 << 2;
const CRYPTO_ALGO_TKIP: u32 = 2;
const CRYPTO_ALGO_AES_CCM: u32 = 4;
const WSEC_PRIMARY_KEY: u32 = 1 << 1;
/// `bwfm_wsec_key`, padded to its C size.
const WSEC_KEY_LEN: usize = 164;
/// `bwfm_ext_join_params`: SSID, the join scan's parameters, the association's BSSID.
const EXT_JOIN_LEN: usize = 68;
/// `bwfm_join_params`, for firmware that does not know the `join` iovar.
const JOIN_LEN: usize = 48;

/// How long the association and the handshake may take.
const JOIN_WAIT_US: u64 = 15_000_000;
/// After message 4: let the firmware send it before a key exists to encrypt it.
const MSG4_DRAIN_US: u32 = 50_000;

// RSN suites (IEEE 802.11-2016 9.4.2.25): OUI 00-0F-AC and a type.
const SUITE_TKIP: u8 = 2;
const SUITE_CCMP: u8 = 4;
const AKM_PSK: u8 = 2;
const IEEE_OUI: [u8; 3] = [0x00, 0x0F, 0xAC];

/// The network to join, from `\SystemRoot\bin\wifi.txt` (`ssid=`, `psk=`, `hidden=1`), the
/// same file the OptiPlex's WDI path reads. The passphrase is never logged.
#[derive(Clone, Copy)]
pub struct WifiConfig {
    pub ssid: [u8; 32],
    pub ssid_len: usize,
    pub pass: [u8; 63],
    pub pass_len: usize,
    pub hidden: bool,
}

impl WifiConfig {
    pub fn parse(text: &[u8]) -> Option<Self> {
        let mut c = Self { ssid: [0; 32], ssid_len: 0, pass: [0; 63], pass_len: 0, hidden: false };
        for line in text.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if let Some(v) = line.strip_prefix(b"ssid=") {
                c.ssid_len = v.len().min(32);
                c.ssid[..c.ssid_len].copy_from_slice(&v[..c.ssid_len]);
            } else if let Some(v) = line.strip_prefix(b"psk=") {
                c.pass_len = v.len().min(63);
                c.pass[..c.pass_len].copy_from_slice(&v[..c.pass_len]);
            } else if let Some(v) = line.strip_prefix(b"hidden=") {
                c.hidden = v.first() == Some(&b'1');
            }
        }
        // WPA2-Personal passphrases are 8 to 63 characters.
        (c.ssid_len > 0 && c.pass_len >= 8).then_some(c)
    }

    pub fn ssid(&self) -> &[u8] {
        &self.ssid[..self.ssid_len]
    }
}

/// Our RSN element for an AP advertising `ap`: version 1, the AP's group cipher,
/// CCMP pairwise, PSK, no capabilities. WPA2-Personal with CCMP is all this
/// driver joins; anything else is refused with what was missing.
pub fn our_rsn(ap: &[u8]) -> Result<[u8; 22], &'static str> {
    if ap.len() < 8 || ap[0] != 0x30 || ap[2..4] != [1, 0] {
        return Err("no RSN element (not WPA2)");
    }
    let suite = |at: usize| ap.get(at..at + 4).filter(|s| s[..3] == IEEE_OUI).map(|s| s[3]);
    let group = suite(4).ok_or("unknown group cipher")?;
    if group != SUITE_CCMP && group != SUITE_TKIP {
        return Err("group cipher neither CCMP nor TKIP");
    }
    let list = |at: usize| -> Option<(usize, usize)> {
        let n = u16::from_le_bytes([*ap.get(at)?, *ap.get(at + 1)?]) as usize;
        Some((at + 2, n))
    };
    let (pw_at, pw_n) = list(8).ok_or("truncated pairwise list")?;
    if !(0..pw_n).any(|i| suite(pw_at + 4 * i) == Some(SUITE_CCMP)) {
        return Err("no CCMP pairwise cipher");
    }
    let (akm_at, akm_n) = list(pw_at + 4 * pw_n).ok_or("truncated AKM list")?;
    if !(0..akm_n).any(|i| suite(akm_at + 4 * i) == Some(AKM_PSK)) {
        return Err("not a PSK network (802.1X?)");
    }
    let mut ie = [0u8; 22];
    ie[..4].copy_from_slice(&[0x30, 20, 1, 0]);
    ie[4..8].copy_from_slice(&[0x00, 0x0F, 0xAC, group]);
    ie[8..14].copy_from_slice(&[1, 0, 0x00, 0x0F, 0xAC, SUITE_CCMP]);
    ie[14..20].copy_from_slice(&[1, 0, 0x00, 0x0F, 0xAC, AKM_PSK]);
    Ok(ie)
}

/// `bwfm_ext_join_params` for `ssid` at `bssid` (all ones: any).
pub fn ext_join_params(ssid: &[u8], bssid: &[u8; 6]) -> [u8; EXT_JOIN_LEN] {
    let mut p = [0u8; EXT_JOIN_LEN];
    p[0..4].copy_from_slice(&(ssid.len() as u32).to_le_bytes());
    p[4..4 + ssid.len()].copy_from_slice(ssid);
    // The join's own scan: type -1 and the four times -1, the firmware's defaults.
    p[36] = 0xFF;
    p[40..56].fill(0xFF);
    p[56..62].copy_from_slice(bssid);
    p
}

/// `bwfm_wsec_key`: a pairwise key bound to `peer`, or a group key (`peer` None).
pub fn wsec_key(key: &[u8], index: u32, algo: u32, peer: Option<&[u8; 6]>) -> [u8; WSEC_KEY_LEN] {
    let mut k = [0u8; WSEC_KEY_LEN];
    k[0..4].copy_from_slice(&index.to_le_bytes());
    k[4..8].copy_from_slice(&(key.len() as u32).to_le_bytes());
    k[8..8 + key.len().min(32)].copy_from_slice(&key[..key.len().min(32)]);
    k[112..116].copy_from_slice(&algo.to_le_bytes());
    let flags = if peer.is_none() { WSEC_PRIMARY_KEY } else { 0 };
    k[116..120].copy_from_slice(&flags.to_le_bytes());
    if let Some(ea) = peer {
        k[156..162].copy_from_slice(ea);
    }
    k
}

/// A refused command's error, renamed for the step it was, so the log says
/// which iovar the firmware did not take (the status stays).
fn named(what: &'static str) -> impl Fn(Error) -> Error {
    move |e| match e {
        Error::Card("firmware refused the command", status) => Error::Card(what, status),
        e => e,
    }
}

/// How a join ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JoinEnd {
    /// The 4-way handshake completed and both keys are in.
    Up,
    /// The firmware reported the association failed (event, status).
    Refused(u32, u32),
    /// Associated, but no complete handshake in time (wrong passphrase, usually).
    NoHandshake,
    /// No association in time.
    Timeout,
}

/// What a join reports along the way, for the log.
#[derive(Clone, Copy, Debug)]
pub struct JoinLog {
    pub associated: bool,
    pub eapol_in: u32,
    pub eapol_out: u32,
    pub rejected: u32,
    pub last_reject: Option<wpa_rust::Error>,
    /// Optional settings the firmware refused: bit 0 `sup_wpa`, bit 1 `mfp`.
    pub optional_refused: u8,
    pub optional_error: Option<Error>,
    /// The RSN element the firmware's association request carried, and whether it
    /// differed from the one we gave it (then message 2 used the firmware's).
    pub assoc_rsn: [u8; 64],
    pub assoc_rsn_len: usize,
    pub assoc_rsn_differs: bool,
}

impl Default for JoinLog {
    fn default() -> Self {
        Self {
            associated: false,
            eapol_in: 0,
            eapol_out: 0,
            rejected: 0,
            last_reject: None,
            optional_refused: 0,
            optional_error: None,
            assoc_rsn: [0; 64],
            assoc_rsn_len: 0,
            assoc_rsn_differs: false,
        }
    }
}

/// A joined station: what the link keeps after the 4-way handshake, to answer the AP's
/// group rekeys (`Bus::answer_eapol`) for as long as the association lasts.
pub struct Station {
    sup: Supplicant,
    /// The AP's address: EAPOL frames go to it.
    pub aa: [u8; 6],
    group_tkip: bool,
}

/// What one EAPOL frame from the AP did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Answer {
    /// The supplicant refused it (replayed, bad MIC, ...): nothing was sent.
    Rejected(wpa_rust::Error),
    /// `sent`: a reply went out. `complete`: it ended the 4-way handshake. `group`: a
    /// group key went in (message 3, or a rekey).
    Answered { sent: bool, complete: bool, group: bool },
}

/// Received frames the join cares about, collected during a poll.
struct Inbox {
    eapol: [[u8; 256]; 4],
    eapol_len: [usize; 4],
    n: usize,
    linked: bool,
    refused: Option<(u32, u32)>,
}

impl Inbox {
    fn take(&mut self, f: &Frame) {
        if f.channel != proto::CHANNEL_EVENT && f.channel != proto::CHANNEL_DATA {
            return;
        }
        let Some((_, eth)) = proto::strip_data_header(f.payload) else { return };
        if let Some(e) = proto::parse_event(eth) {
            match e.event_type {
                event::E_LINK if e.flags & 1 != 0 => self.linked = true,
                event::E_SET_SSID if e.status != 0 => self.refused = Some((e.event_type, e.status)),
                event::E_DEAUTH | event::E_DEAUTH_IND | event::E_DISASSOC | event::E_DISASSOC_IND
                    if self.linked =>
                {
                    self.refused = Some((e.event_type, e.reason))
                }
                _ => {}
            }
            return;
        }
        if eth.len() > 14 && u16::from_be_bytes([eth[12], eth[13]]) == ETHERTYPE_EAPOL && self.n < self.eapol.len() {
            let body = &eth[14..];
            let k = body.len().min(256);
            self.eapol[self.n][..k].copy_from_slice(&body[..k]);
            self.eapol_len[self.n] = k;
            self.n += 1;
        }
    }
}

/// Fresh randomness for the SNonce: SHA-1 over timer readings taken around work
/// whose duration varies (the PSK derivation, the association) and both addresses,
/// as the WDI path does. The kernel offers a driver no RNG here.
struct Nonce(Sha1);

impl Nonce {
    fn stir<P: Platform>(&mut self, bus: &Bus<P>) {
        self.0.update(&bus.chip.sdio.hc.p.now_us().to_le_bytes());
    }

    fn snonce(mut self, aa: &[u8; 6], spa: &[u8; 6]) -> [u8; 32] {
        self.0.update(aa);
        self.0.update(spa);
        let d1 = self.0.clone().digest();
        self.0.update(&d1);
        let d2 = self.0.digest();
        let mut n = [0u8; 32];
        n[..20].copy_from_slice(&d1);
        n[20..].copy_from_slice(&d2[..12]);
        n
    }
}

impl<P: Platform> Bus<P> {
    /// Run one EAPOL frame from the AP (from the 802.1X header) through the station's
    /// supplicant: send its reply, then install the keys it yields. The pairwise key goes
    /// in only after message 4 has left, since the key it confirms must not encrypt it.
    pub fn answer_eapol(&mut self, st: &mut Station, body: &[u8], mut each: impl FnMut(Frame)) -> Result<Answer, Error> {
        let mut reply = Reply { frame: [0; MAX_REPLY], len: 0, install_ptk: false, gtk: None, complete: false };
        if let Err(e) = st.sup.receive(body, &mut reply) {
            return Ok(Answer::Rejected(e));
        }
        let sent = reply.len != 0;
        if sent {
            self.send_data(&st.aa, ETHERTYPE_EAPOL, &reply.frame[..reply.len], &mut each)?;
        }
        if reply.install_ptk {
            self.chip.sdio.hc.p.stall_us(MSG4_DRAIN_US);
            let key = wsec_key(st.sup.tk(), 0, CRYPTO_ALGO_AES_CCM, Some(&st.aa));
            self.set_var(b"wsec_key", &key, &mut each).map_err(named("pairwise wsec_key refused"))?;
        }
        let group = reply.gtk.is_some();
        if let Some(g) = reply.gtk {
            let algo = if st.group_tkip { CRYPTO_ALGO_TKIP } else { CRYPTO_ALGO_AES_CCM };
            let key = wsec_key(&g.key[..g.len], g.id as u32, algo, None);
            self.set_var(b"wsec_key", &key, &mut each).map_err(named("group wsec_key refused"))?;
        }
        Ok(Answer::Answered { sent, complete: reply.complete, group })
    }

    /// The RSN element in the association request the firmware sent: `assoc_info`
    /// gives the request's IE length, `assoc_req_ies` the IEs. Some((offset, len))
    /// into `out`, or None when the request carried none.
    fn assoc_req_rsn(&mut self, out: &mut [u8; 480], mut each: impl FnMut(Frame)) -> Result<Option<(usize, usize)>, Error> {
        // The query must hold the firmware's whole assoc_info structure, or it answers
        // BCME_BUFTOOSHORT (-14, seen on the Pi 5); the lengths are its first 8 bytes.
        let info = &mut out[..];
        self.get_var(b"assoc_info", info, &mut each)?;
        let req_len = (u32::from_le_bytes([info[0], info[1], info[2], info[3]]) as usize).min(out.len());
        out.fill(0);
        if req_len == 0 {
            return Ok(None);
        }
        let n = self.get_var(b"assoc_req_ies", &mut out[..req_len], &mut each)?.min(req_len);
        let ies = &out[..n];
        Ok(crate::scan::find_ie(ies, 0x30).map(|rsn| {
            let at = rsn.as_ptr() as usize - ies.as_ptr() as usize;
            (at, rsn.len())
        }))
    }

    /// Join `cfg`'s network at `target` (the strongest scan entry for its SSID).
    /// `read_mac` must have run. `each` gets every frame the join does not consume.
    pub fn join(&mut self, cfg: &WifiConfig, target: &Bss, log: &mut JoinLog, each: impl FnMut(Frame)) -> Result<JoinEnd, Error> {
        self.join_station(cfg, target, log, each).map(|(end, _)| end)
    }

    /// `join`, keeping the station when the link comes up: its supplicant answers the
    /// AP's group rekeys from then on (`Bus::service`).
    pub fn join_station(
        &mut self,
        cfg: &WifiConfig,
        target: &Bss,
        log: &mut JoinLog,
        mut each: impl FnMut(Frame),
    ) -> Result<(JoinEnd, Option<Station>), Error> {
        let rsn = our_rsn(target.rsn()).map_err(|why| Error::Card(why, target.channel as u32))?;
        // Events and message 1 can arrive while a command still waits for its reply:
        // every frame read from here on goes through the inbox first.
        let inbox = RefCell::new(Inbox { eapol: [[0; 256]; 4], eapol_len: [0; 4], n: 0, linked: false, refused: None });
        let mut each = |f: Frame| {
            inbox.borrow_mut().take(&f);
            each(f);
        };
        let group_tkip = rsn[7] == SUITE_TKIP;
        let mut nonce = Nonce(Sha1::new());
        nonce.stir(self);
        let pmk = psk(&cfg.pass[..cfg.pass_len], cfg.ssid());
        nonce.stir(self);

        // Firmware without its own supplicant need not know `sup_wpa` (Linux only sets
        // it where the firmware has one; OpenBSD ignores the answer): the handshake is
        // ours either way. The refusal is logged, not fatal.
        if let Err(e) = self.set_var(b"sup_wpa", &0u32.to_le_bytes(), &mut each) {
            log.optional_refused |= 1;
            log.optional_error = Some(e);
        }
        self.set_var(b"wpaie", &rsn, &mut each).map_err(named("wpaie refused"))?;
        self.set_var(b"wpa_auth", &WPA_AUTH_WPA2_PSK.to_le_bytes(), &mut each).map_err(named("wpa_auth refused"))?;
        let wsec = WSEC_AES | if group_tkip { WSEC_TKIP } else { 0 };
        self.set_var(b"wsec", &wsec.to_le_bytes(), &mut each).map_err(named("wsec refused"))?;
        self.set_var(b"auth", &0u32.to_le_bytes(), &mut each).map_err(named("auth refused"))?;
        // Management frame protection off; older firmware does not know the iovar.
        if let Err(e) = self.set_var(b"mfp", &0u32.to_le_bytes(), &mut each) {
            log.optional_refused |= 2;
            log.optional_error = Some(e);
        }
        let params = ext_join_params(cfg.ssid(), &target.bssid);
        if self.set_var(b"join", &params, &mut each).is_err() {
            let mut j = [0u8; JOIN_LEN];
            j[..36].copy_from_slice(&params[..36]);
            j[36..42].copy_from_slice(&target.bssid);
            self.dcmd(cmd::C_SET_SSID, true, &[&j], &mut [], &mut each).map_err(named("SET_SSID refused"))?;
        }

        let aa = target.bssid;
        let spa = self.mac;
        nonce.stir(self);
        let snonce = nonce.snonce(&aa, &spa);
        let mut st = Station { sup: Supplicant::new(&pmk, &aa, &spa, &rsn, &snonce), aa, group_tkip };
        let mut checked_assoc_ie = false;
        let start = self.chip.sdio.hc.p.now_us();
        let mut frame = [0u8; 256];
        loop {
            self.poll(&mut each)?;
            let (refused, linked, n) = {
                let b = inbox.borrow();
                (b.refused, b.linked, b.n)
            };
            if let Some((e, s)) = refused {
                return Ok((JoinEnd::Refused(e, s), None));
            }
            log.associated |= linked;
            if n > 0 && !checked_assoc_ie {
                // Message 2 must repeat the RSN element the association request carried,
                // byte for byte, and the firmware wrote that request: ask it what it sent.
                checked_assoc_ie = true;
                let mut req = [0u8; 480];
                match self.assoc_req_rsn(&mut req, &mut each) {
                    Ok(Some((at, len))) => {
                        let sent = &req[at..at + len];
                        log.assoc_rsn_len = len.min(log.assoc_rsn.len());
                        log.assoc_rsn[..log.assoc_rsn_len].copy_from_slice(&sent[..log.assoc_rsn_len]);
                        if sent != rsn {
                            log.assoc_rsn_differs = true;
                            st.sup = Supplicant::new(&pmk, &aa, &spa, sent, &snonce);
                        }
                    }
                    Ok(None) => log.assoc_rsn_len = 0,
                    Err(e) => log.optional_error = Some(e),
                }
            }
            for i in 0..n {
                let len = {
                    let b = inbox.borrow();
                    frame[..b.eapol_len[i]].copy_from_slice(&b.eapol[i][..b.eapol_len[i]]);
                    b.eapol_len[i]
                };
                log.eapol_in += 1;
                match self.answer_eapol(&mut st, &frame[..len], &mut each)? {
                    Answer::Rejected(e) => {
                        log.rejected += 1;
                        log.last_reject = Some(e);
                    }
                    Answer::Answered { sent, complete, .. } => {
                        log.eapol_out += sent as u32;
                        if complete {
                            return Ok((JoinEnd::Up, Some(st)));
                        }
                    }
                }
            }
            // Those frames are handled; frames that arrived during the sends above
            // were queued behind them and are not.
            {
                let mut b = inbox.borrow_mut();
                let left = b.n - n;
                for k in 0..left {
                    b.eapol[k] = b.eapol[n + k];
                    b.eapol_len[k] = b.eapol_len[n + k];
                }
                b.n = left;
            }
            if self.chip.sdio.hc.p.now_us().wrapping_sub(start) > JOIN_WAIT_US {
                return Ok((if log.associated { JoinEnd::NoHandshake } else { JoinEnd::Timeout }, None));
            }
            self.chip.sdio.hc.p.stall_us(10_000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Buffers;
    use crate::chip::Chip;
    use crate::sdio::Sdio;
    use crate::sim::{Card, Sim, AP_BSSID, AP_PASS, AP_SSID};

    fn up(card: Card) -> Bus<Sim> {
        let mut s = Sdio::new(Sim::new(card));
        s.init().unwrap();
        let mut chip = Chip::attach(s).map_err(|(_, e)| e).unwrap();
        let mut fw = vec![0u8; 64];
        fw[..4].copy_from_slice(&0xB83E_F198u32.to_le_bytes());
        chip.load_firmware(&fw, &[0u8; 8]).unwrap();
        chip.start().unwrap();
        let mut bus = Bus::new(chip, Box::leak(Box::<Buffers>::default()));
        bus.read_mac(|_| {}).unwrap();
        bus.bring_up(|_| {}).unwrap();
        bus
    }

    fn target(bus: &mut Bus<Sim>) -> Bss {
        let mut best: Option<Bss> = None;
        bus.scan(|b| {
            if b.ssid() == AP_SSID && best.is_none_or(|x| b.rssi > x.rssi) {
                best = Some(*b);
            }
        })
        .unwrap();
        best.expect("the AP is in the scan")
    }

    fn cfg(pass: &[u8]) -> WifiConfig {
        let text = [b"ssid=".as_slice(), AP_SSID, b"\r\npsk=", pass, b"\r\n"].concat();
        WifiConfig::parse(&text).unwrap()
    }

    #[test]
    fn the_config_file_reads_as_the_wdi_path_reads_it() {
        let c = WifiConfig::parse(b"ssid=piconet\r\npsk=correct horse\r\nhidden=1\r\n").unwrap();
        assert_eq!((c.ssid(), &c.pass[..c.pass_len], c.hidden), (&b"piconet"[..], &b"correct horse"[..], true));
        assert!(WifiConfig::parse(b"ssid=x\npsk=short\n").is_none(), "a WPA2 passphrase is at least 8");
        assert!(WifiConfig::parse(b"psk=longenough\n").is_none(), "no SSID");
    }

    #[test]
    fn our_rsn_takes_the_aps_group_cipher_and_insists_on_ccmp_and_psk() {
        let ccmp = [0x30, 20, 1, 0, 0, 0x0F, 0xAC, 4, 1, 0, 0, 0x0F, 0xAC, 4, 1, 0, 0, 0x0F, 0xAC, 2, 0x0C, 0];
        assert_eq!(our_rsn(&ccmp).unwrap()[..20], ccmp[..20]);
        let mut tkip_group = ccmp;
        tkip_group[7] = 2;
        assert_eq!(our_rsn(&tkip_group).unwrap()[7], 2);
        let mut tkip_only = ccmp;
        tkip_only[13] = 2;
        assert_eq!(our_rsn(&tkip_only), Err("no CCMP pairwise cipher"));
        let mut eap = ccmp;
        eap[19] = 1;
        assert_eq!(our_rsn(&eap), Err("not a PSK network (802.1X?)"));
        assert_eq!(our_rsn(&[0xDD, 4, 0, 0x50, 0xF2, 1]), Err("no RSN element (not WPA2)"));
    }

    #[test]
    fn a_join_runs_the_handshake_and_installs_both_keys_after_message_4() {
        let mut bus = up(Card::cyw43455());
        let t = target(&mut bus);
        let mut log = JoinLog::default();
        assert_eq!(bus.join(&cfg(AP_PASS), &t, &mut log, |_| {}).unwrap(), JoinEnd::Up);
        assert_eq!((log.eapol_in, log.eapol_out, log.rejected), (2, 2, 0));
        let card = bus.chip.sdio.hc.p.card();
        assert_eq!(card.wpaie[..20], our_rsn(t.rsn()).unwrap()[..20], "the firmware got our RSN element");
        assert_eq!(card.joined_bssid, Some(AP_BSSID));
        // The AP saw message 4 before the pairwise key existed, and both keys match its own.
        assert_eq!(card.ap_log, vec!["msg2 ok", "msg4 ok", "pairwise key", "group key"]);
        assert!(card.keys_match, "TK and GTK as the AP derived them");
        assert!(!log.assoc_rsn_differs);
    }

    #[test]
    fn message_2_repeats_the_rsn_element_the_firmware_actually_sent() {
        let mut card = Card::cyw43455();
        card.fw_rewrites_rsn_caps = Some(0x000C);
        let mut bus = up(card);
        let t = target(&mut bus);
        let mut log = JoinLog::default();
        assert_eq!(bus.join(&cfg(AP_PASS), &t, &mut log, |_| {}).unwrap(), JoinEnd::Up);
        assert!(log.assoc_rsn_differs);
        assert_eq!(&log.assoc_rsn[20..22], &[0x0C, 0x00]);
    }

    #[test]
    fn a_wrong_passphrase_associates_but_never_completes() {
        let mut bus = up(Card::cyw43455());
        let t = target(&mut bus);
        let mut log = JoinLog::default();
        assert_eq!(bus.join(&cfg(b"not the passphrase"), &t, &mut log, |_| {}).unwrap(), JoinEnd::NoHandshake);
        assert!(log.associated);
        assert_eq!(bus.chip.sdio.hc.p.card().ap_log, vec!["msg2 bad mic"]);
    }

    #[test]
    fn a_refused_association_comes_back_with_its_status() {
        let mut card = Card::cyw43455();
        card.refuse_join = true;
        let mut bus = up(card);
        let t = target(&mut bus);
        let end = bus.join(&cfg(AP_PASS), &t, &mut JoinLog::default(), |_| {}).unwrap();
        assert_eq!(end, JoinEnd::Refused(event::E_SET_SSID, 1));
    }
}
