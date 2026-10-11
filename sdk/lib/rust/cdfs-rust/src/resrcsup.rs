#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExAcquireSharedStarveExclusive, ExReleaseResourceLite, FS_FILTER_CALLBACK_DATA, PFILE_OBJECT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FCB, IRP_CONTEXT};
use ntkrnl_sys::ddk::{FILE_OBJECT, IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_RESRCSUP};

extern "system" {
    fn IoGetTopLevelIrp() -> *mut IRP;
}
extern "system" {
    fn IoSetTopLevelIrp(Irp: *mut IRP);
}
extern "C" {
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_RESRCSUP;

// Verified vs headers:
//   FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION = (UCHAR)-1 = 0xFF (iotypes.h)
//   _FS_FILTER_SECTION_SYNC_TYPE_SyncTypeCreateSection = 1 (bindgen)
//   STATUS_FILE_LOCKED_WITH_ONLY_READERS = 0x0000012A (ntstatus.h)
//   STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY = 0x00000126 (ntstatus.h)
const FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION: u8 = 0xFF;
const SyncTypeCreateSection: u32 = 1;
const STATUS_FILE_LOCKED_WITH_ONLY_READERS: i32 = 0x0000_012A;
const STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY: i32 = 0x0000_0126;
// CdAcquireResource — translated from drivers/filesystems/cdfs/resrcsup.c (grind
// dropped it). The single routine used to acquire file-system resources.
//   TYPE_OF_ACQUIRE: AcquireExclusive=0, AcquireShared=1, AcquireSharedStarveExclusive=2.
//   IRP_CONTEXT_FLAG_WAIT = 0x00000004 (cdstruc.h).  STATUS_CANT_WAIT = 0xC00000D8.
// Defined extern "system" to match the (@16) call-site decls on i386.
const ACQUIRE_EXCLUSIVE: u32 = 0;
const ACQUIRE_SHARED: u32 = 1;
const ACQUIRE_SHARED_STARVE_EXCLUSIVE: u32 = 2;
const IRP_CONTEXT_FLAG_WAIT_RS: u32 = 0x00000004;
#[no_mangle]
pub unsafe extern "C" fn CdAcquireResource(
    IrpContext: *mut IRP_CONTEXT,
    Resource: ntkrnl_sys::ddk::PERESOURCE,
    IgnoreWait: u8,
    Type: u32,
) -> u8 {
    let mut Wait: u8 = 0; // FALSE
    PAGED_CODE!();

    if IgnoreWait == 0 && ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT_RS) != 0 {
        Wait = 1; // TRUE
    }

    let Acquired: u8 = match Type {
        ACQUIRE_EXCLUSIVE => ExAcquireResourceExclusiveLite(Resource, Wait),
        ACQUIRE_SHARED => ExAcquireResourceSharedLite(Resource, Wait),
        ACQUIRE_SHARED_STARVE_EXCLUSIVE => ExAcquireSharedStarveExclusive(Resource, Wait),
        _ => {
            NT_ASSERT!(0); // NT_ASSERT(FALSE)
            0
        }
    };

    if Acquired == 0 && IgnoreWait == 0 {
        crate::ddk_shims::CdRaiseStatus(IrpContext, crate::glue::STATUS_CANT_WAIT);
    }

    Acquired
}

#[no_mangle]
pub unsafe extern "system" fn CdAcquireForCache(Fcb: *mut FCB, Wait: i32) -> i32 {
    // PAGED_CODE();
    if ExAcquireResourceSharedLite((*Fcb).__bindgen_anon_1.Header.as_ref().Resource, Wait as u8) == 0 {
        return 0;
    }
    // NT_ASSERT(IoGetTopLevelIrp() == NULL);
    ntkrnl_sys::ASSERT!(IoGetTopLevelIrp().is_null());
    let fsrtl_cache_top_level_irp: *mut IRP = 1 as *mut IRP;
    IoSetTopLevelIrp(fsrtl_cache_top_level_irp);
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn CdNoopAcquire(
    Fcb: *mut core::ffi::c_void,
    Wait: u8,
) -> u8 {
    // PAGED_CODE();
    // UNREFERENCED_PARAMETER( Fcb );
    let _ = Fcb;
    // UNREFERENCED_PARAMETER( Wait );
    let _ = Wait;
    // return TRUE;
    1
}
#[no_mangle]
pub unsafe extern "system" fn CdNoopRelease(Fcb: *mut core::ffi::c_void) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Fcb);
}

// CdReleaseFromCache: NTAPI cache-manager release callback. The C body lived in
// resrcsup.c (replaced by Rust); the grind dropped it. Faithful translation of
// drivers/filesystems/cdfs/resrcsup.c CdReleaseFromCache.
//   FSRTL_CACHE_TOP_LEVEL_IRP = (PIRP)1  (fsrtltypes.h)
#[no_mangle]
pub unsafe extern "system" fn CdReleaseFromCache(Fcb: *mut FCB) {
    PAGED_CODE!();
    let fsrtl_cache_top_level_irp: *mut IRP = 1 as *mut IRP;
    NT_ASSERT!((IoGetTopLevelIrp() == fsrtl_cache_top_level_irp) as i32);
    IoSetTopLevelIrp(core::ptr::null_mut());
    ExReleaseResourceLite((*Fcb).__bindgen_anon_1.Header.as_ref().Resource);
}

#[no_mangle]
pub unsafe extern "system" fn CdFilterCallbackAcquireForCreateSection(
    CallbackData: *mut FS_FILTER_CALLBACK_DATA,
    CompletionContext: *mut *mut core::ffi::c_void,
) -> i32 {
    let FileObject: *mut FILE_OBJECT;
    PAGED_CODE!();
    NT_ASSERT!(((*CallbackData).Operation == FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION) as i32);
    NT_ASSERT!(((*CallbackData).SizeOfFsFilterCallbackData
            == core::mem::size_of::<FS_FILTER_CALLBACK_DATA>() as u32) as i32);
    // UNREFERENCED_PARAMETER
    let _ = CompletionContext;
    FileObject = (*CallbackData).FileObject;
    if FileObject.is_null()
        || (*FileObject).FsContext.is_null()
        || (*((*FileObject).FsContext as *mut FCB))
            .FcbNonpaged
            .is_null()
    {
        let fs_context = if !FileObject.is_null() {
            (*FileObject).FsContext
        } else {
            core::ptr::null_mut()
        };
        let fcb_nonpaged = if !FileObject.is_null()
            && !(*FileObject).FsContext.is_null()
        {
            (*((*FileObject).FsContext as *mut FCB)).FcbNonpaged
        } else {
            core::ptr::null_mut()
        };
        DbgPrint(
            b"CdFilterCallbackAcquireForCreateSection: NULL guard hit! FileObject=%p FsContext=%p FcbNonpaged=%p\n\0".as_ptr() as *const i8,
            FileObject,
            fs_context,
            fcb_nonpaged,
        );
        return STATUS_INVALID_PARAMETER;
    }
    let fcb = (*FileObject).FsContext as *mut FCB;
    ExAcquireResourceExclusiveLite(
        core::ptr::addr_of_mut!((*(*fcb).FcbNonpaged).FcbResource),
        1,
    );
    ExAcquireSharedStarveExclusive((*fcb).__bindgen_anon_1.Header.as_ref().Resource, 1);
    if (*CallbackData).Parameters.AcquireForSectionSynchronization.SyncType
        == SyncTypeCreateSection
    {
        return STATUS_FILE_LOCKED_WITH_ONLY_READERS;
    } else {
        return STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CdReleaseForCreateSection(FileObject: PFILE_OBJECT) {
    PAGED_CODE!();

    let fcb: *mut FCB = (*FileObject).FsContext as *mut FCB;

    ExReleaseResourceLite(core::ptr::addr_of_mut!((*(*fcb).FcbNonpaged).FcbResource));
    ExReleaseResourceLite((*fcb).__bindgen_anon_1.Header.as_ref().Resource);
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
//     ResrcSup.c
//
// Abstract:
//
//     This module implements the Cdfs Resource acquisition routines
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_RESRCSUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdAcquireForCache)
// #pragma alloc_text(PAGE, CdFilterCallbackAcquireForCreateSection)
// #pragma alloc_text(PAGE, CdAcquireResource)
// #pragma alloc_text(PAGE, CdNoopAcquire)
// #pragma alloc_text(PAGE, CdNoopRelease)
// #pragma alloc_text(PAGE, CdReleaseForCreateSection)
// #pragma alloc_text(PAGE, CdReleaseFromCache)
// #endif
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// (Type == AcquireExclusive && return != FALSE, _Acquires_exclusive_lock_(*Resource))
// (Type == AcquireShared && return != FALSE, _Acquires_shared_lock_(*Resource))
// (Type == AcquireSharedStarveExclusive && return != FALSE, _Acquires_shared_lock_(*Resource))
// (IgnoreWait == FALSE, )
// BOOLEAN
// CdAcquireResource (
//      PIRP_CONTEXT IrpContext,
//      PERESOURCE Resource,
//      BOOLEAN IgnoreWait,
//      TYPE_OF_ACQUIRE Type
//     )
//
// /*++
//
// Routine Description:
//
//     This is the single routine used to acquire file system resources.  It
//     looks at the IgnoreWait flag to determine whether to try to acquire the
//     resource without waiting.  Returning TRUE/FALSE to indicate success or
//     failure.  Otherwise it is driven by the WAIT flag in the IrpContext and
//     will raise CANT_WAIT on a failure.
//
// Arguments:
//
//     Resource - This is the resource to try and acquire.
//
//     IgnoreWait - If TRUE then this routine will not wait to acquire the
//         resource and will return a boolean indicating whether the resource was
//         acquired.  Otherwise we use the flag in the IrpContext and raise
//         if the resource is not acquired.
//
//     Type - Indicates how we should try to get the resource.
//
// Return Value:
//
//     BOOLEAN - TRUE if the resource is acquired.  FALSE if not acquired and
//         IgnoreWait is specified.  Otherwise we raise CANT_WAIT.
//
// --*/
//
// {
//     BOOLEAN Wait = FALSE;
//     BOOLEAN Acquired;
//     PAGED_CODE();
//
//     //
//     //  We look first at the IgnoreWait flag, next at the flag in the Irp
//     //  Context to decide how to acquire this resource.
//     //
//
//     if (!IgnoreWait && FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//         Wait = TRUE;
//     }
//
//     //
//     //  Attempt to acquire the resource either shared or exclusively.
//     //
//
//     switch (Type) {
//         case AcquireExclusive:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28137, "prefast believes Wait should be a constant, but this is ok for CDFS" )
// #endif
//             Acquired = ExAcquireResourceExclusiveLite( Resource, Wait );
//             break;
//
//         case AcquireShared:
//
//             Acquired = ExAcquireResourceSharedLite( Resource, Wait );
//             break;
//
//         case AcquireSharedStarveExclusive:
//
//             Acquired = ExAcquireSharedStarveExclusive( Resource, Wait );
//             break;
//
//         default:
//             Acquired = FALSE;
//             NT_ASSERT( FALSE );
//     }
//
//     //
//     //  If not acquired and the user didn't specifiy IgnoreWait then
//     //  raise CANT_WAIT.
//     //
//
//     if (!Acquired && !IgnoreWait) {
//
//         CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//     return Acquired;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// (return!=0, _Acquires_shared_lock_(*Fcb->Resource))
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdAcquireForCache (
//      PFCB Fcb,
//      BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a file.  It is subsequently called by the Lazy Writer for synchronization.
//
// Arguments:
//
//     Fcb -  The pointer supplied as context to the cache initialization
//            routine.
//
//     Wait - TRUE if the caller is willing to block.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     if (!ExAcquireResourceSharedLite( Fcb->Resource, Wait )) {
//
//         return FALSE;
//     }
//
//     NT_ASSERT(IoGetTopLevelIrp() == NULL);
//     IoSetTopLevelIrp((PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     return TRUE;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_lock_(*Fcb->Resource)
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdReleaseFromCache (
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a virtual file.  It is subsequently called by the Lazy Writer to release
//     a resource acquired above.
//
// Arguments:
//
//     Fcb -  The pointer supplied as context to the cache initialization
//            routine.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     NT_ASSERT(IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//     IoSetTopLevelIrp( NULL );
//
//     ExReleaseResourceLite( Fcb->Resource );
// }
//
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdNoopAcquire (
//      PVOID Fcb,
//      BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     This routine does nothing.
//
// Arguments:
//
//     Fcb - The Fcb/Vcb which was specified as a context parameter for this
//           routine.
//
//     Wait - TRUE if the caller is willing to block.
//
// Return Value:
//
//     TRUE
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Fcb );
//     UNREFERENCED_PARAMETER( Wait );
//
//     return TRUE;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdNoopRelease (
//      PVOID Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine does nothing.
//
// Arguments:
//
//     Fcb - The Fcb/Vcb which was specified as a context parameter for this
//           routine.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Fcb );
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFilterCallbackAcquireForCreateSection (
//      PFS_FILTER_CALLBACK_DATA CallbackData,
//     _Unreferenced_parameter_ PVOID *CompletionContext
//     )
//
// /*++
//
// Routine Description:
//
//     This is the callback routine for MM to use to acquire the file exclusively.
//
// Arguments:
//
//     FS_FILTER_CALLBACK_DATA - Filter based callback data that provides the file object we
//                               want to acquire.
//
//     CompletionContext - Ignored.
//
// Return Value:
//
//     On success we return STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY.
//
//     If SyncType is SyncTypeCreateSection, we return a status that indicates there are no
//     writers to this file.
//
// --*/
//
// {
//     PFILE_OBJECT FileObject;
//
//
//     PAGED_CODE();
//
//     NT_ASSERT( CallbackData->Operation == FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION );
//     NT_ASSERT( CallbackData->SizeOfFsFilterCallbackData == sizeof(FS_FILTER_CALLBACK_DATA) );
//
//     //
//     //  Get the file object from the callback data.
//     //
//
//     FileObject = CallbackData->FileObject;
//
//     //
//     //  Guard against NULL FsContext — can happen when exception handling
//     //  in cdfs leaves FCB partially initialized (PSEH_CLANG catch path).
//     //
//
//     if (!FileObject || !FileObject->FsContext ||
//         !((PFCB) FileObject->FsContext)->FcbNonpaged) {
//
//         DbgPrint("CdFilterCallbackAcquireForCreateSection: NULL guard hit! FileObject=%p FsContext=%p FcbNonpaged=%p\n",
//                  FileObject,
//                  FileObject ? FileObject->FsContext : NULL,
//                  (FileObject && FileObject->FsContext) ? (PVOID)((PFCB) FileObject->FsContext)->FcbNonpaged : NULL);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Get the Fcb resource exclusively.
//     //
//
//     ExAcquireResourceExclusiveLite( &((PFCB) FileObject->FsContext)->FcbNonpaged->FcbResource,
//                                     TRUE );
//
//     //
//     //  Take the File resource shared.  We need this later on when MM calls
//     //  QueryStandardInfo to get the file size.
//     //
//     //  If we don't use StarveExclusive,  then we can get wedged behind an
//     //  exclusive waiter who is waiting on someone else holding it shared in the
//     //  read->initializecachemap path (which calls createsection) who is in turn
//     //  waiting on us to finish the create section.
//     //
//
//     ExAcquireSharedStarveExclusive( ((PFCB) FileObject->FsContext)->Resource,
//                                     TRUE );
//
//     //
//     //  CDFS is a read-only file system, so we can always indicate no writers.
//     //  We only do this for create section synchronization.  For others we
//     //  return the generic success STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY.
//     //
//
//     if (CallbackData->Parameters.AcquireForSectionSynchronization.SyncType == SyncTypeCreateSection) {
//
//         return STATUS_FILE_LOCKED_WITH_ONLY_READERS;
//
//     } else {
//
//         return STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY;
//     }
//
//     UNREFERENCED_PARAMETER( CompletionContext );
// }
//
//
// _Function_class_(FAST_IO_RELEASE_FILE)
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdReleaseForCreateSection (
//      PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is the callback routine for MM to use to release a file acquired with
//     the AcquireForCreateSection call above.
//
// Arguments:
//
//     FileObject - File object for a Cdfs stream.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Release the resources.
//     //
//
//     ExReleaseResourceLite( &((PFCB) FileObject->FsContext)->FcbNonpaged->FcbResource );
//     ExReleaseResourceLite( ((PFCB) FileObject->FsContext)->Resource);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
