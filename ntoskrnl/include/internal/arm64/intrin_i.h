/*
 * ARM64 intrinsics for ReactOS kernel
 */
#pragma once

FORCEINLINE
VOID
KeArm64HaltProcessor(VOID)
{
    //
    // Enter Wait-For-Interrupt Mode (WFI)
    //
    __wfi();
}

FORCEINLINE
ULONG64
KeArm64CurrentEL(VOID)
{
    //
    // Get current exception level
    //
    return __getReg(18) & 0x0F;  // CurrentEL is in CPSR-like location
}

FORCEINLINE
VOID
KeArm64FlushTlb(VOID)
{
    //
    // TLBI VMALLE1 - Invalidate all TLB entries for EL1
    //
    __dmb(_ARM64_BARRIER_ISH);
    __isb(_ARM64_BARRIER_SY);
}

FORCEINLINE
VOID
KeArm64InvalidateTlbEntry(IN PVOID Address)
{
    //
    // TLBI VAE1, <Xt> - Invalidate TLB entry by VA, EL1
    //
    UNREFERENCED_PARAMETER(Address);
    __dmb(_ARM64_BARRIER_ISH);
}

FORCEINLINE
VOID
KeArm64InvalidateAllCaches(VOID)
{
    //
    // Data cache clean and invalidate
    //
    __dmb(_ARM64_BARRIER_ISH);
    __isb(_ARM64_BARRIER_SY);
}

FORCEINLINE
VOID
KeArm64FlushIcache(VOID)
{
    //
    // IC IALLU - Invalidate all instruction caches to PoU
    //
    __isb(_ARM64_BARRIER_SY);
}

FORCEINLINE
VOID
KeArm64WaitForInterrupt(VOID)
{
    //
    // WFI - Wait For Interrupt
    //
    __wfi();
}

/* Page table / MMU control */
FORCEINLINE
ULONG64
KeArm64GetTTBR0(VOID)
{
    /* Read TTBR0_EL1 - Translation Table Base Register 0 */
    return 0;  /* Stub - needs system register access */
}

FORCEINLINE
ULONG64
KeArm64GetTTBR1(VOID)
{
    /* Read TTBR1_EL1 - Translation Table Base Register 1 */
    return 0;  /* Stub - needs system register access */
}

FORCEINLINE
ULONG64
KeArm64GetSCTLR(VOID)
{
    /* Read SCTLR_EL1 - System Control Register */
    return 0;  /* Stub - needs system register access */
}
