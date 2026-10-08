use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PIRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FCB, IRP_CONTEXT, IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS, IRP_CONTEXT_LITE, PCCB, PFCB, PIRP_CONTEXT, PIRP_CONTEXT_LITE, PVCB, THREAD_CONTEXT, TYPE_OF_OPEN, VCB, _TYPE_OF_OPEN_UnopenedFileObject as UnopenedFileObject, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _VCB_CONDITION_VcbMountInProgress as VcbMountInProgress, _VCB_CONDITION_VcbMounted as VcbMounted};
use ntkrnl_sys::ddk::{IO_WORKITEM, PFILE_OBJECT, WORK_QUEUE_ITEM, WORK_QUEUE_TYPE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use crate::glue::{CDFS_BUG_CHECK_CLOSE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdData};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
    fn CdDecodeFileObject( IrpContext: PIRP_CONTEXT, FileObject: PFILE_OBJECT, Fcb: *mut PFCB, Ccb: *mut PCCB, ) -> TYPE_OF_OPEN;
}

extern "C" {
}
extern "C" {
    fn ASSERT_IRP_CONTEXT(p: PIRP_CONTEXT);
}
extern "C" {
    fn ASSERT_FCB(p: PFCB);
}
extern "C" {
    fn CdCreateIrpContextLite(IrpContext: PIRP_CONTEXT) -> *mut IRP_CONTEXT_LITE;
}
extern "C" {
    fn CdLockCdData();
}
extern "C" {
    fn CdUnlockCdData();
}
extern "C" {
    fn CdCleanupIrpContext(IrpContext: PIRP_CONTEXT, arg1: u32);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}
extern "C" {
    fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY);
}
extern "system" {
    fn IoQueueWorkItem( IoObject: *mut IO_WORKITEM, WorkerRoutine: unsafe extern "system" fn(PDEVICE_OBJECT, *mut core::ffi::c_void), QueueType: WORK_QUEUE_TYPE, Context: *mut core::ffi::c_void, );
}
extern "system" {
    fn ASSERT_OPTIONAL_VCB(Vcb: PVCB);
}
extern "C" {
    fn CdAcquireCdData(IrpContext: PIRP_CONTEXT);
}
extern "C" {
    fn CdReleaseCdData(IrpContext: PIRP_CONTEXT);
}
extern "C" {
    fn CdDeleteCcb(IrpContext: PIRP_CONTEXT, Ccb: PCCB);
}
extern "C" {
    fn CdCheckForDismount(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Param: i32) -> u32;
}
extern "C" {
    fn FlagOn(Flags: u32, Flag: u32) -> i32;
}
extern "C" {
    fn CdAcquireVcbShared(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Wait: u8) -> BOOLEAN;
}
extern "C" {
    fn CdFreeIrpContextLite(Ptr: *mut core::ffi::c_void);
}
extern "C" {
    fn CdInitializeStackIrpContext(StackIrpContext: *mut IRP_CONTEXT, Lite: *mut IRP_CONTEXT_LITE);
}
extern "C" {
    fn CdReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdSetThreadContext(IrpContext: *mut IRP_CONTEXT, ThreadContext: *mut THREAD_CONTEXT);
}
extern "C" {
    fn SafeNodeType(Node: *mut IRP_CONTEXT) -> u32;
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Mask: u32);
}
extern "C" {
    fn ClearFlag(Flags: *mut u32, Mask: u32);
}
extern "C" {
    // Additional cdfs-internal NTAPI routines used by the (re-)translated bodies below.
    fn CdAcquireFcbExclusive(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Wait: BOOLEAN) -> u32;
}
extern "C" {
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdDecrementReferenceCounts(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, RefCount: u32, UserRefCount: u32);
}
extern "C" {
    fn CdTeardownStructures(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, RemovedFcb: *mut BOOLEAN);
}
extern "C" {
    fn CdReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_CLOSE;

// ---- Local constants (verified against drivers/filesystems/cdfs headers) ----
// nodetype.h: CDFS_NTC_IRP_CONTEXT_LITE = 0x0309
const CDFS_NTC_IRP_CONTEXT_LITE: u16 = 0x0309;
// cdstruc.h flag bits.
const IRP_CONTEXT_FLAG_MORE_PROCESSING: u32 = 0x00000002;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
const IRP_CONTEXT_FLAG_TOP_LEVEL: u32 = 0x00000010;
// IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS (0x20) imported from crate::types.
const IRP_CONTEXT_FLAG_IN_FSP: u32 = 0x00000040; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH
// cdstruc.h: IRP_CONTEXT_FSP_FLAGS = WAIT | TOP_LEVEL | TOP_LEVEL_CDFS | IN_FSP
#[allow(dead_code)]
const IRP_CONTEXT_FSP_FLAGS: u32 =
    IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_TOP_LEVEL | IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS | IRP_CONTEXT_FLAG_IN_FSP;
// extypes.h _WORK_QUEUE_TYPE::CriticalWorkQueue = 0
const CriticalWorkQueue: WORK_QUEUE_TYPE = 0;

//
//  Local support routine
//
//  _Requires_lock_held_(_Global_critical_region_)
//  BOOLEAN CdCommonClosePrivate( IrpContext, Vcb, Fcb, UserReference, FromFsd )
//
//  This is the worker routine for the close operation.  Returns TRUE if this
//  thread processed the close, FALSE if the caller should post the request.
//  cdfs-internal => plain cdecl on all arches (no NTAPI in C prototype).
#[no_mangle]
pub unsafe extern "C" fn CdCommonClosePrivate(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Fcb: PFCB,
    UserReference: ULONG,
    FromFsd: BOOLEAN,
) -> BOOLEAN {
    let mut RemovedFcb: BOOLEAN = 0;

    // PAGED_CODE();
    // ASSERT_IRP_CONTEXT( IrpContext );
    // ASSERT_FCB( Fcb );

    //
    //  Try to acquire the Vcb and Fcb.  If we can't acquire them then return
    //  FALSE and let our caller post the request to the async queue.
    //
    if CdAcquireVcbShared(IrpContext, Vcb, (FromFsd as i32) as _) != 0 {
        if CdAcquireFcbExclusive(IrpContext, Fcb, FromFsd) == 0 {
            //
            //  We couldn't get the Fcb.  Release the Vcb and let our caller
            //  know to post this request.
            //
            CdReleaseVcb(IrpContext, Vcb);
            return 0; // FALSE
        }
    } else {
        //
        //  We didn't get the Vcb.  Let our caller know to post this request.
        //
        return 0; // FALSE
    }

    //
    //  Lock the Vcb and decrement the reference counts.
    //
    CdLockVcb(IrpContext, Vcb);
    CdDecrementReferenceCounts(IrpContext, Fcb, 1, UserReference);
    CdUnlockVcb(IrpContext, Vcb);

    //
    //  Call our teardown routine to see if this object can go away.
    //  If we don't remove the Fcb then release it.
    //
    CdTeardownStructures(IrpContext, Fcb, &mut RemovedFcb);

    if RemovedFcb == 0 {
        CdReleaseFcb(IrpContext, Fcb);
    } else {
        // _Analysis_assume_lock_not_held_(Fcb->FcbNonpaged->FcbResource);
    }

    //
    //  Release the Vcb and return to our caller (we completed this request).
    //
    CdReleaseVcb(IrpContext, Vcb);

    1 // TRUE
}

//
//  CdCloseWorker is NTAPI (stdcall on i386) — IO_WORKITEM_ROUTINE.
//  Worker routine to call CdFspClose.
//  IO_WORKITEM_ROUTINE — NTAPI (stdcall on i386); must be "system" to match the
//  IoQueueWorkItem callback fn-pointer type.
#[no_mangle]
pub unsafe extern "system" fn CdCloseWorker(DeviceObject: PDEVICE_OBJECT, Context: PVOID) {
    // PAGED_CODE();
    let _ = DeviceObject; // UNREFERENCED_PARAMETER
    let _ = Context; // UNREFERENCED_PARAMETER

    CdFspClose(core::ptr::null_mut());
}

//
//  VOID CdQueueClose( IrpContext, Fcb, UserReference, DelayedClose )
//
//  Queue a close request to either the async or delayed close queue.  For the
//  delayed queue we allocate a smaller IRP_CONTEXT_LITE; if that allocation
//  fails we fall back to the async queue using the original IrpContext.
//  cdfs-internal => plain cdecl.
#[no_mangle]
pub unsafe extern "C" fn CdQueueClose(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    UserReference: ULONG,
    DelayedClose: BOOLEAN,
) {
    let mut IrpContextLite: *mut IRP_CONTEXT_LITE = core::ptr::null_mut();
    let mut StartWorker: BOOLEAN = 0; // FALSE

    // PAGED_CODE();
    // ASSERT_IRP_CONTEXT( IrpContext );
    // ASSERT_FCB( Fcb );

    //
    //  Start with the delayed queue request.  We can move this to the async
    //  queue if there is an allocation failure.
    //
    if DelayedClose != 0 {
        //
        //  Try to allocate non-paged pool for the IRP_CONTEXT_LITE.
        //
        IrpContextLite = CdCreateIrpContextLite(IrpContext);
    }

    //
    //  We want to clear the top level context in this thread if
    //  necessary.  Call our cleanup routine to do the work.
    //
    SetFlag(&raw mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING);
    CdCleanupIrpContext(IrpContext, 1 /* TRUE */);

    //
    //  Synchronize with the CdData lock.
    //
    CdLockCdData();

    //
    //  If we have an IrpContextLite then put the request on the delayed close queue.
    //
    if !IrpContextLite.is_null() {
        //
        //  Initialize the IrpContextLite.
        //
        (*IrpContextLite).NodeTypeCode = CDFS_NTC_IRP_CONTEXT_LITE;
        (*IrpContextLite).NodeByteSize = core::mem::size_of::<IRP_CONTEXT_LITE>() as _;
        (*IrpContextLite).Fcb = Fcb;
        (*IrpContextLite).UserReference = UserReference;
        (*IrpContextLite).RealDevice = (*IrpContext).RealDevice;

        //
        //  Add this to the delayed close list and increment the count.
        //
        InsertTailList(
            &raw mut CdData.DelayedCloseQueue,
            &raw mut (*IrpContextLite).DelayedCloseLinks,
        );

        CdData.DelayedCloseCount += 1;

        //
        //  If we are above our threshold then start the delayed close operation.
        //
        if CdData.DelayedCloseCount > CdData.MaxDelayedCloseCount {
            CdData.ReduceDelayedClose = 1; // TRUE

            if CdData.FspCloseActive == 0 {
                CdData.FspCloseActive = 1; // TRUE
                StartWorker = 1; // TRUE
            }
        }

        //
        //  Unlock the CdData.
        //
        CdUnlockCdData();

        //
        //  Cleanup the IrpContext.
        //
        CdCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);

    //
    //  Otherwise drop into the async case below.
    //
    } else {
        //
        //  Store the information about the file object into the IrpContext.
        //
        (*IrpContext).Irp = Fcb as PIRP;
        (*IrpContext).ExceptionStatus = UserReference as NTSTATUS;

        //
        //  Add this to the async close list and increment the count.
        //
        InsertTailList(
            &raw mut CdData.AsyncCloseQueue,
            &raw mut (*IrpContext).WorkQueueItem.List,
        );

        CdData.AsyncCloseCount += 1;

        //
        //  Remember to start the Fsp close thread if not currently started.
        //
        if CdData.FspCloseActive == 0 {
            CdData.FspCloseActive = 1; // TRUE
            StartWorker = 1; // TRUE
        }

        //
        //  Unlock the CdData.
        //
        CdUnlockCdData();
    }

    //
    //  Start the FspClose thread if we need to.
    //
    if StartWorker != 0 {
        IoQueueWorkItem(
            CdData.CloseItem,
            CdCloseWorker,
            CriticalWorkQueue,
            core::ptr::null_mut(),
        );
    }
}

//
//  Local support routine
//
//  PIRP_CONTEXT CdRemoveClose( Vcb )
//
//  Scan the async and delayed close queues for a suitable entry.  If Vcb is
//  specified we scan both queues looking for an entry with that Vcb.  Otherwise
//  we look in the async queue first, then the delayed queue (only if the delayed
//  close operation is active).  Returns NULL if no work item found, else the
//  IrpContext or IrpContextLite for the request.
//  cdfs-internal => plain cdecl.  (A caller references it undecorated as
//  `_CdRemoveClose`, i.e. the standard i386 cdecl single-underscore decoration.)
#[no_mangle]
pub unsafe extern "C" fn CdRemoveClose(Vcb: PVCB) -> PIRP_CONTEXT {
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let mut NextIrpContext: PIRP_CONTEXT;
    let mut NextIrpContextLite: PIRP_CONTEXT_LITE;
    let mut Entry: PLIST_ENTRY;

    // PAGED_CODE();
    // ASSERT_OPTIONAL_VCB( Vcb );

    //
    //  Lock the CdData to perform the scan.
    //
    CdLockCdData();

    //
    //  First check the list of async closes.
    //
    Entry = CdData.AsyncCloseQueue.Flink;

    while Entry != &raw mut CdData.AsyncCloseQueue {
        //
        //  Extract the IrpContext.  WorkQueueItem.List is the embedded link;
        //  CONTAINING_RECORD( Entry, IRP_CONTEXT, WorkQueueItem.List ).
        //
        NextIrpContext = (Entry as *mut u8)
            .sub(core::mem::offset_of!(IRP_CONTEXT, WorkQueueItem)
                + core::mem::offset_of!(WORK_QUEUE_ITEM, List)) as PIRP_CONTEXT;

        //
        //  If no Vcb was specified or this Vcb is for our volume then close it.
        //
        if Vcb.is_null() || (*NextIrpContext).Vcb == Vcb {
            RemoveEntryList(Entry);
            CdData.AsyncCloseCount -= 1;

            IrpContext = NextIrpContext;
            break;
        }

        //
        //  Move to the next entry.
        //
        Entry = (*Entry).Flink;
    }

    //
    //  If we didn't find anything look through the delayed close queue.
    //  We only check it if we were given a Vcb or the delayed close
    //  operation is active.
    //
    if IrpContext.is_null()
        && (!Vcb.is_null()
            || (CdData.ReduceDelayedClose != 0
                && CdData.DelayedCloseCount > CdData.MinDelayedCloseCount))
    {
        Entry = CdData.DelayedCloseQueue.Flink;

        while Entry != &raw mut CdData.DelayedCloseQueue {
            //
            //  Extract the IrpContextLite.
            //  CONTAINING_RECORD( Entry, IRP_CONTEXT_LITE, DelayedCloseLinks ).
            //
            NextIrpContextLite = (Entry as *mut u8)
                .sub(core::mem::offset_of!(IRP_CONTEXT_LITE, DelayedCloseLinks))
                as PIRP_CONTEXT_LITE;

            //
            //  If no Vcb was specified or this Vcb is for our volume then close it.
            //
            if Vcb.is_null() || (*(*NextIrpContextLite).Fcb).Vcb == Vcb {
                RemoveEntryList(Entry);
                CdData.DelayedCloseCount -= 1;

                IrpContext = NextIrpContextLite as PIRP_CONTEXT;
                break;
            }

            //
            //  Move to the next entry.
            //
            Entry = (*Entry).Flink;
        }
    }

    //
    //  If the Vcb wasn't specified and we couldn't find an entry then turn
    //  off the Fsp thread.
    //
    if Vcb.is_null() && IrpContext.is_null() {
        CdData.FspCloseActive = 0; // FALSE
        CdData.ReduceDelayedClose = 0; // FALSE
    }

    //
    //  Unlock the CdData.
    //
    CdUnlockCdData();

    IrpContext
}
#[no_mangle]
pub unsafe extern "C" fn CdFspClose(Vcb: *mut VCB) {
    // Local variables
    let mut IrpContext: *mut IRP_CONTEXT;
    let mut stack_irp_context: IRP_CONTEXT = core::mem::zeroed();
    let mut thread_context: THREAD_CONTEXT = core::mem::zeroed();
    let mut Fcb: *mut FCB;
    let mut UserReference: u32;
    let mut VcbHoldCount: u32 = 0;
    let mut CurrentVcb: *mut VCB = core::ptr::null_mut();
    let mut PotentialVcbTeardown: i32 = 0; // BOOLEAN

    // PAGED_CODE(); // no-op in mechanical translation

    FsRtlEnterFileSystem();

    // Main loop
    loop {
        IrpContext = CdRemoveClose(Vcb);
        if IrpContext.is_null() {
            break;
        }

        // Check node type
        if SafeNodeType(IrpContext) != /* CDFS_NTC_IRP_CONTEXT (grind had 0x0) */ 0x0308 {
            // IrpContext is PIRP_CONTEXT_LITE
            let lite = IrpContext as *mut IRP_CONTEXT_LITE;
            Fcb = (*lite).Fcb;
            UserReference = (*lite).UserReference;

            // Initialize stack IRP context with the lite values
            CdInitializeStackIrpContext(
                &raw mut stack_irp_context,
                lite,
            );

            // Free the lite context
            CdFreeIrpContextLite(IrpContext as *mut core::ffi::c_void);

            IrpContext = &raw mut stack_irp_context;
        } else {
            // Cleanup existing IrpContext
            Fcb = (*IrpContext).Irp as *mut FCB;
            (*IrpContext).Irp = core::ptr::null_mut();
            UserReference = (*IrpContext).ExceptionStatus as u32;
            (*IrpContext).ExceptionStatus = STATUS_SUCCESS;
        }

        // _Analysis_assume_ comment
        // _Analysis_assume_(Fcb != NULL && Fcb->Vcb != NULL);

        // Set FSP flag (grind had 0x2 = MORE_PROCESSING; real FSP_FLAGS = 0x74)
        SetFlag(&raw mut (*IrpContext).Flags, IRP_CONTEXT_FSP_FLAGS);

        // If Vcb supplied, clear top-level flags (grind had 0x4|0x8 = WAIT|FORCE_POST)
        if !Vcb.is_null() {
            ClearFlag(
                &raw mut (*IrpContext).Flags,
                IRP_CONTEXT_FLAG_TOP_LEVEL | IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS,
            );
        }

        CdSetThreadContext(IrpContext, &raw mut thread_context);

        // Compute potential teardown
        PotentialVcbTeardown = if Vcb.is_null()
            && (*(*Fcb).Vcb).VcbCondition != /* VcbMounted */ 0
            && (*(*Fcb).Vcb).VcbCondition != /* VcbMountInProgress */ 1
            && (*(*Fcb).Vcb).VcbCleanup == 0
        {
            1
        } else {
            0
        };

        // Check if we need to release/acquire Vcb
        if PotentialVcbTeardown != 0
            || VcbHoldCount > (*(core::ptr::addr_of!(CdData))).MinDelayedCloseCount
            || (*Fcb).Vcb != CurrentVcb
        {
            // Release current Vcb if held
            if !CurrentVcb.is_null() {
                CdReleaseVcb(IrpContext, CurrentVcb);
            }

            if PotentialVcbTeardown != 0 {
                CdAcquireCdData(IrpContext);

                // Re-check under global lock
                PotentialVcbTeardown = if Vcb.is_null()
                    && (*(*Fcb).Vcb).VcbCondition != /* VcbMounted */ 0
                    && (*(*Fcb).Vcb).VcbCondition != /* VcbMountInProgress */ 1
                    && (*(*Fcb).Vcb).VcbCleanup == 0
                {
                    1
                } else {
                    0
                };

                if PotentialVcbTeardown == 0 {
                    CdReleaseCdData(IrpContext);
                }
            }

            CurrentVcb = (*Fcb).Vcb;

            // _Analysis_assume_(CurrentVcb != NULL);

            CdAcquireVcbShared(IrpContext, CurrentVcb, 0);  // FALSE => 0
            VcbHoldCount = 0;
        } else {
            VcbHoldCount += 1;
        }

        // Perform close operation
        CdCommonClosePrivate(IrpContext, CurrentVcb, Fcb, UserReference, 0); // FALSE => 0

        // If potential teardown, release and dismount check
        if PotentialVcbTeardown != 0 {
            CdReleaseVcb(IrpContext, CurrentVcb);
            CdCheckForDismount(IrpContext, CurrentVcb, 0); // FALSE => 0
            CurrentVcb = core::ptr::null_mut();
            CdReleaseCdData(IrpContext);
            PotentialVcbTeardown = 0;
        }

        // Complete request to cleanup IrpContext
        CdCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);
    }

    // Release any remaining Vcb hold
    if !CurrentVcb.is_null() {
        CdReleaseVcb(IrpContext, CurrentVcb);
    }

    // #ifdef _MSC_VER
    // #pragma prefast(suppress:26165, "Esp:1153")
    // #endif

    FsRtlExitFileSystem();
}
#[no_mangle]
pub unsafe extern "C" fn CdCommonClose(
    mut IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let TypeOfOpen: TYPE_OF_OPEN;
    let Vcb: PVCB;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut UserReference: u32 = 0;
    let mut PotentialVcbTeardown: i32 = 0; // BOOLEAN

    // PAGED_CODE();
    // ASSERT_IRP_CONTEXT( IrpContext );
    // ASSERT_IRP( Irp );

    if (*IrpContext).Vcb.is_null() {
        CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    let iostack = IoGetCurrentIrpStackLocation(Irp);
    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        (*iostack).FileObject,
        &mut Fcb as *mut PFCB,
        &mut Ccb as *mut PCCB,
    );

    if TypeOfOpen == UnopenedFileObject {
        CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    Vcb = (*Fcb).Vcb;

    if !Ccb.is_null() {
        UserReference = 1;
        CdDeleteCcb(IrpContext, Ccb);
    }

    if ((*Vcb).VcbCondition == VcbMounted)
        && ((*Fcb).FcbReference == 1)
        && ((TypeOfOpen == UserFileOpen) || (TypeOfOpen == UserDirectoryOpen))
    {
        CdQueueClose(IrpContext, Fcb, UserReference, 1);
        IrpContext = ::core::ptr::null_mut();
    } else {
        if ((*Vcb).VcbCleanup == 0) && ((*Vcb).VcbCondition != VcbMounted) {
            CdAcquireCdData(IrpContext);
            if ((*Vcb).VcbCleanup == 0)
                && ((*Vcb).VcbCondition != VcbMounted)
                && ((*Vcb).VcbCondition != VcbMountInProgress)
                && FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS) != 0
            {
                PotentialVcbTeardown = 1;
            } else {
                // intentional empty branch
            }
            if PotentialVcbTeardown == 0 {
                CdReleaseCdData(IrpContext);
            }
        }
        if CdCommonClosePrivate(IrpContext, Vcb, Fcb, UserReference, 1) == 0 {
            CdQueueClose(IrpContext, Fcb, UserReference, 0);
            IrpContext = ::core::ptr::null_mut();
        } else if PotentialVcbTeardown != 0 {
            CdCheckForDismount(IrpContext, Vcb, 0);
        }
    }

    CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);

    if PotentialVcbTeardown != 0 {
        CdReleaseCdData(IrpContext);
    }

    STATUS_SUCCESS
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
//     Close.c
//
// Abstract:
//
//     This module implements the File Close routine for Cdfs called by the
//     Fsd/Fsp dispatch routines.
//
//     The close operation interacts with both the async and delayed close queues
//     in the CdData structure.  Since close may be called recursively we may
//     violate the locking order in acquiring the Vcb or Fcb.  In this case
//     we may move the request to the async close queue.  If this is the last
//     reference on the Fcb and there is a chance the user may reopen this
//     file again soon we would like to defer the close.  In this case we
//     may move the request to the async close queue.
//
//     Once we are past the decode file operation there is no need for the
//     file object.  If we are moving the request to either of the work
//     queues then we remember all of the information from the file object and
//     complete the request with STATUS_SUCCESS.  The Io system can then
//     reuse the file object and we can complete the request when convenient.
//
//     The async close queue consists of requests which we would like to
//     complete as soon as possible.  They are queued using the original
//     IrpContext where some of the fields have been overwritten with
//     information from the file object.  We will extract this information,
//     cleanup the IrpContext and then call the close worker routine.
//
//     The delayed close queue consists of requests which we would like to
//     defer the close for.  We keep size of this list within a range
//     determined by the size of the system.  We let it grow to some maximum
//     value and then shrink to some minimum value.  We allocate a small
//     structure which contains the key information from the file object
//     and use this information along with an IrpContext on the stack
//     to complete the request.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_CLOSE)
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdCommonClosePrivate (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFCB Fcb,
//      ULONG UserReference,
//      BOOLEAN FromFsd
//     );
//
// VOID
// CdQueueClose (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG UserReference,
//      BOOLEAN DelayedClose
//     );
//
// PIRP_CONTEXT
// CdRemoveClose (
//      PVCB Vcb
//     );
//
// //  Tell prefast this is a workitem routine
// IO_WORKITEM_ROUTINE CdCloseWorker;
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdCloseWorker (
//      PDEVICE_OBJECT DeviceObject,
//      PVOID Context
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdFspClose)
// #pragma alloc_text(PAGE, CdCommonClose)
// #pragma alloc_text(PAGE, CdCommonClosePrivate)
// #pragma alloc_text(PAGE, CdQueueClose)
// #pragma alloc_text(PAGE, CdRemoveClose)
// #pragma alloc_text(PAGE, CdCloseWorker)
// #endif
//
//
// VOID
// CdFspClose (
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to process the close queues in the CdData.  If the
//     Vcb is passed then we want to remove all of the closes for this Vcb.
//     Otherwise we will do as many of the delayed closes as we need to do.
//
// Arguments:
//
//     Vcb - If specified then we are looking for all of the closes for the
//         given Vcb.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PIRP_CONTEXT IrpContext;
//     IRP_CONTEXT StackIrpContext;
//
//     THREAD_CONTEXT ThreadContext = {0};
//
//     PFCB Fcb;
//     ULONG UserReference;
//
//     ULONG VcbHoldCount = 0;
//     PVCB CurrentVcb = NULL;
//
//     BOOLEAN PotentialVcbTeardown = FALSE;
//
//     PAGED_CODE();
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Continue processing until there are no more closes to process.
//     //
//
//     while ((IrpContext = CdRemoveClose( Vcb )) != NULL) {
//
//         //
//         //  If we don't have an IrpContext then use the one on the stack.
//         //  Initialize it for this request.
//         //
//
//         if (SafeNodeType( IrpContext ) != CDFS_NTC_IRP_CONTEXT ) {
//
//             //
//             //  Update the local values from the IrpContextLite.
//             //
//
//             Fcb = ((PIRP_CONTEXT_LITE) IrpContext)->Fcb;
//             UserReference = ((PIRP_CONTEXT_LITE) IrpContext)->UserReference;
//
//             //
//             //  Update the stack irp context with the values from the
//             //  IrpContextLite.
//             //
//
//             CdInitializeStackIrpContext( &StackIrpContext,
//                                          (PIRP_CONTEXT_LITE) IrpContext );
//
//             //
//             //  Free the IrpContextLite.
//             //
//
//             CdFreeIrpContextLite( *(PVOID*)&IrpContext ); /* ReactOS Change: GCC "error: invalid lvalue in unary '&'" */
//
//             //
//             //  Remember we have the IrpContext from the stack.
//             //
//
//             IrpContext = &StackIrpContext;
//
//         //
//         //  Otherwise cleanup the existing IrpContext.
//         //
//
//         } else {
//
//             //
//             //  Remember the Fcb and user reference count.
//             //
//
//             Fcb = (PFCB) IrpContext->Irp;
//             IrpContext->Irp = NULL;
//
//             UserReference = (ULONG) IrpContext->ExceptionStatus;
//             IrpContext->ExceptionStatus = STATUS_SUCCESS;
//         }
//
//         _Analysis_assume_(Fcb != NULL && Fcb->Vcb != NULL);
//
//         //
//         //  We have an IrpContext.  Now we need to set the top level thread
//         //  context.
//         //
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FSP_FLAGS );
//
//         //
//         //  If we were given a Vcb then there is a request on top of this.
//         //
//
//         if (ARGUMENT_PRESENT( Vcb )) {
//
//             ClearFlag( IrpContext->Flags,
//                        IRP_CONTEXT_FLAG_TOP_LEVEL | IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS );
//         }
//
//         CdSetThreadContext( IrpContext, &ThreadContext );
//
//         //
//         //  If we have hit the maximum number of requests to process without
//         //  releasing the Vcb then release the Vcb now.  If we are holding
//         //  a different Vcb to this one then release the previous Vcb.
//         //
//         //  In either case acquire the current Vcb.
//         //
//         //  We use the MinDelayedCloseCount from the CdData since it is
//         //  a convenient value based on the system size.  Only thing we are trying
//         //  to do here is prevent this routine starving other threads which
//         //  may need this Vcb exclusively.
//         //
//         //  Note that the check for potential teardown below is unsafe.  We'll
//         //  repeat later within the cddata lock.
//         //
//
//         PotentialVcbTeardown = !ARGUMENT_PRESENT( Vcb ) &&
//                                (Fcb->Vcb->VcbCondition != VcbMounted) &&
//                                (Fcb->Vcb->VcbCondition != VcbMountInProgress) &&
//                                (Fcb->Vcb->VcbCleanup == 0);
//
//         if (PotentialVcbTeardown ||
//             (VcbHoldCount > CdData.MinDelayedCloseCount) ||
//             (Fcb->Vcb != CurrentVcb)) {
//
//             if (CurrentVcb != NULL) {
//
//                 CdReleaseVcb( IrpContext, CurrentVcb );
//             }
//
//             if (PotentialVcbTeardown) {
//
//                 CdAcquireCdData( IrpContext );
//
//                 //
//                 //  Repeat the checks with global lock held.  The volume could have
//                 //  been remounted while we didn't hold the lock.
//                 //
//
//                 PotentialVcbTeardown = !ARGUMENT_PRESENT( Vcb ) &&
//                                        (Fcb->Vcb->VcbCondition != VcbMounted) &&
//                                        (Fcb->Vcb->VcbCondition != VcbMountInProgress) &&
//                                        (Fcb->Vcb->VcbCleanup == 0);
//
//                 if (!PotentialVcbTeardown)  {
//
//                     CdReleaseCdData( IrpContext);
//                 }
//             }
//
//             CurrentVcb = Fcb->Vcb;
//
//             _Analysis_assume_( CurrentVcb != NULL );
//
//             CdAcquireVcbShared( IrpContext, CurrentVcb, FALSE );
//
//             VcbHoldCount = 0;
//
//         } else {
//
//             VcbHoldCount += 1;
//         }
//
//         //
//         //  Call our worker routine to perform the close operation.
//         //
//
//         CdCommonClosePrivate( IrpContext, CurrentVcb, Fcb, UserReference, FALSE );
//
//         //
//         //  If the reference count on this Vcb is below our residual reference
//         //  then check if we should dismount the volume.
//         //
//
//         if (PotentialVcbTeardown) {
//
//             CdReleaseVcb( IrpContext, CurrentVcb );
//             CdCheckForDismount( IrpContext, CurrentVcb, FALSE );
//
//             CurrentVcb = NULL;
//
//             CdReleaseCdData( IrpContext );
//             PotentialVcbTeardown = FALSE;
//         }
//
//         //
//         //  Complete the current request to cleanup the IrpContext.
//         //
//
//         CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//     }
//
//     //
//     //  Release any Vcb we may still hold.
//     //
//
//     if (CurrentVcb != NULL) {
//
//         CdReleaseVcb( IrpContext, CurrentVcb );
//
//     }
//
// #ifdef _MSC_VER
// #pragma prefast(suppress:26165, "Esp:1153")
// #endif
//     FsRtlExitFileSystem();
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonClose (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the Fsd entry for the close operation.  We decode the file
//     object to find the CDFS structures and type of open.  We call our internal
//     worker routine to perform the actual work.  If the work wasn't completed
//     then we post to one of our worker queues.  The Ccb isn't needed after this
//     point so we delete the Ccb and return STATUS_SUCCESS to our caller in all
//     cases.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     STATUS_SUCCESS
//
// --*/
//
// {
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//     ULONG UserReference = 0;
//
//     BOOLEAN PotentialVcbTeardown = FALSE;
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  If we were called with our file system device object instead of a
//     //  volume device object, just complete this request with STATUS_SUCCESS.
//     //
//
//     if (IrpContext->Vcb == NULL) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Decode the file object to get the type of open and Fcb/Ccb.
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext,
//                                      IoGetCurrentIrpStackLocation( Irp )->FileObject,
//                                      &Fcb,
//                                      &Ccb );
//
//     //
//     //  No work to do for unopened file objects.
//     //
//
//     if (TypeOfOpen == UnopenedFileObject) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//         return STATUS_SUCCESS;
//     }
//
//     Vcb = Fcb->Vcb;
//
//     //
//     //  Clean up any CCB associated with this open.
//     //
//
//     if (Ccb != NULL) {
//
//         UserReference = 1;
//
//         //
//         //  We can always deallocate the Ccb if present.
//         //
//
//         CdDeleteCcb( IrpContext, Ccb );
//     }
//
//     //
//     //  If this is the last reference to a user file or directory on a
//     //  currently mounted volume, then post it to the delayed close queue.  Note
//     //  that the VcbCondition check is unsafe,  but it doesn't really matter -
//     //  we just might delay the volume teardown a little by posting this close.
//     //
//
//     if ((Vcb->VcbCondition == VcbMounted) &&
//         (Fcb->FcbReference == 1) &&
//         ((TypeOfOpen == UserFileOpen) ||
//          (TypeOfOpen == UserDirectoryOpen))) {
//
//         CdQueueClose( IrpContext, Fcb, UserReference, TRUE );
//         IrpContext = NULL;
//
//     //
//     //  Otherwise try to process this close.  Post to the async close queue
//     //  if we can't acquire all of the resources.
//     //
//
//     }
//     else {
//
//         //
//         //  If we may be dismounting this volume then acquire the CdData
//         //  resource.
//         //
//         //  Since we now must make volumes go away as soon as reasonable after
//         //  the last user handles closes, key off of the cleanup count.  It is
//         //  OK to do this more than neccesary.  Since this Fcb could be holding
//         //  a number of other Fcbs (and thus their references), a simple check
//         //  on reference count is not appropriate.
//         //
//         //  Do an unsafe check first to avoid taking the (global) cddata lock in the
//         //  common case.
//         //
//
//         if ((Vcb->VcbCleanup == 0) &&
//             (Vcb->VcbCondition != VcbMounted))  {
//
//             //
//             //  Possible dismount.  Acquire CdData to synchronise with the remount path
//             //  before looking at the vcb condition again.
//             //
//
//             CdAcquireCdData( IrpContext );
//
//             if ((Vcb->VcbCleanup == 0) &&
//                 (Vcb->VcbCondition != VcbMounted) &&
//                 (Vcb->VcbCondition != VcbMountInProgress) &&
//                 FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS ))  {
//
//                 PotentialVcbTeardown = TRUE;
//             }
//             else {
//
//                 //
//                 //  We can't dismount this volume now,  there are other references or
//                 //  it's just been remounted.
//                 //
//             }
//
//             //
//             //  Drop the global lock if we don't need it anymore.
//             //
//
//             if (!PotentialVcbTeardown) {
//
//                 CdReleaseCdData( IrpContext );
//             }
//         }
//
//         //
//         //  Call the worker routine to perform the actual work.  This routine
//         //  should never raise except for a fatal error.
//         //
//
//         if (!CdCommonClosePrivate( IrpContext, Vcb, Fcb, UserReference, TRUE )) {
//
//             //
//             //  If we didn't complete the request then post the request as needed.
//             //
//
//             CdQueueClose( IrpContext, Fcb, UserReference, FALSE );
//             IrpContext = NULL;
//
//         //
//         //  Check whether we should be dismounting the volume and then complete
//         //  the request.
//         //
//
//         }
//         else if (PotentialVcbTeardown) {
//
//             CdCheckForDismount( IrpContext, Vcb, FALSE );
//         }
//     }
//
//     //
//     //  Always complete this request with STATUS_SUCCESS.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     if (PotentialVcbTeardown) {
//
//         CdReleaseCdData( IrpContext );
//     }
//
//     //
//     //  Always return STATUS_SUCCESS for closes.
//     //
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdCommonClosePrivate (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFCB Fcb,
//      ULONG UserReference,
//      BOOLEAN FromFsd
//     )
//
// /*++
//
// Routine Description:
//
//     This is the worker routine for the close operation.  We can be called in
//     an Fsd thread or from a worker Fsp thread.  If called from the Fsd thread
//     then we acquire the resources without waiting.  Otherwise we know it is
//     safe to wait.
//
//     We check to see whether we should post this request to the delayed close
//     queue.  If we are to process the close here then we acquire the Vcb and
//     Fcb.  We will adjust the counts and call our teardown routine to see
//     if any of the structures should go away.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Fcb - Fcb for this request.
//
//     UserReference - Number of user references for this file object.  This is
//         zero for an internal stream.
//
//     FromFsd - This request was called from an Fsd thread.  Indicates whether
//         we should wait to acquire resources.
//
//     DelayedClose - Address to store whether we should try to put this on
//         the delayed close queue.  Ignored if this routine can process this
//         close.
//
// Return Value:
//
//     BOOLEAN - TRUE if this thread processed the close, FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN RemovedFcb;
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_FCB( Fcb );
//
//     //
//     //  Try to acquire the Vcb and Fcb.  If we can't acquire them then return
//     //  and let our caller know he should post the request to the async
//     //  queue.
//     //
//
//     if (CdAcquireVcbShared( IrpContext, Vcb, FromFsd )) {
//
//         if (!CdAcquireFcbExclusive( IrpContext, Fcb, FromFsd )) {
//
//             //
//             //  We couldn't get the Fcb.  Release the Vcb and let our caller
//             //  know to post this request.
//             //
//
//             CdReleaseVcb( IrpContext, Vcb );
//             return FALSE;
//         }
//
//     //
//     //  We didn't get the Vcb.  Let our caller know to post this request.
//     //
//
//     } else {
//
//         return FALSE;
//     }
//
//     //
//     //  Lock the Vcb and decrement the reference counts.
//     //
//
//     CdLockVcb( IrpContext, Vcb );
//     CdDecrementReferenceCounts( IrpContext, Fcb, 1, UserReference );
//     CdUnlockVcb( IrpContext, Vcb );
//
//     //
//     //  Call our teardown routine to see if this object can go away.
//     //  If we don't remove the Fcb then release it.
//     //
//
//     CdTeardownStructures( IrpContext, Fcb, &RemovedFcb );
//
//     if (!RemovedFcb) {
//
//         CdReleaseFcb( IrpContext, Fcb );
//     }
//     else {
//         _Analysis_assume_lock_not_held_(Fcb->FcbNonpaged->FcbResource);
//     }
//
//     //
//     //  Release the Vcb and return to our caller.  Let him know we completed
//     //  this request.
//     //
//
//     CdReleaseVcb( IrpContext, Vcb );
//
//     return TRUE;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdCloseWorker (
//      PDEVICE_OBJECT DeviceObject,
//      PVOID Context
//     )
// /*++
//
// Routine Description:
//
//     Worker routine to call CsFspClose.
//
// Arguments:
//
//     DeviceObject - Filesystem registration device object
//
//     Context - Callers context
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
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Context );
//
//     CdFspClose (NULL);
// }
//
//
// VOID
// CdQueueClose (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG UserReference,
//      BOOLEAN DelayedClose
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to queue a request to either the async or delayed
//     close queue.  For the delayed queue we need to allocate a smaller
//     structure to contain the information about the file object.  We do
//     that so we don't put the larger IrpContext structures into this long
//     lived queue.  If we can allocate this structure then we put this
//     on the async queue instead.
//
// Arguments:
//
//     Fcb - Fcb for this file object.
//
//     UserReference - Number of user references for this file object.  This is
//         zero for an internal stream.
//
//     DelayedClose - Indicates whether this should go on the async or delayed
//         close queue.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PIRP_CONTEXT_LITE IrpContextLite = NULL;
//     BOOLEAN StartWorker = FALSE;
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_FCB( Fcb );
//
//     //
//     //  Start with the delayed queue request.  We can move this to the async
//     //  queue if there is an allocation failure.
//     //
//
//     if (DelayedClose) {
//
//         //
//         //  Try to allocate non-paged pool for the IRP_CONTEXT_LITE.
//         //
//
//         IrpContextLite = CdCreateIrpContextLite( IrpContext );
//     }
//
//     //
//     //  We want to clear the top level context in this thread if
//     //  necessary.  Call our cleanup routine to do the work.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING );
//     CdCleanupIrpContext( IrpContext, TRUE );
//
//     //
//     //  Synchronize with the CdData lock.
//     //
//
//     CdLockCdData();
//
//     //
//     //  If we have an IrpContext then put the request on the delayed close queue.
//     //
//
//     if (IrpContextLite != NULL) {
//
//         //
//         //  Initialize the IrpContextLite.
//         //
//
//         IrpContextLite->NodeTypeCode = CDFS_NTC_IRP_CONTEXT_LITE;
//         IrpContextLite->NodeByteSize = sizeof( IRP_CONTEXT_LITE );
//         IrpContextLite->Fcb = Fcb;
//         IrpContextLite->UserReference = UserReference;
//         IrpContextLite->RealDevice = IrpContext->RealDevice;
//
//         //
//         //  Add this to the delayed close list and increment
//         //  the count.
//         //
//
//         InsertTailList( &CdData.DelayedCloseQueue,
//                         &IrpContextLite->DelayedCloseLinks );
//
//         CdData.DelayedCloseCount += 1;
//
//         //
//         //  If we are above our threshold then start the delayed
//         //  close operation.
//         //
//
//         if (CdData.DelayedCloseCount > CdData.MaxDelayedCloseCount) {
//
//             CdData.ReduceDelayedClose = TRUE;
//
//             if (!CdData.FspCloseActive) {
//
//                 CdData.FspCloseActive = TRUE;
//                 StartWorker = TRUE;
//             }
//         }
//
//         //
//         //  Unlock the CdData.
//         //
//
//         CdUnlockCdData();
//
//         //
//         //  Cleanup the IrpContext.
//         //
//
//         CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//
//     //
//     //  Otherwise drop into the async case below.
//     //
//
//     } else {
//
//         //
//         //  Store the information about the file object into the IrpContext.
//         //
//
//         IrpContext->Irp = (PIRP) Fcb;
//         IrpContext->ExceptionStatus = (NTSTATUS) UserReference;
//
//         //
//         //  Add this to the async close list and increment the count.
//         //
//
//         InsertTailList( &CdData.AsyncCloseQueue,
//                         &IrpContext->WorkQueueItem.List );
//
//         CdData.AsyncCloseCount += 1;
//
//         //
//         //  Remember to start the Fsp close thread if not currently started.
//         //
//
//         if (!CdData.FspCloseActive) {
//
//             CdData.FspCloseActive = TRUE;
//
//             StartWorker = TRUE;
//         }
//
//         //
//         //  Unlock the CdData.
//         //
//
//         CdUnlockCdData();
//     }
//
//     //
//     //  Start the FspClose thread if we need to.
//     //
//
//     if (StartWorker) {
//
//         IoQueueWorkItem( CdData.CloseItem, CdCloseWorker, CriticalWorkQueue, NULL );
//     }
//
//     //
//     //  Return to our caller.
//     //
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// PIRP_CONTEXT
// CdRemoveClose (
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
// Arguments:
//
//     This routine is called to scan the async and delayed close queues looking
//     for a suitable entry.  If the Vcb is specified then we scan both queues
//     looking for an entry with the same Vcb.  Otherwise we will look in the
//     async queue first for any close item.  If none found there then we look
//     in the delayed close queue provided that we have triggered the delayed
//     close operation.
//
// Return Value:
//
//     PIRP_CONTEXT - NULL if no work item found.  Otherwise it is the pointer to
//         either the IrpContext or IrpContextLite for this request.
//
// --*/
//
// {
//     PIRP_CONTEXT IrpContext = NULL;
//     PIRP_CONTEXT NextIrpContext;
//     PIRP_CONTEXT_LITE NextIrpContextLite;
//
//     PLIST_ENTRY Entry;
//
//     PAGED_CODE();
//
//     ASSERT_OPTIONAL_VCB( Vcb );
//
//     //
//     //  Lock the CdData to perform the scan.
//     //
//
//     CdLockCdData();
//
//     //
//     //  First check the list of async closes.
//     //
//
//     Entry = CdData.AsyncCloseQueue.Flink;
//
//     while (Entry != &CdData.AsyncCloseQueue) {
//
//         //
//         //  Extract the IrpContext.
//         //
//
//         NextIrpContext = CONTAINING_RECORD( Entry,
//                                             IRP_CONTEXT,
//                                             WorkQueueItem.List );
//
//         //
//         //  If no Vcb was specified or this Vcb is for our volume
//         //  then perform the close.
//         //
//
//         if (!ARGUMENT_PRESENT( Vcb ) || (NextIrpContext->Vcb == Vcb)) {
//
//             RemoveEntryList( Entry );
//             CdData.AsyncCloseCount -= 1;
//
//             IrpContext = NextIrpContext;
//             break;
//         }
//
//         //
//         //  Move to the next entry.
//         //
//
//         Entry = Entry->Flink;
//     }
//
//     //
//     //  If we didn't find anything look through the delayed close
//     //  queue.
//     //
//     //  We will only check the delayed close queue if we were given
//     //  a Vcb or the delayed close operation is active.
//     //
//
//     if ((IrpContext == NULL) &&
//         (ARGUMENT_PRESENT( Vcb ) ||
//          (CdData.ReduceDelayedClose &&
//           (CdData.DelayedCloseCount > CdData.MinDelayedCloseCount)))) {
//
//         Entry = CdData.DelayedCloseQueue.Flink;
//
//         while (Entry != &CdData.DelayedCloseQueue) {
//
//             //
//             //  Extract the IrpContext.
//             //
//
//             NextIrpContextLite = CONTAINING_RECORD( Entry,
//                                                     IRP_CONTEXT_LITE,
//                                                     DelayedCloseLinks );
//
//             //
//             //  If no Vcb was specified or this Vcb is for our volume
//             //  then perform the close.
//             //
//
//             if (!ARGUMENT_PRESENT( Vcb ) || (NextIrpContextLite->Fcb->Vcb == Vcb)) {
//
//                 RemoveEntryList( Entry );
//                 CdData.DelayedCloseCount -= 1;
//
//                 IrpContext = (PIRP_CONTEXT) NextIrpContextLite;
//                 break;
//             }
//
//             //
//             //  Move to the next entry.
//             //
//
//             Entry = Entry->Flink;
//         }
//     }
//
//     //
//     //  If the Vcb wasn't specified and we couldn't find an entry
//     //  then turn off the Fsp thread.
//     //
//
//     if (!ARGUMENT_PRESENT( Vcb ) && (IrpContext == NULL)) {
//
//         CdData.FspCloseActive = FALSE;
//         CdData.ReduceDelayedClose = FALSE;
//     }
//
//     //
//     //  Unlock the CdData.
//     //
//
//     CdUnlockCdData();
//
//     return IrpContext;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
