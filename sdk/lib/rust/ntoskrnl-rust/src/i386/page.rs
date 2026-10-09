use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
// RtlZeroMemory is only used in the non-x86 (64-bit) code path.
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::rtl::RtlZeroMemory;

use ntkrnl_sys::ddk::{
    _MMPTE as DDK_MMPTE,
    PEPROCESS,
    PPFN_NUMBER,
    SWAPENTRY,
    KeBugCheck,
    KeBugCheckEx,
    KeInvalidateTlbEntry,
    MiDecrementPageTableReferences,
    MiDeletePde,
    MiIncrementPageTableReferences,
    MiLockProcessWorkingSetShared,
    MiLockProcessWorkingSetUnsafe,
    MiMakePdeExistAndMakeValid,
    MiMakeProtectionMask,
    MiUnlockProcessWorkingSetShared,
    MiUnlockProcessWorkingSetUnsafe,
    MmIsPageInUse,
    PsGetCurrentThread,
};

// PAGE_WRITETHROUGH: defined in ntoskrnl/include/internal/mm.h as 1024
const PAGE_WRITETHROUGH: ULONG = 1024;

// MiIsPdeForAddressValid is only available on 64-bit (amd64).
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::MiIsPdeForAddressValid;

// MiMakeSystemAddressValid and PsGetCurrentProcess are only used in the 64-bit non-x86 code path.
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::MiMakeSystemAddressValid;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::ARM3::helpers::*;

// ============================================================================
// MmProtectToPteMask — protection-to-PTE-flags translation table.
// Constants come from helpers (which reads from miarm.h values).
// ============================================================================

#[no_mangle]
pub static MmProtectToPteMask: [ULONG_PTR; 32] = [
    0,
    PTE_READONLY | PTE_ENABLE_CACHE,
    PTE_EXECUTE | PTE_ENABLE_CACHE,
    PTE_EXECUTE_READ | PTE_ENABLE_CACHE,
    PTE_READWRITE | PTE_ENABLE_CACHE,
    PTE_WRITECOPY | PTE_ENABLE_CACHE,
    PTE_EXECUTE_READWRITE | PTE_ENABLE_CACHE,
    PTE_EXECUTE_WRITECOPY | PTE_ENABLE_CACHE,
    0,
    PTE_READONLY | PTE_DISABLE_CACHE,
    PTE_EXECUTE | PTE_DISABLE_CACHE,
    PTE_EXECUTE_READ | PTE_DISABLE_CACHE,
    PTE_READWRITE | PTE_DISABLE_CACHE,
    PTE_WRITECOPY | PTE_DISABLE_CACHE,
    PTE_EXECUTE_READWRITE | PTE_DISABLE_CACHE,
    PTE_EXECUTE_WRITECOPY | PTE_DISABLE_CACHE,
    0,
    PTE_READONLY | PTE_ENABLE_CACHE,
    PTE_EXECUTE | PTE_ENABLE_CACHE,
    PTE_EXECUTE_READ | PTE_ENABLE_CACHE,
    PTE_READWRITE | PTE_ENABLE_CACHE,
    PTE_WRITECOPY | PTE_ENABLE_CACHE,
    PTE_EXECUTE_READWRITE | PTE_ENABLE_CACHE,
    PTE_EXECUTE_WRITECOPY | PTE_ENABLE_CACHE,
    0,
    PTE_READONLY | PTE_WRITECOMBINED_CACHE,
    PTE_EXECUTE | PTE_WRITECOMBINED_CACHE,
    PTE_EXECUTE_READ | PTE_WRITECOMBINED_CACHE,
    PTE_READWRITE | PTE_WRITECOMBINED_CACHE,
    PTE_WRITECOPY | PTE_WRITECOMBINED_CACHE,
    PTE_EXECUTE_READWRITE | PTE_WRITECOMBINED_CACHE,
    PTE_EXECUTE_WRITECOPY | PTE_WRITECOMBINED_CACHE,
];

#[no_mangle]
pub static MmProtectToValue: [ULONG; 32] = [
    PAGE_NOACCESS,
    PAGE_READONLY,
    PAGE_EXECUTE,
    PAGE_EXECUTE_READ,
    PAGE_READWRITE,
    PAGE_WRITECOPY,
    PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_NOCACHE | PAGE_READONLY,
    PAGE_NOCACHE | PAGE_EXECUTE,
    PAGE_NOCACHE | PAGE_EXECUTE_READ,
    PAGE_NOCACHE | PAGE_READWRITE,
    PAGE_NOCACHE | PAGE_WRITECOPY,
    PAGE_NOCACHE | PAGE_EXECUTE_READWRITE,
    PAGE_NOCACHE | PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_GUARD | PAGE_READONLY,
    PAGE_GUARD | PAGE_EXECUTE,
    PAGE_GUARD | PAGE_EXECUTE_READ,
    PAGE_GUARD | PAGE_READWRITE,
    PAGE_GUARD | PAGE_WRITECOPY,
    PAGE_GUARD | PAGE_EXECUTE_READWRITE,
    PAGE_GUARD | PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_WRITECOMBINE | PAGE_READONLY,
    PAGE_WRITECOMBINE | PAGE_EXECUTE,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_READ,
    PAGE_WRITECOMBINE | PAGE_READWRITE,
    PAGE_WRITECOMBINE | PAGE_WRITECOPY,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_READWRITE,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_WRITECOPY,
];

extern "system" {
    fn MiFillSystemPageDirectory(Base: PVOID, NumberOfBytes: SIZE_T);
}

// Helper: cast hand-written MMPTE pointer to DDK _MMPTE pointer for DDK function calls.
// The two types have identical layout (single ULONG_PTR field); the cast is safe.
#[inline(always)]
unsafe fn as_ddk_pde(p: *mut MMPTE) -> *mut DDK_MMPTE {
    p as *mut DDK_MMPTE
}

// MiEnsureKernelPdeValid is only used on non-x86 (64-bit paging has PPE/PXE levels).
#[cfg(not(target_arch = "x86"))]
unsafe fn MiEnsureKernelPdeValid(Address: PVOID) {
    #[cfg(target_pointer_width = "64")]
    let PointerPxe = MiAddressToPxe(Address);
    let PointerPpe = MiAddressToPpe(Address);
    let PointerPde = MiAddressToPde(Address);
    let mut PageFrameIndex: PFN_NUMBER;
    let mut TempPte = MMPTE { Long: 0 };
    let mut OldIrql: KIRQL;

    #[cfg(target_pointer_width = "64")]
    {
        if !(*PointerPxe).valid() {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
    }

    if !(*PointerPpe).valid() {
        OldIrql = MiAcquirePfnLock();
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiReleasePfnLock(OldIrql);

        TempPte.Long = ValidKernelPde.Long;
        TempPte.set_page_frame_number(PageFrameIndex);
        MI_WRITE_VALID_PTE(PointerPpe, TempPte);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        RtlZeroMemory(MiPteToAddress(PointerPpe), PAGE_SIZE);
    }

    if !(*PointerPde).valid() {
        OldIrql = MiAcquirePfnLock();
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiReleasePfnLock(OldIrql);

        TempPte.Long = ValidKernelPde.Long;
        TempPte.set_page_frame_number(PageFrameIndex);
        MI_WRITE_VALID_PTE(PointerPde, TempPte);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        RtlZeroMemory(MiPteToAddress(PointerPde), PAGE_SIZE);
    }
}

unsafe fn MiIsPageTablePresent(Address: PVOID) -> BOOLEAN {
    #[cfg(target_arch = "x86")]
    {
        // On i386: use MmWorkingSetList->UsedPageTableEntries[pde_index]
        // MiGetPdeOffset gives the PDE index for this address.
        // MmWorkingSetList is PVOID; access UsedPageTableEntries[] at byte offset 56.
        const MMWSL_USED_PTE_OFFSET: usize = 56;
        let pde_offset = MiGetPdeOffset(Address);
        let wsl_ptr = MmWorkingSetList as *mut u8;
        let entries_ptr = wsl_ptr.add(MMWSL_USED_PTE_OFFSET) as *mut USHORT;
        let Ret = *entries_ptr.add(pde_offset) != 0;
        Ret as BOOLEAN
    }

    #[cfg(not(target_arch = "x86"))]
    {
        // On 64-bit: PMMPPE and PMMPXE are the same as PMMPTE (just type aliases).
        // MiAddressToPpe/MiAddressToPxe return *mut MMPTE (hand-written type).
        let PointerPde: PMMPDE;
        let PointerPpe: PMMPTE;
        #[cfg(target_pointer_width = "64")]
        let PointerPxe: PMMPTE;
        let mut Pfn: PMMPFN;

        #[cfg(target_pointer_width = "64")]
        {
            PointerPxe = MiAddressToPxe(Address);
            if (*PointerPxe).valid() || (*PointerPxe).transition() {
                Pfn = MiGetPfnEntry(PFN_FROM_PXE(PointerPxe));
                if mmpfn_used_page_table_entries(Pfn) == 0 {
                    return 0u8;
                }
            } else if mmpfn_used_page_table_entries_from_pte(&*PointerPxe) == 0 {
                return 0u8;
            }

            if !(*PointerPxe).valid() {
                MiMakeSystemAddressValid(MiPteToAddress(PointerPxe), PsGetCurrentProcess() as _);
            }
        }

        PointerPpe = MiAddressToPpe(Address);
        if (*PointerPpe).valid() || (*PointerPpe).transition() {
            Pfn = MiGetPfnEntry(PFN_FROM_PPE(PointerPpe));
            if mmpfn_used_page_table_entries(Pfn) == 0 {
                return 0u8;
            }
        } else if mmpfn_used_page_table_entries_from_pte(&*PointerPpe) == 0 {
            return 0u8;
        }

        if !(*PointerPpe).valid() {
            MiMakeSystemAddressValid(MiPteToAddress(PointerPpe), PsGetCurrentProcess() as _);
        }

        PointerPde = MiAddressToPde(Address);
        if !(*PointerPde).valid() && !(*PointerPde).transition() {
            return (mmpfn_used_page_table_entries_from_pte(&*PointerPde) != 0) as BOOLEAN;
        }

        // PFN_FROM_PDE is the same as PFN_FROM_PTE (PDE is just a PTE pointer alias).
        Pfn = MiGetPfnEntry(PFN_FROM_PTE(PointerPde));
        (mmpfn_used_page_table_entries(Pfn) != 0) as BOOLEAN
    }
}

// On 64-bit, MiIsPageTablePresent uses UsedPageTableEntries stored in soft PTE
// bits for invalid PTEs, not in the PFN database.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn mmpfn_used_page_table_entries_from_pte(pte: &MMPTE) -> ULONG {
    pte.soft_used_page_table_entries()
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPfnForProcess(
    Process: PEPROCESS,
    Address: PVOID,
) -> PFN_NUMBER {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Page: PFN_NUMBER;

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if MiIsPageTablePresent(Address) == 0 {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0;
    }

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    Page = if (*PointerPte).valid() {
        PFN_FROM_PTE(PointerPte)
    } else {
        0
    };

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    Page
}

unsafe fn MmDeleteVirtualMappingEx(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
    IsPhysical: BOOLEAN,
) -> BOOLEAN {
    let PointerPte: PMMPTE;
    let mut OldPte = MMPTE { Long: 0 };
    let ValidPde: BOOLEAN;

    if Process.is_null() {
        if Address < MmSystemRangeStart {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        #[cfg(target_arch = "x86")]
        {
            ValidPde = MiSynchronizeSystemPde_Bool(MiAddressToPde(Address));
        }
        #[cfg(not(target_arch = "x86"))]
        {
            ValidPde = MiIsPdeForAddressValid(Address) as BOOLEAN;
        }
    } else {
        if Address >= MmSystemRangeStart
            || (Address as usize).wrapping_add(PAGE_SIZE) >= MmSystemRangeStart as usize
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        ValidPde = MiIsPageTablePresent(Address);
        if ValidPde != 0 {
            MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
        }
    }

    if ValidPde != 0 {
        PointerPte = MiAddressToPte(Address);
        OldPte.Long = InterlockedExchangePte(PointerPte, 0);

        KeInvalidateTlbEntry(Address);

        if OldPte.Long != 0 {
            if !WasDirty.is_null() {
                // Dirty bit is bit 6 on x86/x86_64 -- of the OLD value: the exchange above
                // has already zeroed *PointerPte. Reading it there reported every page clean,
                // so nothing written through a file mapping ever reached the file (setup's
                // whole cab extraction came out zero-filled).
                *WasDirty = ((OldPte.Long >> 6) & 1) as BOOLEAN;
            }
            if !Page.is_null() {
                *Page = OldPte.page_frame_number() as _;
            }
        }
    }

    if !Process.is_null() {
        if OldPte.Long != 0 {
            if MiDecrementPageTableReferences(Address) == 0 {
                let OldIrql = MiAcquirePfnLock();
                MiDeletePde(as_ddk_pde(MiAddressToPde(Address)), Process);
                MiReleasePfnLock(OldIrql);
            }
        }

        if IsPhysical == 0 && OldPte.valid() {
            let Pfn1: PMMPFN;
            let OldIrql: KIRQL;

            OldIrql = MiAcquirePfnLock();
            Pfn1 = MiGetPfnEntry(OldPte.page_frame_number());
            if !Pfn1.is_null() {
                let sc = (*Pfn1).share_count();
                (*Pfn1).set_share_count(sc.wrapping_sub(1));
                if (*Pfn1).share_count() == 0 {
                    (*Pfn1).set_page_location(TransitionPage);
                }
            }
            MiReleasePfnLock(OldIrql);
        }

        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    (OldPte.Long != 0) as BOOLEAN
}

// NTAPI/stdcall (@16) — the no-NTAPI decl in internal/mm.h is misleading. The callers reach
// this through `VirtFn = extern "system" fn(..)` (marea.rs); cdecl here desyncs ESP (§9.99).
#[no_mangle]
pub unsafe extern "system" fn MmDeleteVirtualMapping(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let WasDirty = opaque_ptr(WasDirty);
    MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmDeletePhysicalMapping(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let WasDirty = opaque_ptr(WasDirty);
    MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmDeletePageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: *mut SWAPENTRY,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let SwapEntry = opaque_ptr(SwapEntry);
    let PointerPte: PMMPTE;
    let mut OldPte = MMPTE { Long: 0 };

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    OldPte.Long = InterlockedExchangePte(PointerPte, 0);
    if (OldPte.Long & 0x800) == 0 || OldPte.valid() {
        KeBugCheckEx(
            MEMORY_MANAGEMENT,
            OldPte.Long as _,
            Process as _,
            Address as _,
            0,
        );
    }

    if MiDecrementPageTableReferences(Address) == 0 {
        // [guard] let OldIrql = MiAcquirePfnLock();
        let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        MiDeletePde(as_ddk_pde(MiPteToPde(PointerPte)), Process);
        drop(pfn_guard);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    *SwapEntry = (OldPte.Long >> 1) as _;
}

#[no_mangle]
pub unsafe extern "system" fn MmIsPagePresent(Process: PEPROCESS, Address: PVOID) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let Ret: BOOLEAN;

    if Address >= MmSystemRangeStart {
        #[cfg(target_arch = "x86")]
        {
            if MiSynchronizeSystemPde_Bool(MiAddressToPde(Address)) == 0 {
                return 0u8;
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            if MiIsPdeForAddressValid(Address) == 0 {
                return 0u8;
            }
        }

        return (*MiAddressToPte(Address)).valid() as BOOLEAN;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if MiIsPageTablePresent(Address) == 0 {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0u8;
    }

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    Ret = (*MiAddressToPte(Address)).valid() as BOOLEAN;

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmIsDisabledPage(Process: PEPROCESS, Address: PVOID) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let Ret: BOOLEAN;
    let PointerPte: PMMPTE;

    if Address >= MmSystemRangeStart {
        #[cfg(target_arch = "x86")]
        {
            if MiSynchronizeSystemPde_Bool(MiAddressToPde(Address)) == 0 {
                return 0u8;
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            if MiIsPdeForAddressValid(Address) == 0 {
                return 0u8;
            }
        }
    } else {
        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if MiIsPageTablePresent(Address) == 0 {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return 0u8;
        }

        MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
    }

    PointerPte = MiAddressToPte(Address);
    // A "disabled" page is one that is: not valid, not a swap entry (bit 11 = 0x800 not set),
    // but has a non-zero page frame number.
    Ret = (!(*PointerPte).valid()
        && ((*PointerPte).Long & 0x800) == 0
        && (*PointerPte).page_frame_number() != 0) as BOOLEAN;

    if Address < MmSystemRangeStart {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmIsPageSwapEntry(Process: PEPROCESS, Address: PVOID) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let Ret: BOOLEAN;
    let PointerPte: PMMPTE;

    if Address >= MmSystemRangeStart {
        return 0u8;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if MiIsPageTablePresent(Address) == 0 {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0u8;
    }

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    Ret = (!(*PointerPte).valid() && ((*PointerPte).Long & 0x800) != 0) as BOOLEAN;

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: *mut SWAPENTRY,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let SwapEntry = opaque_ptr(SwapEntry);
    let PointerPte: PMMPTE;

    if Address >= MmSystemRangeStart {
        *SwapEntry = 0;
        return;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if MiIsPageTablePresent(Address) == 0 {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        *SwapEntry = 0;
        return;
    }

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    if !(*PointerPte).valid() && ((*PointerPte).Long & 0x800) != 0 {
        *SwapEntry = ((*PointerPte).Long >> 1) as _;
    } else {
        *SwapEntry = 0;
    }

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: SWAPENTRY,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Pte: ULONG_PTR;

    if (SwapEntry & (1 << (core::mem::size_of::<SWAPENTRY>() * 8 - 1))) != 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    Pte = InterlockedExchangePte(PointerPte, (SwapEntry as ULONG_PTR) << 1);
    if Pte != 0 {
        KeBugCheckEx(
            MEMORY_MANAGEMENT,
            SwapEntry as _,
            Process as _,
            Address as _,
            0,
        );
    }

    MiIncrementPageTableReferences(Address);
    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    STATUS_SUCCESS
}

unsafe fn MmCreateVirtualMappingUnsafeEx(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
    IsPhysical: BOOLEAN,
) -> NTSTATUS {
    let ProtectionMask: ULONG;
    let PointerPte: PMMPTE;
    let mut TempPte = MMPTE { Long: 0 };
    let Pte: ULONG_PTR;

    ProtectionMask = MiMakeProtectionMask(flProtect);

    if Process.is_null() {
        if Address < MmSystemRangeStart {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        #[cfg(target_arch = "x86")]
        {
            if MiSynchronizeSystemPde_Bool(MiAddressToPde(Address)) == 0 {
                MiFillSystemPageDirectory(Address, PAGE_SIZE);
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            if MiIsPdeForAddressValid(Address) == 0 {
                MiEnsureKernelPdeValid(Address);
            }
        }
    } else {
        if Address >= MmSystemRangeStart
            || (Address as usize).wrapping_add(PAGE_SIZE) >= MmSystemRangeStart as usize
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
    }

    PointerPte = MiAddressToPte(Address);

    MI_MAKE_HARDWARE_PTE(&mut TempPte, PointerPte, ProtectionMask as ULONG_PTR, Page);

    Pte = InterlockedExchangePte(PointerPte, TempPte.Long);
    if Pte != 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if IsPhysical == 0 {
        let Pfn1: PMMPFN;
        let OldIrql: KIRQL;

        OldIrql = MiAcquirePfnLock();
        Pfn1 = MmPfnDatabase.add(TempPte.page_frame_number());
        let sc = (*Pfn1).share_count();
        (*Pfn1).set_share_count(sc.wrapping_add(1));
        (*Pfn1).set_page_location(ActiveAndValid);
        MiReleasePfnLock(OldIrql);
    }

    if Address < MmSystemRangeStart {
        MiIncrementPageTableReferences(Address);
        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateVirtualMappingUnsafe(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePhysicalMapping(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateVirtualMapping(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    if MmIsPageInUse(Page as _) == 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    MmCreateVirtualMappingUnsafe(Process, Address, flProtect, Page)
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPageProtect(Process: PEPROCESS, Address: PVOID) -> ULONG {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let mut Protect: ULONG;

    if Address >= MmSystemRangeStart {
        #[cfg(target_arch = "x86")]
        {
            if MiSynchronizeSystemPde_Bool(MiAddressToPde(Address)) == 0 {
                return PAGE_NOACCESS;
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            if MiIsPdeForAddressValid(Address) == 0 {
                return PAGE_NOACCESS;
            }
        }
    } else {
        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if MiIsPageTablePresent(Address) == 0 {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return PAGE_NOACCESS;
        }

        MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
    }

    PointerPte = MiAddressToPte(Address);

    // Extract protection from PTE bits.
    // Bit 0: Valid (Present)
    // Bit 1: Write
    // Bit 2: Owner (user/supervisor)
    // Bit 3: WriteThrough
    // Bit 4: CacheDisable
    // Bit 5: Accessed
    // Bit 6: Dirty
    // Bit 9: CopyOnWrite (software bit on x86)
    // Bit 63 (x86_64): NoExecute
    let pte_long = (*PointerPte).Long;
    if (pte_long & 1) == 0 {
        // Not valid/present
        Protect = PAGE_NOACCESS;
    } else {
        let is_cow = (pte_long & (1 << 9)) != 0;
        let is_write = (pte_long & (1 << 1)) != 0;
        if is_cow {
            Protect = PAGE_WRITECOPY;
        } else if is_write {
            Protect = PAGE_READWRITE;
        } else {
            Protect = PAGE_READONLY;
        }
        // On x86_64, check NX bit (bit 63) to determine execute permission.
        #[cfg(target_arch = "x86_64")]
        {
            let no_execute = (pte_long >> 63) & 1;
            if no_execute == 0 {
                Protect <<= 4; // PAGE_READONLY -> PAGE_EXECUTE_READ, etc.
            }
        }
        // On x86, all pages are executable (no NX in 32-bit mode without PAE/PSE36).
        #[cfg(target_arch = "x86")]
        {
            // No NoExecute bit on plain x86 (without PAE). Leave Protect as-is.
        }
        let cache_disable = (pte_long & (1 << 4)) != 0;
        let write_through = (pte_long & (1 << 3)) != 0;
        if cache_disable {
            Protect |= PAGE_NOCACHE;
        }
        if write_through {
            Protect |= PAGE_WRITETHROUGH;
        }
    }

    if Address < MmSystemRangeStart {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    }

    Protect
}

#[no_mangle]
pub unsafe extern "system" fn MmSetPageProtect(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let ProtectionMask: ULONG;
    let PointerPte: PMMPTE;
    let mut TempPte = MMPTE { Long: 0 };
    let mut OldPte = MMPTE { Long: 0 };

    ProtectionMask = MiMakeProtectionMask(flProtect);

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);

    // Build new PTE: keep PageFrameNumber, apply new protection flags.
    let pfn = (*PointerPte).page_frame_number();
    TempPte.Long = 0;
    TempPte.set_page_frame_number(pfn);
    TempPte.Long |= MmProtectToPteMask[ProtectionMask as usize];
    // Set Owner bit (bit 2 = user page)
    TempPte.Long |= 1 << 2;

    if ProtectionMask != MM_NOACCESS && (ProtectionMask & MM_GUARDPAGE) == 0 {
        // Set Valid bit
        TempPte.Long |= 1;
    }

    // Copy Accessed (bit 5) and Dirty (bit 6) from old PTE.
    let old_long = (*PointerPte).Long;
    let accessed = old_long & (1 << 5);
    let dirty = old_long & (1 << 6);
    TempPte.Long |= accessed | dirty;

    OldPte.Long = InterlockedExchangePte(PointerPte, TempPte.Long);

    if !OldPte.valid()
        && ((OldPte.Long & 0x800) != 0 || OldPte.page_frame_number() == 0)
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if OldPte.Long != TempPte.Long {
        KeInvalidateTlbEntry(Address);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

#[no_mangle]
pub unsafe extern "system" fn MmSetDirtyBit(Process: PEPROCESS, Address: PVOID, Bit: BOOLEAN) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    if !(*PointerPte).valid()
        && (((*PointerPte).Long & 0x800) != 0 || (*PointerPte).page_frame_number() == 0)
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    // Set or clear Dirty bit (bit 6 on x86/x86_64).
    if Bit != 0 {
        (*PointerPte).Long |= 1 << 6;
    } else {
        (*PointerPte).Long &= !(1 << 6);
    }

    if Bit == 0 {
        KeInvalidateTlbEntry(Address);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitGlobalKernelPageDirectory() {}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn Mmi386MakeKernelPageTableGlobal(Address: PVOID) -> BOOLEAN {
    let Address = opaque_ptr(Address);
    let PointerPde = MiAddressToPde(Address);
    let PointerPte = MiAddressToPte(Address);

    if !(*PointerPde).valid() {
        if MiSynchronizeSystemPde_Bool(PointerPde) == 0 {
            return 0u8;
        }
        return (*PointerPte).valid() as BOOLEAN;
    }
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/i386/page.c
//  * PURPOSE:         Low level memory management manipulation
//  *
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #include <mm/ARM3/miarm.h>
//
// #ifndef _MI_PAGING_LEVELS
// #error "Dude, fix your stuff before using this file"
// #endif
//
// /* GLOBALS *****************************************************************/
// const
// ULONG_PTR
// MmProtectToPteMask[32] =
// {
//     //
//     // These are the base MM_ protection flags
//     //
//     0,
//     PTE_READONLY            | PTE_ENABLE_CACHE,
//     PTE_EXECUTE             | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_READ        | PTE_ENABLE_CACHE,
//     PTE_READWRITE           | PTE_ENABLE_CACHE,
//     PTE_WRITECOPY           | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_READWRITE   | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_WRITECOPY   | PTE_ENABLE_CACHE,
//     //
//     // These OR in the MM_NOCACHE flag
//     //
//     0,
//     PTE_READONLY            | PTE_DISABLE_CACHE,
//     PTE_EXECUTE             | PTE_DISABLE_CACHE,
//     PTE_EXECUTE_READ        | PTE_DISABLE_CACHE,
//     PTE_READWRITE           | PTE_DISABLE_CACHE,
//     PTE_WRITECOPY           | PTE_DISABLE_CACHE,
//     PTE_EXECUTE_READWRITE   | PTE_DISABLE_CACHE,
//     PTE_EXECUTE_WRITECOPY   | PTE_DISABLE_CACHE,
//     //
//     // These OR in the MM_DECOMMIT flag, which doesn't seem supported on x86/64/ARM
//     //
//     0,
//     PTE_READONLY            | PTE_ENABLE_CACHE,
//     PTE_EXECUTE             | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_READ        | PTE_ENABLE_CACHE,
//     PTE_READWRITE           | PTE_ENABLE_CACHE,
//     PTE_WRITECOPY           | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_READWRITE   | PTE_ENABLE_CACHE,
//     PTE_EXECUTE_WRITECOPY   | PTE_ENABLE_CACHE,
//     //
//     // These OR in the MM_NOACCESS flag, which seems to enable WriteCombining?
//     //
//     0,
//     PTE_READONLY            | PTE_WRITECOMBINED_CACHE,
//     PTE_EXECUTE             | PTE_WRITECOMBINED_CACHE,
//     PTE_EXECUTE_READ        | PTE_WRITECOMBINED_CACHE,
//     PTE_READWRITE           | PTE_WRITECOMBINED_CACHE,
//     PTE_WRITECOPY           | PTE_WRITECOMBINED_CACHE,
//     PTE_EXECUTE_READWRITE   | PTE_WRITECOMBINED_CACHE,
//     PTE_EXECUTE_WRITECOPY   | PTE_WRITECOMBINED_CACHE,
// };
//
// const
// ULONG MmProtectToValue[32] =
// {
//     PAGE_NOACCESS,
//     PAGE_READONLY,
//     PAGE_EXECUTE,
//     PAGE_EXECUTE_READ,
//     PAGE_READWRITE,
//     PAGE_WRITECOPY,
//     PAGE_EXECUTE_READWRITE,
//     PAGE_EXECUTE_WRITECOPY,
//     PAGE_NOACCESS,
//     PAGE_NOCACHE | PAGE_READONLY,
//     PAGE_NOCACHE | PAGE_EXECUTE,
//     PAGE_NOCACHE | PAGE_EXECUTE_READ,
//     PAGE_NOCACHE | PAGE_READWRITE,
//     PAGE_NOCACHE | PAGE_WRITECOPY,
//     PAGE_NOCACHE | PAGE_EXECUTE_READWRITE,
//     PAGE_NOCACHE | PAGE_EXECUTE_WRITECOPY,
//     PAGE_NOACCESS,
//     PAGE_GUARD | PAGE_READONLY,
//     PAGE_GUARD | PAGE_EXECUTE,
//     PAGE_GUARD | PAGE_EXECUTE_READ,
//     PAGE_GUARD | PAGE_READWRITE,
//     PAGE_GUARD | PAGE_WRITECOPY,
//     PAGE_GUARD | PAGE_EXECUTE_READWRITE,
//     PAGE_GUARD | PAGE_EXECUTE_WRITECOPY,
//     PAGE_NOACCESS,
//     PAGE_WRITECOMBINE | PAGE_READONLY,
//     PAGE_WRITECOMBINE | PAGE_EXECUTE,
//     PAGE_WRITECOMBINE | PAGE_EXECUTE_READ,
//     PAGE_WRITECOMBINE | PAGE_READWRITE,
//     PAGE_WRITECOMBINE | PAGE_WRITECOPY,
//     PAGE_WRITECOMBINE | PAGE_EXECUTE_READWRITE,
//     PAGE_WRITECOMBINE | PAGE_EXECUTE_WRITECOPY,
// };
//
// /* FUNCTIONS ***************************************************************/
//
// static NTSTATUS
// MiFillSystemPageDirectory(IN PVOID Base,
//                           IN SIZE_T NumberOfBytes)
// {
//     PMMPDE PointerPde, LastPde, SystemMapPde;
//     MMPDE TempPde;
//     PFN_NUMBER PageFrameIndex;
//     KIRQL OldIrql;
//     BOOLEAN AllocatedPde;
//
//     /* Find the PDEs we need to check */
//     PointerPde = MiAddressToPde(Base);
//     LastPde = MiAddressToPde((PVOID)((ULONG_PTR)Base + NumberOfBytes - 1));
//
//     /* Check them all */
//     while (PointerPde <= LastPde)
//     {
//         /* Should not be a large page */
//         ASSERT(PointerPde->u.Hard.LargePage == FALSE);
//
//         /* Check if we don't have a PDE yet */
//         AllocatedPde = FALSE;
//         if (PointerPde->u.Hard.Valid == 0)
//         {
//             /* Need to sync from the master page directory */
//             AllocatedPde = MiSynchronizeSystemPde(PointerPde);
//         }
//
//         /* Now we should have one */
//         if (AllocatedPde == FALSE)
//         {
//             /* We need to allocate a new PDE */
//             OldIrql = MiAcquirePfnLock();
//             PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
//             MiReleasePfnLock(OldIrql);
//
//             /* Initialize it */
//             TempPde = ValidKernelPde;
//             TempPde.u.Hard.PageFrameNumber = PageFrameIndex;
//             MI_WRITE_VALID_PDE(PointerPde, TempPde);
//
//             /* Zero it out */
//             RtlZeroMemory(MiPdeToAddress(PointerPde), PAGE_SIZE);
//         }
//
//         /* Next */
//         PointerPde++;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// PFN_NUMBER
// NTAPI
// MmGetPfnForProcess(IN PEPROCESS Process,
//                    IN PVOID Address)
// {
//     PMMPTE PointerPte;
//     PFN_NUMBER Page;
//
//     MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());
//
//     if (MiIsPageTablePresent(Address) == FALSE)
//     {
//         MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
//         return 0;
//     }
//
//     MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
//
//     PointerPte = MiAddressToPte(Address);
//     Page = (PointerPte->u.Hard.Valid) ? PFN_FROM_PTE(PointerPte) : 0;
//
//     MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
//
//     return Page;
// }
