use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    LARGE_INTEGER,
    PETHREAD,
    MiDecrementShareCount,
    MiInitializePfnAndMakePteValid,
    PsGetCurrentThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeQueryTickCount(TickCount: *mut LARGE_INTEGER);
    static ValidKernelPte: MMPTE;
    static mut ExpPoolFlags: ULONG;
    static mut MmSystemPteBase: PMMPTE;
    fn MiReserveAlignedSystemPtes( NumberOfPtes: ULONG, SystemPtePoolType: MMSYSTEM_PTE_POOL_TYPE, Alignment: ULONG, ) -> PMMPTE;
    fn KeFlushEntireTb(Invalid: BOOLEAN, AllProcessors: BOOLEAN);
    fn RtlFillMemory(Destination: PVOID, Length: SIZE_T, Fill: UCHAR);
    fn MiDeleteSystemPageableVm(StartingPte: PMMPTE, NumberOfPtes: PFN_NUMBER, Flags: ULONG, ResidentPages: *mut PFN_NUMBER) -> ULONG;
}

const SystemPteSpace: MMSYSTEM_PTE_POOL_TYPE = 0;
// POOL_FLAG_SPECIAL_POOL comes from `ntkrnl_sys::mm` (0x20, miarm.h:328). A local
// const here USED to say 0x10 — that is POOL_FLAG_CHECK_DEADLOCK — and being local
// it silently shadowed the glob-imported real one. MiInitializeSpecialPool then set
// the wrong bit forever, so ExAllocatePoolWithTag's `ExpPoolFlags & 0x20` test never
// fired: special pool armed, initialized, printed "ACTIVE", and routed not one
// allocation. Never redeclare a constant that the crate already imports.
const PTE_PER_PAGE: usize = PAGE_SIZE / core::mem::size_of::<MMPTE>();
const SPECIAL_POOL_PAGED_PTE: ULONG_PTR = 0x2000;
const SPECIAL_POOL_NONPAGED_PTE: ULONG_PTR = 0x4000;
const SPECIAL_POOL_PAGED: ULONG = 0x8000;

#[no_mangle]
pub static mut MmSpecialPoolStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSpecialPoolEnd: PVOID = core::ptr::null_mut();
static mut MiSpecialPoolExtra: PVOID = core::ptr::null_mut();
static mut MiSpecialPoolExtraCount: ULONG = 0;

static mut MiSpecialPoolFirstPte: PMMPTE = core::ptr::null_mut();
static mut MiSpecialPoolLastPte: PMMPTE = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSpecialPagesInUse: PFN_COUNT = 0;
#[no_mangle]
pub static mut MmSpecialPagesInUsePeak: PFN_COUNT = 0;
static mut MiSpecialPagesPagable: PFN_COUNT = 0;
static mut MiSpecialPagesPagablePeak: PFN_COUNT = 0;
static mut MiSpecialPagesNonPaged: PFN_COUNT = 0;
static mut MiSpecialPagesNonPagedPeak: PFN_COUNT = 0;
static mut MiSpecialPagesNonPagedMaximum: PFN_COUNT = 0;

#[no_mangle]
pub static mut MmSpecialPoolCatchOverruns: BOOLEAN = 1;

#[repr(C)]
struct MI_FREED_SPECIAL_POOL {
    OverlaidPoolHeader: POOL_HEADER,
    Signature: ULONG,
    TickCount: ULONG,
    NumberOfBytesRequested: ULONG,
    Pagable: BOOLEAN,
    VirtualAddress: PVOID,
    StackPointer: PVOID,
    StackBytes: ULONG,
    Thread: PETHREAD,
    StackData: [UCHAR; 0x400],
}
type PMI_FREED_SPECIAL_POOL = *mut MI_FREED_SPECIAL_POOL;

#[no_mangle]
pub unsafe extern "system" fn MmUseSpecialPool(NumberOfBytes: SIZE_T, Tag: ULONG) -> BOOLEAN {
    if NumberOfBytes > (PAGE_SIZE - core::mem::size_of::<POOL_HEADER>()) {
        return 0;
    }

    if MmSpecialPoolTag == '*' as u32 {
        return 1;
    }

    if Tag == MmSpecialPoolTag { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn MmIsSpecialPoolAddress(P: PVOID) -> BOOLEAN {
    let P = opaque_ptr(P);
    if (P >= MmSpecialPoolStart) && (P <= MmSpecialPoolEnd) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn MmIsSpecialPoolAddressFree(P: PVOID) -> BOOLEAN {
    let P = opaque_ptr(P);
    let pointer_pte = MiAddressToPte(P);

    if (*pointer_pte).soft_page_file_high() == SPECIAL_POOL_PAGED_PTE ||
       (*pointer_pte).soft_page_file_high() == SPECIAL_POOL_NONPAGED_PTE
    {
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeSpecialPool() {
    let mut special_pool_ptes: ULONG;
    let mut pointer_pte: PMMPTE;

    if (MmSpecialPoolTag == 0) || (MmSpecialPoolTag == u32::MAX) {
        return;
    }

    if MmNumberOfSystemPtes >= 0x3000 {
        special_pool_ptes = MmNumberOfSystemPtes / 3;
    } else {
        special_pool_ptes = MmNumberOfSystemPtes / 6;
    }

    if special_pool_ptes > 0x6000 {
        special_pool_ptes = 0x6000;
    }

    // C: PAGE_ROUND_UP — round UP, not down (down can yield 0 on small-PTE
    // configs, silently disabling special pool; must stay >= PTE_PER_PAGE).
    special_pool_ptes = ((special_pool_ptes as usize + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)) as ULONG;

    loop {
        pointer_pte = MiReserveAlignedSystemPtes(special_pool_ptes, SystemPteSpace, 0);
        if !pointer_pte.is_null() {
            break;
        }

        special_pool_ptes -= PTE_PER_PAGE as ULONG;
        if special_pool_ptes == 0 {
            break;
        }
    }

    if special_pool_ptes == 0 {
        return;
    }

    MiSpecialPoolFirstPte = pointer_pte;
    MmSpecialPoolStart = MiPteToAddress(pointer_pte);

    for i in 0..(PTE_PER_PAGE / 2) {
        (*pointer_pte).set_list_next_entry((pointer_pte.add(2) as usize - MmSystemPteBase as usize) / core::mem::size_of::<MMPTE>() as ULONG_PTR);
        pointer_pte = pointer_pte.add(2);
    }

    MiSpecialPoolExtra = pointer_pte as PVOID;
    MiSpecialPoolExtraCount = special_pool_ptes - PTE_PER_PAGE as ULONG;

    MiSpecialPoolLastPte = pointer_pte.sub(2);
    (*MiSpecialPoolLastPte).set_list_next_entry(MM_EMPTY_PTE_LIST as ULONG_PTR);

    MmSpecialPoolEnd = MiPteToAddress(MiSpecialPoolLastPte.add(1));

    MiSpecialPagesNonPagedMaximum = (MmResidentAvailablePages >> 4) as PFN_COUNT;

    if MmNumberOfPhysicalPages > 0x3FFF {
        MiSpecialPagesNonPagedMaximum = (MmResidentAvailablePages >> 3) as PFN_COUNT;
    }

    ExpPoolFlags |= POOL_FLAG_SPECIAL_POOL;
}

#[no_mangle]
pub unsafe extern "system" fn MmExpandSpecialPool() -> NTSTATUS {
    let mut pointer_pte: PMMPTE;

    if MiSpecialPoolExtraCount == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    pointer_pte = MiSpecialPoolExtra as PMMPTE;
    (*MiSpecialPoolFirstPte).set_list_next_entry((pointer_pte as usize - MmSystemPteBase as usize) / core::mem::size_of::<MMPTE>() as ULONG_PTR);

    for i in 0..(PTE_PER_PAGE / 2) {
        (*pointer_pte).set_list_next_entry((pointer_pte.add(2) as usize - MmSystemPteBase as usize) / core::mem::size_of::<MMPTE>() as ULONG_PTR);
        pointer_pte = pointer_pte.add(2);
    }

    MiSpecialPoolExtra = pointer_pte as PVOID;
    MiSpecialPoolExtraCount -= PTE_PER_PAGE as ULONG;

    MiSpecialPoolLastPte = pointer_pte.sub(2);
    (*MiSpecialPoolLastPte).set_list_next_entry(MM_EMPTY_PTE_LIST as ULONG_PTR);

    MmSpecialPoolEnd = MiPteToAddress(MiSpecialPoolLastPte.add(1));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocateSpecialPool(
    NumberOfBytes: SIZE_T,
    Tag: ULONG,
    PoolType: POOL_TYPE,
    SpecialType: ULONG,
) -> PVOID {
    let mut irql: KIRQL;
    let mut temp_pte = ValidKernelPte;
    let mut pointer_pte: PMMPTE;
    let page_frame_number: PFN_NUMBER;
    let mut tick_count: i64 = 0;
    let entry: PVOID;
    let header: PPOOL_HEADER;
    let mut pages_in_use: PFN_COUNT;
    let pool_type = PoolType & BASE_POOL_TYPE_MASK;

    if MiSpecialPoolFirstPte.is_null() {
        return core::ptr::null_mut();
    }

    irql = KeGetCurrentIrql();

    if ((pool_type == PagedPool) && (irql > APC_LEVEL)) ||
       ((pool_type != PagedPool) && (irql > DISPATCH_LEVEL))
    {
        KeBugCheckEx(
            SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
            irql as ULONG_PTR,
            pool_type as ULONG_PTR,
            NumberOfBytes,
            0x30,
        );
    }

    // Some allocations from Mm must never use special pool (recursion under the
    // PFN/segment paths). C uses 'tSmM' (= from_be_bytes, displays "MmSt"); the
    // live Rust segment allocator (ARM3/section.rs) actually tags 0x744D6D4D, so
    // exclude both the C-intent value and the as-shipped value.
    if Tag == u32::from_be_bytes(*b"tSmM") || Tag == 0x744D6D4D {
        return core::ptr::null_mut();
    }

    if (pool_type == NonPagedPool) && (MiSpecialPagesNonPaged > MiSpecialPagesNonPagedMaximum) {
        return core::ptr::null_mut();
    }

    irql = MiAcquirePfnLock();

    if MmAvailablePages < 0x100 {
        MiReleasePfnLock(irql);
        return core::ptr::null_mut();
    }

    if (*MiSpecialPoolFirstPte).list_next_entry() == MM_EMPTY_PTE_LIST {
        if MmExpandSpecialPool() < 0 {
            MiReleasePfnLock(irql);
            return core::ptr::null_mut();
        }
    }

    KeQueryTickCount(core::ptr::addr_of_mut!(tick_count) as *mut LARGE_INTEGER);

    pointer_pte = MiSpecialPoolFirstPte;

    MiSpecialPoolFirstPte = MmSystemPteBase.add((*pointer_pte).list_next_entry() as usize);

    if pool_type == PagedPool {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGED_POOL);
    } else {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_NONPAGED_POOL);
    }
    ntkrnl_sys::MI_SET_PROCESS2!("Kernel-Special");
    page_frame_number = MiRemoveAnyPage(MI_GET_NEXT_COLOR());

    temp_pte.set_page_frame_number(page_frame_number);
    MiInitializePfnAndMakePteValid(page_frame_number as _, pointer_pte as *mut ntkrnl_sys::ddk::_MMPTE, core::mem::transmute(temp_pte));

    MiReleasePfnLock(irql);

    pages_in_use = InterlockedIncrementUL(core::ptr::addr_of_mut!(MmSpecialPagesInUse));
    if pages_in_use > MmSpecialPagesInUsePeak {
        MmSpecialPagesInUsePeak = pages_in_use;
    }

    let entry_addr = MiPteToAddress(pointer_pte);
    RtlFillMemory(entry_addr, PAGE_SIZE, (tick_count as u32) as UCHAR);

    let entry_final: PVOID;
    if (SpecialType != 0) && ((SpecialType == 1) || (MmSpecialPoolCatchOverruns == 0)) {
        header = (entry_addr as usize + PAGE_SIZE - core::mem::size_of::<POOL_HEADER>()) as PPOOL_HEADER;
        entry_final = entry_addr;
    } else {
        header = entry_addr as PPOOL_HEADER;
        entry_final = ((entry_addr as usize - NumberOfBytes + PAGE_SIZE) & !((core::mem::size_of::<POOL_HEADER>() as isize - 1) as usize)) as PVOID;
    }

    RtlZeroMemory(header as PVOID, core::mem::size_of::<POOL_HEADER>());

    (*header).Ulong1 = NumberOfBytes as ULONG;

    if pool_type == PagedPool {
        (*header).Ulong1 |= SPECIAL_POOL_PAGED;
        { let v = (*pointer_pte.add(1)).soft_page_file_high() | (SPECIAL_POOL_PAGED_PTE as ULONG_PTR); (*pointer_pte.add(1)).set_soft_page_file_high(v); }

        pages_in_use = InterlockedIncrementUL(core::ptr::addr_of_mut!(MiSpecialPagesPagable));
        if pages_in_use > MiSpecialPagesPagablePeak {
            MiSpecialPagesPagablePeak = pages_in_use;
        }
    } else {
        { let v = (*pointer_pte.add(1)).soft_page_file_high() | (SPECIAL_POOL_NONPAGED_PTE as ULONG_PTR); (*pointer_pte.add(1)).set_soft_page_file_high(v); }

        pages_in_use = InterlockedIncrementUL(core::ptr::addr_of_mut!(MiSpecialPagesNonPaged));
        if pages_in_use > MiSpecialPagesNonPagedPeak {
            MiSpecialPagesNonPagedPeak = pages_in_use;
        }
    }

    (*header).PoolTag = Tag;
    // C stores (UCHAR)TickCount.LowPart; keep to 8 bits so the 9-bit BlockSize
    // field can never disagree with the 8-bit fill pattern checks.
    (*header).set_BlockSize((tick_count as u32 as u8) as u16);
    entry_final
}

unsafe fn MiSpecialPoolCheckPattern(P: PUCHAR, Header: PPOOL_HEADER) {
    let bytes_requested = ((*Header).Ulong1 & !SPECIAL_POOL_PAGED) & 0xFFFF;

    let ptr = P.add(bytes_requested as usize);

    let mut bytes_to_check = (PAGE_ALIGN(P as usize) as usize + PAGE_SIZE - ptr as usize) as ULONG;

    if ((P as usize) & (PAGE_SIZE - 1)) == 0 {
        bytes_to_check -= core::mem::size_of::<POOL_HEADER>() as ULONG;
    }

    for index in 0..bytes_to_check {
        if *ptr.add(index as usize) != (*Header).BlockSize() as u8 {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                P as ULONG_PTR,
                ptr.add(index as usize) as ULONG_PTR,
                (*Header).BlockSize() as u8 as ULONG_PTR,
                0x24,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeSpecialPool(P: PVOID) {
    let P = opaque_ptr(P);
    let pointer_pte: PMMPTE;
    let header: PPOOL_HEADER;
    let mut overruns = 0u8;
    let mut irql = KeGetCurrentIrql();
    let pool_type: POOL_TYPE;
    let bytes_requested: ULONG;
    let mut bytes_real: ULONG = 0;
    let ptr_offset: ULONG;
    let freed_header: PMI_FREED_SPECIAL_POOL;
    let mut tick_count: i64 = 0;
    let pfn: PMMPFN;

    pointer_pte = MiAddressToPte(P);

    if (*pointer_pte).valid() == false {
        if (*pointer_pte).soft_protection() == MM_NOACCESS || (*pointer_pte).soft_protection() == 0 {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                P as ULONG_PTR,
                pointer_pte as ULONG_PTR,
                0,
                0x20,
            );
        }
    }

    ptr_offset = ((P as usize) & (PAGE_SIZE - 1)) as ULONG;
    if ptr_offset != 0 {
        header = PAGE_ALIGN(P as usize) as PPOOL_HEADER;
        overruns = 1;
    } else {
        header = (PAGE_ALIGN(P as usize) + PAGE_SIZE - core::mem::size_of::<POOL_HEADER>()) as PPOOL_HEADER;
    }

    if ((*header).Ulong1 & SPECIAL_POOL_PAGED) == 0 {
        pool_type = NonPagedPool;
        if irql > DISPATCH_LEVEL {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                irql as ULONG_PTR,
                pool_type as ULONG_PTR,
                P as ULONG_PTR,
                0x31,
            );
        }
    } else {
        pool_type = PagedPool;
        if irql > APC_LEVEL {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                irql as ULONG_PTR,
                pool_type as ULONG_PTR,
                P as ULONG_PTR,
                0x31,
            );
        }
    }

    bytes_requested = ((*header).Ulong1 & !SPECIAL_POOL_PAGED) & 0xFFFF;

    if overruns != 0 {
        bytes_real = (PAGE_SIZE - ptr_offset as usize) as ULONG;

        if bytes_requested > bytes_real {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                P as ULONG_PTR,
                bytes_requested as ULONG_PTR,
                bytes_real as ULONG_PTR,
                0x21,
            );
        }

        if (bytes_requested + core::mem::size_of::<POOL_HEADER>() as ULONG) < bytes_real {
            KeBugCheckEx(
                SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                P as ULONG_PTR,
                bytes_requested as ULONG_PTR,
                bytes_real as ULONG_PTR,
                0x22,
            );
        }

        let mut b = (header as usize + core::mem::size_of::<POOL_HEADER>()) as PUCHAR;
        while (b as usize) < (P as usize) {
            if *b != (*header).BlockSize() as u8 {
                KeBugCheckEx(
                    SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
                    P as ULONG_PTR,
                    b as ULONG_PTR,
                    (*header).BlockSize() as u8 as ULONG_PTR,
                    0x23,
                );
            }
            b = b.add(1);
        }
    }

    MiSpecialPoolCheckPattern(P as PUCHAR, header);

    KeQueryTickCount(core::ptr::addr_of_mut!(tick_count) as *mut LARGE_INTEGER);
    freed_header = PAGE_ALIGN(P as usize) as PMI_FREED_SPECIAL_POOL;
    (*freed_header).Signature = 0x98764321;
    (*freed_header).TickCount = (tick_count as u32);
    (*freed_header).NumberOfBytesRequested = bytes_requested;
    (*freed_header).Pagable = pool_type as BOOLEAN;
    (*freed_header).VirtualAddress = P;
    (*freed_header).Thread = PsGetCurrentThread();
    (*freed_header).StackPointer = core::ptr::null_mut();
    (*freed_header).StackBytes = 0;

    if pool_type == NonPagedPool {
        pfn = MI_PFN_ELEMENT((*pointer_pte).page_frame_number());

        InterlockedDecrementUL(core::ptr::addr_of_mut!(MiSpecialPagesNonPaged));

        irql = MiAcquirePfnLock();

        MI_SET_PFN_DELETED(pfn);
        MiDecrementShareCount(pfn as *mut ntkrnl_sys::ddk::_MMPFN, (*pointer_pte).page_frame_number() as _);

        MI_ERASE_PTE(pointer_pte);

        KeFlushEntireTb(1, 1);
    } else {
        MiDeleteSystemPageableVm(pointer_pte, 1, 0, core::ptr::null_mut());

        InterlockedDecrementUL(core::ptr::addr_of_mut!(MiSpecialPagesPagable));

        irql = MiAcquirePfnLock();
    }

    MI_ERASE_PTE(pointer_pte.add(1));

    (*MiSpecialPoolLastPte).set_list_next_entry((pointer_pte as usize - MmSystemPteBase as usize) / core::mem::size_of::<MMPTE>() as ULONG_PTR);

    (*pointer_pte).set_list_next_entry(MM_EMPTY_PTE_LIST as ULONG_PTR);
    MiSpecialPoolLastPte = pointer_pte;

    MiReleasePfnLock(irql);

    InterlockedDecrementUL(core::ptr::addr_of_mut!(MmSpecialPagesInUse));
}

unsafe fn MiTestSpecialPool() {
    let mut p1: PVOID;
    let mut p2: [PVOID; 100] = [core::ptr::null_mut(); 100];
    let mut byte_size: ULONG;
    let pool_type = PagedPool;

    for i in 0usize..100 {
        byte_size = ((100 * (i + 1)) % (PAGE_SIZE - core::mem::size_of::<POOL_HEADER>())) as ULONG;
        p1 = MmAllocateSpecialPool(byte_size as SIZE_T, u32::from_le_bytes(*b"TEST"), pool_type, 0);
        MmFreeSpecialPool(p1);
    }

    for i in 0usize..100 {
        byte_size = ((100 * (i + 1)) % (PAGE_SIZE - core::mem::size_of::<POOL_HEADER>())) as ULONG;
        p2[i] = MmAllocateSpecialPool(byte_size as SIZE_T, u32::from_le_bytes(*b"TEST"), pool_type, 0);
    }
    for i in 0usize..100 {
        MmFreeSpecialPool(p2[i]);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/special.c
//  * PURPOSE:         ARM Memory Manager Special Pool implementation
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /*
//     References:
//     https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/special-pool
// */
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
// extern ULONG ExpPoolFlags;
// extern PMMPTE MmSystemPteBase;
//
// PMMPTE
// NTAPI
// MiReserveAlignedSystemPtes(IN ULONG NumberOfPtes,
//                            IN MMSYSTEM_PTE_POOL_TYPE SystemPtePoolType,
//                            IN ULONG Alignment);
//
// /* GLOBALS ********************************************************************/
//
// #define SPECIAL_POOL_PAGED_PTE    0x2000
// #define SPECIAL_POOL_NONPAGED_PTE 0x4000
// #define SPECIAL_POOL_PAGED        0x8000
//
// PVOID MmSpecialPoolStart;
// PVOID MmSpecialPoolEnd;
// PVOID MiSpecialPoolExtra;
// ULONG MiSpecialPoolExtraCount;
//
// PMMPTE MiSpecialPoolFirstPte;
// PMMPTE MiSpecialPoolLastPte;
//
// PFN_COUNT MmSpecialPagesInUse;
// PFN_COUNT MmSpecialPagesInUsePeak;
// PFN_COUNT MiSpecialPagesPagable;
// PFN_COUNT MiSpecialPagesPagablePeak;
// PFN_COUNT MiSpecialPagesNonPaged;
// PFN_COUNT MiSpecialPagesNonPagedPeak;
// PFN_COUNT MiSpecialPagesNonPagedMaximum;
//
// BOOLEAN MmSpecialPoolCatchOverruns = TRUE;
//
// typedef struct _MI_FREED_SPECIAL_POOL
// {
//     POOL_HEADER OverlaidPoolHeader;
//     /* TODO: Add overlaid verifier pool header */
//     ULONG Signature;
//     ULONG TickCount;
//     ULONG NumberOfBytesRequested;
//     BOOLEAN Pagable;
//     PVOID VirtualAddress;
//     PVOID StackPointer;
//     ULONG StackBytes;
//     PETHREAD Thread;
//     UCHAR StackData[0x400];
// } MI_FREED_SPECIAL_POOL, *PMI_FREED_SPECIAL_POOL;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID NTAPI MiTestSpecialPool(VOID);
//
// BOOLEAN
// NTAPI
// MmUseSpecialPool(SIZE_T NumberOfBytes, ULONG Tag)
// {
//     /* Special pool is not suitable for allocations bigger than 1 page */
//     if (NumberOfBytes > (PAGE_SIZE - sizeof(POOL_HEADER)))
//     {
//         return FALSE;
//     }
//
//     if (MmSpecialPoolTag == '*')
//     {
//         return TRUE;
//     }
//
//     return Tag == MmSpecialPoolTag;
// }
//
// BOOLEAN
// NTAPI
// MmIsSpecialPoolAddress(PVOID P)
// {
//     return ((P >= MmSpecialPoolStart) &&
//             (P <= MmSpecialPoolEnd));
// }
//
// BOOLEAN
// NTAPI
// MmIsSpecialPoolAddressFree(PVOID P)
// {
//     PMMPTE PointerPte;
//
//     ASSERT(MmIsSpecialPoolAddress(P));
//     PointerPte = MiAddressToPte(P);
//
//     if (PointerPte->u.Soft.PageFileHigh == SPECIAL_POOL_PAGED_PTE ||
//         PointerPte->u.Soft.PageFileHigh == SPECIAL_POOL_NONPAGED_PTE)
//     {
//         /* Guard page PTE */
//         return FALSE;
//     }
//
//     /* Free PTE */
//     return TRUE;
// }
//
// VOID
// NTAPI
// MiInitializeSpecialPool(VOID)
// {
//     ULONG SpecialPoolPtes, i;
//     PMMPTE PointerPte;
//
//     /* Check if there is a special pool tag */
//     if ((MmSpecialPoolTag == 0) ||
//         (MmSpecialPoolTag == -1)) return;
//
//     /* Calculate number of system PTEs for the special pool */
//     if (MmNumberOfSystemPtes >= 0x3000)
//         SpecialPoolPtes = MmNumberOfSystemPtes / 3;
//     else
//         SpecialPoolPtes = MmNumberOfSystemPtes / 6;
//
//     /* Don't let the number go too high */
//     if (SpecialPoolPtes > 0x6000) SpecialPoolPtes = 0x6000;
//
//     /* Round up to the page size */
//     SpecialPoolPtes = PAGE_ROUND_UP(SpecialPoolPtes);
//
//     ASSERT((SpecialPoolPtes & (PTE_PER_PAGE - 1)) == 0);
//
//     /* Reserve those PTEs */
//     do
//     {
//         PointerPte = MiReserveAlignedSystemPtes(SpecialPoolPtes,
//                                                 SystemPteSpace,
//                                                 /*0x400000*/0); // FIXME:
//         if (PointerPte) break;
//
//         /* Reserving didn't work, so try to reduce the requested size */
//         ASSERT(SpecialPoolPtes >= PTE_PER_PAGE);
//         SpecialPoolPtes -= PTE_PER_PAGE;
//     } while (SpecialPoolPtes);
//
//     /* Fail if we couldn't reserve them at all */
//     if (!SpecialPoolPtes) return;
//
//     /* Make sure we got enough */
//     ASSERT(SpecialPoolPtes >= PTE_PER_PAGE);
//
//     /* Save first PTE and its address */
//     MiSpecialPoolFirstPte = PointerPte;
//     MmSpecialPoolStart = MiPteToAddress(PointerPte);
//
//     for (i = 0; i < PTE_PER_PAGE / 2; i++)
//     {
//         /* Point it to the next entry */
//         PointerPte->u.List.NextEntry = &PointerPte[2] - MmSystemPteBase;
//
//         /* Move to the next pair */
//         PointerPte += 2;
//     }
//
//     /* Save extra values */
//     MiSpecialPoolExtra = PointerPte;
//     MiSpecialPoolExtraCount = SpecialPoolPtes - PTE_PER_PAGE;
//
//     /* Mark the previous PTE as the last one */
//     MiSpecialPoolLastPte = PointerPte - 2;
//     MiSpecialPoolLastPte->u.List.NextEntry = MM_EMPTY_PTE_LIST;
//
//     /* Save end address of the special pool */
//     MmSpecialPoolEnd = MiPteToAddress(MiSpecialPoolLastPte + 1);
//
//     /* Calculate maximum non-paged part of the special pool */
//     MiSpecialPagesNonPagedMaximum = MmResidentAvailablePages >> 4;
//
//     /* And limit it if it turned out to be too big */
//     if (MmNumberOfPhysicalPages > 0x3FFF)
//         MiSpecialPagesNonPagedMaximum = MmResidentAvailablePages >> 3;
//
//     DPRINT1("Special pool start %p - end %p\n", MmSpecialPoolStart, MmSpecialPoolEnd);
//     ExpPoolFlags |= POOL_FLAG_SPECIAL_POOL;
//
//     //MiTestSpecialPool();
// }
//
// NTSTATUS
// NTAPI
// MmExpandSpecialPool(VOID)
// {
//     ULONG i;
//     PMMPTE PointerPte;
//
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     if (MiSpecialPoolExtraCount == 0)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     PointerPte = MiSpecialPoolExtra;
//     ASSERT(MiSpecialPoolFirstPte == MiSpecialPoolLastPte);
//     ASSERT(MiSpecialPoolFirstPte->u.List.NextEntry == MM_EMPTY_PTE_LIST);
//     MiSpecialPoolFirstPte->u.List.NextEntry = PointerPte - MmSystemPteBase;
//
//     ASSERT(MiSpecialPoolExtraCount >= PTE_PER_PAGE);
//     for (i = 0; i < PTE_PER_PAGE / 2; i++)
//     {
//         /* Point it to the next entry */
//         PointerPte->u.List.NextEntry = &PointerPte[2] - MmSystemPteBase;
//
//         /* Move to the next pair */
//         PointerPte += 2;
//     }
//
//     /* Save remaining extra values */
//     MiSpecialPoolExtra = PointerPte;
//     MiSpecialPoolExtraCount -= PTE_PER_PAGE;
//
//     /* Mark the previous PTE as the last one */
//     MiSpecialPoolLastPte = PointerPte - 2;
//     MiSpecialPoolLastPte->u.List.NextEntry = MM_EMPTY_PTE_LIST;
//
//     /* Save new end address of the special pool */
//     MmSpecialPoolEnd = MiPteToAddress(MiSpecialPoolLastPte + 1);
//
//     return STATUS_SUCCESS;
// }
//
// PVOID
// NTAPI
// MmAllocateSpecialPool(SIZE_T NumberOfBytes, ULONG Tag, POOL_TYPE PoolType, ULONG SpecialType)
// {
//     KIRQL Irql;
//     MMPTE TempPte = ValidKernelPte;
//     PMMPTE PointerPte;
//     PFN_NUMBER PageFrameNumber;
//     LARGE_INTEGER TickCount;
//     PVOID Entry;
//     PPOOL_HEADER Header;
//     PFN_COUNT PagesInUse;
//
//     DPRINT("MmAllocateSpecialPool(%x %x %x %x)\n", NumberOfBytes, Tag, PoolType, SpecialType);
//
//     /* Check if the pool is initialized and quit if it's not */
//     if (!MiSpecialPoolFirstPte) return NULL;
//
//     /* Get the pool type */
//     PoolType &= BASE_POOL_TYPE_MASK;
//
//     /* Check whether current IRQL matches the pool type */
//     Irql = KeGetCurrentIrql();
//
//     if (((PoolType == PagedPool) && (Irql > APC_LEVEL)) ||
//         ((PoolType != PagedPool) && (Irql > DISPATCH_LEVEL)))
//     {
//         /* Bad caller */
//         KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                      Irql,
//                      PoolType,
//                      NumberOfBytes,
//                      0x30);
//     }
//
//     /* Some allocations from Mm must never use special pool */
//     if (Tag == 'tSmM')
//     {
//         /* Reject and let normal pool handle it */
//         return NULL;
//     }
//
//     /* TODO: Take into account various limitations */
//
//     /* Heed the maximum limit of nonpaged pages */
//     if ((PoolType == NonPagedPool) &&
//         (MiSpecialPagesNonPaged > MiSpecialPagesNonPagedMaximum))
//     {
//         return NULL;
//     }
//
//     /* Lock PFN database */
//     Irql = MiAcquirePfnLock();
//
//     /* Reject allocation in case amount of available pages is too small */
//     if (MmAvailablePages < 0x100)
//     {
//         /* Release the PFN database lock */
//         MiReleasePfnLock(Irql);
//         DPRINT1("Special pool: MmAvailablePages 0x%x is too small\n", MmAvailablePages);
//         return NULL;
//     }
//
//     /* Check if special pool PTE list is exhausted */
//     if (MiSpecialPoolFirstPte->u.List.NextEntry == MM_EMPTY_PTE_LIST)
//     {
//         /* Try to expand it */
//         if (!NT_SUCCESS(MmExpandSpecialPool()))
//         {
//             /* No reserves left, reject this allocation */
//             static int once;
//             MiReleasePfnLock(Irql);
//             if (!once++) DPRINT1("Special pool: No PTEs left!\n");
//             return NULL;
//         }
//         ASSERT(MiSpecialPoolFirstPte->u.List.NextEntry != MM_EMPTY_PTE_LIST);
//     }
//
//     /* Save allocation time */
//     KeQueryTickCount(&TickCount);
//
//     /* Get a pointer to the first PTE */
//     PointerPte = MiSpecialPoolFirstPte;
//
//     /* Set the first PTE pointer to the next one in the list */
//     MiSpecialPoolFirstPte = MmSystemPteBase + PointerPte->u.List.NextEntry;
//
//     /* Allocate a physical page */
//     if (PoolType == PagedPool)
//     {
//         MI_SET_USAGE(MI_USAGE_PAGED_POOL);
//     }
//     else
//     {
//         MI_SET_USAGE(MI_USAGE_NONPAGED_POOL);
//     }
//     MI_SET_PROCESS2("Kernel-Special");
//     PageFrameNumber = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
//
//     /* Initialize PFN and make it valid */
//     TempPte.set_page_frame_number(PageFrameNumber);
//     MiInitializePfnAndMakePteValid(PageFrameNumber, PointerPte, TempPte);
//
//     /* Release the PFN database lock */
//     MiReleasePfnLock(Irql);
//
//     /* Increase page counter */
//     PagesInUse = InterlockedIncrementUL(&MmSpecialPagesInUse);
//     if (PagesInUse > MmSpecialPagesInUsePeak)
//         MmSpecialPagesInUsePeak = PagesInUse;
//
//     /* Put some content into the page. Low value of tick count would do */
//     Entry = MiPteToAddress(PointerPte);
//     RtlFillMemory(Entry, PAGE_SIZE, TickCount.__bindgen_anon_1.__bindgen_anon_1.LowPart);
//
//     /* Calculate header and entry addresses */
//     if ((SpecialType != 0) &&
//         ((SpecialType == 1) || (!MmSpecialPoolCatchOverruns)))
//     {
//         /* We catch underruns. Data is at the beginning of the page */
//         Header = (PPOOL_HEADER)((PUCHAR)Entry + PAGE_SIZE - sizeof(POOL_HEADER));
//     }
//     else
//     {
//         /* We catch overruns. Data is at the end of the page */
//         Header = (PPOOL_HEADER)Entry;
//         Entry = (PVOID)((ULONG_PTR)((PUCHAR)Entry - NumberOfBytes + PAGE_SIZE) & ~((LONG_PTR)sizeof(POOL_HEADER) - 1));
//     }
//
//     /* Initialize the header */
//     RtlZeroMemory(Header, sizeof(POOL_HEADER));
//
//     /* Save allocation size there */
//     Header->Ulong1 = (ULONG)NumberOfBytes;
//
//     /* Make sure it's all good */
//     ASSERT((NumberOfBytes <= PAGE_SIZE - sizeof(POOL_HEADER)) &&
//            (PAGE_SIZE <= 32 * 1024));
//
//     /* Mark it as paged or nonpaged */
//     if (PoolType == PagedPool)
//     {
//         /* Add pagedpool flag into the pool header too */
//         Header->Ulong1 |= SPECIAL_POOL_PAGED;
//
//         /* Also mark the next PTE as special-pool-paged */
//         PointerPte[1].u.Soft.PageFileHigh |= SPECIAL_POOL_PAGED_PTE;
//
//         /* Increase pagable counter */
//         PagesInUse = InterlockedIncrementUL(&MiSpecialPagesPagable);
//         if (PagesInUse > MiSpecialPagesPagablePeak)
//             MiSpecialPagesPagablePeak = PagesInUse;
//     }
//     else
//     {
//         /* Mark the next PTE as special-pool-nonpaged */
//         PointerPte[1].u.Soft.PageFileHigh |= SPECIAL_POOL_NONPAGED_PTE;
//
//         /* Increase nonpaged counter */
//         PagesInUse = InterlockedIncrementUL(&MiSpecialPagesNonPaged);
//         if (PagesInUse > MiSpecialPagesNonPagedPeak)
//             MiSpecialPagesNonPagedPeak = PagesInUse;
//     }
//
//     /* Finally save tag and put allocation time into the header's blocksize.
//        That time will be used to check memory consistency within the allocated
//        page. */
//     Header->PoolTag = Tag;
//     Header->BlockSize = (UCHAR)TickCount.__bindgen_anon_1.__bindgen_anon_1.LowPart;
//     DPRINT("%p\n", Entry);
//     return Entry;
// }
//
// VOID
// NTAPI
// MiSpecialPoolCheckPattern(PUCHAR P, PPOOL_HEADER Header)
// {
//     ULONG BytesToCheck, BytesRequested, Index;
//     PUCHAR Ptr;
//
//     /* Get amount of bytes user requested to be allocated by clearing out the paged mask */
//     BytesRequested = (Header->Ulong1 & ~SPECIAL_POOL_PAGED) & 0xFFFF;
//     ASSERT(BytesRequested <= PAGE_SIZE - sizeof(POOL_HEADER));
//
//     /* Get a pointer to the end of user's area */
//     Ptr = P + BytesRequested;
//
//     /* Calculate how many bytes to check */
//     BytesToCheck = (ULONG)((PUCHAR)PAGE_ALIGN(P) + PAGE_SIZE - Ptr);
//
//     /* Remove pool header size if we're catching underruns */
//     if (((ULONG_PTR)P & (PAGE_SIZE - 1)) == 0)
//     {
//         /* User buffer is located in the beginning of the page */
//         BytesToCheck -= sizeof(POOL_HEADER);
//     }
//
//     /* Check the pattern after user buffer */
//     for (Index = 0; Index < BytesToCheck; Index++)
//     {
//         /* Bugcheck if bytes don't match */
//         if (Ptr[Index] != Header->BlockSize)
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          (ULONG_PTR)P,
//                          (ULONG_PTR)&Ptr[Index],
//                          Header->BlockSize,
//                          0x24);
//         }
//     }
// }
//
// VOID
// NTAPI
// MmFreeSpecialPool(PVOID P)
// {
//     PMMPTE PointerPte;
//     PPOOL_HEADER Header;
//     BOOLEAN Overruns = FALSE;
//     KIRQL Irql = KeGetCurrentIrql();
//     POOL_TYPE PoolType;
//     ULONG BytesRequested, BytesReal = 0;
//     ULONG PtrOffset;
//     PUCHAR b;
//     PMI_FREED_SPECIAL_POOL FreedHeader;
//     LARGE_INTEGER TickCount;
//     PMMPFN Pfn;
//
//     DPRINT("MmFreeSpecialPool(%p)\n", P);
//
//     /* Get the PTE */
//     PointerPte = MiAddressToPte(P);
//
//     /* Check if it's valid */
//     if (PointerPte->u.Hard.Valid == 0)
//     {
//         /* Bugcheck if it has NOACCESS or 0 set as protection */
//         if (PointerPte->u.Soft.Protection() == MM_NOACCESS ||
//             !PointerPte->u.Soft.Protection())
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          (ULONG_PTR)P,
//                          (ULONG_PTR)PointerPte,
//                          0,
//                          0x20);
//         }
//     }
//
//     /* Determine if it's a underruns or overruns pool pointer */
//     PtrOffset = (ULONG)((ULONG_PTR)P & (PAGE_SIZE - 1));
//     if (PtrOffset)
//     {
//         /* Pool catches overruns */
//         Header = PAGE_ALIGN(P);
//         Overruns = TRUE;
//     }
//     else
//     {
//         /* Pool catches underruns */
//         Header = (PPOOL_HEADER)((PUCHAR)PAGE_ALIGN(P) + PAGE_SIZE - sizeof(POOL_HEADER));
//     }
//
//     /* Check if it's non paged pool */
//     if ((Header->Ulong1 & SPECIAL_POOL_PAGED) == 0)
//     {
//         /* Non-paged allocation, ensure that IRQ is not higher that DISPATCH */
//         PoolType = NonPagedPool;
//         ASSERT(PointerPte[1].u.Soft.PageFileHigh == SPECIAL_POOL_NONPAGED_PTE);
//         if (Irql > DISPATCH_LEVEL)
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          Irql,
//                          PoolType,
//                          (ULONG_PTR)P,
//                          0x31);
//         }
//     }
//     else
//     {
//         /* Paged allocation, ensure */
//         PoolType = PagedPool;
//         ASSERT(PointerPte[1].u.Soft.PageFileHigh == SPECIAL_POOL_PAGED_PTE);
//         if (Irql > APC_LEVEL)
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          Irql,
//                          PoolType,
//                          (ULONG_PTR)P,
//                          0x31);
//         }
//     }
//
//     /* Get amount of bytes user requested to be allocated by clearing out the paged mask */
//     BytesRequested = (Header->Ulong1 & ~SPECIAL_POOL_PAGED) & 0xFFFF;
//     ASSERT(BytesRequested <= PAGE_SIZE - sizeof(POOL_HEADER));
//
//     /* Check memory before the allocated user buffer in case of overruns detection */
//     if (Overruns)
//     {
//         /* Calculate the real placement of the buffer */
//         BytesReal = PAGE_SIZE - PtrOffset;
//
//         /* If they mismatch, it's unrecoverable */
//         if (BytesRequested > BytesReal)
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          (ULONG_PTR)P,
//                          BytesRequested,
//                          BytesReal,
//                          0x21);
//         }
//
//         if (BytesRequested + sizeof(POOL_HEADER) < BytesReal)
//         {
//             KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                          (ULONG_PTR)P,
//                          BytesRequested,
//                          BytesReal,
//                          0x22);
//         }
//
//         /* Actually check the memory pattern */
//         for (b = (PUCHAR)(Header + 1); b < (PUCHAR)P; b++)
//         {
//             if (*b != Header->BlockSize)
//             {
//                 /* Bytes mismatch */
//                 KeBugCheckEx(SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION,
//                              (ULONG_PTR)P,
//                              (ULONG_PTR)b,
//                              Header->BlockSize,
//                              0x23);
//             }
//         }
//     }
//
//     /* Check the memory pattern after the user buffer */
//     MiSpecialPoolCheckPattern(P, Header);
//
//     /* Fill the freed header */
//     KeQueryTickCount(&TickCount);
//     FreedHeader = (PMI_FREED_SPECIAL_POOL)PAGE_ALIGN(P);
//     FreedHeader->Signature = 0x98764321;
//     FreedHeader->TickCount = TickCount.__bindgen_anon_1.__bindgen_anon_1.LowPart;
//     FreedHeader->NumberOfBytesRequested = BytesRequested;
//     FreedHeader->Pagable = PoolType;
//     FreedHeader->VirtualAddress = P;
//     FreedHeader->Thread = PsGetCurrentThread();
//     /* TODO: Fill StackPointer and StackBytes */
//     FreedHeader->StackPointer = NULL;
//     FreedHeader->StackBytes = 0;
//
//     if (PoolType == NonPagedPool)
//     {
//         /* Non pagable. Get PFN element corresponding to the PTE */
//         Pfn = MI_PFN_ELEMENT(PointerPte->u.Hard.PageFrameNumber);
//
//         /* Count the page as free */
//         InterlockedDecrementUL(&MiSpecialPagesNonPaged);
//
//         /* Lock PFN database */
//         Irql = MiAcquirePfnLock();
//
//         /* Delete this PFN */
//         MI_SET_PFN_DELETED(Pfn);
//
//         /* Decrement share count of this PFN */
//         MiDecrementShareCount(Pfn, PointerPte->u.Hard.PageFrameNumber);
//
//         MI_ERASE_PTE(PointerPte);
//
//         /* Flush the TLB */
//         //FIXME: Use KeFlushSingleTb() instead
//         KeFlushEntireTb(TRUE, TRUE);
//     }
//     else
//     {
//         /* Pagable. Delete that virtual address */
//         MiDeleteSystemPageableVm(PointerPte, 1, 0, NULL);
//
//         /* Count the page as free */
//         InterlockedDecrementUL(&MiSpecialPagesPagable);
//
//         /* Lock PFN database */
//         Irql = MiAcquirePfnLock();
//     }
//
//     /* Mark next PTE as invalid */
//     MI_ERASE_PTE(PointerPte + 1);
//
//     /* Make sure that the last entry is really the last one */
//     ASSERT(MiSpecialPoolLastPte->u.List.NextEntry == MM_EMPTY_PTE_LIST);
//
//     /* Update the current last PTE next pointer */
//     MiSpecialPoolLastPte->u.List.NextEntry = PointerPte - MmSystemPteBase;
//
//     /* PointerPte becomes the new last PTE */
//     PointerPte->u.List.NextEntry = MM_EMPTY_PTE_LIST;
//     MiSpecialPoolLastPte = PointerPte;
//
//     /* Release the PFN database lock */
//     MiReleasePfnLock(Irql);
//
//     /* Update page counter */
//     InterlockedDecrementUL(&MmSpecialPagesInUse);
// }
//
// VOID
// NTAPI
// MiTestSpecialPool(VOID)
// {
//     ULONG i;
//     PVOID p1, p2[100];
//     //PUCHAR p3;
//     ULONG ByteSize;
//     POOL_TYPE PoolType = PagedPool;
//
//     // First allocate/free
//     for (i=0; i<100; i++)
//     {
//         ByteSize = (100 * (i+1)) % (PAGE_SIZE - sizeof(POOL_HEADER));
//         p1 = MmAllocateSpecialPool(ByteSize, 'TEST', PoolType, 0);
//         DPRINT1("p1 %p size %lu\n", p1, ByteSize);
//         MmFreeSpecialPool(p1);
//     }
//
//     // Now allocate all at once, then free at once
//     for (i=0; i<100; i++)
//     {
//         ByteSize = (100 * (i+1)) % (PAGE_SIZE - sizeof(POOL_HEADER));
//         p2[i] = MmAllocateSpecialPool(ByteSize, 'TEST', PoolType, 0);
//         DPRINT1("p2[%lu] %p size %lu\n", i, p1, ByteSize);
//     }
//     for (i=0; i<100; i++)
//     {
//         DPRINT1("Freeing %p\n", p2[i]);
//         MmFreeSpecialPool(p2[i]);
//     }
//
//     // Overrun the buffer to test
//     //ByteSize = 16;
//     //p3 = MmAllocateSpecialPool(ByteSize, 'TEST', NonPagedPool, 0);
//     //p3[ByteSize] = 0xF1; // This should cause an exception
//
//     // Underrun the buffer to test
//     //p3 = MmAllocateSpecialPool(ByteSize, 'TEST', NonPagedPool, 1);
//     //p3--;
//     //*p3 = 0xF1; // This should cause an exception
//
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
