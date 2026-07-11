/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Phase D syscall dispatch — hosts the wow64gen-generated 32->64
 *              thunk table (wow64_thunks.gen.c, built by CMake from
 *              ntdll.spec types + sysfuncs.h service-number order) in its own
 *              translation unit so its generic extern Nt* declarations cannot
 *              clash with wow64.c's typed ones. wow64.c calls
 *              Wow64DispatchSystemService for any non-magic service number.
 */

#define WIN32_NO_STATUS
#include <windef.h>

typedef LONG NTSTATUS;

ULONG
WINAPI
DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

#include "wow64_thunks.gen.c"

#define WOW64_STATUS_INVALID_SYSTEM_SERVICE ((NTSTATUS)0xC000001CL)
#define WOW64_MANUAL_UNIMPL ((NTSTATUS)0x7FFFFFFF)

/* wow64_manual.c: hand-written deep marshallers for the manual-list syscalls
 * (struct layouts that differ 32<->64). Returns WOW64_MANUAL_UNIMPL if the
 * name has no marshaller yet. */
NTSTATUS Wow64ManualDispatch(const char *Name, ULONG *Arguments);

/* Serial-noisy per-syscall trace for M3 bring-up; flip to 0 once the guest
 * loader path is stable (each guest syscall costs a DbgPrint line). */
ULONG Wow64TraceSyscalls = 1;

NTSTATUS
Wow64DispatchSystemService(ULONG ServiceNumber, PULONG Arguments)
{
    const WOW64_THUNK_ENTRY *Entry;
    NTSTATUS Status;

    if (ServiceNumber >= Wow64ThunkCount)
    {
        DbgPrint("[WOW64] service 0x%lx OUT OF RANGE (table has %u) args=%p\n",
                 ServiceNumber, Wow64ThunkCount, Arguments);
        return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
    }

    Entry = &Wow64ThunkTable[ServiceNumber];
    if (!Entry->Thunk)
    {
        Status = Wow64ManualDispatch(Entry->Name, Arguments);
        if (Status == WOW64_MANUAL_UNIMPL)
        {
            DbgPrint("[WOW64] %s (0x%lx) NEEDS MANUAL MARSHALLER — not yet implemented, failing loud\n",
                     Entry->Name, ServiceNumber);
            return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
        }
        if (Wow64TraceSyscalls)
            DbgPrint("[WOW64] %s(0x%lx) [manual] => %08lx\n", Entry->Name, ServiceNumber, Status);
        return Status;
    }

    Status = Entry->Thunk(Arguments);
    if (Wow64TraceSyscalls)
        DbgPrint("[WOW64] %s(0x%lx) => %08lx\n", Entry->Name, ServiceNumber, Status);
    return Status;
}
