use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, IoAcquireVpbSpinLock, IoReleaseVpbSpinLock, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PVPB, VPB};
use core::ffi::c_void;

extern "C" {
    fn ASSERT_IRP_CONTEXT(arg: *mut IRP_CONTEXT);
    fn ASSERT_VCB(arg: *mut VCB);
    fn ASSERT_EXCLUSIVE_CDDATA();
    fn CdAcquireVcbExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, arg2: i32);
    fn CdFspClose(Vcb: *mut VCB);
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn CdDeleteVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn CdReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn _Analysis_assume_lock_not_held_(lock: *mut core::ffi::c_void);
}
extern "C" {
    fn CdMarkRealDevForVerify(Device: *mut DEVICE_OBJECT);
    fn ASSERT_EXCLUSIVE_VCB(Vcb: *mut VCB);
    fn NT_ASSERT(condition: i32);
    fn CdFreePool(Ptr: *mut *mut core::ffi::c_void);
    fn CdPurgeVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, flag: i32);
    fn ClearFlag(Flags: *mut u32, Flag: u32);
    fn FlagOn(Flags: u32, Flag: u32) -> u32;
    fn PAGED_CODE();
    fn CdRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: i32);
    fn CdRealDevNeedsVerify(DeviceObject: PDEVICE_OBJECT) -> u8;
    fn CdPerformDevIoCtrl( IrpContext: *mut IRP_CONTEXT, IoControlCode: u32, DeviceObject: PDEVICE_OBJECT, InputBuffer: *mut core::ffi::c_void, InputBufferLength: u32, b1: i32, b2: i32, Iosb: *mut _IO_STATUS_BLOCK, ) -> i32;
    fn CdIsRawDevice(IrpContext: *mut IRP_CONTEXT, Status: i32) -> u8;
    fn _SEH2_TRY();
    fn _SEH2_EXCEPT(filter: fn(/* ... */) -> i32);
    fn _SEH2_END();
    fn _SEH2_GetExceptionInformation() -> *mut core::ffi::c_void;
    fn _SEH2_GetExceptionCode() -> i32;
    fn CdExceptionFilter( _: *mut PIRP_CONTEXT, _: *mut core::ffi::c_void, ) -> i32;
    fn CdProcessException( IrpContext: *mut PIRP_CONTEXT, Irp: *mut PIRP, ExceptionCode: i32, ) -> NTSTATUS;
    fn CdOperationIsDasdOpen(IrpContext: *mut PIRP_CONTEXT) -> i32;
    fn CdAcquireVcbShared( IrpContext: *mut PIRP_CONTEXT, Vcb: *mut VCB, Force: i32, );
    fn CdAcquireCdData(IrpContext: *mut PIRP_CONTEXT);
    fn CdReleaseCdData(IrpContext: *mut PIRP_CONTEXT);
    fn CdCompleteRequest( IrpContext: *mut PIRP_CONTEXT, Irp: *mut PIRP, Status: NTSTATUS, );
    fn CdFsdPostRequest( IrpContext: *mut PIRP_CONTEXT, Irp: *mut PIRP, ) -> NTSTATUS;
    fn CdNormalizeAndRaiseStatus( IrpContext: *mut PIRP_CONTEXT, Status: NTSTATUS, );
    fn ASSERT_IRP(_: *mut PIRP);
    fn IoVerifyVolume( DeviceToVerify: PDEVICE_OBJECT, AllowRawMount: i32, ) -> NTSTATUS;
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> i32;
    fn IoSetHardErrorOrVerifyDevice( Irp: *mut PIRP, DeviceToVerify: PDEVICE_OBJECT, );
    fn IoGetCurrentIrpStackLocation(Irp: *mut PIRP) -> *mut IO_STACK_LOCATION;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_VERFYSUP;

const VCB_STATE_VPB_NOT_ON_DEVICE: u32 = 0x0002; // TODO: verify the actual value

unsafe extern "system" {
    fn PAGED_CODE();
    fn CdRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: i32);
    fn CdRealDevNeedsVerify(Device: *mut DEVICE_OBJECT) -> i32;
}
#[no_mangle]
pub unsafe extern "C" fn CdPerformVerify(
    IrpContext: *mut PIRP_CONTEXT,
    Irp: *mut PIRP,
    DeviceToVerify: PDEVICE_OBJECT,
) -> NTSTATUS {
    // Declare externs for SEH macros and other unknown functions/constants

    // Constants likely defined by the codebase
    const VcbMounted: u32 = 0;      // placeholder
    const VcbNotMounted: u32 = 1;   // placeholder
    const VcbInvalid: u32 = 2;      // placeholder
    const VcbDismountInProgress: u32 = 3; // placeholder
    const CDFS_RESIDUAL_REFERENCE: u32 = 0; // placeholder
    const IO_REMOUNT: u32 = 0;      // placeholder

    let mut Vcb: *mut VCB;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IrpSp: *mut IO_STACK_LOCATION;

    ASSERT_IRP_CONTEXT(IrpContext);
    ASSERT_IRP(Irp);

    // Early return if this is a mount/verify FSCTL
    if ((*IrpContext).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL)
        && (((*IrpContext).MinorFunction == 0) /*IRP_MN_MOUNT_VOLUME*/ // placeholder
            || ((*IrpContext).MinorFunction == 1)) /* IRP_MN_VERIFY_VOLUME */)
    {
        return CdFsdPostRequest(IrpContext, Irp);
    }

    // Extract Vcb from DeviceObject using CONTAINING_RECORD
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    // CONTAINING_RECORD: cast to VOLUME_DEVICE_OBJECT* then take &Vcb
    Vcb = core::ptr::addr_of_mut!(
        (*((*IrpSp).DeviceObject as *mut u8)
            .sub(core::mem::offset_of!(VOLUME_DEVICE_OBJECT, DeviceObject))
            as *mut VOLUME_DEVICE_OBJECT)
    )
    .Vcb
        as *mut VCB;

    // SEH_TRY block: we approximate with a guarded region via macros/functions
    _SEH2_TRY();

    // -------- try body --------
    // Send verify FSCTL to the underlying device
    Status = IoVerifyVolume(DeviceToVerify, CdOperationIsDasdOpen(IrpContext));

    // Acquire shared Vcb for stable VcbCondition
    CdAcquireVcbShared(IrpContext, Vcb, 0); // FALSE -> 0

    // Check verify result and current VcbCondition
    if (Status == STATUS_WRONG_VOLUME) && ((*Vcb).VcbCondition == VcbMounted) {
        Status = STATUS_SUCCESS;
    } else if (Status == STATUS_SUCCESS) && ((*Vcb).VcbCondition != VcbMounted) {
        // Volume is not mounted (some other volume is on device)
        Status = STATUS_WRONG_VOLUME;
    }

    // If Vcb suggests dismount, call CdCheckForDismount; otherwise release Vcb
    if (((*Vcb).VcbCondition == VcbNotMounted)
        || ((*Vcb).VcbCondition == VcbInvalid)
        || ((*Vcb).VcbCondition == VcbDismountInProgress))
        && ((*Vcb).VcbReference <= CDFS_RESIDUAL_REFERENCE)
    {
        CdReleaseVcb(IrpContext, Vcb);
        CdAcquireCdData(IrpContext);
        CdCheckForDismount(IrpContext, Vcb, 0); // FALSE
        CdReleaseCdData(IrpContext);
    } else {
        CdReleaseVcb(IrpContext, Vcb);
    }

    // If create succeeded and verify ok, complete with REPARSE
    if ((*IrpContext).MajorFunction == IRP_MJ_CREATE)
        && (*(*IrpSp).FileObject).RelatedFileObject.is_null()
        && (Status == STATUS_SUCCESS || Status == STATUS_WRONG_VOLUME)
    {
        (*(*Irp).IoStatus).Information = IO_REMOUNT;
        CdCompleteRequest(IrpContext, Irp, STATUS_REPARSE);
        Status = STATUS_REPARSE;
        Irp = core::ptr::null_mut();
        IrpContext = core::ptr::null_mut();
    }
    // Else if there's an error and Irp still exists, raise/hard error
    else if !Irp.is_null() && !NT_SUCCESS(Status) {
        if IoIsErrorUserInduced(Status) != 0 {
            IoSetHardErrorOrVerifyDevice(Irp, DeviceToVerify);
        }
        CdNormalizeAndRaiseStatus(IrpContext, Status);
    }

    // If we still have an IrpContext, post request
    if !IrpContext.is_null() {
        Status = CdFsdPostRequest(IrpContext, Irp);
    }

    // -------- end try body --------

    // SEH_EXCEPT block: catch exception, process it, and update Status
    _SEH2_EXCEPT(CdExceptionFilter(
        IrpContext,
        _SEH2_GetExceptionInformation(),
    ));
    {
        // The filter returned EXCEPTION_EXECUTE_HANDLER (non-zero), so execute this block
        Status = CdProcessException(IrpContext, Irp, _SEH2_GetExceptionCode());
    }
    _SEH2_END();

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdCheckForDismount(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Force: i32,
) -> i32 {
    unsafe {
        let mut UnlockVcb: i32 = 1;
        let mut VcbPresent: i32 = 1;
        let mut SavedIrql: KIRQL = 0;

        ASSERT_IRP_CONTEXT(IrpContext);
        ASSERT_VCB(Vcb);
        ASSERT_EXCLUSIVE_CDDATA();

        CdAcquireVcbExclusive(IrpContext, Vcb, 0); // FALSE

        CdFspClose(Vcb);

        CdLockVcb(IrpContext, Vcb);

        if (*Vcb).VcbCondition != 0 /* VcbDismountInProgress */ {
            if (*Vcb).VcbUserReference <= 0 /* CDFS_RESIDUAL_USER_REFERENCE */ || Force != 0 {
                CdUnlockVcb(IrpContext, Vcb);
                UnlockVcb = 0;
                VcbPresent = CdDismountVcb(IrpContext, Vcb);
            }
        } else if (*Vcb).VcbReference == 0 {
            IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(SavedIrql));

            if (*(*Vcb).Vpb).ReferenceCount == 1 {
                IoReleaseVpbSpinLock(SavedIrql);
                CdUnlockVcb(IrpContext, Vcb);
                UnlockVcb = 0;
                CdDeleteVcb(IrpContext, Vcb);
                VcbPresent = 0;
            } else {
                IoReleaseVpbSpinLock(SavedIrql);
            }
        }

        if UnlockVcb != 0 {
            CdUnlockVcb(IrpContext, Vcb);
        }

        if VcbPresent != 0 {
            CdReleaseVcb(IrpContext, Vcb);
        } else {
            _Analysis_assume_lock_not_held_((*Vcb).VcbResource as *mut core::ffi::c_void);
        }

        VcbPresent
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdMarkDevForVerifyIfVcbMounted(Vcb: *mut VCB) -> i32 {
    let mut marked: i32 = 0;
    let mut saved_irql: KIRQL;

    IoAcquireVpbSpinLock(&raw mut saved_irql);

    if (*(*(*Vcb).Vpb).RealDevice).Vpb == (*Vcb).Vpb {
        CdMarkRealDevForVerify((*(*Vcb).Vpb).RealDevice);
        marked = 1;
    } else {
        (*Vcb).VcbState |= VCB_STATE_VPB_NOT_ON_DEVICE;
    }

    IoReleaseVpbSpinLock(saved_irql);

    return marked;
}
#[no_mangle]
pub unsafe extern "C" fn CdVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    unsafe {
        PAGED_CODE();

        let mut Status: i32 = STATUS_SUCCESS;
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut MediaChangeCount: u32 = 0;
        let mut ForceVerify: u8 = 0;
        let mut DevMarkedForVerify: u8 = CdRealDevNeedsVerify((*(*Vcb).Vpb).RealDevice);

        // Fail immediately if the volume is in the progress of being dismounted
        // or has been marked invalid.
        if (*Vcb).VcbCondition == VcbInvalid
            || ((*Vcb).VcbCondition == VcbDismountInProgress
                && (*IrpContext).MajorFunction != IRP_MJ_CREATE)
        {
            if FlagOn((*Vcb).VcbState, VCB_STATE_DISMOUNTED) != 0 {
                CdRaiseStatus(IrpContext, STATUS_VOLUME_DISMOUNTED);
            } else {
                CdRaiseStatus(IrpContext, STATUS_FILE_INVALID);
            }
        }

        // If the media is removable and the verify volume flag in the
        // device object is not set then we want to ping the device
        // to see if it needs to be verified.
        if FlagOn((*Vcb).VcbState, VCB_STATE_REMOVABLE_MEDIA) != 0 && DevMarkedForVerify == 0 {
            if (*Vcb).VcbCondition != VcbMountInProgress {
                Status = CdPerformDevIoCtrl(
                    IrpContext,
                    IOCTL_CDROM_CHECK_VERIFY,
                    (*Vcb).TargetDeviceObject,
                    core::ptr::addr_of_mut!(MediaChangeCount) as *mut core::ffi::c_void,
                    core::mem::size_of::<u32>() as u32,
                    0,
                    0,
                    core::ptr::addr_of_mut!(Iosb),
                );

                if Iosb.Information as usize != core::mem::size_of::<u32>() {
                    // Be safe about the count in case the driver didn't fill it in
                    MediaChangeCount = 0;
                }

                // There are four cases when we want to do a verify. These are the first three.
                // 1. We are mounted, and the device has become empty
                // 2. The device has returned verify required (=> DO_VERIFY_VOL flag is
                //    set, but could be due to hardware condition)
                // 3. Media change count doesn't match the one in the Vcb
                if ((*Vcb).VcbCondition == VcbMounted && CdIsRawDevice(IrpContext, Status) != 0)
                    || Status == STATUS_VERIFY_REQUIRED
                    || (NT_SUCCESS(Status) && (*Vcb).MediaChangeCount != MediaChangeCount)
                {
                    // If we are currently the volume on the device then it is our
                    // responsibility to set the verify flag. If we're not on the device,
                    // then we shouldn't touch the flag.
                    if FlagOn((*Vcb).VcbState, VCB_STATE_VPB_NOT_ON_DEVICE) == 0
                        && DevMarkedForVerify == 0
                    {
                        DevMarkedForVerify = CdMarkDevForVerifyIfVcbMounted(Vcb);
                    }
                    ForceVerify = 1;
                    // NOTE that we no longer update the media change count here. We
                    // do so only when we've actually completed a verify at a particular
                    // change count value.
                }
            }

            // This is the 4th verify case.
            // We ALWAYS force CREATE requests on unmounted volumes through the
            // verify path. These requests could have been in limbo between
            // IoCheckMountedVpb and us when a verify/mount took place and caused
            // a completely different fs/volume to be mounted. In this case the
            // checks above may not have caught the condition, since we may already
            // have verified (wrong volume) and decided that we have nothing to do.
            // We want the requests to be re routed to the currently mounted volume,
            // since they were directed at the 'drive', not our volume.
            if NT_SUCCESS(Status) && ForceVerify == 0 && DevMarkedForVerify == 0
                && (*IrpContext).MajorFunction == IRP_MJ_CREATE
            {
                let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation((*IrpContext).Irp);
                let related_is_null = (*(*IrpSp).FileObject).RelatedFileObject.is_null();
                let condition = (*Vcb).VcbCondition == VcbDismountInProgress
                    || (*Vcb).VcbCondition == VcbNotMounted;
                ForceVerify = if related_is_null && condition { 1 } else { 0 };

                // Note that we don't touch the device verify flag here. If it required
                // it would have been caught and set by the first set of checks.
            }
        }

        // Raise the verify / error if necessary.
        if ForceVerify != 0 || DevMarkedForVerify != 0 || !NT_SUCCESS(Status) {
            IoSetHardErrorOrVerifyDevice((*IrpContext).Irp, (*(*Vcb).Vpb).RealDevice);
            let status_to_raise = if ForceVerify != 0 || DevMarkedForVerify != 0 {
                STATUS_VERIFY_REQUIRED
            } else {
                Status
            };
            CdRaiseStatus(IrpContext, status_to_raise);
        }

        // Based on the condition of the Vcb we'll either return to our
        // caller or raise an error condition
        match (*Vcb).VcbCondition {
            VcbNotMounted => {
                IoSetHardErrorOrVerifyDevice((*IrpContext).Irp, (*(*Vcb).Vpb).RealDevice);
                CdRaiseStatus(IrpContext, STATUS_WRONG_VOLUME);
            }
            VcbInvalid | VcbDismountInProgress => {
                if FlagOn((*Vcb).VcbState, VCB_STATE_DISMOUNTED) != 0 {
                    CdRaiseStatus(IrpContext, STATUS_VOLUME_DISMOUNTED);
                } else {
                    CdRaiseStatus(IrpContext, STATUS_FILE_INVALID);
                }
            }
            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdVerifyFcbOperation(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) -> i32 {
    let Vcb: *mut VCB = (*Fcb).Vcb;
    let RealDevice: *mut DEVICE_OBJECT = (*(*Vcb).Vpb).RealDevice;
    let Irp: *mut IRP;

    PAGED_CODE();

    // Check that the fileobject has not been cleaned up.
    if !IrpContext.is_null() {
        let FileObject: *mut FILE_OBJECT;

        Irp = (*IrpContext).Irp;
        FileObject = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;

        if !FileObject.is_null() && ((*FileObject).Flags & FO_CLEANUP_COMPLETE) != 0 {
            let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

            // Following FAT, we allow certain operations even on cleaned up file objects.
            // Everything else, we fail.
            if ((*Irp).Flags & IRP_PAGING_IO) != 0
                || (*IrpSp).MajorFunction == IRP_MJ_CLOSE
                || (*IrpSp).MajorFunction == IRP_MJ_QUERY_INFORMATION
                || ((*IrpSp).MajorFunction == IRP_MJ_READ
                    && ((*IrpSp).MinorFunction & IRP_MN_COMPLETE) != 0)
            {
                // NOTHING
                {}
            } else {
                CdRaiseStatus(IrpContext, STATUS_FILE_CLOSED);
            }
        }
    }

    // Fail immediately if the volume is in the progress of being dismounted
    // or has been marked invalid.
    if (*Vcb).VcbCondition == VcbInvalid || (*Vcb).VcbCondition == VcbDismountInProgress {
        if !IrpContext.is_null() {
            if ((*Vcb).VcbState & VCB_STATE_DISMOUNTED) != 0 {
                CdRaiseStatus(IrpContext, STATUS_VOLUME_DISMOUNTED);
            } else {
                CdRaiseStatus(IrpContext, STATUS_FILE_INVALID);
            }
        }
        return 0;
    }

    // Always fail if the volume needs to be verified.
    if CdRealDevNeedsVerify(RealDevice) != 0 {
        if !IrpContext.is_null() {
            IoSetHardErrorOrVerifyDevice((*IrpContext).Irp, RealDevice);
            CdRaiseStatus(IrpContext, STATUS_VERIFY_REQUIRED);
        }
        return 0;

    // All operations are allowed on mounted.
    } else if (*Vcb).VcbCondition == VcbMounted || (*Vcb).VcbCondition == VcbMountInProgress {
        return 1;

    // Fail all requests for fast Io on other Vcb conditions.
    } else if IrpContext.is_null() {
        return 0;

    // The remaining case is VcbNotMounted.
    // Mark the device to be verified and raise WRONG_VOLUME.
    } else if (*Vcb).VcbCondition == VcbNotMounted {
        IoSetHardErrorOrVerifyDevice((*IrpContext).Irp, RealDevice);
        CdRaiseStatus(IrpContext, STATUS_WRONG_VOLUME);
        // return FALSE; // unreachable code
    }

    return 1;
}
#[no_mangle] #[no_mangle]
#[no_mangle] pub unsafe extern "C" fn CdDismountVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) -> u8 {
    let mut OldVpb: *mut VPB = core::ptr::null_mut();
    let mut VcbPresent: u8 = 1;
    let mut SavedIrql: KIRQL = 0;
    let mut FinalReference: u8 = 0;

    ASSERT_EXCLUSIVE_CDDATA();
    ASSERT_EXCLUSIVE_VCB(Vcb);

    CdLockVcb(IrpContext, Vcb);

    NT_ASSERT( ((*Vcb).VcbCondition != VcbDismountInProgress) as i32 );

    (*Vcb).VcbCondition = VcbDismountInProgress;

    if !(*Vcb).XASector.is_null() {
        CdFreePool(core::ptr::addr_of_mut!((*Vcb).XASector));
        (*Vcb).XASector = core::ptr::null_mut();
        (*Vcb).XADiskOffset = 0;
    }

    if !(*Vcb).RootIndexFcb.is_null() {
        (*(*Vcb).RootIndexFcb).FcbReference -= 1;
        (*(*Vcb).RootIndexFcb).FcbUserReference -= 1;
    }

    if !(*Vcb).PathTableFcb.is_null() {
        (*(*Vcb).PathTableFcb).FcbReference -= 1;
        (*(*Vcb).PathTableFcb).FcbUserReference -= 1;
    }

    if !(*Vcb).VolumeDasdFcb.is_null() {
        (*(*Vcb).VolumeDasdFcb).FcbReference -= 1;
        (*(*Vcb).VolumeDasdFcb).FcbUserReference -= 1;
    }

    CdUnlockVcb(IrpContext, Vcb);

    CdPurgeVolume(IrpContext, Vcb, 1);

    CdFspClose(Vcb);

    OldVpb = (*Vcb).Vpb;

    CdLockVcb(IrpContext, Vcb);
    (*Vcb).VcbReference -= 1;

    IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(SavedIrql));

    FinalReference = ((*Vcb).VcbReference == 0 && (*OldVpb).ReferenceCount == 1) as u8;

    if (*(*OldVpb).RealDevice).Vpb == OldVpb {
        if FinalReference == 0 {
            NT_ASSERT( (!(*Vcb).SwapVpb.is_null()) as i32 );

            (*(*Vcb).SwapVpb).Type = IO_TYPE_VPB;
            (*(*Vcb).SwapVpb).Size = core::mem::size_of::<VPB>();
            (*(*Vcb).SwapVpb).RealDevice = (*OldVpb).RealDevice;
            (*(*(*Vcb).SwapVpb).RealDevice).Vpb = (*Vcb).SwapVpb;
            (*(*Vcb).SwapVpb).Flags = FlagOn( (*OldVpb).Flags, VPB_REMOVE_PENDING );

            IoReleaseVpbSpinLock(SavedIrql);

            (*Vcb).SwapVpb = core::ptr::null_mut();

            CdUnlockVcb(IrpContext, Vcb);
        } else {
            (*OldVpb).ReferenceCount -= 1;
            (*OldVpb).DeviceObject = core::ptr::null_mut();
            ClearFlag(core::ptr::addr_of_mut!((*OldVpb).Flags), VPB_MOUNTED);
            ClearFlag(core::ptr::addr_of_mut!((*OldVpb).Flags), VPB_LOCKED);

            (*Vcb).Vpb = core::ptr::null_mut();

            IoReleaseVpbSpinLock(SavedIrql);
            CdUnlockVcb(IrpContext, Vcb);
            CdDeleteVcb(IrpContext, Vcb);
            VcbPresent = 0;
        }
    } else if FinalReference != 0 {
        (*OldVpb).ReferenceCount -= 1;

        IoReleaseVpbSpinLock(SavedIrql);
        CdUnlockVcb(IrpContext, Vcb);
        CdDeleteVcb(IrpContext, Vcb);
        VcbPresent = 0;
    } else {
        IoReleaseVpbSpinLock(SavedIrql);
        CdUnlockVcb(IrpContext, Vcb);
    }

    return VcbPresent;
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
//     VerfySup.c
//
// Abstract:
//
//     This module implements the Cdfs Verification routines.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_VERFYSUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdVerifyFcbOperation)
// #pragma alloc_text(PAGE, CdVerifyVcb)
// #endif
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdPerformVerify (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PDEVICE_OBJECT DeviceToVerify
//     )
//
// /*++
//
// Routine Description:
//
//     This routines performs an IoVerifyVolume operation and takes the
//     appropriate action.  If the verify is successful then we send the originating
//     Irp off to an Ex Worker Thread.  This routine is called from the exception handler.
//
//     No file system resources are held when this routine is called.
//
// Arguments:
//
//     Irp - The irp to send off after all is well and done.
//
//     Device - The real device needing verification.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PVCB Vcb;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  Check if this Irp has a status of Verify required and if it does
//     //  then call the I/O system to do a verify.
//     //
//     //  Skip the IoVerifyVolume if this is a mount or verify request
//     //  itself.  Trying a recursive mount will cause a deadlock with
//     //  the DeviceObject->DeviceLock.
//     //
//
//     if ((IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) &&
//         ((IrpContext->MinorFunction == IRP_MN_MOUNT_VOLUME) ||
//          (IrpContext->MinorFunction == IRP_MN_VERIFY_VOLUME))) {
//
//         return CdFsdPostRequest( IrpContext, Irp );
//     }
//
//     //
//     //  Extract a pointer to the Vcb from the VolumeDeviceObject.
//     //  Note that since we have specifically excluded mount,
//     //  requests, we know that IrpSp->DeviceObject is indeed a
//     //  volume device object.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     Vcb = &CONTAINING_RECORD( IrpSp->DeviceObject,
//                               VOLUME_DEVICE_OBJECT,
//                               DeviceObject )->Vcb;
//     _SEH2_TRY {
//
//         //
//         //  Send down the verify FSCTL.  Note that this is sent to the
//         //  currently mounted volume,  which may not be this one.
//         //
//         //  We will allow Raw to mount this volume if we were doing a
//         //  an absolute DASD open.
//         //
//
//         Status = IoVerifyVolume( DeviceToVerify, CdOperationIsDasdOpen( IrpContext));
//
//         //
//         //  Acquire the Vcb so we're working with a stable VcbCondition.
//         //
//
//         CdAcquireVcbShared( IrpContext, Vcb, FALSE);
//
//         //
//         //  If the verify operation completed it will return
//         //  either STATUS_SUCCESS or STATUS_WRONG_VOLUME, exactly.
//         //
//         //  If CdVerifyVolume encountered an error during
//         //  processing, it will return that error.  If we got
//         //  STATUS_WRONG_VOLUME from the verify, and our volume
//         //  is now mounted, commute the status to STATUS_SUCCESS.
//         //
//
//         if ((Status == STATUS_WRONG_VOLUME) &&
//             (Vcb->VcbCondition == VcbMounted)) {
//
//             Status = STATUS_SUCCESS;
//         }
//         else if ((STATUS_SUCCESS == Status) && (Vcb->VcbCondition != VcbMounted))  {
//
//             //
//             //  If the verify succeeded,  but our volume is not mounted,
//             //  then some other volume is on the device.
//             //
//
//             Status = STATUS_WRONG_VOLUME;
//         }
//
//         //
//         //  Do a quick unprotected check here.  The routine will do
//         //  a safe check.  After here we can release the resource.
//         //  Note that if the volume really went away, we will be taking
//         //  the Reparse path.
//         //
//
//         //
//         //  If the device might need to go away then call our dismount routine.
//         //
//
//         if (((Vcb->VcbCondition == VcbNotMounted) ||
//              (Vcb->VcbCondition == VcbInvalid) ||
//              (Vcb->VcbCondition == VcbDismountInProgress)) &&
//             (Vcb->VcbReference <= CDFS_RESIDUAL_REFERENCE)) {
//
//             CdReleaseVcb( IrpContext, Vcb);
//
//             CdAcquireCdData( IrpContext );
//             CdCheckForDismount( IrpContext, Vcb, FALSE );
//             CdReleaseCdData( IrpContext );
//         }
//         else {
//
//             CdReleaseVcb( IrpContext, Vcb);
//         }
//
//         //
//         //  If this is a create and the verify succeeded then complete the
//         //  request with a REPARSE status.
//         //
//
//         if ((IrpContext->MajorFunction == IRP_MJ_CREATE) &&
//             (IrpSp->FileObject->RelatedFileObject == NULL) &&
//             ((Status == STATUS_SUCCESS) || (Status == STATUS_WRONG_VOLUME))) {
//
//             Irp->IoStatus.Information = IO_REMOUNT;
//
//             CdCompleteRequest( IrpContext, Irp, STATUS_REPARSE );
//             Status = STATUS_REPARSE;
//             Irp = NULL;
//             IrpContext = NULL;
//
//         //
//         //  If there is still an error to process then call the Io system
//         //  for a popup.
//         //
//
//         } else if ((Irp != NULL) && !NT_SUCCESS( Status )) {
//
//             //
//             //  Fill in the device object if required.
//             //
//
//             if (IoIsErrorUserInduced( Status ) ) {
//
//                 IoSetHardErrorOrVerifyDevice( Irp, DeviceToVerify );
//             }
//
//             CdNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//
//         //
//         //  If there is still an Irp, send it off to an Ex Worker thread.
//         //
//
//         if (IrpContext != NULL) {
//
//             Status = CdFsdPostRequest( IrpContext, Irp );
//         }
//
//     } _SEH2_EXCEPT(CdExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the verify or raised
//         //  an error ourselves.  So we'll abort the I/O request with
//         //  the error status that we get back from the execption code.
//         //
//
//         Status = CdProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     return Status;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdCheckForDismount (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      BOOLEAN Force
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to check if a volume is ready for dismount.  This
//     occurs when only file system references are left on the volume.
//
//     If the dismount is not currently underway and the user reference count
//     has gone to zero then we can begin the dismount.
//
//     If the dismount is in progress and there are no references left on the
//     volume (we check the Vpb for outstanding references as well to catch
//     any create calls dispatched to the file system) then we can delete
//     the Vcb.
//
// Arguments:
//
//     Vcb - Vcb for the volume to try to dismount.
//
//     Force - Whether we will force this volume to be dismounted.
//
// Return Value:
//
//     BOOLEAN - True if the Vcb was not gone by the time this function finished,
//         False if it was deleted.
//
//     This is only a trustworthy indication to the caller if it had the vcb
//     exclusive itself.
//
// --*/
//
// {
//     BOOLEAN UnlockVcb = TRUE;
//     BOOLEAN VcbPresent = TRUE;
//     KIRQL SavedIrql;
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_VCB( Vcb );
//
//     ASSERT_EXCLUSIVE_CDDATA;
//
//     //
//     //  Acquire and lock this Vcb to check the dismount state.
//     //
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  Lets get rid of any pending closes for this volume.
//     //
//
//     CdFspClose( Vcb );
//
//     CdLockVcb( IrpContext, Vcb );
//
//     //
//     //  If the dismount is not already underway then check if the
//     //  user reference count has gone to zero or we are being forced
//     //  to disconnect.  If so start the teardown on the Vcb.
//     //
//
//     if (Vcb->VcbCondition != VcbDismountInProgress) {
//
//         if (Vcb->VcbUserReference <= CDFS_RESIDUAL_USER_REFERENCE || Force) {
//
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//             VcbPresent = CdDismountVcb( IrpContext, Vcb );
//         }
//
//     //
//     //  If the teardown is underway and there are absolutely no references
//     //  remaining then delete the Vcb.  References here include the
//     //  references in the Vcb and Vpb.
//     //
//
//     } else if (Vcb->VcbReference == 0) {
//
//         IoAcquireVpbSpinLock( &SavedIrql );
//
//         //
//         //  If there are no file objects and no reference counts in the
//         //  Vpb we can delete the Vcb.  Don't forget that we have the
//         //  last reference in the Vpb.
//         //
//
//         if (Vcb->Vpb->ReferenceCount == 1) {
//
//             IoReleaseVpbSpinLock( SavedIrql );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//             CdDeleteVcb( IrpContext, Vcb );
//             VcbPresent = FALSE;
//
//         } else {
//
//             IoReleaseVpbSpinLock( SavedIrql );
//         }
//     }
//
//     //
//     //  Unlock the Vcb if still held.
//     //
//
//     if (UnlockVcb) {
//
//         CdUnlockVcb( IrpContext, Vcb );
//     }
//
//     //
//     //  Release any resources still acquired.
//     //
//
//     if (VcbPresent) {
//
//         CdReleaseVcb( IrpContext, Vcb );
//     }
//     else {
//         _Analysis_assume_lock_not_held_(Vcb->VcbResource);
//     }
//
//     return VcbPresent;
// }
//
//
// BOOLEAN
// CdMarkDevForVerifyIfVcbMounted (
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks to see if the specified Vcb is currently mounted on
//     the device or not.  If it is,  it sets the verify flag on the device, if
//     not then the state is noted in the Vcb.
//
// Arguments:
//
//     Vcb - This is the volume to check.
//
// Return Value:
//
//     TRUE if the device has been marked for verify here,  FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN Marked = FALSE;
//     KIRQL SavedIrql;
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//     if (Vcb->Vpb->RealDevice->Vpb == Vcb->Vpb)  {
//
//         CdMarkRealDevForVerify( Vcb->Vpb->RealDevice);
//         Marked = TRUE;
//     }
//     else {
//
//         //
//         //  Flag this to avoid the VPB spinlock in future passes.
//         //
//
//         SetFlag( Vcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE);
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     return Marked;
// }
//
//
// VOID
// CdVerifyVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks that the current Vcb is valid and currently mounted
//     on the device.  It will raise on an error condition.
//
//     We check whether the volume needs verification and the current state
//     of the Vcb.
//
// Arguments:
//
//     Vcb - This is the volume to verify.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     IO_STATUS_BLOCK Iosb;
//     ULONG MediaChangeCount = 0;
//     BOOLEAN ForceVerify = FALSE;
//     BOOLEAN DevMarkedForVerify;
//
//     PAGED_CODE();
//
//     //
//     //  Fail immediately if the volume is in the progress of being dismounted
//     //  or has been marked invalid.
//     //
//
//     if ((Vcb->VcbCondition == VcbInvalid) ||
//         ((Vcb->VcbCondition == VcbDismountInProgress) &&
//          (IrpContext->MajorFunction != IRP_MJ_CREATE))) {
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_DISMOUNTED )) {
//
//             CdRaiseStatus( IrpContext, STATUS_VOLUME_DISMOUNTED );
//
//         } else {
//
//             CdRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         }
//     }
//
//     //
//     //  Capture the real device verify state.
//     //
//
//     DevMarkedForVerify = CdRealDevNeedsVerify( Vcb->Vpb->RealDevice);
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_REMOVABLE_MEDIA ) && !DevMarkedForVerify) {
//
//         //
//         //  If the media is removable and the verify volume flag in the
//         //  device object is not set then we want to ping the device
//         //  to see if it needs to be verified.
//         //
//
//         if (Vcb->VcbCondition != VcbMountInProgress) {
//
//             Status = CdPerformDevIoCtrl( IrpContext,
//                                          IOCTL_CDROM_CHECK_VERIFY,
//                                          Vcb->TargetDeviceObject,
//                                          &MediaChangeCount,
//                                          sizeof(ULONG),
//                                          FALSE,
//                                          FALSE,
//                                          &Iosb );
//
//             if (Iosb.Information != sizeof(ULONG)) {
//
//                 //
//                 //  Be safe about the count in case the driver didn't fill it in
//                 //
//
//                 MediaChangeCount = 0;
//             }
//
//             //
//             //  There are four cases when we want to do a verify.  These are the
//             //  first three.
//             //
//             //  1. We are mounted,  and the device has become empty
//             //  2. The device has returned verify required (=> DO_VERIFY_VOL flag is
//             //     set, but could be due to hardware condition)
//             //  3. Media change count doesn't match the one in the Vcb
//             //
//
//             if (((Vcb->VcbCondition == VcbMounted) &&
//                  CdIsRawDevice( IrpContext, Status ))
//                 ||
//                 (Status == STATUS_VERIFY_REQUIRED)
//                 ||
//                 (NT_SUCCESS(Status) &&
//                  (Vcb->MediaChangeCount != MediaChangeCount))) {
//
//                 //
//                 //  If we are currently the volume on the device then it is our
//                 //  responsibility to set the verify flag.  If we're not on the device,
//                 //  then we shouldn't touch the flag.
//                 //
//
//                 if (!FlagOn( Vcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE) &&
//                     !DevMarkedForVerify)  {
//
//                      DevMarkedForVerify = CdMarkDevForVerifyIfVcbMounted( Vcb);
//                 }
//
//                 ForceVerify = TRUE;
//
//                 //
//                 //  NOTE that we no longer update the media change count here. We
//                 //  do so only when we've actually completed a verify at a particular
//                 //  change count value.
//                 //
//             }
//         }
//
//         //
//         //  This is the 4th verify case.
//         //
//         //  We ALWAYS force CREATE requests on unmounted volumes through the
//         //  verify path.  These requests could have been in limbo between
//         //  IoCheckMountedVpb and us when a verify/mount took place and caused
//         //  a completely different fs/volume to be mounted.  In this case the
//         //  checks above may not have caught the condition,  since we may already
//         //  have verified (wrong volume) and decided that we have nothing to do.
//         //  We want the requests to be re routed to the currently mounted volume,
//         //  since they were directed at the 'drive',  not our volume.
//         //
//
//         if (NT_SUCCESS( Status) && !ForceVerify && !DevMarkedForVerify &&
//             (IrpContext->MajorFunction == IRP_MJ_CREATE))  {
//
//             PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( IrpContext->Irp);
//
//             ForceVerify = (IrpSp->FileObject->RelatedFileObject == NULL) &&
//                           ((Vcb->VcbCondition == VcbDismountInProgress) ||
//                            (Vcb->VcbCondition == VcbNotMounted));
//
//             //
//             //  Note that we don't touch the device verify flag here.  It required
//             //  it would have been caught and set by the first set of checks.
//             //
//         }
//     }
//
//     //
//     //  Raise the verify / error if neccessary.
//     //
//
//     if (ForceVerify || DevMarkedForVerify || !NT_SUCCESS( Status)) {
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->Irp,
//                                       Vcb->Vpb->RealDevice );
//
//         CdRaiseStatus( IrpContext, (ForceVerify || DevMarkedForVerify)
//                                    ? STATUS_VERIFY_REQUIRED
//                                    : Status);
//     }
//
//     //
//     //  Based on the condition of the Vcb we'll either return to our
//     //  caller or raise an error condition
//     //
//
//     switch (Vcb->VcbCondition) {
//
//     case VcbNotMounted:
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->Irp, Vcb->Vpb->RealDevice );
//
//         CdRaiseStatus( IrpContext, STATUS_WRONG_VOLUME );
//         break;
//
//     case VcbInvalid:
//     case VcbDismountInProgress:
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_DISMOUNTED )) {
//
//             CdRaiseStatus( IrpContext, STATUS_VOLUME_DISMOUNTED );
//
//         } else {
//
//             CdRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         }
//         break;
//
//     /* ReactOS Change: GCC "enumeration value not handled in switch" */
//     default: break;
//     }
// }
//
//
// BOOLEAN
// CdVerifyFcbOperation (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to verify that the state of the Fcb is valid
//     to allow the current operation to continue.  We use the state of the
//     Vcb, target device and type of operation to determine this.
//
// Arguments:
//
//     IrpContext - IrpContext for the request.  If not present then we
//         were called from the fast IO path.
//
//     Fcb - Fcb to perform the request on.
//
// Return Value:
//
//     BOOLEAN - TRUE if the request can continue, FALSE otherwise.
//
// --*/
//
// {
//     PVCB Vcb = Fcb->Vcb;
//     PDEVICE_OBJECT RealDevice = Vcb->Vpb->RealDevice;
//     PIRP Irp;
//
//     PAGED_CODE();
//
//     //
//     //  Check that the fileobject has not been cleaned up.
//     //
//
//     if ( ARGUMENT_PRESENT( IrpContext ))  {
//
//         PFILE_OBJECT FileObject;
//
//         Irp = IrpContext->Irp;
//         FileObject = IoGetCurrentIrpStackLocation( Irp)->FileObject;
//
//         if ( FileObject && FlagOn( FileObject->Flags, FO_CLEANUP_COMPLETE))  {
//
//             PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//             //
//             //  Following FAT,  we allow certain operations even on cleaned up
//             //  file objects.  Everything else,  we fail.
//             //
//
//             if ( (FlagOn(Irp->Flags, IRP_PAGING_IO)) ||
//                  (IrpSp->MajorFunction == IRP_MJ_CLOSE ) ||
//                  (IrpSp->MajorFunction == IRP_MJ_QUERY_INFORMATION) ||
//                  ( (IrpSp->MajorFunction == IRP_MJ_READ) &&
//                    FlagOn(IrpSp->MinorFunction, IRP_MN_COMPLETE) ) ) {
//
//                 NOTHING;
//
//             } else {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CLOSED );
//             }
//         }
//     }
//
//     //
//     //  Fail immediately if the volume is in the progress of being dismounted
//     //  or has been marked invalid.
//     //
//
//     if ((Vcb->VcbCondition == VcbInvalid) ||
//         (Vcb->VcbCondition == VcbDismountInProgress)) {
//
//         if (ARGUMENT_PRESENT( IrpContext )) {
//
//             if (FlagOn( Vcb->VcbState, VCB_STATE_DISMOUNTED )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_VOLUME_DISMOUNTED );
//
//             } else {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//             }
//         }
//
//         return FALSE;
//     }
//
//     //
//     //  Always fail if the volume needs to be verified.
//     //
//
//     if (CdRealDevNeedsVerify( RealDevice)) {
//
//         if (ARGUMENT_PRESENT( IrpContext )) {
//
//             IoSetHardErrorOrVerifyDevice( IrpContext->Irp,
//                                           RealDevice );
//
//             CdRaiseStatus( IrpContext, STATUS_VERIFY_REQUIRED );
//         }
//
//         return FALSE;
//
//     //
//     //
//     //  All operations are allowed on mounted.
//     //
//
//     } else if ((Vcb->VcbCondition == VcbMounted) ||
//                (Vcb->VcbCondition == VcbMountInProgress)) {
//
//         return TRUE;
//
//     //
//     //  Fail all requests for fast Io on other Vcb conditions.
//     //
//
//     } else if (!ARGUMENT_PRESENT( IrpContext )) {
//
//         return FALSE;
//
//     //
//     //  The remaining case is VcbNotMounted.
//     //  Mark the device to be verified and raise WRONG_VOLUME.
//     //
//
//     } else if (Vcb->VcbCondition == VcbNotMounted) {
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->Irp, RealDevice );
//         CdRaiseStatus( IrpContext, STATUS_WRONG_VOLUME );
//
// //        return FALSE; // unreachable code
//     }
//
//     return TRUE;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdDismountVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when all of the user references to a volume are
//     gone.  We will initiate all of the teardown any system resources.
//
//     If all of the references to this volume are gone at the end of this routine
//     then we will complete the teardown of this Vcb and mark the current Vpb
//     as not mounted.  Otherwise we will allocated a new Vpb for this device
//     and keep the current Vpb attached to the Vcb.
//
// Arguments:
//
//     Vcb - Vcb for the volume to dismount.
//
// Return Value:
//
//     BOOLEAN - TRUE if we didn't delete the Vcb, FALSE otherwise.
//
// --*/
//
// {
//     PVPB OldVpb;
//     BOOLEAN VcbPresent = TRUE;
//     KIRQL SavedIrql;
//
//     BOOLEAN FinalReference;
//
//     ASSERT_EXCLUSIVE_CDDATA;
//     ASSERT_EXCLUSIVE_VCB( Vcb );
//
//     CdLockVcb( IrpContext, Vcb );
//
//     //
//     //  We should only take this path once.
//     //
//
//     NT_ASSERT( Vcb->VcbCondition != VcbDismountInProgress );
//
//     //
//     //  Mark the Vcb as DismountInProgress.
//     //
//
//     Vcb->VcbCondition = VcbDismountInProgress;
//
//     if (Vcb->XASector != NULL) {
//
//         CdFreePool( &Vcb->XASector );
//         Vcb->XASector = 0;
//         Vcb->XADiskOffset = 0;
//     }
//
//     //
//     //  Remove our reference to the internal Fcb's.  The Fcb's will then
//     //  be removed in the purge path below.
//     //
//
//     if (Vcb->RootIndexFcb != NULL) {
//
//         Vcb->RootIndexFcb->FcbReference -= 1;
//         Vcb->RootIndexFcb->FcbUserReference -= 1;
//     }
//
//     if (Vcb->PathTableFcb != NULL) {
//
//         Vcb->PathTableFcb->FcbReference -= 1;
//         Vcb->PathTableFcb->FcbUserReference -= 1;
//     }
//
//     if (Vcb->VolumeDasdFcb != NULL) {
//
//         Vcb->VolumeDasdFcb->FcbReference -= 1;
//         Vcb->VolumeDasdFcb->FcbUserReference -= 1;
//     }
//
//     CdUnlockVcb( IrpContext, Vcb );
//
//     //
//     //  Purge the volume.
//     //
//
//     CdPurgeVolume( IrpContext, Vcb, TRUE );
//
//     //
//     //  Empty the delayed and async close queues.
//     //
//
//     CdFspClose( Vcb );
//
//     OldVpb = Vcb->Vpb;
//
//     //
//     //  Remove the mount volume reference.
//     //
//
//     CdLockVcb( IrpContext, Vcb );
//     Vcb->VcbReference -= 1;
//
//     //
//     //  Acquire the Vpb spinlock to check for Vpb references.
//     //
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     //
//     //  Remember if this is the last reference on this Vcb.  We incremented
//     //  the count on the Vpb earlier so we get one last crack it.  If our
//     //  reference has gone to zero but the vpb reference count is greater
//     //  than zero then the Io system will be responsible for deleting the
//     //  Vpb.
//     //
//
//     FinalReference = (BOOLEAN) ((Vcb->VcbReference == 0) &&
//                                 (OldVpb->ReferenceCount == 1));
//
//     //
//     //  There is a reference count in the Vpb and in the Vcb.  We have
//     //  incremented the reference count in the Vpb to make sure that
//     //  we have last crack at it.  If this is a failed mount then we
//     //  want to return the Vpb to the IO system to use for the next
//     //  mount request.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//     if (OldVpb->RealDevice->Vpb == OldVpb) {
//
//         //
//         //  If not the final reference then swap out the Vpb.  We must
//         //  preserve the REMOVE_PENDING flag so that the device is
//         //  not remounted in the middle of a PnP remove operation.
//         //
//
//         if (!FinalReference) {
//
//             NT_ASSERT( Vcb->SwapVpb != NULL );
//
//             Vcb->SwapVpb->Type = IO_TYPE_VPB;
//             Vcb->SwapVpb->Size = sizeof( VPB );
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(disable: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//             Vcb->SwapVpb->RealDevice = OldVpb->RealDevice;
//             Vcb->SwapVpb->RealDevice->Vpb = Vcb->SwapVpb;
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//             Vcb->SwapVpb->Flags = FlagOn( OldVpb->Flags, VPB_REMOVE_PENDING );
//
//             IoReleaseVpbSpinLock( SavedIrql );
//
//             //
//             //  Indicate we used up the swap.
//             //
//
//             Vcb->SwapVpb = NULL;
//
//             CdUnlockVcb( IrpContext, Vcb );
//
//         //
//         //  We want to leave the Vpb for the IO system.  Mark it
//         //  as being not mounted.  Go ahead and delete the Vcb as
//         //  well.
//         //
//
//         } else {
//
//             //
//             //  Make sure to remove the last reference on the Vpb.
//             //
//
//             OldVpb->ReferenceCount -= 1;
//
//             OldVpb->DeviceObject = NULL;
//             ClearFlag( Vcb->Vpb->Flags, VPB_MOUNTED );
//             ClearFlag( Vcb->Vpb->Flags, VPB_LOCKED );
//
//             //
//             //  Clear the Vpb flag so we know not to delete it.
//             //
//
//             Vcb->Vpb = NULL;
//
//             IoReleaseVpbSpinLock( SavedIrql );
//             CdUnlockVcb( IrpContext, Vcb );
//             CdDeleteVcb( IrpContext, Vcb );
//             VcbPresent = FALSE;
//         }
//
//     //
//     //  Someone has already swapped in a new Vpb.  If this is the final reference
//     //  then the file system is responsible for deleting the Vpb.
//     //
//
//     } else if (FinalReference) {
//
//         //
//         //  Make sure to remove the last reference on the Vpb.
//         //
//
//         OldVpb->ReferenceCount -= 1;
//
//         IoReleaseVpbSpinLock( SavedIrql );
//         CdUnlockVcb( IrpContext, Vcb );
//         CdDeleteVcb( IrpContext, Vcb );
//         VcbPresent = FALSE;
//
//     //
//     //  The current Vpb is no longer the Vpb for the device (the IO system
//     //  has already allocated a new one).  We leave our reference in the
//     //  Vpb and will be responsible for deleting it at a later time.
//     //
//
//     } else {
//
//         IoReleaseVpbSpinLock( SavedIrql );
//         CdUnlockVcb( IrpContext, Vcb );
//     }
//
//     //
//     //  Let our caller know whether the Vcb is still present.
//     //
//
//     return VcbPresent;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
