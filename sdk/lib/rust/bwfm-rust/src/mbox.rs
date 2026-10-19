//! The Raspberry Pi 4's VideoCore mailbox, property channel: what the Pi 4 keeps behind
//! its firmware and the Wi-Fi needs. Its WL_REG_ON is not a SoC pin but line 1 of the
//! firmware's GPIO expander, and its Arasan SDHCI reports no base clock (the EMMC
//! clock, which only the firmware knows). UEFI's RpiFirmwareDxe asks the same way.
//!
//! One request at a time, polled: bwfm's bring-up is the only caller, at PASSIVE.

use ntkrnl_sys::ddk::{
    MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmUnmapIoSpace,
    _MEMORY_CACHING_TYPE_MmNonCached, PHYSICAL_ADDRESS,
};

use crate::driver::{map, Kernel};

/// The ARM's mailbox 0 (read) and 1 (write), in the Pi 4's low-peripheral map.
const MBOX: u64 = 0xFE00_B880;
const MBOX_READ: usize = 0x00;
const MBOX_STATUS: usize = 0x18;
const MBOX_WRITE: usize = 0x20;
const FULL: u32 = 1 << 31;
const EMPTY: u32 = 1 << 30;
const CHANNEL_PROPERTY: u32 = 8;
/// The VideoCore sees ARM RAM below 1 GB at this bus alias (uncached).
const BUS_ALIAS: u32 = 0xC000_0000;
const RESPONSE_OK: u32 = 0x8000_0000;
const WAIT_US: u64 = 500_000;

pub const TAG_GET_CLOCK_RATE: u32 = 0x0003_0002;
pub const TAG_SET_GPIO_STATE: u32 = 0x0003_8041;
/// The clock the Arasan SDHCI runs from.
pub const CLOCK_EMMC: u32 = 1;
/// The firmware's GPIO expander starts at 128; WL_ON is its line 1.
pub const GPIO_WL_ON: u32 = 128 + 1;

/// One property tag with up to two words in and out. The answer's words, or None
/// (logged by the caller) when the mailbox or the firmware refused.
pub unsafe fn property(tag: u32, args: [u32; 2]) -> Option<[u32; 2]> {
    let regs = map(MBOX, 0x40) as *mut u32;
    if regs.is_null() {
        return None;
    }
    let rd = |off: usize| core::ptr::read_volatile(regs.add(off / 4));
    let mut low: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
    let none: PHYSICAL_ADDRESS = core::mem::zeroed();
    low.QuadPart = 0;
    high.QuadPart = 0x3FFF_FFFF;
    let buf = MmAllocateContiguousMemorySpecifyCache(64, low, high, none, _MEMORY_CACHING_TYPE_MmNonCached) as *mut u32;
    if buf.is_null() {
        MmUnmapIoSpace(regs.cast(), 0x40);
        return None;
    }
    // Size, request code, then the tag: its id, value buffer size, request size, values.
    let msg = [32u32, 0, tag, 8, 8, args[0], args[1], 0];
    for (i, w) in msg.iter().enumerate() {
        core::ptr::write_volatile(buf.add(i), *w);
    }
    let pa = MmGetPhysicalAddress(buf.cast()).QuadPart as u32;
    let word = (pa | BUS_ALIAS) & !0xF | CHANNEL_PROPERTY;
    let start = Kernel::now_us_static();
    let late = || Kernel::now_us_static().wrapping_sub(start) > WAIT_US;
    let mut answer = None;
    while rd(MBOX_STATUS) & FULL != 0 && !late() {}
    if !late() {
        core::ptr::write_volatile(regs.add(MBOX_WRITE / 4), word);
        loop {
            if late() {
                break;
            }
            if rd(MBOX_STATUS) & EMPTY != 0 {
                continue;
            }
            if rd(MBOX_READ) == word {
                if core::ptr::read_volatile(buf.add(1)) == RESPONSE_OK {
                    answer = Some([core::ptr::read_volatile(buf.add(5)), core::ptr::read_volatile(buf.add(6))]);
                }
                break;
            }
        }
    }
    MmFreeContiguousMemorySpecifyCache(buf.cast(), 64, _MEMORY_CACHING_TYPE_MmNonCached);
    MmUnmapIoSpace(regs.cast(), 0x40);
    answer
}
