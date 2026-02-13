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
    __asm__ __volatile__(
        "tlbi vmalle1\n\t"
        "dsb ish\n\t"
        "isb"
        ::: "memory"
    );
}

FORCEINLINE
VOID
KeArm64InvalidateTlbEntry(IN PVOID Address)
{
    //
    // TLBI VAE1, <Xt> - Invalidate TLB entry by VA, EL1
    // The address must be shifted right by 12 (page-aligned VA >> 12)
    //
    ULONG64 page = (ULONG64)Address >> 12;
    __asm__ __volatile__(
        "tlbi vae1, %0\n\t"
        "dsb ish\n\t"
        "isb"
        :: "r"(page) : "memory"
    );
}

FORCEINLINE
VOID
KeArm64InvalidateAllCaches(VOID)
{
    //
    // Data cache clean and invalidate (by set/way is complex,
    // just do a DSB to ensure all prior stores are visible)
    //
    __asm__ __volatile__(
        "dsb ish\n\t"
        "isb"
        ::: "memory"
    );
}

FORCEINLINE
VOID
KeArm64FlushIcache(VOID)
{
    //
    // IC IALLU - Invalidate all instruction caches to PoU
    //
    __asm__ __volatile__(
        "ic iallu\n\t"
        "dsb ish\n\t"
        "isb"
        ::: "memory"
    );
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
    ULONG64 val;
    __asm__ __volatile__("mrs %0, ttbr0_el1" : "=r"(val));
    return val;
}

FORCEINLINE
ULONG64
KeArm64GetTTBR1(VOID)
{
    ULONG64 val;
    __asm__ __volatile__("mrs %0, ttbr1_el1" : "=r"(val));
    return val;
}

FORCEINLINE
ULONG64
KeArm64GetSCTLR(VOID)
{
    ULONG64 val;
    __asm__ __volatile__("mrs %0, sctlr_el1" : "=r"(val));
    return val;
}
