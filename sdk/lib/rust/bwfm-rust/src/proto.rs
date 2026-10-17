//! The CYW43455's host protocol over SDIO, as bytes: SDPCM framing, BCDC commands and data
//! headers, and firmware events. Pure code over buffers, so the host tests it
//! (`tools/test_bwfm_proto.py`) before there is a bus to put it on.
//!
//! Ported from OpenBSD's bwfm (`sys/dev/sdmmc/if_bwfm_sdio.c`, `sys/dev/ic/bwfm.c`,
//! `bwfmreg.h`), which carries this notice:
//!
//! Copyright (c) 2010-2016 Broadcom Corporation
//! Copyright (c) 2016,2017 Patrick Wildt <patrick@blueri.se>
//!
//! Permission to use, copy, modify, and/or distribute this software for any
//! purpose with or without fee is hereby granted, provided that the above
//! copyright notice and this permission notice appear in all copies.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//! WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//! ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//! WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//! ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//! OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

// ---- SDPCM -----------------------------------------------------------------------------------
//
// Every transfer on function 2 is a frame: a hardware header (length and its complement, both
// little-endian) and a software header (sequence, channel, the next frame's length in 16-byte
// units, where the payload starts, flow control, and the highest sequence the chip will
// accept), then the payload.

pub const HW_HEADER: usize = 4;
pub const SW_HEADER: usize = 8;
pub const HEADER: usize = HW_HEADER + SW_HEADER;

pub const CHANNEL_CONTROL: u8 = 0;
pub const CHANNEL_EVENT: u8 = 1;
pub const CHANNEL_DATA: u8 = 2;
pub const CHANNEL_GLOM: u8 = 3;
const CHANNEL_MASK: u8 = 0x0F;

/// SDIO block size on function 2: a frame longer than one block is padded to whole blocks.
pub const BLOCK: usize = 512;

/// The length a frame of `len` bytes is written as: whole blocks past one block, else a
/// multiple of four.
pub fn padded(len: usize) -> usize {
    let to = if len > BLOCK && len % BLOCK != 0 { BLOCK } else { 4 };
    len.div_ceil(to) * to
}

/// Build a frame on `channel` with sequence `seq` from the payload `parts`, zero-padded as
/// `padded` says. Returns the length to write, or `None` when `out` is too small.
pub fn frame(channel: u8, seq: u8, parts: &[&[u8]], out: &mut [u8]) -> Option<usize> {
    let len = HEADER + parts.iter().map(|p| p.len()).sum::<usize>();
    let total = padded(len);
    if len > u16::MAX as usize || total > out.len() {
        return None;
    }
    out[0..2].copy_from_slice(&(len as u16).to_le_bytes());
    out[2..4].copy_from_slice(&(!(len as u16)).to_le_bytes());
    out[4] = seq;
    out[5] = channel & CHANNEL_MASK;
    out[6] = 0; // next length: the chip's to announce, not ours
    out[7] = HEADER as u8; // data offset
    out[8] = 0; // flow control
    out[9] = 0; // max sequence
    out[10] = 0;
    out[11] = 0;
    let mut at = HEADER;
    for p in parts {
        out[at..at + p.len()].copy_from_slice(p);
        at += p.len();
    }
    out[len..total].fill(0);
    Some(total)
}

/// A received frame's headers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RxHeader {
    /// The whole frame, headers included.
    pub len: u16,
    pub seq: u8,
    pub channel: u8,
    /// The next frame's length in bytes, when the chip says (it reads ahead with it).
    pub next_len: u16,
    /// Where the payload starts, from the start of the frame.
    pub data_offset: u8,
    pub flow_control: u8,
    /// The highest sequence number the chip will accept: the credit window's end.
    pub max_seq: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RxError {
    /// The length and its complement disagree.
    Checksum,
    /// Shorter than its own headers, or a payload offset outside it.
    Length,
}

/// Parse the first `HEADER` bytes of a frame. `Ok(None)`: no frame (both lengths zero).
pub fn parse_header(h: &[u8; HEADER]) -> Result<Option<RxHeader>, RxError> {
    let len = u16::from_le_bytes([h[0], h[1]]);
    let check = u16::from_le_bytes([h[2], h[3]]);
    if len == 0 && check == 0 {
        return Ok(None);
    }
    if len ^ check != 0xFFFF {
        return Err(RxError::Checksum);
    }
    if (len as usize) < HEADER {
        return Err(RxError::Length);
    }
    let r = RxHeader {
        len,
        seq: h[4],
        channel: h[5] & CHANNEL_MASK,
        next_len: (h[6] as u16) << 4,
        data_offset: h[7],
        flow_control: h[8],
        max_seq: h[9],
    };
    if (r.data_offset as usize) < HEADER || r.data_offset as u16 > len {
        return Err(RxError::Length);
    }
    Ok(Some(r))
}

/// The transmit credit: frames may go out while `seq` is behind the chip's `max_seq`, by
/// less than half the sequence space (the comparison wraps).
#[derive(Clone, Copy, Debug, Default)]
pub struct Credit {
    pub seq: u8,
    pub max_seq: u8,
}

impl Credit {
    /// Frames that may be sent now.
    pub fn window(&self) -> u8 {
        let w = self.max_seq.wrapping_sub(self.seq);
        if w & 0x80 != 0 { 0 } else { w }
    }
    /// The sequence for the next frame, consuming one credit; `None` when there is none.
    pub fn take(&mut self) -> Option<u8> {
        if self.window() == 0 {
            return None;
        }
        let s = self.seq;
        self.seq = self.seq.wrapping_add(1);
        Some(s)
    }
}

/// A glom (superframe) descriptor: the payload is a list of little-endian u16 lengths, one per
/// frame that the next read returns back to back.
pub fn glom_lengths(payload: &[u8]) -> impl Iterator<Item = u16> + '_ {
    payload.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]))
}

// ---- BCDC control ----------------------------------------------------------------------------
//
// A command on the control channel: command number, payload length, flags (GET/SET, the
// interface, a request id the reply echoes, an error bit), status; little-endian. Then the
// payload, which for an iovar is `name\0value`.

pub const DCMD_HEADER: usize = 16;
const DCMD_ERROR: u32 = 1 << 0;
const DCMD_SET: u32 = 1 << 1;

/// The firmware command numbers used here (bwfmreg.h `BWFM_C_*`, prefix kept: commands and
/// events share names with different numbers).
pub mod cmd {
    pub const C_GET_VERSION: u32 = 1;
    pub const C_UP: u32 = 2;
    pub const C_DOWN: u32 = 3;
    pub const C_GET_RATE: u32 = 12;
    pub const C_SET_INFRA: u32 = 20;
    pub const C_SET_AUTH: u32 = 22;
    pub const C_GET_BSSID: u32 = 23;
    pub const C_SET_SSID: u32 = 26;
    pub const C_SET_KEY: u32 = 45;
    pub const C_SET_PASSIVE_SCAN: u32 = 49;
    pub const C_DISASSOC: u32 = 52;
    pub const C_GET_VAR: u32 = 262;
    pub const C_SET_VAR: u32 = 263;
}

/// Encode a command: header then `data`. Returns the length, or `None` when `out` is short.
pub fn encode_dcmd(command: u32, set: bool, id: u16, ifidx: u8, data: &[u8], out: &mut [u8]) -> Option<usize> {
    let len = DCMD_HEADER + data.len();
    if len > out.len() {
        return None;
    }
    let flags = if set { DCMD_SET } else { 0 } | ((ifidx as u32 & 0xF) << 12) | ((id as u32) << 16);
    out[0..4].copy_from_slice(&command.to_le_bytes());
    out[4..8].copy_from_slice(&(data.len() as u32).to_le_bytes());
    out[8..12].copy_from_slice(&flags.to_le_bytes());
    out[12..16].copy_from_slice(&0u32.to_le_bytes());
    out[DCMD_HEADER..len].copy_from_slice(data);
    Some(len)
}

/// A command's reply.
#[derive(Debug, PartialEq, Eq)]
pub struct DcmdReply<'a> {
    pub command: u32,
    pub id: u16,
    pub ifidx: u8,
    /// The firmware's status when it flagged an error, else `None`.
    pub error: Option<i32>,
    pub data: &'a [u8],
}

pub fn parse_dcmd(buf: &[u8]) -> Option<DcmdReply<'_>> {
    if buf.len() < DCMD_HEADER {
        return None;
    }
    let u = |i: usize| u32::from_le_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
    let (command, len, flags, status) = (u(0), u(4) as usize, u(8), u(12));
    let data = &buf[DCMD_HEADER..];
    Some(DcmdReply {
        command,
        id: (flags >> 16) as u16,
        ifidx: ((flags >> 12) & 0xF) as u8,
        error: if flags & DCMD_ERROR != 0 { Some(status as i32) } else { None },
        // The reply may come padded: the header's length is the payload's.
        data: &data[..len.min(data.len())],
    })
}

/// An iovar's payload: `name`, a NUL, then `value`. Returns the length.
pub fn iovar(name: &[u8], value: &[u8], out: &mut [u8]) -> Option<usize> {
    let len = name.len() + 1 + value.len();
    if len > out.len() {
        return None;
    }
    out[..name.len()].copy_from_slice(name);
    out[name.len()] = 0;
    out[name.len() + 1..len].copy_from_slice(value);
    Some(len)
}

// ---- BCDC data -------------------------------------------------------------------------------

pub const DATA_HEADER: usize = 4;
const BCDC_PROTO_VER: u8 = 2;

/// The header in front of each Ethernet frame we send: protocol version 2, the 802.1D
/// priority, interface 0, no extra offset.
pub fn data_header(priority: u8) -> [u8; DATA_HEADER] {
    [BCDC_PROTO_VER << 4, priority & 7, 0, 0]
}

/// A received data or event frame's BCDC header removed: (interface, Ethernet frame). The
/// data offset counts 4-byte words past the header.
pub fn strip_data_header(buf: &[u8]) -> Option<(u8, &[u8])> {
    if buf.len() < DATA_HEADER {
        return None;
    }
    let skip = DATA_HEADER + ((buf[3] as usize) << 2);
    if skip > buf.len() {
        return None;
    }
    Some((buf[2] & 0xF, &buf[skip..]))
}

// ---- events ----------------------------------------------------------------------------------
//
// An Ethernet frame of type 0x886C: a Broadcom header (subtype, length, version, OUI 00-10-18,
// user subtype 1) and the event message, whose fields are big-endian.

pub const ETHERTYPE_LINK_CTL: u16 = 0x886C;
const BRCM_OUI: [u8; 3] = [0x00, 0x10, 0x18];
const ETH_HEADER: usize = 14;
const BRCM_HEADER: usize = 10;
const EVENT_MSG: usize = 48; // through bsscfgidx: 2+2+4*5+6+16+1+1

/// Event numbers (bwfmreg.h `BWFM_E_*`).
pub mod event {
    pub const E_SET_SSID: u32 = 0;
    pub const E_JOIN: u32 = 1;
    pub const E_AUTH: u32 = 3;
    pub const E_DEAUTH: u32 = 5;
    pub const E_DEAUTH_IND: u32 = 6;
    pub const E_ASSOC: u32 = 7;
    pub const E_DISASSOC: u32 = 11;
    pub const E_DISASSOC_IND: u32 = 12;
    pub const E_LINK: u32 = 16;
    pub const E_EAPOL_MSG: u32 = 25;
    pub const E_IF: u32 = 54;
    pub const E_ESCAN_RESULT: u32 = 69;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Event<'a> {
    pub event_type: u32,
    pub status: u32,
    pub reason: u32,
    pub flags: u16,
    pub addr: [u8; 6],
    pub ifidx: u8,
    /// The event's data (an escan result's BSS list, for one).
    pub data: &'a [u8],
}

/// The event an Ethernet frame carries, if it is one.
pub fn parse_event(eth: &[u8]) -> Option<Event<'_>> {
    if eth.len() < ETH_HEADER + BRCM_HEADER + EVENT_MSG {
        return None;
    }
    if u16::from_be_bytes([eth[12], eth[13]]) != ETHERTYPE_LINK_CTL {
        return None;
    }
    let b = &eth[ETH_HEADER..];
    if b[5..8] != BRCM_OUI || u16::from_be_bytes([b[8], b[9]]) != 1 {
        return None;
    }
    let m = &b[BRCM_HEADER..];
    let be32 = |i: usize| u32::from_be_bytes([m[i], m[i + 1], m[i + 2], m[i + 3]]);
    let datalen = be32(20) as usize;
    let rest = &m[EVENT_MSG..];
    let mut addr = [0u8; 6];
    addr.copy_from_slice(&m[24..30]);
    Some(Event {
        flags: u16::from_be_bytes([m[2], m[3]]),
        event_type: be32(4),
        status: be32(8),
        reason: be32(12),
        addr,
        ifidx: m[46],
        data: &rest[..datalen.min(rest.len())],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_layout_and_padding() {
        let mut out = [0xEEu8; 64];
        let n = frame(CHANNEL_CONTROL, 7, &[b"abc", b"de"], &mut out).unwrap();
        // 12 + 5 = 17 bytes, padded to 20.
        assert_eq!(n, 20);
        assert_eq!(&out[0..4], &[17, 0, !17u8, 0xFF]);
        assert_eq!(&out[4..12], &[7, 0, 0, 12, 0, 0, 0, 0]);
        assert_eq!(&out[12..17], b"abcde");
        assert_eq!(&out[17..20], &[0, 0, 0]);
        assert_eq!(out[20], 0xEE, "nothing written past the padded length");
    }

    #[test]
    fn frames_past_a_block_pad_to_whole_blocks() {
        assert_eq!(padded(512), 512);
        assert_eq!(padded(513), 1024);
        assert_eq!(padded(1024), 1024);
        assert_eq!(padded(511), 512);
        assert_eq!(padded(13), 16);
        let mut out = [0u8; 2048];
        let body = [1u8; 600];
        assert_eq!(frame(CHANNEL_DATA, 0, &[&body], &mut out), Some(1024));
        assert_eq!(frame(CHANNEL_DATA, 0, &[&body], &mut out[..1000]), None);
    }

    #[test]
    fn header_round_trip_and_rejections() {
        let mut out = [0u8; 32];
        frame(CHANNEL_EVENT, 9, &[b"xyzw"], &mut out).unwrap();
        out[6] = 3; // the chip's next-length announcement, in 16-byte units
        out[9] = 20;
        let h = parse_header(out[..HEADER].try_into().unwrap()).unwrap().unwrap();
        assert_eq!(h, RxHeader { len: 16, seq: 9, channel: CHANNEL_EVENT, next_len: 48, data_offset: 12, flow_control: 0, max_seq: 20 });

        assert_eq!(parse_header(&[0; HEADER]), Ok(None));
        let mut bad = out;
        bad[2] ^= 1;
        assert_eq!(parse_header(bad[..HEADER].try_into().unwrap()), Err(RxError::Checksum));
        let mut short = [0u8; HEADER];
        short[0..4].copy_from_slice(&[8, 0, !8u8, 0xFF]);
        assert_eq!(parse_header(&short), Err(RxError::Length));
        let mut off = out;
        off[7] = 4; // payload inside the headers
        assert_eq!(parse_header(off[..HEADER].try_into().unwrap()), Err(RxError::Length));
    }

    #[test]
    fn credit_window_wraps() {
        let mut c = Credit { seq: 250, max_seq: 2 };
        assert_eq!(c.window(), 8);
        for want in [250u8, 251, 252, 253, 254, 255, 0, 1] {
            assert_eq!(c.take(), Some(want));
        }
        assert_eq!(c.take(), None);
        // A max sequence behind ours (by less than half the space) is no credit at all.
        assert_eq!(Credit { seq: 10, max_seq: 5 }.window(), 0);
    }

    #[test]
    fn glom_descriptor() {
        let d = [0x40, 0x00, 0x10, 0x02];
        assert_eq!(glom_lengths(&d).collect::<Vec<_>>(), vec![0x40, 0x210]);
    }

    #[test]
    fn dcmd_round_trip() {
        let mut v = [0u8; 32];
        let n = iovar(b"cur_etheraddr", &[], &mut v).unwrap();
        assert_eq!(&v[..n], b"cur_etheraddr\0");
        let mut out = [0u8; 64];
        let len = encode_dcmd(cmd::C_GET_VAR, false, 0x1234, 0, &v[..n], &mut out).unwrap();
        assert_eq!(len, DCMD_HEADER + 14);
        assert_eq!(&out[0..4], &262u32.to_le_bytes());
        assert_eq!(&out[4..8], &14u32.to_le_bytes());
        assert_eq!(&out[8..12], &0x1234_0000u32.to_le_bytes());

        // The reply: same header, the MAC as data, padded past the stated length.
        let mut reply = [0u8; 40];
        reply[..16].copy_from_slice(&out[..16]);
        reply[4..8].copy_from_slice(&6u32.to_le_bytes());
        reply[16..22].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
        let r = parse_dcmd(&reply).unwrap();
        assert_eq!(r, DcmdReply { command: 262, id: 0x1234, ifidx: 0, error: None, data: &[2, 0, 0, 0, 0, 1] });

        let set = encode_dcmd(cmd::C_UP, true, 1, 0, &[], &mut out).unwrap();
        assert_eq!(set, DCMD_HEADER);
        assert_eq!(&out[8..12], &(0x1_0000u32 | 2).to_le_bytes());
        out[8] |= 1; // error flag
        out[12..16].copy_from_slice(&(-23i32).to_le_bytes());
        assert_eq!(parse_dcmd(&out[..set]).unwrap().error, Some(-23));
        assert_eq!(parse_dcmd(&out[..8]), None);
    }

    #[test]
    fn data_header_and_strip() {
        assert_eq!(data_header(5), [0x20, 5, 0, 0]);
        assert_eq!(data_header(9)[1], 1, "priority is three bits");
        let rx = [0x20, 0, 0x01, 1, 0xAA, 0xBB, 0xCC, 0xDD, 9, 8, 7];
        assert_eq!(strip_data_header(&rx), Some((1, &[9u8, 8, 7][..])));
        assert_eq!(strip_data_header(&rx[..6]), None);
    }

    fn event_frame(event_type: u32, status: u32, data: &[u8]) -> Vec<u8> {
        let mut f = vec![0u8; 12];
        f.extend_from_slice(&ETHERTYPE_LINK_CTL.to_be_bytes());
        f.extend_from_slice(&[0x80, 0x01, 0, 0, 1, 0x00, 0x10, 0x18, 0, 1]);
        let mut m = vec![0u8; EVENT_MSG];
        m[0..2].copy_from_slice(&2u16.to_be_bytes());
        m[2..4].copy_from_slice(&1u16.to_be_bytes());
        m[4..8].copy_from_slice(&event_type.to_be_bytes());
        m[8..12].copy_from_slice(&status.to_be_bytes());
        m[12..16].copy_from_slice(&3u32.to_be_bytes());
        m[20..24].copy_from_slice(&(data.len() as u32).to_be_bytes());
        m[24..30].copy_from_slice(&[0x10, 0x7C, 0x61, 0xA7, 0x7B, 0x21]);
        m[46] = 0;
        f.extend_from_slice(&m);
        f.extend_from_slice(data);
        f.extend_from_slice(&[0xFF; 3]); // trailing pad the firmware may leave
        f
    }

    #[test]
    fn events_are_big_endian() {
        let f = event_frame(event::E_LINK, 0, b"ies");
        let e = parse_event(&f).unwrap();
        assert_eq!(e.event_type, event::E_LINK);
        assert_eq!(e.reason, 3);
        assert_eq!(e.flags, 1);
        assert_eq!(e.addr, [0x10, 0x7C, 0x61, 0xA7, 0x7B, 0x21]);
        assert_eq!(e.data, b"ies");

        let mut not_event = f.clone();
        not_event[13] = 0x8E; // EAPOL, not an event
        assert_eq!(parse_event(&not_event), None);
        let mut other_oui = f.clone();
        other_oui[ETH_HEADER + 6] = 0x11;
        assert_eq!(parse_event(&other_oui), None);
        assert_eq!(parse_event(&f[..40]), None);
    }
}
