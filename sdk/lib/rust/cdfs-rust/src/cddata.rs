use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::ddk::{FAST_IO_DISPATCH, IoGetCurrentIrpStackLocation, IoGetDeviceToVerify, IoGetStackLimits, IoGetTopLevelIrp, IoMarkIrpPending, IoRaiseHardError, IoSetDeviceToVerify, IoSetTopLevelIrp, KeAreAllApcsDisabled, LARGE_INTEGER, PDEVICE_OBJECT, PEXCEPTION_POINTERS, PFILE_OBJECT, PIO_STATUS_BLOCK, PLARGE_INTEGER, PsGetCurrentThread, IRP, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CD_DATA, CdFsdPostRequest, CdPerformVerify, FCB, IRP_CONTEXT, IRP_CONTEXT_FLAG_DISABLE_POPUPS, IRP_CONTEXT_FLAG_FORCE_POST, IRP_CONTEXT_FLAG_MORE_PROCESSING, IRP_CONTEXT_FLAG_TOP_LEVEL, THREAD_CONTEXT, TYPE_OF_OPEN, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, RAW_DIRENT, SECTOR_SIZE, SYSTEM_USE_XA};
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlFastCheckLockForRead, FsRtlIsNtstatusExpected, FsRtlNormalizeNtstatus, IoCompleteRequest, IoIsErrorUserInduced, ASSERT_IRP, ASSERT_OPTIONAL_IRP_CONTEXT, ClearFlag};
use crate::glue::{CDFS_BUG_CHECK_CDDATA, IRP_MN_COMPLETE, STATUS_CANT_WAIT, STATUS_DISK_CORRUPT_ERROR, STATUS_DRIVER_INTERNAL_ERROR, STATUS_FILE_CORRUPT_ERROR, STATUS_VERIFY_REQUIRED};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
    fn CdFastDecodeFileObject(FileObject: PFILE_OBJECT, Fcb: *mut *mut FCB) -> TYPE_OF_OPEN;
}

extern "C" {
    fn AssertVerifyDevice(IrpContext: *mut IRP_CONTEXT, Status: i32);
}
extern "C" {
    fn NT_ASSERT(cond: i32);
}
extern "C" {
    fn CdBugCheck(param1: usize, param2: usize, param3: usize);
}
extern "C" {
    fn AssertVerifyDeviceIrp(Irp: ntkrnl_sys::ddk::PIRP);
}
extern "C" {
    fn CdCleanupIrpContext(IrpContext: *mut core::ffi::c_void, Flag: i32);
}
extern "C" {
    fn PAGED_CODE();
}
extern "C" {
    fn ASSERT_IRP_CONTEXT(irp: *mut IRP_CONTEXT);
}
extern "C" {
    fn FlagOn(flags: u32, flag: u32) -> i32;
}
extern "C" {
    fn SetFlag(flags: *mut u32, flag: u32);
}
extern "system" {
    fn IoWithinStackLimits(address: usize, size: u32) -> i32;
}
extern "C" {
    fn ASSERT_OPTIONAL_IRP(Irp: ntkrnl_sys::ddk::PIRP);
}
extern "C" {
    fn CanFsdWait(Irp: ntkrnl_sys::ddk::PIRP) -> u8;
}
extern "C" {
    fn CdCommonCleanup(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonClose(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonCreate(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonDevControl(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonDirControl(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonFsControl(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonLockControl(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonPnp(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonQueryInfo(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonQueryVolInfo(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonRead(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonSetInfo(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonShutdown(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCommonWrite(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCompleteMdl(IrpContext: crate::types::PIRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP) -> i32;
}
extern "C" {
    fn CdCreateIrpContext(Irp: ntkrnl_sys::ddk::PIRP, Wait: u8) -> crate::types::PIRP_CONTEXT;
}
extern "C" {
    fn SafeNodeType(Node: *mut IRP_CONTEXT) -> u32;
}

// ---- file-scope preamble (translated) ----
pub static mut CdTestTopLevel: u8 = 1;
pub static mut CdTestRaisedStatus: u8 = 1;
pub static mut CdBreakOnAnyRaise: u8 = 0;
pub static mut CdTraceRaises: u8 = 0;
pub static mut CdInterestingExceptionCodes: [i32; 10] = [
    STATUS_DISK_CORRUPT_ERROR,
    STATUS_FILE_CORRUPT_ERROR,
    0, 0, 0, 0, 0, 0, 0, 0,
];
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_CDDATA;

// #[no_mangle] so the canonical storage is exported as `_CdData` for the modules
// that reach it via `extern { static mut CdData }` (e.g. read.rs) — without this
// they'd bind to a separate, diverging copy.
#[no_mangle]
pub static mut CdData: CD_DATA = unsafe { core::mem::zeroed() };
pub static mut CdFastIoDispatch: FAST_IO_DISPATCH = unsafe { core::mem::zeroed() };

pub static mut CdUnicodeSelfArray: [u16; 1] = [0x002Eu16];
pub static mut CdUnicodeParentArray: [u16; 2] = [0x002Eu16, 0x002Eu16];

pub static mut CdUnicodeDirectoryNames: [_UNICODE_STRING; 2] = [
    _UNICODE_STRING { Length: 2, MaximumLength: 2, Buffer: core::ptr::addr_of_mut!(CdUnicodeSelfArray) as *mut u16 },
    _UNICODE_STRING { Length: 4, MaximumLength: 4, Buffer: core::ptr::addr_of_mut!(CdUnicodeParentArray) as *mut u16 },
];

pub static mut CdHsgId: [i8; 5] = [b'C' as i8, b'D' as i8, b'R' as i8, b'O' as i8, b'M' as i8];
pub static mut CdIsoId: [i8; 5] = [b'C' as i8, b'D' as i8, b'0' as i8, b'0' as i8, b'1' as i8];
pub static mut CdXaId: [i8; 8] = [b'C' as i8, b'D' as i8, b'-' as i8, b'X' as i8, b'A' as i8, b'0' as i8, b'0' as i8, b'1' as i8];
pub static mut CdAudioLabel: [u16; 8] = [
    b'A' as u16, b'u' as u16, b'd' as u16, b'i' as u16, b'o' as u16,
    b' ' as u16, b'C' as u16, b'D' as u16,
];
pub const CdAudioLabelLength: u16 = core::mem::size_of::<[u16; 8]>() as u16;
pub static mut CdAudioFileName: [i8; 11] = [
    b'T' as i8, b'r' as i8, b'a' as i8, b'c' as i8, b'k' as i8,
    b'0' as i8, b'0' as i8, b'.' as i8, b'c' as i8, b'd' as i8, b'a' as i8,
];
pub const CdAudioFileNameLength: u8 = core::mem::size_of::<[i8; 11]>() as u8;
pub const CdAudioDirentSize: u32 = (core::mem::offset_of!(RAW_DIRENT, FileId)
    + core::mem::size_of::<[i8; 11]>()
    + core::mem::size_of::<SYSTEM_USE_XA>()) as u32;
pub const CdAudioDirentsPerSector: u32 = SECTOR_SIZE / CdAudioDirentSize;
pub const CdAudioSystemUseOffset: u32 = (core::mem::offset_of!(RAW_DIRENT, FileId)
    + core::mem::size_of::<[i8; 11]>()) as u32;

pub static mut CdJolietEscape: [*const i8; 3] = [
    b"%/@\0".as_ptr() as *const i8,
    b"%/C\0".as_ptr() as *const i8,
    b"%/E\0".as_ptr() as *const i8,
];

pub static mut CdAudioPlayHeader: [i32; 11] = [
    0x46464952,
    36i32,
    0x41444443,
    0x20746d66,
    24,
    0x00000001,
    0,
    0,
    0,
    0,
    0,
];

pub static mut CdXAAudioPhileHeader: [i32; 11] = [
    0x46464952,
    -8i32,
    0x45564157,
    0x20746d66,
    16,
    0x00020001,
    44100,
    176400,       // 2352 * 75
    0x00100004,
    0x61746164,
    -44i32,
];

pub static mut CdXAFileHeader: [i32; 11] = [
    0x46464952,
    -8i32,
    0x41584443,
    0x20746d66,
    16,
    0,
    0x41580000,
    0,
    0,
    0x61746164,
    -44i32,
];

// C source: `CDFS_TELEMETRY_DATA_CONTEXT CdTelemetryData;` is guarded by
// `#ifdef CDFS_TELEMETRY_DATA`, which is NOT defined in the ReactOS build, so
// the symbol is compiled out. Gated identically here.
#[cfg(feature = "cdfs_telemetry_data")]
pub static mut CdTelemetryData: CDFS_TELEMETRY_DATA_CONTEXT = unsafe { core::mem::zeroed() };

const IRP_INPUT_OPERATION: u32 = 0x40;
const IO_CD_ROM_INCREMENT: i8 = 1;

// IRP_MJ_* major codes not re-exported by ntkrnl_sys::constants on this target.
// Values verified against sdk/include/xdk/iotypes.h. Typed UCHAR to match the
// IRP_CONTEXT.MajorFunction field so the match arms below are refutable (without
// these, an undefined name in a match arm silently becomes a catch-all binding).
const IRP_MJ_QUERY_VOLUME_INFORMATION: u8 = 0x0a;
const IRP_MJ_DIRECTORY_CONTROL: u8 = 0x0c;
const IRP_MJ_DEVICE_CONTROL: u8 = 0x0e;
const IRP_MJ_SHUTDOWN: u8 = 0x10;
const IRP_MJ_PNP: u8 = 0x1b;

#[no_mangle]
pub unsafe extern "system" fn CdFsdDispatch(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
) -> i32 {
    // IRP_MJ_* major codes (wdm.h) — local with verified values so the dispatch
    // match has no value collisions (the grind's mixed const sources produced an
    // unreachable-pattern arm). Verified against sdk/include/xdk/iotypes.h.
    const MJ_CREATE: u8 = 0x00;
    const MJ_CLOSE: u8 = 0x02;
    const MJ_READ: u8 = 0x03;
    const MJ_WRITE: u8 = 0x04;
    const MJ_QUERY_INFORMATION: u8 = 0x05;
    const MJ_SET_INFORMATION: u8 = 0x06;
    const MJ_QUERY_VOLUME_INFORMATION: u8 = 0x0a;
    const MJ_DIRECTORY_CONTROL: u8 = 0x0c;
    const MJ_FILE_SYSTEM_CONTROL: u8 = 0x0d;
    const MJ_DEVICE_CONTROL: u8 = 0x0e;
    const MJ_SHUTDOWN: u8 = 0x10;
    const MJ_LOCK_CONTROL: u8 = 0x11;
    const MJ_CLEANUP: u8 = 0x12;
    const MJ_PNP: u8 = 0x1b;

    // C: THREAD_CONTEXT ThreadContext = {0}; PIRP_CONTEXT IrpContext = NULL;
    let mut thread_context: THREAD_CONTEXT = core::mem::zeroed();
    let mut irp_context: crate::types::PIRP_CONTEXT = core::ptr::null_mut();
    let mut wait: u8 = 0;

    ASSERT_OPTIONAL_IRP(Irp);
    let _ = DeviceObject;

    FsRtlEnterFileSystem();

    // Loop until the request is completed or posted. Each pass runs under the C's
    // try/except (ntkrnl_sys::seh): a raise anywhere below — CdRaiseStatus,
    // STATUS_CANT_WAIT, STATUS_VERIFY_REQUIRED, a Cc in-page error — reaches
    // CdExceptionFilter and CdProcessException, which post, retry or complete the
    // IRP. Under DUMMY_PSEH that arm was dead and every raise bugchecked 0x1E.
    let mut status: i32;
    loop {
        let mut st = (irp_context, STATUS_SUCCESS);
        let caught = ntkrnl_sys::seh::try_except(
            &mut st,
            |st| {
                if st.0.is_null() {
                    // FileObject == NULL means a mount, which is always waitable.
                    if (*IoGetCurrentIrpStackLocation(Irp)).FileObject.is_null() {
                        wait = 1;
                    } else {
                        wait = CanFsdWait(Irp);
                    }
                    st.0 = CdCreateIrpContext(Irp, wait);
                    CdSetThreadContext(st.0, &mut thread_context as *mut THREAD_CONTEXT);
                } else {
                    SetFlag(core::ptr::addr_of_mut!((*st.0).Flags), IRP_CONTEXT_FLAG_MORE_PROCESSING);
                    CdCleanupIrpContext(st.0 as *mut core::ffi::c_void, 0);
                }
                let irp_context = st.0;

                st.1 = match (*irp_context).MajorFunction {
                    MJ_CREATE => CdCommonCreate(irp_context, Irp),
                    MJ_CLOSE => CdCommonClose(irp_context, Irp),
                    MJ_READ => {
                        if FlagOn((*irp_context).MinorFunction as u32, IRP_MN_COMPLETE) != 0 {
                            CdCompleteMdl(irp_context, Irp)
                        } else {
                            CdCommonRead(irp_context, Irp)
                        }
                    }
                    MJ_WRITE => CdCommonWrite(irp_context, Irp),
                    MJ_QUERY_INFORMATION => CdCommonQueryInfo(irp_context, Irp),
                    MJ_SET_INFORMATION => CdCommonSetInfo(irp_context, Irp),
                    MJ_QUERY_VOLUME_INFORMATION => CdCommonQueryVolInfo(irp_context, Irp),
                    MJ_DIRECTORY_CONTROL => CdCommonDirControl(irp_context, Irp),
                    MJ_FILE_SYSTEM_CONTROL => CdCommonFsControl(irp_context, Irp),
                    MJ_DEVICE_CONTROL => CdCommonDevControl(irp_context, Irp),
                    MJ_LOCK_CONTROL => CdCommonLockControl(irp_context, Irp),
                    MJ_CLEANUP => CdCommonCleanup(irp_context, Irp),
                    MJ_PNP => CdCommonPnp(irp_context, Irp),
                    MJ_SHUTDOWN => CdCommonShutdown(irp_context, Irp),
                    _ => {
                        CdCompleteRequest(irp_context as *mut core::ffi::c_void, Irp, STATUS_INVALID_DEVICE_REQUEST);
                        STATUS_INVALID_DEVICE_REQUEST
                    }
                };
            },
            |st, pointers| CdExceptionFilter(st.0, pointers as PEXCEPTION_POINTERS),
        );
        irp_context = st.0;
        status = match caught {
            Ok(()) => st.1,
            Err(code) => CdProcessException(irp_context, Irp, code),
        };

        if status != STATUS_CANT_WAIT {
            break;
        }
    }

    FsRtlExitFileSystem();
    status
}

// ntstatus.h: STATUS_UNEXPECTED_IO_ERROR = 0xC00000E9 (grind fabricated 0xC00000FF).
const STATUS_UNEXPECTED_IO_ERROR: i32 = 0xC00000E9u32 as i32;
#[no_mangle]
pub unsafe extern "C" fn CdRaiseStatusEx(
    IrpContext: *mut IRP_CONTEXT,
    Status: i32,
    NormalizeStatus: u8,
    FileId: u32,
    Line: u32,
) {
    let mut BreakIn: u8 = 0;
    AssertVerifyDevice(IrpContext, Status);
    if CdTraceRaises != 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"%p CdRaiseStatusEx 0x%x @ fid %d, line %d\n\0".as_ptr() as *const i8,
            ntkrnl_sys::ddk::PsGetCurrentThread() as *mut core::ffi::c_void,
            Status,
            FileId,
            Line,
        );
    }
    if CdTestRaisedStatus != 0 && CdBreakOnAnyRaise == 0 {
        for Index in 0..(core::mem::size_of::<[i32; 10]>() / core::mem::size_of::<i32>()) {
            if ntkrnl_sys::base::STATUS_SUCCESS != CdInterestingExceptionCodes[Index]
                && CdInterestingExceptionCodes[Index] == Status
            {
                BreakIn = 1;
                break;
            }
        }
    }
    if BreakIn != 0 || CdBreakOnAnyRaise != 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"CDFS: Breaking on raised status %08x  (BI=%d,BA=%d)\n\0".as_ptr() as *const i8,
            Status,
            BreakIn as i32,
            CdBreakOnAnyRaise as i32,
        );
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"CDFS: (FILEID %d LINE %d)\n\0".as_ptr() as *const i8,
            FileId,
            Line,
        );
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"CDFS: Contact CDFS.SYS component owner for triage.\n\0".as_ptr() as *const i8,
        );
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"CDFS: 'eb %p 0;eb %p 0' to disable this alert.\n\0".as_ptr() as *const i8,
            core::ptr::addr_of!(CdTestRaisedStatus) as *const core::ffi::c_void,
            core::ptr::addr_of!(CdBreakOnAnyRaise) as *const core::ffi::c_void,
        );
        NT_ASSERT(0);
    }
    if NormalizeStatus != 0 {
        (*IrpContext).ExceptionStatus =
            FsRtlNormalizeNtstatus(Status, STATUS_UNEXPECTED_IO_ERROR);
    } else {
        (*IrpContext).ExceptionStatus = Status;
    }
    (*IrpContext).RaisedAtLineFile = (FileId << 16) | Line;
    ntkrnl_sys::ddk::ExRaiseStatus((*IrpContext).ExceptionStatus);
}
#[no_mangle]
pub unsafe extern "C" fn CdExceptionFilter(
    IrpContext: *mut IRP_CONTEXT,
    ExceptionPointer: PEXCEPTION_POINTERS,
) -> i32 {
    const EXCEPTION_EXECUTE_HANDLER: i32 = 1;
    const STATUS_IN_PAGE_ERROR: i32 = 0xC0000006u32 as i32;
    let mut ExceptionCode: i32;
    let mut TestStatus: i32 = 1; // TRUE

    // ASSERT_OPTIONAL_IRP_CONTEXT(IrpContext): NULL is legal. An exception raised
    // inside CdCreateIrpContext reaches this filter before there is a context.

    let exception_record = (*ExceptionPointer).ExceptionRecord;
    ExceptionCode = (*exception_record).ExceptionCode;

    if (ExceptionCode == STATUS_IN_PAGE_ERROR) && ((*exception_record).NumberParameters >= 3) {
        ExceptionCode = (*exception_record).ExceptionInformation[2] as i32;
    }

    if !IrpContext.is_null() {
        if (*IrpContext).ExceptionStatus == ntkrnl_sys::base::STATUS_SUCCESS {
            (*IrpContext).ExceptionStatus = ExceptionCode;
        } else {
            TestStatus = 0; // FALSE
        }
    }

    if !IrpContext.is_null() {
        AssertVerifyDevice(IrpContext, (*IrpContext).ExceptionStatus);
    }

    if TestStatus != 0 && FsRtlIsNtstatusExpected(ExceptionCode) == 0 {
        CdBugCheck(
            exception_record as usize,
            (*ExceptionPointer).ContextRecord as usize,
            (*exception_record).ExceptionAddress as usize,
        );
    }

    EXCEPTION_EXECUTE_HANDLER
}
#[no_mangle]
pub unsafe extern "C" fn CdProcessException(
    IrpContext: *mut IRP_CONTEXT,
    Irp: ntkrnl_sys::ddk::PIRP,
    mut ExceptionCode: ntkrnl_sys::base::NTSTATUS,
) -> ntkrnl_sys::base::NTSTATUS {
    let mut Device: ntkrnl_sys::ddk::PDEVICE_OBJECT = core::ptr::null_mut();
    let Vpb: ntkrnl_sys::ddk::PVPB;
    let mut Thread: ntkrnl_sys::ddk::PETHREAD;

    ASSERT_OPTIONAL_IRP_CONTEXT(IrpContext);
    ASSERT_IRP(Irp);

    if IrpContext.is_null() {
        CdCompleteRequest(core::ptr::null_mut(), Irp, ExceptionCode);
        return ExceptionCode;
    }

    ExceptionCode = (*IrpContext).ExceptionStatus;

    // _SEH2_TRY / _SEH2_EXCEPT( CdExceptionFilter(...) ): CdFsdPostRequest's
    // children can raise (Mdl allocation).
    if let Err(code) = ntkrnl_sys::seh::try_except(
        &mut ExceptionCode,
        |ExceptionCode| {
            if *ExceptionCode == STATUS_CANT_WAIT {
                if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_FORCE_POST) != 0 {
                    *ExceptionCode = CdFsdPostRequest(IrpContext, Irp);
                }
            } else if *ExceptionCode == STATUS_VERIFY_REQUIRED
                && FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_TOP_LEVEL) != 0
                && KeAreAllApcsDisabled() != 0
            {
                *ExceptionCode = CdFsdPostRequest(IrpContext, Irp);
            }
        },
        |_, pointers| CdExceptionFilter(IrpContext, pointers as PEXCEPTION_POINTERS),
    ) {
        ExceptionCode = code;
    }

    if ExceptionCode == STATUS_PENDING
        || ExceptionCode == STATUS_CANT_WAIT {
        return ExceptionCode;
    }

    ClearFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_MORE_PROCESSING);

    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_TOP_LEVEL) == 0 {
        CdCompleteRequest(IrpContext as *mut core::ffi::c_void, Irp, ExceptionCode);
        return ExceptionCode;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = ExceptionCode;

    if IoIsErrorUserInduced(ExceptionCode) != 0 {
        if ExceptionCode == STATUS_VERIFY_REQUIRED {
            Device = IoGetDeviceToVerify((*Irp).Tail.Overlay.Thread);
            IoSetDeviceToVerify((*Irp).Tail.Overlay.Thread, core::ptr::null_mut());

            if Device.is_null() {
                Device = IoGetDeviceToVerify(PsGetCurrentThread());
                IoSetDeviceToVerify(PsGetCurrentThread(), core::ptr::null_mut());
                NT_ASSERT((!Device.is_null()) as i32);
            }

            if !(*IrpContext).Vcb.is_null() {
                Device = (*(*(*IrpContext).Vcb).Vpb).RealDevice;
            }

            if Device.is_null() {
                ExceptionCode = STATUS_DRIVER_INTERNAL_ERROR;
                CdCompleteRequest(IrpContext as *mut core::ffi::c_void, Irp, ExceptionCode);
                return ExceptionCode;
            }

            return CdPerformVerify(IrpContext, Irp, Device);
        }

        if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS) != 0 {
            CdCompleteRequest(IrpContext as *mut core::ffi::c_void, Irp, ExceptionCode);
            return ExceptionCode;
        } else {
            let file_object = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;
            if !file_object.is_null() {
                Vpb = (*file_object).Vpb;
            } else {
                Vpb = core::ptr::null_mut();
            }

            Thread = (*Irp).Tail.Overlay.Thread;
            Device = IoGetDeviceToVerify(Thread);

            if Device.is_null() {
                Thread = PsGetCurrentThread();
                Device = IoGetDeviceToVerify(Thread);
                NT_ASSERT((!Device.is_null()) as i32);
            }

            if !(*IrpContext).Vcb.is_null() {
                Device = (*(*(*IrpContext).Vcb).Vpb).RealDevice;
            }

            if Device.is_null() {
                CdCompleteRequest(IrpContext as *mut core::ffi::c_void, Irp, ExceptionCode);
                return ExceptionCode;
            }

            IoMarkIrpPending(Irp);
            IoRaiseHardError(Irp, Vpb, Device);

            IoSetDeviceToVerify(Thread, core::ptr::null_mut());

            CdCompleteRequest(IrpContext as *mut core::ffi::c_void, core::ptr::null_mut(), ntkrnl_sys::base::STATUS_SUCCESS);
            return ntkrnl_sys::constants::STATUS_PENDING;
        }
    }

    CdCompleteRequest(IrpContext as *mut core::ffi::c_void, Irp, ExceptionCode);
    return ExceptionCode;
}

#[no_mangle]
pub unsafe extern "C" fn CdCompleteRequest(
    IrpContext: *mut core::ffi::c_void,
    Irp: ntkrnl_sys::ddk::PIRP,
    Status: i32,
) {
    // ASSERT_OPTIONAL_IRP_CONTEXT(IrpContext);
    // ASSERT_OPTIONAL_IRP(Irp);

    if !IrpContext.is_null() {
        CdCleanupIrpContext(IrpContext, 0);
    }

    if !Irp.is_null() {
        if (Status < 0) && ((*Irp).Flags & IRP_INPUT_OPERATION) != 0 {
            (*Irp).IoStatus.Information = 0;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

        AssertVerifyDeviceIrp(Irp);

        IoCompleteRequest(Irp, IO_CD_ROM_INCREMENT);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdSetThreadContext(
    IrpContext: *mut IRP_CONTEXT,
    ThreadContext: *mut THREAD_CONTEXT,
) {
    PAGED_CODE();
    ASSERT_IRP_CONTEXT(IrpContext);

    let CurrentThreadContext = IoGetTopLevelIrp() as *mut THREAD_CONTEXT;

    if CurrentThreadContext.is_null() {
        SetFlag(
            core::ptr::addr_of_mut!((*IrpContext).Flags),
            0x10, // IRP_CONTEXT_FLAG_TOP_LEVEL (grind had 0x2 = MORE_PROCESSING)
        );
    }

    let mut condition = false;

    // Evaluate the big OR condition with exact C short-circuit behaviour
    if !condition {
        condition = FlagOn((*IrpContext).Flags, 0x10 /* IRP_CONTEXT_FLAG_TOP_LEVEL (grind had 0x2) */) != 0;
    }

    if !condition {
        #[cfg(not(feature = "reactos"))]
        {
            condition = !IoWithinStackLimits(
                CurrentThreadContext as usize,
                core::mem::size_of::<THREAD_CONTEXT>() as u32,
            ) != 0;
        }
        #[cfg(feature = "reactos")]
        {
            let mut stack_top: usize = 0;
            let mut stack_bottom: usize = 0;
            // IoGetStackLimits takes PULONG_PTR (per-arch u32/u64); usize is the
            // same width on every target, so cast the pointers to the expected type.
            IoGetStackLimits(
                core::ptr::addr_of_mut!(stack_top) as *mut _,
                core::ptr::addr_of_mut!(stack_bottom) as *mut _,
            );
            let ctx_ptr = CurrentThreadContext as usize;
            condition = (ctx_ptr > stack_bottom.wrapping_sub(
                core::mem::size_of::<THREAD_CONTEXT>(),
            )) || (ctx_ptr <= stack_top);
        }
    }

    if !condition {
        condition = FlagOn(
            CurrentThreadContext as usize as u32,
            0x3,
        ) != 0;
    }

    if !condition {
        condition = (*CurrentThreadContext).Cdfs != 0x53464443;
    }

    if condition {
        (*ThreadContext).Cdfs = 0x53464443;
        (*ThreadContext).SavedTopLevelIrp = CurrentThreadContext as *mut IRP;
        (*ThreadContext).TopLevelIrpContext = IrpContext;
        IoSetTopLevelIrp(ThreadContext as *mut IRP);
        (*IrpContext).TopLevel = IrpContext;
        (*IrpContext).ThreadContext = ThreadContext;
        SetFlag(
            core::ptr::addr_of_mut!((*IrpContext).Flags),
            0x20, // IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS (grind had 0x4 = WAIT)
        );
    } else {
        (*IrpContext).TopLevel = (*CurrentThreadContext).TopLevelIrpContext;
    }
}
#[no_mangle]
pub unsafe extern "system" fn CdFastIoCheckIfPossible(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    CheckForReadOperation: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> i32 {
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let TypeOfOpen: TYPE_OF_OPEN;
    let mut LargeLength: LARGE_INTEGER = core::mem::zeroed();

    // PAGED_CODE()
    if unsafe { KeGetCurrentIrql() } > APC_LEVEL {
    }

    let _ = (Wait, DeviceObject); // UNREFERENCED_PARAMETER

    TypeOfOpen = CdFastDecodeFileObject(FileObject, &mut Fcb);

    if (TypeOfOpen != UserFileOpen) || (CheckForReadOperation == 0) {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1;
    }

    LargeLength.QuadPart = Length as i64;

    // C: Fcb->FileLock — lives in the FCB_DATA arm of the FCB type union
    // (bindgen collapsed __bindgen_anon_2 to an opaque blob; reinterpret it).
    let FcbData = core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut crate::types::FCB_DATA;
    if (*FcbData).FileLock.is_null()
        || FsRtlFastCheckLockForRead(
            (*FcbData).FileLock,
            FileOffset,
            &mut LargeLength,
            LockKey,
            FileObject,
            PsGetCurrentProcess() as *mut core::ffi::c_void,
        ) != 0
    {
        return 1;
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CdSerial32(
    Buffer: *const u8,
    ByteCount: u32,
) -> u32 {
    // PAGED_CODE();
    let mut buffer = Buffer;
    let mut checksum_bytes: [u8; 4] = [0; 4]; // replaces union Checksum
    // Checksum.SerialId = 0; achieved by zero-initialization above
    let mut byte_count = ByteCount;
    while byte_count != 0 {
        byte_count -= 1;
        let idx = (byte_count & 3) as usize;
        checksum_bytes[idx] = checksum_bytes[idx].wrapping_add(*buffer);
        buffer = buffer.add(1);
    }
    u32::from_ne_bytes(checksum_bytes)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     CdData.c
//
// Abstract:
//
//     This module declares the global data used by the Cdfs file system.
//
//     This module also handles the dispath routines in the Fsd threads as well as
//     handling the IrpContext and Irp through the exception path.
//
//
// --*/
//
// #include "cdprocs.h"
//
// #ifdef CD_SANITY
// BOOLEAN CdTestTopLevel = TRUE;
// BOOLEAN CdTestRaisedStatus = TRUE;
// BOOLEAN CdBreakOnAnyRaise = FALSE;
// BOOLEAN CdTraceRaises = FALSE;
// NTSTATUS CdInterestingExceptionCodes[] = { STATUS_DISK_CORRUPT_ERROR,
//                                            STATUS_FILE_CORRUPT_ERROR,
//                                            0, 0, 0, 0, 0, 0, 0, 0 };
// #endif
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_CDDATA)
//
// //
// //  Global data structures
// //
//
// CD_DATA CdData;
// FAST_IO_DISPATCH CdFastIoDispatch;
//
// //
// //  Reserved directory strings.
// //
//
// WCHAR CdUnicodeSelfArray[] = { L'.' };
// WCHAR CdUnicodeParentArray[] = { L'.', L'.' };
//
// UNICODE_STRING CdUnicodeDirectoryNames[] = {
//     { 2, 2, CdUnicodeSelfArray},
//     { 4, 4, CdUnicodeParentArray}
// };
//
// //
// //  Volume descriptor identifier strings.
// //
//
// CHAR CdHsgId[] = { 'C', 'D', 'R', 'O', 'M' };
// CHAR CdIsoId[] = { 'C', 'D', '0', '0', '1' };
// CHAR CdXaId[] = { 'C', 'D', '-', 'X', 'A', '0', '0', '1' };
//
// //
// //  Volume label for audio disks.
// //
//
// WCHAR CdAudioLabel[] = { L'A', L'u', L'd', L'i', L'o', L' ', L'C', L'D' };
// USHORT CdAudioLabelLength = sizeof( CdAudioLabel );
//
// //
// //  Pseudo file names for audio disks.
// //
//
// CHAR CdAudioFileName[] = { 'T', 'r', 'a', 'c', 'k', '0', '0', '.', 'c', 'd', 'a' };
// UCHAR CdAudioFileNameLength = sizeof( CdAudioFileName );
// ULONG CdAudioDirentSize = FIELD_OFFSET( RAW_DIRENT, FileId ) + sizeof( CdAudioFileName ) + sizeof( SYSTEM_USE_XA );
// ULONG CdAudioDirentsPerSector = SECTOR_SIZE / (FIELD_OFFSET( RAW_DIRENT, FileId ) + sizeof( CdAudioFileName ) + sizeof( SYSTEM_USE_XA ));
// ULONG CdAudioSystemUseOffset = FIELD_OFFSET( RAW_DIRENT, FileId ) + sizeof( CdAudioFileName );
//
// //
// //  Escape sequences for mounting Unicode volumes.
// //
//
// PCHAR CdJolietEscape[] = { "%/@", "%/C", "%/E" };
//
// //
// //  Audio Play Files consist completely of this header block.  These
// //  files are readable in the root of any audio disc regardless of
// //  the capabilities of the drive.
// //
// //  The "Unique Disk ID Number" is a calculated value consisting of
// //  a combination of parameters, including the number of tracks and
// //  the starting locations of those tracks.
// //
// //  Applications interpreting CDDA RIFF files should be advised that
// //  additional RIFF file chunks may be added to this header in the
// //  future in order to add information, such as the disk and song title.
// //
//
// LONG CdAudioPlayHeader[] = {
//     0x46464952,                         // Chunk ID = 'RIFF'
//     4 * 11 - 8,                         // Chunk Size = (file size - 8)
//     0x41444443,                         // 'CDDA'
//     0x20746d66,                         // 'fmt '
//     24,                                 // Chunk Size (of 'fmt ' subchunk) = 24
//     0x00000001,                         // WORD Format Tag, WORD Track Number
//     0x00000000,                         // DWORD Unique Disk ID Number
//     0x00000000,                         // DWORD Track Starting Sector (LBN)
//     0x00000000,                         // DWORD Track Length (LBN count)
//     0x00000000,                         // DWORD Track Starting Sector (MSF)
//     0x00000000                          // DWORD Track Length (MSF)
// };
//
// //  Audio Philes begin with this header block to identify the data as a
// //  PCM waveform.  AudioPhileHeader is coded as if it has no data included
// //  in the waveform.  Data must be added in 2352-byte multiples.
// //
// //  Fields marked 'ADJUST' need to be adjusted based on the size of the
// //  data: Add (nSectors*2352) to the DWORDs at offsets 1*4 and 10*4.
// //
// //  File Size of TRACK??.WAV = nSectors*2352 + sizeof(AudioPhileHeader)
// //  RIFF('WAVE' fmt(1, 2, 44100, 176400, 16, 4) data( <CD Audio Raw Data> )
// //
// //  The number of sectors in a CD-XA CD-DA file is (DataLen/2048).
// //  CDFS will expose these files to applications as if they were just
// //  'WAVE' files, adjusting the file size so that the RIFF file is valid.
// //
// //  NT NOTE: We do not do any fidelity adjustment. These are presented as raw
// //  2352 byte sectors - 95 has the glimmer of an idea to allow CDFS to expose
// //  the CDXA CDDA data at different sampling rates in a virtual directory
// //  structure, but we will never do that.
// //
//
// LONG CdXAAudioPhileHeader[] = {
//     0x46464952,                         // Chunk ID = 'RIFF'
//     -8,                                 // Chunk Size = (file size - 8) ADJUST1
//     0x45564157,                         // 'WAVE'
//     0x20746d66,                         // 'fmt '
//     16,                                 // Chunk Size (of 'fmt ' subchunk) = 16
//     0x00020001,                         // WORD Format Tag WORD nChannels
//     44100,                              // DWORD nSamplesPerSecond
//     2352 * 75,                          // DWORD nAvgBytesPerSec
//     0x00100004,                         // WORD nBlockAlign WORD nBitsPerSample
//     0x61746164,                         // 'data'
//     -44                                 // <CD Audio Raw Data>          ADJUST2
// };
//
// //
// //  XA Files begin with this RIFF header block to identify the data as
// //  raw CD-XA sectors.  Data must be added in 2352-byte multiples.
// //
// //  This header is added to all CD-XA files which are marked as having
// //  mode2form2 sectors.
// //
// //  Fields marked 'ADJUST' need to be adjusted based on the size of the
// //  data: Add file size to the marked DWORDS.
// //
// //  File Size of TRACK??.WAV = nSectors*2352 + sizeof(XAFileHeader)
// //
// //  RIFF('CDXA' FMT(Owner, Attr, 'X', 'A', FileNum, 0) data ( <CDXA Raw Data> )
// //
//
// LONG CdXAFileHeader[] = {
//     0x46464952,                         // Chunk ID = 'RIFF'
//     -8,                                 // Chunk Size = (file size - 8) ADJUST
//     0x41584443,                         // 'CDXA'
//     0x20746d66,                         // 'fmt '
//     16,                                 // Chunk Size (of CDXA chunk) = 16
//     0,                                  // DWORD Owner ID
//     0x41580000,                         // WORD Attributes
//                                         // BYTE Signature byte 1 'X'
//                                         // BYTE Signature byte 2 'A'
//     0,                                  // BYTE File Number
//     0,                                  // BYTE Reserved[7]
//     0x61746164,                         // 'data'
//     -44                                 // <CD-XA Raw Sectors>          ADJUST
// };
//
// #ifdef CDFS_TELEMETRY_DATA
//
// //
// // Telemetry Data for reporting
// //
//
// CDFS_TELEMETRY_DATA_CONTEXT CdTelemetryData;
//
// #endif // CDFS_TELEMETRY_DATA
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdFastIoCheckIfPossible)
// #pragma alloc_text(PAGE, CdSerial32)
// #pragma alloc_text(PAGE, CdSetThreadContext)
// #endif
//
// _IRQL_requires_max_(APC_LEVEL)
// __drv_dispatchType(DRIVER_DISPATCH)
// __drv_dispatchType(IRP_MJ_CREATE)
// __drv_dispatchType(IRP_MJ_CLOSE)
// __drv_dispatchType(IRP_MJ_READ)
// __drv_dispatchType(IRP_MJ_WRITE)
// __drv_dispatchType(IRP_MJ_QUERY_INFORMATION)
// __drv_dispatchType(IRP_MJ_SET_INFORMATION)
// __drv_dispatchType(IRP_MJ_QUERY_VOLUME_INFORMATION)
// __drv_dispatchType(IRP_MJ_DIRECTORY_CONTROL)
// __drv_dispatchType(IRP_MJ_FILE_SYSTEM_CONTROL)
// __drv_dispatchType(IRP_MJ_DEVICE_CONTROL)
// __drv_dispatchType(IRP_MJ_LOCK_CONTROL)
// __drv_dispatchType(IRP_MJ_CLEANUP)
// __drv_dispatchType(IRP_MJ_PNP)
// __drv_dispatchType(IRP_MJ_SHUTDOWN)
// NTSTATUS
// NTAPI
// CdFsdDispatch (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the driver entry to all of the Fsd dispatch points.
//
//     Conceptually the Io routine will call this routine on all requests
//     to the file system.  We case on the type of request and invoke the
//     correct handler for this type of request.  There is an exception filter
//     to catch any exceptions in the CDFS code as well as the CDFS process
//     exception routine.
//
//     This routine allocates and initializes the IrpContext for this request as
//     well as updating the top-level thread context as necessary.  We may loop
//     in this routine if we need to retry the request for any reason.  The
//     status code STATUS_CANT_WAIT is used to indicate this.  Suppose the disk
//     in the drive has changed.  An Fsd request will proceed normally until it
//     recognizes this condition.  STATUS_VERIFY_REQUIRED is raised at that point
//     and the exception code will handle the verify and either return
//     STATUS_CANT_WAIT or STATUS_PENDING depending on whether the request was
//     posted.
//
// Arguments:
//
//     DeviceObject - Supplies the volume device object for this request
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
//     THREAD_CONTEXT ThreadContext = {0};
//     PIRP_CONTEXT IrpContext = NULL;
//     BOOLEAN Wait;
//
// #ifdef CD_SANITY
//     PVOID PreviousTopLevel;
// #endif
//
//     NTSTATUS Status;
//
// #if DBG
//
//     KIRQL SaveIrql = KeGetCurrentIrql();
//
// #endif
//
//     ASSERT_OPTIONAL_IRP( Irp );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     FsRtlEnterFileSystem();
//
// #ifdef CD_SANITY
//     PreviousTopLevel = IoGetTopLevelIrp();
// #endif
//
//     //
//     //  Loop until this request has been completed or posted.
//     //
//
//     do {
//
//         //
//         //  Use a try-except to handle the exception cases.
//         //
//
//         _SEH2_TRY {
//
//             //
//             //  If the IrpContext is NULL then this is the first pass through
//             //  this loop.
//             //
//
//             if (IrpContext == NULL) {
//
//                 //
//                 //  Decide if this request is waitable an allocate the IrpContext.
//                 //  If the file object in the stack location is NULL then this
//                 //  is a mount which is always waitable.  Otherwise we look at
//                 //  the file object flags.
//                 //
//
//                 if (IoGetCurrentIrpStackLocation( Irp )->FileObject == NULL) {
//
//                     Wait = TRUE;
//
//                 } else {
//
//                     Wait = CanFsdWait( Irp );
//                 }
//
//                 IrpContext = CdCreateIrpContext( Irp, Wait );
//
//                 //
//                 //  Update the thread context information.
//                 //
//
//                 CdSetThreadContext( IrpContext, &ThreadContext );
//
// #ifdef CD_SANITY
//                 NT_ASSERT( !CdTestTopLevel ||
//                         SafeNodeType( IrpContext->TopLevel ) == CDFS_NTC_IRP_CONTEXT );
// #endif
//
//             //
//             //  Otherwise cleanup the IrpContext for the retry.
//             //
//
//             } else {
//
//                 //
//                 //  Set the MORE_PROCESSING flag to make sure the IrpContext
//                 //  isn't inadvertently deleted here.  Then cleanup the
//                 //  IrpContext to perform the retry.
//                 //
//
//                 SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING );
//                 CdCleanupIrpContext( IrpContext, FALSE );
//             }
//
//             //
//             //  Case on the major irp code.
//             //
//
//             switch (IrpContext->MajorFunction) {
//
//             case IRP_MJ_CREATE :
//
//                 Status = CdCommonCreate( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_CLOSE :
//
//                 Status = CdCommonClose( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_READ :
//
//                 //
//                 //  If this is an Mdl complete request, don't go through
//                 //  common read.
//                 //
//
//                 if (FlagOn( IrpContext->MinorFunction, IRP_MN_COMPLETE )) {
//
//                     Status = CdCompleteMdl( IrpContext, Irp );
//
//                 } else {
//
//                     Status = CdCommonRead( IrpContext, Irp );
//                 }
//
//                 break;
//
//             case IRP_MJ_WRITE :
//
//                 Status = CdCommonWrite( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_QUERY_INFORMATION :
//
//                 Status = CdCommonQueryInfo( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_SET_INFORMATION :
//
//                 Status = CdCommonSetInfo( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_QUERY_VOLUME_INFORMATION :
//
//                 Status = CdCommonQueryVolInfo( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_DIRECTORY_CONTROL :
//
//                 Status = CdCommonDirControl( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_FILE_SYSTEM_CONTROL :
//
//                 Status = CdCommonFsControl( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_DEVICE_CONTROL :
//
//                 Status = CdCommonDevControl( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_LOCK_CONTROL :
//
//                 Status = CdCommonLockControl( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_CLEANUP :
//
//                 Status = CdCommonCleanup( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_PNP :
//
//                 Status = CdCommonPnp( IrpContext, Irp );
//                 break;
//
//             case IRP_MJ_SHUTDOWN :
//
//                 Status = CdCommonShutdown( IrpContext, Irp );
//                 break;
//
//             default :
//
//                 Status = STATUS_INVALID_DEVICE_REQUEST;
//                 CdCompleteRequest( IrpContext, Irp, Status );
//             }
//
//         } _SEH2_EXCEPT( CdExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//             Status = CdProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//         } _SEH2_END;
//
//     } while (Status == STATUS_CANT_WAIT);
//
// #ifdef CD_SANITY
//     NT_ASSERT( !CdTestTopLevel ||
//             (PreviousTopLevel == IoGetTopLevelIrp()) );
// #endif
//
//     FsRtlExitFileSystem();
//
//     NT_ASSERT( SaveIrql == KeGetCurrentIrql( ));
//
//     return Status;
// }
//
//
// #ifdef CD_SANITY
//
// VOID
// CdRaiseStatusEx (
//      PIRP_CONTEXT IrpContext,
//      NTSTATUS Status,
//      BOOLEAN NormalizeStatus,
//      ULONG FileId,
//      ULONG Line
//     )
// {
//     BOOLEAN BreakIn = FALSE;
//
//     AssertVerifyDevice( IrpContext, Status);
//
//     if (CdTraceRaises)  {
//
//         DbgPrint( "%p CdRaiseStatusEx 0x%x @ fid %d, line %d\n", PsGetCurrentThread(), Status, FileId, Line);
//     }
//
//     if (CdTestRaisedStatus && !CdBreakOnAnyRaise)  {
//
//         ULONG Index;
//
//         for (Index = 0;
//              Index < (sizeof( CdInterestingExceptionCodes) / sizeof( CdInterestingExceptionCodes[0]));
//              Index++)  {
//
//             if ((STATUS_SUCCESS != CdInterestingExceptionCodes[Index]) &&
//                 (CdInterestingExceptionCodes[Index] == Status))  {
//
//                 BreakIn = TRUE;
//                 break;
//             }
//         }
//     }
//
//     if (BreakIn || CdBreakOnAnyRaise)  {
//
//         DbgPrint( "CDFS: Breaking on raised status %08x  (BI=%d,BA=%d)\n", Status, BreakIn, CdBreakOnAnyRaise);
//         DbgPrint( "CDFS: (FILEID %d LINE %d)\n", FileId, Line);
//         DbgPrint( "CDFS: Contact CDFS.SYS component owner for triage.\n");
//         DbgPrint( "CDFS: 'eb %p 0;eb %p 0' to disable this alert.\n", &CdTestRaisedStatus, &CdBreakOnAnyRaise);
//
//         NT_ASSERT(FALSE);
//     }
//
//     if (NormalizeStatus)  {
//
//         IrpContext->ExceptionStatus = FsRtlNormalizeNtstatus( Status, STATUS_UNEXPECTED_IO_ERROR);
//     }
//     else {
//
//         IrpContext->ExceptionStatus = Status;
//     }
//
//     IrpContext->RaisedAtLineFile = (FileId << 16) | Line;
//
//     ExRaiseStatus( IrpContext->ExceptionStatus);
// }
//
// #endif
//
//
// LONG
// CdExceptionFilter (
//      PIRP_CONTEXT IrpContext,
//      PEXCEPTION_POINTERS ExceptionPointer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to decide whether we will handle a raised exception
//     status.  If CDFS explicitly raised an error then this status is already
//     in the IrpContext.  We choose which is the correct status code and
//     either indicate that we will handle the exception or bug-check the system.
//
// Arguments:
//
//     ExceptionCode - Supplies the exception code to being checked.
//
// Return Value:
//
//     ULONG - returns EXCEPTION_EXECUTE_HANDLER or bugchecks
//
// --*/
//
// {
//     NTSTATUS ExceptionCode;
//     BOOLEAN TestStatus = TRUE;
//
//     ASSERT_OPTIONAL_IRP_CONTEXT( IrpContext );
//
//     ExceptionCode = ExceptionPointer->ExceptionRecord->ExceptionCode;
//
//     //
//     // If the exception is STATUS_IN_PAGE_ERROR, get the I/O error code
//     // from the exception record.
//     //
//
//     if ((ExceptionCode == STATUS_IN_PAGE_ERROR) &&
//         (ExceptionPointer->ExceptionRecord->NumberParameters >= 3)) {
//
//         ExceptionCode =
//             (NTSTATUS)ExceptionPointer->ExceptionRecord->ExceptionInformation[2];
//     }
//
//     //
//     //  If there is an Irp context then check which status code to use.
//     //
//
//     if (ARGUMENT_PRESENT( IrpContext )) {
//
//         if (IrpContext->ExceptionStatus == STATUS_SUCCESS) {
//
//             //
//             //  Store the real status into the IrpContext.
//             //
//
//             IrpContext->ExceptionStatus = ExceptionCode;
//
//         } else {
//
//             //
//             //  No need to test the status code if we raised it ourselves.
//             //
//
//             TestStatus = FALSE;
//         }
//     }
//
//     AssertVerifyDevice( IrpContext, IrpContext->ExceptionStatus );
//
//     //
//     //  Bug check if this status is not supported.
//     //
//
//     if (TestStatus && !FsRtlIsNtstatusExpected( ExceptionCode )) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: __WARNING_USE_OTHER_FUNCTION, "We're corrupted." )
// #endif
//         CdBugCheck( (ULONG_PTR) ExceptionPointer->ExceptionRecord,
//                     (ULONG_PTR) ExceptionPointer->ContextRecord,
//                     (ULONG_PTR) ExceptionPointer->ExceptionRecord->ExceptionAddress );
//
//     }
//
//     return EXCEPTION_EXECUTE_HANDLER;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdProcessException (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      NTSTATUS ExceptionCode
//     )
//
// /*++
//
// Routine Description:
//
//     This routine processes an exception.  It either completes the request
//     with the exception status in the IrpContext, sends this off to the Fsp
//     workque or causes it to be retried in the current thread if a verification
//     is needed.
//
//     If the volume needs to be verified (STATUS_VERIFY_REQUIRED) and we can
//     do the work in the current thread we will translate the status code
//     to STATUS_CANT_WAIT to indicate that we need to retry the request.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
//     ExceptionCode - Supplies the normalized exception status being handled
//
// Return Value:
//
//     NTSTATUS - Returns the results of either posting the Irp or the
//         saved completion status.
//
// --*/
//
// {
//     PDEVICE_OBJECT Device = NULL;
//     PVPB Vpb;
//     PETHREAD Thread;
//
//     ASSERT_OPTIONAL_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  If there is not an irp context, then complete the request with the
//     //  current status code.
//     //
//
//     if (!ARGUMENT_PRESENT( IrpContext )) {
//
//         CdCompleteRequest( NULL, Irp, ExceptionCode );
//         return ExceptionCode;
//     }
//
//     //
//     //  Get the real exception status from the IrpContext.
//     //
//
//     ExceptionCode = IrpContext->ExceptionStatus;
//
//     //
//     //  Check if we are posting this request.  One of the following must be true
//     //  if we are to post a request.
//     //
//     //      - Status code is STATUS_CANT_WAIT and the request is asynchronous
//     //          or we are forcing this to be posted.
//     //
//     //      - Status code is STATUS_VERIFY_REQUIRED and we are at APC level
//     //          or higher, or within a guarded region.  Can't wait for IO in
//     //          the verify path in this case.
//     //
//     //  Set the MORE_PROCESSING flag in the IrpContext to keep if from being
//     //  deleted if this is a retryable condition.
//     //
//     //
//     //  Note that (children of) CdFsdPostRequest can raise (Mdl allocation).
//     //
//
//     _SEH2_TRY {
//
//         if (ExceptionCode == STATUS_CANT_WAIT) {
//
//             if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_FORCE_POST )) {
//
//                 ExceptionCode = CdFsdPostRequest( IrpContext, Irp );
//             }
//         }
//         else if ((ExceptionCode == STATUS_VERIFY_REQUIRED) &&
//                  FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL ) &&
//                  KeAreAllApcsDisabled()) {
//
//             ExceptionCode = CdFsdPostRequest( IrpContext, Irp );
//         }
//     }
//     _SEH2_EXCEPT( CdExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() ))  {
//
//         ExceptionCode = _SEH2_GetExceptionCode();
//     } _SEH2_END;
//     //
//     //  If we posted the request or our caller will retry then just return here.
//     //
//
//     if ((ExceptionCode == STATUS_PENDING) ||
//         (ExceptionCode == STATUS_CANT_WAIT)) {
//
//         return ExceptionCode;
//     }
//
//     ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING );
//
//     //
//     //  If we are not a top level request then we just complete the request
//     //  with the current status code.
//     //
//
//     if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL )) {
//
//         CdCompleteRequest( IrpContext, Irp, ExceptionCode );
//         return ExceptionCode;
//     }
//
//     //
//     //  Store this error into the Irp for posting back to the Io system.
//     //
//
//     Irp->IoStatus.Status = ExceptionCode;
//
//     if (IoIsErrorUserInduced( ExceptionCode )) {
//
//         //
//         //  Check for the various error conditions that can be caused by,
//         //  and possibly resolved my the user.
//         //
//
//         if (ExceptionCode == STATUS_VERIFY_REQUIRED) {
//
//             //
//             //  Now we are at the top level file system entry point.
//             //
//             //  If we have already posted this request then the device to
//             //  verify is in the original thread.  Find this via the Irp.
//             //
//
//             Device = IoGetDeviceToVerify( Irp->Tail.Overlay.Thread );
//             IoSetDeviceToVerify( Irp->Tail.Overlay.Thread, NULL );
//
//             //
//             //  If there is no device in that location then check in the
//             //  current thread.
//             //
//
//             if (Device == NULL) {
//
//                 Device = IoGetDeviceToVerify( PsGetCurrentThread() );
//                 IoSetDeviceToVerify( PsGetCurrentThread(), NULL );
//
//                 NT_ASSERT( Device != NULL );
//
//             }
//
//             //
//             //  It turns out some storage drivers really do set invalid non-NULL device
//             //  objects to verify.
//             //
//             //  To work around this, completely ignore the device to verify in the thread,
//             //  and just use our real device object instead.
//             //
//
//             if (IrpContext->Vcb) {
//
//                 Device = IrpContext->Vcb->Vpb->RealDevice;
//             }
//
//             //
//             //  Let's not BugCheck just because the device to verify is somehow still NULL.
//             //
//
//             if (Device == NULL) {
//
//                 ExceptionCode = STATUS_DRIVER_INTERNAL_ERROR;
//
//                 CdCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//                 return ExceptionCode;
//             }
//
//             //
//             //  CdPerformVerify() will do the right thing with the Irp.
//             //  If we return STATUS_CANT_WAIT then the current thread
//             //  can retry the request.
//             //
//
//             return CdPerformVerify( IrpContext, Irp, Device );
//         }
//
//         //
//         //  The other user induced conditions generate an error unless
//         //  they have been disabled for this request.
//         //
//
//         if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS )) {
//
//             CdCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//             return ExceptionCode;
//
//         }
//         //
//         //  Generate a pop-up.
//         //
//         else {
//
//             if (IoGetCurrentIrpStackLocation( Irp )->FileObject != NULL) {
//
//                 Vpb = IoGetCurrentIrpStackLocation( Irp )->FileObject->Vpb;
//
//             } else {
//
//                 Vpb = NULL;
//             }
//
//
//             //
//             //  The device to verify is either in my thread local storage
//             //  or that of the thread that owns the Irp.
//             //
//
//             Thread = Irp->Tail.Overlay.Thread;
//             Device = IoGetDeviceToVerify( Thread );
//
//             if (Device == NULL) {
//
//                 Thread = PsGetCurrentThread();
//                 Device = IoGetDeviceToVerify( Thread );
//
//                 NT_ASSERT( Device != NULL );
//             }
//
//             //
//             //  It turns out some storage drivers really do set invalid non-NULL device
//             //  objects to verify.
//             //
//             //  To work around this, completely ignore the device to verify in the thread,
//             //  and just use our real device object instead.
//             //
//
//             if (IrpContext->Vcb) {
//
//                 Device = IrpContext->Vcb->Vpb->RealDevice;
//             }
//
//             //
//             //  Let's not BugCheck just because the device to verify is somehow still NULL.
//             //
//
//             if (Device == NULL) {
//
//                 CdCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//                 return ExceptionCode;
//             }
//
//             //
//             //  This routine actually causes the pop-up.  It usually
//             //  does this by queuing an APC to the callers thread,
//             //  but in some cases it will complete the request immediately,
//             //  so it is very important to IoMarkIrpPending() first.
//             //
//
//             IoMarkIrpPending( Irp );
//             IoRaiseHardError( Irp, Vpb, Device );
//
//             //
//             //  We will be handing control back to the caller here, so
//             //  reset the saved device object.
//             //
//
//             IoSetDeviceToVerify( Thread, NULL );
//
//             //
//             //  The Irp will be completed by Io or resubmitted.  In either
//             //  case we must clean up the IrpContext here.
//             //
//
//             CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//             return STATUS_PENDING;
//         }
//     }
//
//     //
//     //  This is just a run of the mill error.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//     return ExceptionCode;
// }
//
//
// VOID
// CdCompleteRequest (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      NTSTATUS Status
//     )
//
// /*++
//
// Routine Description:
//
//     This routine completes a Irp and cleans up the IrpContext.  Either or
//     both of these may not be specified.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed.
//
//     Status - Supplies the status to complete the Irp with
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ASSERT_OPTIONAL_IRP_CONTEXT( IrpContext );
//     ASSERT_OPTIONAL_IRP( Irp );
//
//     //
//     //  Cleanup the IrpContext if passed in here.
//     //
//
//     if (ARGUMENT_PRESENT( IrpContext )) {
//
//         CdCleanupIrpContext( IrpContext, FALSE );
//     }
//
//     //
//     //  If we have an Irp then complete the irp.
//     //
//
//     if (ARGUMENT_PRESENT( Irp )) {
//
//         //
//         //  Clear the information field in case we have used this Irp
//         //  internally.
//         //
//
//         if (NT_ERROR( Status ) &&
//             FlagOn( Irp->Flags, IRP_INPUT_OPERATION )) {
//
//             Irp->IoStatus.Information = 0;
//         }
//
//         Irp->IoStatus.Status = Status;
//
//         AssertVerifyDeviceIrp( Irp );
//
//         IoCompleteRequest( Irp, IO_CD_ROM_INCREMENT );
//     }
//
//     return;
// }
//
//
// VOID
// CdSetThreadContext (
//      PIRP_CONTEXT IrpContext,
//      PTHREAD_CONTEXT ThreadContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called at each Fsd/Fsp entry point set up the IrpContext
//     and thread local storage to track top level requests.  If there is
//     not a Cdfs context in the thread local storage then we use the input one.
//     Otherwise we use the one already there.  This routine also updates the
//     IrpContext based on the state of the top-level context.
//
//     If the TOP_LEVEL flag in the IrpContext is already set when we are called
//     then we force this request to appear top level.
//
// Arguments:
//
//     ThreadContext - Address on stack for local storage if not already present.
//
//     ForceTopLevel - We force this request to appear top level regardless of
//         any previous stack value.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PTHREAD_CONTEXT CurrentThreadContext;
// #ifdef __REACTOS__
//     ULONG_PTR StackTop;
//     ULONG_PTR StackBottom;
// #endif
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//
//     //
//     //  Get the current top-level irp out of the thread storage.
//     //  If NULL then this is the top-level request.
//     //
//
//     CurrentThreadContext = (PTHREAD_CONTEXT) IoGetTopLevelIrp();
//
//     if (CurrentThreadContext == NULL) {
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL );
//     }
//
//     //
//     //  Initialize the input context unless we are using the current
//     //  thread context block.  We use the new block if our caller
//     //  specified this or the existing block is invalid.
//     //
//     //  The following must be true for the current to be a valid Cdfs context.
//     //
//     //      Structure must lie within current stack.
//     //      Address must be ULONG aligned.
//     //      Cdfs signature must be present.
//     //
//     //  If this is not a valid Cdfs context then use the input thread
//     //  context and store it in the top level context.
//     //
//
// #ifdef __REACTOS__
//     IoGetStackLimits( &StackTop, &StackBottom);
// #endif
//
// #ifdef _MSC_VER
// #pragma warning(suppress: 6011) // Bug in PREFast around bitflag operations
// #endif
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL ) ||
// #ifndef __REACTOS__
//         (!IoWithinStackLimits( (ULONG_PTR)CurrentThreadContext, sizeof( THREAD_CONTEXT ) ) ||
// #else
//         (((ULONG_PTR) CurrentThreadContext > StackBottom - sizeof( THREAD_CONTEXT )) ||
//          ((ULONG_PTR) CurrentThreadContext <= StackTop) ||
// #endif
//          FlagOn( (ULONG_PTR) CurrentThreadContext, 0x3 ) ||
//          (CurrentThreadContext->Cdfs != 0x53464443))) {
//
//         ThreadContext->Cdfs = 0x53464443;
//         ThreadContext->SavedTopLevelIrp = (PIRP) CurrentThreadContext;
//         ThreadContext->TopLevelIrpContext = IrpContext;
//         IoSetTopLevelIrp( (PIRP) ThreadContext );
//
//         IrpContext->TopLevel = IrpContext;
//         IrpContext->ThreadContext = ThreadContext;
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL_CDFS );
//
//     //
//     //  Otherwise use the IrpContext in the thread context.
//     //
//
//     } else {
//
//         IrpContext->TopLevel = CurrentThreadContext->TopLevelIrpContext;
//     }
//
//     return;
// }
//
//
// _Function_class_(FAST_IO_CHECK_IF_POSSIBLE)
// _IRQL_requires_same_
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastIoCheckIfPossible (
//      PFILE_OBJECT FileObject,
//      PLARGE_INTEGER FileOffset,
//      ULONG Length,
//      BOOLEAN Wait,
//      ULONG LockKey,
//      BOOLEAN CheckForReadOperation,
//     
//     (return != FALSE, (_Old_(IoStatus)))
//     
//     PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if fast i/o is possible for a read/write operation
//
// Arguments:
//
//     FileObject - Supplies the file object used in the query
//
//     FileOffset - Supplies the starting byte offset for the read/write operation
//
//     Length - Supplies the length, in bytes, of the read/write operation
//
//     Wait - Indicates if we can wait
//
//     LockKey - Supplies the lock key
//
//     CheckForReadOperation - Indicates if this is a check for a read or write
//         operation
//
//     IoStatus - Receives the status of the operation if our return value is
//         FastIoReturnError
//
// Return Value:
//
//     BOOLEAN - TRUE if fast I/O is possible and FALSE if the caller needs
//         to take the long route.
//
// --*/
//
// {
//     PFCB Fcb;
//     TYPE_OF_OPEN TypeOfOpen;
//     LARGE_INTEGER LargeLength;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Wait );
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Decode the type of file object we're being asked to process and
//     //  make sure that is is only a user file open.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     if ((TypeOfOpen != UserFileOpen) || !CheckForReadOperation) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         return TRUE;
//     }
//
//     LargeLength.QuadPart = Length;
//
//     //
//     //  Check whether the file locks will allow for fast io.
//     //
//
//     if ((Fcb->FileLock == NULL) ||
//         FsRtlFastCheckLockForRead( Fcb->FileLock,
//                                    FileOffset,
//                                    &LargeLength,
//                                    LockKey,
//                                    FileObject,
//                                    PsGetCurrentProcess() )) {
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// ULONG
// CdSerial32 (
//      PCHAR Buffer,
//      ULONG ByteCount
//     )
// /*++
//
// Routine Description:
//
//     This routine is called to generate a 32 bit serial number.  This is
//     done by doing four separate checksums into an array of bytes and
//     then treating the bytes as a ULONG.
//
// Arguments:
//
//     Buffer - Pointer to the buffer to generate the ID for.
//
//     ByteCount - Number of bytes in the buffer.
//
// Return Value:
//
//     ULONG - The 32 bit serial number.
//
// --*/
//
// {
//     union {
//         UCHAR   Bytes[4];
//         ULONG   SerialId;
//     } Checksum;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the serial number.
//     //
//
//     Checksum.SerialId = 0;
//
//     //
//     //  Continue while there are more bytes to use.
//     //
//
//     while (ByteCount--) {
//
//         //
//         //  Increment this sub-checksum.
//         //
//
//         Checksum.Bytes[ByteCount & 0x3] += *(Buffer++);
//     }
//
//     //
//     //  Return the checksums as a ULONG.
//     //
//
//     return Checksum.SerialId;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
