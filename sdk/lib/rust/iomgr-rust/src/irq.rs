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
};
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
unsafe fn assigned_messages(Pdo: PDEVICE_OBJECT) -> impl Iterator<Item = (ULONG, KIRQL, KAFFINITY)> {
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
        // CONNECT_MESSAGE_BASED_PASSIVE (5) and anything newer.
        _ => STATUS_NOT_SUPPORTED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoDisconnectInterruptEx(
    Parameters: PIO_DISCONNECT_INTERRUPT_PARAMETERS,
) {
    match (*Parameters).Version {
        CONNECT_MESSAGE_BASED => {
            let table = (*Parameters).ConnectionContext.InterruptMessageTable;
            if !table.is_null() {
                IopDisconnectMessages(table);
            }
        }
        _ => {
            let object = (*Parameters).ConnectionContext.InterruptObject;
            if !object.is_null() {
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
