use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, PEPROCESS,
    KeZeroPages,
    MiCheckPdeForPagedPool,
    MiMapPageInHyperSpace,
    MiUnmapPageInHyperSpace,
    MmDereferencePage,
    MmRebalanceMemoryConsumers,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

#[no_mangle]
pub static mut MmDynamicPfn: BOOLEAN = 0;
#[no_mangle]
pub static mut MmMirroring: BOOLEAN = 0;

// Registry-controlled values owned by ARM3/pfnlist. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static PFNLIST_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "DynamicMemory",
            core::ptr::addr_of_mut!(MmDynamicPfn)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "Mirroring",
            core::ptr::addr_of_mut!(MmMirroring)),
    ] });
#[no_mangle]
pub static mut MmSystemPageColor: ULONG = 0;

#[no_mangle]
pub static mut MmTransitionSharedPages: ULONG = 0;
#[no_mangle]
pub static mut MmTotalPagesForPagingFile: ULONG = 0;

#[no_mangle]
pub static mut MmZeroedPageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (ZeroedPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmFreePageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (FreePageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};


#[no_mangle]
pub static mut MmStandbyPageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (StandbyPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmStandbyPageListByPriority: [MMPFNLIST; 8] = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmModifiedPageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (ModifiedPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmModifiedPageListByColor: [MMPFNLIST; 1] = [MMPFNLIST {
    Total: 0,
    ListName: (ModifiedPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
}];

#[no_mangle]
pub static mut MmModifiedNoWritePageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (ModifiedNoWritePageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmBadPageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (BadPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmRomPageListHead: MMPFNLIST = MMPFNLIST {
    Total: 0,
    ListName: (StandbyPageList) as u32,
    Flink: LIST_HEAD,
    Blink: LIST_HEAD,
};

#[no_mangle]
pub static mut MmPageLocationList: [PMMPFNLIST; 8] = unsafe {[
    core::ptr::addr_of_mut!(MmZeroedPageListHead),
    core::ptr::addr_of_mut!(MmFreePageListHead),
    core::ptr::addr_of_mut!(MmStandbyPageListHead),
    core::ptr::addr_of_mut!(MmModifiedPageListHead),
    core::ptr::addr_of_mut!(MmModifiedNoWritePageListHead),
    core::ptr::addr_of_mut!(MmBadPageListHead),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
]};

#[inline]
unsafe fn mi_pfn_corrupt(code: ULONG_PTR, p1: ULONG_PTR, p2: ULONG_PTR, p3: ULONG_PTR) {
    KeBugCheckEx(PFN_LIST_CORRUPT, code, p1, p2, p3);
}

#[inline]
unsafe fn mi_validate_pfn_index(page_frame_index: PFN_NUMBER, code: ULONG_PTR) {
    if page_frame_index == 0
        || page_frame_index > MmHighestPhysicalPage
        || page_frame_index < MmLowestPhysicalPage
    {
        mi_pfn_corrupt(
            code,
            page_frame_index as ULONG_PTR,
            MmLowestPhysicalPage as ULONG_PTR,
            MmHighestPhysicalPage as ULONG_PTR,
        );
    }
}

#[no_mangle]
pub static mut MI_PFN_CURRENT_USAGE: ULONG = 0;
#[no_mangle]
pub static mut MI_PFN_CURRENT_PROCESS_NAME: [i8; 16] = [
    b'N' as i8, b'o' as i8, b'n' as i8, b'e' as i8, b' ' as i8,
    b'y' as i8, b'e' as i8, b't' as i8, 0, 0, 0, 0, 0, 0, 0, 0,
];

unsafe fn MiIncrementAvailablePages() {
    MmAvailablePages += 1;

    if MmAvailablePages == MmLowMemoryThreshold {
        KeClearEvent(MiLowMemoryEvent);
    } else if MmAvailablePages == MmHighMemoryThreshold {
        KeSetEvent(MiHighMemoryEvent, 0, 0u8);
    }
}

unsafe fn MiDecrementAvailablePages() {
    if MmAvailablePages == MmHighMemoryThreshold {
        KeClearEvent(MiHighMemoryEvent);
    } else if MmAvailablePages == MmLowMemoryThreshold {
        KeSetEvent(MiLowMemoryEvent, 0, 0u8);
        // Crossing the low mark is when the balancer starts giving back cached section
        // pages (balance.rs MiTrimMemoryConsumer); wake it now rather than at its timer.
        MmRebalanceMemoryConsumers();
    }

    MmAvailablePages -= 1;
    if MmAvailablePages < MmMinimumFreePages {
        MmRebalanceMemoryConsumers();
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiZeroPhysicalPage(PageFrameIndex: PFN_NUMBER) {
    let mut old_irql: KIRQL = 0;
    let process = PsGetCurrentProcess();

    let virtual_address = MiMapPageInHyperSpace(
        process as PEPROCESS,
        (PageFrameIndex) as _,
        core::ptr::addr_of_mut!(old_irql),
    );
    KeZeroPages(virtual_address, (PAGE_SIZE) as u32);
    MiUnmapPageInHyperSpace(process as *mut core::ffi::c_void as *mut _EPROCESS, virtual_address, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn MiUnlinkFreeOrZeroedPage(Entry: PMMPFN) {
    let Entry = opaque_ptr(Entry);
    let old_flink: PFN_NUMBER;
    let mut old_blink: PFN_NUMBER;
    let list_head: PMMPFNLIST;
    let list_name: MMLISTS;
    let color: usize;
    let color_table: PMMCOLOR_TABLES;
    let pfn1: PMMPFN;

    list_head = MmPageLocationList[(*Entry).page_location() as usize];
    if list_head.is_null() {
        mi_pfn_corrupt(0xA0, MiGetPfnEntryIndex(Entry) as ULONG_PTR, (*Entry).page_location() as ULONG_PTR, 0);
    }
    list_name = (*list_head).ListName;
    if (*Entry).end_of_allocation()
        || (*Entry).start_of_allocation()
        || list_name > FreePageList as u32
        || (*list_head).Total == 0
    {
        mi_pfn_corrupt(
            0xA0,
            MiGetPfnEntryIndex(Entry) as ULONG_PTR,
            list_name as ULONG_PTR,
            (*list_head).Total as ULONG_PTR,
        );
    }

    (*list_head).Total -= 1;

    old_flink = (*Entry).flink();
    old_blink = (*Entry).blink();

    if old_flink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_flink)).set_share_count(old_blink as ULONG_PTR);
    } else {
        (*list_head).Blink = old_blink;
    }

    if old_blink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_blink)).set_flink(old_flink);
    } else {
        (*list_head).Flink = old_flink;
    }

    old_blink = MiGetPfnEntryIndex(Entry);
    color = (old_blink & MmSecondaryColorMask as usize) as usize;

    color_table = &mut *MmFreePagesByColor[list_name as usize].add(color);

    let old_flink_color = (*color_table).Flink;
    if old_flink_color == old_blink {
        (*color_table).Flink = mmpfn_original_pte_long(Entry) as PFN_NUMBER;
        if (*color_table).Flink != LIST_HEAD {
            (*MI_PFN_ELEMENT((*color_table).Flink)).set_pte_frame(COLORED_LIST_HEAD as PFN_NUMBER);
        } else {
            (*color_table).Blink = LIST_HEAD as PVOID;
        }
    } else {
        if (*Entry).pte_frame() == COLORED_LIST_HEAD as PFN_NUMBER {
            mi_pfn_corrupt(
                0xA1,
                MiGetPfnEntryIndex(Entry) as ULONG_PTR,
                (*Entry).pte_frame() as ULONG_PTR,
                mmpfn_original_pte_long(Entry) as ULONG_PTR,
            );
        }
        let pfn1 = MI_PFN_ELEMENT((*Entry).pte_frame());
        mmpfn_set_original_pte_long(pfn1, (mmpfn_original_pte_long(Entry)) as usize);

        if mmpfn_original_pte_long(Entry) as PFN_NUMBER != LIST_HEAD {
            let pfn1 = MI_PFN_ELEMENT(mmpfn_original_pte_long(Entry) as PFN_NUMBER);
            (*pfn1).set_pte_frame(((*Entry).pte_frame()) as PFN_NUMBER);
        } else {
            (*color_table).Blink = pfn1 as PVOID;
        }
    }

    if (*color_table).Count == 0 {
        mi_pfn_corrupt(0xA2, old_blink as ULONG_PTR, list_name as ULONG_PTR, color as ULONG_PTR);
    }
    (*color_table).Count -= 1;

    mmpfn_set_original_pte_long(Entry, (0) as usize);
    (*Entry).set_flink(0);
    (*Entry).set_share_count((0) as ULONG_PTR);

    MiDecrementAvailablePages();
}

#[no_mangle]
pub unsafe extern "system" fn MiUnlinkPageFromList(Pfn: PMMPFN) {
    let Pfn = opaque_ptr(Pfn);
    let mut list_head: PMMPFNLIST;
    let old_flink: PFN_NUMBER;
    let old_blink: PFN_NUMBER;

    list_head = MmPageLocationList[(*Pfn).page_location() as usize];
    if list_head.is_null() {
        mi_pfn_corrupt(0xA3, MiGetPfnEntryIndex(Pfn) as ULONG_PTR, (*Pfn).page_location() as ULONG_PTR, 0);
    }
    if (*Pfn).reference_count() != 0 || (*list_head).ListName < StandbyPageList as u32 {
        mi_pfn_corrupt(
            0xA3,
            MiGetPfnEntryIndex(Pfn) as ULONG_PTR,
            (*Pfn).reference_count() as ULONG_PTR,
            (*list_head).ListName as ULONG_PTR,
        );
    }

    if list_head == core::ptr::addr_of_mut!(MmStandbyPageListHead) {
        if mmpfn_rom_pte(Pfn) || !(*Pfn).prototype_pte() {
            mi_pfn_corrupt(
                0xA4,
                MiGetPfnEntryIndex(Pfn) as ULONG_PTR,
                mmpfn_rom_pte(Pfn) as ULONG_PTR,
                (*Pfn).prototype_pte() as ULONG_PTR,
            );
        }
        list_head = &mut MmStandbyPageListByPriority[mmpfn_priority(Pfn) as usize];
        MiDecrementAvailablePages();
        MmTransitionSharedPages -= 1;
    } else if list_head == core::ptr::addr_of_mut!(MmModifiedPageListHead) {
        let original_pte = mmpfn_original_pte(Pfn);
        if mmpte_software_prototype(&original_pte) {
            mi_pfn_corrupt(
                0xA5,
                MiGetPfnEntryIndex(Pfn) as ULONG_PTR,
                mmpfn_original_pte_long(Pfn) as ULONG_PTR,
                0,
            );
        }
        (*list_head).Total -= 1;
        MmTotalPagesForPagingFile -= 1;
        list_head = &mut MmModifiedPageListByColor[0];
        MmTransitionSharedPages -= 1;
    } else if list_head == core::ptr::addr_of_mut!(MmModifiedNoWritePageListHead) {
        mi_pfn_corrupt(0xA6, MiGetPfnEntryIndex(Pfn) as ULONG_PTR, 0, 0);
    }

    if (*Pfn).end_of_allocation() || (*Pfn).start_of_allocation() || (*list_head).Total == 0 {
        mi_pfn_corrupt(
            0xA7,
            MiGetPfnEntryIndex(Pfn) as ULONG_PTR,
            (*Pfn).end_of_allocation() as ULONG_PTR,
            (*Pfn).start_of_allocation() as ULONG_PTR,
        );
    }

    old_flink = (*Pfn).flink();
    old_blink = (*Pfn).blink();

    if old_flink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_flink)).set_share_count(old_blink as ULONG_PTR);
    } else {
        (*list_head).Blink = old_blink;
    }

    if old_blink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_blink)).set_flink(old_flink);
    } else {
        (*list_head).Flink = old_flink;
    }

    // Upstream carries a `/* ReactOS Hack */` that unconditionally zeroes
    // OriginalPte here. That is fine for private (modified-list) pages, but it
    // is WRONG for prototype (standby-list) pages: standby/modified lists link
    // via u1.Flink (flink()) and u2.Blink (share_count()), which we clear just
    // below — OriginalPte is NOT the list linkage for these lists (only the
    // free/zeroed COLOR lists reuse OriginalPte as their Flink). For a shared
    // prototype page, OriginalPte holds the segment prototype PTE that encodes
    // the page protection; MiCompleteProtoPteFault reads it back when the page
    // is faulted in from transition. Zeroing it here made the page resolve as
    // MM_NOACCESS on the next proto fault -> read-only mapping despite a
    // PAGE_READWRITE section -> BUGCHECK 0xBE on the first win32k/GDI write to
    // a session view brought back from standby. Preserve OriginalPte for
    // prototype pages so the protection survives the standby round-trip.
    if !(*Pfn).prototype_pte() {
        mmpfn_set_original_pte_long(Pfn, (0) as usize);
    }
    (*Pfn).set_flink(0);
    (*Pfn).set_share_count((0) as ULONG_PTR);

    (*list_head).Total -= 1;
}

#[no_mangle]
pub unsafe extern "system" fn MiRemovePageByColor(
    PageIndex: PFN_NUMBER,
    Color: ULONG,
) -> PFN_NUMBER {
    let pfn1: PMMPFN;
    let list_head: PMMPFNLIST;
    let list_name: MMLISTS;
    let old_flink: PFN_NUMBER;
    let old_blink: PFN_NUMBER;
    let old_color: USHORT;
    let old_cache: USHORT;
    let color_table: PMMCOLOR_TABLES;

    pfn1 = MI_PFN_ELEMENT(PageIndex);

    old_color = (mmpfn_page_color(pfn1)) as u16;
    old_cache = (*pfn1).cache_attribute();

    list_head = MmPageLocationList[(*pfn1).page_location() as usize];
    if list_head.is_null() {
        mi_pfn_corrupt(0xA8, PageIndex as ULONG_PTR, (*pfn1).page_location() as ULONG_PTR, Color as ULONG_PTR);
    }
    list_name = (*list_head).ListName;
    if Color > MmSecondaryColorMask
        || (*pfn1).removal_requested()
        || mmpfn_rom_pte(pfn1)
        || list_name > FreePageList as u32
        || (*list_head).Total == 0
    {
        mi_pfn_corrupt(
            0xA8,
            PageIndex as ULONG_PTR,
            Color as ULONG_PTR,
            list_name as ULONG_PTR,
        );
    }

    (*list_head).Total -= 1;

    old_flink = (*pfn1).flink();
    old_blink = (*pfn1).blink();

    if old_flink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_flink)).set_share_count(old_blink as ULONG_PTR);
    } else {
        (*list_head).Blink = old_blink;
    }

    if old_blink != LIST_HEAD {
        (*MI_PFN_ELEMENT(old_blink)).set_flink(old_flink);
    } else {
        (*list_head).Flink = old_flink;
    }

    (*pfn1).set_flink(0);
    (*pfn1).set_share_count((0) as ULONG_PTR);

    mmpfn_set_mmpfnentry_raw(pfn1, (0) as u16);
    mmpfn_set_page_color(pfn1, ((old_color) as u16) as u32);
    (*pfn1).set_cache_attribute((old_cache) as u16);

    color_table = &mut *MmFreePagesByColor[list_name as usize].add(Color as usize);

    (*color_table).Flink = mmpfn_original_pte_long(pfn1) as PFN_NUMBER;

    if (*color_table).Flink == LIST_HEAD {
        (*color_table).Blink = LIST_HEAD as PVOID;
    } else {
        (*MI_PFN_ELEMENT((*color_table).Flink)).set_pte_frame(COLORED_LIST_HEAD as PFN_NUMBER);
    }

    if (*color_table).Count == 0 {
        mi_pfn_corrupt(0xA9, PageIndex as ULONG_PTR, list_name as ULONG_PTR, Color as ULONG_PTR);
    }
    (*color_table).Count -= 1;

    mmpfn_set_original_pte_long(pfn1, (0) as usize);

    MiDecrementAvailablePages();

    PageIndex
}

#[no_mangle]
pub unsafe extern "system" fn MiRemoveAnyPage(Color: ULONG) -> PFN_NUMBER {
    let mut page_index: PFN_NUMBER;
    let pfn1: PMMPFN;
    let mut color = Color;

    if MmAvailablePages == 0 {
        return 0;
    }

    page_index = (*MmFreePagesByColor[FreePageList as usize].add(Color as usize)).Flink;
    if page_index == LIST_HEAD {
        page_index = (*MmFreePagesByColor[ZeroedPageList as usize].add(Color as usize)).Flink;
        if page_index == LIST_HEAD {
            page_index = MmFreePageListHead.Flink;
            color = (page_index & MmSecondaryColorMask as usize) as ULONG;
            if page_index == LIST_HEAD {
                page_index = MmZeroedPageListHead.Flink;
                color = (page_index & MmSecondaryColorMask as usize) as ULONG;
                if page_index == LIST_HEAD {
                }
            }
        }
    }

    page_index = MiRemovePageByColor(page_index, color);

    pfn1 = MI_PFN_ELEMENT(page_index);

    page_index
}

#[no_mangle]
pub unsafe extern "system" fn MiRemoveZeroPage(Color: ULONG) -> PFN_NUMBER {
    let mut page_index: PFN_NUMBER;
    let pfn1: PMMPFN;
    let mut zero = 0u8;
    let mut color = Color;

    if MmAvailablePages == 0 {
        return 0;
    }

    page_index = (*MmFreePagesByColor[ZeroedPageList as usize].add(Color as usize)).Flink;
    if page_index == LIST_HEAD {
        page_index = MmZeroedPageListHead.Flink;
        if page_index == LIST_HEAD {
            zero = 1u8;

            page_index = (*MmFreePagesByColor[FreePageList as usize].add(Color as usize)).Flink;
            if page_index == LIST_HEAD {
                page_index = MmFreePageListHead.Flink;
                color = (page_index & MmSecondaryColorMask as usize) as ULONG;
                if page_index == LIST_HEAD {
                }
            }
        } else {
            color = (page_index & MmSecondaryColorMask as usize) as ULONG;
        }
    }

    pfn1 = MI_PFN_ELEMENT(page_index);

    page_index = MiRemovePageByColor(page_index, color);

    if zero != 0 {
        MiZeroPhysicalPage(page_index);
    }

    page_index
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertPageInFreeList(PageFrameIndex: PFN_NUMBER) {
    let list_head: PMMPFNLIST;
    let last_page: PFN_NUMBER;
    let pfn1: PMMPFN;
    let color: usize;
    let blink: PMMPFN;
    let color_table: PMMCOLOR_TABLES;

    mi_validate_pfn_index(PageFrameIndex, 0xAA);
    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    if (*pfn1).must_be_cached()
        || mmpfn_rom_pte(pfn1)
        || (*pfn1).removal_requested()
        || (*pfn1).verifier_allocation()
        || (*pfn1).reference_count() != 0
    {
        mi_pfn_corrupt(
            0xAB,
            PageFrameIndex as ULONG_PTR,
            (*pfn1).reference_count() as ULONG_PTR,
            (*pfn1).entire_frame() as ULONG_PTR,
        );
    }

    list_head = core::ptr::addr_of_mut!(MmFreePageListHead);
    (*list_head).Total += 1;

    last_page = (*list_head).Blink;
    if last_page != LIST_HEAD {
        (*MI_PFN_ELEMENT(last_page)).set_flink(PageFrameIndex);
    } else {
        (*list_head).Flink = PageFrameIndex;
    }

    (*list_head).Blink = PageFrameIndex;

    (*pfn1).set_flink(LIST_HEAD);
    (*pfn1).set_share_count((last_page) as ULONG_PTR);

    (*pfn1).set_page_location((FreePageList) as u16);
    mmpfn_set_priority(pfn1, ((3) as u16) as u32);

    mmpfn_set_in_page_error_u4(pfn1, (0) != 0);
    (*pfn1).set_awe_allocation((0) != 0);

    MiIncrementAvailablePages();

    color = (PageFrameIndex & MmSecondaryColorMask as usize) as usize;

    color_table = &mut *MmFreePagesByColor[FreePageList as usize].add(color);
    if (*color_table).Flink == LIST_HEAD {
        (*pfn1).set_pte_frame((COLORED_LIST_HEAD) as PFN_NUMBER);
        (*color_table).Flink = PageFrameIndex;
    } else {
        blink = (*color_table).Blink as PMMPFN;
        mmpfn_set_original_pte_long(blink, (PageFrameIndex as usize) as usize);
        (*pfn1).set_pte_frame((MiGetPfnEntryIndex(blink)) as PFN_NUMBER);
    }

    (*color_table).Blink = pfn1 as PVOID;
    mmpfn_set_original_pte_long(pfn1, (LIST_HEAD as usize) as usize);
    (*color_table).Count += 1;

    if (*list_head).Total >= 8 {
        KeSetEvent(
            core::ptr::addr_of_mut!(MmZeroingPageEvent) as *mut core::ffi::c_void,
            0,
            0u8,
        );
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MiInsertStandbyListAtFront(PageFrameIndex: PFN_NUMBER) {
    MiInsertStandbyListAtFrontImpl(PageFrameIndex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MiInsertStandbyListAtFront(PageFrameIndex: PFN_NUMBER) {
    MiInsertStandbyListAtFrontImpl(PageFrameIndex);
}

unsafe fn MiInsertStandbyListAtFrontImpl(PageFrameIndex: PFN_NUMBER) {
    let list_head: PMMPFNLIST;
    let flink: PFN_NUMBER;
    let pfn1: PMMPFN;
    let mut pfn2: PMMPFN;

    mi_validate_pfn_index(PageFrameIndex, 0xAC);
    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    if (*pfn1).must_be_cached()
        || (*pfn1).reference_count() != 0
        || !(*pfn1).prototype_pte()
        || mmpfn_rom_pte(pfn1)
    {
        mi_pfn_corrupt(
            0xAD,
            PageFrameIndex as ULONG_PTR,
            (*pfn1).reference_count() as ULONG_PTR,
            (*pfn1).entire_frame() as ULONG_PTR,
        );
    }

    MmTransitionSharedPages += 1;

    list_head = &mut MmStandbyPageListByPriority[mmpfn_priority(pfn1) as usize];
    (*list_head).Total += 1;

    flink = (*list_head).Flink;
    (*list_head).Flink = PageFrameIndex;

    (*pfn1).set_flink(flink);
    (*pfn1).set_share_count((LIST_HEAD) as ULONG_PTR);

    if flink != LIST_HEAD {
        pfn2 = MI_PFN_ELEMENT(flink);
        (*pfn2).set_share_count((PageFrameIndex) as ULONG_PTR);
    } else {
        (*list_head).Blink = PageFrameIndex;
    }

    (*pfn1).set_page_location((StandbyPageList) as u16);

    MiIncrementAvailablePages();
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertPageInList(
    ListHead: PMMPFNLIST,
    PageFrameIndex: PFN_NUMBER,
) {
    let mut flink: PFN_NUMBER;
    let last_page: PFN_NUMBER;
    let pfn1: PMMPFN;
    let mut pfn2: PMMPFN;
    let mut list_name: MMLISTS;
    let mut color_head: PMMCOLOR_TABLES;
    let color: usize;
    let mut list_head = ListHead;

    if list_head == core::ptr::addr_of_mut!(MmFreePageListHead) {
        mi_pfn_corrupt(0xAE, PageFrameIndex as ULONG_PTR, list_head as ULONG_PTR, 0);
    }
    mi_validate_pfn_index(PageFrameIndex, 0xAF);
    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    if (*pfn1).reference_count() != 0 || mmpfn_rom_pte(pfn1) {
        mi_pfn_corrupt(
            0xB0,
            PageFrameIndex as ULONG_PTR,
            (*pfn1).reference_count() as ULONG_PTR,
            mmpfn_rom_pte(pfn1) as ULONG_PTR,
        );
    }

    list_name = (*list_head).ListName;
    if (list_name == StandbyPageList as u32) || (list_name == ModifiedPageList as u32) {
        let original_pte = mmpfn_original_pte(pfn1);
        if (!mmpte_software_prototype(&original_pte))
            && (mmpte_software_transition(&original_pte))
        {
            KeBugCheckEx(MEMORY_MANAGEMENT, 0x8888, 0, 0, 0);
        }
    }

    if list_head == core::ptr::addr_of_mut!(MmStandbyPageListHead) {
        list_head = &mut MmStandbyPageListByPriority[mmpfn_priority(pfn1) as usize];
        if (*list_head).ListName != list_name {
            mi_pfn_corrupt(0xB1, PageFrameIndex as ULONG_PTR, list_name as ULONG_PTR, (*list_head).ListName as ULONG_PTR);
        }
    }

    (*list_head).Total += 1;

    if list_head == core::ptr::addr_of_mut!(MmModifiedPageListHead) {
        let original_pte = mmpfn_original_pte(pfn1);
        if mmpte_software_prototype(&original_pte) {
            mi_pfn_corrupt(0xB2, PageFrameIndex as ULONG_PTR, mmpfn_original_pte_long(pfn1) as ULONG_PTR, 0);
        }
        list_head = &mut MmModifiedPageListByColor[0];
        if (*list_head).ListName != list_name {
            mi_pfn_corrupt(0xB3, PageFrameIndex as ULONG_PTR, list_name as ULONG_PTR, (*list_head).ListName as ULONG_PTR);
        }
        (*list_head).Total += 1;
        MmTotalPagesForPagingFile += 1;
    }

    if (*pfn1).removal_requested() {
        mi_pfn_corrupt(0xB4, PageFrameIndex as ULONG_PTR, 0, 0);
    }

    if list_name == (ZeroedPageList) as u32 {
        flink = (*list_head).Flink;
        (*list_head).Flink = PageFrameIndex;

        (*pfn1).set_flink(flink);
        (*pfn1).set_share_count((LIST_HEAD) as ULONG_PTR);

        if flink != LIST_HEAD {
            pfn2 = MI_PFN_ELEMENT(flink);
            (*pfn2).set_share_count((PageFrameIndex) as ULONG_PTR);
        } else {
            (*list_head).Blink = PageFrameIndex;
        }
    } else {
        last_page = (*list_head).Blink;
        if last_page != LIST_HEAD {
            (*MI_PFN_ELEMENT(last_page)).set_flink(PageFrameIndex);
        } else {
            (*list_head).Flink = PageFrameIndex;
        }

        (*list_head).Blink = PageFrameIndex;

        (*pfn1).set_flink(LIST_HEAD);
        (*pfn1).set_share_count((last_page) as ULONG_PTR);
    }

    (*pfn1).set_page_location((list_name) as u16);

    if list_name <= (StandbyPageList) as u32 {
        MiIncrementAvailablePages();
        if list_name != ZeroedPageList as u32 || mmpfn_in_page_error_u4(pfn1) {
            mi_pfn_corrupt(
                0xB5,
                PageFrameIndex as ULONG_PTR,
                list_name as ULONG_PTR,
                mmpfn_in_page_error_u4(pfn1) as ULONG_PTR,
            );
        }

        color = (PageFrameIndex & MmSecondaryColorMask as usize) as usize;
        color_head = &mut *MmFreePagesByColor[ZeroedPageList as usize].add(color);

        flink = (*color_head).Flink;
        mmpfn_set_original_pte_long(pfn1, (flink as usize) as usize);
        (*pfn1).set_pte_frame((COLORED_LIST_HEAD) as PFN_NUMBER);

        (*color_head).Flink = PageFrameIndex;

        if flink != LIST_HEAD {
            pfn2 = MI_PFN_ELEMENT(flink);
            (*pfn2).set_pte_frame((PageFrameIndex) as PFN_NUMBER);
        } else {
            (*color_head).Blink = pfn1 as PVOID;
        }

        (*color_head).Count += 1;
    } else if list_name == (ModifiedPageList) as u32 {
        let original_pte = mmpfn_original_pte(pfn1);
        if mmpte_software_prototype(&original_pte) || original_pte.soft_page_file_high() != 0 {
            mi_pfn_corrupt(0xB6, PageFrameIndex as ULONG_PTR, mmpfn_original_pte_long(pfn1) as ULONG_PTR, 0);
        }
        MmTransitionSharedPages += 1;
        (*(PsGetCurrentProcess() as *mut _EPROCESS)).ModifiedPageCount += 1;
    } else if list_name == (ModifiedNoWritePageList) as u32 {
        mi_pfn_corrupt(0xB7, PageFrameIndex as ULONG_PTR, list_name as ULONG_PTR, 0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializePfn(
    PageFrameIndex: PFN_NUMBER,
    PointerPte: PMMPTE,
    Modified: BOOLEAN,
) {
    let PointerPte = opaque_ptr(PointerPte);
    let pfn1: PMMPFN;
    let mut status: NTSTATUS;
    let pointer_pte_pte: PMMPTE;

    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    (*pfn1).set_pte_address((PointerPte) as PMMPTE);

    if ((*PointerPte).valid() as usize) == 1 {
        MI_MAKE_SOFTWARE_PTE(
            &mut *mmpfn_original_pte_ptr(pfn1),
            MM_READWRITE,
        );
    } else {
        mmpfn_set_original_pte(pfn1, *PointerPte);
    }

    (*pfn1).set_reference_count((1) as USHORT);
    (*pfn1).set_share_count((1) as ULONG_PTR);
    (*pfn1).set_page_location((ActiveAndValid) as u16);
    (*pfn1).set_modified((Modified) != 0);

    // CacheAttribute: the C (ours and upstream ReactOS) never sets it here, and
    // MiNonCached is 0 — so a page coming off the free list reads as
    // "deliberately uncached" when nobody ever decided anything. That is not
    // cosmetic: MiFlushTbAndCapture consults this on EVERY protection change and
    // force-ORs MM_NOCACHE onto the mask for a MiNonCached PFN, so a plain
    // VirtualProtect on private memory silently rewrites the page as uncached.
    // On x86 that costs performance; on arm64 MM_NOCACHE selects MAIR AttrIndx 1
    // = Device-nGnRnE, where every UNALIGNED access alignment-faults — and a
    // 32-bit guest whose emulator write-protects RWX pages for self-modifying
    // code then re-executes the faulting unaligned store forever. Windows sets
    // MiCached here; pages that really are uncached (MMIO) never come through
    // this path — MmMapIoSpace and the MDL paths set the attribute explicitly.
    (*pfn1).set_cache_attribute(MiCached);

    pointer_pte_pte = MiAddressToPte(PointerPte as PVOID);
    if ((*pointer_pte_pte).valid() as usize) == 0 {
        status = MiCheckPdeForPagedPool(PointerPte as *mut c_void);
        if status < 0 {
            KeBugCheckEx(
                MEMORY_MANAGEMENT,
                0x61940,
                PointerPte as ULONG_PTR,
                (*pointer_pte_pte).Long as ULONG_PTR,
                MiPteToAddress(PointerPte) as ULONG_PTR,
            );
        }
    }

    let page_frame_index = PFN_FROM_PTE(pointer_pte_pte);
    (*pfn1).set_pte_frame((page_frame_index) as PFN_NUMBER);

    let pfn1_parent = MI_PFN_ELEMENT(page_frame_index);
    (*pfn1_parent).set_share_count((*pfn1_parent).share_count().wrapping_add((1) as ULONG_PTR));
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializePfnAndMakePteValid(
    PageFrameIndex: PFN_NUMBER,
    PointerPte: PMMPTE,
    TempPte: MMPTE,
) {
    let PointerPte = opaque_ptr(PointerPte);
    let pfn1: PMMPFN;
    let mut status: NTSTATUS;
    let pointer_pte_pte: PMMPTE;

    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    (*pfn1).set_pte_address((PointerPte) as PMMPTE);
    mmpfn_set_original_pte(pfn1, DemandZeroPte);

    (*pfn1).set_reference_count((*pfn1).reference_count().wrapping_add((1) as USHORT));
    (*pfn1).set_share_count((*pfn1).share_count().wrapping_add((1) as ULONG_PTR));
    (*pfn1).set_page_location((ActiveAndValid) as u16);
    (*pfn1).set_modified((1) != 0);
    // Same reasoning as MiInitializePfn above: cached RAM, stated explicitly.
    (*pfn1).set_cache_attribute(MiCached);

    pointer_pte_pte = MiAddressToPte(PointerPte as PVOID);
    if ((*pointer_pte_pte).valid() as usize) == 0 {
        status = MiCheckPdeForPagedPool(PointerPte as *mut c_void);
        if status < 0 {
            KeBugCheckEx(
                MEMORY_MANAGEMENT,
                0x61940,
                PointerPte as ULONG_PTR,
                (*pointer_pte_pte).Long as ULONG_PTR,
                MiPteToAddress(PointerPte) as ULONG_PTR,
            );
        }
    }

    let page_frame_index = PFN_FROM_PTE(pointer_pte_pte);
    (*pfn1).set_pte_frame((page_frame_index) as PFN_NUMBER);

    let pfn1_parent = MI_PFN_ELEMENT(page_frame_index);
    (*pfn1_parent).set_share_count((*pfn1_parent).share_count().wrapping_add((1) as ULONG_PTR));

    MI_WRITE_VALID_PTE(PointerPte, TempPte);
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeAndChargePfn(
    PageFrameIndex: PPFN_NUMBER,
    PointerPde: PMMPDE,
    ContainingPageFrame: PFN_NUMBER,
    SessionAllocation: BOOLEAN,
) -> NTSTATUS {
    let mut temp_pde: MMPDE;
    let old_irql: KIRQL;

    temp_pde = if SessionAllocation != 0 {
        ValidKernelPdeLocal
    } else {
        ValidKernelPde
    };

    old_irql = MiAcquirePfnLock();

    if ((*PointerPde).valid() as usize) == 1 {
        MiReleasePfnLock(old_irql);
        return STATUS_RETRY;
    }

    *PageFrameIndex = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
    temp_pde.set_page_frame_number((*PageFrameIndex as usize) as PFN_NUMBER);
    MI_WRITE_VALID_PDE(PointerPde, temp_pde);

    MiInitializePfnForOtherProcess(
        *PageFrameIndex,
        PointerPde as PVOID,
        ContainingPageFrame,
    );

    MiReleasePfnLock(old_irql);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MiDecrementShareCount(Pfn1: PMMPFN, PageFrameIndex: PFN_NUMBER) {
    let Pfn1 = opaque_ptr(Pfn1);
    let mut pointer_pte: PMMPTE;
    let pointer_pte_pte: PMMPTE;
    let mut temp_pte: MMPTE;
    let expected_pfn = if PageFrameIndex != 0 && PageFrameIndex <= MmHighestPhysicalPage {
        MI_PFN_ELEMENT(PageFrameIndex)
    } else {
        core::ptr::null_mut()
    };

    if Pfn1.is_null()
        || PageFrameIndex == 0
        || PageFrameIndex > MmHighestPhysicalPage
        || Pfn1 != expected_pfn
        || MI_IS_ROS_PFN(Pfn1)
    {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            0x98,
            PageFrameIndex as ULONG_PTR,
            Pfn1 as ULONG_PTR,
            expected_pfn as ULONG_PTR,
        );
    }

    if ((*Pfn1).page_location() != ActiveAndValid)
        && ((*Pfn1).page_location() != StandbyPageList)
    {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            0x99,
            PageFrameIndex as ULONG_PTR,
            (*Pfn1).page_location() as ULONG_PTR,
            0,
        );
    }

    if (*Pfn1).reference_count() == 0
        || (*Pfn1).share_count() == 0
        || (*Pfn1).share_count() >= 0xF000000
    {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            0x9A,
            PageFrameIndex as ULONG_PTR,
            (*Pfn1).reference_count() as ULONG_PTR,
            (*Pfn1).share_count() as ULONG_PTR,
        );
    }

    (*Pfn1).set_share_count((*Pfn1).share_count() - 1);
    if (*Pfn1).share_count() == 0 {
        if ((*Pfn1).prototype_pte() as u16) != 0 {
            pointer_pte = (*Pfn1).pte_address();
            pointer_pte_pte = MiAddressToPte(pointer_pte as PVOID);
            if (pointer_pte as usize) < (MmPagedPoolStart as usize)
                || (pointer_pte as usize) > (MmPagedPoolEnd as usize)
                || !(*pointer_pte_pte).valid()
            {
                KeBugCheckEx(
                    PFN_LIST_CORRUPT,
                    0x9B,
                    PageFrameIndex as ULONG_PTR,
                    pointer_pte as ULONG_PTR,
                    (*pointer_pte_pte).Long as ULONG_PTR,
                );
            }

            pointer_pte = (*Pfn1).pte_address();
            temp_pte = *pointer_pte;
            temp_pte.set_transition(true);
            temp_pte.set_valid(false);
            temp_pte.set_prototype(false);
            let original_pte = mmpfn_original_pte(Pfn1);
            temp_pte.set_soft_protection((mmpte_software_protection(&original_pte)) as ULONG);
            MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);
        }

        (*Pfn1).set_page_location((TransitionPage) as u16);

        if (*Pfn1).reference_count() == 1 {
            if MI_IS_PFN_DELETED(Pfn1) {
                (*Pfn1).set_reference_count(0);
                (*Pfn1).set_page_location(ActiveAndValid);
                MiInsertPageInFreeList(PageFrameIndex);
            } else {
                MiDecrementReferenceCount(Pfn1, PageFrameIndex);
            }
        } else {
            { let _rc = (*Pfn1).reference_count(); (*Pfn1).set_reference_count(_rc - 1); }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiDecrementReferenceCount(Pfn1: PMMPFN, PageFrameIndex: PFN_NUMBER) {
    let Pfn1 = opaque_ptr(Pfn1);
    if Pfn1.is_null() {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            0x9C,
            PageFrameIndex as ULONG_PTR,
            0,
            0,
        );
    }

    if MI_IS_ROS_PFN(Pfn1) {
        MmDereferencePage((PageFrameIndex) as _);
        return;
    }

    let expected_pfn = if PageFrameIndex <= MmHighestPhysicalPage {
        MI_PFN_ELEMENT(PageFrameIndex)
    } else {
        core::ptr::null_mut()
    };
    if PageFrameIndex > MmHighestPhysicalPage
        || Pfn1 != expected_pfn
        || (*Pfn1).reference_count() == 0
        || (*Pfn1).reference_count() >= 2500
    {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            0x9C,
            PageFrameIndex as ULONG_PTR,
            Pfn1 as ULONG_PTR,
            (*Pfn1).reference_count() as ULONG_PTR,
        );
    }

    { let _rc = (*Pfn1).reference_count(); (*Pfn1).set_reference_count(_rc - 1); }
    if (*Pfn1).reference_count() != 0 {
        return;
    }

    if (*Pfn1).share_count() != 0 {
        KeBugCheckEx(
            PFN_LIST_CORRUPT,
            7,
            PageFrameIndex as ULONG_PTR,
            (*Pfn1).share_count() as ULONG_PTR,
            0,
        );
    }

    if MI_IS_PFN_DELETED(Pfn1) {
        MiInsertPageInFreeList(PageFrameIndex);
        return;
    }

    if ((*Pfn1).modified() as u16) != 0 {
        MiInsertPageInList(core::ptr::addr_of_mut!(MmModifiedPageListHead), PageFrameIndex);
    } else {
        MiInsertStandbyListAtFront(PageFrameIndex);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializePfnForOtherProcess(
    PageFrameIndex: PFN_NUMBER,
    PteAddress: PVOID,
    PteFrame: PFN_NUMBER,
) {
    let PteAddress = opaque_ptr(PteAddress);
    let pfn1: PMMPFN;

    pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
    (*pfn1).set_pte_address((PteAddress as PMMPTE) as PMMPTE);

    MI_MAKE_SOFTWARE_PTE(
        &mut *mmpfn_original_pte_ptr(pfn1),
        MM_READWRITE,
    );

    (*pfn1).set_reference_count((1) as USHORT);
    (*pfn1).set_share_count((1) as ULONG_PTR);
    (*pfn1).set_page_location((ActiveAndValid) as u16);
    (*pfn1).set_modified((1u8) != 0);
    mmpfn_set_in_page_error_u4(pfn1, (0u8) != 0);
    // Same reasoning as MiInitializePfn above: cached RAM, stated explicitly.
    (*pfn1).set_cache_attribute(MiCached);

    if PteFrame != 0 {
        (*pfn1).set_pte_frame((PteFrame) as PFN_NUMBER);

        let pfn1_parent = MI_PFN_ELEMENT(PteFrame);
        (*pfn1_parent).set_share_count((*pfn1_parent).share_count().wrapping_add((1) as ULONG_PTR));
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/pfnlist.c
//  * PURPOSE:         ARM Memory Manager PFN List Manipulation
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
// #if DBG
// #define ASSERT_LIST_INVARIANT(x) \
// do { \
//     ASSERT(((x)->Total == 0 && \
//             (x)->Flink == LIST_HEAD && \
//             (x)->Blink == LIST_HEAD) || \
//            ((x)->Total != 0 && \
//             (x)->Flink != LIST_HEAD && \
//             (x)->Blink != LIST_HEAD)); \
// } while (0)
// #else
// #define ASSERT_LIST_INVARIANT(x)
// #endif
//
// /* GLOBALS ********************************************************************/
//
// BOOLEAN MmDynamicPfn;
// BOOLEAN MmMirroring;
// ULONG MmSystemPageColor;
//
// ULONG MmTransitionSharedPages;
// ULONG MmTotalPagesForPagingFile;
//
// MMPFNLIST MmZeroedPageListHead = {0, ZeroedPageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmFreePageListHead = {0, FreePageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmStandbyPageListHead = {0, StandbyPageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmStandbyPageListByPriority[8];
// MMPFNLIST MmModifiedPageListHead = {0, ModifiedPageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmModifiedPageListByColor[1] = {{0, ModifiedPageList, LIST_HEAD, LIST_HEAD}};
// MMPFNLIST MmModifiedNoWritePageListHead = {0, ModifiedNoWritePageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmBadPageListHead = {0, BadPageList, LIST_HEAD, LIST_HEAD};
// MMPFNLIST MmRomPageListHead = {0, StandbyPageList, LIST_HEAD, LIST_HEAD};
//
// PMMPFNLIST MmPageLocationList[] =
// {
//     &MmZeroedPageListHead,
//     &MmFreePageListHead,
//     &MmStandbyPageListHead,
//     &MmModifiedPageListHead,
//     &MmModifiedNoWritePageListHead,
//     &MmBadPageListHead,
//     NULL,
//     NULL
// };
//
// ULONG MI_PFN_CURRENT_USAGE;
// CHAR MI_PFN_CURRENT_PROCESS_NAME[16] = "None yet";
//
// /* FUNCTIONS ******************************************************************/
// static
// VOID
// MiIncrementAvailablePages(
//     VOID)
// {
//     /* Increment available pages */
//     MmAvailablePages++;
//
//     /* Check if we've reached the configured low memory threshold */
//     if (MmAvailablePages == MmLowMemoryThreshold)
//     {
//         /* Clear the event, because now we're ABOVE the threshold */
//         KeClearEvent(MiLowMemoryEvent);
//     }
//     else if (MmAvailablePages == MmHighMemoryThreshold)
//     {
//         /* Otherwise check if we reached the high threshold and signal the event */
//         KeSetEvent(MiHighMemoryEvent, 0, FALSE);
//     }
// }
//
// static
// VOID
// MiDecrementAvailablePages(
//     VOID)
// {
//     ASSERT(MmAvailablePages > 0);
//
//     /* See if we hit any thresholds */
//     if (MmAvailablePages == MmHighMemoryThreshold)
//     {
//         /* Clear the high memory event */
//         KeClearEvent(MiHighMemoryEvent);
//     }
//     else if (MmAvailablePages == MmLowMemoryThreshold)
//     {
//         /* Signal the low memory event */
//         KeSetEvent(MiLowMemoryEvent, 0, FALSE);
//     }
//
//     /* One less page */
//     MmAvailablePages--;
//     if (MmAvailablePages < MmMinimumFreePages)
//     {
//         /* FIXME: Should wake up the MPW and working set manager, if we had one */
//
//         DPRINT1("Running low on pages: %lu remaining\n", MmAvailablePages);
//
//         /* Call RosMm and see if it can release any pages for us */
//         MmRebalanceMemoryConsumers();
//     }
// }
//
// VOID
// NTAPI
// MiZeroPhysicalPage(IN PFN_NUMBER PageFrameIndex)
// {
//     KIRQL OldIrql;
//     PVOID VirtualAddress;
//     PEPROCESS Process = PsGetCurrentProcess();
//
//     /* Map in hyperspace, then wipe it using XMMI or MEMSET */
//     VirtualAddress = MiMapPageInHyperSpace(Process, PageFrameIndex, &OldIrql);
//     ASSERT(VirtualAddress);
//     KeZeroPages(VirtualAddress, PAGE_SIZE);
//     MiUnmapPageInHyperSpace(Process, VirtualAddress, OldIrql);
// }
//
// VOID
// NTAPI
// MiUnlinkFreeOrZeroedPage(IN PMMPFN Entry)
// {
//     PFN_NUMBER OldFlink, OldBlink;
//     PMMPFNLIST ListHead;
//     MMLISTS ListName;
//     ULONG Color;
//     PMMCOLOR_TABLES ColorTable;
//     PMMPFN Pfn1;
//
//     /* Make sure the PFN lock is held */
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Make sure the PFN entry isn't in-use */
//     ASSERT(Entry->u3.e1.WriteInProgress == 0);
//     ASSERT(Entry->u3.e1.ReadInProgress == 0);
//
//     /* Find the list for this entry, make sure it's the free or zero list */
//     ListHead = MmPageLocationList[Entry->u3.e1.PageLocation];
//     ListName = ListHead->ListName;
//     ASSERT(ListHead != NULL);
//     ASSERT(ListName <= FreePageList);
//     ASSERT_LIST_INVARIANT(ListHead);
//
//     /* Remove one count */
//     ASSERT(ListHead->Total != 0);
//     ListHead->Total--;
//
//     /* Get the forward and back pointers */
//     OldFlink = Entry->u1.Flink;
//     OldBlink = Entry->u2.Blink;
//
//     /* Check if the next entry is the list head */
//     if (OldFlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldFlink)->u2.Blink = OldBlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Blink = OldBlink;
//     }
//
//     /* Check if the back entry is the list head */
//     if (OldBlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldBlink)->u1.Flink = OldFlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Flink = OldFlink;
//     }
//
//     /* Get the page color */
//     OldBlink = MiGetPfnEntryIndex(Entry);
//     Color = OldBlink & MmSecondaryColorMask;
//
//     /* Get the first page on the color list */
//     ColorTable = &MmFreePagesByColor[ListName][Color];
//
//     /* Check if this was was actually the head */
//     OldFlink = ColorTable->Flink;
//     if (OldFlink == OldBlink)
//     {
//         /* Make the table point to the next page this page was linking to */
//         ColorTable->Flink = Entry->OriginalPte.Long;
//         if (ColorTable->Flink != LIST_HEAD)
//         {
//             /* And make the previous link point to the head now */
//             MI_PFN_ELEMENT(ColorTable->Flink)->u4.PteFrame = COLORED_LIST_HEAD;
//         }
//         else
//         {
//             /* And if that page was the head, loop the list back around */
//             ColorTable->Blink = (PVOID)LIST_HEAD;
//         }
//     }
//     else
//     {
//         /* This page shouldn't be pointing back to the head */
//         ASSERT(Entry->u4.PteFrame != COLORED_LIST_HEAD);
//
//         /* Make the back link point to whoever the next page is */
//         Pfn1 = MI_PFN_ELEMENT(Entry->u4.PteFrame);
//         Pfn1->OriginalPte.Long = Entry->OriginalPte.Long;
//
//         /* Check if this page was pointing to the head */
//         if (Entry->OriginalPte.Long != LIST_HEAD)
//         {
//             /* Make the back link point to the head */
//             Pfn1 = MI_PFN_ELEMENT(Entry->OriginalPte.Long);
//             Pfn1->u4.PteFrame = Entry->u4.PteFrame;
//         }
//         else
//         {
//             /* Then the table is directly back pointing to this page now */
//             ColorTable->Blink = Pfn1;
//         }
//     }
//
//     /* One less colored page */
//     ASSERT(ColorTable->Count >= 1);
//     ColorTable->Count--;
//
//     /* ReactOS Hack */
//     Entry->OriginalPte.Long = 0;
//
//     /* We are not on a list anymore */
//     Entry->u1.Flink = Entry->u2.Blink = 0;
//     ASSERT_LIST_INVARIANT(ListHead);
//
//     /* Decrement number of available pages */
//     MiDecrementAvailablePages();
//
// #if MI_TRACE_PFNS
//     ASSERT(MI_PFN_CURRENT_USAGE != MI_USAGE_NOT_SET);
//     Entry->PfnUsage = MI_PFN_CURRENT_USAGE;
//     memcpy(Entry->ProcessName, MI_PFN_CURRENT_PROCESS_NAME, 16);
//     Entry->CallSite = _ReturnAddress();
//     MI_PFN_CURRENT_USAGE = MI_USAGE_NOT_SET;
//     super::helpers::MI_SET_PROCESS2("Not Set");
// #endif
// }
//
// VOID
// NTAPI
// MiUnlinkPageFromList(IN PMMPFN Pfn)
// {
//     PMMPFNLIST ListHead;
//     PFN_NUMBER OldFlink, OldBlink;
//
//     /* Make sure the PFN lock is held */
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* ARM3 should only call this for dead pages */
//     ASSERT(Pfn->u3.e2.ReferenceCount == 0);
//
//     /* Transition pages are supposed to be standby/modified/nowrite */
//     ListHead = MmPageLocationList[Pfn->u3.e1.PageLocation];
//     ASSERT(ListHead->ListName >= StandbyPageList);
//
//     /* Check if this was standby, or modified */
//     if (ListHead == &MmStandbyPageListHead)
//     {
//         /* Should not be a ROM page */
//         ASSERT(Pfn->u3.e1.Rom == 0);
//
//         /* Get the exact list */
//         ListHead = &MmStandbyPageListByPriority[Pfn->u4.Priority];
//
//         /* Decrement number of available pages */
//         MiDecrementAvailablePages();
//
//         /* Decrease transition page counter */
//         ASSERT(Pfn->u3.e1.PrototypePte == 1); /* Only supported ARM3 case */
//         MmTransitionSharedPages--;
//     }
//     else if (ListHead == &MmModifiedPageListHead)
//     {
//         /* Only shared memory (page-file backed) modified pages are supported */
//         ASSERT(Pfn->(OriginalPte.prototype() as usize) == 0);
//
//         /* Decrement the counters */
//         ListHead->Total--;
//         MmTotalPagesForPagingFile--;
//
//         /* Pick the correct colored list */
//         ListHead = &MmModifiedPageListByColor[0];
//
//         /* Decrease transition page counter */
//         MmTransitionSharedPages--;
//     }
//     else if (ListHead == &MmModifiedNoWritePageListHead)
//     {
//         /* List not yet supported */
//         ASSERT(FALSE);
//     }
//
//     /* Nothing should be in progress and the list should not be empty */
//     ASSERT(Pfn->u3.e1.WriteInProgress == 0);
//     ASSERT(Pfn->u3.e1.ReadInProgress == 0);
//     ASSERT(ListHead->Total != 0);
//
//     /* Get the forward and back pointers */
//     OldFlink = Pfn->u1.Flink;
//     OldBlink = Pfn->u2.Blink;
//
//     /* Check if the next entry is the list head */
//     if (OldFlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldFlink)->u2.Blink = OldBlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Blink = OldBlink;
//     }
//
//     /* Check if the back entry is the list head */
//     if (OldBlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldBlink)->u1.Flink = OldFlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Flink = OldFlink;
//     }
//
//     /* ReactOS Hack */
//     Pfn->OriginalPte.Long = 0;
//
//     /* We are not on a list anymore */
//     Pfn->u1.Flink = Pfn->u2.Blink = 0;
//
//     /* Remove one entry from the list */
//     ListHead->Total--;
//
//     ASSERT_LIST_INVARIANT(ListHead);
// }
//
// PFN_NUMBER
// NTAPI
// MiRemovePageByColor(IN PFN_NUMBER PageIndex,
//                     IN ULONG Color)
// {
//     PMMPFN Pfn1;
//     PMMPFNLIST ListHead;
//     MMLISTS ListName;
//     PFN_NUMBER OldFlink, OldBlink;
//     USHORT OldColor, OldCache;
//     PMMCOLOR_TABLES ColorTable;
//
//     /* Make sure PFN lock is held */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(Color < MmSecondaryColors);
//
//     /* Get the PFN entry */
//     Pfn1 = MI_PFN_ELEMENT(PageIndex);
//     ASSERT(Pfn1->u3.e1.RemovalRequested == 0);
//     ASSERT(Pfn1->u3.e1.Rom == 0);
//
//     /* Capture data for later */
//     OldColor = Pfn1->u3.e1.PageColor;
//     OldCache = Pfn1->u3.e1.CacheAttribute;
//
//     /* Could be either on free or zero list */
//     ListHead = MmPageLocationList[Pfn1->u3.e1.PageLocation];
//     ASSERT_LIST_INVARIANT(ListHead);
//     ListName = ListHead->ListName;
//     ASSERT(ListName <= FreePageList);
//
//     /* Remove a page */
//     ListHead->Total--;
//
//     /* Get the forward and back pointers */
//     OldFlink = Pfn1->u1.Flink;
//     OldBlink = Pfn1->u2.Blink;
//
//     /* Check if the next entry is the list head */
//     if (OldFlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldFlink)->u2.Blink = OldBlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Blink = OldBlink;
//     }
//
//     /* Check if the back entry is the list head */
//     if (OldBlink != LIST_HEAD)
//     {
//         /* It is not, so set the backlink of the actual entry, to our backlink */
//         MI_PFN_ELEMENT(OldBlink)->u1.Flink = OldFlink;
//     }
//     else
//     {
//         /* Set the list head's backlink instead */
//         ListHead->Flink = OldFlink;
//     }
//
//     /* We are not on a list anymore */
//     ASSERT_LIST_INVARIANT(ListHead);
//     Pfn1->u1.Flink = Pfn1->u2.Blink = 0;
//
//     /* Zero flags but restore color and cache */
//     Pfn1->u3.e2.ShortFlags = 0;
//     Pfn1->u3.e1.PageColor = OldColor;
//     Pfn1->u3.e1.CacheAttribute = OldCache;
//
//     /* Get the first page on the color list */
//     ASSERT(Color < MmSecondaryColors);
//     ColorTable = &MmFreePagesByColor[ListName][Color];
//     ASSERT(ColorTable->Count >= 1);
//
//     /* Set the forward link to whoever we were pointing to */
//     ColorTable->Flink = Pfn1->OriginalPte.Long;
//
//     /* Get the first page on the color list */
//     if (ColorTable->Flink == LIST_HEAD)
//     {
//         /* This is the beginning of the list, so set the sentinel value */
//         ColorTable->Blink = (PVOID)LIST_HEAD;
//     }
//     else
//     {
//         /* The list is empty, so we are the first page */
//         MI_PFN_ELEMENT(ColorTable->Flink)->u4.PteFrame = COLORED_LIST_HEAD;
//     }
//
//     /* One less page */
//     ColorTable->Count--;
//
//     /* ReactOS Hack */
//     Pfn1->OriginalPte.Long = 0;
//
//     /* Decrement number of available pages */
//     MiDecrementAvailablePages();
//
// #if MI_TRACE_PFNS
//     ASSERT(MI_PFN_CURRENT_USAGE != MI_USAGE_NOT_SET);
//     Pfn1->PfnUsage = MI_PFN_CURRENT_USAGE;
//     memcpy(Pfn1->ProcessName, MI_PFN_CURRENT_PROCESS_NAME, 16);
//     Pfn1->CallSite = _ReturnAddress();
//     MI_PFN_CURRENT_USAGE = MI_USAGE_NOT_SET;
//     super::helpers::MI_SET_PROCESS2("Not Set");
// #endif
//
//     /* Return the page */
//     return PageIndex;
// }
//
// PFN_NUMBER
// NTAPI
// MiRemoveAnyPage(IN ULONG Color)
// {
//     PFN_NUMBER PageIndex;
//     PMMPFN Pfn1;
//
//     /* Make sure PFN lock is held and we have pages */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(Color < MmSecondaryColors);
//     if (MmAvailablePages == 0)
//     {
//         return 0;
//     }
//
//     /* Check the colored free list */
//     PageIndex = MmFreePagesByColor[FreePageList][Color].Flink;
//     if (PageIndex == LIST_HEAD)
//     {
//         /* Check the colored zero list */
//         PageIndex = MmFreePagesByColor[ZeroedPageList][Color].Flink;
//         if (PageIndex == LIST_HEAD)
//         {
//             /* Check the free list */
//             ASSERT_LIST_INVARIANT(&MmFreePageListHead);
//             PageIndex = MmFreePageListHead.Flink;
//             Color = PageIndex & MmSecondaryColorMask;
//             if (PageIndex == LIST_HEAD)
//             {
//                 /* Check the zero list */
//                 ASSERT_LIST_INVARIANT(&MmZeroedPageListHead);
//                 PageIndex = MmZeroedPageListHead.Flink;
//                 Color = PageIndex & MmSecondaryColorMask;
//                 ASSERT(PageIndex != LIST_HEAD);
//                 if (PageIndex == LIST_HEAD)
//                 {
//                     /* FIXME: Should check the standby list */
//                     ASSERT(MmZeroedPageListHead.Total == 0);
//                 }
//             }
//         }
//     }
//
//     /* Remove the page from its list */
//     PageIndex = MiRemovePageByColor(PageIndex, Color);
//     ASSERT(PageIndex != 0);
//
//     /* Sanity checks */
//     Pfn1 = MI_PFN_ELEMENT(PageIndex);
//     ASSERT((Pfn1->u3.e1.PageLocation == FreePageList) ||
//            (Pfn1->u3.e1.PageLocation == ZeroedPageList));
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     ASSERT(Pfn1->u2.ShareCount == 0);
//     ASSERT_LIST_INVARIANT(&MmFreePageListHead);
//     ASSERT_LIST_INVARIANT(&MmZeroedPageListHead);
//
//     /* Return the page */
//     return PageIndex;
// }
//
// PFN_NUMBER
// NTAPI
// MiRemoveZeroPage(IN ULONG Color)
// {
//     PFN_NUMBER PageIndex;
//     PMMPFN Pfn1;
//     BOOLEAN Zero = FALSE;
//
//     /* Make sure PFN lock is held and we have pages */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(Color < MmSecondaryColors);
//     if (MmAvailablePages == 0)
//     {
//         return 0;
//     }
//
//     /* Check the colored zero list */
//     PageIndex = MmFreePagesByColor[ZeroedPageList][Color].Flink;
//     if (PageIndex == LIST_HEAD)
//     {
//         /* Check the zero list */
//         ASSERT_LIST_INVARIANT(&MmZeroedPageListHead);
//         PageIndex = MmZeroedPageListHead.Flink;
//         if (PageIndex == LIST_HEAD)
//         {
//             /* This means there's no zero pages, we have to look for free ones */
//             ASSERT(MmZeroedPageListHead.Total == 0);
//             Zero = TRUE;
//
//             /* Check the colored free list */
//             PageIndex = MmFreePagesByColor[FreePageList][Color].Flink;
//             if (PageIndex == LIST_HEAD)
//             {
//                 /* Check the free list */
//                 ASSERT_LIST_INVARIANT(&MmFreePageListHead);
//                 PageIndex = MmFreePageListHead.Flink;
//                 Color = PageIndex & MmSecondaryColorMask;
//                 ASSERT(PageIndex != LIST_HEAD);
//                 if (PageIndex == LIST_HEAD)
//                 {
//                     /* FIXME: Should check the standby list */
//                     ASSERT(MmZeroedPageListHead.Total == 0);
//                 }
//             }
//         }
//         else
//         {
//             Color = PageIndex & MmSecondaryColorMask;
//         }
//     }
//
//     /* Sanity checks */
//     Pfn1 = MI_PFN_ELEMENT(PageIndex);
//     ASSERT((Pfn1->u3.e1.PageLocation == FreePageList) ||
//            (Pfn1->u3.e1.PageLocation == ZeroedPageList));
//
//     /* Remove the page from its list */
//     PageIndex = MiRemovePageByColor(PageIndex, Color);
//     ASSERT(PageIndex != 0);
//     ASSERT(Pfn1 == MI_PFN_ELEMENT(PageIndex));
//
//     /* Zero it, if needed */
//     if (Zero) MiZeroPhysicalPage(PageIndex);
//
//     /* Sanity checks */
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     ASSERT(Pfn1->u2.ShareCount == 0);
//     ASSERT_LIST_INVARIANT(&MmFreePageListHead);
//     ASSERT_LIST_INVARIANT(&MmZeroedPageListHead);
//
//     /* Return the page */
//     return PageIndex;
// }
//
// VOID
// NTAPI
// MiInsertPageInFreeList(IN PFN_NUMBER PageFrameIndex)
// {
//     PMMPFNLIST ListHead;
//     PFN_NUMBER LastPage;
//     PMMPFN Pfn1;
//     ULONG Color;
//     PMMPFN Blink;
//     PMMCOLOR_TABLES ColorTable;
//
//     /* Make sure the page index is valid */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT((PageFrameIndex != 0) &&
//            (PageFrameIndex <= MmHighestPhysicalPage) &&
//            (PageFrameIndex >= MmLowestPhysicalPage));
//
//     /* Get the PFN entry */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//
//     /* Sanity checks that a right kind of page is being inserted here */
//     ASSERT(Pfn1->u4.MustBeCached == 0);
//     ASSERT(Pfn1->u3.e1.Rom != 1);
//     ASSERT(Pfn1->u3.e1.RemovalRequested == 0);
//     ASSERT(Pfn1->u4.VerifierAllocation == 0);
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//
//     /* Get the free page list and increment its count */
//     ListHead = &MmFreePageListHead;
//     ASSERT_LIST_INVARIANT(ListHead);
//     ListHead->Total++;
//
//     /* Get the last page on the list */
//     LastPage = ListHead->Blink;
//     if (LastPage != LIST_HEAD)
//     {
//         /* Link us with the previous page, so we're at the end now */
//         MI_PFN_ELEMENT(LastPage)->u1.Flink = PageFrameIndex;
//     }
//     else
//     {
//         /* The list is empty, so we are the first page */
//         ListHead->Flink = PageFrameIndex;
//     }
//
//     /* Now make the list head point back to us (since we go at the end) */
//     ListHead->Blink = PageFrameIndex;
//     ASSERT_LIST_INVARIANT(ListHead);
//
//     /* And initialize our own list pointers */
//     Pfn1->u1.Flink = LIST_HEAD;
//     Pfn1->u2.Blink = LastPage;
//
//     /* Set the list name and default priority */
//     Pfn1->u3.e1.PageLocation = FreePageList;
//     Pfn1->u4.Priority = 3;
//
//     /* Clear some status fields */
//     Pfn1->u4.InPageError = 0;
//     Pfn1->u4.AweAllocation = 0;
//
//     /* Increment number of available pages */
//     MiIncrementAvailablePages();
//
//     /* Get the page color */
//     Color = PageFrameIndex & MmSecondaryColorMask;
//
//     /* Get the first page on the color list */
//     ColorTable = &MmFreePagesByColor[FreePageList][Color];
//     if (ColorTable->Flink == LIST_HEAD)
//     {
//         /* The list is empty, so we are the first page */
//         Pfn1->u4.PteFrame = COLORED_LIST_HEAD;
//         ColorTable->Flink = PageFrameIndex;
//     }
//     else
//     {
//         /* Get the previous page */
//         Blink = (PMMPFN)ColorTable->Blink;
//
//         /* Make it link to us, and link back to it */
//         Blink->OriginalPte.Long = PageFrameIndex;
//         Pfn1->u4.PteFrame = MiGetPfnEntryIndex(Blink);
//     }
//
//     /* Now initialize our own list pointers */
//     ColorTable->Blink = Pfn1;
//
//     /* This page is now the last */
//     Pfn1->OriginalPte.Long = LIST_HEAD;
//
//     /* And increase the count in the colored list */
//     ColorTable->Count++;
//
//     /* Notify zero page thread if enough pages are on the free list now */
//     if (ListHead->Total >= 8)
//     {
//         /* Set the event */
//         KeSetEvent(&MmZeroingPageEvent, IO_NO_INCREMENT, FALSE);
//     }
//
// #if MI_TRACE_PFNS
//     Pfn1->PfnUsage = MI_USAGE_FREE_PAGE;
//     RtlZeroMemory(Pfn1->ProcessName, 16);
//     Pfn1->CallSite = NULL;
// #endif
// }
//
// VOID
// FASTCALL
// MiInsertStandbyListAtFront(IN PFN_NUMBER PageFrameIndex)
// {
//     PMMPFNLIST ListHead;
//     PFN_NUMBER Flink;
//     PMMPFN Pfn1, Pfn2;
//
//     /* Make sure the lock is held */
//     DPRINT("Inserting page: %lx into standby list !\n", PageFrameIndex);
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Make sure the PFN is valid */
//     ASSERT((PageFrameIndex != 0) &&
//            (PageFrameIndex <= MmHighestPhysicalPage) &&
//            (PageFrameIndex >= MmLowestPhysicalPage));
//
//     /* Grab the PFN and validate it is the right kind of PFN being inserted */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     ASSERT(Pfn1->u4.MustBeCached == 0);
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     ASSERT(Pfn1->u3.e1.PrototypePte == 1);
//     ASSERT(Pfn1->u3.e1.Rom != 1);
//
//     /* One more transition page on a list */
//     MmTransitionSharedPages++;
//
//     /* Get the standby page list and increment its count */
//     ListHead = &MmStandbyPageListByPriority [Pfn1->u4.Priority];
//     ASSERT_LIST_INVARIANT(ListHead);
//     ListHead->Total++;
//
//     /* Make the head of the list point to this page now */
//     Flink = ListHead->Flink;
//     ListHead->Flink = PageFrameIndex;
//
//     /* Make the page point to the previous head, and back to the list */
//     Pfn1->u1.Flink = Flink;
//     Pfn1->u2.Blink = LIST_HEAD;
//
//     /* Was the list empty? */
//     if (Flink != LIST_HEAD)
//     {
//         /* It wasn't, so update the backlink of the previous head page */
//         Pfn2 = MI_PFN_ELEMENT(Flink);
//         Pfn2->u2.Blink = PageFrameIndex;
//     }
//     else
//     {
//         /* It was empty, so have it loop back around to this new page */
//         ListHead->Blink = PageFrameIndex;
//     }
//
//     /* Move the page onto its new location */
//     Pfn1->u3.e1.PageLocation = StandbyPageList;
//
//     /* Increment number of available pages */
//     MiIncrementAvailablePages();
// }
//
// VOID
// NTAPI
// MiInsertPageInList(IN PMMPFNLIST ListHead,
//                    IN PFN_NUMBER PageFrameIndex)
// {
//     PFN_NUMBER Flink, LastPage;
//     PMMPFN Pfn1, Pfn2;
//     MMLISTS ListName;
//     PMMCOLOR_TABLES ColorHead;
//     ULONG Color;
//
//     /* For free pages, use MiInsertPageInFreeList */
//     ASSERT(ListHead != &MmFreePageListHead);
//
//     /* Make sure the lock is held */
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Make sure the PFN is valid */
//     ASSERT((PageFrameIndex) &&
//            (PageFrameIndex <= MmHighestPhysicalPage) &&
//            (PageFrameIndex >= MmLowestPhysicalPage));
//
//     /* Page should be unused */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     ASSERT(Pfn1->u3.e1.Rom != 1);
//
//     /* Is a standby or modified page being inserted? */
//     ListName = ListHead->ListName;
//     if ((ListName == StandbyPageList) || (ListName == ModifiedPageList))
//     {
//         /* If the page is in transition, it must also be a prototype page */
//         if ((Pfn1->(OriginalPte.prototype() as usize) == 0) &&
//             (Pfn1->(OriginalPte.transition() as usize) == 1))
//         {
//             /* Crash the system on inconsistency */
//             KeBugCheckEx(MEMORY_MANAGEMENT, 0x8888, 0, 0, 0);
//         }
//     }
//
//     /* Standby pages are prioritized, so we need to get the real head */
//     if (ListHead == &MmStandbyPageListHead)
//     {
//         /* Obviously the prioritized list should still have the same name */
//         ListHead = &MmStandbyPageListByPriority[Pfn1->u4.Priority];
//         ASSERT(ListHead->ListName == ListName);
//     }
//
//     /* Increment the list count */
//     ListHead->Total++;
//
//     /* Is a modified page being inserted? */
//     if (ListHead == &MmModifiedPageListHead)
//     {
//         /* For now, only single-prototype pages should end up in this path */
//         DPRINT("Modified page being added: %lx\n", PageFrameIndex);
//         ASSERT(Pfn1->(OriginalPte.prototype() as usize) == 0);
//
//         /* Modified pages are colored when they are selected for page file */
//         ListHead = &MmModifiedPageListByColor[0];
//         ASSERT (ListHead->ListName == ListName);
//         ListHead->Total++;
//
//         /* Increment the number of paging file modified pages */
//         MmTotalPagesForPagingFile++;
//     }
//
//     /* Don't handle bad pages yet yet */
//     ASSERT(Pfn1->u3.e1.RemovalRequested == 0);
//
//     /* Zero pages go to the head, all other pages go to the end */
//     if (ListName == ZeroedPageList)
//     {
//         /* Make the head of the list point to this page now */
//         Flink = ListHead->Flink;
//         ListHead->Flink = PageFrameIndex;
//
//         /* Make the page point to the previous head, and back to the list */
//         Pfn1->u1.Flink = Flink;
//         Pfn1->u2.Blink = LIST_HEAD;
//
//         /* Was the list empty? */
//         if (Flink != LIST_HEAD)
//         {
//             /* It wasn't, so update the backlink of the previous head page */
//             Pfn2 = MI_PFN_ELEMENT(Flink);
//             Pfn2->u2.Blink = PageFrameIndex;
//         }
//         else
//         {
//             /* It was empty, so have it loop back around to this new page */
//             ListHead->Blink = PageFrameIndex;
//         }
//     }
//     else
//     {
//         /* Get the last page on the list */
//         LastPage = ListHead->Blink;
//         if (LastPage != LIST_HEAD)
//         {
//             /* Link us with the previous page, so we're at the end now */
//             MI_PFN_ELEMENT(LastPage)->u1.Flink = PageFrameIndex;
//         }
//         else
//         {
//             /* The list is empty, so we are the first page */
//             ListHead->Flink = PageFrameIndex;
//         }
//
//         /* Now make the list head point back to us (since we go at the end) */
//         ListHead->Blink = PageFrameIndex;
//         ASSERT_LIST_INVARIANT(ListHead);
//
//         /* And initialize our own list pointers */
//         Pfn1->u1.Flink = LIST_HEAD;
//         Pfn1->u2.Blink = LastPage;
//     }
//
//     /* Move the page onto its new location */
//     Pfn1->u3.e1.PageLocation = ListName;
//
//     /* For zero/free pages, we also have to handle the colored lists */
//     if (ListName <= StandbyPageList)
//     {
//         /* Increment number of available pages */
//         MiIncrementAvailablePages();
//
//         /* Sanity checks */
//         ASSERT(ListName == ZeroedPageList);
//         ASSERT(Pfn1->u4.InPageError == 0);
//
//         /* Get the page color */
//         Color = PageFrameIndex & MmSecondaryColorMask;
//
//         /* Get the list for this color */
//         ColorHead = &MmFreePagesByColor[ZeroedPageList][Color];
//
//         /* Get the old head */
//         Flink = ColorHead->Flink;
//
//         /* Make this page point back to the list, and point forwards to the old head */
//         Pfn1->OriginalPte.Long = Flink;
//         Pfn1->u4.PteFrame = COLORED_LIST_HEAD;
//
//         /* Set the new head */
//         ColorHead->Flink = PageFrameIndex;
//
//         /* Was the head empty? */
//         if (Flink != LIST_HEAD)
//         {
//             /* No, so make the old head point to this page */
//             Pfn2 = MI_PFN_ELEMENT(Flink);
//             Pfn2->u4.PteFrame = PageFrameIndex;
//         }
//         else
//         {
//             /* Yes, make it loop back to this page */
//             ColorHead->Blink = (PVOID)Pfn1;
//         }
//
//         /* One more paged on the colored list */
//         ColorHead->Count++;
//
// #if MI_TRACE_PFNS
//             ASSERT(MI_PFN_CURRENT_USAGE == MI_USAGE_NOT_SET);
//             Pfn1->PfnUsage = MI_USAGE_FREE_PAGE;
//             RtlZeroMemory(Pfn1->ProcessName, 16);
//             Pfn1->CallSite = NULL;
// #endif
//     }
//     else if (ListName == ModifiedPageList)
//     {
//         /* In ARM3, page must be destined for page file, and not yet written out */
//         ASSERT(Pfn1->(OriginalPte.prototype() as usize) == 0);
//         ASSERT(Pfn1->OriginalPte.soft_page_file_high() == 0) as ULONG_PTR);
//
//         /* One more transition page */
//         MmTransitionSharedPages++;
//
//         /* Increment the number of per-process modified pages */
//         PsGetCurrentProcess()->ModifiedPageCount++;
//
//         /* FIXME: Wake up modified page writer if there are not enough free pages */
//     }
//     else if (ListName == ModifiedNoWritePageList)
//     {
//         /* This list is not yet implemented */
//         ASSERT(FALSE);
//     }
// }
//
// VOID
// NTAPI
// MiInitializePfn(IN PFN_NUMBER PageFrameIndex,
//                 IN PMMPTE PointerPte,
//                 IN BOOLEAN Modified)
// {
//     PMMPFN Pfn1;
//     NTSTATUS Status;
//     PMMPTE PointerPtePte;
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Setup the PTE */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     Pfn1->PteAddress = PointerPte;
//
//     DPRINT("Called for %p from %p\n", Pfn1, _ReturnAddress());
//
//     /* Check if this PFN is part of a valid address space */
//     if (PointerPte->u.Hard.Valid == 1)
//     {
//         /* Only valid from MmCreateProcessAddressSpace path */
//         ASSERT(PsGetCurrentProcess()->Vm.WorkingSetSize == 0);
//
//         /* Make this a demand zero PTE */
//         MI_MAKE_SOFTWARE_PTE(&Pfn1->OriginalPte, MM_READWRITE);
//     }
//     else
//     {
//         /* Copy the PTE data */
//         Pfn1->OriginalPte = *PointerPte;
//         ASSERT(!((Pfn1->(OriginalPte.prototype() as usize) == 0) &&
//                  (Pfn1->(OriginalPte.transition() as usize) == 1)));
//     }
//
//     /* Otherwise this is a fresh page -- set it up */
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     Pfn1->u3.e2.ReferenceCount = 1;
//     Pfn1->u2.ShareCount = 1;
//     Pfn1->u3.e1.PageLocation = ActiveAndValid;
//     ASSERT(Pfn1->u3.e1.Rom == 0);
//     Pfn1->u3.e1.Modified = Modified;
//
//     /* Get the page table for the PTE */
//     PointerPtePte = MiAddressToPte(PointerPte);
//     if (PointerPtePte->u.Hard.Valid == 0)
//     {
//         /* Make sure the PDE gets paged in properly */
//         Status = MiCheckPdeForPagedPool(PointerPte);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Crash */
//             KeBugCheckEx(MEMORY_MANAGEMENT,
//                          0x61940,
//                          (ULONG_PTR)PointerPte,
//                          (ULONG_PTR)PointerPtePte->u.Long,
//                          (ULONG_PTR)MiPteToAddress(PointerPte));
//         }
//     }
//
//     /* Get the PFN for the page table */
//     PageFrameIndex = PFN_FROM_PTE(PointerPtePte);
//     ASSERT(PageFrameIndex != 0);
//     Pfn1->u4.PteFrame = PageFrameIndex;
//
//     DPRINT("Incrementing share count of %lp from %p\n", PageFrameIndex, _ReturnAddress());
//
//     /* Increase its share count so we don't get rid of it */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     Pfn1->u2.ShareCount++;
// }
//
// VOID
// NTAPI
// MiInitializePfnAndMakePteValid(IN PFN_NUMBER PageFrameIndex,
//                                IN PMMPTE PointerPte,
//                                IN MMPTE TempPte)
// {
//     PMMPFN Pfn1;
//     NTSTATUS Status;
//     PMMPTE PointerPtePte;
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* PTE must be invalid */
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//
//     /* Setup the PTE */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     Pfn1->PteAddress = PointerPte;
//     Pfn1->OriginalPte = DemandZeroPte;
//
//     DPRINT("Incrementing %p from %p\n", Pfn1, _ReturnAddress());
//
//     /* Otherwise this is a fresh page -- set it up */
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     Pfn1->u3.e2.ReferenceCount++;
//     Pfn1->u2.ShareCount++;
//     Pfn1->u3.e1.PageLocation = ActiveAndValid;
//     ASSERT(Pfn1->u3.e1.Rom == 0);
//     Pfn1->u3.e1.Modified = 1;
//
//     /* Get the page table for the PTE */
//     PointerPtePte = MiAddressToPte(PointerPte);
//     if (PointerPtePte->u.Hard.Valid == 0)
//     {
//         /* Make sure the PDE gets paged in properly */
//         Status = MiCheckPdeForPagedPool(PointerPte);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Crash */
//             KeBugCheckEx(MEMORY_MANAGEMENT,
//                          0x61940,
//                          (ULONG_PTR)PointerPte,
//                          (ULONG_PTR)PointerPtePte->u.Long,
//                          (ULONG_PTR)MiPteToAddress(PointerPte));
//         }
//     }
//
//     /* Get the PFN for the page table */
//     PageFrameIndex = PFN_FROM_PTE(PointerPtePte);
//     ASSERT(PageFrameIndex != 0);
//     Pfn1->u4.PteFrame = PageFrameIndex;
//
//     /* Increase its share count so we don't get rid of it */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     Pfn1->u2.ShareCount++;
//     DPRINT("Incrementing %p from %p\n", Pfn1, _ReturnAddress());
//
//     /* Write valid PTE */
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
// }
//
// NTSTATUS
// NTAPI
// MiInitializeAndChargePfn(OUT PPFN_NUMBER PageFrameIndex,
//                          IN PMMPDE PointerPde,
//                          IN PFN_NUMBER ContainingPageFrame,
//                          IN BOOLEAN SessionAllocation)
// {
//     MMPDE TempPde;
//     KIRQL OldIrql;
//
//     /* Use either a global or local PDE */
//     TempPde = SessionAllocation ? ValidKernelPdeLocal : ValidKernelPde;
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Make sure nobody is racing us */
//     if (PointerPde->u.Hard.Valid == 1)
//     {
//         /* Return special error if that was the case */
//         MiReleasePfnLock(OldIrql);
//         return STATUS_RETRY;
//     }
//
//     /* Grab a zero page and set the PFN, then make it valid */
//     *PageFrameIndex = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
//     TempPde.set_page_frame_number((*PageFrameIndex) as PFN_NUMBER);
//     MI_WRITE_VALID_PDE(PointerPde, TempPde);
//
//     /* Initialize the PFN */
//     MiInitializePfnForOtherProcess(*PageFrameIndex,
//                                    PointerPde,
//                                    ContainingPageFrame);
//     ASSERT(MI_PFN_ELEMENT(*PageFrameIndex)->u1.WsIndex == 0);
//
//     /* Release the lock and return success */
//     MiReleasePfnLock(OldIrql);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiDecrementShareCount(IN PMMPFN Pfn1,
//                       IN PFN_NUMBER PageFrameIndex)
// {
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     ASSERT(PageFrameIndex > 0);
//     ASSERT(MI_PFN_ELEMENT(PageFrameIndex) != NULL);
//     ASSERT(Pfn1 == MI_PFN_ELEMENT(PageFrameIndex));
//     ASSERT(MI_IS_ROS_PFN(Pfn1) == FALSE);
//
//     DPRINT("Decrementing %p from %p\n", Pfn1, _ReturnAddress());
//
//     /* Page must be in-use */
//     if ((Pfn1->u3.e1.PageLocation != ActiveAndValid) &&
//         (Pfn1->u3.e1.PageLocation != StandbyPageList))
//     {
//         /* Otherwise we have PFN corruption */
//         KeBugCheckEx(PFN_LIST_CORRUPT,
//                      0x99,
//                      PageFrameIndex,
//                      Pfn1->u3.e1.PageLocation,
//                      0);
//     }
//
//     /* Page should at least have one reference */
//     ASSERT(Pfn1->u3.e2.ReferenceCount != 0);
//
//     /* Check if the share count is now 0 */
//     ASSERT(Pfn1->u2.ShareCount < 0xF000000);
//     if (!--Pfn1->u2.ShareCount)
//     {
//         /* Was this a prototype PTE? */
//         if (Pfn1->u3.e1.PrototypePte)
//         {
//             /* Grab the PTE address and make sure it's in prototype pool */
//             PointerPte = Pfn1->PteAddress;
//             ASSERT((PointerPte >= (PMMPTE)MmPagedPoolStart) && (PointerPte <= (PMMPTE)MmPagedPoolEnd));
//
//             /* The PTE that backs it should also be valdi */
//             PointerPte = MiAddressToPte(PointerPte);
//             ASSERT(PointerPte->u.Hard.Valid == 1);
//
//             /* Get the original prototype PTE and turn it into a transition PTE */
//             PointerPte = Pfn1->PteAddress;
//             TempPte = *PointerPte;
//             TempPte.set_transition(true);
//             TempPte.set_valid(false);
//             TempPte.set_prototype(false);
//             TempPte.set_soft_protection((Pfn1->OriginalPte.soft_protection()) as ULONG);
//             MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//             DPRINT("Marking PTE: %p as transition (%p - %lx)\n", PointerPte, Pfn1, MiGetPfnEntryIndex(Pfn1));
//         }
//
//         /* Put the page in transition */
//         Pfn1->u3.e1.PageLocation = TransitionPage;
//
//         /* PFN lock must be held */
//         MI_ASSERT_PFN_LOCK_HELD();
//
//         if (Pfn1->u3.e2.ReferenceCount == 1)
//         {
//             /* Is there still a PFN for this page? */
//             if (MI_IS_PFN_DELETED(Pfn1))
//             {
//                 /* Clear the last reference */
//                 Pfn1->u3.e2.ReferenceCount = 0;
//                 ASSERT(Pfn1->(OriginalPte.prototype() as usize) == 0);
//
//                 /* Mark the page temporarily as valid, we're going to make it free soon */
//                 Pfn1->u3.e1.PageLocation = ActiveAndValid;
//
//                 /* Bring it back into the free list */
//                 MiInsertPageInFreeList(PageFrameIndex);
//             }
//             else
//             {
//                 /* PFN not yet deleted, drop a ref count */
//                 MiDecrementReferenceCount(Pfn1, PageFrameIndex);
//             }
//         }
//         else
//         {
//             /* Otherwise, just drop the reference count */
//             InterlockedDecrement16((PSHORT)&Pfn1->u3.e2.ReferenceCount);
//         }
//     }
// }
//
// VOID
// NTAPI
// MmDereferencePage(PFN_NUMBER Pfn);
//
// VOID
// NTAPI
// MiDecrementReferenceCount(IN PMMPFN Pfn1,
//                           IN PFN_NUMBER PageFrameIndex)
// {
//     /* PFN lock must be held */
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Handle RosMm PFNs here, too (in case they got locked/unlocked by ARM3) */
//     if (MI_IS_ROS_PFN(Pfn1))
//     {
//         MmDereferencePage(PageFrameIndex);
//         return;
//     }
//
//     /* Sanity checks on the page */
//     if (PageFrameIndex > MmHighestPhysicalPage ||
//         Pfn1 != MI_PFN_ELEMENT(PageFrameIndex) ||
//         Pfn1->u3.e2.ReferenceCount == 0 ||
//         Pfn1->u3.e2.ReferenceCount >= 2500)
//     {
//         DPRINT1("PageFrameIndex=0x%lx, MmHighestPhysicalPage=0x%lx\n", PageFrameIndex, MmHighestPhysicalPage);
//         DPRINT1("Pfn1=%p, Element=%p, RefCount=%u\n", Pfn1, MI_PFN_ELEMENT(PageFrameIndex), Pfn1->u3.e2.ReferenceCount);
//         ASSERT(PageFrameIndex <= MmHighestPhysicalPage);
//         ASSERT(Pfn1 == MI_PFN_ELEMENT(PageFrameIndex));
//         ASSERT(Pfn1->u3.e2.ReferenceCount != 0);
//         ASSERT(Pfn1->u3.e2.ReferenceCount < 2500);
//     }
//
//     /* Dereference the page, bail out if it's still alive */
//     InterlockedDecrement16((PSHORT)&Pfn1->u3.e2.ReferenceCount);
//     if (Pfn1->u3.e2.ReferenceCount) return;
//
//     /* Nobody should still have reference to this page */
//     if (Pfn1->u2.ShareCount != 0)
//     {
//         /* Otherwise something's really wrong */
//         KeBugCheckEx(PFN_LIST_CORRUPT, 7, PageFrameIndex, Pfn1->u2.ShareCount, 0);
//     }
//
//     /* And it should be lying on some page list */
//     ASSERT(Pfn1->u3.e1.PageLocation != ActiveAndValid);
//
//     /* Did someone set the delete flag? */
//     if (MI_IS_PFN_DELETED(Pfn1))
//     {
//         /* Insert it into the free list, there's nothing left to do */
//         MiInsertPageInFreeList(PageFrameIndex);
//         return;
//     }
//
//     /* Check to see which list this page should go into */
//     if (Pfn1->u3.e1.Modified == 1)
//     {
//         /* Push it into the modified page list */
//         MiInsertPageInList(&MmModifiedPageListHead, PageFrameIndex);
//     }
//     else
//     {
//         /* Otherwise, insert this page into the standby list */
//         ASSERT(Pfn1->u3.e1.RemovalRequested == 0);
//         MiInsertStandbyListAtFront(PageFrameIndex);
//     }
// }
//
// VOID
// NTAPI
// MiInitializePfnForOtherProcess(IN PFN_NUMBER PageFrameIndex,
//                                IN PVOID PteAddress,
//                                IN PFN_NUMBER PteFrame)
// {
//     PMMPFN Pfn1;
//
//     /* Setup the PTE */
//     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//     Pfn1->PteAddress = PteAddress;
//
//     /* Make this a software PTE */
//     MI_MAKE_SOFTWARE_PTE(&Pfn1->OriginalPte, MM_READWRITE);
//
//     DPRINT("Called for %p from %p\n", Pfn1, _ReturnAddress());
//
//     /* Setup the page */
//     ASSERT(Pfn1->u3.e2.ReferenceCount == 0);
//     Pfn1->u3.e2.ReferenceCount = 1;
//     Pfn1->u2.ShareCount = 1;
//     Pfn1->u3.e1.PageLocation = ActiveAndValid;
//     Pfn1->u3.e1.Modified = TRUE;
//     Pfn1->u4.InPageError = FALSE;
//
//     /* Did we get a PFN for the page table */
//     if (PteFrame)
//     {
//         /* Store it */
//         Pfn1->u4.PteFrame = PteFrame;
//
//         /* Increase its share count so we don't get rid of it */
//         Pfn1 = MI_PFN_ELEMENT(PteFrame);
//
//         DPRINT("Incrementing %p from %p\n", Pfn1, _ReturnAddress());
//         Pfn1->u2.ShareCount++;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
