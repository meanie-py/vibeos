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

#include "wow64_status.h"

/* wow64_manual.c: hand-written deep marshallers for the manual-list syscalls
 * (struct layouts that differ 32<->64). A name with no marshaller falls
 * through to the generated table, so an unhandled one comes back as
 * WOW64_AUTOMARSHAL_UNIMPL, not WOW64_MANUAL_UNIMPL -- test both. */
NTSTATUS Wow64ManualDispatch(const char *Name, ULONG *Arguments);

/* Serial-noisy per-syscall trace for M3 bring-up (each guest syscall costs a
 * DbgPrint line, callbacks too). OFF since 2026-09-26: the loader path is long
 * stable, and every amd64 boot runs user32_winetest as a WoW64 guest, so the
 * trace grew the serial to ~40MB per sweep and slowed the whole TCG guest until
 * unrelated apps missed their window timeouts. Set to 1 to debug a thunk. */
ULONG Wow64TraceSyscalls = 0;

/* wow64_win32k.c: the win32k (NtGdi/NtUser) table (guest EAX = 0x1000 +
 * w32ksvc32.h index -- the win32k service-table tag bit). */
NTSTATUS Wow64DispatchWin32kService(ULONG ServiceNumber, PULONG Arguments);

NTSTATUS
Wow64DispatchSystemService(ULONG ServiceNumber, PULONG Arguments)
{
    const WOW64_THUNK_ENTRY *Entry;
    NTSTATUS Status;

    if ((ServiceNumber & 0xF000) == 0x1000)
        return Wow64DispatchWin32kService(ServiceNumber, Arguments);

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
        if (WOW64_IS_UNIMPL(Status))
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
