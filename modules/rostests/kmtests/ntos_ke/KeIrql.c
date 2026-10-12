/*
 * PROJECT:         ReactOS kernel-mode tests
 * LICENSE:         GPLv2+ - See COPYING in the top level directory
 * PURPOSE:         Kernel-Mode Test Suite Interrupt Request Level test
 * PROGRAMMER:      Thomas Faber <thomas.faber@reactos.org>
 */

#if defined(_M_AMD64) || defined(_M_ARM64)
/* 64-bit has a single CLOCK_LEVEL; KeRaiseIrql/KeLowerIrql are macros */
#define CLOCK1_LEVEL CLOCK_LEVEL
#define CLOCK2_LEVEL CLOCK_LEVEL
#else
__declspec(dllimport) void __stdcall KeRaiseIrql(unsigned char, unsigned char *);
__declspec(dllimport) void __stdcall KeLowerIrql(unsigned char);
#endif

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

#ifdef _M_AMD64
/* Windows x64's ntoskrnl does not export KeRaiseIrqlToSynchLevel: the WDK makes it an
 * inline KfRaiseIrql(SYNCH_LEVEL). Ours exports it (xdk/amd64/ke.h says why), and a static
 * import of it kept the whole of kmtest_drv.sys off real Windows -- StartService failed
 * with ERROR_PROC_NOT_FOUND and not one kmtest ran (`vibeos-cli winref kmtest`,
 * 2026-10-12). So resolve it at run time, at PASSIVE_LEVEL, and when it is absent do what
 * the WDK inline does. */
static KIRQL (NTAPI *ExportedRaiseToSynch)(VOID);
static KIRQL TestRaiseIrqlToSynchLevel(KIRQL SynchIrql)
{
    if (ExportedRaiseToSynch)
        return ExportedRaiseToSynch();
    return KfRaiseIrql(SynchIrql);
}
#define KeRaiseIrqlToSynchLevel() TestRaiseIrqlToSynchLevel(SynchIrql)
#endif

START_TEST(KeIrql)
{
    KIRQL Irql, Irql2, PrevIrql, SynchIrql;

    /* we should be called at PASSIVE_LEVEL */
    ok_irql(PASSIVE_LEVEL);

#ifdef _M_AMD64
    {
        UNICODE_STRING Name = RTL_CONSTANT_STRING(L"KeRaiseIrqlToSynchLevel");
        ExportedRaiseToSynch = (KIRQL (NTAPI *)(VOID))MmGetSystemRoutineAddress(&Name);
    }
#endif

    PrevIrql = KeGetCurrentIrql();

    /* SYNCH_LEVEL is different for UP/MP */
    if (KmtIsMultiProcessorBuild)
        SynchIrql = IPI_LEVEL - 2;
    else
        SynchIrql = DISPATCH_LEVEL;

    /* some Irqls MUST work */
    {
    const KIRQL Irqls[] = { LOW_LEVEL, PASSIVE_LEVEL, APC_LEVEL, DISPATCH_LEVEL,
#ifndef _M_ARM64 /* CMCI is an x86 machine-check concept; no such IRQL on ARM64 */
                            CMCI_LEVEL,
#endif
                            CLOCK1_LEVEL, CLOCK2_LEVEL, CLOCK_LEVEL,
                            PROFILE_LEVEL, IPI_LEVEL, /*POWER_LEVEL,*/ SynchIrql, HIGH_LEVEL };
    int i;
    for (i = 0; i < sizeof Irqls / sizeof Irqls[0]; ++i)
    {
        KeRaiseIrql(Irqls[i], &Irql2);
        ok_eq_uint(Irql2, PrevIrql);
        ok_irql(Irqls[i]);
        KeLowerIrql(Irql2);
        ok_irql(PrevIrql);
    }
    }

    /* raising/lowering to the current level should have no effect */
    ok_irql(PASSIVE_LEVEL);
    KeRaiseIrql(PASSIVE_LEVEL, &Irql);
    ok_eq_uint(Irql, PASSIVE_LEVEL);
    KeLowerIrql(PASSIVE_LEVEL);
    ok_irql(PASSIVE_LEVEL);

    /* try to raise to each Irql and back */
    for (Irql = PASSIVE_LEVEL; Irql <= HIGH_LEVEL; ++Irql)
    {
        DPRINT("Raising to %u\n", Irql);
        KeRaiseIrql(Irql, &Irql2);
        ok_eq_uint(Irql2, PrevIrql);
        KeLowerIrql(Irql2);
        ok_irql(PrevIrql);
    }

    /* go through all Irqls in order, skip the ones that the system doesn't accept */
    for (Irql = PASSIVE_LEVEL; Irql <= HIGH_LEVEL; ++Irql)
    {
        DPRINT("Raising to %u\n", Irql);
        KeRaiseIrql(Irql, &Irql2);
        ok_eq_uint(Irql2, PrevIrql);
        Irql2 = KeGetCurrentIrql();
        ok(Irql2 <= Irql, "New Irql is %u, expected <= requested value of %u\n", Irql2, Irql);
        PrevIrql = Irql2;
    }

    ok_irql(HIGH_LEVEL);

    /* now go back again, skipping the ones that don't work */
    for (Irql = HIGH_LEVEL; Irql > PASSIVE_LEVEL;)
    {
        DPRINT("Lowering to %u\n", Irql - 1);
        KeLowerIrql(Irql - 1);
        Irql2 = KeGetCurrentIrql();
        ok(Irql2 < Irql, "New Irql is %u, expected <= requested value of %u\n", Irql2, Irql - 1);
        if (Irql2 < Irql)
            Irql = Irql2;
        else
            --Irql;
    }

    /* test KeRaiseIrqlToDpcLevel */
    ok_irql(PASSIVE_LEVEL);
    Irql = KeRaiseIrqlToDpcLevel();
    ok_irql(DISPATCH_LEVEL);
    ok_eq_uint(Irql, PASSIVE_LEVEL);
    Irql = KeRaiseIrqlToDpcLevel();
    ok_irql(DISPATCH_LEVEL);
    ok_eq_uint(Irql, DISPATCH_LEVEL);
    KeLowerIrql(PASSIVE_LEVEL);

    /* test KeRaiseIrqlToSynchLevel */
    ok_irql(PASSIVE_LEVEL);
    Irql = KeRaiseIrqlToSynchLevel();
    ok_irql(SynchIrql);
    ok_eq_uint(Irql, PASSIVE_LEVEL);
    Irql = KeRaiseIrqlToSynchLevel();
    ok_irql(SynchIrql);
    ok_eq_uint(Irql, SynchIrql);
    KeLowerIrql(PASSIVE_LEVEL);

    /* these bugcheck on a checked build but run fine on free! */
    if (!KmtIsCheckedBuild)
    {
        KeRaiseIrql(HIGH_LEVEL, &Irql);
        KeRaiseIrql(APC_LEVEL, &Irql);
        ok_irql(APC_LEVEL);
        KeLowerIrql(HIGH_LEVEL);
        ok_irql(HIGH_LEVEL);
        KeLowerIrql(PASSIVE_LEVEL);
    }

#if !defined(_M_AMD64) && !defined(_M_ARM64)
    /* try the actual exports, not only the fastcall versions */
    ok_irql(PASSIVE_LEVEL);
    (KeRaiseIrql)(HIGH_LEVEL, &Irql);
    ok_irql(HIGH_LEVEL);
    ok_eq_uint(Irql, PASSIVE_LEVEL);
    (KeLowerIrql)(Irql);
    ok_irql(PASSIVE_LEVEL);
#endif

    /* make sure we exit gracefully */
    ok_irql(PASSIVE_LEVEL);
    KeLowerIrql(PASSIVE_LEVEL);
}
