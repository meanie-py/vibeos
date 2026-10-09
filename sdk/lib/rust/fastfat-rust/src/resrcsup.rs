use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExAcquireSharedWaitForExclusive, ExGetExclusiveWaiterCount, ExGetSharedWaiterCount, ExIsResourceAcquiredSharedLite, ExReleaseResourceLite, _KWAIT_REASON_Executive, FS_FILTER_CALLBACK_DATA, IoGetTopLevelIrp, IoSetTopLevelIrp, KeWaitForSingleObject, PDEVICE_OBJECT, PFILE_OBJECT, PFSRTL_COMMON_FCB_HEADER, PFS_FILTER_CALLBACK_DATA, PIRP, PLARGE_INTEGER, PsGetCurrentThread};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FCB, FatData, IRP_CONTEXT, _TYPE_OF_OPEN_DirectoryFile as DirectoryFile};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_OBJECT, FSRTL_COMMON_FCB_HEADER, IRP};

extern "system" {
    fn BooleanFlagOn(Flags: u32, Flag: u32) -> u8;
    fn FatVerifyOperationIsLegal(IrpContext: *mut IRP_CONTEXT);
    fn FatReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FlagOn(Flags: u32, Flag: u32) -> u8;
    fn ExAcquireResourceSharedLite(Resource: *mut ERESOURCE, Wait: u8) -> u8;
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn NT_ASSERT(condition: i32);
    fn NodeType(fcb: *mut FCB) -> u32;
    fn FatDecodeFileObject( FileObject: PFILE_OBJECT, Vcb: *mut PVCB, Fcb: *mut PFCB, Ccb: *mut PCCB, ) -> TYPE_OF_OPEN;
    fn KeGetCurrentIrql() -> u8;
    fn DbgBreakPoint();
    fn UNREFERENCED_PARAMETER(p: *mut core::ffi::c_void);
}

// Forward declarations for symbols not imported

// Placeholder constant (value not provided by imports)
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)



type ULONG = u32;
type PIRP_CONTEXT = *mut core::ffi::c_void; // placeholder
type PVCB = *mut VCB;
// FIX (arm64 boot hang): use the REAL VCB + ERESOURCE from the shared bindgen
// types, NOT a bogus local mirror. The old local `struct VCB { Resource:
// ERESOURCE }` placed Resource at offset 0, so FatAcquireExclusiveVcb_Real's /
// FatAcquireSharedVcb's `&(*Vcb).Resource` resolved to the VCB base (the FSRTL
// VolumeFileHeader) instead of the real ERESOURCE at VCB+440. The create path
// then acquired a garbage "resource" (the header) and waited on it forever —
// the arm64 smss-load boot hang. (Other fns here use the real VCB via
// FCB.Vcb's declared type, which is why only these two were affected.)
use crate::types::VCB;
use ntkrnl_sys::ddk::ERESOURCE;

const IRP_NOCACHE: u32 = 0x01;

const FAT_NTC_FCB: u32 = 0x0502; // TODO: correct value
// fsrtltypes.h: #define FSRTL_CACHE_TOP_LEVEL_IRP (0x02), used as (PIRP). ONE definition for
// the file: it used to carry four (-2, 2, -1, 1), so each Acquire/Release pair set one value
// and tested for another. FatReleaseForCcFlush never recognised FatAcquireForCcFlush's -2,
// left it as the thread's top-level IRP, and the next FatCommonWrite on that thread took
// -2 for a real IRP (it is > FSRTL_MAX_TOP_LEVEL_IRP_FLAG unsigned) and faulted reading its
// NodeType: BUGCHECK 0x23 in kmtest ZwCreateSection on arm64.
const FSRTL_CACHE_TOP_LEVEL_IRP: *mut IRP = 0x02 as *mut IRP;



type PFCB = *mut core::ffi::c_void;
type PCCB = *mut core::ffi::c_void;
type TYPE_OF_OPEN = u32;


const FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION: u8 = 0xFF; // (UCHAR)-1
const SyncTypeCreateSection: u32 = 1; // known value
const STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY: i32 = 0x0126; // FIXME: verify value
const STATUS_FILE_LOCKED_WITH_ONLY_READERS: i32 = 0x012A; // FIXME: set actual value
const STATUS_FILE_LOCKED_WITH_WRITERS: i32 = 0x012B; // FIXME: set actual value
#[no_mangle]
pub unsafe extern "system" fn FatAcquireExclusiveVcb_Real(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    NoOpCheck: u8,
) -> u8 {
    // PAGED_CODE() – ignored in mechanical transliteration
    // #pragma prefast( suppress: 28137, ... ) – ignored

    if ExAcquireResourceExclusiveLite(
        &raw mut (*Vcb).Resource as *mut _,
        BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT),
    ) != 0
    {
        if NoOpCheck == 0 {
            // _SEH2_TRY / _SEH2_FINALLY
            crate::seh::try_finally(&mut (), |_| {
                FatVerifyOperationIsLegal(IrpContext);
            }, |_, abnormal| {
                if abnormal {
                    FatReleaseVcb(IrpContext, Vcb);
                }
            });
        }
        return 1; // TRUE
    }
    0 // FALSE
}
#[no_mangle]
pub unsafe extern "system" fn FatAcquireSharedVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> i32 {
    PAGED_CODE!();
    if ExAcquireResourceSharedLite(
        core::ptr::addr_of_mut!((*Vcb).Resource) as *mut _,
        BooleanFlagOn((*( IrpContext as *mut IRP_CONTEXT )).Flags, IRP_CONTEXT_FLAG_WAIT),
    ) != 0
    {
        // _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            FatVerifyOperationIsLegal(IrpContext as *mut IRP_CONTEXT);
        }, |_, abnormal| {
            if abnormal {
                FatReleaseVcb(IrpContext as *mut IRP_CONTEXT, Vcb);
            }
        });
        return 1;
    }
    0
}

/// resrcsup.c `FatAcquireExclusiveFcb`. This one was dropped in translation and a C
/// shim in rust_cohab_stub.c stood in for it: a bare ExAcquireResourceExclusiveLite,
/// with no OutstandingAsyncWrites drain and no FatVerifyOperationIsLegal. Without the
/// drain an exclusive acquirer (truncate, set-EOF, cleanup, flush) can run while a
/// non-cached async write it cannot see is still landing in the same file.
#[no_mangle]
pub unsafe extern "system" fn FatAcquireExclusiveFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) -> u8 {
    loop {
        if ExAcquireResourceExclusiveLite(
            ((*Fcb).Header.Resource).cast(),
            BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u8,
        ) != 0
        {
            // Anything other than a non-cached write must wait out the async writes
            // (and so must that write, if others are queued on the resource); then
            // drop the resource and take it again.
            if (*(*Fcb).NonPaged).OutstandingAsyncWrites != 0
                && ((*IrpContext).MajorFunction != IRP_MJ_WRITE
                    || FlagOn((*(*IrpContext).OriginatingIrp).Flags, IRP_NOCACHE) == 0
                    || ExGetSharedWaiterCount((*Fcb).Header.Resource) != 0
                    || ExGetExclusiveWaiterCount((*Fcb).Header.Resource) != 0)
            {
                KeWaitForSingleObject(
                    (*(*Fcb).NonPaged).OutstandingAsyncEvent as *mut c_void,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0,
                    core::ptr::null_mut(),
                );
                FatReleaseFcb(IrpContext, Fcb);
                continue;
            }
            // _SEH2_TRY / _SEH2_FINALLY
            crate::seh::try_finally(&mut (), |_| {
                FatVerifyOperationIsLegal(IrpContext);
            }, |_, abnormal| {
                if abnormal {
                    FatReleaseFcb(IrpContext, Fcb);
                }
            });
            return 1;
        } else {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatAcquireSharedFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) -> u8 {
    loop {
        if ExAcquireResourceSharedLite(
            ((*Fcb).Header.Resource).cast(),
            BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u8,
        ) != 0
        {
            if (*(*Fcb).NonPaged).OutstandingAsyncWrites != 0
                && ((*IrpContext).MajorFunction != IRP_MJ_WRITE
                    || FlagOn((*(*IrpContext).OriginatingIrp).Flags, IRP_NOCACHE) == 0
                    || ExGetSharedWaiterCount((*Fcb).Header.Resource) != 0
                    || ExGetExclusiveWaiterCount((*Fcb).Header.Resource) != 0)
            {
                KeWaitForSingleObject(
                    (*(*Fcb).NonPaged).OutstandingAsyncEvent as *mut c_void,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0,
                    core::ptr::null_mut(),
                );
                FatReleaseFcb(IrpContext, Fcb);
                continue;
            }
            // _SEH2_TRY / _SEH2_FINALLY
            crate::seh::try_finally(&mut (), |_| {
                FatVerifyOperationIsLegal(IrpContext);
            }, |_, abnormal| {
                if abnormal {
                    FatReleaseFcb(IrpContext, Fcb);
                }
            });
            return 1;
        } else {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatAcquireSharedFcbWaitForEx(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) -> i32 {
    PAGED_CODE!();

    NT_ASSERT((FlagOn(
        (*(*IrpContext).OriginatingIrp).Flags,
        IRP_NOCACHE,
    ) != 0) as i32);

    NT_ASSERT((FlagOn(
        (*IrpContext).Flags,
        IRP_CONTEXT_FLAG_WAIT,
    ) == 0) as i32);

    'RetryFcbSharedWaitEx: loop {
        if ExAcquireSharedWaitForExclusive(
            (*Fcb).Header.Resource,
            0,
        ) != 0
        {
            if ((*(*Fcb).NonPaged).OutstandingAsyncWrites != 0)
                && ((*IrpContext).MajorFunction != IRP_MJ_WRITE)
            {
                KeWaitForSingleObject(
                    (*(*Fcb).NonPaged).OutstandingAsyncEvent as *mut core::ffi::c_void,
                    _KWAIT_REASON_Executive,
                    (0i32) as i8,
                    0,
                    core::ptr::null_mut(),
                );
                FatReleaseFcb(IrpContext, Fcb);
                continue 'RetryFcbSharedWaitEx;
            }

            // _SEH2_TRY / _SEH2_FINALLY
            crate::seh::try_finally(&mut (), |_| {
                FatVerifyOperationIsLegal(IrpContext);
            }, |_, abnormal| {
                if abnormal {
                    FatReleaseFcb(IrpContext, Fcb);
                }
            });

            return 1;
        } else {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatAcquireFcbForLazyWrite(
    Fcb: *mut core::ffi::c_void,
    Wait: u8,
) -> u8 {
    // PAGED_CODE(); // no‑op in Rust

    // Local constants that would normally come from headers
    const FAT_NTC_FCB: u32 = 0x0502; // TODO: actual value unknown

    let fcb_ptr = Fcb as *mut FCB;

    // Condition: Fcb == Vcb->EaFcb
    let vcb = (*fcb_ptr).Vcb; // *mut VCB
    let eafcb = (*vcb).EaFcb; // *mut c_void
    let is_eafcb = Fcb == (eafcb).cast();

    // Choose resource based on condition
    let resource = if is_eafcb {
        (*fcb_ptr).Header.Resource
    } else {
        (*fcb_ptr).Header.PagingIoResource
    };

    // Try to acquire the resource shared
    if ExAcquireResourceSharedLite((resource).cast(), Wait) == 0 {
        return 0; // FALSE
    }

    // Assert node type
    let node_type = (*fcb_ptr).Header.NodeTypeCode;
    ntkrnl_sys::ASSERT!(node_type == (FAT_NTC_FCB) as i16);

    // Assert that LazyWriteThread is currently NULL
    ntkrnl_sys::ASSERT!((*fcb_ptr).Specific.Fcb.as_ref().LazyWriteThread.is_null());

    // Set this thread as the LazyWriteThread
    (*fcb_ptr).Specific.Fcb.as_mut().LazyWriteThread = (PsGetCurrentThread()).cast();

    // Assert that the current thread is not NULL
    ntkrnl_sys::ASSERT!(!PsGetCurrentThread().is_null());

    // Set global lazy write thread if not already set
    let fat_data = core::ptr::addr_of_mut!(FatData);
    if (*fat_data).LazyWriteThread.is_null() {
        (*fat_data).LazyWriteThread = (PsGetCurrentThread()).cast();
    }

    // Kludge: pretend this is the top‑level IRP
    ntkrnl_sys::ASSERT!(IoGetTopLevelIrp().is_null());
    IoSetTopLevelIrp((FSRTL_CACHE_TOP_LEVEL_IRP as *mut core::ffi::c_void).cast());

    1 // TRUE
}


#[no_mangle]
pub unsafe extern "system" fn FatReleaseFcbFromLazyWrite(Fcb: *mut core::ffi::c_void) {
    // PAGED_CODE();

    let fcb_ptr = Fcb as *mut FCB;

    // Assert that this really is an fcb and that this thread really owns
    // the lazy writer mark in the fcb.
    let node_type = NodeType(fcb_ptr);
    NT_ASSERT((node_type == FAT_NTC_FCB) as i32);

    let cur_thread = PsGetCurrentThread();
    NT_ASSERT((!cur_thread.is_null()) as i32);

    let lazy_write_thread = (*fcb_ptr).Specific.Fcb.as_ref().LazyWriteThread;
    NT_ASSERT((lazy_write_thread == (cur_thread).cast()) as i32);

    // Release the lazy writer mark.
    (*fcb_ptr).Specific.Fcb.as_mut().LazyWriteThread = core::ptr::null_mut();

    // Check here for the EA File.  It turns out we needed the normal
    // resource shared in this case.  Otherwise it was the PagingIoResource.
    let vcb = (*fcb_ptr).Vcb;
    let ea_fcb = (*vcb).EaFcb;
    let resource = if Fcb as *mut core::ffi::c_void == ea_fcb as *mut core::ffi::c_void {
        (*fcb_ptr).Header.Resource
    } else {
        (*fcb_ptr).Header.PagingIoResource
    };
    ExReleaseResourceLite(resource);

    // Clear the kludge at this point.
    let top_irp = IoGetTopLevelIrp();
    NT_ASSERT((top_irp == FSRTL_CACHE_TOP_LEVEL_IRP) as i32);

    IoSetTopLevelIrp(core::ptr::null_mut());
}


#[no_mangle]
pub unsafe extern "system" fn FatAcquireFcbForReadAhead(
    Fcb: *mut core::ffi::c_void,
    Wait: u8,
) -> u8 {
    PAGED_CODE!();

    //  We acquire the normal file resource shared here to synchronize
    //  correctly with purges.
    //
    //  Note that we do not need to disable APC delivery to guard
    //  against a rogue user issuing a suspend APC. That is because
    //  it is guaranteed that the caller is either in the system context,
    //  to which a user cannot deliver a suspend APC, or the caller has
    //  already disabled kernel APC delivery before calling. This is true
    //  for all the other pre-acquire routines as well.

    let fcb_ptr = Fcb as *mut FCB;
    if ExAcquireResourceSharedLite(
        (*fcb_ptr).Header.Resource as *mut _,
        Wait,
    ) == 0
    {
        return 0;
    }

    //  This is a kludge because Cc is really the top level.  We it
    //  enters the file system, we will think it is a resursive call
    //  and complete the request with hard errors or verify.  It will
    //  have to deal with them, somehow....

    ntkrnl_sys::ASSERT!(IoGetTopLevelIrp().is_null());

    IoSetTopLevelIrp(FSRTL_CACHE_TOP_LEVEL_IRP);

    1
}

#[no_mangle]
pub unsafe extern "system" fn FatReleaseFcbFromReadAhead(Fcb: *mut core::ffi::c_void) {

    // PAGED_CODE() – normally a no-op in non‑checked builds; left as-is.
    let top_irp = IoGetTopLevelIrp();
    NT_ASSERT((top_irp == FSRTL_CACHE_TOP_LEVEL_IRP) as i32);

    IoSetTopLevelIrp(core::ptr::null_mut());

    // Cast the context pointer to the Fcb (the caller provides the Fcb type)
    let fcb = Fcb as *mut FCB;
    // Header.Resource is a PERESOURCE, matching the ExReleaseResourceLite signature.
    ExReleaseResourceLite((*fcb).Header.Resource);
}


#[no_mangle]
pub unsafe extern "system" fn FatAcquireForCcFlush(
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> i32 {
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Header: PFSRTL_COMMON_FCB_HEADER;
    let Type: TYPE_OF_OPEN;

    // PAGED_CODE(); ignored
    // UNREFERENCED_PARAMETER( DeviceObject );
    let _ = DeviceObject;

    // NT_ASSERT( IoGetTopLevelIrp() != (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP );
    // Only that: a NULL top-level IRP is legal here -- the next line handles it -- and the
    // `!is_null() &&` this once carried fired BUGCHECK 0x23 on arm64 when kmtest's
    // ZwCreateSection flushed with no top-level IRP set.
    ntkrnl_sys::ASSERT!(IoGetTopLevelIrp() != FSRTL_CACHE_TOP_LEVEL_IRP);

    if IoGetTopLevelIrp().is_null() {
        IoSetTopLevelIrp(FSRTL_CACHE_TOP_LEVEL_IRP);
    }

    Type = FatDecodeFileObject(
        FileObject,
        core::ptr::addr_of_mut!(Vcb),
        core::ptr::addr_of_mut!(Fcb),
        core::ptr::addr_of_mut!(Ccb),
    );

    Header = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;

    if Type < DirectoryFile {
        let resource = (*Header).Resource;
        if !resource.is_null() {
            if ExIsResourceAcquiredSharedLite(resource) == 0 {
                ExAcquireResourceExclusiveLite(resource, 1);
            } else {
                ExAcquireResourceSharedLite(resource as *mut ERESOURCE, 1);
            }
        }
    }

    if !(*Header).PagingIoResource.is_null() {
        ExAcquireResourceSharedLite(((*Header).PagingIoResource).cast(), 1);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FatReleaseForCcFlush(
    file_object: *mut FILE_OBJECT,
    device_object: *mut DEVICE_OBJECT,
) -> i32 {

    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();
    let mut type_: i32;
    let header: *mut FSRTL_COMMON_FCB_HEADER;

    PAGED_CODE!();

    let _ = device_object;

    if (ntkrnl_sys::ddk::IoGetTopLevelIrp() as *mut core::ffi::c_void) == FSRTL_CACHE_TOP_LEVEL_IRP as *mut core::ffi::c_void {
        ntkrnl_sys::ddk::IoSetTopLevelIrp(core::ptr::null_mut());
    }

    type_ = FatDecodeFileObject(
        file_object,
        core::ptr::addr_of_mut!(vcb) as *mut PVCB,
        core::ptr::addr_of_mut!(fcb) as *mut PFCB,
        core::ptr::addr_of_mut!(ccb) as *mut PCCB,
    ) as i32;
    header = (*file_object).FsContext as *mut FSRTL_COMMON_FCB_HEADER;

    if type_ < (DirectoryFile) as i32 {
        if !(*header).Resource.is_null() {
            ntkrnl_sys::ddk::ExReleaseResourceLite((*header).Resource);
        }
    }

    if !(*header).PagingIoResource.is_null() {
        ntkrnl_sys::ddk::ExReleaseResourceLite((*header).PagingIoResource);
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn FatNoOpAcquire(
    Fcb: *mut core::ffi::c_void,
    Wait: u8,
) -> u8 {
    let _ = Fcb;
    let _ = Wait;

    if KeGetCurrentIrql() > 1 {
        DbgBreakPoint();
    }

    ntkrnl_sys::ASSERT!(IoGetTopLevelIrp().is_null());

    IoSetTopLevelIrp((1 as *mut core::ffi::c_void).cast());

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn FatNoOpRelease(Fcb: *mut core::ffi::c_void) {
    PAGED_CODE!();
    NT_ASSERT((IoGetTopLevelIrp() as *mut core::ffi::c_void == (FSRTL_CACHE_TOP_LEVEL_IRP).cast()) as i32);
    IoSetTopLevelIrp(core::ptr::null_mut());
    UNREFERENCED_PARAMETER(Fcb);
}

#[no_mangle]
pub unsafe extern "system" fn FatFilterCallbackAcquireForCreateSection(
    CallbackData: *mut FS_FILTER_CALLBACK_DATA,
    CompletionContext: *mut *mut core::ffi::c_void,
) -> i32 {
    let _ = CompletionContext; // UNREFERENCED_PARAMETER

    PAGED_CODE!();

    NT_ASSERT(((*CallbackData).Operation == (FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION) as u8) as i32);
    NT_ASSERT(((*CallbackData).SizeOfFsFilterCallbackData == (core::mem::size_of::<FS_FILTER_CALLBACK_DATA>()) as u32) as i32);

    let fcb: *mut FCB = (*(*CallbackData).FileObject).FsContext as *mut FCB;

    if !(*fcb).Header.Resource.is_null() {
        ExAcquireResourceExclusiveLite((*fcb).Header.Resource, 1);
    }

    if (*CallbackData).Parameters.AcquireForSectionSynchronization.SyncType != SyncTypeCreateSection {
        return STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY;
    } else if (*fcb).ShareAccess.Writers == 0 {
        return STATUS_FILE_LOCKED_WITH_ONLY_READERS;
    } else {
        return STATUS_FILE_LOCKED_WITH_WRITERS;
    }
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
//     This module implements the Fat Resource acquisition routines
//
//
// --*/
//
// #include "fatprocs.h"
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatAcquireFcbForLazyWrite)
// #pragma alloc_text(PAGE, FatAcquireFcbForReadAhead)
// #pragma alloc_text(PAGE, FatAcquireExclusiveFcb)
// #pragma alloc_text(PAGE, FatAcquireSharedFcb)
// #pragma alloc_text(PAGE, FatAcquireSharedFcbWaitForEx)
// #pragma alloc_text(PAGE, FatAcquireExclusiveVcb_Real)
// #pragma alloc_text(PAGE, FatAcquireSharedVcb)
// #pragma alloc_text(PAGE, FatNoOpAcquire)
// #pragma alloc_text(PAGE, FatNoOpRelease)
// #pragma alloc_text(PAGE, FatReleaseFcbFromLazyWrite)
// #pragma alloc_text(PAGE, FatReleaseFcbFromReadAhead)
// #pragma alloc_text(PAGE, FatAcquireForCcFlush)
// #pragma alloc_text(PAGE, FatReleaseForCcFlush)
// #pragma alloc_text(PAGE, FatFilterCallbackAcquireForCreateSection)
// #endif
//
// _Requires_lock_held_(_Global_critical_region_)
// (return != FALSE && NoOpCheck != FALSE, _Acquires_exclusive_lock_(Vcb->Resource))
// FINISHED
// FatAcquireExclusiveVcb_Real (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN BOOLEAN NoOpCheck
//     )
//
// /*++
//
// Routine Description:
//
//     This routine acquires exclusive access to the Vcb.
//
//     After we acquire the resource check to see if this operation is legal.
//     If it isn't (ie. we get an exception), release the resource.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to acquire
//
//     NoOpCheck - if TRUE then don't do any verification of the request/volume state.
//
// Return Value:
//
//     FINISHED - TRUE if we have the resource and FALSE if we needed to block
//         for the resource but Wait is FALSE.
//
// --*/
//
// {
//     PAGED_CODE();
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #endif
//     if (ExAcquireResourceExclusiveLite( &Vcb->Resource, BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT))) {
//
//         if (!NoOpCheck) {
//
//             _SEH2_TRY {
//
//                 FatVerifyOperationIsLegal( IrpContext );
//
//             } _SEH2_FINALLY {
//
//                 if ( _SEH2_AbnormalTermination() ) {
//
//                     FatReleaseVcb( IrpContext, Vcb );
//                 }
//             } _SEH2_END;
//         }
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// (return != 0, _Acquires_shared_lock_(Vcb->Resource))
// FINISHED
// FatAcquireSharedVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine acquires shared access to the Vcb.
//
//     After we acquire the resource check to see if this operation is legal.
//     If it isn't (ie. we get an exception), release the resource.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to acquire
//
// Return Value:
//
//     FINISHED - TRUE if we have the resource and FALSE if we needed to block
//         for the resource but Wait is FALSE.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     if (ExAcquireResourceSharedLite( &Vcb->Resource,
//                                      BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT))) {
//
//         _SEH2_TRY {
//
//             FatVerifyOperationIsLegal( IrpContext );
//
//         } _SEH2_FINALLY {
//
//             if ( _SEH2_AbnormalTermination() ) {
//
//                 FatReleaseVcb( IrpContext, Vcb );
//             }
//         } _SEH2_END;
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Acquires_exclusive_lock_(*Fcb->Header.Resource)
// FINISHED
// FatAcquireExclusiveFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine acquires exclusive access to the Fcb.
//
//     After we acquire the resource check to see if this operation is legal.
//     If it isn't (ie. we get an exception), release the resource.
//
// Arguments:
//
//     Fcb - Supplies the Fcb to acquire
//
// Return Value:
//
//     FINISHED - TRUE if we have the resource and FALSE if we needed to block
//         for the resource but Wait is FALSE.
//
// --*/
//
// {
//     PAGED_CODE();
//
// RetryFcbExclusive:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #endif
//     if (ExAcquireResourceExclusiveLite( Fcb->Header.Resource, BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT))) {
//
//         //
//         //  Check for anything other than a non-cached write if the
//         //  async count is non-zero in the Fcb, or if others are waiting
//         //  for the resource.  Then wait for all outstanding I/O to finish,
//         //  drop the resource, and wait again.
//         //
//
//         if ((Fcb->NonPaged->OutstandingAsyncWrites != 0) &&
//             ((IrpContext->MajorFunction != IRP_MJ_WRITE) ||
//              !FlagOn(IrpContext->OriginatingIrp->Flags, IRP_NOCACHE) ||
//              (ExGetSharedWaiterCount(Fcb->Header.Resource) != 0) ||
//              (ExGetExclusiveWaiterCount(Fcb->Header.Resource) != 0))) {
//
//             KeWaitForSingleObject( Fcb->NonPaged->OutstandingAsyncEvent,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    (PLARGE_INTEGER) NULL );
//
//             FatReleaseFcb( IrpContext, Fcb );
//
//             goto RetryFcbExclusive;
//         }
//
//         _SEH2_TRY {
//
//             FatVerifyOperationIsLegal( IrpContext );
//
//         } _SEH2_FINALLY {
//
//             if ( _SEH2_AbnormalTermination() ) {
//
//                 FatReleaseFcb( IrpContext, Fcb );
//             }
//         } _SEH2_END;
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
//  _Requires_lock_held_(_Global_critical_region_)
// _Acquires_shared_lock_(*Fcb->Header.Resource)
// FINISHED
// FatAcquireSharedFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine acquires shared access to the Fcb.
//
//     After we acquire the resource check to see if this operation is legal.
//     If it isn't (ie. we get an exception), release the resource.
//
// Arguments:
//
//     Fcb - Supplies the Fcb to acquire
//
// Return Value:
//
//     FINISHED - TRUE if we have the resource and FALSE if we needed to block
//         for the resource but Wait is FALSE.
//
// --*/
//
// {
//     PAGED_CODE();
//
// RetryFcbShared:
//
//     if (ExAcquireResourceSharedLite( Fcb->Header.Resource, BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT))) {
//
//         //
//         //  Check for anything other than a non-cached write if the
//         //  async count is non-zero in the Fcb, or if others are waiting
//         //  for the resource.  Then wait for all outstanding I/O to finish,
//         //  drop the resource, and wait again.
//         //
//
//         if ((Fcb->NonPaged->OutstandingAsyncWrites != 0) &&
//             ((IrpContext->MajorFunction != IRP_MJ_WRITE) ||
//              !FlagOn(IrpContext->OriginatingIrp->Flags, IRP_NOCACHE) ||
//              (ExGetSharedWaiterCount(Fcb->Header.Resource) != 0) ||
//              (ExGetExclusiveWaiterCount(Fcb->Header.Resource) != 0))) {
//
//             KeWaitForSingleObject( Fcb->NonPaged->OutstandingAsyncEvent,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    (PLARGE_INTEGER) NULL );
//
//             FatReleaseFcb( IrpContext, Fcb );
//
//             goto RetryFcbShared;
//         }
//
//         _SEH2_TRY {
//
//             FatVerifyOperationIsLegal( IrpContext );
//
//         } _SEH2_FINALLY {
//
//             if ( _SEH2_AbnormalTermination() ) {
//
//                 FatReleaseFcb( IrpContext, Fcb );
//             }
//         } _SEH2_END;
//
//
//         return TRUE;
//
//     } else {
//
//         return FALSE;
//     }
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// (return != 0, _Acquires_shared_lock_(*Fcb->Header.Resource))
// FINISHED
// FatAcquireSharedFcbWaitForEx (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine acquires shared access to the Fcb, waiting first for any
//     exclusive accessors to get the Fcb first.
//
//     After we acquire the resource check to see if this operation is legal.
//     If it isn't (ie. we get an exception), release the resource.
//
// Arguments:
//
//     Fcb - Supplies the Fcb to acquire
//
// Return Value:
//
//     FINISHED - TRUE if we have the resource and FALSE if we needed to block
//         for the resource but Wait is FALSE.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     NT_ASSERT( FlagOn(IrpContext->OriginatingIrp->Flags, IRP_NOCACHE) );
//     NT_ASSERT( !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
// RetryFcbSharedWaitEx:
//
//     if (ExAcquireSharedWaitForExclusive( Fcb->Header.Resource, FALSE )) {
//
//         //
//         //  Check for anything other than a non-cached write if the
//         //  async count is non-zero in the Fcb. Then wait for all
//         //  outstanding I/O to finish, drop the resource, and wait again.
//         //
//
//         if ((Fcb->NonPaged->OutstandingAsyncWrites != 0) &&
//             (IrpContext->MajorFunction != IRP_MJ_WRITE)) {
//
//             KeWaitForSingleObject( Fcb->NonPaged->OutstandingAsyncEvent,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    (PLARGE_INTEGER) NULL );
//
//             FatReleaseFcb( IrpContext, Fcb );
//
//             goto RetryFcbSharedWaitEx;
//         }
//
//         _SEH2_TRY {
//
//             FatVerifyOperationIsLegal( IrpContext );
//
//         } _SEH2_FINALLY {
//
//             if ( _SEH2_AbnormalTermination() ) {
//
//                 FatReleaseFcb( IrpContext, Fcb );
//             }
//         } _SEH2_END;
//
//
//         return TRUE;
//
//     } else {
//
//         return FALSE;
//     }
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// NTAPI
// FatAcquireFcbForLazyWrite (
//     IN PVOID Fcb,
//     IN BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a file.  It is subsequently called by the Lazy Writer prior to its
//     performing lazy writes to the file.
//
// Arguments:
//
//     Fcb - The Fcb which was specified as a context parameter for this
//           routine.
//
//     Wait - TRUE if the caller is willing to block.
//
// Return Value:
//
//     FALSE - if Wait was specified as FALSE and blocking would have
//             been required.  The Fcb is not acquired.
//
//     TRUE - if the Fcb has been acquired
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Check here for the EA File.  It turns out we need the normal
//     //  resource shared in this case.  Otherwise we take the paging
//     //  I/O resource shared.
//     //
//
//     //
//     //  Note that we do not need to disable APC delivery to guard
//     //  against a rogue user issuing a suspend APC. That is because
//     //  it is guaranteed that the caller is either in the system context,
//     //  to which a user cannot deliver a suspend APC, or the caller has
//     //  already disabled kernel APC delivery before calling. This is true
//     //  for all the other pre-acquire routines as well.
//     //
//
//     if (!ExAcquireResourceSharedLite( Fcb == ((PFCB)Fcb)->Vcb->EaFcb ?
//                                   ((PFCB)Fcb)->Header.Resource :
//                                   ((PFCB)Fcb)->Header.PagingIoResource,
//                                   Wait )) {
//
//         return FALSE;
//     }
//
//     //
//     // We assume the Lazy Writer only acquires this Fcb once.
//     // Therefore, it should be guaranteed that this flag is currently
//     // clear (the ASSERT), and then we will set this flag, to insure
//     // that the Lazy Writer will never try to advance Valid Data, and
//     // also not deadlock by trying to get the Fcb exclusive.
//     //
//
//
//     NT_ASSERT( NodeType(((PFCB)Fcb)) == FAT_NTC_FCB );
//     NT_ASSERT( ((PFCB)Fcb)->Specific.Fcb.LazyWriteThread == NULL );
//
//     ((PFCB)Fcb)->Specific.Fcb.LazyWriteThread = PsGetCurrentThread();
//
//     NT_ASSERT( NULL != PsGetCurrentThread() );
//
//     if (NULL == FatData.LazyWriteThread) {
//
//         FatData.LazyWriteThread = PsGetCurrentThread();
//     }
//
//     //
//     //  This is a kludge because Cc is really the top level.  When it
//     //  enters the file system, we will think it is a resursive call
//     //  and complete the request with hard errors or verify.  It will
//     //  then have to deal with them, somehow....
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == NULL);
//
//     IoSetTopLevelIrp((PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     return TRUE;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI
// FatReleaseFcbFromLazyWrite (
//     IN PVOID Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a file.  It is subsequently called by the Lazy Writer after its
//     performing lazy writes to the file.
//
// Arguments:
//
//     Fcb - The Fcb which was specified as a context parameter for this
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
//     //
//     //  Assert that this really is an fcb and that this thread really owns
//     //  the lazy writer mark in the fcb.
//     //
//
//     NT_ASSERT( NodeType(((PFCB)Fcb)) == FAT_NTC_FCB );
//     NT_ASSERT( NULL != PsGetCurrentThread() );
//     NT_ASSERT( ((PFCB)Fcb)->Specific.Fcb.LazyWriteThread == PsGetCurrentThread() );
//
//     //
//     //  Release the lazy writer mark.
//     //
//
//     ((PFCB)Fcb)->Specific.Fcb.LazyWriteThread = NULL;
//
//     //
//     //  Check here for the EA File.  It turns out we needed the normal
//     //  resource shared in this case.  Otherwise it was the PagingIoResource.
//     //
//
//     ExReleaseResourceLite( Fcb == ((PFCB)Fcb)->Vcb->EaFcb ?
//                        ((PFCB)Fcb)->Header.Resource :
//                        ((PFCB)Fcb)->Header.PagingIoResource );
//
//     //
//     //  Clear the kludge at this point.
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     IoSetTopLevelIrp( NULL );
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// NTAPI
// FatAcquireFcbForReadAhead (
//     IN PVOID Fcb,
//     IN BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a file.  It is subsequently called by the Lazy Writer prior to its
//     performing read ahead to the file.
//
// Arguments:
//
//     Fcb - The Fcb which was specified as a context parameter for this
//           routine.
//
//     Wait - TRUE if the caller is willing to block.
//
// Return Value:
//
//     FALSE - if Wait was specified as FALSE and blocking would have
//             been required.  The Fcb is not acquired.
//
//     TRUE - if the Fcb has been acquired
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  We acquire the normal file resource shared here to synchronize
//     //  correctly with purges.
//     //
//
//     //
//     //  Note that we do not need to disable APC delivery to guard
//     //  against a rogue user issuing a suspend APC. That is because
//     //  it is guaranteed that the caller is either in the system context,
//     //  to which a user cannot deliver a suspend APC, or the caller has
//     //  already disabled kernel APC delivery before calling. This is true
//     //  for all the other pre-acquire routines as well.
//     //
//
//     if (!ExAcquireResourceSharedLite( ((PFCB)Fcb)->Header.Resource,
//                                   Wait )) {
//
//         return FALSE;
//     }
//
//     //
//     //  This is a kludge because Cc is really the top level.  We it
//     //  enters the file system, we will think it is a resursive call
//     //  and complete the request with hard errors or verify.  It will
//     //  have to deal with them, somehow....
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == NULL);
//
//     IoSetTopLevelIrp((PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     return TRUE;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI
// FatReleaseFcbFromReadAhead (
//     IN PVOID Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     The address of this routine is specified when creating a CacheMap for
//     a file.  It is subsequently called by the Lazy Writer after its
//     read ahead.
//
// Arguments:
//
//     Fcb - The Fcb which was specified as a context parameter for this
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
//     //
//     //  Clear the kludge at this point.
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     IoSetTopLevelIrp( NULL );
//
//     ExReleaseResourceLite( ((PFCB)Fcb)->Header.Resource );
//
//     return;
// }
//
//
// _Function_class_(FAST_IO_ACQUIRE_FOR_CCFLUSH)
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// NTAPI
// FatAcquireForCcFlush (
//     IN PFILE_OBJECT FileObject,
//     IN PDEVICE_OBJECT DeviceObject
//     )
// {
//     PFCB Fcb;
//     PCCB Ccb;
//     PVCB Vcb;
//     PFSRTL_COMMON_FCB_HEADER Header;
//     TYPE_OF_OPEN Type;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Once again, the hack for making this look like
//     //  a recursive call if needed. We cannot let ourselves
//     //  verify under something that has resources held.
//     //
//     //  This value is good.  We should never try to acquire
//     //  the file this way underneath of the cache.
//     //
//
//     NT_ASSERT( IoGetTopLevelIrp() != (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP );
//
//     if (IoGetTopLevelIrp() == NULL) {
//
//         IoSetTopLevelIrp((PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//     }
//
//     //
//     //  Time for some exposition.
//     //
//     //  Lockorder for FAT is main->bcb->pagingio. Invert this at your obvious peril.
//     //  The default logic for AcquireForCcFlush breaks this since in writethrough
//     //  unpinrepinned we will grab the bcb then Mm will use the callback (which
//     //  orders us with respect to the MmCollidedFlushEvent) to help us. If for
//     //  directories/ea we then grab the main we are out of order.
//     //
//     //  Fortunately, we do not need main. We only need paging - just look at the write
//     //  path. This is basic pre-acquisition.
//     //
//     //  Regular files require both resources, and are safe since we never pin them.
//     //
//
//     //
//     //  Note that we do not need to disable APC delivery to guard
//     //  against a rogue user issuing a suspend APC. That is because
//     //  it is guaranteed that the caller is either in the system context,
//     //  to which a user cannot deliver a suspend APC, or the caller has
//     //  already disabled kernel APC delivery before calling. This is true
//     //  for all the other pre-acquire routines as well.
//     //
//
//     Type = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//     Header = (PFSRTL_COMMON_FCB_HEADER) FileObject->FsContext;
//
//     if (Type < DirectoryFile) {
//
//         if (Header->Resource) {
//
//             if (!ExIsResourceAcquiredSharedLite( Header->Resource )) {
//
//                 ExAcquireResourceExclusiveLite( Header->Resource, TRUE );
//
//             } else {
//
//                 ExAcquireResourceSharedLite( Header->Resource, TRUE );
//             }
//         }
//     }
//
//     if (Header->PagingIoResource) {
//
//         ExAcquireResourceSharedLite( Header->PagingIoResource, TRUE );
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// NTAPI
// FatReleaseForCcFlush (
//     IN PFILE_OBJECT FileObject,
//     IN PDEVICE_OBJECT DeviceObject
//     )
// {
//     PFCB Fcb;
//     PCCB Ccb;
//     PVCB Vcb;
//     PFSRTL_COMMON_FCB_HEADER Header;
//     TYPE_OF_OPEN Type;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Clear up our hint.
//     //
//
//     if (IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP) {
//
//         IoSetTopLevelIrp( NULL );
//     }
//
//     Type = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//     Header = (PFSRTL_COMMON_FCB_HEADER) FileObject->FsContext;
//
//     if (Type < DirectoryFile) {
//
//         if (Header->Resource) {
//
//             ExReleaseResourceLite( Header->Resource );
//         }
//     }
//
//     if (Header->PagingIoResource) {
//
//         ExReleaseResourceLite( Header->PagingIoResource );
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// BOOLEAN
// NTAPI
// FatNoOpAcquire (
//     IN PVOID Fcb,
//     IN BOOLEAN Wait
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
//     Fcb - The Fcb/Dcb/Vcb which was specified as a context parameter for this
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
//     UNREFERENCED_PARAMETER( Fcb );
//     UNREFERENCED_PARAMETER( Wait );
//
//     PAGED_CODE();
//
//     //
//     //  This is a kludge because Cc is really the top level.  We it
//     //  enters the file system, we will think it is a resursive call
//     //  and complete the request with hard errors or verify.  It will
//     //  have to deal with them, somehow....
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == NULL);
//
//     IoSetTopLevelIrp((PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     return TRUE;
// }
//
//
// VOID
// NTAPI
// FatNoOpRelease (
//     IN PVOID Fcb
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
//     Fcb - The Fcb/Dcb/Vcb which was specified as a context parameter for this
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
//     //
//     //  Clear the kludge at this point.
//     //
//
//     NT_ASSERT(IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP);
//
//     IoSetTopLevelIrp( NULL );
//
//     UNREFERENCED_PARAMETER( Fcb );
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// NTAPI
// FatFilterCallbackAcquireForCreateSection (
//     IN PFS_FILTER_CALLBACK_DATA CallbackData,
//     OUT PVOID *CompletionContext
//     )
//
// /*++
//
// Routine Description:
//
//     This is the callback routine for MM to use to acquire the file exclusively.
//
//     NOTE:  This routine expects the default FSRTL routine to be used to release
//            the resource.  If this routine is ever changed to acquire something
//            other than main, a corresponding release routine will be required.
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
//     If SyncType is SyncTypeCreateSection, we return a status that indicates whether there
//     are any writers to this file.  Note that main is acquired, so new handles cannot be opened.
//
// --*/
//
// {
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     NT_ASSERT( CallbackData->Operation == FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION );
//     NT_ASSERT( CallbackData->SizeOfFsFilterCallbackData == sizeof(FS_FILTER_CALLBACK_DATA) );
//
//     //
//     //  Grab the Fcb from the callback data file object.
//     //
//
//     Fcb = CallbackData->FileObject->FsContext;
//
//     //
//     //  Take main exclusive.
//     //
//
//     //
//     //  Note that we do not need to disable APC delivery to guard
//     //  against a rogue user issuing a suspend APC. That is because
//     //  it is guaranteed that the caller is either in the system context,
//     //  to which a user cannot deliver a suspend APC, or the caller has
//     //  already disabled kernel APC delivery before calling. This is true
//     //  for all the other pre-acquire routines as well.
//     //
//
//     if (Fcb->Header.Resource) {
//
//         ExAcquireResourceExclusiveLite( Fcb->Header.Resource, TRUE );
//     }
//
//     //
//     //  Return the appropriate status based on the type of synchronization and whether anyone
//     //  has write access to this file.
//     //
//
//     if (CallbackData->Parameters.AcquireForSectionSynchronization.SyncType != SyncTypeCreateSection) {
//
//         return STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY;
//
//     } else if (Fcb->ShareAccess.Writers == 0) {
//
//         return STATUS_FILE_LOCKED_WITH_ONLY_READERS;
//
//     } else {
//
//         return STATUS_FILE_LOCKED_WITH_WRITERS;
//     }
//
//     UNREFERENCED_PARAMETER( CompletionContext );
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
