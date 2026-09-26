use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion, ZwAllocateVirtualMemory, ZwFreeVirtualMemory};
use ntkrnl_sys::ps::PsGetCurrentProcess;
// Ex*PushLock* are DDK inlines (not exported symbols) — wrap the ntkrnl_sys inline impls.
#[inline(always)] unsafe fn ExInitializePushLock(p: PEX_PUSH_LOCK) { ntkrnl_sys::inline::ex::ExInitializePushLock(p as *mut _); }
#[inline(always)] unsafe fn ExAcquirePushLockExclusive(p: PEX_PUSH_LOCK) { ntkrnl_sys::inline::ex::ExAcquirePushLockExclusive(p as *mut _); }
#[inline(always)] unsafe fn ExReleasePushLockExclusive(p: PEX_PUSH_LOCK) { ntkrnl_sys::inline::ex::ExReleasePushLockExclusive(p as *mut _); }

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _GDI_POOL_SECTION {
    pub leInUseLink: _LIST_ENTRY,
    pub leReadyLink: _LIST_ENTRY,
    pub pvBaseAddress: PVOID,
    pub ulCommitBitmap: ULONG,
    pub cAllocCount: ULONG,
    pub bitmap: RTL_BITMAP,
    pub aulBits: [ULONG; 1],
}
pub type GDI_POOL_SECTION = _GDI_POOL_SECTION;
pub type PGDI_POOL_SECTION = *mut _GDI_POOL_SECTION;

#[repr(C)]
pub struct _GDI_POOL {
    pub ulTag: ULONG,
    pub cjAllocSize: ULONG,
    pub cjSectionSize: ULONG,
    pub cSlotsPerSection: ULONG,
    pub cEmptySections: ULONG,
    pub pushlock: EX_PUSH_LOCK,
    pub leInUseList: _LIST_ENTRY,
    pub leEmptyList: _LIST_ENTRY,
    pub leReadyList: _LIST_ENTRY,
}
pub type GDI_POOL = _GDI_POOL;
pub type PGDI_POOL = *mut _GDI_POOL;

use ntkrnl_sys::ddk::{RtlInitializeBitMap, RtlFindClearBitsAndSet, RtlClearBit, RtlClearAllBits};
// ---- dropped-preamble helpers (macros / intrinsics with no symbol) ----
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn NtCurrentProcess() -> HANDLE { (-1isize as usize) as HANDLE }
#[inline(always)] unsafe fn InitializeListHead(h: PLIST_ENTRY) { (*h).Flink = h; (*h).Blink = h; }
#[inline(always)] unsafe fn IsListEmpty(h: PLIST_ENTRY) -> u8 { ((*h).Flink == h) as u8 }
#[inline(always)] unsafe fn InsertHeadList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let f = (*h).Flink; (*e).Flink = f; (*e).Blink = h; (*f).Blink = e; (*h).Flink = e;
}
#[inline(always)] unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink; (*e).Flink = h; (*e).Blink = b; (*b).Flink = e; (*h).Blink = e;
}
#[inline(always)] unsafe fn RemoveEntryList(e: PLIST_ENTRY) -> u8 {
    let f = (*e).Flink; let b = (*e).Blink; (*b).Flink = f; (*f).Blink = b; (f == b) as u8
}
#[inline(always)] unsafe fn RemoveHeadList(h: PLIST_ENTRY) -> PLIST_ENTRY {
    let e = (*h).Flink; let f = (*e).Flink; (*h).Flink = f; (*f).Blink = h; e
}
// PAGE_ALIGN / ADDRESS_AND_SIZE_TO_SPAN_PAGES (PAGE_SIZE=0x1000, PAGE_SHIFT=12).
#[inline(always)] unsafe fn PAGE_ALIGN(va: PVOID) -> PVOID { ((va as usize) & !0xfff_usize) as PVOID }
#[inline(always)] fn ADDRESS_AND_SIZE_TO_SPAN_PAGES(va: PVOID, size: SIZE_T) -> ULONG {
    ((((va as usize) & 0xfff) + (size as usize) + 0xfff) >> 12) as ULONG
}

pub const GDI_POOL_ALLOCATION_GRANULARITY: ULONG = 64 * 1024;

unsafe fn GdiPoolAllocateSection(pPool: PGDI_POOL) -> PGDI_POOL_SECTION {
    let pSection: PGDI_POOL_SECTION;
    let mut pvBaseAddress: PVOID;
    let mut cjSize: SIZE_T;
    let status: NTSTATUS;

    cjSize = core::mem::size_of::<_GDI_POOL_SECTION>() as SIZE_T
        + ((*pPool).cSlotsPerSection / core::mem::size_of::<ULONG>() as ULONG) as SIZE_T;
    pSection = EngAllocMem(0, cjSize as _, (*pPool).ulTag) as PGDI_POOL_SECTION;
    if pSection.is_null() {
        return core::ptr::null_mut();
    }

    cjSize = (64 * 1024) as SIZE_T;
    pvBaseAddress = core::ptr::null_mut();
    // DC_ATTR/BRUSH_ATTR slots are user-visible; a WoW64 guest reads them
    // through 32-bit pointers, so reserve below 4GB for such processes.
    status = ZwAllocateVirtualMemory(
        -1isize as HANDLE,
        &mut pvBaseAddress,
        crate::init::W32Wow64ZeroBits(PsGetCurrentProcess() as _) as _,
        &mut cjSize,
        MEM_RESERVE,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(status) {
        EngFreeMem(pSection as PVOID);
        return core::ptr::null_mut();
    }

    (*pSection).pvBaseAddress = pvBaseAddress;
    (*pSection).ulCommitBitmap = 0;
    (*pSection).cAllocCount = 0;
    RtlInitializeBitMap(
        core::ptr::addr_of_mut!((*pSection).bitmap) as *mut _,
        core::ptr::addr_of_mut!((*pSection).aulBits) as *mut ULONG,
        (*pPool).cSlotsPerSection,
    );
    RtlClearAllBits(core::ptr::addr_of_mut!((*pSection).bitmap) as *mut _);

    pSection
}

unsafe fn GdiPoolDeleteSection(pPool: PGDI_POOL, pSection: PGDI_POOL_SECTION) {
    let mut cjSize: SIZE_T = 0;

    let status = ZwFreeVirtualMemory(
        -1isize as HANDLE,
        core::ptr::addr_of_mut!((*pSection).pvBaseAddress),
        &mut cjSize,
        MEM_RELEASE,
    );
    let _ = status;

    EngFreeMem(pSection as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn GdiPoolAllocate(pPool: PGDI_POOL) -> PVOID {
    let mut pSection: PGDI_POOL_SECTION;
    let ulIndex: ULONG;
    let cjOffset: ULONG;
    let mut ulPageBit: ULONG;
    let ple: PLIST_ENTRY;
    let mut pvAlloc: PVOID;
    let mut pvBaseAddress: PVOID;
    let mut cjSize: SIZE_T;
    let status: NTSTATUS;

    /* Disable APCs and acquire the pool lock */
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*pPool).pushlock));

    'done: {
        /* Check if we have a ready section */
        if IsListEmpty(core::ptr::addr_of_mut!((*pPool).leReadyList)) == 0 {
            /* Get a free section */
            ple = (*pPool).leReadyList.Flink;
            pSection = (ple as *mut u8)
                .sub(core::mem::offset_of!(_GDI_POOL_SECTION, leReadyLink))
                as PGDI_POOL_SECTION;
        } else {
            /* No, check if we have something on the empty list */
            if IsListEmpty(core::ptr::addr_of_mut!((*pPool).leEmptyList)) == 0 {
                /* Yes, remove it from the empty list */
                ple = RemoveHeadList(core::ptr::addr_of_mut!((*pPool).leEmptyList));
                pSection = (ple as *mut u8)
                    .sub(core::mem::offset_of!(_GDI_POOL_SECTION, leInUseLink))
                    as PGDI_POOL_SECTION;
                (*pPool).cEmptySections -= 1;
            } else {
                /* No, allocate a new section */
                pSection = GdiPoolAllocateSection(pPool);
                if pSection.is_null() {
                    pvAlloc = core::ptr::null_mut();
                    break 'done;
                }
            }

            /* Insert it into the in-use and ready list */
            InsertHeadList(
                core::ptr::addr_of_mut!((*pPool).leInUseList),
                core::ptr::addr_of_mut!((*pSection).leInUseLink),
            );
            InsertHeadList(
                core::ptr::addr_of_mut!((*pPool).leReadyList),
                core::ptr::addr_of_mut!((*pSection).leReadyLink),
            );
        }

        /* Find and set a single bit */
        ulIndex = RtlFindClearBitsAndSet(core::ptr::addr_of_mut!((*pSection).bitmap) as *mut _, 1, 0);

        /* Calculate the allocation address */
        cjOffset = ulIndex * (*pPool).cjAllocSize;
        pvAlloc = ((*pSection).pvBaseAddress as ULONG_PTR + cjOffset as ULONG_PTR) as PVOID;

        /* Check if memory is committed */
        ulPageBit = 1 << (cjOffset / PAGE_SIZE);
        ulPageBit |= 1 << ((cjOffset + (*pPool).cjAllocSize - 1) / PAGE_SIZE);
        if ((*pSection).ulCommitBitmap & ulPageBit) != ulPageBit {
            /* Commit the pages */
            pvBaseAddress = PAGE_ALIGN(pvAlloc);
            cjSize = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(pvAlloc, (*pPool).cjAllocSize as SIZE_T)
                * PAGE_SIZE) as SIZE_T;
            status = ZwAllocateVirtualMemory(
                NtCurrentProcess(),
                core::ptr::addr_of_mut!(pvBaseAddress),
                0,
                core::ptr::addr_of_mut!(cjSize),
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !NT_SUCCESS(status) {
                pvAlloc = core::ptr::null_mut();
                break 'done;
            }

            (*pSection).ulCommitBitmap |= ulPageBit;
        }

        /* Increase alloc count */
        (*pSection).cAllocCount += 1;

        /* Check if section is now busy */
        if (*pSection).cAllocCount == (*pPool).cSlotsPerSection {
            /* Remove the section from the ready list */
            RemoveEntryList(core::ptr::addr_of_mut!((*pSection).leReadyLink));
        }
    }

    /* Release the pool lock and enable APCs */
    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*pPool).pushlock));
    KeLeaveCriticalRegion();

    pvAlloc
}
#[no_mangle]
pub unsafe extern "system" fn GdiPoolFree(pPool: PGDI_POOL, pvAlloc: PVOID) {
    let mut ple: PLIST_ENTRY;
    let mut pSection: PGDI_POOL_SECTION = core::ptr::null_mut();
    let mut cjOffset: ULONG_PTR;
    let mut ulIndex: ULONG;

    /* Freeing NULL is a caller bug (the C asserts every attr it frees is non-NULL), and it came in
     * floods of thousands just before a NULL-region bugcheck. Name the first few callers. */
    if pvAlloc.is_null() {
        static NULL_FREES: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        if NULL_FREES.fetch_add(1, core::sync::atomic::Ordering::Relaxed) < 3 {
            let mut frames: [PVOID; 8] = [core::ptr::null_mut(); 8];
            let n = crate::gdidbg::DbgCaptureStackBackTace(frames.as_mut_ptr(), 0, 8);
            for f in &frames[..n as usize] {
                DbgPrint(b"[GDIPOOL] NULL free from %p\n\0".as_ptr() as *const i8, *f);
            }
        }
    }

    /* Disable APCs and acquire the pool lock */
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*pPool).pushlock) as *mut _);

    'done: {
        ple = (*pPool).leInUseList.Flink;
        while ple != core::ptr::addr_of_mut!((*pPool).leInUseList) {
            /* Get the pointer to the section */
            pSection = (ple as usize
                - core::mem::offset_of!(_GDI_POOL_SECTION, leInUseLink))
                as PGDI_POOL_SECTION;

            /* Calculate offset */
            cjOffset = (pvAlloc as ULONG_PTR) - ((*pSection).pvBaseAddress as ULONG_PTR);

            /* Check if the allocation is from this section */
            if cjOffset < (*pPool).cjSectionSize as ULONG_PTR {
                /* Calculate the index of the allocation */
                ulIndex = (cjOffset / (*pPool).cjAllocSize as ULONG_PTR) as ULONG;

                /* Mark it as free */
                RtlClearBit(core::ptr::addr_of_mut!((*pSection).bitmap) as *mut _, ulIndex);

                /* Decrease allocation count */
                (*pSection).cAllocCount -= 1;

                /* Check if the section got valid now */
                if (*pSection).cAllocCount == (*pPool).cSlotsPerSection - 1 {
                    /* Insert it into the ready list */
                    InsertTailList(
                        core::ptr::addr_of_mut!((*pPool).leReadyList),
                        core::ptr::addr_of_mut!((*pSection).leReadyLink),
                    );
                } else if (*pSection).cAllocCount == 0 {
                    /* Remove the section from the lists */
                    RemoveEntryList(core::ptr::addr_of_mut!((*pSection).leInUseLink));
                    RemoveEntryList(core::ptr::addr_of_mut!((*pSection).leReadyLink));

                    if (*pPool).cEmptySections >= 1 {
                        /* Delete the section */
                        GdiPoolDeleteSection(pPool, pSection);
                    } else {
                        /* Insert it into the empty list */
                        InsertHeadList(
                            core::ptr::addr_of_mut!((*pPool).leEmptyList),
                            core::ptr::addr_of_mut!((*pSection).leInUseLink),
                        );
                        (*pPool).cEmptySections += 1;
                    }
                }

                break 'done;
            }

            ple = (*ple).Flink;
        }

        DbgPrint(
            b"failed to free. pvAlloc=%p, base=%p, size=%lx\n\0".as_ptr() as *const i8,
            pvAlloc,
            if !pSection.is_null() {
                (*pSection).pvBaseAddress
            } else {
                core::ptr::null_mut()
            },
            (*pPool).cjSectionSize,
        );
    }

    /* Release the pool lock and enable APCs */
    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*pPool).pushlock) as *mut _);
    KeLeaveCriticalRegion();
}
#[no_mangle]
pub unsafe extern "system" fn GdiPoolCreate(
    cjAllocSize: ULONG,
    ulTag: ULONG,
) -> PGDI_POOL {
    let pPool = EngAllocMem(0, core::mem::size_of::<GDI_POOL>() as ULONG, u32::from_le_bytes(*b"Gpol")) as PGDI_POOL;
    if pPool.is_null() {
        return core::ptr::null_mut();
    }

    ExInitializePushLock(core::ptr::addr_of_mut!((*pPool).pushlock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!((*pPool).leInUseList));
    InitializeListHead(core::ptr::addr_of_mut!((*pPool).leReadyList));
    InitializeListHead(core::ptr::addr_of_mut!((*pPool).leEmptyList));
    (*pPool).cEmptySections = 0;
    (*pPool).cjAllocSize = cjAllocSize;
    (*pPool).ulTag = ulTag;
    (*pPool).cjSectionSize = (64 * 1024) as ULONG;
    (*pPool).cSlotsPerSection = (*pPool).cjSectionSize / cjAllocSize;

    pPool
}
#[no_mangle]
pub unsafe extern "system" fn GdiPoolDestroy(pPool: PGDI_POOL) {
    let mut pSection: PGDI_POOL_SECTION;
    let mut ple: PLIST_ENTRY;

    while IsListEmpty(core::ptr::addr_of_mut!((*pPool).leEmptyList)) == 0 {
        ple = RemoveHeadList(core::ptr::addr_of_mut!((*pPool).leEmptyList));
        pSection = (ple as *mut u8).offset(
            -(core::mem::offset_of!(_GDI_POOL_SECTION, leInUseLink) as isize),
        ) as PGDI_POOL_SECTION;
        GdiPoolDeleteSection(pPool, pSection);
    }

    while IsListEmpty(core::ptr::addr_of_mut!((*pPool).leInUseList)) == 0 {
        ple = RemoveHeadList(core::ptr::addr_of_mut!((*pPool).leInUseList));
        pSection = (ple as *mut u8).offset(
            -(core::mem::offset_of!(_GDI_POOL_SECTION, leInUseLink) as isize),
        ) as PGDI_POOL_SECTION;
        GdiPoolDeleteSection(pPool, pSection);
    }

    EngFreeMem(pPool as *mut core::ffi::c_void);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/gdipool.c
//  * PURPOSE:         Static size allocator for user mode object attributes
//  * PROGRAMMERS:     Timo Kreuzer
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _GDI_POOL_SECTION
// {
//     LIST_ENTRY leInUseLink;
//     LIST_ENTRY leReadyLink;
//
//     PVOID pvBaseAddress;
//
//     ULONG ulCommitBitmap;
//     ULONG cAllocCount;
//
//     RTL_BITMAP bitmap;
//     ULONG aulBits[1];
// } GDI_POOL_SECTION, *PGDI_POOL_SECTION;
//
// typedef struct _GDI_POOL
// {
//     ULONG ulTag;
//     ULONG cjAllocSize;
//     ULONG cjSectionSize; // 32 * cjAllocSize, rounded up to pages
//     ULONG cSlotsPerSection;
//     ULONG cEmptySections;
//     EX_PUSH_LOCK pushlock; // For pool growth
//
//     LIST_ENTRY leInUseList;
//     LIST_ENTRY leEmptyList;
//     LIST_ENTRY leReadyList;
// } GDI_POOL;
//
// #define GDI_POOL_ALLOCATION_GRANULARITY 64 * 1024
//
// static
// PGDI_POOL_SECTION
// GdiPoolAllocateSection(PGDI_POOL pPool)
// {
//     PGDI_POOL_SECTION pSection;
//     PVOID pvBaseAddress;
//     SIZE_T cjSize;
//     NTSTATUS status;
//
//     /* Allocate a section object */
//     cjSize = sizeof(GDI_POOL_SECTION) + pPool->cSlotsPerSection / sizeof(ULONG);
//     pSection = EngAllocMem(0, cjSize, pPool->ulTag);
//     if (!pSection)
//     {
//         return NULL;
//     }
//
//     /* Reserve user mode memory */
//     cjSize = GDI_POOL_ALLOCATION_GRANULARITY;
//     pvBaseAddress = NULL;
//     status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                      &pvBaseAddress,
//                                      0,
//                                      &cjSize,
//                                      MEM_RESERVE,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(status))
//     {
//         EngFreeMem(pSection);
//         return NULL;
//     }
//
//     /* Initialize the section */
//     pSection->pvBaseAddress = pvBaseAddress;
//     pSection->ulCommitBitmap = 0;
//     pSection->cAllocCount = 0;
//     RtlInitializeBitMap(&pSection->bitmap,
//                         pSection->aulBits,
//                         pPool->cSlotsPerSection);
//     RtlClearAllBits(&pSection->bitmap);
//
//     /* Return the section */
//     return pSection;
// }
//
// static
// VOID
// GdiPoolDeleteSection(PGDI_POOL pPool, PGDI_POOL_SECTION pSection)
// {
//     NTSTATUS status;
//     SIZE_T cjSize = 0;
//
//     /* Should not have any allocations */
//     if (pSection->cAllocCount != 0)
//     {
//         DPRINT1("There are %lu allocations left, section=%p, pool=%p\n",
//                 pSection->cAllocCount, pSection, pPool);
//         DBG_DUMP_EVENT_LIST(&pPool->slhLog);
//         ASSERT(FALSE);
//     }
//
//     /* Release the virtual memory */
//     status = ZwFreeVirtualMemory(NtCurrentProcess(),
//                                  &pSection->pvBaseAddress,
//                                  &cjSize,
//                                  MEM_RELEASE);
//     ASSERT(NT_SUCCESS(status));
//
//     /* Free the section object */
//     EngFreeMem(pSection);
// }
//
// PVOID
// NTAPI
// GdiPoolAllocate(
//     PGDI_POOL pPool)
// {
//     PGDI_POOL_SECTION pSection;
//     ULONG ulIndex, cjOffset, ulPageBit;
//     PLIST_ENTRY ple;
//     PVOID pvAlloc, pvBaseAddress;
//     SIZE_T cjSize;
//     NTSTATUS status;
//
//     /* Disable APCs and acquire the pool lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&pPool->pushlock);
//
//     /* Check if we have a ready section */
//     if (!IsListEmpty(&pPool->leReadyList))
//     {
//         /* Get a free section */
//         ple = pPool->leReadyList.Flink;
//         pSection = CONTAINING_RECORD(ple, GDI_POOL_SECTION, leReadyLink);
//         if (pSection->cAllocCount >= pPool->cSlotsPerSection)
//         {
//             DPRINT1("pSection->cAllocCount=%lu, pPool->cSlotsPerSection=%lu\n",
//                     pSection->cAllocCount, pPool->cSlotsPerSection);
//             DBG_DUMP_EVENT_LIST(&pPool->slhLog);
//             ASSERT(FALSE);
//         }
//         ASSERT(pSection->cAllocCount < pPool->cSlotsPerSection);
//     }
//     else
//     {
//         /* No, check if we have something on the empty list */
//         if (!IsListEmpty(&pPool->leEmptyList))
//         {
//             /* Yes, remove it from the empty list */
//             ple = RemoveHeadList(&pPool->leEmptyList);
//             pSection = CONTAINING_RECORD(ple, GDI_POOL_SECTION, leInUseLink);
//             pPool->cEmptySections--;
//             ASSERT(pSection->cAllocCount == 0);
//         }
//         else
//         {
//             /* No, allocate a new section */
//             pSection = GdiPoolAllocateSection(pPool);
//             if (!pSection)
//             {
//                 DPRINT1("Couldn't allocate a section\n");
//                 pvAlloc = NULL;
//                 goto done;
//             }
//         }
//
//         /* Insert it into the in-use and ready list */
//         InsertHeadList(&pPool->leInUseList, &pSection->leInUseLink);
//         InsertHeadList(&pPool->leReadyList, &pSection->leReadyLink);
//     }
//
//     /* Find and set a single bit */
//     ulIndex = RtlFindClearBitsAndSet(&pSection->bitmap, 1, 0);
//     ASSERT(ulIndex != MAXULONG);
//
//     /* Calculate the allocation address */
//     cjOffset = ulIndex * pPool->cjAllocSize;
//     pvAlloc = (PVOID)((ULONG_PTR)pSection->pvBaseAddress + cjOffset);
//
//     /* Check if memory is committed */
//     ulPageBit = 1 << (cjOffset / PAGE_SIZE);
//     ulPageBit |= 1 << ((cjOffset + pPool->cjAllocSize - 1) / PAGE_SIZE);
//     if ((pSection->ulCommitBitmap & ulPageBit) != ulPageBit)
//     {
//         /* Commit the pages */
//         pvBaseAddress = PAGE_ALIGN(pvAlloc);
//         cjSize = ADDRESS_AND_SIZE_TO_SPAN_PAGES(pvAlloc, pPool->cjAllocSize) * PAGE_SIZE;
//         status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                          &pvBaseAddress,
//                                          0,
//                                          &cjSize,
//                                          MEM_COMMIT,
//                                          PAGE_READWRITE);
//         if (!NT_SUCCESS(status))
//         {
//             pvAlloc = NULL;
//             goto done;
//         }
//
//         pSection->ulCommitBitmap |= ulPageBit;
//     }
//
//     /* Increase alloc count */
//     pSection->cAllocCount++;
//     ASSERT(RtlNumberOfSetBits(&pSection->bitmap) == pSection->cAllocCount);
//     DBG_LOGEVENT(&pPool->slhLog, EVENT_ALLOCATE, pvAlloc);
//
//     /* Check if section is now busy */
//     if (pSection->cAllocCount == pPool->cSlotsPerSection)
//     {
//         /* Remove the section from the ready list */
//         RemoveEntryList(&pSection->leReadyLink);
//     }
//
// done:
//     /* Release the pool lock and enable APCs */
//     ExReleasePushLockExclusive(&pPool->pushlock);
//     KeLeaveCriticalRegion();
//
//     DPRINT("GdiPoolallocate: %p\n", pvAlloc);
//     return pvAlloc;
// }
//
// VOID
// NTAPI
// GdiPoolFree(
//     PGDI_POOL pPool,
//     PVOID pvAlloc)
// {
//     PLIST_ENTRY ple;
//     PGDI_POOL_SECTION pSection = NULL;
//     ULONG_PTR cjOffset;
//     ULONG ulIndex;
//     DPRINT("GdiPoolFree: %p\n", pvAlloc);
//
//     /* Disable APCs and acquire the pool lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&pPool->pushlock);
//
//     /* Loop all used sections */
//     for (ple = pPool->leInUseList.Flink;
//          ple != &pPool->leInUseList;
//          ple = ple->Flink)
//     {
//         /* Get the pointer to the section */
//         pSection = CONTAINING_RECORD(ple, GDI_POOL_SECTION, leInUseLink);
//
//         /* Calculate offset */
//         cjOffset = (ULONG_PTR)pvAlloc - (ULONG_PTR)pSection->pvBaseAddress;
//
//         /* Check if the allocation is from this section */
//         if (cjOffset < pPool->cjSectionSize)
//         {
//             /* Calculate the index of the allocation */
//             ulIndex = cjOffset / pPool->cjAllocSize;
//
//             /* Mark it as free */
//             ASSERT(RtlTestBit(&pSection->bitmap, ulIndex) == TRUE);
//             RtlClearBit(&pSection->bitmap, ulIndex);
//
//             /* Decrease allocation count */
//             pSection->cAllocCount--;
//             ASSERT(RtlNumberOfSetBits(&pSection->bitmap) == pSection->cAllocCount);
//             DBG_LOGEVENT(&pPool->slhLog, EVENT_FREE, pvAlloc);
//
//             /* Check if the section got valid now */
//             if (pSection->cAllocCount == pPool->cSlotsPerSection - 1)
//             {
//                 /* Insert it into the ready list */
//                 InsertTailList(&pPool->leReadyList, &pSection->leReadyLink);
//             }
//             /* Check if it got empty now */
//             else if (pSection->cAllocCount == 0)
//             {
//                 /* Remove the section from the lists */
//                 RemoveEntryList(&pSection->leInUseLink);
//                 RemoveEntryList(&pSection->leReadyLink);
//
//                 if (pPool->cEmptySections >= 1)
//                 {
//                     /* Delete the section */
//                     GdiPoolDeleteSection(pPool, pSection);
//                 }
//                 else
//                 {
//                     /* Insert it into the empty list */
//                     InsertHeadList(&pPool->leEmptyList, &pSection->leInUseLink);
//                     pPool->cEmptySections++;
//                 }
//             }
//
//             goto done;
//         }
//     }
//
//     DbgPrint("failed to free. pvAlloc=%p, base=%p, size=%lx\n",
//              pvAlloc, pSection ? pSection->pvBaseAddress : NULL, pPool->cjSectionSize);
//     ASSERT(FALSE);
//     // KeBugCheck()
//
// done:
//     /* Release the pool lock and enable APCs */
//     ExReleasePushLockExclusive(&pPool->pushlock);
//     KeLeaveCriticalRegion();
// }
//
// PGDI_POOL
// NTAPI
// GdiPoolCreate(
//     ULONG cjAllocSize,
//     ULONG ulTag)
// {
//     PGDI_POOL pPool;
//
//     /* Allocate a pool object */
//     pPool = EngAllocMem(0, sizeof(GDI_POOL), 'lopG');
//     if (!pPool) return NULL;
//
//     /* Initialize the object */
//     ExInitializePushLock(&pPool->pushlock);
//     InitializeListHead(&pPool->leInUseList);
//     InitializeListHead(&pPool->leReadyList);
//     InitializeListHead(&pPool->leEmptyList);
//     pPool->cEmptySections = 0;
//     pPool->cjAllocSize = cjAllocSize;
//     pPool->ulTag = ulTag;
//     pPool->cjSectionSize = GDI_POOL_ALLOCATION_GRANULARITY;
//     pPool->cSlotsPerSection = pPool->cjSectionSize / cjAllocSize;
//     DBG_INITLOG(&pPool->slhLog);
//
//     return pPool;
// }
//
// VOID
// NTAPI
// GdiPoolDestroy(PGDI_POOL pPool)
// {
//     PGDI_POOL_SECTION pSection;
//     PLIST_ENTRY ple;
//
//     /* Loop all empty sections, removing them */
//     while (!IsListEmpty(&pPool->leEmptyList))
//     {
//         /* Delete the section */
//         ple = RemoveHeadList(&pPool->leEmptyList);
//         pSection = CONTAINING_RECORD(ple, GDI_POOL_SECTION, leInUseLink);
//         GdiPoolDeleteSection(pPool, pSection);
//     }
//
//     /* Loop all ready sections, removing them */
//     while (!IsListEmpty(&pPool->leInUseList))
//     {
//         /* Delete the section */
//         ple = RemoveHeadList(&pPool->leInUseList);
//         pSection = CONTAINING_RECORD(ple, GDI_POOL_SECTION, leInUseLink);
//         GdiPoolDeleteSection(pPool, pSection);
//     }
//
//     DBG_CLEANUP_EVENT_LIST(&pPool->slhLog);
//
//     EngFreeMem(pPool);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
