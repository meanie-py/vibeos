use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::base::NT_SUCCESS;
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
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

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
    PMMVAD, PMMSUPPORT, PMMADDRESS_NODE, PSUBSECTION,
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
const VadAwe: u32 = 0x03;
const VadDevicePhysicalMemory: u32 = 1;
const VadLargePages: u32 = 0x05;
const VadImageMap: u32 = 2;
const VadRotatePhysical: u32 = 6;
const VadLargePageSection: u32 = 7;
const TableFoundNode: TABLE_SEARCH_RESULT = ntkrnl_sys::ddk::_TABLE_SEARCH_RESULT_TableFoundNode;
const HighPagePriority: u32 = 32; // header value; the hand copy said 2
const IoReadAccess: u32 = 0;

extern "system" {
    fn MmRebalanceMemoryConsumersAndWait();
    fn ObDereferenceObject(Object: PVOID);
    fn MmMdlPagesAreZero(Mdl: PMDL) -> BOOLEAN;
    fn MiFlushTbAndCapture(
        Vad: PMMVAD, PointerPte: PMMPTE, ProtectionMask: ULONG,
        Pfn1: PMMPFN, CaptureDirtyBit: BOOLEAN,
    );
    static SeLockMemoryPrivilege: ntkrnl_sys::ddk::_LUID;
}
extern "C" {
    fn MmInitializeMdl(Mdl: PMDL, BaseVa: PVOID, Length: SIZE_T);
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
const STATUS_WAS_LOCKED: NTSTATUS = 0x40000019;
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
    fn KeSweepICache(BaseAddress: PVOID, Length: SIZE_T);
    fn MiLockProcessWorkingSet(Process: PEPROCESS, Thread: PETHREAD);
    fn MiUnlockProcessWorkingSet(Process: PEPROCESS, Thread: PETHREAD);
    static MmUserProbeAddress: ULONG_PTR;
}

// ExFreePool + ExAllocatePoolZero are NTAPI (stdcall on i386) — pull from bindgen.
// (ExAllocatePoolZero used to be declared `extern "C"` here when the symbol
// was provided by a cdecl C trampoline; now that mm-util-inlines/util_inlines.rs
// owns the stdcall definition outright, use bindgen's correct stdcall import.)
use ntkrnl_sys::ddk::{ExAllocatePoolZero, ExFreePool};


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
    let Vad = opaque_ptr(Vad);
    let Process = opaque_ptr(Process);
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let mut OnPdeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPpeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPxe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPxeBoundary = true;

    PointerPte = MiAddressToPte(StartingAddress as PVOID);
    LastPte = MiAddressToPte(EndingAddress as PVOID);

    let mut CommittedPages = if (*Vad).u.VadFlags.MemCommit() != 0 {
        BYTES_TO_PAGES(EndingAddress - StartingAddress) as ULONG
    } else {
        0
    };

    while PointerPte <= LastPte {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
                    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        { OnPxeBoundary = MiIsPteOnPxeBoundary(PointerPte); }
    }

    CommittedPages
}

#[no_mangle]
pub unsafe extern "system" fn MiMakeSystemAddressValid(
    PageTableVirtualAddress: PVOID,
    CurrentProcess: PEPROCESS,
) -> ULONG {
    let PageTableVirtualAddress = opaque_ptr(PageTableVirtualAddress);
    let CurrentProcess = opaque_ptr(CurrentProcess);
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
        // Out of pages is a wait, not an in-page error: the C bugchecks on any
        // failure ("should not fail"), which turned an exhausted VM into 0x7A.
        // The while re-tests validity after the wait.
        if Status == STATUS_NO_MEMORY {
            MmRebalanceMemoryConsumersAndWait();
        } else if Status < 0 {
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::KERNEL_DATA_INPAGE_ERROR,
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
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let mut Status: NTSTATUS;
    let mut LockChange = 0u32;

    while MmIsAddressValid(VirtualAddress) == 0 {
        MiReleasePfnLock(OldIrql);

        Status = MmAccessFault(0u32, VirtualAddress, 0i8, core::ptr::null_mut());
        // As MiMakeSystemAddressValid: out of pages is a wait, not 0x7A.
        if Status == STATUS_NO_MEMORY {
            MmRebalanceMemoryConsumersAndWait();
        } else if Status < 0 {
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::KERNEL_DATA_INPAGE_ERROR,
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
    let mut PointerPte = opaque_ptr(PointerPte);
    let ValidPages = opaque_ptr(ValidPages);
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
                MiDecrementShareCount(Pfn2 as ntkrnl_sys::ddk::PMMPFN, PageTableIndex as ntkrnl_sys::ddk::PFN_NUMBER);
                MI_SET_PFN_DELETED(Pfn1);
                MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);
                MI_ERASE_PTE(PointerPte);
                MiReleasePfnLock(OldIrql);
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
    let PointerPte = opaque_ptr(PointerPte);
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let CurrentProcess = opaque_ptr(CurrentProcess);
    let ProtoPteArg = opaque_ptr(ProtoPteArg);
    let mut Pfn1: PMMPFN;
    let TempPte: MMPTE;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut PointerPde: PMMPDE;

    TempPte = *PointerPte;

    if !TempPte.valid() {
        if TempPte.transition() {
            PageFrameIndex = PFN_FROM_PTE(core::ptr::addr_of!(TempPte) as PMMPTE);
            Pfn1 = MiGetPfnEntry(PageFrameIndex);

            if Pfn1.is_null() {
                MI_ERASE_PTE(PointerPte);
                return;
            }

            MI_ERASE_PTE(PointerPte);

            let pte_frame = (*Pfn1).pte_frame();
            let pte_frame_pfn = MiGetPfnEntry(pte_frame);
            if !pte_frame_pfn.is_null() {
                MiDecrementShareCount(pte_frame_pfn as ntkrnl_sys::ddk::PMMPFN, pte_frame as ntkrnl_sys::ddk::PFN_NUMBER);
            }

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

    if Pfn1.is_null() {
        MI_ERASE_PTE(PointerPte);
        return;
    }

    if (*Pfn1).prototype_pte() {
        PointerPde = MiPteToPde(PointerPte);
        if !(*PointerPde).valid() {
            #[cfg(target_pointer_width = "32")]
            {
                let check_status = MiCheckPdeForPagedPool(VirtualAddress);
                if check_status < 0 {
                    KeBugCheckEx(
                        ntkrnl_sys::bugcodes::MEMORY_MANAGEMENT,
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
                    ntkrnl_sys::bugcodes::MEMORY_MANAGEMENT,
                    0x61940,
                    PointerPte as ULONG_PTR,
                    (*PointerPte).Long,
                    VirtualAddress as ULONG_PTR,
                );
            }
        }

        PointerPde = MiPteToPde(PointerPte);
        let pde_pfn = MiGetPfnEntry((*PointerPde).page_frame_number());
        if !pde_pfn.is_null() {
            MiDecrementShareCount(
                pde_pfn as ntkrnl_sys::ddk::PMMPFN,
                (*PointerPde).page_frame_number() as ntkrnl_sys::ddk::PFN_NUMBER,
            );
        }
        MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);

        if (PointerPte <= MiHighestUserPte) && (ProtoPteArg != (*Pfn1).pte_address()) {
            if (PAGE_ALIGN(VirtualAddress as usize) as PVOID != USER_SHARED_DATA as PVOID) ||
                (MmHighestUserAddress as ULONG_PTR <= USER_SHARED_DATA)
            {
                // Sibling of the 0x401 back-pointer check below (already loud as
                // [PFNMISMATCH]): a user PTE whose prototype-PTE argument disagrees
                // with the PFN's recorded PteAddress. Same reuse signature, same
                // arm64 suppression, so arm it the same way.
                mm_guard_trip!(
                    "virtual.MiDeletePte proto-pte-mismatch(0x400)", 8,
                    "PointerPte" => PointerPte as ULONG_PTR,
                    "ProtoPteArg" => ProtoPteArg as ULONG_PTR,
                    "Pfn->PteAddress" => (*Pfn1).pte_address() as ULONG_PTR,
                    "VirtualAddress" => VirtualAddress as ULONG_PTR,
                );
                #[cfg(not(target_arch = "aarch64"))]
                {
                    KeBugCheckEx(
                        ntkrnl_sys::bugcodes::MEMORY_MANAGEMENT,
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
        // The PFN back-pointer must name the PTE we are tearing down. x86 bugchecks
        // here (MEMORY_MANAGEMENT/0x401); arm64 has silently REPAIRED the mismatch
        // since the Milestone-11 session-space bring-up, with a comment conceding
        // "pages are reused across session/process boundaries and the PFN database
        // doesn't track the new PTE correctly" — which is a verbatim description of
        // the §9.97 physical-page-reuse corruption (task #19). So arm64 has been
        // suppressing the one detector that fires AT the reuse.
        //
        // Keep the repair (removing it would turn a boot into a bugcheck), but make
        // it LOUD. A `[PFNMISMATCH]` line names the PTE and the stale back-pointer at
        // the moment of divergence — the forensics the free-side and alloc-side
        // tripwires in §9.97 both failed to produce. Silence is also a result: it
        // eliminates MiDeletePte as the reuse path for a few lines of probe.
        #[cfg(target_arch = "aarch64")]
        {
            if ((*Pfn1).pte_address() as ULONG_PTR & !0x1) != PointerPte as ULONG_PTR {
                extern "C" {
                    fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
                }
                ARM64_UART_TRACE_HEX(
                    b"[PFNMISMATCH] MiDeletePte PointerPte=\0".as_ptr(),
                    PointerPte as ULONG_PTR,
                );
                ARM64_UART_TRACE_HEX(
                    b"[PFNMISMATCH]   Pfn->PteAddress=\0".as_ptr(),
                    (*Pfn1).pte_address() as ULONG_PTR,
                );
                ARM64_UART_TRACE_HEX(
                    b"[PFNMISMATCH]   PTE.Long=\0".as_ptr(),
                    (*PointerPte).Long as ULONG_PTR,
                );
                ARM64_UART_TRACE_HEX(
                    b"[PFNMISMATCH]   PageFrameIndex=\0".as_ptr(),
                    PageFrameIndex as ULONG_PTR,
                );
            }
            (*Pfn1).set_pte_address(PointerPte);
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if ((*Pfn1).pte_address() as ULONG_PTR & !0x1) != PointerPte as ULONG_PTR {
                KeBugCheckEx(
                    ntkrnl_sys::bugcodes::MEMORY_MANAGEMENT,
                    0x401,
                    PointerPte as ULONG_PTR,
                    (*PointerPte).Long,
                    (*Pfn1).pte_address() as ULONG_PTR,
                );
            }
        }

        MI_ERASE_PTE(PointerPte);
        let pte_frame = (*Pfn1).pte_frame();
        let pte_frame_pfn = MiGetPfnEntry(pte_frame);
        if !pte_frame_pfn.is_null() {
            MiDecrementShareCount(pte_frame_pfn as ntkrnl_sys::ddk::PMMPFN, pte_frame as ntkrnl_sys::ddk::PFN_NUMBER);
        }
        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, PageFrameIndex as ntkrnl_sys::ddk::PFN_NUMBER);
    }

    // All CPUs, not the C's KeFlushCurrentTb: see MiProcessValidPteList.
    ntkrnl_sys::inline::mm::KeFlushProcessTb();
}

#[no_mangle]
pub unsafe extern "system" fn MiDeleteVirtualAddresses(
    mut Va: ULONG_PTR,
    EndingAddress: ULONG_PTR,
    Vad: PMMVAD,
) {
    let Vad = opaque_ptr(Vad);
    let mut PointerPte: PMMPTE;
    let mut ProtoPtr: PMMPTE;
    let mut LastPrototypePte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPxe: PMMPTE;
    let mut TempPte: MMPTE;
    let CurrentProcess: PEPROCESS;
    let mut OldIrql: KIRQL;
    let mut AddressGap = 0u8;
    let mut _Subsection: PSUBSECTION;

    CurrentProcess = PsGetCurrentProcess() as *mut _EPROCESS;

    if !Vad.is_null() && ((*Vad).u.VadFlags.PrivateMemory() == 0) && !(*Vad).FirstPrototypePte.is_null() {
        ProtoPtr = (*Vad).FirstPrototypePte as PMMPTE;
        LastPrototypePte = (*Vad).FirstPrototypePte.add(1) as PMMPTE;
    } else {
        ProtoPtr = core::ptr::null_mut();
        LastPrototypePte = core::ptr::null_mut();
    }

    while Va <= EndingAddress {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
            _Subsection = MiLocateSubsection(Vad, (Va >> PAGE_SHIFT) as ntkrnl_sys::ddk::ULONG_PTR);
            if !_Subsection.is_null() {
                LastPrototypePte = ((*_Subsection).SubsectionBase as PMMPTE).add((*_Subsection).PtesInSubsection as usize);
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
                        _Subsection = MiLocateSubsection(Vad, (Va >> PAGE_SHIFT) as ntkrnl_sys::ddk::ULONG_PTR);
                        if !_Subsection.is_null() {
                            LastPrototypePte = ((*_Subsection).SubsectionBase as PMMPTE).add((*_Subsection).PtesInSubsection as usize);
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
    let SourceProcess = opaque_ptr(SourceProcess);
    let SourceAddress = opaque_ptr(SourceAddress);
    let TargetProcess = opaque_ptr(TargetProcess);
    let TargetAddress = opaque_ptr(TargetAddress);
    let ReturnSize = opaque_ptr(ReturnSize);
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
            // Validate-first: the C wrapped this in _SEH2_TRY/_SEH2_EXCEPT and the
            // translation kept the body and dropped the handler, so a bad pointer
            // raised into nothing and BUGCHECKed instead of returning a status.
            // Setting Status here is what makes the `if _FailedInProbe != 0`
            // branch below reachable again -- it is the C's discriminator between
            // a failed probe and a faulting copy, and nothing could set it.
            _FailedInProbe = 1;
            match ntkrnl_sys::probe::probe_user_buffer(
                SourceAddress as *const core::ffi::c_void,
                BufferSize as usize,
                core::mem::size_of::<i8>(),
                PreviousMode,
            ) {
                Ok(()) => _FailedInProbe = 0,
                Err(st) => Status = st,
            }
        }

        if Status >= 0 {
            MmInitializeMdl(Mdl as ntkrnl_sys::ddk::PMDL, CurrentAddress, CurrentSize);
            // The C's _SEH2_TRY covered this call, and MmProbeAndLockPages RAISES BY DESIGN on
            // a page it cannot lock, so the status-returning variant stands in for the handler.
            // Before it existed a user ReadProcessMemory over an unmapped page BUGCHECKed 0x1E.
            Status = crate::ARM3::mdlsup::MmTryProbeAndLockPages(Mdl as PMDL, PreviousMode, IoReadAccess);
            if Status >= 0 {
                PagesLocked = 1;
            }
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
            // Validate-first: the C wrapped this in _SEH2_TRY/_SEH2_EXCEPT and the
            // translation kept the body and dropped the handler, so a bad pointer
            // raised into nothing and BUGCHECKed instead of returning a status.
            // Setting Status here is what makes the `if _FailedInProbe != 0`
            // branch below reachable again -- it is the C's discriminator between
            // a failed probe and a faulting copy, and nothing could set it.
            _FailedInProbe = 1;
            match ntkrnl_sys::probe::probe_user_buffer_write(
                TargetAddress as *const core::ffi::c_void,
                BufferSize as usize,
                core::mem::size_of::<i8>(),
                PreviousMode,
            ) {
                Ok(()) => _FailedInProbe = 0,
                Err(st) => Status = st,
            }
        }

        if Status >= 0 {
            // FAULT-PROTECTED, not a bare memcpy. The C's _SEH2_TRY covered the
            // RtlCopyMemory as well as the probe, and a range check proves where a
            // buffer IS, never that it stays mapped -- measured: converting only the
            // probe moved the BUGCHECK from ProbeForWrite into memcpy, same 0x1E, one
            // frame along. _FailedInProbe stays 0 here, which is what selects the C's
            // copy-faulted arm (STATUS_PARTIAL_COPY) over its probe-failed one.
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                CurrentTargetAddress as *mut core::ffi::c_void,
                MdlAddress as *const core::ffi::c_void,
                CurrentSize as usize,
            ) {
                Status = st;
            }
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = STATUS_PARTIAL_COPY;
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
    let SourceProcess = opaque_ptr(SourceProcess);
    let SourceAddress = opaque_ptr(SourceAddress);
    let TargetProcess = opaque_ptr(TargetProcess);
    let TargetAddress = opaque_ptr(TargetAddress);
    let ReturnSize = opaque_ptr(ReturnSize);
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
            // Validate-first: the C wrapped this in _SEH2_TRY/_SEH2_EXCEPT and the
            // translation kept the body and dropped the handler, so a bad pointer
            // raised into nothing and BUGCHECKed instead of returning a status.
            // Setting Status here is what makes the `if _FailedInProbe != 0`
            // branch below reachable again -- it is the C's discriminator between
            // a failed probe and a faulting copy, and nothing could set it.
            _FailedInProbe = 1;
            match ntkrnl_sys::probe::probe_user_buffer(
                SourceAddress as *const core::ffi::c_void,
                BufferSize as usize,
                core::mem::size_of::<i8>(),
                PreviousMode,
            ) {
                Ok(()) => _FailedInProbe = 0,
                Err(st) => Status = st,
            }
        }

        if Status >= 0 {
            // FAULT-PROTECTED, not a bare memcpy. The C's _SEH2_TRY covered the
            // RtlCopyMemory as well as the probe, and a range check proves where a
            // buffer IS, never that it stays mapped -- measured: converting only the
            // probe moved the BUGCHECK from ProbeForWrite into memcpy, same 0x1E, one
            // frame along. _FailedInProbe stays 0 here, which is what selects the C's
            // copy-faulted arm (STATUS_PARTIAL_COPY) over its probe-failed one.
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                PoolAddress as *mut core::ffi::c_void,
                CurrentAddress as *const core::ffi::c_void,
                CurrentSize as usize,
            ) {
                Status = st;
            }
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = STATUS_PARTIAL_COPY;
            }
            break;
        }

        KeStackAttachProcess(core::ptr::addr_of_mut!((*(TargetProcess as *mut _EPROCESS)).Pcb), core::ptr::addr_of_mut!(ApcState));

        Status = STATUS_SUCCESS;
        if CurrentTargetAddress == TargetAddress && PreviousMode != 0i8 {
            // Validate-first: the C wrapped this in _SEH2_TRY/_SEH2_EXCEPT and the
            // translation kept the body and dropped the handler, so a bad pointer
            // raised into nothing and BUGCHECKed instead of returning a status.
            // Setting Status here is what makes the `if _FailedInProbe != 0`
            // branch below reachable again -- it is the C's discriminator between
            // a failed probe and a faulting copy, and nothing could set it.
            _FailedInProbe = 1;
            match ntkrnl_sys::probe::probe_user_buffer_write(
                TargetAddress as *const core::ffi::c_void,
                BufferSize as usize,
                core::mem::size_of::<i8>(),
                PreviousMode,
            ) {
                Ok(()) => _FailedInProbe = 0,
                Err(st) => Status = st,
            }
        }

        if Status >= 0 {
            // FAULT-PROTECTED, not a bare memcpy. The C's _SEH2_TRY covered the
            // RtlCopyMemory as well as the probe, and a range check proves where a
            // buffer IS, never that it stays mapped -- measured: converting only the
            // probe moved the BUGCHECK from ProbeForWrite into memcpy, same 0x1E, one
            // frame along. _FailedInProbe stays 0 here, which is what selects the C's
            // copy-faulted arm (STATUS_PARTIAL_COPY) over its probe-failed one.
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                CurrentTargetAddress as *mut core::ffi::c_void,
                PoolAddress as *const core::ffi::c_void,
                CurrentSize as usize,
            ) {
                Status = st;
            }
        }

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));

        if Status < 0 {
            if _FailedInProbe != 0 {
                *ReturnSize = BufferSize - RemainingSize;
            } else {
                *ReturnSize = 0;
                Status = STATUS_PARTIAL_COPY;
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
    let Va = opaque_ptr(Va);
    let Vad = opaque_ptr(Vad);
    let TargetProcess = opaque_ptr(TargetProcess);
    let ReturnedProtect = opaque_ptr(ReturnedProtect);
    let NextVa = opaque_ptr(NextVa);
    let mut PointerPte: PMMPTE;
    let mut ProtoPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    { PointerPxe = MiAddressToPxe(Va); }

    *NextVa = ((Va as ULONG_PTR) + PAGE_SIZE) as PVOID;

    loop {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let MemoryInformation = opaque_ptr(MemoryInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
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

    // The C names VadRotatePhysical and VadImageMap here; the translation had
    // invented 2 and 1, so every image view reported MEM_PRIVATE and a
    // device-physical view MEM_IMAGE.
    if ((*Vad).u.VadFlags.PrivateMemory() != 0)
        || ((*Vad).u.VadFlags.VadType() == ntkrnl_sys::ddk::_MI_VAD_TYPE_VadRotatePhysical as _)
    {
        MemoryInfo.Type = MEM_PRIVATE;
    } else if (*Vad).u.VadFlags.VadType() == ntkrnl_sys::ddk::_MI_VAD_TYPE_VadImageMap as _ {
        MemoryInfo.Type = 0x1000000; // MEM_IMAGE
    } else {
        MemoryInfo.Type = 0x40000; // MEM_MAPPED
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
    let Vad = opaque_ptr(Vad);
    let Process = opaque_ptr(Process);
    let mut PointerPte: PMMPTE;
    let LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let mut OnPdeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPpe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPpeBoundary = true;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut PointerPxe: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut OnPxeBoundary = true;

    PointerPte = MiAddressToPte(StartingAddress as PVOID);
    LastPte = MiAddressToPte(EndingAddress as PVOID);

    while PointerPte <= LastPte {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
                    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
    let TargetProcess = opaque_ptr(TargetProcess);
    let PointerPte: PMMPTE;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let PointerPpe = MiPdeToPpe(PointerPde);
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let PointerPxe = MiPdeToPxe(PointerPde);

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        if (*PointerPxe).valid() &&
           (*PointerPpe).valid() &&
           (*PointerPde).valid()
        {
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
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
    let ValidPteList = opaque_ptr(ValidPteList);
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

    // The C flushes only THIS processor (KeFlushCurrentTb), which is wrong the
    // moment the process has threads on another CPU: that CPU keeps translating
    // the decommitted VA to the frame just freed, so its writes land in a page
    // the zero-page thread is about to clear or MM has already handed to someone
    // else, while this CPU recommits the VA to a fresh zero page. arm64 -smp 2
    // regstress lost whole user-heap pages this way (free-list headers read back
    // as zero) and a kernel pool page once (BAD_POOL_HEADER). Every CPU must drop
    // the stale entry before the frames can be reused, i.e. before the PFN lock
    // is released.
    ntkrnl_sys::inline::mm::KeFlushProcessTb();
    MiReleasePfnLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn MiDecommitPages(
    mut StartingAddress: PVOID,
    EndingPte: PMMPTE,
    Process: PEPROCESS,
    Vad: PMMVAD,
) -> ULONG {
    let mut StartingAddress = opaque_ptr(StartingAddress);
    let EndingPte = opaque_ptr(EndingPte);
    let Process = opaque_ptr(Process);
    let Vad = opaque_ptr(Vad);
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
                } else if PteContents.transition() && !PteContents.prototype() {
                    // A trimmed page, still on the standby/modified list. The C asserts
                    // this away; overwriting the PTE leaves the PFN pointing at it and
                    // the frame leaked on its list for good. Detach it as delete does
                    // (MiDeletePte's transition path: PFN deleted, freed if
                    // unreferenced), then mark the PTE decommitted.
                    let OldIrql = MiAcquirePfnLock();
                    MiDeletePte(PointerPte, StartingAddress, Process, core::ptr::null_mut());
                    MiReleasePfnLock(OldIrql);
                    MI_WRITE_INVALID_PTE(PointerPte, MmDecommittedPte);
                } else {
                    // Demand-zero is the expected case. A paged-out or prototype PTE in
                    // a private range is not, and decommitting it would drop its
                    // backing silently: say so (the C ASSERTs).
                    if PteContents.prototype() || PteContents.page_frame_number() != 0 {
                        mm_guard_trip!(
                            "virtual.MiDecommitPages unexpected-invalid-pte", 8,
                            "PointerPte" => PointerPte as ULONG_PTR,
                            "Pte" => PteContents.Long as ULONG_PTR,
                            "VirtualAddress" => StartingAddress as ULONG_PTR,
                        );
                    }
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
    let SourceProcess = opaque_ptr(SourceProcess);
    let SourceAddress = opaque_ptr(SourceAddress);
    let TargetProcess = opaque_ptr(TargetProcess);
    let TargetAddress = opaque_ptr(TargetAddress);
    let ReturnSize = opaque_ptr(ReturnSize);
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
    let _Process = opaque_ptr(_Process);
    let _BaseAddress = opaque_ptr(_BaseAddress);
    let _RegionSize = opaque_ptr(_RegionSize);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
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
    let mut PointerPte = opaque_ptr(PointerPte);
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
                MiMakeSystemAddressValidPfn(PointerPte as PVOID, OldIrql);  // nolint: irql-balance (drops the PFN lock via OldIrql before it faults or waits, retakes it after)
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
    let Process = opaque_ptr(Process);
    let BaseAddress = opaque_ptr(BaseAddress);
    let NumberOfBytesToProtect = opaque_ptr(NumberOfBytesToProtect);
    let OldAccessProtection = opaque_ptr(OldAccessProtection);
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
                    // [guard] let OldIrql = MiAcquirePfnLock();
                    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

                    // A transition PTE built from scratch, not the valid one
                    // with Valid cleared as the C does (virtual.c): the C relies on
                    // x86, where no hardware bit lands on a software field that
                    // matters. On arm64 hardware bit 10 is the Access Flag and
                    // software bit 10 is Prototype, so every touched page made
                    // NOACCESS read back as a prototype PTE; the next
                    // MiGetPageProtection chased the "prototype pointer" with the
                    // PFN lock held, faulted, blocked, and wedged the machine
                    // (page heap's guard pages did it at once).
                    let Page = PFN_FROM_PTE(&mut PteContents as PMMPTE);
                    // The page leaves the working set here, so its dirty bit must
                    // reach the PFN now (as TrimWsList does) or a written page
                    // goes to the standby list as clean and can be repurposed.
                    if MI_PTE_IS_DIRTY(PteContents.Long as u64) {
                        (*Pfn1).set_modified(true);
                    }
                    MI_MAKE_TRANSITION_PTE(&mut PteContents, Page, ProtectionMask);
                    // Decrease PFN share count and write the PTE
                    MiDecrementShareCount(Pfn1 as ntkrnl_sys::ddk::PMMPFN, Page as ntkrnl_sys::ddk::PFN_NUMBER);
                    // FIXME: remove the page from the WS
                    MI_WRITE_INVALID_PTE(PointerPte, PteContents);
                    KeInvalidateTlbEntry(MiPteToAddress(PointerPte));

                    // We are done for this PTE
                    drop(pfn_guard);
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
    let Address = opaque_ptr(Address);
    Address
}

#[no_mangle]
pub unsafe extern "system" fn MmUnsecureVirtualMemory(
    _SecureMem: PVOID,
) {
    let _SecureMem = opaque_ptr(_SecureMem);
}

#[no_mangle]
pub unsafe extern "system" fn NtAllocateVirtualMemory(
    ProcessHandle: HANDLE,
    UBaseAddress: *mut PVOID,
    ZeroBits: ULONG_PTR,
    URegionSize: *mut SIZE_T,
    AllocationType: ULONG,
    Protect: ULONG,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let UBaseAddress = opaque_ptr(UBaseAddress);
    let URegionSize = opaque_ptr(URegionSize);

    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Vad: PMMVAD = core::ptr::null_mut();
    let mut FoundVad: PMMVAD = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let AddressSpace: PMMSUPPORT;
    let PBaseAddress: PVOID;
    let mut PRegionSize: ULONG_PTR;
    let mut StartingAddress: ULONG_PTR;
    let mut EndingAddress: ULONG_PTR;
    let mut HighestAddress: ULONG_PTR = MM_HIGHEST_VAD_ADDRESS;
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let CurrentThread: PETHREAD = PsGetCurrentThread();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let ProtectionMask: ULONG;
    let mut QuotaCharge: ULONG = 0;
    let mut QuotaFree: ULONG = 0;
    let mut Attached: BOOLEAN = 0;
    let mut ChangeProtection: BOOLEAN = 0;
    let mut QuotaCharged: BOOLEAN = 0;
    let mut TempPte: MMPTE = core::mem::zeroed();
    let mut PointerPte: PMMPTE;
    let mut LastPte: PMMPTE;
    let mut PointerPde: PMMPDE;
    let Result: TABLE_SEARCH_RESULT;

    /* Check for valid Zero bits. 64-bit NT semantics: a ZeroBits above 32 is
     * not a shift count but an address MASK — the allocation must land at or
     * below it (WOW64 layers pass (1<<31)-1 to force guest-reachable <2GB
     * allocations; FEX's BTCpuProcessInit BOP page depends on this). */
    let ZeroBitsIsMask =
        core::mem::size_of::<ULONG_PTR>() == 8 && ZeroBits > 32;
    if !ZeroBitsIsMask && ZeroBits > MI_MAX_ZERO_BITS as ULONG_PTR {
        return STATUS_INVALID_PARAMETER_3;
    }

    /* Check for valid Allocation Types */
    if (AllocationType
        & !(MEM_COMMIT | MEM_RESERVE | MEM_RESET | MEM_PHYSICAL
            | (MEM_TOP_DOWN as ULONG) | MEM_WRITE_WATCH | MEM_LARGE_PAGES))
        != 0
    {
        return STATUS_INVALID_PARAMETER_5;
    }

    /* Check for at least one of these Allocation Types to be set */
    if (AllocationType & (MEM_COMMIT | MEM_RESERVE | MEM_RESET)) == 0 {
        return STATUS_INVALID_PARAMETER_5;
    }

    /* MEM_RESET is an exclusive flag, make sure that is valid too */
    if (AllocationType & MEM_RESET) != 0 && AllocationType != MEM_RESET {
        return STATUS_INVALID_PARAMETER_5;
    }

    /* Check if large pages are being used */
    if (AllocationType & MEM_LARGE_PAGES) != 0 {
        if (AllocationType & MEM_COMMIT) == 0 {
            return STATUS_INVALID_PARAMETER_5;
        }
        if (AllocationType & (MEM_PHYSICAL | MEM_RESET | MEM_WRITE_WATCH)) != 0 {
            return STATUS_INVALID_PARAMETER_5;
        }
    }

    /* MEM_WRITE_WATCH can only be used if MEM_RESERVE is also used */
    if (AllocationType & MEM_WRITE_WATCH) != 0 && (AllocationType & MEM_RESERVE) == 0 {
        return STATUS_INVALID_PARAMETER_5;
    }

    /* Check for valid MEM_PHYSICAL usage */
    if (AllocationType & MEM_PHYSICAL) != 0 {
        if (AllocationType & MEM_RESERVE) == 0 {
            return STATUS_INVALID_PARAMETER_5;
        }
        if (AllocationType & !(MEM_RESERVE | (MEM_TOP_DOWN as ULONG) | MEM_PHYSICAL)) != 0 {
            return STATUS_INVALID_PARAMETER_5;
        }
        if Protect != PAGE_READWRITE {
            return STATUS_INVALID_PARAMETER_6;
        }
    }

    /* Calculate the protection mask and make sure it's valid */
    ProtectionMask = MiMakeProtectionMask(Protect);
    if ProtectionMask == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    /* Probe + capture user pointers (DUMMY_PSEH: no SEH wrap) */
    if PreviousMode != 0i32 as i8 {
        // C: inside _SEH2_TRY; the handler returns the exception code. No SEH here,
        // so the Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(UBaseAddress, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(URegionSize, 1, PreviousMode) {
            return status;
        }
    }
    PBaseAddress = match ntkrnl_sys::probe::read_back(UBaseAddress) { Ok(v) => v, Err(s) => return s };
    PRegionSize = match ntkrnl_sys::probe::read_back(URegionSize) { Ok(v) => v as ULONG_PTR, Err(s) => return s };

    /* Make sure the allocation isn't past the VAD area */
    if (PBaseAddress as ULONG_PTR) > MM_HIGHEST_VAD_ADDRESS {
        return STATUS_INVALID_PARAMETER_2;
    }

    /* Make sure the allocation wouldn't overflow past the VAD area */
    if (MM_HIGHEST_VAD_ADDRESS + 1 - (PBaseAddress as ULONG_PTR)) < PRegionSize {
        return STATUS_INVALID_PARAMETER_4;
    }

    /* Make sure there's a size specified */
    if PRegionSize == 0 {
        return STATUS_INVALID_PARAMETER_4;
    }

    /* If this is for the current process, just use PsGetCurrentProcess */
    if ProcessHandle == NtCurrentProcess() as HANDLE {
        Process = CurrentProcess;
    } else {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_OPERATION,
            PsProcessType,
            PreviousMode,
            core::ptr::addr_of_mut!(Process) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
        if CurrentProcess != Process {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
                core::ptr::addr_of_mut!(ApcState),
            );
            Attached = 1;
        }
    }

    /* MEM_LARGE_PAGES requires SeLockMemoryPrivilege */
    if (AllocationType & MEM_LARGE_PAGES) != 0
        && SeSinglePrivilegeCheck(SeLockMemoryPrivilege, PreviousMode) == 0
    {
        Status = STATUS_PRIVILEGE_NOT_HELD;
        // jump to FailPathNoLock
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        return Status;
    }

    /* Reject things we don't yet support */
    if (AllocationType & MEM_LARGE_PAGES) == MEM_LARGE_PAGES
        || (AllocationType & MEM_PHYSICAL) == MEM_PHYSICAL
        || (AllocationType & MEM_WRITE_WATCH) == MEM_WRITE_WATCH
    {
        Status = STATUS_INVALID_PARAMETER;
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        return Status;
    }

    /* Reservation / blind-commit path */
    if PBaseAddress.is_null() || (AllocationType & MEM_RESERVE) != 0 {
        if (Protect & (PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY)) != 0 {
            Status = STATUS_INVALID_PAGE_PROTECTION;
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }

        if PBaseAddress.is_null() {
            /* Blind commit — kernel picks address */
            PRegionSize = ROUND_TO_PAGES(PRegionSize as usize) as ULONG_PTR;
            EndingAddress = 0;
            StartingAddress = 0;

            if ZeroBitsIsMask {
                /* Mask form: constrain the top; a mask above the VAD ceiling
                 * is simply no constraint. */
                if ZeroBits < MM_HIGHEST_VAD_ADDRESS {
                    HighestAddress = ZeroBits;
                }
            } else if ZeroBits != 0 {
                /* NT counts the zero bits of a 32-bit address on every architecture
                 * (MM_USER_ADDRESS_RANGE_LIMIT is 0xFFFFFFFF on x64 too) and clamps the
                 * result to the VAD ceiling. The C shifted MAXULONG_PTR and failed
                 * STATUS_INVALID_PARAMETER_3 above the ceiling, so ZeroBits 1 failed on
                 * i386 and 1..21 meant nothing on 64-bit (kmtest ZwAllocateVirtualMemory's
                 * ZERO BITS TESTS and ZeroBitsTest, written against Windows). */
                HighestAddress = (0xFFFF_FFFFu32 as ULONG_PTR) >> ZeroBits;
                if HighestAddress > MM_HIGHEST_VAD_ADDRESS {
                    HighestAddress = MM_HIGHEST_VAD_ADDRESS;
                }
            }
        } else {
            /* Reservation at a specific address — align to 64 KB granularity */
            EndingAddress =
                ((PBaseAddress as ULONG_PTR) + PRegionSize - 1) | (PAGE_SIZE - 1) as ULONG_PTR;
            PRegionSize = EndingAddress + 1
                - ((PBaseAddress as ULONG_PTR) & !((_64K as ULONG_PTR) - 1));
            StartingAddress = PBaseAddress as ULONG_PTR;
        }

        /* Commit charge for a reserve-and-commit: the whole new VAD, taken before
         * anything is built so running out fails here (commit.rs). */
        let mut CommitPages: usize = 0;
        if (AllocationType & MEM_COMMIT) != 0 {
            CommitPages = (PRegionSize as usize) >> PAGE_SHIFT;
            if !super::commit::MiChargeCommitment(CommitPages, Process) {
                if Attached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                if ProcessHandle != NtCurrentProcess() as HANDLE {
                    ObDereferenceObject(Process as PVOID);
                }
                return super::commit::STATUS_COMMITMENT_LIMIT;
            }
        }

        /* Charge quotas for the VAD */
        Status = PsChargeProcessNonPagedPoolQuota(
            Process,
            core::mem::size_of::<MMVAD_LONG>() as _,
        );
        if Status < 0 {
            super::commit::MiReturnCommitment(CommitPages, Process);
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }

        QuotaCharged = 1;

        /* Allocate and initialize the VAD */
        Vad = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<MMVAD_LONG>() as _,
            u32::from_le_bytes(*b"VadS"),
        ) as PMMVAD;
        if Vad.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            super::commit::MiReturnCommitment(CommitPages, Process);
            if QuotaCharged != 0 {
                PsReturnProcessNonPagedPoolQuota(
                    Process,
                    core::mem::size_of::<MMVAD_LONG>() as _,
                );
            }
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }

        core::ptr::write_bytes(Vad as *mut u8, 0, core::mem::size_of::<MMVAD_LONG>());
        if (AllocationType & MEM_COMMIT) != 0 {
            (*Vad).u.VadFlags.set_MemCommit(1);
        }
        (*Vad).u.VadFlags.set_Protection(ProtectionMask as _);
        (*Vad).u.VadFlags.set_PrivateMemory(1);
        (*Vad).ControlArea = core::ptr::null_mut();

        /* Insert the VAD */
        Status = MiInsertVadEx(
            Vad,
            core::ptr::addr_of_mut!(StartingAddress) as _,
            PRegionSize as _,
            HighestAddress as _,
            MM_VIRTMEM_GRANULARITY as _,
            AllocationType,
        );
        if Status < 0 {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"VadS"));
            super::commit::MiReturnCommitment(CommitPages, Process);
            if QuotaCharged != 0 {
                PsReturnProcessNonPagedPoolQuota(
                    Process,
                    core::mem::size_of::<MMVAD_LONG>() as _,
                );
            }
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }

        /* The charge was for the size asked; the VAD is what was placed. Settle the
         * difference so a later release returns exactly what is held. */
        if CommitPages != 0 {
            let Placed = ((*Vad).EndingVpn as usize) - ((*Vad).StartingVpn as usize) + 1;
            if Placed < CommitPages {
                super::commit::MiReturnCommitment(CommitPages - Placed, Process);
                CommitPages = Placed;
            } else if Placed > CommitPages
                && super::commit::MiChargeCommitment(Placed - CommitPages, Process)
            {
                CommitPages = Placed;
            }
            (*Vad).u.VadFlags.set_CommitCharge(CommitPages as _);
        }

        // ARM64: clear stale bootloader identity-mapping PTEs over the newly
        // committed user range (ntoskrnl/mm/ARM3/virtual.c:4819-4869). FreeLDR
        // leaves TTBR0 identity entries behind — VA==PA, global, EL1-only — and
        // TTBR0 *is* the user half on ARM64, so they survive into user address
        // space. They are VALID descriptors, so two things go wrong: the fault
        // that should demand-page the new allocation never fires, and teardown
        // later reads the identity PTE and decodes a PFN that names no frame
        // (VA 0x50000 -> pte 0x50703 -> PFN 0x50, far below the 0x40000000 DRAM
        // base). MmDeleteRmap then reports mapping-not-in-list and
        // MmReleasePageMemoryConsumer hands the allocator a frame that was
        // never allocated.
        //
        // This block was dropped in translation behind "arm64 boot currently
        // fails before user-mode allocation, so this path isn't exercised".
        // That stopped being true once arm64 booted to desktop; the note
        // outlived its premise and the gap became a live memory corruptor.
        // amd64 has the equivalent guard in ARM3/sysldr.rs, which is why this
        // reproduced on arm64 only.
        #[cfg(target_arch = "aarch64")]
        {
            if (AllocationType & MEM_COMMIT) != 0
                && StartingAddress < MmSystemRangeStart as ULONG_PTR
            {
                // ARM64 descriptor bits[1:0]: 0b11 = valid table/page entry,
                // 0b01 = valid 2MB BLOCK. HARDWARE_PTE calls bit 1 NotLargePage
                // (sdk/include/ndk/arm64/mmtypes.h) — there is no shared
                // accessor because bit 1 is `Write` on x86.
                const NOT_LARGE_PAGE: ULONG_PTR = 1 << 1;
                let mut need_flush = false;
                let mut Va = StartingAddress;

                while Va < StartingAddress + PRegionSize {
                    let PointerPxe = MiAddressToPxe(Va as PVOID);
                    let PointerPpe = MiAddressToPpe(Va as PVOID);
                    let PointerPde = MiAddressToPde(Va as PVOID);

                    if (*PointerPxe).valid() && (*PointerPpe).valid() && (*PointerPde).valid() {
                        if ((*PointerPde).Long & NOT_LARGE_PAGE) == 0 {
                            // Block descriptor: drop it and skip its whole 2MB.
                            (*PointerPde).Long = 0;
                            need_flush = true;
                            Va |= 0x1FFFFF;
                        } else {
                            let PointerPte = MiAddressToPte(Va as PVOID);
                            if (*PointerPte).valid() {
                                (*PointerPte).Long = 0;
                                need_flush = true;
                            }
                        }
                    }

                    Va += PAGE_SIZE as ULONG_PTR;
                }

                if need_flush {
                    KeFlushEntireTb(1, 1);
                }
            }
        }

        /* Detach from target if needed and return success */
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }

        /* Write back base + size (DUMMY_PSEH: no SEH wrap; ignore failure) */
        let _ = ntkrnl_sys::probe::write_back(URegionSize, PRegionSize as SIZE_T);
        let _ = ntkrnl_sys::probe::write_back(UBaseAddress, StartingAddress as PVOID);
        return STATUS_SUCCESS;
    }

    /* MEM_COMMIT on top of an existing reservation. */
    EndingAddress =
        ((PBaseAddress as ULONG_PTR) + PRegionSize - 1) | (PAGE_SIZE - 1) as ULONG_PTR;
    StartingAddress = PAGE_ALIGN(PBaseAddress as usize) as ULONG_PTR;
    PRegionSize = EndingAddress - StartingAddress + 1;

    /* Lock the address space and verify the process isn't dying */
    AddressSpace = MmGetCurrentAddressSpace();
    MmLockAddressSpace(AddressSpace as PVOID);
    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        Status = STATUS_PROCESS_IS_TERMINATING;
        // jump to FailPath
        MmUnlockAddressSpace(AddressSpace as PVOID);
        if Vad.is_null() == false { /* unreachable here */ }
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        return Status;
    }

    /* Find the VAD */
    Result = MiCheckForConflictingNode(
        (StartingAddress >> PAGE_SHIFT) as _,
        (EndingAddress >> PAGE_SHIFT) as _,
        core::ptr::addr_of_mut!((*Process).VadRoot),
        core::ptr::addr_of_mut!(FoundVad) as *mut PMMADDRESS_NODE,
    );

    /* Unified failure tail (mirrors the C `FailPath` / `FailPathNoLock` convergence). */
    macro_rules! fail_path {
        ($status:expr) => {{
            Status = $status;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            if !Vad.is_null() {
                ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"VadS"));
            }
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }};
    }

    if Result != TableFoundNode {
        fail_path!(STATUS_CONFLICTING_ADDRESSES);
    }

    if (AllocationType & MEM_RESET) == MEM_RESET {
        // HACK: pretend success
        Status = STATUS_SUCCESS;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        if !Vad.is_null() {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"VadS"));
        }
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        return Status;
    }

    /* Reject AWE / DevicePhysical / LargePages VADs */
    let vad_type = (*FoundVad).u.VadFlags.VadType();
    if vad_type == VadAwe as _
        || vad_type == VadDevicePhysicalMemory as _
        || vad_type == VadLargePages as _
    {
        fail_path!(STATUS_CONFLICTING_ADDRESSES);
    }

    /* Range must fit inside the VAD */
    if ((StartingAddress >> PAGE_SHIFT) < (*FoundVad).StartingVpn as ULONG_PTR)
        || ((EndingAddress >> PAGE_SHIFT) > (*FoundVad).EndingVpn as ULONG_PTR)
    {
        fail_path!(STATUS_CONFLICTING_ADDRESSES);
    }

    /* Reject ROSMM (non-ARM3) sections */
    if MI_IS_ROSMM_VAD(FoundVad as PVOID) {
        fail_path!(STATUS_ALREADY_COMMITTED);
    }

    /* Section-backed (PrivateMemory == 0) commit path */
    if (*FoundVad).u.VadFlags.PrivateMemory() == 0 {
        if (*FoundVad).u.VadFlags.VadType() == VadLargePageSection as _ {
            fail_path!(STATUS_INVALID_PAGE_PROTECTION);
        }

        if (Protect & (PAGE_NOCACHE | PAGE_WRITECOMBINE)) != 0
            && (*FoundVad).u.VadFlags.VadType() != VadRotatePhysical as _
        {
            fail_path!(STATUS_INVALID_PAGE_PROTECTION);
        }

        /* Secured VAD check */
        if (*FoundVad).u.VadFlags.NoChange() != 0 {
            Status = MiCheckSecuredVad(
                FoundVad,
                PBaseAddress,
                PRegionSize as _,
                ProtectionMask,
            );
            if Status < 0 {
                fail_path!(Status);
            }
        }

        /* ARM3 only handles shared-memory sections (no file backing) */
        // ASSERT((*(*FoundVad).ControlArea).FilePointer.is_null());

        if (*FoundVad).u.VadFlags.VadType() == VadRotatePhysical as _
            && (Protect
                & (PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY | PAGE_NOACCESS | PAGE_GUARD))
                != 0
        {
            fail_path!(STATUS_INVALID_PAGE_PROTECTION);
        }

        /* Compute prototype-PTE range and quota */
        PointerPte =
            MI_GET_PROTOTYPE_PTE_FOR_VPN(FoundVad, StartingAddress >> PAGE_SHIFT);
        LastPte =
            MI_GET_PROTOTYPE_PTE_FOR_VPN(FoundVad, EndingAddress >> PAGE_SHIFT);
        QuotaCharge = (LastPte.offset_from(PointerPte) + 1) as ULONG;
        /* The section's charge (no process: it outlives any one mapper), taken for
         * the whole range and settled after the walk, like the private path. */
        if !super::commit::MiChargeCommitment(QuotaCharge as usize, core::ptr::null_mut()) {
            fail_path!(super::commit::STATUS_COMMITMENT_LIMIT);
        }
        KeAcquireGuardedMutexUnsafe(core::ptr::addr_of_mut!(MmSectionCommitMutex));

        /* Get the segment template PTE and walk pages */
        TempPte = *(core::ptr::addr_of!(
            (*(*(*FoundVad).ControlArea).Segment).SegmentPteTemplate
        ) as *const MMPTE);
        while PointerPte <= LastPte {
            if (*PointerPte).Long == 0 {
                MI_WRITE_INVALID_PTE(PointerPte, TempPte);
            } else {
                QuotaFree += 1;
            }
            PointerPte = PointerPte.add(1);
        }

        // ASSERT(QuotaCharge >= QuotaFree);
        QuotaCharge -= QuotaFree;
        (*(*(*FoundVad).ControlArea).Segment).NumberOfCommittedPages += QuotaCharge;
        KeReleaseGuardedMutexUnsafe(core::ptr::addr_of_mut!(MmSectionCommitMutex));
        super::commit::MiReturnCommitment(QuotaFree as usize, core::ptr::null_mut());

        Status = STATUS_SUCCESS;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        if !Vad.is_null() {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"VadS"));
        }
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        if NT_SUCCESS(Status) {
            let _ = ntkrnl_sys::probe::write_back(URegionSize, PRegionSize as SIZE_T);
            let _ = ntkrnl_sys::probe::write_back(UBaseAddress, StartingAddress as PVOID);
        }
        return Status;
    }

    /* Demand-zero commit for normal private VAD */
    // ASSERT(VadType == VadNone)

    if (Protect & PAGE_WRITECOPY) != 0 || (Protect & PAGE_EXECUTE_WRITECOPY) != 0 {
        fail_path!(STATUS_INVALID_PAGE_PROTECTION);
    }

    /* Build a demand-zero invalid PTE */
    TempPte = core::mem::zeroed();
    TempPte.set_soft_protection(ProtectionMask);

    /* Compute PTE/PDE range */
    PointerPde = MiAddressToPde(StartingAddress as PVOID);
    PointerPte = MiAddressToPte(StartingAddress as PVOID);
    LastPte = MiAddressToPte(EndingAddress as PVOID);

    /* Commit charge: the whole range up front, so a commit past the limit fails
     * before any PTE changes; the walk below counts the pages that really go from
     * uncommitted to committed and the rest is handed back (commit.rs). Upstream
     * added the full range to the process count whether or not it was already
     * committed, and never charged the system. */
    let pages = LastPte.offset_from(PointerPte) as ULONG_PTR + 1;
    if !super::commit::MiChargeCommitment(pages as usize, Process) {
        fail_path!(super::commit::STATUS_COMMITMENT_LIMIT);
    }
    let MemCommitVad = (*FoundVad).u.VadFlags.MemCommit() != 0;
    let mut NewlyCommitted: ULONG_PTR = 0;

    /* Lock the working set */
    MiLockProcessWorkingSetUnsafe(Process, CurrentThread);

    /* Make the first PDE valid and walk PTEs */
    MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);
    while PointerPte <= LastPte {
        if MiIsPteOnPdeBoundary(PointerPte) {
            PointerPde = MiPteToPde(PointerPte);
            MiMakePdeExistAndMakeValid(PointerPde, Process, MM_NOIRQL);
        }

        if (*PointerPte).Long == 0 {
            /* A blank PTE is already committed in a reserve-and-commit VAD. */
            if !MemCommitVad {
                NewlyCommitted += 1;
            }
            MiIncrementPageTableReferences(MiPteToAddress(PointerPte));
            MI_WRITE_INVALID_PTE(PointerPte, TempPte);
        } else if (*PointerPte).Long == MmDecommittedPte.Long {
            NewlyCommitted += 1;
            MI_WRITE_INVALID_PTE(PointerPte, TempPte);
        } else if ChangeProtection == 0 && Protect != MiGetPageProtection(PointerPte) {
            ChangeProtection = 1;
        }

        PointerPte = PointerPte.add(1);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, CurrentThread);
    Status = STATUS_SUCCESS;

    super::commit::MiReturnCommitment((pages - NewlyCommitted) as usize, Process);
    let new_commit_charge =
        (*FoundVad).u.VadFlags.CommitCharge() as ULONG_PTR + NewlyCommitted;
    (*FoundVad).u.VadFlags.set_CommitCharge(new_commit_charge as _);

    MmUnlockAddressSpace(AddressSpace as PVOID);

    if !NT_SUCCESS(Status) {
        if !Vad.is_null() {
            ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"VadS"));
        }
    }

    if ChangeProtection != 0 {
        let mut ProtectBaseAddress: PVOID = StartingAddress as PVOID;
        let mut ProtectSize: SIZE_T = PRegionSize as SIZE_T;
        let mut OldProtection: ULONG = 0;
        MiProtectVirtualMemory(
            Process,
            core::ptr::addr_of_mut!(ProtectBaseAddress),
            core::ptr::addr_of_mut!(ProtectSize),
            Protect,
            core::ptr::addr_of_mut!(OldProtection),
        );
    }

    if Attached != 0 {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
    }
    if ProcessHandle != NtCurrentProcess() as HANDLE {
        ObDereferenceObject(Process as PVOID);
    }

    if NT_SUCCESS(Status) {
        let _ = ntkrnl_sys::probe::write_back(URegionSize, PRegionSize as SIZE_T);
        let _ = ntkrnl_sys::probe::write_back(UBaseAddress, StartingAddress as PVOID);
    } else if QuotaCharged != 0 {
        PsReturnProcessNonPagedPoolQuota(
            Process,
            core::mem::size_of::<MMVAD_LONG>() as _,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtFreeVirtualMemory(
    ProcessHandle: HANDLE,
    UBaseAddress: *mut PVOID,
    URegionSize: *mut SIZE_T,
    FreeType: ULONG,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let UBaseAddress = opaque_ptr(UBaseAddress);
    let URegionSize = opaque_ptr(URegionSize);

    let mut PRegionSize: SIZE_T;
    let PBaseAddress: PVOID;
    let mut AlreadyDecommitted: LONG_PTR;
    let mut CommitReduction: LONG_PTR = 0;
    let mut FirstCommit: LONG_PTR;
    let mut StartingAddress: ULONG_PTR;
    let mut EndingAddress: ULONG_PTR;
    let mut Vad: PMMVAD;
    let mut NewVad: PMMVAD;
    let mut Status: NTSTATUS;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let AddressSpace: PMMSUPPORT;
    let CurrentThread: PETHREAD = PsGetCurrentThread();
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut Attached: BOOLEAN = 0;

    /* Only two flags are supported, exclusively. */
    if FreeType != MEM_RELEASE && FreeType != MEM_DECOMMIT {
        return STATUS_INVALID_PARAMETER_4;
    }

    /* Probe + capture (DUMMY_PSEH: no SEH wrap) */
    if PreviousMode != 0i32 as i8 {
        // C: inside _SEH2_TRY; the handler returns the exception code. No SEH here,
        // so the Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(UBaseAddress, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(URegionSize as *mut ULONG, 1, PreviousMode) {
            return status;
        }
    }
    PBaseAddress = match ntkrnl_sys::probe::read_back(UBaseAddress) { Ok(v) => v, Err(s) => return s };
    PRegionSize = match ntkrnl_sys::probe::read_back(URegionSize) { Ok(v) => v, Err(s) => return s };

    /* Ensure base + size sit inside user space */
    if (PBaseAddress as ULONG_PTR) >= MM_HIGHEST_USER_ADDRESS {
        return STATUS_INVALID_PARAMETER_2;
    }
    if (MM_HIGHEST_USER_ADDRESS - (PBaseAddress as ULONG_PTR)) < PRegionSize as ULONG_PTR {
        return STATUS_INVALID_PARAMETER_3;
    }

    /* Resolve target process */
    if ProcessHandle == NtCurrentProcess() as HANDLE {
        Process = CurrentProcess;
    } else {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_VM_OPERATION,
            PsProcessType,
            PreviousMode,
            core::ptr::addr_of_mut!(Process) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
        if CurrentProcess != Process {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
                core::ptr::addr_of_mut!(ApcState),
            );
            Attached = 1;
        }
    }

    /* Lock address space */
    AddressSpace = MmGetCurrentAddressSpace();
    MmLockAddressSpace(AddressSpace as PVOID);
    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        Status = STATUS_PROCESS_IS_TERMINATING;
        MmUnlockAddressSpace(AddressSpace as PVOID);
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        return Status;
    }

    /* Compute aligned start/end and locate VAD */
    StartingAddress = PAGE_ALIGN(PBaseAddress as usize) as ULONG_PTR;
    EndingAddress =
        ((PBaseAddress as ULONG_PTR) + PRegionSize as ULONG_PTR - 1) | (PAGE_SIZE - 1) as ULONG_PTR;
    Vad = MiLocateAddress(StartingAddress as PVOID);

    macro_rules! fail_path {
        ($status:expr) => {{
            Status = $status;
            MmUnlockAddressSpace(AddressSpace as PVOID);
            if Attached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            if ProcessHandle != NtCurrentProcess() as HANDLE {
                ObDereferenceObject(Process as PVOID);
            }
            return Status;
        }};
    }

    if Vad.is_null() {
        fail_path!(STATUS_MEMORY_NOT_ALLOCATED);
    }

    if ((*Vad).EndingVpn as ULONG_PTR) < (EndingAddress >> PAGE_SHIFT) {
        fail_path!(STATUS_UNABLE_TO_FREE_VM);
    }

    /* Only private memory (or rotate VADs) can be freed here, and not DevicePhysical */
    if ((*Vad).u.VadFlags.PrivateMemory() == 0
        && (*Vad).u.VadFlags.VadType() != VadRotatePhysical as _)
        || (*Vad).u.VadFlags.VadType() == VadDevicePhysicalMemory as _
    {
        fail_path!(STATUS_UNABLE_TO_DELETE_SECTION);
    }

    /* Don't allow freeing of protected VADs (PEB/TEB/shared user page).
     * Upstream's pre-fix C guarded this with a debug-only ASSERT, which our
     * translation correctly rendered as a comment -- so in a release build there
     * was no guard at all on either side, and any usermode caller could hand
     * NtFreeVirtualMemory its own PEB, TEB or the shared user data page and fall
     * straight into the MEM_RELEASE path below. Upstream replaced the ASSERT with
     * a real runtime check. Upstream-Commit: fca332046ed4 */
    if (*Vad).u.VadFlags.NoChange() != 0 {
        fail_path!(STATUS_INVALID_PAGE_PROTECTION);
    }

    if (FreeType & MEM_RELEASE) != 0 {
        // ASSERT(VadType == VadNone)
        if PRegionSize == 0 {
            /* Whole-VAD release */
            if ((PBaseAddress as ULONG_PTR) >> PAGE_SHIFT) != (*Vad).StartingVpn as ULONG_PTR {
                fail_path!(STATUS_FREE_VM_NOT_AT_BASE);
            }
            StartingAddress = ((*Vad).StartingVpn as ULONG_PTR) << PAGE_SHIFT;
            EndingAddress = (((*Vad).EndingVpn as ULONG_PTR) << PAGE_SHIFT)
                | (PAGE_SIZE - 1) as ULONG_PTR;

            MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
            /* What the whole VAD holds committed goes back (upstream returned
             * nothing here, so every whole-VAD release leaked its charge). */
            CommitReduction =
                MiCalculatePageCommitment(StartingAddress, EndingAddress, Vad, Process) as LONG_PTR;
            MiRemoveNode(
                Vad as PMMADDRESS_NODE,
                core::ptr::addr_of_mut!((*Process).VadRoot),
            );
            PsReturnProcessNonPagedPoolQuota(
                Process,
                core::mem::size_of::<MMVAD_LONG>() as _,
            );
        } else {
            /* Partial release: cases A/B/C/D */
            let starting_vpn = StartingAddress >> PAGE_SHIFT;
            let ending_vpn = EndingAddress >> PAGE_SHIFT;
            if starting_vpn == (*Vad).StartingVpn as ULONG_PTR {
                if ending_vpn == (*Vad).EndingVpn as ULONG_PTR {
                    /* Case D — whole VAD */
                    MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
                    CommitReduction = MiCalculatePageCommitment(
                        StartingAddress,
                        EndingAddress,
                        Vad,
                        Process,
                    ) as LONG_PTR;
                    MiRemoveNode(
                        Vad as PMMADDRESS_NODE,
                        core::ptr::addr_of_mut!((*Process).VadRoot),
                    );
                    PsReturnProcessNonPagedPoolQuota(
                        Process,
                        core::mem::size_of::<MMVAD_LONG>() as _,
                    );
                } else {
                    /* Case A — release prefix */
                    MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
                    CommitReduction = MiCalculatePageCommitment(
                        StartingAddress,
                        EndingAddress,
                        Vad,
                        Process,
                    ) as LONG_PTR;
                    let new_cc = (*Vad).u.VadFlags.CommitCharge() as LONG_PTR - CommitReduction;
                    (*Vad).u.VadFlags.set_CommitCharge(new_cc as _);
                    (*Vad).StartingVpn = ((EndingAddress + 1) >> PAGE_SHIFT) as _;
                    Vad = core::ptr::null_mut();
                }
            } else if ending_vpn == (*Vad).EndingVpn as ULONG_PTR {
                /* Case C — release suffix */
                MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
                CommitReduction = MiCalculatePageCommitment(
                    StartingAddress,
                    EndingAddress,
                    Vad,
                    Process,
                ) as LONG_PTR;
                let new_cc = (*Vad).u.VadFlags.CommitCharge() as LONG_PTR - CommitReduction;
                (*Vad).u.VadFlags.set_CommitCharge(new_cc as _);
                (*Vad).EndingVpn = ((StartingAddress - 1) >> PAGE_SHIFT) as _;
                Vad = core::ptr::null_mut();
            } else {
                /* Case B — release middle, must split */
                NewVad = ExAllocatePoolZero(
                    NonPagedPool as u32,
                    core::mem::size_of::<MMVAD_LONG>() as _,
                    u32::from_le_bytes(*b"VadS"),
                ) as PMMVAD;
                if NewVad.is_null() {
                    fail_path!(STATUS_INSUFFICIENT_RESOURCES);
                }

                Status = PsChargeProcessNonPagedPoolQuota(
                    Process,
                    core::mem::size_of::<MMVAD_LONG>() as _,
                );
                if Status < 0 {
                    ExFreePoolWithTag(NewVad as PVOID, u32::from_le_bytes(*b"VadS"));
                    fail_path!(STATUS_QUOTA_EXCEEDED);
                }

                (*NewVad).StartingVpn = ((EndingAddress + 1) >> PAGE_SHIFT) as _;
                (*NewVad).EndingVpn = (*Vad).EndingVpn;
                (*NewVad).u.LongFlags = (*Vad).u.LongFlags;
                (*NewVad).u.VadFlags.set_CommitCharge(0);

                MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
                CommitReduction = MiCalculatePageCommitment(
                    StartingAddress,
                    EndingAddress,
                    Vad,
                    Process,
                ) as LONG_PTR;

                (*Vad).EndingVpn = ((StartingAddress - 1) >> PAGE_SHIFT) as _;

                MiInsertVad(NewVad, core::ptr::addr_of_mut!((*Process).VadRoot));

                FirstCommit = MiCalculatePageCommitment(
                    ((*Vad).StartingVpn as ULONG_PTR) << PAGE_SHIFT,
                    StartingAddress - 1,
                    Vad,
                    Process,
                ) as LONG_PTR;
                let cc = (*Vad).u.VadFlags.CommitCharge() as LONG_PTR;
                (*NewVad)
                    .u
                    .VadFlags
                    .set_CommitCharge((cc - CommitReduction - FirstCommit) as _);
                (*Vad).u.VadFlags.set_CommitCharge(FirstCommit as _);
                Vad = core::ptr::null_mut();
            }
        }

        /* Drop the page range and release the working set */
        MiDeleteVirtualAddresses(StartingAddress, EndingAddress, core::ptr::null_mut());
        MiUnlockProcessWorkingSetUnsafe(Process, CurrentThread);
        Status = STATUS_SUCCESS;

        // FinalPath: update process counters, unlock, optional free VAD, detach, write back
        PRegionSize = (EndingAddress - StartingAddress + 1) as SIZE_T;
        super::commit::MiReturnCommitment(CommitReduction as usize, Process);
        if (FreeType & MEM_RELEASE) != 0 {
            (*Process).VirtualSize -= PRegionSize as ntkrnl_sys::ddk::SIZE_T;
        }
        MmUnlockAddressSpace(AddressSpace as PVOID);
        if !Vad.is_null() {
            ExFreePool(Vad as PVOID);
        }
        if Attached != 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
        }
        if ProcessHandle != NtCurrentProcess() as HANDLE {
            ObDereferenceObject(Process as PVOID);
        }
        let _ = ntkrnl_sys::probe::write_back(URegionSize, PRegionSize);
        let _ = ntkrnl_sys::probe::write_back(UBaseAddress, StartingAddress as PVOID);
        return Status;
    }

    /* DECOMMIT path */
    if (*Vad).u.VadFlags.VadType() == VadAwe as _
        || (*Vad).u.VadFlags.VadType() == VadLargePages as _
        || (*Vad).u.VadFlags.VadType() == VadRotatePhysical as _
    {
        fail_path!(STATUS_MEMORY_NOT_ALLOCATED);
    }

    if PRegionSize == 0 {
        if ((PBaseAddress as ULONG_PTR) >> PAGE_SHIFT) != (*Vad).StartingVpn as ULONG_PTR {
            fail_path!(STATUS_FREE_VM_NOT_AT_BASE);
        }
        EndingAddress =
            (((*Vad).EndingVpn as ULONG_PTR) << PAGE_SHIFT) | (PAGE_SIZE - 1) as ULONG_PTR;
    }

    /* What is committed in the range is what the decommit gives back, counted the
     * way every other charge and return counts it (MiCalculatePageCommitment), so
     * the ledger balances; MiDecommitPages's own count of already-decommitted PTEs
     * does not know a blank PTE in a reserve-only VAD was never committed. */
    MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
    CommitReduction =
        MiCalculatePageCommitment(StartingAddress, EndingAddress, Vad, Process) as LONG_PTR;
    MiUnlockProcessWorkingSetUnsafe(Process, CurrentThread);
    AlreadyDecommitted = MiDecommitPages(
        StartingAddress as PVOID,
        MiAddressToPte(EndingAddress as PVOID),
        Process,
        Vad,
    ) as LONG_PTR;
    let _ = AlreadyDecommitted;
    let new_cc = (*Vad).u.VadFlags.CommitCharge() as LONG_PTR - CommitReduction;
    (*Vad).u.VadFlags.set_CommitCharge(new_cc as _);

    /* FinalPath shared with above (Vad is intentionally NOT freed for decommit) */
    let _ = Vad; // keep Vad alive — decommit must not free the VAD
    Status = STATUS_SUCCESS;
    PRegionSize = (EndingAddress - StartingAddress + 1) as SIZE_T;
    super::commit::MiReturnCommitment(CommitReduction as usize, Process);
    if (FreeType & MEM_RELEASE) != 0 {
        (*Process).VirtualSize -= PRegionSize as ntkrnl_sys::ddk::SIZE_T;
    }
    MmUnlockAddressSpace(AddressSpace as PVOID);
    if Attached != 0 {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
    }
    if ProcessHandle != NtCurrentProcess() as HANDLE {
        ObDereferenceObject(Process as PVOID);
    }
    let _ = ntkrnl_sys::probe::write_back(URegionSize, PRegionSize);
    let _ = ntkrnl_sys::probe::write_back(UBaseAddress, StartingAddress as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPhysicalAddress(
    Address: PVOID,
) -> PHYSICAL_ADDRESS {
    let Address = opaque_ptr(Address);
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let TempPde: MMPTE;
    let TempPte: MMPTE;

    let pde_valid = {
        let mut result = true;
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let Buffer = opaque_ptr(Buffer);
    let NumberOfBytesRead = opaque_ptr(NumberOfBytesRead);
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

        // C: inside _SEH2_TRY; the handler returns the exception code.
        if !NumberOfBytesRead.is_null() {
            if let Err(status) = ntkrnl_sys::probe::probe_write_generic(NumberOfBytesRead, 1, PreviousMode) {
                return status;
            }
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

    // C: guarded write, the _SEH2_EXCEPT handler does nothing.
    if !NumberOfBytesRead.is_null() {
        ntkrnl_sys::probe::write_user_nofail(NumberOfBytesRead, BytesRead, PreviousMode);
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let Buffer = opaque_ptr(Buffer);
    let NumberOfBytesWritten = opaque_ptr(NumberOfBytesWritten);
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

        // C: inside _SEH2_TRY; the handler returns the exception code.
        if !NumberOfBytesWritten.is_null() {
            if let Err(status) = ntkrnl_sys::probe::probe_write_generic(NumberOfBytesWritten, 1, PreviousMode) {
                return status;
            }
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

    // C: guarded write, the _SEH2_EXCEPT handler does nothing.
    if !NumberOfBytesWritten.is_null() {
        ntkrnl_sys::probe::write_user_nofail(NumberOfBytesWritten, BytesWritten, PreviousMode);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushInstructionCache(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    FlushSize: SIZE_T,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let UnsafeBaseAddress = opaque_ptr(UnsafeBaseAddress);
    let UnsafeNumberOfBytesToProtect = opaque_ptr(UnsafeNumberOfBytesToProtect);
    let UnsafeOldAccessProtection = opaque_ptr(UnsafeOldAccessProtection);
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
        // C: inside _SEH2_TRY; the handler returns the exception code. No SEH here,
        // so the Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(UnsafeBaseAddress, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(UnsafeNumberOfBytesToProtect, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(UnsafeOldAccessProtection, 1, PreviousMode) {
            return status;
        }

        BaseAddress = match ntkrnl_sys::probe::read_back(UnsafeBaseAddress) { Ok(v) => v, Err(s) => return s };
        NumberOfBytesToProtect = match ntkrnl_sys::probe::read_back(UnsafeNumberOfBytesToProtect) { Ok(v) => v, Err(s) => return s };
    } else {
        BaseAddress = match ntkrnl_sys::probe::read_back(UnsafeBaseAddress) { Ok(v) => v, Err(s) => return s };
        NumberOfBytesToProtect = match ntkrnl_sys::probe::read_back(UnsafeNumberOfBytesToProtect) { Ok(v) => v, Err(s) => return s };
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

    let _ = ntkrnl_sys::probe::write_back(UnsafeOldAccessProtection, OldAccessProtection);
    let _ = ntkrnl_sys::probe::write_back(UnsafeBaseAddress, BaseAddress);
    let _ = ntkrnl_sys::probe::write_back(UnsafeNumberOfBytesToProtect, NumberOfBytesToProtect);

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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        PointerPxe = MiAddressToPxe(*BaseAddress);
    }

    LastPte = MiAddressToPte(((EndAddress as ULONG_PTR) - 1) as PVOID);

    MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

    loop {
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            while !(*PointerPxe).valid() ||
                  !(*PointerPpe).valid() ||
                  !(*PointerPde).valid() ||
                  !(*PointerPte).valid() {
                MiUnlockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

                CurrentVa = MiPteToAddress(PointerPte);

                TempStatus = MmAccessFault(1u32, CurrentVa, 0i8, 0xBADBADA3BADBADA3u64 as PVOID);
                if TempStatus < 0 {
                    ntkrnl_sys::ASSERT!(false);
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
                    ntkrnl_sys::ASSERT!(false);
                    Status = TempStatus;
                    MmUnlockAddressSpace(AddressSpace as PVOID);
                    return Status;
                }

                MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);
            }
        }

        Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
        ntkrnl_sys::ASSERT!(!Pfn1.is_null());

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
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let NumberOfBytesToLock = opaque_ptr(NumberOfBytesToLock);
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

    // C probes these unconditionally (no previous-mode test), inside _SEH2_TRY; the handler returns the exception code. No SEH here,
    // so the Result probe returns it instead of raising unhandled.
    if let Err(status) = ntkrnl_sys::probe::probe_write_generic(BaseAddress, 1, 1i8) {
        return status;
    }
    if let Err(status) = ntkrnl_sys::probe::probe_write_generic(NumberOfBytesToLock, 1, 1i8) {
        return status;
    }

    CapturedBaseAddress = match ntkrnl_sys::probe::read_back(BaseAddress) { Ok(v) => v, Err(s) => return s };
    CapturedBytesToLock = match ntkrnl_sys::probe::read_back(NumberOfBytesToLock) { Ok(v) => v, Err(s) => return s };

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

    if let Err(s) = ntkrnl_sys::probe::write_back(BaseAddress, CapturedBaseAddress) { return s; }
    if let Err(s) = ntkrnl_sys::probe::write_back(NumberOfBytesToLock, CapturedBytesToLock) { return s; }

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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        PointerPxe = MiAddressToPxe(*BaseAddress);
    }

    LastPte = MiAddressToPte(((EndAddress as ULONG_PTR) - 1) as PVOID);

    MiLockProcessWorkingSet(CurrentProcess, PsGetCurrentThread() as PETHREAD);

    loop {
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            if !(*PointerPxe).valid() ||
               !(*PointerPpe).valid() ||
               !(*PointerPde).valid() ||
               !(*PointerPte).valid() {
                Status = STATUS_NOT_LOCKED;
            } else {
                Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
                ntkrnl_sys::ASSERT!(!Pfn1.is_null());

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
                ntkrnl_sys::ASSERT!(!Pfn1.is_null());

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
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let NumberOfBytesToUnlock = opaque_ptr(NumberOfBytesToUnlock);
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

    // C probes these unconditionally (no previous-mode test), inside _SEH2_TRY; the handler returns the exception code. No SEH here,
    // so the Result probe returns it instead of raising unhandled.
    if let Err(status) = ntkrnl_sys::probe::probe_write_generic(BaseAddress, 1, 1i8) {
        return status;
    }
    if let Err(status) = ntkrnl_sys::probe::probe_write_generic(NumberOfBytesToUnlock, 1, 1i8) {
        return status;
    }

    CapturedBaseAddress = match ntkrnl_sys::probe::read_back(BaseAddress) { Ok(v) => v, Err(s) => return s };
    CapturedBytesToUnlock = match ntkrnl_sys::probe::read_back(NumberOfBytesToUnlock) { Ok(v) => v, Err(s) => return s };

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

    if let Err(s) = ntkrnl_sys::probe::write_back(BaseAddress, CapturedBaseAddress) { return s; }
    if let Err(s) = ntkrnl_sys::probe::write_back(NumberOfBytesToUnlock, CapturedBytesToUnlock) { return s; }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    NumberOfBytesToFlush: PSIZE_T,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let NumberOfBytesToFlush = opaque_ptr(NumberOfBytesToFlush);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedBaseAddress: PVOID = core::ptr::null_mut();
    let mut CapturedBytesToFlush: SIZE_T = 0;
    let mut LocalStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if PreviousMode != 0i32 as i8 {
        // C: inside _SEH2_TRY; the handler returns the exception code. No SEH here,
        // so the Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(BaseAddress, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(NumberOfBytesToFlush, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(IoStatusBlock, 1, PreviousMode) {
            return status;
        }

        CapturedBaseAddress = match ntkrnl_sys::probe::read_back(BaseAddress) { Ok(v) => v, Err(s) => return s };
        CapturedBytesToFlush = match ntkrnl_sys::probe::read_back(NumberOfBytesToFlush) { Ok(v) => v, Err(s) => return s };
    } else {
        CapturedBaseAddress = match ntkrnl_sys::probe::read_back(BaseAddress) { Ok(v) => v, Err(s) => return s };
        CapturedBytesToFlush = match ntkrnl_sys::probe::read_back(NumberOfBytesToFlush) { Ok(v) => v, Err(s) => return s };
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

    // C: one _SEH2_TRY with an empty handler; a fault stops the remaining stores.
    let _ = ntkrnl_sys::probe::write_back(BaseAddress, PAGE_ALIGN(CapturedBaseAddress as usize) as PVOID)
        .and_then(|()| ntkrnl_sys::probe::write_back(NumberOfBytesToFlush, 0))
        .and_then(|()| ntkrnl_sys::probe::write_back(IoStatusBlock as *mut _IO_STATUS_BLOCK, LocalStatusBlock));

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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let UserAddressArray = opaque_ptr(UserAddressArray);
    let EntriesInUserAddressArray = opaque_ptr(EntriesInUserAddressArray);
    let Granularity = opaque_ptr(Granularity);
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

        // C: inside _SEH2_TRY; the handler returns the exception code. No SEH here,
        // so the Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(EntriesInUserAddressArray, 1, PreviousMode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(Granularity, 1, PreviousMode) {
            return status;
        }

        // The capture is inside the same C _SEH2_TRY.
        CapturedEntryCount = match ntkrnl_sys::probe::probe_read_generic(EntriesInUserAddressArray, 1, PreviousMode) {
            Ok(v) => v,
            Err(status) => return status,
        };

        if CapturedEntryCount == 0 {
            return STATUS_INVALID_PARAMETER_5;
        }

        if CapturedEntryCount > (MAXULONG_PTR / core::mem::size_of::<ULONG_PTR>()) {
            return STATUS_INVALID_PARAMETER_5;
        }

        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer_write(
            UserAddressArray as PVOID,
            CapturedEntryCount * core::mem::size_of::<PVOID>(),
            core::mem::size_of::<PVOID>(),
            PreviousMode,
        ) {
            return status;
        }
    } else {
        CapturedEntryCount = match ntkrnl_sys::probe::read_back(EntriesInUserAddressArray) { Ok(v) => v, Err(s) => return s };
        ntkrnl_sys::ASSERT!(CapturedEntryCount != 0);
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

    // C: guarded write whose handler's status is discarded (returns SUCCESS).
    ntkrnl_sys::probe::write_user_nofail(EntriesInUserAddressArray, 0, PreviousMode);
    ntkrnl_sys::probe::write_user_nofail(Granularity, PAGE_SIZE as ULONG, PreviousMode);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtResetWriteWatch(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    RegionSize: SIZE_T,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let mut EndAddress: PVOID;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL as u8);

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
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let BaseAddress = opaque_ptr(BaseAddress);
    let MemoryInformation = opaque_ptr(MemoryInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let PreviousMode: KPROCESSOR_MODE;

    if BaseAddress > MM_HIGHEST_USER_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER;
    }

    PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0i32 as i8 {
        // C: inside _SEH2_TRY; the handler captures the code and returns it.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer_write(
            MemoryInformation,
            MemoryInformationLength,
            core::mem::size_of::<ULONG_PTR>(),
            PreviousMode,
        ) {
            return status;
        }

        if !ReturnLength.is_null() {
            if let Err(status) = ntkrnl_sys::probe::probe_write_generic(ReturnLength, 1, PreviousMode) {
                return status;
            }
        }
    }

    // The C's MiQuery* helpers store into MemoryInformation and ReturnLength inside
    // _SEH2_TRY. Hand them a pool copy (OutputBounce) and a kernel length instead;
    // both reach the caller once, below, through the fault-tolerant copy.
    let bounce = match ntkrnl_sys::probe::OutputBounce::new(MemoryInformation, MemoryInformationLength, PreviousMode) {
        Ok(b) => b,
        Err(s) => return s,
    };
    let UserReturnLength = ReturnLength;
    let mut KernelReturnLength: SIZE_T = SIZE_T::MAX;
    let MemoryInformation = bounce.buffer();
    let ReturnLength: PSIZE_T = if UserReturnLength.is_null() {
        core::ptr::null_mut()
    } else if PreviousMode != 0 {
        &mut KernelReturnLength
    } else {
        UserReturnLength
    };

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

    // MEMORY_SECTION_NAME's string Buffer points just past it, inside the buffer.
    if Status >= 0
        && MemoryInformationClass == MemorySectionName
        && bounce.contains(MemoryInformation, core::mem::size_of::<MEMORY_SECTION_NAME_T>())
    {
        bounce.rebase(core::ptr::addr_of_mut!((*(MemoryInformation as *mut _MEMORY_SECTION_NAME)).SectionFileName.Buffer));
    }
    if let Err(s) = bounce.finish() {
        return s;
    }
    if PreviousMode != 0 && !UserReturnLength.is_null() && KernelReturnLength != SIZE_T::MAX {
        if let Err(s) = ntkrnl_sys::probe::write_user(UserReturnLength, KernelReturnLength, 0) {
            return s;
        }
    }

    Status
}

// === REMOVED: duplicate NtAllocateVirtualMemory (head version retained) ===

// === REMOVED: duplicate NtFreeVirtualMemory (head version retained) ===

// === REMOVED: duplicate MmGetPhysicalAddress (head version retained) ===
