//! The chip behind SDIO function 1: the Broadcom/Cypress silicon backplane.
//!
//! Function 1 shows a 32 KiB window of the chip's 32-bit backplane, placed by
//! three address registers; an access with bit 15 set in its SDIO address is a
//! 4-byte backplane access. Before anything but those registers answers, the
//! host must ask for the ALP (always-on) clock. Then the chip-common core at
//! 0x1800_0000 gives the chip id, and its enumeration ROM the cores. Then the
//! firmware goes in: halt the ARM CR4, write the program at the bottom of its
//! RAM and the NVRAM at the top, point the reset vector at it, release the core,
//! and enable function 2, whose I/O-ready bit is the firmware saying it runs.
//! After OpenBSD's `if_bwfm_sdio.c` (`bwfm_sdio_attach`, `bwfm_sdio_preinit`,
//! `bwfm_sdio_load_microcode`, `bwfm_sdio_htclk`, `bwfm_sdio_ram_read_write`,
//! `bwfm_sdio_buscore_*`), `bwfm.c` (`bwfm_chip_attach`, `bwfm_chip_ai_*`,
//! `bwfm_chip_dmp_*`, `bwfm_chip_cr4_*`, `bwfm_chip_tcm_*`,
//! `bwfm_chip_sr_capable`) and their register names (`if_bwfm_sdio.h`,
//! `bwfmreg.h`), ISC licensed:
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
pub const F1_WATERMARK: u32 = 0x10008;
pub const F1_WAKEUPCTRL: u32 = 0x1001E;
const WAKEUPCTRL_HTWAIT: u8 = 1 << 1;
pub const F1_SLEEPCSR: u32 = 0x1001F;
const SLEEPCSR_KSO: u8 = 1 << 0;
/// Broadcom's own CCCR bytes (function 0).
pub const CCCR_BRCM_CARDCAP: u32 = 0xF0;
const CARDCAP_CMD14_SUPPORT: u8 = 1 << 1;
const CARDCAP_CMD14_EXT: u8 = 1 << 2;
pub const CCCR_BRCM_CARDCTRL: u32 = 0xF1;
const CARDCTRL_WLANRESET: u8 = 1 << 1;

pub const CLK_FORCE_ALP: u8 = 0x01;
pub const CLK_FORCE_HT: u8 = 0x02;
pub const CLK_HT_AVAIL_REQ: u8 = 0x10;
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

/// Chip-common's pointer to the enumeration ROM.
pub const CHIPCOMMON_EROMPTR: u32 = 0xFC;
const CC_CAPABILITIES: u32 = 0x004;
const CC_CAPABILITIES_PMU: u32 = 0x1000_0000;
const CC_CAPABILITIES_EXT: u32 = 0x0AC;
const CC_CAPABILITIES_EXT_AOB: u32 = 0x40;
const PMU_CONTROL: u32 = 0x600;
const PMU_CONTROL_RES_RELOAD: u32 = 0x2 << 13;
const PMU_CAPABILITIES: u32 = 0x604;
const PMU_CHIPCONTROL_ADDR: u32 = 0x650;
const PMU_CHIPCONTROL_DATA: u32 = 0x654;

// A core's wrapper (agent) registers.
const AGENT_IOCTL: u32 = 0x408;
const IOCTL_CLK: u32 = 0x0001;
const IOCTL_FGC: u32 = 0x0002;
const IOCTL_ARMCR4_CPUHALT: u32 = 0x0020;
const IOCTL_D11_PHYCLOCKEN: u32 = 0x0004;
const IOCTL_D11_PHYRESET: u32 = 0x0008;
const AGENT_RESET_CTL: u32 = 0x800;
const RESET_CTL_RESET: u32 = 0x1;

// The ARM CR4's own registers: its tightly coupled memory banks.
const ARMCR4_CAP: u32 = 0x04;
const ARMCR4_BANKIDX: u32 = 0x40;
const ARMCR4_BANKINFO: u32 = 0x44;
const BANKINFO_BSZ_MASK: u32 = 0x7F;
const BANKINFO_BLK_1K: u32 = 0x200;
/// Where the CR4 sees its RAM on a 4345 (`bwfm_chip_tcm_rambase`).
pub const RAMBASE_4345: u32 = 0x19_8000;

// The SDIO device core: the mailbox between host and firmware.
pub const SDPCMD_INTSTATUS: u32 = 0x020;
pub const SDPCMD_HOSTINTMASK: u32 = 0x024;
pub const SDPCMD_TOSBMAILBOXDATA: u32 = 0x048;
const INTSTATUS_HMB_SW_MASK: u32 = 0xF0;
const INTSTATUS_CHIPACTIVE: u32 = 1 << 29;
/// What the SDIO core may interrupt the host for: the mailbox (frame indication
/// among it) and the chip waking.
const HOSTINTMASK: u32 = INTSTATUS_HMB_SW_MASK | INTSTATUS_CHIPACTIVE;
const SDPCM_PROT_VERSION: u32 = 4;

// Core ids (a component descriptor's part number).
pub const CORE_CHIPCOMMON: u16 = 0x800;
pub const CORE_INTERNAL_MEM: u16 = 0x80E;
pub const CORE_80211: u16 = 0x812;
pub const CORE_PMU: u16 = 0x827;
pub const CORE_SDIO_DEV: u16 = 0x829;
pub const CORE_ARM_CM3: u16 = 0x82A;
pub const CORE_ARM_CR4: u16 = 0x83E;
pub const CORE_GCI: u16 = 0x840;
pub const CORE_ARM_CA7: u16 = 0x847;
pub const CORE_SYS_MEM: u16 = 0x849;

// Enumeration ROM descriptors (bwfmreg.h `BWFM_DMP_*`).
const DESC_MASK: u32 = 0xF;
const DESC_COMPONENT: u32 = 0x1;
const DESC_MASTER_PORT: u32 = 0x3;
const DESC_ADDRESS: u32 = 0x5;
const DESC_ADDRSIZE_GT32: u32 = 0x8;
const DESC_EOT: u32 = 0xF;
const COMP_PARTNUM: u32 = 0x000F_FF00;
const COMP_REVISION: u32 = 0xFF00_0000;
const COMP_NUM_SWRAP: u32 = 0x00F8_0000;
const COMP_NUM_MWRAP: u32 = 0x0007_C000;
const SLAVE_ADDR_BASE: u32 = 0xFFFF_F000;
const SLAVE_TYPE: u32 = 0xC0;
const SLAVE_TYPE_SLAVE: u32 = 0;
const SLAVE_TYPE_SWRAP: u32 = 2;
const SLAVE_TYPE_MWRAP: u32 = 3;
const SLAVE_SIZE_TYPE: u32 = 0x30;
const SLAVE_SIZE_4K: u32 = 0;
const SLAVE_SIZE_8K: u32 = 1;
const SLAVE_SIZE_DESC: u32 = 3;
/// The ROM is a few hundred bytes; a scan this long is reading garbage.
const EROM_MAX_READS: usize = 1024;
pub const MAX_CORES: usize = 24;

/// One core on the backplane: its registers and its wrapper (agent) registers.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Core {
    pub id: u16,
    pub rev: u8,
    pub base: u32,
    pub wrap: u32,
}

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
    /// In enumeration ROM order.
    pub cores: [Core; MAX_CORES],
    pub ncores: usize,
    /// The CR4's RAM, as the CR4 addresses it.
    pub rambase: u32,
    pub ramsize: u32,
    pub cc_caps: u32,
    pub cc_caps_ext: u32,
    pub pmurev: u32,
}

/// What `load_firmware` did, for the log.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Loaded {
    pub firmware: usize,
    pub nvram: usize,
    pub nvram_at: u32,
    pub reset_vector: u32,
}

impl<P: Platform> Chip<P> {
    /// Set the block sizes, enable function 1, start the ALP clock and read the
    /// chip id. `sdio` must have been through `Sdio::init`.
    pub fn attach(mut sdio: Sdio<P>) -> Result<Self, (Sdio<P>, Error)> {
        if let Err(e) = Self::prepare(&mut sdio) {
            return Err((sdio, e));
        }
        let mut chip = Self {
            sdio,
            window: u32::MAX,
            id: ChipId::default(),
            cores: [Core::default(); MAX_CORES],
            ncores: 0,
            rambase: 0,
            ramsize: 0,
            cc_caps: 0,
            cc_caps_ext: 0,
            pmurev: 0,
        };
        match chip.read_4(CHIPCOMMON_BASE) {
            Ok(raw) => chip.id = ChipId::decode(raw),
            Err(e) => return Err((chip.sdio, e)),
        }
        // Only the AI interconnect has a ROM to scan; OpenBSD supports nothing else.
        if chip.id.socitype != 1 {
            return Err((chip.sdio, Error::Card("not an AXI (AI) interconnect", chip.id.raw)));
        }
        if let Err(e) = chip.scan_cores().and_then(|_| chip.settle()) {
            return Err((chip.sdio, e));
        }
        Ok(chip)
    }

    /// With the cores known: hold the CR4 halted, size its RAM, read the
    /// capabilities, and leave the chip as `bwfm_sdio_attach` does: kept awake
    /// (KSO), the WLAN side reset, the PMU's resources reloaded, function 2 off
    /// and no clock request.
    fn settle(&mut self) -> Result<(), Error> {
        let cr4 = self.need(CORE_ARM_CR4, "no ARM CR4 core: not a chip this driver loads")?;
        self.set_passive()?;
        self.ramsize = self.tcm_ramsize(cr4)?;
        self.rambase = match self.id.chip {
            0x4345 => RAMBASE_4345,
            c => return Err(Error::Card("RAM base unknown for this chip", c as u32)),
        };
        let cc = self.need(CORE_CHIPCOMMON, "no chip-common core")?;
        self.cc_caps = self.read_4(cc.base + CC_CAPABILITIES)?;
        self.cc_caps_ext = self.read_4(cc.base + CC_CAPABILITIES_EXT)?;
        if self.cc_caps & CC_CAPABILITIES_PMU != 0 {
            self.pmurev = self.read_4(self.pmu().base + PMU_CAPABILITIES)? & 0xFF;
        }
        let sdio = self.need(CORE_SDIO_DEV, "no SDIO device core")?;
        if sdio.rev >= 12 {
            let s = self.sdio.read_1(1, F1_SLEEPCSR)?;
            if s & SLEEPCSR_KSO == 0 {
                self.sdio.write_1(1, F1_SLEEPCSR, s | SLEEPCSR_KSO)?;
            }
        }
        let c = self.sdio.read_1(0, CCCR_BRCM_CARDCTRL)?;
        self.sdio.write_1(0, CCCR_BRCM_CARDCTRL, c | CARDCTRL_WLANRESET)?;
        let pmu = self.pmu().base + PMU_CONTROL;
        let v = self.read_4(pmu)?;
        self.write_4(pmu, v | PMU_CONTROL_RES_RELOAD)?;
        self.sdio.disable_function(2)?;
        self.sdio.write_1(1, F1_CHIPCLKCSR, 0)
    }

    fn need(&self, id: u16, what: &'static str) -> Result<Core, Error> {
        self.core(id).ok_or(Error::Card(what, id as u32))
    }

    /// The PMU's registers: its own core on newer chip-commons, else chip-common's.
    fn pmu(&self) -> Core {
        let cc = self.core(CORE_CHIPCOMMON).unwrap_or_default();
        if cc.rev >= 35 && self.cc_caps_ext & CC_CAPABILITIES_EXT_AOB != 0 {
            if let Some(p) = self.core(CORE_PMU) {
                return p;
            }
        }
        cc
    }

    // ---- AI core control (`bwfm_chip_ai_*`) --------------------------------

    fn ai_disable(&mut self, core: Core, prereset: u32, reset: u32) -> Result<(), Error> {
        if self.read_4(core.wrap + AGENT_RESET_CTL)? & RESET_CTL_RESET == 0 {
            self.write_4(core.wrap + AGENT_IOCTL, prereset | IOCTL_FGC | IOCTL_CLK)?;
            self.read_4(core.wrap + AGENT_IOCTL)?;
            self.write_4(core.wrap + AGENT_RESET_CTL, RESET_CTL_RESET)?;
            self.sdio.hc.p.stall_us(20);
            let mut held = false;
            for _ in 0..300 {
                if self.read_4(core.wrap + AGENT_RESET_CTL)? == RESET_CTL_RESET {
                    held = true;
                    break;
                }
            }
            if !held {
                return Err(Error::Card("core never entered reset", core.id as u32));
            }
        }
        self.write_4(core.wrap + AGENT_IOCTL, reset | IOCTL_FGC | IOCTL_CLK)?;
        self.read_4(core.wrap + AGENT_IOCTL).map(|_| ())
    }

    fn ai_reset(&mut self, core: Core, prereset: u32, reset: u32, postreset: u32) -> Result<(), Error> {
        self.ai_disable(core, prereset, reset)?;
        let mut out = false;
        for _ in 0..50 {
            if self.read_4(core.wrap + AGENT_RESET_CTL)? & RESET_CTL_RESET == 0 {
                out = true;
                break;
            }
            self.write_4(core.wrap + AGENT_RESET_CTL, 0)?;
            self.sdio.hc.p.stall_us(60);
        }
        if !out {
            return Err(Error::Card("core never left reset", core.id as u32));
        }
        self.write_4(core.wrap + AGENT_IOCTL, postreset | IOCTL_CLK)?;
        self.read_4(core.wrap + AGENT_IOCTL).map(|_| ())
    }

    /// Halt the CR4 and hold the radio's cores in reset (`bwfm_chip_cr4_set_passive`).
    fn set_passive(&mut self) -> Result<(), Error> {
        let cr4 = self.need(CORE_ARM_CR4, "no ARM CR4 core")?;
        let v = self.read_4(cr4.wrap + AGENT_IOCTL)?;
        self.ai_reset(cr4, v & IOCTL_ARMCR4_CPUHALT, IOCTL_ARMCR4_CPUHALT, IOCTL_ARMCR4_CPUHALT)?;
        for i in 0..self.ncores {
            let c = self.cores[i];
            if c.id == CORE_80211 {
                self.ai_disable(c, IOCTL_D11_PHYRESET | IOCTL_D11_PHYCLOCKEN, IOCTL_D11_PHYCLOCKEN)?;
            }
        }
        Ok(())
    }

    /// Clear the mailbox, plant the reset vector at address 0 and let the CR4
    /// run (`bwfm_sdio_buscore_activate`, `bwfm_chip_cr4_set_active`).
    fn set_active(&mut self, rstvec: u32) -> Result<(), Error> {
        let sdio = self.need(CORE_SDIO_DEV, "no SDIO device core")?;
        self.write_4(sdio.base + SDPCMD_INTSTATUS, 0xFFFF_FFFF)?;
        if rstvec != 0 {
            self.write_4(0, rstvec)?;
        }
        let cr4 = self.need(CORE_ARM_CR4, "no ARM CR4 core")?;
        self.ai_reset(cr4, IOCTL_ARMCR4_CPUHALT, 0, 0)
    }

    /// The CR4's RAM: the sum of its TCM banks (`bwfm_chip_tcm_ramsize`).
    fn tcm_ramsize(&mut self, cr4: Core) -> Result<u32, Error> {
        let cap = self.read_4(cr4.base + ARMCR4_CAP)?;
        let banks = (cap & 0xF) + ((cap >> 4) & 0xF);
        let mut size = 0;
        for i in 0..banks {
            self.write_4(cr4.base + ARMCR4_BANKIDX, i)?;
            let info = self.read_4(cr4.base + ARMCR4_BANKINFO)?;
            let blk = if info & BANKINFO_BLK_1K != 0 { 1024 } else { 8192 };
            size += ((info & BANKINFO_BSZ_MASK) + 1) * blk;
        }
        Ok(size)
    }

    /// Save/restore: the firmware may power the chip down between frames
    /// (`bwfm_chip_sr_capable`, the 4345 case).
    fn sr_capable(&mut self) -> Result<bool, Error> {
        if self.pmurev < 17 || self.id.chip != 0x4345 {
            return Ok(false);
        }
        let pmu = self.pmu().base;
        self.write_4(pmu + PMU_CHIPCONTROL_ADDR, 3)?;
        Ok(self.read_4(pmu + PMU_CHIPCONTROL_DATA)? & (1 << 2) != 0)
    }

    // ---- clocks (`bwfm_sdio_htclk`) ----------------------------------------

    /// Request a clock and wait for it: ALP alone, or HT (which brings ALP).
    fn clock_on(&mut self, ht: bool) -> Result<(), Error> {
        let (req, want) =
            if ht { (CLK_HT_AVAIL_REQ, CLK_ALP_AVAIL | CLK_HT_AVAIL) } else { (CLK_ALP_AVAIL_REQ, CLK_ALP_AVAIL) };
        self.sdio.write_1(1, F1_CHIPCLKCSR, req)?;
        let start = self.sdio.hc.p.now_us();
        loop {
            let c = self.sdio.read_1(1, F1_CHIPCLKCSR)?;
            if c & want == want {
                return Ok(());
            }
            if self.sdio.hc.p.now_us().wrapping_sub(start) > 5_000_000 {
                return Err(Error::Card(if ht { "HT clock never came up" } else { "ALP clock never came up" }, c as u32));
            }
            self.sdio.hc.p.stall_us(1000);
        }
    }

    // ---- RAM (`bwfm_sdio_ram_read_write`) ----------------------------------

    /// Move `len` bytes at backplane `addr` window by window, 4-byte accesses
    /// only: a tail shorter than a word goes through `word`, zero-padded.
    fn ram(&mut self, addr: u32, len: usize, mut each: impl FnMut(&mut Self, u32, usize, usize) -> Result<(), Error>) -> Result<(), Error> {
        let mut off = 0;
        while off < len {
            let sb = addr + off as u32;
            self.place_window(sb)?;
            let sd = sb & SB_OFT_ADDR_MASK;
            let size = (len - off).min((SB_OFT_ADDR_MASK + 1 - sd) as usize);
            each(self, sd | SB_ACCESS_2_4B, off, size)?;
            off += size;
        }
        Ok(())
    }

    pub fn ram_write(&mut self, addr: u32, data: &[u8]) -> Result<(), Error> {
        self.ram(addr, data.len(), |c, sd, off, size| {
            let whole = size & !3;
            c.sdio.write(1, sd, true, &data[off..off + whole])?;
            if whole < size {
                let mut w = [0u8; 4];
                w[..size - whole].copy_from_slice(&data[off + whole..off + size]);
                c.sdio.write(1, sd + whole as u32, true, &w)?;
            }
            Ok(())
        })
    }

    pub fn ram_read(&mut self, addr: u32, out: &mut [u8]) -> Result<(), Error> {
        let len = out.len();
        self.ram(addr, len, |c, sd, off, size| {
            let whole = size & !3;
            c.sdio.read(1, sd, true, &mut out[off..off + whole])?;
            if whole < size {
                let mut w = [0u8; 4];
                c.sdio.read(1, sd + whole as u32, true, &mut w)?;
                out[off + whole..off + size].copy_from_slice(&w[..size - whole]);
            }
            Ok(())
        })
    }

    /// Read `data.len()` bytes back from `addr` and compare, a chunk at a time.
    fn verify(&mut self, addr: u32, data: &[u8], what: &'static str) -> Result<(), Error> {
        let mut buf = [0u8; 2048];
        for (i, want) in data.chunks(buf.len()).enumerate() {
            let at = addr + (i * buf.len()) as u32;
            let got = &mut buf[..want.len()];
            self.ram_read(at, got)?;
            if let Some(k) = got.iter().zip(want).position(|(a, b)| a != b) {
                return Err(Error::Card(what, at + k as u32));
            }
        }
        Ok(())
    }

    // ---- the firmware ------------------------------------------------------

    /// Write the firmware at the bottom of the CR4's RAM and the converted
    /// NVRAM (`nvram::convert`) at the top, verify both, and start the CR4 at
    /// the firmware's first word (`bwfm_sdio_load_microcode`), all under the
    /// ALP clock.
    pub fn load_firmware(&mut self, firmware: &[u8], nvram: &[u8]) -> Result<Loaded, Error> {
        if firmware.len() < 4 || nvram.len() % 4 != 0 || firmware.len() + nvram.len() > self.ramsize as usize {
            return Err(Error::Card("firmware and NVRAM do not fit the chip's RAM", self.ramsize));
        }
        self.clock_on(false)?;
        let result = self.load(firmware, nvram);
        self.sdio.write_1(1, F1_CHIPCLKCSR, 0)?;
        result
    }

    fn load(&mut self, firmware: &[u8], nvram: &[u8]) -> Result<Loaded, Error> {
        self.ram_write(self.rambase, firmware)?;
        self.verify(self.rambase, firmware, "firmware read back differently")?;
        let nvram_at = self.rambase + self.ramsize - nvram.len() as u32;
        self.ram_write(nvram_at, nvram)?;
        self.verify(nvram_at, nvram, "NVRAM read back differently")?;
        let reset_vector = u32::from_le_bytes([firmware[0], firmware[1], firmware[2], firmware[3]]);
        self.set_active(reset_vector)?;
        Ok(Loaded { firmware: firmware.len(), nvram: nvram.len(), nvram_at, reset_vector })
    }

    /// After `load_firmware`: run on HT, tell the firmware our mailbox version,
    /// and enable function 2. The firmware sets its I/O-ready once it is up, so
    /// that is the wait for it (`bwfm_sdio_preinit`). Then unmask the mailbox
    /// interrupts and, on a save/restore chip, let it wake for HT on its own.
    pub fn start(&mut self) -> Result<(), Error> {
        self.clock_on(true)?;
        let clk = self.sdio.read_1(1, F1_CHIPCLKCSR)?;
        self.sdio.write_1(1, F1_CHIPCLKCSR, clk | CLK_FORCE_HT)?;
        let sdio = self.need(CORE_SDIO_DEV, "no SDIO device core")?;
        self.write_4(sdio.base + SDPCMD_TOSBMAILBOXDATA, SDPCM_PROT_VERSION << 16)?;
        self.sdio.enable_function(2)?;
        self.write_4(sdio.base + SDPCMD_HOSTINTMASK, HOSTINTMASK)?;
        self.sdio.write_1(1, F1_WATERMARK, 8)?;
        if self.sr_capable()? {
            let w = self.sdio.read_1(1, F1_WAKEUPCTRL)?;
            self.sdio.write_1(1, F1_WAKEUPCTRL, w | WAKEUPCTRL_HTWAIT)?;
            self.sdio.write_1(0, CCCR_BRCM_CARDCAP, CARDCAP_CMD14_SUPPORT | CARDCAP_CMD14_EXT)?;
            self.sdio.write_1(1, F1_CHIPCLKCSR, CLK_FORCE_HT)
        } else {
            self.sdio.write_1(1, F1_CHIPCLKCSR, clk)
        }
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
        sdio.hc.p.stall_us(65);
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

    /// Clear the SDIO core's pending host interrupts (write 1 to clear). Until they
    /// are, the card holds its interrupt (DAT1) asserted. Returns those that were set.
    pub fn ack_interrupts(&mut self) -> Result<u32, Error> {
        let sdio = self.need(CORE_SDIO_DEV, "no SDIO device core")?;
        let s = self.read_4(sdio.base + SDPCMD_INTSTATUS)? & HOSTINTMASK;
        if s != 0 {
            self.write_4(sdio.base + SDPCMD_INTSTATUS, s)?;
        }
        Ok(s)
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

    pub fn core(&self, id: u16) -> Option<Core> {
        self.cores[..self.ncores].iter().copied().find(|c| c.id == id)
    }

    /// Walk the enumeration ROM: each component's two descriptors give its id,
    /// revision and wrapper counts, and the address descriptors after them its
    /// register and wrapper bases (`bwfm_chip_dmp_erom_scan`).
    fn scan_cores(&mut self) -> Result<(), Error> {
        let mut erom = self.read_4(CHIPCOMMON_BASE + CHIPCOMMON_EROMPTR)?;
        let mut reads = 0;
        let mut desc = 0;
        while desc != DESC_EOT {
            reads += 1;
            if reads > EROM_MAX_READS {
                return Err(Error::Card("enumeration ROM never ends", erom));
            }
            let a = self.read_4(erom)?;
            desc = a & DESC_MASK;
            erom += 4;
            if desc == DESC_COMPONENT {
                if let Some(core) = self.component(a, &mut erom, &mut reads)? {
                    if self.ncores == MAX_CORES {
                        return Err(Error::Card("more cores than MAX_CORES", core.id as u32));
                    }
                    self.cores[self.ncores] = core;
                    self.ncores += 1;
                }
            }
        }
        Ok(())
    }

    /// The component whose first descriptor is `a` (`erom` just past it), or
    /// None when it has no wrappers (other than the PMU and GCI, which have
    /// none) or no register region.
    fn component(&mut self, a: u32, erom: &mut u32, reads: &mut usize) -> Result<Option<Core>, Error> {
        let id = ((a & COMP_PARTNUM) >> 8) as u16;
        let b = self.read_4(*erom)?;
        *erom += 4;
        if b & DESC_MASK != DESC_COMPONENT {
            return Err(Error::Card("component descriptor without its second word", b));
        }
        let nmw = (b & COMP_NUM_MWRAP) >> 14;
        let nsw = (b & COMP_NUM_SWRAP) >> 19;
        let rev = ((b & COMP_REVISION) >> 24) as u8;
        if nmw + nsw == 0 && id != CORE_PMU && id != CORE_GCI {
            return Ok(None);
        }
        Ok(self.region(erom, reads)?.map(|(base, wrap)| Core { id, rev, base, wrap }))
    }

    /// The register base and wrapper base following a component, or None when
    /// it has none (`bwfm_chip_dmp_get_regaddr`). Leaves `erom` at the next
    /// component.
    fn region(&mut self, erom: &mut u32, reads: &mut usize) -> Result<Option<(u32, u32)>, Error> {
        let (mut base, mut wrap) = (0, 0);
        let v = self.read_4(*erom)?;
        let wraptype = if v & DESC_MASK == DESC_MASTER_PORT {
            *erom += 4;
            SLAVE_TYPE_MWRAP
        } else if (v & DESC_MASK) & !DESC_ADDRSIZE_GT32 == DESC_ADDRESS {
            SLAVE_TYPE_SWRAP
        } else {
            return Ok(None);
        };
        loop {
            let v = loop {
                *reads += 1;
                if *reads > EROM_MAX_READS {
                    return Err(Error::Card("enumeration ROM never ends", *erom));
                }
                let v = self.read_4(*erom)?;
                match v & DESC_MASK {
                    DESC_COMPONENT => return Ok(Some((base, wrap))),
                    DESC_EOT => return Ok(None),
                    _ => {}
                }
                *erom += 4;
                if (v & DESC_MASK) & !DESC_ADDRSIZE_GT32 == DESC_ADDRESS {
                    break v;
                }
            };
            if v & DESC_ADDRSIZE_GT32 != 0 {
                *erom += 4;
            }
            let size = (v & SLAVE_SIZE_TYPE) >> 4;
            if size == SLAVE_SIZE_DESC {
                let d = self.read_4(*erom)?;
                *erom += if d & DESC_ADDRSIZE_GT32 != 0 { 8 } else { 4 };
            }
            if size != SLAVE_SIZE_4K && size != SLAVE_SIZE_8K {
                continue;
            }
            let stype = (v & SLAVE_TYPE) >> 6;
            if base == 0 && stype == SLAVE_TYPE_SLAVE {
                base = v & SLAVE_ADDR_BASE;
            }
            if wrap == 0 && stype == wraptype {
                wrap = v & SLAVE_ADDR_BASE;
            }
            if base != 0 && wrap != 0 {
                return Ok(Some((base, wrap)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Card, Sim, CR4_WRAP};

    fn attached_with(card: Card) -> Chip<Sim> {
        let mut s = Sdio::new(Sim::new(card));
        s.init().unwrap();
        Chip::attach(s).map_err(|(_, e)| e).unwrap()
    }

    fn attached() -> Chip<Sim> {
        attached_with(Card::cyw43455())
    }

    /// A firmware image whose length is not a whole word and which crosses a
    /// 32 KiB window, with its reset vector first.
    fn firmware() -> Vec<u8> {
        let mut fw: Vec<u8> = (0..40_001u32).map(|i| (i * 7 + 3) as u8).collect();
        fw[..4].copy_from_slice(&0xB800_0EA0u32.to_le_bytes());
        fw
    }

    fn nvram() -> Vec<u8> {
        let mut out = [0u8; 64];
        let n = crate::nvram::convert(b"boardflags=0x00480201\nccode=X2\n", &mut out).unwrap();
        out[..n].to_vec()
    }

    #[test]
    fn attach_reads_the_pi_5s_chip_id() {
        let chip = attached();
        // What the Pi 5's Linux reads at 0x18000000: BCM4345 rev 6, AXI.
        assert_eq!(chip.id.raw, 0x1526_4345);
        assert_eq!((chip.id.chip, chip.id.rev, chip.id.socitype), (0x4345, 6, 1));
        let card = chip.sdio.hc.p.card();
        assert_eq!(card.io_enable, 0b10, "function 1 on, function 2 off");
        assert_eq!(card.clkcsr, 0, "attach leaves no clock request");
        assert_eq!((card.fbr_block[1], card.fbr_block[2]), (64, 512));
    }

    #[test]
    fn attach_halts_the_cr4_and_sizes_its_ram() {
        let chip = attached();
        assert_eq!((chip.rambase, chip.ramsize), (0x19_8000, 256 * 1024));
        assert_eq!(chip.pmurev, 24);
        let card = chip.sdio.hc.p.card();
        assert_eq!(card.word(CR4_WRAP + 0x408), IOCTL_ARMCR4_CPUHALT | IOCTL_CLK, "halted, clocked");
        assert_eq!(card.word(CR4_WRAP + 0x800), 0, "out of reset");
        assert!(!card.firmware_running());
        assert_eq!(card.f1_regs[(F1_SLEEPCSR - 0x10000) as usize] & SLEEPCSR_KSO, SLEEPCSR_KSO);
        assert_eq!(card.cccr[CCCR_BRCM_CARDCTRL as usize] & CARDCTRL_WLANRESET, CARDCTRL_WLANRESET);
    }

    #[test]
    fn the_firmware_goes_in_verified_and_function_2_says_it_runs() {
        let mut chip = attached();
        let (fw, nv) = (firmware(), nvram());
        let loaded = chip.load_firmware(&fw, &nv).unwrap();
        assert_eq!(loaded.reset_vector, 0xB800_0EA0);
        assert_eq!(loaded.nvram_at, 0x19_8000 + 256 * 1024 - nv.len() as u32);
        {
            let card = chip.sdio.hc.p.card();
            assert_eq!(card.word(0), 0xB800_0EA0, "reset vector planted at 0");
            assert!(card.firmware_running());
            // The last, partial word went in zero-padded.
            let last = 0x19_8000 + 40_000;
            assert_eq!(card.word(last), fw[40_000] as u32);
            assert_eq!(card.clkcsr, 0, "ALP dropped after the load");
        }
        let mut back = vec![0u8; fw.len()];
        chip.ram_read(chip.rambase, &mut back).unwrap();
        assert_eq!(back, fw);
        chip.start().unwrap();
        let card = chip.sdio.hc.p.card();
        assert_eq!(card.io_enable, 0b110);
        assert_eq!(card.word(0x1800_4000 + SDPCMD_TOSBMAILBOXDATA), 4 << 16);
        assert_eq!(card.word(0x1800_4000 + SDPCMD_HOSTINTMASK), 0xF0 | 1 << 29);
        assert_eq!(card.f1_regs[(F1_WATERMARK - 0x10000) as usize], 8);
        assert_eq!(card.clkcsr & CLK_FORCE_HT, 0, "no save/restore: HT not forced at the end");
        assert_eq!(card.clkcsr & CLK_HT_AVAIL_REQ, CLK_HT_AVAIL_REQ);
    }

    #[test]
    fn ram_that_reads_back_wrong_is_caught_where_it_differs() {
        let mut card = Card::cyw43455();
        card.stuck_bit_at = Some(0x19_8000 + 0x9000);
        let mut chip = attached_with(card);
        let fw = vec![0u8; 0xA000];
        match chip.load_firmware(&fw, &nvram()) {
            Err(Error::Card(what, at)) => assert_eq!((what, at), ("firmware read back differently", 0x19_8000 + 0x9000)),
            r => panic!("expected a verify failure, got {r:?}"),
        }
        assert!(!chip.sdio.hc.p.card().firmware_running(), "never started");
    }

    #[test]
    fn firmware_that_never_comes_up_is_a_timeout_on_function_2() {
        let mut card = Card::cyw43455();
        card.fw_never_ready = true;
        let mut chip = attached_with(card);
        chip.load_firmware(&firmware(), &nvram()).unwrap();
        match chip.start() {
            Err(Error::Card(what, _)) => assert!(what.contains("ready"), "{what}"),
            r => panic!("expected a function 2 timeout, got {r:?}"),
        }
    }

    #[test]
    fn firmware_too_big_for_the_ram_is_refused_before_any_write() {
        let mut chip = attached();
        let fw = vec![1u8; 256 * 1024];
        assert!(chip.load_firmware(&fw, &nvram()).is_err());
        assert_eq!(chip.sdio.hc.p.card().word(0x19_8000), 0);
    }

    #[test]
    fn the_enumeration_rom_gives_each_core_its_bases() {
        let chip = attached();
        // sim.rs's ROM: chip-common, the CR4 (through a master port), the SDIO
        // device, a component with no wrappers (skipped), the PMU, then EOT.
        assert_eq!(chip.ncores, 4);
        assert_eq!(
            chip.core(CORE_CHIPCOMMON),
            Some(Core { id: CORE_CHIPCOMMON, rev: 0x2B, base: 0x1800_0000, wrap: 0x1810_0000 })
        );
        assert_eq!(chip.core(CORE_ARM_CR4), Some(Core { id: CORE_ARM_CR4, rev: 7, base: 0x1800_2000, wrap: 0x1810_2000 }));
        assert_eq!(chip.core(CORE_PMU), Some(Core { id: CORE_PMU, rev: 1, base: 0x1801_2000, wrap: 0x1811_2000 }));
        assert_eq!(chip.core(CORE_80211), None);
    }

    #[test]
    fn the_window_moves_only_when_the_32k_changes() {
        let mut chip = attached();
        chip.read_4(0x1800_0000).unwrap(); // the ROM scan left the window elsewhere
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
