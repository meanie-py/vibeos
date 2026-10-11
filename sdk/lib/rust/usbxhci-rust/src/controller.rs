//! Controller bring-up (4.2), the command ring (4.6), the event ring (4.9.4) and the
//! interrupt path.
//!
//! One contiguous, page-aligned common buffer from usbport holds everything the
//! controller owns for its whole life. Per-device and per-endpoint memory comes
//! from the endpoint buffers usbport allocates on OpenEndpoint (see `endpoint`).

use crate::hw::*;
use crate::types::*;
use crate::xlog;
use core::sync::atomic::{AtomicBool, Ordering};
use ntkrnl_sys::ddk::{KeStallExecutionProcessor, BOOLEAN, KIRQL, PULONG, PVOID, ULONG};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};

pub const PAGE: usize = 4096;

/// Device slots we enable. Keyboards, mice and a hub or two need far fewer; the
/// cap bounds the per-slot memory (two pages each) and the lookup tables.
pub const MAX_SLOTS: usize = 16;
/// Scratchpad pages we can provide (HCSPARAMS2). QEMU asks for none; real
/// controllers ask for a handful. More than this fails the start loudly.
pub const MAX_SCRATCHPADS: usize = 32;
/// TRBs per ring page; the last one on a producer ring is the Link TRB.
pub const RING_TRBS: usize = PAGE / TRB_SIZE;

// Common buffer layout. Every region is 64-byte aligned and none crosses a 64 KB
// boundary, as 6.1 and 6.5 require.
const OFF_DCBAA: usize = 0; // (MAX_SLOTS + 1) * 8 bytes
const OFF_ERST: usize = 2048; // one 16-byte entry, 64-byte aligned
const OFF_SPARRAY: usize = 2112; // MAX_SCRATCHPADS * 8 bytes
const OFF_CMD_RING: usize = PAGE;
const OFF_EVT_RING: usize = 2 * PAGE;
const OFF_INPUT_CTX: usize = 3 * PAGE;
const OFF_SCRATCH: usize = 4 * PAGE;
/// Per slot, two pages: the output device context, then the EP0 transfer ring.
pub const SLOTS_OFFSET: usize = OFF_SCRATCH + MAX_SCRATCHPADS * PAGE;
pub const COMMON_BUFFER_SIZE: usize = SLOTS_OFFSET + MAX_SLOTS * 2 * PAGE;
const _: () = assert!(OFF_SPARRAY + MAX_SCRATCHPADS * 8 <= PAGE);
const _: () = assert!((MAX_SLOTS + 1) * 8 <= OFF_ERST);

/// A producer ring (command ring or a transfer ring): one page of TRBs whose last
/// entry links back to the first and toggles the cycle state.
#[repr(C)]
pub struct Ring {
    pub va: *mut Trb,
    pub pa: u64,
    pub enqueue: u32,
    pub cycle: u32,
}

impl Ring {
    pub unsafe fn init(&mut self, va: *mut u8, pa: u64) {
        core::ptr::write_bytes(va, 0, PAGE);
        self.va = va as *mut Trb;
        self.pa = pa;
        self.enqueue = 0;
        self.cycle = 1;
        let link = self.va.add(RING_TRBS - 1);
        core::ptr::write_volatile(&mut (*link).param_lo, pa as u32);
        core::ptr::write_volatile(&mut (*link).param_hi, (pa >> 32) as u32);
        core::ptr::write_volatile(&mut (*link).status, 0);
        // Cycle bit 0: not yet owned by the controller. It is handed over when the
        // producer reaches it (push below).
        core::ptr::write_volatile(&mut (*link).control, (TRB_LINK << TRB_TYPE_SHIFT) | TRB_TC);
        dma_clean(va, PAGE);
    }

    pub fn pa_of(&self, index: u32) -> u64 {
        self.pa + index as u64 * TRB_SIZE as u64
    }

    /// Write one TRB and hand it to the controller. The body goes first and the
    /// control word, which carries the cycle bit, goes last behind a barrier.
    /// `control` must not include the cycle bit. Returns the TRB's bus address.
    pub unsafe fn push(&mut self, param: u64, status: u32, control: u32) -> u64 {
        self.push_inner(param, status, control, false)
    }

    /// Keep the first TRB of a TD CPU-owned until all its TRBs and software
    /// bookkeeping exist. An already-running ring can fetch without a doorbell.
    pub unsafe fn push_hidden(&mut self, param: u64, status: u32, control: u32) -> u64 {
        self.push_inner(param, status, control, true)
    }

    pub unsafe fn publish(&self, index: u32, cycle: u32) {
        let trb = self.va.add(index as usize);
        let control = core::ptr::read_volatile(&(*trb).control);
        wmb();
        core::ptr::write_volatile(&mut (*trb).control, (control & !TRB_CYCLE) | cycle);
        dma_clean(trb as *const u8, TRB_SIZE);
    }

    unsafe fn push_inner(&mut self, param: u64, status: u32, control: u32, hidden: bool) -> u64 {
        let idx = self.enqueue;
        let trb = self.va.add(idx as usize);
        core::ptr::write_volatile(&mut (*trb).param_lo, param as u32);
        core::ptr::write_volatile(&mut (*trb).param_hi, (param >> 32) as u32);
        core::ptr::write_volatile(&mut (*trb).status, status);
        wmb();
        core::ptr::write_volatile(&mut (*trb).control, control | (self.cycle ^ hidden as u32));
        dma_clean(trb as *const u8, TRB_SIZE);
        let pa = self.pa_of(idx);

        self.enqueue += 1;
        if self.enqueue as usize == RING_TRBS - 1 {
            // Hand the Link TRB over with the current cycle, keeping its chain bit
            // equal to the TRB before it so a chained TD may span the wrap.
            let link = self.va.add(RING_TRBS - 1);
            let chain = control & TRB_CH;
            wmb();
            core::ptr::write_volatile(
                &mut (*link).control,
                (TRB_LINK << TRB_TYPE_SHIFT) | TRB_TC | chain | self.cycle,
            );
            dma_clean(link as *const u8, TRB_SIZE);
            self.enqueue = 0;
            self.cycle ^= 1;
        }
        pa
    }

    /// Free TRB slots, leaving one gap so a full ring is never mistaken for empty.
    pub fn space(&self, dequeue: u32) -> u32 {
        let usable = RING_TRBS as u32 - 1;
        (dequeue + usable - self.enqueue - 1) % usable
    }
}

/// Result of a command, filled in by the event ring consumer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CommandResult {
    pub code: u32,
    pub slot: u32,
}

/// Miniport extension (MiniPortExtensionSize); usbport zeroes it before start.
#[repr(C)]
pub struct XhciExtension {
    pub cap: *mut u8,
    pub op: *mut u8,
    pub rt: *mut u8,
    pub db: *mut u8,
    pub hci_version: u32,
    pub max_slots: u32,
    pub max_ports: u32,
    /// Bytes per context: 32, or 64 when HCCPARAMS1.CSZ is set.
    pub ctx_size: u32,
    pub scratchpads: u32,
    pub ac64: u32,
    /// The USB2-protocol ports, as 1-based xHCI port numbers. usbport's root hub
    /// port N is xHCI port `usb2_first + N - 1`.
    pub usb2_first: u32,
    pub usb2_count: u32,
    pub usb3_first: u32,
    pub usb3_count: u32,

    pub va: *mut u8,
    pub pa: u64,
    pub dcbaa: *mut u64,
    pub cmd: Ring,
    pub evt_va: *mut Trb,
    pub evt_pa: u64,
    pub evt_dequeue: u32,
    pub evt_cycle: u32,
    pub input_ctx_va: *mut u8,
    pub input_ctx_pa: u64,

    pub cmd_lock: usize,
    pub evt_lock: usize,
    /// Bus address of the command TRB in flight, and its completion once seen.
    pub cmd_trb: u64,
    pub cmd_done: AtomicBool,
    pub cmd_result: CommandResult,
    pub pending_port_change: AtomicBool,
    pub pending_transfer: AtomicBool,
    /// Permanent failure until PnP restarts the controller. No new DMA allowed.
    pub failed: AtomicBool,
    pub hardware_error: AtomicBool,

    /// Indexed by slot ID; entry 0 is unused.
    pub slots: [crate::endpoint::Slot; MAX_SLOTS + 1],
    /// Endpoint lookup for transfer events: [slot][DCI] -> usbport endpoint.
    pub eps: [[*mut crate::endpoint::XhciEndpoint; 32]; MAX_SLOTS + 1],

    pub interrupts_on: bool,
    pub frame_hi: u32,
    pub last_frame: u32,
    pub started: bool,
}

#[inline]
pub unsafe fn ext_of(p: PVOID) -> &'static mut XhciExtension {
    &mut *(p as *mut XhciExtension)
}

/// Bounded register polling at DISPATCH_LEVEL; never parks the thread.
unsafe fn poll_until(mut cond: impl FnMut() -> bool, micros: u32) -> bool {
    let mut waited = 0;
    while !cond() {
        if waited >= micros {
            return false;
        }
        KeStallExecutionProcessor(10);
        waited += 10;
    }
    true
}

/// Walk the extended capability list (7): take the controller from the firmware
/// (USB Legacy Support) and learn which ports speak which protocol.
unsafe fn walk_extended_caps(x: &mut XhciExtension) {
    let hcc = rd32(x.cap, HCCPARAMS1);
    let mut off = ((hcc >> 16) as usize) * 4;
    let mut guard = 0;
    while off != 0 && guard < 64 {
        guard += 1;
        let v = rd32(x.cap, off);
        match v & 0xFF {
            XECP_LEGACY => {
                if v & LEGACY_BIOS_OWNED != 0 {
                    wr32(x.cap, off, v | LEGACY_OS_OWNED);
                    let cap = x.cap;
                    let ok = poll_until(|| rd32(cap, off) & LEGACY_BIOS_OWNED == 0, 1_000_000);
                    xlog!("firmware handoff %s", if ok { b"done\0".as_ptr() } else { b"TIMED OUT, taking it anyway\0".as_ptr() });
                    if !ok {
                        wr32(x.cap, off, (rd32(x.cap, off) & !LEGACY_BIOS_OWNED) | LEGACY_OS_OWNED);
                    }
                }
                // USBLEGCTLSTS: disable every SMI source, clear the RW1C events.
                let ctl = rd32(x.cap, off + 4);
                wr32(x.cap, off + 4, (ctl & !0x0000_E011) | 0xE000_0000);
            }
            XECP_PROTOCOL => {
                let major = v >> 24;
                let ports = rd32(x.cap, off + 8);
                let first = ports & 0xFF;
                let count = (ports >> 8) & 0xFF;
                if major == 2 && x.usb2_count == 0 {
                    x.usb2_first = first;
                    x.usb2_count = count;
                } else if major == 3 && x.usb3_count == 0 {
                    x.usb3_first = first;
                    x.usb3_count = count;
                }
            }
            _ => {}
        }
        let next = ((v >> 8) & 0xFF) as usize;
        if next == 0 {
            break;
        }
        off += next * 4;
    }
}

/// Clear PCI COMMAND.InterruptDisable (bit 10), set by pci.sys's xHCI quiesce.
unsafe fn unmask_intx(ext: PVOID) {
    const COMMAND: ULONG = 0x04;
    const INTX_DISABLE: u16 = 1 << 10;
    let p = &*core::ptr::addr_of!(crate::RegPacket);
    let Some(rw) = p.UsbPortReadWriteConfigSpace else { return };
    let mut cmd: u16 = 0;
    if rw(ext, 1, &mut cmd as *mut u16 as PVOID, COMMAND, 2) != MP_STATUS_SUCCESS {
        return;
    }
    if cmd & INTX_DISABLE != 0 {
        cmd &= !INTX_DISABLE;
        rw(ext, 0, &mut cmd as *mut u16 as PVOID, COMMAND, 2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_StartController(
    xhciExtension: PVOID,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    let r = &*Resources;

    if r.ResourcesTypes & USBPORT_RESOURCES_MEMORY == 0 || r.ResourceBase.is_null() {
        xlog!("start: no memory resource (types %x)", r.ResourcesTypes);
        return MP_STATUS_ERROR;
    }
    x.cap = r.ResourceBase as *mut u8;
    // CAPLENGTH and HCIVERSION share the first dword; read it whole, since not
    // every controller (QEMU's among them) answers byte or halfword reads there.
    let cap0 = rd32(x.cap, CAPLENGTH);
    let caplen = (cap0 & 0xFF) as usize;
    x.hci_version = cap0 >> 16;
    x.op = x.cap.add(caplen);
    x.rt = x.cap.add((rd32(x.cap, RTSOFF) & !0x1F) as usize);
    x.db = x.cap.add((rd32(x.cap, DBOFF) & !0x3) as usize);

    let hcs1 = rd32(x.cap, HCSPARAMS1);
    let hcs2 = rd32(x.cap, HCSPARAMS2);
    let hcc = rd32(x.cap, HCCPARAMS1);
    x.max_slots = core::cmp::min(hcs1 & 0xFF, MAX_SLOTS as u32);
    x.max_ports = hcs1 >> 24;
    x.ctx_size = if hcc & HCCPARAMS1_CSZ != 0 { 64 } else { 32 };
    x.ac64 = hcc & HCCPARAMS1_AC64;
    x.scratchpads = (((hcs2 >> 21) & 0x1F) << 5) | ((hcs2 >> 27) & 0x1F);

    if x.scratchpads as usize > MAX_SCRATCHPADS {
        xlog!("start: controller wants %u scratchpad pages, driver provides %u", x.scratchpads, MAX_SCRATCHPADS as u32);
        return MP_STATUS_ERROR;
    }
    if rd32(x.op, PAGESIZE) & 1 == 0 {
        xlog!("start: controller does not support 4K pages (PAGESIZE %x)", rd32(x.op, PAGESIZE));
        return MP_STATUS_ERROR;
    }

    walk_extended_caps(x);
    if x.usb2_count == 0 {
        // No protocol capability (not seen in practice): expose every port.
        x.usb2_first = 1;
        x.usb2_count = x.max_ports;
    }

    // Common buffer. usbport hands over the bus address as 32 bits; the regions
    // below inherit that, so every pointer we give the controller fits 32 bits
    // even when HCCPARAMS1.AC64 is clear.
    x.va = r.StartVA as *mut u8;
    x.pa = r.StartPA as u64;

    // Halt (5.4.1: R/S=0, wait for HCHalted), then reset.
    let op = x.op;
    wr32(op, USBCMD, rd32(op, USBCMD) & !USBCMD_RS);
    if !poll_until(|| rd32(op, USBSTS) & USBSTS_HCH != 0, 20_000) {
        xlog!("start: controller did not halt (USBSTS %x)", rd32(op, USBSTS));
        return MP_STATUS_HW_ERROR;
    }
    wr32(op, USBCMD, USBCMD_HCRST);
    if !poll_until(|| rd32(op, USBCMD) & USBCMD_HCRST == 0 && rd32(op, USBSTS) & USBSTS_CNR == 0, 1_000_000) {
        xlog!("start: reset did not complete (USBCMD %x USBSTS %x)", rd32(op, USBCMD), rd32(op, USBSTS));
        return MP_STATUS_HW_ERROR;
    }
    core::ptr::write_bytes(x.va, 0, COMMON_BUFFER_SIZE);
    core::ptr::write_bytes(x.slots.as_mut_ptr(), 0, x.slots.len());
    x.eps = [[core::ptr::null_mut(); 32]; MAX_SLOTS + 1];
    x.failed.store(false, Ordering::Release);
    x.hardware_error.store(false, Ordering::Release);

    // The reset left the controller quiet (USBCMD.INTE and IMAN clear), so its
    // INTx can be unmasked: pci.sys masks it at enumeration because firmware may
    // hand the controller over still asserting it. No-op without a PCI parent.
    unmask_intx(xhciExtension);

    wr32(op, CONFIG, (rd32(op, CONFIG) & !0xFF) | x.max_slots);

    // Device Context Base Address Array; entry 0 points at the scratchpad array.
    x.dcbaa = x.va.add(OFF_DCBAA) as *mut u64;
    if x.scratchpads != 0 {
        let arr = x.va.add(OFF_SPARRAY) as *mut u64;
        for i in 0..x.scratchpads as usize {
            core::ptr::write_volatile(arr.add(i), x.pa + (OFF_SCRATCH + i * PAGE) as u64);
        }
        core::ptr::write_volatile(x.dcbaa, x.pa + OFF_SPARRAY as u64);
    }
    wr64(op, DCBAAP, x.pa + OFF_DCBAA as u64);

    // Command ring; RCS = 1.
    x.cmd.init(x.va.add(OFF_CMD_RING), x.pa + OFF_CMD_RING as u64);
    wr64(op, CRCR, x.cmd.pa | 1);

    // Event ring: one segment, one page. Order per 4.9.4: ERSTSZ, ERDP, ERSTBA.
    x.evt_va = x.va.add(OFF_EVT_RING) as *mut Trb;
    x.evt_pa = x.pa + OFF_EVT_RING as u64;
    x.evt_dequeue = 0;
    x.evt_cycle = 1;
    let erst = x.va.add(OFF_ERST) as *mut ErstEntry;
    (*erst).base = x.evt_pa;
    (*erst).size = RING_TRBS as u32;
    x.input_ctx_va = x.va.add(OFF_INPUT_CTX);
    x.input_ctx_pa = x.pa + OFF_INPUT_CTX as u64;
    dma_clean(x.va, COMMON_BUFFER_SIZE);

    let ir = x.rt.add(IR0);
    wr32(ir, ERSTSZ, 1);
    wr64(ir, ERDP, x.evt_pa);
    wr64(ir, ERSTBA, x.pa + OFF_ERST as u64);
    wr32(ir, IMOD, 160); // 40 us moderation, in 250 ns units
    wr32(ir, IMAN, IMAN_IP); // clear anything pending, IE off

    // Run with interrupts off. usbport opens its ISR gate only after this
    // returns and then calls EnableInterrupts; an interrupt raised before that
    // (the NoOp below would raise one) is declined by the gate, and a declined
    // level-triggered line fires forever. Until then everything polls.
    wr32(op, USBCMD, USBCMD_RS | USBCMD_HSEE);
    if !poll_until(|| rd32(op, USBSTS) & USBSTS_HCH == 0, 20_000) {
        xlog!("start: controller did not run (USBSTS %x)", rd32(op, USBSTS));
        return MP_STATUS_HW_ERROR;
    }
    x.interrupts_on = false;
    x.started = true;

    // Port power, when the controller switches it (HCCPARAMS1.PPC).
    if hcc & HCCPARAMS1_PPC != 0 {
        for port in 1..=x.max_ports {
            let off = PORTSC_BASE + 0x10 * (port as usize - 1);
            let v = rd32(op, off);
            if v & PORTSC_PP == 0 {
                wr32(op, off, (v & PORTSC_PRESERVE) | PORTSC_PP);
            }
        }
    }

    // Prove both rings work before usbport builds anything on them.
    let r = command(x, 0, 0, TRB_NOOP_COMMAND << TRB_TYPE_SHIFT);
    xlog!(
        "controller running: xHCI %x.%02x, %u slots, %u ports (USB2 %u-%u, USB3 %u-%u), ctx %u, scratchpads %u, NoOp cc %u",
        x.hci_version >> 8,
        x.hci_version & 0xFF,
        x.max_slots,
        x.max_ports,
        x.usb2_first,
        x.usb2_first + x.usb2_count - 1,
        x.usb3_first,
        x.usb3_first + x.usb3_count.saturating_sub(1),
        x.ctx_size,
        x.scratchpads,
        r.code,
    );
    if r.code != CC_SUCCESS {
        if !x.failed.load(Ordering::Acquire) { fail_controller(x); }
        return MP_STATUS_HW_ERROR;
    }
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_StopController(xhciExtension: PVOID, _DisableInterrupts: BOOLEAN) {
    let x = ext_of(xhciExtension);
    if x.op.is_null() {
        return;
    }
    fail_controller(x);
    xlog!("controller stopped");
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_SuspendController(xhciExtension: PVOID) {
    XHCI_StopController(xhciExtension, 1);
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_ResumeController(_xhciExtension: PVOID) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}

/// Fail closed before any DMA memory can be reused. usbport's reset/remove
/// invalidation callbacks are unimplemented, so don't pretend they will recover
/// ownership for us. The controller remains unavailable until a PnP restart.
pub unsafe fn fail_controller(x: &mut XhciExtension) {
    x.failed.store(true, Ordering::Release);
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.evt_lock, &mut irql);
    wr32(x.op, USBCMD, rd32(x.op, USBCMD) & !(USBCMD_RS | USBCMD_INTE));
    if !poll_until(|| rd32(x.op, USBSTS) & USBSTS_HCH != 0, 100_000) {
        // Abort/Close cannot return an error to stop usbport freeing the buffer.
        // Bugcheck through panic-common instead of returning live DMA memory.
        panic!("xHCI failed to halt; refusing to release DMA-owned memory");
    }
    x.started = false;
    x.interrupts_on = false;
    for slot in 1..=MAX_SLOTS {
        for dci in 1..32 {
            let ep = x.eps[slot][dci];
            if ep.is_null() { continue; }
            crate::endpoint::controller_failed(&mut *ep);
        }
    }
    x.pending_transfer.store(true, Ordering::Release);
    KeReleaseSpinLock(&mut x.evt_lock, irql);
    xlog!("controller halted; outstanding transfers failed, PnP restart required");
}

/// Issue one command and wait for its completion event. Commands are rare
/// (device attach, endpoint open/close) and usbport's callers are synchronous, so
/// the wait polls the event ring itself instead of depending on the DPC, which
/// cannot run on this CPU while we spin at DISPATCH_LEVEL.
pub unsafe fn command(x: &mut XhciExtension, param: u64, status: u32, control: u32) -> CommandResult {
    let irql = lock_commands(x);
    let r = command_locked(x, param, status, control);
    unlock_commands(x, irql);
    r
}

/// The command lock also guards the shared input context, so a caller that
/// builds one holds it across both.
pub unsafe fn lock_commands(x: &mut XhciExtension) -> KIRQL {
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.cmd_lock, &mut irql);
    irql
}

pub unsafe fn unlock_commands(x: &mut XhciExtension, irql: KIRQL) {
    KeReleaseSpinLock(&mut x.cmd_lock, irql);
}

pub unsafe fn command_locked(x: &mut XhciExtension, param: u64, status: u32, control: u32) -> CommandResult {
    if x.failed.load(Ordering::Acquire) {
        return CommandResult { code: 0, slot: 0 };
    }
    // The event consumer runs under a different usbport lock on another CPU.
    // Install the waiter before publishing a command to the running ring.
    let mut event_irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.evt_lock, &mut event_irql);
    x.cmd_done.store(false, Ordering::SeqCst);
    let index = x.cmd.enqueue;
    let cycle = x.cmd.cycle;
    x.cmd_trb = x.cmd.push_hidden(param, status, control);
    x.cmd.publish(index, cycle);
    KeReleaseSpinLock(&mut x.evt_lock, event_irql);
    wmb();
    wr32(x.db, 0, 0);

    let mut result = CommandResult { code: 0, slot: 0 };
    let mut waited = 0u32;
    loop {
        process_events(x);
        if x.cmd_done.load(Ordering::SeqCst) {
            result = x.cmd_result;
            break;
        }
        if waited >= 500_000 {
            xlog!("command type %u timed out (USBSTS %x CRCR %x)", trb_type(control), rd32(x.op, USBSTS), rd32(x.op, CRCR));
            fail_controller(x);
            break;
        }
        KeStallExecutionProcessor(10);
        waited += 10;
    }
    result
}

/// Ring an endpoint's doorbell (5.6): target = DCI.
pub unsafe fn ring_doorbell(x: &XhciExtension, slot: u32, dci: u32) {
    if x.failed.load(Ordering::Acquire) { return; }
    wmb();
    wr32(x.db, slot as usize * 4, dci);
}

/// What one pass over the event ring saw, so callers know whom to notify.
#[derive(Default)]
pub struct EventSummary {
    pub port_change: bool,
    pub transfer: bool,
}

/// Consume every event the controller has posted. Safe to call from the DPC,
/// the poll timer and a command wait at once: one consumer at a time.
pub unsafe fn process_events(x: &mut XhciExtension) -> EventSummary {
    let mut s = EventSummary::default();
    if x.evt_va.is_null() {
        return s;
    }
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.evt_lock, &mut irql);
    if x.failed.load(Ordering::Acquire) {
        KeReleaseSpinLock(&mut x.evt_lock, irql);
        return s;
    }
    loop {
        let trb = x.evt_va.add(x.evt_dequeue as usize);
        dma_invalidate(trb as *const u8, TRB_SIZE);
        let control = core::ptr::read_volatile(&(*trb).control);
        if control & TRB_CYCLE != x.evt_cycle {
            break;
        }
        rmb();
        let param = core::ptr::read_volatile(&(*trb).param_lo) as u64
            | (core::ptr::read_volatile(&(*trb).param_hi) as u64) << 32;
        let status = core::ptr::read_volatile(&(*trb).status);
        match trb_type(control) {
            TRB_EV_COMMAND_COMPLETION => {
                if param == x.cmd_trb {
                    x.cmd_result = CommandResult { code: status >> 24, slot: control >> 24 };
                    x.cmd_done.store(true, Ordering::SeqCst);
                }
            }
            TRB_EV_PORT_STATUS_CHANGE => {
                s.port_change = true;
                // The first events of a boot, with the port's state: a port whose
                // change bits keep re-arming shows up here, not in usbhub.
                static LOGGED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                if LOGGED.fetch_add(1, Ordering::Relaxed) < 64 {
                    let hw = (param as u32) >> 24;
                    let v = if hw >= 1 { rd32(x.op, PORTSC_BASE + 0x10 * (hw as usize - 1)) } else { 0 };
                    xlog!("port change event: hw port %u, PORTSC %x (op %p)", hw, v, x.op);
                }
            }
            TRB_EV_TRANSFER => {
                crate::endpoint::on_transfer_event(x, param, status, control);
                s.transfer = true;
            }
            TRB_EV_HOST_CONTROLLER => {
                xlog!("host controller event, cc %u", status >> 24);
                x.hardware_error.store(true, Ordering::Release);
            }
            _ => {}
        }
        x.evt_dequeue += 1;
        if x.evt_dequeue as usize == RING_TRBS {
            x.evt_dequeue = 0;
            x.evt_cycle ^= 1;
        }
    }
    // Command waits consume events too, but cannot call into usbport while its
    // miniport lock is held. Latch notifications for the next DPC/timer poll.
    if s.port_change {
        x.pending_port_change.store(true, Ordering::Release);
    }
    if s.transfer {
        x.pending_transfer.store(true, Ordering::Release);
    }
    let erdp = x.evt_pa + x.evt_dequeue as u64 * TRB_SIZE as u64;
    wr64(x.rt.add(IR0), ERDP, erdp | ERDP_EHB as u64);
    KeReleaseSpinLock(&mut x.evt_lock, irql);
    s
}

unsafe fn notify(x: &mut XhciExtension, _s: EventSummary) {
    let p = &*core::ptr::addr_of!(crate::RegPacket);
    if x.pending_port_change.swap(false, Ordering::AcqRel) {
        if let Some(f) = p.UsbPortInvalidateRootHub {
            f(x as *mut _ as PVOID);
        }
    }
    if x.pending_transfer.swap(false, Ordering::AcqRel) {
        if let Some(f) = p.UsbPortInvalidateEndpoint {
            f(x as *mut _ as PVOID, core::ptr::null_mut());
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_InterruptService(xhciExtension: PVOID) -> BOOLEAN {
    let x = ext_of(xhciExtension);
    if !x.started {
        return 0;
    }
    let sts = rd32(x.op, USBSTS);
    if sts == 0xFFFF_FFFF || sts & (USBSTS_EINT | USBSTS_HSE | USBSTS_PCD) == 0 {
        return 0;
    }
    // Acknowledge: USBSTS.EINT (RW1C) then IMAN.IP (RW1C). Event processing and
    // ERDP happen in the DPC.
    wr32(x.op, USBSTS, sts & (USBSTS_EINT | USBSTS_PCD | USBSTS_HSE));
    let ir = x.rt.add(IR0);
    wr32(ir, IMAN, rd32(ir, IMAN) | IMAN_IP);
    if sts & USBSTS_HSE != 0 {
        // USBSTS.HSE was just acknowledged. Preserve it for CheckController,
        // which runs under usbport's MiniportSpinLock and may safely quiesce DMA.
        x.hardware_error.store(true, Ordering::Release);
        xlog!("host system error (USBSTS %x)", sts);
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_InterruptDpc(xhciExtension: PVOID, EnableInterrupts: BOOLEAN) {
    let x = ext_of(xhciExtension);
    let s = process_events(x);
    notify(x, s);
    if EnableInterrupts != 0 {
        XHCI_EnableInterrupts(xhciExtension);
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_EnableInterrupts(xhciExtension: PVOID) {
    let x = ext_of(xhciExtension);
    if x.op.is_null() || x.failed.load(Ordering::Acquire) {
        return;
    }
    let ir = x.rt.add(IR0);
    wr32(ir, IMAN, IMAN_IE);
    wr32(x.op, USBCMD, rd32(x.op, USBCMD) | USBCMD_INTE);
    x.interrupts_on = true;
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_DisableInterrupts(xhciExtension: PVOID) {
    let x = ext_of(xhciExtension);
    if x.op.is_null() {
        return;
    }
    wr32(x.op, USBCMD, rd32(x.op, USBCMD) & !USBCMD_INTE);
    x.interrupts_on = false;
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_PollController(xhciExtension: PVOID) {
    let x = ext_of(xhciExtension);
    if !x.started {
        if x.failed.load(Ordering::Acquire) {
            notify(x, EventSummary::default());
        }
        return;
    }
    let s = process_events(x);
    notify(x, s);
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_CheckController(xhciExtension: PVOID) {
    let x = ext_of(xhciExtension);
    if !x.started {
        return;
    }
    let sts = rd32(x.op, USBSTS);
    if sts == 0xFFFF_FFFF {
        fail_controller(x);
        let p = &*core::ptr::addr_of!(crate::RegPacket);
        if let Some(f) = p.UsbPortInvalidateController {
            f(xhciExtension, USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE);
        }
    } else if sts & (USBSTS_HCE | USBSTS_HSE) != 0 || x.hardware_error.load(Ordering::Acquire) {
        xlog!("controller error (USBSTS %x)", sts);
        fail_controller(x);
    }
}

/// 32-bit frame number from MFINDEX (14 bits of microframes, so 11 bits of
/// frames), extended in software.
#[no_mangle]
pub unsafe extern "system" fn XHCI_Get32BitFrameNumber(xhciExtension: PVOID) -> ULONG {
    let x = ext_of(xhciExtension);
    if x.rt.is_null() {
        return 0;
    }
    let frame = (rd32(x.rt, MFINDEX) >> 3) & 0x7FF;
    if frame < x.last_frame {
        x.frame_hi = x.frame_hi.wrapping_add(0x800);
    }
    x.last_frame = frame;
    x.frame_hi | frame
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_InterruptNextSOF(_xhciExtension: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn XHCI_FlushInterrupts(_xhciExtension: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn XHCI_TakePortControl(_xhciExtension: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn XHCI_StartSendOnePacket(
    _xhciExtension: PVOID,
    _PacketParameters: PVOID,
    _Data: PVOID,
    _pDataLength: PULONG,
    _BufferVA: PVOID,
    _BufferPA: PVOID,
    _BufferLength: ULONG,
    _pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_EndSendOnePacket(
    _xhciExtension: PVOID,
    _PacketParameters: PVOID,
    _Data: PVOID,
    _pDataLength: PULONG,
    _BufferVA: PVOID,
    _BufferPA: PVOID,
    _BufferLength: ULONG,
    _pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_PassThru(
    _xhciExtension: PVOID,
    _PassThruParameters: PVOID,
    _ParameterLength: ULONG,
    _pParameters: PVOID,
) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}
