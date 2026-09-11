use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    _LARGE_INTEGER,
    _MMPAGING_FILE,
    MmCheckDirtySegment,
    MmDereferenceSegment,
    MmGetLRUFirstUserPage,
    MmGetLRUNextUserPage,
    MmGetSectionAssociation,
    PMM_SECTION_SEGMENT,
    ZwClose,
};

unsafe fn MiShutdownSystem() {
    let mut i: ULONG;

    // Loop through all the paging files
    i = 0;
    while i < MmNumberOfPagingFiles {
        let pf = MmPagingFile[i as usize] as *mut _MMPAGING_FILE;
        // Free page file name
        ntkrnl_sys::ASSERT!((*pf).PageFileName.Buffer != core::ptr::null_mut());
        ExFreePoolWithTag((*pf).PageFileName.Buffer as PVOID, TAG_MM);
        (*pf).PageFileName.Buffer = core::ptr::null_mut();

        // And close them
        ZwClose((*pf).FileHandle);

        i += 1;
    }

    // Loop through all the pages owned by the legacy Mm and page them out, if needed.
    // We do it as long as there are dirty pages, since flushing can cause the FS to dirtify new ones.
    loop {
        let mut dirty: BOOLEAN = 0u8;

        let mut page: PFN_NUMBER = MmGetLRUFirstUserPage() as PFN_NUMBER;
        while page != 0 {
            let mut segment_offset: _LARGE_INTEGER = core::mem::zeroed();
            let segment: PMM_SECTION_SEGMENT = MmGetSectionAssociation(
                page as _,
                &mut segment_offset,
            );

            if !segment.is_null() {
                if (*((*segment).Flags)) & MM_DATAFILE_SEGMENT != 0 {
                    MmLockSectionSegment(segment as PVOID);

                    let entry: ULONG_PTR = MmGetPageEntrySectionSegment(
                        segment as PVOID,
                        &segment_offset.QuadPart as *const i64 as *const u64,
                    );

                    if !IS_SWAP_FROM_SSE(entry) && IS_DIRTY_SSE(entry) {
                        dirty = 1u8;
                        MmCheckDirtySegment(segment, &mut segment_offset, 0u8, 1u8);
                    }

                    MmUnlockSectionSegment(segment as PVOID);
                }

                MmDereferenceSegment(segment);
            }

            page = MmGetLRUNextUserPage(page as _, 0u8) as PFN_NUMBER;
        }

        if dirty == 0 {
            break;
        }
    }
}

// MmShutdownSystem is defined without NTAPI in C (cdecl) — use extern "C".
// On i386, "system" = stdcall (_MmShutdownSystem@4) but C expects cdecl (_MmShutdownSystem).
#[no_mangle]
pub unsafe extern "C" fn MmShutdownSystem(Phase: ULONG) {
    if Phase == 0 {
        MiShutdownSystem();
    } else if Phase == 1 {
        let mut i: ULONG;

        // Loop through all the paging files
        i = 0;
        while i < MmNumberOfPagingFiles {
            let pf = MmPagingFile[i as usize] as *mut _MMPAGING_FILE;
            // And dereference them
            ObDereferenceObject((*pf).FileObject as PVOID);

            i += 1;
        }
    } else {
        ntkrnl_sys::ASSERT!(Phase == 2);

        // UNIMPLEMENTED
    }
}
