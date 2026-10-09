use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PFILE_OBJECT, PSECTION_OBJECT_POINTERS};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FAT_BUG_CHECK_FILOBSUP, FAT_DIRENT_ATTR_VOLUME_ID, FCB, IRP_CONTEXT, VCB, _TYPE_OF_OPEN as TYPE_OF_OPEN};
use ntkrnl_sys::ddk::{FILE_OBJECT, SECTION_OBJECT_POINTERS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_FILOBSUP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DCB};

extern "system" {
    } extern "C" { fn DebugTrace(level: i32, dbg: u32, fmt: *const i8, ...); } extern "system" {
    fn NT_ASSERT(condition: i32);
    fn FatFspClose(vcb: *mut VCB);
    fn FatGetNextFcbTopDown( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, OriginalFcb: *mut FCB, ) -> *mut FCB;
    fn FlagOn(flags: u32, flag: u32) -> i32;
}
extern "system" {
    fn NodeType(ptr: *mut core::ffi::c_void) -> u32;
    fn SetFlag(flags: *mut u32, flag: u32);
    fn ARGUMENT_PRESENT(p: *mut core::ffi::c_void) -> u8;
    fn FatBugCheck(arg1: u32, arg2: u32, arg3: u32) -> !;
    fn FatVcbAcquiredExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u8;
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: i32);
    fn ClearFlag(State: *mut u32, Bit: u32);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8;
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn CcPurgeCacheSection( Section: *mut SECTION_OBJECT_POINTERS, FileOffset: *mut LARGE_INTEGER, Length: u32, UninitializeCacheMaps: i32, ) -> u8;
    fn MmFlushImageSection( Section: *mut SECTION_OBJECT_POINTERS, FlushType: u32, ) -> u8;
    fn FatFlushFile( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, FlushType: FAT_FLUSH_TYPE, ) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_FILOBSUP;
pub const Dbg: u32 = DEBUG_TRACE_FILOBSUP;

// _TYPE_OF_OPEN (fatprocs.h / crate::types) is 1-based: UnopenedFileObject = 1.
// These were mistranslated as a 0-based enum (UnopenedFileObject=0, UserFileOpen=1,
// EaFile=2...), so FatDecodeFileObject RETURNED filobsup's UserFileOpen(1) for a real
// file while every consumer (read/write/cleanup, using crate::types) reads 1 as
// UnopenedFileObject → "Illegal TypeOfOpen" FatBugCheck 0x23 on the first paging read.
const UnopenedFileObject: i32 = 1;
const UserFileOpen: i32 = 2;
const UserDirectoryOpen: i32 = 3;
const UserVolumeOpen: i32 = 4;
const VirtualVolumeFile: i32 = 5;
const DirectoryFile: i32 = 6;
const EaFile: i32 = 7;

const FAT_NTC_VCB: u32 = 0x0501;
const FAT_NTC_FCB: u32 = 0x0502;
const FAT_NTC_DCB: u32 = 0x0503;
const FAT_NTC_ROOT_DCB: u32 = 0x0504;
const FAT_NTC_CCB: u32 = 0x0507;

const FO_TEMPORARY_FILE: u32 = 0x8000;
const FCB_STATE_TEMPORARY: u32 = 0x20; // placeholder, real value unknown

const STATUS_CANT_WAIT: i32 = 0xC00000D8_u32 as i32; // 0xC0000258
const VCB_STATE_FLAG_LOCKED: u32 = 0x01; // FIXME: unknown value
const VCB_STATE_FLAG_DELETED_FCB: u32 = 0x0100; // FIXME: unknown value
const FCB_STATE_FORCE_MISS_IN_PROGRESS: u32 = 0x08; // FIXME: unknown value
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // FIXME: unknown value  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const MmFlushForWrite: u32 = 0x00000002;

// FAT_FLUSH_TYPE enum from fatprocs.h
type FAT_FLUSH_TYPE = u32;
#[no_mangle]
pub unsafe extern "system" fn FatSetFileObject(
    FileObject: *mut FILE_OBJECT,
    TypeOfOpen: TYPE_OF_OPEN,
    VcbOrFcbOrDcb: *mut core::ffi::c_void,
    Ccb: *mut CCB,
) {
    unsafe {
        // PAGED_CODE(); (not available, assume call)
        PAGED_CODE!();

        DebugTrace(
            1,
            Dbg,
            b"FatSetFileObject, FileObject = %p\n\0".as_ptr() as *const i8,
            FileObject,
        );

        // NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB))
        {
            let cond_ccb: i32 = if Ccb.is_null() || (NodeType(Ccb as *mut core::ffi::c_void) == FAT_NTC_CCB) {
                1
            } else {
                0
            };
            NT_ASSERT(cond_ccb);
        }

        // Complex type-of-open assertion
        {
            let node_type: u32 = if !Ccb.is_null() {
                NodeType(Ccb as *mut core::ffi::c_void)
            } else {
                0 // won't be used in the null branch
            };
            let cond = (TypeOfOpen == (UnopenedFileObject) as u32)
                || ((TypeOfOpen == (UserFileOpen) as u32)
                    && (NodeType(VcbOrFcbOrDcb) == FAT_NTC_FCB)
                    && !Ccb.is_null())
                || ((TypeOfOpen == (EaFile) as u32)
                    && (NodeType(VcbOrFcbOrDcb) == FAT_NTC_FCB)
                    && Ccb.is_null())
                || ((TypeOfOpen == (UserDirectoryOpen) as u32)
                    && ((NodeType(VcbOrFcbOrDcb) == FAT_NTC_DCB)
                        || (NodeType(VcbOrFcbOrDcb) == FAT_NTC_ROOT_DCB))
                    && !Ccb.is_null())
                || ((TypeOfOpen == (UserVolumeOpen) as u32)
                    && (NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB)
                    && !Ccb.is_null())
                || ((TypeOfOpen == (VirtualVolumeFile) as u32)
                    && (NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB)
                    && Ccb.is_null())
                || ((TypeOfOpen == (DirectoryFile) as u32)
                    && ((NodeType(VcbOrFcbOrDcb) == FAT_NTC_DCB)
                        || (NodeType(VcbOrFcbOrDcb) == FAT_NTC_ROOT_DCB))
                    && Ccb.is_null());

            NT_ASSERT(if cond { 1 } else { 0 });
        }

        // UNREFERENCED_PARAMETER( TypeOfOpen );
        let _ = TypeOfOpen;

        // NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB))
        {
            let cond_ccb: i32 = if Ccb.is_null() || (NodeType(Ccb as *mut core::ffi::c_void) == FAT_NTC_CCB) {
                1
            } else {
                0
            };
            NT_ASSERT(cond_ccb);
        }

        if !VcbOrFcbOrDcb.is_null() {
            // Set Vpb
            if NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB {
                (*(FileObject)).Vpb = (*(VcbOrFcbOrDcb as *mut VCB)).Vpb;
            } else {
                (*(FileObject)).Vpb = (*((*(VcbOrFcbOrDcb as *mut FCB)).Vcb)).Vpb;

                // Check temporary file flag
                let fcb_state = (*(VcbOrFcbOrDcb as *mut FCB)).FcbState;
                if FlagOn(fcb_state, FCB_STATE_TEMPORARY) != 0 {
                    SetFlag(
                        &raw mut (*FileObject).Flags,
                        FO_TEMPORARY_FILE,
                    );
                }
            }
        }

        // NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB))
        {
            let cond_ccb: i32 = if Ccb.is_null() || (NodeType(Ccb as *mut core::ffi::c_void) == FAT_NTC_CCB) {
                1
            } else {
                0
            };
            NT_ASSERT(cond_ccb);
        }

        // Set FsContext fields if FileObject present
        if ARGUMENT_PRESENT(FileObject as *mut core::ffi::c_void) != 0 {
            (*FileObject).FsContext = VcbOrFcbOrDcb;
            (*FileObject).FsContext2 = (Ccb).cast();
        }

        // NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB))
        {
            let cond_ccb: i32 = if Ccb.is_null() || (NodeType(Ccb as *mut core::ffi::c_void) == FAT_NTC_CCB) {
                1
            } else {
                0
            };
            NT_ASSERT(cond_ccb);
        }

        DebugTrace(-1, Dbg, b"FatSetFileObject -> VOID\n\0".as_ptr() as *const i8, 0);
        // return;
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatDecodeFileObject(
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut *mut VCB,
    FcbOrDcb: *mut *mut FCB,
    Ccb: *mut *mut CCB,
) -> TYPE_OF_OPEN {
    // PAGED_CODE(); -- no‑op for this translation

    let TypeOfOpen: TYPE_OF_OPEN;
    let FsContext: *mut core::ffi::c_void;
    let FsContext2: *mut core::ffi::c_void;

    FsContext = (*FileObject).FsContext;
    FsContext2 = (*FileObject).FsContext2;

    if FsContext.is_null() {
        *Ccb = core::ptr::null_mut();
        *FcbOrDcb = core::ptr::null_mut();
        *Vcb = core::ptr::null_mut();
        TypeOfOpen = (UnopenedFileObject) as u32;
    } else {
        match NodeType(FsContext) {
            FAT_NTC_VCB => {
                *Ccb = FsContext2 as *mut CCB;
                *FcbOrDcb = core::ptr::null_mut();
                *Vcb = FsContext as *mut VCB;
                if (*Ccb).is_null() {
                    TypeOfOpen = (VirtualVolumeFile) as u32;
                } else {
                    TypeOfOpen = (UserVolumeOpen) as u32;
                }
            }
            FAT_NTC_ROOT_DCB | FAT_NTC_DCB => {
                *Ccb = FsContext2 as *mut CCB;
                *FcbOrDcb = FsContext as *mut FCB;
                *Vcb = (*(*FcbOrDcb)).Vcb;
                if (*Ccb).is_null() {
                    TypeOfOpen = (DirectoryFile) as u32;
                } else {
                    TypeOfOpen = (UserDirectoryOpen) as u32;
                }
                DebugTrace(
                    0,
                    Dbg,
                    b"Referencing directory: %wZ\n\0".as_ptr() as *const i8,
                    core::ptr::addr_of!((*(*FcbOrDcb)).FullFileName) as *const _,
                );
            }
            FAT_NTC_FCB => {
                *Ccb = FsContext2 as *mut CCB;
                *FcbOrDcb = FsContext as *mut FCB;
                *Vcb = (*(*FcbOrDcb)).Vcb;
                if !(*Ccb).is_null() {
                    TypeOfOpen = (UserFileOpen) as u32;
                    DebugTrace(
                        0,
                        Dbg,
                        b"Referencing file: %wZ\n\0".as_ptr() as *const i8,
                        core::ptr::addr_of!((*(*FcbOrDcb)).FullFileName) as *const _,
                    );
                } else {
                    if *FcbOrDcb == (*(*Vcb)).EaFcb {
                        TypeOfOpen = (EaFile) as u32;
                        DebugTrace(
                            0,
                            Dbg,
                            b"Referencing EA file: %wZ\n\0".as_ptr() as *const i8,
                            core::ptr::addr_of!((*(*FcbOrDcb)).FullFileName) as *const _,
                        );
                    } else {
                        // #ifdef _MSC_VER
                        // #pragma prefast(suppress:28159, "things are seriously wrong if we get here")
                        // #endif
                        FatBugCheck(NodeType(FsContext), 0, 0);
                    }
                }
            }
            _ => {
                // #ifdef _MSC_VER
                // #pragma prefast(suppress:28159, "things are seriously wrong if we get here")
                // #endif
                FatBugCheck(NodeType(FsContext), 0, 0);
            }
        }
    }

    DebugTrace(
        0,
        Dbg,
        b"FatDecodeFileObject -> VCB(%p)\n\0".as_ptr() as *const i8,
        *Vcb,
    );
    DebugTrace(
        0,
        Dbg,
        b"FatDecodeFileObject -> FCB(%p)\n\0".as_ptr() as *const i8,
        *FcbOrDcb,
    );
    DebugTrace(
        0,
        Dbg,
        b"FatDecodeFileObject -> CCB(%p)\n\0".as_ptr() as *const i8,
        *Ccb,
    );
    DebugTrace(
        -1,
        Dbg,
        b"FatDecodeFileObject -> TypeOfOpen = %08lx\n\0".as_ptr() as *const i8,
        TypeOfOpen,
    );

    TypeOfOpen
}
#[no_mangle]
pub unsafe extern "system" fn FatPurgeReferencedFileObjects(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Flags: u32,
) {
    let mut Fcb = Fcb;
    let OriginalFcb = Fcb;
    let mut NextFcb: *mut FCB = core::ptr::null_mut();

    PAGED_CODE!();

    DebugTrace(
        1,
        Dbg,
        b"FatPurgeReferencedFileObjects, Fcb = %p\n\0".as_ptr() as *const i8,
        Fcb,
    );

    NT_ASSERT(FlagOn(
        (*IrpContext).Flags,
        IRP_CONTEXT_FLAG_WAIT,
    ) as i32);

    // First, if we have a delayed close, force it closed.
    FatFspClose((*Fcb).Vcb);

    // Walk the directory tree forcing sections closed.
    // Note that it very important to get the next node to visit before
    // acting on the current node.
    while Fcb != core::ptr::null_mut() {
        NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, OriginalFcb as *mut DCB);

        // Check for the EA file fcb
        if FlagOn(
            (((*Fcb).DirentFatFlags) as u8) as u32,
            FAT_DIRENT_ATTR_VOLUME_ID,
        ) == 0 {
            FatForceCacheMiss(IrpContext, Fcb, Flags);
        }

        Fcb = NextFcb;
    }

    DebugTrace(
        -1,
        Dbg,
        b"FatPurgeReferencedFileObjects (VOID)\n\0".as_ptr() as *const i8,
        0 as i32,
    );

    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatForceCacheMiss(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FlushType: FAT_FLUSH_TYPE,
) {
    let mut ChildrenAcquired: i32 = 0; // FALSE
    let Vcb: *mut VCB;

    PAGED_CODE!();

    //
    //  If we can't wait, bail.
    //

    let vcb_assert = (*Fcb).Vcb;
    NT_ASSERT((FatVcbAcquiredExclusive(IrpContext, vcb_assert) != 0
        || FlagOn((*vcb_assert).VcbState, VCB_STATE_FLAG_LOCKED) != 0) as i32);

    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) == 0 {
        FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
    }

    //
    //  If we are purging a directory file object, we must acquire all the
    //  FCBs exclusive so that the parent directory is not being pinned.
    //  Careful, we can collide with something acquiring up the tree like
    //  an unpin repinned flush (FsRtlAcquireFileForCcFlush ...) of a parent
    //  dir on extending writethrough of a child file (oops).  So get things
    //  going up the tree, not down.
    //

    let mut queue_ptr: *mut LIST_ENTRY = core::ptr::null_mut();

    if NodeType((Fcb).cast()) != FAT_NTC_FCB
        && !IsListEmpty(
            core::ptr::addr_of!((*Fcb).Specific.Dcb.as_ref().ParentDcbQueue) as *mut LIST_ENTRY,
        )
    {
        queue_ptr = core::ptr::addr_of!((*Fcb).Specific.Dcb.as_ref().ParentDcbQueue)
            as *mut LIST_ENTRY;

        ChildrenAcquired = 1;

        let mut Links: *mut LIST_ENTRY = (*queue_ptr).Flink;
        while Links != queue_ptr {
            let TempFcb = (Links as usize
                - core::mem::offset_of!(FCB, ParentDcbLinks))
                as *mut FCB;

            let _ = FatAcquireExclusiveFcb(IrpContext, TempFcb);
            Links = (*Links).Flink;
        }
    }

    let _ = FatAcquireExclusiveFcb(IrpContext, Fcb);

    //
    //  We use this flag to indicate to a close beneath us that
    //  the Fcb resource should be freed before deleting the Fcb.
    //

    Vcb = (*Fcb).Vcb;

    SetFlag(&mut (*Fcb).FcbState, FCB_STATE_FORCE_MISS_IN_PROGRESS);

    ClearFlag(&mut (*Vcb).VcbState, VCB_STATE_FLAG_DELETED_FCB);

    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| {
        if (FlushType as u32) != 0 {
            let _ = FatFlushFile(IrpContext, Fcb, FlushType as u32);
        }

        //
        //  The Flush may have made the Fcb go away
        //

        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_DELETED_FCB) == 0 {
            let Section = core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers);

            let DataSectionExists: i32 = if (*Section).DataSectionObject
                != core::ptr::null_mut()
            {
                1
            } else {
                0
            };
            let ImageSectionExists: i32 = if (*Section).ImageSectionObject
                != core::ptr::null_mut()
            {
                1
            } else {
                0
            };

            //
            //  Note, it is critical to do the Image section first as the
            //  purge of the data section may cause the image section to go
            //  away, but the opposite is not true.
            //

            if ImageSectionExists != 0 {
                let _ = MmFlushImageSection(Section, MmFlushForWrite);
            }

            if DataSectionExists != 0 {
                let _ = CcPurgeCacheSection(
                    Section,
                    core::ptr::null_mut::<LARGE_INTEGER>(),
                    0,
                    0,
                );
            }
        }
    }, |_, _abnormal| {
        //
        //  If we purging a directory file object, release all the Fcb
        //  resources that we acquired above.  The Dcb cannot have vanished
        //  if there were Fcbs underneath it, and the Fcbs couldn't have gone
        //  away since I own the Vcb.
        //

        if ChildrenAcquired != 0 {
            let mut Links: *mut LIST_ENTRY = (*queue_ptr).Flink;

            while Links != queue_ptr {
                let TempFcb = (Links as usize
                    - core::mem::offset_of!(FCB, ParentDcbLinks))
                    as *mut FCB;

                FatReleaseFcb(IrpContext, TempFcb);
                Links = (*Links).Flink;
            }
        }

        //
        //  Since we have the Vcb exclusive we know that if any closes
        //  come in it is because the CcPurgeCacheSection caused the
        //  Fcb to go away.  Also in close, the Fcb was released
        //  before being freed.
        //

        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_DELETED_FCB) == 0 {
            ClearFlag(
                &mut (*Fcb).FcbState,
                FCB_STATE_FORCE_MISS_IN_PROGRESS,
            );

            FatReleaseFcb(IrpContext, Fcb);
        }
    });
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     FilObSup.c
//
// Abstract:
//
//     This module implements the Fat File object support routines.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_FILOBSUP)
//
// //
// //  The debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_FILOBSUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatForceCacheMiss)
// #pragma alloc_text(PAGE, FatPurgeReferencedFileObjects)
// #pragma alloc_text(PAGE, FatSetFileObject)
// #pragma alloc_text(PAGE, FatDecodeFileObject)
// #endif
//
//
// VOID
// FatSetFileObject (
//     IN PFILE_OBJECT FileObject OPTIONAL,
//     IN TYPE_OF_OPEN TypeOfOpen,
//     IN PVOID VcbOrFcbOrDcb,
//     IN PCCB Ccb OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine sets the file system pointers within the file object
//
// Arguments:
//
//     FileObject - Supplies a pointer to the file object being modified, and
//         can optionally be null.
//
//     TypeOfOpen - Supplies the type of open denoted by the file object.
//         This is only used by this procedure for sanity checking.
//
//     VcbOrFcbOrDcb - Supplies a pointer to either a vcb, fcb, or dcb
//
//     Ccb - Optionally supplies a pointer to a ccb
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSetFileObject, FileObject = %p\n", FileObject );
//
//     NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB));
//
//
//     NT_ASSERT(((TypeOfOpen == UnopenedFileObject))
//
//                 ||
//
//            ((TypeOfOpen == UserFileOpen) &&
//             (NodeType(VcbOrFcbOrDcb) == FAT_NTC_FCB) &&
//             (Ccb != NULL))
//
//                 ||
//
//            ((TypeOfOpen == EaFile) &&
//             (NodeType(VcbOrFcbOrDcb) == FAT_NTC_FCB) &&
//             (Ccb == NULL))
//
//                 ||
//
//            ((TypeOfOpen == UserDirectoryOpen) &&
//             ((NodeType(VcbOrFcbOrDcb) == FAT_NTC_DCB) || (NodeType(VcbOrFcbOrDcb) == FAT_NTC_ROOT_DCB)) &&
//             (Ccb != NULL))
//
//                 ||
//
//            ((TypeOfOpen == UserVolumeOpen) &&
//             (NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB) &&
//             (Ccb != NULL))
//
//                 ||
//
//            ((TypeOfOpen == VirtualVolumeFile) &&
//             (NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB) &&
//             (Ccb == NULL))
//
//                 ||
//
//            ((TypeOfOpen == DirectoryFile) &&
//             ((NodeType(VcbOrFcbOrDcb) == FAT_NTC_DCB) || (NodeType(VcbOrFcbOrDcb) == FAT_NTC_ROOT_DCB)) &&
//             (Ccb == NULL))
//             );
//
//
//     UNREFERENCED_PARAMETER( TypeOfOpen );
//
//     //
//     //  If we were given an Fcb, Dcb, or Vcb, we have some processing to do.
//     //
//
//     NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB));
//
//     if ( VcbOrFcbOrDcb != NULL ) {
//
//         //
//         //  Set the Vpb field in the file object, and if we were given an
//         //  Fcb or Dcb move the field over to point to the nonpaged Fcb/Dcb
//         //
//
//         if (NodeType(VcbOrFcbOrDcb) == FAT_NTC_VCB) {
//
//             FileObject->Vpb = ((PVCB)VcbOrFcbOrDcb)->Vpb;
//
//         } else {
//
//             FileObject->Vpb = ((PFCB)VcbOrFcbOrDcb)->Vcb->Vpb;
//
//             //
//             //  If this is a temporary file, note it in the FcbState
//             //
//
//             if (FlagOn(((PFCB)VcbOrFcbOrDcb)->FcbState, FCB_STATE_TEMPORARY)) {
//
//                 SetFlag(FileObject->Flags, FO_TEMPORARY_FILE);
//             }
//         }
//     }
//
//     NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB));
//
//     //
//     //  Now set the fscontext fields of the file object
//     //
//
//     if (ARGUMENT_PRESENT( FileObject )) {
//
//         FileObject->FsContext  = VcbOrFcbOrDcb;
//         FileObject->FsContext2 = Ccb;
//     }
//
//     NT_ASSERT((Ccb == NULL) || (NodeType(Ccb) == FAT_NTC_CCB));
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatSetFileObject -> VOID\n", 0);
//
//     return;
// }
//
// TYPE_OF_OPEN
// FatDecodeFileObject (
//      PFILE_OBJECT FileObject,
//      PVCB *Vcb,
//      PFCB *FcbOrDcb,
//      PCCB *Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This procedure takes a pointer to a file object, that has already been
//     opened by the Fat file system and figures out what really is opened.
//
// Arguments:
//
//     FileObject - Supplies the file object pointer being interrogated
//
//     Vcb - Receives a pointer to the Vcb for the file object.
//
//     FcbOrDcb - Receives a pointer to the Fcb/Dcb for the file object, if
//         one exists.
//
//     Ccb - Receives a pointer to the Ccb for the file object, if one exists.
//
// Return Value:
//
//     TYPE_OF_OPEN - returns the type of file denoted by the input file object.
//
//         UserFileOpen - The FO represents a user's opened data file.
//             Ccb, FcbOrDcb, and Vcb are set.  FcbOrDcb points to an Fcb.
//
//         UserDirectoryOpen - The FO represents a user's opened directory.
//             Ccb, FcbOrDcb, and Vcb are set.  FcbOrDcb points to a Dcb/RootDcb
//
//         UserVolumeOpen - The FO represents a user's opened volume.
//             Ccb and Vcb are set. FcbOrDcb is null.
//
//         VirtualVolumeFile - The FO represents the special virtual volume file.
//             Vcb is set, and Ccb and FcbOrDcb are null.
//
//         DirectoryFile - The FO represents a special directory file.
//             Vcb and FcbOrDcb are set. Ccb is null.  FcbOrDcb points to a
//             Dcb/RootDcb.
//
//         EaFile - The FO represents an Ea Io stream file.
//             FcbOrDcb, and Vcb are set.  FcbOrDcb points to an Fcb, and Ccb is
//             null.
//
// --*/
//
// {
//     TYPE_OF_OPEN TypeOfOpen;
//     PVOID FsContext;
//     PVOID FsContext2;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDecodeFileObject, FileObject = %p\n", FileObject);
//
//     //
//     //  Reference the fs context fields of the file object, and zero out
//     //  the out pointer parameters.
//     //
//
//     FsContext = FileObject->FsContext;
//     FsContext2 = FileObject->FsContext2;
//
//     //
//     //  Special case the situation where FsContext is null
//     //
//
//     if (FsContext == NULL) {
//
//         *Ccb = NULL;
//         *FcbOrDcb = NULL;
//         *Vcb = NULL;
//
//         TypeOfOpen = UnopenedFileObject;
//
//     } else {
//
//         //
//         //  Now we can case on the node type code of the fscontext pointer
//         //  and set the appropriate out pointers
//         //
//
//         switch (NodeType(FsContext)) {
//
//         case FAT_NTC_VCB:
//
//             *Ccb = FsContext2;
//             *FcbOrDcb = NULL;
//             *Vcb = FsContext;
//
//             TypeOfOpen = ( *Ccb == NULL ? VirtualVolumeFile : UserVolumeOpen );
//
//             break;
//
//         case FAT_NTC_ROOT_DCB:
//         case FAT_NTC_DCB:
//
//             *Ccb = FsContext2;
//             *FcbOrDcb = FsContext;
//             *Vcb = (*FcbOrDcb)->Vcb;
//
//             TypeOfOpen = ( *Ccb == NULL ? DirectoryFile : UserDirectoryOpen );
//
//             DebugTrace(0, Dbg, "Referencing directory: %wZ\n", &(*FcbOrDcb)->FullFileName);
//
//             break;
//
//         case FAT_NTC_FCB:
//
//             *Ccb = FsContext2;
//             *FcbOrDcb = FsContext;
//             *Vcb = (*FcbOrDcb)->Vcb;
//
//             if (*Ccb != NULL ) {
//
//                 TypeOfOpen = UserFileOpen;
//                 DebugTrace(0, Dbg, "Referencing file: %wZ\n", &(*FcbOrDcb)->FullFileName);
//
//             } else {
//
//                 //
//                 // No Ccb means this is a special open.
//                 //
//
//
//                 if ( *FcbOrDcb == (*Vcb)->EaFcb ) {
//
//                     TypeOfOpen = EaFile;
//                     DebugTrace(0, Dbg, "Referencing EA file: %wZ\n", &(*FcbOrDcb)->FullFileName);
//
//                 } else {
//
// #ifdef _MSC_VER
// #pragma prefast(suppress:28159, "things are seriously wrong if we get here")
// #endif
//                     FatBugCheck( NodeType(FsContext), 0, 0 );
//
//                 }
//
//             }
//
//             break;
//
//         default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( NodeType(FsContext), 0, 0 );
//         }
//     }
//
//     //
//     //  and return to our caller
//     //
//
//     DebugTrace(0, Dbg, "FatDecodeFileObject -> VCB(%p)\n", *Vcb);
//     DebugTrace(0, Dbg, "FatDecodeFileObject -> FCB(%p)\n", *FcbOrDcb);
//     DebugTrace(0, Dbg, "FatDecodeFileObject -> CCB(%p)\n", *Ccb);
//     DebugTrace(-1, Dbg, "FatDecodeFileObject -> TypeOfOpen = %08lx\n", TypeOfOpen);
//
//     return TypeOfOpen;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatPurgeReferencedFileObjects (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
//
// /*++
//
// Routine Description:
//
//     This routine non-recursively walks from the given FcbOrDcb and trys
//     to force Cc or Mm to close any sections it may be holding on to.
//
// Arguments:
//
//     Fcb - Supplies a pointer to either an fcb or a dcb
//
//     FlushType - Specifies the kind of flushing to perform
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFCB OriginalFcb = Fcb;
//     PFCB NextFcb;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatPurgeReferencedFileObjects, Fcb = %p\n", Fcb );
//
//     NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//     //
//     //  First, if we have a delayed close, force it closed.
//     //
//
//     FatFspClose(Fcb->Vcb);
//
//     //
//     //  Walk the directory tree forcing sections closed.
//     //
//     //  Note that it very important to get the next node to visit before
//     //  acting on the current node.  This is because acting on a node may
//     //  make it, and an arbitrary number of direct ancestors, vanish.
//     //  Since we never visit ancestors in our top-down enumeration scheme, we
//     //  can safely continue the enumeration even when the tree is vanishing
//     //  beneath us.  This is way cool.
//     //
//
//     while ( Fcb != NULL ) {
//
//         NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, OriginalFcb);
//
//         //
//         //  Check for the EA file fcb
//         //
//
//         if ( !FlagOn(Fcb->DirentFatFlags, FAT_DIRENT_ATTR_VOLUME_ID) ) {
//
//             FatForceCacheMiss( IrpContext, Fcb, FlushType );
//         }
//
//         Fcb = NextFcb;
//     }
//
//     DebugTrace(-1, Dbg, "FatPurgeReferencedFileObjects (VOID)\n", 0 );
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatForceCacheMiss (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
//
// /*++
//
// Routine Description:
//
//     The following routine asks either Cc or Mm to get rid of any cached
//     pages on a file.  Note that this will fail if a user has mapped a file.
//
//     If there is a shared cache map, purge the cache section.  Otherwise
//     we have to go and ask Mm to blow away the section.
//
//     NOTE: This caller MUST own the Vcb exclusive.
//
// Arguments:
//
//     Fcb - Supplies a pointer to an fcb
//
//     FlushType - Specifies the kind of flushing to perform
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PVCB Vcb;
//     BOOLEAN ChildrenAcquired = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  If we can't wait, bail.
//     //
//
//     NT_ASSERT( FatVcbAcquiredExclusive( IrpContext, Fcb->Vcb ) ||
//             FlagOn( Fcb->Vcb->VcbState, VCB_STATE_FLAG_LOCKED ) );
//
//     if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//         FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//     //
//     //  If we are purging a directory file object, we must acquire all the
//     //  FCBs exclusive so that the parent directory is not being pinned.
//     //  Careful, we can collide with something acquiring up the tree like
//     //  an unpin repinned flush (FsRtlAcquireFileForCcFlush ...) of a parent
//     //  dir on extending writethrough of a child file (oops).  So get things
//     //  going up the tree, not down.
//     //
//
//     if ((NodeType(Fcb) != FAT_NTC_FCB) &&
//         !IsListEmpty(&Fcb->Specific.Dcb.ParentDcbQueue)) {
//
//         PLIST_ENTRY Links;
//         PFCB TempFcb;
//
//         ChildrenAcquired = TRUE;
//
//         for (Links = Fcb->Specific.Dcb.ParentDcbQueue.Flink;
//              Links != &Fcb->Specific.Dcb.ParentDcbQueue;
//              Links = Links->Flink) {
//
//             TempFcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, TempFcb );
//         }
//     }
//
//     (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//
//     //
//     //  We use this flag to indicate to a close beneath us that
//     //  the Fcb resource should be freed before deleting the Fcb.
//     //
//
//     Vcb = Fcb->Vcb;
//
//     SetFlag( Fcb->FcbState, FCB_STATE_FORCE_MISS_IN_PROGRESS );
//
//     ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB );
//
//     _SEH2_TRY {
//
//         BOOLEAN DataSectionExists;
//         BOOLEAN ImageSectionExists;
//
//         PSECTION_OBJECT_POINTERS Section;
//
//         if ( FlushType ) {
//
//             (VOID)FatFlushFile( IrpContext, Fcb, FlushType );
//         }
//
//         //
//         //  The Flush may have made the Fcb go away
//         //
//
//         if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB)) {
//
//             Section = &Fcb->NonPaged->SectionObjectPointers;
//
//             DataSectionExists = (BOOLEAN)(Section->DataSectionObject != NULL);
//             ImageSectionExists = (BOOLEAN)(Section->ImageSectionObject != NULL);
//
//             //
//             //  Note, it is critical to do the Image section first as the
//             //  purge of the data section may cause the image section to go
//             //  away, but the opposite is not true.
//             //
//
//             if (ImageSectionExists) {
//
//                 (VOID)MmFlushImageSection( Section, MmFlushForWrite );
//             }
//
//             if (DataSectionExists) {
//
//                 CcPurgeCacheSection( Section, NULL, 0, FALSE );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         //
//         //  If we purging a directory file object, release all the Fcb
//         //  resources that we acquired above.  The Dcb cannot have vanished
//         //  if there were Fcbs underneath it, and the Fcbs couldn't have gone
//         //  away since I own the Vcb.
//         //
//
//         if (ChildrenAcquired) {
//
//             PLIST_ENTRY Links;
//             PFCB TempFcb;
//
//             for (Links = Fcb->Specific.Dcb.ParentDcbQueue.Flink;
//                  Links != &Fcb->Specific.Dcb.ParentDcbQueue;
//                  Links = Links->Flink) {
//
//                 TempFcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//                 FatReleaseFcb( IrpContext, TempFcb );
//             }
//         }
//
//         //
//         //  Since we have the Vcb exclusive we know that if any closes
//         //  come in it is because the CcPurgeCacheSection caused the
//         //  Fcb to go away.  Also in close, the Fcb was released
//         //  before being freed.
//         //
//
//         if ( !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB) ) {
//
//             ClearFlag( Fcb->FcbState, FCB_STATE_FORCE_MISS_IN_PROGRESS );
//
//             FatReleaseFcb( (IRPCONTEXT), Fcb );
//         }
//     } _SEH2_END;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
