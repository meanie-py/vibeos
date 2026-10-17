//! The chip behind SDIO function 1: the Broadcom/Cypress silicon backplane.
//!
//! Function 1 shows a 32 KiB window of the chip's 32-bit backplane, placed by
//! three address registers; an access with bit 15 set in its SDIO address is a
//! 4-byte backplane access. Before anything but those registers answers, the
//! host must ask for the ALP (always-on) clock. Then the chip-common core at
//! 0x1800_0000 gives the chip id. After OpenBSD's `if_bwfm_sdio.c`
//! (`bwfm_sdio_attach`, `bwfm_sdio_backplane`, `bwfm_sdio_read_4`) and its
//! register names (`if_bwfm_sdio.h`), ISC licensed:
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

use sdhci_rust::{Error, Platform};

use crate::sdio::Sdio;

pub const F1_SBADDR_LOW: u32 = 0x1000A;
pub const F1_SBADDR_MID: u32 = 0x1000B;
pub const F1_SBADDR_HIGH: u32 = 0x1000C;
pub const F1_CHIPCLKCSR: u32 = 0x1000E;
pub const F1_SDIOPULLUP: u32 = 0x1000F;

pub const CLK_FORCE_ALP: u8 = 0x01;
pub const CLK_ALP_AVAIL_REQ: u8 = 0x08;
pub const CLK_FORCE_HW_CLKREQ_OFF: u8 = 0x20;
pub const CLK_ALP_AVAIL: u8 = 0x40;
pub const CLK_HT_AVAIL: u8 = 0x80;

/// The low 15 bits of a backplane address go in the SDIO address; the rest
/// place the window.
pub const SB_OFT_ADDR_MASK: u32 = 0x7FFF;
pub const SB_ACCESS_2_4B: u32 = 0x8000;

/// The chip-common core; its first register is the chip id.
pub const CHIPCOMMON_BASE: u32 = 0x1800_0000;

/// The block sizes OpenBSD and Linux both use for the 43455: the backplane
/// function's 64 and the frame function's 512.
pub const F1_BLOCK: u16 = 64;
pub const F2_BLOCK: u16 = 512;

/// How long the chip gets to start its ALP clock. Linux allows the PMU 15 ms.
const ALP_WAIT_US: u64 = 100_000;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct ChipId {
    pub raw: u32,
    /// 0x4345 for the CYW43455.
    pub chip: u16,
    pub rev: u8,
    pub package: u8,
    pub cores: u8,
    /// Interconnect: 1 = AXI (AI), which has an enumeration ROM.
    pub socitype: u8,
}

impl ChipId {
    pub fn decode(raw: u32) -> Self {
        Self {
            raw,
            chip: raw as u16,
            rev: ((raw >> 16) & 0xF) as u8,
            package: ((raw >> 20) & 0xF) as u8,
            cores: ((raw >> 24) & 0xF) as u8,
            socitype: (raw >> 28) as u8,
        }
    }
}

pub struct Chip<P: Platform> {
    pub sdio: Sdio<P>,
    /// The backplane address the window is at; `u32::MAX` until first placed.
    window: u32,
    pub id: ChipId,
}

impl<P: Platform> Chip<P> {
    /// Set the block sizes, enable function 1, start the ALP clock and read the
    /// chip id. `sdio` must have been through `Sdio::init`.
    pub fn attach(mut sdio: Sdio<P>) -> Result<Self, (Sdio<P>, Error)> {
        if let Err(e) = Self::prepare(&mut sdio) {
            return Err((sdio, e));
        }
        let mut chip = Self { sdio, window: u32::MAX, id: ChipId::default() };
        match chip.read_4(CHIPCOMMON_BASE) {
            Ok(raw) => chip.id = ChipId::decode(raw),
            Err(e) => return Err((chip.sdio, e)),
        }
        Ok(chip)
    }

    fn prepare(sdio: &mut Sdio<P>) -> Result<(), Error> {
        sdio.set_block_size(1, F1_BLOCK)?;
        sdio.set_block_size(2, F2_BLOCK)?;
        sdio.enable_function(1)?;
        let want = CLK_FORCE_HW_CLKREQ_OFF | CLK_ALP_AVAIL_REQ;
        sdio.write_1(1, F1_CHIPCLKCSR, want)?;
        let got = sdio.read_1(1, F1_CHIPCLKCSR)?;
        if got & !(CLK_ALP_AVAIL | CLK_HT_AVAIL) != want {
            return Err(Error::Card("chip clock request did not stick (CHIPCLKCSR)", got as u32));
        }
        let start = sdio.hc.p.now_us();
        loop {
            let csr = sdio.read_1(1, F1_CHIPCLKCSR)?;
            if csr & CLK_ALP_AVAIL != 0 {
                break;
            }
            if sdio.hc.p.now_us().wrapping_sub(start) > ALP_WAIT_US {
                return Err(Error::Card("ALP clock never came up (CHIPCLKCSR)", csr as u32));
            }
            sdio.hc.p.stall_us(100);
        }
        sdio.write_1(1, F1_CHIPCLKCSR, CLK_FORCE_HW_CLKREQ_OFF | CLK_FORCE_ALP)?;
        sdio.write_1(1, F1_SDIOPULLUP, 0)
    }

    /// Point the function 1 window at the 32 KiB containing `addr`.
    fn place_window(&mut self, addr: u32) -> Result<(), Error> {
        let base = addr & !SB_OFT_ADDR_MASK;
        if base == self.window {
            return Ok(());
        }
        self.window = u32::MAX; // unknown until all three bytes are written
        self.sdio.write_1(1, F1_SBADDR_LOW, (base >> 8) as u8 & 0x80)?;
        self.sdio.write_1(1, F1_SBADDR_MID, (base >> 16) as u8)?;
        self.sdio.write_1(1, F1_SBADDR_HIGH, (base >> 24) as u8)?;
        self.window = base;
        Ok(())
    }

    pub fn read_4(&mut self, addr: u32) -> Result<u32, Error> {
        self.place_window(addr)?;
        let mut b = [0u8; 4];
        self.sdio.read(1, (addr & SB_OFT_ADDR_MASK) | SB_ACCESS_2_4B, true, &mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    pub fn write_4(&mut self, addr: u32, v: u32) -> Result<(), Error> {
        self.place_window(addr)?;
        self.sdio.write(1, (addr & SB_OFT_ADDR_MASK) | SB_ACCESS_2_4B, true, &v.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Card, Sim};

    fn attached() -> Chip<Sim> {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        Chip::attach(s).map_err(|(_, e)| e).unwrap()
    }

    #[test]
    fn attach_reads_the_pi_5s_chip_id() {
        let chip = attached();
        // What the Pi 5's Linux reads at 0x18000000: BCM4345 rev 6, AXI.
        assert_eq!(chip.id.raw, 0x1526_4345);
        assert_eq!((chip.id.chip, chip.id.rev, chip.id.socitype), (0x4345, 6, 1));
        let card = chip.sdio.hc.p.card();
        assert_eq!(card.io_enable & 0b10, 0b10);
        assert_eq!(card.clkcsr, CLK_FORCE_HW_CLKREQ_OFF | CLK_FORCE_ALP | CLK_ALP_AVAIL);
        assert_eq!((card.fbr_block[1], card.fbr_block[2]), (64, 512));
    }

    #[test]
    fn the_window_moves_only_when_the_32k_changes() {
        let mut chip = attached();
        let before = chip.sdio.hc.p.card().window_writes;
        chip.write_4(0x1800_0004, 0xDEAD_BEEF).unwrap();
        assert_eq!(chip.read_4(0x1800_0004).unwrap(), 0xDEAD_BEEF);
        assert_eq!(chip.sdio.hc.p.card().window_writes, before);
        chip.write_4(0x0019_8000, 7).unwrap(); // RAM, another window
        assert_eq!(chip.sdio.hc.p.card().window_writes, before + 3);
        assert_eq!(chip.sdio.hc.p.card().window, 0x0019_8000);
        assert_eq!(chip.read_4(0x0019_8000).unwrap(), 7);
    }

    #[test]
    fn a_chip_whose_alp_clock_never_starts_fails_loudly() {
        let mut card = Card::cyw43455();
        card.alp_never = true;
        let mut s = Sdio::new(Sim::new(card));
        s.init().unwrap();
        match Chip::attach(s) {
            Err((_, Error::Card(what, _))) => assert!(what.contains("ALP")),
            _ => panic!("attach should fail without ALP"),
        }
    }
}
