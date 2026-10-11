//! Intel's chipset-integrated ("PCH") gigabit MACs for the e1000 backend: the
//! I217 (Lynx Point, the OptiPlex 9020's I217-LM) and I218. They keep the 8254x
//! register file and legacy descriptor rings `e1000.rs` programs, and differ in
//! the bring-up `e1000::bringup` calls into this module for:
//!
//! - Reset is a CTRL write through the register BAR (no I/O BAR is required),
//!   taken with the software semaphore, and resets the PHY too.
//! - The PHY sits at MDIO address 2, and firmware may leave it in SMBus mode,
//!   where MDIO cannot reach it until the MAC drops its SMBus override and the
//!   PHY's LANPHYPC power-cycle is toggled.
//! - There is no EEPROM read register: the NVM lives in the SPI flash, and the
//!   hardware loads the MAC address from it into receive address 0 at reset.
//! - A few register bits the datasheet requires set after reset.
//!
//! The sequences follow Linux's e1000e (ich8lan.c) for pch_lpt.

use crate::serial::nsprint;

/// Device IDs handled here: I217-LM/V (Lynx Point) and I218-LM/V (Lynx Point
/// LP and Wildcat Point). The I219 (Sunrise Point on) moves the NVM into the
/// register BAR and needs a descriptor-ring flush on reset; not handled.
const PCH_DEVICES: [u16; 8] = [0x153A, 0x153B, 0x155A, 0x1559, 0x15A0, 0x15A1, 0x15A2, 0x15A3];

pub fn is_pch(device_id: u16) -> bool {
    PCH_DEVICES.contains(&device_id)
}

// Registers and bits (Intel's I217 datasheet; e1000e's names).
const REG_CTRL: u32 = 0x0000;
const REG_STATUS: u32 = 0x0008;
const REG_MDIC: u32 = 0x0020;
const REG_ICR: u32 = 0x00C0;
const REG_IMC: u32 = 0x00D8;
const REG_RCTL: u32 = 0x0100;
const REG_TCTL: u32 = 0x0400;
const CTRL_LRST: u32 = 1 << 3;
const CTRL_ASDE: u32 = 1 << 5;
const CTRL_SLU: u32 = 1 << 6;
const CTRL_RST: u32 = 1 << 26;
const CTRL_VME: u32 = 1 << 30;
const MDIC_REGADD_SHIFT: u32 = 16;
const MDIC_PHYADD_SHIFT: u32 = 21;
const MDIC_OP_READ: u32 = 2 << 26;
const MDIC_R: u32 = 1 << 28;
const MDIC_E: u32 = 1 << 30;
const REG_CTRL_EXT: u32 = 0x0018;
const REG_FWSM: u32 = 0x5B54;
const REG_EXTCNF_CTRL: u32 = 0x0F00;
const REG_TXDCTL0: u32 = 0x3828;
const REG_TARC0: u32 = 0x3840;
const CTRL_LANPHYPC_OVERRIDE: u32 = 1 << 16;
const CTRL_LANPHYPC_VALUE: u32 = 1 << 17;
const CTRL_FRCSPD: u32 = 1 << 11;
const CTRL_FRCDPX: u32 = 1 << 12;
const CTRL_PHY_RST: u32 = 1 << 31;
const CTRL_EXT_LPCD: u32 = 1 << 2;
const CTRL_EXT_FORCE_SMBUS: u32 = 1 << 11;
const CTRL_EXT_PHYPDEN: u32 = 1 << 20;
const STATUS_LU: u32 = 1 << 1;
const STATUS_LAN_INIT_DONE: u32 = 1 << 9;
const STATUS_PHYRA: u32 = 1 << 10;
const EXTCNF_CTRL_SWFLAG: u32 = 1 << 5;
const FWSM_RSPCIPHY: u32 = 1 << 6;
const MDIC_OP_WRITE: u32 = 1 << 26;

// PHY registers. Page select (31) takes page << 5 on these PHYs.
const PHY_ADDR: u32 = 2;
const PHY_BMCR: u32 = 0;
const PHY_BMSR: u32 = 1;
const PHY_ID1: u32 = 2;
const PHY_ID2: u32 = 3;
const PHY_PAGE_SELECT: u32 = 31;
const BMCR_RESTART_AN: u16 = 1 << 9;
const BMCR_POWER_DOWN: u16 = 1 << 11;
const BMCR_AN_ENABLE: u16 = 1 << 12;
/// CV_SMB_CTRL, page 769 register 23: bit 0 forces the PHY into SMBus mode.
const CV_SMB_PAGE: u32 = 769;
const CV_SMB_CTRL: u32 = 23;
const CV_SMB_FORCE_SMBUS: u16 = 1 << 0;

/// The device's register window (BAR0, mapped uncached).
type Regs = *mut u32;

unsafe fn rd(a: Regs, reg: u32) -> u32 {
    core::ptr::read_volatile(a.add(reg as usize / 4))
}

unsafe fn wr(a: Regs, reg: u32, v: u32) {
    core::ptr::write_volatile(a.add(reg as usize / 4), v)
}

/// Busy-wait `us` microseconds (KeStallExecutionProcessor).
unsafe fn stall(us: u32) {
    ntkrnl_sys::ddk::KeStallExecutionProcessor(us);
}

/// Posted writes reach the device before a delay is timed from them.
unsafe fn flush(a: Regs) {
    rd(a, REG_STATUS);
}

unsafe fn msleep(ms: u32) {
    for _ in 0..ms {
        stall(1000);
    }
}

/// The software/firmware semaphore over the PHY and NVM (EXTCNF_CTRL.SWFLAG):
/// set it and wait until it reads back set, up to 100 ms.
unsafe fn acquire_swflag(a: Regs) -> bool {
    for _ in 0..100 {
        let v = rd(a, REG_EXTCNF_CTRL);
        if v & EXTCNF_CTRL_SWFLAG == 0 {
            wr(a, REG_EXTCNF_CTRL, v | EXTCNF_CTRL_SWFLAG);
        }
        if rd(a, REG_EXTCNF_CTRL) & EXTCNF_CTRL_SWFLAG != 0 {
            return true;
        }
        msleep(1);
    }
    nsprint!("[NETSTACK] e1000 PCH: software semaphore not granted (EXTCNF_CTRL {:#x})\n", rd(a, REG_EXTCNF_CTRL));
    false
}

unsafe fn release_swflag(a: Regs) {
    wr(a, REG_EXTCNF_CTRL, rd(a, REG_EXTCNF_CTRL) & !EXTCNF_CTRL_SWFLAG);
}

unsafe fn mdic(a: Regs, cmd: u32) -> Option<u16> {
    wr(a, REG_MDIC, cmd | (PHY_ADDR << MDIC_PHYADD_SHIFT));
    for _ in 0..640 {
        stall(50);
        let v = rd(a, REG_MDIC);
        if v & MDIC_R != 0 {
            // Back-to-back MDIC transactions can return the previous data.
            stall(100);
            return if v & MDIC_E != 0 { None } else { Some(v as u16) };
        }
    }
    None
}

unsafe fn phy_read(a: Regs, reg: u32) -> Option<u16> {
    mdic(a, (reg << MDIC_REGADD_SHIFT) | MDIC_OP_READ)
}

unsafe fn phy_write(a: Regs, reg: u32, v: u16) -> bool {
    mdic(a, (reg << MDIC_REGADD_SHIFT) | MDIC_OP_WRITE | v as u32).is_some()
}

/// The PHY answers on MDIO with a plausible ID.
unsafe fn phy_accessible(a: Regs) -> bool {
    let id1 = phy_read(a, PHY_ID1);
    let id2 = phy_read(a, PHY_ID2);
    matches!((id1, id2), (Some(h), Some(l)) if h != 0 && h != 0xFFFF && l != 0xFFFF)
}

/// Power-cycle the PHY through the LANPHYPC pin, which also takes it out of
/// SMBus mode (e1000_toggle_lanphypc_pch_lpt).
unsafe fn toggle_lanphypc(a: Regs) {
    let mut ctrl = rd(a, REG_CTRL);
    ctrl |= CTRL_LANPHYPC_OVERRIDE;
    ctrl &= !CTRL_LANPHYPC_VALUE;
    wr(a, REG_CTRL, ctrl);
    flush(a);
    stall(10);
    ctrl &= !CTRL_LANPHYPC_OVERRIDE;
    wr(a, REG_CTRL, ctrl);
    flush(a);
    for _ in 0..20 {
        if rd(a, REG_CTRL_EXT) & CTRL_EXT_LPCD != 0 {
            break;
        }
        msleep(5);
    }
    msleep(30);
}

/// Make the PHY reachable over MDIO, then take MAC and PHY out of SMBus mode
/// (e1000_init_phy_workarounds_pchlan for pch_lpt). Caller holds the semaphore.
unsafe fn wake_phy(a: Regs) -> bool {
    let mut ok = phy_accessible(a);
    if !ok {
        // Forcing the MAC to SMBus first lets an SMBus-mode PHY answer.
        wr(a, REG_CTRL_EXT, rd(a, REG_CTRL_EXT) | CTRL_EXT_FORCE_SMBUS);
        msleep(50);
        ok = phy_accessible(a);
    }
    if !ok {
        if rd(a, REG_FWSM) & FWSM_RSPCIPHY == 0 {
            nsprint!("[NETSTACK] e1000 PCH: PHY unreachable and its reset is blocked by firmware\n");
            return false;
        }
        nsprint!("[NETSTACK] e1000 PCH: PHY unreachable, toggling LANPHYPC\n");
        toggle_lanphypc(a);
        ok = phy_accessible(a);
        if !ok {
            wr(a, REG_CTRL_EXT, rd(a, REG_CTRL_EXT) & !CTRL_EXT_FORCE_SMBUS);
            ok = phy_accessible(a);
        }
    }
    if !ok {
        nsprint!("[NETSTACK] e1000 PCH: PHY does not answer on MDIO\n");
        return false;
    }
    // Unforce SMBus mode in the PHY, then in the MAC.
    if phy_write(a, PHY_PAGE_SELECT, (CV_SMB_PAGE << 5) as u16) {
        if let Some(v) = phy_read(a, CV_SMB_CTRL) {
            phy_write(a, CV_SMB_CTRL, v & !CV_SMB_FORCE_SMBUS);
        }
        phy_write(a, PHY_PAGE_SELECT, 0);
    }
    wr(a, REG_CTRL_EXT, rd(a, REG_CTRL_EXT) & !CTRL_EXT_FORCE_SMBUS);
    true
}

/// Power the PHY up and restart autonegotiation. Caller holds the semaphore.
unsafe fn start_autoneg(a: Regs) {
    let id = (phy_read(a, PHY_ID1).unwrap_or(0) as u32) << 16 | phy_read(a, PHY_ID2).unwrap_or(0) as u32;
    match phy_read(a, PHY_BMCR) {
        Some(bmcr) => {
            let new = (bmcr & !BMCR_POWER_DOWN) | BMCR_AN_ENABLE | BMCR_RESTART_AN;
            phy_write(a, PHY_BMCR, new);
            nsprint!("[NETSTACK] e1000 PCH: PHY id {:08x}, BMCR {:04x} -> {:04x} (autonegotiating)\n", id, bmcr as u32, new as u32);
        }
        None => nsprint!("[NETSTACK] e1000 PCH: PHY BMCR read failed\n"),
    }
}

/// The PCH reset (e1000_reset_hw_ich8lan): quiesce, reset MAC and PHY under the
/// semaphore (the reset releases it), wait for the hardware's own NVM-driven
/// configuration, then wake the PHY and start the link. False if the device
/// never came out of reset.
pub unsafe fn reset(a: Regs) -> bool {
    wr(a, REG_IMC, 0xFFFF_FFFF);
    wr(a, REG_RCTL, 0);
    wr(a, REG_TCTL, 0);
    flush(a);
    msleep(10);

    let mut ctrl = rd(a, REG_CTRL);
    // Firmware may block resetting the PHY; the MAC reset still runs.
    if rd(a, REG_FWSM) & FWSM_RSPCIPHY != 0 {
        ctrl |= CTRL_PHY_RST;
    }
    acquire_swflag(a);
    // No flush after this write: reading a register during the reset hangs it.
    wr(a, REG_CTRL, ctrl | CTRL_RST);
    msleep(20);

    let mut back = false;
    for _ in 0..100 {
        if rd(a, REG_CTRL) & CTRL_RST == 0 {
            back = true;
            break;
        }
        msleep(1);
    }
    if !back {
        nsprint!("[NETSTACK] e1000 PCH: device did not come out of reset\n");
        return false;
    }

    // Basic configuration from the NVM is done when LAN_INIT_DONE sets (up to 150 ms).
    let mut init = false;
    for _ in 0..150 {
        if rd(a, REG_STATUS) & STATUS_LAN_INIT_DONE != 0 {
            init = true;
            break;
        }
        msleep(1);
    }
    let status = rd(a, REG_STATUS);
    wr(a, REG_STATUS, status & !(STATUS_LAN_INIT_DONE | STATUS_PHYRA));
    if !init {
        nsprint!("[NETSTACK] e1000 PCH: LAN_INIT_DONE never set (STATUS {:#x}); continuing\n", status);
    }

    wr(a, REG_IMC, 0xFFFF_FFFF);
    rd(a, REG_ICR);

    // Bits the datasheet requires after reset (e1000_initialize_hw_bits_ich8lan).
    wr(a, REG_CTRL_EXT, rd(a, REG_CTRL_EXT) | (1 << 22) | CTRL_EXT_PHYPDEN);
    wr(a, REG_TXDCTL0, rd(a, REG_TXDCTL0) | (1 << 22));
    wr(a, REG_TARC0, rd(a, REG_TARC0) | (1 << 23) | (1 << 24) | (1 << 26) | (1 << 27));

    // The MAC takes speed and duplex from the PHY.
    let mut ctrl = rd(a, REG_CTRL);
    ctrl &= !(CTRL_LRST | CTRL_VME | CTRL_FRCSPD | CTRL_FRCDPX);
    ctrl |= CTRL_ASDE | CTRL_SLU;
    wr(a, REG_CTRL, ctrl);

    if acquire_swflag(a) {
        if wake_phy(a) {
            start_autoneg(a);
        }
        release_swflag(a);
    }
    // Autonegotiation takes a few seconds; DHCP retries if it takes longer, so this
    // only waits long enough to report the outcome.
    let mut up = false;
    for _ in 0..50 {
        if rd(a, REG_STATUS) & STATUS_LU != 0 {
            up = true;
            break;
        }
        msleep(100);
    }
    let bmsr = if acquire_swflag(a) {
        let v = phy_read(a, PHY_BMSR);
        release_swflag(a);
        v
    } else {
        None
    };
    nsprint!(
        "[NETSTACK] e1000 PCH: link {} (STATUS {:#x}, PHY BMSR {:#x})\n",
        if up { "up" } else { "down after 5 s" },
        rd(a, REG_STATUS),
        bmsr.unwrap_or(0xFFFF)
    );
    nsprint!(
        "[NETSTACK] e1000 PCH: reset done (STATUS {:#x}, CTRL {:#x}, CTRL_EXT {:#x})\n",
        rd(a, REG_STATUS),
        rd(a, REG_CTRL),
        rd(a, REG_CTRL_EXT)
    );
    true
}
