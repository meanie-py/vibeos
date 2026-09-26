/*
 * wow64_pfnmap.c — map the 64-bit user32's client window procs to the guest's.
 *
 * win32k gives a built-in control class (Button, Edit, ...) the client proc from
 * gpsi->apfnClientA/W. Those tables are filled by the NATIVE 64-bit user32, so
 * in a WoW64 process every such proc is a 64-bit address the 32-bit guest cannot
 * call. Narrowing one to 32 bits produced an execute fault at its low half
 * (user32:class, eip=b2041ac0 = apfnClientW[0] of the host).
 *
 * The guest has its own tables with the same slot order (it passes them to
 * NtUserInitializeClientPfnArrays, which wow64_manual.c intercepts and keeps out
 * of the GLOBAL gpsi). So a proc found at slot i of apfnClientA/W maps to slot i
 * of the guest's A/W table. Anything else passes through narrowed, unchanged.
 *
 * This is the only file in wow64.dll that includes ntuser.h. The SERVERINFO
 * offsets come from the real header, never hand-counted.
 */

#define WIN32_NO_STATUS
#define _INC_WINDOWS
#define COM_NO_WINDOWS_H
#include <windef.h>
#include <winbase.h>
#include <wingdi.h>
#include <winuser.h>
#include <imm.h>
#include <immdev.h>
#include <imm32_undoc.h>
#include <ndk/rtlfuncs.h>
#include <ntusrtyp.h>
#include <ntuser.h>

C_ASSERT(sizeof(PFNCLIENT) % sizeof(ULONG_PTR) == 0);
#define N_PFNCLIENT (sizeof(PFNCLIENT) / sizeof(ULONG_PTR))

static const SERVERINFO *Wow64pPsi64;   /* this process's view of gpsi */
static const ULONG *Wow64pGuestPfnA;    /* guest PFNCLIENT, 4-byte slots */
static const ULONG *Wow64pGuestPfnW;

/* NtUserProcessConnect: the 64-bit siClient.psi (a client VA in this process). */
VOID Wow64pPfnMapSetPsi(ULONGLONG Psi64)
{
    Wow64pPsi64 = (const SERVERINFO *)(ULONG_PTR)Psi64;
}

/* NtUserInitializeClientPfnArrays: the guest's pfnClientA/W. They are statics
 * in the guest's user32, so the pointers stay valid for the process lifetime,
 * and reading through them at map time always sees the current values. */
VOID Wow64pPfnMapSetGuest(ULONG PfnA32, ULONG PfnW32)
{
    Wow64pGuestPfnA = (const ULONG *)(ULONG_PTR)PfnA32;
    Wow64pGuestPfnW = (const ULONG *)(ULONG_PTR)PfnW32;
}

/* A proc about to reach the guest. Returns the guest's equivalent or the
 * narrowed value. */
ULONG Wow64pMapClientProc(ULONGLONG Proc64)
{
    const ULONG_PTR *A, *W;
    SIZE_T i;

    if (!Proc64 || Proc64 <= 0xFFFFFFFFULL || !Wow64pPsi64 ||
        !Wow64pGuestPfnA || !Wow64pGuestPfnW)
        return (ULONG)Proc64;

    A = (const ULONG_PTR *)&Wow64pPsi64->apfnClientA;
    W = (const ULONG_PTR *)&Wow64pPsi64->apfnClientW;
    for (i = 0; i < N_PFNCLIENT; i++)
    {
        if (W[i] == Proc64)
            return Wow64pGuestPfnW[i];
        if (A[i] == Proc64)
            return Wow64pGuestPfnA[i];
    }
    return (ULONG)Proc64;
}

/* The inverse, for a proc the guest hands the kernel (a class or window proc,
 * GetCPD's ProcIn). A guest system proc becomes the host's proc for the same
 * slot, so win32k's own A/W bookkeeping, which compares against gpsi, sees
 * the procs it expects. Without it an ANSI Button kept its W proc under the
 * ANSI flag, and every GetWindowLongPtrW returned a CPD handle. */
ULONGLONG Wow64pUnmapClientProc(ULONG Proc32)
{
    const ULONG_PTR *A, *W;
    SIZE_T i;

    if (!Proc32 || !Wow64pPsi64 || !Wow64pGuestPfnA || !Wow64pGuestPfnW)
        return Proc32;

    A = (const ULONG_PTR *)&Wow64pPsi64->apfnClientA;
    W = (const ULONG_PTR *)&Wow64pPsi64->apfnClientW;
    for (i = 0; i < N_PFNCLIENT; i++)
    {
        if (Wow64pGuestPfnW[i] == Proc32)
            return W[i];
        if (Wow64pGuestPfnA[i] == Proc32)
            return A[i];
    }
    return Proc32;
}
