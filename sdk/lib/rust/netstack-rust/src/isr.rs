//! Interrupt-driven RX (M10): NIC interrupt → ISR → DPC → event.
//!
//! The smoltcp worker busy-polls with a short `delay_ms` by default. This module
//! connects the NIC's interrupt so an arriving frame (or a finished transmit)
//! wakes the worker through a `SynchronizationEvent` instead, dropping idle CPU
//! and latency. The discipline (matches `scsiport-rust`'s model):
//!
//!   - ISR (at DIRQL): the backend's `ack` reads and clears the device's own
//!     interrupt status, deasserting a shared level line. Not ours → FALSE, so
//!     the chained ISRs run; ours → TRUE, queueing the DPC if the worker has
//!     something to do.
//!   - DPC (at DISPATCH): `KeSetEvent` on the event.
//!   - worker (at PASSIVE): waits on the event with the poll deadline as the
//!     fallback timeout, then runs `iface.poll` exactly as before.
//!
//! The ISR/DPC touch ONLY the status register and the event/DPC — never the
//! Interface / SocketSet / descriptor rings — so the worker stays the sole owner
//! of the smoltcp state at PASSIVE.
//!
//! Two callers: virtio-net's legacy INTx on x86, routed by the x86 HAL's
//! `HalGetInterruptVector` (`connect`), and the platform NICs on arm64, whose
//! translated interrupt resource comes from PnP START (`connect_resource`).
//!
//! ABI note: the ISR (PKSERVICE_ROUTINE) and DPC (PKDEFERRED_ROUTINE) callbacks
//! are NTAPI — cdecl on amd64, but **stdcall on i386** — so both are declared
//! `extern "system"` (which resolves to the right convention per arch); a plain
//! `extern "C"` would mis-decode the stack on i386.

use alloc::boxed::Box;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ntkrnl_sys::ddk::{
    IoConnectInterrupt, KAFFINITY, KDPC, KEVENT, KSPIN_LOCK, KeInitializeDpc, KeInitializeEvent,
    KeInsertQueueDpc, KeSetEvent, PKDEFERRED_ROUTINE, PKINTERRUPT, PKSERVICE_ROUTINE, PVOID, _KDPC, _KINTERRUPT,
};

use crate::serial::nsprint;

const LEVEL_SENSITIVE: u32 = 0; // _KINTERRUPT_MODE_LevelSensitive (c_uint)
const LATCHED: u32 = 1; // _KINTERRUPT_MODE_Latched
const SYNCHRONIZATION_EVENT: u32 = 1; // _EVENT_TYPE_SynchronizationEvent (c_uint)

/// What a backend's `ack` found in its device's interrupt status.
#[derive(Clone, Copy, PartialEq)]
pub enum Ack {
    /// Status clear: another device on a shared line.
    NotOurs,
    /// Ours, acknowledged, nothing for the worker.
    Handled,
    /// Ours, acknowledged: wake the worker.
    Wake,
}

/// Reads and clears the device's interrupt status, at DIRQL. `ctx` is what was
/// passed to connect.
pub type AckFn = unsafe fn(ctx: PVOID) -> Ack;

/// Count of interrupts that woke the worker (DPC fires) + a one-shot "first"
/// latch, so a runtime test can prove the ISR→DPC→event path is live rather
/// than the worker merely riding the fallback timeout.
static RX_IRQS: AtomicU32 = AtomicU32::new(0);
static FIRST_IRQ: AtomicBool = AtomicBool::new(false);

/// Total interrupts that woke the worker so far (for diagnostics/tests).
pub fn rx_irq_count() -> u32 {
    RX_IRQS.load(Ordering::Relaxed)
}

/// ISR/DPC shared state. Boxed + leaked to `'static`: the worker never returns,
/// and the ISR/DPC reference this asynchronously, so it needs a stable address
/// that outlives connect.
#[repr(C)]
struct RxIsr {
    event: KEVENT,
    dpc: KDPC,
    spinlock: KSPIN_LOCK,
    interrupt: PKINTERRUPT,
    ack: AckFn,
    ctx: PVOID,
}

/// Interrupt service routine (DIRQL). Ack the device, wake the worker.
unsafe extern "system" fn isr(_interrupt: *mut _KINTERRUPT, ctx: PVOID) -> u8 {
    let s = ctx as *mut RxIsr;
    match ((*s).ack)((*s).ctx) {
        Ack::NotOurs => 0, // FALSE: let the chained ISRs look
        Ack::Handled => 1,
        Ack::Wake => {
            // Defer the wake to DISPATCH (can't touch the event dispatcher at DIRQL).
            KeInsertQueueDpc(&mut (*s).dpc, null_mut(), null_mut());
            1
        }
    }
}

/// DPC (DISPATCH). Signal the worker's event.
unsafe extern "system" fn dpc(_dpc: *mut _KDPC, ctx: PVOID, _a1: PVOID, _a2: PVOID) {
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

/// Connect `ack` to a translated interrupt. Returns the event the worker should
/// wait on, or null on failure (the worker then keeps polling: no functional
/// regression). On a shared line the caller must not let its device raise the
/// interrupt until this has returned non-null, or nobody claims it.
pub unsafe fn connect_resource(
    ack: AckFn,
    ctx: PVOID,
    vector: u32,
    irql: u8,
    affinity: KAFFINITY,
    latched: bool,
    shared: bool,
) -> *mut KEVENT {
    let s = Box::leak(Box::new(RxIsr {
        event: core::mem::zeroed(),
        dpc: core::mem::zeroed(),
        spinlock: 0,
        interrupt: null_mut(),
        ack,
        ctx,
    }));
    KeInitializeEvent(&mut s.event, SYNCHRONIZATION_EVENT, 0);
    // The NTAPI callback fn-pointer types are cdecl on amd64 but stdcall on i386;
    // our `extern "system"` fns share that ABI but are nominally distinct from the
    // bindgen `extern "C"`/`"stdcall"` alias, so bridge with a transmute (the same
    // idiom engine.rs uses for PKSTART_ROUTINE / tdidev for PDRIVER_DISPATCH).
    let dpc_routine: PKDEFERRED_ROUTINE = Some(core::mem::transmute(dpc as *const ()));
    let isr_routine: PKSERVICE_ROUTINE = Some(core::mem::transmute(isr as *const ()));
    KeInitializeDpc(&mut s.dpc, dpc_routine, s as *mut _ as PVOID);

    let status = IoConnectInterrupt(
        &mut s.interrupt,
        isr_routine,
        s as *mut _ as PVOID,
        &mut s.spinlock,
        vector,
        irql,
        irql,
        if latched { LATCHED } else { LEVEL_SENSITIVE },
        shared as u8,
        affinity,
        0, // FloatingSave
    );
    if status < 0 {
        nsprint!("[NETSTACK] M10: IoConnectInterrupt failed {:#010x} — polling\n", status as u32);
        return null_mut();
    }
    nsprint!("[NETSTACK] M10: interrupt-driven RX connected (vector {}, irql {})\n", vector, irql);
    &mut s.event
}

/// virtio-net's ack: read and clear the ISR status register (deasserting the
/// shared INTx line); wake the worker if the used ring advanced.
unsafe fn virtio_ack(dev: PVOID) -> Ack {
    const VIRTIO_ISR_QUEUE: u32 = 1 << 0; // InterruptStatus::QUEUE_INTERRUPT
    match crate::device::ack_interrupt(dev as *mut crate::device::NetDev) {
        0 => Ack::NotOurs,
        bits if bits & VIRTIO_ISR_QUEUE != 0 => Ack::Wake,
        _ => Ack::Handled,
    }
}

/// Connect a PCI function's INTx (`interrupt_line`, from `pci::read_interrupt_line`)
/// to `ack`. On x86 the line is the firmware-routed IRQ; on arm64 it is the GIC
/// INTID, which the arm64 HAL passes through as the vector. Returns the event the
/// worker should wait on, or null on any failure. INTx is level-triggered and
/// shared, so the caller lets its function assert it only after this succeeds.
pub unsafe fn connect_line(ack: AckFn, ctx: PVOID, interrupt_line: u32) -> *mut KEVENT {
    const PCI_BUS: i32 = 5; // _INTERFACE_TYPE_PCIBus (INTERFACE_TYPE = c_int)
    let mut irql: u8 = 0;
    let mut affinity: KAFFINITY = 0;
    // For PCI the bus interrupt level and vector are both the routed IRQ line.
    let vector = ntkrnl_sys::ddk::HalGetInterruptVector(
        PCI_BUS,
        0,
        interrupt_line,
        interrupt_line,
        &mut irql,
        &mut affinity,
    );
    if vector == 0 || affinity == 0 {
        nsprint!("[NETSTACK] M10: HalGetInterruptVector failed (line {}) — polling\n", interrupt_line);
        return null_mut();
    }
    nsprint!("[NETSTACK] M10: INTx line {}\n", interrupt_line);
    connect_resource(ack, ctx, vector, irql, affinity, false, true)
}

/// Connect the virtio-net INTx to the RX ISR.
pub unsafe fn connect(dev: *mut crate::device::NetDev, interrupt_line: u32) -> *mut KEVENT {
    connect_line(virtio_ack, dev as PVOID, interrupt_line)
}
