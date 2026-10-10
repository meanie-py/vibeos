use ntkrnl_sys::globals::*;
// The real definitions, so the compiler checks every call against them.
use crate::timesup::{FatFatDateToNtTime, FatFatTimeToNtTime};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLONG, ExLocalTimeToSystemTime, FILE_BOTH_DIR_INFORMATION, FILE_DIRECTORY_INFORMATION, FILE_FULL_DIR_INFORMATION, FILE_ID_BOTH_DIR_INFORMATION, FILE_ID_FULL_DIR_INFORMATION, FILE_NAMES_INFORMATION, IoSetTopLevelIrp, OEM_STRING, PFILE_BOTH_DIR_INFORMATION, PFILE_DIRECTORY_INFORMATION, PFILE_FULL_DIR_INFORMATION, PFILE_ID_BOTH_DIR_INFORMATION, PFILE_ID_FULL_DIR_INFORMATION, PFILE_NAMES_INFORMATION, PIO_STACK_LOCATION, PIRP, PSTRING, PUNICODE_STRING, PWCH, RtlFreeOemString, RtlOemToUnicodeN, RtlUpcaseUnicodeString, RtlUpcaseUnicodeStringToCountedOemString};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DCB, DIRENT, FAT_DATA, FAT_TIME, IRP_CONTEXT, VCB};
use ntkrnl_sys::ddk::{IO_STACK_LOCATION, IRP, PFILE_OBJECT, PNOTIFY_SYNC};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlDoesNameContainWildCards, FsRtlEnterFileSystem, FsRtlExitFileSystem};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_UNICODE_STRING};

extern "system" {
    static mut FatData: FAT_DATA;
    static FatJanOne1980: ntkrnl_sys::ddk::LARGE_INTEGER;
    fn UInt32x32To64(a: u32, b: u32) -> u64;
    static Fat8QMdot3QM: [u16; 12];
    static Dbg: u32;
    } extern "C" { fn DebugTrace(level: i32, mask: u32, format: *const i8, ...); } extern "system" {
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: i32);
}
extern "system" {
    fn DebugUnwind(name: *const i8) -> ();
    fn FlagOn(Flags: u32, Flag: u32) -> i32;
    fn SetFlag(Flags: *mut u32, Flag: u32) -> ();
    fn BooleanFlagOn(Flags: u32, Flag: u32) -> u8;
    fn FatDecodeFileObject(FileObject: PFILE_OBJECT, Vcb: *mut *mut VCB, Dcb: *mut *mut DCB, Ccb: *mut *mut CCB) -> u32;
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB) -> u8;
    fn FatFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS) -> !;
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB) -> ();
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut c_void) -> ();
    fn FatFreeStringBuffer(String: *mut c_void) -> ();
    fn FatSetFullFileNameInFcb(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB) -> ();
    fn FatVerifyFcb(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB) -> ();
    fn FsRtlNotifyFullChangeDirectory( NotifySync: PNOTIFY_SYNC, NotifyList: PLIST_ENTRY, FsContext: *mut c_void, FullDirectoryName: PSTRING, WatchTree: BOOLEAN, IgnoreBuffer: BOOLEAN, CompletionFilter: ULONG, NotifyIrp: *mut IRP, TraverseCallback: PVOID, SubjectContext: *mut c_void, ) -> ();
    fn ClearFlag(Flags: *mut u32, Flag: u32);
    fn Fat8dot3ToString( IrpContext: *mut core::ffi::c_void, Dirent: *mut core::ffi::c_void, Upcase: u8, String: *mut OEM_STRING, );
    fn FatAcquireSharedFcb( IrpContext: *mut core::ffi::c_void, Dcb: *mut core::ffi::c_void, ) -> u8;
    fn FatBugCheck(arg1: u32, arg2: u32, arg3: u32);
    fn FatConvertToSharedFcb( IrpContext: *mut core::ffi::c_void, Dcb: *mut core::ffi::c_void, );
    fn FatGetEaLength( IrpContext: *mut core::ffi::c_void, Vcb: *mut core::ffi::c_void, Dirent: *mut core::ffi::c_void, EaSize: *mut u32, );
    fn FatIsNameShortOemValid( IrpContext: *mut core::ffi::c_void, OemString: OEM_STRING, ContainsWildCards: u8, AllowWildCards: u8, IsConstant: u8, ) -> u8;
    fn FatLocateDirent( IrpContext: *mut core::ffi::c_void, Dcb: *mut core::ffi::c_void, Ccb: *mut core::ffi::c_void, CurrentVbo: u32, MatchFlags: *mut u32, Dirent: *mut *mut core::ffi::c_void, Bcb: *mut *mut core::ffi::c_void, NextVbo: *mut u32, FileNameDos: *mut u8, LongFileName: *mut _UNICODE_STRING, OrigFileName: *mut _UNICODE_STRING, );
    fn FatMapUserBuffer( IrpContext: *mut core::ffi::c_void, Irp: *mut core::ffi::c_void, ) -> *mut u8;
    fn FatStringTo8dot3( IrpContext: *mut core::ffi::c_void, LocalBestFit: OEM_STRING, Constant: *mut OEM_STRING, );
    fn FatResetExceptionState(IrpContext: *mut core::ffi::c_void);
    fn NT_ASSERT(condition: u8);
    fn RtlEqualMemory(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, length: u32) -> u8;
    fn RtlOemStringToCountedUnicodeSize(OemString: *const OEM_STRING) -> u32;
    fn QuadAlign(x: u32) -> u32;
    // try_return was a no-op cohab stub — it is C's `try_return(X)` macro
    // (Status = X; goto try_exit), now inlined as `return;`/do_return at each
    // call site (see FatQueryDirectory). Do NOT re-introduce it as an extern fn.
    fn CanFsdWait(Irp: PIRP) -> i32;
    fn FatIsIrpTopLevel(Irp: PIRP) -> u8;
    fn FatCreateIrpContext(Irp: PIRP, CanWait: i32) -> PIRP_CONTEXT;
    fn FatProcessException(IrpContext: PIRP_CONTEXT, Irp: PIRP, ExceptionCode: u32) -> NTSTATUS;
}
extern "C" {
    // wdm FORCEINLINE helper: cdecl body, bindgen class — was "system" (§9.99).
    fn IoGetCurrentIrpStackLocation(Irp: *mut IRP) -> PIO_STACK_LOCATION;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_DIRCTRL;

const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // placeholder  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const FCB_STATE_DELETE_ON_CLOSE: u32 = 0x01; // placeholder
const SL_WATCH_TREE: u32 = 0x01; // placeholder
const UserDirectoryOpen: u32 = crate::types::_TYPE_OF_OPEN_UserDirectoryOpen; // 3 — bindgen ground truth (was 1)

// External declarations for unknown functions

// Required constants
const CCB_FLAG_MATCH_ALL: u32 = 0x00000001;
const CCB_FLAG_FREE_UNICODE: u32 = 0x08;
const CCB_FLAG_FREE_OEM_BEST_FIT: u32 = 0x00000004;
const CCB_FLAG_SKIP_SHORT_NAME_COMPARE: u32 = 0x02;
const CCB_FLAG_QUERY_TEMPLATE_MIXED: u32 = 0x1000;
const TAG_FILENAME_BUFFER: u32 = 0x6E746146; // 'NamB'
const FAT_CREATE_INITIAL_NAME_BUF_SIZE: u32 = 0x20; // example
const DOS_QM: u16 = b'?' as u16;
const FAT_BUG_CHECK_DIRCTRL: u32 = 0x00090000;
const DEBUG_TRACE_DIRCTRL: u32 = 0x80;
const SL_RESTART_SCAN: u32 = 0x00000001;
const SL_RETURN_SINGLE_ENTRY: u32 = 0x00000002;
const SL_INDEX_SPECIFIED: u32 = 0x00000004;
const FAT_DIRENT_ATTR_DIRECTORY: u32 = 0x10;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as i32;
const STATUS_NO_SUCH_FILE: NTSTATUS = 0xC000000Fu32 as i32;
const STATUS_NO_MORE_FILES: NTSTATUS = 0x80000006u32 as i32;
const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005u32 as i32;
const STATUS_INVALID_INFO_CLASS: NTSTATUS = 0xC0000003u32 as i32;
const STATUS_UNMAPPABLE_CHARACTER: NTSTATUS = 0xC0000162_u32 as i32;
const STATUS_SUCCESS: NTSTATUS = 0;
#[allow(dead_code)] // C `try_return(NOTHING)` sentinel; now inlined as a Status-preserving return
const NOTHING: NTSTATUS = 0;

type NTSTATUS = i32;
// fat.h:38: `typedef ULONG32 VBO` — 4 bytes, not 8. The local u64 alias here
// happened to match dirsup's equally-wrong LARGE_INTEGER, so the two bugs hid
// each other; both now defer to the one authoritative definition.
use crate::types::VBO;
type PIRP_CONTEXT = *mut core::ffi::c_void;
type PVCB = *mut core::ffi::c_void;
type PDCB = *mut core::ffi::c_void;
type PCCB = *mut core::ffi::c_void;
type PBCB = *mut core::ffi::c_void;
type PDIRENT = *mut core::ffi::c_void;

// File-level global array (assumed available)

#[no_mangle]
pub unsafe extern "system" fn FatFsdDirectoryControl(
    VolumeDeviceObject: *mut core::ffi::c_void,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();

    PAGED_CODE!();
    DebugTrace(1, Dbg, b"FatFsdDirectoryControl\n\0".as_ptr() as *const i8);

    FsRtlEnterFileSystem();

    // FatIsIrpTopLevel makes this Irp the top-level one when there is none; the
    // translation also cleared it again here, before the try, which C does not do.
    let TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY { ... } _SEH2_EXCEPT( FatExceptionFilter( IrpContext, ... ) )
    let p_ic = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_ic as *const PIRP_CONTEXT, || {
        *p_ic = FatCreateIrpContext(Irp, CanFsdWait(Irp));
        *p_status = FatCommonDirectoryControl(*p_ic as *mut IRP_CONTEXT, Irp);
    }) {
        Status = FatProcessException(IrpContext, Irp, code as u32);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();

    DebugTrace(-1, Dbg, b"FatFsdDirectoryControl -> %08lx\n\0".as_ptr() as *const i8, Status);

    UNREFERENCED_PARAMETER!(VolumeDeviceObject);

    Status
}


#[no_mangle]
pub unsafe extern "system" fn FatCommonDirectoryControl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> i32 {
    // PAGED_CODE();

    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatCommonDirectoryControl\n\0".as_ptr() as *const i8, 0_i32);
    DebugTrace(0, Dbg, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as *mut core::ffi::c_void);
    DebugTrace(0, Dbg, b"MinorFunction = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).MinorFunction as u32);

    let Status: i32;

    const IRP_MN_QUERY_DIRECTORY: u8 = 0x01;
    const IRP_MN_NOTIFY_CHANGE_DIRECTORY: u8 = 0x02;

    match (*IrpSp).MinorFunction {
        IRP_MN_QUERY_DIRECTORY => {
            Status = FatQueryDirectory(IrpContext as PIRP_CONTEXT, Irp);
        }
        IRP_MN_NOTIFY_CHANGE_DIRECTORY => {
            Status = FatNotifyChangeDirectory(IrpContext as *mut IRP_CONTEXT, Irp);
        }
        _ => {
            DebugTrace(
                0,
                Dbg,
                b"Invalid Directory Control Minor Function %08lx\n\0".as_ptr() as *const i8,
                (*IrpSp).MinorFunction as u32,
            );
            FatCompleteRequest(IrpContext, Irp, 0xC0000010_u32 as i32); // STATUS_INVALID_DEVICE_REQUEST
            Status = 0xC0000010_u32 as i32; // STATUS_INVALID_DEVICE_REQUEST
        }
    }

    DebugTrace(-1_i32, Dbg, b"FatCommonDirectoryControl -> %08lx\n\0".as_ptr() as *const i8, Status as u32);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatQueryDirectory(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpSp: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Dcb: *mut DCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let mut Bcb: *mut core::ffi::c_void = core::ptr::null_mut();

    let mut i: u32 = 0;
    let mut Buffer: *mut u8 = core::ptr::null_mut();
    let mut UserBufferLength: CLONG = 0;

    let mut UniArgFileName: PUNICODE_STRING = core::ptr::null_mut();
    let mut LongFileNameBuffer: [u16; FAT_CREATE_INITIAL_NAME_BUF_SIZE as usize] = [0u16; FAT_CREATE_INITIAL_NAME_BUF_SIZE as usize];
    let mut LongFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut OrigFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileInformationClass: u32 = 0;
    let mut FileIndex: u32 = 0;
    let mut MatchFlags: u32 = 0;
    let mut RestartScan: u8 = 0;
    let mut ReturnSingleEntry: u8 = 0;
    let mut IndexSpecified: u8 = 0;

    let mut InitialQuery: u8 = 0;
    let mut CurrentVbo: VBO = 0;
    let mut UpdateCcb: u8 = 0;
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut Fat8Dot3Buffer: [u8; 12] = [0u8; 12];
    let mut Fat8Dot3String: OEM_STRING = core::mem::zeroed();
    let mut DiskAllocSize: u32 = 0;

    let mut NextEntry: u32 = 0;
    let mut LastEntry: u32 = 0;

    let mut DirInfo: *mut FILE_DIRECTORY_INFORMATION = core::ptr::null_mut();
    let mut FullDirInfo: *mut FILE_FULL_DIR_INFORMATION = core::ptr::null_mut();
    let mut BothDirInfo: *mut FILE_BOTH_DIR_INFORMATION = core::ptr::null_mut();
    let mut IdFullDirInfo: *mut FILE_ID_FULL_DIR_INFORMATION = core::ptr::null_mut();
    let mut IdBothDirInfo: *mut FILE_ID_BOTH_DIR_INFORMATION = core::ptr::null_mut();
    let mut NamesInfo: *mut FILE_NAMES_INFORMATION = core::ptr::null_mut();

    // PAGED_CODE(); - placeholder, could be a function call or attribute

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatQueryDirectory...\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b" Wait                   = %08lx\n\0".as_ptr() as *const i8, FlagOn((*(IrpContext as *mut IRP_CONTEXT)).Flags, IRP_CONTEXT_FLAG_WAIT));
    DebugTrace(0, Dbg, b" Irp                    = %p\n\0".as_ptr() as *const i8, Irp);
    DebugTrace(0, Dbg, b" ->Length               = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.QueryDirectory.Length);
    DebugTrace(0, Dbg, b" ->FileName             = %wZ\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.QueryDirectory.FileName);
    DebugTrace(0, Dbg, b" ->FileInformationClass = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.QueryDirectory.FileInformationClass);
    DebugTrace(0, Dbg, b" ->FileIndex            = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.QueryDirectory.FileIndex);
    DebugTrace(0, Dbg, b" ->UserBuffer           = %p\n\0".as_ptr() as *const i8, (*Irp).AssociatedIrp.SystemBuffer);
    DebugTrace(0, Dbg, b" ->RestartScan          = %08lx\n\0".as_ptr() as *const i8, FlagOn(((*IrpSp).Flags) as u32, SL_RESTART_SCAN));
    DebugTrace(0, Dbg, b" ->ReturnSingleEntry    = %08lx\n\0".as_ptr() as *const i8, FlagOn(((*IrpSp).Flags) as u32, SL_RETURN_SINGLE_ENTRY));
    DebugTrace(0, Dbg, b" ->IndexSpecified       = %08lx\n\0".as_ptr() as *const i8, FlagOn(((*IrpSp).Flags) as u32, SL_INDEX_SPECIFIED));

    UserBufferLength = (*IrpSp).Parameters.QueryDirectory.Length;
    FileInformationClass = (*IrpSp).Parameters.QueryDirectory.FileInformationClass;
    FileIndex = (*IrpSp).Parameters.QueryDirectory.FileIndex;

    UniArgFileName = (*IrpSp).Parameters.QueryDirectory.FileName;

    RestartScan = BooleanFlagOn(((*IrpSp).Flags) as u32, SL_RESTART_SCAN) as u8;
    ReturnSingleEntry = BooleanFlagOn(((*IrpSp).Flags) as u32, SL_RETURN_SINGLE_ENTRY) as u8;
    IndexSpecified = BooleanFlagOn(((*IrpSp).Flags) as u32, SL_INDEX_SPECIFIED) as u8;

    if FatDecodeFileObject((*IrpSp).FileObject, &mut Vcb, &mut Dcb, &mut Ccb) != UserDirectoryOpen
        || (!UniArgFileName.is_null()
            && (*UniArgFileName).Length % core::mem::size_of::<u16>() as u16 != 0)
    {
        FatCompleteRequest(IrpContext as *mut IRP_CONTEXT, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatQueryDirectory -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8, 0);
        return STATUS_INVALID_PARAMETER;
    }

    Bcb = core::ptr::null_mut();
    UpdateCcb = 1;
    Dirent = core::ptr::null_mut();

    Fat8Dot3String.MaximumLength = 12;
    Fat8Dot3String.Buffer = Fat8Dot3Buffer.as_mut_ptr() as *mut i8;

    LongFileName.Length = 0;
    LongFileName.MaximumLength = core::mem::size_of_val(&LongFileNameBuffer) as u16;
    LongFileName.Buffer = LongFileNameBuffer.as_mut_ptr();

    InitialQuery = ((*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer == core::ptr::null_mut()
        && FlagOn((*Ccb).Flags, CCB_FLAG_MATCH_ALL) == 0) as u8;
    Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    DiskAllocSize = 1 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

    if InitialQuery != 0 || RestartScan != 0 {
        if FatAcquireExclusiveFcb(IrpContext as *mut IRP_CONTEXT, Dcb) == 0 {
            DebugTrace(0, Dbg, b"FatQueryDirectory -> Enqueue to Fsp\n\0".as_ptr() as *const i8, 0);
            Status = FatFsdPostRequest(IrpContext as *mut IRP_CONTEXT, Irp);
            DebugTrace(-1, Dbg, b"FatQueryDirectory -> %08lx\n\0".as_ptr() as *const i8, Status);
            return Status;
        }
        if RestartScan == 0 && !(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer.is_null() {
            InitialQuery = 0;
            FatConvertToSharedFcb(IrpContext, Dcb as *mut c_void);
        }
    } else {
        if FatAcquireSharedFcb(IrpContext, Dcb as *mut c_void) == 0 {
            DebugTrace(0, Dbg, b"FatQueryDirectory -> Enqueue to Fsp\n\0".as_ptr() as *const i8, 0);
            Status = FatFsdPostRequest(IrpContext as *mut IRP_CONTEXT, Irp);
            DebugTrace(-1, Dbg, b"FatQueryDirectory -> %08lx\n\0".as_ptr() as *const i8, Status);
            return Status;
        }
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_Status = core::ptr::addr_of_mut!(Status);
    let p_UpdateCcb = core::ptr::addr_of_mut!(UpdateCcb);
    let p_CurrentVbo = core::ptr::addr_of_mut!(CurrentVbo);
    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    let p_LongFileName = core::ptr::addr_of_mut!(LongFileName);
    crate::seh::try_finally(&mut (), |_| {
        let mut BaseLength: u32 = 0;
        let mut BytesConverted: u32 = 0;

        Buffer = FatMapUserBuffer(IrpContext, (Irp).cast());
        FatVerifyFcb(IrpContext as *mut IRP_CONTEXT, Dcb);

        if IndexSpecified != 0 {
            *p_CurrentVbo = FileIndex as VBO + core::mem::size_of::<DIRENT>() as VBO;
        } else if RestartScan != 0 {
            *p_CurrentVbo = 0;
            (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OffsetToStartSearchFrom = 0;
        } else {
            *p_CurrentVbo = (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OffsetToStartSearchFrom;
        }

        if InitialQuery != 0 || (RestartScan != 0 && !UniArgFileName.is_null() && (*UniArgFileName).Length != 0) {
            if RestartScan != 0 {
                if !(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer.is_null() {
                    if FlagOn((*Ccb).Flags, CCB_FLAG_FREE_UNICODE) != 0 {
                        ExFreePoolWithTag((*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer as *mut core::ffi::c_void, TAG_FILENAME_BUFFER);
                        ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_UNICODE);
                    }
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Buffer = core::ptr::null_mut();
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Length = 0;
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.MaximumLength = 0;
                }
                if !(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Wild.as_ref().Buffer.is_null() {
                    if FlagOn((*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT) != 0 {
                        RtlFreeOemString((*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Wild.as_mut() as *mut OEM_STRING);
                        ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT);
                    }
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Wild.as_mut().Buffer = core::ptr::null_mut();
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Wild.as_mut().Length = 0;
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Wild.as_mut().MaximumLength = 0;
                }
                (*Ccb).ContainsWildCards = 0;
                ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_MATCH_ALL);
                ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_UNICODE);
                ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE);
                ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_QUERY_TEMPLATE_MIXED);
            }

            if UniArgFileName.is_null()
                || (*UniArgFileName).Length == 0
                || (*UniArgFileName).Buffer.is_null()
                || ((*UniArgFileName).Length == core::mem::size_of::<u16>() as u16
                    && *(*UniArgFileName).Buffer == '*' as u16)
                || ((*UniArgFileName).Length == 12 * core::mem::size_of::<u16>() as u16
                    && RtlEqualMemory(
                        (*UniArgFileName).Buffer as *const core::ffi::c_void,
                        Fat8QMdot3QM.as_ptr() as *const core::ffi::c_void,
                        12 * core::mem::size_of::<u16>() as u32,
                    ) != 0)
            {
                (*Ccb).ContainsWildCards = 1;
                SetFlag(&mut (*Ccb).Flags, CCB_FLAG_MATCH_ALL);
            } else {
                let mut ExtendedName: u8 = 0;
                let mut LocalBestFit: OEM_STRING = core::mem::zeroed();

                (*Ccb).ContainsWildCards = FsRtlDoesNameContainWildCards(UniArgFileName);

                for i in 0..(*UniArgFileName).Length as usize / core::mem::size_of::<u16>() {
                    if *(*UniArgFileName).Buffer.add(i) >= 0x80 {
                        ExtendedName = 1;
                        break;
                    }
                }

                if ExtendedName != 0 {
                    *p_Status = RtlUpcaseUnicodeString(&mut (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate, UniArgFileName, 1);
                    if (*p_Status) < 0 {
                        return; // try_returnStatus
                    }
                    SetFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_UNICODE);

                    *p_Status = RtlUpcaseUnicodeStringToCountedOemString(&mut LocalBestFit, UniArgFileName, 1);
                    if (*p_Status) < 0 {
                        if (*p_Status) == STATUS_UNMAPPABLE_CHARACTER {
                            SetFlag(&mut (*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE);
                        } else {
                            return; // try_returnStatus
                        }
                    } else {
                        SetFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT);
                    }
                } else {
                    let Buffers: *mut core::ffi::c_void;

                    Buffers = FsRtlAllocatePoolWithTag(
                        PagedPool,
                        (*UniArgFileName).Length as u32 + (*UniArgFileName).Length as u32 / core::mem::size_of::<u16>() as u32,
                        TAG_FILENAME_BUFFER,
                    );
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Buffer = Buffers as *mut u16;
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Length = (*UniArgFileName).Length;
                    (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.MaximumLength = (*UniArgFileName).Length;
                    LocalBestFit.Buffer =
                        (Buffers as *mut u8).add((*UniArgFileName).Length as usize) as *mut i8;
                    LocalBestFit.Length = ((*UniArgFileName).Length as u32 / core::mem::size_of::<u16>() as u32) as u16;
                    LocalBestFit.MaximumLength = LocalBestFit.Length;

                    SetFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_UNICODE);

                    for i in 0..(*UniArgFileName).Length as usize / core::mem::size_of::<u16>() {
                        let c: u16 = *(*UniArgFileName).Buffer.add(i);
                        *(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Buffer.add(i) = if c < 'a' as u16 {
                            c
                        } else if c <= 'z' as u16 {
                            c - ('a' as u16 - 'A' as u16)
                        } else {
                            c
                        };
                        *LocalBestFit.Buffer.add(i) = *(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer.add(i) as i8;
                    }
                }

                if FlagOn((*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE) == 0
                    && FatIsNameShortOemValid(IrpContext, LocalBestFit, (*Ccb).ContainsWildCards, 0, 0) == 0
                {
                    if ExtendedName != 0 {
                        RtlFreeOemString(&mut LocalBestFit);
                        ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT);
                    }
                    SetFlag(&mut (*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE);
                }

                if FlagOn((*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE) == 0 {
                    if (*Ccb).ContainsWildCards != 0 {
                        *(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Wild.as_mut() = LocalBestFit;
                    } else {
                        FatStringTo8dot3(IrpContext, LocalBestFit, (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Constant.as_mut() as *mut _ as *mut OEM_STRING);
                        if FlagOn((*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT) != 0 {
                            RtlFreeOemString(&mut LocalBestFit);
                            ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_FREE_OEM_BEST_FIT);
                        }
                    }
                }
            }
            FatConvertToSharedFcb(IrpContext, Dcb as *mut c_void);
        }

        LastEntry = 0;
        NextEntry = 0;

        // FILE_INFORMATION_CLASS raw enum values (ntifs.h). The original
        // translation renumbered these switch labels 1..6 BY POSITION, so the
        // raw class from the IRP (line: FileInformationClass = ...QueryDirectory
        // .FileInformationClass, an unmapped enum) only matched for classes 1 &
        // 2 (accidentally). FindFirstFileW sends FileBothDirectoryInformation
        // (=3), which hit the positional arm '3' meant for FileIdFull — so the
        // Both body was never filled: zeroed FileAttributes + FileNameLength ->
        // empty-name/attr-0 WIN32_FIND_DATA -> malformed shell pidls (arm64
        // start menu, any FAT directory enumeration). x86 is immune only
        // because it boots CDFS off the ISO; arm64 is the sole fastfat boot.
        const FIC_DIRECTORY: u32 = 1;
        const FIC_FULL_DIRECTORY: u32 = 2;
        const FIC_BOTH_DIRECTORY: u32 = 3;
        const FIC_NAMES: u32 = 12;
        const FIC_ID_BOTH_DIRECTORY: u32 = 37;
        const FIC_ID_FULL_DIRECTORY: u32 = 38;

        match FileInformationClass {
            FIC_DIRECTORY => {
                BaseLength = core::mem::offset_of!(FILE_DIRECTORY_INFORMATION, FileName) as u32 + 0;
            }
            FIC_FULL_DIRECTORY => {
                BaseLength = core::mem::offset_of!(FILE_FULL_DIR_INFORMATION, FileName) as u32 + 0;
            }
            FIC_ID_FULL_DIRECTORY => {
                BaseLength = core::mem::offset_of!(FILE_ID_FULL_DIR_INFORMATION, FileName) as u32 + 0;
            }
            FIC_NAMES => {
                BaseLength = core::mem::offset_of!(FILE_NAMES_INFORMATION, FileName) as u32 + 0;
            }
            FIC_BOTH_DIRECTORY => {
                BaseLength = core::mem::offset_of!(FILE_BOTH_DIR_INFORMATION, FileName) as u32 + 0;
            }
            FIC_ID_BOTH_DIRECTORY => {
                BaseLength = core::mem::offset_of!(FILE_ID_BOTH_DIR_INFORMATION, FileName) as u32 + 0;
            }
            _ => {
                *p_Status = STATUS_INVALID_INFO_CLASS;
                return; // try_return(Status = STATUS_INVALID_INFO_CLASS)
            }
        }

        // NextVbo is the byte offset of the dirent FatLocateDirent found LAST time, and it
        // must survive into the next iteration: FatLocateDirent's pinned-page fast path
        // advances the still-pinned Dirent pointer by (CurrentVbo - *NextVbo) / 32. The C
        // declares it inside the loop UNINITIALISED and gets the old value back only
        // because the stack slot is reused. The translation's `= 0` per iteration made
        // every later call on the first directory page start from the wrong dirent: names
        // came back as bare 8.3 (mid-LFN), repeated, or skipped, and most of
        // \SystemRoot\Fonts (Marlett among them) was never enumerated on arm64's FAT
        // ramdisk. Hoisted so the value carries by construction. fastfat's VBO is a
        // ULONG32 (fat.h:38); this module's VBO comes from an ntkrnl_sys glob and is 64-bit,
        // so the declaration says u32 explicitly, as FatLocateDirent writes it.
        let mut NextVbo: u32 = 0;
        loop {
            let mut FileNameLength: u32 = 0;
            let mut BytesRemainingInBuffer: u32 = 0;
            let mut FileNameDos: u8 = 0;

            DebugTrace(0, Dbg, b"FatQueryDirectory -> Top of loop\n\0".as_ptr() as *const i8, 0);

            if ReturnSingleEntry != 0 && NextEntry != 0 {
                return; // try_returnStatus
            }

            FatLocateDirent(
                IrpContext,
                Dcb as *mut c_void,
                Ccb as *mut c_void,
                (*p_CurrentVbo) as u32,
                &mut MatchFlags,
                &mut Dirent as *mut *mut DIRENT as *mut *mut c_void,
                &mut (*p_Bcb),
                &mut NextVbo,
                &mut FileNameDos,
                &mut (*p_LongFileName),
                &mut OrigFileName,
            );

            if Dirent.is_null() {
                DebugTrace(0, Dbg, b"FatQueryDirectory -> No dirent\n\0".as_ptr() as *const i8, 0);
                if NextEntry == 0 {
                    *p_UpdateCcb = 0;
                    if InitialQuery != 0 {
                        *p_Status = STATUS_NO_SUCH_FILE;
                    } else {
                        *p_Status = STATUS_NO_MORE_FILES;
                    }
                }
                return; // try_returnStatus
            }

            // Inner _SEH2_TRY block.
            // try_return inside the inner closure can only `return` from the
            // closure, not out of the enclosing loop; this flag propagates the
            // C `goto try_exit` past the loop (checked right after the inner
            // try_except below).
            let mut do_return = false;
            let inner_result = crate::seh::try_except(&mut (), |_| {
                // SEH_TRY: start
                Fat8dot3ToString(IrpContext, Dirent as *mut c_void, 1, &mut Fat8Dot3String);

                if (*p_LongFileName).Length == 0 {
                    // First block: short name path
                    FileNameLength = RtlOemStringToCountedUnicodeSize(&Fat8Dot3String);
                    BytesRemainingInBuffer = UserBufferLength as u32 - NextEntry;

                    if NextEntry != 0
                        && (BaseLength + FileNameLength > BytesRemainingInBuffer
                            || (UserBufferLength as u32) < NextEntry)
                    {
                        DebugTrace(0, Dbg, b"Next entry won't fit\n\0".as_ptr() as *const i8, 0);
                        // try_return(Status = STATUS_SUCCESS) — inner closure: flag + break loop
                        *p_Status = STATUS_SUCCESS;
                        do_return = true;
                        return;
                    }

                    NT_ASSERT((BytesRemainingInBuffer >= BaseLength) as u8);
                    core::ptr::write_bytes(Buffer.add(NextEntry as usize), 0, BaseLength as usize);

                    // C switch FALLS THROUGH 5|2|6|3 -> 1: every dir-info class
                    // ALSO fills the FILE_DIRECTORY_INFORMATION body (times,
                    // sizes, FileAttributes, FileIndex, FileNameLength). The
                    // original translation lost the fallthrough — class 5
                    // (FileBothDirectoryInformation, the FindFirstFileW level)
                    // returned zeroed entries: empty names + attrs=0, which
                    // upstream turned into malformed shell pidls (arm64 start
                    // menu / any 2-level shell path on FAT volumes).
                    match FileInformationClass {
                        FIC_BOTH_DIRECTORY | FIC_FULL_DIRECTORY | FIC_ID_BOTH_DIRECTORY | FIC_ID_FULL_DIRECTORY | FIC_DIRECTORY => {
                            if FileInformationClass != FIC_DIRECTORY {
                                DebugTrace(0, Dbg, b"FatQueryDirectory -> Getting file full directory information\n\0".as_ptr() as *const i8, 0);
                                FullDirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_FULL_DIR_INFORMATION);
                                // inner SEH_TRY for GetEaLength
                                // _SEH2_TRY { FatGetEaLength } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                                let ea_result = crate::seh::try_except(&mut (), |_| {
                                    FatGetEaLength(IrpContext, Vcb as *mut c_void, Dirent as *mut c_void, &mut (*FullDirInfo).EaSize);
                                }, |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER);
                                if ea_result.is_err() {
                                    FatResetExceptionState(IrpContext);
                                    (*FullDirInfo).EaSize = 0;
                                }
                            }
                            // C fallthrough target: FileDirectoryInformation body
                            DirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_DIRECTORY_INFORMATION);
                            FatGetDirTimes(IrpContext as *mut IRP_CONTEXT, Dirent, DirInfo);
                            (*DirInfo).EndOfFile.QuadPart = (*Dirent).FileSize as i64;
                            if FlagOn((*Dirent).Attributes as u32, FAT_DIRENT_ATTR_DIRECTORY) == 0 {
                                (*DirInfo).AllocationSize.QuadPart =
                                    (((*Dirent).FileSize as i64 + DiskAllocSize as i64 - 1) / DiskAllocSize as i64)
                                        * DiskAllocSize as i64 ;
                            }
                            if (*Dirent).Attributes != 0 {
                                (*DirInfo).FileAttributes = (*Dirent).Attributes as u32;
                            } else {
                                (*DirInfo).FileAttributes = 0;
                                (*DirInfo).FileAttributes |= FILE_ATTRIBUTE_NORMAL;
                            }
                            (*DirInfo).FileIndex = (NextVbo) as u32;
                            (*DirInfo).FileNameLength = FileNameLength;
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Name = \"%Z\"\n\0".as_ptr() as *const i8, &Fat8Dot3String);
                        }
                        FIC_NAMES => {
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Getting file names information\n\0".as_ptr() as *const i8, 0);
                            NamesInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_NAMES_INFORMATION);
                            (*NamesInfo).FileIndex = (NextVbo) as u32;
                            (*NamesInfo).FileNameLength = FileNameLength;
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Name = \"%Z\"\n\0".as_ptr() as *const i8, &Fat8Dot3String);
                        }
                        _ => {
                            FatBugCheck(FileInformationClass, 0, 0);
                        }
                    }

                    BytesConverted = 0;
                    *p_Status = RtlOemToUnicodeN(
                        Buffer.add(NextEntry as usize + BaseLength as usize) as *mut u16,
                        BytesRemainingInBuffer - BaseLength,
                        &mut BytesConverted,
                        Fat8Dot3String.Buffer,
                        (Fat8Dot3String.Length) as u32,
                    );

                    if BytesConverted < FileNameLength {
                        NT_ASSERT((NextEntry == 0) as u8);
                        *p_Status = STATUS_BUFFER_OVERFLOW;
                    }

                    *(Buffer.add(LastEntry as usize) as *mut u32) = NextEntry - LastEntry;
                    (*Irp).IoStatus.Information = (QuadAlign((*Irp).IoStatus.Information as u32) as usize)
                        .wrapping_add(BaseLength as usize).wrapping_add(BytesConverted as usize) as _;

                    if (*p_Status) < 0 {
                        // try_return(NOTHING) — inner closure: preserve Status (error), break loop
                        do_return = true;
                        return;
                    }
                } else {
                    // Long filename path
                    let mut ShortNameLength: u32 = 0;
                    FileNameLength = (*p_LongFileName).Length as u32;

                    BytesRemainingInBuffer = UserBufferLength as u32 - NextEntry;

                    if NextEntry != 0
                        && (BaseLength + FileNameLength > BytesRemainingInBuffer
                            || (UserBufferLength as u32) < NextEntry)
                    {
                        DebugTrace(0, Dbg, b"Next entry won't fit\n\0".as_ptr() as *const i8, 0);
                        // try_return(Status = STATUS_SUCCESS) — inner closure: flag + break loop
                        *p_Status = STATUS_SUCCESS;
                        do_return = true;
                        return;
                    }

                    NT_ASSERT((BytesRemainingInBuffer >= BaseLength) as u8);
                    core::ptr::write_bytes(Buffer.add(NextEntry as usize), 0, BaseLength as usize);

                    // C switch FALLS THROUGH 5|6 -> 2|3 -> 1 (see the short-name
                    // block above for the full story): Both classes fill the
                    // ShortName, all non-1 classes fill EaSize, and EVERY class
                    // also fills the FILE_DIRECTORY_INFORMATION body. The lost
                    // fallthrough zeroed FileAttributes/FileNameLength for
                    // class 5 = FindFirstFileW on FAT.
                    match FileInformationClass {
                        FIC_BOTH_DIRECTORY | FIC_ID_BOTH_DIRECTORY | FIC_FULL_DIRECTORY | FIC_ID_FULL_DIRECTORY | FIC_DIRECTORY => {
                            if FileInformationClass == FIC_BOTH_DIRECTORY || FileInformationClass == FIC_ID_BOTH_DIRECTORY {
                                BothDirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_BOTH_DIR_INFORMATION);
                                Fat8dot3ToString(IrpContext, Dirent as *mut c_void, 0, &mut Fat8Dot3String);
                                NT_ASSERT((Fat8Dot3String.Length <= 12) as u8);
                                *p_Status = RtlOemToUnicodeN(
                                    &mut (*BothDirInfo).ShortName[0],
                                    12 * core::mem::size_of::<u16>() as u32,
                                    &mut ShortNameLength,
                                    Fat8Dot3String.Buffer,
                                    (Fat8Dot3String.Length) as u32,
                                );
                                NT_ASSERT(((*p_Status) != STATUS_BUFFER_OVERFLOW) as u8);
                                NT_ASSERT((ShortNameLength <= 12 * core::mem::size_of::<u16>() as u32) as u8);
                                (*BothDirInfo).ShortNameLength = (ShortNameLength as u8) as i8;

                                if (*p_Status) < 0 {
                                    // try_return(NOTHING) — inner closure: preserve Status, break loop
                                    do_return = true;
                                    return;
                                }
                            }
                            if FileInformationClass != FIC_DIRECTORY {
                                DebugTrace(0, Dbg, b"FatQueryDirectory -> Getting file full directory information\n\0".as_ptr() as *const i8, 0);
                                FullDirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_FULL_DIR_INFORMATION);
                                // _SEH2_TRY { FatGetEaLength } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                                let ea_result = crate::seh::try_except(&mut (), |_| {
                                    FatGetEaLength(IrpContext, Vcb as *mut c_void, Dirent as *mut c_void, &mut (*FullDirInfo).EaSize);
                                }, |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER);
                                if ea_result.is_err() {
                                    FatResetExceptionState(IrpContext);
                                    (*FullDirInfo).EaSize = 0;
                                }
                            }
                            // C fallthrough target: FileDirectoryInformation body
                            DirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_DIRECTORY_INFORMATION);
                            FatGetDirTimes(IrpContext as *mut IRP_CONTEXT, Dirent, DirInfo);
                            (*DirInfo).EndOfFile.QuadPart = (*Dirent).FileSize as i64;
                            if FlagOn((*Dirent).Attributes as u32, FAT_DIRENT_ATTR_DIRECTORY) == 0 {
                                (*DirInfo).AllocationSize.QuadPart =
                                    (((*Dirent).FileSize as i64 + DiskAllocSize as i64 - 1) / DiskAllocSize as i64)
                                        * DiskAllocSize as i64 ;
                            }
                            if (*Dirent).Attributes != 0 {
                                (*DirInfo).FileAttributes = (*Dirent).Attributes as u32;
                            } else {
                                (*DirInfo).FileAttributes = 0;
                                (*DirInfo).FileAttributes |= FILE_ATTRIBUTE_NORMAL;
                            }
                            (*DirInfo).FileIndex = (NextVbo) as u32;
                            (*DirInfo).FileNameLength = FileNameLength;
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Name = \"%Z\"\n\0".as_ptr() as *const i8, &Fat8Dot3String);
                        }
                        FIC_NAMES => {
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Getting file names information\n\0".as_ptr() as *const i8, 0);
                            NamesInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_NAMES_INFORMATION);
                            (*NamesInfo).FileIndex = (NextVbo) as u32;
                            (*NamesInfo).FileNameLength = FileNameLength;
                            DebugTrace(0, Dbg, b"FatQueryDirectory -> Name = \"%Z\"\n\0".as_ptr() as *const i8, &Fat8Dot3String);
                        }
                        _ => {
                            FatBugCheck(FileInformationClass, 0, 0);
                        }
                    }

                    BytesConverted = if BytesRemainingInBuffer - BaseLength >= FileNameLength {
                        FileNameLength
                    } else {
                        BytesRemainingInBuffer - BaseLength
                    };
                    // RtlCopyMemory: BytesConverted is a BYTE count. As a u16 copy it moved
                    // twice the bytes, reading past LongFileName and writing past the entry.
                    // It faulted (BUGCHECK 0x50) once a long name's overread crossed an
                    // unmapped page, first seen enumerating %windir%\winsxs.
                    core::ptr::copy_nonoverlapping(
                        (*p_LongFileName).Buffer as *const u8,
                        Buffer.add(NextEntry as usize + BaseLength as usize),
                        BytesConverted as usize,
                    );

                    *(Buffer.add(LastEntry as usize) as *mut u32) = NextEntry - LastEntry;
                    (*Irp).IoStatus.Information = (QuadAlign((*Irp).IoStatus.Information as u32) as usize)
                        .wrapping_add(BaseLength as usize).wrapping_add(BytesConverted as usize) as _;

                    if BytesConverted < FileNameLength {
                        NT_ASSERT((NextEntry == 0) as u8);
                        // try_return(Status = STATUS_BUFFER_OVERFLOW) — inner closure: flag + break loop
                        *p_Status = STATUS_BUFFER_OVERFLOW;
                        do_return = true;
                        return;
                    }
                }

                // Fill in FileId
                match FileInformationClass {
                    FIC_ID_BOTH_DIRECTORY => {
                        IdBothDirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_ID_BOTH_DIR_INFORMATION);
                        (*IdBothDirInfo).FileId.QuadPart =
                            crate::fileid::fat_generate_file_id_from_dirent_and_offset(
                                Dcb, Dirent as *const c_void, NextVbo as u32);
                    }
                    FIC_ID_FULL_DIRECTORY => {
                        IdFullDirInfo = &mut *(Buffer.add(NextEntry as usize) as *mut FILE_ID_FULL_DIR_INFORMATION);
                        (*IdFullDirInfo).FileId.QuadPart =
                            crate::fileid::fat_generate_file_id_from_dirent_and_offset(
                                Dcb, Dirent as *const c_void, NextVbo as u32);
                    }
                    _ => {}
                }
            }, |_, _pointers| crate::seh::EXCEPTION_EXECUTE_HANDLER);

            // _SEH2_EXCEPT (EXCEPTION_EXECUTE_HANDLER)
            if let Err(code) = inner_result {
                (*Irp).IoStatus.Information = 0;
                *p_UpdateCcb = 0;
                // try_return(Status = _SEH2_GetExceptionCode()) — outer closure
                *p_Status = code;
                return;
            }

            // Propagate a try_return that fired inside the inner (per-entry)
            // closure: it could only `return` from that closure, so break the
            // enclosing loop here (C `goto try_exit`).
            if do_return {
                return;
            }

            LastEntry = NextEntry;
            NextEntry += QuadAlign(BaseLength + BytesConverted) as u32;
            *p_CurrentVbo = NextVbo as VBO + core::mem::size_of::<DIRENT>() as VBO;
        }
    }, |_, abnormal| {
        // _SEH2_FINALLY. The C finally releases the Fcb, unpins the Bcb, and frees the
        // LongFileName buffer UNCONDITIONALLY — the original translation dropped all
        // three, leaking the directory Fcb's ERESOURCE on every query. Leaked shared
        // holds are invisible until any thread queues an exclusive acquire on that Fcb;
        // ERESOURCE fairness then blocks all new shared acquirers too, wedging every
        // subsequent open that traverses the directory (the arm64 taskbar/
        // CreateStartMenu hang: the second SHGetSpecialFolderLocation (CSIDL_PROGRAMS)
        // deadlocked opening ...\Start Menu\Programs).
        FatReleaseFcb(IrpContext as *mut IRP_CONTEXT, Dcb);
        FatUnpinBcb(IrpContext as *mut IRP_CONTEXT, p_Bcb);
        FatFreeStringBuffer(p_LongFileName as *mut c_void);

        if !abnormal {
            if *p_UpdateCcb != 0 {
                (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OffsetToStartSearchFrom = *p_CurrentVbo as u32;
            }
            FatCompleteRequest(IrpContext as *mut IRP_CONTEXT, Irp, *p_Status);
        }

        DebugTrace(-1, Dbg, b"FatQueryDirectory -> %08lx\n\0".as_ptr() as *const i8, *p_Status);
    });

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FatGetDirTimes(
    IrpContext: *mut IRP_CONTEXT,
    Dirent: *mut DIRENT,
    DirInfo: *mut FILE_DIRECTORY_INFORMATION,
) {
    // PAGED_CODE(); // omitted

    //
    // Start with the Last Write Time.
    //
    (*DirInfo).LastWriteTime = FatFatTimeToNtTime(
        IrpContext as *mut IRP_CONTEXT,
        core::ptr::read(core::ptr::addr_of!((*Dirent).LastWriteTime)),
        0u8,
    );

    //
    // These fields are only non-zero when in Chicago mode.
    //
    if ((*&raw const FatData).ChicagoMode() as u8) != 0 {
        //
        // Do a quick check here for Creation and LastAccess
        // times that are the same as the LastWriteTime.
        //
        if core::ptr::read_unaligned(
            core::ptr::addr_of!((*Dirent).CreationTime) as *const CLONG,
        ) == core::ptr::read_unaligned(
            core::ptr::addr_of!((*Dirent).LastWriteTime) as *const CLONG,
        ) {
            (*DirInfo).CreationTime.QuadPart =
                (*DirInfo).LastWriteTime.QuadPart
                    + ((*Dirent).CreationMSec as i64) * 10 * 1000 * 10;
        } else {
            //
            // Only do the really hard work if this field is non-zero.
            //
            if *((Dirent as *const u16).add(8)) != 0 {
                (*DirInfo).CreationTime = FatFatTimeToNtTime(
                    IrpContext as *mut IRP_CONTEXT,
                    core::ptr::read(core::ptr::addr_of!((*Dirent).CreationTime)),
                    (*Dirent).CreationMSec,
                );
            } else {
                ExLocalTimeToSystemTime(
                    core::ptr::addr_of!(FatJanOne1980) as *mut ntkrnl_sys::ddk::LARGE_INTEGER,
                    core::ptr::addr_of!((*DirInfo).CreationTime) as *mut ntkrnl_sys::ddk::LARGE_INTEGER,
                );
            }
        }

        //
        // Do a quick check for LastAccessDate.
        //
        if core::mem::transmute::<_, u16>(core::ptr::read(core::ptr::addr_of!((*Dirent).LastAccessDate)))
            == core::mem::transmute::<_, u16>(core::ptr::read(core::ptr::addr_of!((*Dirent).LastWriteTime.Date)))
        {
            let write_time = &(*Dirent).LastWriteTime.Time;

            (*DirInfo).LastAccessTime.QuadPart =
                (*DirInfo).LastWriteTime.QuadPart
                    - (UInt32x32To64(
                        (write_time.DoubleSeconds() as u32) * 2
                            + (write_time.Minute() as u32) * 60
                            + (write_time.Hour() as u32) * 60 * 60 ,
                        (1000 * 1000 * 10) as u32,
                    )) as i64;
        } else {
            //
            // Only do the really hard work if this field is non-zero.
            //
            if *((Dirent as *const u16).add(9)) != 0 {
                (*DirInfo).LastAccessTime = FatFatDateToNtTime(
                    IrpContext,
                    core::mem::transmute(core::ptr::read(core::ptr::addr_of!((*Dirent).LastAccessDate))),
                );
            } else {
                ExLocalTimeToSystemTime(
                    core::ptr::addr_of!(FatJanOne1980) as *mut ntkrnl_sys::ddk::LARGE_INTEGER,
                    core::ptr::addr_of!((*DirInfo).LastAccessTime) as *mut ntkrnl_sys::ddk::LARGE_INTEGER,
                );
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatNotifyChangeDirectory(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpSp: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Dcb: *mut DCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let mut CompletionFilter: ULONG = 0;
    let mut WatchTree: BOOLEAN = 0;
    let mut CompleteRequest: BOOLEAN = 0;

    // PAGED_CODE();

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatNotifyChangeDirectory...\n\0".as_ptr() as *const i8, 0u32);
    DebugTrace(0, Dbg, b" Wait               = %08lx\n\0".as_ptr() as *const i8, FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32);
    DebugTrace(0, Dbg, b" Irp                = %p\n\0".as_ptr() as *const i8, Irp as u32);
    DebugTrace(0, Dbg, b" ->CompletionFilter = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.NotifyDirectory.CompletionFilter);

    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    CompleteRequest = 0; // FALSE

    if FatDecodeFileObject((*IrpSp).FileObject, &mut Vcb, &mut Dcb, &mut Ccb) != UserDirectoryOpen {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatQueryDirectory -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8, 0u32);
        return STATUS_INVALID_PARAMETER;
    }

    CompletionFilter = (*IrpSp).Parameters.NotifyDirectory.CompletionFilter;
    WatchTree = BooleanFlagOn(((*IrpSp).Flags) as u32, SL_WATCH_TREE) as u8;

    if FatAcquireExclusiveFcb(IrpContext, Dcb) == 0 {
        DebugTrace(0, Dbg, b"FatNotifyChangeDirectory -> Cannot Acquire Fcb\n\0".as_ptr() as *const i8, 0u32);
        Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace(-1, Dbg, b"FatNotifyChangeDirectory -> %08lx\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_Status = core::ptr::addr_of_mut!(Status);
    let p_CompleteRequest = core::ptr::addr_of_mut!(CompleteRequest);
    crate::seh::try_finally(&mut (), |_| {
        FatVerifyFcb(IrpContext, Dcb);
        FatSetFullFileNameInFcb(IrpContext, Dcb);

        if FlagOn((*Dcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0 {
            FatRaiseStatus(IrpContext, STATUS_DELETE_PENDING);
        }

        FsRtlNotifyFullChangeDirectory(
            (*Vcb).NotifySync,
            &mut (*Vcb).DirNotifyList as *mut LIST_ENTRY,
            Ccb as *mut c_void,
            core::ptr::addr_of!((*Dcb).FullFileName) as *const _ as *mut u8 as PSTRING,
            WatchTree,
            0, // FALSE
            CompletionFilter,
            Irp,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        *p_Status = STATUS_PENDING;
        *p_CompleteRequest = 1; // TRUE
    }, |_, _abnormal| {
        DebugUnwind(b"FatNotifyChangeDirectory\0".as_ptr() as *const i8);
        FatReleaseFcb(IrpContext, Dcb);

        if *p_CompleteRequest != 0 {
            FatCompleteRequest(IrpContext, core::ptr::null_mut(), 0);
        }

        DebugTrace(-1, Dbg, b"FatNotifyChangeDirectory -> %08lx\n\0".as_ptr() as *const i8, *p_Status as u32);
    });
    return Status;
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
//     DirCtrl.c
//
// Abstract:
//
//     This module implements the File Directory Control routines for Fat called
//     by the dispatch driver.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_DIRCTRL)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_DIRCTRL)
//
// WCHAR Fat8QMdot3QM[12] = { DOS_QM, DOS_QM, DOS_QM, DOS_QM, DOS_QM, DOS_QM, DOS_QM, DOS_QM,
//                            L'.', DOS_QM, DOS_QM, DOS_QM};
//
// //
// //  Local procedure prototypes
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatQueryDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// VOID
// FatGetDirTimes(
//     PIRP_CONTEXT IrpContext,
//     PDIRENT Dirent,
//     PFILE_DIRECTORY_INFORMATION DirInfo
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatNotifyChangeDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonDirectoryControl)
// #pragma alloc_text(PAGE, FatFsdDirectoryControl)
// #pragma alloc_text(PAGE, FatNotifyChangeDirectory)
// #pragma alloc_text(PAGE, FatQueryDirectory)
// #pragma alloc_text(PAGE, FatGetDirTimes)
//
// #endif
//
//
// _Function_class_(IRP_MJ_DIRECTORY_CONTROL)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdDirectoryControl (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of directory control
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file exists
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
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdDirectoryControl\n", 0);
//
//     //
//     //  Call the common directory Control routine, with blocking allowed if
//     //  synchronous
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         Status = FatCommonDirectoryControl( IrpContext, Irp );
//
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
//     DebugTrace(-1, Dbg, "FatFsdDirectoryControl -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonDirectoryControl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for doing directory control operations called
//     by both the fsd and fsp threads
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
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//
//     PAGED_CODE();
//
//     //
//     //  Get a pointer to the current Irp stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonDirectoryControl\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp );
//     DebugTrace( 0, Dbg, "MinorFunction = %08lx\n", IrpSp->MinorFunction );
//
//     //
//     //  We know this is a directory control so we'll case on the
//     //  minor function, and call a internal worker routine to complete
//     //  the irp.
//     //
//
//     switch ( IrpSp->MinorFunction ) {
//
//     case IRP_MN_QUERY_DIRECTORY:
//
//         Status = FatQueryDirectory( IrpContext, Irp );
//         break;
//
//     case IRP_MN_NOTIFY_CHANGE_DIRECTORY:
//
//         Status = FatNotifyChangeDirectory( IrpContext, Irp );
//         break;
//
//     default:
//
//         DebugTrace(0, Dbg, "Invalid Directory Control Minor Function %08lx\n", IrpSp->MinorFunction);
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     DebugTrace(-1, Dbg, "FatCommonDirectoryControl -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatQueryDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query directory operation.  It is responsible
//     for either completing of enqueuing the input Irp.
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
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PDCB Dcb;
//     PCCB Ccb;
//     PBCB Bcb;
//
//     ULONG i;
//     PUCHAR Buffer;
//     CLONG UserBufferLength;
//
//     PUNICODE_STRING UniArgFileName;
//     WCHAR LongFileNameBuffer[ FAT_CREATE_INITIAL_NAME_BUF_SIZE];
//     UNICODE_STRING LongFileName;
//     UNICODE_STRING OrigFileName;
//     FILE_INFORMATION_CLASS FileInformationClass;
//     ULONG FileIndex;
//     ULONG MatchFlags = 0;
//     BOOLEAN RestartScan;
//     BOOLEAN ReturnSingleEntry;
//     BOOLEAN IndexSpecified;
//
//     BOOLEAN InitialQuery;
//     VBO CurrentVbo = 0;
//     BOOLEAN UpdateCcb;
//     PDIRENT Dirent;
//     UCHAR Fat8Dot3Buffer[12];
//     OEM_STRING Fat8Dot3String;
//     ULONG DiskAllocSize;
//
//     ULONG NextEntry;
//     ULONG LastEntry;
//
//     PFILE_DIRECTORY_INFORMATION DirInfo;
//     PFILE_FULL_DIR_INFORMATION FullDirInfo;
//     PFILE_BOTH_DIR_INFORMATION BothDirInfo;
//     PFILE_ID_FULL_DIR_INFORMATION IdFullDirInfo;
//     PFILE_ID_BOTH_DIR_INFORMATION IdBothDirInfo;
//     PFILE_NAMES_INFORMATION NamesInfo;
//
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Display the input values.
//     //
//     DebugTrace(+1, Dbg, "FatQueryDirectory...\n", 0);
//     DebugTrace( 0, Dbg, " Wait                   = %08lx\n", FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT));
//     DebugTrace( 0, Dbg, " Irp                    = %p\n", Irp);
//     DebugTrace( 0, Dbg, " ->Length               = %08lx\n", IrpSp->Parameters.QueryDirectory.Length);
//     DebugTrace( 0, Dbg, " ->FileName             = %wZ\n", IrpSp->Parameters.QueryDirectory.FileName);
//     DebugTrace( 0, Dbg, " ->FileInformationClass = %08lx\n", IrpSp->Parameters.QueryDirectory.FileInformationClass);
//     DebugTrace( 0, Dbg, " ->FileIndex            = %08lx\n", IrpSp->Parameters.QueryDirectory.FileIndex);
//     DebugTrace( 0, Dbg, " ->UserBuffer           = %p\n", Irp->AssociatedIrp.SystemBuffer);
//     DebugTrace( 0, Dbg, " ->RestartScan          = %08lx\n", FlagOn( IrpSp->Flags, SL_RESTART_SCAN ));
//     DebugTrace( 0, Dbg, " ->ReturnSingleEntry    = %08lx\n", FlagOn( IrpSp->Flags, SL_RETURN_SINGLE_ENTRY ));
//     DebugTrace( 0, Dbg, " ->IndexSpecified       = %08lx\n", FlagOn( IrpSp->Flags, SL_INDEX_SPECIFIED ));
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     UserBufferLength = IrpSp->Parameters.QueryDirectory.Length;
//
//     FileInformationClass = IrpSp->Parameters.QueryDirectory.FileInformationClass;
//     FileIndex = IrpSp->Parameters.QueryDirectory.FileIndex;
//
//     UniArgFileName = IrpSp->Parameters.QueryDirectory.FileName;
//
//     RestartScan       = BooleanFlagOn(IrpSp->Flags, SL_RESTART_SCAN);
//     ReturnSingleEntry = BooleanFlagOn(IrpSp->Flags, SL_RETURN_SINGLE_ENTRY);
//     IndexSpecified    = BooleanFlagOn(IrpSp->Flags, SL_INDEX_SPECIFIED);
//
//     //
//     //  Check on the type of open.  We return invalid parameter for all
//     //  but UserDirectoryOpens.  Also check that the filename is a valid
//     //  UNICODE string.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject,
//                              &Vcb,
//                              &Dcb,
//                              &Ccb) != UserDirectoryOpen ||
//         (UniArgFileName &&
//          UniArgFileName->Length % sizeof(WCHAR))) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         DebugTrace(-1, Dbg, "FatQueryDirectory -> STATUS_INVALID_PARAMETER\n", 0);
//
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Initialize the local variables.
//     //
//
//     Bcb = NULL;
//     UpdateCcb = TRUE;
//     Dirent = NULL;
//
//     Fat8Dot3String.MaximumLength = 12;
//     Fat8Dot3String.Buffer = (PCHAR)Fat8Dot3Buffer;
//
//     LongFileName.Length = 0;
//     LongFileName.MaximumLength = sizeof( LongFileNameBuffer);
//     LongFileName.Buffer = LongFileNameBuffer;
//
//     InitialQuery = (BOOLEAN)((Ccb->UnicodeQueryTemplate.Buffer == NULL) &&
//                              !FlagOn(Ccb->Flags, CCB_FLAG_MATCH_ALL));
//     Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//
//     DiskAllocSize = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     //
//     //  If this is the initial query, then grab exclusive access in
//     //  order to update the search string in the Ccb.  We may
//     //  discover that we are not the initial query once we grab the Fcb
//     //  and downgrade our status.
//     //
//     //  If restartscan is set, we may be replacing the query template,
//     //  so take the FCB exclusive to protect against multiple people
//     //  changing the CCB at once.
//     //
//
//     if (InitialQuery || RestartScan) {
//
//         if (!FatAcquireExclusiveFcb( IrpContext, Dcb )) {
//
//             DebugTrace(0, Dbg, "FatQueryDirectory -> Enqueue to Fsp\n", 0);
//             Status = FatFsdPostRequest( IrpContext, Irp );
//             DebugTrace(-1, Dbg, "FatQueryDirectory -> %08lx\n", Status);
//
//             return Status;
//         }
//
//         if (!RestartScan && (Ccb->UnicodeQueryTemplate.Buffer != NULL)) {
//
//             InitialQuery = FALSE;
//
//             FatConvertToSharedFcb( IrpContext, Dcb );
//         }
//
//     } else {
//
//         if (!FatAcquireSharedFcb( IrpContext, Dcb )) {
//
//             DebugTrace(0, Dbg, "FatQueryDirectory -> Enqueue to Fsp\n", 0);
//             Status = FatFsdPostRequest( IrpContext, Irp );
//             DebugTrace(-1, Dbg, "FatQueryDirectory -> %08lx\n", Status);
//
//             return Status;
//
//         }
//     }
//
//     _SEH2_TRY {
//
//         ULONG BaseLength;
//         ULONG BytesConverted;
//
//         //
//         // If we are in the Fsp now because we had to wait earlier,
//         // we must map the user buffer, otherwise we can use the
//         // user's buffer directly.
//         //
//
//         Buffer = FatMapUserBuffer( IrpContext, Irp );
//
//         //
//         //  Make sure the Dcb is still good.
//         //
//
//         FatVerifyFcb( IrpContext, Dcb );
//
//         //
//         //  Determine where to start the scan.  Highest priority is given
//         //  to the file index.  Lower priority is the restart flag.  If
//         //  neither of these is specified, then the Vbo offset field in the
//         //  Ccb is used.
//         //
//
//         if (IndexSpecified) {
//
//             CurrentVbo = FileIndex + sizeof( DIRENT );
//
//         } else if (RestartScan) {
//
//             CurrentVbo = 0;
//             Ccb->OffsetToStartSearchFrom = 0;
//
//         } else {
//
//             CurrentVbo = Ccb->OffsetToStartSearchFrom;
//         }
//
//         //
//         //  If this is the first try then allocate a buffer for the file
//         //  name.
//         //
//
//         if (InitialQuery ||
//             (RestartScan && UniArgFileName != NULL && UniArgFileName->Length != 0)) {
//
//             //
//             //  If we're restarting the scan, clear out the pattern in the Ccb and regenerate it,
//             //
//
//             if (RestartScan) {
//
//                 if (Ccb->UnicodeQueryTemplate.Buffer) {
//
//                     if (FlagOn(Ccb->Flags, CCB_FLAG_FREE_UNICODE)) {
//
//                         ExFreePoolWithTag(Ccb->UnicodeQueryTemplate.Buffer, TAG_FILENAME_BUFFER);
//                         ClearFlag(Ccb->Flags, CCB_FLAG_FREE_UNICODE);
//                     }
//
//                     Ccb->UnicodeQueryTemplate.Buffer = NULL;
//                     Ccb->UnicodeQueryTemplate.Length = 0;
//                     Ccb->UnicodeQueryTemplate.MaximumLength = 0;
//                 }
//
//                 if (Ccb->OemQueryTemplate.Wild.Buffer) {
//
//                     if (FlagOn(Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT)) {
//
//                         RtlFreeOemString( &Ccb->OemQueryTemplate.Wild );
//                         ClearFlag(Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT);
//                     }
//
//                     Ccb->OemQueryTemplate.Wild.Buffer = NULL;
//                     Ccb->OemQueryTemplate.Wild.Length = 0;
//                     Ccb->OemQueryTemplate.Wild.MaximumLength = 0;
//                 }
//
//                 Ccb->ContainsWildCards = FALSE;
//                 ClearFlag(Ccb->Flags, CCB_FLAG_MATCH_ALL);
//                 ClearFlag(Ccb->Flags, CCB_FLAG_FREE_UNICODE);
//                 ClearFlag(Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE);
//                 ClearFlag(Ccb->Flags, CCB_FLAG_QUERY_TEMPLATE_MIXED);
//
//             }
//
//             //
//             //  If either:
//             //
//             //  - No name was specified
//             //  - An empty name was specified
//             //  - We received a '*'
//             //  - The user specified the DOS equivolent of ????????.???
//             //
//             //  then match all names.
//             //
//
//             if ((UniArgFileName == NULL) ||
//                 (UniArgFileName->Length == 0) ||
//                 (UniArgFileName->Buffer == NULL) ||
//                 ((UniArgFileName->Length == sizeof(WCHAR)) &&
//                  (UniArgFileName->Buffer[0] == L'*')) ||
//                 ((UniArgFileName->Length == 12*sizeof(WCHAR)) &&
//                  (RtlEqualMemory( UniArgFileName->Buffer,
//                                   Fat8QMdot3QM,
//                                   12*sizeof(WCHAR) )))) {
//
//                 Ccb->ContainsWildCards = TRUE;
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_MATCH_ALL );
//
//             } else {
//
//                 BOOLEAN ExtendedName = FALSE;
//                 OEM_STRING LocalBestFit;
//
//                 //
//                 //  First and formost, see if the name has wild cards.
//                 //
//
//                 Ccb->ContainsWildCards =
//                     FsRtlDoesNameContainWildCards( UniArgFileName );
//
//                 //
//                 //  Now check to see if the name contains any extended
//                 //  characters
//                 //
//
//                 for (i=0; i < UniArgFileName->Length / sizeof(WCHAR); i++) {
//
//                     if (UniArgFileName->Buffer[i] >= 0x80) {
//
//                         ExtendedName = TRUE;
//                         break;
//                     }
//                 }
//
//                 //
//                 //  OK, now do the conversions we need.
//                 //
//
//                 if (ExtendedName) {
//
//                     Status = RtlUpcaseUnicodeString( &Ccb->UnicodeQueryTemplate,
//                                                      UniArgFileName,
//                                                      TRUE );
//
//                     if (!NT_SUCCESS(Status)) {
//
//                         try_return( Status );
//                     }
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_FREE_UNICODE );
//
//                     //
//                     //  Upcase the name and convert it to the Oem code page.
//                     //
//
//                     Status = RtlUpcaseUnicodeStringToCountedOemString( &LocalBestFit,
//                                                                        UniArgFileName,
//                                                                        TRUE );
//
//                     //
//                     //  If this conversion failed for any reason other than
//                     //  an unmappable character fail the request.
//                     //
//
//                     if (!NT_SUCCESS(Status)) {
//
//                         if (Status == STATUS_UNMAPPABLE_CHARACTER) {
//
//                             SetFlag( Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE );
//
//                         } else {
//
//                             try_return( Status );
//                         }
//
//                     } else {
//
//                         SetFlag( Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT );
//                     }
//
//                 } else {
//
//                     PVOID Buffers;
//
//                     //
//                     //  This case is optimized because I know I only have to
//                     //  worry about a-z.
//                     //
//
//                     Buffers = FsRtlAllocatePoolWithTag( PagedPool,
//                                                         UniArgFileName->Length +
//                                                         UniArgFileName->Length / sizeof(WCHAR),
//                                                         TAG_FILENAME_BUFFER );
//
//                     Ccb->UnicodeQueryTemplate.Buffer = Buffers;
//                     Ccb->UnicodeQueryTemplate.Length = UniArgFileName->Length;
//                     Ccb->UnicodeQueryTemplate.MaximumLength = UniArgFileName->Length;
//
//                     LocalBestFit.Buffer = (PCHAR)Buffers + UniArgFileName->Length;
//                     LocalBestFit.Length = UniArgFileName->Length / sizeof(WCHAR);
//                     LocalBestFit.MaximumLength = LocalBestFit.Length;
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_FREE_UNICODE );
//
//                     for (i=0; i < UniArgFileName->Length / sizeof(WCHAR); i++) {
//
//                         WCHAR c = UniArgFileName->Buffer[i];
//
//                         LocalBestFit.Buffer[i] = (UCHAR)
//                         (Ccb->UnicodeQueryTemplate.Buffer[i] =
//                              (c < 'a' ? c : c <= 'z' ? c - ('a' - 'A') : c));
//                     }
//                 }
//
//                 //
//                 //  At this point we now have the upcased unicode name,
//                 //  and the two Oem names if they could be represented in
//                 //  this code page.
//                 //
//                 //  Now determine if the Oem names are legal for what we
//                 //  going to try and do.  Mark them as not usable is they
//                 //  are not legal.  Note that we can optimize extended names
//                 //  since they are actually both the same string.
//                 //
//
//                 if (!FlagOn( Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE ) &&
//                     !FatIsNameShortOemValid( IrpContext,
//                                              LocalBestFit,
//                                              Ccb->ContainsWildCards,
//                                              FALSE,
//                                              FALSE )) {
//
//                     if (ExtendedName) {
//
//                         RtlFreeOemString( &LocalBestFit );
//                         ClearFlag( Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT );
//                     }
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE );
//                 }
//
//                 //
//                 //  OK, now both locals oem strings correctly reflect their
//                 //  usability.  Now we want to load up the Ccb structure.
//                 //
//                 //  Now we will branch on two paths of wheather the name
//                 //  is wild or not.
//                 //
//
//                 if (!FlagOn( Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE )) {
//
//                     if (Ccb->ContainsWildCards) {
//
//                         Ccb->OemQueryTemplate.Wild = LocalBestFit;
//
//                     } else {
//
//                         FatStringTo8dot3( IrpContext,
//                                           LocalBestFit,
//                                           &Ccb->OemQueryTemplate.Constant );
//
//                         if (FlagOn(Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT)) {
//
//                             RtlFreeOemString( &LocalBestFit );
//                             ClearFlag( Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT );
//                         }
//                     }
//                 }
//             }
//
//             //
//             //  We convert to shared access.
//             //
//
//             FatConvertToSharedFcb( IrpContext, Dcb );
//         }
//
//         LastEntry = 0;
//         NextEntry = 0;
//
//         switch (FileInformationClass) {
//
//         case FileDirectoryInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_DIRECTORY_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         case FileFullDirectoryInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_FULL_DIR_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         case FileIdFullDirectoryInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_ID_FULL_DIR_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         case FileNamesInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_NAMES_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         case FileBothDirectoryInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_BOTH_DIR_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         case FileIdBothDirectoryInformation:
//
//             BaseLength = FIELD_OFFSET( FILE_ID_BOTH_DIR_INFORMATION,
//                                        FileName[0] );
//             break;
//
//         default:
//
//             try_return( Status = STATUS_INVALID_INFO_CLASS );
//         }
//
//         //
//         //  At this point we are about to enter our query loop.  We have
//         //  determined the index into the directory file to begin the
//         //  search.  LastEntry and NextEntry are used to index into the user
//         //  buffer.  LastEntry is the last entry we've added, NextEntry is
//         //  current one we're working on.  If NextEntry is non-zero, then
//         //  at least one entry was added.
//         //
//
//         while ( TRUE ) {
//
//             VBO NextVbo;
//             ULONG FileNameLength;
//             ULONG BytesRemainingInBuffer;
//             BOOLEAN FileNameDos;
//
//             DebugTrace(0, Dbg, "FatQueryDirectory -> Top of loop\n", 0);
//
//             //
//             //  If the user had requested only a single match and we have
//             //  returned that, then we stop at this point.
//             //
//
//             if (ReturnSingleEntry && NextEntry != 0) {
//
//                 try_return( Status );
//             }
//
//
//             //
//             //  We call FatLocateDirent to lock down the next matching dirent.
//             //
//
//             FatLocateDirent( IrpContext,
//                              Dcb,
//                              Ccb,
//                              CurrentVbo,
//                              &MatchFlags,
//                              &Dirent,
//                              &Bcb,
//                              &NextVbo,
//                              &FileNameDos,
//                              &LongFileName,
//                              &OrigFileName );
//
//             //
//             //  If we didn't receive a dirent, then we are at the end of the
//             //  directory.  If we have returned any files, we exit with
//             //  success, otherwise we return STATUS_NO_MORE_FILES.
//             //
//
//             if (!Dirent) {
//
//                 DebugTrace(0, Dbg, "FatQueryDirectory -> No dirent\n", 0);
//
//                 if (NextEntry == 0) {
//
//                     UpdateCcb = FALSE;
//
//                     if (InitialQuery) {
//
//                         Status = STATUS_NO_SUCH_FILE;
//
//                     } else {
//
//                         Status = STATUS_NO_MORE_FILES;
//                     }
//                 }
//
//                 try_return( Status );
//             }
//
//
//             //
//             //  Protect access to the user buffer with an exception handler.
//             //  Since (at our request) IO doesn't buffer these requests, we have
//             //  to guard against a user messing with the page protection and other
//             //  such trickery.
//             //
//
//             _SEH2_TRY {
//
//                 Fat8dot3ToString( IrpContext, Dirent, TRUE, &Fat8Dot3String );
//
//
//                 if (LongFileName.Length == 0) {
//
//                     //
//                     //  Now we have an entry to return to our caller.  We'll convert
//                     //  the name from the form in the dirent to a <name>.<ext> form.
//                     //  We'll case on the type of information requested and fill up
//                     //  the user buffer if everything fits.
//                     //
//
//                     //
//                     //  Determine the UNICODE length of the file name.
//                     //
//
//                     FileNameLength = RtlOemStringToCountedUnicodeSize(&Fat8Dot3String);
//
//                     //
//                     //  Here are the rules concerning filling up the buffer:
//                     //
//                     //  1.  The Io system garentees that there will always be
//                     //      enough room for at least one base record.
//                     //
//                     //  2.  If the full first record (including file name) cannot
//                     //      fit, as much of the name as possible is copied and
//                     //      STATUS_BUFFER_OVERFLOW is returned.
//                     //
//                     //  3.  If a subsequent record cannot completely fit into the
//                     //      buffer, none of it (as in 0 bytes) is copied, and
//                     //      STATUS_SUCCESS is returned.  A subsequent query will
//                     //      pick up with this record.
//                     //
//
//                     BytesRemainingInBuffer = UserBufferLength - NextEntry;
//
//                     if ( (NextEntry != 0) &&
//                          ( (BaseLength + FileNameLength > BytesRemainingInBuffer) ||
//                            (UserBufferLength < NextEntry) ) ) {
//
//                         DebugTrace(0, Dbg, "Next entry won't fit\n", 0);
//
//                         try_return( Status = STATUS_SUCCESS );
//                     }
//
//                     NT_ASSERT( BytesRemainingInBuffer >= BaseLength );
//
//                     //
//                     //  Zero the base part of the structure.
//                     //
//
//                     RtlZeroMemory( &Buffer[NextEntry], BaseLength );
//
//                     switch ( FileInformationClass ) {
//
//                     //
//                     //  Now fill the base parts of the strucure that are applicable.
//                     //
//
//                     case FileBothDirectoryInformation:
//                     case FileFullDirectoryInformation:
//                     case FileIdBothDirectoryInformation:
//                     case FileIdFullDirectoryInformation:
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Getting file full directory information\n", 0);
//
//                         //
//                         //  Get the Ea file length.
//                         //
//
//                         FullDirInfo = (PFILE_FULL_DIR_INFORMATION)&Buffer[NextEntry];
//
//                         //
//                         //  If the EAs are corrupt, ignore the error.  We don't want
//                         //  to abort the directory query.
//                         //
//
//                         _SEH2_TRY {
//
//                             FatGetEaLength( IrpContext,
//                                             Vcb,
//                                             Dirent,
//                                             &FullDirInfo->EaSize );
//
//                         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//
//                               FatResetExceptionState( IrpContext );
//                               FullDirInfo->EaSize = 0;
//                         } _SEH2_END;
//
//                     case FileDirectoryInformation:
//
//                         DirInfo = (PFILE_DIRECTORY_INFORMATION)&Buffer[NextEntry];
//
//                         FatGetDirTimes( IrpContext, Dirent, DirInfo );
//
//                         DirInfo->EndOfFile.QuadPart = Dirent->FileSize;
//
//                         if (!FlagOn( Dirent->Attributes, FAT_DIRENT_ATTR_DIRECTORY )) {
//
//
//                             DirInfo->AllocationSize.QuadPart =
//                                (((Dirent->FileSize + DiskAllocSize - 1) / DiskAllocSize) *
//                                 DiskAllocSize );
//                         }
//
//                         if (Dirent->Attributes != 0) {
//                             DirInfo->FileAttributes = Dirent->Attributes;
//
//
//                         } else {
//
//                             DirInfo->FileAttributes = 0;
//
//                                 DirInfo->FileAttributes |= FILE_ATTRIBUTE_NORMAL;
//                         }
//
//                         DirInfo->FileIndex = NextVbo;
//
//                         DirInfo->FileNameLength = FileNameLength;
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Name = \"%Z\"\n", &Fat8Dot3String);
//
//                         break;
//
//                     case FileNamesInformation:
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Getting file names information\n", 0);
//
//                         NamesInfo = (PFILE_NAMES_INFORMATION)&Buffer[NextEntry];
//
//                         NamesInfo->FileIndex = NextVbo;
//
//                         NamesInfo->FileNameLength = FileNameLength;
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Name = \"%Z\"\n", &Fat8Dot3String );
//
//                         break;
//
//                     default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                         FatBugCheck( FileInformationClass, 0, 0 );
//                     }
//
//                     BytesConverted = 0;
//
//                     Status = RtlOemToUnicodeN( (PWCH)&Buffer[NextEntry + BaseLength],
//                                                BytesRemainingInBuffer - BaseLength,
//                                                &BytesConverted,
//                                                Fat8Dot3String.Buffer,
//                                                Fat8Dot3String.Length );
//
//                     //
//                     //  Check for the case that a single entry doesn't fit.
//                     //  This should only get this far on the first entry
//                     //
//
//                     if (BytesConverted < FileNameLength) {
//
//                         NT_ASSERT( NextEntry == 0 );
//                         Status = STATUS_BUFFER_OVERFLOW;
//                     }
//
//                     //
//                     //  Set up the previous next entry offset
//                     //
//
//                     *((PULONG)(&Buffer[LastEntry])) = NextEntry - LastEntry;
//
//                     //
//                     //  And indicate how much of the user buffer we have currently
//                     //  used up.  We must compute this value before we long align
//                     //  ourselves for the next entry
//                     //
//
//                     Irp->IoStatus.Information = QuadAlign( Irp->IoStatus.Information ) +
//                                                 BaseLength + BytesConverted;
//
//                     //
//                     //  If something happened with the conversion, bail here.
//                     //
//
//                     if ( !NT_SUCCESS( Status ) ) {
//
//                         try_return( NOTHING );
//                     }
//
//                 } else {
//
//                     ULONG ShortNameLength;
//
//                     FileNameLength = LongFileName.Length;
//
//                     //
//                     //  Here are the rules concerning filling up the buffer:
//                     //
//                     //  1.  The Io system garentees that there will always be
//                     //      enough room for at least one base record.
//                     //
//                     //  2.  If the full first record (including file name) cannot
//                     //      fit, as much of the name as possible is copied and
//                     //      STATUS_BUFFER_OVERFLOW is returned.
//                     //
//                     //  3.  If a subsequent record cannot completely fit into the
//                     //      buffer, none of it (as in 0 bytes) is copied, and
//                     //      STATUS_SUCCESS is returned.  A subsequent query will
//                     //      pick up with this record.
//                     //
//
//                     BytesRemainingInBuffer = UserBufferLength - NextEntry;
//
//                     if ( (NextEntry != 0) &&
//                          ( (BaseLength + FileNameLength > BytesRemainingInBuffer) ||
//                            (UserBufferLength < NextEntry) ) ) {
//
//                         DebugTrace(0, Dbg, "Next entry won't fit\n", 0);
//
//                         try_return( Status = STATUS_SUCCESS );
//                     }
//
//                     NT_ASSERT( BytesRemainingInBuffer >= BaseLength );
//
//                     //
//                     //  Zero the base part of the structure.
//                     //
//
//                     RtlZeroMemory( &Buffer[NextEntry], BaseLength );
//
//                     switch ( FileInformationClass ) {
//
//                     //
//                     //  Now fill the base parts of the strucure that are applicable.
//                     //
//
//                     case FileBothDirectoryInformation:
//                     case FileIdBothDirectoryInformation:
//
//                         BothDirInfo = (PFILE_BOTH_DIR_INFORMATION)&Buffer[NextEntry];
//
//                         //
//                         //  Now we have an entry to return to our caller.  We'll convert
//                         //  the name from the form in the dirent to a <name>.<ext> form.
//                         //  We'll case on the type of information requested and fill up
//                         //  the user buffer if everything fits.
//                         //
//
//                         Fat8dot3ToString( IrpContext, Dirent, FALSE, &Fat8Dot3String );
//
//                         NT_ASSERT( Fat8Dot3String.Length <= 12 );
//
//                         Status = RtlOemToUnicodeN( &BothDirInfo->ShortName[0],
//                                                    12*sizeof(WCHAR),
//                                                    &ShortNameLength,
//                                                    Fat8Dot3String.Buffer,
//                                                    Fat8Dot3String.Length );
//
//                         NT_ASSERT( Status != STATUS_BUFFER_OVERFLOW );
//                         NT_ASSERT( ShortNameLength <= 12*sizeof(WCHAR) );
//
//                         //
//                         //  Copy the length into the dirinfo structure.  Note
//                         //  that the LHS below is a USHORT, so it can not
//                         //  be specificed as the OUT parameter above.
//                         //
//
//                         BothDirInfo->ShortNameLength = (UCHAR)ShortNameLength;
//
//                         //
//                         //  If something happened with the conversion, bail here.
//                         //
//
//                         if ( !NT_SUCCESS( Status ) ) {
//
//                             try_return( NOTHING );
//                         }
//
//                     case FileFullDirectoryInformation:
//                     case FileIdFullDirectoryInformation:
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Getting file full directory information\n", 0);
//
//                         //
//                         //  Get the Ea file length.
//                         //
//
//                         FullDirInfo = (PFILE_FULL_DIR_INFORMATION)&Buffer[NextEntry];
//
//                         //
//                         //  If the EAs are corrupt, ignore the error.  We don't want
//                         //  to abort the directory query.
//                         //
//
//                         _SEH2_TRY {
//
//                             FatGetEaLength( IrpContext,
//                                             Vcb,
//                                             Dirent,
//                                             &FullDirInfo->EaSize );
//
//                         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//
//                               FatResetExceptionState( IrpContext );
//                               FullDirInfo->EaSize = 0;
//                         } _SEH2_END;
//
//                     case FileDirectoryInformation:
//
//                         DirInfo = (PFILE_DIRECTORY_INFORMATION)&Buffer[NextEntry];
//
//                         FatGetDirTimes( IrpContext, Dirent, DirInfo );
//
//                         DirInfo->EndOfFile.QuadPart = Dirent->FileSize;
//
//                         if (!FlagOn( Dirent->Attributes, FAT_DIRENT_ATTR_DIRECTORY )) {
//
//
//                             DirInfo->AllocationSize.QuadPart = (
//                                                             (( Dirent->FileSize
//                                                                + DiskAllocSize - 1 )
//                                                              / DiskAllocSize )
//                                                             * DiskAllocSize );
//                         }
//
//                         if (Dirent->Attributes != 0) {
//                             DirInfo->FileAttributes = Dirent->Attributes;
//
//
//                         } else {
//
//                             DirInfo->FileAttributes = 0;
//
//
//                                 DirInfo->FileAttributes |= FILE_ATTRIBUTE_NORMAL;
//                             }
//
//
//                         DirInfo->FileIndex = NextVbo;
//
//                         DirInfo->FileNameLength = FileNameLength;
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Name = \"%Z\"\n", &Fat8Dot3String);
//
//                         break;
//
//                     case FileNamesInformation:
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Getting file names information\n", 0);
//
//                         NamesInfo = (PFILE_NAMES_INFORMATION)&Buffer[NextEntry];
//
//                         NamesInfo->FileIndex = NextVbo;
//
//                         NamesInfo->FileNameLength = FileNameLength;
//
//                         DebugTrace(0, Dbg, "FatQueryDirectory -> Name = \"%Z\"\n", &Fat8Dot3String );
//
//                         break;
//
//                     default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                         FatBugCheck( FileInformationClass, 0, 0 );
//                     }
//
//                     BytesConverted = BytesRemainingInBuffer - BaseLength >= FileNameLength ?
//                                      FileNameLength :
//                                      BytesRemainingInBuffer - BaseLength;
//
//                     RtlCopyMemory( &Buffer[NextEntry + BaseLength],
//                                    &LongFileName.Buffer[0],
//                                    BytesConverted );
//
//                     //
//                     //  Set up the previous next entry offset
//                     //
//
//                     *((PULONG)(&Buffer[LastEntry])) = NextEntry - LastEntry;
//
//                     //
//                     //  And indicate how much of the user buffer we have currently
//                     //  used up.  We must compute this value before we long align
//                     //  ourselves for the next entry
//                     //
//
//                     Irp->IoStatus.Information = QuadAlign( Irp->IoStatus.Information ) +
//                                                 BaseLength + BytesConverted;
//
//                     //
//                     //  Check for the case that a single entry doesn't fit.
//                     //  This should only get this far on the first entry.
//                     //
//
//                     if (BytesConverted < FileNameLength) {
//
//                         NT_ASSERT( NextEntry == 0 );
//
//                         try_return( Status = STATUS_BUFFER_OVERFLOW );
//                     }
//                 }
//
//                 //
//                 //  Finish up by filling in the FileId
//                 //
//
//                 switch ( FileInformationClass ) {
//
//                 case FileIdBothDirectoryInformation:
//
//                     IdBothDirInfo = (PFILE_ID_BOTH_DIR_INFORMATION)&Buffer[NextEntry];
//                     IdBothDirInfo->FileId.QuadPart = FatGenerateFileIdFromDirentAndOffset( Dcb, Dirent, NextVbo );
//                     break;
//
//                 case FileIdFullDirectoryInformation:
//
//                     IdFullDirInfo = (PFILE_ID_FULL_DIR_INFORMATION)&Buffer[NextEntry];
//                     IdFullDirInfo->FileId.QuadPart = FatGenerateFileIdFromDirentAndOffset( Dcb, Dirent, NextVbo );
//                     break;
//
//                 default:
//                     break;
//                 }
//
//             }  _SEH2_EXCEPT (EXCEPTION_EXECUTE_HANDLER) {
//
//                   //
//                   //  We had a problem filling in the user's buffer, so stop and
//                   //  fail this request.  This is the only reason any exception
//                   //  would have occured at this level.
//                   //
//
//                   Irp->IoStatus.Information = 0;
//                   UpdateCcb = FALSE;
//                   try_return( Status = _SEH2_GetExceptionCode());
//             } _SEH2_END;
//
//             //
//             //  Set ourselves up for the next iteration
//             //
//
//             LastEntry = NextEntry;
//             NextEntry += (ULONG)QuadAlign(BaseLength + BytesConverted);
//
//             CurrentVbo = NextVbo + sizeof( DIRENT );
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatQueryDirectory );
//
//         FatReleaseFcb( IrpContext, Dcb );
//
//         //
//         //  Unpin data in cache if still held.
//         //
//
//         FatUnpinBcb( IrpContext, Bcb );
//
//         //
//         //  Free any dynamically allocated string buffer
//         //
//
//         FatFreeStringBuffer( &LongFileName);
//
//         //
//         //  Perform any cleanup.  If this is the first query, then store
//         //  the filename in the Ccb if successful.  Also update the
//         //  VBO index for the next search.  This is done by transferring
//         //  from shared access to exclusive access and copying the
//         //  data from the local copies.
//         //
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             if (UpdateCcb) {
//
//                 //
//                 //  Store the most recent VBO to use as a starting point for
//                 //  the next search.
//                 //
//
//                 Ccb->OffsetToStartSearchFrom = CurrentVbo;
//             }
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatQueryDirectory -> %08lx\n", Status);
//
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// VOID
// FatGetDirTimes(
//     PIRP_CONTEXT IrpContext,
//     PDIRENT Dirent,
//     PFILE_DIRECTORY_INFORMATION DirInfo
//     )
//
// /*++
//
// Routine Description:
//
//     This routine pulls the date/time information from a dirent and fills
//     in the DirInfo structure.
//
// Arguments:
//
//     Dirent - Supplies the dirent
//     DirInfo - Supplies the target structure
//
// Return Value:
//
//     VOID
//
// --*/
//
//
// {
//     PAGED_CODE();
//
//     //
//     //  Start with the Last Write Time.
//     //
//
//     DirInfo->LastWriteTime =
//         FatFatTimeToNtTime( IrpContext,
//                             Dirent->LastWriteTime,
//                             0 );
//
//     //
//     //  These fields are only non-zero when in Chicago mode.
//     //
//
//     if (FatData.ChicagoMode) {
//
//         //
//         //  Do a quick check here for Creation and LastAccess
//         //  times that are the same as the LastWriteTime.
//         //
//
//         if (*((UNALIGNED LONG *)&Dirent->CreationTime) ==
//             *((UNALIGNED LONG *)&Dirent->LastWriteTime)) {
//
//             DirInfo->CreationTime.QuadPart =
//
//                 DirInfo->LastWriteTime.QuadPart +
//                 Dirent->CreationMSec * 10 * 1000 * 10;
//
//         } else {
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[8] != 0) {
//
//                 DirInfo->CreationTime =
//                     FatFatTimeToNtTime( IrpContext,
//                                         Dirent->CreationTime,
//                                         Dirent->CreationMSec );
//
//             } else {
//
//                 ExLocalTimeToSystemTime( &FatJanOne1980,
//                                          &DirInfo->CreationTime );
//             }
//         }
//
//         //
//         //  Do a quick check for LastAccessDate.
//         //
//
//         if (*((PUSHORT)&Dirent->LastAccessDate) ==
//             *((PUSHORT)&Dirent->LastWriteTime.Date)) {
//
//             PFAT_TIME WriteTime;
//
//             WriteTime = &Dirent->LastWriteTime.Time;
//
//             DirInfo->LastAccessTime.QuadPart =
//                 DirInfo->LastWriteTime.QuadPart -
//                 UInt32x32To64(((WriteTime->DoubleSeconds * 2) +
//                                (WriteTime->Minute * 60) +
//                                (WriteTime->Hour * 60 * 60)),
//                               1000 * 1000 * 10);
//
//         } else {
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[9] != 0) {
//
//                 DirInfo->LastAccessTime =
//                     FatFatDateToNtTime( IrpContext,
//                                         Dirent->LastAccessDate );
//
//             } else {
//
//                 ExLocalTimeToSystemTime( &FatJanOne1980,
//                                          &DirInfo->LastAccessTime );
//             }
//         }
//     }
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatNotifyChangeDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the notify change directory operation.  It is
//     responsible for either completing of enqueuing the input Irp.
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
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//     PVCB Vcb;
//     PDCB Dcb;
//     PCCB Ccb;
//     ULONG CompletionFilter;
//     BOOLEAN WatchTree;
//
//     BOOLEAN CompleteRequest;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatNotifyChangeDirectory...\n", 0);
//     DebugTrace( 0, Dbg, " Wait               = %08lx\n", FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT));
//     DebugTrace( 0, Dbg, " Irp                = %p\n", Irp);
//     DebugTrace( 0, Dbg, " ->CompletionFilter = %08lx\n", IrpSp->Parameters.NotifyDirectory.CompletionFilter);
//
//     //
//     //  Always set the wait flag in the Irp context for the original request.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Assume we don't complete request.
//     //
//
//     CompleteRequest = FALSE;
//
//     //
//     //  Check on the type of open.  We return invalid parameter for all
//     //  but UserDirectoryOpens.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject,
//                              &Vcb,
//                              &Dcb,
//                              &Ccb ) != UserDirectoryOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         DebugTrace(-1, Dbg, "FatQueryDirectory -> STATUS_INVALID_PARAMETER\n", 0);
//
//         return STATUS_INVALID_PARAMETER;
//
//     }
//
//     //
//     //  Reference our input parameter to make things easier
//     //
//
//     CompletionFilter = IrpSp->Parameters.NotifyDirectory.CompletionFilter;
//     WatchTree = BooleanFlagOn( IrpSp->Flags, SL_WATCH_TREE );
//
//     //
//     //  Try to acquire exclusive access to the Dcb and enqueue the Irp to the
//     //  Fsp if we didn't get access
//     //
//
//     if (!FatAcquireExclusiveFcb( IrpContext, Dcb )) {
//
//         DebugTrace(0, Dbg, "FatNotifyChangeDirectory -> Cannot Acquire Fcb\n", 0);
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatNotifyChangeDirectory -> %08lx\n", Status);
//         return Status;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure the Fcb is still good
//         //
//
//         FatVerifyFcb( IrpContext, Dcb );
//
//         //
//         //  We need the full name.
//         //
//
//         FatSetFullFileNameInFcb( IrpContext, Dcb );
//
//         //
//         //  If the file is marked as DELETE_PENDING then complete this
//         //  request immediately.
//         //
//
//         if (FlagOn( Dcb->FcbState, FCB_STATE_DELETE_ON_CLOSE )) {
//
//             FatRaiseStatus( IrpContext, STATUS_DELETE_PENDING );
//         }
//
//         //
//         //  Call the Fsrtl package to process the request.
//         //
//
//         FsRtlNotifyFullChangeDirectory( Vcb->NotifySync,
//                                         &Vcb->DirNotifyList,
//                                         Ccb,
//                                         (PSTRING)&Dcb->FullFileName,
//                                         WatchTree,
//                                         FALSE,
//                                         CompletionFilter,
//                                         Irp,
//                                         NULL,
//                                         NULL );
//
//         Status = STATUS_PENDING;
//
//         CompleteRequest = TRUE;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatNotifyChangeDirectory );
//
//         FatReleaseFcb( IrpContext, Dcb );
//
//         //
//         //  If the dir notify package is holding the Irp, we discard the
//         //  the IrpContext.
//         //
//
//         if (CompleteRequest) {
//
//             FatCompleteRequest( IrpContext, FatNull, 0 );
//         }
//
//         DebugTrace(-1, Dbg, "FatNotifyChangeDirectory -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
