use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::base::{STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, ExQueueWorkItem, ExReleaseResourceForThreadLite, ExSetResourceOwnerPointer, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAllocateMdl, IoBuildDeviceIoControlRequest, IoBuildPartialMdl, IoFreeIrp, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoGetTopLevelIrp, IoIsSystemThread, IoMakeAssociatedIrp, IoMarkIrpPending, IoSetCompletionRoutine, IoSetNextIrpStackLocation, KEVENT, KeDelayExecutionThread, KeGetCurrentProcessorNumber, KeInitializeEvent, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, MmProbeAndLockPages, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PETHREAD, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PKEVENT, PLARGE_INTEGER, PMDL, PPFN_NUMBER, PREVENT_MEDIA_REMOVAL, PsGetCurrentThread, PsGetThreadProcess, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CLEAN_AND_DIRTY_VOLUME_PACKET, FAT_BUG_CHECK_DEVIOSUP, FAT_DATA, FAT_RESERVE_MDL_SIZE, FCB, FCB_STATE_PAGING_FILE, FatReserveEvent, FatReserveMdl, IRP_CONTEXT_FLAG_USER_IO, LBO, PFAT_IO_CONTEXT, PFCB, VCB_STATE_FLAG_REMOVAL_PREVENTED, VOLUME_DEVICE_OBJECT};
use ntkrnl_sys::ddk::{KPROCESSOR_MODE, LOCK_OPERATION};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_DEVIOSUP, FO_FILE_FAST_IO_READ, FO_FILE_MODIFIED, MDL_MAPPED_TO_SYSTEM_VA, STATUS_FILE_CORRUPT_ERROR, STATUS_MORE_PROCESSING_REQUIRED, STATUS_VERIFY_REQUIRED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{ExInterlockedAddUlong, IO_COMPLETION_ROUTINE, FsRtlAllocatePoolWithQuotaTag, FsRtlIsNtstatusExpected, IoCallDriver};
use crate::types::{_FAT_IO_CONTEXT, _IRP_CONTEXT, _VCB};
use ntkrnl_sys::ddk::{PIO_COMPLETION_ROUTINE, WORK_QUEUE_TYPE, _DEVICE_OBJECT, _IO_STACK_LOCATION, _IRP};
use crate::types::{IO_RUN, IRP_CONTEXT, VCB};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, IRP, MDL};
// Local KeGetCurrentThread via ntkrnl_sys inline (x86_64 FORCEINLINE; arm64/i386 use mm::KeGetCurrentThread from glob above)
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::inline::tier1::KeGetCurrentThread;
// Local ExInitializeWorkItem via ntkrnl_sys inline
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
// Local MmGetMdlPfnArray via ntkrnl_sys inline
use ntkrnl_sys::inline::mm::MmGetMdlPfnArray;

extern "C" {
    // ddk_shims.rs defines MmPrepareMdlForReuse cdecl (§9.30 cc-match rule)
    fn MmPrepareMdlForReuse(Mdl: PMDL);
}
extern "system" {
    static mut FatIoCallDriverCount: u32;
    static mut FatDiskAccountingEnabled: u8;
    static Dbg: u32;
    static mut FatData: FAT_DATA;
    static IRP_PAGING_IO: u32;
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    } extern "C" { fn DebugTrace(level: i32, dbg: i32, format: *const i8, ...); } extern "system" {
    fn NT_ASSERT(condition: i32);
    fn FlagOn(Flags: u32, Flag: u32) -> i32;
    fn FatLookupFileAllocation( IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB, StartingVbo: u32, NextLbo: *mut i64, NextByteCount: *mut u32, NextIsAllocated: *mut u8, EndOnMax: *mut u8, p: *mut c_void, );
    fn FatPopUpFileCorrupt(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn KeFlushIoBuffers(Mdl: *mut MDL, ReadOperation: i32, DmaOperation: i32);
}
extern "system" {
    fn FatBugCheck(a: usize, b: usize, c: usize) -> !;
    fn FatLookupMcbEntry(vcb: *mut core::ffi::c_void, mcb: *mut core::ffi::c_void, vbo: u32, next_lbo: *mut u64, next_byte_count: *mut u32, first_index: *mut u32) -> u8;
    fn FatGetNextMcbEntry(vcb: *mut core::ffi::c_void, mcb: *mut core::ffi::c_void, current_index: u32, next_vbo: *mut u32, next_lbo: *mut u64, next_byte_count: *mut u32) -> u8;
    fn FatUpdateIOCountersPCW(is_write: u32, count: u64);
    fn SetFlag(flags: *mut u32, bit: u32);
    fn PsUpdateDiskCounters(process: *mut core::ffi::c_void, bytes_read: u64, bytes_written: u64, read_ops: u32, write_ops: u32, flags: u32);
    fn BooleanFlagOn(flags: u32, flag: u32) -> u8;
    fn DebugUnwind(function: *const core::ffi::c_void);
    fn FatLowLevelReadWrite( IrpContext: *mut IRP_CONTEXT, DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, Vcb: *mut VCB, ) -> i32; // returns NTSTATUS but (VOID) cast fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: i32);
    fn MmUnlockPages(mdl: PMDL);
    fn FsRtlUpdateDiskCounters(read: u64, write: u64);
    fn FsRtlIsTotalDeviceFailure(NtStatus: NTSTATUS) -> BOOLEAN;
    fn ClearFlag(flags: *mut u32, flag: u32);
    fn FatCreateIrpContext(Irp: PIRP, RecursiveCall: i32) -> *mut IRP_CONTEXT;
    fn FatFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: PIRP) -> u32;
    fn DbgBreakPoint();
    } extern "C" { fn DbgPrint(fmt: *const i8, ...); } extern "system" {
    static mut FatBreakOnInterestingIoCompletion: i32;

    fn CollectDiskIoStats(Vcb: *mut VCB, Function: u32, IsUserIo: u8, Count: u32);
    fn FsRtlAllocatePoolWithTag(PoolType: u32, NumberOfBytes: u32, Tag: u32) -> *mut core::ffi::c_void;
}
#[cfg(target_arch = "x86")]
extern "stdcall" {
    fn FatFspMarkVolumeDirtyWithRecover(Parameter: PVOID);
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn FatFspMarkVolumeDirtyWithRecover(Parameter: PVOID);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_DEVIOSUP;

// Constants from wdm.h / ntddk.h not in bindgen surface
const NormalPagePriority: u32 = 16;
const IoReadAccess: u32 = 0;   // _LOCK_OPERATION_IoReadAccess
const CriticalWorkQueue: u32 = 0; // _WORK_QUEUE_TYPE_CriticalWorkQueue
// IOCTL_DISK_MEDIA_REMOVAL = CTL_CODE(IOCTL_DISK_BASE=7, 0x201, METHOD_BUFFERED=0, FILE_READ_ACCESS=1)
//   = (7 << 16) | (1 << 14) | (0x201 << 2) | 0 = 0x70048
const IOCTL_DISK_MEDIA_REMOVAL: u32 = 0x00074804;
// SL_INVOKE_ON_* flags (from wdm.h Control field)
const SL_INVOKE_ON_SUCCESS: u8 = 0x40;
const SL_INVOKE_ON_ERROR: u8 = 0x80;
const SL_INVOKE_ON_CANCEL: u8 = 0x20;

/// MmGetMdlVirtualAddress — C macro: (Mdl)->StartVa + (Mdl)->ByteOffset
#[inline(always)]
unsafe fn MmGetMdlVirtualAddress(mdl: *mut MDL) -> *mut c_void {
    ((*mdl).StartVa as *mut u8).add((*mdl).ByteOffset as usize) as *mut c_void
}

/// DebugDoit(X) is a retail no-op debug macro (the argument expression still
/// evaluates for its side effects). Model it as an inline generic so it emits no
/// linker symbol — avoids an i386 stdcall-arity (@0 vs @4) cohab mismatch.
#[inline(always)]
unsafe fn DebugDoit<T>(_expr: T) {}

/// Helper: transmute an `extern "system"` completion routine to the `extern "C"`
/// type that PIO_COMPLETION_ROUTINE requires.  On amd64 the calling conventions
/// are identical so this is a zero-cost cast.
#[inline(always)]
unsafe fn as_completion_routine(
    f: unsafe extern "system" fn(
        *mut ntkrnl_sys::ddk::_DEVICE_OBJECT,
        *mut ntkrnl_sys::ddk::_IRP,
        *mut c_void,
    ) -> i32,
) -> ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE {
    Some(core::mem::transmute(f))
}

#[repr(C)]
pub struct FAT_SYNC_CONTEXT {
    pub Iosb: _IO_STATUS_BLOCK,
    pub Event: KEVENT,
}
pub type PFAT_SYNC_CONTEXT = *mut FAT_SYNC_CONTEXT;

// ---- hand-translated (translate-time parser missed the C tail; see deviosup.c:2406+) ----

//  FatDoCompletionZero macro: zero the buffer behind ZeroMdl on success, then free it.
unsafe fn FatDoCompletionZeroHelper(Irp: PIRP, Context: crate::types::PFAT_IO_CONTEXT) {
    if !(*Context).ZeroMdl.is_null() {
        if ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
            core::ptr::write_bytes(
                (*(*Context).ZeroMdl).MappedSystemVa as *mut u8,
                0,
                (*(*Context).ZeroMdl).ByteCount as usize,
            );
        }
        IoFreeMdl((*Context).ZeroMdl);
        (*Context).ZeroMdl = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatMultiSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let Context = Contxt as crate::types::PFAT_IO_CONTEXT;
    let MasterIrp = (*Context).MasterIrp;

    //  If we got an error (or verify required), remember it in the Irp
    if !ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        core::ptr::write(
            addr_of_mut!((*MasterIrp).IoStatus),
            core::ptr::read(addr_of!((*Irp).IoStatus)),
        );
    }

    //  We must do this here since IoCompleteRequest won't get a chance
    //  on this associated Irp.
    IoFreeMdl((*Irp).MdlAddress);
    IoFreeIrp(Irp);

    if InterlockedDecrement(addr_of_mut!((*Context).IrpCount)) == 0 {
        FatDoCompletionZeroHelper(MasterIrp, Context);
        KeSetEvent((*Context).Wait.SyncEvent.as_mut() as *mut _ as *mut c_void, 0, 0);
    }

    let _ = DeviceObject;
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn FatMultiAsyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS
    let Context = Contxt as crate::types::PFAT_IO_CONTEXT;
    let MasterIrp = (*Context).MasterIrp;
    let mut PostRequest = false;

    //  If we got an error (or verify required), remember it in the Irp
    if !ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        core::ptr::write(
            addr_of_mut!((*MasterIrp).IoStatus),
            core::ptr::read(addr_of!((*Irp).IoStatus)),
        );
    }

    if InterlockedDecrement(addr_of_mut!((*Context).IrpCount)) == 0 {
        FatDoCompletionZeroHelper(MasterIrp, Context);

        if ntkrnl_sys::base::NT_SUCCESS((*MasterIrp).IoStatus.__bindgen_anon_1.Status) {
            (*MasterIrp).IoStatus.Information =
                ((*Context).Wait.Async.as_ref().RequestedByteCount as ULONG_PTR) as _;

            //  Now if this wasn't PagingIo, set either the read or write bit.
            if (*MasterIrp).Flags & crate::glue::IRP_PAGING_IO == 0 {
                let FileObject = (*Context).Wait.Async.as_ref().FileObject;
                let major = (*IoGetCurrentIrpStackLocation(MasterIrp)).MajorFunction;
                (*FileObject).Flags |= if major == ntkrnl_sys::constants::IRP_MJ_READ {
                    crate::glue::FO_FILE_FAST_IO_READ
                } else {
                    crate::glue::FO_FILE_MODIFIED
                };
            }
        } else {
            //  Post STATUS_VERIFY_REQUIRED failures. Only post top level IRPs,
            //  because recursive I/Os cannot process volume verification.
            if (*Context).IrpContextFlags & crate::types::IRP_CONTEXT_FLAG_RECURSIVE_CALL == 0
                && (*MasterIrp).IoStatus.__bindgen_anon_1.Status
                    == crate::glue::STATUS_VERIFY_REQUIRED
            {
                PostRequest = true;
            }
        }

        //  If this was a special async write, decrement the count.  Set the
        //  event if this was the final outstanding I/O for the file.
        let NonPagedFcb = (*Context).Wait.Async.as_ref().NonPagedFcb;
        if !NonPagedFcb.is_null()
            && ExInterlockedAddUlong(
                addr_of_mut!((*NonPagedFcb).OutstandingAsyncWrites),
                0xffffffff,
                addr_of_mut!(FatData.GeneralSpinLock),
            ) == 1
        {
            KeSetEvent(((*NonPagedFcb).OutstandingAsyncEvent).cast(), 0, 0);
        }

        //  Now release the resources.
        let resource = (*Context).Wait.Async.as_ref().Resource;
        if !resource.is_null() {
            ExReleaseResourceForThreadLite(
                resource,
                (*Context).Wait.Async.as_ref().ResourceThreadId,
            );
        }
        let resource2 = (*Context).Wait.Async.as_ref().Resource2;
        if !resource2.is_null() {
            ExReleaseResourceForThreadLite(
                resource2,
                (*Context).Wait.Async.as_ref().ResourceThreadId,
            );
        }

        //  Mark the master Irp pending
        IoMarkIrpPending(MasterIrp);

        //  and finally, free the context record.
        ExFreePool(Context as *mut c_void);

        if PostRequest {
            // _SEH2_EXCEPT(FatExceptionFilter(NULL, ..)): if we failed to post
            // the IRP, we just have to return the failure to the user.
            let NoIrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
            let p_status = addr_of_mut!(Status);
            let _ = crate::seh::fat_try_except(addr_of!(NoIrpContext), || {
                let IrpContext =
                    FatCreateIrpContext(Irp, 1) as *mut crate::types::IRP_CONTEXT;
                (*IrpContext).Flags &= !crate::types::IRP_CONTEXT_FLAG_RECURSIVE_CALL;
                FatFsdPostRequest(IrpContext as *mut IRP_CONTEXT, Irp);
                *p_status = crate::glue::STATUS_MORE_PROCESSING_REQUIRED;
            });
        }
    }

    let _ = DeviceObject;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatSpecialSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let SyncContext = Contxt as PFAT_SYNC_CONTEXT;

    core::ptr::write(
        addr_of_mut!((*SyncContext).Iosb),
        core::ptr::read(addr_of!((*Irp).IoStatus)),
    );

    KeSetEvent((addr_of_mut!((*SyncContext).Event)).cast(), 0, 0);

    let _ = DeviceObject;
    0 // STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FatSingleSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let Context = Contxt as crate::types::PFAT_IO_CONTEXT;

    FatDoCompletionZeroHelper(Irp, Context);

    KeSetEvent((*Context).Wait.SyncEvent.as_mut() as *mut _ as *mut c_void, 0, 0);

    let _ = DeviceObject;
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn FatPagingFileCompletionRoutineCatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let Context = Contxt as PFAT_PAGING_FILE_CONTEXT;

    //  Cleanup the existing Mdl, perhaps by returning the reserve.
    if (*Irp).MdlAddress == FatReserveMdl {
        MmPrepareMdlForReuse((*Irp).MdlAddress);
        KeSetEvent((addr_of_mut!(FatReserveEvent)).cast(), 0, 0);
    } else {
        IoFreeMdl((*Irp).MdlAddress);
    }

    //  Restore the original Mdl.
    (*Irp).MdlAddress = (*Context).RestoreMdl;

    //  If the IRP is succeeding or the failure handler did not post off the
    //  completion, we're done and should set the event to let the master
    //  know the IRP is his again.
    if ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status)
        || FatPagingFileErrorHandler(Irp, addr_of_mut!((*Context).Event)) == 0
    {
        KeSetEvent((addr_of_mut!((*Context).Event)).cast(), 0, 0);
    }

    let _ = DeviceObject;
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}
// ---- re-stitched from bank: 20260612-152050-deviosup.c/05_FatSingleNonAlignedSync.rs ----

#[no_mangle]
pub unsafe extern "system" fn FatSingleNonAlignedSync(
    irp_context: *mut IRP_CONTEXT,
    vcb: *mut VCB,
    buffer: *mut u8,
    lbo: i64,
    byte_count: u32,
    irp: *mut IRP,
) {
    let irp_sp: PIO_STACK_LOCATION;
    let mdl: PMDL;
    let saved_mdl: PMDL;
    #[cfg(not(reactos))]
    let mut is_a_write: bool = false;

    PAGED_CODE!();

    DebugTrace(
        1,
        (Dbg) as i32,
        b"FatSingleNonAlignedAsync\n\0".as_ptr() as *const i8,
        0,
    );
    DebugTrace(
        0,
        (Dbg) as i32,
        b"MajorFunction = %08lx\n\0".as_ptr() as *const i8,
        (*irp_context).MajorFunction as u32,
    );
    DebugTrace(0, (Dbg) as i32, b"Vcb           = %p\n\0".as_ptr() as *const i8, vcb);
    DebugTrace(
        0,
        (Dbg) as i32,
        b"Buffer        = %p\n\0".as_ptr() as *const i8,
        buffer,
    );
    DebugTrace(0, (Dbg) as i32, b"Lbo           = %08lx\n\0".as_ptr() as *const i8, lbo);
    DebugTrace(
        0,
        (Dbg) as i32,
        b"ByteCount     = %08lx\n\0".as_ptr() as *const i8,
        byte_count,
    );
    DebugTrace(0, (Dbg) as i32, b"Irp           = %p\n\0".as_ptr() as *const i8, irp);

    saved_mdl = (*irp).MdlAddress;
    (*irp).MdlAddress = core::ptr::null_mut();

    mdl = IoAllocateMdl(buffer as *mut c_void, byte_count, 0u8, 0u8, (irp).cast());

    if mdl.is_null() {
        (*irp).MdlAddress = saved_mdl;
        FatRaiseStatus(irp_context, STATUS_INSUFFICIENT_RESOURCES);
    }

    crate::seh::try_finally(
        &mut (),
        |_| {
            MmProbeAndLockPages(mdl, KernelMode, (IoWriteAccess) as u32);
        },
        |_, abnormal| {
            if abnormal {
                IoFreeMdl(mdl);
                (*irp).MdlAddress = saved_mdl;
            }
        },
    );

    // Context is IrpContext->FatIoContext itself (the completion routine
    // reads it as a PFAT_IO_CONTEXT), not the address of the field.
    IoSetCompletionRoutine(
        (irp).cast(),
        as_completion_routine(FatSingleSyncCompletionRoutine),
        (*irp_context).FatIoContext as *mut c_void,
        1u8,
        1u8,
        1u8,
    );

    irp_sp = IoGetNextIrpStackLocation((irp).cast());
    (*irp_sp).MajorFunction = (*irp_context).MajorFunction;
    (*irp_sp).Parameters.Read.Length = byte_count;
    (*irp_sp).Parameters.Read.ByteOffset.QuadPart = lbo;

    #[cfg(not(reactos))]
    {
        is_a_write = (*irp_sp).MajorFunction == (IRP_MJ_WRITE) as u8;
    }

    if (*vcb).VerifyThread == KeGetCurrentThread() as *mut _ {
        SetFlag(&raw mut (*irp_context).Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY);
    }
    if FlagOn((*irp_context).Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY) != 0 {
        SetFlag((&raw mut (*irp_sp).Flags).cast(), SL_OVERRIDE_VERIFY_VOLUME);
    }

    FatIoCallDriverCount += 1;

    crate::seh::try_finally(
        &mut (),
        |_| {
            // FatLowLevelReadWrite is IoCallDriver.
            IoCallDriver((*vcb).TargetDeviceObject, (irp).cast());
            FatWaitSync(irp_context);
        },
        |_, _abnormal| {
            MmUnlockPages(mdl);
            IoFreeMdl(mdl);
            (*irp).MdlAddress = saved_mdl;
        },
    );

    if FatDiskAccountingEnabled != 0 {
        #[cfg(ntddi_ge_ntddi_win8)]
        {
            FsRtlUpdateDiskCounters(
                if is_a_write { 0 } else { byte_count as u64 },
                if is_a_write { byte_count as u64 } else { 0 },
            );
        }
    }

    DebugTrace(
        -1,
        (Dbg) as i32,
        b"FatSingleNonAlignedSync -> VOID\n\0".as_ptr() as *const i8,
        0,
    );
    return;
}


#[repr(C)]
pub struct FAT_PAGING_FILE_CONTEXT {
    pub Event: KEVENT,
    pub RestoreMdl: PMDL,
}
pub type PFAT_PAGING_FILE_CONTEXT = *mut FAT_PAGING_FILE_CONTEXT;

// Forward declarations for unknown functions and globals

// Opaque type declarations (assumed to exist elsewhere)

// Use the provided imports for known NT functions

// Forward extern declarations for unknown functions

// Unknown constants - placeholder values
const IRP_CONTEXT_FLAG_OVERRIDE_VERIFY: u32 = 0x1000;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000002;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x00000004;
const SL_WRITE_THROUGH: u32 = 0x04;
const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x00000002;
const IRP_MJ_WRITE: u32 = 4;


const STATUS_INVALID_USER_BUFFER: i32 = 0xC00000E8_u32 as i32;

const MdlMappingNoExecute: u32 = 0x40000000;

unsafe extern "system" {
    fn BlockAlignTruncate(length: u32, alignment: u32) -> u32;
}

// Assume these constants from the original environment:
const MDL_SOURCE_IS_NONPAGED_POOL: u32 = 0x0004;
const MDL_PAGES_LOCKED: u32 = 0x0002;

/// Extern reference to the global FatData structure.
/// The caller provides the actual definition.
// The FAT_DATA type is expected to be defined elsewhere; for now we rely on
// the fact that ZeroPage is a PVOID field.

// Forward declarations for unknown functions/macros

// Constants not available in imports
const FAT_MAX_IO_RUNS_ON_STACK: usize = 5; // fatdata.h: ((ULONG) 5)
const TAG_IO_RUNS: u32 = 0x69746146; // Just a placeholder; use real tag if known

// Global FatData (assumed defined elsewhere; minimal access)

const IRP_BUFFERED_IO: u32 = 0x00000010;
const IRP_DEALLOCATE_BUFFER: u32 = 0x00000020;
// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: POOL_TYPE = 0;
                                     // -- ReactOS downgrades the NT6.2 NX pool types, so the
                                     // literal 0x200 the translation used is not a pool type
                                     // this kernel knows.
const TAG_IO_USER_BUFFER: u32 = 0x51746146; // 'bUu' tag placeholder

// Declarations for unknown external functions used by this function

// Constants from file level or needed locally
const IoWriteAccess: u32 = 1; // LOCK_OPERATION: _LOCK_OPERATION_IoWriteAccess
// fatprocs.h:47: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: u32 = 4;
                                           // NonPagedPoolCacheAligned` (= 4, pooltype.h)
const TAG_IO_BUFFER: u32 = 0x4F746146; // nolint: const-contradiction ('OtaF' -- fastfat/nodetype.h; cdfs' is 'fbdC')
#[no_mangle]
pub unsafe extern "system" fn FatPagingFileIo(Irp: PIRP, Fcb: PFCB) {

    #[repr(C)]
    struct LARGE_INTEGER {
        QuadPart: i64,
    }

    let mut Vbo: u32;
    let mut ByteCount: u32;
    let mut Mdl: PMDL;
    let mut NextLbo: u64 = 0;
    let mut NextVbo: u32 = 0;
    let mut NextByteCount: u32 = 0;
    let mut RemainingByteCount: u32;
    let mut MustSucceed: u32;
    let mut FirstIndex: u32 = 0;
    let mut CurrentIndex: u32;
    let mut LastIndex: u32 = 0;
    let mut LastLbo: u64 = 0;
    let mut LastByteCount: u32 = 0;
    let mut MdlIsReserve: u32 = 0;
    let mut IrpIsMaster: u32 = 0;
    let mut Context: FAT_PAGING_FILE_CONTEXT = core::mem::zeroed();
    let mut IrpCount: i32;
    let mut AssocIrp: PIRP;
    let IrpSp: PIO_STACK_LOCATION;
    let mut NextIrpSp: PIO_STACK_LOCATION;
    let mut BufferOffset: u32;
    let DeviceObject: PDEVICE_OBJECT;
    let mut IsAWrite: u32 = 0;

    DebugTrace(1, (Dbg as u32) as i32, c"FatPagingFileIo\n".as_ptr() as *const i8, 0);
    DebugTrace(0, (Dbg as u32) as i32, c"Irp = %p\n".as_ptr() as *const i8, Irp as usize);
    DebugTrace(0, (Dbg as u32) as i32, c"Fcb = %p\n".as_ptr() as *const i8, Fcb as usize);

    if FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0 {
        ntkrnl_sys::ke::DbgBreakPoint();
    }

    BufferOffset = 0;
    DeviceObject = (*(*Fcb).Vcb).TargetDeviceObject;
    IrpSp = ntkrnl_sys::ddk::IoGetCurrentIrpStackLocation(Irp);

    Vbo = (*IrpSp).Parameters.Read.ByteOffset.__bindgen_anon_1.LowPart;
    ByteCount = (*IrpSp).Parameters.Read.Length;
    IsAWrite = ((*IrpSp).MajorFunction == ntkrnl_sys::constants::IRP_MJ_WRITE) as u32;

    MustSucceed = FatLookupMcbEntry(
        (*Fcb).Vcb as *mut c_void,
        addr_of_mut!((*Fcb).Mcb) as *mut c_void,
        Vbo,
        addr_of_mut!(NextLbo) as *mut u64,
        addr_of_mut!(NextByteCount),
        addr_of_mut!(FirstIndex),
    ) as u32;

    if MustSucceed == 0 {
        FatBugCheck(Vbo as usize, ByteCount as usize, 0);
    }

    if FatDiskAccountingEnabled != 0 {
        let ThreadIssuingIo: ntkrnl_sys::ddk::PETHREAD = PsGetCurrentThread();
        let IsWriteOperation = ((*IrpSp).MajorFunction == ntkrnl_sys::constants::IRP_MJ_WRITE) as u32;
        PsUpdateDiskCounters(
            ntkrnl_sys::ddk::PsGetThreadProcess(ThreadIssuingIo) as *mut core::ffi::c_void,
            if IsWriteOperation != 0 { (0u32) as u64 } else { (ByteCount) as u64 },
            if IsWriteOperation != 0 { (ByteCount) as u64 } else { (0u32) as u64 },
            if IsWriteOperation != 0 { 0u32 } else { 1u32 },
            if IsWriteOperation != 0 { 1u32 } else { 0u32 },
            0,
        );
    }

    if NextByteCount >= ByteCount {
        DebugTrace(0, (Dbg as u32) as i32, c"Passing Irp on to Disk Driver\n".as_ptr() as *const i8, 0);

        NextIrpSp = ntkrnl_sys::ddk::IoGetNextIrpStackLocation(Irp);

        (*NextIrpSp).MajorFunction = (*IrpSp).MajorFunction;
        (*NextIrpSp).Parameters.Read.Length = ByteCount;
        (*NextIrpSp).Parameters.Read.ByteOffset.QuadPart = NextLbo as i64;

        SetFlag((addr_of_mut!((*NextIrpSp).Flags)).cast() , SL_OVERRIDE_VERIFY_VOLUME);

        ntkrnl_sys::ddk::IoSetCompletionRoutine(
            Irp,
            as_completion_routine(FatPagingFileCompletionRoutine),
            Irp as *mut core::ffi::c_void,
            0,
            1,
            1,
        );

        IoCallDriver(DeviceObject, Irp);

        if FatDiskAccountingEnabled != 0 {
            FatUpdateIOCountersPCW(IsAWrite, ByteCount as u64);
        }

        DebugTrace(-1, (Dbg as u32) as i32, c"FatPagingFileIo -> VOID\n".as_ptr() as *const i8, 0);
        return;
    }

    MustSucceed = FatLookupMcbEntry(
        (*Fcb).Vcb as *mut c_void,
        addr_of_mut!((*Fcb).Mcb) as *mut c_void,
        Vbo.wrapping_add(ByteCount).wrapping_sub(1),
        addr_of_mut!(LastLbo) as *mut u64,
        addr_of_mut!(LastByteCount),
        addr_of_mut!(LastIndex),
    ) as u32;

    if MustSucceed == 0 {
        FatBugCheck(Vbo.wrapping_add(ByteCount).wrapping_sub(1) as usize, 1, 0);
    }

    CurrentIndex = FirstIndex;

    (*Irp).IoStatus.__bindgen_anon_1.Status = ntkrnl_sys::base::STATUS_SUCCESS as i32;
    (*Irp).IoStatus.Information = (ByteCount as usize) as _;

    let count: i32 = (LastIndex.wrapping_sub(FirstIndex).wrapping_add(1)) as i32;
    (*Irp).AssociatedIrp.IrpCount = count;
    IrpCount = count;

    while CurrentIndex <= LastIndex {
        AssocIrp = null_mut();

        if NextByteCount > ByteCount {
            NextByteCount = ByteCount;
        }
        RemainingByteCount = 0;

        Mdl = ntkrnl_sys::ddk::IoAllocateMdl(
            ((*Irp).UserBuffer as usize + BufferOffset as usize) as *mut core::ffi::c_void,
            NextByteCount,
            0,
            0,
            AssocIrp,
        );

        if Mdl.is_null() {
            ntkrnl_sys::ddk::KeWaitForSingleObject(
                (addr_of_mut!(FatReserveEvent)).cast(),
                ntkrnl_sys::ddk::_KWAIT_REASON_Executive,
                0i8 /*KernelMode*/,
                0,
                null_mut(),
            );

            Mdl = FatReserveMdl;
            MdlIsReserve = 1;

            if NextByteCount > (FAT_RESERVE_MDL_SIZE as u32 * ntkrnl_sys::base::PAGE_SIZE as u32) {
                RemainingByteCount = NextByteCount - (FAT_RESERVE_MDL_SIZE as u32 * ntkrnl_sys::base::PAGE_SIZE as u32);
                NextByteCount = FAT_RESERVE_MDL_SIZE as u32 * ntkrnl_sys::base::PAGE_SIZE as u32;
            }
        }

        ntkrnl_sys::ddk::IoBuildPartialMdl(
            (*Irp).MdlAddress,
            Mdl,
            ((*Irp).UserBuffer as usize + BufferOffset as usize) as *mut core::ffi::c_void,
            NextByteCount,
        );

        if MdlIsReserve == 0 &&
            (IrpCount != 1 || (CurrentIndex == LastIndex && RemainingByteCount == 0))
        {
            AssocIrp = ntkrnl_sys::ddk::IoMakeAssociatedIrp(
                Irp,
                ((*DeviceObject).StackSize as i32 + 1) as i8,
            );
        }

        if AssocIrp.is_null() {
            AssocIrp = Irp;
            IrpIsMaster = 1;

            while (*Irp).AssociatedIrp.IrpCount != IrpCount {
                let timeout = LARGE_INTEGER { QuadPart: -300000 };
                ntkrnl_sys::ddk::KeDelayExecutionThread(
                    0i8 /*KernelMode*/,
                    0,
                    addr_of!(timeout) as *const _ as *mut _,
                );
            }

            if !ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                if IrpCount == 0 {
                    ntkrnl_sys::ke::DbgBreakPoint();
                }
                break;
            }
        } else {
            IrpCount -= 1;
        }

        if IrpIsMaster == 0 {
            ntkrnl_sys::ddk::IoSetNextIrpStackLocation(AssocIrp);
            NextIrpSp = ntkrnl_sys::ddk::IoGetCurrentIrpStackLocation(AssocIrp);

            (*NextIrpSp).MajorFunction = (*IrpSp).MajorFunction;
            (*NextIrpSp).Parameters.Read.Length = NextByteCount;
            (*NextIrpSp).Parameters.Read.ByteOffset.QuadPart = Vbo as i64;
            (*NextIrpSp).DeviceObject = (*IrpSp).DeviceObject;
        } else {
            ntkrnl_sys::ddk::KeInitializeEvent(
                core::ptr::addr_of_mut!(Context.Event),
                ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent,
                0,
            );
            Context.RestoreMdl = (*Irp).MdlAddress;
        }

        (*AssocIrp).MdlAddress = Mdl;

        if IrpIsMaster != 0 {
            ntkrnl_sys::ddk::IoSetCompletionRoutine(
                AssocIrp,
                as_completion_routine(FatPagingFileCompletionRoutineCatch),
                addr_of!(Context) as *const _ as *mut core::ffi::c_void,
                1,
                1,
                1,
            );
        } else {
            ntkrnl_sys::ddk::IoSetCompletionRoutine(
                AssocIrp,
                as_completion_routine(FatPagingFileCompletionRoutine),
                Irp as *mut core::ffi::c_void,
                0,
                1,
                1,
            );
        }

        NextIrpSp = ntkrnl_sys::ddk::IoGetNextIrpStackLocation(AssocIrp);

        SetFlag((addr_of_mut!((*NextIrpSp).Flags)).cast() , SL_OVERRIDE_VERIFY_VOLUME);

        (*NextIrpSp).MajorFunction = (*IrpSp).MajorFunction;
        (*NextIrpSp).Parameters.Read.Length = NextByteCount;
        (*NextIrpSp).Parameters.Read.ByteOffset.QuadPart = NextLbo as i64;

        IoCallDriver(DeviceObject, AssocIrp);

        if FatDiskAccountingEnabled != 0 {
            FatUpdateIOCountersPCW(IsAWrite, NextByteCount as u64);
        }

        if IrpIsMaster != 0 {
            ntkrnl_sys::ddk::KeWaitForSingleObject(
                (addr_of_mut!(Context.Event)).cast(),
                ntkrnl_sys::ddk::_KWAIT_REASON_Executive,
                0i8 /*KernelMode*/,
                0,
                null_mut(),
            );
            IrpIsMaster = 0;
            MdlIsReserve = 0;

            if !ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                if IrpCount == 0 {
                    ntkrnl_sys::ke::DbgBreakPoint();
                }
                break;
            }
        }

        Vbo = Vbo.wrapping_add(NextByteCount);
        BufferOffset = BufferOffset.wrapping_add(NextByteCount);
        ByteCount = ByteCount.wrapping_sub(NextByteCount);

        if RemainingByteCount != 0 {
            NextLbo = NextLbo.wrapping_add(NextByteCount as u64);
            NextVbo = NextVbo.wrapping_add(NextByteCount);
            NextByteCount = RemainingByteCount;
        } else {
            CurrentIndex = CurrentIndex.wrapping_add(1);

            if CurrentIndex <= LastIndex {
                if ByteCount == 0 {
                    ntkrnl_sys::ke::DbgBreakPoint();
                }

                FatGetNextMcbEntry(
                    (*Fcb).Vcb as *mut c_void,
                    addr_of_mut!((*Fcb).Mcb) as *mut c_void,
                    CurrentIndex,
                    addr_of_mut!(NextVbo),
                    addr_of_mut!(NextLbo),
                    addr_of_mut!(NextByteCount),
                );

                if NextVbo != Vbo {
                    ntkrnl_sys::ke::DbgBreakPoint();
                }
            }
        }
    }

    if IrpCount != 0 {
        while (*Irp).AssociatedIrp.IrpCount != IrpCount {
            let timeout = LARGE_INTEGER { QuadPart: -300000 };
            ntkrnl_sys::ddk::KeDelayExecutionThread(
                0i8 /*KernelMode*/,
                0,
                addr_of!(timeout) as *const _ as *mut _,
            );
        }

        ntkrnl_sys::driver_ddk::IoCompleteRequest(Irp, ntkrnl_sys::constants::IO_DISK_INCREMENT);
    }

    DebugTrace(-1, (Dbg as u32) as i32, c"FatPagingFileIo -> VOID\n".as_ptr() as *const i8, 0);
}


const IRP_MJ_READ: u8 = 3;
// fsrtltypes.h value; a wrong copy of these flags leaks a bogus top-level IRP that
// FatCommonWrite then dereferences (see resrcsup.rs FSRTL_CACHE_TOP_LEVEL_IRP).
const FSRTL_MOD_WRITE_TOP_LEVEL_IRP: *mut core::ffi::c_void = 0x03 as *mut core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn FatUpdateDiskStats(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    ByteCount: u32,
) {
    let mut originating_thread: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut num_reads: u32 = 0;
    let mut num_writes: u32 = 0;
    let mut bytes_to_read: u64 = 0;
    let mut bytes_to_write: u64 = 0;

    if (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction as u8 == IRP_MJ_READ {
        num_reads += 1;
        bytes_to_read = ByteCount as u64;
        if !(*(Irp as *mut _IRP)).Tail.Overlay.Thread.is_null()
            && (IoIsSystemThread((*(Irp as *mut _IRP)).Tail.Overlay.Thread) == 0)
        {
            originating_thread = (*(Irp as *mut _IRP)).Tail.Overlay.Thread as *mut core::ffi::c_void;
        } else if IoIsSystemThread(PsGetCurrentThread()) == 0 {
            originating_thread = PsGetCurrentThread() as *mut core::ffi::c_void;
        } else if IoIsSystemThread(PsGetCurrentThread()) != 0
            && (IoGetTopLevelIrp() == (Irp).cast())
        {
            originating_thread = PsGetCurrentThread() as *mut core::ffi::c_void;
        }
    } else if (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction as u8 == (IRP_MJ_WRITE) as u8 {
        num_writes += 1;
        bytes_to_write = ByteCount as u64;
        if IoGetTopLevelIrp() == (Irp).cast() {
            if !(*(Irp as *mut _IRP)).Tail.Overlay.Thread.is_null()
                && (IoIsSystemThread((*(Irp as *mut _IRP)).Tail.Overlay.Thread) == 0)
            {
                originating_thread = (*(Irp as *mut _IRP)).Tail.Overlay.Thread as *mut core::ffi::c_void;
            } else {
                originating_thread = PsGetCurrentThread() as *mut core::ffi::c_void;
            }
        } else if (IoGetTopLevelIrp() as *mut core::ffi::c_void) == FSRTL_MOD_WRITE_TOP_LEVEL_IRP {
            originating_thread = PsGetCurrentThread() as *mut core::ffi::c_void;
        }
    }

    if !originating_thread.is_null() {
        PsUpdateDiskCounters(
            PsGetThreadProcess(originating_thread as PETHREAD) as *mut core::ffi::c_void,
            bytes_to_read,
            bytes_to_write,
            num_reads,
            num_writes,
            0u32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatNonCachedIo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FcbOrDcb: *mut FCB,
    StartingVbo: u32,
    ByteCount: u32,
    UserByteCount: u32,
    StreamFlags: u32,
) -> i32 {
    let mut StartingVbo = StartingVbo;
    let mut ByteCount = ByteCount;
    PAGED_CODE!();

    // UNREFERENCED_PARAMETER(StreamFlags)
    let _ = StreamFlags;

    DebugTrace(1, (Dbg) as i32, b"FatNonCachedIo\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, (Dbg) as i32, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as *const core::ffi::c_void);
    DebugTrace(0, (Dbg) as i32, b"MajorFunction = %08lx\n\0".as_ptr() as *const i8, (*IrpContext).MajorFunction as u64);
    DebugTrace(0, (Dbg) as i32, b"FcbOrDcb      = %p\n\0".as_ptr() as *const i8, FcbOrDcb as *const core::ffi::c_void);
    DebugTrace(0, (Dbg) as i32, b"StartingVbo   = %08lx\n\0".as_ptr() as *const i8, StartingVbo as u64);
    DebugTrace(0, (Dbg) as i32, b"ByteCount     = %08lx\n\0".as_ptr() as *const i8, ByteCount as u64);

    if FlagOn((*Irp).Flags, IRP_PAGING_IO) == 0 {
        let vcb = (*FcbOrDcb).Vcb;
        let index = (KeGetCurrentProcessorNumber() as u32 % FatData.NumberProcessors) as usize;
        let stats = &mut *((*vcb).Statistics.add(index));
        let fat_stats = &mut stats.Fat;
        if (*IrpContext).MajorFunction == IRP_MJ_READ {
            fat_stats.NonCachedReads += 1;
            fat_stats.NonCachedReadBytes += (ByteCount as u64) as u32;
        } else {
            fat_stats.NonCachedWrites += 1;
            fat_stats.NonCachedWriteBytes += (ByteCount as u64) as u32;
        }
    }

    let mut NextRun: u32 = 0;
    let mut BufferOffset: u32 = 0;
    let OriginalByteCount: u32 = ByteCount;
    let Wait: u8 = (BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT)) as u8;

    if FatDiskAccountingEnabled != 0 {
        FatUpdateDiskStats(IrpContext, Irp, ByteCount);
    }

    FatLockUserBuffer(
        IrpContext,
        Irp,
        if (*IrpContext).MajorFunction == IRP_MJ_READ {
            IoWriteAccess as LOCK_OPERATION
        } else {
            IoReadAccess as LOCK_OPERATION
        },
        ByteCount,
    );

    if UserByteCount != ByteCount {
        NT_ASSERT((ByteCount > UserByteCount) as i32);
        // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

        let Mdl = IoAllocateMdl(
            ((*Irp).UserBuffer as *mut u8).add(UserByteCount as usize) as *mut c_void,
            ByteCount - UserByteCount,
            0,
            0,
            core::ptr::null_mut(),
        );
        if Mdl.is_null() {
            FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }
        IoBuildPartialMdl(
            (*Irp).MdlAddress,
            Mdl,
            ((*Irp).UserBuffer as *mut u8).add(UserByteCount as usize) as *mut c_void,
            ByteCount - UserByteCount,
        );
        (*(*IrpContext).FatIoContext).ZeroMdl = Mdl;

        if MmGetSystemAddressForMdlSafe(
            Mdl,
            NormalPagePriority | MdlMappingNoExecute,
        ).is_null() {
            FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }
    }

    let mut NextLbo: u64 = 0;
    let mut NextByteCount: u32 = 0;
    let mut NextIsAllocated: u8 = 0;
    let mut EndOnMax: u8 = 0;
    let mut FirstIndex: u32 = 0;
    FatLookupFileAllocation(
        IrpContext,
        FcbOrDcb,
        StartingVbo,
        &mut NextLbo as *mut u64 as *mut i64,
        &mut NextByteCount as *mut u32,
        &mut NextIsAllocated as *mut u8,
        &mut EndOnMax as *mut u8,
        &mut FirstIndex as *mut u32 as *mut c_void,
    );

    if NextIsAllocated == 0 {
        FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
        FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }
    NT_ASSERT((NextByteCount != 0) as i32);

    // Single run case
    if NextByteCount >= ByteCount - (if EndOnMax != 0 { 1 } else { 0 }) {
        if FlagOn((*Irp).Flags, IRP_PAGING_IO) != 0 {
            CollectDiskIoStats(
                (*FcbOrDcb).Vcb,
                (*IrpContext).MajorFunction as u32,
                FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_USER_IO) as u8,
                1,
            );
        } else {
            let vcb = (*FcbOrDcb).Vcb;
            let index = (KeGetCurrentProcessorNumber() as u32 % FatData.NumberProcessors) as usize;
            let stats = &mut *((*vcb).Statistics.add(index));
            if (*IrpContext).MajorFunction == IRP_MJ_READ {
                stats.Fat.NonCachedDiskReads += 1;
            } else {
                stats.Fat.NonCachedDiskWrites += 1;
            }
        }

        DebugTrace(0, (Dbg) as i32, b"Passing 1 Irp on to Disk Driver\n\0".as_ptr() as *const i8, 0);


        FatSingleAsync(IrpContext, (*FcbOrDcb).Vcb, NextLbo as i64, ByteCount, Irp);
    } else {
        let mut LastLbo: u64 = 0;
        let mut LastByteCount: u32 = 0;
        let mut LastIsAllocated: u8 = 0;
        let mut LastIndex: u32 = 0;
        FatLookupFileAllocation(
            IrpContext,
            FcbOrDcb,
            StartingVbo + ByteCount - 1,
            &mut LastLbo as *mut u64 as *mut i64,
            &mut LastByteCount as *mut u32,
            &mut LastIsAllocated as *mut u8,
            &mut EndOnMax as *mut u8,
            &mut LastIndex as *mut u32 as *mut c_void,
        );

        if LastIsAllocated == 0 {
            FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
            FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        let NeededCount = (LastIndex - FirstIndex + 1) as usize;
        let mut stack_io_runs: [IO_RUN; FAT_MAX_IO_RUNS_ON_STACK] = core::mem::zeroed();
        let mut io_runs_ptr: *mut IO_RUN;
        if NeededCount > FAT_MAX_IO_RUNS_ON_STACK {
            io_runs_ptr = FsRtlAllocatePoolWithTag(
                PagedPool,
                (NeededCount * core::mem::size_of::<IO_RUN>()) as u32,
                TAG_IO_RUNS,
            ) as *mut IO_RUN;
        } else {
            io_runs_ptr = stack_io_runs.as_mut_ptr();
        }

        NT_ASSERT((LastIndex != FirstIndex) as i32);

        let mut CurrentIndex = FirstIndex;
        while CurrentIndex <= LastIndex {
            NT_ASSERT((NextByteCount != 0) as i32);
            NT_ASSERT((ByteCount != 0) as i32);

            if NextByteCount > ByteCount {
                NextByteCount = ByteCount;
            }

            let run_ptr = io_runs_ptr.add(NextRun as usize);
            (*run_ptr).Vbo = StartingVbo;
            (*run_ptr).Lbo = NextLbo as i64;
            (*run_ptr).Offset = BufferOffset;
            (*run_ptr).ByteCount = NextByteCount;
            NextRun += 1;

            StartingVbo += NextByteCount;
            BufferOffset += NextByteCount;
            ByteCount -= NextByteCount;

            CurrentIndex += 1;
            if CurrentIndex <= LastIndex {
                NT_ASSERT((ByteCount != 0) as i32);

                let mut NextVbo: u32 = 0; // placeholder, value unused
                FatGetNextMcbEntry(
                    (*FcbOrDcb).Vcb as *mut c_void,
                    &mut (*FcbOrDcb).Mcb as *mut _ as *mut c_void,
                    CurrentIndex,
                    &mut NextVbo as *mut u32,
                    &mut NextLbo as *mut u64,
                    &mut NextByteCount as *mut u32,
                );

                // NextVbo == StartingVbo assertion is assumed satisfied by MCB structure;
                // we skip the assert because we don't have NextVbo variable.
                // NT_ASSERT(NextVbo == StartingVbo);
            }
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = OriginalByteCount as _;

        if FlagOn((*Irp).Flags, IRP_PAGING_IO) != 0 {
            CollectDiskIoStats(
                (*FcbOrDcb).Vcb,
                (*IrpContext).MajorFunction as u32,
                FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_USER_IO) as u8,
                NextRun,
            );
        }

        let p_stack_io_runs = stack_io_runs.as_mut_ptr();
        crate::seh::try_finally(
            &mut (),
            |_| {
                DebugTrace(0, (Dbg) as i32, b"Passing Multiple Irps on to Disk Driver\n\0".as_ptr() as *const i8, 0);
                FatMultipleAsync(IrpContext, ((*FcbOrDcb).Vcb).cast(), Irp, NextRun, io_runs_ptr);
            },
            |_, _abnormal| {
                if io_runs_ptr != p_stack_io_runs {
                    ExFreePool(io_runs_ptr as *mut core::ffi::c_void);
                }
            },
        );
    }

    if Wait == 0 {
        DebugTrace(-1, (Dbg) as i32, b"FatNonCachedIo -> STATUS_PENDING\n\0".as_ptr() as *const i8, 0);
        return STATUS_PENDING;
    }

    FatWaitSync(IrpContext);

    DebugTrace(
        -1,
        (Dbg) as i32,
        b"FatNonCachedIo -> 0x%08lx\n\0".as_ptr() as *const i8,
        (*Irp).IoStatus.__bindgen_anon_1.Status as u64,
    );

    (*Irp).IoStatus.__bindgen_anon_1.Status
}


#[no_mangle]
pub unsafe extern "system" fn FatNonCachedNonAlignedRead(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FcbOrDcb: *mut FCB,
    StartingVbo: u32,
    ByteCount: u32,
) {
    let mut StartingVbo = StartingVbo;
    let mut ByteCount = ByteCount;
    let mut NextLbo: i64 = 0;
    let mut NextByteCount: u32 = 0;
    let mut NextIsAllocated: u8 = 0;
    let mut SectorSize: u32;
    let mut BytesToCopy: u32 = 0;
    let mut OriginalByteCount: u32;
    let mut OriginalStartingVbo: u32;
    let mut EndOnMax: u8 = 0;
    let mut UserBuffer: *mut u8;
    let mut DiskBuffer: *mut u8 = core::ptr::null_mut();
    let mut Mdl: *mut MDL = core::ptr::null_mut();
    let mut SavedMdl: *mut MDL = core::ptr::null_mut();
    let mut SavedUserBuffer: *mut c_void = core::ptr::null_mut();

    // PAGED_CODE();
    PAGED_CODE!();

    DebugTrace(1, (Dbg) as i32, b"FatNonCachedNonAlignedRead\n\0".as_ptr() as *const i8, 0usize);
    DebugTrace(0, (Dbg) as i32, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as usize);
    DebugTrace(0, (Dbg) as i32, b"MajorFunction = %08lx\n\0".as_ptr() as *const i8, (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction as usize);
    DebugTrace(0, (Dbg) as i32, b"FcbOrDcb      = %p\n\0".as_ptr() as *const i8, FcbOrDcb as usize);
    DebugTrace(0, (Dbg) as i32, b"StartingVbo   = %08lx\n\0".as_ptr() as *const i8, StartingVbo as usize);
    DebugTrace(0, (Dbg) as i32, b"ByteCount     = %08lx\n\0".as_ptr() as *const i8, ByteCount as usize);

    OriginalByteCount = ByteCount;
    OriginalStartingVbo = StartingVbo;
    SectorSize = ((*(*FcbOrDcb).Vcb).Bpb.BytesPerSector) as u32;

    NT_ASSERT((FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WAIT) != 0) as i32);

    FatLockUserBuffer(IrpContext, Irp, IoWriteAccess as LOCK_OPERATION, ByteCount);
    UserBuffer = FatMapUserBuffer(IrpContext, Irp) as *mut u8;

    DiskBuffer = FsRtlAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        ROUND_TO_PAGES((SectorSize) as usize) as u32,
        TAG_IO_BUFFER,
    ) as *mut u8;

    crate::seh::try_finally(
        &mut (),
        |_| {
            'try_exit: {
                    // Beginning alignment handling
                    if (StartingVbo & (SectorSize - 1)) != 0 {
                        let mut Hole: u32;

                        FatLookupFileAllocation(
                            IrpContext,
                            FcbOrDcb,
                            StartingVbo,
                            core::ptr::addr_of_mut!(NextLbo),
                            core::ptr::addr_of_mut!(NextByteCount),
                            core::ptr::addr_of_mut!(NextIsAllocated),
                            core::ptr::addr_of_mut!(EndOnMax),
                            core::ptr::null_mut(),
                        );

                        if NextIsAllocated == 0 {
                            FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
                            FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR as i32);
                        }

                        FatSingleNonAlignedSync(
                            IrpContext,
                            ((*FcbOrDcb).Vcb).cast(),
                            DiskBuffer,
                            NextLbo & !((SectorSize as i64) - 1),
                            SectorSize,
                            Irp,
                        );

                        if !NT_SUCCESS((*(Irp as *mut _IRP)).IoStatus.__bindgen_anon_1.Status) {
                            break 'try_exit;
                        }

                        Hole = StartingVbo & (SectorSize - 1);
                        BytesToCopy = if ByteCount >= SectorSize - Hole {
                            SectorSize - Hole
                        } else {
                            ByteCount
                        };

                        core::ptr::copy_nonoverlapping(
                            DiskBuffer.add(Hole as usize),
                            UserBuffer,
                            BytesToCopy as usize,
                        );

                        StartingVbo += BytesToCopy;
                        ByteCount -= BytesToCopy;

                        if ByteCount == 0 {
                            break 'try_exit;
                        }
                    }

                    // Assert sector alignment of remaining range start
                    NT_ASSERT(((StartingVbo & (SectorSize - 1)) == 0) as i32);

                    // Tail alignment handling
                    if (ByteCount & (SectorSize - 1)) != 0 {
                        let LastSectorVbo: u32 = StartingVbo + (ByteCount & !(SectorSize - 1));

                        FatLookupFileAllocation(
                            IrpContext,
                            FcbOrDcb,
                            LastSectorVbo,
                            core::ptr::addr_of_mut!(NextLbo),
                            core::ptr::addr_of_mut!(NextByteCount),
                            core::ptr::addr_of_mut!(NextIsAllocated),
                            core::ptr::addr_of_mut!(EndOnMax),
                            core::ptr::null_mut(),
                        );

                        if NextIsAllocated == 0 {
                            FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
                            FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR as i32);
                        }

                        FatSingleNonAlignedSync(
                            IrpContext,
                            ((*FcbOrDcb).Vcb).cast(),
                            DiskBuffer,
                            NextLbo,
                            SectorSize,
                            Irp,
                        );

                        if !NT_SUCCESS((*(Irp as *mut _IRP)).IoStatus.__bindgen_anon_1.Status) {
                            break 'try_exit;
                        }

                        BytesToCopy = ByteCount & (SectorSize - 1);
                        UserBuffer = UserBuffer.add((LastSectorVbo - OriginalStartingVbo) as usize);
                        core::ptr::copy_nonoverlapping(
                            DiskBuffer,
                            UserBuffer,
                            BytesToCopy as usize,
                        );

                        ByteCount -= BytesToCopy;
                        if ByteCount == 0 {
                            break 'try_exit;
                        }
                    }

                    // Assert both start and count are sector aligned
                    NT_ASSERT((((StartingVbo | ByteCount) & (SectorSize - 1)) == 0) as i32);

                    SavedMdl = ((*(Irp as *mut _IRP)).MdlAddress).cast();
                    (*(Irp as *mut _IRP)).MdlAddress = core::ptr::null_mut();
                    SavedUserBuffer = (*(Irp as *mut _IRP)).UserBuffer;
                    (*(Irp as *mut _IRP)).UserBuffer = (MmGetMdlVirtualAddress(SavedMdl) as *mut u8)
                        .add((StartingVbo - OriginalStartingVbo) as usize) as *mut c_void;

                    Mdl = IoAllocateMdl((*(Irp as *mut _IRP)).UserBuffer, ByteCount, 0, 0, (Irp).cast()).cast() ;
                    if Mdl.is_null() {
                        (*(Irp as *mut _IRP)).MdlAddress = (SavedMdl).cast();
                        (*(Irp as *mut _IRP)).UserBuffer = SavedUserBuffer;
                        FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES as i32);
                    }

                    IoBuildPartialMdl(SavedMdl, Mdl, (*(Irp as *mut _IRP)).UserBuffer, ByteCount);

                    crate::seh::try_finally(
                        &mut (),
                        |_| {
                            FatNonCachedIo(IrpContext, Irp, FcbOrDcb, StartingVbo, ByteCount, ByteCount, 0);
                        },
                        |_, _abnormal| {
                            IoFreeMdl((*(Irp as *mut _IRP)).MdlAddress);
                            (*(Irp as *mut _IRP)).MdlAddress = (SavedMdl).cast();
                            (*(Irp as *mut _IRP)).UserBuffer = SavedUserBuffer;
                        },
                    );
            }
        },
        |_, abnormal| {
            ExFreePool(DiskBuffer as *mut c_void);

            if !abnormal && NT_SUCCESS((*(Irp as *mut _IRP)).IoStatus.__bindgen_anon_1.Status) {
                (*(Irp as *mut _IRP)).IoStatus.Information = (OriginalByteCount as u32) as _;
                KeFlushIoBuffers(((*(Irp as *mut _IRP)).MdlAddress).cast(), 1, 0);
            }
        },
    );

    DebugTrace(-1, (Dbg) as i32, b"FatNonCachedNonAlignedRead -> VOID\n\0".as_ptr() as *const i8, 0usize);
}

#[no_mangle]
pub unsafe extern "system" fn FatMultipleAsync(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    MasterIrp: *mut IRP,
    MultipleIrpCount: u32,
    IoRuns: *mut IO_RUN,
) {
    // local variables
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IrpSp: PIO_STACK_LOCATION;
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut Wait: i32;               /* BOOLEAN */
    let mut Context: PFAT_IO_CONTEXT;
    let mut IsAWrite: i32 = 0;       /* BOOLEAN */
    let mut Length: u32 = 0;
    let mut UnwindRunCount: u32 = 0;
    let mut ExceptionExpected: i32 = 1; // TRUE

    // PAGED_CODE(); // not needed in translation

    DebugTrace(1, 0, b"FatMultipleAsync\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, 0, b"MajorFunction    = %08lx\n\0".as_ptr() as *const i8, (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction as u32);
    DebugTrace(0, 0, b"Vcb              = %p\n\0".as_ptr() as *const i8, Vcb);
    DebugTrace(0, 0, b"MasterIrp        = %p\n\0".as_ptr() as *const i8, MasterIrp);
    DebugTrace(0, 0, b"MultipleIrpCount = %08lx\n\0".as_ptr() as *const i8, MultipleIrpCount);
    DebugTrace(0, 0, b"IoRuns           = %08lx\n\0".as_ptr() as *const i8, IoRuns);

    // If this I/O originating during FatVerifyVolume, bypass the verify logic.
    if (*(Vcb as *mut _VCB)).VerifyThread == KeGetCurrentThread() as *mut _ {
        SetFlag(addr_of_mut!((*(IrpContext as *mut _IRP_CONTEXT)).Flags), IRP_CONTEXT_FLAG_OVERRIDE_VERIFY);
    }

    // Set up things according to whether this is truly async.
    Wait = BooleanFlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WAIT) as i32;
    Context = (*(IrpContext as *mut _IRP_CONTEXT)).FatIoContext;

    // Finish initializing Context, for use in Read/Write Multiple Asynch.
    (*(Context as *mut _FAT_IO_CONTEXT)).MasterIrp = MasterIrp;

    IrpSp = IoGetCurrentIrpStackLocation(MasterIrp as *mut _IRP);
    IsAWrite = ((*(IrpSp as *mut _IO_STACK_LOCATION)).MajorFunction == (IRP_MJ_WRITE) as u8) as i32;
    Length = (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.Length;

    // _SEH2_TRY { .. } _SEH2_FINALLY { unwind the associated Irps on a raise }
    let p_unwind_run_count = addr_of_mut!(UnwindRunCount);
    let p_exception_expected = addr_of_mut!(ExceptionExpected);
    crate::seh::try_finally(
        &mut (),
        |_| {
        // Iterate through the runs, doing everything that can fail
        (*p_unwind_run_count) = 0;
        while (*p_unwind_run_count) < MultipleIrpCount {
            // Create an associated IRP, making sure there is one stack entry for us, as well.
            (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).SavedIrp = core::ptr::null_mut::<IRP>();
            Irp = IoMakeAssociatedIrp(
                MasterIrp as *mut _IRP,
                ((*(*((Vcb as *mut _VCB))).TargetDeviceObject).StackSize + 1) as i8,
            );
            if Irp.is_null() {
                FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
            }
            (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).SavedIrp = Irp;

            // Allocate and build a partial Mdl for the request.
            Mdl = IoAllocateMdl(
                ((*(MasterIrp as *mut _IRP)).UserBuffer as *mut u8).offset((*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).Offset as isize) as *mut core::ffi::c_void,
                (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).ByteCount,
                0,   // FALSE
                0,   // FALSE
                Irp,
            );
            if Mdl.is_null() {
                FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
            }

            // Sanity Check (NT_ASSERT)
            // NT_ASSERT( Mdl == Irp->MdlAddress );

            IoBuildPartialMdl(
                (*(MasterIrp as *mut _IRP)).MdlAddress,
                Mdl,
                ((*(MasterIrp as *mut _IRP)).UserBuffer as *mut u8).offset((*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).Offset as isize) as *mut core::ffi::c_void,
                (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).ByteCount,
            );

            // Get the first IRP stack location in the associated Irp
            IoSetNextIrpStackLocation(Irp);
            IrpSp = IoGetCurrentIrpStackLocation(Irp);

            // Setup the Stack location to describe our read.
            (*(IrpSp as *mut _IO_STACK_LOCATION)).MajorFunction = (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction;
            (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.Length = (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).ByteCount;
            (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.ByteOffset.QuadPart = (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).Vbo as i64;

            // Set up the completion routine
            IoSetCompletionRoutine(
                Irp,
                if Wait != 0 {
                    as_completion_routine(FatMultiSyncCompletionRoutine)
                } else {
                    as_completion_routine(FatMultiAsyncCompletionRoutine)
                },
                Context as *mut core::ffi::c_void,
                1, // TRUE
                1, // TRUE
                1, // TRUE
            );

            // Setup the next IRP stack location in the associated Irp for the disk driver beneath us.
            IrpSp = IoGetNextIrpStackLocation(Irp);

            // Setup the Stack location to do a read from the disk driver.
            (*(IrpSp as *mut _IO_STACK_LOCATION)).MajorFunction = (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction;
            (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.Length = (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).ByteCount;
            (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.ByteOffset.QuadPart = (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).Lbo;

            // If this Irp is the result of a WriteThrough operation, tell the device to write it through.
            if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH) != 0 {
                (*(IrpSp as *mut _IO_STACK_LOCATION)).Flags |= SL_WRITE_THROUGH as u8;
            }

            // If this I/O requires override verify, bypass the verify logic.
            if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY) != 0 {
                (*(IrpSp as *mut _IO_STACK_LOCATION)).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;
            }

            (*p_unwind_run_count) += 1;
        }

        // Now we no longer expect an exception. If the driver raises, we must bugcheck.
        (*p_exception_expected) = 0;

        // We only need to set the associated IRP count in the master irp to make it a master IRP.
        // But we set the count to one more than our caller requested, because we do not want the I/O system
        // to complete the I/O. We also set our own count.
        (*(Context as *mut _FAT_IO_CONTEXT)).IrpCount = MultipleIrpCount as i32;
        (*(MasterIrp as *mut _IRP)).AssociatedIrp.IrpCount = (MultipleIrpCount) as i32;

        if Wait != 0 {
            (*(MasterIrp as *mut _IRP)).AssociatedIrp.IrpCount += 1;
        } else if FlagOn((*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().ResourceThreadId as u32, 3) != 0 {
            // For async requests if we acquired locks, transition the lock owners to an object,
            // since when we return this thread could go away before request completion, and the
            // resource package may try to boost priority.
            if !(*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().Resource.is_null() {
                ExSetResourceOwnerPointer(
                    (*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().Resource,
                    (*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().ResourceThreadId as *mut core::ffi::c_void,
                );
            }
            if !(*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().Resource2.is_null() {
                ExSetResourceOwnerPointer(
                    (*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().Resource2,
                    (*(Context as *mut _FAT_IO_CONTEXT)).Wait.Async.as_ref().ResourceThreadId as *mut core::ffi::c_void,
                );
            }
        }

        // Back up a copy of the IrpContext flags for later use in async completion.
        (*(Context as *mut _FAT_IO_CONTEXT)).IrpContextFlags = (*(IrpContext as *mut _IRP_CONTEXT)).Flags;

        // Now that all the dangerous work is done, issue the read requests
        (*p_unwind_run_count) = 0;
        while (*p_unwind_run_count) < MultipleIrpCount {
            Irp = (*(IoRuns as *mut IO_RUN).offset((*p_unwind_run_count) as isize)).SavedIrp;
            DebugDoit({ FatIoCallDriverCount += 1; });

            // If IoCallDriver returns an error, it has completed the Irp and the error will be
            // caught by our completion routines and dealt with as a normal IO error.
            FatLowLevelReadWrite(
                IrpContext,
                (*(Vcb as *mut _VCB)).TargetDeviceObject,
                Irp,
                Vcb,
            );

            (*p_unwind_run_count) += 1;
        }

        // We just issued an IO to the storage stack, update the counters indicating so.
        if FatDiskAccountingEnabled != 0 {
            FatUpdateIOCountersPCW(IsAWrite as u32, Length as u64);
        }
        },
        |_, abnormal| {
            DebugUnwind(FatMultipleAsync as *const core::ffi::c_void);

            if abnormal {
                // If the driver raised, we are hosed. He is not supposed to raise, and it is impossible
                // for us to figure out how to clean up.
                if *p_exception_expected == 0 {
                    // NT_ASSERT( ExceptionExpected );
                    FatBugCheck(0, 0, 0);
                }

                // Unwind: free resources of runs up to and including the failed one
                let mut i: u32 = 0;
                while i <= *p_unwind_run_count {
                    let Irp: PIRP = (*(IoRuns as *mut IO_RUN).offset(i as isize)).SavedIrp;
                    if !Irp.is_null() {
                        if !(*Irp).MdlAddress.is_null() {
                            IoFreeMdl((*Irp).MdlAddress);
                        }
                        IoFreeIrp(Irp);
                    }
                    i += 1;
                }
            }

            // And return to our caller
            DebugTrace(-1, 0, b"FatMultipleAsync -> VOID\n\0".as_ptr() as *const i8, 0);
        },
    );
    // _SEH2_END
    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatSingleAsync(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Lbo: LBO,
    ByteCount: u32,
    Irp: *mut IRP,
) {
    // PAGED_CODE();
    let IrpSp: *mut _IO_STACK_LOCATION;
    let Context: *mut _FAT_IO_CONTEXT;
    let mut IsAWrite: i32 = 0;

    DebugTrace(1, (Dbg) as i32, b"FatSingleAsync\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, (Dbg) as i32, b"MajorFunction = %08lx\n\0".as_ptr() as *const i8, (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction as u32);
    DebugTrace(0, (Dbg) as i32, b"Vcb           = %p\n\0".as_ptr() as *const i8, Vcb as *const core::ffi::c_void);
    DebugTrace(0, (Dbg) as i32, b"Lbo           = %08lx\n\0".as_ptr() as *const i8, Lbo);
    DebugTrace(0, (Dbg) as i32, b"ByteCount     = %08lx\n\0".as_ptr() as *const i8, ByteCount);
    DebugTrace(0, (Dbg) as i32, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as *const core::ffi::c_void);

    if (*(Vcb as *mut _VCB)).VerifyThread == KeGetCurrentThread() as *mut _ {
        SetFlag(addr_of_mut!((*(IrpContext as *mut _IRP_CONTEXT)).Flags), IRP_CONTEXT_FLAG_OVERRIDE_VERIFY);
    }

    let completion_routine: Option<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, *mut core::ffi::c_void) -> NTSTATUS> =
        if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WAIT) != 0 {
            Some(FatSingleSyncCompletionRoutine)
        } else {
            Some(FatSingleAsyncCompletionRoutine)
        };

    IrpSp = IoGetNextIrpStackLocation((Irp).cast()) as *mut _IO_STACK_LOCATION;

    // Expand IoSetCompletionRoutine macro
    (*(IrpSp as *mut _IO_STACK_LOCATION)).CompletionRoutine = core::mem::transmute(completion_routine);
    (*(IrpSp as *mut _IO_STACK_LOCATION)).Context = (*(IrpContext as *mut _IRP_CONTEXT)).FatIoContext as *mut core::ffi::c_void;
    (*(IrpSp as *mut _IO_STACK_LOCATION)).Control = SL_INVOKE_ON_SUCCESS | SL_INVOKE_ON_ERROR | SL_INVOKE_ON_CANCEL;

    (*(IrpSp as *mut _IO_STACK_LOCATION)).MajorFunction = (*(IrpContext as *mut _IRP_CONTEXT)).MajorFunction;
    (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.Length = ByteCount;
    (*(IrpSp as *mut _IO_STACK_LOCATION)).Parameters.Read.ByteOffset.QuadPart = Lbo;

    IsAWrite = (((*(IrpSp as *mut _IO_STACK_LOCATION)).MajorFunction as u32) == IRP_MJ_WRITE) as i32;

    if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH) != 0 {
        (*(IrpSp as *mut _IO_STACK_LOCATION)).Flags |= SL_WRITE_THROUGH as u8;
    }

    if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY) != 0 {
        (*(IrpSp as *mut _IO_STACK_LOCATION)).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;
    }

    if FlagOn((*(IrpContext as *mut _IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WAIT) == 0
        && FlagOn((*(*(IrpContext as *mut _IRP_CONTEXT)).FatIoContext).Wait.Async.as_ref().ResourceThreadId as u32, 3) != 0
    {
        Context = (*(IrpContext as *mut _IRP_CONTEXT)).FatIoContext as *mut _FAT_IO_CONTEXT;

        if !(*Context).Wait.Async.as_ref().Resource.is_null() {
            ExSetResourceOwnerPointer(
                (*Context).Wait.Async.as_ref().Resource,
                (*Context).Wait.Async.as_ref().ResourceThreadId as *mut core::ffi::c_void,
            );
        }
        if !(*Context).Wait.Async.as_ref().Resource2.is_null() {
            ExSetResourceOwnerPointer(
                (*Context).Wait.Async.as_ref().Resource2,
                (*Context).Wait.Async.as_ref().ResourceThreadId as *mut core::ffi::c_void,
            );
        }
    }

    (*(*(IrpContext as *mut _IRP_CONTEXT)).FatIoContext).IrpContextFlags = (*(IrpContext as *mut _IRP_CONTEXT)).Flags;

    let tmp = {
        FatIoCallDriverCount = FatIoCallDriverCount.wrapping_add(1);
        DebugDoit(());
        FatIoCallDriverCount
    };

    IoCallDriver((*(Vcb as *mut _VCB)).TargetDeviceObject, (Irp).cast());

    if FatDiskAccountingEnabled != 0 {
        FatUpdateIOCountersPCW(IsAWrite as u32, ByteCount as u64);
    }

    DebugTrace(-1, (Dbg) as i32, b"FatSingleAsync -> VOID\n\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatPagingFileErrorHandler(Irp: PIRP, Event: PKEVENT) -> NTSTATUS {
    // fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NonPagedPoolNx: u32 = 0;
    const FAT_TAG: u32 = 0x46617420; // ' taF' (LE bytes)

    let status: NTSTATUS;

    if FsRtlIsTotalDeviceFailure((*Irp).IoStatus.__bindgen_anon_1.Status) != 0 {
        status = STATUS_SUCCESS;
    } else {
        let packet: *mut CLEAN_AND_DIRTY_VOLUME_PACKET =
            ExAllocatePoolWithTag(
                NonPagedPoolNx,
                (core::mem::size_of::<CLEAN_AND_DIRTY_VOLUME_PACKET>() as u32) as usize,
                FAT_TAG,
            ) as *mut CLEAN_AND_DIRTY_VOLUME_PACKET;

        if !packet.is_null() {
            let irp_sp = IoGetCurrentIrpStackLocation(Irp);
            let device_obj = (*irp_sp).DeviceObject as *mut VOLUME_DEVICE_OBJECT;
            (*packet).Vcb = core::ptr::addr_of_mut!((*device_obj).Vcb);
            (*packet).Irp = Irp;
            (*packet).Event = Event;
            ExInitializeWorkItem(
                core::ptr::addr_of_mut!((*packet).Item),
                Some(core::mem::transmute::<*const (), _>(FatFspMarkVolumeDirtyWithRecover as *const ())),
                packet as PVOID,
            );
            ExQueueWorkItem(core::ptr::addr_of_mut!((*packet).Item), CriticalWorkQueue as WORK_QUEUE_TYPE);
            status = STATUS_MORE_PROCESSING_REQUIRED;
        } else {
            status = STATUS_SUCCESS;
        }
    }

    status
}


#[no_mangle]
pub unsafe extern "system" fn FatPagingFileCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MasterIrp: PVOID,
) -> NTSTATUS {

    unsafe {
        DebugTrace(
            1,
            (Dbg) as i32,
            b"FatPagingFileCompletionRoutine, MasterIrp = %p\n\0".as_ptr() as *const i8,
            MasterIrp,
        );

        NT_ASSERT((!NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status)) as i32);

        if (Irp as *mut core::ffi::c_void) != MasterIrp {
            let master = MasterIrp as *mut IRP;
            (*master).IoStatus = (*Irp).IoStatus;
        }

        DebugTrace(
            -1,
            (Dbg) as i32,
            b"FatPagingFileCompletionRoutine => (done)\n\0".as_ptr() as *const i8,
            0,
        );

        let _ = DeviceObject;

        FatPagingFileErrorHandler(Irp, core::ptr::null_mut())
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatSingleAsyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    const STATUS_SUCCESS_CONST: i32 = 0;
    const Dbg: u32 = DEBUG_TRACE_DEVIOSUP;

    let mut Status: NTSTATUS = STATUS_SUCCESS_CONST;
    let Context: *mut _FAT_IO_CONTEXT = Contxt as *mut _FAT_IO_CONTEXT;
    let mut PostRequest: i32 = 0;

    DebugTrace(1, Dbg as i32, b"FatSingleAsyncCompletionRoutine, Context = %p\n\0".as_ptr() as *const i8, Context);

    // FatDoCompletionZero expansion
    if !(*Context).ZeroMdl.is_null() {
        NT_ASSERT(
            (((*(*Context).ZeroMdl).MdlFlags & ((MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) as u32 as i16)) != 0) as i32
        );
        if ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
            core::ptr::write_bytes(
                (*(*Context).ZeroMdl).MappedSystemVa,
                0,
                (*(*Context).ZeroMdl).ByteCount as usize,
            );
        }
        IoFreeMdl((*Context).ZeroMdl);
        (*Context).ZeroMdl = core::ptr::null_mut();
    }

    if ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        NT_ASSERT(((*Irp).IoStatus.Information != 0) as i32);
        (*Irp).IoStatus.Information = (*Context).Wait.Async.as_ref().RequestedByteCount as _;
        NT_ASSERT(((*Irp).IoStatus.Information != 0) as i32);

        if FlagOn((*Irp).Flags, IRP_PAGING_IO) == 0 {
            let major_function = (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction;
            SetFlag(
                addr_of_mut!((*(*Context).Wait.Async.as_ref().FileObject).Flags),
                if major_function == IRP_MJ_READ as u8 {
                    FO_FILE_FAST_IO_READ
                } else {
                    FO_FILE_MODIFIED
                },
            );
        }
    } else {
        #[cfg(debug_assertions)]
        {
            if !(ntkrnl_sys::base::NT_SUCCESS(FatBreakOnInterestingIoCompletion)
                || (*Irp).IoStatus.__bindgen_anon_1.Status != FatBreakOnInterestingIoCompletion)
            {
                DbgBreakPoint();
            }
        }

        if FlagOn((*Context).IrpContextFlags, crate::types::IRP_CONTEXT_FLAG_RECURSIVE_CALL) == 0
            && (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED
        {
            PostRequest = 1;
        }
    }

    // _Analysis_assume_: assert non-null path is analysed statically (no-op at runtime)
    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

    if !(*Context).Wait.Async.as_ref().NonPagedFcb.is_null()
        && ExInterlockedAddUlong(
            addr_of_mut!((*(*Context).Wait.Async.as_ref().NonPagedFcb).OutstandingAsyncWrites),
            0xffffffff,
            addr_of_mut!(FatData.GeneralSpinLock),
        ) == 1
    {
        KeSetEvent((*(*Context).Wait.Async.as_ref().NonPagedFcb).OutstandingAsyncEvent as PVOID, 0, 0);
    }

    if !(*Context).Wait.Async.as_ref().Resource.is_null() {
        ExReleaseResourceForThreadLite(
            (*Context).Wait.Async.as_ref().Resource,
            (*Context).Wait.Async.as_ref().ResourceThreadId,
        );
    }

    if !(*Context).Wait.Async.as_ref().Resource2.is_null() {
        ExReleaseResourceForThreadLite(
            (*Context).Wait.Async.as_ref().Resource2,
            (*Context).Wait.Async.as_ref().ResourceThreadId,
        );
    }

    IoMarkIrpPending(Irp);

    ExFreePool(Context as *mut c_void);

    if PostRequest != 0 {
        // _SEH2_EXCEPT(FatExceptionFilter(NULL, ..)) { NOTHING }: if we failed
        // to post the IRP, we just have to return the failure to the user.
        let NoIrpContext: *mut crate::types::_IRP_CONTEXT = core::ptr::null_mut();
        let p_status = addr_of_mut!(Status);
        let _ = crate::seh::fat_try_except(addr_of!(NoIrpContext), || {
            let IrpContext = FatCreateIrpContext(Irp, 1);
            ClearFlag(addr_of_mut!((*IrpContext).Flags), crate::types::IRP_CONTEXT_FLAG_RECURSIVE_CALL);
            FatFsdPostRequest(IrpContext, Irp);
            *p_status = STATUS_MORE_PROCESSING_REQUIRED;
        });
    }

    DebugTrace(
        -1,
        Dbg as i32,
        b"FatSingleAsyncCompletionRoutine -> STATUS_MORE_PROCESSING_REQUIRED\n\0".as_ptr() as *const i8,
        0,
    );

    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatLockUserBuffer(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Operation: LOCK_OPERATION,
    BufferLength: u32,
) {
    let mut Mdl: *mut MDL;
    unsafe {
        Mdl = core::ptr::null_mut();

        if (*Irp).MdlAddress.is_null() {
            Mdl = IoAllocateMdl(
                (*Irp).UserBuffer,
                BufferLength,
                0,
                0,
                Irp,
            );
            if Mdl.is_null() {
                FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
            }

            let caught = crate::seh::try_except(
                &mut (),
                |_| {
                    MmProbeAndLockPages(Mdl, (*Irp).RequestorMode as _, Operation as _);
                },
                |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER,
            );
            if let Err(status) = caught {
                IoFreeMdl(Mdl);
                (*Irp).MdlAddress = core::ptr::null_mut();
                FatRaiseStatus(
                    IrpContext,
                    if FsRtlIsNtstatusExpected(status) != 0 {
                        status
                    } else {
                        STATUS_INVALID_USER_BUFFER
                    },
                );
            }
        }

        let _ = IrpContext;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatMapUserBuffer(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> *mut core::ffi::c_void {
    let _ = IrpContext;
    PAGED_CODE!();
    let irp = &*(Irp as *mut _IRP);
    if irp.MdlAddress.is_null() {
        return irp.UserBuffer;
    }
    let Address = MmGetSystemAddressForMdlSafe(irp.MdlAddress, NormalPagePriority | MdlMappingNoExecute);
    if Address.is_null() {
        ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
    }
    Address
}

#[no_mangle]
pub unsafe extern "system" fn FatBufferUserBuffer(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    BufferLength: u32,
) -> *mut u8 {
    let mut UserBuffer: *mut u8;

    let _ = IrpContext;

    if BufferLength == 0 {
        return core::ptr::null_mut();
    }

    if (*Irp).AssociatedIrp.SystemBuffer == core::ptr::null_mut() {
        UserBuffer = FatMapUserBuffer(IrpContext, Irp).cast();

        (*Irp).AssociatedIrp.SystemBuffer = FsRtlAllocatePoolWithQuotaTag(
            NonPagedPoolNx,
            BufferLength,
            TAG_IO_USER_BUFFER,
        );

        (*Irp).Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;

        // RtlCopyMemory(SystemBuffer <- UserBuffer) under
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER). (The translation had the copy
        // reversed, overwriting the caller's buffer with uninitialized pool.)
        let SystemBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut u8;
        let caught = crate::seh::try_except(
            &mut (),
            |_| {
                core::ptr::copy_nonoverlapping(
                    UserBuffer as *const u8,
                    SystemBuffer,
                    BufferLength as usize,
                );
            },
            |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER,
        );
        if let Err(Status) = caught {
            FatRaiseStatus(
                IrpContext,
                if FsRtlIsNtstatusExpected(Status) != 0 {
                    Status
                } else {
                    STATUS_INVALID_USER_BUFFER
                },
            );
        }
    }

    return (*Irp).AssociatedIrp.SystemBuffer as *mut u8;
}

#[no_mangle]
pub unsafe extern "system" fn FatToggleMediaEjectDisable(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    PreventRemoval: u8,
) -> i32 {
    let Irp: *mut IRP;
    let mut SavedIrql: KIRQL = 0;
    let mut Status: i32;
    let mut SyncContext: FAT_SYNC_CONTEXT = core::mem::zeroed();
    let mut Prevent: PREVENT_MEDIA_REMOVAL = core::mem::zeroed();

    let _ = &IrpContext;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!(FatData.GeneralSpinLock)).cast(),
        &mut SavedIrql,
    );

    if (PreventRemoval ^ (BooleanFlagOn(((*(Vcb as *mut _VCB)).VcbState as u8) as u32, VCB_STATE_FLAG_REMOVAL_PREVENTED)) as u8) == 0 {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!(FatData.GeneralSpinLock)).cast(),
            SavedIrql,
        );
        return STATUS_SUCCESS;
    } else {
        (*(Vcb as *mut _VCB)).VcbState ^= VCB_STATE_FLAG_REMOVAL_PREVENTED;
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!(FatData.GeneralSpinLock)).cast(),
            SavedIrql,
        );
    }

    Prevent.PreventMediaRemoval = PreventRemoval;

    KeInitializeEvent(core::ptr::addr_of_mut!(SyncContext.Event), 0, 0);

    Irp = (IoBuildDeviceIoControlRequest(
        IOCTL_DISK_MEDIA_REMOVAL,
        (*(Vcb as *mut _VCB)).TargetDeviceObject,
        &mut Prevent as *mut PREVENT_MEDIA_REMOVAL as *mut c_void,
        core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as u32,
        core::ptr::null_mut(),
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )).cast();

    if !Irp.is_null() {
        (*(Irp as *mut _IRP)).UserIosb = &mut (*(Irp as *mut _IRP)).IoStatus;

        IoSetCompletionRoutine(
            Irp,
            as_completion_routine(FatSpecialSyncCompletionRoutine),
            &mut SyncContext as *mut FAT_SYNC_CONTEXT as *mut c_void,
            1,
            1,
            1,
        );

        Status = IoCallDriver((*(Vcb as *mut _VCB)).TargetDeviceObject, (Irp).cast());

        if Status == STATUS_PENDING {
            let _ = KeWaitForSingleObject(
                addr_of_mut!(SyncContext.Event) as *mut c_void,
                0u32 /*Executive*/,
                KernelMode,
                0,
                core::ptr::null_mut(),
            );
            Status = SyncContext.Iosb.__bindgen_anon_1.Status as i32;
        }

        return Status;
    }

    return STATUS_INSUFFICIENT_RESOURCES;
}

#[no_mangle]
pub unsafe extern "system" fn FatPerformDevIoCtrl(
    IrpContext: *mut core::ffi::c_void,
    IoControlCode: u32,
    Device: PDEVICE_OBJECT,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: u32,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: u32,
    InternalDeviceIoControl: u8,
    OverrideVerify: u8,
    Iosb: *mut _IO_STATUS_BLOCK,
) -> NTSTATUS {
    const NotificationEvent: u32 = 0;
    const Executive: u32 = 0;
    const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;

    let mut Status: NTSTATUS;
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut LocalIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IosbToUse: *mut _IO_STATUS_BLOCK = &mut LocalIosb as *mut _;

    // PAGED_CODE(); -- nothing to do in Rust
    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    if !Iosb.is_null() {
        IosbToUse = Iosb;
    }

    (*IosbToUse).__bindgen_anon_1.Status = 0;
    (*IosbToUse).Information = 0;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        Device,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        InternalDeviceIoControl,
        core::ptr::addr_of_mut!(Event),
        IosbToUse,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if OverrideVerify != 0 {
        let next_stack = IoGetNextIrpStackLocation(Irp);
        (*next_stack).Flags |= SL_OVERRIDE_VERIFY_VOLUME;
    }

    Status = IoCallDriver(Device, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            addr_of_mut!(Event) as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );
        Status = (*IosbToUse).__bindgen_anon_1.Status;
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatBuildZeroMdl(
    IrpContext: *mut IRP_CONTEXT,
    Length: u32,
) -> PMDL {
    let mut Length = Length;
    let mut zero_mdl: PMDL;
    let mut saved_byte_count: u32;
    let mut page: *mut ULONG_PTR; // PFN_NUMBER = ULONG_PTR
    let mut i: u32;

    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    // Spin down trying to get an MDL which can describe our operation.
    loop {
        zero_mdl = IoAllocateMdl(
            FatData.ZeroPage as *mut core::ffi::c_void,
            Length,
            0, // FALSE
            0, // FALSE
            core::ptr::null_mut(),
        );

        // Throttle ourselves to what we've physically allocated.
        if !zero_mdl.is_null() || Length <= (PAGE_SIZE) as u32 {
            break;
        }

        // Fallback by half and round down to a page multiple.
        ntkrnl_sys::ASSERT!((*(*IrpContext).Vcb).Bpb.BytesPerSector <= PAGE_SIZE as u16);
        Length = BlockAlignTruncate(Length / 2, (PAGE_SIZE) as u32);
        if Length < (PAGE_SIZE) as u32 {
            Length = (PAGE_SIZE) as u32;
        }
    }

    if zero_mdl.is_null() {
        return core::ptr::null_mut();
    }

    // If we have throttled all the way down, stop and just build a
    // simple MDL describing our previous allocation.
    if Length == (PAGE_SIZE) as u32 {
        MmBuildMdlForNonPagedPool(zero_mdl);
        return zero_mdl;
    }

    // Now we will temporarily lock the allocated pages only, and then
    // replicate the page frame numbers through the entire MDL to keep
    // writing the same pages of zeros.
    saved_byte_count = (*zero_mdl).ByteCount;
    (*zero_mdl).ByteCount = (PAGE_SIZE) as u32;
    MmBuildMdlForNonPagedPool(zero_mdl);

    (*zero_mdl).MdlFlags &= (!MDL_SOURCE_IS_NONPAGED_POOL) as i16;
    (*zero_mdl).MdlFlags |= (MDL_PAGES_LOCKED) as i16;
    (*zero_mdl).MappedSystemVa = core::ptr::null_mut();
    (*zero_mdl).StartVa = core::ptr::null_mut();
    (*zero_mdl).ByteCount = saved_byte_count;

    page = MmGetMdlPfnArray(zero_mdl);
    i = 1;
    let span = ADDRESS_AND_SIZE_TO_SPAN_PAGES(core::ptr::null_mut(), saved_byte_count as usize);
    while i < (span) as u32 {
        *page.add(i as usize) = *page;
        i += 1;
    }

    zero_mdl
}



// ==== re-stitched (genuine drop, authoritative) ====
#[no_mangle]
pub unsafe extern "system" fn FatWaitSync(IrpContext: *mut IRP_CONTEXT) {
    const Dbg: i32 = 0;
    const Executive: u32 = 0; // KWAIT_REASON::Executive
    // PAGED_CODE() omitted (no-op in free build)

    DebugTrace(
        1,
        Dbg,
        b"FatWaitSync, Context = %p\n\0".as_ptr() as *const i8,
        (*IrpContext).FatIoContext,
    );

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*((*IrpContext).FatIoContext)).Wait.SyncEvent) as *mut _,
        Executive,
        KernelMode,
        0,
        core::ptr::null_mut(),
    );

    KeClearEvent(
        core::ptr::addr_of_mut!((*((*IrpContext).FatIoContext)).Wait.SyncEvent) as *mut _,
    );

    DebugTrace(
        -1,
        Dbg,
        b"FatWaitSync -> VOID\n\0".as_ptr() as *const i8,
        0,
    );
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
//     DevIoSup.c
//
// Abstract:
//
//     This module implements the low lever disk read/write support for Fat.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_DEVIOSUP)
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_DEVIOSUP)
//
// #define CollectDiskIoStats(VCB,FUNCTION,IS_USER_IO,COUNT) {                                    \
//     PFILESYSTEM_STATISTICS Stats = &(VCB)->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors].Common;   \
//     if (IS_USER_IO) {                                                                          \
//         if ((FUNCTION) == IRP_MJ_WRITE) {                                                      \
//             Stats->UserDiskWrites += (COUNT);                                                  \
//         } else {                                                                               \
//             Stats->UserDiskReads += (COUNT);                                                   \
//         }                                                                                      \
//     } else {                                                                                   \
//         if ((FUNCTION) == IRP_MJ_WRITE) {                                                      \
//             Stats->MetaDataDiskWrites += (COUNT);                                              \
//         } else {                                                                               \
//             Stats->MetaDataDiskReads += (COUNT);                                               \
//         }                                                                                      \
//     }                                                                                          \
// }
//
// typedef struct _FAT_SYNC_CONTEXT {
//
//     //
//     //  Io status block for the request
//     //
//
//     IO_STATUS_BLOCK Iosb;
//
//     //
//     //  Event to be signaled when the request completes
//     //
//
//     KEVENT Event;
//
// } FAT_SYNC_CONTEXT, *PFAT_SYNC_CONTEXT;
//
//
// //
// // Completion Routine declarations
// //
//
// IO_COMPLETION_ROUTINE FatMultiSyncCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatMultiSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatMultiAsyncCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatMultiAsyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatSpecialSyncCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatSpecialSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatSingleSyncCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatSingleSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatSingleAsyncCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatSingleAsyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatPagingFileCompletionRoutine;
//
/* NTSTATUS */
// NTAPI
// FatPagingFileCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID MasterIrp
//     );
//
// IO_COMPLETION_ROUTINE FatPagingFileCompletionRoutineCatch;
//
/* NTSTATUS */
// NTAPI
// FatPagingFileCompletionRoutineCatch (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// VOID
// FatSingleNonAlignedSync (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PUCHAR Buffer,
//     IN LBO Lbo,
//     IN ULONG ByteCount,
//     IN PIRP Irp
//     );
//
// //
// //  The following macro decides whether to send a request directly to
// //  the device driver, or to other routines.  It was meant to
// //  replace IoCallDriver as transparently as possible.  It must only be
// //  called with a read or write Irp.
// //
// //  NTSTATUS
// //  FatLowLevelReadWrite (
// //      PIRP_CONTEXT IrpContext,
// //      PDEVICE_OBJECT DeviceObject,
// //      PIRP Irp,
// //      PVCB Vcb
// //      );
// //
//
// #define FatLowLevelReadWrite(IRPCONTEXT,DO,IRP,VCB) ( \
//     IoCallDriver((DO),(IRP))                          \
// )
//
// //
// //  The following macro handles completion-time zeroing of buffers.
// //
//
// #define FatDoCompletionZero( I, C )                                     \
//     if ((C)->ZeroMdl) {                                                 \
//         NT_ASSERT( (C)->ZeroMdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA |  \
//                                           MDL_SOURCE_IS_NONPAGED_POOL));\
//         if (NT_SUCCESS((I)->IoStatus.Status)) {                         \
//             RtlZeroMemory( (C)->ZeroMdl->MappedSystemVa,                \
//                            (C)->ZeroMdl->ByteCount );                   \
//         }                                                               \
//         IoFreeMdl((C)->ZeroMdl);                                        \
//         (C)->ZeroMdl = NULL;                                            \
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// #define FatUpdateIOCountersPCW(IsAWrite,Count)                          \
//     FsRtlUpdateDiskCounters( ((IsAWrite) ? 0       : (Count) ),         \
//                              ((IsAWrite) ? (Count) : 0) )
// #else
// #define FatUpdateIOCountersPCW(IsAWrite,Count)
// #endif
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatMultipleAsync)
// #pragma alloc_text(PAGE, FatSingleAsync)
// #pragma alloc_text(PAGE, FatSingleNonAlignedSync)
// #pragma alloc_text(PAGE, FatWaitSync)
// #pragma alloc_text(PAGE, FatLockUserBuffer)
// #pragma alloc_text(PAGE, FatBufferUserBuffer)
// #pragma alloc_text(PAGE, FatMapUserBuffer)
// #pragma alloc_text(PAGE, FatNonCachedIo)
// #pragma alloc_text(PAGE, FatNonCachedNonAlignedRead)
// #pragma alloc_text(PAGE, FatPerformDevIoCtrl)
// #endif
//
// typedef struct FAT_PAGING_FILE_CONTEXT {
//     KEVENT Event;
//     PMDL RestoreMdl;
// } FAT_PAGING_FILE_CONTEXT, *PFAT_PAGING_FILE_CONTEXT;
//
//
// VOID
// FatPagingFileIo (
//     IN PIRP Irp,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached disk io described in its parameters.
//     This routine nevers blocks, and should only be used with the paging
//     file since no completion processing is performed.
//
// Arguments:
//
//     Irp - Supplies the requesting Irp.
//
//     Fcb - Supplies the file to act on.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     //
//     // Declare some local variables for enumeration through the
//     // runs of the file.
//     //
//
//     VBO Vbo;
//     ULONG ByteCount;
//
//     PMDL Mdl;
//     LBO NextLbo;
//     VBO NextVbo = 0;
//     ULONG NextByteCount;
//     ULONG RemainingByteCount;
//     BOOLEAN MustSucceed;
//
//     ULONG FirstIndex;
//     ULONG CurrentIndex;
//     ULONG LastIndex;
//
//     LBO LastLbo;
//     ULONG LastByteCount;
//
//     BOOLEAN MdlIsReserve = FALSE;
//     BOOLEAN IrpIsMaster = FALSE;
//     FAT_PAGING_FILE_CONTEXT Context;
//     LONG IrpCount;
//
//     PIRP AssocIrp;
//     PIO_STACK_LOCATION IrpSp;
//     PIO_STACK_LOCATION NextIrpSp;
//     ULONG BufferOffset;
//     PDEVICE_OBJECT DeviceObject;
//
// #ifndef __REACTOS__
//     BOOLEAN IsAWrite = FALSE;
// #endif
//
//     DebugTrace(+1, Dbg, "FatPagingFileIo\n", 0);
//     DebugTrace( 0, Dbg, "Irp = %p\n", Irp );
//     DebugTrace( 0, Dbg, "Fcb = %p\n", Fcb );
//
//     NT_ASSERT( FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE ));
//
//     //
//     //  Initialize some locals.
//     //
//
//     BufferOffset = 0;
//     DeviceObject = Fcb->Vcb->TargetDeviceObject;
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     Vbo = IrpSp->Parameters.Read.ByteOffset.LowPart;
//     ByteCount = IrpSp->Parameters.Read.Length;
// #ifndef __REACTOS__
//     IsAWrite = (IrpSp->MajorFunction == IRP_MJ_WRITE);
// #endif
//
//     MustSucceed = FatLookupMcbEntry( Fcb->Vcb, &Fcb->Mcb,
//                                      Vbo,
//                                      &NextLbo,
//                                      &NextByteCount,
//                                      &FirstIndex);
//
//     //
//     //  If this run isn't present, something is very wrong.
//     //
//
//     if (!MustSucceed) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( Vbo, ByteCount, 0 );
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     //
//     //  Charge the IO to paging file to current thread
//     //
//
//     if (FatDiskAccountingEnabled) {
//
//         PETHREAD ThreadIssuingIo = PsGetCurrentThread();
//         BOOLEAN IsWriteOperation = FALSE;
//
//         if (IrpSp->MajorFunction == IRP_MJ_WRITE) {
//             IsWriteOperation = TRUE;
//         }
//
//         PsUpdateDiskCounters( PsGetThreadProcess( ThreadIssuingIo ),
//                               (IsWriteOperation ? 0 : ByteCount ),       // bytes to read
//                               (IsWriteOperation ? ByteCount : 0),       // bytes to write
//                               (IsWriteOperation ? 0 : 1),               // # of reads
//                               (IsWriteOperation ? 1 : 0),               // # of writes
//                               0 );
//     }
// #endif
//
//     // See if the write covers a single valid run, and if so pass
//     // it on.
//     //
//
//     if ( NextByteCount >= ByteCount ) {
//
//         DebugTrace( 0, Dbg, "Passing Irp on to Disk Driver\n", 0 );
//
//         //
//         //  Setup the next IRP stack location for the disk driver beneath us.
//         //
//
//         NextIrpSp = IoGetNextIrpStackLocation( Irp );
//
//         NextIrpSp->MajorFunction = IrpSp->MajorFunction;
//         NextIrpSp->Parameters.Read.Length = ByteCount;
//         NextIrpSp->Parameters.Read.ByteOffset.QuadPart = NextLbo;
//
//         //
//         //  Since this is Paging file IO, we'll just ignore the verify bit.
//         //
//
//         SetFlag( NextIrpSp->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//
//         //
//         //  Set up the completion routine address in our stack frame.
//         //  This is only invoked on error or cancel, and just copies
//         //  the error Status into master irp's iosb.
//         //
//         //  If the error implies a media problem, it also enqueues a
//         //  worker item to write out the dirty bit so that the next
//         //  time we run we will do a autochk /r
//         //
//
//         IoSetCompletionRoutine( Irp,
//                                 &FatPagingFileCompletionRoutine,
//                                 Irp,
//                                 FALSE,
//                                 TRUE,
//                                 TRUE );
//
//         //
//         //  Issue the read/write request
//         //
//         //  If IoCallDriver returns an error, it has completed the Irp
//         //  and the error will be dealt with as a normal IO error.
//         //
//
//         (VOID)IoCallDriver( DeviceObject, Irp );
//
//         //
//         //  We just issued an IO to the storage stack, update the counters indicating so.
//         //
//
//         if (FatDiskAccountingEnabled) {
//
//             FatUpdateIOCountersPCW( IsAWrite, ByteCount );
//         }
//
//         DebugTrace(-1, Dbg, "FatPagingFileIo -> VOID\n", 0);
//         return;
//     }
//
//     //
//     //  Find out how may runs there are.
//     //
//
//     MustSucceed = FatLookupMcbEntry( Fcb->Vcb, &Fcb->Mcb,
//                                      Vbo + ByteCount - 1,
//                                      &LastLbo,
//                                      &LastByteCount,
//                                      &LastIndex);
//
//     //
//     //  If this run isn't present, something is very wrong.
//     //
//
//     if (!MustSucceed) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( Vbo + ByteCount - 1, 1, 0 );
//     }
//
//     CurrentIndex = FirstIndex;
//
//     //
//     //  Now set up the Irp->IoStatus.  It will be modified by the
//     //  multi-completion routine in case of error or verify required.
//     //
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = ByteCount;
//
//     //
//     //  Loop while there are still byte writes to satisfy.  The way we'll work this
//     //  is to hope for the best - one associated IRP per run, which will let us be
//     //  completely async after launching all the IO.
//     //
//     //  IrpCount will indicate the remaining number of associated Irps to launch.
//     //
//     //  All we have to do is make sure IrpCount doesn't hit zero before we're building
//     //  the very last Irp.  If it is positive when we're done, it means we have to
//     //  wait for the rest of the associated Irps to come back before we complete the
//     //  master by hand.
//     //
//     //  This will keep the master from completing early.
//     //
//
//     Irp->AssociatedIrp.IrpCount = IrpCount = LastIndex - FirstIndex + 1;
//
//     while (CurrentIndex <= LastIndex) {
//
//         //
//         //  Reset this for unwinding purposes
//         //
//
//         AssocIrp = NULL;
//
//         //
//         //  If next run is larger than we need, "ya get what ya need".
//         //
//
//         if (NextByteCount > ByteCount) {
//             NextByteCount = ByteCount;
//         }
//
//         RemainingByteCount = 0;
//
//         //
//         // Allocate and build a partial Mdl for the request.
//         //
//
//         Mdl = IoAllocateMdl( (PCHAR)Irp->UserBuffer + BufferOffset,
//                              NextByteCount,
//                              FALSE,
//                              FALSE,
//                              AssocIrp );
//
//         if (Mdl == NULL) {
//
//             //
//             //  Pick up the reserve MDL
//             //
//
//             KeWaitForSingleObject( &FatReserveEvent, Executive, KernelMode, FALSE, NULL );
//
//             Mdl = FatReserveMdl;
//             MdlIsReserve = TRUE;
//
//             //
//             //  Trim to fit the size of the reserve MDL.
//             //
//
//             if (NextByteCount > FAT_RESERVE_MDL_SIZE * PAGE_SIZE) {
//
//                 RemainingByteCount = NextByteCount - FAT_RESERVE_MDL_SIZE * PAGE_SIZE;
//                 NextByteCount = FAT_RESERVE_MDL_SIZE * PAGE_SIZE;
//             }
//         }
//
//         IoBuildPartialMdl( Irp->MdlAddress,
//                            Mdl,
//                            (PCHAR)Irp->UserBuffer + BufferOffset,
//                            NextByteCount );
//
//         //
//         //  Now that we have properly bounded this piece of the transfer, it is
//         //  time to read/write it.  We can simplify life slightly by always
//         //  re-using the master IRP for cases where we use the reserve MDL,
//         //  since we'll always be synchronous for those and can use a single
//         //  completion context on our local stack.
//         //
//         //  We also must prevent ourselves from issuing an associated IRP that would
//         //  complete the master UNLESS this is the very last IRP we'll issue.
//         //
//         //  This logic looks a bit complicated, but is hopefully understandable.
//         //
//
//         if (!MdlIsReserve &&
//             (IrpCount != 1 ||
//              (CurrentIndex == LastIndex &&
//               RemainingByteCount == 0))) {
//
//             AssocIrp = IoMakeAssociatedIrp( Irp, (CCHAR)(DeviceObject->StackSize + 1) );
//         }
//
//         if (AssocIrp == NULL) {
//
//             AssocIrp = Irp;
//             IrpIsMaster = TRUE;
//
//             //
//             //  We need to drain the associated Irps so we can reliably figure out if
//             //  the master Irp is showing a failed status, in which case we bail out
//             //  immediately - as opposed to putting the value in the status field in
//             //  jeopardy due to our re-use of the master Irp.
//             //
//
//             while (Irp->AssociatedIrp.IrpCount != IrpCount) {
//
//                 KeDelayExecutionThread (KernelMode, FALSE, &Fat30Milliseconds);
//             }
//
//             //
//             //  Note that since we failed to launch this associated Irp, that the completion
//             //  code at the bottom will take care of completing the master Irp.
//             //
//
//             if (!NT_SUCCESS(Irp->IoStatus.Status)) {
//
//                 NT_ASSERT( IrpCount );
//                 break;
//             }
//
//         } else {
//
//             //
//             //  Indicate we used an associated Irp.
//             //
//
//             IrpCount -= 1;
//         }
//
//         //
//         //  With an associated IRP, we must take over the first stack location so
//         //  we can have one to put the completion routine on.  When re-using the
//         //  master IRP, its already there.
//         //
//
//         if (!IrpIsMaster) {
//
//             //
//             //  Get the first IRP stack location in the associated Irp
//             //
//
//             IoSetNextIrpStackLocation( AssocIrp );
//             NextIrpSp = IoGetCurrentIrpStackLocation( AssocIrp );
//
//             //
//             //  Setup the Stack location to describe our read.
//             //
//
//             NextIrpSp->MajorFunction = IrpSp->MajorFunction;
//             NextIrpSp->Parameters.Read.Length = NextByteCount;
//             NextIrpSp->Parameters.Read.ByteOffset.QuadPart = Vbo;
//
//             //
//             //  We also need the VolumeDeviceObject in the Irp stack in case
//             //  we take the failure path.
//             //
//
//             NextIrpSp->DeviceObject = IrpSp->DeviceObject;
//
//         } else {
//
//             //
//             //  Save the MDL in the IRP and prepare the stack
//             //  context for the completion routine.
//             //
//
//             KeInitializeEvent( &Context.Event, SynchronizationEvent, FALSE );
//             Context.RestoreMdl = Irp->MdlAddress;
//         }
//
//         //
//         //  And drop our Mdl into the Irp.
//         //
//
//         AssocIrp->MdlAddress = Mdl;
//
//         //
//         //  Set up the completion routine address in our stack frame.
//         //  For true associated IRPs, this is only invoked on error or
//         //  cancel, and just copies the error Status into master irp's
//         //  iosb.
//         //
//         //  If the error implies a media problem, it also enqueues a
//         //  worker item to write out the dirty bit so that the next
//         //  time we run we will do a autochk /r
//         //
//
//         if (IrpIsMaster) {
//
//             IoSetCompletionRoutine( AssocIrp,
//                                     FatPagingFileCompletionRoutineCatch,
//                                     &Context,
//                                     TRUE,
//                                     TRUE,
//                                     TRUE );
//
//         } else {
//
//             IoSetCompletionRoutine( AssocIrp,
//                                     FatPagingFileCompletionRoutine,
//                                     Irp,
//                                     FALSE,
//                                     TRUE,
//                                     TRUE );
//         }
//
//         //
//         //  Setup the next IRP stack location for the disk driver beneath us.
//         //
//
//         NextIrpSp = IoGetNextIrpStackLocation( AssocIrp );
//
//         //
//         //  Since this is paging file IO, we'll just ignore the verify bit.
//         //
//
//         SetFlag( NextIrpSp->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//
//         //
//         //  Setup the Stack location to do a read from the disk driver.
//         //
//
//         NextIrpSp->MajorFunction = IrpSp->MajorFunction;
//         NextIrpSp->Parameters.Read.Length = NextByteCount;
//         NextIrpSp->Parameters.Read.ByteOffset.QuadPart = NextLbo;
//
//         (VOID)IoCallDriver( DeviceObject, AssocIrp );
//
//         //
//         //  We just issued an IO to the storage stack, update the counters indicating so.
//         //
//
//         if (FatDiskAccountingEnabled) {
//
//             FatUpdateIOCountersPCW( IsAWrite, (ULONG64)NextByteCount );
//         }
//
//         //
//         //  Wait for the Irp in the catch case and drop the flags.
//         //
//
//         if (IrpIsMaster) {
//
//             KeWaitForSingleObject( &Context.Event, Executive, KernelMode, FALSE, NULL );
//             IrpIsMaster = MdlIsReserve = FALSE;
//
//             //
//             //  If the Irp is showing a failed status, there is no point in continuing.
//             //  In doing so, we get to avoid squirreling away the failed status in case
//             //  we were to re-use the master irp again.
//             //
//             //  Note that since we re-used the master, we must not have issued the "last"
//             //  associated Irp, and thus the completion code at the bottom will take care
//             //  of that for us.
//             //
//
//             if (!NT_SUCCESS(Irp->IoStatus.Status)) {
//
//                 NT_ASSERT( IrpCount );
//                 break;
//             }
//         }
//
//         //
//         //  Now adjust everything for the next pass through the loop.
//         //
//
//         Vbo += NextByteCount;
//         BufferOffset += NextByteCount;
//         ByteCount -= NextByteCount;
//
//         //
//         //  Try to lookup the next run, if we are not done and we got
//         //  all the way through the current run.
//         //
//
//         if (RemainingByteCount) {
//
//             //
//             //  Advance the Lbo/Vbo if we have more to do in the current run.
//             //
//
//             NextLbo += NextByteCount;
//             NextVbo += NextByteCount;
//
//             NextByteCount = RemainingByteCount;
//
//         } else {
//
//             CurrentIndex += 1;
//
//             if ( CurrentIndex <= LastIndex ) {
//
//                 NT_ASSERT( ByteCount != 0 );
//
//                 FatGetNextMcbEntry( Fcb->Vcb, &Fcb->Mcb,
//                                     CurrentIndex,
//                                     &NextVbo,
//                                     &NextLbo,
//                                     &NextByteCount );
//
//                 NT_ASSERT( NextVbo == Vbo );
//             }
//         }
//     } // while ( CurrentIndex <= LastIndex )
//
//     //
//     //  If we didn't get enough associated Irps going to make this asynchronous, we
//     //  twiddle our thumbs and wait for those we did launch to complete.
//     //
//
//     if (IrpCount) {
//
//         while (Irp->AssociatedIrp.IrpCount != IrpCount) {
//
//             KeDelayExecutionThread (KernelMode, FALSE, &Fat30Milliseconds);
//         }
//
//         IoCompleteRequest( Irp, IO_DISK_INCREMENT );
//     }
//
//     DebugTrace(-1, Dbg, "FatPagingFileIo -> VOID\n", 0);
//     return;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
// VOID
// FatUpdateDiskStats (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN ULONG ByteCount
//     )
// /*++
//
// Routine Description:
//
//     Charge appropriate process for the IO this IRP will cause.
//
// Arguments:
//
//     IrpContext- The Irp Context
//
//     Irp - Supplies the requesting Irp.
//
//     ByteCount - The lengh of the operation.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PETHREAD OriginatingThread = NULL;
//     ULONG NumReads = 0;
//     ULONG NumWrites = 0;
//     ULONGLONG BytesToRead = 0;
//     ULONGLONG BytesToWrite = 0;
//
//     //
//     //  Here we attempt to charge the IO back to the originating process.
//     //   - These checks are intended to cover following cases:
//     //      o Buffered sync reads
//     //      o Unbuffered sync read
//     //      o Inline metadata reads
//     //      o memory mapped reads (in-line faulting of data)
//     //
//
//     if (IrpContext->MajorFunction == IRP_MJ_READ) {
//
//         NumReads++;
//         BytesToRead = ByteCount;
//
//         if ((Irp->Tail.Overlay.Thread != NULL) &&
//             !IoIsSystemThread( Irp->Tail.Overlay.Thread )) {
//
//             OriginatingThread = Irp->Tail.Overlay.Thread;
//
//         } else if (!IoIsSystemThread( PsGetCurrentThread() )) {
//
//             OriginatingThread = PsGetCurrentThread();
//
//         //
//         //  We couldn't find a non-system entity, so this should be charged to system.
//         //  Do so only if we are top level.
//         //  If we are not top-level then the read was initiated by someone like Cc (read ahead)
//         //  who should have already accounted for this IO.
//         //
//
//         } else if (IoIsSystemThread( PsGetCurrentThread() ) &&
//                    (IoGetTopLevelIrp() == Irp)) {
//
//             OriginatingThread = PsGetCurrentThread();
//         }
//
//     //
//     //  Charge the write to Originating process.
//     //  Intended to cover the following writes:
//     //      - Unbuffered sync write
//     //      - unbuffered async write
//     //
//     //  If we re not top-level, then it should already have been accounted for
//     //  somewhere else (Cc).
//     //
//
//     } else if (IrpContext->MajorFunction == IRP_MJ_WRITE) {
//
//         NumWrites++;
//         BytesToWrite = ByteCount;
//
//         if (IoGetTopLevelIrp() == Irp) {
//
//             if ((Irp->Tail.Overlay.Thread != NULL) &&
//                 !IoIsSystemThread( Irp->Tail.Overlay.Thread )) {
//
//                 OriginatingThread = Irp->Tail.Overlay.Thread;
//
//             } else {
//
//                 OriginatingThread = PsGetCurrentThread();
//             }
//
//         //
//         //  For mapped page writes
//         //
//
//         } else if (IoGetTopLevelIrp() == (PIRP)FSRTL_MOD_WRITE_TOP_LEVEL_IRP) {
//
//             OriginatingThread = PsGetCurrentThread();
//         }
//     }
//
//     if (OriginatingThread != NULL) {
//
//         PsUpdateDiskCounters( PsGetThreadProcess( OriginatingThread ),
//                               BytesToRead,
//                               BytesToWrite,
//                               NumReads,
//                               NumWrites,
//                               0 );
//     }
// }
//
// #endif
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
/* NTSTATUS */
// FatNonCachedIo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB FcbOrDcb,
//     IN ULONG StartingVbo,
//     IN ULONG ByteCount,
//     IN ULONG UserByteCount,
//     IN ULONG StreamFlags
//     )
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached disk io described in its parameters.
//     The choice of a single run is made if possible, otherwise multiple runs
//     are executed.
//
// Arguments:
//
//     IrpContext->MajorFunction - Supplies either IRP_MJ_READ or IRP_MJ_WRITE.
//
//     Irp - Supplies the requesting Irp.
//
//     FcbOrDcb - Supplies the file to act on.
//
//     StartingVbo - The starting point for the operation.
//
//     ByteCount - The lengh of the operation.
//
//     UserByteCount - The last byte the user can see, rest to be zeroed.
//
//     StreamFlags - flag to indicate special attributes for a NonCachedIo.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//
//     //
//     // Declare some local variables for enumeration through the
//     // runs of the file, and an array to store parameters for
//     // parallel I/Os
//     //
//
//     BOOLEAN Wait;
//
//     LBO NextLbo;
//     VBO NextVbo;
//     ULONG NextByteCount;
//     BOOLEAN NextIsAllocated;
//
//     LBO LastLbo;
//     ULONG LastByteCount;
//     BOOLEAN LastIsAllocated;
//
//     BOOLEAN EndOnMax;
//
//     ULONG FirstIndex;
//     ULONG CurrentIndex;
//     ULONG LastIndex;
//
//     ULONG NextRun;
//     ULONG BufferOffset;
//     ULONG OriginalByteCount;
//
//
//
//     IO_RUN StackIoRuns[FAT_MAX_IO_RUNS_ON_STACK];
//     PIO_RUN IoRuns;
//
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( StreamFlags );
//
//     DebugTrace(+1, Dbg, "FatNonCachedIo\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp );
//     DebugTrace( 0, Dbg, "MajorFunction = %08lx\n", IrpContext->MajorFunction );
//     DebugTrace( 0, Dbg, "FcbOrDcb      = %p\n", FcbOrDcb );
//     DebugTrace( 0, Dbg, "StartingVbo   = %08lx\n", StartingVbo );
//     DebugTrace( 0, Dbg, "ByteCount     = %08lx\n", ByteCount );
//
//     if (!FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//
//         PFILE_SYSTEM_STATISTICS Stats =
//             &FcbOrDcb->Vcb->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors];
//
//         if (IrpContext->MajorFunction == IRP_MJ_READ) {
//             Stats->Fat.NonCachedReads += 1;
//             Stats->Fat.NonCachedReadBytes += ByteCount;
//         } else {
//             Stats->Fat.NonCachedWrites += 1;
//             Stats->Fat.NonCachedWriteBytes += ByteCount;
//         }
//     }
//
//     //
//     //  Initialize some locals.
//     //
//
//     NextRun = 0;
//     BufferOffset = 0;
//     OriginalByteCount = ByteCount;
//
//     Wait = BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     //
//     //  Disk IO accounting
//     //
//
//     if (FatDiskAccountingEnabled) {
//
//         FatUpdateDiskStats( IrpContext,
//                             Irp,
//                             ByteCount );
//     }
// #endif
//
//     //
//     // For nonbuffered I/O, we need the buffer locked in all
//     // cases.
//     //
//     // This call may raise.  If this call succeeds and a subsequent
//     // condition is raised, the buffers are unlocked automatically
//     // by the I/O system when the request is completed, via the
//     // Irp->MdlAddress field.
//     //
//
//     FatLockUserBuffer( IrpContext,
//                        Irp,
//                        (IrpContext->MajorFunction == IRP_MJ_READ) ?
//                        IoWriteAccess : IoReadAccess,
//                        ByteCount );
//
//
//
//     //
//     //  No zeroing for trailing sectors if requested.
//     //  Otherwise setup the required zeroing for read requests.
//     //
//
//
//     if (UserByteCount != ByteCount) {
//
//
//         PMDL Mdl;
//
//         NT_ASSERT( ByteCount > UserByteCount );
//         _Analysis_assume_(ByteCount > UserByteCount);
//
//         Mdl = IoAllocateMdl( (PUCHAR) Irp->UserBuffer + UserByteCount,
//                              ByteCount - UserByteCount,
//                              FALSE,
//                              FALSE,
//                              NULL );
//
//         if (Mdl == NULL) {
//
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         IoBuildPartialMdl( Irp->MdlAddress,
//                            Mdl,
//                            (PUCHAR) Irp->UserBuffer + UserByteCount,
//                            ByteCount - UserByteCount );
//
//         IrpContext->FatIoContext->ZeroMdl = Mdl;
//
//         //
//         //  Map the MDL now so we can't fail at IO completion time.  Note
//         //  that this will be only a single page.
//         //
//
//         if (MmGetSystemAddressForMdlSafe( Mdl, NormalPagePriority | MdlMappingNoExecute ) == NULL) {
//
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//     }
//
//
//     //
//     //  Try to lookup the first run.  If there is just a single run,
//     //  we may just be able to pass it on.
//     //
//
//     FatLookupFileAllocation( IrpContext,
//                              FcbOrDcb,
//                              StartingVbo,
//                              &NextLbo,
//                              &NextByteCount,
//                              &NextIsAllocated,
//                              &EndOnMax,
//                              &FirstIndex );
//
//     //
//     //  We just added the allocation, thus there must be at least
//     //  one entry in the mcb corresponding to our write, ie.
//     //  NextIsAllocated must be true.  If not, the pre-existing file
//     //  must have an allocation error.
//     //
//
//     if ( !NextIsAllocated ) {
//
//         FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//
//         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     NT_ASSERT( NextByteCount != 0 );
//
//     //
//     //  If the request was not aligned correctly, read in the first
//     //  part first.
//     //
//
//
//     //
//     //  See if the write covers a single valid run, and if so pass
//     //  it on.  We must bias this by the byte that is lost at the
//     //  end of the maximal file.
//     //
//
//     if ( NextByteCount >= ByteCount - (EndOnMax ? 1 : 0)) {
//
//         if (FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//             CollectDiskIoStats(FcbOrDcb->Vcb, IrpContext->MajorFunction,
//                                FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_USER_IO), 1);
//         } else {
//
//             PFILE_SYSTEM_STATISTICS Stats =
//                 &FcbOrDcb->Vcb->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors];
//
//             if (IrpContext->MajorFunction == IRP_MJ_READ) {
//                 Stats->Fat.NonCachedDiskReads += 1;
//             } else {
//                 Stats->Fat.NonCachedDiskWrites += 1;
//             }
//         }
//
//         DebugTrace( 0, Dbg, "Passing 1 Irp on to Disk Driver\n", 0 );
//
//         FatSingleAsync( IrpContext,
//                         FcbOrDcb->Vcb,
//                         NextLbo,
//                         ByteCount,
//                         Irp );
//
//     } else {
//
//         //
//         //  If there we can't wait, and there are more runs than we can handle,
//         //  we will have to post this request.
//         //
//
//         FatLookupFileAllocation( IrpContext,
//                                  FcbOrDcb,
//                                  StartingVbo + ByteCount - 1,
//                                  &LastLbo,
//                                  &LastByteCount,
//                                  &LastIsAllocated,
//                                  &EndOnMax,
//                                  &LastIndex );
//
//         //
//         // Since we already added the allocation for the whole
//         // write, assert that we find runs until ByteCount == 0
//         // Otherwise this file is corrupt.
//         //
//
//         if ( !LastIsAllocated ) {
//
//             FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//         }
//
//         if (LastIndex - FirstIndex + 1 > FAT_MAX_IO_RUNS_ON_STACK) {
//
//             IoRuns = FsRtlAllocatePoolWithTag( PagedPool,
//                                                (LastIndex - FirstIndex + 1) * sizeof(IO_RUN),
//                                                TAG_IO_RUNS );
//
//         } else {
//
//             IoRuns = StackIoRuns;
//         }
//
//         NT_ASSERT( LastIndex != FirstIndex );
//
//         CurrentIndex = FirstIndex;
//
//         //
//         // Loop while there are still byte writes to satisfy.
//         //
//
//         while (CurrentIndex <= LastIndex) {
//
//
//             NT_ASSERT( NextByteCount != 0);
//             NT_ASSERT( ByteCount != 0);
//
//             //
//             // If next run is larger than we need, "ya get what you need".
//             //
//
//             if (NextByteCount > ByteCount) {
//                 NextByteCount = ByteCount;
//             }
//
//             //
//             // Now that we have properly bounded this piece of the
//             // transfer, it is time to write it.
//             //
//             // We remember each piece of a parallel run by saving the
//             // essential information in the IoRuns array.  The tranfers
//             // are started up in parallel below.
//             //
//
//             IoRuns[NextRun].Vbo = StartingVbo;
//             IoRuns[NextRun].Lbo = NextLbo;
//             IoRuns[NextRun].Offset = BufferOffset;
//             IoRuns[NextRun].ByteCount = NextByteCount;
//             NextRun += 1;
//
//             //
//             // Now adjust everything for the next pass through the loop.
//             //
//
//             StartingVbo += NextByteCount;
//             BufferOffset += NextByteCount;
//             ByteCount -= NextByteCount;
//
//             //
//             // Try to lookup the next run (if we are not done).
//             //
//
//             CurrentIndex += 1;
//
//             if ( CurrentIndex <= LastIndex ) {
//
//                 NT_ASSERT( ByteCount != 0 );
//
//                 FatGetNextMcbEntry( FcbOrDcb->Vcb, &FcbOrDcb->Mcb,
//                                     CurrentIndex,
//                                     &NextVbo,
//                                     &NextLbo,
//                                     &NextByteCount );
//
//
//                 NT_ASSERT(NextVbo == StartingVbo);
//
//
//             }
//
//         } // while ( CurrentIndex <= LastIndex )
//
//         //
//         //  Now set up the Irp->IoStatus.  It will be modified by the
//         //  multi-completion routine in case of error or verify required.
//         //
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = OriginalByteCount;
//
//         if (FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//             CollectDiskIoStats(FcbOrDcb->Vcb, IrpContext->MajorFunction,
//                                FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_USER_IO), NextRun);
//         }
//
//         //
//         //  OK, now do the I/O.
//         //
//
//         _SEH2_TRY {
//
//             DebugTrace( 0, Dbg, "Passing Multiple Irps on to Disk Driver\n", 0 );
//
//             FatMultipleAsync( IrpContext,
//                               FcbOrDcb->Vcb,
//                               Irp,
//                               NextRun,
//                               IoRuns );
//
//         } _SEH2_FINALLY {
//
//             if (IoRuns != StackIoRuns) {
//
//                 ExFreePool( IoRuns );
//             }
//         } _SEH2_END;
//     }
//
//     if (!Wait) {
//
//         DebugTrace(-1, Dbg, "FatNonCachedIo -> STATUS_PENDING\n", 0);
//         return STATUS_PENDING;
//     }
//
//     FatWaitSync( IrpContext );
//
//
//     DebugTrace(-1, Dbg, "FatNonCachedIo -> 0x%08lx\n", Irp->IoStatus.Status);
//     return Irp->IoStatus.Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatNonCachedNonAlignedRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB FcbOrDcb,
//     IN ULONG StartingVbo,
//     IN ULONG ByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached disk io described in its parameters.
//     This routine differs from the above in that the range does not have to be
//     sector aligned.  This accomplished with the use of intermediate buffers.
//
// Arguments:
//
//     IrpContext->MajorFunction - Supplies either IRP_MJ_READ or IRP_MJ_WRITE.
//
//     Irp - Supplies the requesting Irp.
//
//     FcbOrDcb - Supplies the file to act on.
//
//     StartingVbo - The starting point for the operation.
//
//     ByteCount - The lengh of the operation.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     //
//     // Declare some local variables for enumeration through the
//     // runs of the file, and an array to store parameters for
//     // parallel I/Os
//     //
//
//     LBO NextLbo;
//     ULONG NextByteCount;
//     BOOLEAN NextIsAllocated;
//
//     ULONG SectorSize;
//     ULONG BytesToCopy;
//     ULONG OriginalByteCount;
//     ULONG OriginalStartingVbo;
//
//     BOOLEAN EndOnMax;
//
//     PUCHAR UserBuffer;
//     PUCHAR DiskBuffer = NULL;
//
//     PMDL Mdl;
//     PMDL SavedMdl;
//     PVOID SavedUserBuffer;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatNonCachedNonAlignedRead\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp );
//     DebugTrace( 0, Dbg, "MajorFunction = %08lx\n", IrpContext->MajorFunction );
//     DebugTrace( 0, Dbg, "FcbOrDcb      = %p\n", FcbOrDcb );
//     DebugTrace( 0, Dbg, "StartingVbo   = %08lx\n", StartingVbo );
//     DebugTrace( 0, Dbg, "ByteCount     = %08lx\n", ByteCount );
//
//     //
//     //  Initialize some locals.
//     //
//
//     OriginalByteCount = ByteCount;
//     OriginalStartingVbo = StartingVbo;
//     SectorSize = FcbOrDcb->Vcb->Bpb.BytesPerSector;
//
//     NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//     //
//     // For nonbuffered I/O, we need the buffer locked in all
//     // cases.
//     //
//     // This call may raise.  If this call succeeds and a subsequent
//     // condition is raised, the buffers are unlocked automatically
//     // by the I/O system when the request is completed, via the
//     // Irp->MdlAddress field.
//     //
//
//     FatLockUserBuffer( IrpContext,
//                        Irp,
//                        IoWriteAccess,
//                        ByteCount );
//
//     UserBuffer = FatMapUserBuffer( IrpContext, Irp );
//
//     //
//     //  Allocate the local buffer
//     //
//
//     DiskBuffer = FsRtlAllocatePoolWithTag( NonPagedPoolNxCacheAligned,
//                                            (ULONG) ROUND_TO_PAGES( SectorSize ),
//                                            TAG_IO_BUFFER );
//
//     //
//     //  We use a try block here to ensure the buffer is freed, and to
//     //  fill in the correct byte count in the Iosb.Information field.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  If the beginning of the request was not aligned correctly, read in
//         //  the first part first.
//         //
//
//         if ( StartingVbo & (SectorSize - 1) ) {
//
//             VBO Hole;
//
//             //
//             // Try to lookup the first run.
//             //
//
//             FatLookupFileAllocation( IrpContext,
//                                      FcbOrDcb,
//                                      StartingVbo,
//                                      &NextLbo,
//                                      &NextByteCount,
//                                      &NextIsAllocated,
//                                      &EndOnMax,
//                                      NULL );
//
//             //
//             // We just added the allocation, thus there must be at least
//             // one entry in the mcb corresponding to our write, ie.
//             // NextIsAllocated must be true.  If not, the pre-existing file
//             // must have an allocation error.
//             //
//
//             if ( !NextIsAllocated ) {
//
//                 FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             FatSingleNonAlignedSync( IrpContext,
//                                      FcbOrDcb->Vcb,
//                                      DiskBuffer,
//                                      NextLbo & ~((LONG)SectorSize - 1),
//                                      SectorSize,
//                                      Irp );
//
//             if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//                 try_return( NOTHING );
//             }
//
//             //
//             //  Now copy the part of the first sector that we want to the user
//             //  buffer.
//             //
//
//             Hole = StartingVbo & (SectorSize - 1);
//
//             BytesToCopy = ByteCount >= SectorSize - Hole ?
//                                        SectorSize - Hole : ByteCount;
//
//             RtlCopyMemory( UserBuffer, DiskBuffer + Hole, BytesToCopy );
//
//             StartingVbo += BytesToCopy;
//             ByteCount -= BytesToCopy;
//
//             if ( ByteCount == 0 ) {
//
//                 try_return( NOTHING );
//             }
//         }
//
//         NT_ASSERT( (StartingVbo & (SectorSize - 1)) == 0 );
//
//         //
//         //  If there is a tail part that is not sector aligned, read it.
//         //
//
//         if ( ByteCount & (SectorSize - 1) ) {
//
//             VBO LastSectorVbo;
//
//             LastSectorVbo = StartingVbo + (ByteCount & ~(SectorSize - 1));
//
//             //
//             // Try to lookup the last part of the requested range.
//             //
//
//             FatLookupFileAllocation( IrpContext,
//                                      FcbOrDcb,
//                                      LastSectorVbo,
//                                      &NextLbo,
//                                      &NextByteCount,
//                                      &NextIsAllocated,
//                                      &EndOnMax,
//                                      NULL );
//
//             //
//             // We just added the allocation, thus there must be at least
//             // one entry in the mcb corresponding to our write, ie.
//             // NextIsAllocated must be true.  If not, the pre-existing file
//             // must have an allocation error.
//             //
//
//             if ( !NextIsAllocated ) {
//
//                 FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             FatSingleNonAlignedSync( IrpContext,
//                                      FcbOrDcb->Vcb,
//                                      DiskBuffer,
//                                      NextLbo,
//                                      SectorSize,
//                                      Irp );
//
//             if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//                 try_return( NOTHING );
//             }
//
//             //
//             //  Now copy over the part of this last sector that we need.
//             //
//
//             BytesToCopy = ByteCount & (SectorSize - 1);
//
//             UserBuffer += LastSectorVbo - OriginalStartingVbo;
//
//             RtlCopyMemory( UserBuffer, DiskBuffer, BytesToCopy );
//
//             ByteCount -= BytesToCopy;
//
//             if ( ByteCount == 0 ) {
//
//                 try_return( NOTHING );
//             }
//         }
//
//         NT_ASSERT( ((StartingVbo | ByteCount) & (SectorSize - 1)) == 0 );
//
//         //
//         //  Now build a Mdl describing the sector aligned balance of the transfer,
//         //  and put it in the Irp, and read that part.
//         //
//
//         SavedMdl = Irp->MdlAddress;
//         Irp->MdlAddress = NULL;
//
//         SavedUserBuffer = Irp->UserBuffer;
//
//         Irp->UserBuffer = (PUCHAR)MmGetMdlVirtualAddress( SavedMdl ) +
//                           (StartingVbo - OriginalStartingVbo);
//
//         Mdl = IoAllocateMdl( Irp->UserBuffer,
//                              ByteCount,
//                              FALSE,
//                              FALSE,
//                              Irp );
//
//         if (Mdl == NULL) {
//
//             Irp->MdlAddress = SavedMdl;
//             Irp->UserBuffer = SavedUserBuffer;
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         IoBuildPartialMdl( SavedMdl,
//                            Mdl,
//                            Irp->UserBuffer,
//                            ByteCount );
//
//         //
//         //  Try to read in the pages.
//         //
//
//         _SEH2_TRY {
//
//             FatNonCachedIo( IrpContext,
//                             Irp,
//                             FcbOrDcb,
//                             StartingVbo,
//                             ByteCount,
//                             ByteCount,
//                             0 );
//
//         } _SEH2_FINALLY {
//
//             IoFreeMdl( Irp->MdlAddress );
//
//             Irp->MdlAddress = SavedMdl;
//             Irp->UserBuffer = SavedUserBuffer;
//         } _SEH2_END;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         ExFreePool( DiskBuffer );
//
//         if ( !_SEH2_AbnormalTermination() && NT_SUCCESS(Irp->IoStatus.Status) ) {
//
//             Irp->IoStatus.Information = OriginalByteCount;
//
//             //
//             //  We now flush the user's buffer to memory.
//             //
//
//             KeFlushIoBuffers( Irp->MdlAddress, TRUE, FALSE );
//         }
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatNonCachedNonAlignedRead -> VOID\n", 0);
//     return;
// }
//
//
// VOID
// FatMultipleAsync (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PIRP MasterIrp,
//     IN ULONG MultipleIrpCount,
//     IN PIO_RUN IoRuns
//     )
//
// /*++
//
// Routine Description:
//
//     This routine first does the initial setup required of a Master IRP that is
//     going to be completed using associated IRPs.  This routine should not
//     be used if only one async request is needed, instead the single read/write
//     async routines should be called.
//
//     A context parameter is initialized, to serve as a communications area
//     between here and the common completion routine.  This initialization
//     includes allocation of a spinlock.  The spinlock is deallocated in the
//     FatWaitSync routine, so it is essential that the caller insure that
//     this routine is always called under all circumstances following a call
//     to this routine.
//
//     Next this routine reads or writes one or more contiguous sectors from
//     a device asynchronously, and is used if there are multiple reads for a
//     master IRP.  A completion routine is used to synchronize with the
//     completion of all of the I/O requests started by calls to this routine.
//
//     Also, prior to calling this routine the caller must initialize the
//     IoStatus field in the Context, with the correct success status and byte
//     count which are expected if all of the parallel transfers complete
//     successfully.  After return this status will be unchanged if all requests
//     were, in fact, successful.  However, if one or more errors occur, the
//     IoStatus will be modified to reflect the error status and byte count
//     from the first run (by Vbo) which encountered an error.  I/O status
//     from all subsequent runs will not be indicated.
//
// Arguments:
//
//     IrpContext->MajorFunction - Supplies either IRP_MJ_READ or IRP_MJ_WRITE.
//
//     Vcb - Supplies the device to be read
//
//     MasterIrp - Supplies the master Irp.
//
//     MulitpleIrpCount - Supplies the number of multiple async requests
//         that will be issued against the master irp.
//
//     IoRuns - Supplies an array containing the Vbo, Lbo, BufferOffset, and
//         ByteCount for all the runs to executed in parallel.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IrpSp;
//     PMDL Mdl;
//     BOOLEAN Wait;
//     PFAT_IO_CONTEXT Context;
// #ifndef __REACTOS__
//     BOOLEAN IsAWrite = FALSE;
//     ULONG Length = 0;
// #endif
//
//     ULONG UnwindRunCount = 0;
//
//     BOOLEAN ExceptionExpected = TRUE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatMultipleAsync\n", 0);
//     DebugTrace( 0, Dbg, "MajorFunction    = %08lx\n", IrpContext->MajorFunction );
//     DebugTrace( 0, Dbg, "Vcb              = %p\n", Vcb );
//     DebugTrace( 0, Dbg, "MasterIrp        = %p\n", MasterIrp );
//     DebugTrace( 0, Dbg, "MultipleIrpCount = %08lx\n", MultipleIrpCount );
//     DebugTrace( 0, Dbg, "IoRuns           = %08lx\n", IoRuns );
//
//     //
//     //  If this I/O originating during FatVerifyVolume, bypass the
//     //  verify logic.
//     //
//
//     if (Vcb->VerifyThread == KeGetCurrentThread() as *mut _) {
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY );
//     }
//
//     //
//     //  Set up things according to whether this is truely async.
//     //
//
//     Wait = BooleanFlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     Context = IrpContext->FatIoContext;
//
//     //
//     //  Finish initializing Context, for use in Read/Write Multiple Asynch.
//     //
//
//     Context->MasterIrp = MasterIrp;
//
//     IrpSp = IoGetCurrentIrpStackLocation( MasterIrp );
// #ifndef __REACTOS__
//     IsAWrite = (IrpSp->MajorFunction == IRP_MJ_WRITE);
//     Length = IrpSp->Parameters.Read.Length;
// #endif
//
//     _SEH2_TRY {
//
//         //
//         //  Itterate through the runs, doing everything that can fail
//         //
//
//         for ( UnwindRunCount = 0;
//               UnwindRunCount < MultipleIrpCount;
//               UnwindRunCount++ ) {
//
//             //
//             //  Create an associated IRP, making sure there is one stack entry for
//             //  us, as well.
//             //
//
//             IoRuns[UnwindRunCount].SavedIrp = 0;
//
//             Irp = IoMakeAssociatedIrp( MasterIrp,
//                                        (CCHAR)(Vcb->TargetDeviceObject->StackSize + 1) );
//
//             if (Irp == NULL) {
//
//                 FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//             }
//
//             IoRuns[UnwindRunCount].SavedIrp = Irp;
//
//             //
//             // Allocate and build a partial Mdl for the request.
//             //
//
//             Mdl = IoAllocateMdl( (PCHAR)MasterIrp->UserBuffer +
//                                  IoRuns[UnwindRunCount].Offset,
//                                  IoRuns[UnwindRunCount].ByteCount,
//                                  FALSE,
//                                  FALSE,
//                                  Irp );
//
//             if (Mdl == NULL) {
//
//                 FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//             }
//
//             //
//             //  Sanity Check
//             //
//
//             NT_ASSERT( Mdl == Irp->MdlAddress );
//
//             IoBuildPartialMdl( MasterIrp->MdlAddress,
//                                Mdl,
//                                (PCHAR)MasterIrp->UserBuffer +
//                                IoRuns[UnwindRunCount].Offset,
//                                IoRuns[UnwindRunCount].ByteCount );
//
//             //
//             //  Get the first IRP stack location in the associated Irp
//             //
//
//             IoSetNextIrpStackLocation( Irp );
//             IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//             //
//             //  Setup the Stack location to describe our read.
//             //
//
//             IrpSp->MajorFunction = IrpContext->MajorFunction;
//             IrpSp->Parameters.Read.Length = IoRuns[UnwindRunCount].ByteCount;
//             IrpSp->Parameters.Read.ByteOffset.QuadPart = IoRuns[UnwindRunCount].Vbo;
//
//             //
//             // Set up the completion routine address in our stack frame.
//             //
//
//             IoSetCompletionRoutine( Irp,
//                                     Wait ?
//                                     &FatMultiSyncCompletionRoutine :
//                                     &FatMultiAsyncCompletionRoutine,
//                                     Context,
//                                     TRUE,
//                                     TRUE,
//                                     TRUE );
//
//             //
//             //  Setup the next IRP stack location in the associated Irp for the disk
//             //  driver beneath us.
//             //
//
//             IrpSp = IoGetNextIrpStackLocation( Irp );
//
//             //
//             //  Setup the Stack location to do a read from the disk driver.
//             //
//
//             IrpSp->MajorFunction = IrpContext->MajorFunction;
//             IrpSp->Parameters.Read.Length = IoRuns[UnwindRunCount].ByteCount;
//             IrpSp->Parameters.Read.ByteOffset.QuadPart = IoRuns[UnwindRunCount].Lbo;
//
//             //
//             //  If this Irp is the result of a WriteThough operation,
//             //  tell the device to write it through.
//             //
//
//             if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH )) {
//
//                 SetFlag( IrpSp->Flags, SL_WRITE_THROUGH );
//             }
//
//             //
//             //  If this I/O requires override verify, bypass the verify logic.
//             //
//
//             if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY )) {
//
//                 SetFlag( IrpSp->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//             }
//         }
//
//         //
//         //  Now we no longer expect an exception.  If the driver raises, we
//         //  must bugcheck, because we do not know how to recover from that
//         //  case.
//         //
//
//         ExceptionExpected = FALSE;
//
//         //
//         //  We only need to set the associated IRP count in the master irp to
//         //  make it a master IRP.  But we set the count to one more than our
//         //  caller requested, because we do not want the I/O system to complete
//         //  the I/O.  We also set our own count.
//         //
//
//         Context->IrpCount = MultipleIrpCount;
//         MasterIrp->AssociatedIrp.IrpCount = MultipleIrpCount;
//
//         if (Wait) {
//
//             MasterIrp->AssociatedIrp.IrpCount += 1;
//         }
//         else if (FlagOn( Context->Wait.Async.ResourceThreadId, 3 )) {
//
//             //
//             //  For async requests if we acquired locks, transition the lock owners to an
//             //  object, since when we return this thread could go away before request
//             //  completion, and the resource package may try to boost priority.
//             //
//
//             if (Context->Wait.Async.Resource != NULL) {
//
//                 ExSetResourceOwnerPointer( Context->Wait.Async.Resource,
//                                            (PVOID)Context->Wait.Async.ResourceThreadId );
//             }
//
//             if (Context->Wait.Async.Resource2 != NULL) {
//
//                 ExSetResourceOwnerPointer( Context->Wait.Async.Resource2,
//                                            (PVOID)Context->Wait.Async.ResourceThreadId );
//             }
//         }
//
//         //
//         //  Back up a copy of the IrpContext flags for later use in async completion.
//         //
//
//         Context->IrpContextFlags = IrpContext->Flags;
//
//         //
//         //  Now that all the dangerous work is done, issue the read requests
//         //
//
//         for (UnwindRunCount = 0;
//              UnwindRunCount < MultipleIrpCount;
//              UnwindRunCount++) {
//
//             Irp = IoRuns[UnwindRunCount].SavedIrp;
//
//             DebugDoit( FatIoCallDriverCount += 1);
//
//             //
//             //  If IoCallDriver returns an error, it has completed the Irp
//             //  and the error will be caught by our completion routines
//             //  and dealt with as a normal IO error.
//             //
//
//             (VOID)FatLowLevelReadWrite( IrpContext,
//                                         Vcb->TargetDeviceObject,
//                                         Irp,
//                                         Vcb );
//         }
//
//         //
//         //  We just issued an IO to the storage stack, update the counters indicating so.
//         //
//
//         if (FatDiskAccountingEnabled) {
//
//             FatUpdateIOCountersPCW( IsAWrite, Length );
//         }
//
//     } _SEH2_FINALLY {
//
//         ULONG i;
//
//         DebugUnwind( FatMultipleAsync );
//
//         //
//         //  Only allocating the spinlock, making the associated Irps
//         //  and allocating the Mdls can fail.
//         //
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             //
//             //  If the driver raised, we are hosed.  He is not supposed to raise,
//             //  and it is impossible for us to figure out how to clean up.
//             //
//
//             if (!ExceptionExpected) {
//                 NT_ASSERT( ExceptionExpected );
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck( 0, 0, 0 );
//             }
//
//             //
//             //  Unwind
//             //
//
//             for (i = 0; i <= UnwindRunCount; i++) {
//
//                 if ( (Irp = IoRuns[i].SavedIrp) != NULL ) {
//
//                     if ( Irp->MdlAddress != NULL ) {
//
//                         IoFreeMdl( Irp->MdlAddress );
//                     }
//
//                     IoFreeIrp( Irp );
//                 }
//             }
//         }
//
//         //
//         //  And return to our caller
//         //
//
//         DebugTrace(-1, Dbg, "FatMultipleAsync -> VOID\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// VOID
// FatSingleAsync (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN LBO Lbo,
//     IN ULONG ByteCount,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine reads or writes one or more contiguous sectors from a device
//     asynchronously, and is used if there is only one read necessary to
//     complete the IRP.  It implements the read by simply filling
//     in the next stack frame in the Irp, and passing it on.  The transfer
//     occurs to the single buffer originally specified in the user request.
//
// Arguments:
//
//     IrpContext->MajorFunction - Supplies either IRP_MJ_READ or IRP_MJ_WRITE.
//
//     Vcb - Supplies the device to read
//
//     Lbo - Supplies the starting Logical Byte Offset to begin reading from
//
//     ByteCount - Supplies the number of bytes to read from the device
//
//     Irp - Supplies the master Irp to associated with the async
//           request.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp;
//     PFAT_IO_CONTEXT Context;
// #ifndef __REACTOS__
//     BOOLEAN IsAWrite = FALSE;
// #endif
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSingleAsync\n", 0);
//     DebugTrace( 0, Dbg, "MajorFunction = %08lx\n", IrpContext->MajorFunction );
//     DebugTrace( 0, Dbg, "Vcb           = %p\n", Vcb );
//     DebugTrace( 0, Dbg, "Lbo           = %08lx\n", Lbo);
//     DebugTrace( 0, Dbg, "ByteCount     = %08lx\n", ByteCount);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp );
//
//     //
//     //  If this I/O originating during FatVerifyVolume, bypass the
//     //  verify logic.
//     //
//
//     if (Vcb->VerifyThread == KeGetCurrentThread() as *mut _) {
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY );
//     }
//
//     //
//     //  Set up the completion routine address in our stack frame.
//     //
//
//     IoSetCompletionRoutine( Irp,
//                             FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) ?
//                             &FatSingleSyncCompletionRoutine :
//                             &FatSingleAsyncCompletionRoutine,
//                             IrpContext->FatIoContext,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Setup the next IRP stack location in the associated Irp for the disk
//     //  driver beneath us.
//     //
//
//     IrpSp = IoGetNextIrpStackLocation( Irp );
//
//     //
//     //  Setup the Stack location to do a read from the disk driver.
//     //
//
//     IrpSp->MajorFunction = IrpContext->MajorFunction;
//     IrpSp->Parameters.Read.Length = ByteCount;
//     IrpSp->Parameters.Read.ByteOffset.QuadPart = Lbo;
//
// #ifndef __REACTOS__
//     IsAWrite = (IrpSp->MajorFunction == IRP_MJ_WRITE);
// #endif
//
//     //
//     //  If this Irp is the result of a WriteThough operation,
//     //  tell the device to write it through.
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH )) {
//
//         SetFlag( IrpSp->Flags, SL_WRITE_THROUGH );
//     }
//
//     //
//     //  If this I/O requires override verify, bypass the verify logic.
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY )) {
//
//         SetFlag( IrpSp->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//     }
//
//     //
//     //  For async requests if we acquired locks, transition the lock owners to an
//     //  object, since when we return this thread could go away before request
//     //  completion, and the resource package may try to boost priority.
//     //
//
//     if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ) &&
//         FlagOn( IrpContext->FatIoContext->Wait.Async.ResourceThreadId, 3 )) {
//
//         Context = IrpContext->FatIoContext;
//
//         if (Context->Wait.Async.Resource != NULL) {
//
//             ExSetResourceOwnerPointer( Context->Wait.Async.Resource,
//                                        (PVOID)Context->Wait.Async.ResourceThreadId );
//         }
//
//         if (Context->Wait.Async.Resource2 != NULL) {
//
//             ExSetResourceOwnerPointer( Context->Wait.Async.Resource2,
//                                        (PVOID)Context->Wait.Async.ResourceThreadId );
//         }
//     }
//
//     //
//     //  Back up a copy of the IrpContext flags for later use in async completion.
//     //
//
//     IrpContext->FatIoContext->IrpContextFlags = IrpContext->Flags;
//
//     //
//     //  Issue the read request
//     //
//
//     DebugDoit( FatIoCallDriverCount += 1);
//
//     //
//     //  If IoCallDriver returns an error, it has completed the Irp
//     //  and the error will be caught by our completion routines
//     //  and dealt with as a normal IO error.
//     //
//
//     (VOID)FatLowLevelReadWrite( IrpContext,
//                                 Vcb->TargetDeviceObject,
//                                 Irp,
//                                 Vcb );
//
//     //
//     //  We just issued an IO to the storage stack, update the counters indicating so.
//     //
//
//     if (FatDiskAccountingEnabled) {
//
//         FatUpdateIOCountersPCW( IsAWrite, ByteCount );
//     }
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatSingleAsync -> VOID\n", 0);
//
//     return;
// }
//
//
// VOID
// FatSingleNonAlignedSync (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PUCHAR Buffer,
//     IN LBO Lbo,
//     IN ULONG ByteCount,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine reads or writes one or more contiguous sectors from a device
//     Synchronously, and does so to a buffer that must come from non paged
//     pool.  It saves a pointer to the Irp's original Mdl, and creates a new
//     one describing the given buffer.  It implements the read by simply filling
//     in the next stack frame in the Irp, and passing it on.  The transfer
//     occurs to the single buffer originally specified in the user request.
//
// Arguments:
//
//     IrpContext->MajorFunction - Supplies either IRP_MJ_READ or IRP_MJ_WRITE.
//
//     Vcb - Supplies the device to read
//
//     Buffer - Supplies a buffer from non-paged pool.
//
//     Lbo - Supplies the starting Logical Byte Offset to begin reading from
//
//     ByteCount - Supplies the number of bytes to read from the device
//
//     Irp - Supplies the master Irp to associated with the async
//           request.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp;
//
//     PMDL Mdl;
//     PMDL SavedMdl;
// #ifndef __REACTOS__
//     BOOLEAN IsAWrite = FALSE;
// #endif
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSingleNonAlignedAsync\n", 0);
//     DebugTrace( 0, Dbg, "MajorFunction = %08lx\n", IrpContext->MajorFunction );
//     DebugTrace( 0, Dbg, "Vcb           = %p\n", Vcb );
//     DebugTrace( 0, Dbg, "Buffer        = %p\n", Buffer );
//     DebugTrace( 0, Dbg, "Lbo           = %08lx\n", Lbo);
//     DebugTrace( 0, Dbg, "ByteCount     = %08lx\n", ByteCount);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp );
//
//     //
//     //  Create a new Mdl describing the buffer, saving the current one in the
//     //  Irp
//     //
//
//     SavedMdl = Irp->MdlAddress;
//
//     Irp->MdlAddress = 0;
//
//     Mdl = IoAllocateMdl( Buffer,
//                          ByteCount,
//                          FALSE,
//                          FALSE,
//                          Irp );
//
//     if (Mdl == NULL) {
//
//         Irp->MdlAddress = SavedMdl;
//
//         FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//     }
//
//     //
//     //  Lock the new Mdl in memory.
//     //
//
//     _SEH2_TRY {
//
//         MmProbeAndLockPages( Mdl, KernelMode, IoWriteAccess );
//
//     } _SEH2_FINALLY {
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             IoFreeMdl( Mdl );
//             Irp->MdlAddress = SavedMdl;
//         }
//     } _SEH2_END;
//
//     //
//     // Set up the completion routine address in our stack frame.
//     //
//
//     IoSetCompletionRoutine( Irp,
//                             &FatSingleSyncCompletionRoutine,
//                             IrpContext->FatIoContext,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Setup the next IRP stack location in the associated Irp for the disk
//     //  driver beneath us.
//     //
//
//     IrpSp = IoGetNextIrpStackLocation( Irp );
//
//     //
//     //  Setup the Stack location to do a read from the disk driver.
//     //
//
//     IrpSp->MajorFunction = IrpContext->MajorFunction;
//     IrpSp->Parameters.Read.Length = ByteCount;
//     IrpSp->Parameters.Read.ByteOffset.QuadPart = Lbo;
//
// #ifndef __REACTOS__
//     IsAWrite = (IrpSp->MajorFunction == IRP_MJ_WRITE);
// #endif
//
//     //
//     //  If this I/O originating during FatVerifyVolume, bypass the
//     //  verify logic.
//     //
//
//     if (Vcb->VerifyThread == KeGetCurrentThread() as *mut _) {
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY );
//     }
//
//     //
//     //  If this I/O requires override verify, bypass the verify logic.
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY )) {
//
//         SetFlag( IrpSp->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//     }
//
//     //
//     //  Issue the read request
//     //
//
//     DebugDoit( FatIoCallDriverCount += 1);
//
//     //
//     //  If IoCallDriver returns an error, it has completed the Irp
//     //  and the error will be caught by our completion routines
//     //  and dealt with as a normal IO error.
//     //
//
//     _SEH2_TRY {
//
//         (VOID)FatLowLevelReadWrite( IrpContext,
//                                     Vcb->TargetDeviceObject,
//                                     Irp,
//                                     Vcb );
//
//         FatWaitSync( IrpContext );
//
//     } _SEH2_FINALLY {
//
//         MmUnlockPages( Mdl );
//         IoFreeMdl( Mdl );
//         Irp->MdlAddress = SavedMdl;
//     } _SEH2_END;
//
//     //
//     //  We just issued an IO to the storage stack, update the counters indicating so.
//     //
//
//     if (FatDiskAccountingEnabled) {
//
//         FatUpdateIOCountersPCW( IsAWrite, ByteCount );
//     }
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatSingleNonAlignedSync -> VOID\n", 0);
//
//     return;
// }
//
//
// VOID
// FatWaitSync (
//     IN PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine waits for one or more previously started I/O requests
//     from the above routines, by simply waiting on the event.
//
// Arguments:
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
//     DebugTrace(+1, Dbg, "FatWaitSync, Context = %p\n", IrpContext->FatIoContext );
//
//     KeWaitForSingleObject( &IrpContext->FatIoContext->Wait.SyncEvent,
//                            Executive, KernelMode, FALSE, NULL );
//
//     KeClearEvent( &IrpContext->FatIoContext->Wait.SyncEvent );
//
//     DebugTrace(-1, Dbg, "FatWaitSync -> VOID\n", 0 );
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatMultiSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatRead/WriteMultipleAsynch.  It must synchronize its operation for
//     multiprocessor environments with itself on all other processors, via
//     a spin lock found via the Context parameter.
//
//     The completion routine has the following responsibilities:
//
//         If the individual request was completed with an error, then
//         this completion routine must see if this is the first error
//         (essentially by Vbo), and if so it must correctly reduce the
//         byte count and remember the error status in the Context.
//
//         If the IrpCount goes to 1, then it sets the event in the Context
//         parameter to signal the caller that all of the asynch requests
//         are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//           Irp will no longer be accessible after this routine returns.)
//
//     Contxt - The context parameter which was specified for all of
//              the multiple asynch I/O requests for this MasterIrp.
//
// Return Value:
//
//     The routine returns STATUS_MORE_PROCESSING_REQUIRED so that we can
//     immediately complete the Master Irp without being in a race condition
//     with the IoCompleteRequest thread trying to decrement the IrpCount in
//     the Master Irp.
//
// --*/
//
// {
//
//     PFAT_IO_CONTEXT Context = Contxt;
//     PIRP MasterIrp = Context->MasterIrp;
//
//     DebugTrace(+1, Dbg, "FatMultiSyncCompletionRoutine, Context = %p\n", Context );
//
//     //
//     //  If we got an error (or verify required), remember it in the Irp
//     //
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
// #if DBG
//         if(!( NT_SUCCESS( FatBreakOnInterestingIoCompletion ) || Irp->IoStatus.Status != FatBreakOnInterestingIoCompletion )) {
//             DbgBreakPoint();
//         }
// #endif
//
// #ifdef SYSCACHE_COMPILE
//         DbgPrint( "FAT SYSCACHE: MultiSync (IRP %08x for Master %08x) -> %08x\n", Irp, MasterIrp, Irp->IoStatus );
// #endif
//
//         MasterIrp->IoStatus = Irp->IoStatus;
//     }
//
//     NT_ASSERT( !(NT_SUCCESS( Irp->IoStatus.Status ) && Irp->IoStatus.Information == 0 ));
//
//     //
//     //  We must do this here since IoCompleteRequest won't get a chance
//     //  on this associated Irp.
//     //
//
//     IoFreeMdl( Irp->MdlAddress );
//     IoFreeIrp( Irp );
//
//     if (InterlockedDecrement(&Context->IrpCount) == 0) {
//
//         FatDoCompletionZero( MasterIrp, Context );
//         KeSetEvent( &Context->Wait.SyncEvent, 0, FALSE );
//     }
//
//     DebugTrace(-1, Dbg, "FatMultiSyncCompletionRoutine -> SUCCESS\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatMultiAsyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatRead/WriteMultipleAsynch.  It must synchronize its operation for
//     multiprocessor environments with itself on all other processors, via
//     a spin lock found via the Context parameter.
//
//     The completion routine has has the following responsibilities:
//
//         If the individual request was completed with an error, then
//         this completion routine must see if this is the first error
//         (essentially by Vbo), and if so it must correctly reduce the
//         byte count and remember the error status in the Context.
//
//         If the IrpCount goes to 1, then it sets the event in the Context
//         parameter to signal the caller that all of the asynch requests
//         are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//           Irp will no longer be accessible after this routine returns.)
//
//     Contxt - The context parameter which was specified for all of
//              the multiple asynch I/O requests for this MasterIrp.
//
// Return Value:
//
//     The routine returns STATUS_MORE_PROCESSING_REQUIRED so that we can
//     immediately complete the Master Irp without being in a race condition
//     with the IoCompleteRequest thread trying to decrement the IrpCount in
//     the Master Irp.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFAT_IO_CONTEXT Context = Contxt;
//     PIRP MasterIrp = Context->MasterIrp;
//     BOOLEAN PostRequest = FALSE;
//
//     DebugTrace(+1, Dbg, "FatMultiAsyncCompletionRoutine, Context = %p\n", Context );
//
//     //
//     //  If we got an error (or verify required), remember it in the Irp
//     //
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
// #if DBG
//         if (!( NT_SUCCESS( FatBreakOnInterestingIoCompletion ) || Irp->IoStatus.Status != FatBreakOnInterestingIoCompletion )) {
//             DbgBreakPoint();
//         }
// #endif
//
// #ifdef SYSCACHE_COMPILE
//         DbgPrint( "FAT SYSCACHE: MultiAsync (IRP %08x for Master %08x) -> %08x\n", Irp, MasterIrp, Irp->IoStatus );
// #endif
//
//         MasterIrp->IoStatus = Irp->IoStatus;
//
//     }
//
//     NT_ASSERT( !(NT_SUCCESS( Irp->IoStatus.Status ) && Irp->IoStatus.Information == 0 ));
//
//     if (InterlockedDecrement(&Context->IrpCount) == 0) {
//
//         FatDoCompletionZero( MasterIrp, Context );
//
//         if (NT_SUCCESS(MasterIrp->IoStatus.Status)) {
//
//             MasterIrp->IoStatus.Information =
//                 Context->Wait.Async.RequestedByteCount;
//
//             NT_ASSERT(MasterIrp->IoStatus.Information != 0);
//
//             //
//             //  Now if this wasn't PagingIo, set either the read or write bit.
//             //
//
//             if (!FlagOn(MasterIrp->Flags, IRP_PAGING_IO)) {
//
//                 SetFlag( Context->Wait.Async.FileObject->Flags,
//                          IoGetCurrentIrpStackLocation(MasterIrp)->MajorFunction == IRP_MJ_READ ?
//                          FO_FILE_FAST_IO_READ : FO_FILE_MODIFIED );
//             }
//
//         } else {
//
//             //
//             // Post STATUS_VERIFY_REQUIRED failures. Only post top level IRPs, because recursive I/Os
//             // cannot process volume verification.
//             //
//
//             if (!FlagOn(Context->IrpContextFlags, IRP_CONTEXT_FLAG_RECURSIVE_CALL) &&
//                 (MasterIrp->IoStatus.Status == STATUS_VERIFY_REQUIRED)) {
//                 PostRequest = TRUE;
//             }
//
//         }
//
//         //
//         //  If this was a special async write, decrement the count.  Set the
//         //  event if this was the final outstanding I/O for the file.  We will
//         //  also want to queue an APC to deal with any error conditionions.
//         //
//     	_Analysis_assume_(!(Context->Wait.Async.NonPagedFcb) &&
//         (ExInterlockedAddUlong( &Context->Wait.Async.NonPagedFcb->OutstandingAsyncWrites,
//                                 0xffffffff,
//                                 &FatData.GeneralSpinLock ) != 1));
//         if ((Context->Wait.Async.NonPagedFcb) &&
//             (ExInterlockedAddUlong( &Context->Wait.Async.NonPagedFcb->OutstandingAsyncWrites,
//                                     0xffffffff,
//                                     &FatData.GeneralSpinLock ) == 1)) {
//
//             KeSetEvent( Context->Wait.Async.NonPagedFcb->OutstandingAsyncEvent, 0, FALSE );
//         }
//
//         //
//         //  Now release the resources.
//         //
//
//         if (Context->Wait.Async.Resource != NULL) {
//
//             ExReleaseResourceForThreadLite( Context->Wait.Async.Resource,
//                                             Context->Wait.Async.ResourceThreadId );
//         }
//
//         if (Context->Wait.Async.Resource2 != NULL) {
//
//             ExReleaseResourceForThreadLite( Context->Wait.Async.Resource2,
//                                             Context->Wait.Async.ResourceThreadId );
//         }
//
//         //
//         //  Mark the master Irp pending
//         //
//
//         IoMarkIrpPending( MasterIrp );
//
//         //
//         //  and finally, free the context record.
//         //
//
//         ExFreePool( Context );
//
//         if (PostRequest) {
//
//             PIRP_CONTEXT IrpContext = NULL;
//
//             _SEH2_TRY {
//
//                 IrpContext = FatCreateIrpContext(Irp, TRUE );
//                 ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL);
//                 FatFsdPostRequest( IrpContext, Irp );
//                 Status = STATUS_MORE_PROCESSING_REQUIRED;
//
//             } _SEH2_EXCEPT( FatExceptionFilter(NULL, _SEH2_GetExceptionInformation()) ) {
//
//                 //
//                 // If we failed to post the IRP, we just have to return the failure
//                 // to the user. :(
//                 //
//
//                 NOTHING;
//             } _SEH2_END;
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatMultiAsyncCompletionRoutine -> SUCCESS\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return Status;
// }
//
//
/* NTSTATUS */
// FatPagingFileErrorHandler (
//     IN PIRP Irp,
//     IN PKEVENT Event OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine attempts to guarantee that the media is marked dirty
//     with the surface test bit if a paging file IO fails.
//
//     The work done here has several basic problems
//
//         1) when paging file writes start failing, this is a good sign
//            that the rest of the system is about to fall down around us
//
//         2) it has no forward progress guarantee
//
//     With Whistler, it is actually quite intentional that we're rejiggering
//     the paging file write path to make forward progress at all times.  This
//     means that the cases where it *does* fail, we're truly seeing media errors
//     and this is probably going to mean the paging file is going to stop working
//     very soon.
//
//     It'd be nice to make this guarantee progress.  It would need
//
//         1) a guaranteed worker thread which can only be used by items which
//            will make forward progress (i.e., not block out this one)
//
//         2) the virtual volume file's pages containing the boot sector and
//            1st FAT entry would have to be pinned resident and have a guaranteed
//            mapping address
//
//         3) mark volume would have to have a stashed irp/mdl and roll the write
//            irp, or use a generalized mechanism to guarantee issue of the irp
//
//         4) the lower stack would have to guarantee progress
//
//     Of these, 1 and 4 may actually exist shortly.
//
// Arguments:
//
//     Irp - Pointer to the associated Irp which is being failed.
//
//     Event - Pointer to optional event to be signalled instead of completing
//         the IRP
//
// Return Value:
//
//     Returns STATUS_MORE_PROCESSING_REQUIRED if we managed to queue off the workitem,
//     STATUS_SUCCESS otherwise.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     //
//     //  If this was a media error, we want to chkdsk /r the next time we boot.
//     //
//
//     if (FsRtlIsTotalDeviceFailure(Irp->IoStatus.Status)) {
//
//         Status = STATUS_SUCCESS;
//
//     } else {
//
//         PCLEAN_AND_DIRTY_VOLUME_PACKET Packet;
//
//         //
//         //  We are going to try to mark the volume needing recover.
//         //  If we can't get pool, oh well....
//         //
//
//         Packet = ExAllocatePoolWithTag(NonPagedPoolNx, sizeof(CLEAN_AND_DIRTY_VOLUME_PACKET), ' taF');
//
//         if ( Packet ) {
//
//             Packet->Vcb = &((PVOLUME_DEVICE_OBJECT)IoGetCurrentIrpStackLocation(Irp)->DeviceObject)->Vcb;
//             Packet->Irp = Irp;
//             Packet->Event = Event;
//
//             ExInitializeWorkItem( &Packet->Item,
//                                   &FatFspMarkVolumeDirtyWithRecover,
//                                   Packet );
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "prefast indicates this is obsolete, but it is ok for fastfat to use it" )
// #endif
//             ExQueueWorkItem( &Packet->Item, CriticalWorkQueue );
//
//             Status = STATUS_MORE_PROCESSING_REQUIRED;
//
//         } else {
//
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     return Status;
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatPagingFileCompletionRoutineCatch (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatPagingFileIo that reuse the master irp (that we have to catch
//     on the way back).  It is always invoked.
//
//     The completion routine has has the following responsibility:
//
//         If the error implies a media problem, it enqueues a
//         worker item to write out the dirty bit so that the next
//         time we run we will do a autochk /r.  This is not forward
//         progress guaranteed at the moment.
//
//         Clean up the Mdl used for this partial request.
//
//     Note that if the Irp is failing, the error code is already where
//     we want it.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//           Irp will no longer be accessible after this routine returns.)
//
//     MasterIrp - Pointer to the master Irp.
//
// Return Value:
//
//     Always returns STATUS_MORE_PROCESSING_REQUIRED.
//
// --*/
//
// {
//     PFAT_PAGING_FILE_CONTEXT Context = (PFAT_PAGING_FILE_CONTEXT) Contxt;
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     DebugTrace(+1, Dbg, "FatPagingFileCompletionRoutineCatch, Context = %p\n", Context );
//
//     //
//     //  Cleanup the existing Mdl, perhaps by returning the reserve.
//     //
//
//     if (Irp->MdlAddress == FatReserveMdl) {
//
//         MmPrepareMdlForReuse( Irp->MdlAddress );
//         KeSetEvent( &FatReserveEvent, 0, FALSE );
//
//     } else {
//
//         IoFreeMdl( Irp->MdlAddress );
//     }
//
//     //
//     //  Restore the original Mdl.
//     //
//
//     Irp->MdlAddress = Context->RestoreMdl;
//
//     DebugTrace(-1, Dbg, "FatPagingFileCompletionRoutine => (done)\n", 0 );
//
//     //
//     //  If the IRP is succeeding or the failure handler did not post off the
//     //  completion, we're done and should set the event to let the master
//     //  know the IRP is his again.
//     //
//
//     if (NT_SUCCESS( Irp->IoStatus.Status ) ||
//         FatPagingFileErrorHandler( Irp, &Context->Event ) == STATUS_SUCCESS) {
//
//         KeSetEvent( &Context->Event, 0, FALSE );
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
//
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatPagingFileCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID MasterIrp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatPagingFileIo.  It should only be invoked on error or cancel.
//
//     The completion routine has has the following responsibility:
//
//         Since the individual request was completed with an error,
//         this completion routine must stuff it into the master irp.
//
//         If the error implies a media problem, it also enqueues a
//         worker item to write out the dirty bit so that the next
//         time we run we will do a autochk /r
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//           Irp will no longer be accessible after this routine returns.)
//
//     MasterIrp - Pointer to the master Irp.
//
// Return Value:
//
//     Always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     DebugTrace(+1, Dbg, "FatPagingFileCompletionRoutine, MasterIrp = %p\n", MasterIrp );
//
//     //
//     //  If we got an error (or verify required), remember it in the Irp
//     //
//
//     NT_ASSERT( !NT_SUCCESS( Irp->IoStatus.Status ));
//
//     //
//     //  If we were invoked with an assoicated Irp, copy the error over.
//     //
//
//     if (Irp != MasterIrp) {
//
//         ((PIRP)MasterIrp)->IoStatus = Irp->IoStatus;
//     }
//
//     DebugTrace(-1, Dbg, "FatPagingFileCompletionRoutine => (done)\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return FatPagingFileErrorHandler( Irp, NULL );
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatSpecialSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for a special set of sub irps
//     that have to work at APC level.
//
//     The completion routine has has the following responsibilities:
//
//         It sets the event passed as the context to signal that the
//         request is done.
//
//     By doing this, the caller will be released before final APC
//     completion with knowledge that the IRP is finished.  Final
//     completion will occur at an indeterminate time after this
//     occurs, and by using this completion routine the caller expects
//     to not have any output or status returned.  A junk user Iosb
//     should be used to capture the status without forcing Io to take
//     an exception on NULL.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the Irp for this request.  (This Irp will no longer
//     be accessible after this routine returns.)
//
//     Contxt - The context parameter which was specified in the call to
//              FatRead/WriteSingleAsynch.
//
// Return Value:
//
//     Currently always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     PFAT_SYNC_CONTEXT SyncContext = (PFAT_SYNC_CONTEXT)Contxt;
//
//     UNREFERENCED_PARAMETER( Irp );
//
//     DebugTrace(+1, Dbg, "FatSpecialSyncCompletionRoutine, Context = %p\n", Contxt );
//
//     SyncContext->Iosb = Irp->IoStatus;
//
//     KeSetEvent( &SyncContext->Event, 0, FALSE );
//
//     DebugTrace(-1, Dbg, "FatSpecialSyncCompletionRoutine -> STATUS_SUCCESS\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatSingleSyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatRead/WriteSingleAsynch.
//
//     The completion routine has has the following responsibilities:
//
//         Copy the I/O status from the Irp to the Context, since the Irp
//         will no longer be accessible.
//
//         It sets the event in the Context parameter to signal the caller
//         that all of the asynch requests are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the Irp for this request.  (This Irp will no longer
//     be accessible after this routine returns.)
//
//     Contxt - The context parameter which was specified in the call to
//              FatRead/WriteSingleAsynch.
//
// Return Value:
//
//     Currently always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     PFAT_IO_CONTEXT Context = Contxt;
//
//     DebugTrace(+1, Dbg, "FatSingleSyncCompletionRoutine, Context = %p\n", Context );
//
//     FatDoCompletionZero( Irp, Context );
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
// #if DBG
//         if(!( NT_SUCCESS( FatBreakOnInterestingIoCompletion ) || Irp->IoStatus.Status != FatBreakOnInterestingIoCompletion )) {
//             DbgBreakPoint();
//         }
// #endif
//
//     }
//
//     NT_ASSERT( !(NT_SUCCESS( Irp->IoStatus.Status ) && Irp->IoStatus.Information == 0 ));
//
//     KeSetEvent( &Context->Wait.SyncEvent, 0, FALSE );
//
//     DebugTrace(-1, Dbg, "FatSingleSyncCompletionRoutine -> STATUS_MORE_PROCESSING_REQUIRED\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// //
// // Internal Support Routine
// //
//
/* NTSTATUS */
// NTAPI
// FatSingleAsyncCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads and writes started via
//     FatRead/WriteSingleAsynch.
//
//     The completion routine has has the following responsibilities:
//
//         Copy the I/O status from the Irp to the Context, since the Irp
//         will no longer be accessible.
//
//         It sets the event in the Context parameter to signal the caller
//         that all of the asynch requests are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the Irp for this request.  (This Irp will no longer
//     be accessible after this routine returns.)
//
//     Contxt - The context parameter which was specified in the call to
//              FatRead/WriteSingleAsynch.
//
// Return Value:
//
//     Currently always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PFAT_IO_CONTEXT Context = Contxt;
//     BOOLEAN PostRequest = FALSE;
//
//     DebugTrace(+1, Dbg, "FatSingleAsyncCompletionRoutine, Context = %p\n", Context );
//
//     //
//     //  Fill in the information field correctedly if this worked.
//     //
//
//     FatDoCompletionZero( Irp, Context );
//
//     if (NT_SUCCESS(Irp->IoStatus.Status)) {
//
//         NT_ASSERT( Irp->IoStatus.Information != 0 );
//         Irp->IoStatus.Information = Context->Wait.Async.RequestedByteCount;
//         NT_ASSERT( Irp->IoStatus.Information != 0 );
//
//         //
//         //  Now if this wasn't PagingIo, set either the read or write bit.
//         //
//
//         if (!FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//
//             SetFlag( Context->Wait.Async.FileObject->Flags,
//                      IoGetCurrentIrpStackLocation(Irp)->MajorFunction == IRP_MJ_READ ?
//                      FO_FILE_FAST_IO_READ : FO_FILE_MODIFIED );
//         }
//
//     } else {
//
// #if DBG
//         if(!( NT_SUCCESS( FatBreakOnInterestingIoCompletion ) || Irp->IoStatus.Status != FatBreakOnInterestingIoCompletion )) {
//             DbgBreakPoint();
//         }
// #endif
//
// #ifdef SYSCACHE_COMPILE
//         DbgPrint( "FAT SYSCACHE: SingleAsync (IRP %08x) -> %08x\n", Irp, Irp->IoStatus );
// #endif
//
//         //
//         // Post STATUS_VERIFY_REQUIRED failures. Only post top level IRPs, because recursive I/Os
//         // cannot process volume verification.
//         //
//
//         if (!FlagOn(Context->IrpContextFlags, IRP_CONTEXT_FLAG_RECURSIVE_CALL) &&
//             (Irp->IoStatus.Status == STATUS_VERIFY_REQUIRED)) {
//             PostRequest = TRUE;
//         }
//
//     }
//
//     //
//     //  If this was a special async write, decrement the count.  Set the
//     //  event if this was the final outstanding I/O for the file.  We will
//     //  also want to queue an APC to deal with any error conditionions.
//     //
//     _Analysis_assume_(!(Context->Wait.Async.NonPagedFcb) &&
//         (ExInterlockedAddUlong( &Context->Wait.Async.NonPagedFcb->OutstandingAsyncWrites,
//                                 0xffffffff,
//                                 &FatData.GeneralSpinLock ) != 1));
//
//     if ((Context->Wait.Async.NonPagedFcb) &&
//         (ExInterlockedAddUlong( &Context->Wait.Async.NonPagedFcb->OutstandingAsyncWrites,
//                                 0xffffffff,
//                                 &FatData.GeneralSpinLock ) == 1)) {
//
//         KeSetEvent( Context->Wait.Async.NonPagedFcb->OutstandingAsyncEvent, 0, FALSE );
//     }
//
//     //
//     //  Now release the resources
//     //
//
//     if (Context->Wait.Async.Resource != NULL) {
//
//         ExReleaseResourceForThreadLite( Context->Wait.Async.Resource,
//                                     Context->Wait.Async.ResourceThreadId );
//     }
//
//     if (Context->Wait.Async.Resource2 != NULL) {
//
//         ExReleaseResourceForThreadLite( Context->Wait.Async.Resource2,
//                                     Context->Wait.Async.ResourceThreadId );
//     }
//
//     //
//     //  Mark the Irp pending
//     //
//
//     IoMarkIrpPending( Irp );
//
//     //
//     //  and finally, free the context record.
//     //
//
//     ExFreePool( Context );
//
//     if (PostRequest) {
//
//         PIRP_CONTEXT IrpContext = NULL;
//
//         _SEH2_TRY {
//
//             IrpContext = FatCreateIrpContext(Irp, TRUE );
//             ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL);
//             FatFsdPostRequest( IrpContext, Irp );
//             Status = STATUS_MORE_PROCESSING_REQUIRED;
//
//         } _SEH2_EXCEPT( FatExceptionFilter(NULL, _SEH2_GetExceptionInformation()) ) {
//
//             //
//             // If we failed to post the IRP, we just have to return the failure
//             // to the user. :(
//             //
//
//             NOTHING;
//         } _SEH2_END;
//     }
//
//
//     DebugTrace(-1, Dbg, "FatSingleAsyncCompletionRoutine -> STATUS_MORE_PROCESSING_REQUIRED\n", 0 );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return Status;
// }
//
//
// VOID
// FatLockUserBuffer (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PIRP Irp,
//     IN LOCK_OPERATION Operation,
//     IN ULONG BufferLength
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locks the specified buffer for the specified type of
//     access.  The file system requires this routine since it does not
//     ask the I/O system to lock its buffers for direct I/O.  This routine
//     may only be called from the Fsd while still in the user context.
//
//     Note that this is the *input/output* buffer.
//
// Arguments:
//
//     Irp - Pointer to the Irp for which the buffer is to be locked.
//
//     Operation - IoWriteAccess for read operations, or IoReadAccess for
//                 write operations.
//
//     BufferLength - Length of user buffer.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PMDL Mdl = NULL;
//
//     PAGED_CODE();
//
//     if (Irp->MdlAddress == NULL) {
//
//         //
//         // Allocate the Mdl, and Raise if we fail.
//         //
//
//         Mdl = IoAllocateMdl( Irp->UserBuffer, BufferLength, FALSE, FALSE, Irp );
//
//         if (Mdl == NULL) {
//
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         //
//         // Now probe the buffer described by the Irp.  If we get an exception,
//         // deallocate the Mdl and return the appropriate "expected" status.
//         //
//
//         _SEH2_TRY {
//
//             MmProbeAndLockPages( Mdl,
//                                  Irp->RequestorMode,
//                                  Operation );
//
//         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//
//             NTSTATUS Status;
//
//             Status = _SEH2_GetExceptionCode();
//
//             IoFreeMdl( Mdl );
//             Irp->MdlAddress = NULL;
//
//             FatRaiseStatus( IrpContext,
//                             FsRtlIsNtstatusExpected(Status) ? Status : STATUS_INVALID_USER_BUFFER );
//         } _SEH2_END;
//     }
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// PVOID
// FatMapUserBuffer (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine conditionally maps the user buffer for the current I/O
//     request in the specified mode.  If the buffer is already mapped, it
//     just returns its address.
//
//     Note that this is the *input/output* buffer.
//
// Arguments:
//
//     Irp - Pointer to the Irp for the request.
//
// Return Value:
//
//     Mapped address
//
// --*/
//
// {
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     PAGED_CODE();
//
//     //
//     // If there is no Mdl, then we must be in the Fsd, and we can simply
//     // return the UserBuffer field from the Irp.
//     //
//
//     if (Irp->MdlAddress == NULL) {
//
//         return Irp->UserBuffer;
//
//     } else {
//
//         PVOID Address = MmGetSystemAddressForMdlSafe( Irp->MdlAddress, NormalPagePriority | MdlMappingNoExecute );
//
//         if (Address == NULL) {
//
//             ExRaiseStatus( STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         return Address;
//     }
// }
//
//
// PVOID
// FatBufferUserBuffer (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PIRP Irp,
//     IN ULONG BufferLength
//     )
//
// /*++
//
// Routine Description:
//
//     This routine conditionally buffers the user buffer for the current I/O
//     request.  If the buffer is already buffered, it just returns its address.
//
//     Note that this is the *input* buffer.
//
// Arguments:
//
//     Irp - Pointer to the Irp for the request.
//
//     BufferLength - Length of user buffer.
//
// Return Value:
//
//     Buffered address.
//
// --*/
//
// {
//     PUCHAR UserBuffer;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     PAGED_CODE();
//
//     //
//     //  Handle the no buffer case.
//     //
//
//     if (BufferLength == 0) {
//
//         return NULL;
//     }
//
//     //
//     //  If there is no system buffer we must have been supplied an Mdl
//     //  describing the users input buffer, which we will now snapshot.
//     //
//
//     if (Irp->AssociatedIrp.SystemBuffer == NULL) {
//
//         UserBuffer = FatMapUserBuffer( IrpContext, Irp );
//
//         Irp->AssociatedIrp.SystemBuffer = FsRtlAllocatePoolWithQuotaTag( NonPagedPoolNx,
//                                                                          BufferLength,
//                                                                          TAG_IO_USER_BUFFER );
//
//         //
//         // Set the flags so that the completion code knows to deallocate the
//         // buffer.
//         //
//
//         Irp->Flags |= (IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER);
//
//         _SEH2_TRY {
//
//             RtlCopyMemory( Irp->AssociatedIrp.SystemBuffer,
//                            UserBuffer,
//                            BufferLength );
//
//         } _SEH2_EXCEPT (EXCEPTION_EXECUTE_HANDLER) {
//
//               NTSTATUS Status;
//
//               Status = _SEH2_GetExceptionCode();
//               FatRaiseStatus( IrpContext,
//                               FsRtlIsNtstatusExpected(Status) ? Status : STATUS_INVALID_USER_BUFFER );
//         } _SEH2_END;
//     }
//
//     return Irp->AssociatedIrp.SystemBuffer;
// }
//
//
/* NTSTATUS */
// FatToggleMediaEjectDisable (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN BOOLEAN PreventRemoval
//     )
//
// /*++
//
// Routine Description:
//
//     The routine either enables or disables the eject button on removable
//     media.
//
// Arguments:
//
//     Vcb - Descibes the volume to operate on
//
//     PreventRemoval - TRUE if we should disable the media eject button.  FALSE
//         if we want to enable it.
//
// Return Value:
//
//     Status of the operation.
//
// --*/
//
// {
//     PIRP Irp;
//     KIRQL SavedIrql;
//     NTSTATUS Status;
//     FAT_SYNC_CONTEXT SyncContext;
//     PREVENT_MEDIA_REMOVAL Prevent;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If PreventRemoval is the same as VCB_STATE_FLAG_REMOVAL_PREVENTED,
//     //  no-op this call, otherwise toggle the state of the flag.
//     //
//
//     KeAcquireSpinLock( &FatData.GeneralSpinLock, &SavedIrql );
//
//     if ((PreventRemoval ^
//          BooleanFlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVAL_PREVENTED)) == 0) {
//
//         KeReleaseSpinLock( &FatData.GeneralSpinLock, SavedIrql );
//
//         return STATUS_SUCCESS;
//
//     } else {
//
//         Vcb->VcbState ^= VCB_STATE_FLAG_REMOVAL_PREVENTED;
//
//         KeReleaseSpinLock( &FatData.GeneralSpinLock, SavedIrql );
//     }
//
//     Prevent.PreventMediaRemoval = PreventRemoval;
//
//     KeInitializeEvent( &SyncContext.Event, NotificationEvent, FALSE );
//
//     //
//     //  We build this IRP using a junk Iosb that will receive the final
//     //  completion status since we won't be around for it.
//     //
//     // We fill in the UserIosb manually below,
//     // So passing NULL for the final parameter is ok in this special case.
//     //
// #ifdef _MSC_VER
// #pragma warning(suppress: 6387)
// #endif
//     Irp = IoBuildDeviceIoControlRequest( IOCTL_DISK_MEDIA_REMOVAL,
//                                          Vcb->TargetDeviceObject,
//                                          &Prevent,
//                                          sizeof(PREVENT_MEDIA_REMOVAL),
//                                          NULL,
//                                          0,
//                                          FALSE,
//                                          NULL,
//                                          NULL );
//
//     if ( Irp != NULL ) {
//
//         //
//         //  Use our special completion routine which will remove the requirement that
//         //  the caller must be below APC level.  All it tells us is that the Irp got
//         //  back, but will not tell us if it was succesful or not.  We don't care,
//         //  and there is of course no fallback if the attempt to prevent removal
//         //  doesn't work for some mysterious reason.
//         //
//         //  Normally, all IO is done at passive level. However, MM needs to be able
//         //  to issue IO with fast mutexes locked down, which raises us to APC.  The
//         //  overlying IRP is set up to complete in yet another magical fashion even
//         //  though APCs are disabled, and any IRPage we do in these cases has to do
//         //  the same.  Marking media dirty (and toggling eject state) is one.
//         //
//
//         Irp->UserIosb = &Irp->IoStatus;
//
//         IoSetCompletionRoutine( Irp,
//                                 FatSpecialSyncCompletionRoutine,
//                                 &SyncContext,
//                                 TRUE,
//                                 TRUE,
//                                 TRUE );
//
//         Status = IoCallDriver( Vcb->TargetDeviceObject, Irp );
//
//         if (Status == STATUS_PENDING) {
//
//             (VOID) KeWaitForSingleObject( &SyncContext.Event,
//                                           Executive,
//                                           KernelMode,
//                                           FALSE,
//                                           NULL );
//
//             Status = SyncContext.Iosb.Status;
//         }
//
//         return Status;
//     }
//
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
//
/* NTSTATUS */
// FatPerformDevIoCtrl (
//     IN PIRP_CONTEXT IrpContext,
//     IN ULONG IoControlCode,
//     IN PDEVICE_OBJECT Device,
//     IN PVOID InputBuffer OPTIONAL,
//     IN ULONG InputBufferLength,
//     OUT PVOID OutputBuffer OPTIONAL,
//     IN ULONG OutputBufferLength,
//     IN BOOLEAN InternalDeviceIoControl,
//     IN BOOLEAN OverrideVerify,
//     OUT PIO_STATUS_BLOCK Iosb OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to perform DevIoCtrl functions internally within
//     the filesystem.  We take the status from the driver and return it to our
//     caller.
//
// Arguments:
//
//     IoControlCode - Code to send to driver.
//
//     Device - This is the device to send the request to.
//
//     OutPutBuffer - Pointer to output buffer.
//
//     OutputBufferLength - Length of output buffer above.
//
//     InternalDeviceIoControl - Indicates if this is an internal or external
//         Io control code.
//
//     OverrideVerify - Indicates if we should tell the driver not to return
//         STATUS_VERIFY_REQUIRED for mount and verify.
//
//     Iosb - If specified, we return the results of the operation here.
//
// Return Value:
//
//     NTSTATUS - Status returned by next lower driver.
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIRP Irp;
//     KEVENT Event;
//     IO_STATUS_BLOCK LocalIosb;
//     PIO_STATUS_BLOCK IosbToUse = &LocalIosb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check if the user gave us an Iosb.
//     //
//
//     if (ARGUMENT_PRESENT( Iosb )) {
//
//         IosbToUse = Iosb;
//     }
//
//     IosbToUse->Status = 0;
//     IosbToUse->Information = 0;
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     Irp = IoBuildDeviceIoControlRequest( IoControlCode,
//                                          Device,
//                                          InputBuffer,
//                                          InputBufferLength,
//                                          OutputBuffer,
//                                          OutputBufferLength,
//                                          InternalDeviceIoControl,
//                                          &Event,
//                                          IosbToUse );
//
//     if (Irp == NULL) {
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (OverrideVerify) {
//
//         SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//     }
//
//     Status = IoCallDriver( Device, Irp );
//
//     //
//     //  We check for device not ready by first checking Status
//     //  and then if status pending was returned, the Iosb status
//     //  value.
//     //
//
//     if (Status == STATUS_PENDING) {
//
//         (VOID) KeWaitForSingleObject( &Event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       (PLARGE_INTEGER)NULL );
//
//         Status = IosbToUse->Status;
//     }
//
//     return Status;
// }
//
// PMDL
// FatBuildZeroMdl (
//     __in PIRP_CONTEXT IrpContext,
//     __in ULONG Length
//     )
// /*++
//
// Routine Description:
//
//     Create an efficient mdl that describe a given length of zeros. We'll only
//     use a one page buffer and make a mdl that maps all the pages back to the single
//     physical page. We'll default to a smaller size buffer down to 1 PAGE if memory
//     is tight. The caller should check the Mdl->ByteCount to see the true size
//
// Arguments:
//
//     Length - The desired length of the zero buffer. We may return less than this
//
// Return Value:
//
//     a MDL if successful / NULL if not
//
// --*/
//
// {
//     PMDL ZeroMdl;
//     ULONG SavedByteCount;
//     PPFN_NUMBER Page;
//     ULONG i;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Spin down trying to get an MDL which can describe our operation.
//     //
//
//     while (TRUE) {
//
//         ZeroMdl = IoAllocateMdl( FatData.ZeroPage, Length, FALSE, FALSE, NULL );
//
//         //
//         //  Throttle ourselves to what we've physically allocated.  Note that
//         //  we could have started with an odd multiple of this number.  If we
//         //  tried for exactly that size and failed, we're toast.
//         //
//
//         if (ZeroMdl || (Length <= PAGE_SIZE)) {
//
//             break;
//         }
//
//         //
//         //  Fallback by half and round down to a page multiple.
//         //
//
//         ASSERT( IrpContext->Vcb->Bpb.BytesPerSector <= PAGE_SIZE );
//         Length = BlockAlignTruncate( Length / 2, PAGE_SIZE );
//         if (Length < PAGE_SIZE) {
//             Length = PAGE_SIZE;
//         }
//     }
//
//     if (ZeroMdl == NULL) {
//         return NULL;
//     }
//
//     //
//     //  If we have throttled all the way down, stop and just build a
//     //  simple MDL describing our previous allocation.
//     //
//
//     if (Length == PAGE_SIZE) {
//
//         MmBuildMdlForNonPagedPool( ZeroMdl );
//         return ZeroMdl;
//     }
//
//     //
//     //  Now we will temporarily lock the allocated pages
//     //  only, and then replicate the page frame numbers through
//     //  the entire Mdl to keep writing the same pages of zeros.
//     //
//     //  It would be nice if Mm exported a way for us to not have
//     //  to pull the Mdl apart and rebuild it ourselves, but this
//     //  is so bizarre a purpose as to be tolerable.
//     //
//
//     SavedByteCount = ZeroMdl->ByteCount;
//     ZeroMdl->ByteCount = PAGE_SIZE;
//     MmBuildMdlForNonPagedPool( ZeroMdl );
//
//     ZeroMdl->MdlFlags &= ~MDL_SOURCE_IS_NONPAGED_POOL;
//     ZeroMdl->MdlFlags |= MDL_PAGES_LOCKED;
//     ZeroMdl->MappedSystemVa = NULL;
//     ZeroMdl->StartVa = NULL;
//     ZeroMdl->ByteCount = SavedByteCount;
//     Page = MmGetMdlPfnArray( ZeroMdl );
//     for (i = 1; i < (ADDRESS_AND_SIZE_TO_SPAN_PAGES( 0, SavedByteCount )); i++) {
//         *(Page + i) = *(Page);
//     }
//
//
//     return ZeroMdl;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
