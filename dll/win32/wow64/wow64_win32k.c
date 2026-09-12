/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     wow64win surface — hosts the wow64gen-generated win32k
 *              (NtGdi/NtUser) 32->64 thunk table in its own translation
 *              unit (its generic extern decls must not clash with wow64.c's
 *              or wow64_thunks.c's). Guest EAX carries 0x1000 + index into
 *              w32ksvc32.h order; the host calls resolve through the native
 *              win32u.dll importlib (pure syscall stubs).
 *              See docs/engineering/wow64-plan.md (M4/wow64win).
 */

#define WIN32_NO_STATUS
#include <windef.h>

typedef LONG NTSTATUS;

ULONG
WINAPI
DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

#include "wow64_win32k_thunks.gen.c"

#include "wow64_status.h"

/* Shared with wow64_thunks.c (name-keyed, so one manual dispatcher serves
 * both tables). */
NTSTATUS Wow64ManualDispatch(const char *Name, ULONG *Arguments);
extern ULONG Wow64TraceSyscalls;

/* wow64.c — the guest/host last-error mirror. win32k reports failure by
 * setting last-error in the 64-bit TEB, which a 32-bit guest cannot see; this
 * is the one place every win32k syscall passes through, so it is the one place
 * that has to bridge the syscall direction. (wow64_callback.c bridges the
 * other direction, and has to — see the rationale at the definitions.) */
ULONG Wow64LastErrorToHost(VOID);
VOID Wow64LastErrorToGuestIfChanged(ULONG WasOnEntry);

NTSTATUS
Wow64DispatchWin32kService(ULONG ServiceNumber, PULONG Arguments)
{
    const WOW64_THUNK_ENTRY *Entry;
    ULONG Index = ServiceNumber - 0x1000;
    ULONG LastError;
    NTSTATUS Status;

    if (Index >= Wow64Win32kThunkCount)
    {
        DbgPrint("[WOW64] win32k service 0x%lx OUT OF RANGE (table has %u) args=%p\n",
                 ServiceNumber, Wow64Win32kThunkCount, Arguments);
        return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
    }

    Entry = &Wow64Win32kThunkTable[Index];
    LastError = Wow64LastErrorToHost();

    if (!Entry->Thunk)
    {
        Status = Wow64ManualDispatch(Entry->Name, Arguments);
        if (WOW64_IS_UNIMPL(Status))
        {
            DbgPrint("[WOW64] %s (0x%lx) NEEDS MANUAL MARSHALLER (win32k) — failing loud\n",
                     Entry->Name, ServiceNumber);
            Status = WOW64_STATUS_INVALID_SYSTEM_SERVICE;
        }
        else if (Wow64TraceSyscalls)
        {
            DbgPrint("[WOW64] %s(0x%lx) [manual] => %08lx\n", Entry->Name, ServiceNumber, Status);
        }
    }
    else
    {
        /* M9: on-entry trace so a thunk that FAULTS the kernel (e.g. a
         * passthrough that hands a 4-aligned WoW64 buffer to an align-8
         * ProbeForRead) still names itself on serial — the "=>" line below
         * never prints in that case. */
        if (Wow64TraceSyscalls)
            DbgPrint("[WOW64>] %s(0x%lx) enter arg0=%08lx arg1=%08lx arg2=%08lx\n",
                     Entry->Name, ServiceNumber, Arguments[0], Arguments[1], Arguments[2]);
        Status = Entry->Thunk(Arguments);
        if (Wow64TraceSyscalls)
            DbgPrint("[WOW64] %s(0x%lx) => %08lx\n", Entry->Name, ServiceNumber, Status);
    }

    Wow64LastErrorToGuestIfChanged(LastError);
    return Status;
}
