//! The SD host controller (SDHCI 3.0) and the SD card protocol on top of it.
//!
//! Written for the Raspberry Pi 4's EMMC2 (BCM2711, `ACPI\BRCME88C`), measured
//! from the UEFI Shell before the first line of this file: SDHCI version 3.00,
//! a 100 MHz base clock in the capabilities register, the card at 3.3 V
//! signalling, and the firmware running it at 25 MHz on a 1-bit bus.
//!
//! - **32-bit register access only.** The iProc wrapper around this SDHCI core
//!   drops or corrupts 8- and 16-bit accesses (Linux's sdhci-iproc shadows
//!   every narrow write for the same reason), so every register here is read
//!   and written as a whole dword, and the 16-bit transfer-mode and command
//!   registers go out together as the one dword at 0x0C. That write is also
//!   what issues the command.
//! - **No card-detect line.** The slot's CD pin is not wired to EMMC2; the
//!   firmware forces "card inserted" through the card-detect test bits, and
//!   SDHCI refuses bus power when it believes the slot is empty. So the reset
//!   path forces it again, and "no card" shows up as a command timeout instead.
//! - **Polled PIO.** Data moves through the buffer data port, so there is no
//!   DMA, no cache maintenance and no interrupt; timeouts come from the ARM
//!   generic timer, because KUSER_SHARED_DATA's interrupt time does not advance
//!   on this port.

use core::ptr::{read_unaligned, read_volatile, write_unaligned, write_volatile};

use ntkrnl_sys::ddk::{KeQueryPerformanceCounter, KeStallExecutionProcessor, LARGE_INTEGER};

use crate::print::sdprint;

// Registers (byte offsets). All dword-wide; see the module comment.
const REG_BLOCK: usize = 0x04; // block size [11:0] | block count [31:16]
const REG_ARG: usize = 0x08;
const REG_XFER_CMD: usize = 0x0C; // transfer mode [15:0] | command [31:16]
const REG_RESP: usize = 0x10; // four dwords
const REG_BUFFER: usize = 0x20;
const REG_PRESENT: usize = 0x24;
const REG_HOST_CTRL: usize = 0x28; // host control 1 | power [15:8] | block gap | wakeup
const REG_CLOCK: usize = 0x2C; // clock [15:0] | timeout [19:16] | software reset [26:24]
const REG_INT_STATUS: usize = 0x30; // normal [15:0] | error [31:16], write 1 to clear
const REG_INT_ENABLE: usize = 0x34;
const REG_INT_SIGNAL: usize = 0x38;
const REG_HOST_CTRL2: usize = 0x3C; // auto CMD error [15:0] | host control 2 [31:16]
const REG_CAPS: usize = 0x40;
const REG_VERSION: usize = 0xFC; // slot interrupt [15:0] | host version [31:16]

const PRESENT_CMD_INHIBIT: u32 = 1 << 0;
const PRESENT_DAT_INHIBIT: u32 = 1 << 1;

const HOST_4BIT: u32 = 1 << 1;
/// Card-detect test level + "use the test level": the slot reads as occupied.
const HOST_CD_FORCED: u32 = 0xC0;
const POWER_330: u32 = 0x0E << 8;
const POWER_ON: u32 = 0x01 << 8;

const CLOCK_INTERNAL_ENABLE: u32 = 1 << 0;
const CLOCK_STABLE: u32 = 1 << 1;
const CLOCK_CARD_ENABLE: u32 = 1 << 2;
const CLOCK_TIMEOUT_MASK: u32 = 0xF << 16;
/// Data timeout counter: TMCLK x 2^27, the largest SDHCI defines.
const CLOCK_TIMEOUT_MAX: u32 = 0xE << 16;
const RESET_ALL: u32 = 1 << 24;
const RESET_CMD: u32 = 1 << 25;
const RESET_DAT: u32 = 1 << 26;

const INT_CMD_DONE: u32 = 1 << 0;
const INT_XFER_DONE: u32 = 1 << 1;
const INT_WRITE_READY: u32 = 1 << 4;
const INT_READ_READY: u32 = 1 << 5;
const INT_ERROR: u32 = 1 << 15;
const INT_ERRORS: u32 = 0x03FF_0000; // command and data errors, auto CMD12, ADMA
const INT_CMD_TIMEOUT: u32 = 1 << 16;
/// The status bits to latch: the normal ones we poll for, and every error.
const INT_LATCH: u32 = INT_ERRORS | 0x3F;

// Command register flags: response type, CRC and index checks, data present.
const RESP_NONE: u16 = 0x00;
const RESP_R1: u16 = 0x1A; // also R6, R7
const RESP_R1B: u16 = 0x1B;
const RESP_R2: u16 = 0x09;
const RESP_R3: u16 = 0x02;
const CMD_DATA: u16 = 0x20;

// Transfer mode.
const XFER_BLOCK_COUNT: u16 = 1 << 1;
const XFER_AUTO_CMD12: u16 = 1 << 2;
const XFER_READ: u16 = 1 << 4;
const XFER_MULTI: u16 = 1 << 5;

pub const SECTOR: usize = 512;
const SECTOR_DWORDS: usize = SECTOR / 4;

const ID_CLOCK_HZ: u32 = 400_000;
const XFER_CLOCK_HZ: u32 = 25_000_000;
/// SEND_IF_COND argument: 2.7-3.6 V, check pattern 0xAA.
const IF_COND_ARG: u32 = 0x1AA;
/// ACMD41: the 3.2-3.4 V window, plus HCS for a card that answered CMD8.
const OCR_VOLTAGE: u32 = 0x00FF_8000;
const OCR_HCS: u32 = 1 << 30;
const OCR_BUSY_DONE: u32 = 1 << 31;
/// CMD13 status: READY_FOR_DATA, and CURRENT_STATE = tran.
const STATUS_READY: u32 = 1 << 8;
const STATE_TRAN: u32 = 4;

// The fields reach the log through Debug, which dead-code analysis ignores.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The controller never reached a state (what we waited for, INT_STATUS).
    Timeout(&'static str, u32),
    /// The card or the bus reported an error on a command (index, INT_STATUS).
    Command(u8, u32),
    /// The card's answers make no sense (what was wrong, the value).
    Card(&'static str, u32),
}

/// What the card told us about itself.
#[derive(Clone, Copy, Default)]
pub struct Card {
    pub rca: u32,
    /// SDHC/SDXC: block-addressed. Otherwise SDSC, byte-addressed.
    pub high_capacity: bool,
    pub blocks: u64,
    pub manufacturer: u8,
    pub name: [u8; 5],
    pub revision: u8,
    pub serial: u32,
    pub clock_hz: u32,
}

pub struct Host {
    regs: *mut u32,
    base_hz: u32,
    pub card: Card,
}

/// Microseconds on the ARM generic timer.
fn micros() -> u64 {
    let mut freq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let count = unsafe { KeQueryPerformanceCounter(&mut freq) };
    let f = unsafe { freq.QuadPart }.max(1) as u128;
    (unsafe { count.QuadPart } as u128 * 1_000_000 / f) as u64
}

fn stall_us(us: u32) {
    unsafe { KeStallExecutionProcessor(us) }
}

impl Host {
    /// # Safety
    /// `regs` must map the controller's register window, uncached.
    pub unsafe fn new(regs: *mut u8) -> Self {
        Self { regs: regs.cast(), base_hz: 0, card: Card::default() }
    }

    fn rd(&self, off: usize) -> u32 {
        unsafe { read_volatile(self.regs.add(off / 4)) }
    }
    fn wr(&self, off: usize, v: u32) {
        unsafe { write_volatile(self.regs.add(off / 4), v) }
    }

    /// Spin until `done(INT_STATUS or PRESENT_STATE)` or `us` microseconds pass.
    fn spin_until(&self, reg: usize, us: u64, done: impl Fn(u32) -> bool) -> Result<u32, u32> {
        let start = micros();
        loop {
            let v = self.rd(reg);
            if done(v) {
                return Ok(v);
            }
            if micros().wrapping_sub(start) > us {
                let v = self.rd(reg);
                return if done(v) { Ok(v) } else { Err(v) };
            }
            stall_us(1);
        }
    }

    fn reset(&self, what: u32) -> Result<(), Error> {
        // The other bits of this dword are the clock and the timeout: keep them.
        self.wr(REG_CLOCK, (self.rd(REG_CLOCK) & !(RESET_ALL | RESET_CMD | RESET_DAT)) | what);
        self.spin_until(REG_CLOCK, 100_000, |v| v & what == 0).map_err(|v| Error::Timeout("software reset", v))?;
        Ok(())
    }

    /// After an error: reset the command and data lines and clear the status.
    fn recover(&self) {
        let _ = self.reset(RESET_CMD | RESET_DAT);
        self.wr(REG_INT_STATUS, 0xFFFF_FFFF);
    }

    fn set_clock(&mut self, hz: u32) -> Result<u32, Error> {
        // SDHCI 3.0 10-bit divided clock: base / (2 * N), or base itself for N = 0.
        let n = if hz >= self.base_hz { 0 } else { self.base_hz.div_ceil(2 * hz).min(0x3FF) };
        let actual = if n == 0 { self.base_hz } else { self.base_hz / (2 * n) };
        let keep = self.rd(REG_CLOCK) & CLOCK_TIMEOUT_MASK;
        self.wr(REG_CLOCK, keep); // card clock off before changing it
        let clk = keep | ((n & 0xFF) << 8) | (((n >> 8) & 3) << 6) | CLOCK_INTERNAL_ENABLE;
        self.wr(REG_CLOCK, clk);
        self.spin_until(REG_CLOCK, 150_000, |v| v & CLOCK_STABLE != 0).map_err(|v| Error::Timeout("clock stable", v))?;
        self.wr(REG_CLOCK, clk | CLOCK_CARD_ENABLE);
        stall_us(1000);
        Ok(actual)
    }

    /// Issue a command; for a data command, the caller then moves the data.
    fn issue(&self, index: u8, arg: u32, flags: u16, mode: u16, blocks: u16) -> Result<[u32; 4], Error> {
        let data = flags & CMD_DATA != 0;
        let busy = data || flags == RESP_R1B;
        let inhibit = PRESENT_CMD_INHIBIT | if busy { PRESENT_DAT_INHIBIT } else { 0 };
        if let Err(v) = self.spin_until(REG_PRESENT, 1_000_000, |v| v & inhibit == 0) {
            self.recover();
            return Err(Error::Timeout("command line free", v));
        }
        self.wr(REG_INT_STATUS, 0xFFFF_FFFF);
        if data {
            self.wr(REG_BLOCK, SECTOR as u32 | (blocks as u32) << 16);
        }
        self.wr(REG_ARG, arg);
        self.wr(REG_XFER_CMD, (index as u32) << 24 | (flags as u32) << 16 | mode as u32);
        let status = match self.spin_until(REG_INT_STATUS, 100_000, |v| v & (INT_CMD_DONE | INT_ERROR) != 0) {
            Ok(v) => v,
            Err(v) => {
                self.recover();
                return Err(Error::Timeout("command complete", v));
            }
        };
        if status & INT_ERROR != 0 {
            self.recover();
            return Err(Error::Command(index, status));
        }
        self.wr(REG_INT_STATUS, INT_CMD_DONE);
        let r = [self.rd(REG_RESP), self.rd(REG_RESP + 4), self.rd(REG_RESP + 8), self.rd(REG_RESP + 12)];
        if flags == RESP_R1B {
            // The card holds DAT0 low while busy; the controller reports the end.
            match self.spin_until(REG_INT_STATUS, 2_500_000, |v| v & (INT_XFER_DONE | INT_ERROR) != 0) {
                Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_XFER_DONE),
                Ok(v) | Err(v) => {
                    self.recover();
                    return Err(Error::Command(index, v));
                }
            }
        }
        Ok(r)
    }

    fn cmd(&self, index: u8, arg: u32, flags: u16) -> Result<u32, Error> {
        self.issue(index, arg, flags, 0, 0).map(|r| r[0])
    }

    fn app_cmd(&self, index: u8, arg: u32, flags: u16) -> Result<u32, Error> {
        self.cmd(55, self.card.rca << 16, RESP_R1)?;
        self.cmd(index, arg, flags)
    }

    /// A 136-bit response as the card sent it: SDHCI drops the CRC byte and
    /// right-aligns bits [127:8] in the four response registers.
    fn long_response(r: [u32; 4]) -> u128 {
        ((r[3] as u128) << 96 | (r[2] as u128) << 64 | (r[1] as u128) << 32 | r[0] as u128) << 8
    }

    /// Reset the controller and bring a card from idle to the transfer state.
    pub fn init(&mut self) -> Result<(), Error> {
        let version = (self.rd(REG_VERSION) >> 16) & 0xFF;
        let caps = self.rd(REG_CAPS);
        self.base_hz = ((caps >> 8) & 0xFF) * 1_000_000;
        sdprint!(
            "host: SDHCI spec {} (version {:#x}), caps {:#010x}, base clock {} MHz, firmware left clock {:#06x} host {:#010x}\n",
            match version {
                0 => "1.00",
                1 => "2.00",
                2 => "3.00",
                _ => "4.x",
            },
            self.rd(REG_VERSION) >> 16,
            caps,
            self.base_hz / 1_000_000,
            self.rd(REG_CLOCK) & 0xFFFF,
            self.rd(REG_HOST_CTRL)
        );
        if self.base_hz == 0 {
            return Err(Error::Card("capabilities register has no base clock", caps));
        }
        self.reset(RESET_ALL)?;
        self.wr(REG_HOST_CTRL, HOST_CD_FORCED);
        self.wr(REG_HOST_CTRL, HOST_CD_FORCED | POWER_330);
        self.wr(REG_HOST_CTRL, HOST_CD_FORCED | POWER_330 | POWER_ON);
        self.wr(REG_HOST_CTRL2, 0); // 3.3 V signalling, no UHS
        self.wr(REG_INT_ENABLE, INT_LATCH);
        self.wr(REG_INT_SIGNAL, 0);
        self.wr(REG_CLOCK, (self.rd(REG_CLOCK) & !CLOCK_TIMEOUT_MASK) | CLOCK_TIMEOUT_MAX);
        self.set_clock(ID_CLOCK_HZ)?;

        self.cmd(0, 0, RESP_NONE)?; // GO_IDLE_STATE
        let v2 = match self.cmd(8, IF_COND_ARG, RESP_R1) {
            Ok(r) if r & 0xFFF == IF_COND_ARG => true,
            Ok(r) => return Err(Error::Card("SEND_IF_COND echoed the wrong pattern", r)),
            // A version 1 card does not know CMD8.
            Err(Error::Command(_, s)) if s & INT_CMD_TIMEOUT != 0 => false,
            Err(e) => return Err(e),
        };
        let start = micros();
        let ocr = loop {
            let ocr = self.app_cmd(41, OCR_VOLTAGE | if v2 { OCR_HCS } else { 0 }, RESP_R3)?;
            if ocr & OCR_BUSY_DONE != 0 {
                break ocr;
            }
            if micros().wrapping_sub(start) > 1_000_000 {
                return Err(Error::Card("never left power-up (ACMD41)", ocr));
            }
            stall_us(10_000);
        };
        self.card.high_capacity = ocr & OCR_HCS != 0;

        let cid = Self::long_response(self.issue(2, 0, RESP_R2, 0, 0)?); // ALL_SEND_CID
        self.card.rca = self.cmd(3, 0, RESP_R1)? >> 16; // SEND_RELATIVE_ADDR (R6)
        let csd = Self::long_response(self.issue(9, self.card.rca << 16, RESP_R2, 0, 0)?);
        self.card.blocks = match csd >> 126 {
            // CSD 2.0: C_SIZE [69:48] in 512 KiB units.
            1 => (((csd >> 48) & 0x3F_FFFF) as u64 + 1) * 1024,
            // CSD 1.0: (C_SIZE + 1) << (C_SIZE_MULT + 2) blocks of 2^READ_BL_LEN.
            0 => {
                let c_size = ((csd >> 62) & 0xFFF) as u64;
                let mult = ((csd >> 47) & 7) as u32;
                let bl_len = ((csd >> 80) & 0xF) as u32;
                ((c_size + 1) << (mult + 2) << bl_len) / SECTOR as u64
            }
            s => return Err(Error::Card("unsupported CSD structure (SDUC?)", s as u32)),
        };
        self.card.manufacturer = (cid >> 120) as u8;
        for (i, c) in self.card.name.iter_mut().enumerate() {
            *c = (cid >> (96 - 8 * i)) as u8;
        }
        self.card.revision = (cid >> 56) as u8;
        self.card.serial = (cid >> 24) as u32;

        self.cmd(7, self.card.rca << 16, RESP_R1B)?; // SELECT_CARD
        self.app_cmd(6, 2, RESP_R1)?; // SET_BUS_WIDTH: 4 bits
        self.wr(REG_HOST_CTRL, self.rd(REG_HOST_CTRL) | HOST_4BIT);
        if !self.card.high_capacity {
            self.cmd(16, SECTOR as u32, RESP_R1)?; // SET_BLOCKLEN
        }
        self.card.clock_hz = self.set_clock(XFER_CLOCK_HZ)?;
        self.wait_ready()?;
        Ok(())
    }

    /// Poll SEND_STATUS until the card is back in the transfer state.
    fn wait_ready(&self) -> Result<(), Error> {
        let start = micros();
        loop {
            let s = self.cmd(13, self.card.rca << 16, RESP_R1)?;
            if s & STATUS_READY != 0 && (s >> 9) & 0xF == STATE_TRAN {
                return Ok(());
            }
            if micros().wrapping_sub(start) > 1_000_000 {
                return Err(Error::Card("never returned to the transfer state", s));
            }
            stall_us(100);
        }
    }

    fn address(&self, lba: u64) -> u32 {
        if self.card.high_capacity { lba as u32 } else { (lba * SECTOR as u64) as u32 }
    }

    /// A data error mid-transfer: reset the lines, then stop the card's side.
    fn abort(&self, index: u8, status: u32) -> Error {
        self.recover();
        let _ = self.cmd(12, 0, RESP_R1B); // STOP_TRANSMISSION; it may not be in a data state
        Error::Command(index, status)
    }

    /// Read `count` (1..=65535) sectors at `lba` into `buf`.
    ///
    /// # Safety
    /// `buf` must be valid for `count * 512` bytes of writes.
    pub unsafe fn read(&self, lba: u64, count: u16, buf: *mut u8) -> Result<(), Error> {
        let (index, mode) = if count == 1 {
            (17, XFER_READ)
        } else {
            (18, XFER_READ | XFER_MULTI | XFER_BLOCK_COUNT | XFER_AUTO_CMD12)
        };
        self.issue(index, self.address(lba), RESP_R1 | CMD_DATA, mode, count)?;
        for b in 0..count as usize {
            let s = self.spin_until(REG_INT_STATUS, 1_000_000, |v| v & (INT_READ_READY | INT_ERROR) != 0);
            match s {
                Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_READ_READY),
                Ok(v) | Err(v) => return Err(self.abort(index, v)),
            }
            let dst = buf.add(b * SECTOR) as *mut u32;
            for i in 0..SECTOR_DWORDS {
                write_unaligned(dst.add(i), self.rd(REG_BUFFER));
            }
        }
        match self.spin_until(REG_INT_STATUS, 1_000_000, |v| v & (INT_XFER_DONE | INT_ERROR) != 0) {
            Ok(v) if v & INT_ERROR == 0 => {
                self.wr(REG_INT_STATUS, INT_XFER_DONE);
                Ok(())
            }
            Ok(v) | Err(v) => Err(self.abort(index, v)),
        }
    }

    /// Write `count` (1..=65535) sectors from `buf` at `lba`.
    ///
    /// # Safety
    /// `buf` must be valid for `count * 512` bytes of reads.
    pub unsafe fn write(&self, lba: u64, count: u16, buf: *const u8) -> Result<(), Error> {
        let (index, mode) =
            if count == 1 { (24, 0) } else { (25, XFER_MULTI | XFER_BLOCK_COUNT | XFER_AUTO_CMD12) };
        self.issue(index, self.address(lba), RESP_R1 | CMD_DATA, mode, count)?;
        for b in 0..count as usize {
            let s = self.spin_until(REG_INT_STATUS, 1_000_000, |v| v & (INT_WRITE_READY | INT_ERROR) != 0);
            match s {
                Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_WRITE_READY),
                Ok(v) | Err(v) => return Err(self.abort(index, v)),
            }
            let src = buf.add(b * SECTOR) as *const u32;
            for i in 0..SECTOR_DWORDS {
                self.wr(REG_BUFFER, read_unaligned(src.add(i)));
            }
        }
        // For a write, transfer complete waits out the card's busy (programming).
        match self.spin_until(REG_INT_STATUS, 2_500_000, |v| v & (INT_XFER_DONE | INT_ERROR) != 0) {
            Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_XFER_DONE),
            Ok(v) | Err(v) => return Err(self.abort(index, v)),
        }
        self.wait_ready()
    }
}

/// Microseconds since `start`, for the bring-up throughput lines.
pub fn elapsed_us(start: u64) -> u64 {
    micros().wrapping_sub(start)
}

pub fn now_us() -> u64 {
    micros()
}
