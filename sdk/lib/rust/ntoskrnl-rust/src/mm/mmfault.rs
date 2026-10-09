use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    KPROCESSOR_MODE,
    MEMORY_AREA,
    MM_AVL_TABLE,
    PMMSUPPORT,
    PMMVAD,
    MiLocateVad,
    MiLockProcessWorkingSetShared,
    MiLockWorkingSetShared,
    MiUnlockProcessWorkingSetShared,
    MiUnlockWorkingSetShared,
    MmAccessFaultSectionView,
    MmArmAccessFault,
    MmGetKernelAddressSpace,
    MmLocateMemoryAreaByAddress,
    MmNotPresentFaultSectionView,
    PsGetCurrentThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static mut MiRosKernelVadRoot: MM_AVL_TABLE;
    static mut MmSystemCacheWs: ntkrnl_sys::ddk::_MMSUPPORT;
    // FORCEINLINE shims — must be extern "C" (cdecl) to match C shim calling convention on i386.
    fn MmLockAddressSpace(AddressSpace: PMMSUPPORT);
    fn MmUnlockAddressSpace(AddressSpace: PMMSUPPORT);
}
extern "system" {
    fn Mmi386MakeKernelPageTableGlobal(Address: PVOID) -> BOOLEAN;
}
extern "system" {
    fn MmRebalanceMemoryConsumersAndWait();
}
// MmDebugTrace + MmDebugWsFaultCheck removed — both were diagnostic-only
// helpers (MmDbgLast{Safe,Excl}{Locker,Unlocker} caller tracking + the
// fault-time DPRINT1) that lived in mm_rust_shims.c. The locker tracking
// was eliminated when the WS-lock trampolines were promoted to bare-name
// Rust exports, which made the tracking always-NULL anyway.

unsafe fn MmpAccessFault(
    Mode: KPROCESSOR_MODE,
    Address: ULONG_PTR,
    FromMdl: BOOLEAN,
    FaultCode: ULONG,
) -> NTSTATUS {
    let AddressSpace: PMMSUPPORT;
    let mut MemoryArea: *mut MEMORY_AREA;
    let mut Status: NTSTATUS;
    let mut already_locked = false;

    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        return STATUS_UNSUCCESSFUL;
    }

    if MI_IS_INSTRUCTION_FETCH(FaultCode as ULONG_PTR) {
        return STATUS_ACCESS_VIOLATION;
    }

    if Address >= MmSystemRangeStart as ULONG_PTR {
        if Mode != 0i8 {
            return STATUS_ACCESS_VIOLATION;
        }
        AddressSpace = MmGetKernelAddressSpace();
    } else {
        AddressSpace = core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS)).Vm);
        // Detect recursive lock: if AddressCreationLock is already held by this
        // thread (e.g. page fault during MiUnmapViewOfSection), skip lock/unlock.
        if FromMdl == 0 {
            let process = PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS;
            let owner = (*process).AddressCreationLock.Owner;
            if !owner.is_null() && owner as usize == PsGetCurrentThread() as usize {
                already_locked = true;
            }
        }
    }

    if FromMdl == 0 && !already_locked {
        MmLockAddressSpace(AddressSpace);
    }

    loop {
        MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, Address as PVOID);
        if MemoryArea.is_null() || (*MemoryArea).DeleteInProgress != 0 {
            if FromMdl == 0 && !already_locked {
                MmUnlockAddressSpace(AddressSpace);
            }
            return STATUS_ACCESS_VIOLATION;
        }

        Status = match (*MemoryArea).Type {
            MEMORY_AREA_SECTION_VIEW => {
                MmAccessFaultSectionView(
                    AddressSpace,
                    MemoryArea,
                    Address as PVOID,
                    if FromMdl == 0 || already_locked { 1 } else { 0 },
                )
            }
            _ => STATUS_ACCESS_VIOLATION,
        };

        if Status != STATUS_MM_RESTART_OPERATION {
            break;
        }
    }

    if FromMdl == 0 && !already_locked {
        MmUnlockAddressSpace(AddressSpace);
    }
    Status
}

unsafe fn MmNotPresentFault(
    Mode: KPROCESSOR_MODE,
    Address: ULONG_PTR,
    FromMdl: BOOLEAN,
) -> NTSTATUS {
    let AddressSpace: PMMSUPPORT;
    let mut MemoryArea: *mut MEMORY_AREA;
    let mut Status: NTSTATUS;
    let mut already_locked = false;

    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        return STATUS_UNSUCCESSFUL;
    }

    if Address >= MmSystemRangeStart as ULONG_PTR {
        if Mode != 0i8 {
            return STATUS_ACCESS_VIOLATION;
        }
        AddressSpace = MmGetKernelAddressSpace();
    } else {
        AddressSpace = core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS)).Vm);
        // Detect recursive lock: if AddressCreationLock is already held by this
        // thread (e.g. page fault during MiUnmapViewOfSection), skip lock/unlock.
        if FromMdl == 0 {
            let process = PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS;
            let owner = (*process).AddressCreationLock.Owner;
            if !owner.is_null() && owner as usize == PsGetCurrentThread() as usize {
                already_locked = true;
            }
        }
    }

    if FromMdl == 0 && !already_locked {
        MmLockAddressSpace(AddressSpace);
    }

    loop {
        MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, Address as PVOID);
        if MemoryArea.is_null() || (*MemoryArea).DeleteInProgress != 0 {
            if FromMdl == 0 && !already_locked {
                MmUnlockAddressSpace(AddressSpace);
            }
            return STATUS_ACCESS_VIOLATION;
        }

        Status = match (*MemoryArea).Type {
            MEMORY_AREA_SECTION_VIEW => {
                MmNotPresentFaultSectionView(
                    AddressSpace,
                    MemoryArea,
                    Address as PVOID,
                    if FromMdl == 0 || already_locked { 1 } else { 0 },
                )
            }
            _ => STATUS_ACCESS_VIOLATION,
        };

        if Status != STATUS_MM_RESTART_OPERATION {
            break;
        }
    }

    if FromMdl == 0 && !already_locked {
        MmUnlockAddressSpace(AddressSpace);
    }
    Status
}


#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn KeGetTrapFramePc(TrapFrame: ntkrnl_sys::ddk::PKTRAP_FRAME) -> ULONG_PTR;
}
// On arm64, KeGetTrapFramePc is a macro: TrapFrame->Pc
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeGetTrapFramePc(TrapFrame: ntkrnl_sys::ddk::PKTRAP_FRAME) -> ULONG_PTR {
    (*TrapFrame).Pc as ULONG_PTR
}

#[no_mangle]
pub unsafe extern "system" fn MmAccessFault(
    FaultCode: ULONG,
    Address: PVOID,
    Mode: KPROCESSOR_MODE,
    TrapInformation: PVOID,
) -> NTSTATUS {
    const IRQL_NOT_LESS_OR_EQUAL: ULONG = 0x0000000A;
    let Irql = KeGetCurrentIrql();
    let Status = MmAccessFaultInner(FaultCode, Address, Mode, TrapInformation);

    // Divergence: a kernel-mode trap above APC_LEVEL that MM cannot resolve is
    // IRQL_NOT_LESS_OR_EQUAL, as on Windows, not an exception. The C already
    // says so: MmArmAccessFault answers this case with STATUS_IN_PAGE_ERROR |
    // 0x10000000 ("tell the trap handler to fail"), a value with a reserved bit
    // set that exists only to be recognised, and the older path answers
    // STATUS_UNSUCCESSFUL. No trap handler checks for either, so they were
    // dispatched as an in-page exception that any SEH frame on the stack could
    // swallow. One did: usbport read a NULL at DISPATCH_LEVEL inside a cdfs
    // paging read, cdfs's try/except caught it, and the read failed with the
    // bogus status d0000006 -- half of all amd64-bios boots at -smp 2, with
    // nothing in the log naming usbport. The bugcheck names the driver (0xD1).
    // Calls without a trap frame (MmProbeAndLockPages and friends) keep the
    // status: they report failure to a caller that handles it.
    if !NT_SUCCESS(Status) && Irql > APC_LEVEL as KIRQL && Mode == 0 && !TrapInformation.is_null() {
        let Pc = KeGetTrapFramePc(TrapInformation as ntkrnl_sys::ddk::PKTRAP_FRAME);
        let mut Access: ULONG_PTR = 0;
        if crate::ARM3::helpers::MI_IS_WRITE_ACCESS(FaultCode as ULONG_PTR) {
            Access |= 1;
        }
        if MI_IS_INSTRUCTION_FETCH(FaultCode as ULONG_PTR) {
            Access |= 8;
        }
        KeBugCheckEx(IRQL_NOT_LESS_OR_EQUAL, Address as ULONG_PTR, Irql as ULONG_PTR, Access, Pc);
    }
    Status
}

unsafe fn MmAccessFaultInner(
    FaultCode: ULONG,
    Address: PVOID,
    Mode: KPROCESSOR_MODE,
    TrapInformation: PVOID,
) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    let TrapInformation = opaque_ptr(TrapInformation);
    let mut Vad: PMMVAD = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut IsArm3Fault = 0u8;

    // lockdep tripwire (field-manual §9.13): faulting while already holding a
    // WS lock this fault's handling will re-acquire is a guaranteed
    // self-deadlock (excl hold) or a silent recursion (shared hold). Report at
    // the fault, with the faulting address, instead of wedging mutely.
    #[cfg(feature = "lockdep")]
    {
        let proc_ws = core::ptr::addr_of_mut!(
            (*(PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS)).Vm.WorkingSetMutex
        ) as usize;
        let sys_ws = core::ptr::addr_of_mut!(MmSystemCacheWs.WorkingSetMutex) as usize;
        crate::ke::lockdep::lockdep_fault_check(
            PsGetCurrentThread() as usize,
            Address as usize,
            &[proc_ws, sys_ws],
        );
    }

    if (Address as ULONG_PTR) >= MmSystemRangeStart as ULONG_PTR {
        #[cfg(target_arch = "x86")]
        {
            if Mmi386MakeKernelPageTableGlobal(Address) != 0 {
                return STATUS_SUCCESS;
            }
        }
    }

    if (PAGE_ALIGN(Address as ULONG_PTR) as PVOID == MM_SHARED_USER_DATA_VA as PVOID) ||
        MI_IS_PAGE_TABLE_ADDRESS(Address) {
        return MmArmAccessFault(FaultCode, Address, Mode, TrapInformation);
    }

    if !MmGetKernelAddressSpace().is_null() {
        if Address > MM_HIGHEST_USER_ADDRESS as PVOID {
            if KeGetCurrentIrql() > APC_LEVEL {
                IsArm3Fault = 1;
            } else {
                MiLockWorkingSetShared(PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD, core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT);
                Vad = MiLocateVad(core::ptr::addr_of_mut!(MiRosKernelVadRoot), Address);


                if !Vad.is_null() && !MI_IS_ROSMM_VAD(Vad as PVOID) {
                    IsArm3Fault = 1;
                }

                MiUnlockWorkingSetShared(PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD, core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT);
            }
        } else {
            // Acquire the process WS lock, locate the VAD, release. Matches the
            // C (ntoskrnl/mm/mmfault.c) and the amd64/i386 path — all arches take
            // the lock unconditionally.
            //
            // Previously arm64 carried an `owns`-guard here that skipped the
            // re-acquire when the thread already held the WS lock, band-aiding a
            // fault-inside-teardown self-deadlock (MmAccessFault →
            // MiLockProcessWorkingSetShared while already holding it). That was a
            // §9.13-era workaround: its false "amd64/i386 KVM schedules around it"
            // comment predates the TCG default, and amd64 — which NEVER had the
            // guard — was itself wedging ~50% until §9.13's OriginalPte/RmapListHead
            // PFN-union root fix (and arm64's §9.16 MiPteToPde fix) landed
            // 2026-07-04/05. With the root faults fixed the recursion no longer
            // occurs, so the guard is redundant and removed — arm64 now runs the
            // exact unconditional-lock path amd64/i386 have always booted green on.
            MiLockProcessWorkingSetShared(
                PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS,
                PsGetCurrentThread(),
            );
            Vad = MiLocateVad(
                core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot),
                Address,
            );

            if !Vad.is_null() && !MI_IS_ROSMM_VAD(Vad as PVOID) {
                IsArm3Fault = 1;
            }

            MiUnlockProcessWorkingSetShared(
                PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS,
                PsGetCurrentThread(),
            );
        }
    }

    if IsArm3Fault != 0
        || (Vad.is_null()
            && (Address as ULONG_PTR) >= MmPagedPoolStart as ULONG_PTR
            && (Address as ULONG_PTR) < MmPagedPoolEnd as ULONG_PTR)
        || MmGetKernelAddressSpace().is_null()
    {
        return MmArmAccessFault(FaultCode, Address, Mode, TrapInformation);
    }

    loop {
        if !MI_IS_NOT_PRESENT_FAULT(FaultCode as ULONG_PTR) {
            Status = MmpAccessFault(
                Mode,
                Address as ULONG_PTR,
                if TrapInformation.is_null() { 1 } else { 0 },
                FaultCode,
            );
        } else {
            Status = MmNotPresentFault(
                Mode,
                Address as ULONG_PTR,
                if TrapInformation.is_null() { 1 } else { 0 },
            );
        }

        if Status != STATUS_NO_MEMORY {
            break;
        }

        MmRebalanceMemoryConsumersAndWait();
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/mmfault.c
//  * PURPOSE:         Kernel memory management functions
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #include <cache/section/newmm.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include "ARM3/miarm.h"
//
// extern MM_AVL_TABLE MiRosKernelVadRoot;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// MmpAccessFault(KPROCESSOR_MODE Mode,
//                ULONG_PTR Address,
//                BOOLEAN FromMdl,
//                ULONG FaultCode)
// {
//     PMMSUPPORT AddressSpace;
//     MEMORY_AREA* MemoryArea;
//     NTSTATUS Status;
//
//     DPRINT("MmAccessFault(Mode %d, Address %x)\n", Mode, Address);
//
//     if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//     {
//         DPRINT1("Page fault at high IRQL was %u\n", KeGetCurrentIrql());
//         return(STATUS_UNSUCCESSFUL);
//     }
//
//     /* Instruction fetch and the page is present.
//        This means the page is NX and we cannot do anything to "fix" it. */
//     if (MI_IS_INSTRUCTION_FETCH(FaultCode))
//     {
//         DPRINT1("Page fault instruction fetch at %p\n", Address);
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /*
//      * Find the memory area for the faulting address
//      */
//     if (Address >= (ULONG_PTR)MmSystemRangeStart)
//     {
//         /*
//          * Check permissions
//          */
//         if (Mode != KernelMode)
//         {
//             DPRINT1("MmAccessFault(Mode %d, Address %x)\n", Mode, Address);
//             return(STATUS_ACCESS_VIOLATION);
//         }
//         AddressSpace = MmGetKernelAddressSpace();
//     }
//     else
//     {
//         AddressSpace = &PsGetCurrentProcess()->Vm;
//     }
//
//     if (!FromMdl)
//     {
//         MmLockAddressSpace(AddressSpace);
//     }
//     do
//     {
//         MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, (PVOID)Address);
//         if (MemoryArea == NULL || MemoryArea->DeleteInProgress)
//         {
//             if (!FromMdl)
//             {
//                 MmUnlockAddressSpace(AddressSpace);
//             }
//             return (STATUS_ACCESS_VIOLATION);
//         }
//
//         switch (MemoryArea->Type)
//         {
//         case MEMORY_AREA_SECTION_VIEW:
//             Status = MmAccessFaultSectionView(AddressSpace,
//                                               MemoryArea,
//                                               (PVOID)Address,
//                                               !FromMdl);
//             break;
// #ifdef NEWCC
//         case MEMORY_AREA_CACHE:
//             // This code locks for itself to keep from having to break a lock
//             // passed in.
//             if (!FromMdl)
//                 MmUnlockAddressSpace(AddressSpace);
//             Status = MmAccessFaultCacheSection(Mode, Address, FromMdl);
//             if (!FromMdl)
//                 MmLockAddressSpace(AddressSpace);
//             break;
// #endif
//         default:
//             Status = STATUS_ACCESS_VIOLATION;
//             break;
//         }
//     }
//     while (Status == STATUS_MM_RESTART_OPERATION);
//
//     DPRINT("Completed page fault handling\n");
//     if (!FromMdl)
//     {
//         MmUnlockAddressSpace(AddressSpace);
//     }
//     return(Status);
// }
//
// NTSTATUS
// NTAPI
// MmNotPresentFault(KPROCESSOR_MODE Mode,
//                   ULONG_PTR Address,
//                   BOOLEAN FromMdl)
// {
//     PMMSUPPORT AddressSpace;
//     MEMORY_AREA* MemoryArea;
//     NTSTATUS Status;
//
//     DPRINT("MmNotPresentFault(Mode %d, Address %x)\n", Mode, Address);
//
//     if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//     {
//         DPRINT1("Page fault at high IRQL was %u, address %x\n", KeGetCurrentIrql(), Address);
//         return(STATUS_UNSUCCESSFUL);
//     }
//
//     /*
//      * Find the memory area for the faulting address
//      */
//     if (Address >= (ULONG_PTR)MmSystemRangeStart)
//     {
//         /*
//          * Check permissions
//          */
//         if (Mode != KernelMode)
//         {
//             DPRINT1("Address: %x\n", Address);
//             return(STATUS_ACCESS_VIOLATION);
//         }
//         AddressSpace = MmGetKernelAddressSpace();
//     }
//     else
//     {
//         AddressSpace = &PsGetCurrentProcess()->Vm;
//     }
//
//     if (!FromMdl)
//     {
//         MmLockAddressSpace(AddressSpace);
//     }
//
//     /*
//      * Call the memory area specific fault handler
//      */
//     do
//     {
//         MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, (PVOID)Address);
//         if (MemoryArea == NULL || MemoryArea->DeleteInProgress)
//         {
//             if (!FromMdl)
//             {
//                 MmUnlockAddressSpace(AddressSpace);
//             }
//             return (STATUS_ACCESS_VIOLATION);
//         }
//
//         switch (MemoryArea->Type)
//         {
//         case MEMORY_AREA_SECTION_VIEW:
//             Status = MmNotPresentFaultSectionView(AddressSpace,
//                                                   MemoryArea,
//                                                   (PVOID)Address,
//                                                   !FromMdl);
//             break;
// #ifdef NEWCC
//         case MEMORY_AREA_CACHE:
//             // This code locks for itself to keep from having to break a lock
//             // passed in.
//             if (!FromMdl)
//                 MmUnlockAddressSpace(AddressSpace);
//             Status = MmNotPresentFaultCacheSection(Mode, Address, FromMdl);
//             if (!FromMdl)
//                 MmLockAddressSpace(AddressSpace);
//             break;
// #endif
//         default:
//             Status = STATUS_ACCESS_VIOLATION;
//             break;
//         }
//     }
//     while (Status == STATUS_MM_RESTART_OPERATION);
//
//     DPRINT("Completed page fault handling\n");
//     if (!FromMdl)
//     {
//         MmUnlockAddressSpace(AddressSpace);
//     }
//     return(Status);
// }
//
// extern BOOLEAN Mmi386MakeKernelPageTableGlobal(PVOID Address);
//
// VOID
// NTAPI
// MmRebalanceMemoryConsumersAndWait(VOID);
//
// NTSTATUS
// NTAPI
// MmAccessFault(IN ULONG FaultCode,
//               IN PVOID Address,
//               IN KPROCESSOR_MODE Mode,
//               IN PVOID TrapInformation)
// {
//     PMMVAD Vad = NULL;
//     NTSTATUS Status;
//     BOOLEAN IsArm3Fault = FALSE;
//
//     /* Cute little hack for ROS */
//     if ((ULONG_PTR)Address >= (ULONG_PTR)MmSystemRangeStart)
//     {
// #ifdef _M_IX86
//         /* Check for an invalid page directory in kernel mode */
//         if (Mmi386MakeKernelPageTableGlobal(Address))
//         {
//             /* All is well with the world */
//             return STATUS_SUCCESS;
//         }
// #endif
//     }
//
//     /* Handle shared user page / page table, which don't have a VAD / MemoryArea */
//     if ((PAGE_ALIGN(Address) == (PVOID)MM_SHARED_USER_DATA_VA) ||
//         MI_IS_PAGE_TABLE_ADDRESS(Address))
//     {
//         /* This is an ARM3 fault */
//         DPRINT("ARM3 fault %p\n", Address);
//         return MmArmAccessFault(FaultCode, Address, Mode, TrapInformation);
//     }
//
//     /* Is there a ReactOS address space yet? */
//     if (MmGetKernelAddressSpace())
//     {
//         if (Address > MM_HIGHEST_USER_ADDRESS)
//         {
//             /* At elevated IRQL, skip the VAD lookup — cannot acquire WS lock */
//             if (KeGetCurrentIrql() > APC_LEVEL)
//             {
//                 DPRINT("mmfault: kernel VA %p at IRQL %d, assuming ARM3\n",
//                        Address, KeGetCurrentIrql());
//                 IsArm3Fault = TRUE;
//                 goto SkipVadLookup;
//             }
//             /* Check if this is an ARM3 memory area */
//             MiLockWorkingSetShared(PsGetCurrentThread(), &MmSystemCacheWs);
//             Vad = MiLocateVad(&MiRosKernelVadRoot, Address);
//
//             if ((Vad != NULL) && !MI_IS_ROSMM_VAD(Vad))
//             {
//                 IsArm3Fault = TRUE;
//             }
//
//             MiUnlockWorkingSetShared(PsGetCurrentThread(), &MmSystemCacheWs);
//         }
//         else
//         {
//             /* Could this be a VAD fault from user-mode? */
//             MiLockProcessWorkingSetShared(PsGetCurrentProcess(), PsGetCurrentThread());
//             Vad = MiLocateVad(&PsGetCurrentProcess()->VadRoot, Address);
//
//             if ((Vad != NULL) && !MI_IS_ROSMM_VAD(Vad))
//             {
//                 IsArm3Fault = TRUE;
//             }
//
//             MiUnlockProcessWorkingSetShared(PsGetCurrentProcess(), PsGetCurrentThread());
//         }
//     }
//
// SkipVadLookup:
//     /* Is this an ARM3 VAD, or is there no address space yet? */
//     if (IsArm3Fault ||
//         ((Vad == NULL) &&
//          ((ULONG_PTR)Address >= (ULONG_PTR)MmPagedPoolStart) &&
//          ((ULONG_PTR)Address < (ULONG_PTR)MmPagedPoolEnd)) ||
//         (!MmGetKernelAddressSpace()))
//     {
//         /* This is an ARM3 fault */
//         DPRINT("ARM3 fault %p\n", Vad);
//         return MmArmAccessFault(FaultCode, Address, Mode, TrapInformation);
//     }
//
// Retry:
//     /* Keep same old ReactOS Behaviour */
//     if (!MI_IS_NOT_PRESENT_FAULT(FaultCode))
//     {
//         /* Call access fault */
//         Status = MmpAccessFault(Mode, (ULONG_PTR)Address, TrapInformation ? FALSE : TRUE, FaultCode);
//     }
//     else
//     {
//         /* Call not present */
//         Status = MmNotPresentFault(Mode, (ULONG_PTR)Address, TrapInformation ? FALSE : TRUE);
//     }
//
//     if (Status == STATUS_NO_MEMORY)
//     {
//         MmRebalanceMemoryConsumersAndWait();
//         goto Retry;
//     }
//
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
