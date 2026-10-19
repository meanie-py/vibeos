//! `NdisPhy` — the NDIS-protocol smoltcp `phy::Device` (M11).
//!
//! The drop-in sibling of `VirtioPhy` (`device.rs`): same `phy::Device` surface,
//! so `engine.rs` falls back to it when no NIC of netstack's own is present. Where `VirtioPhy`
//! drives a virtio-net device directly, `NdisPhy` sits as an NDIS *protocol* above
//! an existing miniport (e1000 / rtl8139 / netkvm), per the design (§ M11).
//!
//! **The hard part — IRQL (design § M11).** `ProtocolReceive[Packet]` runs at
//! DISPATCH_LEVEL, but the smoltcp engine is a single PASSIVE worker that solely
//! owns the Interface+SocketSet and allocates freely — neither smoltcp nor the
//! global allocator may run from the receive callback. So RX is a lock-free,
//! non-paged handoff: the callback only **copies bytes into the ring below** (no
//! smoltcp, no pageable alloc) and the worker drains it on its next poll. This is
//! exactly `tcpip/datalink/lan.c`'s `LanSubmitReceiveWork` model.
//!

use alloc::vec;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;

/// Largest Ethernet frame we accept on RX (1500 MTU + 14 header + slack).
pub const FRAME_MAX: usize = 1536;
/// RX ring depth. One full miniport indication burst should fit before the
/// worker's next poll drains it; overflow drops frames (counted) — TCP/ARP retry.
const RING_SLOTS: usize = 64;

#[derive(Clone, Copy)]
struct Slot {
    data: [u8; FRAME_MAX],
    len: usize,
}

const EMPTY_SLOT: Slot = Slot { data: [0u8; FRAME_MAX], len: 0 };

/// A single-producer / single-consumer lock-free ring living in the driver's
/// non-paged .bss. Producer = `ProtocolReceive[Packet]` at DISPATCH (copies a
/// frame in, never allocates, touches only its slot + `tail`). Consumer = the
/// poll worker at PASSIVE (copies a frame out, touches only its slot + `head`).
/// On QEMU's single vCPU the producer preempts the consumer and they never run
/// concurrently; the acquire/release fences make the handoff correct on real SMP
/// too (one producer, one consumer — no CAS needed).
pub struct RxRing {
    slots: UnsafeCell<[Slot; RING_SLOTS]>,
    head: AtomicUsize, // next slot the consumer reads
    tail: AtomicUsize, // next slot the producer writes
    dropped: AtomicUsize,
}

// SAFETY: the SPSC discipline above (disjoint slots when non-empty/non-full,
// acquire/release ordering on head/tail) makes concurrent producer/consumer
// access sound without a lock.
unsafe impl Sync for RxRing {}

impl RxRing {
    const fn new() -> Self {
        Self {
            slots: UnsafeCell::new([EMPTY_SLOT; RING_SLOTS]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
        }
    }

    /// Producer (DISPATCH, no alloc): copy a received frame into the ring. Drops
    /// (and counts) the frame if the ring is full — a transient RX overrun the
    /// upper-layer protocols recover from via retransmit.
    pub fn push(&self, frame: &[u8]) {
        let tail = self.tail.load(Ordering::Relaxed);
        let next = (tail + 1) % RING_SLOTS;
        if next == self.head.load(Ordering::Acquire) {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let n = frame.len().min(FRAME_MAX);
        // SAFETY: the producer owns `slots[tail]` until it publishes `tail`; the
        // consumer never reads `slots[tail]` while head==tail (ring not yet
        // advanced). Single producer, so no producer-producer race.
        unsafe {
            let slots = &mut *self.slots.get();
            slots[tail].data[..n].copy_from_slice(&frame[..n]);
            slots[tail].len = n;
        }
        self.tail.store(next, Ordering::Release);
    }

    /// Consumer (PASSIVE): pop the next frame as an owned buffer, or None if empty.
    pub fn pop(&self) -> Option<Vec<u8>> {
        let head = self.head.load(Ordering::Relaxed);
        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }
        // SAFETY: head != tail means the producer has fully published slots[head]
        // (it stored `tail` with Release after writing the slot); the consumer is
        // the sole reader and owns slots[head] until it advances `head`.
        let frame = unsafe {
            let slots = &*self.slots.get();
            let s = &slots[head];
            s.data[..s.len].to_vec()
        };
        self.head.store((head + 1) % RING_SLOTS, Ordering::Release);
        Some(frame)
    }

    /// RX frames dropped due to ring overrun (diagnostics).
    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// The one RX ring for the (single) bound adapter. `ndis.rs`'s receive callback
/// pushes into it; `NdisPhy::receive` drains it.
pub static RX_RING: RxRing = RxRing::new();

/// The NDIS-protocol smoltcp device. Holds only the adapter MAC; RX comes from
/// the global `RX_RING`, TX goes out via `ndis::send_frame` (the binding handle
/// is global in `ndis.rs`).
pub struct NdisPhy {
    mac: [u8; 6],
}

impl NdisPhy {
    pub fn new(mac: [u8; 6]) -> Self {
        Self { mac }
    }
    pub fn mac(&self) -> [u8; 6] {
        self.mac
    }
}

pub struct NdisRxToken {
    data: Vec<u8>,
}

impl phy::RxToken for NdisRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }
}

/// TX token. Unlike `VirtioTxToken` it borrows nothing — an NDIS send marshals
/// the frame into an `NDIS_PACKET` and hands it to `NdisSend` on the global
/// binding handle, so no `&mut device` is needed.
pub struct NdisTxToken;

impl phy::TxToken for NdisTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let r = f(&mut buf);
        crate::ndis::send_frame(&buf);
        r
    }
}

impl Device for NdisPhy {
    type RxToken<'a> = NdisRxToken;
    type TxToken<'a> = NdisTxToken;

    fn receive(&mut self, _ts: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let data = RX_RING.pop()?;
        Some((NdisRxToken { data }, NdisTxToken))
    }

    fn transmit(&mut self, _ts: Instant) -> Option<Self::TxToken<'_>> {
        Some(NdisTxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
