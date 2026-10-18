//! `GemPhy`: the Raspberry Pi 5's Ethernet, a Cadence GEM (rev 0x00070109) inside
//! the RP1 I/O chip, as a smoltcp `phy::Device`, the sibling of `GenetPhy`.
//!
//! The device does not come from ACPI: worproject's Pi 5 tables declare RP1 but
//! only its two xHCIs, so the HAL synthesizes `VIBEOS\RP1GEM` at PBAR + 0x100000
//! (hal-rust/src/arm64/acpi.rs `find_rp1_gem`). Everything else this driver
//! needs from RP1 sits at fixed offsets from the same window, so it derives
//! RP1's base from the GEM's address. Register values are Linux's
//! (`raspberrypi,rp1-gem` + the BCM54213PE on MDIO address 1, `rgmii-id`), read
//! from a running stager on the bench board.
//!
//! Before the GEM, two things only Linux sets up: the GEM's transmit clock
//! (RP1 clk_eth) and the PHY's reset line (RP1 GPIO 32, active low). Both are
//! checked and, if needed, set here; the boot log records what the firmware
//! left so a failure says which one was missing.
//!
//! Shape, as in GENET v1: polled, copy-based. Unlike GENET the descriptors live
//! in memory (non-cached), 16 bytes each in the GEM's 64-bit addressing mode; the
//! packet buffers are cached and maintained by hand because RP1 is not coherent
//! (`RP1B._CCA` is 0).

use core::sync::atomic::{fence, Ordering};

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;

use alloc::vec::Vec;

use ntkrnl_sys::ddk::{
    KeStallExecutionProcessor, MmAllocateContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmMapIoSpace, PVOID,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};

use crate::genet::{
    dma_arena, dma_clean, dma_invalidate, phys_li, ANAR_100, ANAR_100_FDX, ANAR_10_100_ALL, ANAR_10_FDX,
    AUXCTL_MISC_DATA_MASK, AUXCTL_MISC_READ_SHIFT, AUXCTL_MISC_RGMII_SKEW_EN, AUXCTL_MISC_WRITE_EN,
    AUXCTL_SHADOW_MISC, BMCR_ANE, BMCR_RESET, BMCR_RESTART_AN, BMSR_ANEG_COMPLETE, BMSR_LINK, BRGPHY_AUXCTL,
    BRGPHY_SHADOW_1C, ETH_MAX_FRAME, GBCR_1000_ALL, GBCR_1000_FDX, MII_ANAR, MII_ANLPAR, MII_BMCR, MII_BMSR,
    MII_GBCR, MII_GBSR, MII_PHYIDR1, MII_PHYIDR2, SHADOW_1C_CLK_CTRL, SHADOW_1C_DATA_MASK, SHADOW_1C_GTXCLK_EN,
    SHADOW_1C_WRITE_EN,
};
use crate::pnp::PlatformNic;
use crate::serial::nsprint;
use crate::time::{delay_ms, now};

/// Clause 22 access to clause 45 MMD registers (IEEE 802.3 22.2.4.3.11/12).
const MII_MMD_CTRL: u8 = 13;
const MII_MMD_DATA: u8 = 14;
const MMD_CTRL_DATA_NO_INCR: u16 = 0x4000;
/// MMD 7 (auto-negotiation), register 60: EEE advertisement.
const MMD_AN: u16 = 7;
const MMD_AN_EEE_ADV: u16 = 60;

// ------------------------------------------------------------ GEM registers
const NCR: usize = 0x000;
const NCR_RE: u32 = 1 << 2;
const NCR_TE: u32 = 1 << 3;
const NCR_MPE: u32 = 1 << 4;
const NCR_CLRSTAT: u32 = 1 << 5;
const NCR_TSTART: u32 = 1 << 9;
const NCFGR: usize = 0x004;
const NCFGR_SPD: u32 = 1 << 0;
const NCFGR_FD: u32 = 1 << 1;
const NCFGR_GBE: u32 = 1 << 10;
/// Drop the FCS from received frames.
const NCFGR_DRFCS: u32 = 1 << 17;
/// MDC = pclk / 96: 5 in bits 18-20, as Linux programs it.
const NCFGR_CLK_DIV96: u32 = 5 << 18;
const NCFGR_DBW_SHIFT: u32 = 21;
const NSR: usize = 0x008;
const NSR_IDLE: u32 = 1 << 2;
const USRIO: usize = 0x00c;
const DMACFG: usize = 0x010;
/// 16-beat bursts, full RX packet buffer, full TX packet buffer, 64-bit
/// descriptors. The RX buffer size field is added below.
const DMACFG_BASE: u32 = 0x10 | (3 << 8) | (1 << 10) | (1 << 30);
const DMACFG_RXBS_SHIFT: u32 = 16;
const TSR: usize = 0x014;
/// Transmit go: the TX DMA is running.
const TSR_TGO: u32 = 1 << 3;
const RBQP: usize = 0x018;
const TBQP: usize = 0x01c;
const RSR: usize = 0x020;
const ISR: usize = 0x024;
const IER: usize = 0x028;
const IDR: usize = 0x02c;
const IMR: usize = 0x030;
// Interrupt causes (Linux's macb.h): frame received, RX ran out of buffers, TX
// read a CPU-owned descriptor (went idle), TX underrun, TX error, frame sent,
// receive overrun, and a DMA bus error.
const INT_RCOMP: u32 = 1 << 1;
const INT_RXUBR: u32 = 1 << 2;
const INT_TXUBR: u32 = 1 << 3;
const INT_TUND: u32 = 1 << 4;
const INT_TXERR: u32 = 1 << 6;
const INT_TCOMP: u32 = 1 << 7;
const INT_ROVR: u32 = 1 << 10;
const INT_HRESP: u32 = 1 << 11;
const INT_WAKE: u32 = INT_RCOMP | INT_RXUBR | INT_TXUBR | INT_TUND | INT_TXERR | INT_TCOMP | INT_ROVR | INT_HRESP;
const MAN: usize = 0x034;
/// Clause 22: start 01, code 10.
const MAN_C22: u32 = (1 << 30) | (2 << 16);
const MAN_READ: u32 = 2 << 28;
const MAN_WRITE: u32 = 1 << 28;
const MAN_PHYA_SHIFT: u32 = 23;
const MAN_REGA_SHIFT: u32 = 18;
const HRB: usize = 0x080;
const HRT: usize = 0x084;
const SA1B: usize = 0x088;
const SA1T: usize = 0x08c;
const MID: usize = 0x0fc;
const DCFG1: usize = 0x280;
/// Data bus width the GEM was built with, bits 25-27: 4 = 128-bit, 2 = 64-bit.
const DCFG1_DBWDEF_SHIFT: u32 = 25;
const DCFG6: usize = 0x294;
const TBQPH: usize = 0x4c8;
// Statistics (clear on read): Linux's macb.h GEM_*.
const GEM_TXCNT: usize = 0x108;
const GEM_RXCNT: usize = 0x158;
const GEM_RXFCSCNT: usize = 0x190;
const GEM_RXRESERRCNT: usize = 0x1a0;
const GEM_RXORCNT: usize = 0x1a4;
/// How often the poll path folds the counters in, and reports errors.
const STATS_MS: i64 = 10_000;
const RBQPH: usize = 0x4d4;

// ------------------------------------------------------------- descriptors
const GEM_DESC_COUNT: usize = 128;
const GEM_DESC_SIZE: usize = 16;
const RX_ADDR_USED: u32 = 1 << 0;
const RX_ADDR_WRAP: u32 = 1 << 1;
const RX_LEN_MASK: u32 = 0x1fff;
const RX_SOF: u32 = 1 << 14;
const RX_EOF: u32 = 1 << 15;
const TX_LAST: u32 = 1 << 15;
const TX_WRAP: u32 = 1 << 30;
const TX_USED: u32 = 1 << 31;

/// One buffer per descriptor, 2 KiB apart so each owns its cache lines; the GEM
/// is told 1536 (DMACFG RXBS counts 64-byte units).
const BUF_SIZE: usize = 2048;
const RX_BUF_LEN: u32 = 1536;
/// Descriptors are polled by both sides, so they are non-cached. Below 3 GiB,
/// as the packet buffers are (genet.rs `DMA_LIMIT`).
const DESC_LIMIT: u64 = 0xBFFF_FFFF;

// ------------------------------------------------------ RP1, around the GEM
/// The GEM sits 0x100000 into RP1's peripheral window.
const RP1_GEM_OFFSET: u64 = 0x10_0000;
/// RP1 clocks: clk_eth (the GEM's tx_clk) and clk_eth_tsu, Linux's clk-rp1.c.
const RP1_CLOCKS: u64 = 0x1_8000;
const CLK_ETH_CTRL: usize = 0x064;
const CLK_ETH_DIV_INT: usize = 0x068;
const CLK_ETH_TSU_CTRL: usize = 0x134;
const CLK_CTRL_ENABLE: u32 = 1 << 11;
/// clk_eth runs at 125 MHz with divider 1 (Linux at 1 Gb/s); 100 and 10 Mb/s
/// RGMII want 25 and 2.5 MHz.
const CLK_ETH_DIV_1000: u32 = 1;
const CLK_ETH_DIV_100: u32 = 5;
const CLK_ETH_DIV_10: u32 = 50;
/// GPIO 32 is bank 1's pin 4 (bank 0 is GPIOs 0-27).
const RP1_IO_BANK1: u64 = 0xd_4000;
const GPIO32_CTRL: usize = 4 * 8 + 4;
const GPIO_FUNCSEL_MASK: u32 = 0x1f;
const GPIO_FUNCSEL_SYS_RIO: u32 = 5;
const RP1_RIO1: u64 = 0xe_4000;
const RIO_OUT: usize = 0x0;
const RIO_OE: usize = 0x4;
const RIO_SET: usize = 0x2000;
const RIO_CLR: usize = 0x3000;
const PHY_RESET_PIN: u32 = 1 << 4;
/// GPIO 32's pad: bank 1's pads start after a voltage-select word.
const RP1_PADS_BANK1: u64 = 0xf_4000;
const PAD_GPIO32: usize = 4 + 4 * 4;
/// Output disable: set out of reset, so the pin never sees what RIO drives.
const PAD_OD: u32 = 1 << 7;
const PAD_IE: u32 = 1 << 6;
/// The DT's phy-reset-duration.
const PHY_RESET_MS: i64 = 5;
/// RP1's per-source interrupt routing: MSIX_CFG(n) = 0x108008 + 4n, with
/// atomic set/clear aliases at +0x800/+0xc00. Under this UEFI, RP1's MSI-X is
/// off and its legacy INTA is on, so every enabled source drives INTA, which the
/// BCM2712 root complex delivers as INTID 261: the level, shared line ACPI gives
/// the xHCIs (sources 31 and 36, the only ones the firmware enables) and the HAL
/// gives `VIBEOS\RP1GEM`. Enabling source 6 is all it takes; IACK_EN stays
/// off, since INTA is level and follows the GEM's own status register.
const RP1_PCIE_APBS: u64 = 0x10_8000;
const APBS_REG_SET: usize = 0x800;
const MSIX_CFG_ETH: usize = 0x8 + 4 * 6;
const MSIX_CFG_ENABLE: u32 = 1 << 0;

/// A locally administered address, used only if the firmware will not say the
/// board's own (`board_mac`). Every Pi 5 that falls back shares it.
const FALLBACK_MAC: [u8; 6] = [0x02, 0x56, 0x42, 0x50, 0x35, 0x01];

/// The BCM2712's VideoCore mailbox (soc@107c000000, mailbox@7c013880). RP1
/// only ships on that SoC, so finding RP1 places the mailbox too.
const VC_MBOX: u64 = 0x10_7c01_3880;
const MBOX_READ: usize = 0x00;
const MBOX_STATUS: usize = 0x18;
const MBOX_WRITE: usize = 0x20;
const MBOX_WRITE_STATUS: usize = 0x38;
const MBOX_EMPTY: u32 = 1 << 30;
const MBOX_FULL: u32 = 1 << 31;
const MBOX_CH_PROPERTY: u32 = 8;
const TAG_GET_BOARD_MAC: u32 = 0x0001_0003;
const PROPERTY_OK: u32 = 0x8000_0000;
/// The firmware reads the buffer by its bus address; nothing in the device
/// tree translates it, and below 1 GiB is safe on every board.
const MBOX_BUF_LIMIT: u64 = 0x3FFF_FFFF;

const LINK_WAIT_MS: i64 = 5000;
const LINK_POLL_MS: i64 = 500;

#[derive(Clone, Copy, PartialEq)]
enum Speed {
    S10,
    S100,
    S1000,
}

/// A small MMIO window, mapped once.
struct Mmio(*mut u8);

impl Mmio {
    unsafe fn map(pa: u64, len: usize) -> Option<Self> {
        let va = MmMapIoSpace(phys_li(pa), len as _, MM_NON_CACHED as _) as *mut u8;
        (!va.is_null()).then_some(Mmio(va))
    }
    #[inline]
    fn rd(&self, off: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.0.add(off) as *const u32) }
    }
    #[inline]
    fn wr(&self, off: usize, val: u32) {
        unsafe { core::ptr::write_volatile(self.0.add(off) as *mut u32, val) }
    }
}

pub struct Gem {
    regs: Mmio,
    clocks: Mmio,
    apbs: Mmio,
    mac: [u8; 6],
    phy: u8,
    link: bool,
    ncfgr: u32,
    next_link_poll: i64,
    rx_desc: *mut u32,
    rx_va: *mut u8,
    rx_idx: usize,
    tx_desc: *mut u32,
    tx_va: *mut u8,
    tx_pa: u64,
    tx_prod: usize,
    tx_cons: usize,
    tx_restarts: u32,
    next_stats: i64,
    /// Running totals: frames sent, received, and receive errors (FCS, no
    /// buffer, overrun).
    stats: [u64; 5],
    reported_errors: u64,
}

// The engine worker solely owns the device; the pointers are to non-paged DMA
// memory and MMIO that live for the driver's lifetime.
unsafe impl Send for Gem {}

/// Order the CPU's descriptor writes before what follows, as the DEVICE sees
/// them: the GEM reads descriptors from memory over PCIe while the doorbell
/// (NCR) is a separate MMIO write, so a `dmb ish` (Rust's fences) is not
/// enough and the GEM could read a just-released descriptor as still ours.
/// Linux's wmb()/rmb() are the same instructions.
#[inline]
fn wmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb st", options(nostack, preserves_flags))
    };
    #[cfg(not(target_arch = "aarch64"))]
    fence(Ordering::SeqCst);
}
#[inline]
fn rmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb ld", options(nostack, preserves_flags))
    };
    #[cfg(not(target_arch = "aarch64"))]
    fence(Ordering::SeqCst);
}

#[inline]
unsafe fn desc_rd(ring: *mut u32, idx: usize, word: usize) -> u32 {
    core::ptr::read_volatile(ring.add(idx * GEM_DESC_SIZE / 4 + word))
}
#[inline]
unsafe fn desc_wr(ring: *mut u32, idx: usize, word: usize, val: u32) {
    core::ptr::write_volatile(ring.add(idx * GEM_DESC_SIZE / 4 + word), val)
}

impl Gem {
    fn mdio(&self, cmd: u32) -> Option<u32> {
        let idle = |g: &Self| (0..1000).any(|_| {
            let v = g.regs.rd(NSR) & NSR_IDLE != 0;
            if !v {
                unsafe { KeStallExecutionProcessor(10) };
            }
            v
        });
        if !idle(self) {
            return None;
        }
        self.regs.wr(MAN, cmd);
        idle(self).then(|| self.regs.rd(MAN))
    }
    fn phy_read(&self, reg: u8) -> Option<u16> {
        let cmd = MAN_C22 | MAN_READ | (self.phy as u32) << MAN_PHYA_SHIFT | (reg as u32) << MAN_REGA_SHIFT;
        self.mdio(cmd).map(|v| v as u16)
    }
    fn phy_write(&self, reg: u8, val: u16) -> Option<()> {
        let cmd =
            MAN_C22 | MAN_WRITE | (self.phy as u32) << MAN_PHYA_SHIFT | (reg as u32) << MAN_REGA_SHIFT | val as u32;
        self.mdio(cmd).map(|_| ())
    }

    /// Stop the receiver, transmitter and interrupts, and clear status.
    fn quiesce(&self) {
        self.regs.wr(IDR, 0xFFFF_FFFF);
        self.regs.wr(NCR, 0);
        self.regs.wr(NCR, NCR_CLRSTAT);
        self.regs.wr(TSR, 0xFFFF_FFFF);
        self.regs.wr(RSR, 0xFFFF_FFFF);
        let _ = self.regs.rd(ISR);
        self.regs.wr(HRB, 0);
        self.regs.wr(HRT, 0);
    }

    /// Find the PHY, reset it, set both RGMII delays (`rgmii-id`: the PHY delays
    /// RX and TX clocks), and start auto-negotiation.
    fn phy_init(&mut self) -> Result<(), &'static str> {
        self.phy = (0..32u8)
            .find(|&a| {
                self.phy = a;
                matches!((self.phy_read(MII_PHYIDR1), self.phy_read(MII_PHYIDR2)),
                         (Some(i1), Some(i2)) if i1 != 0xFFFF && i2 != 0xFFFF && (i1, i2) != (0, 0))
            })
            .ok_or("no PHY answers on MDIO")?;
        let id = ((self.phy_read(MII_PHYIDR1).unwrap_or(0) as u32) << 16) | self.phy_read(MII_PHYIDR2).unwrap_or(0) as u32;
        nsprint!("[NETSTACK] gem: PHY {:#010x} at MDIO address {}\n", id, self.phy);

        self.phy_write(MII_BMCR, BMCR_RESET).ok_or("PHY reset write")?;
        if !(0..500).any(|_| {
            let done = self.phy_read(MII_BMCR).is_some_and(|v| v & BMCR_RESET == 0);
            if !done {
                delay_ms(1);
            }
            done
        }) {
            return Err("PHY reset did not complete");
        }

        self.phy_write(BRGPHY_AUXCTL, AUXCTL_SHADOW_MISC | (AUXCTL_SHADOW_MISC << AUXCTL_MISC_READ_SHIFT))
            .ok_or("AUXCTL select")?;
        let misc = self.phy_read(BRGPHY_AUXCTL).ok_or("AUXCTL read")? & AUXCTL_MISC_DATA_MASK;
        self.phy_write(BRGPHY_AUXCTL, AUXCTL_MISC_WRITE_EN | AUXCTL_SHADOW_MISC | misc | AUXCTL_MISC_RGMII_SKEW_EN)
            .ok_or("AUXCTL write")?;
        self.phy_write(BRGPHY_SHADOW_1C, SHADOW_1C_CLK_CTRL).ok_or("SHADOW_1C select")?;
        let clk = self.phy_read(BRGPHY_SHADOW_1C).ok_or("SHADOW_1C read")? & SHADOW_1C_DATA_MASK;
        self.phy_write(BRGPHY_SHADOW_1C, SHADOW_1C_WRITE_EN | SHADOW_1C_CLK_CTRL | clk | SHADOW_1C_GTXCLK_EN)
            .ok_or("SHADOW_1C write")?;

        let anar = self.phy_read(MII_ANAR).ok_or("ANAR read")?;
        self.phy_write(MII_ANAR, anar | ANAR_10_100_ALL).ok_or("ANAR write")?;
        let gbcr = self.phy_read(MII_GBCR).ok_or("GBCR read")?;
        self.phy_write(MII_GBCR, gbcr | GBCR_1000_ALL).ok_or("GBCR write")?;
        // Advertise no Energy Efficient Ethernet. The BCM54213PE advertises EEE for
        // 100BASE-TX and 1000BASE-T out of reset (MMD 7.60 = 0x6), so the link idles in
        // low-power mode between frames, and this driver does nothing about LPI. The
        // result was ping losing ~8% for minutes after boot in a pattern (one frame
        // every 2.5 s, in bursts every ~14 s), every lost request missing from the
        // GEM's own receive counter with no FCS, buffer or overrun error: the frames
        // never reached the MAC. With the advertisement cleared, 200 of 200. Linux
        // likewise lets phylib advertise EEE only when the MAC driver supports it.
        self.mmd_write(MMD_AN, MMD_AN_EEE_ADV, 0).ok_or("EEE advertisement write")?;
        let bmcr = self.phy_read(MII_BMCR).ok_or("BMCR read")?;
        self.phy_write(MII_BMCR, bmcr | BMCR_ANE | BMCR_RESTART_AN).ok_or("BMCR write")?;
        Ok(())
    }

    /// Clause 45 MMD register write through the clause 22 MMD access registers.
    fn mmd_write(&self, devad: u16, reg: u16, val: u16) -> Option<()> {
        self.phy_write(MII_MMD_CTRL, devad)?;
        self.phy_write(MII_MMD_DATA, reg)?;
        self.phy_write(MII_MMD_CTRL, MMD_CTRL_DATA_NO_INCR | devad)?;
        self.phy_write(MII_MMD_DATA, val)
    }

    /// Read the PHY's link state and, on a change, set the MAC's speed and
    /// duplex and the RGMII transmit clock to match.
    fn update_link(&mut self) {
        let Some(bmsr) = self.phy_read(MII_BMSR) else { return };
        let up = bmsr & BMSR_LINK != 0 && bmsr & BMSR_ANEG_COMPLETE != 0;
        if up == self.link {
            return;
        }
        self.link = up;
        if !up {
            nsprint!("[NETSTACK] gem: link down\n");
            return;
        }
        let r = |reg| self.phy_read(reg).unwrap_or(0);
        let (gbcr, gbsr, anlpar, anar) = (r(MII_GBCR), r(MII_GBSR), r(MII_ANLPAR), r(MII_ANAR));
        let gb = (gbsr >> 2) & gbcr;
        let an = anlpar & anar;
        let (speed, full) = if gb & GBCR_1000_ALL != 0 {
            (Speed::S1000, gb & GBCR_1000_FDX != 0)
        } else if an & ANAR_100 != 0 {
            (Speed::S100, an & ANAR_100_FDX != 0)
        } else {
            (Speed::S10, an & ANAR_10_FDX != 0)
        };
        let mut ncfgr = self.ncfgr & !(NCFGR_SPD | NCFGR_FD | NCFGR_GBE);
        let div = match speed {
            Speed::S1000 => {
                ncfgr |= NCFGR_GBE;
                CLK_ETH_DIV_1000
            }
            Speed::S100 => {
                ncfgr |= NCFGR_SPD;
                CLK_ETH_DIV_100
            }
            Speed::S10 => CLK_ETH_DIV_10,
        };
        if full {
            ncfgr |= NCFGR_FD;
        }
        self.clocks.wr(CLK_ETH_DIV_INT, div);
        self.regs.wr(NCFGR, ncfgr);
        nsprint!(
            "[NETSTACK] gem: link up, {} Mb/s {}-duplex\n",
            match speed {
                Speed::S10 => 10,
                Speed::S100 => 100,
                Speed::S1000 => 1000,
            },
            if full { "full" } else { "half" }
        );
    }

    fn poll_link(&mut self) {
        let t = now().total_millis();
        if t >= self.next_link_poll {
            self.next_link_poll = t + LINK_POLL_MS;
            self.update_link();
        }
        if t >= self.next_stats {
            self.next_stats = t + STATS_MS;
            self.fold_stats();
        }
    }

    /// Accumulate the clear-on-read counters, and log a line whenever a
    /// receive error count moved, so a lossy link says where frames went.
    fn fold_stats(&mut self) {
        for (i, reg) in [GEM_TXCNT, GEM_RXCNT, GEM_RXFCSCNT, GEM_RXRESERRCNT, GEM_RXORCNT].into_iter().enumerate() {
            self.stats[i] += self.regs.rd(reg) as u64;
        }
        let errors = self.stats[2] + self.stats[3] + self.stats[4];
        if errors != self.reported_errors {
            self.reported_errors = errors;
            nsprint!(
                "[NETSTACK] gem: tx {} rx {} | rx errors: fcs {} no-buffer {} overrun {} | tx restarts {}\n",
                self.stats[0], self.stats[1], self.stats[2], self.stats[3], self.stats[4], self.tx_restarts
            );
        }
    }

    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    /// The context `irq_ack` takes: the register window.
    pub fn irq_ctx(&self) -> *mut u8 {
        self.regs.0
    }

    /// Unmask the causes the worker acts on, then let RP1 route them. Call only
    /// once an ISR is connected: the line is shared with both xHCIs, and a GEM
    /// interrupt nobody claims would hold the level line asserted.
    pub fn enable_interrupts(&self) {
        let _ = self.regs.rd(ISR);
        self.regs.wr(ISR, 0xFFFF_FFFF);
        self.regs.wr(IER, INT_WAKE);
        self.apbs.wr(APBS_REG_SET + MSIX_CFG_ETH, MSIX_CFG_ENABLE);
        nsprint!("[NETSTACK] gem: interrupts on (IMR {:#x}, RP1 MSIX_CFG(6) {:#x})\n", !self.regs.rd(IMR), self.apbs.rd(MSIX_CFG_ETH));
    }

    /// Frames handed to the GEM that it has not handed back yet.
    /// Whether the PHY reports a link (as of the last poll, at most LINK_POLL_MS old).
    pub fn link_up(&self) -> bool {
        self.link
    }

    pub fn tx_in_flight(&self) -> bool {
        self.tx_cons != self.tx_prod
    }

    pub fn can_recv(&self) -> bool {
        unsafe { desc_rd(self.rx_desc, self.rx_idx, 0) & RX_ADDR_USED != 0 }
    }

    /// Copy the next frame out and give its descriptor back. A bad or
    /// fragmentary frame is consumed and reported as None.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let idx = self.rx_idx;
        unsafe {
            let addr = desc_rd(self.rx_desc, idx, 0);
            if addr & RX_ADDR_USED == 0 {
                return None;
            }
            // The used bit is read before the status it covers.
            rmb();
            let status = desc_rd(self.rx_desc, idx, 1);
            let len = (status & RX_LEN_MASK) as usize;
            let good = status & (RX_SOF | RX_EOF) == RX_SOF | RX_EOF;
            let out = if good && len > 14 && len <= ETH_MAX_FRAME {
                let buf = self.rx_va.add(idx * BUF_SIZE);
                dma_invalidate(buf, len);
                Some(core::slice::from_raw_parts(buf, len).to_vec())
            } else {
                None
            };
            desc_wr(self.rx_desc, idx, 0, addr & !RX_ADDR_USED);
            wmb();
            self.rx_idx = (idx + 1) % GEM_DESC_COUNT;
            out
        }
    }

    /// Retire every transmitted descriptor the GEM has handed back, and restart
    /// a transmitter that went idle with frames still queued: a TSTART written
    /// while TX was finishing the previous frame is ignored, and the GEM had
    /// already read the next descriptor as CPU-owned, so that frame would wait
    /// for the next send (Linux's macb_tx_restart covers the same race).
    fn reap_tx(&mut self) {
        while self.tx_cons != self.tx_prod
            && unsafe { desc_rd(self.tx_desc, self.tx_cons, 1) } & TX_USED != 0
        {
            self.tx_cons = (self.tx_cons + 1) % GEM_DESC_COUNT;
        }
        let tsr = self.regs.rd(TSR);
        if self.tx_cons != self.tx_prod && tsr & TSR_TGO == 0 {
            if self.tx_restarts < 8 {
                nsprint!(
                    "[NETSTACK] gem: TX idle with frames queued (cons {} prod {} TSR {:#x} desc {:#010x}); restarting\n",
                    self.tx_cons,
                    self.tx_prod,
                    tsr,
                    unsafe { desc_rd(self.tx_desc, self.tx_cons, 1) }
                );
            }
            self.tx_restarts += 1;
            self.regs.wr(NCR, self.regs.rd(NCR) | NCR_TSTART);
        }
    }

    pub fn can_send(&mut self) -> bool {
        if !self.link {
            self.poll_link();
            return false;
        }
        self.reap_tx();
        (self.tx_prod + 1) % GEM_DESC_COUNT != self.tx_cons
    }

    pub fn send(&mut self, frame: &[u8]) {
        if frame.len() > ETH_MAX_FRAME {
            return;
        }
        let idx = self.tx_prod;
        unsafe {
            let buf = self.tx_va.add(idx * BUF_SIZE);
            core::ptr::copy_nonoverlapping(frame.as_ptr(), buf, frame.len());
            dma_clean(buf, frame.len());
            let wrap = if idx == GEM_DESC_COUNT - 1 { TX_WRAP } else { 0 };
            // Clearing USED hands the descriptor to the GEM.
            desc_wr(self.tx_desc, idx, 1, frame.len() as u32 | TX_LAST | wrap);
        }
        wmb();
        self.tx_prod = (idx + 1) % GEM_DESC_COUNT;
        self.regs.wr(NCR, self.regs.rd(NCR) | NCR_TSTART);
    }
}

/// Turn clk_eth and clk_eth_tsu on if the firmware left them off, at the
/// divider Linux uses for 1 Gb/s (update_link adjusts it per link speed).
fn enable_clocks(clocks: &Mmio) {
    for ctrl in [CLK_ETH_CTRL, CLK_ETH_TSU_CTRL] {
        let v = clocks.rd(ctrl);
        if v & CLK_CTRL_ENABLE == 0 {
            if ctrl == CLK_ETH_CTRL && clocks.rd(CLK_ETH_DIV_INT) == 0 {
                clocks.wr(CLK_ETH_DIV_INT, CLK_ETH_DIV_1000);
            }
            clocks.wr(ctrl, v | CLK_CTRL_ENABLE);
        }
    }
}

/// Pulse the PHY's reset (GPIO 32, active low) through RP1's RIO block. The
/// firmware leaves the pin unassigned (FUNCSEL 0x1f) with its pad's output
/// disabled; Linux's pinctrl leaves FUNCSEL 5 and pad 0x56.
fn reset_phy(io: &Mmio, rio: &Mmio, pads: &Mmio) {
    pads.wr(PAD_GPIO32, (pads.rd(PAD_GPIO32) & !PAD_OD) | PAD_IE);
    io.wr(GPIO32_CTRL, (io.rd(GPIO32_CTRL) & !GPIO_FUNCSEL_MASK) | GPIO_FUNCSEL_SYS_RIO);
    rio.wr(RIO_CLR + RIO_OUT, PHY_RESET_PIN);
    rio.wr(RIO_SET + RIO_OE, PHY_RESET_PIN);
    delay_ms(PHY_RESET_MS);
    rio.wr(RIO_SET + RIO_OUT, PHY_RESET_PIN);
    // The BCM54213PE answers MDIO a few ms after reset; be generous.
    delay_ms(50);
}

/// Ask the VideoCore firmware for the board's MAC address (property tag
/// GET_BOARD_MAC_ADDRESS), which Linux gets from the device tree the firmware
/// patches. None, with a log line, on any surprise.
unsafe fn board_mac() -> Option<[u8; 6]> {
    let mbox = Mmio::map(VC_MBOX, 0x40)?;
    let buf = MmAllocateContiguousMemorySpecifyCache(64, phys_li(0), phys_li(MBOX_BUF_LIMIT), phys_li(0), MM_NON_CACHED as _)
        as *mut u32;
    if buf.is_null() {
        return None;
    }
    let pa = MmGetPhysicalAddress(buf as PVOID).QuadPart as u64;
    // size, request, tag, value buffer size, request length, 2 value words, end tag
    let msg = [32u32, 0, TAG_GET_BOARD_MAC, 8, 0, 0, 0, 0];
    for (i, w) in msg.iter().enumerate() {
        core::ptr::write_volatile(buf.add(i), *w);
    }
    wmb();
    let wait = |reg: usize, bit: u32| (0..10_000).any(|_| {
        let clear = mbox.rd(reg) & bit == 0;
        if !clear {
            KeStallExecutionProcessor(10);
        }
        clear
    });
    // Drain anything stale, then post.
    while mbox.rd(MBOX_STATUS) & MBOX_EMPTY == 0 {
        let _ = mbox.rd(MBOX_READ);
    }
    if !wait(MBOX_WRITE_STATUS, MBOX_FULL) {
        nsprint!("[NETSTACK] gem: VideoCore mailbox stays full\n");
        return None;
    }
    mbox.wr(MBOX_WRITE, pa as u32 | MBOX_CH_PROPERTY);
    if !wait(MBOX_STATUS, MBOX_EMPTY) {
        nsprint!("[NETSTACK] gem: VideoCore mailbox did not answer (buffer at {:#x})\n", pa);
        return None;
    }
    let reply = mbox.rd(MBOX_READ);
    rmb();
    let code = core::ptr::read_volatile(buf.add(1));
    let (lo, hi) = (core::ptr::read_volatile(buf.add(5)), core::ptr::read_volatile(buf.add(6)));
    let b = lo.to_le_bytes();
    let mac = [b[0], b[1], b[2], b[3], hi.to_le_bytes()[0], hi.to_le_bytes()[1]];
    if reply != (pa as u32 | MBOX_CH_PROPERTY) || code != PROPERTY_OK || mac == [0; 6] || mac[0] & 1 != 0 {
        nsprint!(
            "[NETSTACK] gem: VideoCore mailbox reply {:#x} code {:#x} mac {:02x?}: not usable\n",
            reply, code, mac
        );
        return None;
    }
    Some(mac)
}

/// One non-cached, zeroed descriptor ring below DESC_LIMIT.
unsafe fn desc_ring() -> Option<(*mut u32, u64)> {
    let bytes = GEM_DESC_COUNT * GEM_DESC_SIZE;
    let va = MmAllocateContiguousMemorySpecifyCache(
        bytes as _,
        phys_li(0),
        phys_li(DESC_LIMIT),
        phys_li(0),
        MM_NON_CACHED as _,
    ) as *mut u32;
    if va.is_null() {
        return None;
    }
    core::ptr::write_bytes(va as *mut u8, 0, bytes);
    Some((va, MmGetPhysicalAddress(va as PVOID).QuadPart as u64))
}

/// Take over the GEM the PnP manager started. None, with a serial line naming
/// the failed step, if anything about the hardware is not as expected.
pub fn bringup(nic: PlatformNic) -> Option<Gem> {
    unsafe {
        let rp1 = nic.mmio - RP1_GEM_OFFSET;
        let (Some(regs), Some(clocks), Some(io), Some(rio), Some(apbs), Some(pads)) = (
            Mmio::map(nic.mmio, nic.mmio_len.max(0x4000) as usize),
            Mmio::map(rp1 + RP1_CLOCKS, 0x200),
            Mmio::map(rp1 + RP1_IO_BANK1, 0x40),
            Mmio::map(rp1 + RP1_RIO1, 0x4000),
            Mmio::map(rp1 + RP1_PCIE_APBS, 0x1000),
            Mmio::map(rp1 + RP1_PADS_BANK1, 0x40),
        ) else {
            nsprint!("[NETSTACK] gem: MmMapIoSpace failed (RP1 at {:#x})\n", rp1);
            return None;
        };
        let mid = regs.rd(MID);
        // What the firmware left, in one line, before anything is changed.
        nsprint!(
            "[NETSTACK] gem: RP1 at {:#x}: MID={:#010x} NCR={:#x} NCFGR={:#x} DMACFG={:#x} USRIO={:#x} DCFG1={:#x} DCFG6={:#x} clk_eth={:#x}/{} tsu={:#x} gpio32={:#x} pad={:#x} rio out/oe={}/{} msix6={:#x}\n",
            rp1, mid, regs.rd(NCR), regs.rd(NCFGR), regs.rd(DMACFG), regs.rd(USRIO), regs.rd(DCFG1), regs.rd(DCFG6),
            clocks.rd(CLK_ETH_CTRL), clocks.rd(CLK_ETH_DIV_INT), clocks.rd(CLK_ETH_TSU_CTRL), io.rd(GPIO32_CTRL), pads.rd(PAD_GPIO32),
            rio.rd(RIO_OUT) & PHY_RESET_PIN != 0, rio.rd(RIO_OE) & PHY_RESET_PIN != 0, apbs.rd(MSIX_CFG_ETH)
        );
        // GEM module ID 0x0007 (the macb driver's "is GEM" test is >= 2).
        if mid >> 16 < 2 {
            nsprint!("[NETSTACK] gem: unexpected MID {:#010x} (want a Cadence GEM)\n", mid);
            return None;
        }

        let mut mac = {
            let b = regs.rd(SA1B).to_le_bytes();
            let t = regs.rd(SA1T).to_le_bytes();
            [b[0], b[1], b[2], b[3], t[0], t[1]]
        };
        if mac == [0; 6] || mac[0] & 1 != 0 {
            mac = match board_mac() {
                Some(m) => {
                    nsprint!("[NETSTACK] gem: station address from the VideoCore firmware\n");
                    m
                }
                None => {
                    nsprint!("[NETSTACK] gem: no station address; using locally administered {:02x?}\n", FALLBACK_MAC);
                    FALLBACK_MAC
                }
            };
        }

        enable_clocks(&clocks);
        reset_phy(&io, &rio, &pads);

        let dbw = match (regs.rd(DCFG1) >> DCFG1_DBWDEF_SHIFT) & 7 {
            4 => 2, // 128-bit
            2 => 1, // 64-bit
            _ => 0,
        };
        let ncfgr = NCFGR_FD | NCFGR_GBE | NCFGR_DRFCS | NCFGR_CLK_DIV96 | (dbw << NCFGR_DBW_SHIFT);
        let mut g = Gem {
            regs,
            clocks,
            apbs,
            mac,
            phy: 0,
            link: false,
            ncfgr,
            next_link_poll: 0,
            rx_desc: core::ptr::null_mut(),
            rx_va: core::ptr::null_mut(),
            rx_idx: 0,
            tx_desc: core::ptr::null_mut(),
            tx_va: core::ptr::null_mut(),
            tx_pa: 0,
            tx_prod: 0,
            tx_cons: 0,
            tx_restarts: 0,
            next_stats: 0,
            stats: [0; 5],
            reported_errors: 0,
        };
        g.quiesce();
        g.regs.wr(NCFGR, ncfgr);
        g.regs.wr(NCR, NCR_MPE);
        if let Err(step) = g.phy_init() {
            nsprint!("[NETSTACK] gem: PHY init failed: {}\n", step);
            return None;
        }

        let bytes = GEM_DESC_COUNT * BUF_SIZE;
        let (Some((rx_va, rx_pa)), Some((tx_va, tx_pa)), Some((rx_desc, rx_desc_pa)), Some((tx_desc, tx_desc_pa))) =
            (dma_arena(bytes), dma_arena(bytes), desc_ring(), desc_ring())
        else {
            nsprint!("[NETSTACK] gem: DMA allocation failed\n");
            return None;
        };
        // RX: every descriptor owns its buffer for good; the GEM sets USED when
        // it has filled one. TX: all start USED (CPU-owned).
        for i in 0..GEM_DESC_COUNT {
            let pa = rx_pa + (i * BUF_SIZE) as u64;
            let wrap = if i == GEM_DESC_COUNT - 1 { RX_ADDR_WRAP } else { 0 };
            desc_wr(rx_desc, i, 0, pa as u32 | wrap);
            desc_wr(rx_desc, i, 1, 0);
            desc_wr(rx_desc, i, 2, (pa >> 32) as u32);
            let tpa = tx_pa + (i * BUF_SIZE) as u64;
            let twrap = if i == GEM_DESC_COUNT - 1 { TX_WRAP } else { 0 };
            desc_wr(tx_desc, i, 0, tpa as u32);
            desc_wr(tx_desc, i, 1, TX_USED | twrap);
            desc_wr(tx_desc, i, 2, (tpa >> 32) as u32);
        }
        wmb();
        g.rx_desc = rx_desc;
        g.rx_va = rx_va;
        g.tx_desc = tx_desc;
        g.tx_va = tx_va;
        g.tx_pa = tx_pa;
        g.regs.wr(DMACFG, DMACFG_BASE | ((RX_BUF_LEN / 64) << DMACFG_RXBS_SHIFT));
        g.regs.wr(RBQPH, (rx_desc_pa >> 32) as u32);
        g.regs.wr(RBQP, rx_desc_pa as u32);
        g.regs.wr(TBQPH, (tx_desc_pa >> 32) as u32);
        g.regs.wr(TBQP, tx_desc_pa as u32);
        let m = g.mac;
        g.regs.wr(SA1B, u32::from_le_bytes([m[0], m[1], m[2], m[3]]));
        // Writing SA1T enables the address match.
        g.regs.wr(SA1T, u32::from_le_bytes([m[4], m[5], 0, 0]));
        g.regs.wr(NCR, NCR_MPE | NCR_RE | NCR_TE);

        let deadline = now().total_millis() + LINK_WAIT_MS;
        while !g.link && now().total_millis() < deadline {
            g.update_link();
            if !g.link {
                delay_ms(100);
            }
        }
        nsprint!(
            "[NETSTACK] gem up: mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} regs={:#x} rx ring {:#x} bufs {:#x} tx ring {:#x} bufs {:#x} link={}\n",
            m[0], m[1], m[2], m[3], m[4], m[5], nic.mmio, rx_desc_pa, rx_pa, tx_desc_pa, tx_pa,
            if g.link { "up" } else { "down (still negotiating)" }
        );
        Some(g)
    }
}

/// The ISR's half (DIRQL): read and clear the GEM's interrupt status. This GEM
/// was built without clear-on-read (DCFG1 IRQCOR = 0), so the bits are written
/// back. Touches nothing the worker owns.
pub unsafe fn irq_ack(regs: ntkrnl_sys::ddk::PVOID) -> crate::isr::Ack {
    let isr = regs.cast::<u8>().add(ISR).cast::<u32>();
    let st = core::ptr::read_volatile(isr);
    if st == 0 {
        return crate::isr::Ack::NotOurs;
    }
    core::ptr::write_volatile(isr, st);
    if st & INT_WAKE != 0 { crate::isr::Ack::Wake } else { crate::isr::Ack::Handled }
}

// --------------------------------------------------------- smoltcp adapter
pub struct GemPhy {
    dev: Gem,
}

impl GemPhy {
    pub fn new(dev: Gem) -> Self {
        Self { dev }
    }
    pub fn mac(&self) -> [u8; 6] {
        self.dev.mac_address()
    }
    pub fn dev(&self) -> &Gem {
        &self.dev
    }
}

pub struct GemRxToken {
    data: Vec<u8>,
}

impl phy::RxToken for GemRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }
}

pub struct GemTxToken<'a> {
    dev: &'a mut Gem,
}

impl<'a> phy::TxToken for GemTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = alloc::vec![0u8; len];
        let r = f(&mut buf);
        self.dev.send(&buf);
        r
    }
}

impl Device for GemPhy {
    type RxToken<'a> = GemRxToken;
    type TxToken<'a> = GemTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.dev.poll_link();
        self.dev.reap_tx();
        if !self.dev.can_recv() {
            return None;
        }
        let data = self.dev.receive()?;
        Some((GemRxToken { data }, GemTxToken { dev: &mut self.dev }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if !self.dev.can_send() {
            return None;
        }
        Some(GemTxToken { dev: &mut self.dev })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
