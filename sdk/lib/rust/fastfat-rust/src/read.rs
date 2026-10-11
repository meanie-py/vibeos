#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcCopyRead, CcFlushCache, CcMdlRead, CcSetReadAheadGranularity, ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExReleaseResourceLite, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetRemainingStackSize, IoMarkIrpPending, KEVENT, KeGetCurrentProcessorNumber, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, PCC_FILE_SIZES, PERESOURCE, PETHREAD, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, PKTHREAD, PsGetCurrentThread, PsGetThreadProcess};
use ntkrnl_sys::ddk::_LOCK_OPERATION_IoWriteAccess as IoWriteAccess;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::cachesup::{FatCompleteMdl};
use crate::strucsup::{FatCreateIrpContext};
use crate::types::{CCB, FAT_BUG_CHECK_READ, FCB, IRP_CONTEXT, LBO, PFCB, PIRP_CONTEXT, PVCB, VBO, VCB, VOLUME_DEVICE_OBJECT, _TYPE_OF_OPEN as TYPE_OF_OPEN, _TYPE_OF_OPEN_DirectoryFile as DirectoryFile};
use ntkrnl_sys::ddk::{CACHE_MANAGER_CALLBACKS, CC_FILE_SIZES, ERESOURCE_THREAD, FILE_OBJECT, IRP, OPLOCK, _EPROCESS, _ETHREAD};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_READ, STATUS_VERIFY_REQUIRED, STATUS_FILE_CORRUPT_ERROR, FO_FILE_FAST_IO_READ};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlPostPagingFileStackOverflow, FsRtlAllocatePoolWithTag};
// re-stitched bodies: kernel types + cross-module helpers
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK as IO_STATUS_BLOCK, _IO_STACK_LOCATION as IO_STACK_LOCATION, KWAIT_REASON, KPROCESSOR_MODE};
use crate::types::{FAT_IO_CONTEXT, FatData, _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen, _TYPE_OF_OPEN_VirtualVolumeFile as VirtualVolumeFile, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _TYPE_OF_OPEN_EaFile as EaFile, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen};
use crate::workque::FatOplockComplete;

// Real NT exports not present in ddk_shims; declared here per the C headers.
extern "system" {
    fn FsRtlPostStackOverflow(Context: *mut core::ffi::c_void, Event: PKEVENT, StackOverflowRoutine: *const core::ffi::c_void);
    fn FsRtlCheckLockForReadAccess(FileLock: *const core::ffi::c_void, Irp: PIRP) -> BOOLEAN;
    fn FatBugCheck(arg1: usize, arg2: usize, arg3: usize) -> !;
    fn FsRtlCheckOplock(Oplock: *mut core::ffi::c_void, Irp: PIRP, Context: *mut core::ffi::c_void, CompletionRoutine: *const core::ffi::c_void, PostIrpRoutine: *const core::ffi::c_void) -> NTSTATUS;
}

// real #define values used by the re-stitched bodies.
const FO_SYNCHRONOUS_IO: u32 = 0x00000002;
const IRP_NOCACHE: u32 = 0x00000001;
const IRP_MN_MDL: u32 = 0x02;
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1; // ((LONGLONG)-1)
// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: POOL_TYPE = 0;
const Flush: u32 = 1; // FAT_FLUSH_TYPE::Flush
// NTSTATUS macro: top two bits set (severity == ERROR == 3).
#[inline]
unsafe fn NT_ERROR(status: NTSTATUS) -> i32 {
    (((status as u32) >> 30) == 3) as i32
}

extern "system" {
    } extern "C" { fn DebugTrace(level: i32, flags: u32, message: *const i8, ...); } extern "system" {
    fn FatPrePostIrp(IrpContext: PIRP_CONTEXT, Irp: PIRP);
    fn DebugDump(msg: *const u8, arg1: u32, arg2: *mut core::ffi::c_void);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB) -> BOOLEAN;
    fn FatAcquireSharedFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB) -> BOOLEAN;
    fn FatAcquireSharedFcbWaitForEx(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB) -> BOOLEAN;
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    fn FatDecodeFileObject(FileObject: *mut FILE_OBJECT, Vcb: *mut *mut VCB, FcbOrDcb: *mut *mut FCB, Ccb: *mut *mut CCB) -> TYPE_OF_OPEN;
    fn FatDeleteIrpContext(IrpContext: *mut IRP_CONTEXT);
    fn FatFlushVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Flush: u32) -> u32;
    fn FatFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatGetFcbOplock(FcbOrDcb: *mut FCB) -> *mut OPLOCK;
    fn FatInitializeCacheMap(FileObject: *mut FILE_OBJECT, Sizes: *mut CC_FILE_SIZES, CacheSupported: BOOLEAN, Callbacks: *mut CACHE_MANAGER_CALLBACKS, FcbOrDcb: *mut FCB);
    fn FatIsFastIoPossible(FcbOrDcb: *mut FCB) -> BOOLEAN;
    fn FatLockUserBuffer(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, AccessMode: u32, BufferLength: u32);
    fn FatLookupFileAllocationSize(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatMapUserBuffer(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> *mut core::ffi::c_void;
    fn FatNonCachedIo(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, FcbOrDcb: *mut FCB, StartingVbo: VBO, BytesToRead: ULONG, ByteCount: ULONG, Flags: ULONG) -> NTSTATUS;
    fn FatNonCachedNonAlignedRead(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, FcbOrDcb: *mut FCB, StartingVbo: VBO, ByteCount: ULONG);
    fn FatNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatPopUpFileCorrupt(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatQuickVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatSingleAsync(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, StartingLbo: LBO, ByteCount: ULONG, Irp: *mut IRP);
    fn FatVerifyFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatWaitSync(IrpContext: *mut IRP_CONTEXT);
    fn CollectReadStats(Vcb: *mut VCB, OpenType: TYPE_OF_OPEN, ByteCount: ULONG);
    fn ExGetCurrentResourceThread() -> *mut ERESOURCE_THREAD;
    fn PsUpdateDiskCounters(Process: *mut _EPROCESS, ReadBytes: u64, WriteBytes: u64, ReadCount: u32, WriteCount: u32, Flags: u32);
    fn CcCopyReadEx(FileObject: *mut FILE_OBJECT, FileOffset: *mut LARGE_INTEGER, Length: ULONG, Wait: BOOLEAN, Buffer: *mut core::ffi::c_void, IoStatus: *mut _IO_STATUS_BLOCK, Thread: *mut _ETHREAD) -> BOOLEAN;
    fn FatPagingFileIo(Irp: PIRP, Fcb: *mut FCB);
    fn FatDeviceIsFatFsdo(device_object: *mut core::ffi::c_void) -> u8; // BOOLEAN fn NodeType(ptr: *mut core::ffi::c_void) -> u16;
    fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS pub unsafe extern "system" fn FatPostStackOverflowRead(irp_context: *mut core::ffi::c_void, irp: PIRP, fcb: *mut core::ffi::c_void) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS pub unsafe extern "system" fn FatPostStackOverflowRead(irp_context: *mut core::ffi::c_void, irp: PIRP, fcb: *mut core::ffi::c_void) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS pub unsafe extern "system" fn FatPostStackOverflowRead(irp_context: *mut core::ffi::c_void, irp: PIRP, fcb: *mut core::ffi::c_void) -> u8; // NTSTATUS #[no_mangle] fn FatIsIrpTopLevel(irp: PIRP) -> u8; // BOOLEAN fn FatCreateIrpContext(irp: PIRP, can_wait: i32) -> *mut core::ffi::c_void; // PIRP_CONTEXT fn CanFsdWait(irp: PIRP) -> u8; // BOOLEAN fn FatCompleteMdl(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS pub unsafe extern "system" fn FatPostStackOverflowRead(irp_context: *mut core::ffi::c_void, irp: PIRP, fcb: *mut core::ffi::c_void) -> u8; // NTSTATUS pub unsafe extern "system" fn FatCommonRead(irp_context: *mut core::ffi::c_void, irp: PIRP) -> u8; // NTSTATUS fn FatRaiseStatus(irp_context: *mut core::ffi::c_void, status: i32);
    fn IoSetTopLevelIrp(irp: PIRP);
    fn NodeType(Fcb: PFCB) -> u32;
}

extern "system" {
    fn CanFsdWait(Irp: PIRP) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_READ;
pub const Dbg: u32 = DEBUG_TRACE_READ;
#[cfg(target_arch = "x86")]
pub const OVERFLOW_READ_THRESHHOLD: u32 = 0xE00;
#[cfg(not(target_arch = "x86"))]
pub const OVERFLOW_READ_THRESHHOLD: u32 = 0x1000;

// Forward declarations for unknown external functions and constants used in this function

// Constants (placeholders/typical values)
const NotificationEvent: u32 = 0;            // EVENT_TYPE::NotificationEvent
const Executive: i32 = 0;                    // KWAIT_REASON::Executive
const IRP_PAGING_IO: u32 = 0x02;      // typical value
const FAT_NTC_VCB: u16 = 0x0501;            // FAT_NTC_VCB
const IRP_CONTEXT_FLAG_VERIFY_READ: u32 = 0x80; // placeholder

// Forward declarations for unknown functions/macros

// Constants with placeholder values (exact definitions expected to be provided elsewhere)
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const IRP_CONTEXT_FLAG_USER_IO: u32 = 0x0400;
const IRP_CONTEXT_STACK_IO_CONTEXT: u32 = 0x0100;
const CCB_FLAG_COMPLETE_DISMOUNT: u32 = 0x00010000;
const CCB_FLAG_SENT_FORMAT_UNIT: u32 = 0x00040000;
const CCB_FLAG_DASD_FLUSH_DONE: u32 = 0x0100;
const CCB_FLAG_ALLOW_EXTENDED_DASD_IO: u32 = 0x2000;
const IRP_CONTEXT_FLAG_OVERRIDE_VERIFY: u32 = 0x1000;
const VCB_STATE_FLAG_LOCKED: u32 = 0x0001;
const READ_AHEAD_GRANULARITY: u32 = 0x10000; // typical
// C: fastfat/nodetype.h `#define TAG_FAT_IO_CONTEXT 'XtaF'` (big-endian).
const TAG_FAT_IO_CONTEXT: u32 = u32::from_be_bytes(*b"XtaF");

const FAT_NTC_FCB: u16 = 0x0502; // FAT NTC FCB node type
const FCB_STATE_PAGING_FILE: u32 = 0x04; // bit 5
const IRP_MN_COMPLETE: u32 = 0x04;
const STATUS_INVALID_USER_BUFFER: i32 = 0xC00000E8_u32 as i32; // NTSTATUS

// read.c `SafeZeroMemory`: a try-except around RtlZeroMemory that turns a fault
// on the user buffer into STATUS_INVALID_USER_BUFFER (the macro's implicit
// IrpContext is passed explicitly).
unsafe fn SafeZeroMemory(IrpContext: *mut IRP_CONTEXT, At: *mut core::ffi::c_void, ByteCount: usize) {
    if crate::seh::try_except(
        &mut (),
        |_| {
            core::ptr::write_bytes(At as *mut u8, 0, ByteCount);
        },
        |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER,
    ).is_err() {
        FatRaiseStatus(IrpContext, STATUS_INVALID_USER_BUFFER);
    }
}

#[repr(C)]
struct PAGING_FILE_OVERFLOW_PACKET {
    Irp: PIRP,
    Fcb: *mut core::ffi::c_void,
}

// FatFsdRead — hand-reconstructed from read.c (its DeepSeek translation dropped;
// the assembled orphan was corrupted).
#[no_mangle]
pub unsafe extern "system" fn FatFsdRead(
    VolumeDeviceObject: *mut VOLUME_DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
    let mut TopLevel: BOOLEAN = 0;

    DebugTrace(1, Dbg as u32, b"FatFsdRead\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();

    //  Quick check for paging file IO (fast path — replicate the fsdo check).
    if FatDeviceIsFatFsdo((*IoGetCurrentIrpStackLocation(Irp)).DeviceObject as *mut core::ffi::c_void) == 0 {
        Fcb = (*(*IoGetCurrentIrpStackLocation(Irp)).FileObject).FsContext as *mut FCB;

        if NodeType(Fcb) == FAT_NTC_FCB as u32
            && FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) != 0
        {
            IoMarkIrpPending(Irp);

            if (IoGetRemainingStackSize() as usize) < (OVERFLOW_READ_THRESHHOLD as usize) {
                let mut Event: KEVENT = core::mem::zeroed();
                let mut Packet: PAGING_FILE_OVERFLOW_PACKET = core::mem::zeroed();
                Packet.Irp = Irp;
                Packet.Fcb = Fcb as *mut core::ffi::c_void;
                KeInitializeEvent(&mut Event, NotificationEvent, 0);
                FsRtlPostPagingFileStackOverflow(
                    &mut Packet as *mut _ as *mut core::ffi::c_void,
                    &mut Event,
                    Some(unsafe { core::mem::transmute::<*const (), _>(FatOverflowPagingFileRead as *const ()) }),
                );
                KeWaitForSingleObject(
                    (&mut Event as *mut KEVENT).cast(),
                    (Executive) as u32,
                    0, // KernelMode
                    0,
                    core::ptr::null_mut(),
                );
            } else {
                FatPagingFileIo(Irp, Fcb);
            }

            FsRtlExitFileSystem();
            return STATUS_PENDING;
        }
    }

    // _SEH2_TRY { .. } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ..)) { FatProcessException }
    let p_top_level = core::ptr::addr_of_mut!(TopLevel);
    let p_ic = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
        *p_top_level = FatIsIrpTopLevel(Irp) as BOOLEAN;
        *p_ic = FatCreateIrpContext(Irp, CanFsdWait(Irp) as i32) as *mut IRP_CONTEXT;
        let IrpContext = *p_ic;

        //  An Mdl complete request must not go through common read.
        if FlagOn((*IrpContext).MinorFunction as u32, IRP_MN_COMPLETE) != 0 {
            *p_status = FatCompleteMdl(IrpContext, Irp);
            return; // try_return
        }

        //  Not enough stack → hand off to the stack-overflow thread.
        if (IoGetRemainingStackSize() as usize) < (OVERFLOW_READ_THRESHHOLD as usize) {
            *p_status = FatPostStackOverflowRead(IrpContext, Irp, Fcb);
            return; // try_return
        }

        *p_status = FatCommonRead(IrpContext, Irp);
    }) {
        Status = crate::fatdata::FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();
    DebugTrace(-1, Dbg as u32, b"FatFsdRead -> %08lx\n\0".as_ptr() as *const i8, Status as usize);

    Status
}


#[no_mangle]
pub unsafe extern "system" fn FatStackOverflowRead(
    Context: *mut core::ffi::c_void,
    Event: PKEVENT,
) {
    let IrpContext: PIRP_CONTEXT = Context as PIRP_CONTEXT;
    let mut SavedVerifyThread: PKTHREAD = core::ptr::null_mut();
    let mut Vcb: PVCB = core::ptr::null_mut();

    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // placeholder  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const IRP_CONTEXT_FLAG_VERIFY_READ: u32 = 0x80; // placeholder
    const FAT_NTC_VCB: u32 = 0x0501; // placeholder
    const STATUS_FILE_DELETED: i32 = 0xC0000123_u32 as i32;

    PAGED_CODE!();

    SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_VERIFY_READ) != 0 {
        let IrpStack = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);
        let FileObject = (*IrpStack).FileObject;
        let Fcb: PFCB = (*FileObject).FsContext as PFCB;

        if NodeType(Fcb) == FAT_NTC_VCB {
            Vcb = Fcb as PVCB;
        } else {
            Vcb = (*Fcb).Vcb;
        }

        NT_ASSERT!((!(*Vcb).VerifyThread.is_null()) as i32);
        SavedVerifyThread = (*Vcb).VerifyThread;
        (*Vcb).VerifyThread = (KeGetCurrentThread()).cast();
    }

    // Do the read operation protected by a try-except clause
    let ic = IrpContext;
    if let Err(code) = crate::seh::fat_try_except(core::ptr::addr_of!(ic), || {
        FatCommonRead(IrpContext, (*IrpContext).OriginatingIrp);
    }) {
        let mut ExceptionCode: NTSTATUS = code;
        if ExceptionCode == STATUS_FILE_DELETED {
            ExceptionCode = STATUS_END_OF_FILE;
            (*IrpContext).ExceptionStatus = ExceptionCode;
            (*(*IrpContext).OriginatingIrp).IoStatus.Information = 0;
        }
        crate::fatdata::FatProcessException(IrpContext, (*IrpContext).OriginatingIrp, ExceptionCode);
    }

    if !SavedVerifyThread.is_null() {
        NT_ASSERT!(((*Vcb).VerifyThread == (KeGetCurrentThread()).cast()) as i32);
        (*Vcb).VerifyThread = SavedVerifyThread;
    }

    KeSetEvent((Event).cast(), 0, 0);
}
#[no_mangle]
pub unsafe extern "system" fn FatOverflowPagingFileRead(
    Context: *mut core::ffi::c_void,
    Event: *mut KEVENT,
) {
    let Packet: *mut PAGING_FILE_OVERFLOW_PACKET = Context as *mut PAGING_FILE_OVERFLOW_PACKET;
    FatPagingFileIo((*Packet).Irp, ((*Packet).Fcb).cast());
    KeSetEvent(Event as *mut core::ffi::c_void, 0, 0);
    return;
}



// ==== re-stitched from bank (dropped by _merge_extern_blocks) ====
#[no_mangle]
pub unsafe extern "system" fn FatPostStackOverflowRead(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    Fcb: PFCB,
) -> NTSTATUS {
    const Dbg: u32 = 0;
    const NotificationEvent: u32 = 0;
    const Executive: i32 = 0;
    const KernelMode: u32 = 0;
    const IRP_PAGING_IO: u32 = 0x02;
    const FAT_NTC_VCB: u16 = 0x0501; // nodetype.h: real value
    const IRP_CONTEXT_FLAG_VERIFY_READ: u32 = 0x80;
    // KEVENT Event; PERESOURCE Resource; PVCB Vcb;
    let mut event: KEVENT = core::mem::zeroed();
    let resource: PERESOURCE;
    let vcb: PVCB;

    PAGED_CODE!();
    DebugTrace(
        0,
        Dbg,
        b"Getting too close to stack limit pass request to Fsp\n\0".as_ptr() as *const i8,
    );

    KeInitializeEvent(&raw mut event, NotificationEvent, 0);

    // Preacquire the resource the read path will require
    if ((*Irp).Flags & IRP_PAGING_IO) != 0 && !(*Fcb).Header.PagingIoResource.is_null() {
        resource = (*Fcb).Header.PagingIoResource;
    } else {
        resource = (*Fcb).Header.Resource;
    }

    if !resource.is_null() {
        ExAcquireResourceSharedLite(resource, 1); // TRUE
    }

    // Determine Vcb
    if (*Fcb).Header.NodeTypeCode == FAT_NTC_VCB as i16 {
        vcb = Fcb as PVCB;
    } else {
        vcb = (*Fcb).Vcb;
    }

    let p_event = &raw mut event;
    crate::seh::try_finally(
        &mut (),
        |_| {
        FatPrePostIrp(IrpContext, Irp);

        if (*vcb).VerifyThread as *mut core::ffi::c_void
            == KeGetCurrentThread() as *mut core::ffi::c_void
        {
            // SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_VERIFY_READ);
            (*IrpContext).Flags |= IRP_CONTEXT_FLAG_VERIFY_READ;
        }

        FsRtlPostStackOverflow(
            IrpContext as *mut core::ffi::c_void,
            p_event,
            FatStackOverflowRead as *const () as *const core::ffi::c_void,
        );

        KeWaitForSingleObject(
            p_event.cast(),
            Executive as KWAIT_REASON,
            KernelMode as KPROCESSOR_MODE,
            0,                        // FALSE
            core::ptr::null_mut(),    // NULL
        );
        },
        |_, _abnormal| {
            if !resource.is_null() {
                ExReleaseResourceLite(resource);
            }
        },
    );

    return STATUS_PENDING;
}

// ==== re-stitched from bank (dropped by _merge_extern_blocks) ====
#[no_mangle]
pub unsafe extern "system" fn FatCommonRead(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS {
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const IRP_CONTEXT_FLAG_USER_IO: u32 = 0x0400;
    const IRP_CONTEXT_STACK_IO_CONTEXT: u32 = 0x0100;
    const CCB_FLAG_COMPLETE_DISMOUNT: u32 = 0x00010000;
    const CCB_FLAG_SENT_FORMAT_UNIT: u32 = 0x00040000;
    const CCB_FLAG_DASD_FLUSH_DONE: u32 = 0x0100;
    const CCB_FLAG_ALLOW_EXTENDED_DASD_IO: u32 = 0x2000;
    const IRP_CONTEXT_FLAG_OVERRIDE_VERIFY: u32 = 0x1000;
    const VCB_STATE_FLAG_LOCKED: u32 = 0x0001;
    const READ_AHEAD_GRANULARITY: u32 = 0x10000;
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut FcbOrDcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    let StartingVbo: VBO;
    let mut ByteCount: ULONG;
    let mut RequestedByteCount: ULONG;

    let IrpSp: *mut IO_STACK_LOCATION;
    let FileObject: *mut FILE_OBJECT;
    let mut TypeOfOpen: TYPE_OF_OPEN;

    let mut PostIrp: BOOLEAN = 0;
    let mut OplockPostIrp: BOOLEAN = 0;

    let mut FcbOrDcbAcquired: BOOLEAN = 0;

    let Wait: BOOLEAN;
    let PagingIo: BOOLEAN;
    let mut NonCachedIo: BOOLEAN;
    let SynchronousIo: BOOLEAN;

    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let mut StackFatIoContext: FAT_IO_CONTEXT = core::mem::zeroed();

    let mut SystemBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    let mut StartingByte: LARGE_INTEGER = core::mem::zeroed();

    // PAGED_CODE() – ignored

    // Irp is reassigned (set to NULL on async PENDING hand-off) inside the body.
    let mut Irp = Irp;

    IrpSp = IoGetCurrentIrpStackLocation(Irp);
    FileObject = (*IrpSp).FileObject;

    Wait = BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);
    PagingIo = BooleanFlagOn((*Irp).Flags, IRP_PAGING_IO);
    NonCachedIo = BooleanFlagOn((*Irp).Flags, IRP_NOCACHE);
    SynchronousIo = BooleanFlagOn((*FileObject).Flags, FO_SYNCHRONOUS_IO);

    DebugTrace(1, Dbg, b"CommonRead\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Irp                   = %p\n\0".as_ptr() as *const i8, Irp);
    DebugTrace(0, Dbg, b"  ->ByteCount           = %8lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Read.Length);
    DebugTrace(0, Dbg, b"  ->ByteOffset.LowPart  = %8lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Read.ByteOffset.u.LowPart);
    DebugTrace(0, Dbg, b"  ->ByteOffset.HighPart = %8lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Read.ByteOffset.u.HighPart);

    StartingByte = (*IrpSp).Parameters.Read.ByteOffset;
    StartingVbo = StartingByte.u.LowPart;
    ByteCount = (*IrpSp).Parameters.Read.Length;
    RequestedByteCount = ByteCount;

    if ByteCount == 0 {
        (*Irp).IoStatus.Information = 0;
        FatCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    TypeOfOpen = FatDecodeFileObject(FileObject, &raw mut Vcb, &raw mut FcbOrDcb, &raw mut Ccb);
    NT_ASSERT!((!Vcb.is_null()) as i32);

    if TypeOfOpen == UserVolumeOpen {
        NonCachedIo = 1;
    }

    NT_ASSERT!((!(NonCachedIo == 0 && TypeOfOpen == VirtualVolumeFile)) as i32);

    if PagingIo != 0 {
        CollectReadStats(Vcb, TypeOfOpen, ByteCount);
        if TypeOfOpen == UserFileOpen {
            SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_USER_IO);
        } else {
            ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_USER_IO);
        }
    }

    NT_ASSERT!((FlagOn((*IrpContext).Flags, IRP_CONTEXT_STACK_IO_CONTEXT) == 0) as i32);

    if NonCachedIo != 0 {
        if (*IrpContext).FatIoContext.is_null() {
            if Wait == 0 {
                (*IrpContext).FatIoContext = FsRtlAllocatePoolWithTag(NonPagedPoolNx, core::mem::size_of::<FAT_IO_CONTEXT>() as ULONG, TAG_FAT_IO_CONTEXT) as *mut FAT_IO_CONTEXT;
            } else {
                (*IrpContext).FatIoContext = &raw mut StackFatIoContext as *mut FAT_IO_CONTEXT;
                SetFlag!((*IrpContext).Flags, IRP_CONTEXT_STACK_IO_CONTEXT);
            }
        }

        core::ptr::write_bytes((*IrpContext).FatIoContext as *mut u8, 0, core::mem::size_of::<FAT_IO_CONTEXT>());

        if Wait != 0 {
            KeInitializeEvent(core::ptr::addr_of_mut!((*(*IrpContext).FatIoContext).Wait.SyncEvent) as *mut _, _EVENT_TYPE_NotificationEvent, 0);
        } else {
            if PagingIo != 0 {
                (*(*IrpContext).FatIoContext).Wait.Async.as_mut().ResourceThreadId = ExGetCurrentResourceThread() as ERESOURCE_THREAD;
            } else {
                (*(*IrpContext).FatIoContext).Wait.Async.as_mut().ResourceThreadId = (((*IrpContext).FatIoContext as ULONG_PTR) | 3) as ERESOURCE_THREAD;
            }
            (*(*IrpContext).FatIoContext).Wait.Async.as_mut().RequestedByteCount = ByteCount;
            (*(*IrpContext).FatIoContext).Wait.Async.as_mut().FileObject = FileObject;
        }
    }

    if TypeOfOpen == VirtualVolumeFile || TypeOfOpen == UserVolumeOpen {
        let mut StartingLbo: LBO;
        StartingLbo = StartingByte.QuadPart;

        DebugTrace(0, Dbg, b"Type of read is User Volume or virtual volume file\n\0".as_ptr() as *const i8, 0);

        if TypeOfOpen == UserVolumeOpen {
            if FlagOn((*Ccb).Flags, CCB_FLAG_COMPLETE_DISMOUNT | CCB_FLAG_SENT_FORMAT_UNIT) == 0 {
                FatQuickVerifyVcb(IrpContext, Vcb);
            }

            if FlagOn((*Ccb).Flags, CCB_FLAG_SENT_FORMAT_UNIT) != 0 {
                SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY);
            }

            if FlagOn((*Ccb).Flags, CCB_FLAG_DASD_FLUSH_DONE) == 0 {
                ExAcquireResourceExclusiveLite(&raw mut (*Vcb).Resource, 1);

                crate::seh::try_finally(
                    &mut (),
                    |_| {
                        // If the volume isn't locked, flush it.
                        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_LOCKED) == 0 {
                            FatFlushVolume(IrpContext, Vcb, Flush);
                        }
                    },
                    |_, _abnormal| {
                        ExReleaseResourceLite(&raw mut (*Vcb).Resource);
                    },
                );
                SetFlag!((*Ccb).Flags, CCB_FLAG_DASD_FLUSH_DONE);
            }

            if FlagOn((*Ccb).Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO) == 0 {
                let mut VolumeSize: LBO;
                VolumeSize = (*Vcb).Bpb.BytesPerSector as LBO *
                             (if (*Vcb).Bpb.Sectors != 0 { (*Vcb).Bpb.Sectors as LBO } else { (*Vcb).Bpb.LargeSectors as LBO });

                if StartingLbo >= VolumeSize {
                    (*Irp).IoStatus.Information = 0;
                    FatCompleteRequest(IrpContext, Irp, STATUS_END_OF_FILE);
                    return STATUS_END_OF_FILE;
                }

                if (ByteCount as i64) > VolumeSize - StartingLbo {
                    ByteCount = (VolumeSize - StartingLbo) as ULONG;
                    RequestedByteCount = ByteCount;
                    if Wait == 0 {
                        (*(*IrpContext).FatIoContext).Wait.Async.as_mut().RequestedByteCount = ByteCount;
                    }
                }
            }

            FatLockUserBuffer(IrpContext, Irp, IoWriteAccess, ByteCount);
        } else {
            (*(*Vcb).Statistics.add(KeGetCurrentProcessorNumber() as usize % FatData.NumberProcessors as usize)).Common.MetaDataDiskReads += 1;
        }

        FatSingleAsync(IrpContext, Vcb, StartingLbo, ByteCount, Irp);

        #[cfg(ntddi_ge_ntddi_win8)]
        {
            if FatDiskAccountingEnabled != 0 {
                let ThreadIssuingIo: *mut ETHREAD = PsGetCurrentThread();
                PsUpdateDiskCounters(PsGetThreadProcess(ThreadIssuingIo), ByteCount as u64, 0, 1, 0, 0);
            }
        }

        if Wait == 0 {
            (*IrpContext).FatIoContext = core::ptr::null_mut();
            FatDeleteIrpContext(IrpContext);
            DebugTrace(-1, Dbg, b"FatNonCachedIo -> STATUS_PENDING\n\0".as_ptr() as *const i8, 0);
            return STATUS_PENDING;
        }

        FatWaitSync(IrpContext);

        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        if !NT_SUCCESS(Status) {
            NT_ASSERT!((!(KeGetCurrentThread() as *mut core::ffi::c_void != (*Vcb).VerifyThread as *mut core::ffi::c_void || Status != STATUS_VERIFY_REQUIRED)) as i32);
            FatNormalizeAndRaiseStatus(IrpContext, Status);
        }

        if SynchronousIo != 0 && PagingIo == 0 {
            (*FileObject).CurrentByteOffset.QuadPart = StartingLbo + (*Irp).IoStatus.Information as i64;
        }

        DebugTrace(-1, Dbg, b"CommonRead -> %08lx\n\0".as_ptr() as *const i8, Status);
        FatCompleteRequest(IrpContext, Irp, Status);
        return Status;
    }

    NT_ASSERT!((!FcbOrDcb.is_null()) as i32);

    if StartingByte.u.HighPart != 0 {
        (*Irp).IoStatus.Information = 0;
        FatCompleteRequest(IrpContext, Irp, STATUS_END_OF_FILE);
        return STATUS_END_OF_FILE;
    }

    // _SEH2_TRY { .. try_exit: .. } _SEH2_FINALLY { .. }
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_post_irp = core::ptr::addr_of_mut!(PostIrp);
    let p_fcb_or_dcb_acquired = core::ptr::addr_of_mut!(FcbOrDcbAcquired);
    crate::seh::try_finally(
        &mut (),
        |_| {
            'try_blk: loop {
                    if TypeOfOpen == UserFileOpen {
                        let mut FileSize: ULONG;
                        let mut ValidDataLength: ULONG;

                        DebugTrace(0, Dbg, b"Type of read is user file open\n\0".as_ptr() as *const i8, 0);

                        if PagingIo == 0 && NonCachedIo != 0
                            && !(*FileObject).SectionObjectPointer.is_null()
                            && !(*(*FileObject).SectionObjectPointer).DataSectionObject.is_null() {

                            let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();

                            // #ifndef REDUCE_SYNCHRONIZATION
                            if FatAcquireExclusiveFcb(IrpContext, FcbOrDcb) == 0 {
                                (*p_post_irp) = 1;
                                break 'try_blk;
                            }
                            ExAcquireResourceExclusiveLite((*FcbOrDcb).Header.PagingIoResource, 1);

                            CcFlushCache((*FileObject).SectionObjectPointer,
                                         &raw mut StartingByte,
                                         ByteCount,
                                         &raw mut IoStatus);

                            ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);
                            FatReleaseFcb(IrpContext, FcbOrDcb);

                            if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
                                (*p_status) = IoStatus.__bindgen_anon_1.Status;
                                break 'try_blk;
                            }

                            ExAcquireResourceExclusiveLite((*FcbOrDcb).Header.PagingIoResource, 1);
                            ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);
                        }

                        if PagingIo != 0 {
                            if ExAcquireResourceSharedLite((*FcbOrDcb).Header.PagingIoResource, 1) == 0 {
                                DebugTrace(0, Dbg, b"Cannot acquire FcbOrDcb = %p shared without waiting\n\0".as_ptr() as *const i8, FcbOrDcb);
                                (*p_post_irp) = 1;
                                break 'try_blk;
                            }
                            if Wait == 0 {
                                (*(*IrpContext).FatIoContext).Wait.Async.as_mut().Resource = (*FcbOrDcb).Header.PagingIoResource;
                            }
                        } else {
                            if Wait == 0 && NonCachedIo != 0 {
                                if FatAcquireSharedFcbWaitForEx(IrpContext, FcbOrDcb) == 0 {
                                    DebugTrace(0, Dbg, b"Cannot acquire FcbOrDcb = %p shared without waiting\n\0".as_ptr() as *const i8, FcbOrDcb);
                                    (*p_post_irp) = 1;
                                    break 'try_blk;
                                }
                                (*(*IrpContext).FatIoContext).Wait.Async.as_mut().Resource = (*FcbOrDcb).Header.Resource;
                            } else {
                                if FatAcquireSharedFcb(IrpContext, FcbOrDcb) == 0 {
                                    DebugTrace(0, Dbg, b"Cannot acquire FcbOrDcb = %p shared without waiting\n\0".as_ptr() as *const i8, FcbOrDcb);
                                    (*p_post_irp) = 1;
                                    break 'try_blk;
                                }
                            }
                        }

                        (*p_fcb_or_dcb_acquired) = 1;
                        FatVerifyFcb(IrpContext, FcbOrDcb);

                        if PagingIo == 0 {
                            (*p_status) = FsRtlCheckOplock(FatGetFcbOplock(FcbOrDcb) as *mut core::ffi::c_void,
                                                      *p_irp,
                                                      IrpContext as *mut core::ffi::c_void,
                                                      FatOplockComplete as *const core::ffi::c_void,
                                                      FatPrePostIrp as *const core::ffi::c_void);

                            if (*p_status) != STATUS_SUCCESS {
                                OplockPostIrp = 1;
                                (*p_post_irp) = 1;
                                break 'try_blk; // NOTHING
                            }

                            (*FcbOrDcb).Header.IsFastIoPossible = FatIsFastIoPossible(FcbOrDcb);

                            if PagingIo == 0 &&
                               FsRtlCheckLockForReadAccess((&raw const (*FcbOrDcb).Specific.Fcb.as_ref().FileLock) as *const core::ffi::c_void, *p_irp) == 0 {
                                (*p_status) = STATUS_FILE_LOCK_CONFLICT;
                                break 'try_blk;
                            }
                        }

                        FileSize = (*FcbOrDcb).Header.FileSize.u.LowPart;
                        ValidDataLength = (*FcbOrDcb).Header.ValidDataLength.u.LowPart;

                        if StartingVbo >= FileSize {
                            DebugTrace(0, Dbg, b"End of File\n\0".as_ptr() as *const i8, 0);
                            (*p_status) = STATUS_END_OF_FILE;
                            break 'try_blk;
                        }

                        if ByteCount > FileSize - StartingVbo {
                            ByteCount = FileSize - StartingVbo;
                            RequestedByteCount = ByteCount;
                            if NonCachedIo != 0 && Wait == 0 {
                                (*(*IrpContext).FatIoContext).Wait.Async.as_mut().RequestedByteCount = RequestedByteCount;
                            }
                        }

                        if NonCachedIo != 0 {
                            let mut SectorSize: ULONG;
                            let mut BytesToRead: ULONG;

                            DebugTrace(0, Dbg, b"Non cached read.\n\0".as_ptr() as *const i8, 0);
                            SectorSize = (*Vcb).Bpb.BytesPerSector as ULONG;

                            if ValidDataLength < (*FcbOrDcb).ValidDataToDisk {
                                ValidDataLength = (*FcbOrDcb).ValidDataToDisk;
                            }

                            if StartingVbo + ByteCount > ValidDataLength {
                                SystemBuffer = FatMapUserBuffer(IrpContext, *p_irp);

                                if StartingVbo < ValidDataLength {
                                    let ZeroingOffset = ((ValidDataLength - StartingVbo) + (SectorSize - 1)) & !(SectorSize - 1);
                                    if ByteCount > ZeroingOffset {
                                        SafeZeroMemory(IrpContext, (SystemBuffer as *mut u8).add(ZeroingOffset as usize) as *mut core::ffi::c_void, (ByteCount - ZeroingOffset) as usize);
                                    }
                                } else {
                                    SafeZeroMemory(IrpContext, SystemBuffer, ByteCount as usize);
                                    (*(*p_irp)).IoStatus.Information = ByteCount as _;
                                    (*p_status) = STATUS_SUCCESS;
                                    break 'try_blk;
                                }
                            }

                            ByteCount = if ValidDataLength - StartingVbo < ByteCount { ValidDataLength - StartingVbo } else { ByteCount };
                            BytesToRead = (ByteCount + (SectorSize - 1)) & !(SectorSize - 1);

                            if (StartingVbo & (SectorSize - 1)) != 0 || BytesToRead > (*IrpSp).Parameters.Read.Length {
                                if Wait == 0 {
                                    (*p_post_irp) = 1;
                                    break 'try_blk;
                                }
                                FatNonCachedNonAlignedRead(IrpContext,
                                                           *p_irp,
                                                           FcbOrDcb,
                                                           StartingVbo,
                                                           ByteCount);
                                // prefast suppress
                                BytesToRead = ByteCount;
                            } else {
                                if FatNonCachedIo(IrpContext,
                                                  *p_irp,
                                                  FcbOrDcb,
                                                  StartingVbo,
                                                  BytesToRead,
                                                  ByteCount,
                                                  0) == STATUS_PENDING {

                                    (*IrpContext).FatIoContext = core::ptr::null_mut();
                                    (*p_irp) = core::ptr::null_mut();
                                    (*p_status) = STATUS_PENDING;
                                    break 'try_blk;
                                }
                            }

                            (*p_status) = (*(*p_irp)).IoStatus.__bindgen_anon_1.Status;
                            if !NT_SUCCESS(*p_status) {
                                NT_ASSERT!((!(KeGetCurrentThread() as *mut core::ffi::c_void != (*Vcb).VerifyThread as *mut core::ffi::c_void || (*p_status) != STATUS_VERIFY_REQUIRED)) as i32);
                                FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                            } else {
                                NT_ASSERT!(((*(*p_irp)).IoStatus.Information == BytesToRead as _) as i32);
                                (*(*p_irp)).IoStatus.Information = RequestedByteCount as _;
                            }

                            break 'try_blk; // try_return(*p_status) effectively ends the loop
                        } else {
                            // Cached case
                            if (*FileObject).PrivateCacheMap.is_null() {
                                DebugTrace(0, Dbg, b"Initialize cache mapping.\n\0".as_ptr() as *const i8, 0);

                                if (*FcbOrDcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
                                    FatLookupFileAllocationSize(IrpContext, FcbOrDcb);
                                }
                                if FileSize > (*FcbOrDcb).Header.AllocationSize.u.LowPart {
                                    FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
                                    FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                                }

                                FatInitializeCacheMap(FileObject,
                                                      &raw mut (*FcbOrDcb).Header.AllocationSize as *mut CC_FILE_SIZES,
                                                      0,
                                                      &raw mut FatData.CacheManagerCallbacks,
                                                      FcbOrDcb);
                                CcSetReadAheadGranularity(FileObject, READ_AHEAD_GRANULARITY);
                            }

                            DebugTrace(0, Dbg, b"Cached read.\n\0".as_ptr() as *const i8, 0);

                            if FlagOn((*IrpContext).MinorFunction as u32, IRP_MN_MDL) == 0 {
                                SystemBuffer = FatMapUserBuffer(IrpContext, *p_irp);

                                // CcCopyReadEx or fallback to CcCopyRead
                                #[cfg(ntddi_ge_ntddi_win8)]
                                let copy_result = CcCopyReadEx(FileObject,
                                                               &raw mut StartingByte,
                                                               ByteCount,
                                                               Wait,
                                                               SystemBuffer,
                                                               &raw mut (*(*p_irp)).IoStatus,
                                                               (*(*p_irp)).Tail.Overlay.Thread);
                                #[cfg(not(ntddi_ge_ntddi_win8))]
                                let copy_result = CcCopyRead(FileObject,
                                                             &raw mut StartingByte,
                                                             ByteCount,
                                                             Wait,
                                                             SystemBuffer,
                                                             &raw mut (*(*p_irp)).IoStatus);

                                if copy_result == 0 {
                                    DebugTrace(0, Dbg, b"Cached Read could not wait\n\0".as_ptr() as *const i8, 0);
                                    (*p_post_irp) = 1;
                                    break 'try_blk;
                                }

                                (*p_status) = (*(*p_irp)).IoStatus.__bindgen_anon_1.Status;
                                NT_ASSERT!(NT_SUCCESS(*p_status) as i32);
                                break 'try_blk; // try_return(*p_status)
                            } else {
                                DebugTrace(0, Dbg, b"MDL read.\n\0".as_ptr() as *const i8, 0);
                                NT_ASSERT!((Wait != 0) as i32);
                                CcMdlRead(FileObject,
                                          &raw mut StartingByte,
                                          ByteCount,
                                          &raw mut (*(*p_irp)).MdlAddress,
                                          &raw mut (*(*p_irp)).IoStatus);
                                (*p_status) = (*(*p_irp)).IoStatus.__bindgen_anon_1.Status;
                                NT_ASSERT!(NT_SUCCESS(*p_status) as i32);
                                break 'try_blk;
                            }
                        }
                    }

                    if TypeOfOpen == DirectoryFile || TypeOfOpen == EaFile {
                        let mut SectorSize: ULONG;
                        SectorSize = (*Vcb).Bpb.BytesPerSector as ULONG;

                        NT_ASSERT!((NonCachedIo != 0 && PagingIo != 0) as i32);
                        NT_ASSERT!((((StartingVbo | ByteCount) & (SectorSize - 1)) == 0) as i32);

                        if StartingVbo >= (*FcbOrDcb).Header.AllocationSize.u.LowPart {
                            DebugTrace(0, Dbg, b"PagingIo dirent started beyond EOF.\n\0".as_ptr() as *const i8, 0);
                            (*(*p_irp)).IoStatus.Information = 0;
                            (*p_status) = STATUS_SUCCESS;
                            break 'try_blk;
                        }

                        if StartingVbo + ByteCount > (*FcbOrDcb).Header.AllocationSize.u.LowPart {
                            DebugTrace(0, Dbg, b"PagingIo dirent extending beyond EOF.\n\0".as_ptr() as *const i8, 0);
                            ByteCount = (*FcbOrDcb).Header.AllocationSize.u.LowPart - StartingVbo;
                        }

                        if FatNonCachedIo(IrpContext,
                                          *p_irp,
                                          FcbOrDcb,
                                          StartingVbo,
                                          ByteCount,
                                          ByteCount,
                                          0) == STATUS_PENDING {
                            (*IrpContext).FatIoContext = core::ptr::null_mut();
                            (*p_irp) = core::ptr::null_mut();
                            (*p_status) = STATUS_PENDING;
                            break 'try_blk;
                        }

                        (*p_status) = (*(*p_irp)).IoStatus.__bindgen_anon_1.Status;
                        if !NT_SUCCESS(*p_status) {
                            NT_ASSERT!((!(KeGetCurrentThread() as *mut core::ffi::c_void != (*Vcb).VerifyThread as *mut core::ffi::c_void || (*p_status) != STATUS_VERIFY_REQUIRED)) as i32);
                            FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                        } else {
                            NT_ASSERT!(((*(*p_irp)).IoStatus.Information == ByteCount as _) as i32);
                        }

                        break 'try_blk;
                    }

                    if TypeOfOpen == UserDirectoryOpen {
                        DebugTrace(0, Dbg, b"CommonRead -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8, 0);
                        (*p_status) = STATUS_INVALID_PARAMETER;
                        break 'try_blk;
                    }

                    DebugDump(b"Illegal TypeOfOpen\n\0".as_ptr() as *const u8, 0, FcbOrDcb as *mut core::ffi::c_void);
                    FatBugCheck(TypeOfOpen as ULONG_PTR, FcbOrDcb as ULONG_PTR, 0);
                    #[allow(unreachable_code)]
                    { break 'try_blk; } // never reached
            }

            // try_exit: NOTHING equivalent
            if !(*p_irp).is_null() {
                if (*p_post_irp) == 0 {
                    let ActualBytesRead: ULONG;
                    DebugTrace(0, Dbg, b"Completing request with status = %08lx\n\0".as_ptr() as *const i8, *p_status);
                    DebugTrace(0, Dbg, b"                   Information = %08lx\n\0".as_ptr() as *const i8, (*(*p_irp)).IoStatus.Information);

                    ActualBytesRead = (*(*p_irp)).IoStatus.Information as ULONG;

                    if SynchronousIo != 0 && PagingIo == 0 {
                        (*FileObject).CurrentByteOffset.u.LowPart = StartingVbo + (if NT_ERROR(*p_status) != 0 { 0 } else { ActualBytesRead });
                    }

                    if NT_SUCCESS(*p_status) && PagingIo == 0 {
                        SetFlag!((*FileObject).Flags, FO_FILE_FAST_IO_READ);
                    }
                } else {
                    DebugTrace(0, Dbg, b"Passing request to Fsp\n\0".as_ptr() as *const i8, 0);
                    if OplockPostIrp == 0 {
                        (*p_status) = FatFsdPostRequest(IrpContext, *p_irp);
                    }
                }
            }
        },
        |_, abnormal| {
            if *p_fcb_or_dcb_acquired != 0 && !(*p_irp).is_null() {
                if PagingIo != 0 {
                    ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);
                } else {
                    FatReleaseFcb(core::ptr::null_mut(), FcbOrDcb);
                }
            }

            if *p_post_irp == 0 {
                if FlagOn((*IrpContext).Flags, IRP_CONTEXT_STACK_IO_CONTEXT) != 0 {
                    if !(*(*IrpContext).FatIoContext).ZeroMdl.is_null() {
                        IoFreeMdl((*(*IrpContext).FatIoContext).ZeroMdl);
                    }
                    ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_STACK_IO_CONTEXT);
                    (*IrpContext).FatIoContext = core::ptr::null_mut();
                }

                if !abnormal {
                    FatCompleteRequest(IrpContext, *p_irp, *p_status);
                }
            }

            DebugTrace(-1, Dbg, b"CommonRead -> %08lx\n\0".as_ptr() as *const i8, *p_status);
        },
    );

    Status
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
//     Read.c
//
// Abstract:
//
//     This module implements the File Read routine for Read called by the
//     dispatch driver.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_READ)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_READ)
//
// //
// //  Define stack overflow read threshhold.  For the x86 we'll use a smaller
// //  threshold than for a risc platform.
// //
// //  Empirically, the limit is a result of the (large) amount of stack
// //  neccesary to throw an exception.
// //
//
// #if defined(_M_IX86)
// #define OVERFLOW_READ_THRESHHOLD         (0xE00)
// #else
// #define OVERFLOW_READ_THRESHHOLD         (0x1000)
// #endif // defined(_M_IX86)
//
//
// //
// //  The following procedures handles read stack overflow operations.
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI
// FatStackOverflowRead (
//     IN PVOID Context,
//     IN PKEVENT Event
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPostStackOverflowRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb
//     );
//
// VOID
// NTAPI
// FatOverflowPagingFileRead (
//     IN PVOID Context,
//     IN PKEVENT Event
//     );
//
// //
// //  VOID
// //  SafeZeroMemory (
// //      IN PUCHAR At,
// //      IN ULONG ByteCount
// //      );
// //
//
// //
// //  This macro just puts a nice little try-except around RtlZeroMemory
// //
//
// #define SafeZeroMemory(AT,BYTE_COUNT) {                            \
//     _SEH2_TRY {                                                    \
//         RtlZeroMemory((AT), (BYTE_COUNT));                         \
//     } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {                    \
//          FatRaiseStatus( IrpContext, STATUS_INVALID_USER_BUFFER ); \
//     } _SEH2_END;                                                   \
// }
//
// //
// //  Macro to increment appropriate performance counters.
// //
//
// #define CollectReadStats(VCB,OPEN_TYPE,BYTE_COUNT) {                                         \
//     PFILESYSTEM_STATISTICS Stats = &(VCB)->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors].Common; \
//     if (((OPEN_TYPE) == UserFileOpen)) {                                                     \
//         Stats->UserFileReads += 1;                                                           \
//         Stats->UserFileReadBytes += (ULONG)(BYTE_COUNT);                                     \
//     } else if (((OPEN_TYPE) == VirtualVolumeFile || ((OPEN_TYPE) == DirectoryFile))) {       \
//         Stats->MetaDataReads += 1;                                                           \
//         Stats->MetaDataReadBytes += (ULONG)(BYTE_COUNT);                                     \
//     }                                                                                        \
// }
//
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatStackOverflowRead)
// #pragma alloc_text(PAGE, FatPostStackOverflowRead)
// #pragma alloc_text(PAGE, FatCommonRead)
// #endif
//
//
// _Function_class_(IRP_MJ_READ)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdRead (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the driver entry to the common read routine for NtReadFile calls.
//     For synchronous requests, the CommonRead is called with Wait == TRUE,
//     which means the request will always be completed in the current thread,
//     and never passed to the Fsp.  If it is not a synchronous request,
//     CommonRead is called with Wait == FALSE, which means the request
//     will be passed to the Fsp only if there is a need to block.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file being Read exists
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The FSD status for the IRP
//
// --*/
//
// {
//     PFCB Fcb = NULL;
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel = FALSE;
//
//     DebugTrace(+1, Dbg, "FatFsdRead\n", 0);
//
//     //
//     //  Call the common Read routine, with blocking allowed if synchronous
//     //
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  We are first going to do a quick check for paging file IO.  Since this
//     //  is a fast path, we must replicate the check for the fsdo.
//     //
//
//     if (!FatDeviceIsFatFsdo( IoGetCurrentIrpStackLocation(Irp)->DeviceObject))  {
//
//         Fcb = (PFCB)(IoGetCurrentIrpStackLocation(Irp)->FileObject->FsContext);
//
//         if ((NodeType(Fcb) == FAT_NTC_FCB) &&
//             FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE)) {
//
//             //
//             //  Do the usual STATUS_PENDING things.
//             //
//
//             IoMarkIrpPending( Irp );
//
//             //
//             //  If there is not enough stack to do this read, then post this
//             //  read to the overflow queue.
//             //
//
//             if (IoGetRemainingStackSize() < OVERFLOW_READ_THRESHHOLD) {
//
//                 KEVENT Event;
//                 PAGING_FILE_OVERFLOW_PACKET Packet;
//
//                 Packet.Irp = Irp;
//                 Packet.Fcb = Fcb;
//
//                 KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//                 FsRtlPostPagingFileStackOverflow( &Packet, &Event, FatOverflowPagingFileRead );
//
//                 //
//                 //  And wait for the worker thread to complete the item
//                 //
//
//                 (VOID) KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, NULL );
//
//             } else {
//
//                 //
//                 //  Perform the actual IO, it will be completed when the io finishes.
//                 //
//
//                 FatPagingFileIo( Irp, Fcb );
//             }
//
//             FsRtlExitFileSystem();
//
//             return STATUS_PENDING;
//         }
//     }
//
//     _SEH2_TRY {
//
//         TopLevel = FatIsIrpTopLevel( Irp );
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         //
//         //  If this is an Mdl complete request, don't go through
//         //  common read.
//         //
//
//         if ( FlagOn(IrpContext->MinorFunction, IRP_MN_COMPLETE) ) {
//
//             DebugTrace(0, Dbg, "Calling FatCompleteMdl\n", 0 );
//             try_return( Status = FatCompleteMdl( IrpContext, Irp ));
//         }
//
//         //
//         //  Check if we have enough stack space to process this request.  If there
//         //  isn't enough then we will pass the request off to the stack overflow thread.
//         //
//
//         if (IoGetRemainingStackSize() < OVERFLOW_READ_THRESHHOLD) {
//
//             DebugTrace(0, Dbg, "Passing StackOverflowRead off\n", 0 );
//             try_return( Status = FatPostStackOverflowRead( IrpContext, Irp, Fcb ) );
//         }
//
//         Status = FatCommonRead( IrpContext, Irp );
//
//     try_exit: NOTHING;
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdRead -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPostStackOverflowRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine posts a read request that could not be processed by
//     the fsp thread because of stack overflow potential.
//
// Arguments:
//
//     Irp - Supplies the request to process.
//
//     Fcb - Supplies the file.
//
// Return Value:
//
//     STATUS_PENDING.
//
// --*/
//
// {
//     KEVENT Event;
//     PERESOURCE Resource;
//     PVCB Vcb;
//
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "Getting too close to stack limit pass request to Fsp\n", 0 );
//
//     //
//     //  Initialize an event and get shared on the resource we will
//     //  be later using the common read.
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  Preacquire the resource the read path will require so we know the
//     //  worker thread can proceed without waiting.
//     //
//
//     if (FlagOn(Irp->Flags, IRP_PAGING_IO) && (Fcb->Header.PagingIoResource != NULL)) {
//
//         Resource = Fcb->Header.PagingIoResource;
//
//     } else {
//
//         Resource = Fcb->Header.Resource;
//     }
//
//     //
//     //  If there are no resources assodicated with the file (case: the virtual
//     //  volume file), it is OK.  No resources will be acquired on the other side
//     //  as well.
//     //
//
//     if (Resource) {
//
//         ExAcquireResourceSharedLite( Resource, TRUE );
//     }
//
//     if (NodeType( Fcb ) == FAT_NTC_VCB) {
//
//         Vcb = (PVCB) Fcb;
//
//     } else {
//
//         Vcb = Fcb->Vcb;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Make the Irp just like a regular post request and
//         //  then send the Irp to the special overflow thread.
//         //  After the post we will wait for the stack overflow
//         //  read routine to set the event that indicates we can
//         //  now release the scb resource and return.
//         //
//
//         FatPrePostIrp( IrpContext, Irp );
//
//         //
//         //  If this read is the result of a verify, we have to
//         //  tell the overflow read routne to temporarily
//         //  hijack the Vcb->VerifyThread field so that reads
//         //  can go through.
//         //
//
//         if (Vcb->VerifyThread == KeGetCurrentThread()) {
//
//             SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_VERIFY_READ);
//         }
//
//         FsRtlPostStackOverflow( IrpContext, &Event, FatStackOverflowRead );
//
//         //
//         //  And wait for the worker thread to complete the item
//         //
//
//         KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, NULL );
//
//     } _SEH2_FINALLY {
//
//         if (Resource) {
//
//             ExReleaseResourceLite( Resource );
//         }
//     } _SEH2_END;
//
//     return STATUS_PENDING;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI
// FatStackOverflowRead (
//     IN PVOID Context,
//     IN PKEVENT Event
//     )
//
// /*++
//
// Routine Description:
//
//     This routine processes a read request that could not be processed by
//     the fsp thread because of stack overflow potential.
//
// Arguments:
//
//     Context - Supplies the IrpContext being processed
//
//     Event - Supplies the event to be signaled when we are done processing this
//         request.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIRP_CONTEXT IrpContext = Context;
//     PKTHREAD SavedVerifyThread = NULL;
//     PVCB Vcb = NULL;
//
//     PAGED_CODE();
//
//     //
//     //  Make it now look like we can wait for I/O to complete
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  If this read was as the result of a verify we have to fake out the
//     //  the Vcb->VerifyThread field.
//     //
//
//     if (FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_VERIFY_READ)) {
//
//         PFCB Fcb = (PFCB)IoGetCurrentIrpStackLocation(IrpContext->OriginatingIrp)->
//                     FileObject->FsContext;
//
//         if (NodeType( Fcb ) == FAT_NTC_VCB) {
//
//             Vcb = (PVCB) Fcb;
//
//         } else {
//
//             Vcb = Fcb->Vcb;
//         }
//
//         NT_ASSERT( Vcb->VerifyThread != NULL );
//         SavedVerifyThread = Vcb->VerifyThread;
//         Vcb->VerifyThread = KeGetCurrentThread();
//     }
//
//     //
//     //  Do the read operation protected by a try-except clause
//     //
//
//     _SEH2_TRY {
//
//         (VOID) FatCommonRead( IrpContext, IrpContext->OriginatingIrp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         NTSTATUS ExceptionCode;
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
//         //
//
//         ExceptionCode = _SEH2_GetExceptionCode();
//
//         if (ExceptionCode == STATUS_FILE_DELETED) {
//
//             IrpContext->ExceptionStatus = ExceptionCode = STATUS_END_OF_FILE;
//             IrpContext->OriginatingIrp->IoStatus.Information = 0;
//         }
//
//         (VOID) FatProcessException( IrpContext, IrpContext->OriginatingIrp, ExceptionCode );
//     } _SEH2_END;
//
//     //
//     //  Restore the original VerifyVolumeThread
//     //
//
//     if (SavedVerifyThread != NULL) {
//
//         NT_ASSERT( Vcb->VerifyThread == KeGetCurrentThread() );
//         Vcb->VerifyThread = SavedVerifyThread;
//     }
//
//     //
//     //  Set the stack overflow item's event to tell the original
//     //  thread that we're done.
//     //
//
//     KeSetEvent( Event, 0, FALSE );
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common read routine for NtReadFile, called from both
//     the Fsd, or from the Fsp if a request could not be completed without
//     blocking in the Fsd.  This routine has no code where it determines
//     whether it is running in the Fsd or Fsp.  Instead, its actions are
//     conditionalized by the Wait input parameter, which determines whether
//     it is allowed to block or not.  If a blocking condition is encountered
//     with Wait == FALSE, however, the request is posted to the Fsp, who
//     always calls with WAIT == TRUE.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     PVCB Vcb;
//     PFCB FcbOrDcb;
//     PCCB Ccb;
//
//     VBO StartingVbo;
//     ULONG ByteCount;
//     ULONG RequestedByteCount;
//
//     PIO_STACK_LOCATION IrpSp;
//     PFILE_OBJECT FileObject;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     BOOLEAN PostIrp = FALSE;
//     BOOLEAN OplockPostIrp = FALSE;
//
//     BOOLEAN FcbOrDcbAcquired = FALSE;
//
//     BOOLEAN Wait;
//     BOOLEAN PagingIo;
//     BOOLEAN NonCachedIo;
//     BOOLEAN SynchronousIo;
//
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     FAT_IO_CONTEXT StackFatIoContext;
//
//     //
//     // A system buffer is only used if we have to access the
//     // buffer directly from the Fsp to clear a portion or to
//     // do a synchronous I/O, or a cached transfer.  It is
//     // possible that our caller may have already mapped a
//     // system buffer, in which case we must remember this so
//     // we do not unmap it on the way out.
//     //
//
//     PVOID SystemBuffer = NULL;
//
//     LARGE_INTEGER StartingByte;
//
//     PAGED_CODE();
//
//     //
//     // Get current Irp stack location.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     FileObject = IrpSp->FileObject;
//
//     //
//     // Initialize the appropriate local variables.
//     //
//
//     Wait          = BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//     PagingIo      = BooleanFlagOn(Irp->Flags, IRP_PAGING_IO);
//     NonCachedIo   = BooleanFlagOn(Irp->Flags,IRP_NOCACHE);
//     SynchronousIo = BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO);
//
//     DebugTrace(+1, Dbg, "CommonRead\n", 0);
//     DebugTrace( 0, Dbg, "  Irp                   = %p\n", Irp);
//     DebugTrace( 0, Dbg, "  ->ByteCount           = %8lx\n", IrpSp->Parameters.Read.Length);
//     DebugTrace( 0, Dbg, "  ->ByteOffset.LowPart  = %8lx\n", IrpSp->Parameters.Read.ByteOffset.LowPart);
//     DebugTrace( 0, Dbg, "  ->ByteOffset.HighPart = %8lx\n", IrpSp->Parameters.Read.ByteOffset.HighPart);
//
//     //
//     //  Extract starting Vbo and offset.
//     //
//
//     StartingByte = IrpSp->Parameters.Read.ByteOffset;
//
//     StartingVbo = StartingByte.LowPart;
//
//     ByteCount = IrpSp->Parameters.Read.Length;
//     RequestedByteCount = ByteCount;
//
//     //
//     //  Check for a null request, and return immediately
//     //
//
//     if (ByteCount == 0) {
//
//         Irp->IoStatus.Information = 0;
//         FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // Extract the nature of the read from the file object, and case on it
//     //
//
//     TypeOfOpen = FatDecodeFileObject(FileObject, &Vcb, &FcbOrDcb, &Ccb);
//
//     NT_ASSERT( Vcb != NULL );
//
//     //
//     //  Save callers who try to do cached IO to the raw volume from themselves.
//     //
//
//     if (TypeOfOpen == UserVolumeOpen) {
//
//         NonCachedIo = TRUE;
//     }
//
//     NT_ASSERT(!(NonCachedIo == FALSE && TypeOfOpen == VirtualVolumeFile));
//
//     //
//     // Collect interesting statistics.  The FLAG_USER_IO bit will indicate
//     // what type of io we're doing in the FatNonCachedIo function.
//     //
//
//     if (PagingIo) {
//         CollectReadStats(Vcb, TypeOfOpen, ByteCount);
//
//         if (TypeOfOpen == UserFileOpen) {
//             SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_USER_IO);
//         } else {
//             ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_USER_IO);
//         }
//     }
//
//     NT_ASSERT(!FlagOn( IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT ));
//
//     //
//     //  Allocate if necessary and initialize a FAT_IO_CONTEXT block for
//     //  all non cached Io.  For synchronous Io we use stack storage,
//     //  otherwise we allocate pool.
//     //
//
//     if (NonCachedIo) {
//
//         if (IrpContext->FatIoContext == NULL) {
//
//             if (!Wait) {
//
//                 IrpContext->FatIoContext =
//                     FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                               sizeof(FAT_IO_CONTEXT),
//                                               TAG_FAT_IO_CONTEXT );
//
//             } else {
//
//                 IrpContext->FatIoContext = &StackFatIoContext;
//
//                 SetFlag( IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT );
//             }
//         }
//
//         RtlZeroMemory( IrpContext->FatIoContext, sizeof(FAT_IO_CONTEXT) );
//
//         if (Wait) {
//
//             KeInitializeEvent( &IrpContext->FatIoContext->Wait.SyncEvent,
//                                NotificationEvent,
//                                FALSE );
//
//         } else {
//
//             if (PagingIo) {
//
//                 IrpContext->FatIoContext->Wait.Async.ResourceThreadId =
//                     ExGetCurrentResourceThread();
//
//             } else {
//
//                 IrpContext->FatIoContext->Wait.Async.ResourceThreadId =
//                         ((ULONG_PTR)IrpContext->FatIoContext) | 3;
//             }
//
//             IrpContext->FatIoContext->Wait.Async.RequestedByteCount =
//                 ByteCount;
//
//             IrpContext->FatIoContext->Wait.Async.FileObject = FileObject;
//         }
//
//     }
//
//
//     //
//     // These two cases correspond to either a general opened volume, ie.
//     // open ("a:"), or a read of the volume file (boot sector + fat(s))
//     //
//
//     if ((TypeOfOpen == VirtualVolumeFile) ||
//         (TypeOfOpen == UserVolumeOpen)) {
//
//         LBO StartingLbo;
//
//         StartingLbo = StartingByte.QuadPart;
//
//         DebugTrace(0, Dbg, "Type of read is User Volume or virtual volume file\n", 0);
//
//         if (TypeOfOpen == UserVolumeOpen) {
//
//             //
//             //  Verify that the volume for this handle is still valid
//             //
//
//             //
//             //  Verify that the volume for this handle is still valid, permitting
//             //  operations to proceed on dismounted volumes via the handle which
//             //  performed the dismount.
//             //
//
//             if (!FlagOn( Ccb->Flags, CCB_FLAG_COMPLETE_DISMOUNT | CCB_FLAG_SENT_FORMAT_UNIT )) {
//
//                 FatQuickVerifyVcb( IrpContext, Vcb );
//             }
//
//             //
//             //  If the caller previously sent a format unit command, then we will allow
//             //  their read/write requests to ignore the verify flag on the device, since some
//             //  devices send a media change event after format unit, but we don't want to
//             //  process it yet since we're probably in the process of formatting the
//             //  media.
//             //
//
//             if (FlagOn( Ccb->Flags, CCB_FLAG_SENT_FORMAT_UNIT )) {
//
//                 SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_OVERRIDE_VERIFY );
//             }
//
//             if (!FlagOn( Ccb->Flags, CCB_FLAG_DASD_FLUSH_DONE )) {
//
//                 (VOID)ExAcquireResourceExclusiveLite( &Vcb->Resource, TRUE );
//
//                 _SEH2_TRY {
//
//                     //
//                     //  If the volume isn't locked, flush it.
//                     //
//
//                     if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_LOCKED)) {
//
//                         FatFlushVolume( IrpContext, Vcb, Flush );
//                     }
//
//                 } _SEH2_FINALLY {
//
//                     ExReleaseResourceLite( &Vcb->Resource );
//                 } _SEH2_END;
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_DASD_FLUSH_DONE );
//             }
//
//             if (!FlagOn( Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO )) {
//
//                 LBO VolumeSize;
//
//                 //
//                 //  Make sure we don't try to read past end of volume,
//                 //  reducing the byte count if necessary.
//                 //
//
//                 VolumeSize = (LBO) Vcb->Bpb.BytesPerSector *
//                              (Vcb->Bpb.Sectors != 0 ? Vcb->Bpb.Sectors :
//                                                       Vcb->Bpb.LargeSectors);
//
//                 if (StartingLbo >= VolumeSize) {
//                     Irp->IoStatus.Information = 0;
//                     FatCompleteRequest( IrpContext, Irp, STATUS_END_OF_FILE );
//                     return STATUS_END_OF_FILE;
//                 }
//
//                 if (ByteCount > VolumeSize - StartingLbo) {
//
//                     ByteCount = RequestedByteCount = (ULONG) (VolumeSize - StartingLbo);
//
//                     //
//                     //  For async reads we had set the byte count in the FatIoContext
//                     //  above, so fix that here.
//                     //
//
//                     if (!Wait) {
//
//                         IrpContext->FatIoContext->Wait.Async.RequestedByteCount =
//                             ByteCount;
//                     }
//                 }
//             }
//
//             //
//             //  For DASD we have to probe and lock the user's buffer
//             //
//
//             FatLockUserBuffer( IrpContext, Irp, IoWriteAccess, ByteCount );
//
//
//         } else {
//
//             //
//             //  Virtual volume file open -- increment performance counters.
//             //
//
//             Vcb->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors].Common.MetaDataDiskReads += 1;
//
//         }
//
//         //
//         //  Read the data and wait for the results
//         //
//
//         FatSingleAsync( IrpContext,
//                         Vcb,
//                         StartingLbo,
//                         ByteCount,
//                         Irp );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         //
//         //  Account for DASD Ios
//         //
//
//         if (FatDiskAccountingEnabled) {
//
//             PETHREAD ThreadIssuingIo = PsGetCurrentThread();
//
//             PsUpdateDiskCounters( PsGetThreadProcess( ThreadIssuingIo ),
//                                   ByteCount,
//                                   0,
//                                   1,
//                                   0,
//                                   0 );
//         }
//
// #endif
//
//         if (!Wait) {
//
//             //
//             //  We, nor anybody else, need the IrpContext any more.
//             //
//
//             IrpContext->FatIoContext = NULL;
//
//             FatDeleteIrpContext( IrpContext );
//
//             DebugTrace(-1, Dbg, "FatNonCachedIo -> STATUS_PENDING\n", 0);
//
//             return STATUS_PENDING;
//         }
//
//         FatWaitSync( IrpContext );
//
//         //
//         //  If the call didn't succeed, raise the error status
//         //
//
//         if (!NT_SUCCESS( Status = Irp->IoStatus.Status )) {
//
//             NT_ASSERT( KeGetCurrentThread() != Vcb->VerifyThread || Status != STATUS_VERIFY_REQUIRED );
//             FatNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//
//         //
//         //  Update the current file position
//         //
//
//         if (SynchronousIo && !PagingIo) {
//             FileObject->CurrentByteOffset.QuadPart =
//                 StartingLbo + Irp->IoStatus.Information;
//         }
//
//         DebugTrace(-1, Dbg, "CommonRead -> %08lx\n", Status );
//
//         FatCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     //
//     //  At this point we know there is an Fcb/Dcb.
//     //
//
//     NT_ASSERT( FcbOrDcb != NULL );
//
//     //
//     //  Check for a non-zero high part offset
//     //
//
//     if ( StartingByte.HighPart != 0 ) {
//
//         Irp->IoStatus.Information = 0;
//         FatCompleteRequest( IrpContext, Irp, STATUS_END_OF_FILE );
//         return STATUS_END_OF_FILE;
//     }
//
//     //
//     //  Use a try-finally to free Fcb/Dcb and buffers on the way out.
//     //
//
//     _SEH2_TRY {
//
//         //
//         // This case corresponds to a normal user read file.
//         //
//
//         if ( TypeOfOpen == UserFileOpen
//             ) {
//
//             ULONG FileSize;
//             ULONG ValidDataLength;
//
//             DebugTrace(0, Dbg, "Type of read is user file open\n", 0);
//
//             //
//             //  If this is a noncached transfer and is not a paging I/O, and
//             //  the file has a data section, then we will do a flush here
//             //  to avoid stale data problems.  Note that we must flush before
//             //  acquiring the Fcb shared since the write may try to acquire
//             //  it exclusive.
//             //
//
//             if (!PagingIo && NonCachedIo
//
//                     &&
//
//                 (FileObject->SectionObjectPointer->DataSectionObject != NULL)) {
//
//                 IO_STATUS_BLOCK IoStatus = {0};
//
// #ifndef REDUCE_SYNCHRONIZATION
//                 if (!FatAcquireExclusiveFcb( IrpContext, FcbOrDcb )) {
//
//                     try_return( PostIrp = TRUE );
//                 }
//
//                 ExAcquireResourceExclusiveLite( FcbOrDcb->Header.PagingIoResource, TRUE );
// #endif //REDUCE_SYNCHRONIZATION
//
//                 CcFlushCache( FileObject->SectionObjectPointer,
//                               &StartingByte,
//                               ByteCount,
//                               &IoStatus );
//
// #ifndef REDUCE_SYNCHRONIZATION
//                 ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
//                 FatReleaseFcb( IrpContext, FcbOrDcb );
// #endif //REDUCE_SYNCHRONIZATION
//
//                 if (!NT_SUCCESS( IoStatus.Status)) {
//
//                     try_return( IoStatus.Status );
//                 }
//
// #ifndef REDUCE_SYNCHRONIZATION
//                 ExAcquireResourceExclusiveLite( FcbOrDcb->Header.PagingIoResource, TRUE );
//                 ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
// #endif //REDUCE_SYNCHRONIZATION
//             }
//
//             //
//             // We need shared access to the Fcb/Dcb before proceeding.
//             //
//
//             if ( PagingIo ) {
//
//                 if (!ExAcquireResourceSharedLite( FcbOrDcb->Header.PagingIoResource,
//                                                   TRUE )) {
//
//                     DebugTrace( 0, Dbg, "Cannot acquire FcbOrDcb = %p shared without waiting\n", FcbOrDcb );
//
//                     try_return( PostIrp = TRUE );
//                 }
//
//                 if (!Wait) {
//
//                     IrpContext->FatIoContext->Wait.Async.Resource =
//                         FcbOrDcb->Header.PagingIoResource;
//                 }
//
//             } else {
//
//                 //
//                 //  If this is async I/O, we will wait if there is an
//                 //  exclusive waiter.
//                 //
//
//                 if (!Wait && NonCachedIo) {
//
//                     if (!FatAcquireSharedFcbWaitForEx( IrpContext, FcbOrDcb )) {
//
//                         DebugTrace( 0,
//                                     Dbg,
//                                     "Cannot acquire FcbOrDcb = %p shared without waiting\n",
//                                     FcbOrDcb );
//
//                         try_return( PostIrp = TRUE );
//                     }
//
//                     IrpContext->FatIoContext->Wait.Async.Resource =
//                         FcbOrDcb->Header.Resource;
//
//                 } else {
//
//                     if (!FatAcquireSharedFcb( IrpContext, FcbOrDcb )) {
//
//                         DebugTrace( 0,
//                                     Dbg,
//                                     "Cannot acquire FcbOrDcb = %p shared without waiting\n",
//                                     FcbOrDcb );
//
//                         try_return( PostIrp = TRUE );
//                     }
//                 }
//             }
//
//             FcbOrDcbAcquired = TRUE;
//
//             //
//             //  Make sure the FcbOrDcb is still good
//             //
//
//             FatVerifyFcb( IrpContext, FcbOrDcb );
//
//             //
//             //  We now check whether we can proceed based on the state of
//             //  the file oplocks.
//             //
//
//             if (!PagingIo) {
//
//                 Status = FsRtlCheckOplock( FatGetFcbOplock(FcbOrDcb),
//                                            Irp,
//                                            IrpContext,
//                                            FatOplockComplete,
//                                            FatPrePostIrp );
//
//                 if (Status != STATUS_SUCCESS) {
//
//                     OplockPostIrp = TRUE;
//                     PostIrp = TRUE;
//                     try_return( NOTHING );
//                 }
//
//                 //
//                 //  Reset the flag indicating if Fast I/O is possible since the oplock
//                 //  check could have broken existing (conflicting) oplocks.
//                 //
//
//                 FcbOrDcb->Header.IsFastIoPossible = FatIsFastIoPossible( FcbOrDcb );
//
//                 //
//                 // We have to check for read access according to the current
//                 // state of the file locks, and set FileSize from the Fcb.
//                 //
//
//                 if (!PagingIo &&
//                     !FsRtlCheckLockForReadAccess( &FcbOrDcb->Specific.Fcb.FileLock,
//                                                   Irp )) {
//
//                     try_return( Status = STATUS_FILE_LOCK_CONFLICT );
//                 }
//             }
//
//             //
//             //  Pick up our sizes and check/trim the IO.
//             //
//
//             FileSize = FcbOrDcb->Header.FileSize.LowPart;
//             ValidDataLength = FcbOrDcb->Header.ValidDataLength.LowPart;
//
//             //
//             // If the read starts beyond End of File, return EOF.
//             //
//
//             if (StartingVbo >= FileSize) {
//
//                 DebugTrace( 0, Dbg, "End of File\n", 0 );
//
//                 try_return ( Status = STATUS_END_OF_FILE );
//             }
//
//             //
//             //  If the read extends beyond EOF, truncate the read
//             //
//
//             if (ByteCount > FileSize - StartingVbo) {
//
//                 ByteCount = RequestedByteCount = FileSize - StartingVbo;
//
//                 if (NonCachedIo && !Wait) {
//
//                     IrpContext->FatIoContext->Wait.Async.RequestedByteCount =
//                         RequestedByteCount;
//
//                 }
//             }
//
//             //
//             // HANDLE THE NON-CACHED CASE
//             //
//
//             if ( NonCachedIo ) {
//
//                 ULONG SectorSize;
//                 ULONG BytesToRead;
//
//                 DebugTrace(0, Dbg, "Non cached read.\n", 0);
//
//                 //
//                 //  Get the sector size
//                 //
//
//                 SectorSize = (ULONG)Vcb->Bpb.BytesPerSector;
//
//                 //
//                 //  Start by zeroing any part of the read after Valid Data
//                 //
//
//                 if (ValidDataLength < FcbOrDcb->ValidDataToDisk) {
//
//                     ValidDataLength = FcbOrDcb->ValidDataToDisk;
//                 }
//
//
//                 if ( StartingVbo + ByteCount > ValidDataLength ) {
//
//                     SystemBuffer = FatMapUserBuffer( IrpContext, Irp );
//
//                     if (StartingVbo < ValidDataLength) {
//
//                         ULONG ZeroingOffset;
//
//                         //
//                         //  Now zero out the user's request sector aligned beyond
//                         //  vdl.  We will handle the straddling sector at completion
//                         //  time via the bytecount reduction which immediately
//                         //  follows this check.
//                         //
//                         //  Note that we used to post in this case for async requests.
//                         //  Note also that if the request was wholly beyond VDL that
//                         //  we did not post, therefore this is consistent.  Synchronous
//                         //  zeroing is fine for async requests.
//                         //
//
//                         ZeroingOffset = ((ValidDataLength - StartingVbo) + (SectorSize - 1))
//                                                                         & ~(SectorSize - 1);
//
//                         //
//                         //  If the offset is at or above the byte count, no harm: just means
//                         //  that the read ends in the last sector and the zeroing will be
//                         //  done at completion.
//                         //
//
//                         if (ByteCount > ZeroingOffset) {
//
//                             SafeZeroMemory( (PUCHAR) SystemBuffer + ZeroingOffset,
//                                             ByteCount - ZeroingOffset);
//
//                         }
//
//                     } else {
//
//                         //
//                         //  All we have to do now is sit here and zero the
//                         //  user's buffer, no reading is required.
//                         //
//
//                         SafeZeroMemory( (PUCHAR)SystemBuffer, ByteCount );
//
//                         Irp->IoStatus.Information = ByteCount;
//
//
//                         try_return ( Status = STATUS_SUCCESS );
//                     }
//                 }
//
//
//                 //
//                 //  Reduce the byte count to actually read if it extends beyond
//                 //  Valid Data Length
//                 //
//
//                 ByteCount = (ValidDataLength - StartingVbo < ByteCount) ?
//                              ValidDataLength - StartingVbo : ByteCount;
//
//                 //
//                 //  Round up to a sector boundary, and remember that if we are
//                 //  reading extra bytes we will zero them out during completion.
//                 //
//
//                 BytesToRead = (ByteCount + (SectorSize - 1))
//                                         & ~(SectorSize - 1);
//
//                 //
//                 //  Just to help alleviate confusion.  At this point:
//                 //
//                 //  RequestedByteCount - is the number of bytes originally
//                 //                       taken from the Irp, but constrained
//                 //                       to filesize.
//                 //
//                 //  ByteCount -          is RequestedByteCount constrained to
//                 //                       ValidDataLength.
//                 //
//                 //  BytesToRead -        is ByteCount rounded up to sector
//                 //                       boundry.  This is the number of bytes
//                 //                       that we must physically read.
//                 //
//
//                 //
//                 //  If this request is not properly aligned, or extending
//                 //  to a sector boundary would overflow the buffer, send it off
//                 //  on a special-case path.
//                 //
//
//                 if ( (StartingVbo & (SectorSize - 1)) ||
//                      (BytesToRead > IrpSp->Parameters.Read.Length) ) {
//
//                     //
//                     //  If we can't wait, we must post this.
//                     //
//
//                     if (!Wait) {
//
//                         try_return( PostIrp = TRUE );
//                     }
//
//                     //
//                     //  Do the physical read
//                     //
//
//                     FatNonCachedNonAlignedRead( IrpContext,
//                                                 Irp,
//                                                 FcbOrDcb,
//                                                 StartingVbo,
//                                                 ByteCount );
//
//                     //
//                     //  Set BytesToRead to ByteCount to satify the following ASSERT.
//                     //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//                     BytesToRead = ByteCount;
//
//                 } else {
//
//                     //
//                     //  Perform the actual IO
//                     //
//
//
//                     if (FatNonCachedIo( IrpContext,
//                                         Irp,
//                                         FcbOrDcb,
//                                         StartingVbo,
//                                         BytesToRead,
//                                         ByteCount,
//                                         0) == STATUS_PENDING) {
//
//
//                         IrpContext->FatIoContext = NULL;
//
//                         Irp = NULL;
//
//                         try_return( Status = STATUS_PENDING );
//                     }
//                 }
//
//                 //
//                 //  If the call didn't succeed, raise the error status
//                 //
//
//                 if (!NT_SUCCESS( Status = Irp->IoStatus.Status )) {
//
//                     NT_ASSERT( KeGetCurrentThread() != Vcb->VerifyThread || Status != STATUS_VERIFY_REQUIRED );
//                     FatNormalizeAndRaiseStatus( IrpContext, Status );
//
//                 } else {
//
//                     //
//                     //  Else set the Irp information field to reflect the
//                     //  entire desired read.
//                     //
//
//                     NT_ASSERT( Irp->IoStatus.Information == BytesToRead );
//
//                     Irp->IoStatus.Information = RequestedByteCount;
//                 }
//
//                 //
//                 // The transfer is complete.
//                 //
//
//                 try_return( Status );
//
//             }   // if No Intermediate Buffering
//
//
//             //
//             // HANDLE CACHED CASE
//             //
//
//             else {
//
//                 //
//                 // We delay setting up the file cache until now, in case the
//                 // caller never does any I/O to the file, and thus
//                 // FileObject->PrivateCacheMap == NULL.
//                 //
//
//                 if (FileObject->PrivateCacheMap == NULL) {
//
//                     DebugTrace(0, Dbg, "Initialize cache mapping.\n", 0);
//
//                     //
//                     //  Get the file allocation size, and if it is less than
//                     //  the file size, raise file corrupt error.
//                     //
//
//                     if (FcbOrDcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//                         FatLookupFileAllocationSize( IrpContext, FcbOrDcb );
//                     }
//
//                     if ( FileSize > FcbOrDcb->Header.AllocationSize.LowPart ) {
//
//                         FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//
//                         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                     }
//
//                     //
//                     //  Now initialize the cache map.
//                     //
//
//                     FatInitializeCacheMap( FileObject,
//                                            (PCC_FILE_SIZES)&FcbOrDcb->Header.AllocationSize,
//                                            FALSE,
//                                            &FatData.CacheManagerCallbacks,
//                                            FcbOrDcb );
//
//                     CcSetReadAheadGranularity( FileObject, READ_AHEAD_GRANULARITY );
//                 }
//
//
//                 //
//                 // DO A NORMAL CACHED READ, if the MDL bit is not set,
//                 //
//
//                 DebugTrace(0, Dbg, "Cached read.\n", 0);
//
//                 if (!FlagOn(IrpContext->MinorFunction, IRP_MN_MDL)) {
//
//                     //
//                     //  Get hold of the user's buffer.
//                     //
//
//                     SystemBuffer = FatMapUserBuffer( IrpContext, Irp );
//
//                     //
//                     // Now try to do the copy.
//                     //
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                     if (!CcCopyReadEx( FileObject,
//                                        &StartingByte,
//                                        ByteCount,
//                                        Wait,
//                                        SystemBuffer,
//                                        &Irp->IoStatus,
//                                        Irp->Tail.Overlay.Thread )) {
// #else
//                     if (!CcCopyRead( FileObject,
//                                      &StartingByte,
//                                      ByteCount,
//                                      Wait,
//                                      SystemBuffer,
//                                      &Irp->IoStatus )) {
// #endif
//
//                         DebugTrace( 0, Dbg, "Cached Read could not wait\n", 0 );
//
//                         try_return( PostIrp = TRUE );
//                     }
//
//                     Status = Irp->IoStatus.Status;
//
//                     NT_ASSERT( NT_SUCCESS( Status ));
//
//                     try_return( Status );
//                 }
//
//
//                 //
//                 //  HANDLE A MDL READ
//                 //
//
//                 else {
//
//                     DebugTrace(0, Dbg, "MDL read.\n", 0);
//
//                     NT_ASSERT( Wait );
//
//                     CcMdlRead( FileObject,
//                                &StartingByte,
//                                ByteCount,
//                                &Irp->MdlAddress,
//                                &Irp->IoStatus );
//
//                     Status = Irp->IoStatus.Status;
//
//                     NT_ASSERT( NT_SUCCESS( Status ));
//
//                     try_return( Status );
//                 }
//             }
//         }
//
//         //
//         //  These cases correspond to a system read directory file or
//         //  ea file.
//         //
//
//         if (( TypeOfOpen == DirectoryFile ) || ( TypeOfOpen == EaFile)
//             ) {
//
//             ULONG SectorSize;
//
//
// #if FASTFATDBG
//             if ( TypeOfOpen == DirectoryFile ) {
//                 DebugTrace(0, Dbg, "Type of read is directoryfile\n", 0);
//             } else if ( TypeOfOpen == EaFile) {
//                 DebugTrace(0, Dbg, "Type of read is eafile\n", 0);
//             }
//
// #endif
//
//             //
//             //  For the noncached case, assert that everything is sector
//             //  alligned.
//             //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//             SectorSize = (ULONG)Vcb->Bpb.BytesPerSector;
//
//             //
//             //  We make several assumptions about these two types of files.
//             //  Make sure all of them are true.
//             //
//
//             NT_ASSERT( NonCachedIo && PagingIo );
//             NT_ASSERT( ((StartingVbo | ByteCount) & (SectorSize - 1)) == 0 );
//
//
//                 //
//                 //  These calls must allways be within the allocation size
//                 //
//
//                 if (StartingVbo >= FcbOrDcb->Header.AllocationSize.LowPart) {
//
//                     DebugTrace( 0, Dbg, "PagingIo dirent started beyond EOF.\n", 0 );
//
//                     Irp->IoStatus.Information = 0;
//
//                     try_return( Status = STATUS_SUCCESS );
//                 }
//
//                 if ( StartingVbo + ByteCount > FcbOrDcb->Header.AllocationSize.LowPart ) {
//
//                     DebugTrace( 0, Dbg, "PagingIo dirent extending beyond EOF.\n", 0 );
//                     ByteCount = FcbOrDcb->Header.AllocationSize.LowPart - StartingVbo;
//                 }
//
//
//             //
//             //  Perform the actual IO
//             //
//
//             if (FatNonCachedIo( IrpContext,
//                                 Irp,
//                                 FcbOrDcb,
//                                 StartingVbo,
//                                 ByteCount,
//                                 ByteCount,
//                                 0 ) == STATUS_PENDING) {
//
//                 IrpContext->FatIoContext = NULL;
//
//                 Irp = NULL;
//
//                 try_return( Status = STATUS_PENDING );
//             }
//
//             //
//             //  If the call didn't succeed, raise the error status
//             //
//
//             if (!NT_SUCCESS( Status = Irp->IoStatus.Status )) {
//
//                 NT_ASSERT( KeGetCurrentThread() != Vcb->VerifyThread || Status != STATUS_VERIFY_REQUIRED );
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//
//             } else {
//
//                 NT_ASSERT( Irp->IoStatus.Information == ByteCount );
//             }
//
//             try_return( Status );
//         }
//
//         //
//         // This is the case of a user who openned a directory. No reading is
//         // allowed.
//         //
//
//         if ( TypeOfOpen == UserDirectoryOpen ) {
//
//             DebugTrace( 0, Dbg, "CommonRead -> STATUS_INVALID_PARAMETER\n", 0);
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  If we get this far, something really serious is wrong.
//         //
//
//         DebugDump("Illegal TypeOfOpen\n", 0, FcbOrDcb );
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( TypeOfOpen, (ULONG_PTR) FcbOrDcb, 0 );
//
//     try_exit: NOTHING;
//
//         //
//         //  If the request was not posted and there's an Irp, deal with it.
//         //
//
//         if ( Irp ) {
//
//             if ( !PostIrp ) {
//
//                 ULONG ActualBytesRead;
//
//                 DebugTrace( 0, Dbg, "Completing request with status = %08lx\n",
//                             Status);
//
//                 DebugTrace( 0, Dbg, "                   Information = %08lx\n",
//                             Irp->IoStatus.Information);
//
//                 //
//                 //  Record the total number of bytes actually read
//                 //
//
//                 ActualBytesRead = (ULONG)Irp->IoStatus.Information;
//
//                 //
//                 //  If the file was opened for Synchronous IO, update the current
//                 //  file position.
//                 //
//
//                 if (SynchronousIo && !PagingIo) {
//
//                     FileObject->CurrentByteOffset.LowPart =
//                          StartingVbo + (NT_ERROR( Status ) ? 0 : ActualBytesRead);
//                 }
//
//                 //
//                 //  If this was not PagingIo, mark that the last access
//                 //  time on the dirent needs to be updated on close.
//                 //
//
//                 if (NT_SUCCESS(Status) && !PagingIo) {
//
//                     SetFlag( FileObject->Flags, FO_FILE_FAST_IO_READ );
//                 }
//
//             } else {
//
//                 DebugTrace( 0, Dbg, "Passing request to Fsp\n", 0 );
//
//                 if (!OplockPostIrp) {
//
//                     Status = FatFsdPostRequest( IrpContext, Irp );
//                 }
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonRead );
//
//         //
//         // If the FcbOrDcb has been acquired, release it.
//         //
//
//         if (FcbOrDcbAcquired && Irp) {
//
//             if ( PagingIo ) {
//
//                 ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
//
//             } else {
//
//                 FatReleaseFcb( NULL, FcbOrDcb );
//             }
//         }
//
//         //
//         //  Complete the request if we didn't post it and no exception
//         //
//         //  Note that FatCompleteRequest does the right thing if either
//         //  IrpContext or Irp are NULL
//         //
//
//         if (!PostIrp) {
//
//             //
//             //  If we had a stack io context, we have to make sure the contents
//             //  are cleaned up before we leave.
//             //
//             //  At present with zero mdls, this will only really happen on exceptional
//             //  termination where we failed to dispatch the IO. Cleanup of zero mdls
//             //  normally occurs during completion, but when we bail we must make sure
//             //  the cleanup occurs here or the fatiocontext will go out of scope.
//             //
//             //  If the operation was posted, cleanup occured there.
//             //
//
//             if (FlagOn(IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT)) {
//
//                 if (IrpContext->FatIoContext->ZeroMdl) {
//                     IoFreeMdl( IrpContext->FatIoContext->ZeroMdl );
//                 }
//
//                 ClearFlag(IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT);
//                 IrpContext->FatIoContext = NULL;
//             }
//
//             if (!_SEH2_AbnormalTermination()) {
//
//                 FatCompleteRequest( IrpContext, Irp, Status );
//             }
//         }
//
//         DebugTrace(-1, Dbg, "CommonRead -> %08lx\n", Status );
//     } _SEH2_END;
//
//     return Status;
// }
//
// //
// //  Local support routine
// //
//
// VOID
// NTAPI
// FatOverflowPagingFileRead (
//     IN PVOID Context,
//     IN PKEVENT Event
//     )
//
// /*++
//
// Routine Description:
//
//     The routine simply call FatPagingFileIo.  It is invoked in cases when
//     there was not enough stack space to perform the pagefault in the
//     original thread.  It is also responsible for freeing the packet pool.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
//     Fcb - Supplies the paging file Fcb, since we have it handy.
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     PPAGING_FILE_OVERFLOW_PACKET Packet = Context;
//
//     FatPagingFileIo( Packet->Irp, Packet->Fcb );
//
//     //
//     //  Set the stack overflow item's event to tell the original
//     //  thread that we're done.
//     //
//
//     KeSetEvent( Event, 0, FALSE );
//
//     return;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// fatprocs.h flag macros, inline at the declared widths (were C stubs taking
// ULONG_PTR, which read garbage upper bits from a narrower argument).
#[inline(always)]
unsafe fn BooleanFlagOn(flags: u32, flag: u32) -> BOOLEAN {
    ((flags as u64 & flag as u64) != 0) as BOOLEAN
}

#[inline(always)]
unsafe fn FlagOn(flags: u32, flag: u32) -> u32 {
    (flags as u64 & flag as u64) as u32
}
