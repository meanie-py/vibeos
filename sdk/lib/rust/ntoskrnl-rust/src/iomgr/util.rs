use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::macros::OBJECT_TO_OBJECT_HEADER;

use ntkrnl_sys::ddk::{
    _ETHREAD, _OBJECT_TYPE, PETHREAD, PEPROCESS, PDEVICE_OBJECT, PFILE_OBJECT, PIRP,
    ACCESS_MASK, PACCESS_MASK, PKIRQL,
    FILE_FULL_EA_INFORMATION, PFILE_FULL_EA_INFORMATION,
    FS_INFORMATION_CLASS,
    PsGetCurrentThread, PsIsSystemThread, RtlMapGenericMask,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// FO_NAMED_PIPE is a #define not captured by bindgen
const FO_NAMED_PIPE: ULONG = 0x00000080;

// LockQueueIoCancelLock enum value
const LockQueueIoCancelLock: ULONG = 7;

// KERNEL_STACK_SIZE varies by architecture but is 12288 (0x3000) on i386/arm64
// and 0xC000 on amd64
#[cfg(target_arch = "x86_64")]
const KERNEL_STACK_SIZE: ULONG_PTR = 0xC000;
#[cfg(not(target_arch = "x86_64"))]
const KERNEL_STACK_SIZE: ULONG_PTR = 0x3000;

const STATUS_EA_LIST_INCONSISTENT: NTSTATUS = 0x80000014u32 as NTSTATUS;

extern "system" {
    fn RtlpGetStackLimits(StackBase: PULONG_PTR, StackLimit: PULONG_PTR);
}

#[no_mangle]
pub unsafe extern "system" fn IoComputeDesiredAccessFileObject(
    FileObject: PFILE_OBJECT,
    DesiredAccess: PACCESS_MASK,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    *DesiredAccess = 0;

    if (*OBJECT_TO_OBJECT_HEADER(FileObject as PVOID)).Type != IoFileObjectType as *mut _OBJECT_TYPE {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    *DesiredAccess = (if (*FileObject).Flags & FO_NAMED_PIPE == 0 { FILE_APPEND_DATA } else { 0 }) | FILE_WRITE_DATA;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoAcquireCancelSpinLock(Irql: PKIRQL) {
    *Irql = KeAcquireQueuedSpinLock(LockQueueIoCancelLock);
}

#[no_mangle]
pub unsafe extern "system" fn IoGetInitialStack() -> PVOID {
    (*PsGetCurrentThread()).Tcb.InitialStack
}

#[no_mangle]
pub unsafe extern "system" fn IoGetStackLimits(
    LowLimit: PULONG_PTR,
    HighLimit: PULONG_PTR,
) {
    let LowLimit = opaque_ptr(LowLimit);
    let HighLimit = opaque_ptr(HighLimit);
    let Prcb = KeGetCurrentPrcb();
    let DpcStack = (*Prcb).DpcStack as ULONG_PTR;
    let StackAddress: ULONG_PTR = 0;
    // Take the address of the stack-local variable (mirrors C's &StackAddress)
    let StackAddress: ULONG_PTR = &StackAddress as *const ULONG_PTR as ULONG_PTR;

    RtlpGetStackLimits(LowLimit, HighLimit);

    if (StackAddress < *LowLimit) || (StackAddress > *HighLimit) {
        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            if (*Prcb).DpcRoutineActive != 0
                && (StackAddress <= DpcStack)
                && (StackAddress >= DpcStack - KERNEL_STACK_SIZE)
            {
                *HighLimit = DpcStack;
                *LowLimit = DpcStack - KERNEL_STACK_SIZE;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoIsSystemThread(Thread: PETHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    PsIsSystemThread(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn IoIsWdmVersionAvailable(
    MajorVersion: UCHAR,
    MinorVersion: UCHAR,
) -> BOOLEAN {
    if MajorVersion <= 1 && MinorVersion <= 0x30 {
        1u8
    } else {
        0u8
    }
}

// IoGetCurrentProcess is in shim_exports.rs (macro → PsGetCurrentProcess)

#[no_mangle]
pub unsafe extern "system" fn IoReleaseCancelSpinLock(Irql: KIRQL) {
    KeReleaseQueuedSpinLock(LockQueueIoCancelLock, Irql);
}

#[no_mangle]
pub unsafe extern "system" fn IoThreadToProcess(Thread: PETHREAD) -> PEPROCESS {
    let Thread = opaque_ptr(Thread);
    (*(Thread as *mut _ETHREAD)).ThreadsProcess
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckDesiredAccess(
    DesiredAccess: PACCESS_MASK,
    GrantedAccess: ACCESS_MASK,
) -> NTSTATUS {
    RtlMapGenericMask(
        DesiredAccess,
        core::ptr::addr_of_mut!((*(IoFileObjectType as *mut _OBJECT_TYPE)).TypeInfo.GenericMapping),
    );

    if (!(*DesiredAccess) & GrantedAccess) != 0 {
        return STATUS_ACCESS_DENIED;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckEaBufferValidity(
    EaBuffer: PFILE_FULL_EA_INFORMATION,
    mut EaLength: ULONG,
    ErrorOffset: PULONG,
) -> NTSTATUS {
    let ErrorOffset = opaque_ptr(ErrorOffset);
    let mut NextEntryOffset: ULONG;
    let mut EaNameLength: UCHAR;
    let mut ComputedLength: ULONG;
    let mut Current = EaBuffer;

    loop {
        if EaLength < core::mem::offset_of!(FILE_FULL_EA_INFORMATION, EaName) as ULONG {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        EaNameLength = (*Current).EaNameLength;
        ComputedLength = (*Current).EaValueLength as ULONG
            + EaNameLength as ULONG
            + core::mem::offset_of!(FILE_FULL_EA_INFORMATION, EaName) as ULONG
            + 1;

        if EaLength < ComputedLength {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        // C: `Current->EaName[EaNameLength]` is pointer arithmetic past the
        // declared trailing `CHAR EaName[1]` array (reads the NUL terminator at
        // EaName + EaNameLength). The bindgen type is `[CHAR; 1]`, so indexing it
        // with EaNameLength (>0) is an out-of-bounds *array* access in Rust →
        // panic_bounds_check → the panic handler spins forever (silent HANG, not
        // a bugcheck). Read through the raw pointer to preserve C semantics.
        if *(*Current).EaName.as_ptr().add(EaNameLength as usize) != 0 {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        NextEntryOffset = (*Current).NextEntryOffset;

        if NextEntryOffset == 0 {
            if (EaLength as LONG - ComputedLength as LONG) < 0 {
                *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
                return STATUS_EA_LIST_INCONSISTENT;
            }
            break;
        }

        if ALIGN_UP_BY(ComputedLength as ULONG_PTR, core::mem::size_of::<ULONG>()) != NextEntryOffset as ULONG_PTR {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        if (NextEntryOffset as LONG) < 0 {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        EaLength -= NextEntryOffset;

        if (EaLength as LONG) < 0 {
            *ErrorOffset = ((Current as ULONG_PTR) - (EaBuffer as ULONG_PTR)) as ULONG;
            return STATUS_EA_LIST_INCONSISTENT;
        }

        Current = ((Current as ULONG_PTR) + NextEntryOffset as ULONG_PTR) as PFILE_FULL_EA_INFORMATION;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckFunctionAccess(
    _GrantedAccess: ACCESS_MASK,
    _MajorFunction: UCHAR,
    _MinorFunction: UCHAR,
    _IoControlCode: ULONG,
    _ExtraData: PVOID,
    _ExtraData2: PVOID,
) -> NTSTATUS {
    let _ExtraData = opaque_ptr(_ExtraData);
    let _ExtraData2 = opaque_ptr(_ExtraData2);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoValidateDeviceIoControlAccess(
    _Irp: PIRP,
    _RequiredAccess: ULONG,
) -> NTSTATUS {
    let _Irp = opaque_ptr(_Irp);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoSetDeviceToVerify(
    Thread: PETHREAD,
    DeviceObject: PDEVICE_OBJECT,
) {
    let Thread = opaque_ptr(Thread);
    let DeviceObject = opaque_ptr(DeviceObject);
    (*(Thread as *mut _ETHREAD)).DeviceToVerify = DeviceObject;
}

#[no_mangle]
pub unsafe extern "system" fn IoSetHardErrorOrVerifyDevice(
    Irp: PIRP,
    DeviceObject: PDEVICE_OBJECT,
) {
    let Irp = opaque_ptr(Irp);
    let DeviceObject = opaque_ptr(DeviceObject);
    if (*Irp).Tail.Overlay.Thread.is_null() {
        return;
    }

    (*(*Irp).Tail.Overlay.Thread).DeviceToVerify = DeviceObject;
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceToVerify(Thread: PETHREAD) -> PDEVICE_OBJECT {
    let Thread = opaque_ptr(Thread);
    (*(Thread as *mut _ETHREAD)).DeviceToVerify
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckQuerySetVolumeInformation(
    _FsInformationClass: FS_INFORMATION_CLASS,
    _Length: ULONG,
    _SetOperation: BOOLEAN,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/util.c
//  * PURPOSE:         I/O Utility Functions
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  *                  Daniel Zimmerman (netzimme@aim.com)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// RtlpGetStackLimits(PULONG_PTR StackBase,
//                    PULONG_PTR StackLimit);
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// IoComputeDesiredAccessFileObject(IN PFILE_OBJECT FileObject,
//                                  IN PACCESS_MASK DesiredAccess)
// {
//     /* Assume failure */
//     *DesiredAccess = 0;
//
//     /* First check we really have a FileObject */
//     if (OBJECT_TO_OBJECT_HEADER(FileObject)->Type != IoFileObjectType)
//     {
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Then compute desired access:
//      * Check if the handle has either FILE_WRITE_DATA or FILE_APPEND_DATA was
//      * granted. However, if this is a named pipe, make sure we don't ask for
//      * FILE_APPEND_DATA as it interferes with the FILE_CREATE_PIPE_INSTANCE
//      * access right!
//      */
//     *DesiredAccess = ((!(FileObject->Flags & FO_NAMED_PIPE) ? FILE_APPEND_DATA : 0) | FILE_WRITE_DATA);
//
//     return STATUS_SUCCESS;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoAcquireCancelSpinLock(OUT PKIRQL Irql)
// {
//     /* Just acquire the internal lock */
//     *Irql = KeAcquireQueuedSpinLock(LockQueueIoCancelLock);
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// IoGetInitialStack(VOID)
// {
//     /* Return the initial stack from the TCB */
//     return PsGetCurrentThread()->Tcb.InitialStack;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoGetStackLimits(OUT PULONG_PTR LowLimit,
//                  OUT PULONG_PTR HighLimit)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     ULONG_PTR DpcStack = (ULONG_PTR)(Prcb->DpcStack);
//     volatile ULONG_PTR StackAddress;
//
//     /* Save our stack address so we always know it's valid */
//     StackAddress = (ULONG_PTR)(&StackAddress);
//
//     /* Get stack values */
//     RtlpGetStackLimits(LowLimit, HighLimit);
//
//     /* Check if we're outside the stack */
//     if ((StackAddress < *LowLimit) || (StackAddress > *HighLimit))
//     {
//         /* Check if we may be in a DPC */
//         if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//         {
//             /* Check if we really are in a DPC */
//             if ((Prcb->DpcRoutineActive) &&
//                 (StackAddress <= DpcStack) &&
//                 (StackAddress >= DpcStack - KERNEL_STACK_SIZE))
//             {
//                 /* Use the DPC stack limits */
//                 *HighLimit = DpcStack;
//                 *LowLimit = DpcStack - KERNEL_STACK_SIZE;
//             }
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoIsSystemThread(IN PETHREAD Thread)
// {
//     /* Call the Ps Function */
//     return PsIsSystemThread(Thread);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoIsWdmVersionAvailable(IN UCHAR MajorVersion,
//                         IN UCHAR MinorVersion)
// {
//     /* Return support for WDM 1.30 (Windows Server 2003) */
//     if (MajorVersion <= 1 && MinorVersion <= 0x30) return TRUE;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// PEPROCESS
// NTAPI
// IoGetCurrentProcess(VOID)
// {
//     /* Return the current thread's process */
//     return (PEPROCESS)PsGetCurrentThread()->Tcb.ApcState.Process;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoReleaseCancelSpinLock(IN KIRQL Irql)
// {
//     /* Release the internal lock */
//     KeReleaseQueuedSpinLock(LockQueueIoCancelLock, Irql);
// }
//
// /*
//  * @implemented
//  */
// PEPROCESS
// NTAPI
// IoThreadToProcess(IN PETHREAD Thread)
// {
//     /* Return the thread's process */
//     return Thread->ThreadsProcess;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCheckDesiredAccess(IN OUT PACCESS_MASK DesiredAccess,
//                      IN ACCESS_MASK GrantedAccess)
// {
//     PAGED_CODE();
//
//     /* Map the generic mask */
//     RtlMapGenericMask(DesiredAccess,
//                       &IoFileObjectType->TypeInfo.GenericMapping);
//
//     /* Fail if the access masks don't grant full access */
//     if ((~(*DesiredAccess) & GrantedAccess)) return STATUS_ACCESS_DENIED;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCheckEaBufferValidity(IN PFILE_FULL_EA_INFORMATION EaBuffer,
//                         IN ULONG EaLength,
//                         OUT PULONG ErrorOffset)
// {
//     ULONG NextEntryOffset;
//     UCHAR EaNameLength;
//     ULONG ComputedLength;
//     PFILE_FULL_EA_INFORMATION Current;
//
//     PAGED_CODE();
//
//     /* We will browse all the entries */
//     for (Current = EaBuffer; ; Current = (PFILE_FULL_EA_INFORMATION)((ULONG_PTR)Current + NextEntryOffset))
//     {
//         /* Check that we have enough bits left for the current entry */
//         if (EaLength < FIELD_OFFSET(FILE_FULL_EA_INFORMATION, EaName))
//         {
//             goto FailPath;
//         }
//
//         EaNameLength = Current->EaNameLength;
//         ComputedLength = Current->EaValueLength + EaNameLength + FIELD_OFFSET(FILE_FULL_EA_INFORMATION, EaName) + 1;
//         /* Check that we have enough bits left for storing the name and its value */
//         if (EaLength < ComputedLength)
//         {
//             goto FailPath;
//         }
//
//         /* Make sure the name is null terminated */
//         if (Current->EaName[EaNameLength] != ANSI_NULL)
//         {
//             goto FailPath;
//         }
//
//         /* Get the next entry offset */
//         NextEntryOffset = Current->NextEntryOffset;
//         /* If it's 0, it's a termination case */
//         if (NextEntryOffset == 0)
//         {
//             /* If we don't overflow! */
//             if ((LONG)(EaLength - ComputedLength) < 0)
//             {
//                 goto FailPath;
//             }
//
//             break;
//         }
//
//         /* Compare the next offset we computed with the provided one, they must match */
//         if (ALIGN_UP_BY(ComputedLength, sizeof(ULONG)) != NextEntryOffset)
//         {
//             goto FailPath;
//         }
//
//         /* Check next entry offset value is positive */
//         if ((LONG)NextEntryOffset < 0)
//         {
//             goto FailPath;
//         }
//
//         /* Compute the remaining bits */
//         EaLength -= NextEntryOffset;
//         /* We must have bits left */
//         if ((LONG)EaLength < 0)
//         {
//             goto FailPath;
//         }
//
//         /* Move to the next entry */
//     }
//
//     /* If we end here, everything went OK */
//     return STATUS_SUCCESS;
//
// FailPath:
//     /* If we end here, we failed, set failed offset */
//     *ErrorOffset = (ULONG_PTR)Current - (ULONG_PTR)EaBuffer;
//     return STATUS_EA_LIST_INCONSISTENT;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoCheckFunctionAccess(IN ACCESS_MASK GrantedAccess,
//                       IN UCHAR MajorFunction,
//                       IN UCHAR MinorFunction,
//                       IN ULONG IoControlCode,
//                       IN PVOID ExtraData OPTIONAL,
//                       IN PVOID ExtraData2 OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoValidateDeviceIoControlAccess(IN PIRP Irp,
//                                 IN ULONG RequiredAccess)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoSetDeviceToVerify(IN PETHREAD Thread,
//                     IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Set the pointer in the thread */
//     Thread->DeviceToVerify = DeviceObject;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoSetHardErrorOrVerifyDevice(IN PIRP Irp,
//                              IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Ignore in case the IRP is not associated with any thread */
//     if (!Irp->Tail.Overlay.Thread)
//     {
//         DPRINT1("IoSetHardErrorOrVerifyDevice(0x%p, 0x%p): IRP has no thread, ignoring.\n",
//                 Irp, DeviceObject);
//         return;
//     }
//
//     /* Set the pointer in the IRP */
//     Irp->Tail.Overlay.Thread->DeviceToVerify = DeviceObject;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetDeviceToVerify(IN PETHREAD Thread)
// {
//     /* Return the pointer that was set with IoSetDeviceToVerify */
//     return Thread->DeviceToVerify;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoCheckQuerySetVolumeInformation(IN FS_INFORMATION_CLASS FsInformationClass,
//                                  IN ULONG Length,
//                                  IN BOOLEAN SetOperation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
