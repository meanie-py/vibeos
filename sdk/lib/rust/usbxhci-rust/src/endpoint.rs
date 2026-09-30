//! usbport endpoints and transfers, translated to xHCI device slots, contexts and
//! transfer rings.
//!
//! usbport assumes EHCI-style addressing: it opens a new device's default pipe at
//! USB address 0, sends `SET_ADDRESS` as an ordinary control transfer, then
//! re-opens the pipe at the new address. xHCI instead owns addressing through
//! device slots. The translation:
//!
//! | usbport                                   | here                                        |
//! |-------------------------------------------|---------------------------------------------|
//! | open EP0 at address 0                     | Enable Slot + Address Device (BSR=1)        |
//! | `SET_ADDRESS` on that pipe                | intercepted: Address Device (BSR=0), never on the wire |
//! | open EP0 at address N (the re-open)       | find the slot now known as N; Evaluate Context if MPS changed |
//! | open any other endpoint                   | Configure Endpoint (add)                    |
//! | close it                                  | Configure Endpoint (drop)                   |
//!
//! usbport re-opens a pipe by closing it, freeing its buffer and opening it on a
//! new one, so nothing that must survive that (device contexts, the EP0 ring) can
//! live in an endpoint buffer. Both live per slot in the controller's common
//! buffer instead; only non-default endpoints put their ring in their own buffer.
//!
//! Locking: `ep.lock` guards an endpoint's pending-transfer queue. It is never
//! held across a command, because a command waits by consuming the event ring,
//! and the event ring takes `ep.lock` to record completions.

use crate::controller::*;
use crate::hw::*;
use crate::types::*;
use crate::xlog;
use core::sync::atomic::{AtomicBool, Ordering};
use ntkrnl_sys::ddk::{BOOLEAN, KIRQL, PULONG, PVOID, ULONG};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};

const USBD_STATUS_SUCCESS: USBD_STATUS = 0;
const USBD_STATUS_CRC: USBD_STATUS = 0xC000_0001u32 as USBD_STATUS;
const USBD_STATUS_STALL_PID: USBD_STATUS = 0xC000_0004u32 as USBD_STATUS;
const USBD_STATUS_DEV_NOT_RESPONDING: USBD_STATUS = 0xC000_0005u32 as USBD_STATUS;
const USBD_STATUS_XACT_ERROR: USBD_STATUS = 0xC000_0011u32 as USBD_STATUS;
const USBD_STATUS_BABBLE_DETECTED: USBD_STATUS = 0xC000_0012u32 as USBD_STATUS;
const USBD_STATUS_DATA_BUFFER_ERROR: USBD_STATUS = 0xC000_0013u32 as USBD_STATUS;
const USBD_STATUS_INTERNAL_HC_ERROR: USBD_STATUS = 0x8000_0800u32 as USBD_STATUS;

/// Transfers an endpoint may have queued at once.
const MAX_TDS: usize = 32;
/// Data-stage TRBs per transfer (one per scatter/gather element).
const MAX_DATA_TRBS: usize = 24;
/// Private DMA pages: never share a cache line with a caller's pool allocation.
/// The bitmap permits up to 16 small requests, or one 64 KiB request, per pipe.
const PAYLOAD_PAGES: u32 = 16;
const PAYLOAD_BYTES: usize = PAYLOAD_PAGES as usize * PAGE;
/// Usable TRBs per ring: the last one is the Link TRB.
const RING_USABLE: u32 = RING_TRBS as u32 - 1;

/// Per-slot state, owned by the extension (see module comment for why).
#[repr(C)]
pub struct Slot {
    pub in_use: bool,
    pub ep0_open: bool,
    /// usbport's address for the device (0 until SET_ADDRESS), the key its
    /// endpoints are found by.
    pub usb_address: u32,
    /// Address the controller assigned (informational).
    pub hw_address: u32,
    /// Where the device sits (usbport's root port numbering, the route string
    /// below it) and its xHCI Protocol Speed ID.
    pub root_port: u32,
    pub route: u32,
    pub speed: u32,
    /// For a low/full-speed device below a high-speed hub: that hub's slot and
    /// the hub port leading here (Slot Context "Parent Hub Slot ID"/"Parent
    /// Port Number"), else 0.
    pub tt_slot: u32,
    pub tt_port: u32,
    /// Set once the device's hub descriptor has been seen: bNbrPorts and the
    /// TT think time, applied to the Slot Context by the next Configure Endpoint.
    pub hub_ports: u32,
    pub hub_ttt: u32,
    pub ep0_mps: u32,
    /// Highest DCI with a configured endpoint (Slot Context "Context Entries").
    pub max_dci: u32,
    pub ep0_ring: Ring,
}

/// usbport's per-endpoint miniport area (MiniPortEndpointSize).
#[repr(C)]
pub struct XhciEndpoint {
    pub props: USBPORT_ENDPOINT_PROPERTIES,
    pub slot: u32,
    pub dci: u32,
    pub is_control: bool,
    pub state: u32,
    /// The ring in use: the slot's EP0 ring, or `own_ring` below.
    pub ring: *mut Ring,
    pub own_ring: Ring,
    /// Ring index just past the last completed transfer (the software dequeue).
    pub deq: u32,
    pub halted: bool,
    pub lock: usize,
    pub pending: [*mut XhciTransfer; MAX_TDS],
    pub head: u32,
    pub count: u32,
    pub payload_va: *mut u8,
    pub payload_pa: u64,
    pub payload_used: u32,
}

/// usbport's per-transfer miniport area (MiniPortTransferSize).
#[repr(C)]
pub struct XhciTransfer {
    pub params: *mut USBPORT_TRANSFER_PARAMETERS,
    pub requested: u32,
    pub transferred: u32,
    pub cc: u32,
    pub short_seen: bool,
    pub done: AtomicBool,
    /// Where the transfer sits on the ring.
    pub first: u32,
    pub first_cycle: u32,
    pub ntrbs: u32,
    pub end: u32,
    pub last_pa: u64,
    pub ndata: u32,
    pub data_pa: [u64; MAX_DATA_TRBS],
    pub data_len: [u32; MAX_DATA_TRBS],
    pub caller_va: *mut u8,
    pub inbound: bool,
    pub payload_first: u32,
    pub payload_pages: u32,
}

/// MiniportSpinLock serializes allocation/release; event handling never touches
/// the payload arena. Each allocation owns whole pages, including edge lines.
unsafe fn prepare_payload(ep: &mut XhciEndpoint, td: &mut XhciTransfer, caller: PVOID, inbound: bool) -> bool {
    td.inbound = inbound;
    td.caller_va = caller as *mut u8;
    if td.requested == 0 { return true; }
    if td.requested as usize > PAYLOAD_BYTES || caller.is_null() || ep.payload_va.is_null() {
        return false;
    }
    let pages = (td.requested + PAGE as u32 - 1) / PAGE as u32;
    let mask = (1u32 << pages) - 1;
    let Some(first) = (0..=PAYLOAD_PAGES - pages).find(|&i| ep.payload_used & (mask << i) == 0) else {
        return false;
    };
    ep.payload_used |= mask << first;
    td.payload_first = first;
    td.payload_pages = pages;
    let va = ep.payload_va.add(first as usize * PAGE);
    if !inbound {
        core::ptr::copy_nonoverlapping(td.caller_va, va, td.requested as usize);
    }
    // For IN, this also evicts any dirty old content before device ownership.
    // No CPU writes touch these private pages until completion or a proven stop.
    dma_clean(va, pages as usize * PAGE);
    true
}

unsafe fn finish_payload(ep: &mut XhciEndpoint, td: &mut XhciTransfer) {
    if td.payload_pages == 0 { return; }
    let va = ep.payload_va.add(td.payload_first as usize * PAGE);
    if td.inbound {
        dma_invalidate(va, td.payload_pages as usize * PAGE);
        core::ptr::copy_nonoverlapping(va, td.caller_va, td.transferred.min(td.requested) as usize);
    }
    ep.payload_used &= !(((1u32 << td.payload_pages) - 1) << td.payload_first);
    td.payload_pages = 0;
}

/// Called only after HCHalted is observed, under the event consumer's lock.
pub unsafe fn controller_failed(ep: &mut XhciEndpoint) {
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut ep.lock, &mut irql);
    ep.halted = true;
    for i in 0..ep.count {
        let td = &mut *ep.pending[((ep.head + i) as usize) % MAX_TDS];
        if !td.done.load(Ordering::Acquire) {
            td.cc = 0;
            td.done.store(true, Ordering::Release);
        }
    }
    KeReleaseSpinLock(&mut ep.lock, irql);
}

// ---- contexts ----

#[inline]
fn slot_ctx_offset(slot: u32) -> usize {
    // Two pages per slot after the fixed regions: device context, then EP0 ring.
    SLOTS_OFFSET + (slot as usize - 1) * 2 * PAGE
}

unsafe fn dev_ctx(x: &XhciExtension, slot: u32) -> (*mut u8, u64) {
    let off = slot_ctx_offset(slot);
    (x.va.add(off), x.pa + off as u64)
}

/// Input context entry: 0 = Input Control, 1 = Slot, DCI + 1 = endpoint.
unsafe fn in_ctx(x: &XhciExtension, idx: u32) -> *mut u32 {
    x.input_ctx_va.add(idx as usize * x.ctx_size as usize) as *mut u32
}

/// Output (device) context entry: 0 = Slot, DCI = endpoint.
unsafe fn out_ctx(x: &XhciExtension, slot: u32, idx: u32) -> *mut u32 {
    dev_ctx(x, slot).0.add(idx as usize * x.ctx_size as usize) as *mut u32
}

unsafe fn input_clear(x: &XhciExtension) {
    core::ptr::write_bytes(x.input_ctx_va, 0, 33 * x.ctx_size as usize);
}

unsafe fn input_publish(x: &XhciExtension) {
    dma_clean(x.input_ctx_va, 33 * x.ctx_size as usize);
}

/// usbport's USB_DEVICE_SPEED (0 low, 1 full, 2 high) as an xHCI Protocol Speed ID.
fn psi(usb_speed: u32) -> u32 {
    match usb_speed {
        0 => SPEED_LOW,
        2 => SPEED_HIGH,
        _ => SPEED_FULL,
    }
}

/// Fill an endpoint context (6.2.3).
unsafe fn write_ep_ctx(c: *mut u32, ep_type: u32, mps: u32, interval: u32, ring_pa: u64, cycle: u32, avg_len: u32, esit: u32) {
    let cerr = if ep_type == EP_ISOCH_IN || ep_type == EP_ISOCH_OUT { 0 } else { 3 };
    *c.add(0) = interval << 16;
    *c.add(1) = (cerr << 1) | (ep_type << 3) | (mps << 16);
    let deq = ring_pa | cycle as u64;
    *c.add(2) = deq as u32;
    *c.add(3) = (deq >> 32) as u32;
    *c.add(4) = avg_len | (esit << 16);
}

/// Fill the input Slot Context (6.2.2).
unsafe fn write_slot_ctx(x: &XhciExtension, s: &Slot) {
    let c = in_ctx(x, 1);
    *c.add(0) = s.route | (s.speed << 20) | (s.max_dci.max(1) << 27);
    *c.add(1) = crate::roothub::hw_port(x, s.root_port as u16) << 16;
    *c.add(2) = s.tt_slot | (s.tt_port << 8);
    *c.add(3) = 0;
    apply_hub_fields(c, s);
}

/// Hub, Number of Ports and TT Think Time for a slot known to be a hub. A
/// high-speed hub's controller-side TT handling (split transactions for the
/// low/full-speed devices below it) depends on them.
unsafe fn apply_hub_fields(c: *mut u32, s: &Slot) {
    if s.hub_ports != 0 {
        *c.add(0) |= 1 << 26;
        *c.add(1) = (*c.add(1) & 0x00FF_FFFF) | (s.hub_ports << 24);
        *c.add(2) = (*c.add(2) & !(3 << 16)) | (s.hub_ttt << 16);
    }
}

// ---- commands (each takes the command lock around the shared input context) ----

unsafe fn enable_slot(x: &mut XhciExtension) -> Option<u32> {
    let r = command(x, 0, 0, TRB_ENABLE_SLOT << TRB_TYPE_SHIFT);
    if r.code != CC_SUCCESS || r.slot == 0 || r.slot as usize > MAX_SLOTS {
        xlog!("Enable Slot failed: cc %u slot %u", r.code, r.slot);
        if r.code == CC_SUCCESS && r.slot != 0 {
            command(x, 0, 0, (TRB_DISABLE_SLOT << TRB_TYPE_SHIFT) | (r.slot << 24));
        }
        return None;
    }
    Some(r.slot)
}

unsafe fn disable_slot(x: &mut XhciExtension, slot: u32) {
    let r = command(x, 0, 0, (TRB_DISABLE_SLOT << TRB_TYPE_SHIFT) | (slot << 24));
    if r.code != CC_SUCCESS && !x.failed.load(Ordering::Acquire) {
        device_command_failed(x, slot, 0, TRB_DISABLE_SLOT, r.code);
    }
    core::ptr::write_volatile(x.dcbaa.add(slot as usize), 0);
    dma_clean(x.dcbaa.add(slot as usize) as *const u8, 8);
    x.slots[slot as usize].in_use = false;
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.evt_lock, &mut irql);
    x.eps[slot as usize] = [core::ptr::null_mut(); 32];
    KeReleaseSpinLock(&mut x.evt_lock, irql);
    xlog!("slot %u disabled (cc %u)", slot, r.code);
}

/// Address Device (4.6.5). With `bsr` the device stays at address 0 (Default
/// state) so usbport can read its descriptor first; without it the controller
/// sends SET_ADDRESS itself. The EP0 dequeue pointer is the ring's current
/// enqueue position, so a second Address Device never replays old TDs.
unsafe fn address_device(x: &mut XhciExtension, slot: u32, bsr: bool) -> u32 {
    let irql = lock_commands(x);
    input_clear(x);
    let icc = in_ctx(x, 0);
    *icc.add(1) = 0b11; // add Slot + EP0
    let s = &x.slots[slot as usize];
    write_slot_ctx(x, s);
    let ring = &s.ep0_ring;
    write_ep_ctx(in_ctx(x, 2), EP_CONTROL, s.ep0_mps, 0, ring.pa_of(ring.enqueue), ring.cycle, 8, 0);
    input_publish(x);
    let ctl = (TRB_ADDRESS_DEVICE << TRB_TYPE_SHIFT) | (slot << 24) | if bsr { TRB_BSR } else { 0 };
    let r = command_locked(x, x.input_ctx_pa, 0, ctl);
    unlock_commands(x, irql);
    r.code
}

/// Evaluate Context for a new EP0 max packet size (4.6.7).
unsafe fn evaluate_ep0(x: &mut XhciExtension, slot: u32, mps: u32) -> u32 {
    let irql = lock_commands(x);
    input_clear(x);
    *in_ctx(x, 0).add(1) = 0b10; // EP0 only
    let ring = &x.slots[slot as usize].ep0_ring;
    write_ep_ctx(in_ctx(x, 2), EP_CONTROL, mps, 0, ring.pa_of(ring.enqueue), ring.cycle, 8, 0);
    input_publish(x);
    let r = command_locked(x, x.input_ctx_pa, 0, (TRB_EVALUATE_CONTEXT << TRB_TYPE_SHIFT) | (slot << 24));
    unlock_commands(x, irql);
    r.code
}

/// xHCI Interval (a power of two in 125 us units) from usbport's Period, which is
/// already normalised to a power-of-two number of 1 ms frames.
fn xhci_interval(period_ms: u32) -> u32 {
    let p = period_ms.clamp(1, 32);
    31 - p.leading_zeros() + 3
}

/// Configure Endpoint (4.6.6): add or drop one endpoint, with the Slot Context's
/// Context Entries kept at the highest live DCI.
unsafe fn configure_endpoint(x: &mut XhciExtension, ep: &XhciEndpoint, add: bool) -> u32 {
    let slot = ep.slot;
    let irql = lock_commands(x);
    let old_max_dci = x.slots[slot as usize].max_dci;
    input_clear(x);
    let icc = in_ctx(x, 0);
    if add {
        *icc.add(1) = 1 | (1 << ep.dci);
        // usbport opens a SELECT_INTERFACE's new pipes before it closes the old
        // ones, so the DCI can still be live: Drop and Add together replace it.
        if !x.eps[slot as usize][ep.dci as usize].is_null() {
            *icc.add(0) = 1 << ep.dci;
        }
        let s = &mut x.slots[slot as usize];
        s.max_dci = s.max_dci.max(ep.dci);
    } else {
        *icc.add(0) = 1 << ep.dci;
        *icc.add(1) = 1;
        let mut max = 1;
        for d in 2..32u32 {
            if d != ep.dci && !x.eps[slot as usize][d as usize].is_null() {
                max = d;
            }
        }
        x.slots[slot as usize].max_dci = max;
    }
    // Slot Context: start from the controller's copy, update Context Entries.
    let out = out_ctx(x, slot, 0);
    dma_invalidate(out as *const u8, x.ctx_size as usize);
    let sc = in_ctx(x, 1);
    for i in 0..4 {
        *sc.add(i) = core::ptr::read_volatile(out.add(i));
    }
    *sc.add(0) = (*sc.add(0) & !(0x1F << 27)) | (x.slots[slot as usize].max_dci << 27);
    *sc.add(3) = 0; // slot state/address are output-only
    apply_hub_fields(sc, &x.slots[slot as usize]);

    if add {
        let p = &ep.props;
        let in_dir = p.EndpointAddress & 0x80 != 0;
        let (ty, interval, avg) = match p.TransferType {
            USBPORT_TRANSFER_TYPE_CONTROL => (EP_CONTROL, 0, 8),
            USBPORT_TRANSFER_TYPE_BULK => (if in_dir { EP_BULK_IN } else { EP_BULK_OUT }, 0, 1024),
            USBPORT_TRANSFER_TYPE_INTERRUPT => (
                if in_dir { EP_INT_IN } else { EP_INT_OUT },
                xhci_interval(p.Period as u32),
                p.TotalMaxPacketSize as u32,
            ),
            _ => (if in_dir { EP_ISOCH_IN } else { EP_ISOCH_OUT }, xhci_interval(p.Period as u32), p.TotalMaxPacketSize as u32),
        };
        let esit = if ty == EP_INT_IN || ty == EP_INT_OUT { p.TotalMaxPacketSize as u32 } else { 0 };
        let ring = &*ep.ring;
        write_ep_ctx(in_ctx(x, ep.dci + 1), ty, p.MaxPacketSize & 0x7FF, interval, ring.pa, ring.cycle, avg, esit);
    }
    input_publish(x);
    let r = command_locked(x, x.input_ctx_pa, 0, (TRB_CONFIGURE_ENDPOINT << TRB_TYPE_SHIFT) | (slot << 24));
    if r.code != CC_SUCCESS {
        x.slots[slot as usize].max_dci = old_max_dci;
    }
    unlock_commands(x, irql);
    r.code
}

unsafe fn ep_command(x: &mut XhciExtension, ep: &XhciEndpoint, trb: u32) -> u32 {
    command(x, 0, 0, (trb << TRB_TYPE_SHIFT) | (ep.slot << 24) | (ep.dci << 16)).code
}

unsafe fn set_dequeue(x: &mut XhciExtension, ep: &XhciEndpoint, idx: u32, cycle: u32) -> u32 {
    let pa = (*ep.ring).pa_of(idx) | cycle as u64;
    command(x, pa, 0, (TRB_SET_TR_DEQUEUE << TRB_TYPE_SHIFT) | (ep.slot << 24) | (ep.dci << 16)).code
}

/// Where the controller should resume: the first unfinished transfer, or the
/// enqueue point when none is left. Caller holds `ep.lock`.
unsafe fn resume_point(ep: &XhciEndpoint) -> (u32, u32) {
    for i in 0..ep.count {
        let td = &*ep.pending[((ep.head + i) as usize) % MAX_TDS];
        if !td.done.load(Ordering::SeqCst) {
            return (td.first, td.first_cycle);
        }
    }
    let r = &*ep.ring;
    (r.enqueue, r.cycle)
}

/// Recover a halted endpoint (4.6.8): Reset Endpoint, then move the dequeue
/// pointer past the transfer that failed.
unsafe fn recover(x: &mut XhciExtension, ep: &mut XhciEndpoint) -> bool {
    let cc = ep_command(x, ep, TRB_RESET_ENDPOINT);
    // Context State Error: the endpoint was not halted after all (a Stop won
    // the race); the dequeue move below is still what it needs.
    if cc != CC_SUCCESS && !(cc == CC_CONTEXT_STATE && ep_state(x, ep) != EP_STATE_HALTED) {
        device_command_failed(x, ep.slot, ep.dci, TRB_RESET_ENDPOINT, cc);
        return false;
    }
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut ep.lock, &mut irql);
    let (idx, cyc) = resume_point(ep);
    KeReleaseSpinLock(&mut ep.lock, irql);
    let cc2 = set_dequeue(x, ep, idx, cyc);
    if cc2 != CC_SUCCESS {
        device_command_failed(x, ep.slot, ep.dci, TRB_SET_TR_DEQUEUE, cc2);
        return false;
    }
    ep.halted = false;
    xlog!("slot %u ep %u: halt cleared (reset cc %u, dequeue cc %u)", ep.slot, ep.dci, cc, cc2);
    ring_doorbell(x, ep.slot, ep.dci);
    true
}

// ---- transfer bookkeeping ----

unsafe fn queue_td(ep: &mut XhciEndpoint, td: *mut XhciTransfer) -> bool {
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut ep.lock, &mut irql);
    let ok = (ep.count as usize) < MAX_TDS;
    if ok {
        ep.pending[((ep.head + ep.count) as usize) % MAX_TDS] = td;
        ep.count += 1;
    }
    KeReleaseSpinLock(&mut ep.lock, irql);
    ok
}

fn usbd_status(cc: u32) -> USBD_STATUS {
    match cc {
        CC_SUCCESS | CC_SHORT_PACKET => USBD_STATUS_SUCCESS,
        CC_STALL => USBD_STATUS_STALL_PID,
        CC_BABBLE => USBD_STATUS_BABBLE_DETECTED,
        CC_USB_TRANSACTION | CC_SPLIT_TRANSACTION => USBD_STATUS_XACT_ERROR,
        CC_DATA_BUFFER => USBD_STATUS_DATA_BUFFER_ERROR,
        CC_TRB => USBD_STATUS_INTERNAL_HC_ERROR,
        0 => USBD_STATUS_DEV_NOT_RESPONDING,
        _ => USBD_STATUS_CRC,
    }
}

/// Completion codes after which the controller has halted the endpoint (4.8.3,
/// 4.10.2). A split transaction error is what a low/full-speed device below a
/// hub produces once it is unplugged.
fn halts(cc: u32) -> bool {
    matches!(cc, CC_STALL | CC_BABBLE | CC_USB_TRANSACTION | CC_SPLIT_TRANSACTION | CC_TRB)
}

const EP_STATE_RUNNING: u32 = 1;
const EP_STATE_HALTED: u32 = 2;
const EP_STATE_ERROR: u32 = 4;

/// The controller's own view of an endpoint: Endpoint Context state (6.2.3),
/// 0 disabled, 1 running, 2 halted, 3 stopped, 4 error. With no context memory
/// to read, say Running: callers treat that as "may still be doing DMA".
unsafe fn ep_state(x: &XhciExtension, ep: &XhciEndpoint) -> u32 {
    if x.va.is_null() || ep.slot == 0 {
        return EP_STATE_RUNNING;
    }
    let c = out_ctx(x, ep.slot, ep.dci);
    dma_invalidate(c as *const u8, x.ctx_size as usize);
    core::ptr::read_volatile(c) & 7
}

/// An endpoint or slot command failed. That is a statement about one device
/// (typically one that was just unplugged, whose endpoint the controller has
/// already halted), not about the controller: the controller is dead only if it
/// says so in USBSTS, or a command never completes (`command` handles that).
/// Failing the whole controller here once took every other device on the bus
/// down with an unplugged keyboard.
unsafe fn device_command_failed(x: &mut XhciExtension, slot: u32, dci: u32, trb: u32, cc: u32) {
    let sts = rd32(x.op, USBSTS);
    if sts & (USBSTS_HCH | USBSTS_HCE | USBSTS_HSE) != 0 {
        xlog!("slot %u ep %u: command %u failed, cc %u, and the controller stopped (USBSTS %x)", slot, dci, trb, cc, sts);
        if !x.failed.load(Ordering::Acquire) { fail_controller(x); }
        return;
    }
    xlog!("slot %u ep %u: command %u failed, cc %u; this device only", slot, dci, trb, cc);
}

/// A command meant to take an endpoint's transfers back (Stop Endpoint, Set TR
/// Dequeue, dropping it) failed. Its buffers go back to usbport when we return,
/// so everything turns on whether the controller can still be moving data
/// through it: if its context says Running, the only safe answer is to stop the
/// whole controller, as before. Anything else (halted, as for a device unplugged
/// mid-transfer; stopped; disabled) cannot DMA, so only this endpoint is done.
unsafe fn endpoint_command_failed(x: &mut XhciExtension, ep: &mut XhciEndpoint, trb: u32, cc: u32) {
    let state = ep_state(x, ep);
    if state == EP_STATE_RUNNING {
        xlog!("slot %u ep %u: command %u failed, cc %u, endpoint still running; stopping the controller", ep.slot, ep.dci, trb, cc);
        if !x.failed.load(Ordering::Acquire) { fail_controller(x); }
        return;
    }
    device_command_failed(x, ep.slot, ep.dci, trb, cc);
    ep.halted = true;
}

/// Record a Transfer Event against the transfer it belongs to. Runs inside the
/// event-ring consumer; completion to usbport happens later in PollEndpoint.
pub unsafe fn on_transfer_event(x: &mut XhciExtension, trb_pa: u64, status: u32, control: u32) {
    let slot = (control >> 24) as usize;
    let dci = ((control >> 16) & 0x1F) as usize;
    let cc = status >> 24;
    let residual = status & 0x00FF_FFFF;
    if slot == 0 || slot > MAX_SLOTS || x.eps[slot][dci].is_null() {
        xlog!("transfer event for closed slot %u ep %u (cc %u)", slot as u32, dci as u32, cc);
        return;
    }
    let ep = &mut *x.eps[slot][dci];
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut ep.lock, &mut irql);
    for i in 0..ep.count {
        let td = &mut *ep.pending[((ep.head + i) as usize) % MAX_TDS];
        if td.done.load(Ordering::SeqCst) {
            continue;
        }
        let data_hit = (0..td.ndata as usize).find(|&k| td.data_pa[k] == trb_pa);
        let last_hit = trb_pa == td.last_pa;
        // Errors (and Stop Endpoint events) may point at Setup or a Link TRB,
        // neither of which appears in data_pa/last_pa.
        let ring = &*ep.ring;
        let offset = trb_pa.wrapping_sub(ring.pa);
        let index = (offset / TRB_SIZE as u64) as u32;
        let in_td = offset < PAGE as u64 && offset % TRB_SIZE as u64 == 0 &&
            if index == RING_USABLE {
                td.first + td.ntrbs >= RING_USABLE && td.ntrbs != 0
            } else {
                (index + RING_USABLE - td.first) % RING_USABLE < td.ntrbs
            };
        if !in_td {
            continue;
        }
        // Stopping a ring is not completion of its current request. In
        // particular, aborting a later queued request must not fail the head.
        if matches!(cc, CC_STOPPED | CC_STOPPED_LENGTH_INVALID | CC_STOPPED_SHORT_PACKET) {
            if cc == CC_STOPPED_SHORT_PACKET {
                td.transferred = residual.min(td.requested);
                td.short_seen = true;
            } else if let Some(k) = data_hit {
                let before: u32 = td.data_len[..k].iter().sum();
                td.transferred = before + if cc == CC_STOPPED {
                    td.data_len[k] - residual.min(td.data_len[k])
                } else { 0 };
            }
            break;
        }
        if let Some(k) = data_hit {
            if cc == CC_SHORT_PACKET || cc == CC_SUCCESS {
                let before: u32 = td.data_len[..k].iter().sum();
                td.transferred = before + td.data_len[k] - residual.min(td.data_len[k]);
                td.short_seen |= cc == CC_SHORT_PACKET;
            }
        }
        let finished = if cc != CC_SUCCESS && cc != CC_SHORT_PACKET {
            true // error: the TD is over wherever it happened
        } else if last_hit {
            true
        } else {
            // A short packet mid-TD. Bulk/interrupt TDs end here; a control TD
            // goes on to its Status stage, which reports again.
            cc == CC_SHORT_PACKET && !ep.is_control
        };
        if finished {
            if last_hit && !td.short_seen && (cc == CC_SUCCESS || cc == CC_SHORT_PACKET) && data_hit.is_none() {
                td.transferred = td.requested;
            }
            td.cc = cc;
            if halts(cc) {
                ep.halted = true;
            }
            td.done.store(true, Ordering::SeqCst);
        }
        break;
    }
    KeReleaseSpinLock(&mut ep.lock, irql);
}

// ---- miniport callbacks ----

/// Pair lookup publication/removal with the event consumer's lock. usbport's
/// CloseEndpoint caller frees the endpoint immediately after we return.
unsafe fn set_event_endpoint(x: &mut XhciExtension, slot: usize, dci: usize, ep: *mut XhciEndpoint) {
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut x.evt_lock, &mut irql);
    x.eps[slot][dci] = ep;
    KeReleaseSpinLock(&mut x.evt_lock, irql);
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_QueryEndpointRequirements(
    _xhciExtension: PVOID,
    endpointParameters: PUSBPORT_ENDPOINT_PROPERTIES,
    EndpointRequirements: PUSBPORT_ENDPOINT_REQUIREMENTS,
) {
    let p = &*endpointParameters;
    let req = &mut *EndpointRequirements;
    let ep0 = p.TransferType == USBPORT_TRANSFER_TYPE_CONTROL && p.EndpointAddress & 0x0F == 0;
    // EP0's ring lives with its slot; every endpoint also gets an isolated
    // payload arena from usbport's physically contiguous common-buffer allocator.
    req.HeaderBufferSize = (PAYLOAD_BYTES + if ep0 { 0 } else { PAGE }) as u32;
    req.MaxTransferSize = match p.TransferType {
        USBPORT_TRANSFER_TYPE_INTERRUPT => 0x1000,
        _ => 0x10000,
    };
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_OpenEndpoint(
    xhciExtension: PVOID,
    endpointParameters: PUSBPORT_ENDPOINT_PROPERTIES,
    xhciEndpoint: PVOID,
) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    if x.failed.load(Ordering::Acquire) { return MP_STATUS_HW_ERROR; }
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    core::ptr::write_bytes(ep as *mut XhciEndpoint as *mut u8, 0, core::mem::size_of::<XhciEndpoint>());
    core::ptr::copy_nonoverlapping(endpointParameters, &mut ep.props, 1);
    let p = &*core::ptr::addr_of!(ep.props);
    let addr = p.DeviceAddress as u32;
    ep.is_control = p.TransferType == USBPORT_TRANSFER_TYPE_CONTROL;
    ep.dci = if ep.is_control { dci(p.EndpointAddress as u32 & 0x0F) } else { dci(p.EndpointAddress as u32) };
    ep.state = USBPORT_ENDPOINT_PAUSED;
    let ring_bytes = if ep.dci == 1 { 0 } else { PAGE };
    if p.BufferVA == 0 || p.BufferLength < (ring_bytes + PAYLOAD_BYTES) as u32 {
        return MP_STATUS_NO_RESOURCES;
    }
    ep.payload_va = (p.BufferVA as *mut u8).add(ring_bytes);
    ep.payload_pa = p.BufferPA as u64 + ring_bytes as u64;

    if p.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
        xlog!("isochronous endpoints are not supported yet (dev %u ep %x)", addr, p.EndpointAddress as u32);
        return MP_STATUS_NOT_SUPPORTED;
    }

    if ep.dci == 1 {
        let slot;
        if addr == 0 {
            // A new device; usbport says where (see USBPORT_SetXhciTopology).
            let root_port = p.Reserved3 >> 24;
            let route = p.Reserved3 & 0xF_FFFF;
            let tt_hub = (p.Reserved6 >> 8) & 0xFF;
            let tt_port = p.Reserved6 & 0xFF;
            if root_port == 0 {
                xlog!("new device (port %u): usbport gave no root port", p.PortNumber as u32);
                return MP_STATUS_ERROR;
            }
            let tt_slot = if tt_hub == 0 {
                0
            } else {
                match (1..=MAX_SLOTS as u32).find(|&s| x.slots[s as usize].in_use && x.slots[s as usize].usb_address == tt_hub) {
                    Some(s) => s,
                    None => {
                        xlog!("new device below hub %u: no slot has that address", tt_hub);
                        return MP_STATUS_ERROR;
                    }
                }
            };
            // A slot whose default pipe is closed and that sat at the same place
            // belongs to the device that was there before (or to this one's
            // previous attempt): free it.
            for s in 1..=MAX_SLOTS as u32 {
                let st = &x.slots[s as usize];
                if st.in_use && !st.ep0_open && st.root_port == root_port && st.route == route {
                    disable_slot(x, s);
                }
            }
            slot = match enable_slot(x) {
                Some(s) => s,
                None => return MP_STATUS_NO_RESOURCES,
            };
            // A device on a root port has its speed in PORTSC, which also
            // knows SuperSpeed; usbport says high speed for those (roothub.rs).
            let port_speed = if route == 0 {
                crate::roothub::read_portsc(x, root_port as u16).map_or(0, |v| (v & PORTSC_SPEED_MASK) >> PORTSC_SPEED_SHIFT)
            } else {
                0
            };
            let speed = if port_speed != 0 { port_speed } else { psi(p.DeviceSpeed as u32) };
            let (ctx_va, ctx_pa) = dev_ctx(x, slot);
            core::ptr::write_bytes(ctx_va, 0, PAGE);
            dma_clean(ctx_va, PAGE);
            let st = &mut x.slots[slot as usize];
            st.in_use = true;
            st.usb_address = 0;
            st.hw_address = 0;
            st.root_port = root_port;
            st.route = route;
            st.tt_slot = tt_slot;
            st.tt_port = tt_port;
            st.hub_ports = 0;
            st.hub_ttt = 0;
            st.speed = speed;
            // SuperSpeed EP0 is always 512 bytes (USB 3.2 9.6.1).
            st.ep0_mps = if speed == SPEED_SUPER {
                512
            } else if p.MaxPacketSize != 0 {
                p.MaxPacketSize
            } else {
                8
            };
            st.max_dci = 1;
            st.ep0_ring.init(ctx_va.add(PAGE), ctx_pa + PAGE as u64);
            core::ptr::write_volatile(x.dcbaa.add(slot as usize), ctx_pa);
            dma_clean(x.dcbaa.add(slot as usize) as *const u8, 8);

            let cc = address_device(x, slot, true);
            if cc != CC_SUCCESS {
                xlog!("port %u route %x: Address Device (BSR) failed, cc %u", root_port, route, cc);
                disable_slot(x, slot);
                return MP_STATUS_ERROR;
            }
            xlog!("port %u route %x: new device, slot %u, speed %u, mps %u, tt slot %u port %u",
                root_port, route, slot, x.slots[slot as usize].speed, x.slots[slot as usize].ep0_mps, tt_slot, tt_port);
        } else {
            slot = match (1..=MAX_SLOTS as u32).find(|&s| x.slots[s as usize].in_use && x.slots[s as usize].usb_address == addr) {
                Some(s) => s,
                None => {
                    xlog!("open EP0 at address %u: no slot has that address", addr);
                    return MP_STATUS_ERROR;
                }
            };
            let mps = if p.TotalMaxPacketSize != 0 { p.TotalMaxPacketSize as u32 } else { p.MaxPacketSize };
            if mps != x.slots[slot as usize].ep0_mps {
                let cc = evaluate_ep0(x, slot, mps);
                xlog!("slot %u: EP0 max packet %u (cc %u)", slot, mps, cc);
                if cc != CC_SUCCESS {
                    return MP_STATUS_ERROR;
                }
                x.slots[slot as usize].ep0_mps = mps;
            }
        }
        ep.slot = slot;
        ep.ring = &mut x.slots[slot as usize].ep0_ring;
        let r = &*ep.ring;
        ep.deq = r.enqueue;
        x.slots[slot as usize].ep0_open = true;
        set_event_endpoint(x, slot as usize, 1, ep);
        return MP_STATUS_SUCCESS;
    }

    // Any other endpoint: its ring is the page usbport allocated for it.
    let slot = match (1..=MAX_SLOTS as u32).find(|&s| x.slots[s as usize].in_use && x.slots[s as usize].usb_address == addr) {
        Some(s) => s,
        None => {
            xlog!("open ep %x at address %u: no slot has that address", p.EndpointAddress as u32, addr);
            return MP_STATUS_ERROR;
        }
    };
    if p.BufferVA == 0 || p.BufferLength < PAGE as u32 {
        return MP_STATUS_NO_RESOURCES;
    }
    ep.slot = slot;
    ep.own_ring.init(p.BufferVA as *mut u8, p.BufferPA as u64);
    ep.ring = &mut ep.own_ring;
    ep.deq = 0;
    let cc = configure_endpoint(x, ep, true);
    if cc != CC_SUCCESS {
        xlog!("slot %u: Configure Endpoint (add ep %x) failed, cc %u", slot, p.EndpointAddress as u32, cc);
        return MP_STATUS_ERROR;
    }
    set_event_endpoint(x, slot as usize, ep.dci as usize, ep);
    xlog!("slot %u: ep %x open (dci %u, type %u, mps %u, period %u)", slot, p.EndpointAddress as u32, ep.dci, p.TransferType, p.MaxPacketSize, p.Period as u32);
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_ReopenEndpoint(
    xhciExtension: PVOID,
    endpointParameters: PUSBPORT_ENDPOINT_PROPERTIES,
    xhciEndpoint: PVOID,
) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    let p = &*endpointParameters;
    ep.props.DeviceAddress = p.DeviceAddress;
    ep.props.TotalMaxPacketSize = p.TotalMaxPacketSize;
    ep.props.MaxPacketSize = p.MaxPacketSize;
    if ep.dci == 1 && ep.slot != 0 {
        let mps = p.TotalMaxPacketSize as u32;
        if mps != 0 && mps != x.slots[ep.slot as usize].ep0_mps {
            if evaluate_ep0(x, ep.slot, mps) != CC_SUCCESS {
                return MP_STATUS_ERROR;
            }
            x.slots[ep.slot as usize].ep0_mps = mps;
        }
    }
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_CloseEndpoint(xhciExtension: PVOID, xhciEndpoint: PVOID, _IsDoDisablePeriodic: BOOLEAN) {
    let x = ext_of(xhciExtension);
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    let slot = ep.slot as usize;
    if slot == 0 || !x.slots[slot].in_use {
        return;
    }
    if ep.dci == 1 {
        // Keep the slot: usbport closes EP0 to re-open it at the new address.
        x.slots[slot].ep0_open = false;
        set_event_endpoint(x, slot, 1, core::ptr::null_mut());
        return;
    }
    // A pipe superseded by a later open of the same DCI (SELECT_INTERFACE):
    // the controller already runs the replacement, so dropping would kill it.
    if x.eps[slot][ep.dci as usize] != ep as *mut XhciEndpoint {
        return;
    }
    set_event_endpoint(x, slot, ep.dci as usize, core::ptr::null_mut());
    let cc = configure_endpoint(x, ep, false);
    if cc != CC_SUCCESS {
        endpoint_command_failed(x, ep, TRB_CONFIGURE_ENDPOINT, cc);
    }
}

/// Build one TD on the endpoint's ring.
#[no_mangle]
pub unsafe extern "system" fn XHCI_SubmitTransfer(
    xhciExtension: PVOID,
    xhciEndpoint: PVOID,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    xhciTransfer: PVOID,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    let td_ptr = xhciTransfer as *mut XhciTransfer;
    core::ptr::write_bytes(td_ptr as *mut u8, 0, core::mem::size_of::<XhciTransfer>());
    let td = &mut *td_ptr;
    let params = &*TransferParameters;
    td.params = TransferParameters;
    td.requested = params.TransferBufferLength;

    if ep.slot == 0 || ep.ring.is_null() {
        return MP_STATUS_FAILURE;
    }
    // usbport serializes Submit/Poll/Abort with MiniportSpinLock. The event
    // consumer only marks TDs done; it cannot consume this reserved capacity.
    // Reject BEFORE changing the ring or issuing SET_ADDRESS.
    if ep.count as usize == MAX_TDS {
        return MP_STATUS_FAILURE;
    }
    if x.failed.load(Ordering::Acquire) {
        // Returning FAILURE would have usbport retry forever without submitting
        // (and hence without starting its request timeout). Complete it instead.
        td.cc = 0;
        td.end = (*ep.ring).enqueue;
        td.done.store(true, Ordering::Release);
        queue_td(ep, td_ptr);
        x.pending_transfer.store(true, Ordering::Release);
        return MP_STATUS_SUCCESS;
    }
    if ep.halted && ep.is_control {
        // A STALL on a control pipe is a per-request answer, not a broken pipe:
        // clear it before the next request, as EHCI hardware does by itself.
        if !recover(x, ep) {
            return MP_STATUS_FAILURE;
        }
    }

    let sg = &*SgList;
    if td.requested as usize > PAYLOAD_BYTES {
        return MP_STATUS_FAILURE;
    }
    let n_sg = (td.requested as usize + PAGE - 1) / PAGE;
    let needed = if ep.is_control { 2 + n_sg as u32 } else { n_sg.max(1) as u32 };
    if (*ep.ring).space(ep.deq) < needed { return MP_STATUS_FAILURE; }
    let setup = core::ptr::read_unaligned(&params.SetupPacket as *const _ as *const [u8; 8]);
    let inbound = if ep.is_control { setup[0] & 0x80 != 0 } else { ep.props.EndpointAddress & 0x80 != 0 };
    if !prepare_payload(ep, td, sg.MappedSystemVa, inbound) { return MP_STATUS_FAILURE; }
    // MappedSystemVa already includes the sub-request offset for usbport's split
    // transfers. Rebuild S/G against our private pages, not the caller's physical
    // addresses. Page-sized TRBs cannot cross xHCI's 64 KiB buffer boundary.
    let mut local_sg: [USBPORT_SCATTER_GATHER_ELEMENT; MAX_DATA_TRBS] = core::mem::zeroed();
    for (i, e) in local_sg[..n_sg].iter_mut().enumerate() {
        e.SgPhysicalAddress.QuadPart = (ep.payload_pa + (td.payload_first as usize * PAGE + i * PAGE) as u64) as i64;
        e.SgTransferLength = (td.requested - (i * PAGE) as u32).min(PAGE as u32);
    }
    let elems = &local_sg[..n_sg];
    let mps = (ep.props.MaxPacketSize & 0x7FF).max(1);

    if ep.is_control {
        let request_type = setup[0];
        let request = setup[1];
        let w_value = u16::from_le_bytes([setup[2], setup[3]]) as u32;

        if ep.dci == 1 && request_type == 0x00 && request == 0x05 {
            // SET_ADDRESS: the controller addresses the device itself.
            let slot = ep.slot;
            // usbport reuses an address only after the device that held it was
            // removed, and it does not tell the miniport about removals. A slot
            // still holding this address is that removed device's: free it, or
            // the re-open at this address finds the dead slot first (a keyboard
            // unplugged from one hub port and plugged into another hung there).
            for s in 1..=MAX_SLOTS as u32 {
                if s != slot && x.slots[s as usize].in_use && x.slots[s as usize].usb_address == w_value {
                    xlog!("slot %u: address %u reused by usbport; freeing the removed device's slot", s, w_value);
                    disable_slot(x, s);
                }
            }
            let cc = address_device(x, slot, false);
            let (ctx_va, _) = dev_ctx(x, slot);
            dma_invalidate(ctx_va, x.ctx_size as usize);
            let hw = core::ptr::read_volatile((ctx_va as *const u32).add(3)) & 0xFF;
            let st = &mut x.slots[slot as usize];
            if cc == CC_SUCCESS {
                st.usb_address = w_value;
                st.hw_address = hw;
            }
            xlog!("slot %u: SET_ADDRESS %u -> Address Device cc %u (controller address %u)", slot, w_value, cc, hw);
            td.cc = if cc == CC_SUCCESS { CC_SUCCESS } else { CC_USB_TRANSACTION };
            td.done.store(true, Ordering::SeqCst);
            let r = &*ep.ring;
            td.first = r.enqueue;
            td.first_cycle = r.cycle;
            td.end = r.enqueue;
            if !queue_td(ep, td_ptr) {
                return MP_STATUS_FAILURE;
            }
            let p = &*core::ptr::addr_of!(crate::RegPacket);
            if let Some(f) = p.UsbPortInvalidateEndpoint {
                f(xhciExtension, xhciEndpoint);
            }
            return MP_STATUS_SUCCESS;
        }

        let needed = 2 + n_sg as u32;
        let ring = &mut *ep.ring;
        if ring.space(ep.deq) < needed {
            finish_payload(ep, td);
            return MP_STATUS_FAILURE;
        }
        let data_len = params.TransferBufferLength;
        let data_in = request_type & 0x80 != 0;
        td.first = ring.enqueue;
        td.first_cycle = ring.cycle;
        let trt = if data_len == 0 { TRT_NO_DATA } else if data_in { TRT_IN } else { TRT_OUT };
        ring.push_hidden(u64::from_le_bytes(setup), 8, (TRB_SETUP << TRB_TYPE_SHIFT) | TRB_IDT | trt);
        let mut remaining = data_len;
        for (i, e) in elems.iter().enumerate() {
            let len = e.SgTransferLength;
            remaining = remaining.saturating_sub(len);
            let td_size = ((remaining + mps - 1) / mps).min(31);
            let last = i + 1 == n_sg;
            let mut ctl = if i == 0 { TRB_DATA << TRB_TYPE_SHIFT } else { TRB_NORMAL << TRB_TYPE_SHIFT };
            ctl |= TRB_ISP;
            if !last {
                ctl |= TRB_CH;
            }
            if i == 0 && data_in {
                ctl |= TRB_DIR_IN;
            }
            let pa = e.SgPhysicalAddress.QuadPart as u64;
            td.data_pa[i] = ring.push(pa, len | (td_size << 17), ctl);
            td.data_len[i] = len;
        }
        td.ndata = n_sg as u32;
        let status_in = data_len == 0 || !data_in;
        td.last_pa = ring.push(0, 0, (TRB_STATUS << TRB_TYPE_SHIFT) | TRB_IOC | if status_in { TRB_DIR_IN } else { 0 });
        td.ntrbs = needed;
        td.end = ring.enqueue;
    } else {
        let count = n_sg.max(1) as u32;
        let ring = &mut *ep.ring;
        if ring.space(ep.deq) < count {
            finish_payload(ep, td);
            return MP_STATUS_FAILURE;
        }
        td.first = ring.enqueue;
        td.first_cycle = ring.cycle;
        let mut remaining = params.TransferBufferLength;
        if n_sg == 0 {
            td.last_pa = ring.push_hidden(0, 0, (TRB_NORMAL << TRB_TYPE_SHIFT) | TRB_IOC);
            td.data_pa[0] = td.last_pa;
            td.ndata = 1;
        } else {
            for (i, e) in elems.iter().enumerate() {
                let len = e.SgTransferLength;
                remaining = remaining.saturating_sub(len);
                let td_size = ((remaining + mps - 1) / mps).min(31);
                let last = i + 1 == n_sg;
                let mut ctl = (TRB_NORMAL << TRB_TYPE_SHIFT) | TRB_ISP;
                ctl |= if last { TRB_IOC } else { TRB_CH };
                let pa = e.SgPhysicalAddress.QuadPart as u64;
                td.data_pa[i] = if i == 0 {
                    ring.push_hidden(pa, len | (td_size << 17), ctl)
                } else {
                    ring.push(pa, len | (td_size << 17), ctl)
                };
                td.data_len[i] = len;
                if last {
                    td.last_pa = td.data_pa[i];
                }
            }
            td.ndata = n_sg as u32;
        }
        td.ntrbs = count;
        td.end = ring.enqueue;
    }

    if !queue_td(ep, td_ptr) {
        finish_payload(ep, td);
        xlog!("slot %u ep %u: transfer queue full", ep.slot, ep.dci);
        return MP_STATUS_FAILURE;
    }
    (*ep.ring).publish(td.first, td.first_cycle);
    ring_doorbell(x, ep.slot, ep.dci);
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_SubmitIsoTransfer(
    _xhciExtension: PVOID,
    _xhciEndpoint: PVOID,
    _TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    _xhciTransfer: PVOID,
    _IsoTransfer: PVOID,
) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}

/// usbhub reads a hub's class descriptor before opening its status-change
/// endpoint; remember what the Slot Context needs from it.
unsafe fn note_hub_descriptor(x: &mut XhciExtension, slot: u32, td: &XhciTransfer) {
    let setup = core::ptr::read_unaligned(&(*td.params).SetupPacket as *const _ as *const [u8; 8]);
    // Class GET_DESCRIPTOR, device-to-host. usbhub first asks with descriptor
    // type 0 (which many hubs accept), then 0x29, so identify the reply by its
    // own bDescriptorType (0x29, USB 2.0 hub) rather than by wValue.
    if setup[0] != 0xA0 || setup[1] != 0x06 || td.transferred < 5 || td.caller_va.is_null() {
        return;
    }
    let d = td.caller_va;
    if *d.add(1) != 0x29 {
        return;
    }
    let st = &mut x.slots[slot as usize];
    st.hub_ports = *d.add(2) as u32;
    st.hub_ttt = ((u16::from_le_bytes([*d.add(3), *d.add(4)]) >> 5) & 3) as u32;
    xlog!("slot %u: hub, %u ports, TT think time %u", slot, st.hub_ports, st.hub_ttt);
}

/// Hand every finished transfer, in order, back to usbport.
#[no_mangle]
pub unsafe extern "system" fn XHCI_PollEndpoint(xhciExtension: PVOID, xhciEndpoint: PVOID) {
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    let p = &*core::ptr::addr_of!(crate::RegPacket);
    loop {
        let mut irql: KIRQL = 0;
        KeAcquireSpinLock(&mut ep.lock, &mut irql);
        if ep.count == 0 {
            KeReleaseSpinLock(&mut ep.lock, irql);
            break;
        }
        let td = &mut *ep.pending[ep.head as usize % MAX_TDS];
        if !td.done.load(Ordering::SeqCst) {
            KeReleaseSpinLock(&mut ep.lock, irql);
            break;
        }
        ep.head = (ep.head + 1) % MAX_TDS as u32;
        ep.count -= 1;
        ep.deq = td.end;
        KeReleaseSpinLock(&mut ep.lock, irql);

        finish_payload(ep, td);
        let status = usbd_status(td.cc);
        if status != USBD_STATUS_SUCCESS {
            xlog!("slot %u ep %u: transfer failed, cc %u", ep.slot, ep.dci, td.cc);
        } else if ep.dci == 1 {
            note_hub_descriptor(ext_of(xhciExtension), ep.slot, td);
        }
        if let Some(f) = p.UsbPortCompleteTransfer {
            f(xhciExtension, xhciEndpoint, td.params as PVOID, status, td.transferred);
        }
    }
}

/// Take a transfer back from the controller: stop the endpoint, turn the TD's
/// TRBs into No-Ops, and if it was the one the controller would run next, move
/// the dequeue pointer past it.
#[no_mangle]
pub unsafe extern "system" fn XHCI_AbortTransfer(
    xhciExtension: PVOID,
    xhciEndpoint: PVOID,
    xhciTransfer: PVOID,
    CompletedLength: PULONG,
) {
    let x = ext_of(xhciExtension);
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    let target = xhciTransfer as *mut XhciTransfer;
    let mut was_done = (*target).done.load(Ordering::SeqCst);

    if !was_done && !ep.halted {
        let cc = ep_command(x, ep, TRB_STOP_ENDPOINT);
        if cc == CC_CONTEXT_STATE && matches!(ep_state(x, ep), EP_STATE_HALTED | EP_STATE_ERROR) {
            // Already halted (e.g. its device was unplugged and the last transfer
            // errored): nothing is running, so the TD can be unlinked as it is,
            // and the next submit's recover() resets the endpoint.
            ep.halted = true;
        } else if cc != CC_SUCCESS {
            endpoint_command_failed(x, ep, TRB_STOP_ENDPOINT, cc);
        }
    }

    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(&mut ep.lock, &mut irql);
    // The command wait also consumes events: a real completion may have won
    // the race with Stop Endpoint. Never rewrite that completed TD's TRBs.
    was_done = (*target).done.load(Ordering::SeqCst);
    let mut pos = None;
    for i in 0..ep.count {
        if ep.pending[((ep.head + i) as usize) % MAX_TDS] == target {
            pos = Some(i);
            break;
        }
    }
    let mut move_dequeue = false;
    if let Some(i) = pos {
        // Completed entries can still precede the hardware's current request
        // until PollEndpoint retires them. Queue index zero is not necessarily
        // the controller's next TD.
        move_dequeue = !was_done && (0..i).all(|j| {
            (*ep.pending[((ep.head + j) as usize) % MAX_TDS]).done.load(Ordering::Acquire)
        });
        for j in i..ep.count - 1 {
            ep.pending[((ep.head + j) as usize) % MAX_TDS] = ep.pending[((ep.head + j + 1) as usize) % MAX_TDS];
        }
        ep.count -= 1;
        let td = &*target;
        if !was_done {
            let ring = &*ep.ring;
            let mut idx = td.first;
            for _ in 0..td.ntrbs {
                let trb = ring.va.add(idx as usize);
                let c = core::ptr::read_volatile(&(*trb).control);
                core::ptr::write_volatile(&mut (*trb).param_lo, 0);
                core::ptr::write_volatile(&mut (*trb).param_hi, 0);
                core::ptr::write_volatile(&mut (*trb).status, 0);
                core::ptr::write_volatile(&mut (*trb).control, (c & TRB_CYCLE) | (8 << TRB_TYPE_SHIFT));
                dma_clean(trb as *const u8, TRB_SIZE);
                idx = (idx + 1) % RING_USABLE;
            }
        }
        if i == 0 {
            ep.deq = td.end;
        }
    }
    let (idx, cyc) = resume_point(ep);
    let remaining = ep.count;
    KeReleaseSpinLock(&mut ep.lock, irql);

    if move_dequeue && !ep.halted {
        let cc = set_dequeue(x, ep, idx, cyc);
        if cc != CC_SUCCESS {
            endpoint_command_failed(x, ep, TRB_SET_TR_DEQUEUE, cc);
        }
    }
    finish_payload(ep, &mut *target);
    if !CompletedLength.is_null() {
        *CompletedLength = (*target).transferred;
    }
    if remaining != 0 && !ep.halted {
        ring_doorbell(x, ep.slot, ep.dci);
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_GetEndpointState(_xhciExtension: PVOID, xhciEndpoint: PVOID) -> ULONG {
    (*(xhciEndpoint as *mut XhciEndpoint)).state
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_SetEndpointState(_xhciExtension: PVOID, xhciEndpoint: PVOID, EndpointState: ULONG) {
    (*(xhciEndpoint as *mut XhciEndpoint)).state = EndpointState;
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_GetEndpointStatus(_xhciExtension: PVOID, xhciEndpoint: PVOID) -> ULONG {
    if (*(xhciEndpoint as *mut XhciEndpoint)).halted {
        USBPORT_ENDPOINT_HALT
    } else {
        USBPORT_ENDPOINT_RUN
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_SetEndpointStatus(xhciExtension: PVOID, xhciEndpoint: PVOID, EndpointStatus: ULONG) {
    let ep = &mut *(xhciEndpoint as *mut XhciEndpoint);
    if EndpointStatus == USBPORT_ENDPOINT_RUN && ep.halted {
        recover(ext_of(xhciExtension), ep);
    }
}

/// The controller keeps data toggles; Reset Endpoint (in recover) zeroes them.
#[no_mangle]
pub unsafe extern "system" fn XHCI_SetEndpointDataToggle(_xhciExtension: PVOID, _xhciEndpoint: PVOID, _DataToggle: ULONG) {}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RebalanceEndpoint(
    _xhciExtension: PVOID,
    _EndpointParameters: PUSBPORT_ENDPOINT_PROPERTIES,
    _xhciEndpoint: PVOID,
) {
}
