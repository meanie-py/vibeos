use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _KAPC_STATE,
    _EPROCESS,
    _MM_AVL_TABLE,
    PKPROCESS,
    PMMVAD,
    _ETHREAD,
    _MDL,
    _MMVAD_LONG,
    _MMPTE,
    _MMPFN,
    _MMADDRESS_NODE,
    PEPROCESS,
    PETHREAD,
    PMDL,
    PMMVAD_LONG,
    PMMADDRESS_NODE,
    KPROCESSOR_MODE,
    MEMORY_CACHING_TYPE,
    LOCK_OPERATION,
    PHYSICAL_ADDRESS,
    ACCESS_MASK,
    HANDLE,
    OB_OPEN_REASON,
    LARGE_INTEGER,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    KeFlushProcessTb,
    KeInvalidateAllCaches,
    MmAccessFault,
    MiAllocatePagesForMdl,
    MiCheckForConflictingNode,
    MiDecrementPageTableReferences,
    MiDecrementShareCount,
    MiDeletePde,
    MiDereferencePfnAndDropLockCount,
    MiFindEmptyAddressRangeInTree,
    MiIncrementPageTableReferences,
    MiInsertVad,
    MiLocateAddress,
    MiLockProcessWorkingSet,
    MiLockProcessWorkingSetUnsafe,
    MiMakePdeExistAndMakeValid,
    MiReferenceProbedPageAndBumpLockCount,
    MiRemoveNode,
    MiUnlockProcessWorkingSet,
    MiUnlockProcessWorkingSetUnsafe,
    PsChargeProcessNonPagedPoolQuota,
    PsReturnProcessNonPagedPoolQuota,
    PsGetCurrentThread,
    TABLE_SEARCH_RESULT,
};
use super::helpers::*;


#[no_mangle]
pub static mut MmTrackPtes: BOOLEAN = 0;

#[no_mangle]
pub static mut MmTrackLockedPages: BOOLEAN = 0;

// Registry-controlled values owned by ARM3/mdlsup. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static MDLSUP_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "TrackLockedPages",
            core::ptr::addr_of_mut!(MmTrackLockedPages)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "TrackPtes",
            core::ptr::addr_of_mut!(MmTrackPtes)),
    ] });

#[no_mangle]
pub static mut MmSystemLockPagesCount: SIZE_T = 0;

static mut MiCacheOverride: [ULONG; 4] = [0, 0, 0, 0];

unsafe fn MiMapLockedPagesInUserSpace(
    Mdl: PMDL,
    StartVa: PVOID,
    CacheType: MEMORY_CACHING_TYPE,
    mut BaseAddress: PVOID,
) -> PVOID {
    let mut Status: NTSTATUS;
    let Process = PsGetCurrentProcess() as *mut _EPROCESS;
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;
    let mut Result: TABLE_SEARCH_RESULT = 0;
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let mut EffectiveCacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let IsIoMapping: BOOLEAN;
    let mut OldIrql: KIRQL;
    let mut StartingVa: ULONG_PTR = 0;
    let mut EndingVa: ULONG_PTR;
    let mut Parent: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut Vad: PMMVAD_LONG;
    let NumberOfPages: ULONG;
    let mut PointerPte: PMMPTE;
    let mut PointerPde: PMMPTE;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut MdlPages: PPFN_NUMBER;
    let mut Pfn1: PMMPFN;
    let mut Pfn2: PMMPFN;
    let mut AddressSpaceLocked: BOOLEAN = 0u8;

    NumberOfPages = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(StartVa, MmGetMdlByteCount(Mdl) as usize)) as ULONG;
    MdlPages = MmGetMdlPfnArray(Mdl);

    IsIoMapping = if ((*Mdl).MdlFlags as u32 & MDL_IO_SPACE) != 0 { 1 } else { 0 };
    CacheAttribute = MiPlatformCacheAttributes[IsIoMapping as usize][CacheType as usize];

    if CacheAttribute != (MiCached) as u32 {
    }

    Status = PsChargeProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
    if Status < 0 {
        Vad = core::ptr::null_mut();
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if AddressSpaceLocked != 0 {
            MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
        }
        if !Vad.is_null() {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
            PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
        }
        ExRaiseStatus(Status);
    }

    Vad = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_MMVAD_LONG>(), u32::from_le_bytes(*b"Vadl")) as PMMVAD_LONG;
    if Vad.is_null() {
        PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if AddressSpaceLocked != 0 {
            MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
        }
        if !Vad.is_null() {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
            PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
        }
        ExRaiseStatus(Status);
    }

    RtlZeroMemory(Vad as PVOID, core::mem::size_of::<_MMVAD_LONG>());
    (*Vad).u2.VadFlags2 = core::mem::transmute(1u32 << 0);
    (*Vad).u.VadFlags = core::mem::transmute(((VadDevicePhysicalMemory as ULONG_PTR) << 0) | ((MM_READWRITE as ULONG_PTR) << 7) | (1 as ULONG_PTR) << 12);

    if BaseAddress.is_null() {
        MmLockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
        AddressSpaceLocked = 1;
        if (*Process).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
            Status = STATUS_PROCESS_IS_TERMINATING;
            if AddressSpaceLocked != 0 {
                MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
            }
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
                PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
            }
            ExRaiseStatus(Status);
        }

        Result = MiFindEmptyAddressRangeInTree(
            ((NumberOfPages as ULONG_PTR) << PAGE_SHIFT) as _,
            MM_VIRTMEM_GRANULARITY as _,
            core::ptr::addr_of_mut!((*Process).VadRoot) as *mut _MM_AVL_TABLE,
            &mut Parent,
            &mut StartingVa as *mut usize as *mut _,
        );
        if Result == TableFoundNode {
            Status = STATUS_NO_MEMORY;
            if AddressSpaceLocked != 0 {
                MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
            }
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
                PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
            }
            ExRaiseStatus(Status);
        }
        EndingVa = StartingVa + (NumberOfPages as ULONG_PTR) * PAGE_SIZE - 1;
        BaseAddress = StartingVa as PVOID;
    } else {
        StartingVa = BaseAddress as ULONG_PTR;
        EndingVa = StartingVa + (NumberOfPages as ULONG_PTR) * PAGE_SIZE - 1;

        if (StartingVa & (PAGE_SIZE - 1)) != 0 || EndingVa <= StartingVa || EndingVa > MM_HIGHEST_VAD_ADDRESS as ULONG_PTR {
            Status = STATUS_INVALID_ADDRESS;
            if AddressSpaceLocked != 0 {
                MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
            }
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
                PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
            }
            ExRaiseStatus(Status);
        }

        MmLockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
        AddressSpaceLocked = 1;
        if (*Process).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
            Status = STATUS_PROCESS_IS_TERMINATING;
            if AddressSpaceLocked != 0 {
                MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
            }
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
                PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
            }
            ExRaiseStatus(Status);
        }

        Result = MiCheckForConflictingNode(
            (StartingVa >> PAGE_SHIFT) as _,
            (EndingVa >> PAGE_SHIFT) as _,
            core::ptr::addr_of_mut!((*Process).VadRoot) as *mut _MM_AVL_TABLE,
            &mut Parent,
        );
        if Result == TableFoundNode {
            Status = STATUS_CONFLICTING_ADDRESSES;
            if AddressSpaceLocked != 0 {
                MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
            }
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
                PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);
            }
            ExRaiseStatus(Status);
        }
    }

    (*Vad).StartingVpn = (StartingVa >> PAGE_SHIFT) as _;
    (*Vad).EndingVpn = (EndingVa >> PAGE_SHIFT) as _;

    MiLockProcessWorkingSetUnsafe(Process, Thread);

    MiInsertVad(Vad as PMMVAD, core::ptr::addr_of_mut!((*Process).VadRoot) as *mut _MM_AVL_TABLE);

    if CacheAttribute != (MiCached) as u32 {
        KeFlushEntireTb(1, 1);
        KeInvalidateAllCaches();
    }

    PointerPte = MiAddressToPte(BaseAddress);
    let mut NumberOfPagesLeft = NumberOfPages;
    while NumberOfPagesLeft != 0 && *MdlPages != LIST_HEAD {
        PointerPde = MiPteToPde(PointerPte);
        MiMakePdeExistAndMakeValid(PointerPde as *mut _MMPTE, Process, MM_NOIRQL);

        MiIncrementPageTableReferences(BaseAddress);

        MI_MAKE_HARDWARE_PTE_USER(
            &mut TempPte,
            PointerPte,
            (MM_READWRITE) as usize,
            *MdlPages,
        );

        EffectiveCacheAttribute = CacheAttribute;

        Pfn2 = MiGetPfnEntry(*MdlPages);
        if !Pfn2.is_null() {
            match (*Pfn2).cache_attribute() {
                x if x == (MiNonCached as u8) as u16 => {
                    if CacheAttribute != MiNonCached {
                        MiCacheOverride[1] += 1;
                        EffectiveCacheAttribute = MiNonCached;
                    }
                }
                x if x == (MiCached as u8) as u16 => {
                    if CacheAttribute != (MiCached) as u32 {
                        MiCacheOverride[0] += 1;
                        EffectiveCacheAttribute = (MiCached) as u32;
                    }
                }
                x if x == (MiWriteCombined as u8) as u16 => {
                    if CacheAttribute != MiWriteCombined {
                        MiCacheOverride[2] += 1;
                        EffectiveCacheAttribute = MiWriteCombined;
                    }
                }
                _ => {}
            }
        }

        match EffectiveCacheAttribute {
            x if x == MiNonCached => {
                MI_PAGE_DISABLE_CACHE(&mut TempPte);
                MI_PAGE_WRITE_THROUGH(&mut TempPte);
            }
            x if x == (MiCached) as u32 => {}
            x if x == MiWriteCombined => {
                MI_PAGE_DISABLE_CACHE(&mut TempPte);
                MI_PAGE_WRITE_COMBINED(&mut TempPte);
            }
            _ => {}
        }

        MI_WRITE_VALID_PTE(PointerPte, TempPte);

        Pfn1 = MI_PFN_ELEMENT((*PointerPde).page_frame_number());
        OldIrql = MiAcquirePfnLock();
        (*Pfn1).set_share_count((*Pfn1).share_count().wrapping_add((1) as ULONG_PTR));
        MiReleasePfnLock(OldIrql);

        MdlPages = MdlPages.add(1);
        PointerPte = PointerPte.add(1);
        NumberOfPagesLeft -= 1;
        BaseAddress = (BaseAddress as ULONG_PTR + PAGE_SIZE) as PVOID;
    }

    MiUnlockProcessWorkingSetUnsafe(Process, Thread);
    MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);

    (StartingVa + MmGetMdlByteOffset(Mdl) as ULONG_PTR) as PVOID
}

unsafe fn MiUnmapLockedPagesInUserSpace(mut BaseAddress: PVOID, Mdl: PMDL) {
    let Process = PsGetCurrentProcess() as *mut _EPROCESS;
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;
    let Vad: PVOID;
    let mut PointerPte: PMMPTE;
    let mut PointerPde: PMMPTE;
    let mut OldIrql: KIRQL;
    let NumberOfPages: ULONG;
    let mut MdlPages: PPFN_NUMBER;
    let mut PageTablePage: PFN_NUMBER;

    NumberOfPages = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(MmGetMdlVirtualAddress(Mdl), MmGetMdlByteCount(Mdl) as usize)) as ULONG;
    MdlPages = MmGetMdlPfnArray(Mdl);

    MmLockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
    Vad = MiLocateAddress(BaseAddress) as *mut c_void;
    if Vad.is_null() || (*(Vad as *mut _MMVAD_LONG)).u.VadFlags.VadType() != (VadDevicePhysicalMemory) as _ {
        MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
        return;
    }

    MiLockProcessWorkingSetUnsafe(Process, Thread);

    MiRemoveNode(Vad as PMMADDRESS_NODE, core::ptr::addr_of_mut!((*Process).VadRoot) as *mut _MM_AVL_TABLE);
    PsReturnProcessNonPagedPoolQuota(Process, (core::mem::size_of::<_MMVAD_LONG>()) as _);

    PointerPte = MiAddressToPte(BaseAddress);
    OldIrql = MiAcquirePfnLock();
    let mut NumberOfPagesLeft = NumberOfPages;
    while NumberOfPagesLeft != 0 && *MdlPages != LIST_HEAD {
        MI_ERASE_PTE(PointerPte);

        PointerPde = MiAddressToPde(BaseAddress);
        PageTablePage = (*PointerPde).page_frame_number();
        MiDecrementShareCount(MiGetPfnEntry(PageTablePage) as *mut _MMPFN, (PageTablePage) as _);

        if MiDecrementPageTableReferences(BaseAddress) == 0 {
            MiDeletePde(PointerPde as *mut _MMPTE, Process);
        }

        PointerPte = PointerPte.add(1);
        NumberOfPagesLeft -= 1;
        BaseAddress = (BaseAddress as ULONG_PTR + PAGE_SIZE) as PVOID;
        MdlPages = MdlPages.add(1);
    }

    KeFlushProcessTb();
    MiReleasePfnLock(OldIrql);
    MiUnlockProcessWorkingSetUnsafe(Process, Thread);
    MmUnlockAddressSpace(core::ptr::addr_of_mut!((*Process).Vm) as PVOID);
    ExFreePoolWithTag(Vad, u32::from_le_bytes(*b"Vadl"));
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateMdl(
    Mdl: PMDL,
    Base: PVOID,
    Length: SIZE_T,
) -> PMDL {
    let Mdl = opaque_ptr(Mdl);
    let Base = opaque_ptr(Base);
    let Size: SIZE_T;

    if Mdl.is_null() {
        Size = MmSizeOfMdl(Base, Length);
        let new_mdl = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_MDL);
        if new_mdl.is_null() {
            return core::ptr::null_mut();
        }
        MmInitializeMdl(new_mdl as PMDL, Base, Length);
        return new_mdl as PMDL;
    }

    MmInitializeMdl(Mdl, Base, Length);
    Mdl
}

#[no_mangle]
pub unsafe extern "system" fn MmSizeOfMdl(Base: PVOID, Length: SIZE_T) -> SIZE_T {
    let Base = opaque_ptr(Base);
    core::mem::size_of::<_MDL>() + (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Length) as usize * core::mem::size_of::<PFN_NUMBER>())
}

#[no_mangle]
pub unsafe extern "system" fn MmBuildMdlForNonPagedPool(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
    let mut MdlPages: PPFN_NUMBER;
    let EndPage: PPFN_NUMBER;
    let mut Pfn: PFN_NUMBER = 0;
    let PageCount: PFN_NUMBER;
    let Base: PVOID;
    let mut PointerPte: PMMPTE;

    (*Mdl).Process = core::ptr::null_mut();

    MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
    Base = (*Mdl).StartVa;

    (*Mdl).MappedSystemVa = (Base as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES((*Mdl).MappedSystemVa, (*Mdl).ByteCount as usize) as PFN_NUMBER;
    EndPage = MdlPages.add(PageCount as usize);

    PointerPte = MiAddressToPte(Base);
    while MdlPages < EndPage {
        Pfn = PFN_FROM_PTE(PointerPte);
        *MdlPages = Pfn;
        PointerPte = PointerPte.add(1);
        MdlPages = MdlPages.add(1);
    }

    (*Mdl).MdlFlags |= MDL_SOURCE_IS_NONPAGED_POOL as i16;

    if MiGetPfnEntry(Pfn).is_null() {
        (*Mdl).MdlFlags |= MDL_IO_SPACE as i16;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocatePagesForMdl(
    LowAddress: PHYSICAL_ADDRESS,
    HighAddress: PHYSICAL_ADDRESS,
    SkipBytes: PHYSICAL_ADDRESS,
    TotalBytes: SIZE_T,
) -> PMDL {
    MiAllocatePagesForMdl(LowAddress, HighAddress, SkipBytes, (TotalBytes) as _, MiNotMapped, 0)
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocatePagesForMdlEx(
    LowAddress: PHYSICAL_ADDRESS,
    HighAddress: PHYSICAL_ADDRESS,
    SkipBytes: PHYSICAL_ADDRESS,
    TotalBytes: SIZE_T,
    CacheType: MEMORY_CACHING_TYPE,
    Flags: ULONG,
) -> PMDL {
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;

    if CacheType > MmWriteCombined {
        CacheAttribute = MiNotMapped;
    } else {
        CacheAttribute = MiPlatformCacheAttributes[0][CacheType as usize];
    }

    if (Flags & !(0x00000001 | 0x00000002)) != 0 {
        return core::ptr::null_mut();
    }

    MiAllocatePagesForMdl(LowAddress, HighAddress, SkipBytes, (TotalBytes) as _, CacheAttribute, Flags)
}

#[no_mangle]
pub unsafe extern "system" fn MmFreePagesFromMdl(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
    let Base: PVOID;
    let mut Pages: PPFN_NUMBER;
    let mut NumberOfPages: LONG;
    let mut Pfn1: PMMPFN;
    let OldIrql: KIRQL;

    Base = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    NumberOfPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, (*Mdl).ByteCount as usize) as LONG;

    OldIrql = MiAcquirePfnLock();

    Pages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
    loop {
        if NumberOfPages == 0 {
            break;
        }

        if *Pages == LIST_HEAD {
            break;
        }

        Pfn1 = MiGetPfnEntry(*Pages);

        if (*Pfn1).pte_frame() != 0x1FFEDCB {
            KeBugCheckEx(MEMORY_MANAGEMENT, 0x1236, Mdl as ULONG_PTR, Pages as ULONG_PTR, *Pages as ULONG_PTR);
        }

        (*Pfn1).set_start_of_allocation((0) != 0);
        (*Pfn1).set_end_of_allocation((0) != 0);
        (*Pfn1).set_page_location((StandbyPageList as u8) as u16);
        (*Pfn1).set_share_count((0) as ULONG_PTR);

        if (*Pfn1).reference_count() != 1 {
            { let _rc = (*Pfn1).reference_count(); (*Pfn1).set_reference_count(_rc - 1); }
        } else {
            MiDecrementReferenceCount(Pfn1, *Pages);
        }

        *Pages = LIST_HEAD;
        Pages = Pages.add(1);
        NumberOfPages -= 1;
    }

    MiReleasePfnLock(OldIrql);

    (*Mdl).MdlFlags &= !(MDL_PAGES_LOCKED as i16);
}

#[no_mangle]
pub unsafe extern "system" fn MmMapLockedPagesSpecifyCache(
    Mdl: PMDL,
    AccessMode: KPROCESSOR_MODE,
    CacheType: MEMORY_CACHING_TYPE,
    BaseAddress: PVOID,
    BugCheckOnFailure: ULONG,
    Priority: ULONG,
) -> PVOID {
    let Mdl = opaque_ptr(Mdl);
    let mut BaseAddress = opaque_ptr(BaseAddress);
    let mut Base: PVOID;
    let mut MdlPages: PPFN_NUMBER;
    let LastPage: PPFN_NUMBER;
    let PageCount: PFN_COUNT;
    let IsIoMapping: BOOLEAN;
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let mut PointerPte: PMMPTE;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };

    Base = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;

    if AccessMode == 0i8 {
        MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
        PageCount = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, (*Mdl).ByteCount as usize)) as u32;
        LastPage = MdlPages.add(PageCount as usize);

        IsIoMapping = if ((*Mdl).MdlFlags as u32 & MDL_IO_SPACE) != 0 { 1 } else { 0 };
        CacheAttribute = MiPlatformCacheAttributes[IsIoMapping as usize][CacheType as usize];

        PointerPte = MiReserveSystemPtes(PageCount, SystemPteSpace);
        if PointerPte.is_null() {
            if ((*Mdl).MdlFlags & 0x00000004) != 0 {
                return core::ptr::null_mut();
            }

            if BugCheckOnFailure == 0 {
                return core::ptr::null_mut();
            }

            KeBugCheckEx(ntkrnl_sys::bugcodes::NO_MORE_SYSTEM_PTES, 0, PageCount as ULONG_PTR, 0, 0);
        }

        Base = (MiPteToAddress(PointerPte) as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;

        TempPte = ValidKernelPte;
        match CacheAttribute {
            x if x == MiNonCached => {
                MI_PAGE_DISABLE_CACHE(&mut TempPte);
                MI_PAGE_WRITE_THROUGH(&mut TempPte);
            }
            x if x == MiWriteCombined => {
                MI_PAGE_DISABLE_CACHE(&mut TempPte);
                MI_PAGE_WRITE_COMBINED(&mut TempPte);
            }
            _ => {}
        }

        loop {
            if *MdlPages == LIST_HEAD {
                break;
            }

            TempPte.set_page_frame_number((*MdlPages) as PFN_NUMBER);
            MI_WRITE_VALID_PTE(PointerPte, TempPte);
            PointerPte = PointerPte.add(1);
            MdlPages = MdlPages.add(1);
            if MdlPages >= LastPage {
                break;
            }
        }

        (*Mdl).MappedSystemVa = Base;
        (*Mdl).MdlFlags |= MDL_MAPPED_TO_SYSTEM_VA as i16;

        if ((*Mdl).MdlFlags as u32 & MDL_PARTIAL) != 0 {
            (*Mdl).MdlFlags |= 0x00000010 as i16;
        }

        return Base;
    }

    MiMapLockedPagesInUserSpace(Mdl, Base, CacheType, BaseAddress)
}

#[no_mangle]
pub unsafe extern "system" fn MmMapLockedPages(
    Mdl: PMDL,
    AccessMode: KPROCESSOR_MODE,
) -> PVOID {
    let Mdl = opaque_ptr(Mdl);
    MmMapLockedPagesSpecifyCache(Mdl, AccessMode, 0, core::ptr::null_mut(), 1, HighPagePriority as ULONG)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapLockedPages(mut BaseAddress: PVOID, Mdl: PMDL) {
    let mut BaseAddress = opaque_ptr(BaseAddress);
    let Mdl = opaque_ptr(Mdl);
    let Base: PVOID;
    let mut PageCount: PFN_COUNT;
    let ExtraPageCount: PFN_COUNT;
    let MdlPages: PPFN_NUMBER;
    let mut PointerPte: PMMPTE;

    if BaseAddress as ULONG_PTR > MM_HIGHEST_USER_ADDRESS as ULONG_PTR {
        // Use-after-free defense that is NOT a size cap. A genuinely system-mapped
        // MDL can never span more PTEs than the entire system-PTE pool, so an absurd
        // PageCount is the tell that this MDL is freed pool fill (0xFCFCFCFC…) — reject
        // it. The old `ByteCount > 16MB → return` band-aid (commit 15434cd8dea, an
        // i386-bringup patch) false-fired on every legitimate large MDL: kmtest MmMdl
        // maps 32MB MDLs, so unmap bailed WITHOUT releasing — and worse, without
        // ZEROING — their system PTEs. That left freed pages (MmFreePagesFromMdl runs
        // right after) still mapped by VALID PTEs with live TLB entries: the -smp 2
        // use-after-free alias behind the MmMdl 0x1E, plus a steady system-PTE leak
        // that eventually starved MmMapLockedPagesSpecifyCache to NULL. Releasing
        // properly (below) zeroes the PTEs, closing the alias and returning the pool.
        // MiReleaseSystemPtes bounds-checks StartingPte itself, so a genuinely corrupt
        // (but small) PageCount still can't scribble the pool.
        let total_sys_ptes = core::ptr::addr_of!(crate::ARM3::syspte::MmTotalSystemPtes).read();
        let byte_count = (*Mdl).ByteCount;
        Base = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
        PageCount = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, byte_count as usize)) as u32;

        if PageCount == 0 || PageCount > total_sys_ptes {
            return;
        }

        PointerPte = MiAddressToPte(BaseAddress);

        if ((*Mdl).MdlFlags as u32 & 0x0200) != 0 { // MDL_FREE_EXTRA_PTES
            MdlPages = MmGetMdlPfnArray(Mdl);
            ExtraPageCount = *(MdlPages.add(PageCount as usize)) as PFN_COUNT;
            // Guard: ExtraPageCount from corrupt PFN array entry (use-after-free / out-of-bounds)
            if ExtraPageCount > total_sys_ptes {
                return;
            }
            PageCount += ExtraPageCount;
            PointerPte = PointerPte.sub(ExtraPageCount as usize);
            BaseAddress = (BaseAddress as ULONG_PTR - ((ExtraPageCount as ULONG_PTR) << PAGE_SHIFT)) as PVOID;
        }

        // Total span after adjustments must still fit the pool.
        if PageCount > total_sys_ptes {
            return;
        }

        (*Mdl).MdlFlags &= (!(MDL_MAPPED_TO_SYSTEM_VA | 0x0020 | 0x0200)) as i16; // MDL_PARTIAL_HAS_BEEN_MAPPED | MDL_FREE_EXTRA_PTES

        MiReleaseSystemPtes(PointerPte, PageCount, SystemPteSpace);
    } else {
        MiUnmapLockedPagesInUserSpace(BaseAddress, Mdl);
    }
}

// The exported API raises on failure, as the C does. Rust callers cannot catch that raise (no
// kernel SEH around their call sites), so they call MmTryProbeAndLockPages and get the status.
#[no_mangle]
pub unsafe extern "system" fn MmProbeAndLockPages(
    Mdl: PMDL,
    AccessMode: KPROCESSOR_MODE,
    Operation: LOCK_OPERATION,
) {
    let Status = MmTryProbeAndLockPages(Mdl, AccessMode, Operation);
    if Status < 0 {
        ExRaiseStatus(Status);
    }
}

/// MmProbeAndLockPages returning its failure instead of raising it (docs/engineering/
/// dummy-pseh-verdict.md, class 2: convert the contract at the callee). Every failure path leaves
/// the MDL exactly as the raising version did. Exported so the other Rust crates linked into the
/// kernel image can call it; the raising export stays as the NT ABI for everything else.
#[no_mangle]
pub unsafe extern "system" fn MmTryProbeAndLockPages(
    Mdl: PMDL,
    AccessMode: KPROCESSOR_MODE,
    mut Operation: LOCK_OPERATION,
) -> NTSTATUS {
    let Mdl = opaque_ptr(Mdl);
    let mut MdlPages: PPFN_NUMBER;
    let mut Address: PVOID;
    let mut LastAddress: PVOID;
    let StartAddress: PVOID;
    let mut LockPages: ULONG;
    let TotalPages: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let CurrentProcess: *mut _EPROCESS;
    let mut ProbeStatus: NTSTATUS;
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPxe: PMMPTE;
    let mut PageFrameIndex: PFN_NUMBER;
    let UsePfnLock: BOOLEAN;
    let mut OldIrql: KIRQL;
    let mut Pfn1: PMMPFN;

    MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
    let Base = (*Mdl).StartVa;

    Address = (Base as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    LastAddress = (Address as ULONG_PTR + (*Mdl).ByteCount as ULONG_PTR) as PVOID;
    LockPages = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Address, (*Mdl).ByteCount as usize)) as u32;

    if (AccessMode != 0i8) && ((LastAddress as ULONG_PTR > (0x7FFFFFFF0000u64) as usize) || (Address >= LastAddress)) {
        *MdlPages = LIST_HEAD;
        return STATUS_ACCESS_VIOLATION;
    }

    if Address as ULONG_PTR <= MM_HIGHEST_USER_ADDRESS as ULONG_PTR {
        CurrentProcess = PsGetCurrentProcess() as *mut _EPROCESS;
    } else {
        CurrentProcess = core::ptr::null_mut();
    }

    TotalPages = LockPages;
    StartAddress = Address;

    ProbeStatus = STATUS_SUCCESS;
    let mut mdl_ptr = MdlPages;
    let mut addr = Address;
    let mut pages_left = LockPages;
    loop {
        *mdl_ptr = LIST_HEAD;
        // The C touches each page inside _SEH2_TRY and turns a fault into ProbeStatus. The
        // translation kept a bare `*(addr as *const CHAR);` -- unprotected, and a non-volatile
        // read whose value is unused, so it may not even be emitted. A fault here is a hostile
        // or unmapped caller address (ReadProcessMemory past the end of a stack), and it
        // BUGCHECKed 0x1E. Touch through the fault-tolerant copy instead.
        let mut byte: u8 = 0;
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!(byte) as *mut core::ffi::c_void,
            addr as *const core::ffi::c_void,
            1,
        ) {
            ProbeStatus = st;
            break;
        }
        if (Operation != IoReadAccess) && (addr as ULONG_PTR <= MM_HIGHEST_USER_ADDRESS as ULONG_PTR) {
            // ProbeForWriteChar: write the byte back, fault-protected.
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                addr as *mut core::ffi::c_void,
                core::ptr::addr_of!(byte) as *const core::ffi::c_void,
                1,
            ) {
                ProbeStatus = st;
                break;
            }
        }
        addr = ((addr as ULONG_PTR + PAGE_SIZE) & !(PAGE_SIZE - 1)) as PVOID;
        pages_left -= 1;
        mdl_ptr = mdl_ptr.add(1);
        if addr >= LastAddress || pages_left == 0 {
            break;
        }
    }

    if ProbeStatus < 0 {
        (*Mdl).Process = core::ptr::null_mut();
        return ProbeStatus;
    }

    PointerPte = MiAddressToPte(StartAddress);
    PointerPde = MiAddressToPde(StartAddress);
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        PointerPpe = MiAddressToPpe(StartAddress);
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        PointerPxe = MiAddressToPxe(StartAddress);
    }

    MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;

    if Operation != IoReadAccess {
        (*Mdl).MdlFlags |= MDL_WRITE_OPERATION as i16;
    } else {
        (*Mdl).MdlFlags &= !(MDL_WRITE_OPERATION as i16);
    }

    (*Mdl).MdlFlags |= MDL_PAGES_LOCKED as i16;

    if Base as ULONG_PTR > MM_HIGHEST_USER_ADDRESS as ULONG_PTR {
        (*Mdl).Process = core::ptr::null_mut();
        Operation = IoReadAccess;
        UsePfnLock = 1;
        OldIrql = MiAcquirePfnLock();
    } else {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*CurrentProcess).NumberOfLockedPages) as *mut isize,
            TotalPages as isize,
        );
        (*Mdl).Process = CurrentProcess as PVOID as *mut _EPROCESS;
        MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
        UsePfnLock = 0;
        OldIrql = MM_NOIRQL;
    }

    LastPte = MiAddressToPte((LastAddress as ULONG_PTR - 1) as PVOID);

    loop {
        *MdlPages = LIST_HEAD;
        while {
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            let px_invalid = ((*PointerPxe).valid() as usize) == 0;
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            let px_invalid = false;

            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            let pp_invalid = ((*PointerPpe).valid() as usize) == 0;
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            let pp_invalid = false;

            px_invalid || pp_invalid || ((*PointerPde).valid() as usize) == 0 || ((*PointerPte).valid() as usize) == 0
        } {
            if UsePfnLock != 0 {
                MiReleasePfnLock(OldIrql);
            } else {
                MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
            }

            Address = MiPteToAddress(PointerPte);

            Status = MmAccessFault(0, Address, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
            if Status < 0 {
                // The lock was dropped before MmAccessFault; the C's `goto Cleanup` skips the release.
                // Releasing it again here released a lock this thread no longer held.
                MmUnlockPages(Mdl);
                return Status;
            }

            if UsePfnLock != 0 {
                OldIrql = MiAcquirePfnLock();
            } else {
                MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
            }
        }

        if Operation != IoReadAccess {
            if MI_IS_PAGE_WRITEABLE(PointerPte) == 0 {
                if MI_IS_PAGE_COPY_ON_WRITE(PointerPte) != 0 {
                    Address = MiPteToAddress(PointerPte);
                    if Address as ULONG_PTR <= MM_HIGHEST_USER_ADDRESS as ULONG_PTR {
                        if UsePfnLock != 0 {
                            MiReleasePfnLock(OldIrql);
                        } else {
                            MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
                        }

                        Status = MmAccessFault(1, Address, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
                        if Status < 0 {
                            // The lock was dropped before MmAccessFault; the C's `goto Cleanup` skips the release.
                            // Releasing it again here released a lock this thread no longer held.
                            MmUnlockPages(Mdl);
                            return Status;
                        }

                        if UsePfnLock != 0 {
                            OldIrql = MiAcquirePfnLock();
                        } else {
                            MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
                        }

                        continue;
                    }
                }

                Status = STATUS_ACCESS_VIOLATION;
                if UsePfnLock != 0 {
                    MiReleasePfnLock(OldIrql);
                } else {
                    MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
                }
                MmUnlockPages(Mdl);
                return Status;
            }
        }

        PageFrameIndex = PFN_FROM_PTE(PointerPte);
        Pfn1 = MiGetPfnEntry(PageFrameIndex);
        if !Pfn1.is_null() {
            MiReferenceProbedPageAndBumpLockCount(Pfn1 as *mut _MMPFN);
        } else {
            (*Mdl).MdlFlags |= MDL_IO_SPACE as i16;
        }

        *MdlPages = PageFrameIndex;
        MdlPages = MdlPages.add(1);
        PointerPte = PointerPte.add(1);

        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = PointerPde.add(1);
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        if MiIsPteOnPpeBoundary(PointerPte) {
            PointerPpe = PointerPpe.add(1);
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        if MiIsPteOnPxeBoundary(PointerPte) {
            PointerPxe = PointerPxe.add(1);
        }

        if PointerPte > LastPte {
            break;
        }
    }

    if UsePfnLock != 0 {
        MiReleasePfnLock(OldIrql);
    } else {
        MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as *mut _ETHREAD);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmUnlockPages(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
    let mut MdlPages: PPFN_NUMBER;
    let mut LastPage: PPFN_NUMBER;
    let Process: *mut _EPROCESS;
    let Base: PVOID;
    let Flags: ULONG;
    let PageCount: ULONG;
    let mut OldIrql: KIRQL;
    let mut Pfn1: PMMPFN;

    // Guard against use-after-free: freed pool fill makes ByteCount = 0xFCFCFCFC
    let byte_count = (*Mdl).ByteCount;
    if byte_count > 0x01000000 {
        return;
    }

    Process = (*Mdl).Process as *mut _EPROCESS;
    Flags = ((*Mdl).MdlFlags) as u32;

    if ((*Mdl).MdlFlags as u32 & MDL_MAPPED_TO_SYSTEM_VA) != 0 {
        MmUnmapLockedPages((*Mdl).MappedSystemVa, Mdl);
    }

    MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
    Base = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    PageCount = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, (*Mdl).ByteCount as usize)) as u32;

    if (Flags & MDL_IO_SPACE) != 0 {
        OldIrql = MiAcquirePfnLock();

        LastPage = MdlPages.add(PageCount as usize);
        loop {
            if *MdlPages == LIST_HEAD {
                break;
            }

            Pfn1 = MiGetPfnEntry(*MdlPages);
            if !Pfn1.is_null() {
                MiDereferencePfnAndDropLockCount(Pfn1 as *mut _MMPFN);
            }
            MdlPages = MdlPages.add(1);
            if MdlPages >= LastPage {
                break;
            }
        }

        MiReleasePfnLock(OldIrql);

        if !Process.is_null() {
            InterlockedExchangeAddSizeT(
                core::ptr::addr_of_mut!((*Process).NumberOfLockedPages) as *mut isize,
                -(PageCount as isize),
            );
        }

        (*Mdl).MdlFlags &= !(MDL_IO_SPACE as i16);
        (*Mdl).MdlFlags &= !(MDL_PAGES_LOCKED as i16);
        return;
    }

    if !Process.is_null() {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*Process).NumberOfLockedPages) as *mut isize,
            -(PageCount as isize),
        );
    }

    LastPage = MdlPages.add(PageCount as usize);
    loop {
        if *MdlPages == LIST_HEAD {
            if MdlPages == ((Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER) {
                (*Mdl).MdlFlags &= !(MDL_PAGES_LOCKED as i16);
                return;
            }
            LastPage = MdlPages;
            break;
        }

        *MdlPages = MiGetPfnEntry(*MdlPages) as PFN_NUMBER;
        MdlPages = MdlPages.add(1);
        if MdlPages >= LastPage {
            break;
        }
    }

    MdlPages = (Mdl as usize + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;

    OldIrql = MiAcquirePfnLock();
    loop {
        Pfn1 = *MdlPages as PMMPFN;
        MiDereferencePfnAndDropLockCount(Pfn1 as *mut _MMPFN);
        MdlPages = MdlPages.add(1);
        if MdlPages >= LastPage {
            break;
        }
    }

    MiReleasePfnLock(OldIrql);

    (*Mdl).MdlFlags &= !(MDL_PAGES_LOCKED as i16);
}

#[no_mangle]
pub unsafe extern "system" fn MmAdvanceMdl(Mdl: PMDL, NumberOfBytes: ULONG) -> NTSTATUS {
    let Mdl = opaque_ptr(Mdl);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmMapLockedPagesWithReservedMapping(
    MappingAddress: PVOID,
    PoolTag: ULONG,
    Mdl: PMDL,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    let MappingAddress = opaque_ptr(MappingAddress);
    let Mdl = opaque_ptr(Mdl);
    let mut MdlPages: PPFN_NUMBER;
    let LastPage: PPFN_NUMBER;
    let PageCount: PFN_COUNT;
    let IsIoMapping: BOOLEAN;
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let mut PointerPte: PMMPTE;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };

    MdlPages = MmGetMdlPfnArray(Mdl);
    PageCount = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(MmGetMdlVirtualAddress(Mdl), (*Mdl).ByteCount as usize)) as u32;
    LastPage = MdlPages.add(PageCount as usize);

    IsIoMapping = if ((*Mdl).MdlFlags as u32 & MDL_IO_SPACE) != 0 { 1 } else { 0 };
    CacheAttribute = MiPlatformCacheAttributes[IsIoMapping as usize][CacheType as usize];

    PointerPte = (MiAddressToPte(MappingAddress) as usize - 2 * core::mem::size_of::<_MMPTE>()) as PMMPTE;

    TempPte.Long = PoolTag as usize;
    TempPte.set_valid(false);
    if (*PointerPte.add(1)).Long != TempPte.Long {
        KeBugCheckEx(SYSTEM_PTE_MISUSE, 0x100, MappingAddress as ULONG_PTR, PoolTag as ULONG_PTR, (*PointerPte.add(1)).Long as ULONG_PTR);
    }

    if mmpte_list_next_entry(&*PointerPte) < 3 {
        KeBugCheckEx(SYSTEM_PTE_MISUSE, 0x101, MappingAddress as ULONG_PTR, PoolTag as ULONG_PTR, 0);
    }

    if (mmpte_list_next_entry(&*PointerPte) as PFN_COUNT) - 2 < PageCount {
        return core::ptr::null_mut();
    }
    PointerPte = PointerPte.add(2);

    TempPte = ValidKernelPte;
    match CacheAttribute {
        x if x == MiNonCached => {
            MI_PAGE_DISABLE_CACHE(&mut TempPte);
            MI_PAGE_WRITE_THROUGH(&mut TempPte);
        }
        x if x == MiWriteCombined => {
            MI_PAGE_DISABLE_CACHE(&mut TempPte);
            MI_PAGE_WRITE_COMBINED(&mut TempPte);
        }
        _ => {}
    }

    while (MdlPages < LastPage) && (*MdlPages != LIST_HEAD) {
        TempPte.set_page_frame_number((*MdlPages) as PFN_NUMBER);
        MI_WRITE_VALID_PTE(PointerPte, TempPte);
        PointerPte = PointerPte.add(1);
        MdlPages = MdlPages.add(1);
    }

    (*Mdl).MappedSystemVa = MappingAddress;
    (*Mdl).MdlFlags |= MDL_MAPPED_TO_SYSTEM_VA as i16;

    if ((*Mdl).MdlFlags as u32 & MDL_PARTIAL) != 0 {
        (*Mdl).MdlFlags |= 0x00000010 as i16;
    }

    (MappingAddress as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapReservedMapping(
    mut BaseAddress: PVOID,
    PoolTag: ULONG,
    Mdl: PMDL,
) {
    let mut BaseAddress = opaque_ptr(BaseAddress);
    let Mdl = opaque_ptr(Mdl);
    let Base: PVOID;
    let mut PageCount: PFN_COUNT;
    let ExtraPageCount: PFN_COUNT;
    let MdlPages: PPFN_NUMBER;
    let mut PointerPte: PMMPTE;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };

    Base = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    PageCount = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, (*Mdl).ByteCount as usize)) as u32;

    PointerPte = (MiAddressToPte(BaseAddress) as usize - 2 * core::mem::size_of::<_MMPTE>()) as PMMPTE;

    TempPte.Long = PoolTag as usize;
    TempPte.set_valid(false);
    if (*PointerPte.add(1)).Long != TempPte.Long {
        KeBugCheckEx(SYSTEM_PTE_MISUSE, 0x102, BaseAddress as ULONG_PTR, PoolTag as ULONG_PTR, (*PointerPte.add(1)).Long as ULONG_PTR);
    }

    if mmpte_list_next_entry(&*PointerPte) < 3 {
        KeBugCheckEx(SYSTEM_PTE_MISUSE, 0x103, BaseAddress as ULONG_PTR, PoolTag as ULONG_PTR, 0);
    }

    PointerPte = PointerPte.add(2);

    if ((*Mdl).MdlFlags as u32 & 0x0200) != 0 { // MDL_FREE_EXTRA_PTES
        MdlPages = MmGetMdlPfnArray(Mdl);
        ExtraPageCount = *(MdlPages.add(PageCount as usize)) as PFN_COUNT;
        PageCount += ExtraPageCount;
        PointerPte = (PointerPte as usize - (ExtraPageCount as usize * core::mem::size_of::<_MMPTE>())) as PMMPTE;
        BaseAddress = (BaseAddress as ULONG_PTR - ((ExtraPageCount as ULONG_PTR) << PAGE_SHIFT)) as PVOID;
    }

    RtlZeroMemory(PointerPte as PVOID, PageCount as usize * core::mem::size_of::<_MMPTE>());

    KeFlushEntireTb(1, 1);

    (*Mdl).MdlFlags &= (!(MDL_MAPPED_TO_SYSTEM_VA | 0x0020 | 0x0200)) as i16; // MDL_PARTIAL_HAS_BEEN_MAPPED | MDL_FREE_EXTRA_PTES
}

#[no_mangle]
pub unsafe extern "system" fn MmPrefetchPages(
    NumberOfLists: ULONG,
    ReadLists: *mut PVOID,
) -> NTSTATUS {
    let ReadLists = opaque_ptr(ReadLists);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmProtectMdlSystemAddress(
    MemoryDescriptorList: PMDL,
    NewProtect: ULONG,
) -> NTSTATUS {
    let MemoryDescriptorList = opaque_ptr(MemoryDescriptorList);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmProbeAndLockProcessPages(
    MemoryDescriptorList: PMDL,
    Process: PEPROCESS,
    AccessMode: KPROCESSOR_MODE,
    Operation: LOCK_OPERATION,
) {
    let MemoryDescriptorList = opaque_ptr(MemoryDescriptorList);
    let Process = opaque_ptr(Process);
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut IsAttached: BOOLEAN = 0;

    if Process != PsGetCurrentProcess() as *mut _EPROCESS {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb) as PKPROCESS,
            &mut ApcState as *mut _KAPC_STATE,
        );
        IsAttached = 1;
    }

    // The C detached in a _SEH2_FINALLY; raising out of here instead left the thread attached
    // to the other process. Take the status, detach, then raise.
    let Status = MmTryProbeAndLockPages(MemoryDescriptorList, AccessMode, Operation);

    if IsAttached != 0 {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
    }
    if Status < 0 {
        ExRaiseStatus(Status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmProbeAndLockSelectedPages(
    MemoryDescriptorList: PMDL,
    PageList: *mut LARGE_INTEGER,
    AccessMode: KPROCESSOR_MODE,
    Operation: LOCK_OPERATION,
) {
    let MemoryDescriptorList = opaque_ptr(MemoryDescriptorList);
    let PageList = opaque_ptr(PageList);
}

#[no_mangle]
pub unsafe extern "system" fn MmMapMemoryDumpMdl(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/mdlsup.c
//  * PURPOSE:         ARM Memory Manager Memory Descriptor List (MDL) Management
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// /* GLOBALS ********************************************************************/
//
// BOOLEAN MmTrackPtes;
// BOOLEAN MmTrackLockedPages;
// SIZE_T MmSystemLockPagesCount;
//
// ULONG MiCacheOverride[MiNotMapped + 1];
//
// /* INTERNAL FUNCTIONS *********************************************************/
// static
// PVOID
// NTAPI
// MiMapLockedPagesInUserSpace(
//     _In_ PMDL Mdl,
//     _In_ PVOID StartVa,
//     _In_ MEMORY_CACHING_TYPE CacheType,
//     _In_opt_ PVOID BaseAddress)
// {
//     NTSTATUS Status;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     TABLE_SEARCH_RESULT Result;
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//     MI_PFN_CACHE_ATTRIBUTE EffectiveCacheAttribute;
//     BOOLEAN IsIoMapping;
//     KIRQL OldIrql;
//     ULONG_PTR StartingVa;
//     ULONG_PTR EndingVa;
//     PMMADDRESS_NODE Parent;
//     PMMVAD_LONG Vad;
//     ULONG NumberOfPages;
//     PMMPTE PointerPte;
//     PMMPDE PointerPde;
//     MMPTE TempPte;
//     PPFN_NUMBER MdlPages;
//     PMMPFN Pfn1;
//     PMMPFN Pfn2;
//     BOOLEAN AddressSpaceLocked = FALSE;
//
//     PAGED_CODE();
//
//     DPRINT("MiMapLockedPagesInUserSpace(%p, %p, 0x%x, %p)\n",
//            Mdl, StartVa, CacheType, BaseAddress);
//
//     NumberOfPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(StartVa,
//                                                    MmGetMdlByteCount(Mdl));
//     MdlPages = MmGetMdlPfnArray(Mdl);
//
//     ASSERT(CacheType <= MmWriteCombined);
//
//     IsIoMapping = (Mdl->MdlFlags & MDL_IO_SPACE) != 0;
//     CacheAttribute = MiPlatformCacheAttributes[IsIoMapping][CacheType];
//
//     /* Large pages are always cached, make sure we're not asking for those */
//     if (CacheAttribute != MiCached)
//     {
//         DPRINT1("FIXME: Need to check for large pages\n");
//     }
//
//     Status = PsChargeProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     if (!NT_SUCCESS(Status))
//     {
//         Vad = NULL;
//         goto Error;
//     }
//
//     /* Allocate a VAD for our mapped region */
//     Vad = ExAllocatePoolWithTag(NonPagedPool, sizeof(MMVAD_LONG), 'ldaV');
//     if (Vad == NULL)
//     {
//         PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Error;
//     }
//
//     /* Initialize PhysicalMemory VAD */
//     RtlZeroMemory(Vad, sizeof(*Vad));
//     Vad->u2.VadFlags2.LongVad = 1;
//     Vad->u.VadFlags.VadType() = VadDevicePhysicalMemory;
//     Vad->u.VadFlags.Protection() = MM_READWRITE;
//     Vad->u.VadFlags.PrivateMemory() = 1;
//
//     /* Did the caller specify an address? */
//     if (BaseAddress == NULL)
//     {
//         /* We get to pick the address */
//         MmLockAddressSpace(&Process->Vm);
//         AddressSpaceLocked = TRUE;
//         if (Process->VmDeleted)
//         {
//             Status = STATUS_PROCESS_IS_TERMINATING;
//             goto Error;
//         }
//
//         Result = MiFindEmptyAddressRangeInTree(NumberOfPages << PAGE_SHIFT,
//                                                MM_VIRTMEM_GRANULARITY,
//                                                &Process->VadRoot,
//                                                &Parent,
//                                                &StartingVa);
//         if (Result == TableFoundNode)
//         {
//             Status = STATUS_NO_MEMORY;
//             goto Error;
//         }
//         EndingVa = StartingVa + NumberOfPages * PAGE_SIZE - 1;
//         BaseAddress = (PVOID)StartingVa;
//     }
//     else
//     {
//         /* Caller specified a base address */
//         StartingVa = (ULONG_PTR)BaseAddress;
//         EndingVa = StartingVa + NumberOfPages * PAGE_SIZE - 1;
//
//         /* Make sure it's valid */
//         if (BYTE_OFFSET(StartingVa) != 0 ||
//             EndingVa <= StartingVa ||
//             EndingVa > (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS)
//         {
//             Status = STATUS_INVALID_ADDRESS;
//             goto Error;
//         }
//
//         MmLockAddressSpace(&Process->Vm);
//         AddressSpaceLocked = TRUE;
//         if (Process->VmDeleted)
//         {
//             Status = STATUS_PROCESS_IS_TERMINATING;
//             goto Error;
//         }
//
//         /* Check if it's already in use */
//         Result = MiCheckForConflictingNode(StartingVa >> PAGE_SHIFT,
//                                            EndingVa >> PAGE_SHIFT,
//                                            &Process->VadRoot,
//                                            &Parent);
//         if (Result == TableFoundNode)
//         {
//             Status = STATUS_CONFLICTING_ADDRESSES;
//             goto Error;
//         }
//     }
//
//     Vad->StartingVpn = StartingVa >> PAGE_SHIFT;
//     Vad->EndingVpn = EndingVa >> PAGE_SHIFT;
//
//     MiLockProcessWorkingSetUnsafe(Process, Thread);
//
//     ASSERT(Vad->EndingVpn >= Vad->StartingVpn);
//     MiInsertVad((PMMVAD)Vad, &Process->VadRoot);
//
//     /* Check if this is uncached */
//     if (CacheAttribute != MiCached)
//     {
//         /* Flush all caches */
//         KeFlushEntireTb(TRUE, TRUE);
//         KeInvalidateAllCaches();
//     }
//
//     PointerPte = MiAddressToPte(BaseAddress);
//     while (NumberOfPages != 0 &&
//            *MdlPages != LIST_HEAD)
//     {
//         PointerPde = MiPteToPde(PointerPte);
//         MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);
//         ASSERT(PointerPte->u.Hard.Valid == 0);
//
//         /* Add a PDE reference for each page */
//         MiIncrementPageTableReferences(BaseAddress);
//
//         /* Set up our basic user PTE */
//         MI_MAKE_HARDWARE_PTE_USER(&TempPte,
//                                   PointerPte,
//                                   MM_READWRITE,
//                                   *MdlPages);
//
//         EffectiveCacheAttribute = CacheAttribute;
//
//         /* We need to respect the PFN's caching information in some cases */
//         Pfn2 = MiGetPfnEntry(*MdlPages);
//         if (Pfn2 != NULL)
//         {
//             ASSERT(Pfn2->u3.e2.ReferenceCount != 0);
//
//             switch (Pfn2->u3.e1.CacheAttribute)
//             {
//                 case MiNonCached:
//                     if (CacheAttribute != MiNonCached)
//                     {
//                         MiCacheOverride[1]++;
//                         EffectiveCacheAttribute = MiNonCached;
//                     }
//                     break;
//
//                 case MiCached:
//                     if (CacheAttribute != MiCached)
//                     {
//                         MiCacheOverride[0]++;
//                         EffectiveCacheAttribute = MiCached;
//                     }
//                     break;
//
//                 case MiWriteCombined:
//                     if (CacheAttribute != MiWriteCombined)
//                     {
//                         MiCacheOverride[2]++;
//                         EffectiveCacheAttribute = MiWriteCombined;
//                     }
//                     break;
//
//                 default:
//                     /* We don't support AWE magic (MiNotMapped) */
//                     DPRINT1("FIXME: MiNotMapped is not supported\n");
//                     ASSERT(FALSE);
//                     break;
//             }
//         }
//
//         /* Configure caching */
//         switch (EffectiveCacheAttribute)
//         {
//             case MiNonCached:
//                 MI_PAGE_DISABLE_CACHE(&TempPte);
//                 MI_PAGE_WRITE_THROUGH(&TempPte);
//                 break;
//             case MiCached:
//                 break;
//             case MiWriteCombined:
//                 MI_PAGE_DISABLE_CACHE(&TempPte);
//                 MI_PAGE_WRITE_COMBINED(&TempPte);
//                 break;
//             default:
//                 ASSERT(FALSE);
//                 break;
//         }
//
//         /* Make the page valid */
//         MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//         /* Acquire a share count */
//         Pfn1 = MI_PFN_ELEMENT(PointerPde->u.Hard.PageFrameNumber);
//         DPRINT("Incrementing %p from %p\n", Pfn1, _ReturnAddress());
//         OldIrql = MiAcquirePfnLock();
//         Pfn1->u2.ShareCount++;
//         MiReleasePfnLock(OldIrql);
//
//         /* Next page */
//         MdlPages++;
//         PointerPte++;
//         NumberOfPages--;
//         BaseAddress = (PVOID)((ULONG_PTR)BaseAddress + PAGE_SIZE);
//     }
//
//     MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//     ASSERT(AddressSpaceLocked);
//     MmUnlockAddressSpace(&Process->Vm);
//
//     ASSERT(StartingVa != 0);
//     return (PVOID)((ULONG_PTR)StartingVa + MmGetMdlByteOffset(Mdl));
//
// Error:
//     if (AddressSpaceLocked)
//     {
//         MmUnlockAddressSpace(&Process->Vm);
//     }
//     if (Vad != NULL)
//     {
//         ExFreePoolWithTag(Vad, 'ldaV');
//         PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     }
//     ExRaiseStatus(Status);
// }
//
// static
// VOID
// NTAPI
// MiUnmapLockedPagesInUserSpace(
//     _In_ PVOID BaseAddress,
//     _In_ PMDL Mdl)
// {
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     PMMVAD Vad;
//     PMMPTE PointerPte;
//     PMMPDE PointerPde;
//     KIRQL OldIrql;
//     ULONG NumberOfPages;
//     PPFN_NUMBER MdlPages;
//     PFN_NUMBER PageTablePage;
//
//     DPRINT("MiUnmapLockedPagesInUserSpace(%p, %p)\n", BaseAddress, Mdl);
//
//     NumberOfPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(MmGetMdlVirtualAddress(Mdl),
//                                                    MmGetMdlByteCount(Mdl));
//     ASSERT(NumberOfPages != 0);
//     MdlPages = MmGetMdlPfnArray(Mdl);
//
//     /* Find the VAD */
//     MmLockAddressSpace(&Process->Vm);
//     Vad = MiLocateAddress(BaseAddress);
//     if (!Vad ||
//         Vad->u.VadFlags.VadType() != VadDevicePhysicalMemory)
//     {
//         DPRINT1("MiUnmapLockedPagesInUserSpace invalid for %p\n", BaseAddress);
//         MmUnlockAddressSpace(&Process->Vm);
//         return;
//     }
//
//     MiLockProcessWorkingSetUnsafe(Process, Thread);
//
//     /* Remove it from the process VAD tree */
//     ASSERT(Process->VadRoot.NumberGenericTableElements() >= 1);
//     MiRemoveNode((PMMADDRESS_NODE)Vad, &Process->VadRoot);
//     PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//
//     /* MiRemoveNode should have removed us if we were the hint */
//     ASSERT(Process->VadRoot.NodeHint != Vad);
//
//     PointerPte = MiAddressToPte(BaseAddress);
//     OldIrql = MiAcquirePfnLock();
//     while (NumberOfPages != 0 &&
//            *MdlPages != LIST_HEAD)
//     {
//         ASSERT(MiAddressToPte(PointerPte)->u.Hard.Valid == 1);
//         ASSERT(PointerPte->u.Hard.Valid == 1);
//
//         /* Invalidate it */
//         MI_ERASE_PTE(PointerPte);
//
//         /* We invalidated this PTE, so dereference the PDE */
//         PointerPde = MiAddressToPde(BaseAddress);
//         PageTablePage = PointerPde->u.Hard.PageFrameNumber;
//         MiDecrementShareCount(MiGetPfnEntry(PageTablePage), PageTablePage);
//
//         if (MiDecrementPageTableReferences(BaseAddress) == 0)
//         {
//             ASSERT(MiIsPteOnPdeBoundary(PointerPte + 1) || (NumberOfPages == 1));
//             MiDeletePde(PointerPde, Process);
//         }
//
//         /* Next page */
//         PointerPte++;
//         NumberOfPages--;
//         BaseAddress = (PVOID)((ULONG_PTR)BaseAddress + PAGE_SIZE);
//         MdlPages++;
//     }
//
//     KeFlushProcessTb();
//     MiReleasePfnLock(OldIrql);
//     MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//     MmUnlockAddressSpace(&Process->Vm);
//     ExFreePoolWithTag(Vad, 'ldaV');
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// PMDL
// NTAPI
// MmCreateMdl(IN PMDL Mdl,
//             IN PVOID Base,
//             IN SIZE_T Length)
// {
//     SIZE_T Size;
//
//     //
//     // Check if we don't have an MDL built
//     //
//     if (!Mdl)
//     {
//         //
//         // Calculate the size we'll need  and allocate the MDL
//         //
//         Size = MmSizeOfMdl(Base, Length);
//         Mdl = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_MDL);
//         if (!Mdl) return NULL;
//     }
//
//     //
//     // Initialize it
//     //
//     MmInitializeMdl(Mdl, Base, Length);
//     return Mdl;
// }
//
// /*
//  * @implemented
//  */
// SIZE_T
// NTAPI
// MmSizeOfMdl(IN PVOID Base,
//             IN SIZE_T Length)
// {
//     //
//     // Return the MDL size
//     //
//     return sizeof(MDL) +
//            (ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Length) * sizeof(PFN_NUMBER));
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmBuildMdlForNonPagedPool(IN PMDL Mdl)
// {
//     PPFN_NUMBER MdlPages, EndPage;
//     PFN_NUMBER Pfn, PageCount;
//     PVOID Base;
//     PMMPTE PointerPte;
//
//     //
//     // Sanity checks
//     //
//     ASSERT(Mdl->ByteCount != 0);
//     ASSERT((Mdl->MdlFlags & (MDL_PAGES_LOCKED |
//                              MDL_MAPPED_TO_SYSTEM_VA |
//                              MDL_SOURCE_IS_NONPAGED_POOL |
//                              MDL_PARTIAL)) == 0);
//
//     //
//     // We know the MDL isn't associated to a process now
//     //
//     Mdl->Process = NULL;
//
//     //
//     // Get page and VA information
//     //
//     MdlPages = (PPFN_NUMBER)(Mdl + 1);
//     Base = Mdl->StartVa;
//
//     //
//     // Set the system address and now get the page count
//     //
//     Mdl->MappedSystemVa = (PVOID)((ULONG_PTR)Base + Mdl->ByteOffset);
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Mdl->MappedSystemVa,
//                                                Mdl->ByteCount);
//     ASSERT(PageCount != 0);
//     EndPage = MdlPages + PageCount;
//
//     //
//     // Loop the PTEs
//     //
//     PointerPte = MiAddressToPte(Base);
//     do
//     {
//         //
//         // Write the PFN
//         //
//         Pfn = PFN_FROM_PTE(PointerPte++);
//         *MdlPages++ = Pfn;
//     } while (MdlPages < EndPage);
//
//     //
//     // Set the nonpaged pool flag
//     //
//     Mdl->MdlFlags |= MDL_SOURCE_IS_NONPAGED_POOL;
//
//     //
//     // Check if this is an I/O mapping
//     //
//     if (!MiGetPfnEntry(Pfn)) Mdl->MdlFlags |= MDL_IO_SPACE;
// }
//
// /*
//  * @implemented
//  */
// PMDL
// NTAPI
// MmAllocatePagesForMdl(IN PHYSICAL_ADDRESS LowAddress,
//                       IN PHYSICAL_ADDRESS HighAddress,
//                       IN PHYSICAL_ADDRESS SkipBytes,
//                       IN SIZE_T TotalBytes)
// {
//     //
//     // Call the internal routine
//     //
//     return MiAllocatePagesForMdl(LowAddress,
//                                  HighAddress,
//                                  SkipBytes,
//                                  TotalBytes,
//                                  MiNotMapped,
//                                  0);
// }
//
// /*
//  * @implemented
//  */
// PMDL
// NTAPI
// MmAllocatePagesForMdlEx(IN PHYSICAL_ADDRESS LowAddress,
//                         IN PHYSICAL_ADDRESS HighAddress,
//                         IN PHYSICAL_ADDRESS SkipBytes,
//                         IN SIZE_T TotalBytes,
//                         IN MEMORY_CACHING_TYPE CacheType,
//                         IN ULONG Flags)
// {
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//
//     //
//     // Check for invalid cache type
//     //
//     if (CacheType > MmWriteCombined)
//     {
//         //
//         // Normalize to default
//         //
//         CacheAttribute = MiNotMapped;
//     }
//     else
//     {
//         //
//         // Convert to internal caching attribute
//         //
//         CacheAttribute = MiPlatformCacheAttributes[FALSE][CacheType];
//     }
//
//     //
//     // Only these flags are allowed
//     //
//     if (Flags & ~(MM_DONT_ZERO_ALLOCATION | MM_ALLOCATE_FROM_LOCAL_NODE_ONLY))
//     {
//         //
//         // Silently fail
//         //
//         return NULL;
//     }
//
//     //
//     // Call the internal routine
//     //
//     return MiAllocatePagesForMdl(LowAddress,
//                                  HighAddress,
//                                  SkipBytes,
//                                  TotalBytes,
//                                  CacheAttribute,
//                                  Flags);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmFreePagesFromMdl(IN PMDL Mdl)
// {
//     PVOID Base;
//     PPFN_NUMBER Pages;
//     LONG NumberOfPages;
//     PMMPFN Pfn1;
//     KIRQL OldIrql;
//     DPRINT("Freeing MDL: %p\n", Mdl);
//
//     //
//     // Sanity checks
//     //
//     ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//     ASSERT((Mdl->MdlFlags & MDL_IO_SPACE) == 0);
//     ASSERT(((ULONG_PTR)Mdl->StartVa & (PAGE_SIZE - 1)) == 0);
//
//     //
//     // Get address and page information
//     //
//     Base = (PVOID)((ULONG_PTR)Mdl->StartVa + Mdl->ByteOffset);
//     NumberOfPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Mdl->ByteCount);
//
//     //
//     // Acquire PFN lock
//     //
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // Loop all the MDL pages
//     //
//     Pages = (PPFN_NUMBER)(Mdl + 1);
//     do
//     {
//         //
//         // Reached the last page
//         //
//         if (*Pages == LIST_HEAD) break;
//
//         //
//         // Get the page entry
//         //
//         Pfn1 = MiGetPfnEntry(*Pages);
//         ASSERT(Pfn1);
//         ASSERT(Pfn1->u2.ShareCount == 1);
//         ASSERT(MI_IS_PFN_DELETED(Pfn1) == TRUE);
//         if (Pfn1->u4.PteFrame != 0x1FFEDCB)
//         {
//             /* Corrupted PFN entry or invalid free */
//             KeBugCheckEx(MEMORY_MANAGEMENT, 0x1236, (ULONG_PTR)Mdl, (ULONG_PTR)Pages, *Pages);
//         }
//
//         //
//         // Clear it
//         //
//         Pfn1->u3.e1.StartOfAllocation = 0;
//         Pfn1->u3.e1.EndOfAllocation = 0;
//         Pfn1->u3.e1.PageLocation = StandbyPageList;
//         Pfn1->u2.ShareCount = 0;
//
//         //
//         // Dereference it
//         //
//         ASSERT(Pfn1->u3.e2.ReferenceCount != 0);
//         if (Pfn1->u3.e2.ReferenceCount != 1)
//         {
//             /* Just take off one reference */
//             InterlockedDecrement16((PSHORT)&Pfn1->u3.e2.ReferenceCount);
//         }
//         else
//         {
//             /* We'll be nuking the whole page */
//             MiDecrementReferenceCount(Pfn1, *Pages);
//         }
//
//         //
//         // Clear this page and move on
//         //
//         *Pages++ = LIST_HEAD;
//     } while (--NumberOfPages != 0);
//
//     //
//     // Release the lock
//     //
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // Remove the pages locked flag
//     //
//     Mdl->MdlFlags &= ~MDL_PAGES_LOCKED;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmMapLockedPagesSpecifyCache(IN PMDL Mdl,
//                              IN KPROCESSOR_MODE AccessMode,
//                              IN MEMORY_CACHING_TYPE CacheType,
//                              IN PVOID BaseAddress,
//                              IN ULONG BugCheckOnFailure,
//                              IN ULONG Priority) // MM_PAGE_PRIORITY
// {
//     PVOID Base;
//     PPFN_NUMBER MdlPages, LastPage;
//     PFN_COUNT PageCount;
//     BOOLEAN IsIoMapping;
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     //
//     // Sanity check
//     //
//     ASSERT(Mdl->ByteCount != 0);
//
//     //
//     // Get the base
//     //
//     Base = (PVOID)((ULONG_PTR)Mdl->StartVa + Mdl->ByteOffset);
//
//     //
//     // Handle kernel case first
//     //
//     if (AccessMode == KernelMode)
//     {
//         //
//         // Get the list of pages and count
//         //
//         MdlPages = (PPFN_NUMBER)(Mdl + 1);
//         PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Mdl->ByteCount);
//         LastPage = MdlPages + PageCount;
//
//         //
//         // Sanity checks
//         //
//         ASSERT((Mdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA |
//                                  MDL_SOURCE_IS_NONPAGED_POOL |
//                                  MDL_PARTIAL_HAS_BEEN_MAPPED)) == 0);
//         ASSERT((Mdl->MdlFlags & (MDL_PAGES_LOCKED | MDL_PARTIAL)) != 0);
//
//         //
//         // Get the correct cache type
//         //
//         IsIoMapping = (Mdl->MdlFlags & MDL_IO_SPACE) != 0;
//         CacheAttribute = MiPlatformCacheAttributes[IsIoMapping][CacheType];
//
//         //
//         // Reserve the PTEs
//         //
//         PointerPte = MiReserveSystemPtes(PageCount, SystemPteSpace);
//         if (!PointerPte)
//         {
//             //
//             // If it can fail, return NULL
//             //
//             if (Mdl->MdlFlags & MDL_MAPPING_CAN_FAIL) return NULL;
//
//             //
//             // Should we bugcheck?
//             //
//             if (!BugCheckOnFailure) return NULL;
//
//             //
//             // Yes, crash the system
//             //
//             KeBugCheckEx(NO_MORE_SYSTEM_PTES, 0, PageCount, 0, 0);
//         }
//
//         //
//         // Get the mapped address
//         //
//         Base = (PVOID)((ULONG_PTR)MiPteToAddress(PointerPte) + Mdl->ByteOffset);
//
//         //
//         // Get the template
//         //
//         TempPte = ValidKernelPte;
//         switch (CacheAttribute)
//         {
//             case MiNonCached:
//
//                 //
//                 // Disable caching
//                 //
//                 MI_PAGE_DISABLE_CACHE(&TempPte);
//                 MI_PAGE_WRITE_THROUGH(&TempPte);
//                 break;
//
//             case MiWriteCombined:
//
//                 //
//                 // Enable write combining
//                 //
//                 MI_PAGE_DISABLE_CACHE(&TempPte);
//                 MI_PAGE_WRITE_COMBINED(&TempPte);
//                 break;
//
//             default:
//                 //
//                 // Nothing to do
//                 //
//                 break;
//         }
//
//         //
//         // Loop all PTEs
//         //
//         do
//         {
//             //
//             // We're done here
//             //
//             if (*MdlPages == LIST_HEAD) break;
//
//             //
//             // Write the PTE
//             //
//             TempPte.set_page_frame_number((*MdlPages) as PFN_NUMBER);
//             MI_WRITE_VALID_PTE(PointerPte++, TempPte);
//         } while (++MdlPages < LastPage);
//
//         //
//         // Mark it as mapped
//         //
//         ASSERT((Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA) == 0);
//         Mdl->MappedSystemVa = Base;
//         Mdl->MdlFlags |= MDL_MAPPED_TO_SYSTEM_VA;
//
//         //
//         // Check if it was partial
//         //
//         if (Mdl->MdlFlags & MDL_PARTIAL)
//         {
//             //
//             // Write the appropriate flag here too
//             //
//             Mdl->MdlFlags |= MDL_PARTIAL_HAS_BEEN_MAPPED;
//         }
//
//         //
//         // Return the mapped address
//         //
//         return Base;
//     }
//
//     return MiMapLockedPagesInUserSpace(Mdl, Base, CacheType, BaseAddress);
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmMapLockedPages(IN PMDL Mdl,
//                  IN KPROCESSOR_MODE AccessMode)
// {
//     //
//     // Call the extended version
//     //
//     return MmMapLockedPagesSpecifyCache(Mdl,
//                                         AccessMode,
//                                         MmCached,
//                                         NULL,
//                                         TRUE,
//                                         HighPagePriority);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmUnmapLockedPages(IN PVOID BaseAddress,
//                    IN PMDL Mdl)
// {
//     PVOID Base;
//     PFN_COUNT PageCount, ExtraPageCount;
//     PPFN_NUMBER MdlPages;
//     PMMPTE PointerPte;
//
//     //
//     // Sanity check
//     //
//     ASSERT(Mdl->ByteCount != 0);
//
//     //
//     // Check if this is a kernel request
//     //
//     if (BaseAddress > MM_HIGHEST_USER_ADDRESS)
//     {
//         //
//         // Get base and count information
//         //
//         Base = (PVOID)((ULONG_PTR)Mdl->StartVa + Mdl->ByteOffset);
//         PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Mdl->ByteCount);
//
//         //
//         // Sanity checks
//         //
//         ASSERT((Mdl->MdlFlags & MDL_PARENT_MAPPED_SYSTEM_VA) == 0);
//         ASSERT(PageCount != 0);
//         ASSERT(Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA);
//
//         //
//         // Get the PTE
//         //
//         PointerPte = MiAddressToPte(BaseAddress);
//
//         //
//         // This should be a resident system PTE
//         //
//         ASSERT(PointerPte >= MmSystemPtesStart[SystemPteSpace]);
//         ASSERT(PointerPte <= MmSystemPtesEnd[SystemPteSpace]);
//         ASSERT(PointerPte->u.Hard.Valid == 1);
//
//         //
//         // Check if the caller wants us to free advanced pages
//         //
//         if (Mdl->MdlFlags & MDL_FREE_EXTRA_PTES)
//         {
//             //
//             // Get the MDL page array
//             //
//             MdlPages = MmGetMdlPfnArray(Mdl);
//
//             /* Number of extra pages stored after the PFN array */
//             ExtraPageCount = (PFN_COUNT)*(MdlPages + PageCount);
//
//             //
//             // Do the math
//             //
//             PageCount += ExtraPageCount;
//             PointerPte -= ExtraPageCount;
//             ASSERT(PointerPte >= MmSystemPtesStart[SystemPteSpace]);
//             ASSERT(PointerPte <= MmSystemPtesEnd[SystemPteSpace]);
//
//             //
//             // Get the new base address
//             //
//             BaseAddress = (PVOID)((ULONG_PTR)BaseAddress -
//                                   (ExtraPageCount << PAGE_SHIFT));
//         }
//
//         //
//         // Remove flags
//         //
//         Mdl->MdlFlags &= ~(MDL_MAPPED_TO_SYSTEM_VA |
//                            MDL_PARTIAL_HAS_BEEN_MAPPED |
//                            MDL_FREE_EXTRA_PTES);
//
//         //
//         // Release the system PTEs
//         //
//         MiReleaseSystemPtes(PointerPte, PageCount, SystemPteSpace);
//     }
//     else
//     {
//         MiUnmapLockedPagesInUserSpace(BaseAddress, Mdl);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmProbeAndLockPages(IN PMDL Mdl,
//                     IN KPROCESSOR_MODE AccessMode,
//                     IN LOCK_OPERATION Operation)
// {
//     PPFN_NUMBER MdlPages;
//     PVOID Base, Address, LastAddress, StartAddress;
//     ULONG LockPages, TotalPages;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PEPROCESS CurrentProcess;
//     NTSTATUS ProbeStatus;
//     PMMPTE PointerPte, LastPte;
//     PMMPDE PointerPde;
// #if (_MI_PAGING_LEVELS >= 3)
//     PMMPDE PointerPpe;
// #endif
// #if (_MI_PAGING_LEVELS == 4)
//     PMMPDE PointerPxe;
// #endif
//     PFN_NUMBER PageFrameIndex;
//     BOOLEAN UsePfnLock;
//     KIRQL OldIrql;
//     PMMPFN Pfn1;
//     DPRINT("Probing MDL: %p\n", Mdl);
//
//     //
//     // Sanity checks
//     //
//     ASSERT(Mdl->ByteCount != 0);
//     ASSERT(((ULONG)Mdl->ByteOffset & ~(PAGE_SIZE - 1)) == 0);
//     ASSERT(((ULONG_PTR)Mdl->StartVa & (PAGE_SIZE - 1)) == 0);
//     ASSERT((Mdl->MdlFlags & (MDL_PAGES_LOCKED |
//                              MDL_MAPPED_TO_SYSTEM_VA |
//                              MDL_SOURCE_IS_NONPAGED_POOL |
//                              MDL_PARTIAL |
//                              MDL_IO_SPACE)) == 0);
//
//     //
//     // Get page and base information
//     //
//     MdlPages = (PPFN_NUMBER)(Mdl + 1);
//     Base = Mdl->StartVa;
//
//     //
//     // Get the addresses and how many pages we span (and need to lock)
//     //
//     Address = (PVOID)((ULONG_PTR)Base + Mdl->ByteOffset);
//     LastAddress = (PVOID)((ULONG_PTR)Address + Mdl->ByteCount);
//     LockPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Address, Mdl->ByteCount);
//     ASSERT(LockPages != 0);
//
//     /* Block invalid access */
//     if ((AccessMode != KernelMode) &&
//         ((LastAddress > (PVOID)MM_USER_PROBE_ADDRESS) || (Address >= LastAddress)))
//     {
//         /* Caller should be in SEH, raise the error */
//         *MdlPages = LIST_HEAD;
//         ExRaiseStatus(STATUS_ACCESS_VIOLATION);
//     }
//
//     //
//     // Get the process
//     //
//     if (Address <= MM_HIGHEST_USER_ADDRESS)
//     {
//         //
//         // Get the process
//         //
//         CurrentProcess = PsGetCurrentProcess();
//     }
//     else
//     {
//         //
//         // No process
//         //
//         CurrentProcess = NULL;
//     }
//
//     //
//     // Save the number of pages we'll have to lock, and the start address
//     //
//     TotalPages = LockPages;
//     StartAddress = Address;
//
//     /* Large pages not supported */
//     ASSERT(!MI_IS_PHYSICAL_ADDRESS(Address));
//
//     //
//     // Now probe them
//     //
//     ProbeStatus = STATUS_SUCCESS;
//     _SEH2_TRY
//     {
//         //
//         // Enter probe loop
//         //
//         do
//         {
//             //
//             // Assume failure
//             //
//             *MdlPages = LIST_HEAD;
//
//             //
//             // Read
//             //
//             *(volatile CHAR*)Address;
//
//             //
//             // Check if this is write access (only probe for user-mode)
//             //
//             if ((Operation != IoReadAccess) &&
//                 (Address <= MM_HIGHEST_USER_ADDRESS))
//             {
//                 //
//                 // Probe for write too
//                 //
//                 ProbeForWriteChar(Address);
//             }
//
//             //
//             // Next address...
//             //
//             Address = PAGE_ALIGN((ULONG_PTR)Address + PAGE_SIZE);
//
//             //
//             // Next page...
//             //
//             LockPages--;
//             MdlPages++;
//         } while (Address < LastAddress);
//
//         //
//         // Reset back to the original page
//         //
//         ASSERT(LockPages == 0);
//         MdlPages = (PPFN_NUMBER)(Mdl + 1);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         //
//         // Oops :(
//         //
//         ProbeStatus = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     //
//     // So how did that go?
//     //
//     if (ProbeStatus != STATUS_SUCCESS)
//     {
//         //
//         // Fail
//         //
//         DPRINT1("MDL PROBE FAILED!\n");
//         Mdl->Process = NULL;
//         ExRaiseStatus(ProbeStatus);
//     }
//
//     //
//     // Get the PTE and PDE
//     //
//     PointerPte = MiAddressToPte(StartAddress);
//     PointerPde = MiAddressToPde(StartAddress);
// #if (_MI_PAGING_LEVELS >= 3)
//     PointerPpe = MiAddressToPpe(StartAddress);
// #endif
// #if (_MI_PAGING_LEVELS == 4)
//     PointerPxe = MiAddressToPxe(StartAddress);
// #endif
//
//     //
//     // Sanity check
//     //
//     ASSERT(MdlPages == (PPFN_NUMBER)(Mdl + 1));
//
//     //
//     // Check what kind of operation this is
//     //
//     if (Operation != IoReadAccess)
//     {
//         //
//         // Set the write flag
//         //
//         Mdl->MdlFlags |= MDL_WRITE_OPERATION;
//     }
//     else
//     {
//         //
//         // Remove the write flag
//         //
//         Mdl->MdlFlags &= ~(MDL_WRITE_OPERATION);
//     }
//
//     //
//     // Mark the MDL as locked *now*
//     //
//     Mdl->MdlFlags |= MDL_PAGES_LOCKED;
//
//     //
//     // Check if this came from kernel mode
//     //
//     if (Base > MM_HIGHEST_USER_ADDRESS)
//     {
//         //
//         // We should not have a process
//         //
//         ASSERT(CurrentProcess == NULL);
//         Mdl->Process = NULL;
//
//         //
//         // In kernel mode, we don't need to check for write access
//         //
//         Operation = IoReadAccess;
//
//         //
//         // Use the PFN lock
//         //
//         UsePfnLock = TRUE;
//         OldIrql = MiAcquirePfnLock();
//     }
//     else
//     {
//         //
//         // Sanity checks
//         //
//         ASSERT(TotalPages != 0);
//         ASSERT(CurrentProcess == PsGetCurrentProcess());
//
//         //
//         // Track locked pages
//         //
//         InterlockedExchangeAddSizeT(&CurrentProcess->NumberOfLockedPages,
//                                     TotalPages);
//
//         //
//         // Save the process
//         //
//         Mdl->Process = CurrentProcess;
//
//         /* Lock the process working set */
//         MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//         UsePfnLock = FALSE;
//         OldIrql = MM_NOIRQL;
//     }
//
//     //
//     // Get the last PTE
//     //
//     LastPte = MiAddressToPte((PVOID)((ULONG_PTR)LastAddress - 1));
//
//     //
//     // Loop the pages
//     //
//     do
//     {
//         //
//         // Assume failure and check for non-mapped pages
//         //
//         *MdlPages = LIST_HEAD;
//         while (
// #if (_MI_PAGING_LEVELS == 4)
//                (PointerPxe->u.Hard.Valid == 0) ||
// #endif
// #if (_MI_PAGING_LEVELS >= 3)
//                (PointerPpe->u.Hard.Valid == 0) ||
// #endif
//                (PointerPde->u.Hard.Valid == 0) ||
//                (PointerPte->u.Hard.Valid == 0))
//         {
//             //
//             // What kind of lock were we using?
//             //
//             if (UsePfnLock)
//             {
//                 //
//                 // Release PFN lock
//                 //
//                 MiReleasePfnLock(OldIrql);
//             }
//             else
//             {
//                 /* Release process working set */
//                 MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//             }
//
//             //
//             // Access the page
//             //
//             Address = MiPteToAddress(PointerPte);
//
//             //HACK: Pass a placeholder TrapInformation so the fault handler knows we're unlocked
//             Status = MmAccessFault(FALSE, Address, KernelMode, (PVOID)(ULONG_PTR)0xBADBADA3BADBADA3ULL);
//             if (!NT_SUCCESS(Status))
//             {
//                 //
//                 // Fail
//                 //
//                 DPRINT1("Access fault failed\n");
//                 goto Cleanup;
//             }
//
//             //
//             // What lock should we use?
//             //
//             if (UsePfnLock)
//             {
//                 //
//                 // Grab the PFN lock
//                 //
//                 OldIrql = MiAcquirePfnLock();
//             }
//             else
//             {
//                 /* Lock the process working set */
//                 MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//             }
//         }
//
//         //
//         // Check if this was a write or modify
//         //
//         if (Operation != IoReadAccess)
//         {
//             //
//             // Check if the PTE is not writable
//             //
//             if (MI_IS_PAGE_WRITEABLE(PointerPte) == FALSE)
//             {
//                 //
//                 // Check if it's copy on write
//                 //
//                 if (MI_IS_PAGE_COPY_ON_WRITE(PointerPte))
//                 {
//                     //
//                     // Get the base address and allow a change for user-mode
//                     //
//                     Address = MiPteToAddress(PointerPte);
//                     if (Address <= MM_HIGHEST_USER_ADDRESS)
//                     {
//                         //
//                         // What kind of lock were we using?
//                         //
//                         if (UsePfnLock)
//                         {
//                             //
//                             // Release PFN lock
//                             //
//                             MiReleasePfnLock(OldIrql);
//                         }
//                         else
//                         {
//                             /* Release process working set */
//                             MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//                         }
//
//                         //
//                         // Access the page
//                         //
//
//                         //HACK: Pass a placeholder TrapInformation so the fault handler knows we're unlocked
//                         Status = MmAccessFault(TRUE, Address, KernelMode, (PVOID)(ULONG_PTR)0xBADBADA3BADBADA3ULL);
//                         if (!NT_SUCCESS(Status))
//                         {
//                             //
//                             // Fail
//                             //
//                             DPRINT1("Access fault failed\n");
//                             goto Cleanup;
//                         }
//
//                         //
//                         // Re-acquire the lock
//                         //
//                         if (UsePfnLock)
//                         {
//                             //
//                             // Grab the PFN lock
//                             //
//                             OldIrql = MiAcquirePfnLock();
//                         }
//                         else
//                         {
//                             /* Lock the process working set */
//                             MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//                         }
//
//                         //
//                         // Start over
//                         //
//                         continue;
//                     }
//                 }
//
//                 //
//                 // Fail, since we won't allow this
//                 //
//                 Status = STATUS_ACCESS_VIOLATION;
//                 goto CleanupWithLock;
//             }
//         }
//
//         //
//         // Grab the PFN
//         //
//         PageFrameIndex = PFN_FROM_PTE(PointerPte);
//         Pfn1 = MiGetPfnEntry(PageFrameIndex);
//         if (Pfn1)
//         {
//             /* Either this is for kernel-mode, or the working set is held */
//             ASSERT((CurrentProcess == NULL) || (UsePfnLock == FALSE));
//
//             /* No Physical VADs supported yet */
//             if (CurrentProcess) ASSERT(CurrentProcess->PhysicalVadRoot == NULL);
//
//             /* This address should already exist and be fully valid */
//             MiReferenceProbedPageAndBumpLockCount(Pfn1);
//         }
//         else
//         {
//             //
//             // For I/O addresses, just remember this
//             //
//             Mdl->MdlFlags |= MDL_IO_SPACE;
//         }
//
//         //
//         // Write the page and move on
//         //
//         *MdlPages++ = PageFrameIndex;
//         PointerPte++;
//
//         /* Check if we're on a PDE boundary */
//         if (MiIsPteOnPdeBoundary(PointerPte)) PointerPde++;
// #if (_MI_PAGING_LEVELS >= 3)
//         if (MiIsPteOnPpeBoundary(PointerPte)) PointerPpe++;
// #endif
// #if (_MI_PAGING_LEVELS == 4)
//         if (MiIsPteOnPxeBoundary(PointerPte)) PointerPxe++;
// #endif
//
//     } while (PointerPte <= LastPte);
//
//     //
//     // What kind of lock were we using?
//     //
//     if (UsePfnLock)
//     {
//         //
//         // Release PFN lock
//         //
//         MiReleasePfnLock(OldIrql);
//     }
//     else
//     {
//         /* Release process working set */
//         MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//     }
//
//     //
//     // Sanity check
//     //
//     ASSERT((Mdl->MdlFlags & MDL_DESCRIBES_AWE) == 0);
//     return;
//
// CleanupWithLock:
//     //
//     // This is the failure path
//     //
//     ASSERT(!NT_SUCCESS(Status));
//
//     //
//     // What kind of lock were we using?
//     //
//     if (UsePfnLock)
//     {
//         //
//         // Release PFN lock
//         //
//         MiReleasePfnLock(OldIrql);
//     }
//     else
//     {
//         /* Release process working set */
//         MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread());
//     }
// Cleanup:
//     //
//     // Pages must be locked so MmUnlock can work
//     //
//     ASSERT(Mdl->MdlFlags & MDL_PAGES_LOCKED);
//     MmUnlockPages(Mdl);
//
//     //
//     // Raise the error
//     //
//     ExRaiseStatus(Status);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmUnlockPages(IN PMDL Mdl)
// {
//     PPFN_NUMBER MdlPages, LastPage;
//     PEPROCESS Process;
//     PVOID Base;
//     ULONG Flags, PageCount;
//     KIRQL OldIrql;
//     PMMPFN Pfn1;
//     DPRINT("Unlocking MDL: %p\n", Mdl);
//
//     //
//     // Sanity checks
//     //
//     ASSERT((Mdl->MdlFlags & MDL_PAGES_LOCKED) != 0);
//     ASSERT((Mdl->MdlFlags & MDL_SOURCE_IS_NONPAGED_POOL) == 0);
//     ASSERT((Mdl->MdlFlags & MDL_PARTIAL) == 0);
//     ASSERT(Mdl->ByteCount != 0);
//
//     //
//     // Get the process associated and capture the flags which are volatile
//     //
//     Process = Mdl->Process;
//     Flags = Mdl->MdlFlags;
//
//     //
//     // Automagically undo any calls to MmGetSystemAddressForMdl's for this MDL
//     //
//     if (Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA)
//     {
//         //
//         // Unmap the pages from system space
//         //
//         MmUnmapLockedPages(Mdl->MappedSystemVa, Mdl);
//     }
//
//     //
//     // Get the page count
//     //
//     MdlPages = (PPFN_NUMBER)(Mdl + 1);
//     Base = (PVOID)((ULONG_PTR)Mdl->StartVa + Mdl->ByteOffset);
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Mdl->ByteCount);
//     ASSERT(PageCount != 0);
//
//     //
//     // We don't support AWE
//     //
//     if (Flags & MDL_DESCRIBES_AWE) ASSERT(FALSE);
//
//     //
//     // Check if the buffer is mapped I/O space
//     //
//     if (Flags & MDL_IO_SPACE)
//     {
//         //
//         // Acquire PFN lock
//         //
//         OldIrql = MiAcquirePfnLock();
//
//         //
//         // Loop every page
//         //
//         LastPage = MdlPages + PageCount;
//         do
//         {
//             //
//             // Last page, break out
//             //
//             if (*MdlPages == LIST_HEAD) break;
//
//             //
//             // Check if this page is in the PFN database
//             //
//             Pfn1 = MiGetPfnEntry(*MdlPages);
//             if (Pfn1) MiDereferencePfnAndDropLockCount(Pfn1);
//         } while (++MdlPages < LastPage);
//
//         //
//         // Release the lock
//         //
//         MiReleasePfnLock(OldIrql);
//
//         //
//         // Check if we have a process
//         //
//         if (Process)
//         {
//             //
//             // Handle the accounting of locked pages
//             //
//             ASSERT(Process->NumberOfLockedPages > 0);
//             InterlockedExchangeAddSizeT(&Process->NumberOfLockedPages,
//                                         -(LONG_PTR)PageCount);
//         }
//
//         //
//         // We're done
//         //
//         Mdl->MdlFlags &= ~MDL_IO_SPACE;
//         Mdl->MdlFlags &= ~MDL_PAGES_LOCKED;
//         return;
//     }
//
//     //
//     // Check if we have a process
//     //
//     if (Process)
//     {
//         //
//         // Handle the accounting of locked pages
//         //
//         ASSERT(Process->NumberOfLockedPages > 0);
//         InterlockedExchangeAddSizeT(&Process->NumberOfLockedPages,
//                                     -(LONG_PTR)PageCount);
//     }
//
//     //
//     // Loop every page
//     //
//     LastPage = MdlPages + PageCount;
//     do
//     {
//         //
//         // Last page reached
//         //
//         if (*MdlPages == LIST_HEAD)
//         {
//             //
//             // Were there no pages at all?
//             //
//             if (MdlPages == (PPFN_NUMBER)(Mdl + 1))
//             {
//                 //
//                 // We're already done
//                 //
//                 Mdl->MdlFlags &= ~MDL_PAGES_LOCKED;
//                 return;
//             }
//
//             //
//             // Otherwise, stop here
//             //
//             LastPage = MdlPages;
//             break;
//         }
//
//         /* Save the PFN entry instead for the secondary loop */
//         *MdlPages = (PFN_NUMBER)MiGetPfnEntry(*MdlPages);
//         ASSERT(*MdlPages != 0);
//     } while (++MdlPages < LastPage);
//
//     //
//     // Reset pointer
//     //
//     MdlPages = (PPFN_NUMBER)(Mdl + 1);
//
//     //
//     // Now grab the PFN lock for the actual unlock and dereference
//     //
//     OldIrql = MiAcquirePfnLock();
//     do
//     {
//         /* Get the current entry and reference count */
//         Pfn1 = (PMMPFN)*MdlPages;
//         MiDereferencePfnAndDropLockCount(Pfn1);
//     } while (++MdlPages < LastPage);
//
//     //
//     // Release the lock
//     //
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // We're done
//     //
//     Mdl->MdlFlags &= ~MDL_PAGES_LOCKED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// MmAdvanceMdl(IN PMDL Mdl,
//              IN ULONG NumberOfBytes)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmMapLockedPagesWithReservedMapping(
//     _In_ PVOID MappingAddress,
//     _In_ ULONG PoolTag,
//     _In_ PMDL Mdl,
//     _In_ MEMORY_CACHING_TYPE CacheType)
// {
//     PPFN_NUMBER MdlPages, LastPage;
//     PFN_COUNT PageCount;
//     BOOLEAN IsIoMapping;
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     ASSERT(Mdl->ByteCount != 0);
//
//     // Get the list of pages and count
//     MdlPages = MmGetMdlPfnArray(Mdl);
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(MmGetMdlVirtualAddress(Mdl),
//                                                Mdl->ByteCount);
//     LastPage = MdlPages + PageCount;
//
//     // Sanity checks
//     ASSERT((Mdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA |
//                              MDL_SOURCE_IS_NONPAGED_POOL |
//                              MDL_PARTIAL_HAS_BEEN_MAPPED)) == 0);
//     ASSERT((Mdl->MdlFlags & (MDL_PAGES_LOCKED | MDL_PARTIAL)) != 0);
//
//     // Get the correct cache type
//     IsIoMapping = (Mdl->MdlFlags & MDL_IO_SPACE) != 0;
//     CacheAttribute = MiPlatformCacheAttributes[IsIoMapping][CacheType];
//
//     // Get the first PTE we reserved
//     ASSERT(MappingAddress);
//     PointerPte = MiAddressToPte(MappingAddress) - 2;
//     ASSERT(!PointerPte[0].valid() &&
//            !PointerPte[1].valid());
//
//     // Verify that the pool tag matches
//     TempPte.Long = PoolTag;
//     TempPte.set_valid(false);
//     if (PointerPte[1].Long != TempPte.Long)
//     {
//         KeBugCheckEx(SYSTEM_PTE_MISUSE,
//                      PTE_MAPPING_ADDRESS_NOT_OWNED, /* Trying to map an address it does not own */
//                      (ULONG_PTR)MappingAddress,
//                      PoolTag,
//                      PointerPte[1].Long);
//     }
//
//     // We must have a size, and our helper PTEs must be invalid
//     if (PointerPte[0].u.List.NextEntry < 3)
//     {
//         KeBugCheckEx(SYSTEM_PTE_MISUSE,
//                      PTE_MAPPING_ADDRESS_INVALID, /* Trying to map an invalid address */
//                      (ULONG_PTR)MappingAddress,
//                      PoolTag,
//                      (ULONG_PTR)_ReturnAddress());
//     }
//
//     // If the mapping isn't big enough, fail
//     if (PointerPte[0].u.List.NextEntry - 2 < PageCount)
//     {
//         DPRINT1("Reserved mapping too small. Need %Iu pages, have %Iu\n",
//                         PageCount,
//                         PointerPte[0].u.List.NextEntry - 2);
//         return NULL;
//     }
//     // Skip our two helper PTEs
//     PointerPte += 2;
//
//     // Get the template
//     TempPte = ValidKernelPte;
//     switch (CacheAttribute)
//     {
//         case MiNonCached:
//             // Disable caching
//             MI_PAGE_DISABLE_CACHE(&TempPte);
//             MI_PAGE_WRITE_THROUGH(&TempPte);
//             break;
//
//         case MiWriteCombined:
//             // Enable write combining
//             MI_PAGE_DISABLE_CACHE(&TempPte);
//             MI_PAGE_WRITE_COMBINED(&TempPte);
//             break;
//
//         default:
//             // Nothing to do
//             break;
//     }
//
//     // Loop all PTEs
//     for (; (MdlPages < LastPage) && (*MdlPages != LIST_HEAD); ++MdlPages)
//     {
//         // Write the PTE
//         TempPte.set_page_frame_number((*MdlPages) as PFN_NUMBER);
//         MI_WRITE_VALID_PTE(PointerPte++, TempPte);
//     }
//
//     // Mark it as mapped
//     ASSERT((Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA) == 0);
//     Mdl->MappedSystemVa = MappingAddress;
//     Mdl->MdlFlags |= MDL_MAPPED_TO_SYSTEM_VA;
//
//     // Check if it was partial
//     if (Mdl->MdlFlags & MDL_PARTIAL)
//     {
//         // Write the appropriate flag here too
//         Mdl->MdlFlags |= MDL_PARTIAL_HAS_BEEN_MAPPED;
//     }
//
//     // Return the mapped address
//     return (PVOID)((ULONG_PTR)MappingAddress + Mdl->ByteOffset);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmUnmapReservedMapping(
//     _In_ PVOID BaseAddress,
//     _In_ ULONG PoolTag,
//     _In_ PMDL Mdl)
// {
//     PVOID Base;
//     PFN_COUNT PageCount, ExtraPageCount;
//     PPFN_NUMBER MdlPages;
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     // Sanity check
//     ASSERT(Mdl->ByteCount != 0);
//     ASSERT(BaseAddress > MM_HIGHEST_USER_ADDRESS);
//
//     // Get base and count information
//     Base = (PVOID)((ULONG_PTR)Mdl->StartVa + Mdl->ByteOffset);
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Base, Mdl->ByteCount);
//
//     // Sanity checks
//     ASSERT((Mdl->MdlFlags & MDL_PARENT_MAPPED_SYSTEM_VA) == 0);
//     ASSERT(PageCount != 0);
//     ASSERT(Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA);
//
//     // Get the first PTE we reserved
//     PointerPte = MiAddressToPte(BaseAddress) - 2;
//     ASSERT(!PointerPte[0].valid() &&
//            !PointerPte[1].valid());
//
//     // Verify that the pool tag matches
//     TempPte.Long = PoolTag;
//     TempPte.set_valid(false);
//     if (PointerPte[1].Long != TempPte.Long)
//     {
//         KeBugCheckEx(SYSTEM_PTE_MISUSE,
//                      PTE_UNMAPPING_ADDRESS_NOT_OWNED, /* Trying to unmap an address it does not own */
//                      (ULONG_PTR)BaseAddress,
//                      PoolTag,
//                      PointerPte[1].Long);
//     }
//
//     // We must have a size
//     if (PointerPte[0].u.List.NextEntry < 3)
//     {
//         KeBugCheckEx(SYSTEM_PTE_MISUSE,
//                      PTE_MAPPING_ADDRESS_EMPTY, /* Mapping apparently empty */
//                      (ULONG_PTR)BaseAddress,
//                      PoolTag,
//                      (ULONG_PTR)_ReturnAddress());
//     }
//
//     // Skip our two helper PTEs
//     PointerPte += 2;
//
//     // This should be a resident system PTE
//     ASSERT(PointerPte >= MmSystemPtesStart[SystemPteSpace]);
//     ASSERT(PointerPte <= MmSystemPtesEnd[SystemPteSpace]);
//     ASSERT(PointerPte->u.Hard.Valid == 1);
//
//     // TODO: check the MDL range makes sense with regard to the mapping range
//     // TODO: check if any of them are already zero
//     // TODO: check if any outside the MDL range are nonzero
//     // TODO: find out what to do with extra PTEs
//
//     // Check if the caller wants us to free advanced pages
//     if (Mdl->MdlFlags & MDL_FREE_EXTRA_PTES)
//     {
//         // Get the MDL page array
//         MdlPages = MmGetMdlPfnArray(Mdl);
//
//         /* Number of extra pages stored after the PFN array */
//         ExtraPageCount = MdlPages[PageCount];
//
//         // Do the math
//         PageCount += ExtraPageCount;
//         PointerPte -= ExtraPageCount;
//         ASSERT(PointerPte >= MmSystemPtesStart[SystemPteSpace]);
//         ASSERT(PointerPte <= MmSystemPtesEnd[SystemPteSpace]);
//
//         // Get the new base address
//         BaseAddress = (PVOID)((ULONG_PTR)BaseAddress -
//                               (ExtraPageCount << PAGE_SHIFT));
//     }
//
//     // Zero the PTEs
//     RtlZeroMemory(PointerPte, PageCount * sizeof(MMPTE));
//
//     // Flush the TLB
//     KeFlushEntireTb(TRUE, TRUE);
//
//     // Remove flags
//     Mdl->MdlFlags &= ~(MDL_MAPPED_TO_SYSTEM_VA |
//                        MDL_PARTIAL_HAS_BEEN_MAPPED |
//                        MDL_FREE_EXTRA_PTES);
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// MmPrefetchPages(IN ULONG NumberOfLists,
//                 IN PREAD_LIST *ReadLists)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// MmProtectMdlSystemAddress(IN PMDL MemoryDescriptorList,
//                           IN ULONG NewProtect)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /**
//  * @brief
//  * Probes and locks virtual pages in memory for the specified process.
//  *
//  * @param[in,out] MemoryDescriptorList
//  * Memory Descriptor List (MDL) containing the buffer to be probed and locked.
//  *
//  * @param[in] Process
//  * The process for which the buffer should be probed and locked.
//  *
//  * @param[in] AccessMode
//  * Access mode for probing the pages. Can be KernelMode or UserMode.
//  *
//  * @param[in] LockOperation
//  * The type of the probing and locking operation. Can be IoReadAccess, IoWriteAccess or IoModifyAccess.
//  *
//  * @return
//  * Nothing.
//  *
//  * @see MmProbeAndLockPages
//  *
//  * @remarks Must be called at IRQL <= APC_LEVEL
//  */
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// NTAPI
// MmProbeAndLockProcessPages(
//     _Inout_ PMDL MemoryDescriptorList,
//     _In_ PEPROCESS Process,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ LOCK_OPERATION Operation)
// {
//     KAPC_STATE ApcState;
//     BOOLEAN IsAttached = FALSE;
//
//     if (Process != PsGetCurrentProcess())
//     {
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//         IsAttached = TRUE;
//     }
//
//     /* Protect in try/finally to ensure we detach even if MmProbeAndLockPages() throws an exception */
//     _SEH2_TRY
//     {
//         MmProbeAndLockPages(MemoryDescriptorList, AccessMode, Operation);
//     }
//     _SEH2_FINALLY
//     {
//         if (IsAttached)
//             KeUnstackDetachProcess(&ApcState);
//     }
//     _SEH2_END;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// MmProbeAndLockSelectedPages(IN OUT PMDL MemoryDescriptorList,
//                             IN LARGE_INTEGER PageList[],
//                             IN KPROCESSOR_MODE AccessMode,
//                             IN LOCK_OPERATION Operation)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// MmMapMemoryDumpMdl(IN PMDL Mdl)
// {
//     UNIMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
