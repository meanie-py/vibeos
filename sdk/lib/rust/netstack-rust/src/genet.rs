//! `GenetPhy`: the Raspberry Pi 4's Broadcom GENET v5 Ethernet MAC as a smoltcp
//! `phy::Device`, the platform sibling of `E1000Phy`.
//!
//! The device comes from ACPI, not PCI (`pnp.rs`): `_HID BCM6E4E`, 64 KiB of
//! registers at 0xFD580000, `_CCA 0`, `phy-mode rgmii-rxid`, a BCM54213PE PHY on
//! the MAC's MDIO bus. Programming follows the two BSD-licensed drivers for this
//! exact part: EDK2's BcmGenetDxe (the firmware's own driver, polled like this
//! one) and FreeBSD's if_genet.
//!
//! Shape, as in `E1000Phy`, interrupt-woken: the worker still polls the rings,
//! and the RX-done interrupt (INTRL2_0, the first of the two lines ACPI gives)
//! only wakes it (`irq_ack`, engine.rs). Copy-based, one contiguous DMA arena
//! per direction. Unlike e1000 the descriptors are not in memory at all: GENET
//! keeps its 256 RX and 256 TX descriptors in its own register file, so only
//! the packet buffers are DMA memory. **That memory is not cache-coherent** (`_CCA 0`): the buffers are
//! cached mappings, each TX buffer is cleaned to the point of coherency before
//! its descriptor is handed over, and each received frame is invalidated
//! before the CPU reads it (the same discipline as usbxhci's `hw.rs`).
//!
//! The firmware leaves the station address in UMAC_MAC0/1 (ACPI carries none),
//! so it is read from there before the MAC reset, as Linux's ACPI path does.

use core::sync::atomic::{fence, Ordering};

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;

use alloc::vec::Vec;

use ntkrnl_sys::ddk::{
    KeStallExecutionProcessor, LARGE_INTEGER, MmAllocateContiguousMemorySpecifyCache, MmGetPhysicalAddress,
    MmMapIoSpace, PVOID, _MEMORY_CACHING_TYPE_MmCached as MM_CACHED,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};

use crate::pnp::PlatformNic;
use crate::serial::nsprint;
use crate::time::{delay_ms, now};

// ---------------------------------------------------------------- registers
const SYS_REV_CTRL: usize = 0x000;
const SYS_PORT_CTRL: usize = 0x004;
const SYS_PORT_MODE_EXT_GPHY: u32 = 3;
const SYS_RBUF_FLUSH_CTRL: usize = 0x008;
const SYS_RBUF_FLUSH_RESET: u32 = 1 << 1;
const EXT_RGMII_OOB_CTRL: usize = 0x08c;
const RGMII_OOB_ID_MODE_DISABLE: u32 = 1 << 16;
const RGMII_OOB_RGMII_MODE_EN: u32 = 1 << 6;
const RGMII_OOB_OOB_DISABLE: u32 = 1 << 5;
const RGMII_OOB_RGMII_LINK: u32 = 1 << 4;
// INTRL2_0: the default queue's (16) DMA interrupts, among others.
const INTRL2_CPU_STAT: usize = 0x200;
const INTRL2_CPU_CLEAR: usize = 0x208;
const INTRL2_CPU_MASK_STATUS: usize = 0x20c;
const INTRL2_CPU_SET_MASK: usize = 0x210;
const INTRL2_CPU_CLEAR_MASK: usize = 0x214;
/// The default queue's RX ring passed its MBUF_DONE threshold (one frame).
const UMAC_IRQ_RXDMA_DONE: u32 = 1 << 13;
const RBUF_CTRL: usize = 0x300;
const RBUF_ALIGN_2B: u32 = 1 << 1;
const RBUF_TBUF_SIZE_CTRL: usize = 0x3b4;
const UMAC_CMD: usize = 0x808;
const UMAC_CMD_LCL_LOOP_EN: u32 = 1 << 15;
const UMAC_CMD_SW_RESET: u32 = 1 << 13;
const UMAC_CMD_HD_EN: u32 = 1 << 10;
const UMAC_CMD_SPEED_SHIFT: u32 = 2;
const UMAC_CMD_SPEED_MASK: u32 = 3 << UMAC_CMD_SPEED_SHIFT;
const UMAC_CMD_RXEN: u32 = 1 << 1;
const UMAC_CMD_TXEN: u32 = 1 << 0;
const UMAC_MAC0: usize = 0x80c;
const UMAC_MAC1: usize = 0x810;
const UMAC_MAX_FRAME_LEN: usize = 0x814;
const UMAC_TX_FLUSH: usize = 0xb34;
const UMAC_MIB_CTRL: usize = 0xd80;
const UMAC_MIB_RESET_ALL: u32 = 0b111; // TX | RUNT | RX
const MDIO_CMD: usize = 0xe14;
const MDIO_START_BUSY: u32 = 1 << 29;
const MDIO_READ: u32 = 1 << 27;
const MDIO_WRITE: u32 = 1 << 26;
const MDIO_PMD_SHIFT: u32 = 21;
const MDIO_REG_SHIFT: u32 = 16;
const UMAC_MDF_CTRL: usize = 0xe50;
const fn umac_mdf_addr0(n: usize) -> usize {
    0xe54 + n * 8
}
const fn umac_mdf_addr1(n: usize) -> usize {
    0xe58 + n * 8
}

/// The one queue used, as the firmware does: the default queue, 16.
const QID: usize = 16;
const DESC_COUNT: usize = 256;
const DESC_SIZE: usize = 12;
const RX_BASE: usize = 0x2000;
const TX_BASE: usize = 0x4000;
const fn ring(base: usize) -> usize {
    base + 0xc00 + 0x40 * QID
}
// RX ring registers.
const RX_WRITE_PTR_LO: usize = ring(RX_BASE);
const RX_WRITE_PTR_HI: usize = ring(RX_BASE) + 0x04;
const RX_PROD_INDEX: usize = ring(RX_BASE) + 0x08;
const RX_CONS_INDEX: usize = ring(RX_BASE) + 0x0c;
const RX_RING_BUF_SIZE: usize = ring(RX_BASE) + 0x10;
const RX_START_ADDR_LO: usize = ring(RX_BASE) + 0x14;
const RX_START_ADDR_HI: usize = ring(RX_BASE) + 0x18;
const RX_END_ADDR_LO: usize = ring(RX_BASE) + 0x1c;
const RX_END_ADDR_HI: usize = ring(RX_BASE) + 0x20;
const RX_MBUF_DONE_THRES: usize = ring(RX_BASE) + 0x24;
const RX_XON_XOFF_THRES: usize = ring(RX_BASE) + 0x28;
const RX_READ_PTR_LO: usize = ring(RX_BASE) + 0x2c;
const RX_READ_PTR_HI: usize = ring(RX_BASE) + 0x30;
// TX ring registers (note the different order).
const TX_READ_PTR_LO: usize = ring(TX_BASE);
const TX_READ_PTR_HI: usize = ring(TX_BASE) + 0x04;
const TX_CONS_INDEX: usize = ring(TX_BASE) + 0x08;
const TX_PROD_INDEX: usize = ring(TX_BASE) + 0x0c;
const TX_RING_BUF_SIZE: usize = ring(TX_BASE) + 0x10;
const TX_START_ADDR_LO: usize = ring(TX_BASE) + 0x14;
const TX_START_ADDR_HI: usize = ring(TX_BASE) + 0x18;
const TX_END_ADDR_LO: usize = ring(TX_BASE) + 0x1c;
const TX_END_ADDR_HI: usize = ring(TX_BASE) + 0x20;
const TX_MBUF_DONE_THRES: usize = ring(TX_BASE) + 0x24;
const TX_FLOW_PERIOD: usize = ring(TX_BASE) + 0x28;
const TX_WRITE_PTR_LO: usize = ring(TX_BASE) + 0x2c;
const TX_WRITE_PTR_HI: usize = ring(TX_BASE) + 0x30;
// Per-direction DMA control.
const RX_DMA_RING_CFG: usize = RX_BASE + 0x1040;
const RX_DMA_CTRL: usize = RX_BASE + 0x1044;
const RX_SCB_BURST_SIZE: usize = RX_BASE + 0x104c;
const TX_DMA_RING_CFG: usize = TX_BASE + 0x1040;
const TX_DMA_CTRL: usize = TX_BASE + 0x1044;
const TX_SCB_BURST_SIZE: usize = TX_BASE + 0x104c;
const DMA_CTRL_EN: u32 = 1 << 0;
const DMA_CTRL_RBUF_EN: u32 = 1 << (1 + QID);
/// `brcm,max-dma-burst-size` from the board's _DSD.
const DMA_BURST: u32 = 8;

// Descriptors, in the register file.
const fn desc(base: usize, idx: usize) -> usize {
    base + DESC_SIZE * idx
}
const DESC_STATUS: usize = 0x0;
const DESC_ADDR_LO: usize = 0x4;
const DESC_ADDR_HI: usize = 0x8;
const DESC_BUFLEN_SHIFT: u32 = 16;
const DESC_BUFLEN_MASK: u32 = 0xfff << DESC_BUFLEN_SHIFT;
const DESC_EOP: u32 = 1 << 14;
const DESC_SOP: u32 = 1 << 13;
const RX_DESC_ERROR: u32 = 1 << 2;
const TX_DESC_QTAG: u32 = 0x3f << 7;
const TX_DESC_CRC: u32 = 1 << 6;

// PHY (IEEE 802.3 clause 22, plus the BCM54213PE's shadow registers).
pub(crate) const MII_BMCR: u8 = 0x00;
pub(crate) const MII_BMSR: u8 = 0x01;
pub(crate) const MII_PHYIDR1: u8 = 0x02;
pub(crate) const MII_PHYIDR2: u8 = 0x03;
pub(crate) const MII_ANAR: u8 = 0x04;
pub(crate) const MII_ANLPAR: u8 = 0x05;
pub(crate) const MII_GBCR: u8 = 0x09;
pub(crate) const MII_GBSR: u8 = 0x0a;
pub(crate) const BMCR_RESET: u16 = 1 << 15;
pub(crate) const BMCR_ANE: u16 = 1 << 12;
pub(crate) const BMCR_RESTART_AN: u16 = 1 << 9;
pub(crate) const BMSR_ANEG_COMPLETE: u16 = 1 << 5;
pub(crate) const BMSR_LINK: u16 = 1 << 2;
pub(crate) const ANAR_10_100_ALL: u16 = 0x01e0; // 100FDX | 100 | 10FDX | 10
pub(crate) const ANAR_100: u16 = 0x0180;
pub(crate) const ANAR_100_FDX: u16 = 0x0100;
pub(crate) const ANAR_10_FDX: u16 = 0x0040;
pub(crate) const GBCR_1000_ALL: u16 = 0x0300; // 1000BASE-T FDX | HDX
pub(crate) const GBCR_1000_FDX: u16 = 0x0200;
pub(crate) const BRGPHY_AUXCTL: u8 = 0x18;
pub(crate) const AUXCTL_SHADOW_MISC: u16 = 0x07;
pub(crate) const AUXCTL_MISC_READ_SHIFT: u16 = 12;
pub(crate) const AUXCTL_MISC_DATA_MASK: u16 = 0x7ff8;
pub(crate) const AUXCTL_MISC_WRITE_EN: u16 = 0x8000;
pub(crate) const AUXCTL_MISC_RGMII_SKEW_EN: u16 = 0x0200;
pub(crate) const BRGPHY_SHADOW_1C: u8 = 0x1c;
pub(crate) const SHADOW_1C_WRITE_EN: u16 = 0x8000;
pub(crate) const SHADOW_1C_CLK_CTRL: u16 = 0x03 << 10;
pub(crate) const SHADOW_1C_DATA_MASK: u16 = 0x03ff;
pub(crate) const SHADOW_1C_GTXCLK_EN: u16 = 0x0200;

// ------------------------------------------------------------------- layout
/// Every hardware descriptor owns a buffer; a stride of 2 KiB keeps each one
/// on its own cache lines, which the invalidate-before-read relies on.
const BUF_SIZE: usize = 2048;
/// RX buffer length programmed into the ring (the firmware's GENET_MAX_PACKET_SIZE).
const RX_BUF_LEN: u32 = 1536;
pub(crate) const ETH_MAX_FRAME: usize = 1518;
/// The bus can address only the low 3 GiB (the firmware caps RAM there too).
const DMA_LIMIT: u64 = 0xBFFF_FFFF;
const LINK_WAIT_MS: i64 = 5000;
const LINK_POLL_MS: i64 = 500;

#[inline]
pub(crate) fn phys_li(v: u64) -> LARGE_INTEGER {
    let mut p: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    p.QuadPart = v as i64;
    p
}

// ---- Cache maintenance for non-coherent DMA (see the module comment) ----
#[cfg(target_arch = "aarch64")]
const DCACHE_LINE: usize = 64;

/// Clean (write back) [va, va+len) so the device reads what the CPU wrote.
pub(crate) unsafe fn dma_clean(va: *const u8, len: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        let mut p = (va as usize) & !(DCACHE_LINE - 1);
        while p < va as usize + len {
            core::arch::asm!("dc cvac, {0}", in(reg) p, options(nostack, preserves_flags));
            p += DCACHE_LINE;
        }
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "aarch64"))]
    let _ = (va, len);
}

/// Invalidate [va, va+len) before reading what the device wrote. The CPU never
/// writes an RX buffer after setup, so no dirty line can be lost here.
pub(crate) unsafe fn dma_invalidate(va: *const u8, len: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        let mut p = (va as usize) & !(DCACHE_LINE - 1);
        while p < va as usize + len {
            core::arch::asm!("dc ivac, {0}", in(reg) p, options(nostack, preserves_flags));
            p += DCACHE_LINE;
        }
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "aarch64"))]
    let _ = (va, len);
}

/// One physically contiguous, zeroed, cleaned buffer arena below DMA_LIMIT.
pub(crate) unsafe fn dma_arena(bytes: usize) -> Option<(*mut u8, u64)> {
    let va = MmAllocateContiguousMemorySpecifyCache(
        bytes as _,
        phys_li(0),
        phys_li(DMA_LIMIT),
        phys_li(0),
        MM_CACHED as _,
    ) as *mut u8;
    if va.is_null() {
        return None;
    }
    core::ptr::write_bytes(va, 0, bytes);
    dma_clean(va, bytes);
    let pa = MmGetPhysicalAddress(va as PVOID).QuadPart as u64;
    Some((va, pa))
}

#[derive(Clone, Copy, PartialEq)]
enum Speed {
    S10 = 0,
    S100 = 1,
    S1000 = 2,
}

pub struct Genet {
    regs: *mut u8,
    mac: [u8; 6],
    phy: u8,
    link: bool,
    next_link_poll: i64,
    rx_va: *mut u8,
    rx_cons: u16,
    tx_va: *mut u8,
    tx_pa: u64,
    tx_prod: u16,
    /// An ISR is connected and RX-done may be unmasked (`rearm_irq`).
    irq_on: core::cell::Cell<bool>,
}

// The engine worker solely owns the device; the pointers are to non-paged DMA
// memory and MMIO that live for the driver's lifetime.
unsafe impl Send for Genet {}

impl Genet {
    #[inline]
    fn rd(&self, off: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.regs.add(off) as *const u32) }
    }
    #[inline]
    fn wr(&self, off: usize, val: u32) {
        unsafe { core::ptr::write_volatile(self.regs.add(off) as *mut u32, val) }
    }

    fn mdio(&self, cmd: u32) -> Option<u32> {
        self.wr(MDIO_CMD, cmd | MDIO_START_BUSY);
        for _ in 0..1000 {
            let v = self.rd(MDIO_CMD);
            if v & MDIO_START_BUSY == 0 {
                return Some(v);
            }
            unsafe { KeStallExecutionProcessor(10) };
        }
        None
    }
    fn phy_read(&self, reg: u8) -> Option<u16> {
        let cmd = MDIO_READ | (self.phy as u32) << MDIO_PMD_SHIFT | (reg as u32) << MDIO_REG_SHIFT;
        self.mdio(cmd).map(|v| v as u16)
    }
    fn phy_write(&self, reg: u8, val: u16) -> Option<()> {
        let cmd = MDIO_WRITE | (self.phy as u32) << MDIO_PMD_SHIFT | (reg as u32) << MDIO_REG_SHIFT | val as u32;
        self.mdio(cmd).map(|_| ())
    }

    /// Stop whatever the firmware left running: interrupts, both DMA engines,
    /// the receiver and transmitter. Its buffers are memory we now own.
    fn quiesce(&self) {
        self.wr(INTRL2_CPU_SET_MASK, 0xFFFF_FFFF);
        self.wr(INTRL2_CPU_CLEAR, 0xFFFF_FFFF);
        self.wr(UMAC_CMD, self.rd(UMAC_CMD) & !UMAC_CMD_RXEN);
        self.wr(RX_DMA_CTRL, self.rd(RX_DMA_CTRL) & !(DMA_CTRL_EN | DMA_CTRL_RBUF_EN));
        self.wr(TX_DMA_CTRL, self.rd(TX_DMA_CTRL) & !(DMA_CTRL_EN | DMA_CTRL_RBUF_EN));
        self.wr(UMAC_TX_FLUSH, 1);
        unsafe { KeStallExecutionProcessor(10) };
        self.wr(UMAC_TX_FLUSH, 0);
        self.wr(UMAC_CMD, self.rd(UMAC_CMD) & !UMAC_CMD_TXEN);
    }

    fn reset(&self) {
        let v = self.rd(SYS_RBUF_FLUSH_CTRL);
        self.wr(SYS_RBUF_FLUSH_CTRL, v | SYS_RBUF_FLUSH_RESET);
        unsafe { KeStallExecutionProcessor(10) };
        self.wr(SYS_RBUF_FLUSH_CTRL, v & !SYS_RBUF_FLUSH_RESET);
        unsafe { KeStallExecutionProcessor(10) };
        self.wr(SYS_RBUF_FLUSH_CTRL, 0);
        unsafe { KeStallExecutionProcessor(10) };

        self.wr(UMAC_CMD, 0);
        self.wr(UMAC_CMD, UMAC_CMD_LCL_LOOP_EN | UMAC_CMD_SW_RESET);
        unsafe { KeStallExecutionProcessor(10) };
        self.wr(UMAC_CMD, 0);

        self.wr(UMAC_MIB_CTRL, UMAC_MIB_RESET_ALL);
        self.wr(UMAC_MIB_CTRL, 0);
        self.wr(UMAC_MAX_FRAME_LEN, RX_BUF_LEN);
        self.wr(RBUF_CTRL, self.rd(RBUF_CTRL) | RBUF_ALIGN_2B);
        self.wr(RBUF_TBUF_SIZE_CTRL, 1);
        self.wr(SYS_PORT_CTRL, SYS_PORT_MODE_EXT_GPHY);
    }

    fn set_mac(&self) {
        let m = self.mac;
        self.wr(UMAC_MAC0, u32::from_be_bytes([m[0], m[1], m[2], m[3]]));
        self.wr(UMAC_MAC1, u32::from_be_bytes([0, 0, m[4], m[5]]));
        // Filter 0: our address; filter 1: broadcast. Enable bits count down
        // from bit 16 (filter 0).
        self.wr(umac_mdf_addr0(0), u32::from_be_bytes([0, 0, m[0], m[1]]));
        self.wr(umac_mdf_addr1(0), u32::from_be_bytes([m[2], m[3], m[4], m[5]]));
        self.wr(umac_mdf_addr0(1), 0xffff);
        self.wr(umac_mdf_addr1(1), 0xffff_ffff);
        self.wr(UMAC_MDF_CTRL, (1 << 16) | (1 << 15));
    }

    /// Find the PHY, reset it, set the RGMII receive-clock skew the table's
    /// `rgmii-rxid` asks for, and start auto-negotiation.
    fn phy_init(&mut self) -> Result<(), &'static str> {
        self.phy = (0..32u8)
            .find(|&a| {
                self.phy = a;
                matches!((self.phy_read(MII_PHYIDR1), self.phy_read(MII_PHYIDR2)),
                         (Some(i1), Some(i2)) if i1 != 0xFFFF && i2 != 0xFFFF)
            })
            .ok_or("no PHY answers on MDIO")?;
        let id = ((self.phy_read(MII_PHYIDR1).unwrap_or(0) as u32) << 16) | self.phy_read(MII_PHYIDR2).unwrap_or(0) as u32;
        nsprint!("[NETSTACK] genet: PHY {:#010x} at MDIO address {}\n", id, self.phy);

        self.phy_write(MII_BMCR, BMCR_RESET).ok_or("PHY reset write")?;
        let mut done = false;
        for _ in 0..500 {
            if self.phy_read(MII_BMCR).ok_or("PHY reset read")? & BMCR_RESET == 0 {
                done = true;
                break;
            }
            delay_ms(1);
        }
        if !done {
            return Err("PHY reset did not complete");
        }

        // rgmii-rxid: the PHY delays RX clock; GTXCLK delay stays off.
        self.phy_write(BRGPHY_AUXCTL, AUXCTL_SHADOW_MISC | (AUXCTL_SHADOW_MISC << AUXCTL_MISC_READ_SHIFT))
            .ok_or("AUXCTL select")?;
        let misc = self.phy_read(BRGPHY_AUXCTL).ok_or("AUXCTL read")? & AUXCTL_MISC_DATA_MASK;
        self.phy_write(BRGPHY_AUXCTL, AUXCTL_MISC_WRITE_EN | AUXCTL_SHADOW_MISC | misc | AUXCTL_MISC_RGMII_SKEW_EN)
            .ok_or("AUXCTL write")?;
        self.phy_write(BRGPHY_SHADOW_1C, SHADOW_1C_CLK_CTRL).ok_or("SHADOW_1C select")?;
        let clk = self.phy_read(BRGPHY_SHADOW_1C).ok_or("SHADOW_1C read")? & SHADOW_1C_DATA_MASK;
        self.phy_write(BRGPHY_SHADOW_1C, SHADOW_1C_WRITE_EN | SHADOW_1C_CLK_CTRL | (clk & !SHADOW_1C_GTXCLK_EN))
            .ok_or("SHADOW_1C write")?;

        let anar = self.phy_read(MII_ANAR).ok_or("ANAR read")?;
        self.phy_write(MII_ANAR, anar | ANAR_10_100_ALL).ok_or("ANAR write")?;
        let gbcr = self.phy_read(MII_GBCR).ok_or("GBCR read")?;
        self.phy_write(MII_GBCR, gbcr | GBCR_1000_ALL).ok_or("GBCR write")?;
        let bmcr = self.phy_read(MII_BMCR).ok_or("BMCR read")?;
        self.phy_write(MII_BMCR, bmcr | BMCR_ANE | BMCR_RESTART_AN).ok_or("BMCR write")?;
        Ok(())
    }

    /// Read the PHY's link state and, on a change, program the MAC to match.
    fn update_link(&mut self) {
        let Some(bmsr) = self.phy_read(MII_BMSR) else { return };
        let up = bmsr & BMSR_LINK != 0 && bmsr & BMSR_ANEG_COMPLETE != 0;
        if up == self.link {
            return;
        }
        self.link = up;
        if !up {
            nsprint!("[NETSTACK] genet: link down\n");
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

        let oob = self.rd(EXT_RGMII_OOB_CTRL);
        // rgmii-rxid is an "ID" mode: leave ID_MODE_DISABLE clear.
        let oob = (oob & !RGMII_OOB_OOB_DISABLE & !RGMII_OOB_ID_MODE_DISABLE) | RGMII_OOB_RGMII_LINK | RGMII_OOB_RGMII_MODE_EN;
        self.wr(EXT_RGMII_OOB_CTRL, oob);
        let mut cmd = self.rd(UMAC_CMD) & !UMAC_CMD_SPEED_MASK;
        cmd |= (speed as u32) << UMAC_CMD_SPEED_SHIFT;
        if full {
            cmd &= !UMAC_CMD_HD_EN;
        } else {
            cmd |= UMAC_CMD_HD_EN;
        }
        self.wr(UMAC_CMD, cmd);
        nsprint!(
            "[NETSTACK] genet: link up, {} Mb/s {}-duplex\n",
            match speed {
                Speed::S10 => 10,
                Speed::S100 => 100,
                Speed::S1000 => 1000,
            },
            if full { "full" } else { "half" }
        );
    }

    /// Poll the PHY at most every LINK_POLL_MS; the engine calls this freely.
    fn poll_link(&mut self) {
        let t = now().total_millis();
        if t >= self.next_link_poll {
            self.next_link_poll = t + LINK_POLL_MS;
            self.update_link();
        }
    }

    fn init_rings(&self, rx_pa: u64) {
        let ring_size = ((DESC_COUNT as u32) << 16) | RX_BUF_LEN;
        let end = (DESC_COUNT * DESC_SIZE / 4 - 1) as u32;

        self.wr(TX_SCB_BURST_SIZE, DMA_BURST);
        for (r, v) in [
            (TX_READ_PTR_LO, 0), (TX_READ_PTR_HI, 0), (TX_CONS_INDEX, 0), (TX_PROD_INDEX, 0),
            (TX_RING_BUF_SIZE, ring_size), (TX_START_ADDR_LO, 0), (TX_START_ADDR_HI, 0),
            (TX_END_ADDR_LO, end), (TX_END_ADDR_HI, 0), (TX_MBUF_DONE_THRES, 1), (TX_FLOW_PERIOD, 0),
            (TX_WRITE_PTR_LO, 0), (TX_WRITE_PTR_HI, 0),
        ] {
            self.wr(r, v);
        }
        self.wr(TX_DMA_RING_CFG, 1 << QID);

        self.wr(RX_SCB_BURST_SIZE, DMA_BURST);
        for (r, v) in [
            (RX_WRITE_PTR_LO, 0), (RX_WRITE_PTR_HI, 0), (RX_PROD_INDEX, 0), (RX_CONS_INDEX, 0),
            (RX_RING_BUF_SIZE, ring_size), (RX_START_ADDR_LO, 0), (RX_START_ADDR_HI, 0),
            (RX_END_ADDR_LO, end), (RX_END_ADDR_HI, 0),
            // An RX-done interrupt per frame.
            (RX_MBUF_DONE_THRES, 1),
            // XON at 5 free, XOFF at DESC_COUNT/16.
            (RX_XON_XOFF_THRES, (5 << 16) | (DESC_COUNT as u32 >> 4)),
            (RX_READ_PTR_LO, 0), (RX_READ_PTR_HI, 0),
        ] {
            self.wr(r, v);
        }
        self.wr(RX_DMA_RING_CFG, 1 << QID);

        // Every RX descriptor owns its buffer for good.
        for i in 0..DESC_COUNT {
            let pa = rx_pa + (i * BUF_SIZE) as u64;
            self.wr(desc(RX_BASE, i) + DESC_ADDR_LO, pa as u32);
            self.wr(desc(RX_BASE, i) + DESC_ADDR_HI, (pa >> 32) as u32);
            self.wr(desc(RX_BASE, i) + DESC_STATUS, 0);
        }
    }

    fn enable(&self) {
        self.wr(TX_DMA_CTRL, self.rd(TX_DMA_CTRL) | DMA_CTRL_EN | DMA_CTRL_RBUF_EN);
        self.wr(RX_DMA_CTRL, self.rd(RX_DMA_CTRL) | DMA_CTRL_EN | DMA_CTRL_RBUF_EN);
        self.wr(UMAC_CMD, self.rd(UMAC_CMD) | UMAC_CMD_TXEN | UMAC_CMD_RXEN);
        // Interrupts stay masked until the engine connects an ISR (enable_interrupts).
    }

    /// The context `irq_ack` takes: the register window.
    pub fn irq_ctx(&self) -> *mut u8 {
        self.regs
    }

    /// Unmask RX-done, once an ISR is connected. TX completion stays masked:
    /// the transmitter needs no restart (unlike the GEM's), and the worker
    /// reclaims TX descriptors whenever it runs.
    pub fn enable_interrupts(&self) {
        self.irq_on.set(true);
        self.wr(INTRL2_CPU_CLEAR, 0xFFFF_FFFF);
        self.wr(INTRL2_CPU_CLEAR_MASK, UMAC_IRQ_RXDMA_DONE);
        nsprint!("[NETSTACK] genet: interrupts on (mask {:#x})\n", self.rd(INTRL2_CPU_MASK_STATUS));
    }

    /// Unmask RX-done again after the worker's pass; `irq_ack` masked it. RX-done
    /// stays asserted while the ring holds frames past its threshold, so acking
    /// it in the ISR alone stormed the CPU until the worker drained the ring:
    /// pings peaked at 148 ms and TCP echoes stalled for good. Linux masks it in
    /// the ISR and unmasks after NAPI for the same reason. True if frames are
    /// already waiting, so the worker should not sleep.
    /// Whether the PHY reports a link (as of the last poll, at most LINK_POLL_MS old).
    pub fn link_up(&self) -> bool {
        self.link
    }

    pub fn rearm_irq(&self) -> bool {
        if self.irq_on.get() {
            self.wr(INTRL2_CPU_CLEAR, UMAC_IRQ_RXDMA_DONE);
            self.wr(INTRL2_CPU_CLEAR_MASK, UMAC_IRQ_RXDMA_DONE);
        }
        self.rx_pending() != 0
    }

    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn rx_pending(&self) -> u16 {
        (self.rd(RX_PROD_INDEX) as u16).wrapping_sub(self.rx_cons)
    }

    pub fn can_recv(&self) -> bool {
        self.rx_pending() != 0
    }

    /// Copy the next frame out and return its descriptor to the hardware.
    /// A bad or fragmentary frame is consumed and reported as None.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        if self.rx_pending() == 0 {
            return None;
        }
        // The producer index is read before the descriptor it covers.
        fence(Ordering::Acquire);
        let idx = self.rx_cons as usize % DESC_COUNT;
        let status = self.rd(desc(RX_BASE, idx) + DESC_STATUS);
        let len = ((status & DESC_BUFLEN_MASK) >> DESC_BUFLEN_SHIFT) as usize;
        let good = status & (DESC_SOP | DESC_EOP | RX_DESC_ERROR) == DESC_SOP | DESC_EOP;
        // The frame follows the 2-byte pad RBUF_ALIGN_2B inserts; the length counts it.
        let out = if good && len > 2 + 14 && len - 2 <= ETH_MAX_FRAME {
            unsafe {
                let buf = self.rx_va.add(idx * BUF_SIZE);
                dma_invalidate(buf, len);
                Some(core::slice::from_raw_parts(buf.add(2), len - 2).to_vec())
            }
        } else {
            None
        };
        self.rx_cons = self.rx_cons.wrapping_add(1);
        self.wr(RX_CONS_INDEX, self.rx_cons as u32);
        out
    }

    pub fn can_send(&mut self) -> bool {
        if !self.link {
            self.poll_link();
            return false;
        }
        let in_flight = self.tx_prod.wrapping_sub(self.rd(TX_CONS_INDEX) as u16);
        (in_flight as usize) < DESC_COUNT - 1
    }

    pub fn send(&mut self, frame: &[u8]) {
        if frame.len() > ETH_MAX_FRAME {
            return;
        }
        let idx = self.tx_prod as usize % DESC_COUNT;
        unsafe {
            let buf = self.tx_va.add(idx * BUF_SIZE);
            core::ptr::copy_nonoverlapping(frame.as_ptr(), buf, frame.len());
            dma_clean(buf, frame.len());
        }
        let pa = self.tx_pa + (idx * BUF_SIZE) as u64;
        self.wr(desc(TX_BASE, idx) + DESC_ADDR_LO, pa as u32);
        self.wr(desc(TX_BASE, idx) + DESC_ADDR_HI, (pa >> 32) as u32);
        self.wr(
            desc(TX_BASE, idx) + DESC_STATUS,
            DESC_SOP | DESC_EOP | TX_DESC_CRC | TX_DESC_QTAG | ((frame.len() as u32) << DESC_BUFLEN_SHIFT),
        );
        self.tx_prod = self.tx_prod.wrapping_add(1);
        self.wr(TX_PROD_INDEX, self.tx_prod as u32);
    }
}

/// Take over the GENET the PnP manager started. None, with a serial line naming
/// the failed step, if anything about the hardware is not as expected.
pub fn bringup(nic: PlatformNic) -> Option<Genet> {
    unsafe {
        let regs = MmMapIoSpace(phys_li(nic.mmio), nic.mmio_len.max(0x10000) as _, MM_NON_CACHED as _) as *mut u8;
        if regs.is_null() {
            nsprint!("[NETSTACK] genet: MmMapIoSpace({:#x}) failed\n", nic.mmio);
            return None;
        }
        let mut g = Genet {
            regs,
            mac: [0; 6],
            phy: 0,
            link: false,
            next_link_poll: 0,
            rx_va: core::ptr::null_mut(),
            rx_cons: 0,
            tx_va: core::ptr::null_mut(),
            tx_pa: 0,
            tx_prod: 0,
            irq_on: core::cell::Cell::new(false),
        };
        let rev = g.rd(SYS_REV_CTRL);
        let major = (rev >> 24) & 0xf;
        // GENET v5 reports major 6 in this field (v4 reports 5).
        if major != 6 {
            nsprint!("[NETSTACK] genet: unexpected SYS_REV_CTRL {:#010x} (want GENET v5)\n", rev);
            return None;
        }

        // The station address, before the reset.
        let m0 = g.rd(UMAC_MAC0).to_be_bytes();
        let m1 = g.rd(UMAC_MAC1).to_be_bytes();
        g.mac = [m0[0], m0[1], m0[2], m0[3], m1[2], m1[3]];
        if g.mac == [0; 6] || g.mac[0] & 1 != 0 {
            nsprint!(
                "[NETSTACK] genet: no usable MAC address in UMAC_MAC0/1 ({:02x?}); the firmware sets it\n",
                g.mac
            );
            return None;
        }

        g.quiesce();
        g.reset();
        if let Err(step) = g.phy_init() {
            nsprint!("[NETSTACK] genet: PHY init failed: {}\n", step);
            return None;
        }
        g.set_mac();

        let bytes = DESC_COUNT * BUF_SIZE;
        let Some((rx_va, rx_pa)) = dma_arena(bytes) else {
            nsprint!("[NETSTACK] genet: RX buffer allocation failed\n");
            return None;
        };
        let Some((tx_va, tx_pa)) = dma_arena(bytes) else {
            nsprint!("[NETSTACK] genet: TX buffer allocation failed\n");
            return None;
        };
        g.rx_va = rx_va;
        g.tx_va = tx_va;
        g.tx_pa = tx_pa;
        g.init_rings(rx_pa);
        g.enable();

        // Auto-negotiation takes a few seconds; DHCP would only retry meanwhile.
        let deadline = now().total_millis() + LINK_WAIT_MS;
        while !g.link && now().total_millis() < deadline {
            g.update_link();
            if !g.link {
                delay_ms(100);
            }
        }
        let m = g.mac;
        nsprint!(
            "[NETSTACK] genet up: mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} rev={:#010x} regs={:#x} link={}\n",
            m[0], m[1], m[2], m[3], m[4], m[5], rev, nic.mmio, if g.link { "up" } else { "down (still negotiating)" }
        );
        Some(g)
    }
}

// --------------------------------------------------------- smoltcp adapter
/// The ISR's half (DIRQL): read and clear INTRL2_0's unmasked status. Touches
/// nothing the worker owns.
pub unsafe fn irq_ack(regs: PVOID) -> crate::isr::Ack {
    let r = regs.cast::<u8>();
    let rd = |off: usize| core::ptr::read_volatile(r.add(off).cast::<u32>());
    let st = rd(INTRL2_CPU_STAT) & !rd(INTRL2_CPU_MASK_STATUS);
    if st == 0 {
        return crate::isr::Ack::NotOurs;
    }
    if st & UMAC_IRQ_RXDMA_DONE != 0 {
        // Masked until the worker has drained the ring (Genet::rearm_irq).
        core::ptr::write_volatile(r.add(INTRL2_CPU_SET_MASK).cast::<u32>(), UMAC_IRQ_RXDMA_DONE);
    }
    core::ptr::write_volatile(r.add(INTRL2_CPU_CLEAR).cast::<u32>(), st);
    if st & UMAC_IRQ_RXDMA_DONE != 0 { crate::isr::Ack::Wake } else { crate::isr::Ack::Handled }
}

pub struct GenetPhy {
    dev: Genet,
}

impl GenetPhy {
    pub fn new(dev: Genet) -> Self {
        Self { dev }
    }
    pub fn mac(&self) -> [u8; 6] {
        self.dev.mac_address()
    }
    pub fn dev(&self) -> &Genet {
        &self.dev
    }
}

pub struct GenetRxToken {
    data: Vec<u8>,
}

impl phy::RxToken for GenetRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }
}

pub struct GenetTxToken<'a> {
    dev: &'a mut Genet,
}

impl<'a> phy::TxToken for GenetTxToken<'a> {
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

impl Device for GenetPhy {
    type RxToken<'a> = GenetRxToken;
    type TxToken<'a> = GenetTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.dev.poll_link();
        if !self.dev.can_recv() {
            return None;
        }
        let data = self.dev.receive()?;
        Some((GenetRxToken { data }, GenetTxToken { dev: &mut self.dev }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if !self.dev.can_send() {
            return None;
        }
        Some(GenetTxToken { dev: &mut self.dev })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
