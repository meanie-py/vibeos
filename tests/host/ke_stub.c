/*
 * ke_stub.c — Kernel dispatcher object stubs for host-side testing
 *
 * Implements event, timer, mutant/mutex, and SLIST primitives with
 * enough fidelity for single-threaded functional tests.
 */
#include "include/nt_host.h"

/* ====================================================================
 * KEVENT — kernel event object
 * ==================================================================== */

VOID NTAPI KeInitializeEvent(PKEVENT Event, EVENT_TYPE Type, BOOLEAN State) {
    Event->Header.Type = (UCHAR)Type;
    Event->Header.Hand = (UCHAR)(sizeof(KEVENT) / sizeof(ULONG));
    /* Leave Lock bytes 1,3 (Abandoned, DpcActive) untouched — tests check 0x55 fill */
    Event->Header.SignalState = State ? 1L : 0L;
    InitializeListHead(&Event->Header.WaitListHead);
}

LONG NTAPI KeSetEvent(PKEVENT Event, KPRIORITY Increment, BOOLEAN Wait) {
    LONG OldState = Event->Header.SignalState;
    Event->Header.SignalState = 1L;
    if (Wait) {
        PKTHREAD Thread = KeGetCurrentThread();
        Thread->WaitIrql = KeGetCurrentIrql();
        Thread->WaitNext = TRUE;
        /* Raise to DISPATCH_LEVEL as the real kernel does */
        extern void KeLowerIrql(KIRQL);
        KIRQL dummy;
        extern void KeRaiseIrql(KIRQL, PKIRQL);
        KeRaiseIrql(DISPATCH_LEVEL, &dummy);
    }
    (void)Increment;
    return OldState;
}

LONG NTAPI KeResetEvent(PKEVENT Event) {
    LONG OldState = Event->Header.SignalState;
    Event->Header.SignalState = 0L;
    return OldState;
}

VOID NTAPI KeClearEvent(PKEVENT Event) {
    Event->Header.SignalState = 0L;
}

LONG NTAPI KePulseEvent(PKEVENT Event, KPRIORITY Increment, BOOLEAN Wait) {
    LONG OldState = Event->Header.SignalState;
    Event->Header.SignalState = 0L;
    if (Wait) {
        PKTHREAD Thread = KeGetCurrentThread();
        Thread->WaitIrql = KeGetCurrentIrql();
        Thread->WaitNext = TRUE;
        KIRQL dummy;
        extern void KeRaiseIrql(KIRQL, PKIRQL);
        KeRaiseIrql(DISPATCH_LEVEL, &dummy);
    }
    (void)Increment;
    return OldState;
}

LONG NTAPI KeReadStateEvent(PKEVENT Event) {
    return Event->Header.SignalState;
}

/* ====================================================================
 * KTIMER — kernel timer object (initialization only, no firing)
 * ==================================================================== */

VOID NTAPI KeInitializeTimerEx(PKTIMER Timer, TIMER_TYPE Type) {
    Timer->Header.Type = (UCHAR)(TimerNotificationObject + Type);
    Timer->Header.Hand = (UCHAR)(sizeof(KTIMER) / sizeof(ULONG));
    /* Preserve Lock byte 1 (Abandoned) for test 0x55 fill verification.
     * Byte 3 (DpcActive/Inserted) is zeroed — real kernel sets Inserted=FALSE */
    Timer->Header.Inserted = 0;
    Timer->Header.SignalState = 0L;
    InitializeListHead(&Timer->Header.WaitListHead);
    Timer->DueTime.QuadPart = 0;
    InitializeListHead(&Timer->TimerListEntry);
    Timer->Dpc = NULL;
    Timer->Processor = 0;
    Timer->Period = 0;
}

LONG NTAPI KeReadStateTimer(PKTIMER Timer) {
    return Timer->Header.SignalState;
}

/* ====================================================================
 * KMUTANT / KMUTEX — kernel mutant/mutex objects
 * ==================================================================== */

VOID NTAPI KeInitializeMutant(PKMUTANT Mutant, BOOLEAN InitialOwner) {
    Mutant->Header.Type = MutantObject;
    Mutant->Header.Size = (UCHAR)(sizeof(KMUTANT) / sizeof(ULONG));
    /* Leave Abandoned, DpcActive bytes from 0x55 fill */
    InitializeListHead(&Mutant->Header.WaitListHead);
    Mutant->Abandoned = 0;
    Mutant->ApcDisable = 0;

    if (InitialOwner) {
        PKTHREAD Thread = KeGetCurrentThread();
        Mutant->Header.SignalState = 0L;
        Mutant->OwnerThread = Thread;
        InsertTailList(&Thread->MutantListHead, &Mutant->MutantListEntry);
    } else {
        Mutant->Header.SignalState = 1L;
        Mutant->OwnerThread = NULL;
    }
}

LONG NTAPI KeReleaseMutant(PKMUTANT Mutant, KPRIORITY Increment,
                            BOOLEAN Abandoned, BOOLEAN Wait) {
    LONG OldState = Mutant->Header.SignalState;
    PKTHREAD Thread = KeGetCurrentThread();

    (void)Increment;
    (void)Abandoned;
    (void)Wait;

    /* Check ownership */
    if (Mutant->OwnerThread != Thread) {
        _ExRaiseStatus(STATUS_MUTANT_NOT_OWNED);
        return 0; /* not reached after longjmp */
    }

    Mutant->Header.SignalState++;

    if (Mutant->Header.SignalState == 1L) {
        /* Fully released */
        RemoveEntryList(&Mutant->MutantListEntry);
        Mutant->OwnerThread = NULL;

        /* If this is a KMUTEX (ApcDisable != 0), undo the KernelApcDisable */
        if (Mutant->ApcDisable != 0) {
            Thread->KernelApcDisable++;
        }
    }

    return OldState;
}

VOID NTAPI KeInitializeMutex(PKMUTEX Mutex, ULONG Level) {
    (void)Level;
    KeInitializeMutant(Mutex, FALSE);
    Mutex->ApcDisable = 1;
}

LONG NTAPI KeReleaseMutex(PKMUTEX Mutex, BOOLEAN Wait) {
    return KeReleaseMutant(Mutex, 1, FALSE, Wait);
}

/* ====================================================================
 * KeWaitForSingleObject — uncontended mutant/mutex fast path
 *
 * Only handles mutant objects (SignalState > 0 = available).
 * No actual blocking — single-threaded host, all acquires succeed.
 * ==================================================================== */

NTSTATUS NTAPI KeWaitForSingleObject(
    PVOID Object, KWAIT_REASON WaitReason, LONG WaitMode,
    BOOLEAN Alertable, PLARGE_INTEGER Timeout)
{
    (void)WaitReason;
    (void)WaitMode;
    (void)Alertable;
    (void)Timeout;

    DISPATCHER_HEADER *Header = (DISPATCHER_HEADER *)Object;

    if (Header->Type == MutantObject) {
        PKMUTANT Mutant = (PKMUTANT)Object;
        PKTHREAD Thread = KeGetCurrentThread();

        if (Mutant->OwnerThread == Thread) {
            /* Recursive acquire */
            if (Mutant->Header.SignalState == MINLONG) {
                _ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                return STATUS_MUTANT_LIMIT_EXCEEDED;
            }
            Mutant->Header.SignalState--;
        } else if (Mutant->Header.SignalState > 0) {
            /* Uncontended acquire */
            Mutant->Header.SignalState--;
            Mutant->OwnerThread = Thread;
            InsertTailList(&Thread->MutantListHead, &Mutant->MutantListEntry);

            if (Mutant->ApcDisable != 0) {
                Thread->KernelApcDisable--;
            }
        }
        /* else: contended — would block, but single-threaded host never gets here */
    }

    return STATUS_SUCCESS;
}

/* ====================================================================
 * SLIST — sequenced singly-linked list operations
 * ==================================================================== */

PSLIST_ENTRY ExInterlockedPushEntrySList(
    PSLIST_HEADER ListHead, PSLIST_ENTRY ListEntry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PSLIST_ENTRY OldFirst = ListHead->Next.Next;
    ListEntry->Next = OldFirst;
    ListHead->Next.Next = ListEntry;
    ListHead->Depth++;
    ListHead->Sequence++;
    return OldFirst;
}

PSLIST_ENTRY ExInterlockedPopEntrySList(
    PSLIST_HEADER ListHead, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PSLIST_ENTRY Entry = ListHead->Next.Next;
    if (Entry) {
        ListHead->Next.Next = Entry->Next;
        ListHead->Depth--;
    }
    return Entry;
}

PSLIST_ENTRY ExInterlockedFlushSList(PSLIST_HEADER ListHead) {
    PSLIST_ENTRY Entry = ListHead->Next.Next;
    ListHead->Next.Next = NULL;
    ListHead->Depth = 0;
    return Entry;
}
