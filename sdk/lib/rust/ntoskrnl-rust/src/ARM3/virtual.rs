use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
#[allow(unused_imports)]
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
#[allow(unused_imports)]
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
#[allow(unused_imports)]
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use super::helpers::*;

use ntkrnl_sys::mm::PAGE_READONLY;
use ntkrnl_sys::macros::NtCurrentProcess;

#[allow(unused_imports)]
use ntkrnl_sys::ddk::{
    _EPROCESS,
    PEPROCESS,
    PETHREAD,
    POBJECT_TYPE,
    ACCESS_MASK,
    KPROCESSOR_MODE,
    HANDLE,
    ExGetPreviousMode,
    _KAPC_STATE,
    KeFlushCurrentTb,
    KeRosDumpStackFrames,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    MDL,
    MEMORY_BASIC_INFORMATION,
    MiCheckPdeForPagedPool,
    MiDecrementShareCount,
    MiIncrementPageTableReferences,
    MiLocateSubsection,
    MiLockProcessWorkingSetForFault,
    MiLockProcessWorkingSetUnsafe,
    MiLockWorkingSet,
    MiUnlinkPageFromList,
    MiUnlockProcessWorkingSetForFault,
    MiUnlockProcessWorkingSetUnsafe,
    MiUnlockWorkingSet,
    MmAccessFault,
    MmIsAddressValid,
    MmMapLockedPagesSpecifyCache,
    MmQuerySectionView,
    MmUnmapLockedPages,
    MmUnlockPages,
    PEXCEPTION_POINTERS,
    PEXCEPTION_RECORD,
    MiGetNextNode,
    MiLockProcessWorkingSetShared,
    MiUnlockProcessWorkingSetShared,
    PHYSICAL_ADDRESS,
    ObReferenceObjectByHandle,
    _MEMORY_CACHING_TYPE_MmCached,
    PMMVAD, PMMSUPPORT, PMMADDRESS_NODE,
    _OBJECT_HANDLE_INFORMATION,
    PMDL,
    PsGetCurrentThread,
    KeAreAllApcsDisabled,
    ExAcquireRundownProtection,
    ExReleaseRundownProtection,
    _MMVAD,
    _MMVAD_SHORT,
    _MMVAD_LONG,
    MiCheckForConflictingNode,
    TABLE_SEARCH_RESULT,
    MiInsertVad,
    MiInsertVadEx,
    MiRemoveNode,
    MmProtectSectionView,
    _IO_STATUS_BLOCK,
    PIO_STATUS_BLOCK,
    MiQueryMemorySectionName,
    MiMakeProtectionMask,
    PsChargeProcessNonPagedPoolQuota,
    PsReturnProcessNonPagedPoolQuota,
    MiCheckSecuredVad,
    MiLocateAddress,
    ExFreePoolWithTag,
    SeSinglePrivilegeCheck,
    MiReferenceProbedPageAndBumpLockCount,
    MiDereferencePfnAndDropLockCount,
    _MMWSLE,
    PMMWSLE,
    KeAcquireGuardedMutexUnsafe,
    KeReleaseGuardedMutexUnsafe,
};

// Types/functions not in bindgen — define locally
const VadNone: u32 = 0;
const VadAwe: u32 = 5;
const VadDevicePhysicalMemory: u32 = 1;
const VadLargePages: u32 = 3;
const VadImageMap: u32 = 2;
const VadRotatePhysical: u32 = 6;
const VadLargePageSection: u32 = 7;
const TableFoundNode: TABLE_SEARCH_RESULT = 0;
const HighPagePriority: u32 = 2;
const IoReadAccess: u32 = 0;

extern "system" {
    fn MmInitializeMdl(Mdl: PMDL, BaseVa: PVOID, Length: SIZE_T);
    fn ObDereferenceObject(Object: PVOID);
    fn MmMdlPagesAreZero(Mdl: PMDL) -> BOOLEAN;
    fn MiFlushTbAndCapture(
        Vad: PMMVAD, PointerPte: PMMPTE, ProtectionMask: ULONG,
        Pfn1: PMMPFN, CaptureDirtyBit: BOOLEAN,
    );
    static SeLockMemoryPrivilege: ntkrnl_sys::ddk::_LUID;
}

#[repr(C)]
struct _MEMORY_SECTION_NAME {
    pub SectionFileName: ntkrnl_sys::ddk::_UNICODE_STRING,
}

type KeGuardedMutex = ntkrnl_sys::ddk::_KGUARDED_MUTEX;

// --- Additional types/externs/constants for tail portion ---
type MMVAD_LONG = ntkrnl_sys::ddk::_MMVAD_LONG;
type PMEMORY_AREA = ntkrnl_sys::ddk::PMEMORY_AREA;
type PKPROCESS = ntkrnl_sys::ddk::PKPROCESS;
type MEMORY_INFORMATION_CLASS = u32;
#[allow(non_camel_case_types)]
type MEMORY_SECTION_NAME_T = _MEMORY_SECTION_NAME;
type PMMPPE = PMMPTE;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
type PMMPXE = PMMPTE;
type ULONG64 = u64;

const STATUS_NOT_COMMITTED: NTSTATUS = 0xC000002Du32 as i32;
const STATUS_WAS_LOCKED: NTSTATUS = 0x00000015u32 as i32;
const STATUS_NOT_LOCKED: NTSTATUS = 0xC000002Au32 as i32;
const MAP_PROCESS: ULONG = 1;
const MAP_SYSTEM: ULONG = 2;
const VadWriteWatch: u32 = 4;
const MI_MAX_ZERO_BITS: u32 = 21;
const MEM_RESET: ULONG = 0x00080000;
const MEM_PHYSICAL: ULONG = 0x00400000;
const MEM_WRITE_WATCH: ULONG = 0x00200000;
const MEM_LARGE_PAGES: ULONG = 0x20000000;
const MEM_DECOMMIT: ULONG = 0x4000;
const MemoryBasicInformation: u32 = 0;
const MemorySectionName: u32 = 2;

// FORCEINLINE/macro shims — must be extern "C" (cdecl) to match C shim calling convention.
// On arm64/amd64, extern "C" == extern "system". On i386, extern "system" = stdcall which
// would produce @N-decorated symbols that don't match the cdecl C shims.
extern "C" {
    fn MmGetCurrentAddressSpace() -> PMMSUPPORT;
    fn KeInvalidateTlbEntry(Address: PVOID);
    fn ProbeForWriteSize_t(Address: *mut SIZE_T);
    fn ProbeForWriteIoStatusBlock(Address: PIO_STATUS_BLOCK);
    fn KeSweepICache(BaseAddress: PVOID, Length: SIZE_T);
    fn MiLockProcessWorkingSet(Process: PEPROCESS, Thread: PETHREAD);
    fn MiUnlockProcessWorkingSet(Process: PEPROCESS, Thread: PETHREAD);
    fn ExAllocatePoolZero(PoolType: u32, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePool(P: PVOID);
    static MmUserProbeAddress: ULONG_PTR;
}

// ProbeForWriteUlong/ProbeForWritePointer: provided as NTAPI in tier1_rust_shims.c.
// Must use extern "system" (stdcall on i386) to match the NTAPI calling convention.
extern "system" {
    fn ProbeForWritePointer(Address: *mut PVOID);
    fn ProbeForWriteUlong(Address: *mut ULONG);
}

#[inline]
unsafe fn ALIGN_UP_POINTER_BY(ptr: PVOID, align: usize) -> PVOID {
    (((ptr as usize) + align - 1) & !(align - 1)) as PVOID
}

#[inline]
unsafe fn ALIGN_DOWN_POINTER_BY(ptr: PVOID, align: usize) -> PVOID {
    ((ptr as usize) & !(align - 1)) as PVOID
}
// --- End additional items ---

const MI_MAPPED_COPY_PAGES: usize = 14;
const MI_POOL_COPY_BYTES: usize = 512;
const MI_MAX_TRANSFER_SIZE: usize = 64 * 1024;

extern "system" {
    fn ProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    fn ProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    static mut PsProcessType: POBJECT_TYPE;
    fn MmProbeAndLockPages(MemoryDescriptorList: *mut MDL, AccessMode: KPROCESSOR_MODE, Operation: u32);
    static mut MmSectionCommitMutex: KeGuardedMutex;
    static mut MmDecommittedPte: MMPTE;
}

#[inline]
unsafe fn MI_IS_MAPPED_PTE(pte: *const MMPTE) -> bool {
    (*pte).valid() || (*pte).prototype() || (*pte).transition() || (*pte).page_frame_number() != 0
}

#[inline]
unsafe fn MI_GET_PROTOTYPE_PTE_FOR_VPN(Vad: PMMVAD, Vpn: ULONG_PTR) -> PMMPTE {
    (*Vad).FirstPrototypePte.add(Vpn.wrapping_sub((*Vad).StartingVpn as usize)) as PMMPTE
}

#[no_mangle]
pub unsafe extern "system" fn MiCalculatePageCommitment(
    StartingAddress: ULONG_PTR,
    EndingAddress: ULONG_PTR,
    Vad: PMMVAD,
    Process: PEPROCESS,
) -> ULONG {
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let mut OnPdeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPpeBoundary = true;
    #[cfg(target_arch = "x86_64")]
    let mut PointerPxe: PMMPTE;
    #[cfg(target_arch = "x86_64")]
    let mut OnPxeBoundary = true;

    PointerPte = MiAddressToPte(StartingAddress as PVOID);
    LastPte = MiAddressToPte(EndingAddress as PVOID);

    let mut CommittedPages = if (*Vad).u.VadFlags.MemCommit() != 0 {
        BYTES_TO_PAGES(EndingAddress - StartingAddress) as ULONG
    } else {
        0
    };

    while PointerPte <= LastPte {
        #[cfg(target_arch = "x86_64")]
        {
            if OnPxeBoundary {
                PointerPxe = MiPteToPxe(PointerPte);
                if (*PointerPxe).Long == 0 {
                    PointerPxe = PointerPxe.add(1);
                    PointerPte = MiPxeToPte(PointerPxe);
                    continue;
                }
                if !(*PointerPxe).valid() {
                    MiMakeSystemAddressValid(MiPteToPpe(PointerPte) as PVOID, Process);
                }
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            if OnPpeBoundary {
                PointerPpe = MiPteToPpe(PointerPte);
                if (*PointerPpe).Long == 0 {
                    PointerPpe = PointerPpe.add(1);
                    PointerPte = MiPpeToPte(PointerPpe);
                    #[cfg(target_arch = "x86_64")]
                    { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
                    continue;
                }
                if !(*PointerPpe).valid() {
                    MiMakeSystemAddressValid(MiPteToPde(PointerPte) as PVOID, Process);
                }
            }
        }

        if OnPdeBoundary {
            PointerPde = MiPteToPde(PointerPte);
            if (*PointerPde).Long == 0 {
                PointerPde = PointerPde.add(1);
                PointerPte = MiPdeToPte(PointerPde);
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                { OnPpeBoundary = MiIsPteOnPpeBoundary(PointerPte); }
                #[cfg(target_arch = "x86_64")]
                { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
                continue;
            }
            if !(*PointerPde).valid() {
                MiMakeSystemAddressValid(PointerPte as PVOID, Process);
            }
        }

        if (*PointerPte).Long != 0 {
            if !(*PointerPte).valid() &&
               ((*PointerPte).soft_protection() == MM_DECOMMIT) &&
               (!(*PointerPte).prototype() ||
                ((*PointerPte).soft_page_file_high() == MI_PTE_LOOKUP_NEEDED))
            {
                if (*Vad).u.VadFlags.MemCommit() != 0 {
                    CommittedPages -= 1;
                }
            } else if (*Vad).u.VadFlags.MemCommit() == 0 {
                CommittedPages += 1;
            }
        }

        PointerPte = PointerPte.add(1);
        OnPdeBoundary = MiIsPteOnPdeBoundary(PointerPte);
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        { OnPpeBoundary = MiIsPteOnPpeBoundary(PointerPte); }
        #[cfg(target_arch = "x86_64")]
        { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
    }

    CommittedPages
}

#[no_mangle]
pub unsafe extern "system" fn MiMakeSystemAddressValid(
    PageTableVirtualAddress: PVOID,
    CurrentProcess: PEPROCESS,
) -> ULONG {
    let mut Status: NTSTATUS;
    let mut WsShared: BOOLEAN = 0;
    let mut WsSafe: BOOLEAN = 0;
    let mut LockChange = 0u32;
    let CurrentThread = PsGetCurrentThread() as PETHREAD;

    while MmIsAddressValid(PageTableVirtualAddress) == 0 {
        MiUnlockProcessWorkingSetForFault(
            CurrentProcess,
            CurrentThread,
            core::ptr::addr_of_mut!(WsSafe),
            core::ptr::addr_of_mut!(WsShared),
        );

        Status = MmAccessFault(0u32, PageTableVirtualAddress, 0i8, core::ptr::null_mut());
        if Status < 0 {
            KeBugCheckEx(
                0x7A,
                1,
                Status as ULONG_PTR,
                CurrentProcess as ULONG_PTR,
                PageTableVirtualAddress as ULONG_PTR,
            );
        }

        MiLockProcessWorkingSetForFault(
            CurrentProcess,
            CurrentThread,
            WsSafe,
            WsShared,
        );

        LockChange = 1;
    }

    LockChange
}

#[no_mangle]
pub unsafe extern "system" fn MiMakeSystemAddressValidPfn(
    VirtualAddress: PVOID,
    mut OldIrql: KIRQL,
) -> ULONG {
    let mut Status: NTSTATUS;
    let mut LockChange = 0u32;

    while MmIsAddressValid(VirtualAddress) == 0 {
        MiReleasePfnLock(OldIrql);

        Status = MmAccessFault(0u32, VirtualAddress, 0i8, core::ptr::null_mut());
        if Status < 0 {
            KeBugCheckEx(
                0x7A,
                3,
                Status as ULONG_PTR,
                0,
                VirtualAddress as ULONG_PTR,
            );
        }

        LockChange = 1;
        OldIrql = MiAcquirePfnLock();
    }

    LockChange
}

#[no_mangle]
pub unsafe extern "system" fn MiDeleteSystemPageableVm(
    mut PointerPte: PMMPTE,
    mut PageCount: PFN_NUMBER,
    _Flags: ULONG,
    ValidPages: *mut PFN_NUMBER,
) -> PFN_COUNT {
    let mut ActualPages: PFN_COUNT = 0;
    let CurrentThread = PsGetCurrentThread() as PETHREAD;
    let mut Pfn1: PMMPFN;
    let mut Pfn2: PMMPFN;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut PageTableIndex: PFN_NUMBER;
    let mut OldIrql: KIRQL;

    MiLockWorkingSet(CurrentThread, core::ptr::addr_of_mut!(MmSystemCacheWs) as PMMSUPPORT);

    while PageCount > 0 {
        if (*PointerPte).Long != 0 {
            if (*PointerPte).valid() {
                PageFrameIndex = PFN_FROM_PTE(PointerPte);
                Pfn1 = MiGetPfnEntry(PageFrameIndex);

                if !ValidPages.is_null() {
                    *ValidPages += 1;
                }

                PageTableIndex = (*Pfn1).pte_frame();
                Pfn2 = MiGetPfnEntry(PageTableIndex);

                OldIrql = MiAcquirePfnLock();
                MI_SET_PFN_DELETED(Pfn1);
                MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);
                MiDecrementShareCount(Pfn2 as ntkrnl_sys::ddk::PMMPFN, PageTableIndex as ntkrnl_sys::ddk::PFN_NUMBER);
                MiReleasePfnLock(OldIrql);

                MI_ERASE_PTE(PointerPte);
            } else {
                MI_ERASE_PTE(PointerPte);
            }
            ActualPages += 1;
        }

        PointerPte = PointerPte.add(1);
        PageCount -= 1;
    }

    MiUnlockWorkingSet(CurrentThread, core::ptr::addr_of_mut!(MmSystemCacheWs) as PMMSUPPORT);
    KeFlushEntireTb(1, 1);

    ActualPages
}

#[no_mangle]
pub unsafe extern "system" fn MiDeletePte(
    PointerPte: PMMPTE,
    VirtualAddress: PVOID,
    CurrentProcess: PEPROCESS,
    ProtoPteArg: PMMPTE,
) {
    let mut Pfn1: PMMPFN;
    let TempPte: MMPTE;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut PointerPde: PMMPDE;

    TempPte = *PointerPte;

    if !TempPte.valid() {
        if TempPte.transition() {
            PageFrameIndex = PFN_FROM_PTE(core::ptr::addr_of!(TempPte) as PMMPTE);
            Pfn1 = MiGetPfnEntry(PageFrameIndex);

            MI_ERASE_PTE(PointerPte);
            MiDecrementShareCount(MiGetPfnEntry((*Pfn1).pte_frame()) as ntkrnl_sys::ddk::PMMPFN, (*Pfn1).pte_frame() as ntkrnl_sys::ddk::PFN_NUMBER);

            MI_SET_PFN_DELETED(Pfn1);

            if (*Pfn1).e2_reference_count() == 0 {
                MiUnlinkPageFromList(Pfn1 as ntkrnl_sys::ddk::PMMPFN);
                (*Pfn1).set_e2_reference_count(1);
                MiDecrementReferenceCount(Pfn1, PageFrameIndex);
            }
            return;
        }
    }

    PageFrameIndex = PFN_FROM_PTE(core::ptr::addr_of!(TempPte) as PMMPTE);
    Pfn1 = MiGetPfnEntry(PageFrameIndex);

    #[cfg(target_arch = "aarch64")]
    {
        if Pfn1.is_null() {
            MI_ERASE_PTE(PointerPte);
            return;
        }
    }

    if (*Pfn1).prototype_pte() {
        PointerPde = MiPteToPde(PointerPte);
        if !(*PointerPde).valid() {
            #[cfg(target_pointer_width = "32")]
            {
                let check_status = MiCheckPdeForPagedPool(VirtualAddress);
                if check_status < 0 {
                    KeBugCheckEx(
                        0x1000,
                        0x61940,
                        PointerPte as ULONG_PTR,
                        (*PointerPte).Long,
                        VirtualAddress as ULONG_PTR,
                    );
                }
            }
            #[cfg(not(target_pointer_width = "32"))]
            {
                KeBugCheckEx(
                    0x1000,
                    0x61940,
                    PointerPte as ULONG_PTR,
                    (*PointerPte).Long,
                    VirtualAddress as ULONG_PTR,
                );
            }
        }

        PointerPde = MiPteToPde(PointerPte);
        MiDecrementShareCount(
            MiGetPfnEntry((*PointerPde).page_frame_number()) as ntkrnl_sys::ddk::PMMPFN,
            (*PointerPde).page_frame_number() as ntkrnl_sys::ddk::PFN_NUMBER,
        );
        MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);

        if (PointerPte <= MiHighestUserPte) && (ProtoPteArg != (*Pfn1).pte_address()) {
            if (PAGE_ALIGN(VirtualAddress as usize) as PVOID != USER_SHARED_DATA as PVOID) ||
                (MmHighestUserAddress as ULONG_PTR <= USER_SHARED_DATA)
            {
                #[cfg(not(target_arch = "aarch64"))]
                {
                    KeBugCheckEx(
                        0x1000,
                        0x400,
                        PointerPte as ULONG_PTR,
                        ProtoPteArg as ULONG_PTR,
                        (*Pfn1).pte_address() as ULONG_PTR,
                    );
                }
            }
        }

        MI_ERASE_PTE(PointerPte);
    } else {
        #[cfg(target_arch = "aarch64")]
        {
            (*Pfn1).set_pte_address(PointerPte);
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if ((*Pfn1).pte_address() as ULONG_PTR & !0x1) != PointerPte as ULONG_PTR {
                KeBugCheckEx(
                    0x1000,
                    0x401,
                    PointerPte as ULONG_PTR,
                    (*PointerPte).Long,
                    (*Pfn1).pte_address() as ULONG_PTR,
                );
            }
        }

        MI_ERASE_PTE(PointerPte);
        MiDecrementShareCount(MiGetPfnEntry((*Pfn1).pte_frame()) as ntkrnl_sys::ddk::PMMPFN, (*Pfn1).pte_frame() as ntkrnl_sys::ddk::PFN_NUMBER);
        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);
    }

    KeFlushCurrentTb();
}

#[no_mangle]
pub unsafe extern "system" fn MiDeleteVirtualAddresses(
    mut Va: ULONG_PTR,
    EndingAddress: ULONG_PTR,
    Vad: PMMVAD,
) {
    let mut PointerPte: PMMPTE;
    let mut ProtoPtr: PMMPTE;
    let mut LastPrototypePte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(target_arch = "x86_64")]
    let mut PointerPxe: PMMPTE;
    let mut TempPte: MMPTE;
    let CurrentProcess: PEPROCESS;
    let mut OldIrql: KIRQL;
    let mut AddressGap = 0u8;
    let mut _Subsection: PVOID;

    CurrentProcess = PsGetCurrentProcess() as *mut _EPROCESS;

    if !Vad.is_null() && ((*Vad).u.VadFlags.PrivateMemory() == 0) && !(*Vad).FirstPrototypePte.is_null() {
        ProtoPtr = (*Vad).FirstPrototypePte as PMMPTE;
        LastPrototypePte = (*Vad).FirstPrototypePte.add(1) as PMMPTE;
    } else {
        ProtoPtr = core::ptr::null_mut();
        LastPrototypePte = core::ptr::null_mut();
    }

    while Va <= EndingAddress {
        #[cfg(target_arch = "x86_64")]
        {
            PointerPxe = MiAddressToPxe(Va as PVOID);
            if !(*PointerPxe).valid() {
                if (*PointerPxe).Long == 0 {
                    AddressGap = 1;
                    Va = MiPxeToAddress(PointerPxe.add(1)) as ULONG_PTR;
                    continue;
                }
                MiMakeSystemAddressValid(MiPteToAddress(PointerPxe), CurrentProcess);
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            PointerPpe = MiAddressToPpe(Va as PVOID);
            if !(*PointerPpe).valid() {
                if (*PointerPpe).Long == 0 {
                    AddressGap = 1;
                    Va = MiPpeToAddress(PointerPpe.add(1)) as ULONG_PTR;
                    continue;
                }
                MiMakeSystemAddressValid(MiPteToAddress(PointerPpe), CurrentProcess);
            }
        }

        PointerPde = MiAddressToPde(Va as PVOID);
        if (*PointerPde).Long == 0 {
            AddressGap = 1;
            Va = MiPdeToAddress(PointerPde.add(1)) as ULONG_PTR;
            continue;
        }

        if !(*PointerPde).valid() {
            let PointerPteTemp = MiPteToAddress(PointerPde);
            MiMakeSystemAddressValid(PointerPteTemp, CurrentProcess);
        }

        if (AddressGap != 0) && !LastPrototypePte.is_null() {
            ProtoPtr = MI_GET_PROTOTYPE_PTE_FOR_VPN(Vad, Va >> PAGE_SHIFT);
            _Subsection = MiLocateSubsection(Vad, (Va >> PAGE_SHIFT) as ntkrnl_sys::ddk::ULONG_PTR) as PVOID;
            if !_Subsection.is_null() {
                LastPrototypePte = core::ptr::null_mut();
            } else {
                ProtoPtr = core::ptr::null_mut();
            }
        }

        OldIrql = MiAcquirePfnLock();
        PointerPte = MiAddressToPte(Va as PVOID);

        loop {
            TempPte = *PointerPte;
            if TempPte.Long != 0 {
                if MI_IS_MAPPED_PTE(core::ptr::addr_of!(TempPte)) {
                    if (!LastPrototypePte.is_null()) && (ProtoPtr > LastPrototypePte) {
                        ProtoPtr = MI_GET_PROTOTYPE_PTE_FOR_VPN(Vad, Va >> PAGE_SHIFT);
                        _Subsection = MiLocateSubsection(Vad, (Va >> PAGE_SHIFT) as ntkrnl_sys::ddk::ULONG_PTR) as PVOID;
                        if !_Subsection.is_null() {
                            LastPrototypePte = core::ptr::null_mut();
                        } else {
                            ProtoPtr = core::ptr::null_mut();
                        }
                    }

                    if !TempPte.valid() && TempPte.prototype() {
                        MI_ERASE_PTE(PointerPte);
                    } else {
                        MiDeletePte(PointerPte, Va as PVOID, CurrentProcess, ProtoPtr);
                    }
                } else {
                    MI_ERASE_PTE(PointerPte);
                }

                if MiDecrementPageTableReferences(Va as PVOID) == 0 {
                    MiDeletePde(PointerPde, CurrentProcess);
                    Va = MiPdeToAddress(PointerPde.add(1)) as ULONG_PTR;
                    PointerPte = PointerPte.add(1);
                    if !ProtoPtr.is_null() { ProtoPtr = ProtoPtr.add(1); }
                    break;
                }
            }

            Va = Va.wrapping_add(PAGE_SIZE);
            PointerPte = PointerPte.add(1);
            if !ProtoPtr.is_null() { ProtoPtr = ProtoPtr.add(1); }

            if ((Va & (0x200000 - 1)) == 0) || (Va > EndingAddress) {
                break;
            }
        }

        MiReleasePfnLock(OldIrql);

        if Va > EndingAddress {
            return;
        }

        AddressGap = if PointerPte != MiAddressToPte(Va as PVOID) { 1 } else { 0 };
    }
}

unsafe fn MiGetExceptionInfo(
    ExceptionInfo: PEXCEPTION_POINTERS,
    HaveBadAddress: *mut BOOLEAN,
    BadAddress: *mut ULONG_PTR,
) -> LONG {
    let ExceptionRecord: PEXCEPTION_RECORD;

    *HaveBadAddress = 0u8;
    ExceptionRecord = (*ExceptionInfo).ExceptionRecord;

    if ((*ExceptionRecord).ExceptionCode == 0xC0000005u32 as NTSTATUS) ||
       ((*ExceptionRecord).ExceptionCode == 0x80000001u32 as NTSTATUS) ||
       ((*ExceptionRecord).ExceptionCode == 0xC0000006u32 as NTSTATUS)
    {
        if (*ExceptionRecord).NumberParameters > 1 {
            *HaveBadAddress = 1u8;
            *BadAddress = (*ExceptionRecord).ExceptionInformation[1] as ULONG_PTR;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn MiDoMappedCopy(
    SourceProcess: PEPROCESS,
    SourceAddress: PVOID,
    TargetProcess: PEPROCESS,
    TargetAddress: PVOID,
    BufferSize: SIZE_T,
    PreviousMode: KPROCESSOR_MODE,
    ReturnSize: *mut SIZE_T,
) -> NTSTATUS {
    let mut MdlBuffer: [PFN_NUMBER; (core::mem::size_of::<MDL>() / core::mem::size_of::<PFN_NUMBER>()) + MI_MAPPED_COPY_PAGES + 1] = [0; (core::mem::size_of::<MDL>() / core::mem::size_of::<PFN_NUMBER>()) + MI_MAPPED_COPY_PAGES + 1];
    let Mdl = MdlBuffer.as_mut_ptr() as *mut MDL;
    let mut TotalSize: SIZE_T;
    let mut CurrentSize: SIZE_T;
    let mut RemainingSize: SIZE_T;
    let mut _FailedInProbe = 0u8;
    let mut PagesLocked = 0u8;
    let mut CurrentAddress = SourceAddress;
    let mut CurrentTargetAddress = TargetAddress;
    let mut MdlAddress: PVOID = core::ptr::null_mut();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut _HaveBadAddress: BOOLEAN = 0;
    let mut _BadAddress: ULONG_PTR = 0;
    let mut Status = STATUS_SUCCESS;

    TotalSize = MI_MAPPED_COPY_PAGES * PAGE_SIZE;
    if BufferSize <= TotalSize { TotalSize = BufferSize; }
    CurrentSize = TotalSize;
    RemainingSize = BufferSize;

    while RemainingSize > 0 {
        if RemainingSize < CurrentSize { CurrentSize = RemainingSize; }

        KeStackAttachProcess(core::ptr::addr_of_mut!((*(SourceProcess as *mut _EPROCESS)).Pcb), core::ptr::addr_of_mut!(ApcState));

        Status = STATUS_SUCCESS;
        if CurrentAddress == SourceAddress && PreviousMode != 0i8 {
            _FailedInProbe = 1;
            ProbeForRead(SourceAddress, BufferSize, core::mem::size_of::<i8>() as ULONG);
            _FailedInProbe = 0;
        }

        if Status >= 0 {
            MmInitializeMdl(Mdl as ntkrnl_sys::ddk::PMDL, CurrentAddress, CurrentSize);
            MmProbeAndLockPages(Mdl, PreviousMode, IoReadAccess);
            PagesLocked = 1;
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            break;
        }

        MdlAddress = MmMapLockedPagesSpecifyCache(Mdl as ntkrnl_sys::ddk::PMDL, 0i8, _MEMORY_CACHING_TYPE_MmCached, core::ptr::null_mut(), 0, HighPagePriority);
        if MdlAddress.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        KeStackAttachProcess(core::ptr::addr_of_mut!((*(TargetProcess as *mut _EPROCESS)).Pcb), core::ptr::addr_of_mut!(ApcState));

        Status = STATUS_SUCCESS;
        if CurrentTargetAddress == TargetAddress && PreviousMode != 0i8 {
            _FailedInProbe = 1;
            ProbeForWrite(TargetAddress, BufferSize, core::mem::size_of::<i8>() as ULONG);
            _FailedInProbe = 0;
        }

        if Status >= 0 {
            core::ptr::copy_nonoverlapping(MdlAddress as *const u8, CurrentTargetAddress as *mut u8, CurrentSize);
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = 0xC000009Bu32 as NTSTATUS;
            }
            break;
        }

        MmUnmapLockedPages(MdlAddress, Mdl as ntkrnl_sys::ddk::PMDL);
        MdlAddress = core::ptr::null_mut();
        MmUnlockPages(Mdl as ntkrnl_sys::ddk::PMDL);
        PagesLocked = 0;

        RemainingSize = RemainingSize.wrapping_sub(CurrentSize);
        CurrentAddress = (CurrentAddress as ULONG_PTR).wrapping_add(CurrentSize) as PVOID;
        CurrentTargetAddress = (CurrentTargetAddress as ULONG_PTR).wrapping_add(CurrentSize) as PVOID;
    }

    if !MdlAddress.is_null() {
        MmUnmapLockedPages(MdlAddress, Mdl as ntkrnl_sys::ddk::PMDL);
    }
    if PagesLocked != 0 {
        MmUnlockPages(Mdl as ntkrnl_sys::ddk::PMDL);
    }

    if Status >= 0 {
        *ReturnSize = BufferSize;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiDoPoolCopy(
    SourceProcess: PEPROCESS,
    SourceAddress: PVOID,
    TargetProcess: PEPROCESS,
    TargetAddress: PVOID,
    BufferSize: SIZE_T,
    PreviousMode: KPROCESSOR_MODE,
    ReturnSize: *mut SIZE_T,
) -> NTSTATUS {
    let mut StackBuffer: [u8; MI_POOL_COPY_BYTES] = [0; MI_POOL_COPY_BYTES];
    let mut TotalSize: SIZE_T;
    let mut CurrentSize: SIZE_T;
    let mut RemainingSize: SIZE_T;
    let mut _FailedInProbe = 0u8;
    let mut HavePoolAddress = 0u8;
    let mut CurrentAddress = SourceAddress;
    let mut CurrentTargetAddress = TargetAddress;
    let mut PoolAddress: PVOID;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut _HaveBadAddress: BOOLEAN = 0;
    let mut _BadAddress: ULONG_PTR = 0;
    let mut Status = STATUS_SUCCESS;

    TotalSize = MI_MAX_TRANSFER_SIZE;
    if BufferSize <= MI_MAX_TRANSFER_SIZE { TotalSize = BufferSize; }
    CurrentSize = TotalSize;
    RemainingSize = BufferSize;

    if BufferSize <= MI_POOL_COPY_BYTES {
        PoolAddress = StackBuffer.as_mut_ptr() as PVOID;
    } else {
        PoolAddress = ExAllocatePoolWithTag(NonPagedPool, TotalSize, 0x56d52777);
        HavePoolAddress = 1;
    }

    while RemainingSize > 0 {
        if RemainingSize < CurrentSize { CurrentSize = RemainingSize; }

        KeStackAttachProcess(core::ptr::addr_of_mut!((*(SourceProcess as *mut _EPROCESS)).Pcb), core::ptr::addr_of_mut!(ApcState));

        Status = STATUS_SUCCESS;
        if CurrentAddress == SourceAddress && PreviousMode != 0i8 {
            _FailedInProbe = 1;
            ProbeForRead(SourceAddress, BufferSize, core::mem::size_of::<i8>() as ULONG);
            _FailedInProbe = 0;
        }

        if Status >= 0 {
            core::ptr::copy_nonoverlapping(CurrentAddress as *const u8, PoolAddress as *mut u8, CurrentSize);
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = 0xC000009Bu32 as NTSTATUS;
            }
            break;
        }

        KeStackAttachProcess(core::ptr::addr_of_mut!((*(TargetProcess as *mut _EPROCESS)).Pcb), core::ptr::addr_of_mut!(ApcState));

        Status = STATUS_SUCCESS;
        if CurrentTargetAddress == TargetAddress && PreviousMode != 0i8 {
            _FailedInProbe = 1;
            ProbeForWrite(TargetAddress, BufferSize, core::mem::size_of::<i8>() as ULONG);
            _FailedInProbe = 0;
        }

        if Status >= 0 {
            core::ptr::copy_nonoverlapping(PoolAddress as *const u8, CurrentTargetAddress as *mut u8, CurrentSize);
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = 0xC000009Bu32 as NTSTATUS;
            }
            break;
        }

        RemainingSize = RemainingSize.wrapping_sub(CurrentSize);
        CurrentAddress = (CurrentAddress as ULONG_PTR).wrapping_add(CurrentSize) as PVOID;
        CurrentTargetAddress = (CurrentTargetAddress as ULONG_PTR).wrapping_add(CurrentSize) as PVOID;
    }

    if HavePoolAddress != 0 {
        ExFreePoolWithTag(PoolAddress, 0x56d52777);
    }

    if Status >= 0 {
        *ReturnSize = BufferSize;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiQueryAddressState(
    Va: PVOID,
    Vad: PMMVAD,
    TargetProcess: PEPROCESS,
    ReturnedProtect: PULONG,
    NextVa: *mut PVOID,
) -> ULONG {
    let mut PointerPte: PMMPTE;
    let mut ProtoPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(target_arch = "x86_64")]
    let mut PointerPxe: PMMPTE;
    let mut TempPte: MMPTE;
    let mut TempProtoPte: MMPTE;
    let mut IsDemandZeroPte = 1u8;
    let mut ValidPte = 0u8;
    let mut State = MEM_RESERVE;
    let mut Protect: ULONG = 0;

    PointerPde = MiAddressToPde(Va);
    PointerPte = MiAddressToPte(Va);
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    { PointerPpe = MiAddressToPpe(Va); }
    #[cfg(target_arch = "x86_64")]
    { PointerPxe = MiAddressToPxe(Va); }

    *NextVa = ((Va as ULONG_PTR) + PAGE_SIZE) as PVOID;

    loop {
        #[cfg(target_arch = "x86_64")]
        {
            if (*PointerPxe).Long == 0 {
                *NextVa = MiPxeToAddress(PointerPxe.add(1));
                break;
            }
            if !(*PointerPxe).valid() {
                MiMakeSystemAddressValid(PointerPpe as PVOID, TargetProcess);
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            if (*PointerPpe).Long == 0 {
                *NextVa = MiPpeToAddress(PointerPpe.add(1));
                break;
            }
            if !(*PointerPpe).valid() {
                MiMakeSystemAddressValid(PointerPde as PVOID, TargetProcess);
            }
        }

        if (*PointerPde).Long == 0 {
            *NextVa = MiPdeToAddress(PointerPde.add(1));
            break;
        }
        if !(*PointerPde).valid() {
            MiMakeSystemAddressValid(PointerPte as PVOID, TargetProcess);
        }

        ValidPte = 1;
        break;
    }

    if ValidPte != 0 {
        TempPte = *PointerPte;
        if TempPte.Long != 0 {
            IsDemandZeroPte = 0;

            if (TempPte.soft_protection() == MM_DECOMMIT) &&
               !TempPte.valid() &&
               (!TempPte.prototype() || (TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED))
            {
            } else {
                State = MEM_COMMIT;
                Protect = MiGetPageProtection(PointerPte);

                if !TempPte.valid() &&
                   TempPte.prototype() &&
                   ((*Vad).u.VadFlags.PrivateMemory() == 0) &&
                   !(*Vad).ControlArea.is_null()
                {
                }
            }
        }
    }

    if IsDemandZeroPte != 0 {
        if ((*Vad).u.VadFlags.PrivateMemory() == 0) && !(*Vad).ControlArea.is_null() {
            *NextVa = ((Va as ULONG_PTR) + PAGE_SIZE) as PVOID;
            ProtoPte = MI_GET_PROTOTYPE_PTE_FOR_VPN(Vad, (Va as ULONG_PTR) >> PAGE_SHIFT);
            if !ProtoPte.is_null() {
                TempProtoPte = *ProtoPte;
                if TempProtoPte.Long != 0 {
                    State = MEM_COMMIT;
                    Protect = MmProtectToValue[(*Vad).u.VadFlags.Protection() as usize];
                }
            }
        } else if (*Vad).u.VadFlags.MemCommit() != 0 {
            State = MEM_COMMIT;
            Protect = MmProtectToValue[(*Vad).u.VadFlags.Protection() as usize];
        }
    }

    *ReturnedProtect = Protect;
    State
}

#[no_mangle]
pub unsafe extern "system" fn MiQueryMemoryBasicInformation(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    MemoryInformation: PVOID,
    _MemoryInformationLength: SIZE_T,
    ReturnLength: *mut SIZE_T,
) -> NTSTATUS {
    let mut TargetProcess: PEPROCESS = core::ptr::null_mut();
    let mut Status = STATUS_SUCCESS;
    let mut Vad: PMMVAD = core::ptr::null_mut();
    let mut Address: PVOID;
    let mut NextAddress: PVOID = core::ptr::null_mut();
    let mut Found = 0u8;
    let mut NewProtect: ULONG = 0;
    let mut NewState: ULONG;
    let mut BaseVpn: ULONG_PTR = 0;
    let mut MemoryInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let PreviousMode = ExGetPreviousMode();
    let mut ResultLength: SIZE_T = 0;

    if ((BaseAddress as ULONG_PTR) > MM_HIGHEST_VAD_ADDRESS) || (PAGE_ALIGN(BaseAddress as usize) == MM_SHARED_USER_DATA_VA) {
        Address = PAGE_ALIGN(BaseAddress as usize) as PVOID;
        MemoryInfo.BaseAddress = Address;
        MemoryInfo.AllocationProtect = PAGE_READONLY;
        MemoryInfo.Type = MEM_PRIVATE;

        if Address as ULONG_PTR == MM_SHARED_USER_DATA_VA {
            MemoryInfo.AllocationBase = MM_SHARED_USER_DATA_VA as PVOID;
            MemoryInfo.State = MEM_COMMIT;
            MemoryInfo.Protect = PAGE_READONLY;
            MemoryInfo.RegionSize = PAGE_SIZE as ntkrnl_sys::ddk::SIZE_T;
        } else {
            MemoryInfo.AllocationBase = (MM_HIGHEST_VAD_ADDRESS + 1) as PVOID;
            MemoryInfo.State = MEM_RESERVE;
            MemoryInfo.Protect = 0;
            MemoryInfo.RegionSize = ((MM_HIGHEST_USER_ADDRESS + 1) - (Address as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
        }

        *(MemoryInformation as *mut MEMORY_BASIC_INFORMATION) = MemoryInfo;
        if !ReturnLength.is_null() { *ReturnLength = core::mem::size_of::<MEMORY_BASIC_INFORMATION>(); }

        return Status;
    }

    if ProcessHandle == NtCurrentProcess() as HANDLE {
        TargetProcess = PsGetCurrentProcess() as *mut _EPROCESS;
    } else {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            0x400 as ACCESS_MASK,
            PsProcessType,
            ExGetPreviousMode(),
            core::ptr::addr_of_mut!(TargetProcess) as *mut PVOID,
            core::ptr::null_mut() as *mut ntkrnl_sys::ddk::_OBJECT_HANDLE_INFORMATION,
        );
        if Status < 0 { return Status; }
        KeStackAttachProcess(core::ptr::addr_of_mut!((*TargetProcess).Pcb), core::ptr::addr_of_mut!(ApcState));
    }

    MmLockAddressSpace(core::ptr::addr_of_mut!((*TargetProcess).Vm) as PVOID);
    if (*TargetProcess).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        MmUnlockAddressSpace(core::ptr::addr_of_mut!((*TargetProcess).Vm) as PVOID);
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            ObDereferenceObject(TargetProcess as PVOID);
        }
        return 0xC000010Au32 as NTSTATUS;
    }

    if (*TargetProcess).VadRoot.NumberGenericTableElements() != 0 {
        Vad = (*TargetProcess).VadRoot.BalancedRoot.RightChild as PMMVAD;
        BaseVpn = (BaseAddress as ULONG_PTR) >> PAGE_SHIFT;

        while !Vad.is_null() {
            if (BaseVpn >= (*Vad).StartingVpn as usize) && (BaseVpn <= (*Vad).EndingVpn as usize) {
                Found = 1;
                break;
            }

            if BaseVpn < (*Vad).StartingVpn as usize {
                if (*Vad).LeftChild.is_null() { break; }
                Vad = (*Vad).LeftChild as PMMVAD;
            } else {
                if (*Vad).RightChild.is_null() { break; }
                Vad = (*Vad).RightChild as PMMVAD;
            }
        }
    }

    if Found == 0 {
        Address = PAGE_ALIGN(BaseAddress as usize) as PVOID;

        if !Vad.is_null() {
            if (*Vad).StartingVpn as usize >= BaseVpn {
                MemoryInfo.RegionSize = ((((*Vad).StartingVpn as usize) << PAGE_SHIFT) - (Address as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
            } else {
                Vad = MiGetNextNode(Vad as PMMADDRESS_NODE) as PMMVAD;
                if !Vad.is_null() {
                    MemoryInfo.RegionSize = ((((*Vad).StartingVpn as usize) << PAGE_SHIFT) - (Address as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
                } else {
                    MemoryInfo.RegionSize = ((MM_HIGHEST_VAD_ADDRESS + 1) - (Address as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
                }
            }
        } else {
            MemoryInfo.RegionSize = ((MM_HIGHEST_VAD_ADDRESS + 1) - (Address as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
        }

        MmUnlockAddressSpace(core::ptr::addr_of_mut!((*TargetProcess).Vm) as PVOID);

        if ProcessHandle != NtCurrentProcess() as HANDLE {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            ObDereferenceObject(TargetProcess as PVOID);
        }

        MemoryInfo.BaseAddress = Address;
        MemoryInfo.AllocationBase = core::ptr::null_mut();
        MemoryInfo.AllocationProtect = 0;
        MemoryInfo.State = MEM_FREE;
        MemoryInfo.Protect = 0;
        MemoryInfo.Type = 0;

        *(MemoryInformation as *mut MEMORY_BASIC_INFORMATION) = MemoryInfo;
        if !ReturnLength.is_null() { *ReturnLength = core::mem::size_of::<MEMORY_BASIC_INFORMATION>(); }

        return Status;
    }

    if ((*Vad).u.VadFlags.PrivateMemory() != 0) || ((*Vad).u.VadFlags.VadType() == 2) {
        MemoryInfo.Type = MEM_PRIVATE;
    } else if (*Vad).u.VadFlags.VadType() == 1 {
        MemoryInfo.Type = 0x1000000;
    } else {
        MemoryInfo.Type = 0x40000;
    }

    if MI_IS_ROSMM_VAD(Vad as PVOID) {
        let mut DdkResultLength: ntkrnl_sys::ddk::SIZE_T = 0;
        Status = MmQuerySectionView(
            Vad as ntkrnl_sys::ddk::PMEMORY_AREA,
            BaseAddress,
            core::ptr::addr_of_mut!(MemoryInfo) as ntkrnl_sys::ddk::PMEMORY_BASIC_INFORMATION,
            core::ptr::addr_of_mut!(DdkResultLength),
        );
        ResultLength = DdkResultLength as SIZE_T;
    } else {
        Address = PAGE_ALIGN(BaseAddress as usize) as PVOID;
        MemoryInfo.BaseAddress = Address;
        MemoryInfo.AllocationBase = ((*Vad).StartingVpn << PAGE_SHIFT) as PVOID;
        MemoryInfo.AllocationProtect = MmProtectToValue[(*Vad).u.VadFlags.Protection() as usize];
        MemoryInfo.Type = MEM_PRIVATE;

        MiLockProcessWorkingSetShared(TargetProcess, PsGetCurrentThread() as PETHREAD);

        MemoryInfo.State = MiQueryAddressState(
            Address,
            Vad,
            TargetProcess,
            core::ptr::addr_of_mut!(MemoryInfo.Protect),
            core::ptr::addr_of_mut!(NextAddress),
        );
        Address = NextAddress;

        while ((Address as ULONG_PTR) >> PAGE_SHIFT) <= (*Vad).EndingVpn as usize {
            NewState = MiQueryAddressState(Address, Vad, TargetProcess, core::ptr::addr_of_mut!(NewProtect), core::ptr::addr_of_mut!(NextAddress));
            if (NewState != MemoryInfo.State) || (NewProtect != MemoryInfo.Protect) { break; }
            Address = NextAddress;
        }

        MiUnlockProcessWorkingSetShared(TargetProcess, PsGetCurrentThread() as PETHREAD);

        if ((Address as ULONG_PTR) >> PAGE_SHIFT) > (*Vad).EndingVpn as usize {
            Address = ((((*Vad).EndingVpn as usize) + 1) << PAGE_SHIFT) as PVOID;
        }

        MemoryInfo.RegionSize = ((Address as ULONG_PTR) - (MemoryInfo.BaseAddress as ULONG_PTR)) as ntkrnl_sys::ddk::SIZE_T;
    }

    MmUnlockAddressSpace(core::ptr::addr_of_mut!((*TargetProcess).Vm) as PVOID);

    if ProcessHandle != NtCurrentProcess() as HANDLE {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        ObDereferenceObject(TargetProcess as PVOID);
    }

    *(MemoryInformation as *mut MEMORY_BASIC_INFORMATION) = MemoryInfo;
    if !ReturnLength.is_null() { *ReturnLength = core::mem::size_of::<MEMORY_BASIC_INFORMATION>(); }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiIsEntireRangeCommitted(
    StartingAddress: ULONG_PTR,
    EndingAddress: ULONG_PTR,
    Vad: PMMVAD,
    Process: PEPROCESS,
) -> BOOLEAN {
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let mut OnPdeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPpeBoundary = true;
    #[cfg(target_arch = "x86_64")]
    let mut PointerPxe: PMMPTE;
    #[cfg(target_arch = "x86_64")]
    let mut OnPxeBoundary = true;

    PointerPte = MiAddressToPte(StartingAddress as PVOID);
    LastPte = MiAddressToPte(EndingAddress as PVOID);

    while PointerPte <= LastPte {
        #[cfg(target_arch = "x86_64")]
        {
            if OnPxeBoundary {
                PointerPxe = MiPteToPxe(PointerPte);
                if (*PointerPxe).Long != 0 {
                    if !(*PointerPxe).valid() {
                        MiMakeSystemAddressValid(MiPteToPpe(PointerPte) as PVOID, Process);
                    }
                } else {
                    if (*Vad).u.VadFlags.MemCommit() == 0 { return 0u8; }
                    PointerPxe = PointerPxe.add(1);
                    PointerPte = MiPxeToPte(PointerPxe);
                    continue;
                }
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            if OnPpeBoundary {
                PointerPpe = MiPteToPpe(PointerPte);
                if (*PointerPpe).Long != 0 {
                    if !(*PointerPpe).valid() {
                        MiMakeSystemAddressValid(MiPteToPde(PointerPte) as PVOID, Process);
                    }
                } else {
                    if (*Vad).u.VadFlags.MemCommit() == 0 { return 0u8; }
                    PointerPpe = PointerPpe.add(1);
                    PointerPte = MiPpeToPte(PointerPpe);
                    #[cfg(target_arch = "x86_64")]
                    { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
                    continue;
                }
            }
        }

        if OnPdeBoundary {
            PointerPde = MiPteToPde(PointerPte);
            if (*PointerPde).Long != 0 {
                if !(*PointerPde).valid() {
                    MiMakeSystemAddressValid(PointerPte as PVOID, Process);
                }
            } else {
                if (*Vad).u.VadFlags.MemCommit() == 0 { return 0u8; }
                PointerPde = PointerPde.add(1);
                PointerPte = MiPdeToPte(PointerPde);
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                { OnPpeBoundary = MiIsPteOnPpeBoundary(PointerPte); }
                #[cfg(target_arch = "x86_64")]
                { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
                continue;
            }
        }

        if (*PointerPte).Long == 0 {
            if (*Vad).u.VadFlags.MemCommit() == 0 { return 0u8; }
        } else {
            if ((*PointerPte).soft_protection() == MM_DECOMMIT) &&
               !(*PointerPte).valid() &&
               (!(*PointerPte).prototype() || ((*PointerPte).soft_page_file_high() == MI_PTE_LOOKUP_NEEDED))
            {
                return 0u8;
            }
        }

        PointerPte = PointerPte.add(1);
        OnPdeBoundary = MiIsPteOnPdeBoundary(PointerPte);
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        { OnPpeBoundary = MiIsPteOnPpeBoundary(PointerPte); }
        #[cfg(target_arch = "x86_64")]
        { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn MiMakePdeExistAndMakeValid(
    PointerPde: PMMPDE,
    TargetProcess: PEPROCESS,
    _OldIrql: KIRQL,
) {
    let PointerPte: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let PointerPpe = MiPdeToPpe(PointerPde);
    #[cfg(target_arch = "x86_64")]
    let PointerPxe = MiPdeToPxe(PointerPde);

    #[cfg(target_arch = "x86_64")]
    {
        if (*PointerPxe).valid() &&
           (*PointerPpe).valid() &&
           (*PointerPde).valid()
        {
            return;
        }
    }
    #[cfg(all(target_arch = "aarch64", not(target_arch = "x86_64")))]
    {
        if (*PointerPpe).valid() && (*PointerPde).valid() {
            return;
        }
    }
    #[cfg(all(target_pointer_width = "32", not(target_arch = "aarch64")))]
    {
        if (*PointerPde).valid() {
            return;
        }
    }

    PointerPte = MiPteToAddress(PointerPde) as PMMPTE;

    loop {
        #[cfg(target_arch = "x86_64")]
        {
            if !(*PointerPxe).valid() {
                MiMakeSystemAddressValid(PointerPpe as PVOID, TargetProcess);
            }
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            if !(*PointerPpe).valid() {
                MiMakeSystemAddressValid(PointerPde as PVOID, TargetProcess);
            }
        }

        MiMakeSystemAddressValid(PointerPte as PVOID, TargetProcess);

        if (*PointerPde).valid() {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiProcessValidPteList(
    ValidPteList: *mut PMMPTE,
    Count: ULONG,
) {
    let mut OldIrql: KIRQL;
    let mut i: ULONG;
    let mut TempPte: MMPTE;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut Pfn1: PMMPFN;
    let mut Pfn2: PMMPFN;

    OldIrql = MiAcquirePfnLock();
    i = 0;
    while i != Count {
        TempPte = *(*ValidPteList.add(i as usize));

        PageFrameIndex = PFN_FROM_PTE(core::ptr::addr_of!(TempPte) as PMMPTE);
        Pfn1 = MiGetPfnEntry(PageFrameIndex);
        Pfn2 = MiGetPfnEntry((*Pfn1).pte_frame());

        MiDecrementShareCount(Pfn2 as ntkrnl_sys::ddk::PMMPFN, (*Pfn1).pte_frame() as ntkrnl_sys::ddk::PFN_NUMBER);
        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);

        MI_WRITE_INVALID_PTE(*ValidPteList.add(i as usize), MmDecommittedPte);
        i += 1;
    }

    KeFlushCurrentTb();
    MiReleasePfnLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn MiDecommitPages(
    mut StartingAddress: PVOID,
    EndingPte: PMMPTE,
    Process: PEPROCESS,
    Vad: PMMVAD,
) -> ULONG {
    let mut PointerPte: PMMPTE;
    let mut CommitPte: PMMPTE = core::ptr::null_mut();
    let mut PointerPde: PMMPDE;
    let mut CommitReduction: ULONG = 0;
    let mut ValidPteList: [PMMPTE; 256] = [core::ptr::null_mut(); 256];
    let mut PteCount: ULONG = 0;
    let mut _Pfn1: PMMPFN;
    let mut PteContents: MMPTE;
    let CurrentThread = PsGetCurrentThread() as PETHREAD;

    PointerPde = MiAddressToPde(StartingAddress);
    PointerPte = MiAddressToPte(StartingAddress);
    if (*Vad).u.VadFlags.MemCommit() != 0 {
        CommitPte = MiAddressToPte(((*Vad).EndingVpn << PAGE_SHIFT) as PVOID);
    }
    MiLockProcessWorkingSetUnsafe(Process, CurrentThread);

    MiMakePdeExistAndMakeValid(PointerPde, Process, 0);

    while PointerPte <= EndingPte {
        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = MiAddressToPde(StartingAddress);
            if PteCount != 0 {
                MiProcessValidPteList(ValidPteList.as_mut_ptr(), PteCount);
                PteCount = 0;
            }
            MiMakePdeExistAndMakeValid(PointerPde, Process, 0);
        }

        PteContents = *PointerPte;
        if PteContents.Long != 0 {
            if (*PointerPte).Long == MmDecommittedPte.Long {
                CommitReduction += 1;
            } else {
                if PteContents.valid() {
                    _Pfn1 = MiGetPfnEntry(PteContents.page_frame_number());

                    if PteCount == 256 {
                        MiProcessValidPteList(ValidPteList.as_mut_ptr(), PteCount);
                        PteCount = 0;
                    }
                    ValidPteList[PteCount as usize] = PointerPte;
                    PteCount += 1;
                } else {
                    MI_WRITE_INVALID_PTE(PointerPte, MmDecommittedPte);
                }
            }
        } else {
            MiIncrementPageTableReferences(StartingAddress);
            if PointerPte > CommitPte { CommitReduction += 1; }
            MI_WRITE_INVALID_PTE(PointerPte, MmDecommittedPte);
        }

        PointerPte = PointerPte.add(1);
        StartingAddress = (StartingAddress as ULONG_PTR + PAGE_SIZE) as PVOID;
    }

    if PteCount != 0 {
        MiProcessValidPteList(ValidPteList.as_mut_ptr(), PteCount);
    }
    MiUnlockProcessWorkingSetUnsafe(Process, CurrentThread);
    CommitReduction
}

// ============================================================================
// Translated functions: MmCopyVirtualMemory, MmFlushVirtualMemory,
//                       MiGetPageProtection, MiProtectVirtualMemory
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmCopyVirtualMemory(
    SourceProcess: PEPROCESS,
    SourceAddress: PVOID,
    TargetProcess: PEPROCESS,
    TargetAddress: PVOID,
    BufferSize: SIZE_T,
    PreviousMode: KPROCESSOR_MODE,
    ReturnSize: PSIZE_T,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Process: PEPROCESS = SourceProcess;

    // Don't accept zero-sized buffers
    if BufferSize == 0 {
        return STATUS_SUCCESS;
    }

    // If we are copying from ourselves, lock the target instead
    if SourceProcess == PsGetCurrentProcess() as PEPROCESS {
        Process = TargetProcess;
    }

    // Acquire rundown protection
    if ExAcquireRundownProtection(
        core::ptr::addr_of_mut!((*Process).RundownProtect)
    ) == 0 {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    // See if we should use the pool copy
    if BufferSize > MI_POOL_COPY_BYTES {
        // Use MDL-copy
        Status = MiDoMappedCopy(
            SourceProcess,
            SourceAddress,
            TargetProcess,
            TargetAddress,
            BufferSize,
            PreviousMode,
            ReturnSize,
        );
    } else {
        // Do pool copy
        Status = MiDoPoolCopy(
            SourceProcess,
            SourceAddress,
            TargetProcess,
            TargetAddress,
            BufferSize,
            PreviousMode,
            ReturnSize,
        );
    }

    // Release the lock
    ExReleaseRundownProtection(
        core::ptr::addr_of_mut!((*Process).RundownProtect)
    );
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmFlushVirtualMemory(
    _Process: PEPROCESS,
    _BaseAddress: *mut PVOID,
    _RegionSize: *mut SIZE_T,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    // PAGED_CODE();
    // UNIMPLEMENTED;

    // Report actual state.
    (*IoStatusBlock).__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    (*IoStatusBlock).Information = 0;

    // Pretend success.
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MiGetPageProtection(
    mut PointerPte: PMMPTE,
) -> ULONG {
    let mut TempPte: MMPTE;
    let Pfn: PMMPFN;
    let CurrentProcess: PEPROCESS;
    let CurrentThread: PETHREAD;
    let mut WsSafe: BOOLEAN = 0;
    let mut WsShared: BOOLEAN = 0;
    let Protect: ULONG;
    let OldIrql: KIRQL;
    // PAGED_CODE();

    // Copy this PTE's contents
    TempPte = *PointerPte;

    // Assure it's not totally zero
    ASSERT!(TempPte.Long != 0);

    // Check for a special prototype format
    if !TempPte.valid() && TempPte.prototype() {
        // Check if the prototype PTE is not yet pointing to a PTE
        if TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED {
            // The prototype PTE contains the protection
            return MmProtectToValue[TempPte.soft_protection() as usize];
        }

        // Get a pointer to the underlying shared PTE
        PointerPte = MiProtoPteToPte(&mut TempPte as PMMPTE);

        // Since the PTE we want to read can be paged out at any time, we need
        // to release the working set lock first, so that it can be paged in
        CurrentThread = PsGetCurrentThread();
        CurrentProcess = PsGetCurrentProcess() as PEPROCESS;
        MiUnlockProcessWorkingSetForFault(
            CurrentProcess,
            CurrentThread,
            &mut WsSafe,
            &mut WsShared,
        );

        // Now read the PTE value
        TempPte = *PointerPte;

        // Check if that one is invalid
        if !TempPte.valid() {
            // We get the protection directly from this PTE
            Protect = MmProtectToValue[TempPte.soft_protection() as usize];
        } else {
            // The PTE is valid, so we might need to get the protection from
            // the PFN. Lock the PFN database
            OldIrql = MiAcquirePfnLock();

            // Check if the PDE is still valid
            if !(*MiAddressToPte(PointerPte as PVOID)).valid() {
                // It's not, make it valid
                MiMakeSystemAddressValidPfn(PointerPte as PVOID, OldIrql);
            }

            // Now it's safe to read the PTE value again
            TempPte = *PointerPte;
            ASSERT!(TempPte.Long != 0);

            // Check again if the PTE is invalid
            if !TempPte.valid() {
                // The PTE is not valid, so we can use its protection field
                Protect = MmProtectToValue[TempPte.soft_protection() as usize];
            } else {
                // The PTE is valid, so we can find the protection in the
                // OriginalPte field of the PFN
                Pfn = MI_PFN_ELEMENT(TempPte.page_frame_number());
                Protect = MmProtectToValue[(*mmpfn_original_pte_ptr(Pfn)).soft_protection() as usize];
            }

            // Release the PFN database
            MiReleasePfnLock(OldIrql);
        }

        // Lock the working set again
        MiLockProcessWorkingSetForFault(
            CurrentProcess,
            CurrentThread,
            WsSafe,
            WsShared,
        );

        return Protect;
    }

    // In the easy case of transition or demand zero PTE just return its protection
    if !TempPte.valid() {
        return MmProtectToValue[TempPte.soft_protection() as usize];
    }

    // If we get here, the PTE is valid, so look up the page in PFN database
    Pfn = MiGetPfnEntry(TempPte.page_frame_number());
    if !(*Pfn).prototype_pte() {
        // Return protection of the original pte
        return MmProtectToValue[(*mmpfn_original_pte_ptr(Pfn)).soft_protection() as usize];
    }

    // This is software PTE
    MmProtectToValue[TempPte.soft_protection() as usize]
}

#[no_mangle]
pub unsafe extern "system" fn MiProtectVirtualMemory(
    Process: PEPROCESS,
    BaseAddress: *mut PVOID,
    NumberOfBytesToProtect: *mut SIZE_T,
    NewAccessProtection: ULONG,
    OldAccessProtection: *mut ULONG,
) -> NTSTATUS {
    let mut Vad: PMMVAD;
    let AddressSpace: PMMSUPPORT;
    let StartingAddress: ULONG_PTR;
    let EndingAddress: ULONG_PTR;
    let mut PointerPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let LastPte: PMMPTE;
    let mut PteContents: MMPTE;
    let mut Pfn1: PMMPFN;
    let ProtectionMask: ULONG;
    let mut OldProtect: ULONG;
    let Committed: BOOLEAN;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Thread: PETHREAD = PsGetCurrentThread();
    let Result: TABLE_SEARCH_RESULT;

    // We must be attached
    ASSERT!(Process == PsGetCurrentProcess() as PEPROCESS);

    // Calculate base address for the VAD
    StartingAddress = PAGE_ALIGN(*BaseAddress as usize) as ULONG_PTR;
    EndingAddress = (*BaseAddress as ULONG_PTR + *NumberOfBytesToProtect - 1) | (PAGE_SIZE - 1);

    // Calculate the protection mask and make sure it's valid
    ProtectionMask = MiMakeProtectionMask(NewAccessProtection);
    if ProtectionMask == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    // Lock the address space and make sure the process isn't already dead
    AddressSpace = MmGetCurrentAddressSpace();
    MmLockAddressSpace(AddressSpace as PVOID);
    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        Status = STATUS_PROCESS_IS_TERMINATING;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // Get the VAD for this address range, and make sure it exists
    Vad = core::ptr::null_mut();
    Result = MiCheckForConflictingNode(
        (StartingAddress >> PAGE_SHIFT) as _,
        (EndingAddress >> PAGE_SHIFT) as _,
        &mut (*Process).VadRoot,
        &mut Vad as *mut PMMVAD as *mut PMMADDRESS_NODE,
    );
    if Result != TableFoundNode {
        Status = STATUS_CONFLICTING_ADDRESSES;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // Check if this is a ROSMM VAD
    if MI_IS_ROSMM_VAD(Vad as PVOID) {
        // Not too shabby hack
        *NumberOfBytesToProtect = PAGE_ROUND_UP((*BaseAddress as ULONG_PTR) + (*NumberOfBytesToProtect)) - PAGE_ROUND_DOWN(*BaseAddress as ULONG_PTR);
        *BaseAddress = PAGE_ROUND_DOWN(*BaseAddress as ULONG_PTR) as PVOID;
        Status = MmProtectSectionView(
            AddressSpace as ntkrnl_sys::ddk::PMMSUPPORT,
            Vad as PMEMORY_AREA,
            *BaseAddress,
            *NumberOfBytesToProtect as ntkrnl_sys::ddk::SIZE_T,
            NewAccessProtection,
            OldAccessProtection,
        );
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // Make sure the address is within this VAD's boundaries
    if ((StartingAddress >> PAGE_SHIFT) < (*Vad).StartingVpn as ULONG_PTR) ||
       ((EndingAddress >> PAGE_SHIFT) > (*Vad).EndingVpn as ULONG_PTR)
    {
        Status = STATUS_CONFLICTING_ADDRESSES;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // These kinds of VADs are not supported atm
    if ((*Vad).u.VadFlags.VadType() as u32 == VadAwe) ||
       ((*Vad).u.VadFlags.VadType() as u32 == VadDevicePhysicalMemory) ||
       ((*Vad).u.VadFlags.VadType() as u32 == VadLargePages)
    {
        Status = STATUS_CONFLICTING_ADDRESSES;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // Check for a VAD whose protection can't be changed
    if (*Vad).u.VadFlags.NoChange() == 1 {
        Status = STATUS_INVALID_PAGE_PROTECTION;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    // Is this section, or private memory?
    if (*Vad).u.VadFlags.PrivateMemory() == 0 {
        // Not yet supported
        if (*Vad).u.VadFlags.VadType() as u32 == VadLargePageSection {
            Status = STATUS_CONFLICTING_ADDRESSES;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            return Status;
        }

        // Rotate VADs are not yet supported
        if (*Vad).u.VadFlags.VadType() as u32 == VadRotatePhysical {
            Status = STATUS_CONFLICTING_ADDRESSES;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            return Status;
        }

        // Not valid on section files
        if (NewAccessProtection & (PAGE_NOCACHE | PAGE_WRITECOMBINE)) != 0 {
            Status = STATUS_INVALID_PARAMETER_4;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            return Status;
        }

        // Check if data or page file mapping protection PTE is compatible
        if (*(*Vad).ControlArea).u.Flags.Image() == 0 {
            // Not yet
        }

        // This is a section, and this is not yet supported
        OldProtect = 0;
    } else {
        // Private memory, check protection flags
        if (NewAccessProtection & PAGE_WRITECOPY) != 0 ||
           (NewAccessProtection & PAGE_EXECUTE_WRITECOPY) != 0
        {
            Status = STATUS_INVALID_PARAMETER_4;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            return Status;
        }

        // Lock the working set
        MiLockProcessWorkingSetUnsafe(Process, Thread);

        // Check if all pages in this range are committed
        Committed = MiIsEntireRangeCommitted(
            StartingAddress,
            EndingAddress,
            Vad,
            Process,
        );
        if Committed == 0 {
            MiUnlockProcessWorkingSetUnsafe(Process, Thread);
            Status = STATUS_NOT_COMMITTED;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            return Status;
        }

        // Compute starting and ending PTE and PDE addresses
        PointerPde = MiAddressToPde(StartingAddress as PVOID);
        PointerPte = MiAddressToPte(StartingAddress as PVOID);
        LastPte = MiAddressToPte(EndingAddress as PVOID);

        // Make this PDE valid
        MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);

        // Save protection of the first page
        if (*PointerPte).Long != 0 {
            // Capture the page protection and make the PDE valid
            OldProtect = MiGetPageProtection(PointerPte);
            MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);
        } else {
            // Grab the old protection from the VAD itself
            OldProtect = MmProtectToValue[(*Vad).u.VadFlags.Protection() as usize];
        }

        // Loop all the PTEs now
        while PointerPte <= LastPte {
            // Check if we've crossed a PDE boundary and make the new PDE valid too
            if MiIsPteOnPdeBoundary(PointerPte) {
                PointerPde = MiPteToPde(PointerPte);
                MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);
            }

            // Capture the PTE and check if it was empty
            PteContents = *PointerPte;
            if PteContents.Long == 0 {
                // This used to be a zero PTE and it no longer is, so we must add a
                // reference to the pagetable.
                MiIncrementPageTableReferences(MiPteToAddress(PointerPte));
            }

            // Check what kind of PTE we are dealing with
            if PteContents.valid() {
                // Get the PFN entry
                Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(&mut PteContents as PMMPTE));

                // We don't support this yet
                ASSERT!(!(*Pfn1).prototype_pte());

                // Check if the page should not be accessible at all
                if (NewAccessProtection & PAGE_NOACCESS) != 0 ||
                   (NewAccessProtection & PAGE_GUARD) != 0
                {
                    let OldIrql = MiAcquirePfnLock();

                    // Mark the PTE as transition and change its protection
                    PteContents.set_valid(false);
                    PteContents.set_transition(true);
                    PteContents.set_soft_protection(ProtectionMask);
                    // Decrease PFN share count and write the PTE
                    MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PFN_FROM_PTE(&mut PteContents as PMMPTE) as ntkrnl_sys::ddk::PFN_NUMBER);
                    // FIXME: remove the page from the WS
                    MI_WRITE_INVALID_PTE(PointerPte, PteContents);
                    KeInvalidateTlbEntry(MiPteToAddress(PointerPte));

                    // We are done for this PTE
                    MiReleasePfnLock(OldIrql);
                } else {
                    // Write the protection mask and write it with a TLB flush
                    (*mmpfn_original_pte_ptr(Pfn1)).set_soft_protection(ProtectionMask);
                    MiFlushTbAndCapture(
                        Vad,
                        PointerPte,
                        ProtectionMask,
                        Pfn1,
                        1u8,
                    );
                }
            } else {
                // We don't support these cases yet
                ASSERT!(!PteContents.prototype());

                // The PTE is already demand-zero, just update the protection mask
                PteContents.set_soft_protection(ProtectionMask);
                MI_WRITE_INVALID_PTE(PointerPte, PteContents);
                ASSERT!((*PointerPte).Long != 0);
            }

            // Move to the next PTE
            PointerPte = PointerPte.add(1);
        }

        // Unlock the working set
        MiUnlockProcessWorkingSetUnsafe(Process, Thread);
    }

    // Unlock the address space
    MmUnlockAddressSpace(AddressSpace as PVOID);

    // Return parameters and success
    *NumberOfBytesToProtect = EndingAddress - StartingAddress + 1;
    *BaseAddress = StartingAddress as PVOID;
    *OldAccessProtection = OldProtect;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmGetVirtualForPhysical(
    _PhysicalAddress: PHYSICAL_ADDRESS,
) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn MmSecureVirtualMemory(
    Address: PVOID,
    _Length: SIZE_T,
    _Mode: ULONG,
) -> PVOID {
    Address
}

#[no_mangle]
pub unsafe extern "system" fn MmUnsecureVirtualMemory(
    _SecureMem: PVOID,
) {
}

#[no_mangle]
pub unsafe extern "system" fn NtAllocateVirtualMemory(
    _ProcessHandle: HANDLE,
    _UBaseAddress: *mut PVOID,
    _ZeroBits: ULONG_PTR,
    _URegionSize: *mut SIZE_T,
    _AllocationType: ULONG,
    _Protect: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtFreeVirtualMemory(
    _ProcessHandle: HANDLE,
    _UBaseAddress: *mut PVOID,
    _URegionSize: *mut SIZE_T,
    _FreeType: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPhysicalAddress(
    Address: PVOID,
) -> PHYSICAL_ADDRESS {
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let TempPde: MMPTE;
    let TempPte: MMPTE;

    let pde_valid = {
        let mut result = true;
        #[cfg(target_arch = "x86_64")]
        {
            if !(*MiAddressToPxe(Address)).valid() { result = false; }
        }
        if result {
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            {
                if !(*MiAddressToPpe(Address)).valid() { result = false; }
            }
        }
        if result {
            if !(*MiAddressToPde(Address)).valid() { result = false; }
        }
        result
    };

    if pde_valid {
        TempPde = *MiAddressToPde(Address);
        let is_large = MI_IS_PAGE_LARGE(core::ptr::addr_of!(TempPde)) != 0;

        if is_large {
            PhysicalAddress.QuadPart = ((TempPde.page_frame_number() as i64) << PAGE_SHIFT as i64) +
                                       ((Address as ULONG_PTR) & ((PAGE_SIZE * 512) - 1)) as i64;
            return PhysicalAddress;
        }

        TempPte = *MiAddressToPte(Address);
        if TempPte.valid() {
            PhysicalAddress.QuadPart = ((TempPte.page_frame_number() as i64) << PAGE_SHIFT as i64) +
                                       ((Address as ULONG_PTR) & (PAGE_SIZE - 1)) as i64;
            return PhysicalAddress;
        }
    }

    KeRosDumpStackFrames(core::ptr::null_mut(), 20);
    PhysicalAddress.QuadPart = 0;
    PhysicalAddress
}

// === REMOVED: duplicate MI_MAPPED_COPY_PAGES, MI_POOL_COPY_BYTES, MI_MAX_TRANSFER_SIZE (head version retained) ===

// === REMOVED: duplicate MiProtectVirtualMemory (head extern declaration retained) ===

// === REMOVED: duplicate MiMakePdeExistAndMakeValid (head version retained) ===

// === REMOVED: duplicate MiProcessValidPteList (head version retained) ===

// === REMOVED: duplicate MiDecommitPages (head version retained) ===

// === REMOVED: duplicate MmGetVirtualForPhysical (head version retained) ===

// === REMOVED: duplicate MmSecureVirtualMemory (head version retained) ===

// === REMOVED: duplicate MmUnsecureVirtualMemory (head version retained) ===

#[no_mangle]
pub unsafe extern "system" fn NtReadVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    Buffer: PVOID,
    NumberOfBytesToRead: SIZE_T,
    NumberOfBytesRead: PSIZE_T,
) -> NTSTATUS {
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut BytesRead: SIZE_T = 0;

    if PreviousMode != 0i32 as i8 {
        if (((BaseAddress as ULONG_PTR) + NumberOfBytesToRead) < (BaseAddress as ULONG_PTR)) ||
           (((Buffer as ULONG_PTR) + NumberOfBytesToRead) < (Buffer as ULONG_PTR)) ||
           (((BaseAddress as ULONG_PTR) + NumberOfBytesToRead) > MmUserProbeAddress) ||
           (((Buffer as ULONG_PTR) + NumberOfBytesToRead) > MmUserProbeAddress) {
            return STATUS_ACCESS_VIOLATION;
        }

        if !NumberOfBytesRead.is_null() {
            ProbeForWriteSize_t(NumberOfBytesRead);
        }
    }

    if NumberOfBytesToRead != 0 {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_READ,
            PsProcessType,
            PreviousMode,
            &mut Process as *mut PEPROCESS as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            Status = MmCopyVirtualMemory(
                Process,
                BaseAddress,
                PsGetCurrentProcess() as PEPROCESS,
                Buffer,
                NumberOfBytesToRead as SIZE_T,
                PreviousMode,
                &mut BytesRead as *mut SIZE_T,
            );

            ObDereferenceObject(Process as PVOID);
        }
    }

    if !NumberOfBytesRead.is_null() {
        *NumberOfBytesRead = BytesRead;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWriteVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    Buffer: PVOID,
    NumberOfBytesToWrite: SIZE_T,
    NumberOfBytesWritten: PSIZE_T,
) -> NTSTATUS {
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut BytesWritten: SIZE_T = 0;

    if PreviousMode != 0i32 as i8 {
        if (((BaseAddress as ULONG_PTR) + NumberOfBytesToWrite) < (BaseAddress as ULONG_PTR)) ||
           (((Buffer as ULONG_PTR) + NumberOfBytesToWrite) < (Buffer as ULONG_PTR)) ||
           (((BaseAddress as ULONG_PTR) + NumberOfBytesToWrite) > MmUserProbeAddress) ||
           (((Buffer as ULONG_PTR) + NumberOfBytesToWrite) > MmUserProbeAddress) {
            return STATUS_ACCESS_VIOLATION;
        }

        if !NumberOfBytesWritten.is_null() {
            ProbeForWriteSize_t(NumberOfBytesWritten);
        }
    }

    if NumberOfBytesToWrite != 0 {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_WRITE,
            PsProcessType,
            PreviousMode,
            &mut Process as *mut PEPROCESS as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            Status = MmCopyVirtualMemory(
                PsGetCurrentProcess() as PEPROCESS,
                Buffer,
                Process,
                BaseAddress,
                NumberOfBytesToWrite as SIZE_T,
                PreviousMode,
                &mut BytesWritten as *mut SIZE_T,
            );

            ObDereferenceObject(Process as PVOID);
        }
    }

    if !NumberOfBytesWritten.is_null() {
        *NumberOfBytesWritten = BytesWritten;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushInstructionCache(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    FlushSize: SIZE_T,
) -> NTSTATUS {
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut Process: PKPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if !BaseAddress.is_null() {
        if FlushSize == 0 {
            return STATUS_SUCCESS;
        }

        if ExGetPreviousMode() != 0i32 as i8 {
            if BaseAddress > MmHighestUserAddress {
                return STATUS_ACCESS_VIOLATION;
            }
        }
    }

    if ProcessHandle != NtCurrentProcess() {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_WRITE,
            PsProcessType,
            ExGetPreviousMode(),
            &mut Process as *mut PKPROCESS as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }

        KeStackAttachProcess(Process, &mut ApcState as *mut _KAPC_STATE);
    }

    KeSweepICache(BaseAddress, FlushSize);

    if ProcessHandle != NtCurrentProcess() {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
        ObDereferenceObject(Process as PVOID);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtProtectVirtualMemory(
    ProcessHandle: HANDLE,
    UnsafeBaseAddress: *mut PVOID,
    UnsafeNumberOfBytesToProtect: PSIZE_T,
    NewAccessProtection: ULONG,
    UnsafeOldAccessProtection: PULONG,
) -> NTSTATUS {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut OldAccessProtection: ULONG = 0;
    let Protection: ULONG;
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    let mut BaseAddress: PVOID = core::ptr::null_mut();
    let mut NumberOfBytesToProtect: SIZE_T = 0;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut Attached: BOOLEAN = 0;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

    Protection = NewAccessProtection & !(PAGE_GUARD | PAGE_NOCACHE);
    if Protection != PAGE_NOACCESS &&
       Protection != PAGE_READONLY &&
       Protection != PAGE_READWRITE &&
       Protection != PAGE_WRITECOPY &&
       Protection != PAGE_EXECUTE &&
       Protection != PAGE_EXECUTE_READ &&
       Protection != PAGE_EXECUTE_READWRITE &&
       Protection != PAGE_EXECUTE_WRITECOPY {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    if PreviousMode != 0i32 as i8 {
        ProbeForWritePointer(UnsafeBaseAddress);
        ProbeForWriteSize_t(UnsafeNumberOfBytesToProtect);
        ProbeForWriteUlong(UnsafeOldAccessProtection);

        BaseAddress = *UnsafeBaseAddress;
        NumberOfBytesToProtect = *UnsafeNumberOfBytesToProtect;
    } else {
        BaseAddress = *UnsafeBaseAddress;
        NumberOfBytesToProtect = *UnsafeNumberOfBytesToProtect;
    }

    if BaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER_2;
    }

    if ((MmUserProbeAddress - (BaseAddress as ULONG_PTR)) < NumberOfBytesToProtect) {
        return STATUS_INVALID_PARAMETER_3;
    }

    if NumberOfBytesToProtect == 0 {
        return STATUS_INVALID_PARAMETER_3;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_VM_OPERATION,
        PsProcessType,
        PreviousMode,
        &mut Process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if CurrentProcess != Process {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
            &mut ApcState as *mut _KAPC_STATE,
        );
        Attached = 1;
    }

    Status = MiProtectVirtualMemory(
        Process,
        &mut BaseAddress as *mut PVOID,
        &mut NumberOfBytesToProtect as *mut SIZE_T,
        NewAccessProtection,
        &mut OldAccessProtection as *mut ULONG,
    );

    if Attached != 0 {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
    }

    ObDereferenceObject(Process as PVOID);

    *UnsafeOldAccessProtection = OldAccessProtection;
    *UnsafeBaseAddress = BaseAddress;
    *UnsafeNumberOfBytesToProtect = NumberOfBytesToProtect;

    Status
}

unsafe fn MI_IS_LOCKED_VA(Pfn1: PMMPFN, LockType: ULONG) -> BOOLEAN {
    let Wsle: PMMWSLE = core::ptr::addr_of_mut!((*(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN)).Wsle) as PMMWSLE;

    if ((LockType & MAP_PROCESS) != 0) && ((*Wsle).u1.e1.LockedInWs() != 0) {
        return 1;
    }
    if ((LockType & MAP_SYSTEM) != 0) && ((*Wsle).u1.e1.LockedInMemory() != 0) {
        return 1;
    }

    0
}

unsafe fn MI_LOCK_VA(Pfn1: PMMPFN, LockType: ULONG) {
    let Wsle: PMMWSLE = core::ptr::addr_of_mut!((*(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN)).Wsle) as PMMWSLE;

    if (*Wsle).u1.e1.LockedInWs() == 0 &&
       (*Wsle).u1.e1.LockedInMemory() == 0 {
        MiReferenceProbedPageAndBumpLockCount(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN);
    }

    if (LockType & MAP_PROCESS) != 0 {
        (*Wsle).u1.e1.set_LockedInWs(1);
    }
    if (LockType & MAP_SYSTEM) != 0 {
        (*Wsle).u1.e1.set_LockedInMemory(1);
    }
}

unsafe fn MI_UNLOCK_VA(Pfn1: PMMPFN, LockType: ULONG) {
    let Wsle: PMMWSLE = core::ptr::addr_of_mut!((*(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN)).Wsle) as PMMWSLE;

    if (LockType & MAP_PROCESS) != 0 {
        (*Wsle).u1.e1.set_LockedInWs(0);
    }
    if (LockType & MAP_SYSTEM) != 0 {
        (*Wsle).u1.e1.set_LockedInMemory(0);
    }

    if (*Wsle).u1.e1.LockedInWs() == 0 &&
       (*Wsle).u1.e1.LockedInMemory() == 0 {
        MiDereferencePfnAndDropLockCount(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN);
    }
}

unsafe fn MiCheckVadsForLockOperation(
    BaseAddress: *mut PVOID,
    RegionSize: PSIZE_T,
    EndAddress: *mut PVOID,
) -> NTSTATUS {
    let mut Vad: PMMVAD;
    let mut CurrentVa: PVOID;

    *EndAddress = ((*BaseAddress as ULONG_PTR) + *RegionSize) as PVOID;
    *EndAddress = ALIGN_UP_POINTER_BY(*EndAddress, PAGE_SIZE);
    *BaseAddress = ALIGN_DOWN_POINTER_BY(*BaseAddress, PAGE_SIZE);

    CurrentVa = *BaseAddress;
    while (CurrentVa as ULONG_PTR) < (*EndAddress as ULONG_PTR) {
        Vad = MiLocateAddress(CurrentVa);
        if Vad.is_null() {
            return STATUS_ACCESS_VIOLATION;
        }

        if ((*Vad).u.VadFlags.VadType() as u32 != VadNone as u32) &&
           ((*Vad).u.VadFlags.VadType() as u32 != VadImageMap as u32) &&
           ((*Vad).u.VadFlags.VadType() as u32 != VadWriteWatch as u32) {
            *EndAddress = CurrentVa;
            *RegionSize = ((*EndAddress as ULONG_PTR) - (*BaseAddress as ULONG_PTR)) as SIZE_T;
            return STATUS_INCOMPATIBLE_FILE_MAP;
        }

        CurrentVa = ((((*Vad).EndingVpn + 1) as ULONG_PTR) << PAGE_SHIFT) as PVOID;
    }

    *RegionSize = ((*EndAddress as ULONG_PTR) - (*BaseAddress as ULONG_PTR)) as SIZE_T;
    STATUS_SUCCESS
}

unsafe fn MiLockVirtualMemory(
    BaseAddress: *mut PVOID,
    RegionSize: PSIZE_T,
    MapType: ULONG,
) -> NTSTATUS {
    let CurrentProcess: PEPROCESS;
    let AddressSpace: PMMSUPPORT;
    let mut CurrentVa: PVOID;
    let mut EndAddress: PVOID = core::ptr::null_mut();
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    let mut PointerPpe: PMMPPE;
    #[cfg(target_arch = "aarch64")]
    let mut PointerPxe: PMMPXE;
    let mut Pfn1: PMMPFN;
    let mut Status: NTSTATUS;
    let mut TempStatus: NTSTATUS;

    AddressSpace = MmGetCurrentAddressSpace();
    MmLockAddressSpace(AddressSpace as PVOID);

    CurrentProcess = PsGetCurrentProcess() as PEPROCESS;
    if (*(CurrentProcess as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        Status = STATUS_PROCESS_IS_TERMINATING;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    Status = MiCheckVadsForLockOperation(BaseAddress, RegionSize, &mut EndAddress as *mut PVOID);
    if Status < 0 {
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    CurrentVa = *BaseAddress;
    while (CurrentVa as ULONG_PTR) < (EndAddress as ULONG_PTR) {
        core::ptr::read_volatile(CurrentVa as *const u8);
        CurrentVa = ((CurrentVa as ULONG_PTR) + PAGE_SIZE) as PVOID;
    }

    Status = STATUS_SUCCESS;

    PointerPte = MiAddressToPte(*BaseAddress);
    PointerPde = MiAddressToPde(*BaseAddress);
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        PointerPpe = MiAddressToPpe(*BaseAddress);
    }
    #[cfg(target_arch = "aarch64")]
    {
        PointerPxe = MiAddressToPxe(*BaseAddress);
    }

    LastPte = MiAddressToPte(((EndAddress as ULONG_PTR) - 1) as PVOID);

    MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

    loop {
        #[cfg(target_arch = "aarch64")]
        {
            while !(*PointerPxe).valid() ||
                  !(*PointerPpe).valid() ||
                  !(*PointerPde).valid() ||
                  !(*PointerPte).valid() {
                MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

                CurrentVa = MiPteToAddress(PointerPte);

                TempStatus = MmAccessFault(1u32, CurrentVa, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
                if TempStatus < 0 {
                    debug_assert!(false);
                    Status = TempStatus;
                    MmUnlockAddressSpace(AddressSpace as PVOID);
                    return Status;
                }

                MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
            }
        }
        #[cfg(target_arch = "x86_64")]
        {
            while !(*PointerPpe).valid() ||
                  !(*PointerPde).valid() ||
                  !(*PointerPte).valid() {
                MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

                CurrentVa = MiPteToAddress(PointerPte);

                TempStatus = MmAccessFault(1u32, CurrentVa, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
                if TempStatus < 0 {
                    debug_assert!(false);
                    Status = TempStatus;
                    MmUnlockAddressSpace(AddressSpace as PVOID);
                    return Status;
                }

                MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            while !(*PointerPde).valid() ||
                  !(*PointerPte).valid() {
                MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

                CurrentVa = MiPteToAddress(PointerPte);

                TempStatus = MmAccessFault(1u32, CurrentVa, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
                if TempStatus < 0 {
                    debug_assert!(false);
                    Status = TempStatus;
                    MmUnlockAddressSpace(AddressSpace as PVOID);
                    return Status;
                }

                MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
            }
        }

        Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
        debug_assert!(!Pfn1.is_null());

        if MI_IS_LOCKED_VA(Pfn1, MapType) != 0 {
            Status = STATUS_WAS_LOCKED;
        }

        MI_LOCK_VA(Pfn1, MapType);

        PointerPte = PointerPte.add(1);

        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = PointerPde.add(1);
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            if MiIsPteOnPpeBoundary(PointerPte) {
                PointerPpe = PointerPpe.add(1);
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if MiIsPteOnPxeBoundary(PointerPte) {
                PointerPxe = PointerPxe.add(1);
            }
        }

        if (PointerPte as ULONG_PTR) > (LastPte as ULONG_PTR) {
            break;
        }
    }

    MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

    MmUnlockAddressSpace(AddressSpace as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtLockVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    NumberOfBytesToLock: PSIZE_T,
    MapType: ULONG,
) -> NTSTATUS {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    let mut Status: NTSTATUS;
    let mut Attached: BOOLEAN = 0;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedBaseAddress: PVOID = core::ptr::null_mut();
    let mut CapturedBytesToLock: SIZE_T = 0;

    if (MapType & !(MAP_PROCESS | MAP_SYSTEM)) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (MapType & (MAP_PROCESS | MAP_SYSTEM)) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    ProbeForWritePointer(BaseAddress);
    ProbeForWriteSize_t(NumberOfBytesToLock);

    CapturedBaseAddress = *BaseAddress;
    CapturedBytesToLock = *NumberOfBytesToLock;

    if CapturedBaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER;
    }

    if ((MmUserProbeAddress - (CapturedBaseAddress as ULONG_PTR)) < CapturedBytesToLock) {
        return STATUS_INVALID_PARAMETER;
    }

    if CapturedBytesToLock == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_VM_OPERATION,
        PsProcessType,
        PreviousMode,
        &mut Process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (MapType & MAP_SYSTEM) != 0 {
        if SeSinglePrivilegeCheck(SeLockMemoryPrivilege, PreviousMode) == 0 {
            ObDereferenceObject(Process as PVOID);
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }

    if CurrentProcess != Process {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
            &mut ApcState as *mut _KAPC_STATE,
        );
        Attached = 1;
    }

    Status = MiLockVirtualMemory(
        &mut CapturedBaseAddress as *mut PVOID,
        &mut CapturedBytesToLock as *mut SIZE_T,
        MapType,
    );

    if Attached != 0 {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
    }

    ObDereferenceObject(Process as PVOID);

    *BaseAddress = CapturedBaseAddress;
    *NumberOfBytesToLock = CapturedBytesToLock;

    Status
}

unsafe fn MiUnlockVirtualMemory(
    BaseAddress: *mut PVOID,
    RegionSize: PSIZE_T,
    MapType: ULONG,
) -> NTSTATUS {
    let CurrentProcess: PEPROCESS;
    let AddressSpace: PMMSUPPORT;
    let mut EndAddress: PVOID = core::ptr::null_mut();
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    let mut PointerPpe: PMMPPE;
    #[cfg(target_arch = "aarch64")]
    let mut PointerPxe: PMMPXE;
    let mut Pfn1: PMMPFN;
    let mut Status: NTSTATUS;

    AddressSpace = MmGetCurrentAddressSpace();
    MmLockAddressSpace(AddressSpace as PVOID);

    CurrentProcess = PsGetCurrentProcess() as PEPROCESS;
    if (*(CurrentProcess as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        Status = STATUS_PROCESS_IS_TERMINATING;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    Status = MiCheckVadsForLockOperation(BaseAddress, RegionSize, &mut EndAddress as *mut PVOID);

    if Status == STATUS_ACCESS_VIOLATION {
        Status = STATUS_NOT_LOCKED;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    PointerPte = MiAddressToPte(*BaseAddress);
    PointerPde = MiAddressToPde(*BaseAddress);
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        PointerPpe = MiAddressToPpe(*BaseAddress);
    }
    #[cfg(target_arch = "aarch64")]
    {
        PointerPxe = MiAddressToPxe(*BaseAddress);
    }

    LastPte = MiAddressToPte(((EndAddress as ULONG_PTR) - 1) as PVOID);

    MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

    loop {
        #[cfg(target_arch = "aarch64")]
        {
            if !(*PointerPxe).valid() ||
               !(*PointerPpe).valid() ||
               !(*PointerPde).valid() ||
               !(*PointerPte).valid() {
                Status = STATUS_NOT_LOCKED;
            } else {
                Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
                debug_assert!(!Pfn1.is_null());

                if (((MapType & MAP_SYSTEM) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_SYSTEM) == 0)) ||
                   (((MapType & MAP_PROCESS) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS) == 0)) {
                    Status = STATUS_NOT_LOCKED;

                    if MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS | MAP_SYSTEM) == 0 {
                        // Should remove page from WS
                    }
                }
            }
        }
        #[cfg(target_arch = "x86_64")]
        {
            if !(*PointerPpe).valid() ||
               !(*PointerPde).valid() ||
               !(*PointerPte).valid() {
                Status = STATUS_NOT_LOCKED;
            } else {
                Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
                debug_assert!(!Pfn1.is_null());

                if (((MapType & MAP_SYSTEM) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_SYSTEM) == 0)) ||
                   (((MapType & MAP_PROCESS) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS) == 0)) {
                    Status = STATUS_NOT_LOCKED;

                    if MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS | MAP_SYSTEM) == 0 {
                        // Should remove page from WS
                    }
                }
            }
        }
        #[cfg(target_arch = "x86")]
        {
            if !(*PointerPde).valid() ||
               !(*PointerPte).valid() {
                Status = STATUS_NOT_LOCKED;
            } else {
                Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
                debug_assert!(!Pfn1.is_null());

                if (((MapType & MAP_SYSTEM) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_SYSTEM) == 0)) ||
                   (((MapType & MAP_PROCESS) != 0) && (MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS) == 0)) {
                    Status = STATUS_NOT_LOCKED;

                    if MI_IS_LOCKED_VA(Pfn1, MAP_PROCESS | MAP_SYSTEM) == 0 {
                        // Should remove page from WS
                    }
                }
            }
        }

        PointerPte = PointerPte.add(1);

        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = PointerPde.add(1);
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            if MiIsPteOnPpeBoundary(PointerPte) {
                PointerPpe = PointerPpe.add(1);
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if MiIsPteOnPxeBoundary(PointerPte) {
                PointerPxe = PointerPxe.add(1);
            }
        }

        if (PointerPte as ULONG_PTR) > (LastPte as ULONG_PTR) {
            break;
        }
    }

    if Status == STATUS_NOT_LOCKED {
        MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
        MmUnlockAddressSpace(AddressSpace as PVOID);
        return Status;
    }

    PointerPte = MiAddressToPte(*BaseAddress);
    PointerPde = MiAddressToPde(*BaseAddress);
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        PointerPpe = MiAddressToPpe(*BaseAddress);
    }
    #[cfg(target_arch = "aarch64")]
    {
        PointerPxe = MiAddressToPxe(*BaseAddress);
    }

    loop {
        Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
        MI_UNLOCK_VA(Pfn1, MapType);

        PointerPte = PointerPte.add(1);

        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = PointerPde.add(1);
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            if MiIsPteOnPpeBoundary(PointerPte) {
                PointerPpe = PointerPpe.add(1);
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if MiIsPteOnPxeBoundary(PointerPte) {
                PointerPxe = PointerPxe.add(1);
            }
        }

        if (PointerPte as ULONG_PTR) > (LastPte as ULONG_PTR) {
            break;
        }
    }

    Status = STATUS_SUCCESS;

    MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
    MmUnlockAddressSpace(AddressSpace as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtUnlockVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    NumberOfBytesToUnlock: PSIZE_T,
    MapType: ULONG,
) -> NTSTATUS {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    let mut Status: NTSTATUS;
    let mut Attached: BOOLEAN = 0;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedBaseAddress: PVOID = core::ptr::null_mut();
    let mut CapturedBytesToUnlock: SIZE_T = 0;

    if (MapType & !(MAP_PROCESS | MAP_SYSTEM)) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (MapType & (MAP_PROCESS | MAP_SYSTEM)) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    ProbeForWritePointer(BaseAddress);
    ProbeForWriteSize_t(NumberOfBytesToUnlock);

    CapturedBaseAddress = *BaseAddress;
    CapturedBytesToUnlock = *NumberOfBytesToUnlock;

    if CapturedBaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER;
    }

    if ((MmUserProbeAddress - (CapturedBaseAddress as ULONG_PTR)) < CapturedBytesToUnlock) {
        return STATUS_INVALID_PARAMETER;
    }

    if CapturedBytesToUnlock == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_VM_OPERATION,
        PsProcessType,
        PreviousMode,
        &mut Process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (MapType & MAP_SYSTEM) != 0 {
        if SeSinglePrivilegeCheck(SeLockMemoryPrivilege, PreviousMode) == 0 {
            ObDereferenceObject(Process as PVOID);
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }

    if CurrentProcess != Process {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
            &mut ApcState as *mut _KAPC_STATE,
        );
        Attached = 1;
    }

    Status = MiUnlockVirtualMemory(
        &mut CapturedBaseAddress as *mut PVOID,
        &mut CapturedBytesToUnlock as *mut SIZE_T,
        MapType,
    );

    if Attached != 0 {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
    }

    ObDereferenceObject(Process as PVOID);

    *BaseAddress = CapturedBaseAddress;
    *NumberOfBytesToUnlock = CapturedBytesToUnlock;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    NumberOfBytesToFlush: PSIZE_T,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedBaseAddress: PVOID = core::ptr::null_mut();
    let mut CapturedBytesToFlush: SIZE_T = 0;
    let mut LocalStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if PreviousMode != 0i32 as i8 {
        ProbeForWritePointer(BaseAddress);
        ProbeForWriteSize_t(NumberOfBytesToFlush);
        ProbeForWriteIoStatusBlock(IoStatusBlock);

        CapturedBaseAddress = *BaseAddress;
        CapturedBytesToFlush = *NumberOfBytesToFlush;
    } else {
        CapturedBaseAddress = *BaseAddress;
        CapturedBytesToFlush = *NumberOfBytesToFlush;
    }

    if CapturedBaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER;
    }

    if ((MmUserProbeAddress - (CapturedBaseAddress as ULONG_PTR)) < CapturedBytesToFlush) {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_VM_OPERATION,
        PsProcessType,
        PreviousMode,
        &mut Process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    Status = MmFlushVirtualMemory(
        Process,
        &mut CapturedBaseAddress as *mut PVOID,
        &mut CapturedBytesToFlush as *mut SIZE_T,
        &mut LocalStatusBlock as *mut _IO_STATUS_BLOCK as PIO_STATUS_BLOCK,
    );

    ObDereferenceObject(Process as PVOID);

    *BaseAddress = PAGE_ALIGN(CapturedBaseAddress as usize) as PVOID;
    *NumberOfBytesToFlush = 0;
    *IoStatusBlock = LocalStatusBlock;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtGetWriteWatch(
    ProcessHandle: HANDLE,
    Flags: ULONG,
    BaseAddress: PVOID,
    RegionSize: SIZE_T,
    UserAddressArray: *mut PVOID,
    EntriesInUserAddressArray: PULONG_PTR,
    Granularity: PULONG,
) -> NTSTATUS {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut EndAddress: PVOID;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedEntryCount: ULONG_PTR = 0;

    if PreviousMode != 0i32 as i8 {
        if BaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
            return STATUS_INVALID_PARAMETER_2;
        }

        if ((MmUserProbeAddress - (BaseAddress as ULONG_PTR)) < RegionSize) {
            return STATUS_INVALID_PARAMETER_3;
        }

        ProbeForWriteSize_t(EntriesInUserAddressArray);
        ProbeForWriteUlong(Granularity);

        CapturedEntryCount = *EntriesInUserAddressArray;

        if CapturedEntryCount == 0 {
            return STATUS_INVALID_PARAMETER_5;
        }

        if CapturedEntryCount > (MAXULONG_PTR / core::mem::size_of::<ULONG_PTR>()) {
            return STATUS_INVALID_PARAMETER_5;
        }

        ProbeForWrite(
            UserAddressArray as PVOID,
            CapturedEntryCount * core::mem::size_of::<PVOID>(),
            core::mem::size_of::<PVOID>() as ULONG,
        );
    } else {
        CapturedEntryCount = *EntriesInUserAddressArray;
        debug_assert!(CapturedEntryCount != 0);
    }

    if ProcessHandle == NtCurrentProcess() {
        Process = PsGetCurrentProcess() as PEPROCESS;
    } else {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_OPERATION,
            PsProcessType,
            PreviousMode,
            &mut Process as *mut PEPROCESS as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    }

    EndAddress = ((BaseAddress as ULONG_PTR) + RegionSize - 1) as PVOID;
    if BaseAddress > EndAddress {
        if ProcessHandle != NtCurrentProcess() {
            ObDereferenceObject(Process as PVOID);
        }
        return STATUS_INVALID_PARAMETER_4;
    }

    if ProcessHandle != NtCurrentProcess() {
        ObDereferenceObject(Process as PVOID);
    }

    *EntriesInUserAddressArray = 0;
    *Granularity = PAGE_SIZE as ULONG;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtResetWriteWatch(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    RegionSize: SIZE_T,
) -> NTSTATUS {
    let mut EndAddress: PVOID;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    debug_assert!(KeGetCurrentIrql() == PASSIVE_LEVEL as u8);

    if BaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER_2;
    }

    if ((MmUserProbeAddress - (BaseAddress as ULONG_PTR)) < RegionSize) {
        return STATUS_INVALID_PARAMETER_3;
    }

    if ProcessHandle == NtCurrentProcess() {
        Process = PsGetCurrentProcess() as PEPROCESS;
    } else {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_OPERATION,
            PsProcessType,
            PreviousMode,
            &mut Process as *mut PEPROCESS as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    }

    EndAddress = ((BaseAddress as ULONG_PTR) + RegionSize - 1) as PVOID;
    if BaseAddress > EndAddress {
        if ProcessHandle != NtCurrentProcess() {
            ObDereferenceObject(Process as PVOID);
        }
        return STATUS_INVALID_PARAMETER_3;
    }

    if ProcessHandle != NtCurrentProcess() {
        ObDereferenceObject(Process as PVOID);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    MemoryInformationClass: MEMORY_INFORMATION_CLASS,
    MemoryInformation: PVOID,
    MemoryInformationLength: SIZE_T,
    ReturnLength: PSIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let PreviousMode: KPROCESSOR_MODE;

    if BaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER;
    }

    PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0i32 as i8 {
        ProbeForWrite(
            MemoryInformation,
            MemoryInformationLength,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
        );

        if !ReturnLength.is_null() {
            ProbeForWriteSize_t(ReturnLength);
        }
    }

    match MemoryInformationClass {
        MemoryBasicInformation => {
            if MemoryInformationLength < core::mem::size_of::<MEMORY_BASIC_INFORMATION>() {
                return STATUS_INFO_LENGTH_MISMATCH;
            }
            Status = MiQueryMemoryBasicInformation(
                ProcessHandle,
                BaseAddress,
                MemoryInformation,
                MemoryInformationLength,
                ReturnLength,
            );
        }
        MemorySectionName => {
            if MemoryInformationLength < core::mem::size_of::<MEMORY_SECTION_NAME_T>() {
                return STATUS_INFO_LENGTH_MISMATCH;
            }
            Status = MiQueryMemorySectionName(
                ProcessHandle,
                BaseAddress,
                MemoryInformation,
                MemoryInformationLength as ntkrnl_sys::ddk::SIZE_T,
                ReturnLength as *mut ntkrnl_sys::ddk::SIZE_T,
            );
        }
        _ => {}
    }

    Status
}

// === REMOVED: duplicate NtAllocateVirtualMemory (head version retained) ===

// === REMOVED: duplicate NtFreeVirtualMemory (head version retained) ===

// === REMOVED: duplicate MmGetPhysicalAddress (head version retained) ===