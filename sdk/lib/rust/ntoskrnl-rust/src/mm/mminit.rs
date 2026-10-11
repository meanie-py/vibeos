use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _OBJECT_ATTRIBUTES, _UNICODE_STRING,
    _LDR_DATA_TABLE_ENTRY,
    PLDR_DATA_TABLE_ENTRY,
    PLOADER_PARAMETER_BLOCK,
    PMEMORY_AREA,
    PMMSUPPORT,
    KEVENT,
    FAST_MUTEX,
    LIST_ENTRY,
    ExAcquireResourceExclusiveLite,
    ExInitializeFastMutex,
    ExReleaseResourceLite,
    KeInitializeEvent,
    KeBalanceSetManager,
    MiInitBalancerThread,
    MiInitializeMemoryEvents,
    MiInitializeSessionIds,
    MiInitializeSessionWsSupport,
    MiWriteProtectSystemImage,
    MmCreateMemoryArea,
    MmGetKernelAddressSpace,
    MmInitGlobalKernelPageDirectory,
    MmInitPagingFile,
    MmInitSectionImplementation,
    MmInitializeMemoryConsumer,
    MmInitializeRmapList,
    MmTrimUserMemory,
    PsCreateSystemThread,
    ZwClose,
};

// FORCEINLINE shims — must be extern "C" (cdecl) to match C shim calling convention on i386.
extern "C" {
    fn MmLockAddressSpace(AddressSpace: PMMSUPPORT);
    fn MmUnlockAddressSpace(AddressSpace: PMMSUPPORT);
}

extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
}

#[cfg(target_arch = "x86_64")]
const MI_REAL_SYSTEM_RANGE_START: usize = 0xFFFF800000000000;

#[cfg(target_arch = "x86")]
const MM_HAL_VA_START: usize = 0xFFC00000;
#[cfg(target_arch = "x86")]
const MM_HAL_VA_END: usize = 0xFFFFFFFF;

#[no_mangle]
pub static mut Mm64BitPhysicalAddress: BOOLEAN = 0u8;
#[no_mangle]
pub static mut MmReadClusterSize: ULONG = 0;
#[no_mangle]
pub static mut MmDisablePagingExecutive: u8 = 1; // Forced to off

// Registry-controlled values owned by mm/mminit. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
// Renamed from MMINIT_CONTROL_VECTOR (collided with ARM3::mminit's) so it can be a
// unique #[no_mangle] link symbol for config-rust's runtime CmControlVector assembler.
#[no_mangle]
pub static MM_MMINIT_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "DisablePagingExecutive",
            core::ptr::addr_of_mut!(MmDisablePagingExecutive)),
    ] });

#[no_mangle]
pub static mut MmSharedUserDataPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MmKernelAddressSpace: PMMSUPPORT = core::ptr::null_mut();

// Promoted from `mm_rust_shims.c` — owned here now that section.c is
// translated to Rust. `MmWaitPageEvent` is a KEVENT zero-initialized
// at module load and KeInitializeEvent'd in MmInitSystem; section.rs
// also imports this via `extern "system" { static mut ... }`.
#[no_mangle]
pub static mut MmWaitPageEvent: KEVENT = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmSubsectionBase: ULONG_PTR = 0;

extern "system" {
    static mut MiGlobalPageOperation: FAST_MUTEX;
    static mut MiSegmentList: LIST_ENTRY;
    fn MiRosTrimCache(Target: ULONG, Priority: ULONG, NrFreed: *mut ULONG) -> NTSTATUS;
}

// CODE_SEG("INIT")
unsafe fn MiCreateArm3StaticMemoryArea(BaseAddress: PVOID, Size: usize, Executable: BOOLEAN) {
    let Protection = if Executable != 0 {
        PAGE_EXECUTE_READWRITE
    } else {
        PAGE_READWRITE
    };
    let mut pBaseAddress = BaseAddress;
    let mut MArea: PMEMORY_AREA = core::ptr::null_mut();
    let Status: NTSTATUS;

    Status = MmCreateMemoryArea(
        MmGetKernelAddressSpace(),
        MEMORY_AREA_OWNED_BY_ARM3 | MEMORY_AREA_STATIC,
        &mut pBaseAddress,
        Size as ntkrnl_sys::ddk::SIZE_T,
        Protection,
        &mut MArea,
        0,
        PAGE_SIZE as ULONG,
    );
    // C: ASSERT(Status == STATUS_SUCCESS). A reservation that fails silently leaves its
    // range free for the ROS gap finder, which then hands it out as a system view.
    if Status < 0 {
        mm_guard_trip!(
            "mminit.MiCreateArm3StaticMemoryArea FAILED", 16,
            "Status" => Status as u32 as usize,
            "BaseAddress" => BaseAddress as usize,
            "Size" => Size,
        );
    }
}

// CODE_SEG("INIT")
unsafe fn MiInitSystemMemoryAreas() {
    MmLockAddressSpace(MmGetKernelAddressSpace());

    #[cfg(target_arch = "x86_64")]
    {
        MiCreateArm3StaticMemoryArea(
            MI_REAL_SYSTEM_RANGE_START as PVOID,
            (PTE_BASE as usize) - (MI_REAL_SYSTEM_RANGE_START as usize),
            0u8,
        );
    }

    MiCreateArm3StaticMemoryArea(KSEG0_BASE as PVOID, MmBootImageSize as usize, 1u8);

    MiCreateArm3StaticMemoryArea(
        PTE_BASE as PVOID,
        (PTE_TOP as usize) - (PTE_BASE as usize) + 1,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(
        HYPER_SPACE as PVOID,
        (HYPER_SPACE_END as usize) - (HYPER_SPACE as usize) + 1,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(
        MmPfnDatabase as PVOID,
        (MxPfnAllocation as usize) << PAGE_SHIFT,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(
        MmNonPagedPoolStart,
        MmSizeOfNonPagedPoolInBytes as usize,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(
        MmNonPagedSystemStart,
        ((MmNumberOfSystemPtes + 1) as usize) * PAGE_SIZE,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(
        MmNonPagedPoolExpansionStart,
        (MmNonPagedPoolEnd as usize) - (MmNonPagedPoolExpansionStart as usize),
        0u8,
    );

    MiCreateArm3StaticMemoryArea(MiSystemViewStart, MmSystemViewSize as usize, 0u8);

    MiCreateArm3StaticMemoryArea(
        MmSessionBase,
        (MiSessionSpaceEnd as usize) - (MmSessionBase as usize),
        0u8,
    );
    // Diagnostic (i386 writable-root boot): a ROS system view landed at 0xbf7f0000, which
    // should be inside this reserved session range.
    mm_guard_trip!(
        "mminit.MiInitSystemMemoryAreas session-and-view-ranges", 1,
        "SessionBase" => MmSessionBase as usize,
        "SessionEnd" => MiSessionSpaceEnd as usize,
        "SystemViewStart" => MiSystemViewStart as usize,
        "SystemViewSize" => MmSystemViewSize as usize,
        "BootImageSize" => MmBootImageSize as usize,
    );

    MiCreateArm3StaticMemoryArea(
        MmPagedPoolStart,
        MmSizeOfPagedPoolInBytes as usize,
        0u8,
    );

    MiCreateArm3StaticMemoryArea(MI_DEBUG_MAPPING as PVOID, PAGE_SIZE, 0u8);

    #[cfg(target_arch = "x86")]
    {
        MiCreateArm3StaticMemoryArea(
            MM_HAL_VA_START as PVOID,
            (MM_HAL_VA_END as usize) - (MM_HAL_VA_START as usize) + 1,
            0u8,
        );
    }

    #[cfg(not(target_arch = "x86"))]
    {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            MiCreateArm3StaticMemoryArea(
                PCR as PVOID,
                PAGE_SIZE * (KeNumberProcessors as usize),
                0u8,
            );
        }

        MiCreateArm3StaticMemoryArea(KI_USER_SHARED_DATA as PVOID, PAGE_SIZE, 0u8);
    }

    MmUnlockAddressSpace(MmGetKernelAddressSpace());
}

// CODE_SEG("INIT")
unsafe fn MiDbgDumpAddressSpace() {
}

// CODE_SEG("INIT")
unsafe fn MmInitBsmThread() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = PsCreateSystemThread(
        &mut ThreadHandle,
        THREAD_ALL_ACCESS,
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(KeBalanceSetManager),
        core::ptr::null_mut(),
    );

    ZwClose(ThreadHandle);
    Status
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitSystem(
    Phase: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> BOOLEAN {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut PointerPte: PMMPTE;
    let mut TempPte: MMPTE = ValidKernelPte;
    let mut PageFrameNumber: PFN_NUMBER;
    let mut ListEntry: PLIST_ENTRY;
    let mut DataTableEntry: PLDR_DATA_TABLE_ENTRY;

    #[cfg(feature = "NEWCC")]
    {
        InitializeListHead(&mut MiSegmentList as PLIST_ENTRY);
        ExInitializeFastMutex(&mut MiGlobalPageOperation);
        KeInitializeEvent(&mut MmWaitPageEvent, 1u32, 0u8);
        MmInitializeMemoryConsumer(MC_CACHE, Some(MiRosTrimCache));
    }

    #[cfg(not(feature = "NEWCC"))]
    {
        KeInitializeEvent(&mut MmWaitPageEvent, 1u32, 0u8);
    }

    MmKernelAddressSpace = &mut (*(PsIdleProcess as *mut ntkrnl_sys::ddk::_EPROCESS)).Vm as *mut _ as PMMSUPPORT;

    MiInitSystemMemoryAreas();

    MiDbgDumpAddressSpace();

    MmInitGlobalKernelPageDirectory();
    MmInitializeMemoryConsumer(MC_USER, Some(MmTrimUserMemory));
    MmInitializeRmapList();
    MmInitSectionImplementation();
    MmInitPagingFile();

    MmSharedUserDataPte = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<MMPTE>() as SIZE_T,
        TAG_MM,
    ) as PMMPTE;
    if MmSharedUserDataPte.is_null() {
        return 0u8;
    }

    PointerPte = MiAddressToPte(KI_USER_SHARED_DATA as PVOID);
    PageFrameNumber = PFN_FROM_PTE(PointerPte);

    MI_MAKE_HARDWARE_PTE_KERNEL(
        &mut TempPte,
        PointerPte,
        MM_READONLY,
        PageFrameNumber,
    );
    *MmSharedUserDataPte = TempPte;

    MiInitializeSessionWsSupport();

    MiInitializeSessionIds();

    if MiInitializeMemoryEvents() == 0 {
        return 0u8;
    }

    MiInitBalancerThread();

    MmInitBsmThread();

    ExAcquireResourceExclusiveLite(&mut PsLoadedModuleResource as *mut _ as *mut ntkrnl_sys::ddk::_ERESOURCE, 1u8);
    ListEntry = (*core::ptr::addr_of!(PsLoadedModuleList)).Flink;
    while ListEntry != core::ptr::addr_of_mut!(PsLoadedModuleList) {
        DataTableEntry = CONTAINING_RECORD(
            ListEntry as PVOID,
            core::mem::size_of::<_LDR_DATA_TABLE_ENTRY>(),
            core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
        ) as PLDR_DATA_TABLE_ENTRY;

        MiWriteProtectSystemImage((*DataTableEntry).DllBase);

        ListEntry = (*ListEntry).Flink;
    }
    ExReleaseResourceLite(&mut PsLoadedModuleResource as *mut _ as *mut ntkrnl_sys::ddk::_ERESOURCE);

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/mm/mminit.c
//  * PURPOSE:         Memory Manager Initialization
//  * PROGRAMMERS:
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// #define MODULE_INVOLVED_IN_ARM3
// #include "ARM3/miarm.h"
//
// /* GLOBALS *******************************************************************/
//
// BOOLEAN Mm64BitPhysicalAddress = FALSE;
// ULONG MmReadClusterSize;
// //
// // 0 | 1 is on/off paging, 2 is undocumented
// //
// UCHAR MmDisablePagingExecutive = 1; // Forced to off
// PMMPTE MmSharedUserDataPte;
// PMMSUPPORT MmKernelAddressSpace;
//
// extern KEVENT MmWaitPageEvent;
// extern FAST_MUTEX MiGlobalPageOperation;
// extern LIST_ENTRY MiSegmentList;
// extern NTSTATUS MiRosTrimCache(ULONG Target, ULONG Priority, PULONG NrFreed);
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// //
// // Helper function to create initial memory areas.
// // The created area is always read/write.
// //
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiCreateArm3StaticMemoryArea(PVOID BaseAddress, SIZE_T Size, BOOLEAN Executable)
// {
//     const ULONG Protection = Executable ? PAGE_EXECUTE_READWRITE : PAGE_READWRITE;
//     PVOID pBaseAddress = BaseAddress;
//     PMEMORY_AREA MArea;
//     NTSTATUS Status;
//
//     Status = MmCreateMemoryArea(MmGetKernelAddressSpace(),
//                                 MEMORY_AREA_OWNED_BY_ARM3 | MEMORY_AREA_STATIC,
//                                 &pBaseAddress,
//                                 Size,
//                                 Protection,
//                                 &MArea,
//                                 0,
//                                 PAGE_SIZE);
//     ASSERT(Status == STATUS_SUCCESS);
//     // TODO: Perhaps it would be  prudent to bugcheck here, not only assert?
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiInitSystemMemoryAreas(VOID)
// {
//     //
//     // Create all the static memory areas.
//     //
//
//     MmLockAddressSpace(MmGetKernelAddressSpace());
//
// #ifdef _M_AMD64
//     // Reserved range FFFF800000000000 - FFFFF68000000000
//     MiCreateArm3StaticMemoryArea((PVOID)MI_REAL_SYSTEM_RANGE_START, PTE_BASE - MI_REAL_SYSTEM_RANGE_START, FALSE);
// #endif /* _M_AMD64 */
//
//     // The loader mappings. The only Executable area.
//     MiCreateArm3StaticMemoryArea((PVOID)KSEG0_BASE, MmBootImageSize, TRUE);
//
//     // The PTE base
//     MiCreateArm3StaticMemoryArea((PVOID)PTE_BASE, PTE_TOP - PTE_BASE + 1, FALSE);
//
//     // Hyperspace
//     MiCreateArm3StaticMemoryArea((PVOID)HYPER_SPACE, HYPER_SPACE_END - HYPER_SPACE + 1, FALSE);
//
//     // Protect the PFN database
//     MiCreateArm3StaticMemoryArea(MmPfnDatabase, (MxPfnAllocation << PAGE_SHIFT), FALSE);
//
//     // ReactOS requires a memory area to keep the initial NP area off-bounds
//     MiCreateArm3StaticMemoryArea(MmNonPagedPoolStart, MmSizeOfNonPagedPoolInBytes, FALSE);
//
//     // System PTE space
//     MiCreateArm3StaticMemoryArea(MmNonPagedSystemStart, (MmNumberOfSystemPtes + 1) * PAGE_SIZE, FALSE);
//
//     // Nonpaged pool expansion space
//     MiCreateArm3StaticMemoryArea(MmNonPagedPoolExpansionStart, (ULONG_PTR)MmNonPagedPoolEnd - (ULONG_PTR)MmNonPagedPoolExpansionStart, FALSE);
//
//     // System view space
//     MiCreateArm3StaticMemoryArea(MiSystemViewStart, MmSystemViewSize, FALSE);
//
//     // Session space
//     MiCreateArm3StaticMemoryArea(MmSessionBase, (ULONG_PTR)MiSessionSpaceEnd - (ULONG_PTR)MmSessionBase, FALSE);
//
//     // Paged pool
//     MiCreateArm3StaticMemoryArea(MmPagedPoolStart, MmSizeOfPagedPoolInBytes, FALSE);
//
//     // Debugger mapping
//     MiCreateArm3StaticMemoryArea(MI_DEBUG_MAPPING, PAGE_SIZE, FALSE);
//
// #if defined(_X86_)
//     // Reserved HAL area (includes KUSER_SHARED_DATA and KPCR)
//     MiCreateArm3StaticMemoryArea((PVOID)MM_HAL_VA_START, MM_HAL_VA_END - MM_HAL_VA_START + 1, FALSE);
// #else /* _X86_ */
// #if !defined(_M_AMD64) && !defined(_M_ARM64)
//     // KPCR, one page per CPU. Only for 32-bit kernel.
//     MiCreateArm3StaticMemoryArea(PCR, PAGE_SIZE * KeNumberProcessors, FALSE);
// #endif /* !_M_AMD64 && !_M_ARM64 */
//
//     // KUSER_SHARED_DATA
//     MiCreateArm3StaticMemoryArea((PVOID)KI_USER_SHARED_DATA, PAGE_SIZE, FALSE);
// #endif /* _X86_ */
//
//     MmUnlockAddressSpace(MmGetKernelAddressSpace());
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiDbgDumpAddressSpace(VOID)
// {
//     //
//     // Print the memory layout
//     //
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             KSEG0_BASE,
//             (ULONG_PTR)KSEG0_BASE + MmBootImageSize,
//             "Boot Loaded Image");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmPfnDatabase,
//             (ULONG_PTR)MmPfnDatabase + (MxPfnAllocation << PAGE_SHIFT),
//             "PFN Database");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmNonPagedPoolStart,
//             (ULONG_PTR)MmNonPagedPoolStart + MmSizeOfNonPagedPoolInBytes,
//             "ARM3 Non Paged Pool");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MiSystemViewStart,
//             (ULONG_PTR)MiSystemViewStart + MmSystemViewSize,
//             "System View Space");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmSessionBase,
//             MiSessionSpaceEnd,
//             "Session Space");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             PTE_BASE, PTE_TOP,
//             "Page Tables");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             PDE_BASE, PDE_TOP,
//             "Page Directories");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             HYPER_SPACE, HYPER_SPACE_END,
//             "Hyperspace");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmSystemCacheStart, MmSystemCacheEnd,
//             "System Cache");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmPagedPoolStart,
//             (ULONG_PTR)MmPagedPoolStart + MmSizeOfPagedPoolInBytes,
//             "ARM3 Paged Pool");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmNonPagedSystemStart, MmNonPagedPoolExpansionStart,
//             "System PTE Space");
//     DPRINT1("          0x%p - 0x%p\t%s\n",
//             MmNonPagedPoolExpansionStart, MmNonPagedPoolEnd,
//             "Non Paged Pool Expansion PTE Space");
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MmInitBsmThread(VOID)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ThreadHandle;
//
//     /* Create the thread */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     Status = PsCreateSystemThread(&ThreadHandle,
//                                   THREAD_ALL_ACCESS,
//                                   &ObjectAttributes,
//                                   NULL,
//                                   NULL,
//                                   KeBalanceSetManager,
//                                   NULL);
//
//     /* Close the handle and return status */
//     ZwClose(ThreadHandle);
//     return Status;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MmInitSystem(IN ULONG Phase,
//              IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     extern MMPTE ValidKernelPte;
//     PMMPTE PointerPte;
//     MMPTE TempPte = ValidKernelPte;
//     PFN_NUMBER PageFrameNumber;
//     PLIST_ENTRY ListEntry;
//     PLDR_DATA_TABLE_ENTRY DataTableEntry;
//
//     /* Initialize the kernel address space */
//     ASSERT(Phase == 1);
//
// #ifdef NEWCC
//     InitializeListHead(&MiSegmentList);
//     ExInitializeFastMutex(&MiGlobalPageOperation);
//     KeInitializeEvent(&MmWaitPageEvent, SynchronizationEvent, FALSE);
//     // Until we're fully demand paged, we can do things the old way through
//     // the balance manager
//     // CcInitView will override this...
//     MmInitializeMemoryConsumer(MC_CACHE, MiRosTrimCache);
// #else
//     KeInitializeEvent(&MmWaitPageEvent, SynchronizationEvent, FALSE);
// #endif
//
//     MmKernelAddressSpace = &PsIdleProcess->Vm;
//
//     /* Intialize system memory areas */
//     ARM64_UART_TRACE("[MM1] MiInitSystemMemoryAreas\r\n");
//     MiInitSystemMemoryAreas();
//
//     /* Dump the address space */
//     MiDbgDumpAddressSpace();
//
//     ARM64_UART_TRACE("[MM1] MmInitGlobalKernelPageDirectory\r\n");
//     MmInitGlobalKernelPageDirectory();
//     MmInitializeMemoryConsumer(MC_USER, MmTrimUserMemory);
//     MmInitializeRmapList();
//     ARM64_UART_TRACE("[MM1] MmInitSectionImplementation\r\n");
//     MmInitSectionImplementation();
//     MmInitPagingFile();
//
//     //
//     // Create a PTE to double-map the shared data section. We allocate it
//     // from paged pool so that we can't fault when trying to touch the PTE
//     // itself (to map it), since paged pool addresses will already be mapped
//     // by the fault handler.
//     //
//     MmSharedUserDataPte = ExAllocatePoolWithTag(PagedPool,
//                           sizeof(MMPTE),
//                           TAG_MM);
//     if (!MmSharedUserDataPte) return FALSE;
//
//     //
//     // Now get the PTE for shared data, and read the PFN that holds it
//     //
//     ARM64_UART_TRACE("[MM1] SharedUserData PTE\r\n");
//     PointerPte = MiAddressToPte((PVOID)KI_USER_SHARED_DATA);
//     ASSERT(PointerPte->u.Hard.Valid == 1);
//     PageFrameNumber = PFN_FROM_PTE(PointerPte);
//
//     /* Build the PTE and write it */
//     MI_MAKE_HARDWARE_PTE_KERNEL(&TempPte,
//                                 PointerPte,
//                                 MM_READONLY,
//                                 PageFrameNumber);
//     *MmSharedUserDataPte = TempPte;
//
//     /* Initialize session working set support */
//     ARM64_UART_TRACE("[MM1] MiInitializeSessionWsSupport\r\n");
//     MiInitializeSessionWsSupport();
//
//     /* Setup session IDs */
//     MiInitializeSessionIds();
//
//     /* Setup the memory threshold events */
//     ARM64_UART_TRACE("[MM1] MiInitializeMemoryEvents\r\n");
//     if (!MiInitializeMemoryEvents()) return FALSE;
//
//     /*
//      * Unmap low memory
//      */
//     ARM64_UART_TRACE("[MM1] MiInitBalancerThread\r\n");
//     MiInitBalancerThread();
//
//     /* Initialize the balance set manager */
//     ARM64_UART_TRACE("[MM1] MmInitBsmThread\r\n");
//     MmInitBsmThread();
//
//     /* Loop the boot loaded images (under lock) */
//     ARM64_UART_TRACE("[MM1] MiWriteProtectSystemImage loop\r\n");
//     ExAcquireResourceExclusiveLite(&PsLoadedModuleResource, TRUE);
//     for (ListEntry = PsLoadedModuleList.Flink;
//          ListEntry != &PsLoadedModuleList;
//          ListEntry = ListEntry->Flink)
//     {
//         /* Get the data table entry */
//         DataTableEntry = CONTAINING_RECORD(ListEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//
//         /* Set up the image protection */
//         MiWriteProtectSystemImage(DataTableEntry->DllBase);
//     }
//     ExReleaseResourceLite(&PsLoadedModuleResource);
//
//     ARM64_UART_TRACE("[MM1] MmInitSystem(1) done\r\n");
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
