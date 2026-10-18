use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _IO_INTERRUPT,
    _KINTERRUPT,
    PIO_CONNECT_INTERRUPT_PARAMETERS,
    PIO_DISCONNECT_INTERRUPT_PARAMETERS,
    PIO_INTERRUPT,
    PKINTERRUPT,
    PKSERVICE_ROUTINE,
    KAFFINITY,
    KINTERRUPT_MODE,
    ExAllocatePoolZero,
    CM_PARTIAL_RESOURCE_DESCRIPTOR,
    IO_INTERRUPT_MESSAGE_INFO,
    IO_INTERRUPT_MESSAGE_INFO_ENTRY,
    PIO_INTERRUPT_MESSAGE_INFO,
    PKMESSAGE_SERVICE_ROUTINE,
    PEXTENDED_DEVOBJ_EXTENSION,
    PDEVICE_OBJECT,
    PKSYNCHRONIZE_ROUTINE,
    KDPC,
    KEVENT,
    KMUTEX,
    HANDLE,
    KeInitializeDpc,
    KeInsertQueueDpc,
    KeRemoveQueueDpc,
    KeFlushQueuedDpcs,
    KeInitializeEvent,
    KeSetEvent,
    KeInitializeMutex,
    KeReleaseMutex,
    KeWaitForSingleObject,
    KeSetPriorityThread,
    PsGetCurrentThread,
    PsCreateSystemThread,
    ObReferenceObjectByHandle,
    ObfDereferenceObject,
    ZwClose,
    _EVENT_TYPE_SynchronizationEvent,
};
use core::sync::atomic::{AtomicBool, Ordering};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

type CCHAR = i8;

#[inline(always)]
unsafe fn KeInitializeSpinLock(SpinLock: PKSPIN_LOCK) {
    core::ptr::write_volatile(SpinLock as *mut ULONG_PTR, 0);
}

extern "system" {
    static KeActiveProcessors: KAFFINITY;
    static KeNumberProcessors: i8;
    fn KeInitializeInterrupt( Interrupt: PKINTERRUPT, ServiceRoutine: PKSERVICE_ROUTINE, ServiceContext: PVOID, SpinLock: PKSPIN_LOCK, Vector: ULONG, Irql: KIRQL, SynchronizeIrql: KIRQL, InterruptMode: KINTERRUPT_MODE, ShareVector: BOOLEAN, ProcessorNumber: CCHAR, FloatingSave: BOOLEAN, );
    fn KeConnectInterrupt(Interrupt: PKINTERRUPT) -> BOOLEAN;
    fn KeDisconnectInterrupt(Interrupt: PKINTERRUPT);
    // hal-rust message.rs: the address/data that raises a message vector.
    /// `RequesterId` u32::MAX: unknown here; pci.sys bound each vector to its device
    /// when it programmed the table, which happened at START, before this.
    fn HalVibeosMessageTarget(Vector: ULONG, RequesterId: ULONG, Address: *mut u64, Data: *mut ULONG) -> BOOLEAN;
    fn HalDisableSystemInterrupt(Vector: ULONG, Irql: KIRQL);
    fn HalEnableSystemInterrupt(Vector: ULONG, Irql: KIRQL, Mode: KINTERRUPT_MODE) -> BOOLEAN;
}

const TAG_IO_INTERRUPT: ULONG = 0x6E696F49;

#[repr(C)]
struct CONTAINING_RECORD_HELPER {
    _io_interrupt: _IO_INTERRUPT,
}

unsafe fn container_of_io_interrupt(first_interrupt: PKINTERRUPT) -> PIO_INTERRUPT {
    let offset = core::mem::offset_of!(CONTAINING_RECORD_HELPER, _io_interrupt) 
        + core::mem::offset_of!(_IO_INTERRUPT, FirstInterrupt);
    (first_interrupt as *mut u8).sub(offset) as PIO_INTERRUPT
}

#[no_mangle]
pub unsafe extern "system" fn IoConnectInterrupt(
    InterruptObject: *mut PKINTERRUPT,
    ServiceRoutine: PKSERVICE_ROUTINE,
    ServiceContext: PVOID,
    SpinLock: PKSPIN_LOCK,
    Vector: ULONG,
    Irql: KIRQL,
    SynchronizeIrql: KIRQL,
    InterruptMode: KINTERRUPT_MODE,
    ShareVector: BOOLEAN,
    ProcessorEnableMask: KAFFINITY,
    FloatingSave: BOOLEAN,
) -> NTSTATUS {
    let InterruptObject = opaque_ptr(InterruptObject);
    let ServiceContext = opaque_ptr(ServiceContext);
    let mut Interrupt: PKINTERRUPT;
    let mut InterruptUsed: PKINTERRUPT;
    let IoInterrupt: PIO_INTERRUPT;
    let mut FirstRun: BOOLEAN;
    let mut Count: CCHAR = 0;
    let mut Affinity: KAFFINITY;

    *InterruptObject = core::ptr::null_mut();

    Affinity = ProcessorEnableMask & KeActiveProcessors;
    while Affinity != 0 {
        if (Affinity & 1) != 0 {
            Count += 1;
        }
        Affinity >>= 1;
    }

    if Count == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    IoInterrupt = ExAllocatePoolZero(
        NonPagedPool,
        (((Count - 1) as usize) * core::mem::size_of::<_KINTERRUPT>()
            + core::mem::size_of::<_IO_INTERRUPT>()) as _,
        TAG_IO_INTERRUPT,
    ) as PIO_INTERRUPT;
    if IoInterrupt.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let SpinLock = if SpinLock.is_null() {
        let lock_ptr = core::ptr::addr_of_mut!((*IoInterrupt).SpinLock) as PKSPIN_LOCK;
        KeInitializeSpinLock(lock_ptr);
        lock_ptr
    } else {
        SpinLock
    };

    Interrupt = (IoInterrupt as *mut u8).add(core::mem::size_of::<_IO_INTERRUPT>()) as PKINTERRUPT;
    FirstRun = 1u8;

    Affinity = ProcessorEnableMask & KeActiveProcessors;
    Count = 0;
    while Affinity != 0 {
        if (Affinity & 1) == 0 {
            Count += 1;
            Affinity >>= 1;
            continue;
        }

        InterruptUsed = if FirstRun != 0 {
            core::ptr::addr_of_mut!((*IoInterrupt).FirstInterrupt)
        } else {
            Interrupt
        };

        KeInitializeInterrupt(
            InterruptUsed,
            ServiceRoutine,
            ServiceContext,
            SpinLock,
            Vector,
            Irql,
            SynchronizeIrql,
            InterruptMode,
            ShareVector,
            Count,
            FloatingSave,
        );

        if KeConnectInterrupt(InterruptUsed) == 0 {
            if FirstRun != 0 {
                ExFreePoolWithTag(IoInterrupt as PVOID, TAG_IO_INTERRUPT);
            } else {
                IoDisconnectInterrupt(core::ptr::addr_of_mut!((*IoInterrupt).FirstInterrupt));
            }
            return STATUS_INVALID_PARAMETER;
        }

        if FirstRun != 0 {
            FirstRun = 0;
        } else {
            (*IoInterrupt).Interrupt[Count as usize] = Interrupt;
            Interrupt = Interrupt.add(1);
        }

        Count += 1;
        Affinity >>= 1;
    }

    *InterruptObject = core::ptr::addr_of_mut!((*IoInterrupt).FirstInterrupt);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoDisconnectInterrupt(InterruptObject: PKINTERRUPT) {
    let InterruptObject = opaque_ptr(InterruptObject);
    let IoInterrupt: PIO_INTERRUPT = container_of_io_interrupt(InterruptObject);

    KeDisconnectInterrupt(core::ptr::addr_of_mut!((*IoInterrupt).FirstInterrupt));

    for i in 0..KeNumberProcessors {
        if (*IoInterrupt).Interrupt[i as usize].is_null() {
            continue;
        }
        KeDisconnectInterrupt((*IoInterrupt).Interrupt[i as usize]);
    }

    ExFreePoolWithTag(IoInterrupt as PVOID, TAG_IO_INTERRUPT);
}

unsafe fn IopConnectInterruptExFullySpecific(
    Parameters: PIO_CONNECT_INTERRUPT_PARAMETERS,
) -> NTSTATUS {
    // Irql == PASSIVE_LEVEL is Windows 8's passive-level line interrupt: KMDF asks for it this
    // way for WdfInterruptCreate(PassiveHandling) once it sees 6.2 or later. Handed to
    // IoConnectInterrupt it would be a DIRQL connection at IRQL 0. It is the line fallback of
    // CONNECT_MESSAGE_BASED_PASSIVE instead; the line's real DIRQL is the PDO's assigned
    // interrupt at that vector, since the caller replaced it with PASSIVE_LEVEL.
    let f = &(*Parameters).__bindgen_anon_1.FullySpecified;
    if f.Irql == 0 {
        if !f.SpinLock.is_null() || f.SynchronizeIrql != 0 || f.ServiceRoutine.is_none() {
            return STATUS_INVALID_PARAMETER;
        }
        let Some(line) = assigned_interrupts(f.PhysicalDeviceObject)
            .iter()
            .find(|d| is_interrupt(d, false) && d.u.Interrupt.Vector == f.Vector)
        else {
            return STATUS_INVALID_PARAMETER;
        };
        let source = (
            f.Vector,
            line.u.Interrupt.Level as KIRQL,
            f.ProcessorEnableMask,
            f.InterruptMode,
            f.ShareVector != 0,
        );
        return match IopConnectPassive([source].into_iter(), None, f.ServiceRoutine, f.ServiceContext, f.FloatingSave) {
            Ok(passive) => {
                *f.InterruptObject = (*(*passive).trampolines).interrupt;
                STATUS_SUCCESS
            }
            Err(status) => status,
        };
    }
    let Status = IoConnectInterrupt(
        (*Parameters).__bindgen_anon_1.FullySpecified.InterruptObject,
        (*Parameters).__bindgen_anon_1.FullySpecified.ServiceRoutine,
        (*Parameters).__bindgen_anon_1.FullySpecified.ServiceContext,
        (*Parameters).__bindgen_anon_1.FullySpecified.SpinLock as PKSPIN_LOCK,
        (*Parameters).__bindgen_anon_1.FullySpecified.Vector,
        (*Parameters).__bindgen_anon_1.FullySpecified.Irql,
        (*Parameters).__bindgen_anon_1.FullySpecified.SynchronizeIrql,
        (*Parameters).__bindgen_anon_1.FullySpecified.InterruptMode,
        (*Parameters).__bindgen_anon_1.FullySpecified.ShareVector,
        (*Parameters).__bindgen_anon_1.FullySpecified.ProcessorEnableMask,
        (*Parameters).__bindgen_anon_1.FullySpecified.FloatingSave,
    );
    Status
}

// IO_CONNECT_INTERRUPT_PARAMETERS.Version (wdm.h).
const CONNECT_FULLY_SPECIFIED: ULONG = 1;
const CONNECT_LINE_BASED: ULONG = 2;
const CONNECT_MESSAGE_BASED: ULONG = 3;
const CONNECT_FULLY_SPECIFIED_GROUP: ULONG = 4;
/// Windows 8's; our wdm.h stops at 4. Same parameters as CONNECT_MESSAGE_BASED.
const CONNECT_MESSAGE_BASED_PASSIVE: ULONG = 5;

const CM_RESOURCE_TYPE_INTERRUPT: u8 = 2;
const CM_RESOURCE_SHARE_SHARED: u8 = 3;
const CM_RESOURCE_INTERRUPT_LATCHED: u16 = 0x0001;
const CM_RESOURCE_INTERRUPT_MESSAGE: u16 = 0x0002;
const LEVEL_SENSITIVE: KINTERRUPT_MODE = 0;
const LATCHED: KINTERRUPT_MODE = 1;
const TAG_IO_MESSAGE: ULONG = u32::from_le_bytes(*b"IoMs");

/// The translated interrupt descriptors the PnP manager assigned to `Pdo`, or an
/// empty slice when it has none (not a PnP device, or not started).
unsafe fn assigned_interrupts<'a>(Pdo: PDEVICE_OBJECT) -> &'a [CM_PARTIAL_RESOURCE_DESCRIPTOR] {
    assigned(Pdo, true)
}

/// The device's assigned resources, translated or raw.
unsafe fn assigned<'a>(Pdo: PDEVICE_OBJECT, translated: bool) -> &'a [CM_PARTIAL_RESOURCE_DESCRIPTOR] {
    if Pdo.is_null() {
        return &[];
    }
    let ext = (*Pdo).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;
    if ext.is_null() || (*ext).DeviceNode.is_null() {
        return &[];
    }
    let node = (*ext).DeviceNode;
    let list = if translated { (*node).ResourceListTranslated } else { (*node).ResourceList };
    // PCI devices get exactly one full descriptor; anything else is not ours to guess at.
    if list.is_null() || (*list).Count != 1 {
        return &[];
    }
    let partial = core::ptr::addr_of!((*list).List[0].PartialResourceList);
    core::slice::from_raw_parts(
        core::ptr::addr_of!((*partial).PartialDescriptors[0]),
        (*partial).Count as usize,
    )
}

/// Every message the device was assigned, in order, as (vector, IRQL, affinity). An
/// MSI block is one descriptor whose raw twin carries its MessageCount; its message k
/// is the translated vector + k (the HAL hands a block out as consecutive vectors).
unsafe fn assigned_messages(Pdo: PDEVICE_OBJECT) -> impl Iterator<Item = (ULONG, KIRQL, KAFFINITY)> + Clone {
    let translated = assigned(Pdo, true);
    let raw = assigned(Pdo, false);
    translated.iter().enumerate().filter(|(_, d)| is_interrupt(d, true)).flat_map(move |(i, d)| {
        let count = raw.get(i).map_or(1, |r| (r.u.Interrupt.Level >> 16).max(1));
        let (vector, irql, affinity) = (d.u.Interrupt.Vector, d.u.Interrupt.Level as KIRQL, d.u.Interrupt.Affinity);
        (0..count).map(move |k| (vector + k, irql, affinity))
    })
}

fn is_interrupt(d: &CM_PARTIAL_RESOURCE_DESCRIPTOR, message: bool) -> bool {
    d.Type == CM_RESOURCE_TYPE_INTERRUPT && ((d.Flags & CM_RESOURCE_INTERRUPT_MESSAGE) != 0) == message
}

/// CONNECT_LINE_BASED: the device's line interrupt, from its assigned resources.
unsafe fn IopConnectInterruptExLineBased(
    Pdo: PDEVICE_OBJECT,
    InterruptObject: *mut PKINTERRUPT,
    ServiceRoutine: PKSERVICE_ROUTINE,
    ServiceContext: PVOID,
    SpinLock: PKSPIN_LOCK,
    SynchronizeIrql: KIRQL,
    FloatingSave: BOOLEAN,
) -> NTSTATUS {
    let Some(line) = assigned_interrupts(Pdo).iter().find(|d| is_interrupt(d, false)) else {
        return STATUS_NOT_FOUND;
    };
    let irql = line.u.Interrupt.Level as KIRQL;
    IoConnectInterrupt(
        InterruptObject,
        ServiceRoutine,
        ServiceContext,
        SpinLock,
        line.u.Interrupt.Vector,
        irql,
        SynchronizeIrql.max(irql),
        if line.Flags & CM_RESOURCE_INTERRUPT_LATCHED != 0 { LATCHED } else { LEVEL_SENSITIVE },
        (line.ShareDisposition == CM_RESOURCE_SHARE_SHARED) as BOOLEAN,
        line.u.Interrupt.Affinity,
        FloatingSave,
    )
}

/// What a message's KINTERRUPT calls: the driver's routine, told which message fired.
#[repr(C)]
struct MessageTrampoline {
    routine: PKMESSAGE_SERVICE_ROUTINE,
    context: PVOID,
    message_id: ULONG,
    // Passive connections only (null/None/false otherwise):
    passive: *mut Passive,
    /// The line fallback's routine, which takes no message ID.
    line_routine: PKSERVICE_ROUTINE,
    /// The KINTERRUPT the routine is handed (its first, when it has one per CPU).
    interrupt: PKINTERRUPT,
    /// Raised by the ISR, taken by the worker: the message fired since it last ran.
    pending: AtomicBool,
}

unsafe extern "system" fn message_trampoline(Interrupt: PKINTERRUPT, Context: PVOID) -> BOOLEAN {
    let t = Context as *const MessageTrampoline;
    match (*t).routine {
        Some(routine) => routine(Interrupt, (*t).context, (*t).message_id),
        None => 0,
    }
}

/// One pool block: the IO_INTERRUPT_MESSAGE_INFO handed to the driver (its MessageInfo
/// array sized to `count`), then the trampolines its KINTERRUPTs point at. The driver
/// returns the block's address to IoDisconnectInterruptEx, which frees it whole.
fn message_block_layout(count: usize) -> (usize, usize) {
    let info = core::mem::offset_of!(IO_INTERRUPT_MESSAGE_INFO, MessageInfo)
        + count * core::mem::size_of::<IO_INTERRUPT_MESSAGE_INFO_ENTRY>();
    let align = core::mem::align_of::<MessageTrampoline>();
    let trampolines = (info + align - 1) & !(align - 1);
    (trampolines, trampolines + count * core::mem::size_of::<MessageTrampoline>())
}

unsafe fn IopDisconnectMessages(Table: PIO_INTERRUPT_MESSAGE_INFO) {
    let entries = core::ptr::addr_of_mut!((*Table).MessageInfo) as *mut IO_INTERRUPT_MESSAGE_INFO_ENTRY;
    for i in 0..(*Table).MessageCount as usize {
        let object = (*entries.add(i)).InterruptObject;
        if !object.is_null() {
            IoDisconnectInterrupt(object);
        }
    }
    ExFreePoolWithTag(Table as PVOID, TAG_IO_MESSAGE);
}

/// CONNECT_MESSAGE_BASED: every message the PnP manager assigned, each its own
/// KINTERRUPT. A device with none (its driver did not opt in, or the HAL cannot route
/// messages) gets its line interrupt through FallBackServiceRoutine instead, and
/// Version is rewritten to CONNECT_LINE_BASED to say so, as on Windows.
unsafe fn IopConnectInterruptExMessageBased(Parameters: PIO_CONNECT_INTERRUPT_PARAMETERS) -> NTSTATUS {
    let p = &mut (*Parameters).__bindgen_anon_1.MessageBased;
    let count = assigned_messages(p.PhysicalDeviceObject).count();
    if count == 0 {
        if p.FallBackServiceRoutine.is_none() {
            return STATUS_NOT_FOUND;
        }
        let status = IopConnectInterruptExLineBased(
            p.PhysicalDeviceObject,
            p.ConnectionContext.InterruptObject,
            p.FallBackServiceRoutine,
            p.ServiceContext,
            p.SpinLock as PKSPIN_LOCK,
            p.SynchronizeIrql,
            p.FloatingSave,
        );
        if status >= 0 {
            (*Parameters).Version = CONNECT_LINE_BASED;
        }
        return status;
    }
    if p.MessageServiceRoutine.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    let (trampolines_at, size) = message_block_layout(count);
    let block = ExAllocatePoolZero(NonPagedPool, size as _, TAG_IO_MESSAGE) as *mut u8;
    if block.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let table = block as PIO_INTERRUPT_MESSAGE_INFO;
    let entries = core::ptr::addr_of_mut!((*table).MessageInfo) as *mut IO_INTERRUPT_MESSAGE_INFO_ENTRY;
    let trampolines = block.add(trampolines_at) as *mut MessageTrampoline;
    let unified = assigned_messages(p.PhysicalDeviceObject).map(|(_, irql, _)| irql).max().unwrap_or(0);
    (*table).UnifiedIrql = unified;
    (*table).MessageCount = count as ULONG;

    // Message IDs follow the order the messages were assigned in, which is the
    // MSI-X table order pci.sys requested them in (or an MSI block's own order).
    for (id, (vector, irql, affinity)) in assigned_messages(p.PhysicalDeviceObject).enumerate() {
        let t = trampolines.add(id);
        (*t).routine = p.MessageServiceRoutine;
        (*t).context = p.ServiceContext;
        (*t).message_id = id as ULONG;
        let e = &mut *entries.add(id);
        e.Vector = vector;
        e.Irql = irql;
        e.TargetProcessorSet = affinity;
        e.Mode = LATCHED;
        e.Polarity = 0; // InterruptPolarityUnknown
        let mut address = 0u64;
        if HalVibeosMessageTarget(e.Vector, u32::MAX, &mut address, &mut e.MessageData) != 0 {
            e.MessageAddress.QuadPart = address as i64;
        }
        let status = IoConnectInterrupt(
            &mut e.InterruptObject,
            Some(message_trampoline),
            t as PVOID,
            p.SpinLock as PKSPIN_LOCK,
            e.Vector,
            e.Irql,
            p.SynchronizeIrql.max(unified),
            LATCHED,
            0,
            e.TargetProcessorSet,
            p.FloatingSave,
        );
        if status < 0 {
            (*table).MessageCount = id as ULONG; // disconnect only what connected
            IopDisconnectMessages(table);
            return status;
        }
    }
    *p.ConnectionContext.InterruptMessageTable = table;
    STATUS_SUCCESS
}

/// CONNECT_MESSAGE_BASED_PASSIVE: the driver's routine runs at PASSIVE_LEVEL, on a
/// worker thread of the connection's own. What the hardware raises is still an
/// interrupt at DIRQL; its KINTERRUPT's service routine is `IopPassiveInterruptService`,
/// which only notes which message fired and queues a DPC, the DPC wakes the worker
/// (KeSetEvent is not callable at DIRQL), and the worker calls the driver, holding the
/// connection's mutex, which KeSynchronizeExecution on any of its KINTERRUPTs takes too.
/// Messages are edge-triggered and the device's own, so nothing is masked: one that
/// fires while its routine runs is pending again and is called again. A device with no
/// messages falls back to its line through FallBackServiceRoutine, at passive level too,
/// and there the line has to be masked from the ISR until the routine has run, or a
/// level-triggered line would fire forever. As on Windows, SpinLock must be NULL and
/// SynchronizeIrql PASSIVE_LEVEL: nothing at passive level can hold a spin lock across
/// a driver routine that may block.
struct Passive {
    lock: KMUTEX,
    wake: KEVENT,
    dpc: KDPC,
    /// The worker's ETHREAD, referenced, so disconnecting can wait for it to exit.
    thread: PVOID,
    stop: AtomicBool,
    count: usize,
    trampolines: *mut MessageTrampoline,
    /// The line fallback: masked by the ISR, unmasked by the worker.
    line: Option<(ULONG, KIRQL, KINTERRUPT_MODE)>,
    /// The pool block all of this lives in (an IO_INTERRUPT_MESSAGE_INFO first).
    block: *mut u8,
}

/// The block a passive connection lives in: `message_block_layout`'s, then a Passive.
fn passive_block_layout(count: usize) -> (usize, usize, usize) {
    let (trampolines, end) = message_block_layout(count);
    let align = core::mem::align_of::<Passive>();
    let passive = (end + align - 1) & !(align - 1);
    (trampolines, passive, passive + core::mem::size_of::<Passive>())
}

/// The passive connection a KINTERRUPT belongs to, if it is one.
unsafe fn passive_of(Interrupt: PKINTERRUPT) -> Option<*mut Passive> {
    if Interrupt.is_null()
        || (*Interrupt).ServiceRoutine.map(|f| f as usize) != Some(IopPassiveInterruptService as usize)
    {
        return None;
    }
    Some((*((*Interrupt).ServiceContext as *mut MessageTrampoline)).passive)
}

/// The ISR every passive connection's KINTERRUPTs run, at DIRQL.
unsafe extern "system" fn IopPassiveInterruptService(_Interrupt: PKINTERRUPT, Context: PVOID) -> BOOLEAN {
    let t = Context as *mut MessageTrampoline;
    let p = (*t).passive;
    if let Some((vector, irql, _)) = (*p).line {
        HalDisableSystemInterrupt(vector, irql);
    }
    (*t).pending.store(true, Ordering::Release);
    KeInsertQueueDpc(&mut (*p).dpc, core::ptr::null_mut(), core::ptr::null_mut());
    1
}

unsafe extern "system" fn IopPassiveDpc(_Dpc: *mut KDPC, Context: PVOID, _A: PVOID, _B: PVOID) {
    KeSetEvent(&mut (*(Context as *mut Passive)).wake, 0, 0);
}

/// LOW_REALTIME_PRIORITY, as Windows runs its passive-level interrupt threads: above
/// every ordinary thread, so a busy desktop cannot starve a device of its interrupt.
const PASSIVE_INTERRUPT_PRIORITY: i32 = 16;

unsafe extern "system" fn IopPassiveWorker(Context: PVOID) {
    let p = Context as *mut Passive;
    // An ETHREAD begins with its KTHREAD (Tcb); KeGetCurrentThread is inline-only on amd64.
    KeSetPriorityThread(PsGetCurrentThread().cast(), PASSIVE_INTERRUPT_PRIORITY as _);
    loop {
        KeWaitForSingleObject(&mut (*p).wake as *mut KEVENT as PVOID, 0, 0, 0, core::ptr::null_mut());
        if (*p).stop.load(Ordering::Acquire) {
            break;
        }
        // Until a pass finds nothing pending: a message that fires again while its
        // routine runs is called again, never lost.
        let mut again = true;
        while again {
            again = false;
            for id in 0..(*p).count {
                let t = (*p).trampolines.add(id);
                if !(*t).pending.swap(false, Ordering::AcqRel) {
                    continue;
                }
                again = true;
                KeWaitForSingleObject(&mut (*p).lock as *mut KMUTEX as PVOID, 0, 0, 0, core::ptr::null_mut());
                match ((*t).routine, (*t).line_routine) {
                    (Some(routine), _) => {
                        routine((*t).interrupt, (*t).context, (*t).message_id);
                    }
                    (None, Some(routine)) => {
                        routine((*t).interrupt, (*t).context);
                    }
                    (None, None) => {}
                }
                KeReleaseMutex(&mut (*p).lock, 0);
                if let Some((vector, irql, mode)) = (*p).line {
                    HalEnableSystemInterrupt(vector, irql, mode);
                }
            }
        }
    }
    // Returning ends a system thread (PspSystemThreadStartup terminates it).
}

/// KeSynchronizeExecution on a passive connection's KINTERRUPT: its routines run at
/// PASSIVE_LEVEL under the connection's mutex, which raising IRQL and taking the
/// KINTERRUPT's spin lock would not exclude. Returns the routine's BOOLEAN, or
/// [`NOT_PASSIVE`] when `Interrupt` is not passive, for the caller to do the usual.
#[no_mangle]
pub unsafe extern "system" fn IopSynchronizePassiveInterrupt(
    Interrupt: PKINTERRUPT,
    Routine: PKSYNCHRONIZE_ROUTINE,
    Context: PVOID,
) -> ULONG {
    let Some(p) = passive_of(Interrupt) else {
        return NOT_PASSIVE;
    };
    KeWaitForSingleObject(&mut (*p).lock as *mut KMUTEX as PVOID, 0, 0, 0, core::ptr::null_mut());
    let result = match Routine {
        Some(routine) => routine(Context),
        None => 0,
    };
    KeReleaseMutex(&mut (*p).lock, 0);
    result as ULONG
}

/// IopSynchronizePassiveInterrupt: the interrupt is not a passive connection's.
const NOT_PASSIVE: ULONG = u32::MAX;

/// Undo a passive connection: the worker first, so it can unmask no line after the
/// line is disconnected; then the interrupts; then any DPC they queued meanwhile.
unsafe fn IopDisconnectPassive(p: *mut Passive) {
    if !(*p).thread.is_null() {
        (*p).stop.store(true, Ordering::Release);
        KeSetEvent(&mut (*p).wake, 0, 0);
        KeWaitForSingleObject((*p).thread, 0, 0, 0, core::ptr::null_mut());
        ObfDereferenceObject((*p).thread);
    }
    for id in 0..(*p).count {
        let object = (*(*p).trampolines.add(id)).interrupt;
        if !object.is_null() {
            IoDisconnectInterrupt(object);
        }
    }
    KeRemoveQueueDpc(&mut (*p).dpc);
    KeFlushQueuedDpcs();
    ExFreePoolWithTag((*p).block as PVOID, TAG_IO_MESSAGE);
}

/// One interrupt a passive connection takes: (vector, IRQL, affinity, mode, shared).
type PassiveSource = (ULONG, KIRQL, KAFFINITY, KINTERRUPT_MODE, bool);

/// An iterator of known length (`assigned_messages` counts by walking).
#[derive(Clone)]
struct Counted<I>(I, usize);

impl<I: Iterator> Iterator for Counted<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        let item = self.0.next()?;
        self.1 = self.1.saturating_sub(1);
        Some(item)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<I: Iterator> ExactSizeIterator for Counted<I> {}

unsafe fn IopConnectPassive(
    sources: impl ExactSizeIterator<Item = PassiveSource> + Clone,
    message_routine: PKMESSAGE_SERVICE_ROUTINE,
    line_routine: PKSERVICE_ROUTINE,
    context: PVOID,
    floating_save: BOOLEAN,
) -> Result<*mut Passive, NTSTATUS> {
    let count = sources.len();
    let (trampolines_at, passive_at, size) = passive_block_layout(count);
    let block = ExAllocatePoolZero(NonPagedPool, size as _, TAG_IO_MESSAGE) as *mut u8;
    if block.is_null() {
        return Err(STATUS_INSUFFICIENT_RESOURCES);
    }
    let p = block.add(passive_at) as *mut Passive;
    (*p).count = count;
    (*p).trampolines = block.add(trampolines_at) as *mut MessageTrampoline;
    (*p).block = block;
    if line_routine.is_some() {
        (*p).line = sources.clone().next().map(|(vector, irql, _, mode, _)| (vector, irql, mode));
    }
    KeInitializeMutex(&mut (*p).lock, 0);
    KeInitializeEvent(&mut (*p).wake, _EVENT_TYPE_SynchronizationEvent, 0);
    KeInitializeDpc(&mut (*p).dpc, Some(IopPassiveDpc), p as PVOID);
    for id in 0..count {
        let t = (*p).trampolines.add(id);
        (*t).routine = message_routine;
        (*t).line_routine = line_routine;
        (*t).context = context;
        (*t).message_id = id as ULONG;
        (*t).passive = p;
    }

    // The worker before the interrupts, so the first one has somewhere to go.
    let mut handle: HANDLE = core::ptr::null_mut();
    let status = PsCreateSystemThread(
        &mut handle,
        0x001F_FFFF, // THREAD_ALL_ACCESS
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        // PKSTART_ROUTINE's spelling differs per arch; the convention is NTAPI on all.
        core::mem::transmute(IopPassiveWorker as usize),
        p as PVOID,
    );
    if status < 0 {
        ExFreePoolWithTag(block as PVOID, TAG_IO_MESSAGE);
        return Err(status);
    }
    let status = ObReferenceObjectByHandle(handle, 0, core::ptr::null_mut(), 0, &mut (*p).thread, core::ptr::null_mut());
    ZwClose(handle);
    if status < 0 {
        // Unreachable with a handle just made; but without the object the worker
        // could not be waited for, so it is told to stop and left to exit.
        (*p).stop.store(true, Ordering::Release);
        KeSetEvent(&mut (*p).wake, 0, 0);
        return Err(status);
    }

    for (id, (vector, irql, affinity, mode, shared)) in sources.enumerate() {
        let t = (*p).trampolines.add(id);
        let status = IoConnectInterrupt(
            &mut (*t).interrupt,
            Some(IopPassiveInterruptService),
            t as PVOID,
            core::ptr::null_mut(),
            vector,
            irql,
            irql,
            mode,
            shared as BOOLEAN,
            affinity,
            floating_save,
        );
        if status < 0 {
            IopDisconnectPassive(p);
            return Err(status);
        }
    }
    Ok(p)
}

unsafe fn IopConnectInterruptExPassive(Parameters: PIO_CONNECT_INTERRUPT_PARAMETERS) -> NTSTATUS {
    let p = &mut (*Parameters).__bindgen_anon_1.MessageBased;
    if !p.SpinLock.is_null() || p.SynchronizeIrql != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let count = assigned_messages(p.PhysicalDeviceObject).count();
    if count == 0 {
        if p.FallBackServiceRoutine.is_none() {
            return STATUS_NOT_FOUND;
        }
        let Some(line) = assigned_interrupts(p.PhysicalDeviceObject).iter().find(|d| is_interrupt(d, false)) else {
            return STATUS_NOT_FOUND;
        };
        let source = (
            line.u.Interrupt.Vector,
            line.u.Interrupt.Level as KIRQL,
            line.u.Interrupt.Affinity,
            if line.Flags & CM_RESOURCE_INTERRUPT_LATCHED != 0 { LATCHED } else { LEVEL_SENSITIVE },
            line.ShareDisposition == CM_RESOURCE_SHARE_SHARED,
        );
        return match IopConnectPassive([source].into_iter(), None, p.FallBackServiceRoutine, p.ServiceContext, p.FloatingSave) {
            Ok(passive) => {
                *p.ConnectionContext.InterruptObject = (*(*passive).trampolines).interrupt;
                (*Parameters).Version = CONNECT_LINE_BASED;
                STATUS_SUCCESS
            }
            Err(status) => status,
        };
    }
    if p.MessageServiceRoutine.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    let sources = assigned_messages(p.PhysicalDeviceObject).map(|(vector, irql, affinity)| (vector, irql, affinity, LATCHED, false));
    let sources = Counted(sources, count);
    let passive = match IopConnectPassive(sources.clone(), p.MessageServiceRoutine, None, p.ServiceContext, p.FloatingSave) {
        Ok(passive) => passive,
        Err(status) => return status,
    };
    let table = (*passive).block as PIO_INTERRUPT_MESSAGE_INFO;
    let entries = core::ptr::addr_of_mut!((*table).MessageInfo) as *mut IO_INTERRUPT_MESSAGE_INFO_ENTRY;
    (*table).UnifiedIrql = 0; // PASSIVE_LEVEL: what every routine runs at
    (*table).MessageCount = count as ULONG;
    for (id, (vector, irql, affinity, _, _)) in sources.enumerate() {
        let e = &mut *entries.add(id);
        e.InterruptObject = (*(*passive).trampolines.add(id)).interrupt;
        e.Vector = vector;
        e.Irql = irql;
        e.TargetProcessorSet = affinity;
        e.Mode = LATCHED;
        e.Polarity = 0; // InterruptPolarityUnknown
        let mut address = 0u64;
        if HalVibeosMessageTarget(vector, u32::MAX, &mut address, &mut e.MessageData) != 0 {
            e.MessageAddress.QuadPart = address as i64;
        }
    }
    *p.ConnectionContext.InterruptMessageTable = table;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoConnectInterruptEx(
    Parameters: PIO_CONNECT_INTERRUPT_PARAMETERS,
) -> NTSTATUS {
    match (*Parameters).Version {
        CONNECT_FULLY_SPECIFIED | CONNECT_FULLY_SPECIFIED_GROUP => {
            IopConnectInterruptExFullySpecific(Parameters)
        }
        CONNECT_LINE_BASED => {
            let p = &(*Parameters).__bindgen_anon_1.LineBased;
            IopConnectInterruptExLineBased(
                p.PhysicalDeviceObject,
                p.InterruptObject,
                p.ServiceRoutine,
                p.ServiceContext,
                p.SpinLock as PKSPIN_LOCK,
                p.SynchronizeIrql,
                p.FloatingSave,
            )
        }
        CONNECT_MESSAGE_BASED => IopConnectInterruptExMessageBased(Parameters),
        CONNECT_MESSAGE_BASED_PASSIVE => IopConnectInterruptExPassive(Parameters),
        _ => STATUS_NOT_SUPPORTED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoDisconnectInterruptEx(
    Parameters: PIO_DISCONNECT_INTERRUPT_PARAMETERS,
) {
    match (*Parameters).Version {
        CONNECT_MESSAGE_BASED | CONNECT_MESSAGE_BASED_PASSIVE => {
            let table = (*Parameters).ConnectionContext.InterruptMessageTable;
            if table.is_null() {
            } else if let Some(passive) = passive_of((*table).MessageInfo[0].InterruptObject) {
                IopDisconnectPassive(passive);
            } else {
                IopDisconnectMessages(table);
            }
        }
        _ => {
            let object = (*Parameters).ConnectionContext.InterruptObject;
            if let Some(passive) = passive_of(object) {
                // A passive connection that fell back to the line.
                IopDisconnectPassive(passive);
            } else if !object.is_null() {
                IoDisconnectInterrupt(object);
            }
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/irq.c
//  * PURPOSE:         I/O Wrappers (called Completion Ports) for Kernel Queues
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoConnectInterrupt(OUT PKINTERRUPT *InterruptObject,
//                    IN PKSERVICE_ROUTINE ServiceRoutine,
//                    IN PVOID ServiceContext,
//                    IN PKSPIN_LOCK SpinLock,
//                    IN ULONG Vector,
//                    IN KIRQL Irql,
//                    IN KIRQL SynchronizeIrql,
//                    IN KINTERRUPT_MODE InterruptMode,
//                    IN BOOLEAN ShareVector,
//                    IN KAFFINITY ProcessorEnableMask,
//                    IN BOOLEAN FloatingSave)
// {
//     PKINTERRUPT Interrupt;
//     PKINTERRUPT InterruptUsed;
//     PIO_INTERRUPT IoInterrupt;
//     BOOLEAN FirstRun;
//     CCHAR Count = 0;
//     KAFFINITY Affinity;
//
//     PAGED_CODE();
//
//     /* Assume failure */
//     *InterruptObject = NULL;
//
//     /* Get the affinity */
//     Affinity = ProcessorEnableMask & KeActiveProcessors;
//     while (Affinity)
//     {
//         /* Increase count */
//         if (Affinity & 1) Count++;
//         Affinity >>= 1;
//     }
//
//     /* Make sure we have a valid CPU count */
//     if (!Count) return STATUS_INVALID_PARAMETER;
//
//     /* Allocate the array of I/O interrupts */
//     IoInterrupt = ExAllocatePoolZero(NonPagedPool,
//                                      (Count - 1) * sizeof(KINTERRUPT) +
//                                      sizeof(IO_INTERRUPT),
//                                      TAG_IO_INTERRUPT);
//     if (!IoInterrupt) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Use the structure's spinlock, if none was provided */
//     if (!SpinLock)
//     {
//         SpinLock = &IoInterrupt->SpinLock;
//         KeInitializeSpinLock(SpinLock);
//     }
//
//     /* We first start with a built-in interrupt inside the I/O structure */
//     Interrupt = (PKINTERRUPT)(IoInterrupt + 1);
//     FirstRun = TRUE;
//
//     /* Now create all the interrupts */
//     Affinity = ProcessorEnableMask & KeActiveProcessors;
//     for (Count = 0; Affinity; Count++, Affinity >>= 1)
//     {
//         /* Check if it's enabled for this CPU */
//         if (!(Affinity & 1))
//             continue;
//
//         /* Check which one we will use */
//         InterruptUsed = FirstRun ? &IoInterrupt->FirstInterrupt : Interrupt;
//
//         /* Initialize it */
//         KeInitializeInterrupt(InterruptUsed,
//                               ServiceRoutine,
//                               ServiceContext,
//                               SpinLock,
//                               Vector,
//                               Irql,
//                               SynchronizeIrql,
//                               InterruptMode,
//                               ShareVector,
//                               Count,
//                               FloatingSave);
//
//         /* Connect it */
//         if (!KeConnectInterrupt(InterruptUsed))
//         {
//             /* Check how far we got */
//             if (FirstRun)
//             {
//                 /* We failed early so just free this */
//                 ExFreePoolWithTag(IoInterrupt, TAG_IO_INTERRUPT);
//             }
//             else
//             {
//                 /* Far enough, so disconnect everything */
//                 IoDisconnectInterrupt(&IoInterrupt->FirstInterrupt);
//             }
//
//             /* And fail */
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Now we've used up our First Run */
//         if (FirstRun)
//         {
//             FirstRun = FALSE;
//         }
//         else
//         {
//             /* Move on to the next one */
//             IoInterrupt->Interrupt[(UCHAR)Count] = Interrupt++;
//         }
//     }
//
//     /* Return success */
//     *InterruptObject = &IoInterrupt->FirstInterrupt;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoDisconnectInterrupt(PKINTERRUPT InterruptObject)
// {
//     PIO_INTERRUPT IoInterrupt;
//     ULONG i;
//
//     PAGED_CODE();
//
//     /* Get the I/O interrupt */
//     IoInterrupt = CONTAINING_RECORD(InterruptObject,
//                                     IO_INTERRUPT,
//                                     FirstInterrupt);
//
//     /* Disconnect the first one */
//     KeDisconnectInterrupt(&IoInterrupt->FirstInterrupt);
//
//     /* Now disconnect the others */
//     for (i = 0; i < KeNumberProcessors; i++)
//     {
//         /* Make sure one was registered */
//         if (!IoInterrupt->Interrupt[i])
//             continue;
//
//         /* Disconnect it */
//         KeDisconnectInterrupt(IoInterrupt->Interrupt[i]);
//     }
//
//     /* Free the I/O interrupt */
//     ExFreePoolWithTag(IoInterrupt, TAG_IO_INTERRUPT);
// }
//
// NTSTATUS
// IopConnectInterruptExFullySpecific(
//     _Inout_ PIO_CONNECT_INTERRUPT_PARAMETERS Parameters)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Fallback to standard IoConnectInterrupt */
//     Status = IoConnectInterrupt(Parameters->FullySpecified.InterruptObject,
//                                 Parameters->FullySpecified.ServiceRoutine,
//                                 Parameters->FullySpecified.ServiceContext,
//                                 Parameters->FullySpecified.SpinLock,
//                                 Parameters->FullySpecified.Vector,
//                                 Parameters->FullySpecified.Irql,
//                                 Parameters->FullySpecified.SynchronizeIrql,
//                                 Parameters->FullySpecified.InterruptMode,
//                                 Parameters->FullySpecified.ShareVector,
//                                 Parameters->FullySpecified.ProcessorEnableMask,
//                                 Parameters->FullySpecified.FloatingSave);
//     if (!NT_SUCCESS(Status))
//         DPRINT1("IopConnectInterruptExFullySpecific() failed: 0x%lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IoConnectInterruptEx(
//     _Inout_ PIO_CONNECT_INTERRUPT_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//
//     switch (Parameters->Version)
//     {
//         case CONNECT_FULLY_SPECIFIED:
//             return IopConnectInterruptExFullySpecific(Parameters);
//         case CONNECT_FULLY_SPECIFIED_GROUP:
//             //TODO: We don't do anything for the group type
//             return IopConnectInterruptExFullySpecific(Parameters);
//         case CONNECT_MESSAGE_BASED:
//             DPRINT1("FIXME: Message based interrupts are UNIMPLEMENTED\n");
//             break;
//         case CONNECT_LINE_BASED:
//             DPRINT1("FIXME: Line based interrupts are UNIMPLEMENTED\n");
//             break;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// IoDisconnectInterruptEx(
//     _In_ PIO_DISCONNECT_INTERRUPT_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//
//     //FIXME: This eventually will need to handle more cases
//     if (Parameters->ConnectionContext.InterruptObject)
//         IoDisconnectInterrupt(Parameters->ConnectionContext.InterruptObject);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
