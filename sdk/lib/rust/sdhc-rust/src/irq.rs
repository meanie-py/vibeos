//! The controller's interrupt: ISR → DPC → event, so a wait on the card sleeps
//! instead of spinning on INT_STATUS.
//!
//! The status register stays the worker's. The ISR only masks the signal
//! enables (INT_SIGNAL = 0), which drops the level line while every status bit
//! stays latched, and queues the DPC; the waiter (`Host::wait_int`) reads and
//! clears the status itself and re-arms the signals for its next wait. So the
//! ISR never consumes a bit the worker is looking for, and a signal is only
//! enabled while someone is waiting on it.

use core::ptr::{null_mut, read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use ntkrnl_sys::ddk::{
    IoConnectInterrupt, KeInitializeDpc, KeInitializeEvent, KeInsertQueueDpc, KeSetEvent, KeWaitForSingleObject,
    KAFFINITY, KDPC, KEVENT, KSPIN_LOCK, LARGE_INTEGER, PKDEFERRED_ROUTINE, PKINTERRUPT, PKSERVICE_ROUTINE, PVOID,
    _KDPC, _KINTERRUPT,
};

use crate::print::sdprint;

const LEVEL_SENSITIVE: u32 = 0; // _KINTERRUPT_MODE_LevelSensitive
const LATCHED: u32 = 1; // _KINTERRUPT_MODE_Latched
const SYNCHRONIZATION_EVENT: u32 = 1; // _EVENT_TYPE_SynchronizationEvent
const STATUS_TIMEOUT: i32 = 0x0000_0102;

/// INT_STATUS and INT_SIGNAL, as host.rs names them.
const REG_INT_STATUS: usize = 0x30;
const REG_INT_SIGNAL: usize = 0x38;

/// The register window, once the interrupt is connected; null means poll.
static REGS: AtomicPtr<u32> = AtomicPtr::new(null_mut());
static mut EVENT: KEVENT = unsafe { core::mem::zeroed() };
static mut DPC: KDPC = unsafe { core::mem::zeroed() };
static mut LOCK: KSPIN_LOCK = 0;
static mut INTERRUPT: PKINTERRUPT = null_mut();
static FIRST: AtomicBool = AtomicBool::new(false);

unsafe fn rd(regs: *mut u32, off: usize) -> u32 {
    read_volatile(regs.add(off / 4))
}

unsafe extern "system" fn isr(_interrupt: *mut _KINTERRUPT, _ctx: PVOID) -> u8 {
    let regs = REGS.load(Ordering::Acquire);
    if regs.is_null() || rd(regs, REG_INT_STATUS) & rd(regs, REG_INT_SIGNAL) == 0 {
        return 0; // not ours (or nothing signalled): let a sharer look
    }
    write_volatile(regs.add(REG_INT_SIGNAL / 4), 0);
    KeInsertQueueDpc(&raw mut DPC, null_mut(), null_mut());
    1
}

unsafe extern "system" fn dpc(_dpc: *mut _KDPC, _ctx: PVOID, _a: PVOID, _b: PVOID) {
    if !FIRST.swap(true, Ordering::Relaxed) {
        // Proof the wire is live, not the waits falling through to their timeouts.
        sdprint!("first interrupt serviced\n");
    }
    KeSetEvent(&raw mut EVENT, 0, 0);
}

/// Mask every signal enable, so the controller raises nothing until a waiter arms it.
///
/// # Safety
/// `regs` maps the controller's register window.
pub unsafe fn quiet(regs: *mut u8) {
    write_volatile(regs.cast::<u32>().add(REG_INT_SIGNAL / 4), 0);
}

/// Connect the translated interrupt resource. Leaves the driver polling (and
/// says so) when it fails.
///
/// # Safety
/// `regs` maps the controller for the driver's lifetime; INT_SIGNAL must be 0.
pub unsafe fn connect(regs: *mut u8, vector: u32, irql: u8, affinity: KAFFINITY, latched: bool, shared: bool) {
    KeInitializeEvent(&raw mut EVENT, SYNCHRONIZATION_EVENT as _, 0);
    // NTAPI callbacks: the bindgen aliases are nominally distinct from our
    // `extern "system"` fns, so bridge with a transmute.
    let dpc_routine: PKDEFERRED_ROUTINE = Some(core::mem::transmute(dpc as *const ()));
    let isr_routine: PKSERVICE_ROUTINE = Some(core::mem::transmute(isr as *const ()));
    KeInitializeDpc(&raw mut DPC, dpc_routine, null_mut());
    REGS.store(regs.cast(), Ordering::Release);
    let status = IoConnectInterrupt(
        &raw mut INTERRUPT,
        isr_routine,
        null_mut(),
        &raw mut LOCK,
        vector,
        irql,
        irql,
        if latched { LATCHED } else { LEVEL_SENSITIVE } as _,
        shared as u8,
        affinity,
        0,
    );
    if status < 0 {
        REGS.store(null_mut(), Ordering::Release);
        sdprint!("IoConnectInterrupt failed {:#010x}: polling\n", status as u32);
        return;
    }
    sdprint!("interrupt connected (vector {}, irql {})\n", vector, irql);
}

pub fn connected() -> bool {
    !REGS.load(Ordering::Acquire).is_null()
}

/// Sleep until the ISR fires or `us` microseconds pass. True when it fired.
pub fn wait(us: u64) -> bool {
    let mut timeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    timeout.QuadPart = -((us.max(1) * 10) as i64);
    unsafe { KeWaitForSingleObject((&raw mut EVENT).cast(), 0, 0, 0, &mut timeout) != STATUS_TIMEOUT }
}
