use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{ExInitializePushLock, KPROCESSOR_MODE, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeStackAttachProcess, KeUnstackDetachProcess, KeWaitForSingleObject, KeZeroPages, LCID, MiAcquireExpansionLock, MiDecrementShareCount, MiInitializePfnAndMakePteValid, MiInitializePfnForOtherProcess, MiInitializeSystemSpaceMap, MiReleaseExpansionLock, MiRemoveZeroPageSafe, MmIsAddressValid, PEPROCESS, PETHREAD, PKAPC_STATE, PMMSESSION, PMMWSL, PMMWSLE, PMM_SESSION_SPACE, _EPROCESS, _KGUARDED_MUTEX, _KPROCESS, _MMPFN, _MMPTE, _MMWSL, _MMWSLE, _MM_SESSION_SPACE};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}

/// CONTAINING_RECORD macro: given a pointer to a field within a struct, return
/// a pointer to the containing struct instance.
macro_rules! CONTAINING_RECORD {
    ($address:expr, $type:ty, $field:ident) => {
        ($address as *const u8).sub(core::mem::offset_of!($type, $field)) as *mut $type
    };
}

#[no_mangle]
pub static mut MmSessionSpace: PMM_SESSION_SPACE = core::ptr::null_mut();
static mut MiSessionDataPages: PFN_NUMBER = 0;
static mut MiSessionTagPages: PFN_NUMBER = 0;
static mut MiSessionTagSizePages: PFN_NUMBER = 0;
static mut MiSessionBigPoolPages: PFN_NUMBER = 0;
static mut MiSessionCreateCharge: PFN_NUMBER = 0;
static mut MiSessionIdMutex: _KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmSessionDataPages: LONG = 0;
static mut MiSessionIdBitmap: PRTL_BITMAP = core::ptr::null_mut();
static mut MiSessionLeaderExists: LONG = 0;

static mut MiSessionWsList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmWorkingSetExpansionHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmExpansionLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut MiExpansionLockOwner: PETHREAD = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionWsSupport() {
    InitializeListHead(core::ptr::addr_of_mut!(MiSessionWsList) as PLIST_ENTRY);
}

#[no_mangle]
pub unsafe extern "system" fn MmIsSessionAddress(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    if MI_IS_SESSION_ADDRESS(address) { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSessionLocaleId() -> LCID {
    let process = PsGetCurrentProcess() as *mut _EPROCESS;

    if (*process).Vm.Flags.SessionLeader() == 0 {
        if !(*process).Session.is_null() {
            return (*((*process).Session as PMM_SESSION_SPACE)).LocaleId;
        }
    }

    PsDefaultThreadLocaleId
}

#[no_mangle]
pub unsafe extern "system" fn MmSetSessionLocaleId(locale_id: LCID) {
    let current_process = PsGetCurrentProcess() as *mut _EPROCESS;
    if (*current_process).Vm.Flags.SessionLeader() == 0
        && !(*current_process).Session.is_null()
    {
        (*((*current_process).Session as PMM_SESSION_SPACE)).LocaleId = locale_id;
    } else {
        PsDefaultThreadLocaleId = locale_id;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionIds() {
    let mut total_pages: PFN_NUMBER;
    let size: ULONG;
    let bitmap_size: ULONG;

    total_pages = MI_SESSION_DATA_PAGES_MAXIMUM;
    MiSessionDataPages = ROUND_TO_PAGES((core::mem::size_of::<_MM_SESSION_SPACE>() as ULONG) as usize) >> PAGE_SHIFT;
    total_pages -= MiSessionDataPages;

    MiSessionTagSizePages = 2;
    MiSessionBigPoolPages = 1;
    MiSessionTagPages = MiSessionTagSizePages + MiSessionBigPoolPages;

    MiSessionCreateCharge = 1 + MiSessionDataPages + MiSessionTagPages;

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(MiSessionIdMutex) as *mut KGUARDED_MUTEX);

    size = MI_INITIAL_SESSION_IDS;
    bitmap_size = ((size + 31) / 32) * core::mem::size_of::<ULONG>() as ULONG;
    MiSessionIdBitmap = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<RTL_BITMAP>() as ULONG + bitmap_size) as usize,
        TAG_MM,
    ) as PRTL_BITMAP;
    if !MiSessionIdBitmap.is_null() {
        RtlInitializeBitMap(
            MiSessionIdBitmap,
            (MiSessionIdBitmap as usize + core::mem::size_of::<RTL_BITMAP>()) as PVOID as *mut u32,
            size,
        );
        RtlClearAllBits(MiSessionIdBitmap);
    } else {
        KeBugCheckEx(
            INSTALL_MORE_MEMORY,
            MmNumberOfPhysicalPages as usize,
            MmLowestPhysicalPage as usize,
            MmHighestPhysicalPage as usize,
            0x200,
        );
    }
}

unsafe fn MiSessionLeader(process: PEPROCESS) {
    let old_irql: KIRQL;

    old_irql = MiAcquireExpansionLock();
    (*(process as *mut _EPROCESS)).Vm.Flags.set_SessionLeader(1);
    MiReleaseExpansionLock(old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSessionId(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    let session_global: PMM_SESSION_SPACE;

    if (*(process as *mut _EPROCESS)).Vm.Flags.SessionLeader() == 1 {
        return 0;
    }

    session_global = (*(process as *mut _EPROCESS)).Session as PMM_SESSION_SPACE;
    if session_global.is_null() {
        return 0;
    }
    (*session_global).SessionId
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSessionIdEx(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    let session_global: PMM_SESSION_SPACE;

    if (*(process as *mut _EPROCESS)).Vm.Flags.SessionLeader() == 1 {
        return 0;
    }

    session_global = (*(process as *mut _EPROCESS)).Session as PMM_SESSION_SPACE;
    if session_global.is_null() {
        return 0xFFFFFFFF;
    }
    (*session_global).SessionId
}

#[no_mangle]
pub unsafe extern "system" fn MiReleaseProcessReferenceToSessionDataPage(
    session_global: PMM_SESSION_SPACE,
) {
    let mut i: ULONG;
    let session_id: ULONG;
    let mut pointer_pte: PMMPTE;
    let mut page_frame_index: [PFN_NUMBER; MI_SESSION_DATA_PAGES_MAXIMUM as usize] =
        [0; MI_SESSION_DATA_PAGES_MAXIMUM as usize];
    let mut pfn1: PMMPFN;
    let mut old_irql: KIRQL;

    if InterlockedDecrement(core::ptr::addr_of_mut!(
        (*session_global).ProcessReferenceToSession
    )) != 0
    {
        return;
    }

    session_id = (*session_global).SessionId;

    #[cfg(not(target_arch = "x86_64"))]
    ExFreePoolWithTag((*session_global).PageTables as PVOID, u32::from_be_bytes(*b"tHmM"));

    pointer_pte = MiAddressToPte(session_global as PVOID);
    i = 0;
    while i < MiSessionDataPages as ULONG {
        page_frame_index[i as usize] = PFN_FROM_PTE(pointer_pte.add(i as usize));
        i += 1;
    }

    MiReleaseSystemPtes(pointer_pte, MiSessionDataPages as ULONG, SystemPteSpace);

    i = 0;
    while i < MiSessionDataPages as ULONG {
        pfn1 = MI_PFN_ELEMENT(page_frame_index[i as usize]);
        MI_SET_PFN_DELETED(pfn1);
        i += 1;
    }

    old_irql = MiAcquirePfnLock();
    i = 0;
    while i < MiSessionDataPages as ULONG {
        pfn1 = MI_PFN_ELEMENT(page_frame_index[i as usize]);
        MiDecrementShareCount(pfn1 as *mut _MMPFN, (page_frame_index[i as usize]) as _);
        i += 1;
    }

    MiReleasePfnLock(old_irql);

    InterlockedDecrement(core::ptr::addr_of_mut!(MmSessionDataPages));

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(MiSessionIdMutex) as *mut KGUARDED_MUTEX);
    RtlClearBit(MiSessionIdBitmap, session_id);
    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(MiSessionIdMutex) as *mut KGUARDED_MUTEX);
}

unsafe fn MiDereferenceSessionFinal() {
    let session_global: PMM_SESSION_SPACE;
    let mut old_irql: KIRQL;

    session_global = (*(MmSessionSpace as *mut _MM_SESSION_SPACE)).GlobalVirtualAddress as PMM_SESSION_SPACE;

    old_irql = MiAcquireExpansionLock();

    // C: `u.Flags.DeletePending = 1`. Every session-flag access here used raw bit indices with
    // Initialized (bit 0) and DeletePending (bit 1) swapped; the named accessors fix the class.
    (*session_global).u.Flags.set_DeletePending(1);

    if (*session_global).AttachCount != 0 {
        KeInitializeEvent(
            core::ptr::addr_of_mut!((*session_global).AttachEvent),
            (_EVENT_TYPE_NotificationEvent as i32) as u32,
            0u8,
        );

        MiReleaseExpansionLock(old_irql);

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*session_global).AttachEvent) as PVOID,
            (_KWAIT_REASON_WrVirtualMemory as i32) as u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );

        old_irql = MiAcquireExpansionLock();
    }

    if !(*session_global)
        .Vm
        .WorkingSetExpansionLinks
        .Flink
        .is_null()
    {
        RemoveEntryList(core::ptr::addr_of_mut!((*session_global).Vm.WorkingSetExpansionLinks)
            as PLIST_ENTRY);
        (*session_global).Vm.WorkingSetExpansionLinks.Flink = core::ptr::null_mut();
    }

    if !(*session_global).WsListEntry.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*session_global).WsListEntry) as PLIST_ENTRY);
        (*session_global).WsListEntry.Flink = core::ptr::null_mut();
    }

    MiReleaseExpansionLock(old_irql);

    if (*session_global).Win32KDriverUnload.is_some() {
        ((*session_global).Win32KDriverUnload.unwrap())(core::ptr::null_mut());
    }
}

unsafe fn MiDereferenceSession() {
    let session_global: PMM_SESSION_SPACE;
    let process: PEPROCESS;
    let reference_count: ULONG;
    let session_id: ULONG;

    session_global = (*(MmSessionSpace as *mut _MM_SESSION_SPACE)).GlobalVirtualAddress as PMM_SESSION_SPACE;

    session_id = (*MmSessionSpace).SessionId;

    process = PsGetCurrentProcess() as *mut _EPROCESS;

    InterlockedDecrement(core::ptr::addr_of_mut!(
        (*MmSessionSpace).ResidentProcessCount
    ) as *mut LONG);

    reference_count =
        InterlockedDecrement(core::ptr::addr_of_mut!((*MmSessionSpace).ReferenceCount) as *mut LONG)
            as ULONG;
    if reference_count == 0 {
        MiDereferenceSessionFinal();
        return;
    }

    if (*(process as *mut _EPROCESS)).Vm.Flags.SessionLeader() != 0 {
        KeFlushEntireTb(0u8, 0u8);

        MiReleaseProcessReferenceToSessionDataPage(session_global);
    }

    let flags_ptr = core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags);
    loop {
        let old = *flags_ptr;
        let new = old & !(0x00010000u32);
        if InterlockedCompareExchange(flags_ptr as *mut LONG, new as LONG, old as LONG) as u32 == old {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiSessionRemoveProcess() {
    let current_process = PsGetCurrentProcess() as *mut _EPROCESS;
    let mut old_irql: KIRQL;

    if ((*current_process).__bindgen_anon_2.Flags & 0x00010000) == 0
        || (*current_process).Vm.Flags.SessionLeader() != 0
    {
        return;
    }

    old_irql = MiAcquireExpansionLock();

    RemoveEntryList(core::ptr::addr_of_mut!((*current_process).SessionProcessLinks) as PLIST_ENTRY);

    MiReleaseExpansionLock(old_irql);

    MiDereferenceSession();
}

#[no_mangle]
pub unsafe extern "system" fn MiSessionAddProcess(new_process: PEPROCESS) {
    let new_process = opaque_ptr(new_process);
    let session_global: PMM_SESSION_SPACE;
    let mut old_irql: KIRQL;

    if ((*(PsGetCurrentProcess() as *mut _EPROCESS)).__bindgen_anon_2.Flags & 0x00010000) == 0 {
        return;
    }

    session_global = (*(MmSessionSpace as *mut _MM_SESSION_SPACE)).GlobalVirtualAddress as PMM_SESSION_SPACE;

    InterlockedIncrement(core::ptr::addr_of_mut!((*session_global).ReferenceCount) as *mut LONG);
    InterlockedIncrement(core::ptr::addr_of_mut!((*session_global).ResidentProcessCount) as *mut LONG);
    InterlockedIncrement(core::ptr::addr_of_mut!((*session_global).ProcessReferenceToSession) as *mut LONG);

    (*(new_process as *mut _EPROCESS)).Session = session_global as PVOID;

    old_irql = MiAcquireExpansionLock();

    InsertTailList(
        core::ptr::addr_of_mut!((*session_global).ProcessList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*(new_process as *mut _EPROCESS)).SessionProcessLinks)
            as PLIST_ENTRY,
    );

    MiReleaseExpansionLock(old_irql);

    let flags_ptr = core::ptr::addr_of_mut!((*(new_process as *mut _EPROCESS)).__bindgen_anon_2.Flags);
    *flags_ptr |= 0x00010000u32;
}

unsafe fn MiSessionInitializeWorkingSetList() -> NTSTATUS {
    let mut old_irql: KIRQL;
    let mut pointer_pte: PMMPTE;
    let mut pointer_pde: PMMPTE;
    let mut temp_pte: MMPTE = core::mem::zeroed();
    let mut temp_pde: MMPTE = core::mem::zeroed();
    let mut color: ULONG;
    let mut index: ULONG;
    let mut page_frame_index: PFN_NUMBER;
    let session_global: PMM_SESSION_SPACE;
    let allocated_page_table: BOOLEAN;
    let working_set_list: PMMWSL;

    session_global = (*(MmSessionSpace as *mut _MM_SESSION_SPACE)).GlobalVirtualAddress as PMM_SESSION_SPACE;
    working_set_list = MiSessionSpaceWs as PMMWSL;

    (*MmSessionSpace).Vm.VmWorkingSetList = working_set_list;
    (*MmSessionSpace).Wsle = (working_set_list as usize
        + core::mem::size_of::<_MMWSL>()
        + core::mem::size_of::<PVOID>()) as PMMWSLE;

    pointer_pde = MiAddressToPde(working_set_list as PVOID);
    if (*pointer_pde).valid() {
        allocated_page_table = 0u8;
    } else {
        allocated_page_table = 1u8;
    }

    pointer_pte = MiAddressToPte(working_set_list as PVOID);

    ExInitializePushLock(core::ptr::addr_of_mut!((*session_global).Vm.WorkingSetMutex) as *mut _);
    old_irql = MiAcquirePfnLock();

    if allocated_page_table != 0 {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_INIT_MEMORY);
        color = MI_GET_NEXT_COLOR();
        page_frame_index = (MiRemoveZeroPageSafe(color)) as usize;
        if page_frame_index == 0 {
            page_frame_index = MiRemoveAnyPage(color);

            MiReleasePfnLock(old_irql);
            MiZeroPhysicalPage(page_frame_index);
            old_irql = MiAcquirePfnLock();
        }

        temp_pde = ValidKernelPdeLocal;
        temp_pde.set_page_frame_number((page_frame_index) as PFN_NUMBER);
        MI_WRITE_VALID_PDE(pointer_pde, temp_pde);

        index = ((working_set_list as usize - MmSessionBase as usize) >> 22) as ULONG;
        #[cfg(not(target_pointer_width = "64"))]
        {
            *((*MmSessionSpace).PageTables.add(index as usize)) = core::mem::transmute(temp_pde);
        }

        MiInitializePfnForOtherProcess(
            (page_frame_index) as _,
            pointer_pde as PVOID,
            (*MmSessionSpace).SessionPageDirectoryIndex as _,
        );
        KeZeroPages(pointer_pte as PVOID, (PAGE_SIZE as usize) as u32);
    }

    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_INIT_MEMORY);
    color = MI_GET_NEXT_COLOR();
    page_frame_index = (MiRemoveZeroPageSafe(color)) as usize;
    if page_frame_index == 0 {
        page_frame_index = MiRemoveAnyPage(color);

        MiReleasePfnLock(old_irql);
        MiZeroPhysicalPage(page_frame_index);
        old_irql = MiAcquirePfnLock();
    }

    temp_pte = ValidKernelPteLocal;
    MI_MAKE_DIRTY_PAGE(core::ptr::addr_of_mut!(temp_pte));
    temp_pte.set_page_frame_number((page_frame_index) as PFN_NUMBER);

    MiInitializePfnAndMakePteValid((page_frame_index) as _, pointer_pte as *mut _MMPTE, core::mem::transmute(temp_pte));

    MiReleasePfnLock(old_irql);

    (*MmSessionSpace).Vm.Flags.set_SessionSpace(1);
    (*MmSessionSpace).Vm.MinimumWorkingSetSize = 20;
    (*MmSessionSpace).Vm.MaximumWorkingSetSize = 384;
    (*working_set_list).LastEntry = 20;
    (*working_set_list).HashTable = core::ptr::null_mut();
    (*working_set_list).HashTableSize = 0;
    (*working_set_list).Wsle = (*MmSessionSpace).Wsle;

    old_irql = MiAcquireExpansionLock();

    InsertTailList(
        core::ptr::addr_of_mut!(MiSessionWsList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*session_global).WsListEntry) as PLIST_ENTRY,
    );

    InsertTailList(
        core::ptr::addr_of_mut!(MmWorkingSetExpansionHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*session_global).Vm.WorkingSetExpansionLinks) as PLIST_ENTRY,
    );

    MiReleaseExpansionLock(old_irql);

    STATUS_SUCCESS
}

unsafe fn MiSessionCreateInternal(session_id: *mut ULONG) -> NTSTATUS {
let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let mut new_flags: ULONG;
    let mut flags: ULONG;
    let mut i: ULONG;
    let mut color: ULONG;
    #[cfg(not(target_pointer_width = "64"))]
    let size: ULONG;
    #[cfg(not(target_pointer_width = "64"))]
    let mut page_tables: PMMPTE = core::ptr::null_mut();
    let mut old_irql: KIRQL;
    let mut pointer_pte: PMMPTE;
    let mut session_pte: PMMPTE;
    let mut pointer_pde: PMMPTE;
    let session_global: PMM_SESSION_SPACE;
    let mut temp_pte: MMPTE = core::mem::zeroed();
    let mut temp_pde: MMPTE = core::mem::zeroed();
    let status: NTSTATUS;
    let result: BOOLEAN;
    let mut session_page_dir_index: PFN_NUMBER;
    let mut tag_page: [PFN_NUMBER; MI_SESSION_TAG_PAGES_MAXIMUM as usize] =
        [0; MI_SESSION_TAG_PAGES_MAXIMUM as usize];
    let mut data_page: [PFN_NUMBER; MI_SESSION_DATA_PAGES_MAXIMUM as usize] =
        [0; MI_SESSION_DATA_PAGES_MAXIMUM as usize];

    // C: `Flags & PSF_SESSION_CREATION_UNDERWAY_BIT` / `Flags | ...`, 0x4000.
    // Both halves were written as 0x200, which is PSF_WOW64_VA_SPACE_4GB_BIT —
    // so this did not just test the wrong bit, it SET the wrong one on the
    // EPROCESS, marking every process that creates a session as a 4GB-VA WOW64
    // process. The test and the set agreed with each other, which is exactly
    // why nothing downstream ever noticed.
    flags = (*process).__bindgen_anon_2.Flags;
    loop {
        if (flags & PSF_SESSION_CREATION_UNDERWAY_BIT) != 0 {
            return STATUS_ALREADY_COMMITTED;
        }

        new_flags = InterlockedCompareExchange(
            &mut (*process).__bindgen_anon_2.Flags as *mut ULONG as *mut LONG,
            (flags | PSF_SESSION_CREATION_UNDERWAY_BIT) as LONG,
            flags as LONG,
        ) as ULONG;
        if new_flags == flags {
            break;
        }

        flags = new_flags;
    }

    #[cfg(not(target_pointer_width = "64"))]
    {
        size = ((0x20000000 / PDE_MAPPED_VA) * core::mem::size_of::<_MMPTE>()) as ULONG;
        page_tables = ExAllocatePoolWithTag(NonPagedPool, size as usize, u32::from_be_bytes(*b"tHmM"))
            as PMMPDE;
        RtlZeroMemory(page_tables as PVOID, size as usize);
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(MiSessionIdMutex) as *mut KGUARDED_MUTEX);

    *session_id = RtlFindClearBitsAndSet(MiSessionIdBitmap, 1, 0);
    if *session_id == 0xFFFFFFFF {
        #[cfg(not(target_pointer_width = "64"))]
        ExFreePoolWithTag(page_tables as PVOID, u32::from_be_bytes(*b"tHmM"));
        return STATUS_NO_MEMORY;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(MiSessionIdMutex) as *mut KGUARDED_MUTEX);

    session_pte = MiReserveSystemPtes(MiSessionDataPages as ULONG, SystemPteSpace);

    old_irql = MiAcquirePfnLock();

    temp_pte = ValidKernelPte;
    i = 0;
    while i < MiSessionDataPages as ULONG {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_INIT_MEMORY);
        color = MI_GET_NEXT_COLOR();
        data_page[i as usize] = (MiRemoveZeroPageSafe(color)) as usize;
        if data_page[i as usize] == 0 {
            data_page[i as usize] = MiRemoveAnyPage(color);

            MiReleasePfnLock(old_irql);
            MiZeroPhysicalPage(data_page[i as usize]);
            old_irql = MiAcquirePfnLock();
        }

        temp_pte.set_page_frame_number((data_page[i as usize]) as PFN_NUMBER);
        MI_WRITE_VALID_PTE(session_pte.add(i as usize), temp_pte);
        i += 1;
    }

    session_global = MiPteToAddress(session_pte as PVOID as *mut MMPTE) as PMM_SESSION_SPACE;

    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_INIT_MEMORY);
    color = MI_GET_NEXT_COLOR();
    session_page_dir_index = (MiRemoveZeroPageSafe(color)) as usize;
    if session_page_dir_index == 0 {
        session_page_dir_index = MiRemoveAnyPage(color);

        MiReleasePfnLock(old_irql);
        MiZeroPhysicalPage(session_page_dir_index);
        old_irql = MiAcquirePfnLock();
    }

    temp_pde = ValidKernelPdeLocal;
    temp_pde.set_page_frame_number((session_page_dir_index) as PFN_NUMBER);

    pointer_pde = MiAddressToPde(MmSessionSpace as PVOID);
    MI_WRITE_VALID_PDE(pointer_pde, temp_pde);
    MiInitializePfnForOtherProcess((session_page_dir_index) as _, pointer_pde as PVOID, session_page_dir_index as _);

    temp_pte = ValidKernelPteLocal;
    MI_MAKE_WRITE_PAGE(core::ptr::addr_of_mut!(temp_pte));
    MI_MAKE_DIRTY_PAGE(core::ptr::addr_of_mut!(temp_pte));
    pointer_pte = MiAddressToPte(MmSessionSpace as PVOID);
    i = 0;
    while i < MiSessionDataPages as ULONG {
        temp_pte.set_page_frame_number((data_page[i as usize]) as PFN_NUMBER);
        MiInitializePfnAndMakePteValid((data_page[i as usize]) as _, pointer_pte.add(i as usize) as *mut _MMPTE, core::mem::transmute(temp_pte));
        i += 1;
    }

    i = 0;
    while i < MiSessionTagPages as ULONG {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_INIT_MEMORY);
        color = MI_GET_NEXT_COLOR();
        tag_page[i as usize] = (MiRemoveZeroPageSafe(color)) as usize;
        if tag_page[i as usize] == 0 {
            tag_page[i as usize] = MiRemoveAnyPage(color);

            MiReleasePfnLock(old_irql);
            MiZeroPhysicalPage(tag_page[i as usize]);
            old_irql = MiAcquirePfnLock();
        }

        temp_pte.set_page_frame_number((tag_page[i as usize]) as PFN_NUMBER);
        MiInitializePfnAndMakePteValid(
            (tag_page[i as usize]) as _,
            pointer_pte.add((MiSessionDataPages as ULONG + i) as usize) as *mut _MMPTE,
            core::mem::transmute(temp_pte),
        );
        i += 1;
    }

    MiReleasePfnLock(old_irql);

    (*MmSessionSpace).GlobalVirtualAddress = session_global;
    (*MmSessionSpace).ReferenceCount = 1;
    (*MmSessionSpace).ResidentProcessCount = 1;
    (*MmSessionSpace).u.LongFlags = 0;
    (*MmSessionSpace).SessionId = *session_id;
    (*MmSessionSpace).LocaleId = PsDefaultSystemLocaleId;
    (*MmSessionSpace).SessionPageDirectoryIndex = (session_page_dir_index) as _;
    (*MmSessionSpace).Color = color;
    (*MmSessionSpace).NonPageablePages = (MiSessionCreateCharge as ULONG) as _;
    (*MmSessionSpace).CommittedPages = (MiSessionCreateCharge as ULONG) as _;
    #[cfg(not(target_pointer_width = "64"))]
    {
        (*MmSessionSpace).PageTables = page_tables as *mut _MMPTE;
        *((*MmSessionSpace)
            .PageTables
            .add((pointer_pde as usize - MiAddressToPde(MmSessionBase as PVOID) as usize)
                / core::mem::size_of::<MMPTE>())) = core::mem::transmute(*pointer_pde);
    }
    InitializeListHead(core::ptr::addr_of_mut!((*MmSessionSpace).ImageList) as PLIST_ENTRY);

    status = STATUS_SUCCESS;

    result = MiInitializeSystemSpaceMap(
        core::ptr::addr_of_mut!((*session_global).Session) as PMMSESSION
    );

    InitializeListHead(core::ptr::addr_of_mut!((*session_global).ProcessList) as PLIST_ENTRY);

    let flags_ptr = &mut (*process).__bindgen_anon_2.Flags as *mut ULONG;
    *flags_ptr &= !0x00000200u32;

    (*session_global).ProcessReferenceToSession = 1;

    InterlockedIncrement(core::ptr::addr_of_mut!(MmSessionDataPages));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmSessionCreate(session_id: *mut ULONG) -> NTSTATUS {
    let session_id = opaque_ptr(session_id);
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let session_leader_exists: LONG;
    let mut status: NTSTATUS;

    if ((*process).__bindgen_anon_2.Flags & 0x00010000) != 0 {
        return STATUS_ALREADY_COMMITTED;
    }

    if (*process).Vm.Flags.SessionLeader() == 0 {
        session_leader_exists = InterlockedCompareExchange(
            core::ptr::addr_of_mut!(MiSessionLeaderExists),
            1,
            0,
        );
        if session_leader_exists != 0 {
            return STATUS_INVALID_SYSTEM_SERVICE;
        }

        MiSessionLeader(process as PEPROCESS);
    }

    KeEnterCriticalRegion();
    status = MiSessionCreateInternal(session_id);
    if status < 0 {
        KeLeaveCriticalRegion();
        return status;
    }

    status = MiSessionInitializeWorkingSetList();
    if status < 0 {
        KeLeaveCriticalRegion();
        return status;
    }

    KeLeaveCriticalRegion();

    (*MmSessionSpace).u.Flags.set_Initialized(1);
    (*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags |= 0x00010000u32;
    status
}

#[no_mangle]
pub unsafe extern "system" fn MmSessionDelete(session_id: ULONG) -> NTSTATUS {
    let process = PsGetCurrentProcess() as *mut _EPROCESS;

    if ((*process).__bindgen_anon_2.Flags & 0x00010000) == 0 {
        return STATUS_UNABLE_TO_FREE_VM;
    }

    if (*process).Vm.Flags.SessionLeader() == 0 {
        return STATUS_UNABLE_TO_FREE_VM;
    }

    KeEnterCriticalRegion();
    MiDereferenceSession();
    KeLeaveCriticalRegion();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmAttachSession(
    session_entry: PVOID,
    apc_state: PKAPC_STATE,
) -> NTSTATUS {
    let session_entry = opaque_ptr(session_entry);
    let mut entry_process: PEPROCESS;
    let entry_session: PMM_SESSION_SPACE;
    let current_session: PMM_SESSION_SPACE;
    let current_process: PEPROCESS;
    let mut old_irql: KIRQL;

    entry_process = session_entry as PEPROCESS;

    entry_session = (*(entry_process as *mut _EPROCESS)).Session as PMM_SESSION_SPACE;

    current_process = PsGetCurrentProcess() as *mut _EPROCESS;
    current_session = (*(current_process as *mut _EPROCESS)).Session as PMM_SESSION_SPACE;

    old_irql = MiAcquireExpansionLock();

    if (*entry_session).u.Flags.DeletePending() != 0 {
        MiReleaseExpansionLock(old_irql);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    (*entry_session).AttachCount += 1;

    MiReleaseExpansionLock(old_irql);

    if (*(current_process as *mut _EPROCESS)).Vm.Flags.SessionLeader() == 0
        && !current_session.is_null()
    {
        if current_session == entry_session {
            entry_process = current_process;
        }
    }

    KeStackAttachProcess(
        core::ptr::addr_of_mut!((*(entry_process as *mut _EPROCESS)).Pcb) as PVOID as *mut _KPROCESS,
        apc_state,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmDetachSession(session_entry: PVOID, apc_state: PKAPC_STATE) {
    let session_entry = opaque_ptr(session_entry);
    let entry_process: PEPROCESS;
    let entry_session: PMM_SESSION_SPACE;
    let mut old_irql: KIRQL;
    let delete_pending: BOOLEAN;

    entry_process = session_entry as PEPROCESS;

    entry_session = (*(entry_process as *mut _EPROCESS)).Session as PMM_SESSION_SPACE;

    old_irql = MiAcquireExpansionLock();

    (*entry_session).AttachCount -= 1;

    delete_pending = if (*entry_session).u.Flags.DeletePending() != 0
        && (*entry_session).AttachCount == 0
    {
        1u8
    } else {
        0u8
    };

    MiReleaseExpansionLock(old_irql);

    KeUnstackDetachProcess(apc_state);

    if delete_pending != 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*entry_session).AttachEvent) as PVOID,
            (IO_NO_INCREMENT as i8) as i32,
            0u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmQuitNextSession(session_entry: PVOID) {
    let session_entry = opaque_ptr(session_entry);
    let entry_process: PEPROCESS;

    entry_process = session_entry as PEPROCESS;

    ObDereferenceObject(entry_process as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSessionById(session_id: ULONG) -> PVOID {
    let mut list_entry: PLIST_ENTRY;
    let mut session: PMM_SESSION_SPACE;
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut old_irql: KIRQL;

    old_irql = MiAcquireExpansionLock();

    list_entry = (*core::ptr::addr_of!(MiSessionWsList)).Flink;
    while list_entry != core::ptr::addr_of_mut!(MiSessionWsList) as PLIST_ENTRY {
        session = CONTAINING_RECORD!(list_entry, _MM_SESSION_SPACE, WsListEntry) as PMM_SESSION_SPACE;
        list_entry = (*list_entry).Flink;

        if (*session).SessionId == session_id {
            if !IsListEmpty(core::ptr::addr_of_mut!((*session).ProcessList) as PLIST_ENTRY) {
                process = CONTAINING_RECORD!((*core::ptr::addr_of!((*session).ProcessList)).Flink, _EPROCESS, SessionProcessLinks) as PEPROCESS;

                ObReferenceObject(process as PVOID);
                break;
            }
        }
    }

    MiReleaseExpansionLock(old_irql);

    process as PVOID
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/session.c
//  * PURPOSE:         Session support routines
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Timo Kreuzer (timo.kreuzer@reactos.org)
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
// PMM_SESSION_SPACE MmSessionSpace;
// PFN_NUMBER MiSessionDataPages, MiSessionTagPages, MiSessionTagSizePages;
// PFN_NUMBER MiSessionBigPoolPages, MiSessionCreateCharge;
// KGUARDED_MUTEX MiSessionIdMutex;
// LONG MmSessionDataPages;
// PRTL_BITMAP MiSessionIdBitmap;
// volatile LONG MiSessionLeaderExists;
//
// LIST_ENTRY MiSessionWsList;
// LIST_ENTRY MmWorkingSetExpansionHead;
//
// KSPIN_LOCK MmExpansionLock;
// PETHREAD MiExpansionLockOwner;
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// NTAPI
// MiInitializeSessionWsSupport(VOID)
// {
//     /* Initialize the list heads */
//     InitializeListHead(&MiSessionWsList);
// }
//
// BOOLEAN
// NTAPI
// MmIsSessionAddress(IN PVOID Address)
// {
//     /* Check if it is in range */
//     return MI_IS_SESSION_ADDRESS(Address) ? TRUE : FALSE;
// }
//
// LCID
// NTAPI
// MmGetSessionLocaleId(VOID)
// {
//     PEPROCESS Process;
//     PAGED_CODE();
//
//     //
//     // Get the current process
//     //
//     Process = PsGetCurrentProcess();
//
//     //
//     // Check if it's NOT the Session Leader
//     //
//     if (!Process->Vm.Flags.SessionLeader)
//     {
//         //
//         // Make sure it has a valid Session
//         //
//         if (Process->Session)
//         {
//             //
//             // Get the Locale ID
//             //
//             return ((PMM_SESSION_SPACE)Process->Session)->LocaleId;
//         }
//     }
//
//     //
//     // Not a session leader, return the default
//     //
//     return PsDefaultThreadLocaleId;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// NTAPI
// MmSetSessionLocaleId(
//     _In_ LCID LocaleId)
// {
//     PEPROCESS CurrentProcess;
//     PAGED_CODE();
//
//     /* Get the current process and check if it is in a session */
//     CurrentProcess = PsGetCurrentProcess();
//     if ((CurrentProcess->Vm.Flags.SessionLeader == 0) &&
//         (CurrentProcess->Session != NULL))
//     {
//         /* Set the session locale Id */
//         ((PMM_SESSION_SPACE)CurrentProcess->Session)->LocaleId = LocaleId;
//     }
//     else
//     {
//         /* Set the default locale */
//         PsDefaultThreadLocaleId = LocaleId;
//     }
// }
//
//
// VOID
// NTAPI
// MiInitializeSessionIds(VOID)
// {
//     ULONG Size, BitmapSize;
//     PFN_NUMBER TotalPages;
//
//     /* Setup the total number of data pages needed for the structure */
//     TotalPages = MI_SESSION_DATA_PAGES_MAXIMUM;
//     MiSessionDataPages = ROUND_TO_PAGES(sizeof(MM_SESSION_SPACE)) >> PAGE_SHIFT;
//     ASSERT(MiSessionDataPages <= MI_SESSION_DATA_PAGES_MAXIMUM - 3);
//     TotalPages -= MiSessionDataPages;
//
//     /* Setup the number of pages needed for session pool tags */
//     MiSessionTagSizePages = 2;
//     MiSessionBigPoolPages = 1;
//     MiSessionTagPages = MiSessionTagSizePages + MiSessionBigPoolPages;
//     ASSERT(MiSessionTagPages <= TotalPages);
//     ASSERT(MiSessionTagPages < MI_SESSION_TAG_PAGES_MAXIMUM);
//
//     /* Total pages needed for a session (FIXME: Probably different on PAE/x64) */
//     MiSessionCreateCharge = 1 + MiSessionDataPages + MiSessionTagPages;
//
//     /* Initialize the lock */
//     KeInitializeGuardedMutex(&MiSessionIdMutex);
//
//     /* Allocate the bitmap */
//     Size = MI_INITIAL_SESSION_IDS;
//     BitmapSize = ((Size + 31) / 32) * sizeof(ULONG);
//     MiSessionIdBitmap = ExAllocatePoolWithTag(PagedPool,
//                                               sizeof(RTL_BITMAP) + BitmapSize,
//                                               TAG_MM);
//     if (MiSessionIdBitmap)
//     {
//         /* Free all the bits */
//         RtlInitializeBitMap(MiSessionIdBitmap,
//                             (PVOID)(MiSessionIdBitmap + 1),
//                             Size);
//         RtlClearAllBits(MiSessionIdBitmap);
//     }
//     else
//     {
//         /* Die if we couldn't allocate the bitmap */
//         KeBugCheckEx(INSTALL_MORE_MEMORY,
//                      MmNumberOfPhysicalPages,
//                      MmLowestPhysicalPage,
//                      MmHighestPhysicalPage,
//                      0x200);
//     }
// }
//
// VOID
// NTAPI
// MiSessionLeader(IN PEPROCESS Process)
// {
//     KIRQL OldIrql;
//
//     /* Set the flag while under the expansion lock */
//     OldIrql = MiAcquireExpansionLock();
//     Process->Vm.Flags.SessionLeader = TRUE;
//     MiReleaseExpansionLock(OldIrql);
// }
//
// ULONG
// NTAPI
// MmGetSessionId(IN PEPROCESS Process)
// {
//     PMM_SESSION_SPACE SessionGlobal;
//
//     /* The session leader is always session zero */
//     if (Process->Vm.Flags.SessionLeader == 1) return 0;
//
//     /* Otherwise, get the session global, and read the session ID from it */
//     SessionGlobal = (PMM_SESSION_SPACE)Process->Session;
//     if (!SessionGlobal) return 0;
//     return SessionGlobal->SessionId;
// }
//
// ULONG
// NTAPI
// MmGetSessionIdEx(IN PEPROCESS Process)
// {
//     PMM_SESSION_SPACE SessionGlobal;
//
//     /* The session leader is always session zero */
//     if (Process->Vm.Flags.SessionLeader == 1) return 0;
//
//     /* Otherwise, get the session global, and read the session ID from it */
//     SessionGlobal = (PMM_SESSION_SPACE)Process->Session;
//     if (!SessionGlobal) return -1;
//     return SessionGlobal->SessionId;
// }
//
// VOID
// NTAPI
// MiReleaseProcessReferenceToSessionDataPage(IN PMM_SESSION_SPACE SessionGlobal)
// {
//     ULONG i, SessionId;
//     PMMPTE PointerPte;
//     PFN_NUMBER PageFrameIndex[MI_SESSION_DATA_PAGES_MAXIMUM];
//     PMMPFN Pfn1;
//     KIRQL OldIrql;
//
//     /* Is there more than just this reference? If so, bail out */
//     if (InterlockedDecrement(&SessionGlobal->ProcessReferenceToSession)) return;
//
//     /* Get the session ID */
//     SessionId = SessionGlobal->SessionId;
//     DPRINT1("Last process in session %lu going down!!!\n", SessionId);
//
//     /* Free the session page tables */
// #ifndef _M_AMD64
//     ExFreePoolWithTag(SessionGlobal->PageTables, 'tHmM');
// #endif
//     ASSERT(!MI_IS_PHYSICAL_ADDRESS(SessionGlobal));
//
//     /* Capture the data page PFNs */
//     PointerPte = MiAddressToPte(SessionGlobal);
//     for (i = 0; i < MiSessionDataPages; i++)
//     {
//         PageFrameIndex[i] = PFN_FROM_PTE(PointerPte + i);
//     }
//
//     /* Release them */
//     MiReleaseSystemPtes(PointerPte, MiSessionDataPages, SystemPteSpace);
//
//     /* Mark them as deleted */
//     for (i = 0; i < MiSessionDataPages; i++)
//     {
//         Pfn1 = MI_PFN_ELEMENT(PageFrameIndex[i]);
//         MI_SET_PFN_DELETED(Pfn1);
//     }
//
//     /* Loop every data page and drop a reference count */
//     OldIrql = MiAcquirePfnLock();
//     for (i = 0; i < MiSessionDataPages; i++)
//     {
//         /* Sanity check that the page is correct, then decrement it */
//         Pfn1 = MI_PFN_ELEMENT(PageFrameIndex[i]);
//         ASSERT(Pfn1->u2.ShareCount == 1);
//         ASSERT(Pfn1->u3.e2.ReferenceCount == 1);
//         MiDecrementShareCount(Pfn1, PageFrameIndex[i]);
//     }
//
//     /* Done playing with pages, release the lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Decrement the number of data pages */
//     InterlockedDecrement(&MmSessionDataPages);
//
//     /* Free this session ID from the session bitmap */
//     KeAcquireGuardedMutex(&MiSessionIdMutex);
//     ASSERT(RtlCheckBit(MiSessionIdBitmap, SessionId));
//     RtlClearBit(MiSessionIdBitmap, SessionId);
//     KeReleaseGuardedMutex(&MiSessionIdMutex);
// }
//
// VOID
// NTAPI
// MiDereferenceSessionFinal(VOID)
// {
//     PMM_SESSION_SPACE SessionGlobal;
//     KIRQL OldIrql;
//
//     /* Get the pointer to the global session address */
//     SessionGlobal = MmSessionSpace->GlobalVirtualAddress;
//
//     /* Acquire the expansion lock */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Set delete pending flag, so that processes can no longer attach to this
//        session and the last process that detaches sets the AttachEvent */
//     ASSERT(SessionGlobal->u.Flags.DeletePending == 0);
//     SessionGlobal->u.Flags.DeletePending = 1;
//
//     /* Check if we have any attached processes */
//     if (SessionGlobal->AttachCount)
//     {
//         /* Initialize the event (it's not in use yet!) */
//         KeInitializeEvent(&SessionGlobal->AttachEvent, NotificationEvent, FALSE);
//
//         /* Release the expansion lock for the wait */
//         MiReleaseExpansionLock(OldIrql);
//
//         /* Wait for the event to be set due to the last process detach */
//         KeWaitForSingleObject(&SessionGlobal->AttachEvent, WrVirtualMemory, 0, 0, 0);
//
//         /* Reacquire the expansion lock */
//         OldIrql = MiAcquireExpansionLock();
//
//         /* Makes sure we still have the delete flag and no attached processes */
//         ASSERT(MmSessionSpace->u.Flags.DeletePending == 1);
//         ASSERT(MmSessionSpace->AttachCount == 0);
//     }
//
//     /* Check if the session is in the workingset expansion list */
//     if (SessionGlobal->Vm.WorkingSetExpansionLinks.Flink != NULL)
//     {
//         /* Remove the session from the list and zero the list entry */
//         RemoveEntryList(&SessionGlobal->Vm.WorkingSetExpansionLinks);
//         SessionGlobal->Vm.WorkingSetExpansionLinks.Flink = 0;
//     }
//
//     /* Check if the session is in the workingset list */
//     if (SessionGlobal->WsListEntry.Flink)
//     {
//         /* Remove the session from the list and zero the list entry */
//         RemoveEntryList(&SessionGlobal->WsListEntry);
//         SessionGlobal->WsListEntry.Flink = NULL;
//     }
//
//     /* Release the expansion lock */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Check for a win32k unload routine */
//     if (SessionGlobal->Win32KDriverUnload)
//     {
//         /* Call it */
//         SessionGlobal->Win32KDriverUnload(NULL);
//     }
// }
//
//
// VOID
// NTAPI
// MiDereferenceSession(VOID)
// {
//     PMM_SESSION_SPACE SessionGlobal;
//     PEPROCESS Process;
//     ULONG ReferenceCount, SessionId;
//
//     /* Sanity checks */
//     ASSERT(PsGetCurrentProcess()->ProcessInSession ||
//            ((MmSessionSpace->u.Flags.Initialized == 0) &&
//             (PsGetCurrentProcess()->Vm.Flags.SessionLeader == 1) &&
//             (MmSessionSpace->ReferenceCount == 1)));
//
//     /* The session bit must be set */
//     SessionId = MmSessionSpace->SessionId;
//     ASSERT(RtlCheckBit(MiSessionIdBitmap, SessionId));
//
//     /* Get the current process */
//     Process = PsGetCurrentProcess();
//
//     /* Decrement the process count */
//     InterlockedDecrement(&MmSessionSpace->ResidentProcessCount);
//
//     /* Decrement the reference count and check if was the last reference */
//     ReferenceCount = InterlockedDecrement(&MmSessionSpace->ReferenceCount);
//     if (ReferenceCount == 0)
//     {
//         /* No more references left, kill the session completely */
//         MiDereferenceSessionFinal();
//         return;
//     }
//
//     /* Check if this is the session leader */
//     if (Process->Vm.Flags.SessionLeader)
//     {
//         /* Get the global session address before we kill the session mapping */
//         SessionGlobal = MmSessionSpace->GlobalVirtualAddress;
//
//         /* Delete all session PDEs and flush the TB */
//         //RtlZeroMemory(MiAddressToPde(MmSessionBase),
//         //              BYTES_TO_PAGES(MmSessionSize) * sizeof(MMPDE));
//         KeFlushEntireTb(FALSE, FALSE);
//
//         /* Clean up the references here. */
//         ASSERT(Process->Session == NULL);
//         MiReleaseProcessReferenceToSessionDataPage(SessionGlobal);
//     }
//
//     /* Reset the current process' session flag */
//     RtlInterlockedClearBits(&Process->Flags, PSF_PROCESS_IN_SESSION_BIT);
// }
//
// VOID
// NTAPI
// MiSessionRemoveProcess(VOID)
// {
//     PEPROCESS CurrentProcess = PsGetCurrentProcess();
//     KIRQL OldIrql;
//
//     /* If the process isn't already in a session, or if it's the leader... */
//     if (!(CurrentProcess->Flags & PSF_PROCESS_IN_SESSION_BIT) ||
//         (CurrentProcess->Vm.Flags.SessionLeader))
//     {
//         /* Then there's nothing to do */
//         return;
//     }
//
//     /* Sanity check */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == TRUE);
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Remove the process from the list */
//     RemoveEntryList(&CurrentProcess->SessionProcessLinks);
//
//     /* Release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Dereference the session */
//     MiDereferenceSession();
// }
//
// VOID
// NTAPI
// MiSessionAddProcess(IN PEPROCESS NewProcess)
// {
//     PMM_SESSION_SPACE SessionGlobal;
//     KIRQL OldIrql;
//
//     /* The current process must already be in a session */
//     if (!(PsGetCurrentProcess()->Flags & PSF_PROCESS_IN_SESSION_BIT)) return;
//
//     /* Sanity check */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == TRUE);
//
//     /* Get the global session */
//     SessionGlobal = MmSessionSpace->GlobalVirtualAddress;
//
//     /* Increment counters */
//     InterlockedIncrement((PLONG)&SessionGlobal->ReferenceCount);
//     InterlockedIncrement(&SessionGlobal->ResidentProcessCount);
//     InterlockedIncrement(&SessionGlobal->ProcessReferenceToSession);
//
//     /* Set the session pointer */
//     ASSERT(NewProcess->Session == NULL);
//     NewProcess->Session = SessionGlobal;
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Insert it into the process list */
//     InsertTailList(&SessionGlobal->ProcessList, &NewProcess->SessionProcessLinks);
//
//     /* Release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Set the flag */
//     PspSetProcessFlag(NewProcess, PSF_PROCESS_IN_SESSION_BIT);
// }
//
// NTSTATUS
// NTAPI
// MiSessionInitializeWorkingSetList(VOID)
// {
//     KIRQL OldIrql;
//     PMMPTE PointerPte;
//     PMMPDE PointerPde;
//     MMPTE TempPte;
//     MMPDE TempPde;
//     ULONG Color, Index;
//     PFN_NUMBER PageFrameIndex;
//     PMM_SESSION_SPACE SessionGlobal;
//     BOOLEAN AllocatedPageTable;
//     PMMWSL WorkingSetList;
//
//     /* Get pointers to session global and the session working set list */
//     SessionGlobal = MmSessionSpace->GlobalVirtualAddress;
//     WorkingSetList = (PMMWSL)MiSessionSpaceWs;
//
//     /* Fill out the two pointers */
//     MmSessionSpace->Vm.VmWorkingSetList = WorkingSetList;
//     MmSessionSpace->Wsle = (PMMWSLE)((&WorkingSetList->VadBitMapHint) + 1);
//
//     /* Get the PDE for the working set, and check if it's already allocated */
//     PointerPde = MiAddressToPde(WorkingSetList);
//     if (PointerPde->u.Hard.Valid == 1)
//     {
//         /* Nope, we'll have to do it */
// #if !defined(_M_ARM) && !defined(_M_ARM64)
//         ASSERT(PointerPde->u.Hard.Global == 0);
// #endif
//         AllocatedPageTable = FALSE;
//     }
//     else
//     {
//         /* Yep, that makes our job easier */
//         AllocatedPageTable = TRUE;
//     }
//
//     /* Get the PTE for the working set */
//     PointerPte = MiAddressToPte(WorkingSetList);
//
//     /* Initialize the working set lock, and lock the PFN database */
//     ExInitializePushLock(&SessionGlobal->Vm.WorkingSetMutex);
//     //MmLockPageableSectionByHandle(ExPageLockHandle);
//     OldIrql = MiAcquirePfnLock();
//
//     /* Check if we need a page table */
//     if (AllocatedPageTable != FALSE)
//     {
//         /* Get a zeroed colored zero page */
//         MI_SET_USAGE(MI_USAGE_INIT_MEMORY);
//         Color = MI_GET_NEXT_COLOR();
//         PageFrameIndex = MiRemoveZeroPageSafe(Color);
//         if (!PageFrameIndex)
//         {
//             /* No zero pages, grab a free one */
//             PageFrameIndex = MiRemoveAnyPage(Color);
//
//             /* Zero it outside the PFN lock */
//             MiReleasePfnLock(OldIrql);
//             MiZeroPhysicalPage(PageFrameIndex);
//             OldIrql = MiAcquirePfnLock();
//         }
//
//         /* Write a valid PDE for it */
//         TempPde = ValidKernelPdeLocal;
//         TempPde.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
//         MI_WRITE_VALID_PDE(PointerPde, TempPde);
//
//         /* Add this into the list */
//         Index = ((ULONG_PTR)WorkingSetList - (ULONG_PTR)MmSessionBase) >> 22;
// #if (_MI_PAGING_LEVELS < 3)
//         MmSessionSpace->PageTables[Index] = TempPde;
// #endif
//         /* Initialize the page directory page, and now zero the working set list itself */
//         MiInitializePfnForOtherProcess(PageFrameIndex,
//                                        PointerPde,
//                                        MmSessionSpace->SessionPageDirectoryIndex);
//         KeZeroPages(PointerPte, PAGE_SIZE);
//     }
//
//     /* Get a zeroed colored zero page */
//     MI_SET_USAGE(MI_USAGE_INIT_MEMORY);
//     Color = MI_GET_NEXT_COLOR();
//     PageFrameIndex = MiRemoveZeroPageSafe(Color);
//     if (!PageFrameIndex)
//     {
//         /* No zero pages, grab a free one */
//         PageFrameIndex = MiRemoveAnyPage(Color);
//
//         /* Zero it outside the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         MiZeroPhysicalPage(PageFrameIndex);
//         OldIrql = MiAcquirePfnLock();
//     }
//
//     /* Write a valid PTE for it */
//     TempPte = ValidKernelPteLocal;
//     MI_MAKE_DIRTY_PAGE(&TempPte);
//     TempPte.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
//
//     /* Initialize the working set list page */
//     MiInitializePfnAndMakePteValid(PageFrameIndex, PointerPte as *mut _MMPTE, TempPte);
//
//     /* Now we can release the PFN database lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Fill out the working set structure */
//     MmSessionSpace->Vm.Flags.SessionSpace = 1;
//     MmSessionSpace->Vm.MinimumWorkingSetSize = 20;
//     MmSessionSpace->Vm.MaximumWorkingSetSize = 384;
//     WorkingSetList->LastEntry = 20;
//     WorkingSetList->HashTable = NULL;
//     WorkingSetList->HashTableSize = 0;
//     WorkingSetList->Wsle = MmSessionSpace->Wsle;
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Handle list insertions */
//     ASSERT(SessionGlobal->WsListEntry.Flink == NULL);
//     ASSERT(SessionGlobal->WsListEntry.Blink == NULL);
//     InsertTailList(&MiSessionWsList, &SessionGlobal->WsListEntry);
//
//     ASSERT(SessionGlobal->Vm.WorkingSetExpansionLinks.Flink == NULL);
//     ASSERT(SessionGlobal->Vm.WorkingSetExpansionLinks.Blink == NULL);
//     InsertTailList(&MmWorkingSetExpansionHead,
//                    &SessionGlobal->Vm.WorkingSetExpansionLinks);
//
//     /* Release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* All done, return */
//     //MmUnlockPageableImageSection(ExPageLockHandle);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MiSessionCreateInternal(OUT PULONG SessionId)
// {
//     PEPROCESS Process = PsGetCurrentProcess();
//     ULONG NewFlags, Flags, i, Color;
// #if (_MI_PAGING_LEVELS < 3)
//     ULONG Size;
// #endif // (_MI_PAGING_LEVELS < 3)
//     PMMPDE PageTables = NULL;
//     KIRQL OldIrql;
//     PMMPTE PointerPte, SessionPte;
//     PMMPDE PointerPde;
//     PMM_SESSION_SPACE SessionGlobal;
//     MMPTE TempPte;
//     MMPDE TempPde;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     PFN_NUMBER SessionPageDirIndex;
//     PFN_NUMBER TagPage[MI_SESSION_TAG_PAGES_MAXIMUM];
//     PFN_NUMBER DataPage[MI_SESSION_DATA_PAGES_MAXIMUM];
//
//     /* This should not exist yet */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == FALSE);
//
//     /* Loop so we can set the session-is-creating flag */
//     Flags = Process->Flags;
//     while (TRUE)
//     {
//         /* Check if it's already set */
//         if (Flags & PSF_SESSION_CREATION_UNDERWAY_BIT)
//         {
//             /* Bail out */
//             DPRINT1("Lost session race\n");
//             return STATUS_ALREADY_COMMITTED;
//         }
//
//         /* Now try to set it */
//         NewFlags = InterlockedCompareExchange((PLONG)&Process->Flags,
//                                               Flags | PSF_SESSION_CREATION_UNDERWAY_BIT,
//                                               Flags);
//         if (NewFlags == Flags) break;
//
//         /* It changed, try again */
//         Flags = NewFlags;
//     }
//
//     /* Now we should own the flag */
//     ASSERT(Process->Flags & PSF_SESSION_CREATION_UNDERWAY_BIT);
//
// #if (_MI_PAGING_LEVELS < 3)
//     /*
//      * Session space covers everything from 0xA0000000 to 0xC0000000.
//      * Allocate enough page tables to describe the entire region
//      */
//     Size = (0x20000000 / PDE_MAPPED_VA) * sizeof(MMPTE);
//     PageTables = ExAllocatePoolWithTag(NonPagedPool, Size, 'tHmM');
//     ASSERT(PageTables != NULL);
//     RtlZeroMemory(PageTables, Size);
// #endif // (_MI_PAGING_LEVELS < 3)
//
//     /* Lock the session ID creation mutex */
//     KeAcquireGuardedMutex(&MiSessionIdMutex);
//
//     /* Allocate a new Session ID */
//     *SessionId = RtlFindClearBitsAndSet(MiSessionIdBitmap, 1, 0);
//     if (*SessionId == 0xFFFFFFFF)
//     {
//         /* We ran out of session IDs, we should expand */
//         DPRINT1("Too many sessions created. Expansion not yet supported\n");
// #if (_MI_PAGING_LEVELS < 3)
//         ExFreePoolWithTag(PageTables, 'tHmM');
// #endif // (_MI_PAGING_LEVELS < 3)
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Unlock the session ID creation mutex */
//     KeReleaseGuardedMutex(&MiSessionIdMutex);
//
//     /* Reserve the global PTEs */
//     SessionPte = MiReserveSystemPtes(MiSessionDataPages, SystemPteSpace);
//     ASSERT(SessionPte != NULL);
//
//     /* Acquire the PFN lock while we set everything up */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Loop the global PTEs */
//     TempPte = ValidKernelPte;
//     for (i = 0; i < MiSessionDataPages; i++)
//     {
//         /* Get a zeroed colored zero page */
//         MI_SET_USAGE(MI_USAGE_INIT_MEMORY);
//         Color = MI_GET_NEXT_COLOR();
//         DataPage[i] = MiRemoveZeroPageSafe(Color);
//         if (!DataPage[i])
//         {
//             /* No zero pages, grab a free one */
//             DataPage[i] = MiRemoveAnyPage(Color);
//
//             /* Zero it outside the PFN lock */
//             MiReleasePfnLock(OldIrql);
//             MiZeroPhysicalPage(DataPage[i]);
//             OldIrql = MiAcquirePfnLock();
//         }
//
//         /* Fill the PTE out */
//         TempPte.set_page_frame_number((DataPage[i]) as PFN_NUMBER);
//         MI_WRITE_VALID_PTE(SessionPte + i, TempPte);
//     }
//
//     /* Set the pointer to global space */
//     SessionGlobal = MiPteToAddress(SessionPte);
//
//     /* Get a zeroed colored zero page */
//     MI_SET_USAGE(MI_USAGE_INIT_MEMORY);
//     Color = MI_GET_NEXT_COLOR();
//     SessionPageDirIndex = MiRemoveZeroPageSafe(Color);
//     if (!SessionPageDirIndex)
//     {
//         /* No zero pages, grab a free one */
//         SessionPageDirIndex = MiRemoveAnyPage(Color);
//
//         /* Zero it outside the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         MiZeroPhysicalPage(SessionPageDirIndex);
//         OldIrql = MiAcquirePfnLock();
//     }
//
//     /* Fill the PTE out */
//     TempPde = ValidKernelPdeLocal;
//     TempPde.set_page_frame_number((SessionPageDirIndex) as PFN_NUMBER);
//
//     /* Setup, allocate, fill out the MmSessionSpace PTE */
//     PointerPde = MiAddressToPde(MmSessionSpace);
//     ASSERT(PointerPde->u.Long == 0);
//     MI_WRITE_VALID_PDE(PointerPde, TempPde);
//     MiInitializePfnForOtherProcess(SessionPageDirIndex,
//                                    PointerPde,
//                                    SessionPageDirIndex);
//     ASSERT(MI_PFN_ELEMENT(SessionPageDirIndex)->u1.WsIndex == 0);
//
//      /* Loop all the local PTEs for it */
//     TempPte = ValidKernelPteLocal;
//     PointerPte = MiAddressToPte(MmSessionSpace);
//     for (i = 0; i < MiSessionDataPages; i++)
//     {
//         /* And fill them out */
//         TempPte.set_page_frame_number((DataPage[i]) as PFN_NUMBER);
//         MiInitializePfnAndMakePteValid(DataPage[i], PointerPte + i, TempPte);
//         ASSERT(MI_PFN_ELEMENT(DataPage[i])->u1.WsIndex == 0);
//     }
//
//      /* Finally loop all of the session pool tag pages */
//     for (i = 0; i < MiSessionTagPages; i++)
//     {
//         /* Grab a zeroed colored page */
//         MI_SET_USAGE(MI_USAGE_INIT_MEMORY);
//         Color = MI_GET_NEXT_COLOR();
//         TagPage[i] = MiRemoveZeroPageSafe(Color);
//         if (!TagPage[i])
//         {
//             /* No zero pages, grab a free one */
//             TagPage[i] = MiRemoveAnyPage(Color);
//
//             /* Zero it outside the PFN lock */
//             MiReleasePfnLock(OldIrql);
//             MiZeroPhysicalPage(TagPage[i]);
//             OldIrql = MiAcquirePfnLock();
//         }
//
//         /* Fill the PTE out */
//         TempPte.set_page_frame_number((TagPage[i]) as PFN_NUMBER);
//         MiInitializePfnAndMakePteValid(TagPage[i],
//                                        PointerPte + MiSessionDataPages + i,
//                                        TempPte);
//     }
//
//     /* PTEs have been setup, release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Fill out the session space structure now */
//     MmSessionSpace->GlobalVirtualAddress = SessionGlobal;
//     MmSessionSpace->ReferenceCount = 1;
//     MmSessionSpace->ResidentProcessCount = 1;
//     MmSessionSpace->u.LongFlags = 0;
//     MmSessionSpace->SessionId = *SessionId;
//     MmSessionSpace->LocaleId = PsDefaultSystemLocaleId;
//     MmSessionSpace->SessionPageDirectoryIndex = SessionPageDirIndex;
//     MmSessionSpace->Color = Color;
//     MmSessionSpace->NonPageablePages = MiSessionCreateCharge;
//     MmSessionSpace->CommittedPages = MiSessionCreateCharge;
// #if (_MI_PAGING_LEVELS < 3)
//     MmSessionSpace->PageTables = PageTables;
//     MmSessionSpace->PageTables[PointerPde - MiAddressToPde(MmSessionBase)] = *PointerPde;
// #endif
//     InitializeListHead(&MmSessionSpace->ImageList);
//
//     DPRINT1("Session %lu is ready to go: 0x%p 0x%p, %lx 0x%p\n",
//             *SessionId, MmSessionSpace, SessionGlobal, SessionPageDirIndex, PageTables);
//
//     /* Initialize session pool */
//     //Status = MiInitializeSessionPool();
//     Status = STATUS_SUCCESS;
//     ASSERT(NT_SUCCESS(Status) == TRUE);
//
//     /* Initialize system space */
//     Result = MiInitializeSystemSpaceMap(&SessionGlobal->Session);
//     ASSERT(Result == TRUE);
//
//     /* Initialize the process list, make sure the workign set list is empty */
//     ASSERT(SessionGlobal->WsListEntry.Flink == NULL);
//     ASSERT(SessionGlobal->WsListEntry.Blink == NULL);
//     InitializeListHead(&SessionGlobal->ProcessList);
//
//     /* We're done, clear the flag */
//     ASSERT(Process->Flags & PSF_SESSION_CREATION_UNDERWAY_BIT);
//     PspClearProcessFlag(Process, PSF_SESSION_CREATION_UNDERWAY_BIT);
//
//     /* Insert the process into the session  */
//     ASSERT(Process->Session == NULL);
//     ASSERT(SessionGlobal->ProcessReferenceToSession == 0);
//     SessionGlobal->ProcessReferenceToSession = 1;
//
//     /* We're done */
//     InterlockedIncrement(&MmSessionDataPages);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MmSessionCreate(OUT PULONG SessionId)
// {
//     PEPROCESS Process = PsGetCurrentProcess();
//     ULONG SessionLeaderExists;
//     NTSTATUS Status;
//
//     /* Fail if the process is already in a session */
//     if (Process->Flags & PSF_PROCESS_IN_SESSION_BIT)
//     {
//         DPRINT1("Process already in session\n");
//         return STATUS_ALREADY_COMMITTED;
//     }
//
//     /* Check if the process is already the session leader */
//     if (!Process->Vm.Flags.SessionLeader)
//     {
//         /* Atomically set it as the leader */
//         SessionLeaderExists = InterlockedCompareExchange(&MiSessionLeaderExists, 1, 0);
//         if (SessionLeaderExists)
//         {
//             DPRINT1("Session leader race\n");
//             return STATUS_INVALID_SYSTEM_SERVICE;
//         }
//
//         /* Do the work required to upgrade him */
//         MiSessionLeader(Process);
//     }
//
//     /* Create the session */
//     KeEnterCriticalRegion();
//     Status = MiSessionCreateInternal(SessionId);
//     if (!NT_SUCCESS(Status))
//     {
//         KeLeaveCriticalRegion();
//         return Status;
//     }
//
//     /* Set up the session working set */
//     Status = MiSessionInitializeWorkingSetList();
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         //MiDereferenceSession();
//         ASSERT(FALSE);
//         KeLeaveCriticalRegion();
//         return Status;
//     }
//
//     /* All done */
//     KeLeaveCriticalRegion();
//
//     /* Set and assert the flags, and return */
//     MmSessionSpace->u.Flags.Initialized = 1;
//     PspSetProcessFlag(Process, PSF_PROCESS_IN_SESSION_BIT);
//     ASSERT(MiSessionLeaderExists == 1);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// MmSessionDelete(IN ULONG SessionId)
// {
//     PEPROCESS Process = PsGetCurrentProcess();
//
//     /* Process must be in a session */
//     if (!(Process->Flags & PSF_PROCESS_IN_SESSION_BIT))
//     {
//         DPRINT1("Not in a session!\n");
//         return STATUS_UNABLE_TO_FREE_VM;
//     }
//
//     /* It must be the session leader */
//     if (!Process->Vm.Flags.SessionLeader)
//     {
//         DPRINT1("Not a session leader!\n");
//         return STATUS_UNABLE_TO_FREE_VM;
//     }
//
//     /* Remove one reference count */
//     KeEnterCriticalRegion();
//     MiDereferenceSession();
//     KeLeaveCriticalRegion();
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// NTAPI
// MmAttachSession(
//     _Inout_ PVOID SessionEntry,
//     _Out_ PKAPC_STATE ApcState)
// {
//     PEPROCESS EntryProcess;
//     PMM_SESSION_SPACE EntrySession, CurrentSession;
//     PEPROCESS CurrentProcess;
//     KIRQL OldIrql;
//
//     /* The parameter is the actual process! */
//     EntryProcess = SessionEntry;
//     ASSERT(EntryProcess != NULL);
//
//     /* Sanity checks */
//     ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//     ASSERT(EntryProcess->Vm.Flags.SessionLeader == 0);
//
//     /* Get the session from the process that was passed in */
//     EntrySession = EntryProcess->Session;
//     ASSERT(EntrySession != NULL);
//
//     /* Get the current process and it's session */
//     CurrentProcess = PsGetCurrentProcess();
//     CurrentSession = CurrentProcess->Session;
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Check if the session is about to be deleted */
//     if (EntrySession->u.Flags.DeletePending)
//     {
//         /* We cannot attach to it, so unlock and fail */
//         MiReleaseExpansionLock(OldIrql);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Count the number of attaches */
//     EntrySession->AttachCount++;
//
//     /* we can release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Check if we are not the session leader and we are in a session */
//     if (!CurrentProcess->Vm.Flags.SessionLeader && (CurrentSession != NULL))
//     {
//         /* Are we already in the right session? */
//         if (CurrentSession == EntrySession)
//         {
//             /* We are, so "attach" to the current process */
//             EntryProcess = CurrentProcess;
//         }
//         else
//         {
//             /* We are not, the session id should better not match! */
//             ASSERT(CurrentSession->SessionId != EntrySession->SessionId);
//         }
//     }
//
//     /* Now attach to the process that we have */
//     KeStackAttachProcess(&EntryProcess->Pcb, ApcState);
//
//     /* Success! */
//     return STATUS_SUCCESS;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// NTAPI
// MmDetachSession(
//     _Inout_ PVOID SessionEntry,
//     _In_ PKAPC_STATE ApcState)
// {
//     PEPROCESS EntryProcess;
//     PMM_SESSION_SPACE EntrySession;
//     KIRQL OldIrql;
//     BOOLEAN DeletePending;
//
//     /* The parameter is the actual process! */
//     EntryProcess = SessionEntry;
//     ASSERT(EntryProcess != NULL);
//
//     /* Sanity checks */
//     ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//     ASSERT(EntryProcess->Vm.Flags.SessionLeader == 0);
//
//     /* Get the session from the process that was passed in */
//     EntrySession = EntryProcess->Session;
//     ASSERT(EntrySession != NULL);
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Make sure we have at least one attach and decrement the count */
//     ASSERT(EntrySession->AttachCount >= 1);
//     EntrySession->AttachCount--;
//
//     /* Remember if a delete is pending and we were the last one attached */
//     DeletePending = EntrySession->u.Flags.DeletePending &&
//                     (EntrySession->AttachCount == 0);
//
//     /* Release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     /* Detach from the process */
//     KeUnstackDetachProcess(ApcState);
//
//     /* Check if we need to set the attach event */
//     if (DeletePending)
//         KeSetEvent(&EntrySession->AttachEvent, IO_NO_INCREMENT, FALSE);
// }
//
// VOID
// NTAPI
// MmQuitNextSession(
//     _Inout_ PVOID SessionEntry)
// {
//     PEPROCESS EntryProcess;
//
//     /* The parameter is the actual process! */
//     EntryProcess = SessionEntry;
//     ASSERT(EntryProcess != NULL);
//
//     /* Sanity checks */
//     ASSERT(KeGetCurrentIrql () <= APC_LEVEL);
//     ASSERT(EntryProcess->Vm.Flags.SessionLeader == 0);
//     ASSERT(EntryProcess->Session != NULL);
//
//     /* Get rid of the reference we took */
//     ObDereferenceObject(EntryProcess);
// }
//
// PVOID
// NTAPI
// MmGetSessionById(
//     _In_ ULONG SessionId)
// {
//     PLIST_ENTRY ListEntry;
//     PMM_SESSION_SPACE Session;
//     PEPROCESS Process = NULL;
//     KIRQL OldIrql;
//
//     /* Acquire the expansion lock while touching the session */
//     OldIrql = MiAcquireExpansionLock();
//
//     /* Loop all entries in the session ws list */
//     ListEntry = MiSessionWsList.Flink;
//     while (ListEntry != &MiSessionWsList)
//     {
//         Session = CONTAINING_RECORD(ListEntry, MM_SESSION_SPACE, WsListEntry);
//         ListEntry = ListEntry->Flink;
//
//         /* Check if this is the session we are looking for */
//         if (Session->SessionId == SessionId)
//         {
//             /* Check if we also have a process in the process list */
//             if (!IsListEmpty(&Session->ProcessList))
//             {
//                 Process = CONTAINING_RECORD(Session->ProcessList.Flink,
//                                             EPROCESS,
//                                             SessionProcessLinks);
//
//                 /* Reference the process */
//                 ObReferenceObject(Process);
//                 break;
//             }
//         }
//     }
//
//     /* Release the lock again */
//     MiReleaseExpansionLock(OldIrql);
//
//     return Process;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
