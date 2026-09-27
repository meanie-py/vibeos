//! xHCI hardware definitions (eXtensible Host Controller Interface 1.2), plus the
//! three primitives every producer/consumer path goes through: MMIO access, memory
//! ordering, and cache maintenance for non-coherent DMA.
//!
//! None of this is in an SDK header, so it lives here rather than in the bindgen
//! island. Section numbers refer to the xHCI 1.2 specification.

// ---- Capability registers (5.3), offsets from BAR0 ----
pub const CAPLENGTH: usize = 0x00; // u8; HCIVERSION is the u16 at 0x02
pub const HCSPARAMS1: usize = 0x04;
pub const HCSPARAMS2: usize = 0x08;
pub const HCCPARAMS1: usize = 0x10;
pub const DBOFF: usize = 0x14;
pub const RTSOFF: usize = 0x18;

pub const HCCPARAMS1_AC64: u32 = 1 << 0;
pub const HCCPARAMS1_CSZ: u32 = 1 << 2;
pub const HCCPARAMS1_PPC: u32 = 1 << 3;

// ---- Operational registers (5.4), offsets from BAR0 + CAPLENGTH ----
pub const USBCMD: usize = 0x00;
pub const USBSTS: usize = 0x04;
pub const PAGESIZE: usize = 0x08;
pub const CRCR: usize = 0x18; // 64-bit
pub const DCBAAP: usize = 0x30; // 64-bit
pub const CONFIG: usize = 0x38;
pub const PORTSC_BASE: usize = 0x400; // + 0x10 * (port - 1)

pub const USBCMD_RS: u32 = 1 << 0;
pub const USBCMD_HCRST: u32 = 1 << 1;
pub const USBCMD_INTE: u32 = 1 << 2;
pub const USBCMD_HSEE: u32 = 1 << 3;

pub const USBSTS_HCH: u32 = 1 << 0;
pub const USBSTS_HSE: u32 = 1 << 2;
pub const USBSTS_EINT: u32 = 1 << 3;
pub const USBSTS_PCD: u32 = 1 << 4;
pub const USBSTS_CNR: u32 = 1 << 11;
pub const USBSTS_HCE: u32 = 1 << 12;

// PORTSC (5.4.8)
pub const PORTSC_CCS: u32 = 1 << 0;
pub const PORTSC_PED: u32 = 1 << 1; // RW1CS: writing 1 DISABLES the port
pub const PORTSC_OCA: u32 = 1 << 3;
pub const PORTSC_PR: u32 = 1 << 4;
pub const PORTSC_PLS_SHIFT: u32 = 5;
pub const PORTSC_PLS_MASK: u32 = 0xF << 5;
pub const PORTSC_PP: u32 = 1 << 9;
pub const PORTSC_SPEED_SHIFT: u32 = 10;
pub const PORTSC_SPEED_MASK: u32 = 0xF << 10;
pub const PORTSC_LWS: u32 = 1 << 16;
pub const PORTSC_CSC: u32 = 1 << 17;
pub const PORTSC_PEC: u32 = 1 << 18;
pub const PORTSC_WRC: u32 = 1 << 19;
pub const PORTSC_OCC: u32 = 1 << 20;
pub const PORTSC_PRC: u32 = 1 << 21;
pub const PORTSC_PLC: u32 = 1 << 22;
pub const PORTSC_CEC: u32 = 1 << 23;
pub const PORTSC_CHANGE_BITS: u32 =
    PORTSC_CSC | PORTSC_PEC | PORTSC_WRC | PORTSC_OCC | PORTSC_PRC | PORTSC_PLC | PORTSC_CEC;
/// The bits that read back as state and may be written back unchanged: read-only
/// status plus the RWS fields (PLS, PP, PIC, wake enables). Everything else in
/// PORTSC is RW1C (change bits), RW1CS (PED) or a command (PR, WPR, LWS), so a
/// read-modify-write must start from `value & PORTSC_PRESERVE` or it will clear
/// changes, disable the port, or reset it by accident.
pub const PORTSC_PRESERVE: u32 = 0x4F00_FFE9 & !PORTSC_PED & !PORTSC_PR;

/// PORTSC.Speed with the default Protocol Speed ID mapping (7.2.2.1.1).
pub const SPEED_FULL: u32 = 1;
pub const SPEED_LOW: u32 = 2;
pub const SPEED_HIGH: u32 = 3;
pub const SPEED_SUPER: u32 = 4;

// ---- Runtime registers (5.5), offsets from BAR0 + RTSOFF ----
pub const MFINDEX: usize = 0x00;
pub const IR0: usize = 0x20; // interrupter 0 register set
pub const IMAN: usize = 0x00;
pub const IMOD: usize = 0x04;
pub const ERSTSZ: usize = 0x08;
pub const ERSTBA: usize = 0x10; // 64-bit
pub const ERDP: usize = 0x18; // 64-bit

pub const IMAN_IP: u32 = 1 << 0;
pub const IMAN_IE: u32 = 1 << 1;
pub const ERDP_EHB: u32 = 1 << 3;

// ---- Extended capabilities (7) ----
pub const XECP_LEGACY: u32 = 1;
pub const XECP_PROTOCOL: u32 = 2;
pub const LEGACY_BIOS_OWNED: u32 = 1 << 16;
pub const LEGACY_OS_OWNED: u32 = 1 << 24;

// ---- TRBs (6.4) ----
pub const TRB_SIZE: usize = 16;
pub const TRB_CYCLE: u32 = 1 << 0;
pub const TRB_TC: u32 = 1 << 1; // Link: toggle cycle
pub const TRB_ENT: u32 = 1 << 1; // Evaluate Next TRB
pub const TRB_ISP: u32 = 1 << 2; // Interrupt on Short Packet
pub const TRB_CH: u32 = 1 << 4; // Chain
pub const TRB_IOC: u32 = 1 << 5; // Interrupt On Completion
pub const TRB_IDT: u32 = 1 << 6; // Immediate Data
pub const TRB_BSR: u32 = 1 << 9; // Address Device: Block Set Address Request
pub const TRB_DIR_IN: u32 = 1 << 16; // Data / Status stage direction
pub const TRB_TYPE_SHIFT: u32 = 10;

pub const TRB_NORMAL: u32 = 1;
pub const TRB_SETUP: u32 = 2;
pub const TRB_DATA: u32 = 3;
pub const TRB_STATUS: u32 = 4;
pub const TRB_LINK: u32 = 6;
pub const TRB_ENABLE_SLOT: u32 = 9;
pub const TRB_DISABLE_SLOT: u32 = 10;
pub const TRB_ADDRESS_DEVICE: u32 = 11;
pub const TRB_CONFIGURE_ENDPOINT: u32 = 12;
pub const TRB_EVALUATE_CONTEXT: u32 = 13;
pub const TRB_RESET_ENDPOINT: u32 = 14;
pub const TRB_STOP_ENDPOINT: u32 = 15;
pub const TRB_SET_TR_DEQUEUE: u32 = 16;
pub const TRB_NOOP_COMMAND: u32 = 23;
pub const TRB_EV_TRANSFER: u32 = 32;
pub const TRB_EV_COMMAND_COMPLETION: u32 = 33;
pub const TRB_EV_PORT_STATUS_CHANGE: u32 = 34;
pub const TRB_EV_HOST_CONTROLLER: u32 = 37;

/// Setup stage Transfer Type (TRT), control bits 17:16.
pub const TRT_NO_DATA: u32 = 0 << 16;
pub const TRT_OUT: u32 = 2 << 16;
pub const TRT_IN: u32 = 3 << 16;

// Completion codes (6.4.5)
pub const CC_SUCCESS: u32 = 1;
pub const CC_DATA_BUFFER: u32 = 2;
pub const CC_BABBLE: u32 = 3;
pub const CC_USB_TRANSACTION: u32 = 4;
pub const CC_TRB: u32 = 5;
pub const CC_STALL: u32 = 6;
pub const CC_SHORT_PACKET: u32 = 13;
pub const CC_CONTEXT_STATE: u32 = 19;
pub const CC_STOPPED: u32 = 26;
pub const CC_STOPPED_LENGTH_INVALID: u32 = 27;
pub const CC_STOPPED_SHORT_PACKET: u32 = 28;
pub const CC_SPLIT_TRANSACTION: u32 = 36;

#[inline]
pub const fn trb_type(control: u32) -> u32 {
    (control >> TRB_TYPE_SHIFT) & 0x3F
}

// ---- Contexts (6.2) ----
// Endpoint context types (6.2.3, EP Type)
pub const EP_ISOCH_OUT: u32 = 1;
pub const EP_BULK_OUT: u32 = 2;
pub const EP_INT_OUT: u32 = 3;
pub const EP_CONTROL: u32 = 4;
pub const EP_ISOCH_IN: u32 = 5;
pub const EP_BULK_IN: u32 = 6;
pub const EP_INT_IN: u32 = 7;

/// Device Context Index for an endpoint address: EP0 is 1, then OUT n is 2n and
/// IN n is 2n+1.
#[inline]
pub const fn dci(endpoint_address: u32) -> u32 {
    let num = endpoint_address & 0x0F;
    if num == 0 {
        1
    } else {
        num * 2 + ((endpoint_address >> 7) & 1)
    }
}

// ---- MMIO ----
#[inline]
pub unsafe fn rd32(base: *mut u8, off: usize) -> u32 {
    core::ptr::read_volatile(base.add(off) as *const u32)
}
#[inline]
pub unsafe fn wr32(base: *mut u8, off: usize, v: u32) {
    core::ptr::write_volatile(base.add(off) as *mut u32, v)
}
/// 64-bit registers are written low dword first, then high (4.22.2 permits 32-bit
/// access on controllers without 64-bit MMIO, and every controller accepts it).
#[inline]
pub unsafe fn wr64(base: *mut u8, off: usize, v: u64) {
    wr32(base, off, v as u32);
    wr32(base, off + 4, (v >> 32) as u32);
}

// ---- Ordering ----
// The controller and the CPU share rings in ordinary memory and signal each other
// through MMIO (doorbells, ERDP). On a weakly ordered CPU (the Pi's Cortex-A72) the
// memory writes must be visible before the doorbell, and a TRB's body must be
// visible before the word carrying its cycle bit. QEMU executes both orders
// correctly no matter what, so these are the calls that only real hardware tests.

/// Make prior normal-memory writes visible to the device before later writes
/// (TRB body before its cycle bit; ring contents before a doorbell).
#[inline]
pub fn wmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb st", options(nostack, preserves_flags))
    }
    #[cfg(not(target_arch = "aarch64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

/// Order a read of the cycle-bit word before the reads of the rest of the TRB.
#[inline]
pub fn rmb() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("dsb ld", options(nostack, preserves_flags))
    }
    #[cfg(not(target_arch = "aarch64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

// ---- Cache maintenance for non-coherent DMA ----
// On the Pi the VL805 does not snoop CPU caches (ACPI _CCA 0), and usbport's common
// buffers are cached mappings. Everything the controller reads is cleaned to the
// point of coherency after the CPU writes it, and everything it writes is
// invalidated before the CPU reads it. On x86 and on QEMU DMA is coherent and
// these are no-ops in effect.

#[cfg(target_arch = "aarch64")]
const DCACHE_LINE: usize = 64;

/// Clean (write back) [va, va+len) so the device sees what the CPU wrote.
#[inline]
pub unsafe fn dma_clean(va: *const u8, len: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        let mut p = (va as usize) & !(DCACHE_LINE - 1);
        let end = va as usize + len;
        while p < end {
            core::arch::asm!("dc cvac, {0}", in(reg) p, options(nostack, preserves_flags));
            p += DCACHE_LINE;
        }
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (va, len);
    }
}

/// Invalidate device-owned, cache-line-isolated memory before reading it.
/// The caller must clean it BEFORE handing it to hardware and must not write it
/// until ownership returns. Cleaning here could overwrite newly DMA'd bytes.
#[inline]
pub unsafe fn dma_invalidate(va: *const u8, len: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        let mut p = (va as usize) & !(DCACHE_LINE - 1);
        let end = va as usize + len;
        while p < end {
            core::arch::asm!("dc ivac, {0}", in(reg) p, options(nostack, preserves_flags));
            p += DCACHE_LINE;
        }
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (va, len);
    }
}

// ---- Ring memory ----
/// One TRB as the controller sees it: parameter (2 dwords), status, control.
/// Accessed only through volatile reads and writes.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Trb {
    pub param_lo: u32,
    pub param_hi: u32,
    pub status: u32,
    pub control: u32,
}
const _: () = assert!(core::mem::size_of::<Trb>() == TRB_SIZE);

/// Event Ring Segment Table entry (6.5).
#[repr(C, align(64))]
pub struct ErstEntry {
    pub base: u64,
    pub size: u32,
    pub reserved: u32,
}
