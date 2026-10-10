use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    _KTHREAD, KEVENT, PKTHREAD, PRKEVENT,
    KeSetPriorityThread,
    KeWaitForMultipleObjects,
    KeZeroPages,
    KeClearEvent,
};

#[no_mangle]
pub static mut MmZeroingPageEvent: KEVENT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn MmZeroPageThread() {
    let thread: PKTHREAD = KeGetCurrentThread() as PKTHREAD;
    let mut start_address: PVOID = core::ptr::null_mut();
    let mut end_address: PVOID = core::ptr::null_mut();
    let mut wait_objects: [PVOID; 2] = [core::ptr::null_mut(); 2];

    MiFindInitializationCode(
        core::ptr::addr_of_mut!(start_address),
        core::ptr::addr_of_mut!(end_address),
    );
    if !start_address.is_null() {
        MiFreeInitializationCode(start_address, end_address);
    }

    (*(thread as *mut _KTHREAD)).BasePriority = 0;
    KeSetPriorityThread(thread, 0);

    wait_objects[0] = core::ptr::addr_of_mut!(MmZeroingPageEvent) as PVOID;

    loop {
        let mut old_irql: KIRQL;

        KeWaitForMultipleObjects(
            1,
            wait_objects.as_mut_ptr(),
            1, // WaitAny (WAIT_TYPE: WaitAll=0, WaitAny=1 — harmless at Count=1 but a trap if a second wait object is added)
            8, // WrFreePage
            0, // KernelMode
            0u8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        old_irql = MiAcquirePfnLock();

        loop {
            let mut page_count: ULONG = 0;
            let mut pfn1: PMMPFN = LIST_HEAD as PMMPFN;
            let zero_address: PVOID;
            let mut page_index: PFN_NUMBER;
            let mut free_page: PFN_NUMBER;

            while page_count < MI_ZERO_PTES {
                let pfn2: PMMPFN;

                if MmFreePageListHead.Total == 0 {
                    break;
                }

                page_index = MmFreePageListHead.Flink;
                ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_ZERO_LOOP);
                ntkrnl_sys::MI_SET_PROCESS2!("Kernel 0 Loop");
                free_page = MiRemoveAnyPage(MI_GET_PAGE_COLOR(page_index as ULONG));

                if free_page != page_index {
                    KeBugCheckEx(
                        PFN_LIST_CORRUPT,
                        0x8F,
                        free_page as ULONG_PTR,
                        page_index as ULONG_PTR,
                        0,
                    );
                }

                pfn2 = MiGetPfnEntry(page_index);
                (*pfn2).set_flink(pfn1 as PFN_NUMBER);
                pfn1 = pfn2;
                page_count += 1;
            }
            MiReleasePfnLock(old_irql);

            if page_count == 0 {
                KeClearEvent(core::ptr::addr_of_mut!(MmZeroingPageEvent) as PRKEVENT);
                break;
            }

            zero_address = MiMapPagesInZeroSpace(pfn1, page_count as PFN_NUMBER);
            KeZeroPages(zero_address, (page_count as usize * PAGE_SIZE) as ULONG);
            MiUnmapPagesInZeroSpace(zero_address, page_count as PFN_NUMBER);

            old_irql = MiAcquirePfnLock();

            while pfn1 != LIST_HEAD as PMMPFN {
                page_index = MiGetPfnEntryIndex(pfn1);
                pfn1 = (*pfn1).flink() as PMMPFN;
                MiInsertPageInList(
                    core::ptr::addr_of_mut!(MmZeroedPageListHead),
                    page_index,
                );
            }
        }
    }
}
