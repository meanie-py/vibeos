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
    // The C loops "while dirty" with no bound, so a page that never comes clean -- written
    // and re-dirtied, or never writable -- spins here and the reboot never comes. Count the
    // dirty pages per pass, and name the stubborn ones once the loop should have converged.
    let mut pass: u32 = 0;
    loop {
        let mut dirty: BOOLEAN = 0u8;
        let mut dirty_pages: u32 = 0;
        pass += 1;

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
                        dirty_pages += 1;
                        if (pass == 3 || pass == 4) && dirty_pages <= 8 {
                            ntkrnl_sys::driver_ddk::DbgPrint(
                                c"[MM] MiShutdownSystem: pass %u still dirty: %wZ at %I64x (sse %Ix)\n".as_ptr(),
                                pass,
                                core::ptr::addr_of!((*(*segment).FileObject).FileName),
                                segment_offset.QuadPart,
                                entry,
                            );
                        }
                        MmCheckDirtySegment(segment, &mut segment_offset, 0u8, 1u8);
                    }

                    MmUnlockSectionSegment(segment as PVOID);
                }

                MmDereferenceSegment(segment);
            }

            page = MmGetLRUNextUserPage(page as _, 0u8) as PFN_NUMBER;
        }

        if pass <= 5 || pass % 50 == 0 {
            ntkrnl_sys::driver_ddk::DbgPrint(
                c"[MM] MiShutdownSystem: pass %u wrote %u dirty data-file page(s)\n".as_ptr(),
                pass,
                dirty_pages,
            );
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
