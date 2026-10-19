//! An SD host controller (SDHCI 3.0), below any card protocol: reset, power, the
//! card clock, one command, and PIO data through the buffer port.
//!
//! Two drivers sit on it: sdhc.sys (an SD memory card, `sdhc-rust`) and bwfm.sys
//! (the Pis' SDIO Wi-Fi chip, `bwfm-rust`). Nothing here touches the kernel: the
//! registers, the clock and the interrupt come through [`Platform`], so the same
//! code runs against a simulated controller on the host.
//!
//! - **32-bit register access only.** The Pi 4's iProc wrapper around this core
//!   drops or corrupts 8- and 16-bit accesses (Linux's sdhci-iproc shadows every
//!   narrow write for the same reason), so every register is read and written as
//!   a whole dword, and the 16-bit transfer-mode and command registers go out
//!   together as the one dword at 0x0C. That write is also what issues the
//!   command. A dword is always safe on a standard SDHCI too.
//! - **PIO, interrupt-paced.** Data moves through the buffer data port: no DMA,
//!   no cache maintenance. A wait longer than a few microseconds sleeps on the
//!   controller's interrupt when the platform has one connected
//!   ([`Platform::irq_wait`]); otherwise it spins.

#![cfg_attr(not(test), no_std)]

// Registers (byte offsets). All dword-wide; see the module comment.
pub const REG_BLOCK: usize = 0x04; // block size [11:0] | block count [31:16]
pub const REG_ARG: usize = 0x08;
pub const REG_XFER_CMD: usize = 0x0C; // transfer mode [15:0] | command [31:16]
pub const REG_RESP: usize = 0x10; // four dwords
pub const REG_BUFFER: usize = 0x20;
pub const REG_PRESENT: usize = 0x24;
pub const REG_HOST_CTRL: usize = 0x28; // host control 1 | power [15:8] | block gap | wakeup
pub const REG_CLOCK: usize = 0x2C; // clock [15:0] | timeout [19:16] | software reset [26:24]
pub const REG_INT_STATUS: usize = 0x30; // normal [15:0] | error [31:16], write 1 to clear
pub const REG_INT_ENABLE: usize = 0x34;
pub const REG_INT_SIGNAL: usize = 0x38;
pub const REG_HOST_CTRL2: usize = 0x3C; // auto CMD error [15:0] | host control 2 [31:16]
pub const REG_CAPS: usize = 0x40;
pub const REG_VERSION: usize = 0xFC; // slot interrupt [15:0] | host version [31:16]

pub const PRESENT_CMD_INHIBIT: u32 = 1 << 0;
pub const PRESENT_DAT_INHIBIT: u32 = 1 << 1;

pub const HOST_4BIT: u32 = 1 << 1;
/// High-speed timing: outputs on the rising clock edge, for 26-50 MHz.
pub const HOST_HIGH_SPEED: u32 = 1 << 2;
/// Capabilities: the host can run a high-speed bus.
pub const CAPS_HIGH_SPEED: u32 = 1 << 21;
/// Card-detect test level + "use the test level": the slot reads as occupied.
pub const HOST_CD_FORCED: u32 = 0xC0;
pub const POWER_330: u32 = 0x0E << 8;
pub const POWER_ON: u32 = 0x01 << 8;

pub const CLOCK_INTERNAL_ENABLE: u32 = 1 << 0;
pub const CLOCK_STABLE: u32 = 1 << 1;
pub const CLOCK_CARD_ENABLE: u32 = 1 << 2;
pub const CLOCK_TIMEOUT_MASK: u32 = 0xF << 16;
/// Data timeout counter: TMCLK x 2^27, the largest SDHCI defines.
pub const CLOCK_TIMEOUT_MAX: u32 = 0xE << 16;
pub const RESET_ALL: u32 = 1 << 24;
pub const RESET_CMD: u32 = 1 << 25;
pub const RESET_DAT: u32 = 1 << 26;

pub const INT_CMD_DONE: u32 = 1 << 0;
pub const INT_XFER_DONE: u32 = 1 << 1;
pub const INT_WRITE_READY: u32 = 1 << 4;
pub const INT_READ_READY: u32 = 1 << 5;
/// An SDIO card's interrupt (DAT1 in 4-bit mode). Not latched: it reads set for
/// as long as the card holds the line, and clears when the card's source does.
pub const INT_CARD: u32 = 1 << 8;
pub const INT_ERROR: u32 = 1 << 15;
pub const INT_ERRORS: u32 = 0x03FF_0000; // command and data errors, auto CMD12, ADMA
pub const INT_CMD_TIMEOUT: u32 = 1 << 16;
/// The status bits to latch: the normal ones we poll for, and every error.
pub const INT_LATCH: u32 = INT_ERRORS | 0x3F;
/// Spin this long before sleeping on the interrupt: a command at 25 MHz
/// completes well inside it, and sleeping costs more than it saves.
const SPIN_FIRST_US: u64 = 20;

// Command register flags: response type, CRC and index checks, data present.
pub const RESP_NONE: u16 = 0x00;
pub const RESP_R1: u16 = 0x1A; // also R5, R6, R7
pub const RESP_R1B: u16 = 0x1B;
pub const RESP_R2: u16 = 0x09;
pub const RESP_R3: u16 = 0x02; // also R4: no CRC, no index
pub const CMD_DATA: u16 = 0x20;

// Transfer mode.
pub const XFER_BLOCK_COUNT: u16 = 1 << 1;
pub const XFER_AUTO_CMD12: u16 = 1 << 2;
pub const XFER_READ: u16 = 1 << 4;
pub const XFER_MULTI: u16 = 1 << 5;

/// What the controller needs from the machine it runs on.
pub trait Platform {
    /// Read the 32-bit register at byte offset `off`.
    fn rd(&self, off: usize) -> u32;
    fn wr(&self, off: usize, v: u32);
    /// A monotonic microsecond clock.
    fn now_us(&self) -> u64;
    fn stall_us(&self, us: u32);
    /// Is the controller's interrupt connected? Without it every wait spins.
    fn irq_connected(&self) -> bool {
        false
    }
    /// Sleep until the interrupt fires or `us` passes. True when it fired.
    fn irq_wait(&self, _us: u64) -> bool {
        false
    }
}

// The fields reach the log through Debug, which dead-code analysis ignores.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    /// The controller never reached a state (what we waited for, INT_STATUS).
    Timeout(&'static str, u32),
    /// The card or the bus reported an error on a command (index, INT_STATUS).
    Command(u8, u32),
    /// The card's answers make no sense (what was wrong, the value).
    Card(&'static str, u32),
}

pub struct Sdhci<P: Platform> {
    pub p: P,
    /// From the capabilities register; 0 until [`Sdhci::power_up`].
    pub base_hz: u32,
}

impl<P: Platform> Sdhci<P> {
    pub fn new(p: P) -> Self {
        Self { p, base_hz: 0 }
    }

    pub fn rd(&self, off: usize) -> u32 {
        self.p.rd(off)
    }
    pub fn wr(&self, off: usize, v: u32) {
        self.p.wr(off, v)
    }

    /// Spin until `done(the register)` or `us` microseconds pass.
    pub fn spin_until(&self, reg: usize, us: u64, done: impl Fn(u32) -> bool) -> Result<u32, u32> {
        let start = self.p.now_us();
        loop {
            let v = self.rd(reg);
            if done(v) {
                return Ok(v);
            }
            if self.p.now_us().wrapping_sub(start) > us {
                let v = self.rd(reg);
                return if done(v) { Ok(v) } else { Err(v) };
            }
            self.p.stall_us(1);
        }
    }

    /// Wait for `done(INT_STATUS)`, sleeping on the interrupt for `bits` (plus
    /// errors) once a short spin has not seen it. Same contract as `spin_until`.
    pub fn wait_int(&self, bits: u32, us: u64, done: impl Fn(u32) -> bool) -> Result<u32, u32> {
        if !self.p.irq_connected() {
            return self.spin_until(REG_INT_STATUS, us, done);
        }
        if let Ok(v) = self.spin_until(REG_INT_STATUS, SPIN_FIRST_US.min(us), &done) {
            return Ok(v);
        }
        let start = self.p.now_us();
        loop {
            // Arm, then look again: a bit that latched before the arm raised no
            // interrupt, and the ISR masks the signals again when one fires.
            self.wr(REG_INT_SIGNAL, bits | INT_ERRORS);
            let v = self.rd(REG_INT_STATUS);
            let spent = self.p.now_us().wrapping_sub(start);
            if done(v) || spent > us {
                self.wr(REG_INT_SIGNAL, 0);
                return if done(v) { Ok(v) } else { Err(v) };
            }
            self.p.irq_wait(us - spent);
        }
    }

    pub fn reset(&self, what: u32) -> Result<(), Error> {
        // The other bits of this dword are the clock and the timeout: keep them.
        self.wr(REG_CLOCK, (self.rd(REG_CLOCK) & !(RESET_ALL | RESET_CMD | RESET_DAT)) | what);
        self.spin_until(REG_CLOCK, 100_000, |v| v & what == 0).map_err(|v| Error::Timeout("software reset", v))?;
        Ok(())
    }

    /// After an error: reset the command and data lines and clear the status.
    pub fn recover(&self) {
        let _ = self.reset(RESET_CMD | RESET_DAT);
        self.wr(REG_INT_STATUS, 0xFFFF_FFFF);
    }

    /// The host version field: 0 = 1.00, 1 = 2.00, 2 = 3.00, more = 4.x.
    pub fn version(&self) -> u32 {
        (self.rd(REG_VERSION) >> 16) & 0xFF
    }

    /// Reset everything, then power the bus at 3.3 V with the slot forced
    /// occupied, latch every status we wait on, set the longest data timeout and
    /// start the card clock at `hz`. Returns the clock it got. The base clock is the
    /// capabilities register's, else whatever the platform put in `base_hz` first:
    /// the Pi 4's Arasan host reports none (Linux's SDHCI_QUIRK_MISSING_CAPS).
    pub fn power_up(&mut self, hz: u32) -> Result<u32, Error> {
        let caps = self.rd(REG_CAPS);
        let from_caps = ((caps >> 8) & 0xFF) * 1_000_000;
        if from_caps != 0 {
            self.base_hz = from_caps;
        }
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
        self.set_clock(hz)
    }

    pub fn set_clock(&mut self, hz: u32) -> Result<u32, Error> {
        // SDHCI 3.0 10-bit divided clock: base / (2 * N), or base itself for N = 0.
        let n = if hz >= self.base_hz { 0 } else { self.base_hz.div_ceil(2 * hz).min(0x3FF) };
        let actual = if n == 0 { self.base_hz } else { self.base_hz / (2 * n) };
        let keep = self.rd(REG_CLOCK) & CLOCK_TIMEOUT_MASK;
        self.wr(REG_CLOCK, keep); // card clock off before changing it
        let clk = keep | ((n & 0xFF) << 8) | (((n >> 8) & 3) << 6) | CLOCK_INTERNAL_ENABLE;
        self.wr(REG_CLOCK, clk);
        self.spin_until(REG_CLOCK, 150_000, |v| v & CLOCK_STABLE != 0).map_err(|v| Error::Timeout("clock stable", v))?;
        self.wr(REG_CLOCK, clk | CLOCK_CARD_ENABLE);
        self.p.stall_us(1000);
        Ok(actual)
    }

    /// Switch the host side of the bus to 4 bits (after the card has).
    pub fn set_4bit(&self) {
        self.wr(REG_HOST_CTRL, self.rd(REG_HOST_CTRL) | HOST_4BIT);
    }

    /// Switch the host to high-speed timing (after the card has), before the clock
    /// goes past 25 MHz.
    pub fn set_high_speed(&self) {
        self.wr(REG_HOST_CTRL, self.rd(REG_HOST_CTRL) | HOST_HIGH_SPEED);
    }

    /// Issue a command; for a data command (`CMD_DATA` in `flags`), the caller
    /// then moves `blocks` blocks of `block_size` bytes with [`Sdhci::read_data`]
    /// or [`Sdhci::write_data`] and ends with [`Sdhci::finish`].
    pub fn issue(
        &self,
        index: u8,
        arg: u32,
        flags: u16,
        mode: u16,
        block_size: u16,
        blocks: u16,
    ) -> Result<[u32; 4], Error> {
        let data = flags & CMD_DATA != 0;
        let busy = data || flags == RESP_R1B;
        let inhibit = PRESENT_CMD_INHIBIT | if busy { PRESENT_DAT_INHIBIT } else { 0 };
        if let Err(v) = self.spin_until(REG_PRESENT, 1_000_000, |v| v & inhibit == 0) {
            self.recover();
            return Err(Error::Timeout("command line free", v));
        }
        // Clear everything but the card interrupt, which is the card's to clear.
        self.wr(REG_INT_STATUS, !INT_CARD);
        if data {
            self.wr(REG_BLOCK, (block_size as u32 & 0xFFF) | (blocks as u32) << 16);
        }
        self.wr(REG_ARG, arg);
        self.wr(REG_XFER_CMD, (index as u32) << 24 | (flags as u32) << 16 | mode as u32);
        let status = match self.wait_int(INT_CMD_DONE, 100_000, |v| v & (INT_CMD_DONE | INT_ERROR) != 0) {
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
            match self.wait_int(INT_XFER_DONE, 2_500_000, |v| v & (INT_XFER_DONE | INT_ERROR) != 0) {
                Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_XFER_DONE),
                Ok(v) | Err(v) => {
                    self.recover();
                    return Err(Error::Command(index, v));
                }
            }
        }
        Ok(r)
    }

    /// A command without data; the first response dword.
    pub fn cmd(&self, index: u8, arg: u32, flags: u16) -> Result<u32, Error> {
        self.issue(index, arg, flags, 0, 0, 0).map(|r| r[0])
    }

    /// Read one block (`buf.len()` bytes, the block size the command set) from
    /// the buffer port. Returns the INT_STATUS that stopped it on an error.
    pub fn read_data(&self, buf: &mut [u8]) -> Result<(), u32> {
        let s = self.wait_int(INT_READ_READY, 1_000_000, |v| v & (INT_READ_READY | INT_ERROR) != 0);
        match s {
            Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_READ_READY),
            Ok(v) | Err(v) => return Err(v),
        }
        let mut chunks = buf.chunks_exact_mut(4);
        for c in &mut chunks {
            c.copy_from_slice(&self.rd(REG_BUFFER).to_le_bytes());
        }
        let tail = chunks.into_remainder();
        if !tail.is_empty() {
            let w = self.rd(REG_BUFFER).to_le_bytes();
            let n = tail.len();
            tail.copy_from_slice(&w[..n]);
        }
        Ok(())
    }

    /// Write one block (`buf.len()` bytes) to the buffer port.
    pub fn write_data(&self, buf: &[u8]) -> Result<(), u32> {
        let s = self.wait_int(INT_WRITE_READY, 1_000_000, |v| v & (INT_WRITE_READY | INT_ERROR) != 0);
        match s {
            Ok(v) if v & INT_ERROR == 0 => self.wr(REG_INT_STATUS, INT_WRITE_READY),
            Ok(v) | Err(v) => return Err(v),
        }
        let mut chunks = buf.chunks_exact(4);
        for c in &mut chunks {
            self.wr(REG_BUFFER, u32::from_le_bytes([c[0], c[1], c[2], c[3]]));
        }
        let tail = chunks.remainder();
        if !tail.is_empty() {
            let mut w = [0u8; 4];
            w[..tail.len()].copy_from_slice(tail);
            self.wr(REG_BUFFER, u32::from_le_bytes(w));
        }
        Ok(())
    }

    /// Wait for transfer complete. For a write it also waits out the card's busy.
    pub fn finish(&self, us: u64) -> Result<(), u32> {
        match self.wait_int(INT_XFER_DONE, us, |v| v & (INT_XFER_DONE | INT_ERROR) != 0) {
            Ok(v) if v & INT_ERROR == 0 => {
                self.wr(REG_INT_STATUS, INT_XFER_DONE);
                Ok(())
            }
            Ok(v) | Err(v) => Err(v),
        }
    }
}
