use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{ACCESS_MASK, ExFreePool, HANDLE, KPROCESSOR_MODE, KeAttachProcess, KeDetachProcess, KeInitializeSpinLock, LARGE_INTEGER, MiAcquireExpansionLock, MiArchCreateProcessAddressSpace, MiDecrementShareCount, MiDeleteVirtualAddresses, MiInitializePfn, MiInitializeWorkingSetList, MiInsertVadEx, MiLocateAddress, MiLockProcessWorkingSet, MiLockProcessWorkingSetUnsafe, MiReleaseExpansionLock, MiRemoveMappedView, MiRemoveNode, MiRemoveZeroPageSafe, MiRosCleanupMemoryArea, MiSessionAddProcess, MiSessionRemoveProcess, MiUnlockProcessWorkingSet, MiUnlockProcessWorkingSetUnsafe, MmGetFileObjectForSection, MmGetSessionId, MmLockAddressSpace, MmMapViewOfSection, MmUnlockAddressSpace, PCLIENT_ID, PEPROCESS, PETHREAD, PFILE_OBJECT, PIMAGE_LOAD_CONFIG_DIRECTORY, PIMAGE_NT_HEADERS, PINITIAL_PEB, PINITIAL_TEB, PMMVAD, PMMVAD_LONG, PNT_TIB, POBJECT_NAME_INFORMATION, PPEB, PTEB, _EPROCESS, _ETHREAD, _IMAGE_NT_HEADERS, _KLOCK_QUEUE_HANDLE, _KTHREAD, _MMPFN, _MMPTE, _MMVAD_LONG, _MM_AVL_TABLE};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

#[cfg(target_arch = "aarch64")]
extern "C" {
    fn ARM64_UART_TRACE(msg: *const i8);
    fn ARM64_UART_TRACE_HEX(msg: *const i8, val: ULONG_PTR);
}

// Per-process address-space breadcrumbs: printed only under the PROCTRACE boot option
// (shim_exports.rs, "Process/thread lifecycle tracing").
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn ARM64_PROC_TRACE_HEX(msg: *const i8, val: ULONG_PTR);
}

#[cfg(target_arch = "aarch64")]
unsafe fn MmTraceIrql(label: *const i8) {
    extern "C" { fn MmDbgCheckIrql(label: *const i8); }
    if crate::shim_exports::proc_trace_enabled() {
        MmDbgCheckIrql(label);
    }
}

/// Portable "msg + hex value" trace for the WoW64 PEB32/TEB32 paths, which are
/// live on both arm64 (UART) and amd64 (DbgPrint) hosts.
unsafe fn MiWow64TraceHex(msg: *const i8, val: ULONG_PTR) {
    #[cfg(target_arch = "aarch64")]
    ARM64_UART_TRACE_HEX(msg, val);
    #[cfg(not(target_arch = "aarch64"))]
    {
        extern "C" {
            fn DbgPrint(Format: *const i8, ...) -> u32;
        }
        DbgPrint(b"%s%p\n\0".as_ptr() as *const i8, msg, val as PVOID);
    }
}

static mut MmProcessColorSeed: ULONG = 0x12345678;
#[no_mangle]
pub static mut MmMaximumDeadKernelStacks: ULONG = 5;
#[no_mangle]
pub static mut MmDeadStackSListHead: SLIST_HEADER = unsafe { core::mem::zeroed() };
static mut MmRotatingUniprocessorNumber: ULONG = 0;

/// VAD ceiling for a WoW64 process's 32-bit-reachable PEB32/TEB region: the
/// 32-bit `MM_HIGHEST_USER_ADDRESS`, so guest pointers can reach both.
///
/// This range CONTAINS `MM_SHARED_USER_DATA_VA` (KUSER_SHARED_DATA, 0x7FFE0000),
/// which is safe only because `MiInsertSharedUserPageVad` reserves that page with
/// a real VAD at process init — the VAD allocator has no other way to know the
/// page is taken. Do not assume the ceiling alone protects it.
const MM_HIGHEST_VAD_ADDRESS_WOW64: ULONG_PTR = 0x7FFE_FFFF;

unsafe fn MiCreatePebOrTeb(
    Process: PEPROCESS,
    Size: ULONG,
    BaseAddress: PULONG_PTR,
) -> NTSTATUS {
    MiCreatePebOrTebCapped(Process, Size, 0, BaseAddress)
}

/// `HighestAddressCap` = 0 for the classic behavior; non-zero forces the VAD below
/// that address (WoW64: PEB32/TEB32 must be reachable by 32-bit guest pointers).
unsafe fn MiCreatePebOrTebCapped(
    Process: PEPROCESS,
    Size: ULONG,
    HighestAddressCap: ULONG_PTR,
    BaseAddress: PULONG_PTR,
) -> NTSTATUS {
    let mut Vad: PMMVAD_LONG;
    let mut Status: NTSTATUS;
    let HighestAddress: ULONG_PTR;
    let RandomBase: ULONG_PTR;
    let AlignedSize: ULONG;
    let mut CurrentTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    Status = PsChargeProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
    if Status < 0 {
        return Status;
    }

    Vad = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_MMVAD_LONG>(),
        u32::from_le_bytes(*b"Vadl"),
    ) as PMMVAD_LONG;
    if Vad.is_null() {
        Status = STATUS_NO_MEMORY;
        PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        return Status;
    }

    (*Vad).u.LongFlags = 0;
    (*Vad).u.VadFlags.set_CommitCharge((BYTES_TO_PAGES(Size as usize) as usize) as _);
    (*Vad).u.VadFlags.set_MemCommit(1);
    (*Vad).u.VadFlags.set_PrivateMemory(1);
    (*Vad).u.VadFlags.set_Protection((MM_READWRITE as usize) as _);
    (*Vad).u.VadFlags.set_NoChange(1);
    (*Vad).u1.Parent = core::ptr::null_mut();

    (*Vad).u2.LongFlags2 = 0;
    (*Vad).u2.VadFlags2.set_OneSecured(1);
    (*Vad).u2.VadFlags2.set_LongVad(1);
    (*Vad).u2.VadFlags2.set_ReadOnly(0);

    (*Vad).ControlArea = core::ptr::null_mut();
    (*Vad).FirstPrototypePte = core::ptr::null_mut();

    if Size == core::mem::size_of::<PEB>() as ULONG && HighestAddressCap == 0 {
        KeQueryTickCount(core::ptr::addr_of_mut!(CurrentTime) as *mut i64);
        CurrentTime.__bindgen_anon_1.LowPart &= ((_64K / PAGE_SIZE) - 1) as u32;

        RandomBase = MM_HIGHEST_VAD_ADDRESS as ULONG_PTR + 1;
        let RandomBase = RandomBase - ((CurrentTime.__bindgen_anon_1.LowPart as ULONG_PTR) << PAGE_SHIFT);

        AlignedSize = ROUND_TO_PAGES(Size as usize) as ULONG;
        let RandomBase = if (RandomBase + AlignedSize as ULONG_PTR) > (MM_HIGHEST_VAD_ADDRESS as ULONG_PTR + 1) {
            MM_HIGHEST_VAD_ADDRESS as ULONG_PTR + 1 - AlignedSize as ULONG_PTR
        } else {
            RandomBase
        };

        HighestAddress = RandomBase + AlignedSize as ULONG_PTR - 1;
    } else if HighestAddressCap != 0 {
        HighestAddress = core::cmp::min(HighestAddressCap, MM_HIGHEST_VAD_ADDRESS as ULONG_PTR);
    } else {
        HighestAddress = MM_HIGHEST_VAD_ADDRESS as ULONG_PTR;
    }

    // The PEB/TEB pages are committed private memory: charge them (commit.rs). A TEB's comes
    // back in MmDeleteTeb, a PEB's with the rest of the process's ledger at teardown.
    let CommitPages = BYTES_TO_PAGES(Size as usize) as usize;
    if !crate::ARM3::commit::MiChargeCommitment(CommitPages, Process as _) {
        ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
        PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        return crate::ARM3::commit::STATUS_COMMITMENT_LIMIT;
    }

    *BaseAddress = 0;
    Status = MiInsertVadEx(
        Vad as PMMVAD,
        BaseAddress as *mut usize as *mut _,
        (Size as usize) as _,
        HighestAddress as _,
        PAGE_SIZE as _,
        MEM_TOP_DOWN as u32,
    );
    if Status < 0 {
        crate::ARM3::commit::MiReturnCommitment(CommitPages, Process as _);
        ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
        Status = STATUS_NO_MEMORY;
        PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        return Status;
    }

    // Tripwire for a PEB/TEB placed on top of KUSER_SHARED_DATA. That is only
    // possible if this process never got MiInsertSharedUserPageVad's reservation
    // — arm64 lacked it, and the 5th WoW64 thread's TEB duly landed on
    // 0x7FFE0000, where the structure's first page IS the shared page: it read
    // back as the clock, MmCreateTeb's zeroing wiped the page for the WHOLE
    // process, and everything died later in the loader. Refuse at the point of
    // the mistake rather than N threads downstream.
    // The VAD is already in the tree at this point, so it is owned by the
    // process's address-space teardown — do NOT unwind it here. Failing the
    // creation is enough: the caller aborts, and the leaked VAD dies with the
    // process. Hand-unwinding a just-inserted VAD would need the same lock
    // ordering MmDeleteTeb uses, which is not worth the bugcheck risk on a path
    // that the cap above makes unreachable.
    let Base = *BaseAddress;
    if Base < (MM_SHARED_USER_DATA_VA + PAGE_SIZE as ULONG_PTR)
        && (Base + Size as ULONG_PTR) > MM_SHARED_USER_DATA_VA
    {
        MiWow64TraceHex(
            b"[MM] BUG: PEB/TEB VAD overlaps KUSER_SHARED_DATA at \0".as_ptr() as *const i8,
            Base,
        );
        return STATUS_CONFLICTING_ADDRESSES;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmDeleteTeb(Process: PEPROCESS, Teb: PTEB) {
    let Process = opaque_ptr(Process);
    let Thread = PsGetCurrentThread() as PETHREAD;
    let Vad: PMMVAD;
    let VadTree = core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).VadRoot);


    KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb));

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).AddressCreationLock) as *mut KGUARDED_MUTEX);

    Vad = MiLocateAddress(Teb as PVOID);
    if !Vad.is_null() {
        if (*Vad).StartingVpn != ((Teb as ULONG_PTR >> PAGE_SHIFT)) as _ {
            // Bug in AVL code
        } else {
            // Use the VAD's real extent (WoW64 TEB regions are larger than sizeof(TEB)).
            let TebEnd = ((((*Vad).EndingVpn as ULONG_PTR) + 1) << PAGE_SHIFT) - 1;
            MiLockProcessWorkingSetUnsafe(Process, Thread);

            MiRemoveNode(Vad as *mut _, VadTree);

            MiDeleteVirtualAddresses((Teb as ULONG_PTR) as _, TebEnd as _, core::ptr::null_mut());

            MiUnlockProcessWorkingSetUnsafe(Process, Thread);

            // The charge MiCreatePebOrTebCapped took for these pages.
            crate::ARM3::commit::MiReturnCommitment((*Vad).u.VadFlags.CommitCharge() as usize, Process as _);

            ExFreePool(Vad as PVOID);

            PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        }
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).AddressCreationLock) as *mut KGUARDED_MUTEX);

    KeDetachProcess();
}

#[no_mangle]
pub unsafe extern "system" fn MmDeleteKernelStack(StackBase: PVOID, GuiStack: BOOLEAN) {
    let StackBase = opaque_ptr(StackBase);
    let mut PointerPte: PMMPTE;
    let PageFrameNumber: PFN_NUMBER;
    let PageTableFrameNumber: PFN_NUMBER;
    let StackPages: PFN_COUNT;
    let Pfn1: PMMPFN;
    let Pfn2: PMMPFN;
    let mut i: ULONG;
    let OldIrql: KIRQL;
    let SListEntry: PSLIST_ENTRY;

    PointerPte = MiAddressToPte(StackBase);
    PointerPte = PointerPte.sub(1);

    if GuiStack == 0 {
        if ExQueryDepthSList(core::ptr::addr_of_mut!(MmDeadStackSListHead)) < (MmMaximumDeadKernelStacks) as u16 {
            SListEntry = (StackBase as PSLIST_ENTRY).sub(1);
            InterlockedPushEntrySList(core::ptr::addr_of_mut!(MmDeadStackSListHead), SListEntry);
            return;
        }
    }

    StackPages = (BYTES_TO_PAGES(if GuiStack != 0 { MmLargeStackSize as usize } else { KERNEL_STACK_SIZE })) as u32;

    let OldIrql = MiAcquirePfnLock();

    i = 0;
    while i < StackPages {
        if (*PointerPte).valid() {
            let PageFrameNumber = PFN_FROM_PTE(PointerPte);
            let Pfn1 = MiGetPfnEntry(PageFrameNumber);
            #[cfg(target_arch = "aarch64")]
            {
                if Pfn1.is_null() {
                    MI_WRITE_INVALID_PTE(PointerPte, MmDecommittedPte);
                    PointerPte = PointerPte.sub(1);
                    i += 1;
                    continue;
                }
            }

            let PageTableFrameNumber = (*Pfn1).pte_frame();
            let Pfn2 = MiGetPfnEntry(PageTableFrameNumber);

            MiDecrementShareCount(Pfn2 as *mut _MMPFN, (PageTableFrameNumber) as _);

            MI_SET_PFN_DELETED(Pfn1);
            MiDecrementShareCount(Pfn1 as *mut _MMPFN, (PageFrameNumber) as _);
        }

        PointerPte = PointerPte.sub(1);
        i += 1;
    }

    MiReleasePfnLock(OldIrql);

    MiReleaseSystemPtes(PointerPte, StackPages + 1, SystemPteSpace);
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateKernelStack(GuiStack: BOOLEAN, Node: UCHAR) -> PVOID {
    let StackPtes: PFN_COUNT;
    let StackPages: PFN_COUNT;
    let mut PointerPte: PMMPTE;
    let StackPte: PMMPTE;
    let BaseAddress: PVOID;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut InvalidPte: MMPTE = MMPTE { Long: 0 };
    let OldIrql: KIRQL;
    let PageFrameIndex: PFN_NUMBER;
    let mut i: ULONG;
    let SListEntry: PSLIST_ENTRY;

    #[cfg(target_arch = "aarch64")]
    let GuiStack = 1u8;

    let (StackPtes, StackPages) = if GuiStack != 0 {
        #[cfg(target_arch = "aarch64")]
        {
            let ptes = BYTES_TO_PAGES(MmLargeStackSize as usize);
            (ptes, ptes)
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            (BYTES_TO_PAGES(MmLargeStackSize as usize), BYTES_TO_PAGES(KERNEL_LARGE_STACK_COMMIT))
        }
    } else {
        if ExQueryDepthSList(core::ptr::addr_of_mut!(MmDeadStackSListHead)) > 0 {
            let SListEntry = InterlockedPopEntrySList(core::ptr::addr_of_mut!(MmDeadStackSListHead));
            if !SListEntry.is_null() {
                let BaseAddress = (SListEntry as *mut u8).add(core::mem::size_of::<SLIST_ENTRY>()) as PVOID;
                return BaseAddress;
            }
        }

        let ptes = BYTES_TO_PAGES(KERNEL_STACK_SIZE);
        (ptes, ptes)
    };

    let StackPte = MiReserveSystemPtes((StackPtes + 1) as u32, SystemPteSpace);
    if StackPte.is_null() {
        return core::ptr::null_mut();
    }

    let BaseAddress = MiPteToAddress(StackPte.add(StackPtes + 1));

    PointerPte = StackPte;
    #[cfg(not(target_arch = "aarch64"))]
    {
        if GuiStack != 0 {
            PointerPte = PointerPte.add(BYTES_TO_PAGES(MmLargeStackSize as usize - KERNEL_LARGE_STACK_COMMIT));
        }
    }

    MI_MAKE_SOFTWARE_PTE(&mut *core::ptr::addr_of_mut!(InvalidPte), MM_NOACCESS);

    MI_MAKE_HARDWARE_PTE_KERNEL(&mut *core::ptr::addr_of_mut!(TempPte), PointerPte.add(1), MM_READWRITE, 0);

    let OldIrql = MiAcquirePfnLock();

    i = 0;
    while i < (StackPages) as u32 {
        PointerPte = PointerPte.add(1);

        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_KERNEL_STACK);
        ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
        let PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MI_WRITE_INVALID_PTE(PointerPte, InvalidPte);

        MiInitializePfn((PageFrameIndex) as _, PointerPte as *mut _MMPTE, 1);

        TempPte.set_page_frame_number(PageFrameIndex as PFN_NUMBER);
        MI_WRITE_VALID_PTE(PointerPte, TempPte);

        i += 1;
    }

    MiReleasePfnLock(OldIrql);

    BaseAddress
}

unsafe fn MmGrowKernelStackEx(StackPointer: PVOID, GrowSize: ULONG) -> NTSTATUS {
    let Thread = KeGetCurrentThread();
    let mut LimitPte: PMMPTE;
    let NewLimitPte: PMMPTE;
    let LastPte: PMMPTE;
    let OldIrql: KIRQL;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut InvalidPte: MMPTE = MMPTE { Long: 0 };
    let PageFrameIndex: PFN_NUMBER;

    LimitPte = MiAddressToPte((*(Thread as *mut _KTHREAD)).StackLimit as PVOID);

    let NewLimitPte = MiAddressToPte((StackPointer as ULONG_PTR - GrowSize as ULONG_PTR) as PVOID);
    if NewLimitPte == LimitPte {
        return STATUS_SUCCESS;
    }

    let LastPte = MiAddressToPte(((*(Thread as *mut _KTHREAD)).StackBase as ULONG_PTR - MmLargeStackSize as usize) as PVOID);
    if (NewLimitPte as ULONG_PTR) < (LastPte as ULONG_PTR) {
        return STATUS_STACK_OVERFLOW;
    }

    LimitPte = LimitPte.sub(1);

    MI_MAKE_SOFTWARE_PTE(&mut *core::ptr::addr_of_mut!(InvalidPte), MM_NOACCESS);

    let OldIrql = MiAcquirePfnLock();

    while (LimitPte as ULONG_PTR) >= (NewLimitPte as ULONG_PTR) {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_KERNEL_STACK_EXPANSION);
        ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
        let PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MI_WRITE_INVALID_PTE(LimitPte, InvalidPte);

        MiInitializePfn((PageFrameIndex) as _, LimitPte as *mut _MMPTE, 1);

        MI_MAKE_HARDWARE_PTE_KERNEL(&mut *core::ptr::addr_of_mut!(TempPte), LimitPte, MM_READWRITE, PageFrameIndex);

        MI_WRITE_VALID_PTE(LimitPte, TempPte);
        LimitPte = LimitPte.sub(1);
    }

    MiReleasePfnLock(OldIrql);

    (*(Thread as *mut _KTHREAD)).StackLimit = MiPteToAddress(NewLimitPte) as _;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmGrowKernelStack(StackPointer: PVOID) -> NTSTATUS {
    let StackPointer = opaque_ptr(StackPointer);
    MmGrowKernelStackEx(StackPointer, KERNEL_LARGE_STACK_COMMIT as ULONG)
}

#[no_mangle]
pub unsafe extern "system" fn MmSetMemoryPriorityProcess(
    Process: PEPROCESS,
    MemoryPriority: UCHAR,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let mut MemoryPriority = MemoryPriority;

    if (MmSystemSize == MmSmallSystem) && (MmNumberOfPhysicalPages < ((15 * 1024 * 1024) / PAGE_SIZE)) {
        MemoryPriority = MEMORY_PRIORITY_BACKGROUND as UCHAR;
    }

    let OldPriority = (*(Process as *mut _EPROCESS)).Vm.Flags.MemoryPriority() as UCHAR;
    (*(Process as *mut _EPROCESS)).Vm.Flags.set_MemoryPriority(MemoryPriority as u32);

    OldPriority as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePeb(
    Process: PEPROCESS,
    InitialPeb: PINITIAL_PEB,
    BasePeb: *mut PPEB,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let BasePeb = opaque_ptr(BasePeb);
    let mut Peb: PPEB = core::ptr::null_mut();
    let mut SectionOffset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut ViewSize: SIZE_T = 0;
    let mut TableBase: PVOID = core::ptr::null_mut();
    let NtHeaders: PIMAGE_NT_HEADERS;
    let ImageConfigData: PIMAGE_LOAD_CONFIG_DIRECTORY;
    let mut Status: NTSTATUS;
    let Characteristics: USHORT;
    SectionOffset.QuadPart = (0u64 as ULONGLONG) as i64;
    core::ptr::write_volatile(BasePeb, core::ptr::null_mut());

    KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb));

    Status = MmMapViewOfSection(
        ExpNlsSectionPointer,
        Process,
        &mut TableBase,
        0,
        0,
        core::ptr::addr_of_mut!(SectionOffset),
        &mut ViewSize as *mut SIZE_T as *mut _,
        ViewShare,
        (MEM_TOP_DOWN) as u32,
        PAGE_READONLY,
    );
    if Status < 0 {
        KeDetachProcess();
        return Status;
    }

    Status = MiCreatePebOrTeb(Process, core::mem::size_of::<PEB>() as ULONG, &mut Peb as *mut PPEB as PULONG_PTR);
    if Status < 0 {
        KeDetachProcess();
        return Status;
    }

    RtlZeroMemory(Peb as PVOID, core::mem::size_of::<PEB>());

    (*Peb).ImageBaseAddress = (*(Process as *mut _EPROCESS)).SectionBaseAddress;
    (*Peb).InheritedAddressSpace = (*InitialPeb).InheritedAddressSpace;
    (*Peb).Mutant = (*InitialPeb).Mutant;
    // (*Peb).ImageUsesLargePages = (*InitialPeb).ImageUsesLargePages; // ImageUsesLargePages not in bindgen

    (*Peb).AnsiCodePageData = (TableBase as *mut i8).add((ExpAnsiCodePageDataOffset) as usize) as *mut c_void;
    (*Peb).OemCodePageData = (TableBase as *mut i8).add((ExpOemCodePageDataOffset) as usize) as *mut c_void;
    (*Peb).UnicodeCaseTableData = (TableBase as *mut i8).add((ExpUnicodeCaseTableDataOffset) as usize) as *mut c_void;

    (*Peb).OSMajorVersion = NtMajorVersion;
    (*Peb).OSMinorVersion = NtMinorVersion;
    (*Peb).OSBuildNumber = (NtBuildNumber & 0x3FFF) as USHORT;
    (*Peb).OSPlatformId = VER_PLATFORM_WIN32_NT;
    (*Peb).OSCSDVersion = CmNtCSDVersion as USHORT;

    (*Peb).NumberOfProcessors = (KeNumberProcessors) as u32;
    (*Peb).ImageProcessAffinityMask = (KeActiveProcessors as usize) as _;
    (*Peb).BeingDebugged = (if (*(Process as *mut _EPROCESS)).DebugPort.is_null() { 0u8 } else { 1u8 }) as BOOLEAN;
    (*Peb).NtGlobalFlag = NtGlobalFlag;
    (*Peb).HeapSegmentReserve = (MmHeapSegmentReserve) as _;
    (*Peb).HeapSegmentCommit = (MmHeapSegmentCommit) as _;
    (*Peb).HeapDeCommitTotalFreeThreshold = (MmHeapDeCommitTotalFreeThreshold) as _;
    (*Peb).HeapDeCommitFreeBlockThreshold = (MmHeapDeCommitFreeBlockThreshold) as _;
    (*Peb).CriticalSectionTimeout = LARGE_INTEGER { QuadPart: MmCriticalSectionTimeout };
    (*Peb).MinimumStackCommit = (MmMinimumStackCommitInBytes) as _;
    (*Peb).MaximumNumberOfHeaps = ((PAGE_SIZE - core::mem::size_of::<PEB>()) / core::mem::size_of::<PVOID>()) as u32;
    (*Peb).ProcessHeaps = (Peb as *mut u8).add(core::mem::size_of::<PEB>()) as *mut *mut c_void;

    if !(*(Process as *mut _EPROCESS)).Session.is_null() {
        (*Peb).SessionId = MmGetSessionId(Process);
    }

    // C: `_SEH2_TRY { NtHeaders = RtlImageNtHeader(..); } _SEH2_EXCEPT { detach;
    // return STATUS_INVALID_IMAGE_PROTECT; }`. The headers are read through the new
    // process's view of the image, so this is the first touch of a page that must come
    // in from the image FILE, and that read can fail: an amd64 modrun boot bugchecked
    // 0x1E (c0000006, STATUS_IN_PAGE_ERROR) in RtlpImageNtHeaderEx here. Touch the
    // header page through the protected copy first, and use a captured copy after.
    if ntkrnl_sys::probe::read_back((*Peb).ImageBaseAddress as *const u16).is_err() {
        KeDetachProcess();
        return STATUS_INVALID_IMAGE_PROTECT;
    }
    let NtHeaders = RtlImageNtHeader((*Peb).ImageBaseAddress);
    if NtHeaders.is_null() {
        KeDetachProcess();
        return STATUS_INVALID_IMAGE_PROTECT;
    }
    let Nt: _IMAGE_NT_HEADERS = match ntkrnl_sys::probe::read_back(NtHeaders as *const _IMAGE_NT_HEADERS) {
        Ok(h) => h,
        Err(_) => {
            KeDetachProcess();
            return STATUS_INVALID_IMAGE_PROTECT;
        }
    };
    let Characteristics = Nt.FileHeader.Characteristics;

    if !NtHeaders.is_null() {
        let ImageConfigData = RtlImageDirectoryEntryToData(
            (*Peb).ImageBaseAddress,
            1u8,
            (IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG) as u16,
            &mut ViewSize as *mut SIZE_T as *mut ULONG,
        ) as PIMAGE_LOAD_CONFIG_DIRECTORY;
        // C: this block is one _SEH2_TRY whose handler detaches and returns
        // STATUS_INVALID_IMAGE_PROTECT. The config directory lives in the user
        // mapping of the image, so the probe and the two field reads below must
        // not raise/fault unhandled. (The port probed size_of::<c_void>() = 1 byte
        // here; the C probes sizeof(IMAGE_LOAD_CONFIG_DIRECTORY).)
        #[inline(always)]
        fn pointee_size<T>(_: *const T) -> usize { core::mem::size_of::<T>() }
        let mut ConfigCSDVersion: USHORT = 0;
        let mut ConfigAffinityMask: ULONG_PTR = 0;
        if !ImageConfigData.is_null() {
            // C ProbeForRead checks unconditionally: UserMode (1).
            let captured = ntkrnl_sys::probe::probe_user_buffer(
                ImageConfigData as PVOID,
                pointee_size(ImageConfigData),
                core::mem::size_of::<ULONG>(),
                1,
            )
            .and_then(|()| ntkrnl_sys::probe::probe_read_generic(
                core::ptr::addr_of!((*ImageConfigData).CSDVersion), 1, 1))
            .and_then(|csd| ntkrnl_sys::probe::probe_read_generic(
                core::ptr::addr_of!((*ImageConfigData).ProcessAffinityMask), 1, 1)
                .map(|mask| (csd, mask)));
            match captured {
                Ok((csd, mask)) => {
                    ConfigCSDVersion = csd as USHORT;
                    ConfigAffinityMask = mask as ULONG_PTR;
                }
                Err(_) => {
                    KeDetachProcess();
                    return STATUS_INVALID_IMAGE_PROTECT;
                }
            }
        }

        (*Peb).ImageSubsystem = Nt.OptionalHeader.Subsystem as u32;
        (*Peb).ImageSubsystemMajorVersion = Nt.OptionalHeader.MajorSubsystemVersion as u32;
        (*Peb).ImageSubsystemMinorVersion = Nt.OptionalHeader.MinorSubsystemVersion as u32;

        if Nt.OptionalHeader.Win32VersionValue != 0 {
            (*Peb).OSMajorVersion = (Nt.OptionalHeader.Win32VersionValue & 0xFF) as ULONG;
            (*Peb).OSMinorVersion = ((Nt.OptionalHeader.Win32VersionValue >> 8) & 0xFF) as ULONG;
            (*Peb).OSBuildNumber = ((Nt.OptionalHeader.Win32VersionValue >> 16) & 0x3FFF) as USHORT;
            (*Peb).OSPlatformId = ((Nt.OptionalHeader.Win32VersionValue >> 30) ^ 2) as ULONG;

            if !ImageConfigData.is_null() && ConfigCSDVersion != 0 {
                (*Peb).OSCSDVersion = ConfigCSDVersion as _;
            }
        }

        if !ImageConfigData.is_null() && ConfigAffinityMask != 0 {
            (*Peb).ImageProcessAffinityMask = ConfigAffinityMask as _;
        }

        if ((Characteristics as u32) & IMAGE_FILE_UP_SYSTEM_ONLY) != 0 {
            (*Peb).ImageProcessAffinityMask = AFFINITY_MASK(MmRotatingUniprocessorNumber as _) as _;
            MmRotatingUniprocessorNumber = (MmRotatingUniprocessorNumber + 1) % (KeNumberProcessors) as u32;
        }
    }

    // WoW64 (arm64/amd64 host): an i386 image gets a PEB32 below 2GB and the
    // EPROCESS marked as a WoW64 process (Wow64Process = the PEB32, 5.2-style —
    // this is what NtQueryInformationProcess(ProcessWow64Information) hands back).
    // The 32-bit ntdll map + LdrSystemDllInitBlock population happen in user
    // mode (wow64.dll), not here. See docs/engineering/wow64-plan.md Phase C/E.
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    if Nt.FileHeader.Machine == 0x014c /* IMAGE_FILE_MACHINE_I386 */ {
        use ntkrnl_sys::ddk::{_PEB32, _WOW64_PROCESS};
        let mut Peb32Base: ULONG_PTR = 0;
        Status = MiCreatePebOrTebCapped(
            Process,
            core::mem::size_of::<_PEB32>() as ULONG,
            MM_HIGHEST_VAD_ADDRESS_WOW64,
            &mut Peb32Base,
        );
        if Status < 0 {
            KeDetachProcess();
            return Status;
        }
        let Peb32 = Peb32Base as *mut _PEB32;
        RtlZeroMemory(Peb32 as PVOID, core::mem::size_of::<_PEB32>());

        (*Peb32).InheritedAddressSpace = (*Peb).InheritedAddressSpace;
        (*Peb32).Mutant = (*Peb).Mutant as ULONG_PTR as u32;
        (*Peb32).ImageBaseAddress = (*Peb).ImageBaseAddress as ULONG_PTR as u32;
        // NLS pointers: the 64-bit NLS view lives above 4GB, so a truncated copy
        // would be a wild guest pointer. Left 0 for now; a guest-visible low view
        // is owed when 32-bit code starts consuming NLS (Phase C, later step).
        (*Peb32).OSMajorVersion = (*Peb).OSMajorVersion;
        (*Peb32).OSMinorVersion = (*Peb).OSMinorVersion;
        (*Peb32).OSBuildNumber = (*Peb).OSBuildNumber;
        (*Peb32).OSPlatformId = (*Peb).OSPlatformId;
        (*Peb32).OSCSDVersion = (*Peb).OSCSDVersion;
        (*Peb32).NumberOfProcessors = (*Peb).NumberOfProcessors;
        (*Peb32).BeingDebugged = (*Peb).BeingDebugged;
        (*Peb32).NtGlobalFlag = (*Peb).NtGlobalFlag;
        (*Peb32).CriticalSectionTimeout = (*Peb).CriticalSectionTimeout;
        (*Peb32).HeapSegmentReserve = (*Peb).HeapSegmentReserve as u32;
        (*Peb32).HeapSegmentCommit = (*Peb).HeapSegmentCommit as u32;
        (*Peb32).HeapDeCommitTotalFreeThreshold = (*Peb).HeapDeCommitTotalFreeThreshold as u32;
        (*Peb32).HeapDeCommitFreeBlockThreshold = (*Peb).HeapDeCommitFreeBlockThreshold as u32;
        (*Peb32).MaximumNumberOfHeaps =
            ((PAGE_SIZE as usize - core::mem::size_of::<_PEB32>()) / core::mem::size_of::<u32>()) as u32;
        (*Peb32).ProcessHeaps = (Peb32 as usize as u32) + core::mem::size_of::<_PEB32>() as u32;
        (*Peb32).ImageSubsystem = (*Peb).ImageSubsystem;
        (*Peb32).ImageSubsystemMajorVersion = (*Peb).ImageSubsystemMajorVersion;
        (*Peb32).ImageSubsystemMinorVersion = (*Peb).ImageSubsystemMinorVersion;
        (*Peb32).SessionId = (*Peb).SessionId;

        // NLS tables for the 32-bit world: the 64-bit view above (MEM_TOP_DOWN)
        // lands high, so map a SECOND view of the NLS section constrained below
        // 2GB (ZeroBits mask form) and point PEB32's NLS fields at it —
        // RtlInitCodePageTable in the guest ntdll AVs on the NULL/garbage
        // pointers otherwise. Failure is loud-but-nonfatal: the guest gets
        // NULL pointers and dies visibly in NlsInit, not silently.
        let mut Nls32Base: PVOID = core::ptr::null_mut();
        let mut Nls32Size: SIZE_T = 0;
        let mut Nls32Offset: LARGE_INTEGER = core::mem::zeroed();
        let NlsStatus = MmMapViewOfSection(
            ExpNlsSectionPointer,
            Process,
            &mut Nls32Base,
            0x7FFFFFFFu64, /* ZeroBits mask: at-or-below 2GB */
            0,
            core::ptr::addr_of_mut!(Nls32Offset),
            &mut Nls32Size as *mut SIZE_T as *mut _,
            ViewShare,
            0,
            PAGE_READONLY,
        );
        if NlsStatus >= 0 && (Nls32Base as usize) < 0x8000_0000 {
            (*Peb32).AnsiCodePageData =
                (Nls32Base as usize + ExpAnsiCodePageDataOffset as usize) as u32;
            (*Peb32).OemCodePageData =
                (Nls32Base as usize + ExpOemCodePageDataOffset as usize) as u32;
            (*Peb32).UnicodeCaseTableData =
                (Nls32Base as usize + ExpUnicodeCaseTableDataOffset as usize) as u32;
        } else {
            MiWow64TraceHex(
                b"[MM] MmCreatePeb: below-2GB NLS view FAILED status=\0".as_ptr() as *const i8,
                NlsStatus as usize,
            );
        }

        (*(Process as *mut _EPROCESS)).Wow64Process = Peb32 as *mut _WOW64_PROCESS;
        MiWow64TraceHex(
            b"[MM] MmCreatePeb: WoW64 i386 image, PEB32 at \0".as_ptr() as *const i8,
            Peb32Base,
        );
    }

    KeDetachProcess();
    core::ptr::write_volatile(BasePeb, Peb);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateTeb(
    Process: PEPROCESS,
    ClientId: PCLIENT_ID,
    InitialTeb: PINITIAL_TEB,
    BaseTeb: *mut PTEB,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let ClientId = opaque_ptr(ClientId);
    let BaseTeb = opaque_ptr(BaseTeb);
    let mut Teb: PTEB = core::ptr::null_mut();
    let mut Status = STATUS_SUCCESS;
    core::ptr::write_volatile(BaseTeb, core::ptr::null_mut());

    KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb));

    // WoW64 threads get a combined low region: TEB64 at +0, TEB32 at
    // WOW64_TEB32_OFFSET (sizeof(TEB) = 0x17d8 → TEB64 spans 2 pages), whole
    // thing below 2GB so 32-bit guest pointers can reach both. This function
    // is the allocator; the layout constant lives in ntkrnl-sys so the readers
    // cannot drift from it.
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    use ntkrnl_sys::wow64::WOW64_TEB32_OFFSET;
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    let IsWow64 = !(*(Process as *mut _EPROCESS)).Wow64Process.is_null();
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    let IsWow64 = false;
    if IsWow64 {
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            Status = MiCreatePebOrTebCapped(
                Process,
                (WOW64_TEB32_OFFSET + ROUND_TO_PAGES(core::mem::size_of::<ntkrnl_sys::ddk::_TEB32>())) as ULONG,
                MM_HIGHEST_VAD_ADDRESS_WOW64,
                &mut Teb as *mut PTEB as PULONG_PTR,
            );
        }
    } else {
        Status = MiCreatePebOrTeb(Process, core::mem::size_of::<TEB>() as ULONG, &mut Teb as *mut PTEB as PULONG_PTR);
    }
    if Status < 0 {
        KeDetachProcess();
        return Status;
    }

    RtlZeroMemory(Teb as PVOID, core::mem::size_of::<TEB>());

    #[cfg(target_arch = "x86_64")]
    {
        (*Teb).NtTib.ExceptionList = core::ptr::null_mut();
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        (*Teb).NtTib.ExceptionList = EXCEPTION_CHAIN_END as _;
    }
    (*Teb).NtTib.Self_ = Teb as PNT_TIB;

    // (*Teb).NtTib.Version_ = 30 << 8; // NtTib.Version not in bindgen

    (*Teb).ClientId = *ClientId;
    (*Teb).RealClientId = *ClientId;
    (*Teb).ProcessEnvironmentBlock = (*(Process as *mut _EPROCESS)).Peb;
    (*Teb).CurrentLocale = PsDefaultThreadLocaleId;

    if (*InitialTeb).PreviousStackBase.is_null() && (*InitialTeb).PreviousStackLimit.is_null() {
        (*Teb).NtTib.StackBase = (*InitialTeb).StackBase;
        (*Teb).NtTib.StackLimit = (*InitialTeb).StackLimit;
        (*Teb).DeallocationStack = (*InitialTeb).AllocatedStackBase;
    } else {
        (*Teb).NtTib.StackBase = (*InitialTeb).PreviousStackBase;
        (*Teb).NtTib.StackLimit = (*InitialTeb).PreviousStackLimit;
    }

    (*Teb).StaticUnicodeString.MaximumLength = core::mem::size_of_val(&(*Teb).StaticUnicodeBuffer) as USHORT;
    (*Teb).StaticUnicodeString.Buffer = (*Teb).StaticUnicodeBuffer.as_mut_ptr();

    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    if IsWow64 {
        use ntkrnl_sys::ddk::_TEB32;
        let Teb32 = ntkrnl_sys::wow64::teb32_of(Teb) as *mut _TEB32;
        debug_assert_eq!(Teb32 as usize, Teb as usize + WOW64_TEB32_OFFSET);
        RtlZeroMemory(Teb32 as PVOID, core::mem::size_of::<_TEB32>());

        (*Teb32).NtTib.ExceptionList = 0xFFFF_FFFF; // EXCEPTION_CHAIN_END, 32-bit
        (*Teb32).NtTib.Self_ = Teb32 as usize as u32;
        (*Teb32).ClientId.UniqueProcess = (*ClientId).UniqueProcess as usize as u32;
        (*Teb32).ClientId.UniqueThread = (*ClientId).UniqueThread as usize as u32;
        (*Teb32).RealClientId = (*Teb32).ClientId;
        (*Teb32).ProcessEnvironmentBlock =
            (*(Process as *mut _EPROCESS)).Wow64Process as usize as u32; // = the PEB32
        (*Teb32).CurrentLocale = PsDefaultThreadLocaleId;
        (*Teb32).StaticUnicodeString.MaximumLength =
            core::mem::size_of_val(&(*Teb32).StaticUnicodeBuffer) as USHORT;
        (*Teb32).StaticUnicodeString.Buffer =
            core::ptr::addr_of_mut!((*Teb32).StaticUnicodeBuffer) as usize as u32;
        // Guest stack bounds are set by wow64.dll at thread init.

        // WowTebOffset lives at TEB+0x180c / TEB32+0xfdc in the Win10 layout that
        // wow64fex hardcodes (GetWowTEB). Our 5.2-era structs end before those
        // offsets (sizeof TEB = 0x17d8, TEB32 = 0xfbc) but the pages are committed,
        // so the slots are safe scratch — write them raw.
        core::ptr::write_volatile(
            (Teb as *mut u8).add(0x180c) as *mut i32, // nolint: magic-offset (Win10 TEB.WowTebOffset slot wow64fex hardcodes; past 5.2 struct end, committed scratch)
            WOW64_TEB32_OFFSET as i32,
        );
        core::ptr::write_volatile(
            (Teb32 as *mut u8).add(0xfdc) as *mut i32, // nolint: magic-offset (Win10 TEB32.WowTebOffset slot; past 5.2 struct end, committed scratch)
            -(WOW64_TEB32_OFFSET as i32),
        );
        MiWow64TraceHex(
            b"[MM] MmCreateTeb: WoW64 TEB32 at \0".as_ptr() as *const i8,
            Teb32 as ULONG_PTR,
        );
    }

    KeDetachProcess();
    core::ptr::write_volatile(BaseTeb, Teb);
    Status
}

/// Reserve `MM_SHARED_USER_DATA_VA` with a real VAD so the VAD allocator knows
/// the page is taken. Without it nothing in the tree marks 0x7FFE0000, and a
/// top-down allocation capped at the 32-bit ceiling — i.e. every WoW64 PEB32/TEB
/// — will eventually be placed right on top of KUSER_SHARED_DATA.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn MiInsertSharedUserPageVad(Process: PEPROCESS) -> NTSTATUS {
    let Vad: PMMVAD_LONG;
    let mut BaseAddress: ULONG_PTR;
    let mut Status: NTSTATUS;

    if !(*(Process as *mut _EPROCESS)).QuotaBlock.is_null() {
        Status = PsChargeProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        if Status < 0 {
            return Status;
        }
    }

    let Vad = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_MMVAD_LONG>(),
        u32::from_le_bytes(*b"Vadl"),
    ) as PMMVAD_LONG;
    if Vad.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if !(*(Process as *mut _EPROCESS)).QuotaBlock.is_null() {
            PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        }
        return Status;
    }

    (*Vad).u.LongFlags = 0;
    (*Vad).u.VadFlags.set_CommitCharge(0);
    (*Vad).u.VadFlags.set_NoChange(1);
    (*Vad).u.VadFlags.set_VadType((VadNone as usize) as _);
    (*Vad).u.VadFlags.set_MemCommit(0);
    (*Vad).u.VadFlags.set_Protection((MM_READONLY as usize) as _);
    (*Vad).u.VadFlags.set_PrivateMemory(1);
    (*Vad).u1.Parent = core::ptr::null_mut();

    (*Vad).u2.LongFlags2 = 0;
    (*Vad).u2.VadFlags2.set_OneSecured(1);
    (*Vad).u2.VadFlags2.set_LongVad(1);
    (*Vad).u2.VadFlags2.set_ReadOnly(0);

    (*Vad).ControlArea = core::ptr::null_mut();
    (*Vad).FirstPrototypePte = core::ptr::null_mut();

    BaseAddress = MM_SHARED_USER_DATA_VA as ULONG_PTR;
    Status = MiInsertVadEx(
        Vad as PMMVAD,
        &mut BaseAddress as *mut usize as *mut _,
        (PAGE_SIZE) as _,
        (MM_HIGHEST_VAD_ADDRESS as ULONG_PTR) as _,
        PAGE_SIZE as _,
        MEM_TOP_DOWN as u32,
    );
    if Status < 0 {
        ExFreePoolWithTag(Vad as PVOID, u32::from_le_bytes(*b"Vadl"));
        if !(*(Process as *mut _EPROCESS)).QuotaBlock.is_null() {
            PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
        }
        return Status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmInitializeProcessAddressSpace(
    Process: PEPROCESS,
    ProcessClone: PEPROCESS,
    Section: PVOID,
    Flags: *mut ULONG,
    AuditName: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let ProcessClone = opaque_ptr(ProcessClone);
    let Section = opaque_ptr(Section);
    let Flags = opaque_ptr(Flags);
    let AuditName = opaque_ptr(AuditName);
    let mut Status = STATUS_SUCCESS;
    let mut ViewSize: SIZE_T = 0;
    let mut ImageBase: PVOID = core::ptr::null_mut();
    let PointerPte: PMMPTE;
    let mut OldIrql: KIRQL;
    let PointerPde: PMMPTE;
    let Pfn: PMMPFN;
    let mut PageFrameNumber: PFN_NUMBER;
    let FileName: _UNICODE_STRING;
    let Source: *mut u16;
    let Destination: *mut i8;
    let mut Length: USHORT = 0;

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let PointerPpe: PMMPTE;
    #[cfg(target_arch = "aarch64")]
    let PointerPxe: PMMPTE;

    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"InitProcAS-entry\0".as_ptr() as *const i8);
    }

    #[cfg(target_arch = "aarch64")]
    ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL before attach=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);
    KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb));
    #[cfg(target_arch = "aarch64")]
    ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL after attach=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);

    (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.set_AddressSpaceInitialized(2 as ULONG);

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).AddressCreationLock) as *mut KGUARDED_MUTEX);
    (*(Process as *mut _EPROCESS)).Vm.WorkingSetExpansionLinks.Flink = core::ptr::null_mut();

    (*(Process as *mut _EPROCESS)).VadRoot.BalancedRoot.u1.Parent = core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).VadRoot.BalancedRoot) as *mut _;

    #[cfg(target_arch = "aarch64")]
    ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL before WS lock=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);
    MiLockProcessWorkingSet(Process, PsGetCurrentThread() as PETHREAD);
    #[cfg(target_arch = "aarch64")]
    ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL after WS lock=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);

    OldIrql = MiAcquirePfnLock();

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let PointerPte = MiAddressToPte(PXE_BASE as PVOID);
    #[cfg(all(not(target_arch = "x86_64"), not(target_arch = "aarch64")))]
    let PointerPte = MiAddressToPte(PDE_BASE as PVOID);

    PageFrameNumber = PFN_FROM_PTE(PointerPte);
    MiInitializePfn((PageFrameNumber) as _, PointerPte as *mut _MMPTE, 1u8);

    let PointerPde = MiAddressToPde(HYPER_SPACE as PVOID);
    PageFrameNumber = PFN_FROM_PTE(PointerPde);
    MiInitializePfn((PageFrameNumber) as _, PointerPde as *mut _MMPTE, 1u8);

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let PointerPpe = MiAddressToPpe(HYPER_SPACE as PVOID);
        PageFrameNumber = PFN_FROM_PTE(PointerPpe as PMMPTE);
        MiInitializePfn((PageFrameNumber) as _, PointerPpe as *mut _MMPTE, 1u8);
    }
    // _MI_PAGING_LEVELS == 4 (amd64 AND arm64) — init the PFN for the hyperspace
    // PXE's target (the top-level page-directory-pointer page = DirectoryTableBase[1]).
    // This block was previously gated aarch64-only, so on amd64 the hyperspace PDPT's
    // PFN entry was never initialized (stayed loc=ZeroedPageList / ref=0 with garbage
    // u4.PteFrame). MmDeleteProcessAddressSpace then read that garbage PteFrame and
    // called MiDecrementShareCount(NULL, garbage) → BUGCHECK 0x4E (0x98) at process
    // teardown. amd64 is also 4-level paging, so it needs this exactly like arm64.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let PointerPxe = MiAddressToPxe(HYPER_SPACE as PVOID);
        PageFrameNumber = PFN_FROM_PTE(PointerPxe as PMMPTE);
        MiInitializePfn(PageFrameNumber as _, PointerPxe as *mut _MMPTE, 1u8);
    }

    let PointerPte = MiAddressToPte(MmWorkingSetList as PVOID);
    PageFrameNumber = PFN_FROM_PTE(PointerPte);
    MiInitializePfn((PageFrameNumber) as _, PointerPte as *mut _MMPTE, 1u8);

    MiReleasePfnLock(OldIrql);

    (*(Process as *mut _EPROCESS)).Vm.VmWorkingSetList = MmWorkingSetList as _;

    MiInitializeWorkingSetList(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm));

    let Pfn = MiGetPfnEntry(((*(Process as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] >> PAGE_SHIFT) as usize);
    (*Pfn).set_flink((Process as *mut _ as *mut _) as PFN_NUMBER);

    MiUnlockProcessWorkingSet(Process, PsGetCurrentThread() as PETHREAD);
    #[cfg(target_arch = "aarch64")]
    ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL after WS unlock=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        Status = MiInsertSharedUserPageVad(Process);
        if Status < 0 {
            return Status;
        }
    }

    if !Section.is_null() {
        let FileObject = MmGetFileObjectForSection(Section) as PFILE_OBJECT;
        let FileName = (*FileObject).FileName;
        let mut Source = (FileName.Buffer as *mut u8).add(FileName.Length as usize) as *mut u16;
        if !FileName.Buffer.is_null() {
            while (Source as usize) > (FileName.Buffer as usize) {
                Source = Source.sub(1);
                if *Source == OBJ_NAME_PATH_SEPARATOR as u16 {
                    Source = Source.add(1);
                    break;
                } else {
                    Length += 1;
                }
            }
        }

        let mut Destination = (*(Process as *mut _EPROCESS)).ImageFileName.as_mut_ptr();
        Length = core::cmp::min(Length, (core::mem::size_of_val(&(*(Process as *mut _EPROCESS)).ImageFileName) - 1) as u16);
        let mut i = 0u16;
        while i < Length {
            *Destination = (*Source.add(i as usize)) as u8 as i8;
            Destination = Destination.add(1);
            i += 1;
        }
        *Destination = 0i8;

        if !AuditName.is_null() {
            Status = SeInitializeProcessAuditName(FileObject as PVOID, 0u8, AuditName as *mut PVOID);
            if Status < 0 {
                KeDetachProcess();
                return Status;
            }
        }

        #[cfg(target_arch = "aarch64")]
        ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL before map=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);
        Status = MmMapViewOfSection(
            Section,
            Process,
            &mut ImageBase,
            0,
            0,
            core::ptr::null_mut(),
            &mut ViewSize as *mut SIZE_T as *mut _,
            ViewUnmap,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        #[cfg(target_arch = "aarch64")]
        ARM64_PROC_TRACE_HEX(b"[MM] InitProcAS: IRQL after map=\0".as_ptr() as *const i8, KeGetCurrentIrql() as ULONG_PTR);

        (*(Process as *mut _EPROCESS)).SectionBaseAddress = ImageBase;
    }

    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"before-detach\0".as_ptr() as *const i8);
    }
    KeDetachProcess();
    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"after-detach\0".as_ptr() as *const i8);
    }

    Status
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitializeHandBuiltProcess(
    Process: PEPROCESS,
    DirectoryTableBase: PULONG_PTR,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let DirectoryTableBase = opaque_ptr(DirectoryTableBase);
    *DirectoryTableBase.add(0) = ((*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[0]) as usize;
    *DirectoryTableBase.add(1) = ((*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[1]) as usize;

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).AddressCreationLock) as *mut KGUARDED_MUTEX);
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).HyperSpaceLock));
    (*(Process as *mut _EPROCESS)).Vm.WorkingSetExpansionLinks.Flink = core::ptr::null_mut();
    (*(Process as *mut _EPROCESS)).VadRoot.BalancedRoot.u1.Parent = core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).VadRoot.BalancedRoot) as *mut _;

    (*(Process as *mut _EPROCESS)).Vm.VmWorkingSetList = (*(PsGetCurrentProcess() as *mut _EPROCESS)).Vm.VmWorkingSetList;
    (*(Process as *mut _EPROCESS)).WorkingSetPage = (*(PsGetCurrentProcess() as *mut _EPROCESS)).WorkingSetPage;

    (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.set_HasAddressSpace(1 as ULONG);
    STATUS_SUCCESS
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitializeHandBuiltProcess2(Process: PEPROCESS) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateProcessAddressSpace(
    MinWs: ULONG,
    Process: PEPROCESS,
    DirectoryTableBase: PULONG_PTR,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let DirectoryTableBase = opaque_ptr(DirectoryTableBase);
    let mut OldIrql: KIRQL;
    let mut TableBaseIndex: PFN_NUMBER;
    let mut HyperIndex: PFN_NUMBER;
    let mut WsListIndex: PFN_NUMBER;
    let Color: ULONG;

    (*(Process as *mut _EPROCESS)).NextPageColor = RtlRandom(core::ptr::addr_of_mut!(MmProcessColorSeed)) as USHORT;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).HyperSpaceLock));

    OldIrql = MiAcquirePfnLock();

    let Color = MI_GET_NEXT_PROCESS_COLOR(Process as *mut c_void);
    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_DIRECTORY);
    TableBaseIndex = (MiRemoveZeroPageSafe(Color)) as usize;
    if TableBaseIndex == 0 {
        TableBaseIndex = MiRemoveAnyPage(Color);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(TableBaseIndex);
        OldIrql = MiAcquirePfnLock();
    }
    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_DIRECTORY);
    let Color = MI_GET_NEXT_PROCESS_COLOR(Process as *mut c_void);
    HyperIndex = (MiRemoveZeroPageSafe(Color)) as usize;
    if HyperIndex == 0 {
        HyperIndex = MiRemoveAnyPage(Color);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(HyperIndex);
        OldIrql = MiAcquirePfnLock();
    }
    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_TABLE);
    let Color = MI_GET_NEXT_PROCESS_COLOR(Process as *mut c_void);
    WsListIndex = (MiRemoveZeroPageSafe(Color)) as usize;
    if WsListIndex == 0 {
        WsListIndex = MiRemoveAnyPage(Color);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(WsListIndex);
    } else {
        MiReleasePfnLock(OldIrql);
    }

    (*(Process as *mut _EPROCESS)).WorkingSetPage = (WsListIndex) as _;
    *DirectoryTableBase.add(0) = TableBaseIndex << PAGE_SHIFT;
    *DirectoryTableBase.add(1) = HyperIndex << PAGE_SHIFT;

    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"CreateProcAS-before-MiArch\0".as_ptr() as *const i8);
    }
    if MiArchCreateProcessAddressSpace(Process, DirectoryTableBase as *mut _ as *mut _) == 0 {
        OldIrql = MiAcquirePfnLock();
        MiInsertPageInFreeList(WsListIndex);
        MiInsertPageInFreeList(HyperIndex);
        MiInsertPageInFreeList(TableBaseIndex);
        MiReleasePfnLock(OldIrql);
        (*(Process as *mut _EPROCESS)).WorkingSetPage = 0;
        *DirectoryTableBase.add(0) = 0;
        *DirectoryTableBase.add(1) = 0;
        return 0u8;
    }

    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"CreateProcAS-after-MiArch\0".as_ptr() as *const i8);
    }

    (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.set_AddressSpaceInitialized(1 as ULONG);

    MiSessionAddProcess(Process);

    #[cfg(target_arch = "aarch64")]
    {
        MmTraceIrql(b"CreateProcAS-exit\0".as_ptr() as *const i8);
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn MmCleanProcessAddressSpace(Process: PEPROCESS) {
    let Process = opaque_ptr(Process);
    let mut Vad: PMMVAD;
    let VadTree: *mut _MM_AVL_TABLE;
    let Thread = PsGetCurrentThread() as PETHREAD;

    MiSessionRemoveProcess();

    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.AddressSpaceInitialized() < 2 {
        return;
    }

    MmLockAddressSpace(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm));
    MiLockProcessWorkingSetUnsafe(Process, Thread);

    (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.set_VmDeleted(1u8 as ULONG);
    MiUnlockProcessWorkingSetUnsafe(Process, Thread);

    let VadTree = core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).VadRoot);
    while (*VadTree).NumberGenericTableElements() > 0 {
        Vad = (*VadTree).BalancedRoot.RightChild as PMMVAD;

        if MI_IS_MEMORY_AREA_VAD(Vad as *mut c_void) {
            MiRosCleanupMemoryArea(Process, Vad);
            continue;
        }

        MiLockProcessWorkingSetUnsafe(Process, Thread);

        MiRemoveNode(Vad as *mut _, VadTree);

        if (*Vad).u.VadFlags.PrivateMemory() == 0 && !(*Vad).ControlArea.is_null() {
            MiRemoveMappedView(Process, Vad);
        } else {
            MiDeleteVirtualAddresses(
                ((*Vad).StartingVpn << PAGE_SHIFT) as _,
                ((((*Vad).EndingVpn as usize) << PAGE_SHIFT) | (PAGE_SIZE - 1)) as _,
                Vad,
            );

            MiUnlockProcessWorkingSetUnsafe(Process, Thread);
        }

        ExFreePool(Vad as PVOID);

        PsReturnProcessNonPagedPoolQuota(Process as *mut c_void, core::mem::size_of::<_MMVAD_LONG>());
    }

    MiLockProcessWorkingSetUnsafe(Process, Thread);

    MiDeleteVirtualAddresses((USER_SHARED_DATA as ULONG_PTR) as _, (USER_SHARED_DATA as ULONG_PTR) as _, core::ptr::null_mut());

    MiUnlockProcessWorkingSetUnsafe(Process, Thread);

    // Every VAD is gone, so is every page its commit charge stood for: the rest of
    // the process's ledger goes back in one go, whichever path removed each VAD.
    super::commit::MiReturnProcessCommitment(Process as _);

    super::census::MiMemoryCensusAtExit(Process as _);

    MmUnlockAddressSpace(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm));
}

#[no_mangle]
pub unsafe extern "system" fn MmDeleteProcessAddressSpace(Process: PEPROCESS) {
    let Process = opaque_ptr(Process);
    let Pfn1: PMMPFN;
    let Pfn2: PMMPFN;
    let mut OldIrql: KIRQL;
    let mut PageFrameIndex: PFN_NUMBER;

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        OldIrql = MiAcquireExpansionLock();
        RemoveEntryList(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).MmProcessLinks) as *mut _);
        MiReleaseExpansionLock(OldIrql);
    }

    OldIrql = MiAcquireExpansionLock();
    if !(*(Process as *mut _EPROCESS)).Vm.WorkingSetExpansionLinks.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm.WorkingSetExpansionLinks) as *mut _);
    }
    MiReleaseExpansionLock(OldIrql);

    OldIrql = MiAcquirePfnLock();

    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.AddressSpaceInitialized() == 2 {
        let Pfn1 = MiGetPfnEntry(((*(Process as *mut _EPROCESS)).WorkingSetPage) as usize);
        let Pfn2 = MiGetPfnEntry((*Pfn1).pte_frame());

        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn2 as *mut _MMPFN, ((*Pfn1).pte_frame()) as _);
        MiDecrementShareCount(Pfn1 as *mut _MMPFN, (*(Process as *mut _EPROCESS)).WorkingSetPage);

        PageFrameIndex = ((*(Process as *mut _EPROCESS)).Pcb.DirectoryTableBase[1] >> PAGE_SHIFT) as usize;
        let Pfn1 = MiGetPfnEntry(PageFrameIndex);
        let Pfn2 = MiGetPfnEntry((*Pfn1).pte_frame());

        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn2 as *mut _MMPFN, ((*Pfn1).pte_frame()) as _);
        MiDecrementShareCount(Pfn1 as *mut _MMPFN, (PageFrameIndex) as _);

        PageFrameIndex = ((*(Process as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] >> PAGE_SHIFT) as usize;
        let Pfn1 = MiGetPfnEntry(PageFrameIndex);
        MI_SET_PFN_DELETED(Pfn1);
        MiDecrementShareCount(Pfn1 as *mut _MMPFN, (PageFrameIndex) as _);
        MiDecrementShareCount(Pfn1 as *mut _MMPFN, (PageFrameIndex) as _);
    }

    MiReleasePfnLock(OldIrql);

    if !(*(Process as *mut _EPROCESS)).Session.is_null() {
        MiReleaseProcessReferenceToSessionDataPage((*(Process as *mut _EPROCESS)).Session);
    }

    (*(Process as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] = 0;
    (*(Process as *mut _EPROCESS)).Pcb.DirectoryTableBase[1] = 0;
}

#[no_mangle]
pub unsafe extern "system" fn NtAllocateUserPhysicalPages(
    ProcessHandle: HANDLE,
    NumberOfPages: *mut ULONG_PTR,
    UserPfnArray: *mut ULONG_PTR,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let NumberOfPages = opaque_ptr(NumberOfPages);
    let UserPfnArray = opaque_ptr(UserPfnArray);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtMapUserPhysicalPages(
    VirtualAddresses: PVOID,
    NumberOfPages: ULONG_PTR,
    UserPfnArray: *mut ULONG_PTR,
) -> NTSTATUS {
    let VirtualAddresses = opaque_ptr(VirtualAddresses);
    let UserPfnArray = opaque_ptr(UserPfnArray);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtMapUserPhysicalPagesScatter(
    VirtualAddresses: *mut PVOID,
    NumberOfPages: ULONG_PTR,
    UserPfnArray: *mut ULONG_PTR,
) -> NTSTATUS {
    let VirtualAddresses = opaque_ptr(VirtualAddresses);
    let UserPfnArray = opaque_ptr(UserPfnArray);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtFreeUserPhysicalPages(
    ProcessHandle: HANDLE,
    NumberOfPages: *mut ULONG_PTR,
    UserPfnArray: *mut ULONG_PTR,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let NumberOfPages = opaque_ptr(NumberOfPages);
    let UserPfnArray = opaque_ptr(UserPfnArray);
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/procsup.c
//  * PURPOSE:         ARM Memory Manager Process Related Management
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
// ULONG MmProcessColorSeed = 0x12345678;
// ULONG MmMaximumDeadKernelStacks = 5;
// SLIST_HEADER MmDeadStackSListHead;
// ULONG MmRotatingUniprocessorNumber = 0;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// MiCreatePebOrTeb(IN PEPROCESS Process,
//                  IN ULONG Size,
//                  OUT PULONG_PTR BaseAddress)
// {
//     PMMVAD_LONG Vad;
//     NTSTATUS Status;
//     ULONG_PTR HighestAddress, RandomBase;
//     ULONG AlignedSize;
//     LARGE_INTEGER CurrentTime;
//
//     Status = PsChargeProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate a VAD */
//     Vad = ExAllocatePoolWithTag(NonPagedPool, sizeof(MMVAD_LONG), 'ldaV');
//     if (!Vad)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto FailPath;
//     }
//
//     /* Setup the primary flags with the size, and make it commited, private, RW */
//     Vad->u.LongFlags = 0;
//     Vad->u.VadFlags.CommitCharge = BYTES_TO_PAGES(Size);
//     Vad->u.VadFlags.MemCommit() = TRUE;
//     Vad->u.VadFlags.PrivateMemory() = TRUE;
//     Vad->u.VadFlags.Protection() = MM_READWRITE;
//     Vad->u.VadFlags.NoChange = TRUE;
//     Vad->u1.Parent = NULL;
//
//     /* Setup the secondary flags to make it a secured, writable, long VAD */
//     Vad->u2.LongFlags2 = 0;
//     Vad->u2.VadFlags2.OneSecured = TRUE;
//     Vad->u2.VadFlags2.LongVad = TRUE;
//     Vad->u2.VadFlags2.ReadOnly = FALSE;
//
//     Vad->ControlArea = NULL; // For Memory-Area hack
//     Vad->FirstPrototypePte = NULL;
//
//     /* Check if this is a PEB creation */
//     ASSERT(sizeof(TEB) != sizeof(PEB));
//     if (Size == sizeof(PEB))
//     {
//         /* Create a random value to select one page in a 64k region */
//         KeQueryTickCount(&CurrentTime);
//         CurrentTime.LowPart &= (_64K / PAGE_SIZE) - 1;
//
//         /* Calculate a random base address */
//         RandomBase = (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS + 1;
//         RandomBase -= CurrentTime.LowPart << PAGE_SHIFT;
//
//         /* Make sure the base address is not too high */
//         AlignedSize = ROUND_TO_PAGES(Size);
//         if ((RandomBase + AlignedSize) > (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS + 1)
//         {
//             RandomBase = (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS + 1 - AlignedSize;
//         }
//
//         /* Calculate the highest allowed address */
//         HighestAddress = RandomBase + AlignedSize - 1;
//     }
//     else
//     {
//         HighestAddress = (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS;
//     }
//
//     *BaseAddress = 0;
//     Status = MiInsertVadEx((PMMVAD)Vad,
//                            BaseAddress,
//                            Size,
//                            HighestAddress,
//                            PAGE_SIZE,
//                            MEM_TOP_DOWN);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Vad, 'ldaV');
//         Status = STATUS_NO_MEMORY;
//         goto FailPath;
//     }
//
//
//     /* Success */
//     return STATUS_SUCCESS;
//
// FailPath:
//     PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//
//     return Status;
// }
//
// VOID
// NTAPI
// MmDeleteTeb(IN PEPROCESS Process,
//             IN PTEB Teb)
// {
//     ULONG_PTR TebEnd;
//     PETHREAD Thread = PsGetCurrentThread();
//     PMMVAD Vad;
//     PMM_AVL_TABLE VadTree = &Process->VadRoot;
//     DPRINT("Deleting TEB: %p in %16s\n", Teb, Process->ImageFileName);
//
//     /* TEB is one page */
//     TebEnd = (ULONG_PTR)Teb + ROUND_TO_PAGES(sizeof(TEB)) - 1;
//
//     /* Attach to the process */
//     KeAttachProcess(&Process->Pcb);
//
//     /* Lock the process address space */
//     KeAcquireGuardedMutex(&Process->AddressCreationLock);
//
//     /* Find the VAD, make sure it's a TEB VAD */
//     Vad = MiLocateAddress(Teb);
//     DPRINT("Removing node for VAD: %lx %lx\n", Vad->StartingVpn, Vad->EndingVpn);
//     ASSERT(Vad != NULL);
//     if (Vad->StartingVpn != ((ULONG_PTR)Teb >> PAGE_SHIFT))
//     {
//         /* Bug in the AVL code? */
//         DPRINT1("Corrupted VAD!\n");
//     }
//     else
//     {
//         /* Sanity checks for a valid TEB VAD */
//         ASSERT((Vad->StartingVpn == ((ULONG_PTR)Teb >> PAGE_SHIFT) &&
//                (Vad->EndingVpn == (TebEnd >> PAGE_SHIFT))));
//         ASSERT(Vad->u.VadFlags.NoChange == TRUE);
//         ASSERT(Vad->u2.VadFlags2.OneSecured == TRUE);
//         ASSERT(Vad->u2.VadFlags2.MultipleSecured == FALSE);
//
//         /* Lock the working set */
//         MiLockProcessWorkingSetUnsafe(Process, Thread);
//
//         /* Remove this VAD from the tree */
//         ASSERT(VadTree->NumberGenericTableElements >= 1);
//         MiRemoveNode((PMMADDRESS_NODE)Vad, VadTree);
//
//         /* Delete the pages */
//         MiDeleteVirtualAddresses((ULONG_PTR)Teb, TebEnd, NULL);
//
//         /* Release the working set */
//         MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//
//         /* Remove the VAD */
//         ExFreePool(Vad);
//
//         /* Return the quota the VAD used */
//         PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     }
//
//     /* Release the address space lock */
//     KeReleaseGuardedMutex(&Process->AddressCreationLock);
//
//     /* Detach */
//     KeDetachProcess();
// }
//
// VOID
// NTAPI
// MmDeleteKernelStack(IN PVOID StackBase,
//                     IN BOOLEAN GuiStack)
// {
//     PMMPTE PointerPte;
//     PFN_NUMBER PageFrameNumber, PageTableFrameNumber;
//     PFN_COUNT StackPages;
//     PMMPFN Pfn1, Pfn2;
//     ULONG i;
//     KIRQL OldIrql;
//     PSLIST_ENTRY SListEntry;
//
//     //
//     // This should be the guard page, so decrement by one
//     //
//     PointerPte = MiAddressToPte(StackBase);
//     PointerPte--;
//
//     //
//     // If this is a small stack, just push the stack onto the dead stack S-LIST
//     //
//     if (!GuiStack)
//     {
//         if (ExQueryDepthSList(&MmDeadStackSListHead) < MmMaximumDeadKernelStacks)
//         {
//             SListEntry = ((PSLIST_ENTRY)StackBase) - 1;
//             InterlockedPushEntrySList(&MmDeadStackSListHead, SListEntry);
//             return;
//         }
//     }
//
//     //
//     // Calculate pages used
//     //
//     StackPages = BYTES_TO_PAGES(GuiStack ?
//                                 MmLargeStackSize : KERNEL_STACK_SIZE);
//
//     /* Acquire the PFN lock */
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // Loop them
//     //
//     for (i = 0; i < StackPages; i++)
//     {
//         //
//         // Check if this is a valid PTE
//         //
//         if (PointerPte->u.Hard.Valid == 1)
//         {
//             /* Get the PTE's page */
//             PageFrameNumber = PFN_FROM_PTE(PointerPte);
//             Pfn1 = MiGetPfnEntry(PageFrameNumber);
// #ifdef _M_ARM64
//             if (!Pfn1)
//             {
//                 /* MiGetPfnEntry returned NULL — PFN bitmap doesn't cover this page.
//                  * Just invalidate the PTE and skip PFN accounting. */
//                 MI_WRITE_INVALID_PTE(PointerPte, MmDecommittedPte);
//                 PointerPte--;
//                 continue;
//             }
// #endif
//
//             /* Now get the page of the page table mapping it */
//             PageTableFrameNumber = Pfn1->u4.PteFrame;
//             Pfn2 = MiGetPfnEntry(PageTableFrameNumber);
//
//             /* Remove a shared reference, since the page is going away */
// #ifdef _M_ARM64
//             if (Pfn2)
// #endif
//             MiDecrementShareCount(Pfn2 as *mut _MMPFN, PageTableFrameNumber as u64);
//
//             /* Set the special pending delete marker */
//             MI_SET_PFN_DELETED(Pfn1);
//
//             /* And now delete the actual stack page */
//             MiDecrementShareCount(Pfn1, PageFrameNumber);
//         }
//
//         //
//         // Next one
//         //
//         PointerPte--;
//     }
//
//     //
//     // We should be at the guard page now
//     //
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//
//     /* Release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // Release the PTEs
//     //
//     MiReleaseSystemPtes(PointerPte, StackPages + 1, SystemPteSpace);
// }
//
// PVOID
// NTAPI
// MmCreateKernelStack(IN BOOLEAN GuiStack,
//                     IN UCHAR Node)
// {
//     PFN_COUNT StackPtes, StackPages;
//     PMMPTE PointerPte, StackPte;
//     PVOID BaseAddress;
//     MMPTE TempPte, InvalidPte;
//     KIRQL OldIrql;
//     PFN_NUMBER PageFrameIndex;
//     ULONG i;
//     PSLIST_ENTRY SListEntry;
//
// #ifdef _M_ARM64
//     /* ARM64: Always create large stacks because KeSwitchKernelStack is not
//      * implemented. Without this, GUI thread conversion leaves threads on
//      * the 12KB stack which overflows with deep win32k/user32/gdi32 calls. */
//     GuiStack = TRUE;
// #endif
//
//     //
//     // Calculate pages needed
//     //
//     if (GuiStack)
//     {
// #ifdef _M_ARM64
//         //
//         // ARM64: Commit the full large stack upfront. Kernel stack growth
//         // via guard pages is not reliable on ARM64, so avoid partial commit.
//         //
//         StackPtes = BYTES_TO_PAGES(MmLargeStackSize);
//         StackPages = StackPtes;
// #else
//         //
//         // We'll allocate 64KB stack, but only commit 12K
//         //
//         StackPtes = BYTES_TO_PAGES(MmLargeStackSize);
//         StackPages = BYTES_TO_PAGES(KERNEL_LARGE_STACK_COMMIT);
// #endif
//     }
//     else
//     {
//         //
//         // If the dead stack S-LIST has a stack on it, use it instead of allocating
//         // new system PTEs for this stack
//         //
//         if (ExQueryDepthSList(&MmDeadStackSListHead))
//         {
//             SListEntry = InterlockedPopEntrySList(&MmDeadStackSListHead);
//             if (SListEntry != NULL)
//             {
//                 BaseAddress = (SListEntry + 1);
//                 return BaseAddress;
//             }
//         }
//
//         //
//         // We'll allocate 12K and that's it
//         //
//         StackPtes = BYTES_TO_PAGES(KERNEL_STACK_SIZE);
//         StackPages = StackPtes;
//     }
//
//     //
//     // Reserve stack pages, plus a guard page
//     //
//     StackPte = MiReserveSystemPtes(StackPtes + 1, SystemPteSpace);
//     if (!StackPte) return NULL;
//
//     //
//     // Get the stack address
//     //
//     BaseAddress = MiPteToAddress(StackPte + StackPtes + 1);
//
//     //
//     // Select the right PTE address where we actually start committing pages
//     //
//     PointerPte = StackPte;
// #ifndef _M_ARM64
//     if (GuiStack) PointerPte += BYTES_TO_PAGES(MmLargeStackSize -
//                                                KERNEL_LARGE_STACK_COMMIT);
// #endif
//
//
//     /* Setup the temporary invalid PTE */
//     MI_MAKE_SOFTWARE_PTE(&InvalidPte, MM_NOACCESS);
//
//     /* Setup the template stack PTE */
//     MI_MAKE_HARDWARE_PTE_KERNEL(&TempPte, PointerPte + 1, MM_READWRITE, 0);
//
//     //
//     // Acquire the PFN DB lock
//     //
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // Loop each stack page
//     //
//     for (i = 0; i < StackPages; i++)
//     {
//         //
//         // Next PTE
//         //
//         PointerPte++;
//
//         /* Get a page and write the current invalid PTE */
//         MI_SET_USAGE(MI_USAGE_KERNEL_STACK);
//         super::helpers::MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//         PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
//         MI_WRITE_INVALID_PTE(PointerPte, InvalidPte);
//
//         /* Initialize the PFN entry for this page */
//         MiInitializePfn(PageFrameIndex, PointerPte, 1);
//
//         /* Write the valid PTE */
//         TempPte.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
//         MI_WRITE_VALID_PTE(PointerPte, TempPte);
//     }
//
//     //
//     // Release the PFN lock
//     //
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // Return the stack address
//     //
//     return BaseAddress;
// }
//
// NTSTATUS
// NTAPI
// MmGrowKernelStackEx(IN PVOID StackPointer,
//                     IN ULONG GrowSize)
// {
//     PKTHREAD Thread = KeGetCurrentThread();
//     PMMPTE LimitPte, NewLimitPte, LastPte;
//     KIRQL OldIrql;
//     MMPTE TempPte, InvalidPte;
//     PFN_NUMBER PageFrameIndex;
//
//     //
//     // Make sure the stack did not overflow
//     //
//     ASSERT(((ULONG_PTR)Thread->StackBase - (ULONG_PTR)Thread->StackLimit) <=
//            (MmLargeStackSize + PAGE_SIZE));
//
//     //
//     // Get the current stack limit
//     //
//     LimitPte = MiAddressToPte(Thread->StackLimit);
//     ASSERT(LimitPte->u.Hard.Valid == 1);
//
//     //
//     // Get the new one and make sure this isn't a retarded request
//     //
//     NewLimitPte = MiAddressToPte((PVOID)((ULONG_PTR)StackPointer - GrowSize));
//     if (NewLimitPte == LimitPte) return STATUS_SUCCESS;
//
//     //
//     // Now make sure you're not going past the reserved space
//     //
//     LastPte = MiAddressToPte((PVOID)((ULONG_PTR)Thread->StackBase -
//                                      MmLargeStackSize));
//     if (NewLimitPte < LastPte)
//     {
//         //
//         // Sorry!
//         //
//         return STATUS_STACK_OVERFLOW;
//     }
//
//     //
//     // Calculate the number of new pages
//     //
//     LimitPte--;
//
//     /* Setup the temporary invalid PTE */
//     MI_MAKE_SOFTWARE_PTE(&InvalidPte, MM_NOACCESS);
//
//     //
//     // Acquire the PFN DB lock
//     //
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // Loop each stack page
//     //
//     while (LimitPte >= NewLimitPte)
//     {
//         /* Get a page and write the current invalid PTE */
//         MI_SET_USAGE(MI_USAGE_KERNEL_STACK_EXPANSION);
//         super::helpers::MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//         PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
//         MI_WRITE_INVALID_PTE(LimitPte, InvalidPte);
//
//         /* Initialize the PFN entry for this page */
//         MiInitializePfn(PageFrameIndex, LimitPte, 1);
//
//         /* Setup the template stack PTE */
//         MI_MAKE_HARDWARE_PTE_KERNEL(&TempPte, LimitPte, MM_READWRITE, PageFrameIndex);
//
//         /* Write the valid PTE */
//         MI_WRITE_VALID_PTE(LimitPte--, TempPte);
//     }
//
//     //
//     // Release the PFN lock
//     //
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // Set the new limit
//     //
//     Thread->StackLimit = (ULONG_PTR)MiPteToAddress(NewLimitPte);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MmGrowKernelStack(IN PVOID StackPointer)
// {
//     //
//     // Call the extended version
//     //
//     return MmGrowKernelStackEx(StackPointer, KERNEL_LARGE_STACK_COMMIT);
// }
//
// NTSTATUS
// NTAPI
// MmSetMemoryPriorityProcess(IN PEPROCESS Process,
//                            IN UCHAR MemoryPriority)
// {
//     UCHAR OldPriority;
//
//     //
//     // Check if we have less then 16MB of Physical Memory
//     //
//     if ((MmSystemSize == MmSmallSystem) &&
//         (MmNumberOfPhysicalPages < ((15 * 1024 * 1024) / PAGE_SIZE)))
//     {
//         //
//         // Always use background priority
//         //
//         MemoryPriority = MEMORY_PRIORITY_BACKGROUND;
//     }
//
//     //
//     // Save the old priority and update it
//     //
//     OldPriority = (UCHAR)Process->Vm.Flags.MemoryPriority;
//     Process->Vm.Flags.MemoryPriority = MemoryPriority;
//
//     //
//     // Return the old priority
//     //
//     return OldPriority;
// }
//
// NTSTATUS
// NTAPI
// MmCreatePeb(IN PEPROCESS Process,
//             IN PINITIAL_PEB InitialPeb,
//             OUT PPEB *BasePeb)
// {
//     PPEB Peb = NULL;
//     LARGE_INTEGER SectionOffset;
//     SIZE_T ViewSize = 0;
//     PVOID TableBase = NULL;
//     PIMAGE_NT_HEADERS NtHeaders;
//     PIMAGE_LOAD_CONFIG_DIRECTORY ImageConfigData;
//     NTSTATUS Status;
//     USHORT Characteristics;
//     SectionOffset.QuadPart = (ULONGLONG)0;
//     *BasePeb = NULL;
//
//     //
//     // Attach to Process
//     //
//     ARM64_UART_TRACE("[MM] MmCreatePeb: KeAttachProcess\r\n");
//     KeAttachProcess(&Process->Pcb);
//
//     //
//     // Map NLS Tables
//     //
//     Status = MmMapViewOfSection(ExpNlsSectionPointer,
//                                 (PEPROCESS)Process,
//                                 &TableBase,
//                                 0,
//                                 0,
//                                 &SectionOffset,
//                                 &ViewSize,
//                                 ViewShare,
//                                 MEM_TOP_DOWN,
//                                 PAGE_READONLY);
//     DPRINT("NLS Tables at: %p\n", TableBase);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and exit */
//         KeDetachProcess();
//         return Status;
//     }
//
//     //
//     // Allocate the PEB
//     //
//     Status = MiCreatePebOrTeb(Process, sizeof(PEB), (PULONG_PTR)&Peb);
//     DPRINT("PEB at: %p\n", Peb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and exit */
//         KeDetachProcess();
//         return Status;
//     }
//
//     //
//     // Use SEH in case we can't load the PEB
//     //
//     _SEH2_TRY
//     {
//         //
//         // Initialize the PEB
//         //
//         RtlZeroMemory(Peb, sizeof(PEB));
//
//         //
//         // Set up data
//         //
//         Peb->ImageBaseAddress = Process->SectionBaseAddress;
//         Peb->InheritedAddressSpace = InitialPeb->InheritedAddressSpace;
//         Peb->Mutant = InitialPeb->Mutant;
//         Peb->ImageUsesLargePages = InitialPeb->ImageUsesLargePages;
//
//         //
//         // NLS
//         //
//         Peb->AnsiCodePageData = (PCHAR)TableBase + ExpAnsiCodePageDataOffset;
//         Peb->OemCodePageData = (PCHAR)TableBase + ExpOemCodePageDataOffset;
//         Peb->UnicodeCaseTableData = (PCHAR)TableBase + ExpUnicodeCaseTableDataOffset;
//
//         //
//         // Default Version Data (could get changed below)
//         //
//         Peb->OSMajorVersion = NtMajorVersion;
//         Peb->OSMinorVersion = NtMinorVersion;
//         Peb->OSBuildNumber = (USHORT)(NtBuildNumber & 0x3FFF);
//         Peb->OSPlatformId = VER_PLATFORM_WIN32_NT;
//         Peb->OSCSDVersion = (USHORT)CmNtCSDVersion;
//
//         //
//         // Heap and Debug Data
//         //
//         Peb->NumberOfProcessors = KeNumberProcessors;
//         Peb->ImageProcessAffinityMask = KeActiveProcessors;
//         Peb->BeingDebugged = (BOOLEAN)(Process->DebugPort != NULL);
//         Peb->NtGlobalFlag = NtGlobalFlag;
//         Peb->HeapSegmentReserve = MmHeapSegmentReserve;
//         Peb->HeapSegmentCommit = MmHeapSegmentCommit;
//         Peb->HeapDeCommitTotalFreeThreshold = MmHeapDeCommitTotalFreeThreshold;
//         Peb->HeapDeCommitFreeBlockThreshold = MmHeapDeCommitFreeBlockThreshold;
//         Peb->CriticalSectionTimeout = MmCriticalSectionTimeout;
//         Peb->MinimumStackCommit = MmMinimumStackCommitInBytes;
//         Peb->MaximumNumberOfHeaps = (PAGE_SIZE - sizeof(PEB)) / sizeof(PVOID);
//         Peb->ProcessHeaps = (PVOID*)(Peb + 1);
//
//         //
//         // Session ID
//         //
//         if (Process->Session) Peb->SessionId = MmGetSessionId(Process);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         //
//         // Fail
//         //
//         KeDetachProcess();
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     //
//     // Use SEH in case we can't load the image
//     //
//     _SEH2_TRY
//     {
//         //
//         // Get NT Headers
//         //
//         NtHeaders = RtlImageNtHeader(Peb->ImageBaseAddress);
//         Characteristics = NtHeaders->FileHeader.Characteristics;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         //
//         // Fail
//         //
//         KeDetachProcess();
//         _SEH2_YIELD(return STATUS_INVALID_IMAGE_PROTECT);
//     }
//     _SEH2_END;
//
//     //
//     // Parse the headers
//     //
//     if (NtHeaders)
//     {
//         //
//         // Use SEH in case we can't load the headers
//         //
//         _SEH2_TRY
//         {
//             //
//             // Get the Image Config Data too
//             //
//             ImageConfigData = RtlImageDirectoryEntryToData(Peb->ImageBaseAddress,
//                                                            TRUE,
//                                                            IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
//                                                            (PULONG)&ViewSize);
//             if (ImageConfigData)
//             {
//                 //
//                 // Probe it
//                 //
//                 ProbeForRead(ImageConfigData,
//                              sizeof(IMAGE_LOAD_CONFIG_DIRECTORY),
//                              sizeof(ULONG));
//             }
//
//             //
//             // Write subsystem data
//             //
//             Peb->ImageSubsystem = NtHeaders->OptionalHeader.Subsystem;
//             Peb->ImageSubsystemMajorVersion = NtHeaders->OptionalHeader.MajorSubsystemVersion;
//             Peb->ImageSubsystemMinorVersion = NtHeaders->OptionalHeader.MinorSubsystemVersion;
//
//             //
//             // Check for version data
//             //
//             if (NtHeaders->OptionalHeader.Win32VersionValue)
//             {
//                 //
//                 // Extract values and write them
//                 //
//                 Peb->OSMajorVersion = NtHeaders->OptionalHeader.Win32VersionValue & 0xFF;
//                 Peb->OSMinorVersion = (NtHeaders->OptionalHeader.Win32VersionValue >> 8) & 0xFF;
//                 Peb->OSBuildNumber = (NtHeaders->OptionalHeader.Win32VersionValue >> 16) & 0x3FFF;
//                 Peb->OSPlatformId = (NtHeaders->OptionalHeader.Win32VersionValue >> 30) ^ 2;
//
//                 /* Process CSD version override */
//                 if ((ImageConfigData) && (ImageConfigData->CSDVersion))
//                 {
//                     /* Take the value from the image configuration directory */
//                     Peb->OSCSDVersion = ImageConfigData->CSDVersion;
//                 }
//             }
//
//             /* Process optional process affinity mask override */
//             if ((ImageConfigData) && (ImageConfigData->ProcessAffinityMask))
//             {
//                 /* Take the value from the image configuration directory */
//                 Peb->ImageProcessAffinityMask = ImageConfigData->ProcessAffinityMask;
//             }
//
//             //
//             // Check if this is a UP image
//             if (Characteristics & IMAGE_FILE_UP_SYSTEM_ONLY)
//             {
//                 //
//                 // Set single processor rotating affinity.
//                 // See https://www.microsoftpressstore.com/articles/article.aspx?p=2233328&seqNum=3
//                 //
//                 Peb->ImageProcessAffinityMask = AFFINITY_MASK(MmRotatingUniprocessorNumber);
//                 ASSERT(Peb->ImageProcessAffinityMask & KeActiveProcessors);
//                 MmRotatingUniprocessorNumber = (MmRotatingUniprocessorNumber + 1) % KeNumberProcessors;
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             //
//             // Fail
//             //
//             KeDetachProcess();
//             _SEH2_YIELD(return STATUS_INVALID_IMAGE_PROTECT);
//         }
//         _SEH2_END;
//     }
//
//     //
//     // Detach from the Process
//     //
//     ARM64_UART_TRACE("[MM] MmCreatePeb: KeDetachProcess\r\n");
//     KeDetachProcess();
//     core::ptr::write_volatile(BasePeb, Peb);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MmCreateTeb(IN PEPROCESS Process,
//             IN PCLIENT_ID ClientId,
//             IN PINITIAL_TEB InitialTeb,
//             OUT PTEB *BaseTeb)
// {
//     PTEB Teb;
//     NTSTATUS Status = STATUS_SUCCESS;
//     *BaseTeb = NULL;
//
//     //
//     // Attach to Target
//     //
//     ARM64_UART_TRACE("[MM] MmCreateTeb: KeAttachProcess\r\n");
//     KeAttachProcess(&Process->Pcb);
//
//     //
//     // Allocate the TEB
//     //
//     Status = MiCreatePebOrTeb(Process, sizeof(TEB), (PULONG_PTR)&Teb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and exit */
//         KeDetachProcess();
//         return Status;
//     }
//
//     //
//     // Use SEH in case we can't load the TEB
//     //
//     _SEH2_TRY
//     {
//         //
//         // Initialize the PEB
//         //
//         RtlZeroMemory(Teb, sizeof(TEB));
//
//         //
//         // Set TIB Data
//         //
// #ifdef _M_AMD64
//         Teb->NtTib.ExceptionList = NULL;
// #else
//         Teb->NtTib.ExceptionList = EXCEPTION_CHAIN_END;
// #endif
//         Teb->NtTib.Self = (PNT_TIB)Teb;
//
//         //
//         // Identify this as an OS/2 V3.0 ("Cruiser") TIB
//         //
//         Teb->NtTib.Version = 30 << 8;
//
//         //
//         // Set TEB Data
//         //
//         Teb->ClientId = *ClientId;
//         Teb->RealClientId = *ClientId;
//         Teb->ProcessEnvironmentBlock = Process->Peb;
//         Teb->CurrentLocale = PsDefaultThreadLocaleId;
//
//         //
//         // Check if we have a grandparent TEB
//         //
//         if ((InitialTeb->PreviousStackBase == NULL) &&
//             (InitialTeb->PreviousStackLimit == NULL))
//         {
//             //
//             // Use initial TEB values
//             //
//             Teb->NtTib.StackBase = InitialTeb->StackBase;
//             Teb->NtTib.StackLimit = InitialTeb->StackLimit;
//             Teb->DeallocationStack = InitialTeb->AllocatedStackBase;
//         }
//         else
//         {
//             //
//             // Use grandparent TEB values
//             //
//             Teb->NtTib.StackBase = InitialTeb->PreviousStackBase;
//             Teb->NtTib.StackLimit = InitialTeb->PreviousStackLimit;
//         }
//
//         //
//         // Initialize the static unicode string
//         //
//         Teb->StaticUnicodeString.MaximumLength = sizeof(Teb->StaticUnicodeBuffer);
//         Teb->StaticUnicodeString.Buffer = Teb->StaticUnicodeBuffer;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         //
//         // Get error code
//         //
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     //
//     // Return
//     //
//     ARM64_UART_TRACE("[MM] MmCreateTeb: KeDetachProcess\r\n");
//     KeDetachProcess();
//     core::ptr::write_volatile(BaseTeb, Teb);
//     return Status;
// }
//
// #ifdef _M_AMD64
// static
// NTSTATUS
// MiInsertSharedUserPageVad(
//     _In_ PEPROCESS Process)
// {
//     PMMVAD_LONG Vad;
//     ULONG_PTR BaseAddress;
//     NTSTATUS Status;
//
//     if (Process->QuotaBlock != NULL)
//     {
//         Status = PsChargeProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Ran out of quota.\n");
//             return Status;
//         }
//     }
//
//     /* Allocate a VAD */
//     Vad = ExAllocatePoolWithTag(NonPagedPool, sizeof(MMVAD_LONG), 'ldaV');
//     if (Vad == NULL)
//     {
//         DPRINT1("Failed to allocate VAD for shared user page\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto FailPath;
//     }
//
//     /* Setup the primary flags with the size, and make it private, RO */
//     Vad->u.LongFlags = 0;
//     Vad->u.VadFlags.CommitCharge = 0;
//     Vad->u.VadFlags.NoChange = TRUE;
//     Vad->u.VadFlags.VadType() = VadNone;
//     Vad->u.VadFlags.MemCommit() = FALSE;
//     Vad->u.VadFlags.Protection() = MM_READONLY;
//     Vad->u.VadFlags.PrivateMemory() = TRUE;
//     Vad->u1.Parent = NULL;
//
//     /* Setup the secondary flags to make it a secured, readonly, long VAD */
//     Vad->u2.LongFlags2 = 0;
//     Vad->u2.VadFlags2.OneSecured = TRUE;
//     Vad->u2.VadFlags2.LongVad = TRUE;
//     Vad->u2.VadFlags2.ReadOnly = FALSE;
//
//     Vad->ControlArea = NULL; // For Memory-Area hack
//     Vad->FirstPrototypePte = NULL;
//
//     /* Insert it into the process VAD table */
//     BaseAddress = MM_SHARED_USER_DATA_VA;
//     Status = MiInsertVadEx((PMMVAD)Vad,
//                             &BaseAddress,
//                             PAGE_SIZE,
//                             (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS,
//                             PAGE_SIZE,
//                             MEM_TOP_DOWN);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to insert shared user VAD\n");
//         ExFreePoolWithTag(Vad, 'ldaV');
//         goto FailPath;
//     }
//
//     /* Success */
//     return STATUS_SUCCESS;
//
// FailPath:
//     if (Process->QuotaBlock != NULL)
//         PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//
//     return Status;
// }
// #endif
//
// NTSTATUS
// NTAPI
// MmInitializeProcessAddressSpace(IN PEPROCESS Process,
//                                 IN PEPROCESS ProcessClone OPTIONAL,
//                                 IN PVOID Section OPTIONAL,
//                                 IN OUT PULONG Flags,
//                                 IN POBJECT_NAME_INFORMATION *AuditName OPTIONAL)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     SIZE_T ViewSize = 0;
//     PVOID ImageBase = 0;
//     PMMPTE PointerPte;
//     KIRQL OldIrql;
//     PMMPDE PointerPde;
//     PMMPFN Pfn;
//     PFN_NUMBER PageFrameNumber;
//     UNICODE_STRING FileName;
//     PWCHAR Source;
//     PCHAR Destination;
//     USHORT Length = 0;
//
// #if (_MI_PAGING_LEVELS >= 3)
//     PMMPPE PointerPpe;
// #endif
// #if (_MI_PAGING_LEVELS == 4)
//     PMMPXE PointerPxe;
// #endif
//
//     /* We should have a PDE */
//     ASSERT(Process->Pcb.DirectoryTableBase[0] != 0);
//     ASSERT(Process->PdeUpdateNeeded == FALSE);
//
//     ARM64_UART_TRACE("[MM] InitProcAS: KeAttachProcess\r\n");
//     /* Attach to the process */
//     KeAttachProcess(&Process->Pcb);
//     ARM64_UART_TRACE("[MM] InitProcAS: attached\r\n");
//
//     /* The address space should now been in phase 1 or 0 */
//     ASSERT(Process->AddressSpaceInitialized <= 1);
//     Process->AddressSpaceInitialized = 2;
//
//     /* Initialize the Addresss Space lock */
//     KeInitializeGuardedMutex(&Process->AddressCreationLock);
//     Process->Vm.WorkingSetExpansionLinks.Flink = NULL;
//
//     /* Initialize AVL tree */
//     ASSERT(Process->VadRoot.NumberGenericTableElements() == 0);
//     Process->VadRoot.BalancedRoot.u1.Parent = &Process->VadRoot.BalancedRoot;
//
//     ARM64_UART_TRACE("[MM] InitProcAS: MiLockProcessWorkingSet\r\n");
//     /* Lock our working set */
//     MiLockProcessWorkingSet(Process, PsGetCurrentThread());
//     ARM64_UART_TRACE("[MM] InitProcAS: locked WS\r\n");
//
//     /* Lock PFN database */
//     ARM64_UART_TRACE("[MM] InitProcAS: AcquirePfnLock\r\n");
//     OldIrql = MiAcquirePfnLock();
//     ARM64_UART_TRACE("[MM] InitProcAS: PfnLock acquired\r\n");
//
//     /* Setup the PFN for the PDE base of this process */
// #if (_MI_PAGING_LEVELS == 4)
//     PointerPte = MiAddressToPte(PXE_BASE);
// #elif (_MI_PAGING_LEVELS == 3)
//     PointerPte = MiAddressToPte(PPE_BASE);
// #else
//     PointerPte = MiAddressToPte(PDE_BASE);
// #endif
//     ARM64_UART_TRACE("[MM] InitProcAS: PFN_FROM_PTE(PXE_BASE)\r\n");
//     PageFrameNumber = PFN_FROM_PTE(PointerPte);
//     ARM64_UART_TRACE_HEX("[MM] InitProcAS: PML4 PFN=", PageFrameNumber);
//     ASSERT(Process->Pcb.DirectoryTableBase[0] == PageFrameNumber * PAGE_SIZE);
//     ARM64_UART_TRACE("[MM] InitProcAS: MiInitializePfn(PML4)\r\n");
//     MiInitializePfn(PageFrameNumber, PointerPte, TRUE);
//
//     /* Do the same for hyperspace */
//     ARM64_UART_TRACE("[MM] InitProcAS: hyperspace PDE\r\n");
//     PointerPde = MiAddressToPde(HYPER_SPACE);
//     PageFrameNumber = PFN_FROM_PTE(PointerPde);
//     MiInitializePfn(PageFrameNumber, (PMMPTE)PointerPde, TRUE);
// #if (_MI_PAGING_LEVELS == 2)
//     ASSERT(Process->Pcb.DirectoryTableBase[1] == PageFrameNumber * PAGE_SIZE);
// #endif
//
// #if (_MI_PAGING_LEVELS >= 3)
//     ARM64_UART_TRACE("[MM] InitProcAS: hyperspace PPE\r\n");
//     PointerPpe = MiAddressToPpe((PVOID)HYPER_SPACE);
//     PageFrameNumber = PFN_FROM_PTE(PointerPpe);
//     MiInitializePfn(PageFrameNumber, PointerPpe, TRUE);
// #if (_MI_PAGING_LEVELS == 3)
//     ASSERT(Process->Pcb.DirectoryTableBase[1] == PageFrameNumber * PAGE_SIZE);
// #endif
// #endif
// #if (_MI_PAGING_LEVELS == 4)
//     ARM64_UART_TRACE("[MM] InitProcAS: hyperspace PXE\r\n");
//     PointerPxe = MiAddressToPxe((PVOID)HYPER_SPACE);
//     PageFrameNumber = PFN_FROM_PTE(PointerPxe);
//     MiInitializePfn(PageFrameNumber, PointerPxe, TRUE);
//     ASSERT(Process->Pcb.DirectoryTableBase[1] == PageFrameNumber * PAGE_SIZE);
// #endif
//
//     /* Do the same for the Working set list */
//     ARM64_UART_TRACE("[MM] InitProcAS: WS list PTE\r\n");
//     PointerPte = MiAddressToPte(MmWorkingSetList);
//     PageFrameNumber = PFN_FROM_PTE(PointerPte);
//     MiInitializePfn(PageFrameNumber, PointerPte, TRUE);
//
//     /* All our pages are now active & valid. Release the lock. */
//     MiReleasePfnLock(OldIrql);
//     ARM64_UART_TRACE("[MM] InitProcAS: PfnLock released\r\n");
//
//     /* This should be in hyper space, but not in the mapping range */
//     Process->Vm.VmWorkingSetList = MmWorkingSetList;
//     ASSERT(((ULONG_PTR)MmWorkingSetList >= MI_MAPPING_RANGE_END) && ((ULONG_PTR)MmWorkingSetList <= HYPER_SPACE_END));
//
//     /* Now initialize the working set list */
//     ARM64_UART_TRACE("[MM] InitProcAS: MiInitializeWorkingSetList\r\n");
//     MiInitializeWorkingSetList(&Process->Vm);
//     ARM64_UART_TRACE("[MM] InitProcAS: WS init done\r\n");
//
//     /* The rule is that the owner process is always in the FLINK of the PDE's PFN entry */
//     ARM64_UART_TRACE_HEX("[MM] InitProcAS: DTB[0]=", Process->Pcb.DirectoryTableBase[0]);
//     Pfn = MiGetPfnEntry(Process->Pcb.DirectoryTableBase[0] >> PAGE_SHIFT);
//     ARM64_UART_TRACE_HEX("[MM] InitProcAS: Pfn=", (ULONG_PTR)Pfn);
//     ASSERT(Pfn->u4.PteFrame == MiGetPfnEntryIndex(Pfn));
//     ARM64_UART_TRACE("[MM] InitProcAS: Pfn asserts OK\r\n");
//     ASSERT(Pfn->u1.WsIndex == 0);
//     Pfn->u1.Event = (PKEVENT)Process;
//     ARM64_UART_TRACE("[MM] InitProcAS: Pfn->u1.Event set\r\n");
//
//     /* Sanity check */
//     ASSERT(Process->PhysicalVadRoot == NULL);
//     ARM64_UART_TRACE("[MM] InitProcAS: unlocking WS\r\n");
//
//     /* Release the process working set */
//     MiUnlockProcessWorkingSet(Process, PsGetCurrentThread());
//     ARM64_UART_TRACE("[MM] InitProcAS: WS unlocked\r\n");
//
// #ifdef _M_AMD64
//     /* On x64 we need a VAD for the shared user page */
//     Status = MiInsertSharedUserPageVad(Process);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MiCreateSharedUserPageVad() failed: 0x%lx\n", Status);
//         return Status;
//     }
// #endif
//
//     ARM64_UART_TRACE("[MM] InitProcAS: checking Section\r\n");
//     /* Check if there's a Section Object */
//     if (Section)
//     {
//         ARM64_UART_TRACE("[MM] InitProcAS: have Section\r\n");
//         /* Determine the image file name and save it to EPROCESS */
//         PFILE_OBJECT FileObject = MmGetFileObjectForSection(Section);
//         ARM64_UART_TRACE_HEX("[MM] InitProcAS: FileObject=", (ULONG_PTR)FileObject);
//         ARM64_UART_TRACE("[MM] InitProcAS: reading FileName\r\n");
//         FileName = FileObject->FileName;
//         ARM64_UART_TRACE_HEX("[MM] InitProcAS: FileName.Buf=", (ULONG_PTR)FileName.Buffer);
//         ARM64_UART_TRACE_HEX("[MM] InitProcAS: FileName.Len=", FileName.Length);
//         Source = (PWCHAR)((PCHAR)FileName.Buffer + FileName.Length);
//         if (FileName.Buffer)
//         {
//             /* Loop the file name*/
//             while (Source > FileName.Buffer)
//             {
//                 /* Make sure this isn't a backslash */
//                 if (*--Source == OBJ_NAME_PATH_SEPARATOR)
//                 {
//                     /* If so, stop it here */
//                     Source++;
//                     break;
//                 }
//                 else
//                 {
//                     /* Otherwise, keep going */
//                     Length++;
//                 }
//             }
//         }
//
//         /* Copy the to the process and truncate it to 15 characters if necessary */
//         Destination = Process->ImageFileName;
//         Length = min(Length, sizeof(Process->ImageFileName) - 1);
//         while (Length--) *Destination++ = (UCHAR)*Source++;
//         *Destination = ANSI_NULL;
//
//         /* Check if caller wants an audit name */
//         if (AuditName)
//         {
//             /* Setup the audit name */
//             Status = SeInitializeProcessAuditName(FileObject, FALSE, AuditName);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Fail */
//                 KeDetachProcess();
//                 return Status;
//             }
//         }
//
//         ARM64_UART_TRACE("[MM] InitProcAS: MmMapViewOfSection\r\n");
//         /* Map the section */
//         Status = MmMapViewOfSection(Section,
//                                     Process,
//                                     (PVOID*)&ImageBase,
//                                     0,
//                                     0,
//                                     NULL,
//                                     &ViewSize,
//                                     ViewUnmap,
//                                     MEM_COMMIT,
//                                     PAGE_READWRITE);
//
//         /* Save the pointer */
//         Process->SectionBaseAddress = ImageBase;
//     }
//
//     /* Be nice and detach */
//     KeDetachProcess();
//
//     /* Return status to caller */
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MmInitializeHandBuiltProcess(IN PEPROCESS Process,
//                              IN PULONG_PTR DirectoryTableBase)
// {
//     /* Share the directory base with the idle process */
//     DirectoryTableBase[0] = PsGetCurrentProcess()->Pcb.DirectoryTableBase[0];
//     DirectoryTableBase[1] = PsGetCurrentProcess()->Pcb.DirectoryTableBase[1];
//
//     /* Initialize the Addresss Space */
//     KeInitializeGuardedMutex(&Process->AddressCreationLock);
//     KeInitializeSpinLock(&Process->HyperSpaceLock);
//     Process->Vm.WorkingSetExpansionLinks.Flink = NULL;
//     ASSERT(Process->VadRoot.NumberGenericTableElements() == 0);
//     Process->VadRoot.BalancedRoot.u1.Parent = &Process->VadRoot.BalancedRoot;
//
//     /* Use idle process Working set */
//     Process->Vm.VmWorkingSetList = PsGetCurrentProcess()->Vm.VmWorkingSetList;
//     Process->WorkingSetPage = PsGetCurrentProcess()->WorkingSetPage;
//
//     /* Done */
//     Process->HasAddressSpace = TRUE;//??
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MmInitializeHandBuiltProcess2(IN PEPROCESS Process)
// {
//     /* Lock the VAD, ARM3-owned ranges away */
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// MmCreateProcessAddressSpace(IN ULONG MinWs,
//                             IN PEPROCESS Process,
//                             OUT PULONG_PTR DirectoryTableBase)
// {
//     KIRQL OldIrql;
//     PFN_NUMBER TableBaseIndex, HyperIndex, WsListIndex;
//     ULONG Color;
//
//     /* Make sure we don't already have a page directory setup */
//     ASSERT(Process->Pcb.DirectoryTableBase[0] == 0);
//     ASSERT(Process->Pcb.DirectoryTableBase[1] == 0);
//     ASSERT(Process->WorkingSetPage == 0);
//
//     /* Choose a process color */
//     Process->NextPageColor = (USHORT)RtlRandom(&MmProcessColorSeed);
//
//     /* Setup the hyperspace lock */
//     KeInitializeSpinLock(&Process->HyperSpaceLock);
//
//     /* Lock PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /*
//      * Get a page for the table base, one for hyper space & one for the working set list.
//      * The PFNs for these pages will be initialized in MmInitializeProcessAddressSpace,
//      * when we are already attached to the process.
//      * The other pages (if any) are allocated in the arch-specific part.
//      */
//     Color = MI_GET_NEXT_PROCESS_COLOR(Process);
//     MI_SET_USAGE(MI_USAGE_PAGE_DIRECTORY);
//     TableBaseIndex = MiRemoveZeroPageSafe(Color);
//     if (!TableBaseIndex)
//     {
//         /* No zero pages, grab a free one */
//         TableBaseIndex = MiRemoveAnyPage(Color);
//
//         /* Zero it outside the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         MiZeroPhysicalPage(TableBaseIndex);
//         OldIrql = MiAcquirePfnLock();
//     }
//     MI_SET_USAGE(MI_USAGE_PAGE_DIRECTORY);
//     Color = MI_GET_NEXT_PROCESS_COLOR(Process);
//     HyperIndex = MiRemoveZeroPageSafe(Color);
//     if (!HyperIndex)
//     {
//         /* No zero pages, grab a free one */
//         HyperIndex = MiRemoveAnyPage(Color);
//
//         /* Zero it outside the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         MiZeroPhysicalPage(HyperIndex);
//         OldIrql = MiAcquirePfnLock();
//     }
//     MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//     Color = MI_GET_NEXT_PROCESS_COLOR(Process);
//     WsListIndex = MiRemoveZeroPageSafe(Color);
//     if (!WsListIndex)
//     {
//         /* No zero pages, grab a free one */
//         WsListIndex = MiRemoveAnyPage(Color);
//
//         /* Zero it outside the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         MiZeroPhysicalPage(WsListIndex);
//     }
//     else
//     {
//         /* Release the PFN lock */
//         MiReleasePfnLock(OldIrql);
//     }
//
//     /* Set the base directory pointers */
//     Process->WorkingSetPage = WsListIndex;
//     DirectoryTableBase[0] = TableBaseIndex << PAGE_SHIFT;
//     DirectoryTableBase[1] = HyperIndex << PAGE_SHIFT;
//
//     /* Perform the arch-specific parts */
//     if (!MiArchCreateProcessAddressSpace(Process, DirectoryTableBase))
//     {
//         OldIrql = MiAcquirePfnLock();
//         MiInsertPageInFreeList(WsListIndex);
//         MiInsertPageInFreeList(HyperIndex);
//         MiInsertPageInFreeList(TableBaseIndex);
//         MiReleasePfnLock(OldIrql);
//         Process->WorkingSetPage = 0;
//         DirectoryTableBase[0] = 0;
//         DirectoryTableBase[1] = 0;
//         return FALSE;
//     }
//
//     /* Switch to phase 1 initialization */
//     ASSERT(Process->AddressSpaceInitialized == 0);
//     Process->AddressSpaceInitialized = 1;
//
//     /* Add the process to the session */
//     MiSessionAddProcess(Process);
//     return TRUE;
// }
//
// VOID
// NTAPI
// MmCleanProcessAddressSpace(IN PEPROCESS Process)
// {
//     PMMVAD Vad;
//     PMM_AVL_TABLE VadTree;
//     PETHREAD Thread = PsGetCurrentThread();
//
//     /* Remove from the session */
//     MiSessionRemoveProcess();
//
//     /* Abort early, when the address space wasn't fully initialized */
//     if (Process->AddressSpaceInitialized < 2)
//     {
//         DPRINT1("Incomplete address space for Process %p. Might leak resources.\n",
//                 Process);
//         return;
//     }
//
//     /* Lock the process address space from changes */
//     MmLockAddressSpace(&Process->Vm);
//     MiLockProcessWorkingSetUnsafe(Process, Thread);
//
//     /* VM is deleted now */
//     Process->VmDeleted = TRUE;
//     MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//
//     /* Enumerate the VADs */
//     VadTree = &Process->VadRoot;
//     while (VadTree->NumberGenericTableElements)
//     {
//         /* Grab the current VAD */
//         Vad = (PMMVAD)VadTree->BalancedRoot.RightChild;
//
//         /* Check for old-style memory areas */
//         if (MI_IS_MEMORY_AREA_VAD(Vad))
//         {
//             /* We do not expect ARM3 memory areas here, those are kernel only */
//             ASSERT(MI_IS_ROSMM_VAD(Vad));
//
//             /* Let RosMm handle this */
//             MiRosCleanupMemoryArea(Process, Vad);
//             continue;
//         }
//
//         /* Lock the working set */
//         MiLockProcessWorkingSetUnsafe(Process, Thread);
//
//         /* Remove this VAD from the tree */
//         ASSERT(VadTree->NumberGenericTableElements >= 1);
//         MiRemoveNode((PMMADDRESS_NODE)Vad, VadTree);
//
//         /* Only regular VADs supported for now */
//         ASSERT(Vad->u.VadFlags.VadType() == VadNone);
//
//         /* Check if this is a section VAD */
//         if (!(Vad->u.VadFlags.PrivateMemory()) && (Vad->ControlArea))
//         {
//             /* Remove the view */
//             MiRemoveMappedView(Process, Vad);
//         }
//         else
//         {
//             /* Delete the addresses */
//             MiDeleteVirtualAddresses(Vad->StartingVpn << PAGE_SHIFT,
//                                      (Vad->EndingVpn << PAGE_SHIFT) | (PAGE_SIZE - 1),
//                                      Vad);
//
//             /* Release the working set */
//             MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//         }
//
//         /* Free the VAD memory */
//         ExFreePool(Vad);
//
//         /* Return the quota the VAD used */
//         PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     }
//
//     /* Lock the working set */
//     MiLockProcessWorkingSetUnsafe(Process, Thread);
//     ASSERT(Process->CloneRoot == NULL);
//     ASSERT(Process->PhysicalVadRoot == NULL);
//
//     /* Delete the shared user data section */
//     MiDeleteVirtualAddresses(USER_SHARED_DATA, USER_SHARED_DATA, NULL);
//
//     /* Release the working set */
//     MiUnlockProcessWorkingSetUnsafe(Process, Thread);
//
//     /* Release the address space */
//     MmUnlockAddressSpace(&Process->Vm);
// }
//
// VOID
// NTAPI
// MmDeleteProcessAddressSpace(IN PEPROCESS Process)
// {
//     PMMPFN Pfn1, Pfn2;
//     KIRQL OldIrql;
//     PFN_NUMBER PageFrameIndex;
//
// #if (_MI_PAGING_LEVELS < 3)
//     OldIrql = MiAcquireExpansionLock();
//     RemoveEntryList(&Process->MmProcessLinks);
//     MiReleaseExpansionLock(OldIrql);
// #endif
//
//     //ASSERT(Process->CommitCharge == 0);
//
//     /* Remove us from the list */
//     OldIrql = MiAcquireExpansionLock();
//     if (Process->Vm.WorkingSetExpansionLinks.Flink != NULL)
//         RemoveEntryList(&Process->Vm.WorkingSetExpansionLinks);
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Acquire the PFN lock */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Check for fully initialized process */
//     if (Process->AddressSpaceInitialized == 2)
//     {
//         /* Map the working set page and its page table */
//         Pfn1 = MiGetPfnEntry(Process->WorkingSetPage);
//         Pfn2 = MiGetPfnEntry(Pfn1->u4.PteFrame);
//
//         /* Nuke it */
//         MI_SET_PFN_DELETED(Pfn1);
//         MiDecrementShareCount(Pfn2, Pfn1->u4.PteFrame);
//         MiDecrementShareCount(Pfn1, Process->WorkingSetPage);
//         ASSERT((Pfn1->u3.e2.ReferenceCount == 0) || (Pfn1->u3.e1.WriteInProgress));
//
//         /* Now map hyperspace and its page table */
//         PageFrameIndex = Process->Pcb.DirectoryTableBase[1] >> PAGE_SHIFT;
//         Pfn1 = MiGetPfnEntry(PageFrameIndex);
//         Pfn2 = MiGetPfnEntry(Pfn1->u4.PteFrame);
//
//         /* Nuke it */
//         MI_SET_PFN_DELETED(Pfn1);
//         MiDecrementShareCount(Pfn2, Pfn1->u4.PteFrame);
//         MiDecrementShareCount(Pfn1, PageFrameIndex);
//         ASSERT((Pfn1->u3.e2.ReferenceCount == 0) || (Pfn1->u3.e1.WriteInProgress));
//
//         /* Finally, nuke the PDE itself */
//         PageFrameIndex = Process->Pcb.DirectoryTableBase[0] >> PAGE_SHIFT;
//         Pfn1 = MiGetPfnEntry(PageFrameIndex);
//         MI_SET_PFN_DELETED(Pfn1);
//         MiDecrementShareCount(Pfn1, PageFrameIndex);
//         MiDecrementShareCount(Pfn1, PageFrameIndex);
//
//         /* Page table is now dead. Bye bye... */
//         ASSERT((Pfn1->u3.e2.ReferenceCount == 0) || (Pfn1->u3.e1.WriteInProgress));
//     }
//     else
//     {
//         DPRINT1("Deleting partially initialized address space of Process %p. Might leak resources.\n",
//                 Process);
//     }
//
//     /* Release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Drop a reference on the session */
//     if (Process->Session) MiReleaseProcessReferenceToSessionDataPage(Process->Session);
//
//     /* Clear out the PDE pages */
//     Process->Pcb.DirectoryTableBase[0] = 0;
//     Process->Pcb.DirectoryTableBase[1] = 0;
// }
//
//
// /* SYSTEM CALLS ***************************************************************/
//
// NTSTATUS
// NTAPI
// NtAllocateUserPhysicalPages(IN HANDLE ProcessHandle,
//                             IN OUT PULONG_PTR NumberOfPages,
//                             IN OUT PULONG_PTR UserPfnArray)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtMapUserPhysicalPages(IN PVOID VirtualAddresses,
//                        IN ULONG_PTR NumberOfPages,
//                        IN OUT PULONG_PTR UserPfnArray)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtMapUserPhysicalPagesScatter(IN PVOID *VirtualAddresses,
//                               IN ULONG_PTR NumberOfPages,
//                               IN OUT PULONG_PTR UserPfnArray)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtFreeUserPhysicalPages(IN HANDLE ProcessHandle,
//                         IN OUT PULONG_PTR NumberOfPages,
//                         IN OUT PULONG_PTR UserPfnArray)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
