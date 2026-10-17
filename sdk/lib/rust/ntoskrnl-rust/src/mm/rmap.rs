use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    PEPROCESS,
    PMEMORY_AREA,
    PMMSUPPORT,
    PMM_RMAP_ENTRY,
    PMM_SECTION_SEGMENT,
    LARGE_INTEGER,
    NPAGED_LOOKASIDE_LIST,
    SWAPENTRY,
    ExAcquireRundownProtection,
    ExInitializeNPagedLookasideList,
    ExReleaseRundownProtection,
    KeAttachProcess,
    KeBugCheck,
    KeDetachProcess,
    MmAllocSwapPage,
    MmCheckDirtySegment,
    MmCreatePageFileMapping,
    MmCreateVirtualMapping,
    MmDeletePageFileMapping,
    MmDereferenceSegment,
    MmFindRegion,
    MmFreeSwapPage,
    MmGetPfnForProcess,
    MmGetRmapListHeadPage,
    MmGetSavedSwapEntryPage,
    MmGetSectionAssociation,
    MmLocateMemoryAreaByAddress,
    MmReleasePageMemoryConsumer,
    MmSetRmapListHeadPage,
    MmSetSavedSwapEntryPage,
    MmUnsharePageEntrySectionSegment,
    MmWriteToSwapPage,
    ObReferenceObjectByPointer,
};

// FORCEINLINE shims — must be extern "C" (cdecl) to match C shim calling convention on i386.
extern "C" {
    fn MmLockAddressSpace(AddressSpace: PMMSUPPORT);
    fn MmUnlockAddressSpace(AddressSpace: PMMSUPPORT);
}

extern "system" {
    fn MmLockSectionSegment(Segment: PMM_SECTION_SEGMENT);
    fn MmUnlockSectionSegment(Segment: PMM_SECTION_SEGMENT);
    fn MmGetPageEntrySectionSegment(Segment: PMM_SECTION_SEGMENT, Offset: *mut LARGE_INTEGER) -> ULONG_PTR;
    fn ObDereferenceObject(Object: PVOID);
    // MmDeleteVirtualMapping is NTAPI/stdcall on i386 (@16), NOT the cdecl that bindgen's ddk
    // decl implies. Declare it locally as "system" (matching the def + marea.rs/section.rs) —
    // importing the ddk "C" version creates a same-crate CC disagreement that folds this crate's
    // MmPageOutPhysicalAddress to a ud2 (§9.99) AND mismatches the stdcall def at the call below.
    fn MmDeleteVirtualMapping(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID, WasDirty: *mut BOOLEAN, Page: *mut ntkrnl_sys::ddk::PFN_NUMBER) -> BOOLEAN;
}
extern "C" {
    fn ExAllocateFromNPagedLookasideList(Lookaside: *mut NPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToNPagedLookasideList(Lookaside: *mut NPAGED_LOOKASIDE_LIST, Entry: PVOID);
    fn MmSetDirtyPage(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID);
    fn RMAP_IS_SEGMENT(Address: PVOID) -> bool;
    fn PFN_FROM_SSE(Entry: ULONG_PTR) -> PFN_NUMBER;
    fn MM_IS_WAIT_PTE(Entry: ULONG_PTR) -> bool;
    fn PAGE_ROUND_DOWN(addr: PVOID) -> ULONG_PTR;
    fn InterlockedExchangeAddUL(addend: *mut ULONG, value: ULONG) -> ULONG;
}
extern "C" {
    fn MA_GetStartingAddress(MemoryArea: PMEMORY_AREA) -> ULONG_PTR;
}

// MM_WAIT_ENTRY (ntoskrnl/include/internal/mm.h) — pointer-width dependent.
// 0xFFFFFFFE was a placeholder that disagreed with ntkrnl_sys::mm::MM_WAIT_ENTRY
// (the value section.rs compares against) → page-file wait sentinel mismatch.
#[cfg(target_pointer_width = "64")]
static MM_WAIT_ENTRY: ULONG_PTR = 0x7FFFFFFFFFFFFC00;
#[cfg(target_pointer_width = "32")]
static MM_WAIT_ENTRY: ULONG_PTR = 0x7ffffc00;
// `~((ULONG_PTR)0xff)` — pointer-width, not the 32-bit-truncated 0xFFFFFF00 that
// used to sit here. The one use below (`& !RMAP_SEGMENT_MASK`) survived the
// truncation only because its result is cast to ULONG.
static RMAP_SEGMENT_MASK: ULONG_PTR = !0xffusize;
static MEMORY_AREA_SECTION_VIEW: ULONG = 1;
static MM_SEGMENT_INDELETE: ULONG = 4; // mm.h:242 — was 1 (PHYSICALMEMORY's bit)
static MC_USER: ULONG = 0;

static mut RmapLookasideList: NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

unsafe extern "system" fn RmapListFree(P: PVOID) {
    ExFreePoolWithTag(P, TAG_RMAP);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitializeRmapList() {
    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(RmapLookasideList),
        None,
        core::mem::transmute(RmapListFree as unsafe extern "system" fn(PVOID)),
        0,
        core::mem::size_of::<ntkrnl_sys::ddk::MM_RMAP_ENTRY>() as _,
        TAG_RMAP,
        50,
    );
}

/// Pages the no-mapping (segment-only) path released; said every 1024th.
static SEGMENT_RECLAIMS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

#[no_mangle]
pub unsafe extern "system" fn MmPageOutPhysicalAddress(Page: PFN_NUMBER) -> NTSTATUS {
    let mut entry: PMM_RMAP_ENTRY;
    let mut MemoryArea: PMEMORY_AREA;
    let mut AddressSpace: PMMSUPPORT;
    let mut Address: PVOID = core::ptr::null_mut();
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Segment: PMM_SECTION_SEGMENT = core::ptr::null_mut();
    let mut SegmentOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut OldIrql: KIRQL;

    'GetEntry: loop {
        OldIrql = MiAcquirePfnLock();

        entry = MmGetRmapListHeadPage(Page as _);

        while !entry.is_null() && RMAP_IS_SEGMENT((*entry).Address) {
            entry = (*entry).Next;
        }

        if entry.is_null() {
            // No process maps the page: it is a section segment's cached copy, which the
            // segment releases below if it is clean. arm64 returned UNSUCCESSFUL here from
            // its bring-up (411417cc538, "cannot page out"), so it could never reclaim an
            // image or data-file page once the last view went: a 1024M appstorm64 sweep
            // starved at 7 MB free with 55 MB of such pages held (MEMCENSUS `ros`).
            MiReleasePfnLock(OldIrql);
            break 'GetEntry;
        }

        Process = (*entry).Process;
        Address = (*entry).Address;

        if ((Address as ULONG_PTR) & 0xFFF) != 0 {
            // An rmap entry's Address must be page-aligned. Unaligned means the
            // entry — or the list it came off — is already corrupt, which is the
            // §9.97 signature read from the rmap side.
            mm_guard_trip_atomic!(
                "rmap.MmPageOutPhysicalAddress unaligned-rmap-Address", 8,
                "Page" => Page, "Address" => Address as ULONG_PTR,
                "Process" => Process as ULONG_PTR, "entry" => entry as ULONG_PTR,
            );
            #[cfg(target_arch = "aarch64")]
            {
                MiReleasePfnLock(OldIrql);
                return STATUS_UNSUCCESSFUL;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }

        if ExAcquireRundownProtection(core::ptr::addr_of_mut!(
            (*(Process as *mut _EPROCESS)).RundownProtect
        )) == 0 {
            MiReleasePfnLock(OldIrql);
            return STATUS_PROCESS_IS_TERMINATING;
        }

        Status = ObReferenceObjectByPointer(Process as PVOID, PROCESS_ALL_ACCESS, core::ptr::null_mut(), 0);
        MiReleasePfnLock(OldIrql);
        if Status < 0 {
            ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                (*(Process as *mut _EPROCESS)).RundownProtect
            ));
            return Status;
        }
        AddressSpace = core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm);

        MmLockAddressSpace(AddressSpace);

        MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, Address);
        if MemoryArea.is_null() || (*MemoryArea).DeleteInProgress != 0 {
            MmUnlockAddressSpace(AddressSpace);
            ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                (*(Process as *mut _EPROCESS)).RundownProtect
            ));
            ObDereferenceObject(Process as PVOID);
            continue 'GetEntry;
        }

        if Process != PsInitialSystemProcess as PEPROCESS {
            KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb));
        }

        if MmGetPfnForProcess(Process, Address) != Page as _ {
            if Process != PsInitialSystemProcess as PEPROCESS {
                KeDetachProcess();
            }
            MmUnlockAddressSpace(AddressSpace);
            ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                (*(Process as *mut _EPROCESS)).RundownProtect
            ));
            ObDereferenceObject(Process as PVOID);
            continue 'GetEntry;
        }

        if (*MemoryArea).Type == MEMORY_AREA_SECTION_VIEW {
            let mut Entry: ULONG_PTR;
            let mut Dirty: BOOLEAN = 0;
            let mut MapPage: ntkrnl_sys::ddk::PFN_NUMBER = 0;
            let mut Offset: LARGE_INTEGER;
            let mut Released: BOOLEAN;
            let mut Unmapped: BOOLEAN;

            Offset = core::mem::zeroed();
            Offset.QuadPart = (*MemoryArea).SectionData.ViewOffset as i64
                + (Address as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64;

            Segment = (*MemoryArea).SectionData.Segment;

            MmLockSectionSegment(Segment);

            Entry = MmGetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset));
            if Entry != 0 && MM_IS_WAIT_PTE(Entry) {
                MmUnlockSectionSegment(Segment);
                if Process != PsInitialSystemProcess as PEPROCESS {
                    KeDetachProcess();
                }
                MmUnlockAddressSpace(AddressSpace);
                ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                    (*(Process as *mut _EPROCESS)).RundownProtect
                ));
                ObDereferenceObject(Process as PVOID);
                return STATUS_UNSUCCESSFUL;
            }

            MmDeleteRmap(Page, Process, Address);
            Unmapped = MmDeleteVirtualMapping(
                Process,
                Address,
                core::ptr::addr_of_mut!(Dirty),
                core::ptr::addr_of_mut!(MapPage),
            );
            if Unmapped == 0 || MapPage != Page as _ {
                KeBugCheckEx(
                    MEMORY_MANAGEMENT,
                    Process as ULONG_PTR,
                    Address as ULONG_PTR,
                    b"rmap.c\0".as_ptr() as ULONG_PTR,
                    321,
                );
            }

            if Page != PFN_FROM_SSE(Entry) {
                let mut SwapEntry: SWAPENTRY;

                MmUnlockSectionSegment(Segment);

                SwapEntry = MmGetSavedSwapEntryPage(Page as _);

                if SwapEntry == 0 && Dirty != 0 {
                    SwapEntry = MmAllocSwapPage();
                    if SwapEntry == 0 {
                        let Region = MmFindRegion(
                            MA_GetStartingAddress(MemoryArea) as PVOID,
                            core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
                            Address,
                            core::ptr::null_mut(),
                        );

                        MmCreateVirtualMapping(Process, Address, (*Region).Protect, Page as _);
                        MmInsertRmap(Page, Process, Address);
                        MmSetDirtyPage(Process, Address);

                        MmUnlockAddressSpace(AddressSpace);
                        if Process != PsInitialSystemProcess as PEPROCESS {
                            KeDetachProcess();
                        }
                        ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                            (*(Process as *mut _EPROCESS)).RundownProtect
                        ));
                        ObDereferenceObject(Process as PVOID);

                        return STATUS_UNSUCCESSFUL;
                    }
                }

                if Dirty != 0 {
                    let mut Dummy: SWAPENTRY = 0;

                    MmCreatePageFileMapping(Process, Address, MM_WAIT_ENTRY as _);
                    MmUnlockAddressSpace(AddressSpace);

                    Status = MmWriteToSwapPage(SwapEntry, Page as _);

                    MmLockAddressSpace(AddressSpace);
                    MmDeletePageFileMapping(Process, Address, core::ptr::addr_of_mut!(Dummy));

                    if Status < 0 {
                        let Region = MmFindRegion(
                            MA_GetStartingAddress(MemoryArea) as PVOID,
                            core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
                            Address,
                            core::ptr::null_mut(),
                        );

                        MmSetSavedSwapEntryPage(Page as _, 0);
                        MmFreeSwapPage(SwapEntry);

                        MmCreateVirtualMapping(Process, Address, (*Region).Protect, Page as _);
                        MmInsertRmap(Page, Process, Address);
                        MmSetDirtyPage(Process, Address);

                        MmUnlockAddressSpace(AddressSpace);
                        if Process != PsInitialSystemProcess as PEPROCESS {
                            KeDetachProcess();
                        }
                        ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                            (*(Process as *mut _EPROCESS)).RundownProtect
                        ));
                        ObDereferenceObject(Process as PVOID);

                        return STATUS_UNSUCCESSFUL;
                    }
                }

                if SwapEntry != 0 {
                    MmCreatePageFileMapping(Process, Address, SwapEntry);
                    MmSetSavedSwapEntryPage(Page as _, 0);
                }

                MmUnlockAddressSpace(AddressSpace);
                if Process != PsInitialSystemProcess as PEPROCESS {
                    KeDetachProcess();
                }
                MmReleasePageMemoryConsumer(MC_USER, Page as _);

                ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                    (*(Process as *mut _EPROCESS)).RundownProtect
                ));
                ObDereferenceObject(Process as PVOID);

                return STATUS_SUCCESS;
            }

            Released = MmUnsharePageEntrySectionSegment(
                MemoryArea,
                Segment,
                core::ptr::addr_of_mut!(Offset),
                Dirty,
                1u8,
                core::ptr::null_mut(),
            );

            MmUnlockSectionSegment(Segment);
            if Process != PsInitialSystemProcess as PEPROCESS {
                KeDetachProcess();
            }
            MmUnlockAddressSpace(AddressSpace);

            ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                (*(Process as *mut _EPROCESS)).RundownProtect
            ));
            ObDereferenceObject(Process as PVOID);

            if Released != 0 {
                return STATUS_SUCCESS;
            }
        } else {
            // No section segment owns this page, yet it reached the page-out path
            // as a section-backed page. x86 treats the contradiction as fatal.
            mm_guard_trip!(
                "rmap.MmPageOutPhysicalAddress no-segment-for-page", 8,
                "Page" => Page, "Address" => Address as ULONG_PTR,
                "Process" => Process as ULONG_PTR,
            );
            #[cfg(target_arch = "aarch64")]
            {
                if Process != PsInitialSystemProcess as PEPROCESS {
                    KeDetachProcess();
                }
                MmUnlockAddressSpace(AddressSpace);
                ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                    (*(Process as *mut _EPROCESS)).RundownProtect
                ));
                ObDereferenceObject(Process as PVOID);
                return STATUS_UNSUCCESSFUL;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }

        break;
    }

    // WriteSegment: on every arch (see the no-rmap case above).
    {
        Segment = MmGetSectionAssociation(Page as _, core::ptr::addr_of_mut!(SegmentOffset));
        if !Segment.is_null() {
            let Released: BOOLEAN;

            MmLockSectionSegment(Segment);

            // The page must still be a ReactOS-Mm page that this segment entry names: a
            // stale association would free a frame something else now owns. arm64 never
            // ran this path before, so check rather than trust its bookkeeping.
            let Entry = MmGetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(SegmentOffset));
            let Pfn = MiGetPfnEntry(Page);
            if Entry == 0
                || IS_SWAP_FROM_SSE(Entry)
                || PFN_FROM_SSE(Entry) != Page as _
                || Pfn.is_null()
                || !crate::ARM3::helpers::MI_IS_ROS_PFN(Pfn)
            {
                MmUnlockSectionSegment(Segment);
                MmDereferenceSegment(Segment);
                mm_guard_trip!(
                    "rmap.MmPageOutPhysicalAddress stale-segment-association", 8,
                    "Page" => Page, "Entry" => Entry, "Segment" => Segment as ULONG_PTR,
                );
                return STATUS_UNSUCCESSFUL;
            }

            Released = MmCheckDirtySegment(Segment, core::ptr::addr_of_mut!(SegmentOffset), 0u8, 1u8);

            MmUnlockSectionSegment(Segment);
            MmDereferenceSegment(Segment);

            if Released != 0 {
                let N = SEGMENT_RECLAIMS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                if N % 1024 == 0 {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        c"[MM] reclaimed cached section page #%lu (pfn %Iu, avail %Iu)\n".as_ptr(),
                        N + 1,
                        Page,
                        MmAvailablePages as usize,
                    );
                }
                return STATUS_SUCCESS;
            }
        }
    }

    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn MmInsertRmap(mut Page: PFN_NUMBER, Process: PEPROCESS, mut Address: PVOID) {
    let Process = opaque_ptr(Process);
    let mut Address = opaque_ptr(Address);
    let mut current_entry: PMM_RMAP_ENTRY;
    let new_entry: PMM_RMAP_ENTRY;
    let PrevSize: ULONG;
    let OldIrql: KIRQL;

    if !RMAP_IS_SEGMENT(Address) {
        Address = PAGE_ROUND_DOWN(Address) as PVOID;
    }

    new_entry = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList)) as PMM_RMAP_ENTRY;
    if new_entry.is_null() {
        // The C `KeBugCheck(MEMORY_MANAGEMENT)`s here (mm/rmap.c:363), and so do
        // we now on x86. Returning instead means the caller believes the mapping
        // was recorded when it was not — and a lost reverse mapping is exactly
        // the precondition for freeing a frame that is still mapped.
        //
        // These two sites were the ONLY ones of the 14 armed MM guards that
        // returned on every arch; the other nine recovery points already split
        // arm64-recovers / x86-bugchecks. arm64 keeps the recovery because
        // removing it turns a boot into a bugcheck (same reasoning as
        // [PFNMISMATCH]); x86 now matches both the C and its nine siblings.
        mm_guard_trip!(
            "rmap.MmInsertRmap lookaside-alloc FAILED (rmap NOT inserted)", 8,
            "Page" => Page, "Process" => Process as ULONG_PTR,
            "Address" => Address as ULONG_PTR,
        );
        #[cfg(target_arch = "aarch64")]
        {
            return;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
    }
    (*new_entry).Address = Address;
    (*new_entry).Process = Process;

    if !RMAP_IS_SEGMENT(Address) {
        let actual_pfn = MmGetPfnForProcess(Process, Address);
        if actual_pfn != Page as _ {
            // The C DPRINT1s ("Insert rmap ... which doesn't match physical
            // address ...") and then `KeBugCheck(MEMORY_MANAGEMENT)`
            // (mm/rmap.c:380); x86 now does too. Returning silently leaves the
            // rmap NOT inserted with the caller told nothing. This was once the
            // leading candidate for the arm64 `MmDeleteRmap mapping-not-in-list`
            // seen once per run — that turned out to be stale FreeLDR identity
            // PTEs (§9.98), and this site was probed as never firing. Both PFNs
            // are reported so a future hit says which side is wrong.
            mm_guard_trip!(
                "rmap.MmInsertRmap PFN MISMATCH (rmap NOT inserted)", 8,
                "Page" => Page, "actual_pfn" => actual_pfn as ULONG_PTR,
                "Process" => Process as ULONG_PTR,
                "Address" => Address as ULONG_PTR,
            );
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList), new_entry as PVOID);
            #[cfg(target_arch = "aarch64")]
            {
                return;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }
    }

    OldIrql = MiAcquirePfnLock();
    current_entry = MmGetRmapListHeadPage(Page as _);

    let mut previous_entry: PMM_RMAP_ENTRY = core::ptr::null_mut();
    while !current_entry.is_null() && ((*current_entry).Address < Address) {
        previous_entry = current_entry;
        current_entry = (*current_entry).Next;
    }

    if !current_entry.is_null() && (*current_entry).Address == Address {
        while !current_entry.is_null() && ((*current_entry).Process < Process) {
            previous_entry = current_entry;
            current_entry = (*current_entry).Next;
        }
    }

    if !current_entry.is_null()
        && (*current_entry).Address == Address
        && (*current_entry).Process == Process
    {
        // Duplicate rmap — skip insertion instead of bugcheck (defensive)
        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList), new_entry as PVOID);
        MiReleasePfnLock(OldIrql);
        return;
    }

    (*new_entry).Next = current_entry;
    if !previous_entry.is_null() {
        (*previous_entry).Next = new_entry;
    } else {
        MmSetRmapListHeadPage(Page as _, new_entry);
    }

    MiReleasePfnLock(OldIrql);

    if !RMAP_IS_SEGMENT(Address) {
        PrevSize = InterlockedExchangeAddUL(
            core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm.WorkingSetSize),
            PAGE_SIZE as ULONG,
        );
        if PrevSize >= (*(Process as *mut _EPROCESS)).Vm.PeakWorkingSetSize {
            (*(Process as *mut _EPROCESS)).Vm.PeakWorkingSetSize = PrevSize + PAGE_SIZE as ULONG;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmDeleteRmap(Page: PFN_NUMBER, Process: PEPROCESS, Address: PVOID) {
    let Process = opaque_ptr(Process);
    let mut Address = opaque_ptr(Address);
    let mut current_entry: PMM_RMAP_ENTRY;
    let mut previous_entry: PMM_RMAP_ENTRY;
    let OldIrql: KIRQL;

    OldIrql = MiAcquirePfnLock();
    previous_entry = core::ptr::null_mut();
    current_entry = MmGetRmapListHeadPage(Page as _);

    while !current_entry.is_null() {
        if (*current_entry).Process == Process && (*current_entry).Address == Address {
            if previous_entry.is_null() {
                MmSetRmapListHeadPage(Page as _, (*current_entry).Next);
            } else {
                (*previous_entry).Next = (*current_entry).Next;
            }
            MiReleasePfnLock(OldIrql);

            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList), current_entry as PVOID);
            if !RMAP_IS_SEGMENT(Address) {
                InterlockedExchangeAddUL(
                    core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Vm.WorkingSetSize),
                    -(PAGE_SIZE as i32) as ULONG,
                );
            }
            return;
        }
        previous_entry = current_entry;
        current_entry = (*current_entry).Next;
    }
    // Walked the whole rmap list and the (Process, Address) mapping we were told
    // to delete is not in it. The caller believes this page is mapped there and
    // the PFN database disagrees — so the page's mapping bookkeeping is ALREADY
    // wrong at this point, and that is precisely the precondition for freeing a
    // frame that is still mapped (§9.97). Of the twelve suppressions this is the
    // one most likely to fire first on the reuse path; x86 bugchecks here.
    mm_guard_trip_atomic!(
        "rmap.MmDeleteRmap mapping-not-in-list", 8,
        "Page" => Page, "Process" => Process as ULONG_PTR,
        "Address" => Address as ULONG_PTR,
        "listhead" => MmGetRmapListHeadPage(Page as _) as ULONG_PTR,
    );
    #[cfg(target_arch = "aarch64")]
    {
        MiReleasePfnLock(OldIrql);
        // "Not in the list" cannot distinguish "never inserted" from "inserted
        // and since removed", and it says nothing about where the caller's PFN
        // came from. The flight recorder already carries OP_PAGE_MAP/
        // OP_PAGE_UNMAP keyed by page-aligned VA with the PFN as payload, so the
        // per-VA ledger answers both. Dumped AFTER the PFN lock is dropped —
        // this is serial I/O and must not run at PFN-lock IRQL — and bounded,
        // because the recorder is a ring and an unbounded dump on a hot path
        // would evict the very history it is trying to show.
        {
            use core::sync::atomic::{AtomicU32, Ordering};
            static DUMPED: AtomicU32 = AtomicU32::new(0);
            if DUMPED.fetch_add(1, Ordering::Relaxed) < 2 {
                crate::ke::flightrec::flightrec_dump_object(
                    (Address as usize) & !((PAGE_SIZE as usize) - 1),
                    24,
                );
            }
        }
        return;
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSegmentRmap(Page: PFN_NUMBER, RawOffset: *mut ULONG) -> PVOID {
    let RawOffset = opaque_ptr(RawOffset);
    let Result: PVOID;
    let mut current_entry: PMM_RMAP_ENTRY;

    current_entry = MmGetRmapListHeadPage(Page as _);
    while !current_entry.is_null() {
        if RMAP_IS_SEGMENT((*current_entry).Address) {
            Result = (*current_entry).Process as PVOID;
            *RawOffset = ((*current_entry).Address as ULONG_PTR & !RMAP_SEGMENT_MASK) as ULONG;
            if /* TODO: _CACHE_SECTION_PAGE_TABLE not in bindgen */ false {
                return core::ptr::null_mut();
            }

            return Result;
        }
        current_entry = (*current_entry).Next;
    }

    core::ptr::null_mut()
}

/// Free an rmap entry already detached from its page (mm/standby.rs repurposes a page
/// under the PFN lock and frees its segment entry later, at passive level).
pub(crate) unsafe fn MiFreeRmapEntry(Entry: PMM_RMAP_ENTRY) {
    ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList), Entry as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn MmDeleteSectionAssociation(Page: PFN_NUMBER) {
    let mut current_entry: PMM_RMAP_ENTRY;
    let mut previous_entry: PMM_RMAP_ENTRY;
    // [guard] let OldIrql: KIRQL = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    previous_entry = core::ptr::null_mut();
    current_entry = MmGetRmapListHeadPage(Page as _);
    while !current_entry.is_null() {
        if RMAP_IS_SEGMENT((*current_entry).Address) {
            if previous_entry.is_null() {
                MmSetRmapListHeadPage(Page as _, (*current_entry).Next);
            } else {
                (*previous_entry).Next = (*current_entry).Next;
            }
            // [guard-drop] MiReleasePfnLock(OldIrql);
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(RmapLookasideList), current_entry as PVOID);
            return;
        }
        previous_entry = current_entry;
        current_entry = (*current_entry).Next;
    }
    // [guard-drop] MiReleasePfnLock(OldIrql);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/rmap.c
//  * PURPOSE:         Kernel memory management functions
//  *
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <cache/section/newmm.h>
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES ********************************************************************/
//
// /* GLOBALS ******************************************************************/
//
// static NPAGED_LOOKASIDE_LIST RmapLookasideList;
//
// /* FUNCTIONS ****************************************************************/
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// static
// VOID
// NTAPI
// RmapListFree(
//     _In_ __drv_freesMem(Mem) PVOID P)
// {
//     ExFreePoolWithTag(P, TAG_RMAP);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MmInitializeRmapList(VOID)
// {
//     ExInitializeNPagedLookasideList (&RmapLookasideList,
//                                      NULL,
//                                      RmapListFree,
//                                      0,
//                                      sizeof(MM_RMAP_ENTRY),
//                                      TAG_RMAP,
//                                      50);
// }
//
// NTSTATUS
// NTAPI
// MmPageOutPhysicalAddress(PFN_NUMBER Page)
// {
//     PMM_RMAP_ENTRY entry;
//     PMEMORY_AREA MemoryArea;
//     PMMSUPPORT AddressSpace;
//     PVOID Address = NULL;
//     PEPROCESS Process = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PMM_SECTION_SEGMENT Segment;
//     LARGE_INTEGER SegmentOffset;
//     KIRQL OldIrql;
//
// GetEntry:
//     OldIrql = MiAcquirePfnLock();
//
//     entry = MmGetRmapListHeadPage(Page as _);
//
//     while (entry && RMAP_IS_SEGMENT(entry->Address))
//         entry = entry->Next;
//
//     if (entry == NULL)
//     {
//         MiReleasePfnLock(OldIrql);
// #ifdef _M_ARM64
//         /* ARM64: No valid rmap entry found — cannot page out. */
//         return STATUS_UNSUCCESSFUL;
// #else
//         goto WriteSegment;
// #endif
//     }
//
//     Process = entry->Process;
//     Address = entry->Address;
//
//     if ((((ULONG_PTR)Address) & 0xFFF) != 0)
//     {
// #ifdef _M_ARM64
//         MiReleasePfnLock(OldIrql);
//         return STATUS_UNSUCCESSFUL;
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
// #endif
//     }
//
//     /* This is for user-mode address only */
//     ASSERT(Address < MmSystemRangeStart);
//
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         MiReleasePfnLock(OldIrql);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     Status = ObReferenceObjectByPointer(Process, PROCESS_ALL_ACCESS, NULL, KernelMode);
//     MiReleasePfnLock(OldIrql);
//     if (!NT_SUCCESS(Status))
//     {
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         return Status;
//     }
//     AddressSpace = &Process->Vm;
//
//     MmLockAddressSpace(AddressSpace);
//
//     MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, Address);
//     if (MemoryArea == NULL || MemoryArea->DeleteInProgress)
//     {
//         MmUnlockAddressSpace(AddressSpace);
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         ObDereferenceObject(Process);
//         goto GetEntry;
//     }
//
//
//     /* Attach to it, if needed */
//     ASSERT(PsGetCurrentProcess() == PsInitialSystemProcess);
//     if (Process != PsInitialSystemProcess)
//         KeAttachProcess(&Process->Pcb);
//
//     if (MmGetPfnForProcess(Process, Address) != Page as _)
//     {
//         /* This changed in the short window where we didn't have any locks */
//         if (Process != PsInitialSystemProcess)
//             KeDetachProcess();
//         MmUnlockAddressSpace(AddressSpace);
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         ObDereferenceObject(Process);
//         goto GetEntry;
//     }
//
//     if (MemoryArea->Type == MEMORY_AREA_SECTION_VIEW)
//     {
//         ULONG_PTR Entry;
//         BOOLEAN Dirty;
//         PFN_NUMBER MapPage;
//         LARGE_INTEGER Offset;
//         BOOLEAN Released;
//         BOOLEAN Unmapped;
//
//         Offset.QuadPart = MemoryArea->SectionData.ViewOffset +
//                  ((ULONG_PTR)Address - MA_GetStartingAddress(MemoryArea));
//
//         Segment = MemoryArea->SectionData.Segment;
//
//         MmLockSectionSegment(Segment);
//
//         Entry = MmGetPageEntrySectionSegment(Segment, &Offset);
//         if (Entry && MM_IS_WAIT_PTE(Entry))
//         {
//             /* The segment is being read or something. Give up */
//             MmUnlockSectionSegment(Segment);
//             if (Process != PsInitialSystemProcess)
//                 KeDetachProcess();
//             MmUnlockAddressSpace(AddressSpace);
//             ExReleaseRundownProtection(&Process->RundownProtect);
//             ObDereferenceObject(Process);
//             return(STATUS_UNSUCCESSFUL);
//         }
//
//         /* Delete this virtual mapping in the process */
//         MmDeleteRmap(Page, Process, Address);
//         Unmapped = MmDeleteVirtualMapping(Process, Address, &Dirty, &MapPage);
//         if (!Unmapped || (MapPage != Page as _))
//         {
//             /*
//              * Something's corrupted, we got there because this process is
//              * supposed to be mapping this page there.
//              */
//             KeBugCheckEx(MEMORY_MANAGEMENT,
//                          (ULONG_PTR)Process,
//                          (ULONG_PTR)Address,
//                          (ULONG_PTR)__FILE__,
//                          __LINE__);
//         }
//
//         if (Page != PFN_FROM_SSE(Entry))
//         {
//             SWAPENTRY SwapEntry;
//
//             /* This page is private to the process */
//             MmUnlockSectionSegment(Segment);
//
//             /* Check if we should write it back to the page file */
//             SwapEntry = MmGetSavedSwapEntryPage(Page as _);
//
//             if ((SwapEntry == 0) && Dirty)
//             {
//                 /* We don't have a Swap entry, yet the page is dirty. Get one */
//                 SwapEntry = MmAllocSwapPage();
//                 if (!SwapEntry)
//                 {
//                     PMM_REGION Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
//                             &MemoryArea->SectionData.RegionListHead,
//                             Address, NULL);
//
//                     /* We can't, so let this page in the Process VM */
//                     MmCreateVirtualMapping(Process, Address, Region->Protect, Page);
//                     MmInsertRmap(Page, Process, Address);
//                     MmSetDirtyPage(Process, Address);
//
//                     MmUnlockAddressSpace(AddressSpace);
//                     if (Process != PsInitialSystemProcess)
//                         KeDetachProcess();
//                     ExReleaseRundownProtection(&Process->RundownProtect);
//                     ObDereferenceObject(Process);
//
//                     return STATUS_UNSUCCESSFUL;
//                 }
//             }
//
//             if (Dirty)
//             {
//                 SWAPENTRY Dummy;
//
//                 /* Put a wait entry into the process and unlock */
//                 MmCreatePageFileMapping(Process, Address, MM_WAIT_ENTRY as _);
//                 MmUnlockAddressSpace(AddressSpace);
//
//                 Status = MmWriteToSwapPage(SwapEntry, Page as _);
//
//                 MmLockAddressSpace(AddressSpace);
//                 MmDeletePageFileMapping(Process, Address, &Dummy);
//                 ASSERT(Dummy == MM_WAIT_ENTRY);
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* We failed at saving the content of this page. Keep it in */
//                     PMM_REGION Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
//                             &MemoryArea->SectionData.RegionListHead,
//                             Address, NULL);
//
//                     /* This Swap Entry is useless to us */
//                     MmSetSavedSwapEntryPage(Page as _, 0);
//                     MmFreeSwapPage(SwapEntry);
//
//                     /* We can't, so let this page in the Process VM */
//                     MmCreateVirtualMapping(Process, Address, Region->Protect, Page);
//                     MmInsertRmap(Page, Process, Address);
//                     MmSetDirtyPage(Process, Address);
//
//                     MmUnlockAddressSpace(AddressSpace);
//                     if (Process != PsInitialSystemProcess)
//                         KeDetachProcess();
//                     ExReleaseRundownProtection(&Process->RundownProtect);
//                     ObDereferenceObject(Process);
//
//                     return STATUS_UNSUCCESSFUL;
//                 }
//             }
//
//             if (SwapEntry)
//             {
//                 /* Keep this in the process VM */
//                 MmCreatePageFileMapping(Process, Address, SwapEntry);
//                 MmSetSavedSwapEntryPage(Page as _, 0);
//             }
//
//             /* We can finally let this page go */
//             MmUnlockAddressSpace(AddressSpace);
//             if (Process != PsInitialSystemProcess)
//                 KeDetachProcess();
// #if DBG
//             OldIrql = MiAcquirePfnLock();
//             ASSERT(MmGetRmapListHeadPage(Page as _) == NULL);
//             MiReleasePfnLock(OldIrql);
// #endif
//             MmReleasePageMemoryConsumer(MC_USER, Page as _);
//
//             ExReleaseRundownProtection(&Process->RundownProtect);
//             ObDereferenceObject(Process);
//
//             return STATUS_SUCCESS;
//         }
//
//         /* One less mapping referencing this segment */
//         Released = MmUnsharePageEntrySectionSegment(MemoryArea, Segment, &Offset, Dirty, TRUE, NULL);
//
//         MmUnlockSectionSegment(Segment);
//         if (Process != PsInitialSystemProcess)
//             KeDetachProcess();
//         MmUnlockAddressSpace(AddressSpace);
//
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         ObDereferenceObject(Process);
//
//         if (Released) return STATUS_SUCCESS;
//     }
// #ifdef NEWCC
//     else if (Type == MEMORY_AREA_CACHE)
//     {
//         /* NEWCC does locking itself */
//         MmUnlockAddressSpace(AddressSpace);
//         Status = MmpPageOutPhysicalAddress(Page);
//     }
// #endif
//     else
//     {
// #ifdef _M_ARM64
//         /* ARM64: Unknown memory area type — skip instead of bugcheck */
//         if (Process != PsInitialSystemProcess)
//             KeDetachProcess();
//         MmUnlockAddressSpace(AddressSpace);
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         ObDereferenceObject(Process);
//         return STATUS_UNSUCCESSFUL;
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
// #endif
//     }
//
// #ifndef _M_ARM64
// WriteSegment:
// #endif
//     /* Now write this page to file, if needed */
//     Segment = MmGetSectionAssociation(Page, &SegmentOffset);
//     if (Segment)
//     {
//         BOOLEAN Released;
//
//         MmLockSectionSegment(Segment);
//
//         Released = MmCheckDirtySegment(Segment as PVOID, &SegmentOffset, FALSE, TRUE);
//
//         MmUnlockSectionSegment(Segment);
//         MmDereferenceSegment(Segment);
//
//         if (Released)
//         {
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* If we are here, then we didn't release the page */
//     return STATUS_UNSUCCESSFUL;
// }
//
// VOID
// NTAPI
// MmInsertRmap(PFN_NUMBER Page, PEPROCESS Process,
//              PVOID Address)
// {
//     PMM_RMAP_ENTRY current_entry;
//     PMM_RMAP_ENTRY new_entry;
//     ULONG PrevSize;
//     KIRQL OldIrql;
//
//     if (!RMAP_IS_SEGMENT(Address))
//         Address = (PVOID)PAGE_ROUND_DOWN(Address);
//
//     new_entry = ExAllocateFromNPagedLookasideList(&RmapLookasideList);
//     if (new_entry == NULL)
//     {
//         KeBugCheck(MEMORY_MANAGEMENT);
//     }
//     new_entry->Address = Address;
//     new_entry->Process = (PEPROCESS)Process;
// #if DBG
//     new_entry->Caller = _ReturnAddress();
// #endif
//
//     if (
//         !RMAP_IS_SEGMENT(Address) &&
//         MmGetPfnForProcess(Process, Address) != Page as _)
//     {
//         DPRINT1("Insert rmap (%d, 0x%.8X) 0x%.8X which doesn't match physical "
//                 "address 0x%.8X\n", Process ? Process->UniqueProcessId : 0,
//                 Address,
//                 MmGetPfnForProcess(Process, Address) << PAGE_SHIFT,
//                 Page << PAGE_SHIFT);
//         KeBugCheck(MEMORY_MANAGEMENT);
//     }
//
//     OldIrql = MiAcquirePfnLock();
//     current_entry = MmGetRmapListHeadPage(Page as _);
//
//     PMM_RMAP_ENTRY previous_entry = NULL;
//     /* Keep the list sorted */
//     while (current_entry && (current_entry->Address < Address))
//     {
//         previous_entry = current_entry;
//         current_entry = current_entry->Next;
//     }
//
//     /* In case of clash in the address, sort by process */
//     if (current_entry && (current_entry->Address == Address))
//     {
//         while (current_entry && (current_entry->Process < Process))
//         {
//             previous_entry = current_entry;
//             current_entry = current_entry->Next;
//         }
//     }
//
//     if (current_entry && (current_entry->Address == Address) && (current_entry->Process == Process))
//     {
// #if DBG
//         DbgPrint("MmInsertRmap tries to add a second rmap entry for address %p\n", current_entry->Address);
//         DbgPrint("    current caller  %p\n", new_entry->Caller);
//         DbgPrint("    previous caller %p\n", current_entry->Caller);
// #endif
//         KeBugCheck(MEMORY_MANAGEMENT);
//     }
//
//     new_entry->Next = current_entry;
//     if (previous_entry)
//         previous_entry->Next = new_entry;
//     else
//         MmSetRmapListHeadPage(Page as _, new_entry);
//
//     MiReleasePfnLock(OldIrql);
//
//     if (!RMAP_IS_SEGMENT(Address))
//     {
//         ASSERT(Process != NULL);
//         PrevSize = InterlockedExchangeAddUL(&Process->Vm.WorkingSetSize, PAGE_SIZE);
//         if (PrevSize >= Process->Vm.PeakWorkingSetSize)
//         {
//             Process->Vm.PeakWorkingSetSize = PrevSize + PAGE_SIZE;
//         }
//     }
// }
//
// VOID
// NTAPI
// MmDeleteRmap(PFN_NUMBER Page, PEPROCESS Process,
//              PVOID Address)
// {
//     PMM_RMAP_ENTRY current_entry, previous_entry;
//     KIRQL OldIrql;
//
//     OldIrql = MiAcquirePfnLock();
//     previous_entry = NULL;
//     current_entry = MmGetRmapListHeadPage(Page as _);
//
//     while (current_entry != NULL)
//     {
//         if (current_entry->Process == (PEPROCESS)Process &&
//                 current_entry->Address == Address)
//         {
//             if (previous_entry == NULL)
//             {
//                 MmSetRmapListHeadPage(Page as _, current_entry->Next);
//             }
//             else
//             {
//                 previous_entry->Next = current_entry->Next;
//             }
//             MiReleasePfnLock(OldIrql);
//
//             ExFreeToNPagedLookasideList(&RmapLookasideList, current_entry);
//             if (!RMAP_IS_SEGMENT(Address))
//             {
//                 ASSERT(Process != NULL);
//                 (void)InterlockedExchangeAddUL(&Process->Vm.WorkingSetSize, -PAGE_SIZE);
//             }
//             return;
//         }
//         previous_entry = current_entry;
//         current_entry = current_entry->Next;
//     }
// #ifdef _M_ARM64
//     /* ARM64: No pagefile — the balance manager may race with the owning process
//      * and find the rmap entry already gone.  Just return instead of bugchecking. */
//     MiReleasePfnLock(OldIrql);
//     return;
// #else
//     KeBugCheck(MEMORY_MANAGEMENT);
// #endif
// }
//
// /*
//
// Return the process pointer given when a previous call to MmInsertRmap was
// called with a process and address pointer that conform to the segment rmap
// schema.  In short, this requires the address part to be 0xffffff00 + n
// where n is between 0 and 255.  When such an rmap exists, it specifies a
// segment rmap in which the process part is a pointer to a slice of a section
// page table, and the low 8 bits of the address represent a page index in the
// page table slice.  Together, this information is used by
// MmGetSectionAssociation to determine which page entry points to this page in
// the segment page table.
//
// */
//
// PVOID
// NTAPI
// MmGetSegmentRmap(PFN_NUMBER Page, PULONG RawOffset)
// {
//     PCACHE_SECTION_PAGE_TABLE Result = NULL;
//     PMM_RMAP_ENTRY current_entry;//, previous_entry;
//
//     /* Must hold the PFN lock */
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//
//     //previous_entry = NULL;
//     current_entry = MmGetRmapListHeadPage(Page as _);
//     while (current_entry != NULL)
//     {
//         if (RMAP_IS_SEGMENT(current_entry->Address))
//         {
//             Result = (PCACHE_SECTION_PAGE_TABLE)current_entry->Process;
//             *RawOffset = (ULONG_PTR)current_entry->Address & ~RMAP_SEGMENT_MASK;
//             if (*Result->Segment->Flags & MM_SEGMENT_INDELETE)
//             {
//                 return NULL;
//             }
//
//             return Result;
//         }
//         //previous_entry = current_entry;
//         current_entry = current_entry->Next;
//     }
//
//     return NULL;
// }
//
// /*
//
// Remove the section rmap associated with the indicated page, if it exists.
//
// */
//
// VOID
// NTAPI
// MmDeleteSectionAssociation(PFN_NUMBER Page)
// {
//     PMM_RMAP_ENTRY current_entry, previous_entry;
//     KIRQL OldIrql = MiAcquirePfnLock();
//
//     previous_entry = NULL;
//     current_entry = MmGetRmapListHeadPage(Page as _);
//     while (current_entry != NULL)
//     {
//         if (RMAP_IS_SEGMENT(current_entry->Address))
//         {
//             if (previous_entry == NULL)
//             {
//                 MmSetRmapListHeadPage(Page as _, current_entry->Next);
//             }
//             else
//             {
//                 previous_entry->Next = current_entry->Next;
//             }
//             MiReleasePfnLock(OldIrql);
//             ExFreeToNPagedLookasideList(&RmapLookasideList, current_entry);
//             return;
//         }
//         previous_entry = current_entry;
//         current_entry = current_entry->Next;
//     }
//     MiReleasePfnLock(OldIrql);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
