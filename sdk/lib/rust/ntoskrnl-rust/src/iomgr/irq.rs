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

#[no_mangle]
pub unsafe extern "system" fn IoConnectInterruptEx(
    Parameters: PIO_CONNECT_INTERRUPT_PARAMETERS,
) -> NTSTATUS {
    match (*Parameters).Version {
        0 => IopConnectInterruptExFullySpecific(Parameters),
        1 => IopConnectInterruptExFullySpecific(Parameters),
        2 | 3 => STATUS_SUCCESS,
        _ => STATUS_SUCCESS,
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoDisconnectInterruptEx(
    Parameters: PIO_DISCONNECT_INTERRUPT_PARAMETERS,
) {
    if !(*Parameters).ConnectionContext.InterruptObject.is_null() {
        IoDisconnectInterrupt((*Parameters).ConnectionContext.InterruptObject);
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
