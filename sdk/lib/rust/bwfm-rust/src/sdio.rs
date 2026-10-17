//! SDIO on an SDHCI host: bring an I/O card from power-on to a selected 4-bit bus,
//! then move bytes with CMD52 (one register) and CMD53 (a run of bytes or blocks).
//!
//! From the SDIO Simplified Specification (3.00): CMD5 IO_SEND_OP_COND answers R4,
//! CMD52 IO_RW_DIRECT and CMD53 IO_RW_EXTENDED answer R5, and function 0's CCCR and
//! FBRs describe the card. The CIS walk follows OpenBSD's `sdmmc_io.c` /
//! `sdmmc_cis.c` (ISC).

use sdhci_rust::{
    Error, Platform, Sdhci, CMD_DATA, RESP_NONE, RESP_R1, RESP_R1B, RESP_R3, XFER_BLOCK_COUNT, XFER_MULTI, XFER_READ,
};

pub const ID_CLOCK_HZ: u32 = 400_000;
/// Default speed. High speed (50 MHz, CCCR 0x13) comes once the chip runs.
pub const XFER_CLOCK_HZ: u32 = 25_000_000;

// CCCR (function 0).
pub const CCCR_IO_ENABLE: u32 = 0x02;
pub const CCCR_IO_READY: u32 = 0x03;
pub const CCCR_INT_ENABLE: u32 = 0x04;
pub const CCCR_IO_ABORT: u32 = 0x06;
pub const CCCR_BUS_IF: u32 = 0x07;
pub const CCCR_CIS_PTR: u32 = 0x09;
/// A function's FBR is at 0x100 * n: its CIS pointer at +9, block size at +0x10.
pub const FBR_CIS_PTR: u32 = 0x09;
pub const FBR_BLOCK_SIZE: u32 = 0x10;
const BUS_WIDTH_4: u8 = 0x02;

const CISTPL_NULL: u8 = 0x00;
const CISTPL_MANFID: u8 = 0x20;
const CISTPL_END: u8 = 0xFF;
/// A CIS is a few dozen bytes; stop well short of walking the whole card.
const CIS_MAX: u32 = 512;

/// R4: card ready, number of I/O functions, the OCR.
const R4_READY: u32 = 1 << 31;
/// CMD5's voltage window: 3.2-3.4 V, as ACMD41 asks of a memory card.
const OCR_VOLTAGE: u32 = 0x00FF_8000;
/// R5 flags [15:8]: COM_CRC_ERROR, ILLEGAL_COMMAND, ERROR, FUNCTION_NUMBER,
/// OUT_OF_RANGE. The rest is the card's state.
const R5_ERRORS: u32 = 0xCB00;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct CardInfo {
    pub rca: u16,
    /// I/O functions besides function 0.
    pub functions: u8,
    pub ocr: u32,
    /// CCCR 0x00: CCCR format [3:0], SDIO spec revision [7:4].
    pub cccr_rev: u8,
    /// From the common CIS's MANFID tuple.
    pub vendor: u16,
    pub device: u16,
    pub clock_hz: u32,
}

/// One CMD53: see `Sdio::plan`.
struct Op {
    addr: u32,
    block_mode: bool,
    bs: usize,
    at: usize,
    len: usize,
}

enum Data<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
}

pub struct Sdio<P: Platform> {
    pub hc: Sdhci<P>,
    pub card: CardInfo,
    /// Each function's block size, as last set (function 0 included).
    block: [u16; 8],
}

impl<P: Platform> Sdio<P> {
    pub fn new(p: P) -> Self {
        Self { hc: Sdhci::new(p), card: CardInfo::default(), block: [0; 8] }
    }

    /// Power the host, find the card, select it, read who it is, and go 4-bit.
    pub fn init(&mut self) -> Result<CardInfo, Error> {
        self.hc.power_up(ID_CLOCK_HZ)?;
        // CMD0 resets a memory card; an I/O card ignores it, harmlessly.
        self.hc.cmd(0, 0, RESP_NONE)?;
        let probe = self.hc.cmd(5, 0, RESP_R3)?;
        let ocr = probe & 0x00FF_FFFF;
        if ocr & OCR_VOLTAGE == 0 {
            return Err(Error::Card("no I/O function at 3.3 V (CMD5 OCR)", probe));
        }
        let start = self.hc.p.now_us();
        let r4 = loop {
            let r4 = self.hc.cmd(5, ocr & OCR_VOLTAGE, RESP_R3)?;
            if r4 & R4_READY != 0 {
                break r4;
            }
            if self.hc.p.now_us().wrapping_sub(start) > 1_000_000 {
                return Err(Error::Card("never left power-up (CMD5)", r4));
            }
            self.hc.p.stall_us(10_000);
        };
        self.card.ocr = r4 & 0x00FF_FFFF;
        self.card.functions = ((r4 >> 28) & 7) as u8;
        self.card.rca = (self.hc.cmd(3, 0, RESP_R1)? >> 16) as u16; // SEND_RELATIVE_ADDR (R6)
        self.hc.cmd(7, (self.card.rca as u32) << 16, RESP_R1B)?; // SELECT_CARD
        self.card.cccr_rev = self.read_1(0, 0)?;
        let (vendor, device) = self.manfid(self.cis_pointer(0)?)?;
        self.card.vendor = vendor;
        self.card.device = device;
        let bus = self.read_1(0, CCCR_BUS_IF)?;
        self.write_1(0, CCCR_BUS_IF, (bus & !3) | BUS_WIDTH_4)?;
        self.hc.set_4bit();
        self.card.clock_hz = self.hc.set_clock(XFER_CLOCK_HZ)?;
        Ok(self.card)
    }

    /// CMD52: one byte at `addr` (17 bits) in function `func`'s space.
    fn direct(&self, write: bool, func: u8, addr: u32, data: u8) -> Result<u8, Error> {
        let arg = (write as u32) << 31 | (func as u32 & 7) << 28 | (addr & 0x1_FFFF) << 9 | data as u32;
        let r5 = self.hc.cmd(52, arg, RESP_R1)?;
        if r5 & R5_ERRORS != 0 {
            return Err(Error::Card("CMD52 flags", r5));
        }
        Ok(r5 as u8)
    }

    pub fn read_1(&self, func: u8, addr: u32) -> Result<u8, Error> {
        self.direct(false, func, addr, 0)
    }

    pub fn write_1(&self, func: u8, addr: u32, v: u8) -> Result<(), Error> {
        self.direct(true, func, addr, v).map(|_| ())
    }

    pub fn set_block_size(&mut self, func: u8, size: u16) -> Result<(), Error> {
        let fbr = 0x100 * func as u32 + FBR_BLOCK_SIZE;
        self.write_1(0, fbr, size as u8)?;
        self.write_1(0, fbr + 1, (size >> 8) as u8)?;
        self.block[func as usize & 7] = size;
        Ok(())
    }

    /// Set function `func`'s I/O enable bit and wait for its I/O ready.
    pub fn enable_function(&self, func: u8) -> Result<(), Error> {
        let bit = 1u8 << func;
        let e = self.read_1(0, CCCR_IO_ENABLE)?;
        self.write_1(0, CCCR_IO_ENABLE, e | bit)?;
        let start = self.hc.p.now_us();
        loop {
            let r = self.read_1(0, CCCR_IO_READY)?;
            if r & bit != 0 {
                return Ok(());
            }
            if self.hc.p.now_us().wrapping_sub(start) > 3_000_000 {
                return Err(Error::Card("function never became ready (CCCR IOR)", r as u32));
            }
            self.hc.p.stall_us(1000);
        }
    }

    /// CMD53's commands for `len` bytes at `addr`: whole blocks in one block-mode
    /// command, the rest in byte mode. `incr` advances the card address with each
    /// byte (memory); a fixed address is a FIFO (function 2's frames). Yields
    /// (card address, block mode, block size, range of the buffer).
    fn plan(&self, func: u8, addr: u32, incr: bool, len: usize) -> impl Iterator<Item = Op> {
        // A function whose block size was never set gets byte mode only.
        let bs = self.block[func as usize & 7] as usize;
        let whole = if bs > 0 { len / bs * bs } else { 0 };
        let blocks = (whole > 0).then_some(Op { addr, block_mode: true, bs, at: 0, len: whole });
        let tail = if bs > 0 { bs.min(512) } else { 512 };
        let bytes = (whole..len).step_by(tail).map(move |at| Op {
            addr: if incr { addr + at as u32 } else { addr },
            block_mode: false,
            bs: tail.min(len - at),
            at,
            len: tail.min(len - at),
        });
        blocks.into_iter().chain(bytes)
    }

    fn one_extended(&self, func: u8, incr: bool, op: &Op, mut data: Data) -> Result<(), Error> {
        let write = matches!(data, Data::Write(_));
        let blocks = op.len / op.bs;
        // Byte mode counts bytes, block mode blocks; 0 would mean 512 / forever.
        let count = if op.block_mode { blocks } else { op.len } as u32;
        let arg = (write as u32) << 31
            | (func as u32 & 7) << 28
            | (op.block_mode as u32) << 27
            | (incr as u32) << 26
            | (op.addr & 0x1_FFFF) << 9
            | (count & 0x1FF);
        let mut mode = if write { 0 } else { XFER_READ };
        if op.block_mode && blocks > 1 {
            mode |= XFER_MULTI | XFER_BLOCK_COUNT;
        }
        let n = if op.block_mode { blocks as u16 } else { 1 };
        let r = self.hc.issue(53, arg, RESP_R1 | CMD_DATA, mode, op.bs as u16, n)?;
        if r[0] & R5_ERRORS != 0 {
            self.abort(func);
            return Err(Error::Card("CMD53 flags", r[0]));
        }
        for i in 0..n as usize {
            let at = i * op.bs..(i + 1) * op.bs;
            let moved = match &mut data {
                Data::Read(b) => self.hc.read_data(&mut b[at]),
                Data::Write(b) => self.hc.write_data(&b[at]),
            };
            if let Err(s) = moved {
                self.abort(func);
                return Err(Error::Command(53, s));
            }
        }
        self.hc.finish(1_000_000).map_err(|s| {
            self.abort(func);
            Error::Command(53, s)
        })
    }

    /// After a failed CMD53: reset the host's lines and stop the function's side.
    fn abort(&self, func: u8) {
        self.hc.recover();
        let _ = self.write_1(0, CCCR_IO_ABORT, func & 7);
    }

    pub fn read(&self, func: u8, addr: u32, incr: bool, buf: &mut [u8]) -> Result<(), Error> {
        for op in self.plan(func, addr, incr, buf.len()) {
            self.one_extended(func, incr, &op, Data::Read(&mut buf[op.at..op.at + op.len]))?;
        }
        Ok(())
    }

    pub fn write(&self, func: u8, addr: u32, incr: bool, buf: &[u8]) -> Result<(), Error> {
        for op in self.plan(func, addr, incr, buf.len()) {
            self.one_extended(func, incr, &op, Data::Write(&buf[op.at..op.at + op.len]))?;
        }
        Ok(())
    }

    /// Function `func`'s CIS pointer (function 0: the common CIS).
    pub fn cis_pointer(&self, func: u8) -> Result<u32, Error> {
        let at = if func == 0 { CCCR_CIS_PTR } else { 0x100 * func as u32 + FBR_CIS_PTR };
        Ok(self.read_1(0, at)? as u32 | (self.read_1(0, at + 1)? as u32) << 8 | (self.read_1(0, at + 2)? as u32) << 16)
    }

    /// Walk a CIS for its MANFID tuple: (vendor, device).
    pub fn manfid(&self, cis: u32) -> Result<(u16, u16), Error> {
        let mut at = cis;
        while at < cis + CIS_MAX {
            let code = self.read_1(0, at)?;
            match code {
                CISTPL_END => break,
                CISTPL_NULL => {
                    at += 1;
                    continue;
                }
                _ => {}
            }
            let link = self.read_1(0, at + 1)? as u32;
            if code == CISTPL_MANFID && link >= 4 {
                let b = |i| self.read_1(0, at + 2 + i);
                return Ok((u16::from_le_bytes([b(0)?, b(1)?]), u16::from_le_bytes([b(2)?, b(3)?])));
            }
            if link == 0xFF {
                break;
            }
            at += 2 + link;
        }
        Err(Error::Card("CIS has no MANFID tuple", cis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Card, Sim};

    #[test]
    fn init_selects_the_card_and_reads_who_it_is() {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        let c = s.init().unwrap();
        assert_eq!((c.vendor, c.device), (0x02D0, 0x4345));
        assert_eq!(c.rca, 0x0001);
        assert_eq!(c.functions, 2);
        assert_eq!(c.clock_hz, 25_000_000);
        {
            let card = s.hc.p.card();
            assert!(card.selected && card.cccr[CCCR_BUS_IF as usize] & 3 == BUS_WIDTH_4);
        }
        assert!(s.hc.rd(sdhci_rust::REG_HOST_CTRL) & sdhci_rust::HOST_4BIT != 0);
    }

    #[test]
    fn no_card_is_a_timeout_not_a_hang() {
        let mut s = Sdio::new(Sim::new(Card::absent()));
        assert!(matches!(s.init(), Err(Error::Command(5, _))));
    }

    #[test]
    fn cmd53_splits_blocks_from_the_tail_and_fixed_addresses_stay_put() {
        let mut s = Sdio::new(Sim::new(Card::cyw43455()));
        s.init().unwrap();
        s.set_block_size(2, 64).unwrap();
        let frame: Vec<u8> = (0..150u8).collect();
        s.write(2, 0x8000, false, &frame).unwrap();
        let card = s.hc.p.card();
        // 2 blocks of 64 in one block-mode command, then 22 bytes in byte mode,
        // all at the FIFO's one address.
        assert_eq!(card.f2_writes, vec![(0x8000, true, 128), (0x8000, false, 22)]);
        assert_eq!(card.f2_fifo, frame);
    }
}
