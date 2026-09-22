/*
 * PROJECT:     ReactOS Kernel Security Support Provider Interface Driver
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     Random number generation and entropy gathering
 * COPYRIGHT:   Copyright 2014-2017 Timo Kreuzer <timo.kreuzer@reactos.org>
 */

/* INCLUDES *******************************************************************/

#include "ksecdd.h"

#define NDEBUG
#include <debug.h>


/* GLOBALS ********************************************************************/

static ULONG KsecRandomSeed = 0x62b409a1;


/* FUNCTIONS ******************************************************************/

NTSTATUS
NTAPI
KsecGenRandom(
    PVOID Buffer,
    SIZE_T Length)
{
    LARGE_INTEGER TickCount;
    ULONG i, RandomValue;
    PULONG P;

    /* Try to generate a more random seed */
    KeQueryTickCount(&TickCount);
    KsecRandomSeed ^= _rotl(TickCount.LowPart, (KsecRandomSeed % 23));

    P = Buffer;
    for (i = 0; i < Length / sizeof(ULONG); i++)
    {
        P[i] = RtlRandomEx(&KsecRandomSeed);
    }

    Length &= (sizeof(ULONG) - 1);
    if (Length > 0)
    {
        RandomValue = RtlRandomEx(&KsecRandomSeed);
        RtlCopyMemory(&P[i], &RandomValue, Length);
    }

    return STATUS_SUCCESS;
}

VOID
NTAPI
KsecReadMachineSpecificCounters(
    _Out_ PKSEC_MACHINE_SPECIFIC_COUNTERS MachineSpecificCounters)
{
#if defined(_M_IX86) || defined(_M_AMD64)
    /* Check if RDTSC is available */
    if (ExIsProcessorFeaturePresent(PF_RDTSC_INSTRUCTION_AVAILABLE))
    {
        /* Read the TSC value */
        MachineSpecificCounters->Tsc = __rdtsc();
    }
#if 0 // FIXME: investigate what the requirements are for these
    /* Read the CPU event counter MSRs */
    //MachineSpecificCounters->Ctr0 = __readmsr(0x12);
    //MachineSpecificCounters->Ctr1 = __readmsr(0x13);

    /* Check if this is an MMX capable CPU */
    if (ExIsProcessorFeaturePresent(PF_MMX_INSTRUCTIONS_AVAILABLE))
    {
        /* Read the CPU performance counters 0 and 1 */
        MachineSpecificCounters->Pmc0 = __readpmc(0);
        MachineSpecificCounters->Pmc1 = __readpmc(1);
    }
#endif
#elif defined(_M_ARM)
    /* Read the Cycle Counter Register */
    MachineSpecificCounters->Ccr = _MoveFromCoprocessor(CP15_PMCCNTR);
#elif defined(_M_ARM64)
    /* Use the performance counter as entropy source */
    {
        LARGE_INTEGER pc = KeQueryPerformanceCounter(NULL);
        MachineSpecificCounters->Tsc = pc.QuadPart;
    }
#else
    #error Implement me!
#endif
}

/*!
 *  \see http://blogs.msdn.com/b/michael_howard/archive/2005/01/14/353379.aspx (DEAD_LINK)
 */
NTSTATUS
NTAPI
KsecGatherEntropyData(
    PKSEC_ENTROPY_DATA EntropyData)
{
    MD4_CTX Md4Context;
    PTEB Teb;
    PPEB Peb;
    PWSTR String;
    ULONG ReturnLength;
    NTSTATUS Status;

    /* Query some generic values */
#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '0');
#endif
    EntropyData->CurrentProcessId = PsGetCurrentProcessId();
    EntropyData->CurrentThreadId = PsGetCurrentThreadId();
    KeQueryTickCount(&EntropyData->TickCount);
    KeQuerySystemTime(&EntropyData->SystemTime);
#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '1');
#endif
    EntropyData->PerformanceCounter = KeQueryPerformanceCounter(
                                            &EntropyData->PerformanceFrequency);
#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '2');
#endif

    /* Check if we have a TEB/PEB for the process environment */
    Teb = PsGetCurrentThread()->Tcb.Teb;
#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '3');
#endif
    if (Teb != NULL)
    {
        Peb = Teb->ProcessEnvironmentBlock;

        /* Skip environment hashing — with DUMMY_PSEH, faulting on
         * invalid PEB/ProcessParameters/Environment pointers is fatal.
         * During early boot (ksecdd.sys DriverEntry), the calling thread
         * may be a System process thread with a kernel-mode TEB but no
         * valid user-mode PEB layout. */
#if 0
        /* Initialize the MD4 context */
        MD4Init(&Md4Context);
        _SEH2_TRY
        {
            /* Get the end of the environment */
            String = Peb->ProcessParameters->Environment;
            while (*String)
            {
                String += wcslen(String) + 1;
            }

            /* Update the MD4 context from the environment data */
            MD4Update(&Md4Context,
                      (PUCHAR)Peb->ProcessParameters->Environment,
                      (ULONG)((PUCHAR)String - (PUCHAR)Peb->ProcessParameters->Environment));
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            /* Simply ignore the exception */
        }
        _SEH2_END;

        /* Finalize and copy the MD4 hash */
        MD4Final(&Md4Context);
        RtlCopyMemory(&EntropyData->EnvironmentHash, Md4Context.digest, 16);
#endif
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '4');
#endif
    /* Read some machine specific hardware counters */
    KsecReadMachineSpecificCounters(&EntropyData->MachineSpecificCounters);

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '5');
#endif
    /* Query processor performance information */
    Status = ZwQuerySystemInformation(SystemProcessorPerformanceInformation,
                                      &EntropyData->SystemProcessorPerformanceInformation,
                                      sizeof(SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '6');
#endif
    /* Query system performance information */
    Status = ZwQuerySystemInformation(SystemPerformanceInformation,
                                      &EntropyData->SystemPerformanceInformation,
                                      sizeof(SYSTEM_PERFORMANCE_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '7');
#endif
    /* Query exception information */
    Status = ZwQuerySystemInformation(SystemExceptionInformation,
                                      &EntropyData->SystemExceptionInformation,
                                      sizeof(SYSTEM_EXCEPTION_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '8');
#endif
    /* Query lookaside information */
    Status = ZwQuerySystemInformation(SystemLookasideInformation,
                                      &EntropyData->SystemLookasideInformation,
                                      sizeof(SYSTEM_LOOKASIDE_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, '9');
#endif
    /* Query interrupt information */
    Status = ZwQuerySystemInformation(SystemInterruptInformation,
                                      &EntropyData->SystemInterruptInformation,
                                      sizeof(SYSTEM_INTERRUPT_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, 'A');
#endif
    /* Query process information */
    Status = ZwQuerySystemInformation(SystemProcessInformation,
                                      &EntropyData->SystemProcessInformation,
                                      sizeof(SYSTEM_PROCESS_INFORMATION),
                                      &ReturnLength);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_IX86
        __outbyte(0xE9, 'e'); __outbyte(0xE9, 'X');
#endif
        return Status;
    }

#ifdef _M_IX86
    __outbyte(0xE9, 'e'); __outbyte(0xE9, 'B');
#endif
    return STATUS_SUCCESS;
}
