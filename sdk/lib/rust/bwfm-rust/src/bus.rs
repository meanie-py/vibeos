//! SDPCM over SDIO function 2: the running firmware's frames, and BCDC commands on
//! the control channel.
//!
//! Function 2 is a FIFO. A read at the frame address returns the next frame the
//! chip has queued, or zeros when it has none; a write sends one. Each frame's
//! software header carries the chip's credit (the highest sequence it accepts) and
//! the length of the frame after it, which the next read can take whole. After
//! OpenBSD's `if_bwfm_sdio.c` (`bwfm_sdio_rx_frames`, `bwfm_sdio_tx_ctrlframe`,
//! `bwfm_sdio_frame_read_write`) and `bwfm.c` (`bwfm_proto_bcdc_*`), ISC (notice
//! in chip.rs). Glom (several frames in one read) is not used yet: the firmware
//! only gloms when the host enables it (`bus:rxglom`).

use sdhci_rust::{Error, Platform};

use crate::chip::{Chip, CHIPCOMMON_BASE, SB_ACCESS_2_4B, SB_OFT_ADDR_MASK};
use crate::proto::{self, Credit, RxHeader, HEADER};

/// The largest frame we read: an Ethernet frame with its headers fits well under.
pub const SDPCM_FRAME_MAX: usize = 2048;
/// The largest command: a CLM chunk (1400 bytes and its 12-byte header) behind
/// `clmload\0` and the BCDC header, with room to spare.
pub const DCMD_MAX: usize = 1536;
/// One `clmload` chunk at most (`BWFM_DLOAD_MAX_LEN`).
pub const CLM_CHUNK: usize = 1400;
const DLOAD_FLAG_BEGIN: u16 = 1 << 1;
const DLOAD_FLAG_END: u16 = 1 << 2;
const DLOAD_FLAG_HANDLER_VER_1: u16 = 1 << 12;
const DLOAD_TYPE_CLM: u16 = 2;
const DLOAD_HEADER: usize = 12;
/// How long a command waits for its reply.
const DCMD_WAIT_US: u64 = 1_000_000;
/// How long the bus waits for the firmware's first credit.
const CREDIT_WAIT_US: u64 = 1_000_000;

/// A frame, as `Bus::poll` hands it on: its channel and its payload.
pub struct Frame<'a> {
    pub channel: u8,
    pub payload: &'a [u8],
}

pub struct Bus<P: Platform> {
    pub chip: Chip<P>,
    pub credit: Credit,
    /// The next frame's length, when the last header announced it.
    next_len: usize,
    rx: [u8; SDPCM_FRAME_MAX],
    tx: [u8; SDPCM_FRAME_MAX],
    /// A BCDC command being built: header, then the iovar.
    req: [u8; DCMD_MAX],
    dcmd_id: u16,
    /// Frames polled, and those that were not ours to keep (bad header, unknown
    /// channel, too big), for the log.
    pub frames: u32,
    pub dropped: u32,
}

/// What `Bus::poll` read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Polled {
    pub frames: u32,
    /// A header failed its checks: the FIFO is out of step and was abandoned.
    pub broken: bool,
}

impl<P: Platform> Bus<P> {
    /// The bus over a chip whose firmware `Chip::start` has brought up.
    pub fn new(chip: Chip<P>) -> Self {
        Self {
            chip,
            credit: Credit::default(),
            next_len: 0,
            rx: [0; SDPCM_FRAME_MAX],
            tx: [0; SDPCM_FRAME_MAX],
            req: [0; DCMD_MAX],
            dcmd_id: 0,
            frames: 0,
            dropped: 0,
        }
    }

    /// The function 2 frame address: chip-common's, in its window
    /// (`bwfm_sdio_frame_read_write`).
    fn frame_addr(&mut self) -> Result<u32, Error> {
        // Any 4-byte access to chip-common places the window there.
        self.chip.read_4(CHIPCOMMON_BASE)?;
        Ok((CHIPCOMMON_BASE & SB_OFT_ADDR_MASK) | SB_ACCESS_2_4B)
    }

    /// Read every frame the chip has queued, handing each to `each`. Empty frames
    /// only carry credit.
    pub fn poll(&mut self, mut each: impl FnMut(Frame)) -> Result<Polled, Error> {
        let addr = self.frame_addr()?;
        let mut out = Polled::default();
        loop {
            let first = if self.next_len >= HEADER { self.next_len.min(SDPCM_FRAME_MAX) } else { HEADER };
            self.chip.sdio.read(2, addr, false, &mut self.rx[..first])?;
            let mut h = [0u8; HEADER];
            h.copy_from_slice(&self.rx[..HEADER]);
            let header = match proto::parse_header(&h) {
                Ok(None) => {
                    self.next_len = 0;
                    return Ok(out);
                }
                Ok(Some(r)) => r,
                Err(_) => {
                    self.dropped += 1;
                    self.next_len = 0;
                    out.broken = true;
                    return Ok(out);
                }
            };
            let len = header.len as usize;
            if len > SDPCM_FRAME_MAX || (self.next_len >= HEADER && len > first) {
                self.dropped += 1;
                self.next_len = 0;
                out.broken = true;
                return Ok(out);
            }
            self.credit.max_seq = header.max_seq;
            self.next_len = header.next_len as usize;
            out.frames += 1;
            self.frames += 1;
            if len == HEADER {
                continue; // credit only
            }
            if first < len {
                let rest = (len - first).div_ceil(4) * 4;
                self.chip.sdio.read(2, addr, false, &mut self.rx[first..first + rest])?;
            }
            self.dispatch(&header, &mut each);
        }
    }

    fn dispatch(&mut self, h: &RxHeader, each: &mut impl FnMut(Frame)) {
        let (at, len) = (h.data_offset as usize, h.len as usize);
        match h.channel {
            proto::CHANNEL_CONTROL | proto::CHANNEL_EVENT | proto::CHANNEL_DATA => {
                each(Frame { channel: h.channel, payload: &self.rx[at..len] });
            }
            _ => self.dropped += 1,
        }
    }

    /// Send the first `n` bytes of `req` on the control channel, waiting for
    /// credit (reading frames meanwhile and handing them to `each`).
    fn send_request(&mut self, n: usize, mut each: impl FnMut(Frame)) -> Result<(), Error> {
        let start = self.chip.sdio.hc.p.now_us();
        let seq = loop {
            if let Some(s) = self.credit.take() {
                break s;
            }
            self.poll(&mut each)?;
            if self.chip.sdio.hc.p.now_us().wrapping_sub(start) > CREDIT_WAIT_US {
                return Err(Error::Card("no transmit credit from the firmware", self.credit.max_seq as u32));
            }
            self.chip.sdio.hc.p.stall_us(1000);
        };
        let Some(len) = proto::frame(proto::CHANNEL_CONTROL, seq, &[&self.req[..n]], &mut self.tx) else {
            return Err(Error::Card("control frame too big", n as u32));
        };
        let addr = self.frame_addr()?;
        self.chip.sdio.write(2, addr, true, &self.tx[..len])
    }

    /// One BCDC command and its reply: the reply's data goes to `out`, and its
    /// length comes back. A firmware error is `Error::Card("firmware refused ...", status)`.
    /// Frames that arrive meanwhile and are not the reply go to `each`.
    /// `parts` are the command's data, concatenated.
    pub fn dcmd(
        &mut self,
        command: u32,
        set: bool,
        parts: &[&[u8]],
        out: &mut [u8],
        mut each: impl FnMut(Frame),
    ) -> Result<usize, Error> {
        self.dcmd_id = self.dcmd_id.wrapping_add(1);
        let id = self.dcmd_id;
        let len: usize = parts.iter().map(|p| p.len()).sum();
        if proto::DCMD_HEADER + len > self.req.len() {
            return Err(Error::Card("command too big", len as u32));
        }
        let mut at = proto::DCMD_HEADER;
        for p in parts {
            self.req[at..at + p.len()].copy_from_slice(p);
            at += p.len();
        }
        // The header goes in front of the data already in place.
        let mut head = [0u8; proto::DCMD_HEADER];
        proto::encode_dcmd(command, set, id, 0, &[], &mut head);
        head[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        self.req[..proto::DCMD_HEADER].copy_from_slice(&head);
        self.send_request(at, &mut each)?;
        let start = self.chip.sdio.hc.p.now_us();
        let mut got: Option<Result<usize, Error>> = None;
        loop {
            self.poll(|f| {
                if got.is_none() && f.channel == proto::CHANNEL_CONTROL {
                    if let Some(r) = proto::parse_dcmd(f.payload) {
                        if r.id == id && r.command == command {
                            got = Some(match r.error {
                                Some(status) => Err(Error::Card("firmware refused the command", status as u32)),
                                None => {
                                    let k = r.data.len().min(out.len());
                                    out[..k].copy_from_slice(&r.data[..k]);
                                    Ok(k)
                                }
                            });
                            return;
                        }
                    }
                }
                each(f);
            })?;
            if let Some(r) = got {
                return r;
            }
            if self.chip.sdio.hc.p.now_us().wrapping_sub(start) > DCMD_WAIT_US {
                return Err(Error::Card("no reply to the command", command));
            }
            self.chip.sdio.hc.p.stall_us(1000);
        }
    }

    /// GET an iovar: `name`'s value into `out`.
    pub fn get_var(&mut self, name: &[u8], out: &mut [u8], each: impl FnMut(Frame)) -> Result<usize, Error> {
        // The query's length is also the most the reply may carry: ask for as
        // much as `out` holds, as zeros after the name.
        const ZEROS: [u8; 480] = [0; 480];
        let room = out.len().min(ZEROS.len()).saturating_sub(name.len() + 1);
        self.dcmd(proto::cmd::C_GET_VAR, false, &[name, &[0], &ZEROS[..room]], out, each)
    }

    /// SET an iovar: `name\0` then `value`.
    pub fn set_var(&mut self, name: &[u8], value: &[u8], each: impl FnMut(Frame)) -> Result<(), Error> {
        self.dcmd(proto::cmd::C_SET_VAR, true, &[name, &[0], value], &mut [], each).map(|_| ())
    }

    /// Hand the firmware its regulatory table (the CLM blob) in `clmload`
    /// chunks (`bwfm_process_blob`). Returns the chunks sent.
    pub fn load_clm(&mut self, blob: &[u8], mut each: impl FnMut(Frame)) -> Result<usize, Error> {
        let chunks = blob.len().div_ceil(CLM_CHUNK);
        for (i, chunk) in blob.chunks(CLM_CHUNK).enumerate() {
            let mut flag = DLOAD_FLAG_HANDLER_VER_1;
            if i == 0 {
                flag |= DLOAD_FLAG_BEGIN;
            }
            if i + 1 == chunks {
                flag |= DLOAD_FLAG_END;
            }
            let mut head = [0u8; DLOAD_HEADER];
            head[0..2].copy_from_slice(&flag.to_le_bytes());
            head[2..4].copy_from_slice(&DLOAD_TYPE_CLM.to_le_bytes());
            head[4..8].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
            // CRC 0: the firmware does not check it (OpenBSD and Linux send 0).
            self.dcmd(proto::cmd::C_SET_VAR, true, &[b"clmload\0", &head, chunk], &mut [], &mut each)?;
        }
        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Card, Sim, FW_VERSION};
    use crate::sdio::Sdio;

    fn running() -> Bus<Sim> {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        let mut chip = Chip::attach(s).map_err(|(_, e)| e).unwrap();
        let mut fw = vec![0u8; 4096];
        fw[..4].copy_from_slice(&0xB83E_F198u32.to_le_bytes());
        let mut nv = [0u8; 32];
        let n = crate::nvram::convert(b"a=1\n", &mut nv).unwrap();
        chip.load_firmware(&fw, &nv[..n]).unwrap();
        chip.start().unwrap();
        Bus::new(chip)
    }

    #[test]
    fn the_firmware_announces_credit_in_an_empty_frame() {
        let mut bus = running();
        let mut seen = 0;
        let p = bus.poll(|_| seen += 1).unwrap();
        assert_eq!((p.frames, p.broken, seen), (1, false, 0), "one credit-only frame, nothing handed on");
        assert_eq!(bus.credit.window(), 4);
    }

    #[test]
    fn ver_is_one_command_and_its_reply() {
        let mut bus = running();
        let mut out = [0u8; 256];
        let n = bus.get_var(b"ver", &mut out, |_| {}).unwrap();
        let text = &out[..n];
        assert!(text.starts_with(FW_VERSION), "{:?}", core::str::from_utf8(text));
        assert_eq!(bus.credit.seq, 1, "one credit spent");
        // The request went out as one control frame, padded to a word.
        let card = bus.chip.sdio.hc.p.card();
        let (_, _, len) = *card.f2_writes.last().unwrap();
        assert_eq!(len % 4, 0);
    }

    #[test]
    fn a_late_reply_to_an_earlier_request_is_not_taken_for_this_one() {
        let mut bus = running();
        bus.poll(|_| {}).unwrap(); // take the credit first, so the stale reply waits behind the request
        // A reply with id 0xBEEF for the same command must not answer request 1.
        bus.chip.sdio.hc.p.card_mut().queue_stale_reply(proto::cmd::C_GET_VAR, 0xBEEF, b"stale\0");
        let mut out = [0u8; 256];
        let n = bus.get_var(b"ver", &mut out, |_| {}).unwrap();
        assert!(out[..n].starts_with(FW_VERSION));
    }

    #[test]
    fn requests_go_out_whole() {
        let mut bus = running();
        bus.get_var(b"ver", &mut [0u8; 256], |_| {}).unwrap();
        assert!(!bus.chip.sdio.hc.p.card().desynced);
    }

    #[test]
    fn the_clm_goes_in_chunks_marked_begin_and_end() {
        let mut bus = running();
        let blob: Vec<u8> = (0..2676u32).map(|i| i as u8).collect(); // the Pi's is 2,676 bytes
        assert_eq!(bus.load_clm(&blob, |_| {}).unwrap(), 2);
        let card = bus.chip.sdio.hc.p.card();
        assert_eq!(card.clm, blob, "the chunks reassemble to the blob");
        assert_eq!(card.clm_flags, vec![DLOAD_FLAG_HANDLER_VER_1 | DLOAD_FLAG_BEGIN, DLOAD_FLAG_HANDLER_VER_1 | DLOAD_FLAG_END]);
    }

    #[test]
    fn the_mac_address_is_an_iovar() {
        let mut bus = running();
        let mut mac = [0u8; 6];
        assert_eq!(bus.get_var(b"cur_etheraddr", &mut mac, |_| {}).unwrap(), 6);
        assert_eq!(mac, crate::sim::MAC);
    }

    #[test]
    fn a_firmware_error_comes_back_as_its_status() {
        let mut bus = running();
        let mut out = [0u8; 64];
        match bus.get_var(b"nosuchvar", &mut out, |_| {}) {
            Err(Error::Card(what, status)) => assert_eq!((what, status as i32), ("firmware refused the command", -23)),
            r => panic!("expected the firmware's error, got {r:?}"),
        }
    }

    #[test]
    fn without_credit_a_command_fails_instead_of_overrunning_the_chip() {
        let mut bus = running();
        bus.chip.sdio.hc.p.card_mut().credit_frames = false;
        bus.chip.sdio.hc.p.card_mut().drop_pending_rx();
        assert!(matches!(bus.get_var(b"ver", &mut [0u8; 64], |_| {}), Err(Error::Card(w, _)) if w.contains("credit")));
    }

    #[test]
    fn an_event_queued_before_the_reply_is_handed_on_not_lost() {
        let mut bus = running();
        bus.chip.sdio.hc.p.card_mut().queue_event(b"\x88\x6c event");
        let mut events = Vec::new();
        let n = bus.get_var(b"ver", &mut [0u8; 256], |f| events.push((f.channel, f.payload.to_vec()))).unwrap();
        assert!(n > 0);
        assert_eq!(events, vec![(proto::CHANNEL_EVENT, b"\x88\x6c event".to_vec())]);
    }
}
