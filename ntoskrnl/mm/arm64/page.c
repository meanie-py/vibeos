/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/mm/arm64/page.c
 * PURPOSE:         ARM64 legacy MM page table manipulation
 * PROGRAMMERS:     ReactOS ARM64 port
 *
 * This file implements the architecture-specific page table functions
 * needed by the legacy (non-ARM3) ReactOS memory manager. These are
 * used by the section/cache code path (MmNotPresentFaultSectionView,
 * MmAccessFaultSectionView, etc.) to create/query/delete virtual
 * mappings for section-backed pages (VACBs, file caches, etc.).
 *
 * The ARM3 memory manager has its own PTE manipulation path that does
 * NOT go through these functions.
 */

/* INCLUDES ******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#define MODULE_INVOLVED_IN_ARM3
#include <mm/ARM3/miarm.h>

/* UART TRACES ***************************************************************/

extern volatile PVOID Arm64UartKernelBase;
#define PL011_BASE ((volatile ULONG *)Arm64UartKernelBase)

static void PagePutChar(char c) { *PL011_BASE = (ULONG)c; }
static void PagePuts(const char *s) { while (*s) PagePutChar(*s++); }
static void PagePutHex64(ULONG64 val)
{
    int i;
    PagePuts("0x");
    for (i = 60; i >= 0; i -= 4)
        PagePutChar("0123456789ABCDEF"[(val >> i) & 0xF]);
}

/* Only print per-page verbose traces when trace flag is defined */
#ifdef ARM64_UART_TRACE_ENABLE
#define PAGE_TRACE(s) PagePuts(s)
#define PAGE_TRACE_HEX(v) PagePutHex64(v)
#else
#define PAGE_TRACE(s) ((void)0)
#define PAGE_TRACE_HEX(v) ((void)0)
#endif

/* PAGE TABLE HIERARCHY HELPERS **********************************************/

/*
 * MiEnsureKernelPdeValid — Create missing PDE (and PPE if needed) for a kernel
 * address on demand. The legacy MM code (MmNotPresentFaultSectionView,
 * MmCreateVirtualMapping) can allocate kernel VAs via MmCreateMemoryArea that
 * fall outside the system PTE space initialized during boot. Those addresses
 * may not have PDEs yet. Instead of bugchecking, we create them here.
 *
 * PXE is assumed valid — FreeLDR + MiInitializePageTable create PXEs for
 * the entire upper-half kernel space.
 */
static
VOID
MiEnsureKernelPdeValid(
    _In_ PVOID Address)
{
    PMMPTE PointerPxe = MiAddressToPxe(Address);
    PMMPTE PointerPpe = MiAddressToPpe(Address);
    PMMPDE PointerPde = MiAddressToPde(Address);
    PFN_NUMBER PageFrameIndex;
    MMPTE TempPte;
    KIRQL OldIrql;

    ASSERT(Address >= MmSystemRangeStart);

    /* PXE must be valid (created during boot for all kernel space) */
    if (!PointerPxe->u.Hard.Valid)
    {
        PagePuts("[PAGE] BUG: PXE invalid, cannot create\r\n");
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    /* Create PPE if needed */
    if (!PointerPpe->u.Hard.Valid)
    {
        /* PPE creation trace — disabled for clean console */

        OldIrql = MiAcquirePfnLock();
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiReleasePfnLock(OldIrql);

        TempPte.u.Long = ValidKernelPde.u.Long;
        TempPte.u.Hard.PageFrameNumber = PageFrameIndex;
        MI_WRITE_VALID_PTE(PointerPpe, TempPte);
        __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");
        RtlZeroMemory(MiPteToAddress(PointerPpe), PAGE_SIZE);
    }

    /* Create PDE if needed */
    if (!PointerPde->u.Hard.Valid)
    {
        /* PDE creation trace — disabled for clean console */

        OldIrql = MiAcquirePfnLock();
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiReleasePfnLock(OldIrql);

        TempPte.u.Long = ValidKernelPde.u.Long;
        TempPte.u.Hard.PageFrameNumber = PageFrameIndex;
        MI_WRITE_VALID_PTE(PointerPde, TempPte);
        __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");
        RtlZeroMemory(MiPteToAddress(PointerPde), PAGE_SIZE);
        /* PDE created OK trace — disabled */
    }
}

/* FUNCTIONS *****************************************************************/

BOOLEAN
NTAPI
MmIsPagePresent(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    if (Address >= MmSystemRangeStart)
    {
        ASSERT(Process == NULL);

        if (!MiIsPdeForAddressValid(Address))
            return FALSE;

        return MiAddressToPte(Address)->u.Hard.Valid != 0;
    }

    /* User-mode: not yet implemented for ARM64 */
    ASSERT(Process != NULL);
    ASSERT(Process == PsGetCurrentProcess());

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if (!MiAddressToPxe(Address)->u.Hard.Valid ||
        !MiAddressToPpe(Address)->u.Hard.Valid ||
        !MiAddressToPde(Address)->u.Hard.Valid)
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return FALSE;
    }

    BOOLEAN Ret = MiAddressToPte(Address)->u.Hard.Valid != 0;
    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    return Ret;
}

BOOLEAN
NTAPI
MmIsDisabledPage(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    PMMPTE PointerPte;
    BOOLEAN Ret;

    if (Address >= MmSystemRangeStart)
    {
        ASSERT(Process == NULL);

        if (!MiIsPdeForAddressValid(Address))
            return FALSE;
    }
    else
    {
        ASSERT(Process != NULL);
        ASSERT(Process == PsGetCurrentProcess());

        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if (!MiAddressToPxe(Address)->u.Hard.Valid ||
            !MiAddressToPpe(Address)->u.Hard.Valid ||
            !MiAddressToPde(Address)->u.Hard.Valid)
        {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return FALSE;
        }
    }

    PointerPte = MiAddressToPte(Address);

    /* Disabled = not valid, not a swap entry, but has a page frame number */
    Ret = !PointerPte->u.Hard.Valid
        && !FlagOn(PointerPte->u.Long, 0x800)
        && (PointerPte->u.Hard.PageFrameNumber != 0);

    if (Address < MmSystemRangeStart)
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());

    return Ret;
}

BOOLEAN
NTAPI
MmIsPageSwapEntry(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    PMMPTE PointerPte;
    BOOLEAN Ret;

    /* We never set swap entries for kernel addresses */
    if (Address >= MmSystemRangeStart)
    {
        ASSERT(Process == NULL);
        return FALSE;
    }

    ASSERT(Process != NULL);
    ASSERT(Process == PsGetCurrentProcess());

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if (!MiAddressToPxe(Address)->u.Hard.Valid ||
        !MiAddressToPpe(Address)->u.Hard.Valid ||
        !MiAddressToPde(Address)->u.Hard.Valid)
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return FALSE;
    }

    PointerPte = MiAddressToPte(Address);
    Ret = !PointerPte->u.Hard.Valid && FlagOn(PointerPte->u.Long, 0x800);

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    return Ret;
}

PFN_NUMBER
NTAPI
MmGetPfnForProcess(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    PMMPTE PointerPte;
    PFN_NUMBER Page;

    /* Must be called for user mode only */
    ASSERT(Process != NULL);
    ASSERT(Address < MmSystemRangeStart);
    ASSERT(Process == PsGetCurrentProcess());

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if (!MiAddressToPxe(Address)->u.Hard.Valid ||
        !MiAddressToPpe(Address)->u.Hard.Valid ||
        !MiAddressToPde(Address)->u.Hard.Valid)
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0;
    }

    PointerPte = MiAddressToPte(Address);
    Page = PointerPte->u.Hard.Valid ? PFN_FROM_PTE(PointerPte) : 0;

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    return Page;
}

ULONG
NTAPI
MmGetPageProtect(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    PMMPTE PointerPte;
    ULONG Protect;

    if (Address >= MmSystemRangeStart)
    {
        ASSERT(Process == NULL);

        if (!MiIsPdeForAddressValid(Address))
            return PAGE_NOACCESS;
    }
    else
    {
        ASSERT(Process != NULL);
        ASSERT(Process == PsGetCurrentProcess());

        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if (!MiAddressToPxe(Address)->u.Hard.Valid ||
            !MiAddressToPpe(Address)->u.Hard.Valid ||
            !MiAddressToPde(Address)->u.Hard.Valid)
        {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return PAGE_NOACCESS;
        }
    }

    PointerPte = MiAddressToPte(Address);

    if (!PointerPte->u.Hard.Valid)
    {
        Protect = PAGE_NOACCESS;
    }
    else
    {
        /* Decode ARM64 PTE bits to Windows protection constant.
         * Bit 56 = CopyOnWrite, Bit 55 = Writable, Bit 54 = UserNoExecute */
        if (PointerPte->u.Hard.CopyOnWrite)
            Protect = PAGE_WRITECOPY;
        else if (PointerPte->u.Hard.Writable)
            Protect = PAGE_READWRITE;
        else
            Protect = PAGE_READONLY;

        /* If executable (UXN=0), shift to EXECUTE variant */
        if (!PointerPte->u.Hard.UserNoExecute)
            Protect <<= 4;

        /* Cache attributes: CacheType = AttrIndx into MAIR=0x004400FF
         * (0=Normal-WB cached, 1=Device no-cache, 2=Normal-NC write-combine).
         * Was inverted (== 0); corrected to match the MAIR. */
        if (PointerPte->u.Hard.CacheType == 1)
            Protect |= PAGE_NOCACHE;
        else if (PointerPte->u.Hard.CacheType == 2)
            Protect |= PAGE_WRITECOMBINE;
    }

    if (Address < MmSystemRangeStart)
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());

    return Protect;
}

VOID
NTAPI
MmSetPageProtect(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect)
{
    ULONG ProtectionMask;
    PMMPTE PointerPte;
    MMPTE TempPte, OldPte;

    ASSERT(Process != NULL);
    ASSERT(Address < MmSystemRangeStart);
    ASSERT(Process == PsGetCurrentProcess());

    ProtectionMask = MiMakeProtectionMask(flProtect);
    ASSERT(ProtectionMask != MM_INVALID_PROTECTION);

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    PointerPte = MiAddressToPte(Address);

    TempPte.u.Long = 0;
    TempPte.u.Hard.PageFrameNumber = PointerPte->u.Hard.PageFrameNumber;
    TempPte.u.Long |= MmProtectToPteMask[ProtectionMask];
    TempPte.u.Hard.Owner = 1;

    /* Only set valid bit if we have to */
    if ((ProtectionMask != MM_NOACCESS) && !FlagOn(ProtectionMask, MM_GUARDPAGE))
        TempPte.u.Hard.Valid = 1;

    /* Keep accessed bit from old PTE.
     * NotDirty (AP[2]) comes from the protection mask — it controls
     * hardware read-only on ARM64 without FEAT_HAFDBS.  Preserving
     * it from the old PTE would defeat read-only enforcement. */
    TempPte.u.Hard.Accessed = PointerPte->u.Hard.Accessed;

    OldPte.u.Long = InterlockedExchangePte(PointerPte, TempPte.u.Long);

    if (OldPte.u.Long != TempPte.u.Long)
        KeInvalidateTlbEntry(Address);

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

VOID
NTAPI
MmSetDirtyBit(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ BOOLEAN Dirty)
{
    PMMPTE PointerPte;

    ASSERT(Process != NULL);
    ASSERT(Address < MmSystemRangeStart);
    ASSERT(Process == PsGetCurrentProcess());

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    PointerPte = MiAddressToPte(Address);

    /* ARM64: NotDirty is inverted — 0 means dirty */
    PointerPte->u.Hard.NotDirty = Dirty ? 0 : 1;

    if (!Dirty)
        KeInvalidateTlbEntry(Address);

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

/* Internal implementation for MmCreateVirtualMapping / MmCreatePhysicalMapping */
static
NTSTATUS
MmCreateVirtualMappingUnsafeEx(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page,
    _In_ BOOLEAN IsPhysical)
{
    ULONG ProtectionMask;
    PMMPTE PointerPte;
    MMPTE TempPte;
    ULONG_PTR Pte;

    PAGE_TRACE("[PAGE] CreateVMap addr=");
    PAGE_TRACE_HEX((ULONG64)Address);
    PAGE_TRACE(" pfn=");
    PAGE_TRACE_HEX((ULONG64)Page);
    PAGE_TRACE("\r\n");

    ASSERT(((ULONG_PTR)Address % PAGE_SIZE) == 0);

    ProtectionMask = MiMakeProtectionMask(flProtect);
    PAGE_TRACE("[PAGE] ProtMask=");
    PAGE_TRACE_HEX((ULONG64)ProtectionMask);
    PAGE_TRACE("\r\n");
    /* Caller must have checked */
    ASSERT(ProtectionMask != MM_INVALID_PROTECTION);
    ASSERT(ProtectionMask != MM_NOACCESS);
    ASSERT(ProtectionMask != MM_ZERO_ACCESS);

    if (Process == NULL)
    {
        /* Kernel-mode mapping */
        ASSERT(ProtectionMask != MM_WRITECOPY);
        ASSERT(ProtectionMask != MM_EXECUTE_WRITECOPY);

        if (Address < MmSystemRangeStart)
        {
            PagePuts("[PAGE] BUG: kernel addr < MmSystemRangeStart\r\n");
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        /* Ensure PDE hierarchy is valid for this kernel address.
         * The legacy MM allocator (MmCreateMemoryArea) can place VAs outside
         * the system PTE space, where PDEs may not exist yet. */
        if (!MiIsPdeForAddressValid(Address))
        {
            /* PDE on-demand trace — disabled */
            MiEnsureKernelPdeValid(Address);
        }
    }
    else
    {
        if ((Address >= MmSystemRangeStart) || Add2Ptr(Address, PAGE_SIZE) >= MmSystemRangeStart)
        {
            DPRINT1("Process %p given for kernel-mode mapping at %p\n", Process, Address);
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        ASSERT(Process == PsGetCurrentProcess());
        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        MiMakePdeExistAndMakeValid(MiAddressToPde(Address), Process, MM_NOIRQL);
    }

    PointerPte = MiAddressToPte(Address);

    /* Build the new PTE */
    MI_MAKE_HARDWARE_PTE(&TempPte, PointerPte, ProtectionMask, Page);

    /* Atomically set PTE */
    Pte = InterlockedExchangePte(PointerPte, TempPte.u.Long);

    /* There should not have been anything valid here */
    if (Pte != 0)
    {
        PagePuts("[PAGE] BUG: old PTE was non-zero!\r\n");
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    /* ARM64: ensure PTE store is visible to page table walker */
    __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");

    if (!IsPhysical)
    {
        PMMPFN Pfn1;
        KIRQL OldIrql;

        OldIrql = MiAcquirePfnLock();
        Pfn1 = &MmPfnDatabase[TempPte.u.Hard.PageFrameNumber];
        Pfn1->u2.ShareCount++;
        Pfn1->u3.e1.PageLocation = ActiveAndValid;
        MiReleasePfnLock(OldIrql);
    }

    /* No TLB flush needed: transitioning from invalid to valid can't be cached */

    if (Address < MmSystemRangeStart)
    {
        MiIncrementPageTableReferences(Address);
        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
MmCreateVirtualMappingUnsafe(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page)
{
    return MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, FALSE);
}

NTSTATUS
NTAPI
MmCreateVirtualMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page)
{
    ASSERT((ULONG_PTR)Address % PAGE_SIZE == 0);

    if (!MmIsPageInUse(Page))
    {
        DPRINT1("Page %lx is not in use\n", Page);
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    return MmCreateVirtualMappingUnsafe(Process, Address, flProtect, Page);
}

NTSTATUS
NTAPI
MmCreatePhysicalMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page)
{
    return MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, TRUE);
}

/* Internal implementation for MmDeleteVirtualMapping / MmDeletePhysicalMapping */
static
_Success_(return)
BOOLEAN
MmDeleteVirtualMappingEx(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_opt_ BOOLEAN *WasDirty,
    _Out_opt_ PPFN_NUMBER Page,
    _In_ BOOLEAN IsPhysical)
{
    PMMPTE PointerPte;
    MMPTE OldPte;
    BOOLEAN ValidPde;

    OldPte.u.Long = 0;

    ASSERT(((ULONG_PTR)Address % PAGE_SIZE) == 0);
    ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL);

    if (Process == NULL)
    {
        if (Address < MmSystemRangeStart)
        {
            DPRINT1("NULL process given for user-mode mapping at %p\n", Address);
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        ValidPde = MiIsPdeForAddressValid(Address);
    }
    else
    {
        if ((Address >= MmSystemRangeStart) || Add2Ptr(Address, PAGE_SIZE) >= MmSystemRangeStart)
        {
            DPRINT1("Process %p given for kernel-mode mapping at %p\n", Process, Address);
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        ASSERT(Process == PsGetCurrentProcess());
        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        ValidPde = MiAddressToPxe(Address)->u.Hard.Valid &&
                   MiAddressToPpe(Address)->u.Hard.Valid &&
                   MiAddressToPde(Address)->u.Hard.Valid;

        if (ValidPde)
            MiMakePdeExistAndMakeValid(MiAddressToPde(Address), Process, MM_NOIRQL);
    }

    if (ValidPde)
    {
        PointerPte = MiAddressToPte(Address);
        OldPte.u.Long = InterlockedExchangePte(PointerPte, 0);

        KeInvalidateTlbEntry(Address);

        if (OldPte.u.Long != 0)
        {
            if (WasDirty != NULL)
            {
                /* ARM64: NotDirty is inverted — 0 means dirty */
                *WasDirty = OldPte.u.Hard.Valid ? !OldPte.u.Hard.NotDirty : FALSE;
            }
            if (Page != NULL)
            {
                *Page = OldPte.u.Hard.PageFrameNumber;
            }
        }
    }

    if (Process != NULL)
    {
        if (OldPte.u.Long != 0)
        {
            if (MiDecrementPageTableReferences(Address) == 0)
            {
                KIRQL OldIrql = MiAcquirePfnLock();
                MiDeletePde(MiAddressToPde(Address), Process);
                MiReleasePfnLock(OldIrql);
            }
        }

        if (!IsPhysical && OldPte.u.Hard.Valid)
        {
            PMMPFN Pfn1;
            KIRQL OldIrql;

            OldIrql = MiAcquirePfnLock();
            Pfn1 = &MmPfnDatabase[OldPte.u.Hard.PageFrameNumber];
            ASSERT(Pfn1->u3.e1.PageLocation == ActiveAndValid);
            ASSERT(Pfn1->u2.ShareCount > 0);
            if (--Pfn1->u2.ShareCount == 0)
            {
                Pfn1->u3.e1.PageLocation = TransitionPage;
            }
            MiReleasePfnLock(OldIrql);
        }

        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    if (WasDirty != NULL && OldPte.u.Long == 0)
        *WasDirty = FALSE;
    if (Page != NULL && OldPte.u.Long == 0)
        *Page = 0;

    return OldPte.u.Long != 0;
}

_Success_(return)
BOOLEAN
MmDeleteVirtualMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_opt_ BOOLEAN *WasDirty,
    _Out_opt_ PPFN_NUMBER Page)
{
    return MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, FALSE);
}

_Success_(return)
BOOLEAN
MmDeletePhysicalMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_opt_ BOOLEAN *WasDirty,
    _Out_opt_ PPFN_NUMBER Page)
{
    return MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, TRUE);
}

VOID
NTAPI
MmGetPageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_ SWAPENTRY *SwapEntry)
{
    PMMPTE PointerPte;

    /* No swap entries for kernel addresses */
    if (Address >= MmSystemRangeStart)
    {
        ASSERT(Process == NULL);
        *SwapEntry = 0;
        return;
    }

    ASSERT(Process != NULL);
    ASSERT(Process == PsGetCurrentProcess());

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if (!MiAddressToPxe(Address)->u.Hard.Valid ||
        !MiAddressToPpe(Address)->u.Hard.Valid ||
        !MiAddressToPde(Address)->u.Hard.Valid)
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        *SwapEntry = 0;
        return;
    }

    PointerPte = MiAddressToPte(Address);
    if (!PointerPte->u.Hard.Valid && FlagOn(PointerPte->u.Long, 0x800))
        *SwapEntry = PointerPte->u.Long >> 1;
    else
        *SwapEntry = 0;

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
}

NTSTATUS
NTAPI
MmCreatePageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ SWAPENTRY SwapEntry)
{
    PMMPTE PointerPte;
    ULONG_PTR Pte;

    ASSERT(Process != NULL);
    ASSERT(Address < MmSystemRangeStart);
    ASSERT(Process == PsGetCurrentProcess());

    if (SwapEntry & ((ULONG_PTR)1 << (RTL_BITS_OF(SWAPENTRY) - 1)))
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(MiAddressToPde(Address), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    Pte = InterlockedExchangePte(PointerPte, SwapEntry << 1);
    if (Pte != 0)
    {
        KeBugCheckEx(MEMORY_MANAGEMENT, SwapEntry, (ULONG_PTR)Process, (ULONG_PTR)Address, 0);
    }

    MiIncrementPageTableReferences(Address);
    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    return STATUS_SUCCESS;
}

VOID
NTAPI
MmDeletePageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_ SWAPENTRY *SwapEntry)
{
    PMMPTE PointerPte;
    MMPTE OldPte;

    ASSERT(Process != NULL);
    ASSERT(Address < MmSystemRangeStart);
    ASSERT(Process == PsGetCurrentProcess());
    ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL);

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    ASSERT(MiAddressToPde(Address)->u.Long != 0);
    MiMakePdeExistAndMakeValid(MiAddressToPde(Address), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    OldPte.u.Long = InterlockedExchangePte(PointerPte, 0);

    if (!FlagOn(OldPte.u.Long, 0x800) || OldPte.u.Hard.Valid)
    {
        KeBugCheckEx(MEMORY_MANAGEMENT, OldPte.u.Long, (ULONG_PTR)Process, (ULONG_PTR)Address, 0);
    }

    if (MiDecrementPageTableReferences(Address) == 0)
    {
        KIRQL OldIrql = MiAcquirePfnLock();
        MiDeletePde(MiPteToPde(PointerPte), Process);
        MiReleasePfnLock(OldIrql);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    *SwapEntry = OldPte.u.Long >> 1;
}

BOOLEAN
MiArchCreateProcessAddressSpace(
    _In_ PEPROCESS Process,
    _In_ PULONG_PTR DirectoryTableBase)
{
    KIRQL OldIrql;
    PFN_NUMBER TableBasePfn, HyperPfn, HyperPdPfn, HyperPtPfn;
    PMMPTE SystemPte;
    MMPTE TempPte, PdePte;
    ULONG TableIndex;
    PMMPTE PageTablePointer;
    ULONG PageColor;

    PagePuts("[MM] MiArchCreateProcessAddressSpace\r\n");

    /* Non-arch specific code-path allocated those for us */
    TableBasePfn = DirectoryTableBase[0] >> PAGE_SHIFT;
    HyperPfn = DirectoryTableBase[1] >> PAGE_SHIFT;

    /*
     * Allocate two extra pages for the hyperspace PD and PT.
     * Try zero pages first; fall back to allocate + zero.
     */
    OldIrql = MiAcquirePfnLock();
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process);
    HyperPdPfn = MiRemoveZeroPageSafe(PageColor);
    if (!HyperPdPfn)
    {
        HyperPdPfn = MiRemoveAnyPage(PageColor);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(HyperPdPfn);
        OldIrql = MiAcquirePfnLock();
    }
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process);
    HyperPtPfn = MiRemoveZeroPageSafe(PageColor);
    if (!HyperPtPfn)
    {
        HyperPtPfn = MiRemoveAnyPage(PageColor);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(HyperPtPfn);
    }
    else
    {
        MiReleasePfnLock(OldIrql);
    }

    /* Get a system PTE to temporarily map the new pages */
    SystemPte = MiReserveSystemPtes(1, SystemPteSpace);
    if (!SystemPte)
        return FALSE;

    /* Get the virtual address mapped by the system PTE */
    PageTablePointer = MiPteToAddress(SystemPte);

    /* Build a kernel PTE for the PML4 and map it */
    MI_MAKE_HARDWARE_PTE_KERNEL(&PdePte, SystemPte, MM_READWRITE, TableBasePfn);
    MI_WRITE_VALID_PTE(SystemPte, PdePte);

    /* Zero user-space entries (lower half), copy kernel entries (upper half) */
    TableIndex = PXE_PER_PAGE / 2;
    RtlZeroMemory(PageTablePointer, TableIndex * sizeof(MMPTE));
    RtlCopyMemory(PageTablePointer + TableIndex,
                   MiAddressToPxe(0) + TableIndex,
                   PAGE_SIZE - TableIndex * sizeof(MMPTE));

    /* Sanity check: hyperspace end must be in the kernel half */
    ASSERT(MiAddressToPxeOffset(MmHyperSpaceEnd) >= TableIndex);

    /* Setup a template PTE for page directory mappings */
    TempPte = ValidKernelPte;

    /* Self-map: PML4[0x1ED] points back to the PML4 itself */
    TableIndex = MiAddressToPxeOffset((PVOID)PXE_SELFMAP);
    TempPte.u.Hard.PageFrameNumber = TableBasePfn;
    PageTablePointer[TableIndex] = TempPte;

    /* Hyperspace: PML4[0x1EE] points to the hyperspace PPE page */
    TableIndex = MiAddressToPxeOffset((PVOID)HYPER_SPACE);
    TempPte.u.Hard.PageFrameNumber = HyperPfn;
    PageTablePointer[TableIndex] = TempPte;

    /* Remap system PTE to the hyperspace PPE page */
    PdePte.u.Hard.PageFrameNumber = HyperPfn;
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer);

    /* PPE[0] → hyperspace PD page */
    TempPte.u.Hard.PageFrameNumber = HyperPdPfn;
    PageTablePointer[0] = TempPte;

    /* Remap system PTE to the hyperspace PD page */
    PdePte.u.Hard.PageFrameNumber = HyperPdPfn;
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer);

    /* PDE[0] → hyperspace PT page */
    TempPte.u.Hard.PageFrameNumber = HyperPtPfn;
    PageTablePointer[0] = TempPte;

    /* Remap system PTE to the hyperspace PT page */
    PdePte.u.Hard.PageFrameNumber = HyperPtPfn;
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer);

    /* Write the PTE for the working set list page */
    TempPte.u.Hard.PageFrameNumber = Process->WorkingSetPage;
    TableIndex = MiAddressToPteOffset(MmWorkingSetList);
    PageTablePointer[TableIndex] = TempPte;

    /* Release the system PTE */
    MiReleaseSystemPtes(SystemPte, 1, SystemPteSpace);

    PagePuts("[MM] MiArchCreateProcessAddressSpace done\r\n");

    return TRUE;
}

VOID
NTAPI
MmInitGlobalKernelPageDirectory(VOID)
{
    /* ARM64 uses a single kernel page table (TTBR1) — nothing to synchronize */
}

/* EOF */
