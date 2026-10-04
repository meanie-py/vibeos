use ntkrnl_sys::globals::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut KsecRandomSeed: ULONG = 0x62b409a1 as ULONG;

#[no_mangle]
pub unsafe extern "system" fn KsecGenRandom(
    Buffer: PVOID,
    mut Length: SIZE_T,
) -> NTSTATUS {

    let mut TickCount: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let RandomValue: ULONG;
    let P: PULONG;

    /* Try to generate a more random seed */
    KeQueryTickCount(core::ptr::addr_of_mut!(TickCount) as *mut i64);
    KsecRandomSeed ^= _rotl(
        TickCount.u.LowPart as u32,
        (KsecRandomSeed % 23) as i32,
    ) as ULONG;

    P = Buffer as PULONG;
    i = 0;
    while (i as usize) < (Length as usize) / core::mem::size_of::<ULONG>() {
        *P.offset(i as isize) = RtlRandomEx(core::ptr::addr_of_mut!(KsecRandomSeed));
        i = i.wrapping_add(1);
    }

    Length &= (core::mem::size_of::<ULONG>() - 1) as SIZE_T;
    if Length > 0 {
        RandomValue = RtlRandomEx(core::ptr::addr_of_mut!(KsecRandomSeed));
        RtlCopyMemory(
            P.offset(i as isize) as PVOID,
            core::ptr::addr_of!(RandomValue) as PVOID,
            Length as _,
        );
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn KsecReadMachineSpecificCounters(
    MachineSpecificCounters: PKSEC_MACHINE_SPECIFIC_COUNTERS,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        const PF_RDTSC_INSTRUCTION_AVAILABLE: u32 = 8;

        /* Check if RDTSC is available */
        if ExIsProcessorFeaturePresent(PF_RDTSC_INSTRUCTION_AVAILABLE) != 0 {
            /* Read the TSC value */
            (*MachineSpecificCounters).Tsc = __rdtsc();
        }
    }
    // The C's _M_ARM64 arm is unconditional: PF_RDTSC_INSTRUCTION_AVAILABLE is
    // an x86 feature bit, so gating on it (as this port once did on every arch)
    // left Tsc unset on arm64 and the counter out of the entropy pool.
    #[cfg(target_arch = "aarch64")]
    {
        /* Use the performance counter as entropy source */
        let pc = KeQueryPerformanceCounter(core::ptr::null_mut());
        (*MachineSpecificCounters).Tsc = pc.QuadPart as _;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsecGatherEntropyData(
    EntropyData: PKSEC_ENTROPY_DATA,
) -> NTSTATUS {
    let mut Md4Context: MD4_CTX = core::mem::zeroed();
    let mut Teb: PTEB;
    let mut Peb: PPEB;
    let mut String: PWSTR = core::ptr::null_mut();
    let mut ReturnLength: ULONG = 0;
    let mut Status: NTSTATUS;

    /* Query some generic values */
    (*EntropyData).CurrentProcessId = PsGetCurrentProcessId();
    (*EntropyData).CurrentThreadId = PsGetCurrentThreadId();
    KeQueryTickCount(core::ptr::addr_of_mut!((*EntropyData).TickCount) as *mut i64);
    /* KeQuerySystemTime expansion */
    *(core::ptr::addr_of_mut!((*EntropyData).SystemTime) as *mut u64) =
        *(SharedSystemTime as *mut u64);
    (*EntropyData).PerformanceCounter =
        KeQueryPerformanceCounter(core::ptr::addr_of_mut!((*EntropyData).PerformanceFrequency));

    /* Check if we have a TEB/PEB for the process environment */
    Teb = (*PsGetCurrentThread()).Tcb.Teb as PTEB;
    if !Teb.is_null() {
        Peb = (*Teb).ProcessEnvironmentBlock;

        /* Skip environment hashing — with DUMMY_PSEH, faulting on
         * invalid PEB/ProcessParameters/Environment pointers is fatal. */
        let _ = Peb;
    }

    /* Read some machine specific hardware counters */
    KsecReadMachineSpecificCounters(core::ptr::addr_of_mut!((*EntropyData).MachineSpecificCounters));

    /* Query processor performance information */
    Status = ZwQuerySystemInformation(
        SystemProcessorPerformanceInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemProcessorPerformanceInformation) as PVOID,
        core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query system performance information */
    Status = ZwQuerySystemInformation(
        SystemPerformanceInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemPerformanceInformation) as PVOID,
        core::mem::size_of::<SYSTEM_PERFORMANCE_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query exception information */
    Status = ZwQuerySystemInformation(
        SystemExceptionInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemExceptionInformation) as PVOID,
        core::mem::size_of::<SYSTEM_EXCEPTION_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query lookaside information */
    Status = ZwQuerySystemInformation(
        SystemLookasideInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemLookasideInformation) as PVOID,
        core::mem::size_of::<SYSTEM_LOOKASIDE_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query interrupt information */
    Status = ZwQuerySystemInformation(
        SystemInterruptInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemInterruptInformation) as PVOID,
        core::mem::size_of::<SYSTEM_INTERRUPT_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query process information */
    Status = ZwQuerySystemInformation(
        SystemProcessInformation,
        core::ptr::addr_of_mut!((*EntropyData).SystemProcessInformation) as PVOID,
        core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let _ = Md4Context;
    let _ = String;
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Drivers
//  * PURPOSE:         Kernel Security Support Provider Interface Driver
//  *
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ksecdd.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ********************************************************************/
//
// static ULONG KsecRandomSeed = 0x62b409a1;
//
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// KsecGenRandom(
//     PVOID Buffer,
//     SIZE_T Length)
// {
//     LARGE_INTEGER TickCount;
//     ULONG i, RandomValue;
//     PULONG P;
//
//     /* Try to generate a more random seed */
//     KeQueryTickCount(&TickCount);
//     KsecRandomSeed ^= _rotl(TickCount.LowPart, (KsecRandomSeed % 23));
//
//     P = Buffer;
//     for (i = 0; i < Length / sizeof(ULONG); i++)
//     {
//         P[i] = RtlRandomEx(&KsecRandomSeed);
//     }
//
//     Length &= (sizeof(ULONG) - 1);
//     if (Length > 0)
//     {
//         RandomValue = RtlRandomEx(&KsecRandomSeed);
//         RtlCopyMemory(&P[i], &RandomValue, Length);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// KsecReadMachineSpecificCounters(
//      PKSEC_MACHINE_SPECIFIC_COUNTERS MachineSpecificCounters)
// {
// #if defined(_M_IX86) || defined(_M_AMD64)
//     /* Check if RDTSC is available */
//     if (ExIsProcessorFeaturePresent(PF_RDTSC_INSTRUCTION_AVAILABLE))
//     {
//         /* Read the TSC value */
//         MachineSpecificCounters->Tsc = __rdtsc();
//     }
// #elif defined(_M_ARM)
//     /* Read the Cycle Counter Register */
//     MachineSpecificCounters->Ccr = _MoveFromCoprocessor(CP15_PMCCNTR);
// #elif defined(_M_ARM64)
//     /* Use the performance counter as entropy source */
//     {
//         LARGE_INTEGER pc = KeQueryPerformanceCounter(NULL);
//         MachineSpecificCounters->Tsc = pc.QuadPart;
//     }
// #else
//     #error Implement me!
// #endif
// }
//
// /*!
//  *  \see http://blogs.msdn.com/b/michael_howard/archive/2005/01/14/353379.aspx (DEAD_LINK)
//  */
// NTSTATUS
// NTAPI
// KsecGatherEntropyData(
//     PKSEC_ENTROPY_DATA EntropyData)
// {
//     MD4_CTX Md4Context;
//     PTEB Teb;
//     PPEB Peb;
//     PWSTR String;
//     ULONG ReturnLength;
//     NTSTATUS Status;
//
//     /* Query some generic values */
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '0');
// #endif
//     EntropyData->CurrentProcessId = PsGetCurrentProcessId();
//     EntropyData->CurrentThreadId = PsGetCurrentThreadId();
//     KeQueryTickCount(&EntropyData->TickCount);
//     KeQuerySystemTime(&EntropyData->SystemTime);
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '1');
// #endif
//     EntropyData->PerformanceCounter = KeQueryPerformanceCounter(
//                                             &EntropyData->PerformanceFrequency);
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '2');
// #endif
//
//     /* Check if we have a TEB/PEB for the process environment */
//     Teb = PsGetCurrentThread()->Tcb.Teb;
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '3');
// #endif
//     if (Teb != NULL)
//     {
//         Peb = Teb->ProcessEnvironmentBlock;
//
//         /* Skip environment hashing — with DUMMY_PSEH, faulting on
//          * invalid PEB/ProcessParameters/Environment pointers is fatal.
//          * During early boot (ksecdd.sys DriverEntry), the calling thread
//          * may be a System process thread with a kernel-mode TEB but no
//          * valid user-mode PEB layout. */
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '4');
// #endif
//     /* Read some machine specific hardware counters */
//     KsecReadMachineSpecificCounters(&EntropyData->MachineSpecificCounters);
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '5');
// #endif
//     /* Query processor performance information */
//     Status = ZwQuerySystemInformation(SystemProcessorPerformanceInformation,
//                                       &EntropyData->SystemProcessorPerformanceInformation,
//                                       sizeof(SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '6');
// #endif
//     /* Query system performance information */
//     Status = ZwQuerySystemInformation(SystemPerformanceInformation,
//                                       &EntropyData->SystemPerformanceInformation,
//                                       sizeof(SYSTEM_PERFORMANCE_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '7');
// #endif
//     /* Query exception information */
//     Status = ZwQuerySystemInformation(SystemExceptionInformation,
//                                       &EntropyData->SystemExceptionInformation,
//                                       sizeof(SYSTEM_EXCEPTION_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '8');
// #endif
//     /* Query lookaside information */
//     Status = ZwQuerySystemInformation(SystemLookasideInformation,
//                                       &EntropyData->SystemLookasideInformation,
//                                       sizeof(SYSTEM_LOOKASIDE_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, '9');
// #endif
//     /* Query interrupt information */
//     Status = ZwQuerySystemInformation(SystemInterruptInformation,
//                                       &EntropyData->SystemInterruptInformation,
//                                       sizeof(SYSTEM_INTERRUPT_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, 'A');
// #endif
//     /* Query process information */
//     Status = ZwQuerySystemInformation(SystemProcessInformation,
//                                       &EntropyData->SystemProcessInformation,
//                                       sizeof(SYSTEM_PROCESS_INFORMATION),
//                                       &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_IX86
//         __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
// #endif
//         return Status;
//     }
//
// #ifdef _M_IX86
//     __outbyte(0xE9, 'e'); __outbyte(0xE9, 'B');
// #endif
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
