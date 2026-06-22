//! Interrupt-driven RX (M10, amd64) — virtio INTx → ISR → DPC → event.
//!
//! The smoltcp worker busy-polls with a short `delay_ms` by default. This module
//! connects the virtio-net device's legacy INTx so an arriving frame wakes the
//! worker through a `SynchronizationEvent` instead, dropping idle CPU and RX
//! latency. The discipline (matches `scsiport-rust`'s model):
//!
//!   - ISR (at DIRQL): `ack_interrupt` reads+clears the device ISR status
//!     register (deasserting the shared INTx line); if the used ring advanced it
//!     queues the DPC and returns TRUE, else FALSE so the chained ISRs run.
//!   - DPC (at DISPATCH): `KeSetEvent` on the RX event.
//!   - worker (at PASSIVE): waits on the event with the poll deadline as the
//!     fallback timeout, then runs `iface.poll` exactly as before.
//!
//! The ISR/DPC touch ONLY the ISR status register + the event/DPC — never the
//! Interface / SocketSet / queue descriptors — so the worker stays the sole owner
//! of the smoltcp state at PASSIVE. amd64 only: `HalGetInterruptVector` is an
//! arm64 stub, and there is no i386 net smoke config to validate against.

use alloc::boxed::Box;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ntkrnl_sys::ddk::{
    HalGetInterruptVector, IoConnectInterrupt, KAFFINITY, KDPC, KEVENT, KSPIN_LOCK,
    KeInitializeDpc, KeInitializeEvent, KeInsertQueueDpc, KeSetEvent, PKINTERRUPT, PVOID, _KDPC,
    _KINTERRUPT,
};

use crate::device::{self, NetDev};
use crate::serial::nsprint;

const PCI_BUS: i32 = 5; // _INTERFACE_TYPE_PCIBus (INTERFACE_TYPE = c_int)
const LEVEL_SENSITIVE: u32 = 0; // _KINTERRUPT_MODE_LevelSensitive (c_uint)
const SYNCHRONIZATION_EVENT: u32 = 1; // _EVENT_TYPE_SynchronizationEvent (c_uint)
const VIRTIO_ISR_QUEUE: u32 = 1 << 0; // InterruptStatus::QUEUE_INTERRUPT

/// Count of RX interrupts serviced (DPC fires) + a one-shot "first" latch, so a
/// runtime test can prove the ISR→DPC→event path is live rather than the worker
/// merely riding the fallback timeout.
static RX_IRQS: AtomicU32 = AtomicU32::new(0);
static FIRST_IRQ: AtomicBool = AtomicBool::new(false);

/// Total RX interrupts serviced so far (for diagnostics/tests).
pub fn rx_irq_count() -> u32 {
    RX_IRQS.load(Ordering::Relaxed)
}

/// ISR/DPC shared state. Boxed + leaked to `'static`: the worker never returns,
/// and the ISR/DPC reference this asynchronously, so it needs a stable address
/// that outlives `connect`.
#[repr(C)]
struct RxIsr {
    event: KEVENT,
    dpc: KDPC,
    spinlock: KSPIN_LOCK,
    interrupt: PKINTERRUPT,
    dev: *mut NetDev,
}

/// Interrupt service routine (DIRQL). Ack the device, wake the worker on RX.
unsafe extern "C" fn isr(_interrupt: *mut _KINTERRUPT, ctx: PVOID) -> u8 {
    let s = ctx as *mut RxIsr;
    let bits = device::ack_interrupt((*s).dev);
    if bits == 0 {
        return 0; // FALSE — ISR status clear: not our (shared) interrupt
    }
    if bits & VIRTIO_ISR_QUEUE != 0 {
        // Used ring advanced — defer the wake to DISPATCH (can't touch the event
        // dispatcher at DIRQL).
        KeInsertQueueDpc(&mut (*s).dpc, null_mut(), null_mut());
    }
    1 // TRUE — handled
}

/// DPC (DISPATCH). Signal the worker's RX event.
unsafe extern "C" fn dpc(_dpc: *mut _KDPC, ctx: PVOID, _a1: PVOID, _a2: PVOID) {
    let s = ctx as *mut RxIsr;
    RX_IRQS.fetch_add(1, Ordering::Relaxed);
    if FIRST_IRQ
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        // Proves the wire (ISR acked a real device IRQ → DPC ran) is live, not
        // the worker's fallback timeout. DISPATCH-level nsprint is fine here.
        nsprint!("[NETSTACK] M10: first RX interrupt serviced (event-driven RX live)\n");
    }
    KeSetEvent(&mut (*s).event, 0, 0);
}

/// Connect the virtio-net INTx (`interrupt_line` from PCI config 0x3C) to the RX
/// ISR. Returns the RX event the worker should wait on, or null on any failure
/// (the worker then falls back to pure polling — no functional regression).
pub unsafe fn connect(dev: *mut NetDev, interrupt_line: u32) -> *mut KEVENT {
    let s = Box::leak(Box::new(RxIsr {
        event: core::mem::zeroed(),
        dpc: core::mem::zeroed(),
        spinlock: 0,
        interrupt: null_mut(),
        dev,
    }));
    KeInitializeEvent(&mut s.event, SYNCHRONIZATION_EVENT, 0);
    KeInitializeDpc(&mut s.dpc, Some(dpc), s as *mut _ as PVOID);

    let mut irql: u8 = 0;
    let mut affinity: KAFFINITY = 0;
    // For PCI the bus interrupt level and vector are both the routed IRQ line.
    let vector = HalGetInterruptVector(
        PCI_BUS,
        0,
        interrupt_line,
        interrupt_line,
        &mut irql,
        &mut affinity,
    );
    if vector == 0 || affinity == 0 {
        nsprint!(
            "[NETSTACK] M10: HalGetInterruptVector failed (line {}) — polling\n",
            interrupt_line
        );
        return null_mut();
    }

    let status = IoConnectInterrupt(
        &mut s.interrupt,
        Some(isr),
        s as *mut _ as PVOID,
        &mut s.spinlock,
        vector,
        irql,
        irql,
        LEVEL_SENSITIVE, // INTx is level-triggered
        1,               // ShareVector — INTx is shared
        affinity,
        0, // FloatingSave
    );
    if status < 0 {
        nsprint!(
            "[NETSTACK] M10: IoConnectInterrupt failed {:#010x} — polling\n",
            status as u32
        );
        return null_mut();
    }
    nsprint!(
        "[NETSTACK] M10: interrupt-driven RX connected (line {}, vector {}, irql {})\n",
        interrupt_line, vector, irql
    );
    &mut s.event
}
