//! The SD card protocol, on the shared SDHCI controller layer (`sdhci-rust`, which
//! holds the register rules: dword access only, PIO, interrupt-paced waits).
//!
//! Written for the Raspberry Pi 4's EMMC2 (BCM2711, `ACPI\BRCME88C`), measured
//! from the UEFI Shell before the first line of this file: SDHCI version 3.00,
//! a 100 MHz base clock in the capabilities register, the card at 3.3 V
//! signalling, and the firmware running it at 25 MHz on a 1-bit bus.
//!
//! - **No card-detect line.** The slot's CD pin is not wired to EMMC2; the
//!   firmware forces "card inserted" through the card-detect test bits, and
//!   SDHCI refuses bus power when it believes the slot is empty. So the reset
//!   path forces it again, and "no card" shows up as a command timeout instead.
//! - **Timeouts** come from the ARM generic timer, because KUSER_SHARED_DATA's
//!   interrupt time does not advance on this port.

use ntkrnl_sys::ddk::{KeQueryPerformanceCounter, KeStallExecutionProcessor, LARGE_INTEGER};
use sdhci_rust::{
    Platform, Sdhci, CMD_DATA, INT_CMD_TIMEOUT, RESP_NONE, RESP_R1, RESP_R1B, RESP_R2, RESP_R3, XFER_AUTO_CMD12,
    XFER_BLOCK_COUNT, XFER_MULTI, XFER_READ,
};
pub use sdhci_rust::Error;

use crate::print::sdprint;

pub const SECTOR: usize = 512;

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

/// The mapped register window, the ARM generic timer and irq.rs.
pub struct Kernel {
    regs: *mut u32,
}

impl Platform for Kernel {
    fn rd(&self, off: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.regs.add(off / 4)) }
    }
    fn wr(&self, off: usize, v: u32) {
        unsafe { core::ptr::write_volatile(self.regs.add(off / 4), v) }
    }
    fn now_us(&self) -> u64 {
        micros()
    }
    fn stall_us(&self, us: u32) {
        unsafe { KeStallExecutionProcessor(us) }
    }
    fn irq_connected(&self) -> bool {
        crate::irq::connected()
    }
    fn irq_wait(&self, us: u64) -> bool {
        crate::irq::wait(us)
    }
}

pub struct Host {
    hc: Sdhci<Kernel>,
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
        Self { hc: Sdhci::new(Kernel { regs: regs.cast() }), card: Card::default() }
    }

    fn issue(&self, index: u8, arg: u32, flags: u16, mode: u16, blocks: u16) -> Result<[u32; 4], Error> {
        self.hc.issue(index, arg, flags, mode, SECTOR as u16, blocks)
    }

    fn cmd(&self, index: u8, arg: u32, flags: u16) -> Result<u32, Error> {
        self.hc.cmd(index, arg, flags)
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
        let hc = &self.hc;
        let (version, caps) = (hc.version(), hc.rd(sdhci_rust::REG_CAPS));
        sdprint!(
            "host: SDHCI spec {} (version {:#x}), caps {:#010x}, base clock {} MHz, firmware left clock {:#06x} host {:#010x}\n",
            match version {
                0 => "1.00",
                1 => "2.00",
                2 => "3.00",
                _ => "4.x",
            },
            hc.rd(sdhci_rust::REG_VERSION) >> 16,
            caps,
            ((caps >> 8) & 0xFF),
            hc.rd(sdhci_rust::REG_CLOCK) & 0xFFFF,
            hc.rd(sdhci_rust::REG_HOST_CTRL)
        );
        self.hc.power_up(ID_CLOCK_HZ)?;

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
        self.hc.set_4bit();
        if !self.card.high_capacity {
            self.cmd(16, SECTOR as u32, RESP_R1)?; // SET_BLOCKLEN
        }
        self.card.clock_hz = self.hc.set_clock(XFER_CLOCK_HZ)?;
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
        self.hc.recover();
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
            let dst = core::slice::from_raw_parts_mut(buf.add(b * SECTOR), SECTOR);
            self.hc.read_data(dst).map_err(|v| self.abort(index, v))?;
        }
        self.hc.finish(1_000_000).map_err(|v| self.abort(index, v))
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
            let src = core::slice::from_raw_parts(buf.add(b * SECTOR), SECTOR);
            self.hc.write_data(src).map_err(|v| self.abort(index, v))?;
        }
        // For a write, transfer complete waits out the card's busy (programming).
        self.hc.finish(2_500_000).map_err(|v| self.abort(index, v))?;
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
