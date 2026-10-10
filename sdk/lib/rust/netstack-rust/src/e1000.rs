//! `E1000Phy` — direct-PCI Intel 8254x (e1000) smoltcp `phy::Device` backend.
//!
//! The drop-in sibling of `VirtioPhy` (`device.rs`) for a REAL emulated NIC:
//! QEMU's default `-device e1000` (82540EM, 8086:100E) and friends. Where
//! `VirtioPhy` rides the paravirtual virtio queues, this drives the actual
//! MMIO register file + legacy RX/TX descriptor rings, exactly like a metal
//! driver would — no NDIS, no miniport, pure Rust on the PCI bus.
//!
//! Design mirrors `VirtioPhy`: the engine's worker calls `receive`/`transmit`
//! and sleeps on an event that the INTx ISR (`irq_ack`) sets when a frame
//! lands; with no interrupt it polls on a timeout. Copy-based (RX
//! copies the frame out of the DMA buffer, TX copies into one), single
//! contiguous DMA arena per ring (descriptors + buffers) allocated with
//! `MmAllocateContiguousMemorySpecifyCache` like `hal.rs`. x86 DMA is
//! cache-coherent so cached memory is correct.

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;

use alloc::vec::Vec;

use ntkrnl_sys::ddk::{
    LARGE_INTEGER, MmAllocateContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmMapIoSpace,
    PVOID, _MEMORY_CACHING_TYPE_MmCached as MM_CACHED,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};

use virtio_drivers::transport::pci::bus::DeviceFunction;

use crate::serial::nsprint;
use crate::pci::{enable_bus_master, HalPciAccess};
use virtio_drivers::transport::pci::bus::ConfigurationAccess;

// ---------------------------------------------------------------- registers
const CTRL: usize = 0x0000;
const STATUS: usize = 0x0008;
const EERD: usize = 0x0014;
const IMS: usize = 0x00D0;
const IMC: usize = 0x00D8;
const ICR: usize = 0x00C0;
const RDTR: usize = 0x2820;
const RCTL: usize = 0x0100;
const TCTL: usize = 0x0400;
const TIPG: usize = 0x0410;
const RDBAL: usize = 0x2800;
const RDBAH: usize = 0x2804;
const RDLEN: usize = 0x2808;
const RDH: usize = 0x2810;
const RDT: usize = 0x2818;
const TDBAL: usize = 0x3800;
const TDBAH: usize = 0x3804;
const TDLEN: usize = 0x3808;
const TDH: usize = 0x3810;
const TDT: usize = 0x3818;
const RAL0: usize = 0x5400;
const RAH0: usize = 0x5404;
const MTA_BASE: usize = 0x5200;
const MTA_DWORDS: usize = 128;

const CTRL_ASDE: u32 = 1 << 5;
const CTRL_SLU: u32 = 1 << 6;
const CTRL_RST: u32 = 1 << 26;

const RCTL_EN: u32 = 1 << 1;
const RCTL_BAM: u32 = 1 << 15;
const RCTL_SECRC: u32 = 1 << 26;
// BSIZE 00 + BSEX 0 = 2048-byte buffers (reset default bits already 0).

const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;
const TCTL_CT_SHIFT: u32 = 4;
const TCTL_COLD_SHIFT: u32 = 12;

// ICR / IMS cause bits.
const INT_LSC: u32 = 1 << 2; // link status change
const INT_RXDMT0: u32 = 1 << 4; // RX descriptors below the minimum threshold
const INT_RXO: u32 = 1 << 6; // RX overrun
const INT_RXT0: u32 = 1 << 7; // RX timer: a frame landed
/// The causes the worker acts on. TX completion is not one: the worker reclaims
/// TX descriptors when it next sends.
const INT_WAKE: u32 = INT_RXT0 | INT_RXO | INT_RXDMT0 | INT_LSC;

const RXD_STAT_DD: u8 = 1 << 0;
const RXD_STAT_EOP: u8 = 1 << 1;

const TXD_CMD_EOP: u8 = 1 << 0;
const TXD_CMD_IFCS: u8 = 1 << 1;
const TXD_CMD_RS: u8 = 1 << 3;
const TXD_STAT_DD: u8 = 1 << 0;

// ------------------------------------------------------------------- layout
const N_RX: usize = 32;
const N_TX: usize = 32;
const BUF_SIZE: usize = 2048;
const FRAME_MAX: usize = 1536;

/// Legacy receive descriptor (16 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
struct RxDesc {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

/// Legacy transmit descriptor (16 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
struct TxDesc {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

/// Supported PCI IDs: QEMU `e1000` family (all share the 8254x legacy
/// descriptor programming model).
const E1000_IDS: &[(u16, u16)] = &[
    (0x8086, 0x100E), // 82540EM  — QEMU "e1000" default
    (0x8086, 0x100F), // 82545EM  — QEMU "e1000-82545em"
    (0x8086, 0x10D3), // 82574L   — NOT programmed here; excluded below
];

/// Scan bus 0 for a supported e1000. Returns (df, bar0_physical).
pub fn find_e1000() -> Option<(DeviceFunction, u64)> {
    let access = HalPciAccess;
    for device in 0..32u8 {
        for function in 0..8u8 {
            let df = DeviceFunction { bus: 0, device, function };
            let id = access.read_word(df, 0x00);
            let vendor = (id & 0xFFFF) as u16;
            let dev = (id >> 16) as u16;
            // 82574L (e1000e) has a different RX programming model — skip it.
            if vendor == 0x8086 && (dev == E1000_IDS[0].1 || dev == E1000_IDS[1].1) {
                let bar0 = access.read_word(df, 0x10);
                if bar0 & 1 != 0 {
                    continue; // I/O BAR — not the MMIO we want
                }
                let base = (bar0 & 0xFFFF_FFF0) as u64;
                if base != 0 {
                    return Some((df, base));
                }
            }
        }
    }
    None
}

#[inline]
fn phys_li(v: u64) -> LARGE_INTEGER {
    let mut p: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    p.QuadPart = v as i64;
    p
}

/// One physically-contiguous, sub-4GiB, zeroed DMA arena.
unsafe fn dma_arena(bytes: usize) -> Option<(*mut u8, u64)> {
    let va = MmAllocateContiguousMemorySpecifyCache(
        bytes as _,
        phys_li(0),
        phys_li(0xFFFF_FFFF),
        phys_li(0),
        MM_CACHED as _,
    ) as *mut u8;
    if va.is_null() {
        return None;
    }
    core::ptr::write_bytes(va, 0, bytes);
    let pa = MmGetPhysicalAddress(va as PVOID).QuadPart as u64;
    Some((va, pa))
}

pub struct E1000 {
    df: DeviceFunction,
    regs: *mut u32,
    mac: [u8; 6],
    rx_desc: *mut RxDesc,
    rx_buf: *mut u8,
    rx_next: usize, // next descriptor we expect DD on
    tx_desc: *mut TxDesc,
    tx_buf: *mut u8,
    tx_next: usize, // next descriptor to fill
}

// The engine worker solely owns the device; raw pointers are to non-paged
// DMA memory + MMIO that live for the driver's lifetime.
unsafe impl Send for E1000 {}

impl E1000 {
    #[inline]
    unsafe fn rd(&self, off: usize) -> u32 {
        core::ptr::read_volatile(self.regs.add(off / 4))
    }
    #[inline]
    unsafe fn wr(&self, off: usize, val: u32) {
        core::ptr::write_volatile(self.regs.add(off / 4), val);
    }

    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    /// The INTx line to connect (`pci::read_interrupt_line`).
    pub fn interrupt_line(&self) -> Option<u8> {
        crate::pci::read_interrupt_line(self.df)
    }

    /// The register window, the context `irq_ack` takes.
    pub fn irq_ctx(&self) -> *mut u8 {
        self.regs.cast()
    }

    /// Unmask the causes the worker acts on, then let the function assert INTx.
    /// Call only once an ISR is connected: the line is shared, and an e1000
    /// interrupt nobody claims would hold the level line asserted.
    pub fn enable_interrupts(&self) {
        unsafe {
            let _ = self.rd(ICR);
            self.wr(IMS, INT_WAKE);
        }
        crate::pci::unmask_intx(self.df);
    }

    /// Poll-side RX: is a completed frame waiting?
    pub fn can_recv(&self) -> bool {
        unsafe { (*self.rx_desc.add(self.rx_next)).status & RXD_STAT_DD != 0 }
    }

    /// Copy the next completed frame out and hand its descriptor back to HW.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        unsafe {
            let idx = self.rx_next;
            let desc = &mut *self.rx_desc.add(idx);
            if desc.status & RXD_STAT_DD == 0 {
                return None;
            }
            // v1: no jumbo/multi-descriptor frames (2048-byte buffers, MTU
            // 1500) — a non-EOP descriptor would be a fragment; drop it.
            let ok = desc.status & RXD_STAT_EOP != 0 && desc.errors == 0;
            let len = desc.length as usize;
            let out = if ok && len > 0 && len <= FRAME_MAX {
                let src = self.rx_buf.add(idx * BUF_SIZE);
                let mut v = Vec::with_capacity(len);
                v.extend_from_slice(core::slice::from_raw_parts(src, len));
                Some(v)
            } else {
                None
            };
            // Recycle: clear status, then advance RDT over this slot.
            desc.status = 0;
            self.wr(RDT, idx as u32);
            self.rx_next = (idx + 1) % N_RX;
            out
        }
    }

    /// Poll-side TX: is the next descriptor free?
    pub fn can_send(&self) -> bool {
        unsafe {
            let desc = &*self.tx_desc.add(self.tx_next);
            // Free if never used (cmd==0) or HW reported completion.
            desc.cmd == 0 || desc.status & TXD_STAT_DD != 0
        }
    }

    pub fn send(&mut self, frame: &[u8]) {
        if frame.len() > BUF_SIZE {
            return;
        }
        unsafe {
            let idx = self.tx_next;
            let buf = self.tx_buf.add(idx * BUF_SIZE);
            core::ptr::copy_nonoverlapping(frame.as_ptr(), buf, frame.len());
            let desc = &mut *self.tx_desc.add(idx);
            desc.length = frame.len() as u16;
            desc.cso = 0;
            desc.cmd = TXD_CMD_EOP | TXD_CMD_IFCS | TXD_CMD_RS;
            desc.status = 0;
            desc.css = 0;
            desc.special = 0;
            self.tx_next = (idx + 1) % N_TX;
            self.wr(TDT, self.tx_next as u32);
        }
    }
}

/// Find, reset, and bring up the e1000. Returns None (with a serial line) on
/// any failure — the engine falls back / reports.
pub fn bringup() -> Option<E1000> {
    let (df, bar0) = find_e1000()?;
    enable_bus_master(df);

    unsafe {
        // 128 KiB register window, uncached.
        let regs = MmMapIoSpace(phys_li(bar0), 0x20000, MM_NON_CACHED as _) as *mut u32;
        if regs.is_null() {
            nsprint!("[NETSTACK] e1000: MmMapIoSpace(BAR0={:#x}) failed\n", bar0);
            return None;
        }

        let rd = |off: usize| core::ptr::read_volatile(regs.add(off / 4));
        let wr = |off: usize, val: u32| core::ptr::write_volatile(regs.add(off / 4), val);

        // Mask all interrupts, reset, re-mask (RST clears IMC).
        wr(IMC, 0xFFFF_FFFF);
        wr(CTRL, rd(CTRL) | CTRL_RST);
        let mut spins = 0u32;
        while rd(CTRL) & CTRL_RST != 0 && spins < 100_000 {
            spins += 1;
        }
        wr(IMC, 0xFFFF_FFFF);
        let _ = rd(ICR);

        // Link up + auto speed detect.
        wr(CTRL, (rd(CTRL) | CTRL_SLU | CTRL_ASDE) & !CTRL_RST);

        // MAC: QEMU programs RAL0/RAH0 from -device mac=; fall back to EEPROM.
        let ral = rd(RAL0);
        let rah = rd(RAH0);
        let mut mac = [
            ral as u8,
            (ral >> 8) as u8,
            (ral >> 16) as u8,
            (ral >> 24) as u8,
            rah as u8,
            (rah >> 8) as u8,
        ];
        if ral == 0 && rah & 0xFFFF == 0 {
            for w in 0..3usize {
                wr(EERD, 1 | ((w as u32) << 8));
                let mut v = rd(EERD);
                let mut tries = 0u32;
                while v & (1 << 4) == 0 && tries < 100_000 {
                    v = rd(EERD);
                    tries += 1;
                }
                let data = (v >> 16) as u16;
                mac[w * 2] = data as u8;
                mac[w * 2 + 1] = (data >> 8) as u8;
            }
            wr(RAL0, u32::from_le_bytes([mac[0], mac[1], mac[2], mac[3]]));
            wr(RAH0, (mac[4] as u32) | ((mac[5] as u32) << 8) | (1 << 31));
        }

        // Clear the multicast table.
        for i in 0..MTA_DWORDS {
            wr(MTA_BASE + i * 4, 0);
        }

        // RX ring: descriptors (32 * 16 = 512B) then buffers, one arena.
        let rx_bytes = N_RX * core::mem::size_of::<RxDesc>() + N_RX * BUF_SIZE;
        let (rx_va, rx_pa) = match dma_arena(rx_bytes) {
            Some(x) => x,
            None => {
                nsprint!("[NETSTACK] e1000: RX DMA arena alloc failed\n");
                return None;
            }
        };
        let rx_desc = rx_va as *mut RxDesc;
        let rx_buf = rx_va.add(N_RX * core::mem::size_of::<RxDesc>());
        let rx_buf_pa = rx_pa + (N_RX * core::mem::size_of::<RxDesc>()) as u64;
        for i in 0..N_RX {
            (*rx_desc.add(i)).addr = rx_buf_pa + (i * BUF_SIZE) as u64;
            (*rx_desc.add(i)).status = 0;
        }
        wr(RDBAL, rx_pa as u32);
        wr(RDBAH, (rx_pa >> 32) as u32);
        wr(RDLEN, (N_RX * core::mem::size_of::<RxDesc>()) as u32);
        wr(RDH, 0);
        wr(RDT, (N_RX - 1) as u32);
        wr(RCTL, RCTL_EN | RCTL_BAM | RCTL_SECRC);

        // TX ring.
        let tx_bytes = N_TX * core::mem::size_of::<TxDesc>() + N_TX * BUF_SIZE;
        let (tx_va, tx_pa) = match dma_arena(tx_bytes) {
            Some(x) => x,
            None => {
                nsprint!("[NETSTACK] e1000: TX DMA arena alloc failed\n");
                return None;
            }
        };
        let tx_desc = tx_va as *mut TxDesc;
        let tx_buf = tx_va.add(N_TX * core::mem::size_of::<TxDesc>());
        let tx_buf_pa = tx_pa + (N_TX * core::mem::size_of::<TxDesc>()) as u64;
        for i in 0..N_TX {
            // Each descriptor permanently owns its bounce buffer; send() only
            // copies the frame in and flips cmd/length.
            (*tx_desc.add(i)).addr = tx_buf_pa + (i * BUF_SIZE) as u64;
        }
        wr(TDBAL, tx_pa as u32);
        wr(TDBAH, (tx_pa >> 32) as u32);
        wr(TDLEN, (N_TX * core::mem::size_of::<TxDesc>()) as u32);
        wr(TDH, 0);
        wr(TDT, 0);
        wr(TCTL, TCTL_EN | TCTL_PSP | (0x10 << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT));
        wr(TIPG, 10 | (8 << 10) | (6 << 20));

        let status = rd(STATUS);
        nsprint!(
            "[NETSTACK] e1000 up: mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} status={:#x} bar0={:#x}\n",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5], status, bar0
        );

        // Interrupt as soon as a frame lands, without the receive-delay timer.
        wr(RDTR, 0);

        Some(E1000 {
            df,
            regs,
            mac,
            rx_desc,
            rx_buf,
            rx_next: 0,
            tx_desc,
            tx_buf,
            tx_next: 0,
        })
    }
}

/// ISR-side acknowledge, for `isr::connect_line`. ICR is read-to-clear, which
/// both reports the causes and deasserts INTx; the worker never reads it.
pub unsafe fn irq_ack(regs: PVOID) -> crate::isr::Ack {
    let icr = core::ptr::read_volatile(regs.cast::<u8>().add(ICR).cast::<u32>());
    if icr == 0 {
        return crate::isr::Ack::NotOurs;
    }
    if icr & INT_WAKE != 0 { crate::isr::Ack::Wake } else { crate::isr::Ack::Handled }
}

// --------------------------------------------------------- smoltcp adapter
pub struct E1000Phy {
    dev: E1000,
}

impl E1000Phy {
    pub fn new(dev: E1000) -> Self {
        Self { dev }
    }
    pub fn mac(&self) -> [u8; 6] {
        self.dev.mac_address()
    }
    pub fn dev(&self) -> &E1000 {
        &self.dev
    }
}

pub struct E1000RxToken {
    data: Vec<u8>,
}

impl phy::RxToken for E1000RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }
}

pub struct E1000TxToken<'a> {
    dev: &'a mut E1000,
}

impl<'a> phy::TxToken for E1000TxToken<'a> {
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

impl Device for E1000Phy {
    type RxToken<'a> = E1000RxToken;
    type TxToken<'a> = E1000TxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if !self.dev.can_recv() {
            return None;
        }
        // A completed-but-bad descriptor (fragment/error) is recycled inside
        // receive(); report "nothing" this poll and catch the next frame later.
        let data = self.dev.receive()?;
        Some((E1000RxToken { data }, E1000TxToken { dev: &mut self.dev }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if !self.dev.can_send() {
            return None;
        }
        Some(E1000TxToken { dev: &mut self.dev })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
