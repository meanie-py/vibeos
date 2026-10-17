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
    pub window: u32,
    pub window_writes: u32,
    backplane: HashMap<u32, u32>,
    /// (address, block mode, bytes) of each function 2 CMD53 write.
    pub f2_writes: Vec<(u32, bool, usize)>,
    pub f2_fifo: Vec<u8>,
}

const CIS_AT: u32 = 0x1000;
const EROM: u32 = 0x1800_F000;

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
        cccr[0x09..0x0C].copy_from_slice(&CIS_AT.to_le_bytes()[..3]);
        let mut backplane = HashMap::new();
        backplane.insert(0x1800_0000, 0x1526_4345);
        backplane.insert(0x1800_00FC, EROM);
        for (i, w) in erom().iter().enumerate() {
            backplane.insert(EROM + 4 * i as u32, *w);
        }
        Self {
            present: true,
            alp_never: false,
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
            window: 0,
            window_writes: 0,
            backplane,
            f2_writes: Vec::new(),
            f2_fifo: Vec::new(),
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
            (false, 0x03) => self.io_enable, // ready as soon as enabled
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
                let up = self.clkcsr & 0x40;
                if write {
                    self.clkcsr = (data & 0x3F) | up;
                } else if self.clkcsr & 0x08 != 0 && !self.alp_never {
                    self.alp_polls += 1;
                    if self.alp_polls > 2 {
                        self.clkcsr |= 0x40;
                    }
                }
                self.clkcsr
            }
            _ => 0,
        }
    }

    fn bp_addr(&self, addr: u32) -> u32 {
        assert!(addr & 0x8000 != 0, "a backplane access without the 4-byte flag");
        self.window | (addr & 0x7FFF)
    }

    fn read53(&mut self, func: u8, addr: u32, len: usize) -> Vec<u8> {
        assert_eq!(func, 1, "only the backplane is readable here");
        assert_eq!(len % 4, 0);
        let a = self.bp_addr(addr);
        (0..len / 4).flat_map(|i| self.backplane.get(&(a + 4 * i as u32)).copied().unwrap_or(0).to_le_bytes()).collect()
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
        regs[REG_CAPS / 4] = 100 << 8; // 100 MHz base clock
        regs[REG_VERSION / 4] = 2 << 16; // SDHCI 3.00
        Self { st: RefCell::new(St { regs, card, xfer: None }), now: Cell::new(0) }
    }

    pub fn card(&self) -> Ref<'_, Card> {
        Ref::map(self.st.borrow(), |s| &s.card)
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
