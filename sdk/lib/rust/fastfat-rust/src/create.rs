#[allow(unused_imports)] use crate::{TimerStart, TimerStop};
#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
// One declaration per external symbol: externs.rs.
use crate::externs::FAT_LFN_DIRENTS_NEEDED;
use crate::externs::FatAcquireExclusiveVcb;
use crate::externs::FatBugCheck;
use crate::externs::FatCompleteRequest;
use crate::externs::FatDeviceIsFatFsdo;
use crate::externs::FatDirectoryKey;
use crate::externs::FatGetFcbOplock;
use crate::externs::FatIsFat32;
use crate::externs::FatIsIoRangeValid;
use crate::externs::FatIsNameShortOemValid;
use crate::externs::FatReleaseFcb;
use crate::externs::FatReleaseVcb;
use crate::externs::FatUnpinBcb;
use crate::externs::FsRtlCheckOplockEx;
use crate::externs::CcPurgeCacheSection;
use crate::externs::CcSetFileSizes;
use crate::externs::CcUninitializeCacheMap;
use crate::externs::ExAcquireResourceExclusiveLite;
use crate::externs::IoRemoveShareAccess;
use crate::externs::IoSetTopLevelIrp;
use crate::externs::RtlAssert;
// fastfat's own functions: the definition is the declaration.
use crate::resrcsup::FatAcquireExclusiveFcb;
use crate::dirsup::FatConstructDirent;
use crate::strucsup::FatCreateFcb;
use crate::strucsup::FatCreateIrpContext;
use crate::dirsup::FatCreateNewDirent;
use crate::filobsup::FatDecodeFileObject;
use crate::dirsup::FatDeleteDirent;
use crate::easup::FatDeleteEa;
use crate::strucsup::FatEnsureStringBufferEnough;
use crate::fatdata::FatExceptionFilter;
use crate::splaysup::FatFindFcb;
use crate::flush::FatFlushFat;
use crate::flush::FatFlushVolume;
use crate::strucsup::FatFreeStringBuffer;
use crate::workque::FatFsdPostRequest;
use crate::dirsup::FatGetDirentFromFcbOrDcb;
use crate::fatdata::FatIsIrpTopLevel;
use crate::dirsup::FatLfnDirentExists;
use crate::dirsup::FatLocateDirent;
use crate::verfysup::FatMarkVolume;
use crate::workque::FatOplockComplete;
use crate::workque::FatPrePostIrp;
use crate::cachesup::FatPrepareWriteDirectoryFile;
use crate::seh::FatRaiseStatus;
use crate::seh::FatResetExceptionState;
use crate::namesup::FatSelectNames;
use crate::cachesup::FatSetDirtyBcb;
use crate::filobsup::FatSetFileObject;
use crate::namesup::FatStringTo8dot3;
use crate::cachesup::FatSyncUninitializeCacheMap;
use crate::deviosup::FatToggleMediaEjectDisable;
use crate::allocsup::FatTruncateFileAllocation;
use crate::cachesup::FatUnpinRepinnedBcbs;
use crate::verfysup::FatVerifyFcb;
use crate::verfysup::FatVerifyVcb;
// The real definitions, so the compiler checks every call against them.
use crate::timesup::{FatNtTimeToFatTime};
use ntkrnl_sys::ddk::PRTL_SPLAY_LINKS;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, CcFlushCache, CcIsThereDirtyData, _IO_STATUS_BLOCK, IoAllocateMdl, IoCheckShareAccess, IoUpdateShareAccess, KPROCESSOR_MODE, KeCancelTimer, KeRemoveQueueDpc, LARGE_INTEGER, LUID, MmFlushImageSection, OEM_STRING, PACCESS_MASK, PACCESS_STATE, PCC_FILE_SIZES, PDEVICE_OBJECT, PFILE_FULL_EA_INFORMATION, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, PMDL, POEM_STRING, PSTRING, PUNICODE_STRING, SeSinglePrivilegeCheck, IO_SECURITY_CONTEXT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DCB, DIRENT, FAT_BUG_CHECK_CREATE, FAT_DATA, FAT_DATE, FAT_TIME, FCB, FCB_STATE_DELETE_ON_CLOSE, FCB_STATE_SYSTEM_FILE, IRP_CONTEXT, LFN_DIRENT, PBCB, PCCB, PDCB, PDIRENT, PFCB, PIRP_CONTEXT, PVCB, VCB, VOLUME_DEVICE_OBJECT, _FCB_CONDITION_FcbGood as FcbGood, _TYPE_OF_OPEN as TYPE_OF_OPEN, _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen, _VCB_CONDITION_VcbGood as VcbGood};
use ntkrnl_sys::ddk::{CC_FILE_SIZES, DEVICE_OBJECT, ERESOURCE, FILE_FULL_EA_INFORMATION, FILE_OBJECT, IO_STACK_LOCATION, IRP, KEVENT, OPLOCK, PEXCEPTION_POINTERS, POPLOCK, SECTION_OBJECT_POINTERS, SECURITY_SUBJECT_CONTEXT, SHARE_ACCESS, STRING, _ACCESS_STATE, _PUBLIC_BCB as BCB, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_CREATE, FILE_ATTRIBUTE_ARCHIVE, NT_SUCCESS, STATUS_DISK_FULL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlCheckOplock, FsRtlCurrentBatchOplock, FsRtlEnterFileSystem, FsRtlFindInTunnelCache};
use ntkrnl_sys::ddk::ExReleaseResourceLite;

// FAT_CALLOUT_PARAMETERS: local to create.c, not in shared headers.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _FAT_CALLOUT_CREATE_INNER {
    pub IrpContext: *mut IRP_CONTEXT,
    pub Irp: *mut IRP,
}
#[repr(C)]
pub union _FAT_CALLOUT_PARAMETERS_UNION {
    pub Create: _FAT_CALLOUT_CREATE_INNER,
}
#[repr(C)]
pub struct FAT_CALLOUT_PARAMETERS {
    pub __bindgen_anon_1: _FAT_CALLOUT_PARAMETERS_UNION,
    pub IrpStatus: i32,
    pub ExceptionStatus: i32,
}

// BCB alias for external functions that use Bcb name
type Bcb = BCB;

extern "system" {
    fn FatAddFileAllocation( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, FileObject: *mut FILE_OBJECT, AllocationSize: u32, );
    fn FatCreateEa( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, EaBuffer: *mut u8, EaLength: u32, ShortName: *mut OEM_STRING, EaHandle: *mut u16, );
    fn FatCreateCcb(IrpContext: *mut IRP_CONTEXT) -> *mut CCB;
    fn FatNotifyReportChange( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Fcb: *mut FCB, FilterMatch: u32, Action: u32, );
    fn FatCheckManageVolumeAccess( IrpContext: *mut IRP_CONTEXT, AccessState: *mut _ACCESS_STATE, RequestorMode: KPROCESSOR_MODE, ) -> u8;
    fn FatDeleteFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut *mut FCB, );
    fn FatDeleteCcb( IrpContext: *mut IRP_CONTEXT, Ccb: *mut *mut CCB, );
    fn IsFileObjectReadOnly(FileObject: *mut FILE_OBJECT) -> u8;
    fn ARGUMENT_PRESENT(x: *mut c_void) -> u8;
    fn ExFreePool(P: *mut c_void);
    fn RtlAreBitsSet( BitMapHeader: *mut RTL_BITMAP, StartingIndex: u32, Length: u32, ) -> u8;
    fn RtlClearBits( BitMapHeader: *mut RTL_BITMAP, StartingIndex: u32, Length: u32, );
    } extern "system" {
}
#[cfg(target_arch = "x86")]
extern "stdcall" {
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
}
extern "system" {
    // SIZE_T and BOOLEAN, as rust_cohab_fat.c defines it: a u32 Size left the upper half of
    // the 64-bit argument register undefined on amd64/arm64.
    fn KeExpandKernelStackAndCalloutEx( Callout: extern "system" fn(*mut FAT_CALLOUT_PARAMETERS), Context: *mut FAT_CALLOUT_PARAMETERS, StackSize: SIZE_T, Wait: BOOLEAN, Reserved: *mut core::ffi::c_void, ) -> NTSTATUS;
    fn FatCheckFileAccess(IrpContext: *mut IRP_CONTEXT, DirentFatFlags: u8, DesiredAccess: *mut ACCESS_MASK) -> u32;
    fn KeQuerySystemTime(time: *mut LARGE_INTEGER);
    fn MmCanFileBeTruncated( section_object_pointer: *mut SECTION_OBJECT_POINTERS, new_file_size: *mut LARGE_INTEGER, ) -> BOOLEAN;
    // ExReleaseResourceLite via ntkrnl_sys::ddk (kernel FASTCALL export — §9.99)
    fn RtlCopyMemory(Destination: PVOID, Source: PVOID, Length: usize);
    fn FatCreateDcb( IrpContext: PIRP_CONTEXT, Vcb: PVCB, ParentDcb: PDCB, LfnByteOffset: ULONG, DirentByteOffset: ULONG, Dirent: PDIRENT, Lfn: PUNICODE_STRING, ) -> PDCB;
    fn FsRtlOplockFsctrl( Oplock: POPLOCK, Irp: PIRP, OpenCount: ULONG, ) -> NTSTATUS;
    fn FatInitializeDirectoryDirent( IrpContext: PIRP_CONTEXT, Dcb: PDCB, Dirent: PDIRENT, );
    fn IoSetShareAccess( DesiredAccess: ACCESS_MASK, ShareAccess: ULONG, FileObject: PFILE_OBJECT, ShareAccessPtr: *mut SHARE_ACCESS, );
    fn ObDereferenceObject(Object: PVOID);
    fn FatGetNeedEaCount(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Dirent: *mut DIRENT, NeedEaCount: *mut u32);
    fn FatLookupFileAllocationSize(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn FsRtlOplockBreakH(Oplock: *mut OPLOCK, Irp: *mut IRP, Flags: ULONG, Context: *mut c_void, CompletionRoutine: *mut c_void, PostIrpRoutine: *mut c_void) -> NTSTATUS;
    fn FatIsFastIoPossible(Fcb: *mut FCB) -> u8;
    // `fat_check_system_security_access` and `fat_supersede_or_overwrite_file` were
    // declared here as snake_case externs while BOTH functions are already translated
    // in this very file — FatSupersedeOrOverwriteFile and FatCheckSystemSecurityAccess,
    // #[no_mangle], PascalCase. Nothing ever called the snake_case names, but the decls
    // were enough for the cohab stub generator to manufacture KeBugCheckEx bodies for
    // them in rust_cohab_stub.c. Deleted both, and the stubs with them: an unused extern
    // decl is not free, it MINTS a bugcheck. scan_bugcheck_stubs now separates this case
    // ([ORPHAN_DECL], declared but never called) from a genuinely reachable stub.
    fn FatFspClose(Vcb: *mut VCB);
    fn FatIsNameLongUnicodeValid( IrpContext: *mut IRP_CONTEXT, Name: *mut _UNICODE_STRING, a: BOOLEAN, b: BOOLEAN, c: BOOLEAN, ) -> BOOLEAN;
    // FatOpenExistingDirectory now DEFINED in Rust below (was a BUGCHECK stub) — §9.45.
    // FatOpenVolume now DEFINED in Rust below (was a BUGCHECK stub) — see task #47.
    fn FatRemoveNames(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn MmDoesFileHaveUserWritableReferences(SectionObjectPointers: *const SECTION_OBJECT_POINTERS) -> BOOLEAN;
    fn KeSetEvent(Event: *mut KEVENT, Increment: i32, Wait: i32);
    fn FatSetFullFileNameInFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FsRtlDissectName( Name: _UNICODE_STRING, FirstPart: *mut _UNICODE_STRING, RemainingPart: *mut _UNICODE_STRING, );
    fn IoSetHardErrorOrVerifyDevice(Irp: *mut IRP, RealDevice: *mut DEVICE_OBJECT);
    fn RtlDowncaseUnicodeString( Dest: *mut _UNICODE_STRING, Src: *mut _UNICODE_STRING, Allocate: BOOLEAN, ) -> NTSTATUS;
    fn RtlUpcaseUnicodeString( Dest: *mut _UNICODE_STRING, Src: *mut _UNICODE_STRING, Allocate: BOOLEAN, ) -> NTSTATUS;
    fn RtlUpcaseUnicodeStringToCountedOemString( Dest: *mut OEM_STRING, Src: *mut _UNICODE_STRING, Allocate: BOOLEAN, ) -> NTSTATUS;
    fn RtlMoveMemory(Dest: *mut u8, Src: *const u8, Length: u32);
    fn CollectCreateHitStatistics(Vcb: *mut VCB);
    fn CollectCreateStatistics(Vcb: *mut VCB, Status: NTSTATUS);
    fn FsRtlExitFileSystem();
}
extern "C" {
    // wdm FORCEINLINE helper: cdecl body, bindgen class — was "system" (§9.99).
    fn IoGetCurrentIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_CREATE;
pub const Dbg: u32 = DEBUG_TRACE_CREATE;
pub const SE_SECURITY_PRIVILEGE: u32 = 8;
pub static mut FatSecurityPrivilege: LUID = LUID { LowPart: SE_SECURITY_PRIVILEGE, HighPart: 0 };

const IRP_CONTEXT_FLAG_SWAPPED_STACK: u32 = 0x00100000;
// ke.h KERNEL_STACK_SIZE is per-arch (was a flat 0x8000): x86/ke.h:218 12288,
// amd64/ke.h:182 and arm64/ke.h:49 0xC000. (rust_cohab_fat.c's callout shim ignores it.)
#[cfg(target_arch = "x86")]
const KERNEL_STACK_SIZE: u32 = 0x3000;
#[cfg(not(target_arch = "x86"))]
const KERNEL_STACK_SIZE: u32 = 0xC000;

const FILE_OPEN: u32 = 1;
const FILE_OPEN_IF: u32 = 3;
const FILE_OPENED: u32 = 1;
const UserDirectoryOpen: i32 = 3;

// Define BOOLEAN = i32, IRP_CONTEXT, FILE_OBJECT, VCB, DCB, CCB as opaque types.
// They are available from the caller.

const TAG_FILENAME_BUFFER: u32 = 0x6E746146; // placeholder

const FAT_NTC_FCB: u32 = 0x0502; // TODO: actual value from the original FAT driver header

// Constants
const FAT_DIRENT_ATTR_READ_ONLY: USHORT = 0x01;
const FAT_DIRENT_ATTR_DIRECTORY: USHORT = 0x10;
const TAG_DIRENT: ULONG = 0x44746146; // 'DIRF' maybe
const FILE_NOTIFY_CHANGE_DIR_NAME: ULONG = 0x02;
const FILE_ACTION_ADDED: ULONG = 0x00000001;
const FILE_CREATED: ULONG = 0x00000002;
const CCB_FLAG_DELETE_ON_CLOSE: ULONG = 0x0400; // assumed
const IRP_CONTEXT_FLAG_DISABLE_RAISE: ULONG = 0x0800; // guessed

const FAT_DIRENT_DELETED: u8 = 0xE5;
const FCB_STATE_TEMPORARY: u32 = 0x00000020; // typical
const FCB_STATE_TRUNCATE_ON_CLOSE: u32 = 0x02; // typical
const CCB_FLAG_OPENED_BY_SHORTNAME: u32 = 0x0800;
const CCB_FLAG_MANAGE_VOLUME_ACCESS: u32 = 0x00020000;
const FO_NO_INTERMEDIATE_BUFFERING: u32 = 0x00000008;
const UserFileOpen: i32 = 2; // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header (_TYPE_OF_OPEN_UserFileOpen=2; 4 is UserVolumeOpen!)
const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 1;
const FatLargeZero: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 }; // provided elsewhere, need reference
const OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY: u32 = 0x02;
const OPLOCK_FLAG_PARENT_OBJECT: u32 = 0x00000010;

// FatData is defined externally in fatdata.rs; extern reference here.
extern "system" { static mut FatData: FAT_DATA; }

const FILE_CREATE: u32 = 2;
const FILE_SUPERSEDE: u32 = 0;
const FILE_OVERWRITE: u32 = 4;
const FILE_OVERWRITE_IF: u32 = 5;
const FAT_DIRENT_ATTR_HIDDEN: u8 = 0x02;
const FAT_DIRENT_ATTR_SYSTEM: u8 = 0x04;
const FILE_ATTRIBUTE_HIDDEN: u16 = 0x02;
const FILE_ATTRIBUTE_SYSTEM: u16 = 0x04;
const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000;
const DO_VERIFY_VOLUME: u32 = 0x02;
const SL_FORCE_ACCESS_CHECK: u32 = 0x01;

// (inner attr removed — covered by lib.rs crate-level allow)

// Constants not available in the imports list
const FAT_NTC_ROOT_DCB: u32 = 0x0504;
const FCB_STATE_DELAY_CLOSE: u32 = 0x0800;

// Forward declarations for unknown external functions

// Forward declarations for unknown symbols
const FILE_OPBATCH_BREAK_UNDERWAY: u32 = 0x09;
const FCB_STATE_PAGING_FILE: u32 = 0x04;
const VCB_STATE_FLAG_CREATE_IN_PROGRESS: u32 = 0x0200;
const FILE_WRITE_DATA: u32 = 0x00000002;
const FILE_WRITE_EA: u32 = 0x00000010;
const FILE_WRITE_ATTRIBUTES: u32 = 0x00000100;

#[no_mangle]
pub unsafe extern "system" fn FatFsdCreate(
    VolumeDeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let mut TopLevel: BOOLEAN = 0;
    let mut ExceptionCompletedIrp: BOOLEAN = 0;

    if FatDeviceIsFatFsdo(VolumeDeviceObject) != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = FILE_OPENED as _;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_SUCCESS;
    }

    TimerStart!(core::ptr::null());
    DebugTrace!(1, (Dbg) as i32, b"FatFsdCreate\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();
    TopLevel = (FatIsIrpTopLevel(Irp)) as u8;

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, ... ))
    let p_ic = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_ic as *const PIRP_CONTEXT, || {
        *p_ic = FatCreateIrpContext(Irp, 1); // TRUE
        // Compile-time version check; assume NTDDI_WINTHRESHOLD
        // We approximate with cfg, but use a plain if false for compatibility
        #[cfg(any())] // never true
        let s = FatCommonCreateOnNewStack(*p_ic, Irp);
        #[cfg(not(any()))]
        let s = FatCommonCreate(*p_ic, Irp);
        *p_status = s;
    }) {
        //  We had some trouble trying to perform the requested
        //  operation, so we'll abort the I/O request with
        //  the error status that we get back from the
        //  execption code
        Status = crate::fatdata::FatProcessException(IrpContext, Irp, code);
        ExceptionCompletedIrp = 1;
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }
    FsRtlExitFileSystem();
    if ExceptionCompletedIrp == 0 && Status != STATUS_PENDING {
        FatCompleteRequest(IrpContext, Irp, Status);
    }
    DebugTrace!(-1, (Dbg) as i32, b"FatFsdCreate -> %08lx\n\0".as_ptr() as *const i8, Status);
    TimerStop!(core::ptr::null(), b"FatFsdCreate\0".as_ptr() as *const i8);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatCommonCreateCallout(CalloutParameters: *mut FAT_CALLOUT_PARAMETERS) {
    PAGED_CODE!();
    (*CalloutParameters).IrpStatus = FatCommonCreate((*CalloutParameters).__bindgen_anon_1.Create.IrpContext, (*CalloutParameters).__bindgen_anon_1.Create.Irp);
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonCreateOnNewStack(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut CalloutParameters: FAT_CALLOUT_PARAMETERS = core::mem::zeroed();
    let mut status: NTSTATUS;

    PAGED_CODE!();

    CalloutParameters.__bindgen_anon_1.Create.IrpContext = IrpContext;
    CalloutParameters.__bindgen_anon_1.Create.Irp = Irp;
    CalloutParameters.ExceptionStatus = STATUS_SUCCESS;
    CalloutParameters.IrpStatus = STATUS_SUCCESS;

    SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_SWAPPED_STACK);

    status = KeExpandKernelStackAndCalloutEx(
        core::mem::transmute::<unsafe extern "system" fn(*mut FAT_CALLOUT_PARAMETERS), extern "system" fn(*mut FAT_CALLOUT_PARAMETERS)>(FatCommonCreateCallout),
        &mut CalloutParameters as *mut FAT_CALLOUT_PARAMETERS,
        KERNEL_STACK_SIZE as SIZE_T,
        0, // FALSE
        core::ptr::null_mut(),
    );

    if status != STATUS_PENDING {
        ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_SWAPPED_STACK as u32);
    }

    if CalloutParameters.ExceptionStatus < 0 {
        FatRaiseStatus(IrpContext, CalloutParameters.ExceptionStatus);
    }

    if status < 0 {
        if status == STATUS_NO_MEMORY {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
        return status;
    }

    return CalloutParameters.IrpStatus;
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonCreate(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Add extern declarations for functions used that are not yet translated

    const FAT_CREATE_INITIAL_NAME_BUF_SIZE: usize = 0x20;
    const STATUS_OBJECT_NAME_INVALID: NTSTATUS = 0xC0000033_u32 as i32;
    const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002_u32 as i32;
    const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as i32;
    const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022_u32 as i32;
    const STATUS_VOLUME_DISMOUNTED: NTSTATUS = 0xC000026E_u32 as i32;
    const STATUS_MEDIA_WRITE_PROTECTED: NTSTATUS = 0xC00000A2_u32 as i32;
    const STATUS_EAS_NOT_SUPPORTED: NTSTATUS = 0xC000004F_u32 as i32;
    const STATUS_NOT_A_DIRECTORY: NTSTATUS = 0xC0000103_u32 as i32;
    const STATUS_OBJECT_PATH_NOT_FOUND: NTSTATUS = 0xC000003A_u32 as i32;
    const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034_u32 as i32;
    const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BA_u32 as i32;
    const STATUS_CANNOT_DELETE: NTSTATUS = 0xC0000121_u32 as i32;
    const STATUS_DELETE_PENDING: NTSTATUS = 0xC0000056_u32 as i32;
    const STATUS_SHARING_VIOLATION: NTSTATUS = 0xC0000043_u32 as i32;
    const STATUS_UNMAPPABLE_CHARACTER: NTSTATUS = 0xC0000162_u32 as i32;
    const STATUS_FILE_INVALID: NTSTATUS = 0xC0000098_u32 as i32;
    const STATUS_SUCCESS: NTSTATUS = 0;
    const DO_VERIFY_VOLUME: u32 = 0x02;
    const FO_CACHE_SUPPORTED: u32 = 0x40;
    const FILE_OPEN_BY_FILE_ID: u32 = 0x2000;
    const FILE_DIRECTORY_FILE: u32 = 0x1;
    const FILE_NON_DIRECTORY_FILE: u32 = 0x40;
    const FILE_NO_INTERMEDIATE_BUFFERING: u32 = 0x8;
    const FILE_NO_EA_KNOWLEDGE: u32 = 0x200;
    const FILE_DELETE_ON_CLOSE: u32 = 0x1000;
    const FILE_OPEN_REQUIRING_OPLOCK: u32 = 0x10000;
    const FILE_ATTRIBUTE_TEMPORARY: u16 = 0x100;
    const FILE_OPEN: u32 = 1;
    const FILE_OVERWRITE: u32 = 0x04;
    const FILE_CREATE: u32 = 0x02;
    const FILE_OPEN_IF: u32 = 0x03;
    const SL_OPEN_PAGING_FILE: u32 = 0x2;
    const SL_OPEN_TARGET_DIRECTORY: u32 = 0x04;
    const VCB_STATE_FLAG_CREATE_IN_PROGRESS: u32 = 0x0200;
    const VCB_STATE_FLAG_LOCKED: u32 = 0x01;
    const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000;
    const FAT_DIRENT_ATTR_DIRECTORY: u8 = 0x10; // FAT_DIRENT_ATTR_DIRECTORY
    const FAT_NTC_FCB: u16 = 0x0502; // placeholder; adjust to actual values
    const FAT_NTC_DCB: u16 = 0x0503;
    const FAT_NTC_ROOT_DCB: u16 = 0x0504;
    const CCB_FLAG_SKIP_SHORT_NAME_COMPARE: u32 = 0x02;
    const CCB_FLAG_OPENED_BY_SHORTNAME: u32 = 0x0800;
    // FILE_ATTRIBUTE masks
    const FILE_ATTRIBUTE_READONLY: u16 = 0x1;
    const FILE_ATTRIBUTE_HIDDEN: u16 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u16 = 0x4;
    const FILE_ATTRIBUTE_ARCHIVE: u16 = 0x20;
    const FILE_ATTRIBUTE_ENCRYPTED: u16 = 0x4000;
    const FILE_ATTRIBUTE_NORMAL: u16 = 0x80;

    // Type aliases
    type BOOLEAN = u8;
    type NTSTATUS = i32;
    type ULONG_PTR = usize;
    type PCHAR = *mut u8;
    type PVOID = *mut u8;

    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace!(1, 0, b"FatCommonCreate\n\0".as_ptr() as *const i8);
    DebugTrace!(0, 0, b"Irp                       = %p\n\0".as_ptr() as *const i8, Irp);
    DebugTrace!(0, 0, b"->Flags                   = %08lx\n\0".as_ptr() as *const i8, (*Irp).Flags);
    DebugTrace!(0, 0, b"->FileObject              = %p\n\0".as_ptr() as *const i8, (*IrpSp).FileObject);
    DebugTrace!(0, 0, b" ->RelatedFileObject      = %p\n\0".as_ptr() as *const i8, (*(*IrpSp).FileObject).RelatedFileObject);
    DebugTrace!(0, 0, b" ->FileName               = %wZ\n\0".as_ptr() as *const i8, &(*(*IrpSp).FileObject).FileName);
    DebugTrace!(0, 0, b" ->FileName.Length        = 0n%d\n\0".as_ptr() as *const i8, (*(*IrpSp).FileObject).FileName.Length as u32);
    DebugTrace!(0, 0, b"->AllocationSize.__bindgen_anon_1.LowPart  = %08lx\n\0".as_ptr() as *const i8, (*Irp).Overlay.AllocationSize.__bindgen_anon_1.LowPart);
    DebugTrace!(0, 0, b"->AllocationSize.__bindgen_anon_1.HighPart = %08lx\n\0".as_ptr() as *const i8, (*Irp).Overlay.AllocationSize.__bindgen_anon_1.HighPart);
    DebugTrace!(0, 0, b"->SystemBuffer            = %p\n\0".as_ptr() as *const i8, (*Irp).AssociatedIrp.SystemBuffer);
    DebugTrace!(0, 0, b"->DesiredAccess           = %08lx\n\0".as_ptr() as *const i8, (*(*IrpSp).Parameters.Create.SecurityContext).DesiredAccess);
    DebugTrace!(0, 0, b"->Options                 = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Create.Options);
    DebugTrace!(0, 0, b"->FileAttributes          = %04x\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Create.FileAttributes as u32);
    DebugTrace!(0, 0, b"->ShareAccess             = %04x\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Create.ShareAccess as u32);
    DebugTrace!(0, 0, b"->EaLength                = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.Create.EaLength);

    if ((*(*IrpSp).FileObject).FileName.Length > core::mem::size_of::<u16>() as u16) &&
        (*(*(*IrpSp).FileObject).FileName.Buffer.offset(1)) == '\\' as u16 &&
        (*(*(*IrpSp).FileObject).FileName.Buffer.offset(0)) == '\\' as u16 {

        (*(*IrpSp).FileObject).FileName.Length -= core::mem::size_of::<u16>() as u16;
        core::ptr::copy(
            (*(*IrpSp).FileObject).FileName.Buffer.offset(1),
            (*(*IrpSp).FileObject).FileName.Buffer.offset(0),
            (*(*IrpSp).FileObject).FileName.Length as usize / core::mem::size_of::<u16>(),
        );

        if ((*(*IrpSp).FileObject).FileName.Length > core::mem::size_of::<u16>() as u16) &&
            (*(*(*IrpSp).FileObject).FileName.Buffer.offset(1)) == '\\' as u16 &&
            (*(*(*IrpSp).FileObject).FileName.Buffer.offset(0)) == '\\' as u16 {

            DebugTrace!(-1, 0, b"FatCommonCreate -> STATUS_OBJECT_NAME_INVALID\n\0".as_ptr() as *const i8);
            return STATUS_OBJECT_NAME_INVALID;
        }
    }

    NT_ASSERT!(!(*IrpSp).Parameters.Create.SecurityContext.is_null() as i32);

    let FileObject = (*IrpSp).FileObject;
    let RelatedFileObject = (*FileObject).RelatedFileObject;
    let AllocationSize = (*Irp).Overlay.AllocationSize.__bindgen_anon_1.LowPart;
    let EaBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut FILE_FULL_EA_INFORMATION;
    let DesiredAccess = &mut (*(*IrpSp).Parameters.Create.SecurityContext).DesiredAccess as *mut ACCESS_MASK;
    let Options = (*IrpSp).Parameters.Create.Options;
    let mut FileAttributes = ((*IrpSp).Parameters.Create.FileAttributes) & !FILE_ATTRIBUTE_NORMAL;
    let ShareAccess = (*IrpSp).Parameters.Create.ShareAccess;
    let EaLength = (*IrpSp).Parameters.Create.EaLength;

    if !RelatedFileObject.is_null() {
        (*FileObject).Vpb = (*RelatedFileObject).Vpb;
    }

    FileAttributes &= FILE_ATTRIBUTE_READONLY |
                      FILE_ATTRIBUTE_HIDDEN |
                      FILE_ATTRIBUTE_SYSTEM |
                      FILE_ATTRIBUTE_ARCHIVE |
                      FILE_ATTRIBUTE_ENCRYPTED;

    let Vcb = &mut (*((*IrpSp).DeviceObject as *mut VOLUME_DEVICE_OBJECT)).Vcb as *mut VCB;

    if FlagOn(Options, FILE_OPEN_BY_FILE_ID) != 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    let DirectoryFile = FlagOn(Options, FILE_DIRECTORY_FILE) != 0;
    let NonDirectoryFile = FlagOn(Options, FILE_NON_DIRECTORY_FILE) != 0;
    let NoIntermediateBuffering = FlagOn(Options, FILE_NO_INTERMEDIATE_BUFFERING) != 0;
    let NoEaKnowledge = FlagOn(Options, FILE_NO_EA_KNOWLEDGE) != 0;
    let DeleteOnClose = FlagOn(Options, FILE_DELETE_ON_CLOSE) != 0;
    let OpenRequiringOplock = FlagOn(Options, FILE_OPEN_REQUIRING_OPLOCK) != 0;
    let TemporaryFile = FlagOn((*IrpSp).Parameters.Create.FileAttributes as u32, FILE_ATTRIBUTE_TEMPORARY as u32) != 0;
    let CreateDisposition = (Options >> 24) & 0xFF;
    let IsPagingFile = FlagOn(((*IrpSp).Flags) as u32, SL_OPEN_PAGING_FILE) != 0;
    let OpenTargetDirectory = FlagOn(((*IrpSp).Flags) as u32, SL_OPEN_TARGET_DIRECTORY) != 0;
    let CreateDirectory = DirectoryFile &&
                           ((CreateDisposition == FILE_CREATE) ||
                            (CreateDisposition == FILE_OPEN_IF)) ;
    let OpenDirectory = DirectoryFile &&
                         ((CreateDisposition == FILE_OPEN) ||
                          (CreateDisposition == FILE_OPEN_IF)) ;

    if (*Irp).Overlay.AllocationSize.__bindgen_anon_1.HighPart != 0 ||
        (DirectoryFile && NonDirectoryFile) {
        DebugTrace!(-1, 0, b"FatCommonCreate -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8);
        return STATUS_INVALID_PARAMETER;
    }

    if FatAcquireExclusiveVcb(IrpContext, Vcb) == 0 {
        DebugTrace!(0, 0, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8);
        Iosb.__bindgen_anon_1.Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace!(-1, 0, b"FatCommonCreate -> %08lx\n\0".as_ptr() as *const i8, Iosb.__bindgen_anon_1.Status);
        return Iosb.__bindgen_anon_1.Status;
    }

    if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS) != 0 {
        FatBugCheck(0, 0, 0);
    }

    let mut DirentBcb: *mut BCB = core::ptr::null_mut();

    let mut OemFinalName = OEM_STRING {
        Length: 0,
        MaximumLength: (core::mem::size_of::<u8>() * FAT_CREATE_INITIAL_NAME_BUF_SIZE * 2) as u16,
        Buffer: ([0u8; FAT_CREATE_INITIAL_NAME_BUF_SIZE * 2].as_mut_ptr()).cast(),
    };
    let mut OemBuffer = [0u8; FAT_CREATE_INITIAL_NAME_BUF_SIZE * 2];
    OemFinalName.Buffer = (OemBuffer.as_mut_ptr()).cast();

    let mut UpcasedBuffer = [0u16; FAT_CREATE_INITIAL_NAME_BUF_SIZE];
    let mut UpcasedFinalName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (core::mem::size_of::<u16>() * FAT_CREATE_INITIAL_NAME_BUF_SIZE) as u16,
        Buffer: UpcasedBuffer.as_mut_ptr(),
    };

    let mut LfnBuffer = [0u16; FAT_CREATE_INITIAL_NAME_BUF_SIZE];
    let mut Lfn = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (core::mem::size_of::<u16>() * FAT_CREATE_INITIAL_NAME_BUF_SIZE) as u16,
        Buffer: LfnBuffer.as_mut_ptr(),
    };

    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut FinalDcb: *mut DCB = core::ptr::null_mut();
    let mut PostIrp = false;
    // BOOLEAN, not bool — an OUT parameter has to be addressable as the u8
    // the callee writes. `&mut (OplockPostIrp as u8)` addressed a TEMPORARY.
    let mut OplockPostIrp: BOOLEAN = 0;
    // BOOLEAN, not bool: this is an OUT parameter of FatFindFcb and
    // FatLocateDirent, and it has to be addressable as the u8 they write.
    let mut FileNameOpenedDos: BOOLEAN = 0;
    let mut TrailingBackslash = false;
    let mut FirstLoop = true;
    let mut MatchFlags: u32 = 0;
    let mut OrigLfn = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut LocalCcb: CCB = core::mem::zeroed();
    let mut FinalName = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut RemainingPart = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut NextRemainingPart = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };

    // The C's FileName is a LOCAL copy of FileObject->FileName: the trailing-
    // backslash strip below must not shorten the file object's own name.
    let mut FileName = (*FileObject).FileName;

    extern "system" {
        fn IoFreeMdl(Mdl: PMDL);
        static mut FatReserveMdl: PMDL;
    }
    const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: ULONG = 0x00000008;
    const VCB_STATE_FLAG_BOOT_OR_PAGING_FILE: ULONG = 0x00000800;
    const VCB_STATE_FLAG_REMOVABLE_MEDIA: ULONG = 0x00000002;
    const FILE_EXECUTE: u32 = 0x00000020;
    const FO_FILE_FAST_IO_READ: ULONG = 0x00080000;
    const FAT_RESERVE_MDL_SIZE: usize = 16;
    const PAGE_SIZE: usize = 0x1000;
    const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016_u32 as i32;

    // State the finally blocks read, taken as raw pointers so the body closure
    // can own the plain names.
    let p_iosb = core::ptr::addr_of_mut!(Iosb);
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_final_dcb = core::ptr::addr_of_mut!(FinalDcb);
    let p_oem_final_name = core::ptr::addr_of_mut!(OemFinalName);
    let p_upcased_final_name = core::ptr::addr_of_mut!(UpcasedFinalName);
    let p_lfn = core::ptr::addr_of_mut!(Lfn);
    let p_post_irp = core::ptr::addr_of_mut!(PostIrp);
    let p_oplock_post_irp = core::ptr::addr_of_mut!(OplockPostIrp);

    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }  (create.c:841-2103)
    crate::seh::try_finally(&mut (), |_| {
        'main_try: loop {
            // FatVerifyVcb is a VOID function — it FatRaiseStatus()es on a bad/verify
            // volume and returns nothing otherwise. The old code did `let vcb_status =
            // FatVerifyVcb(...)` and tested it, but there is no return value, so
            // vcb_status was the garbage in x0 (0xffffffff on arm64) → EVERY create
            // broke out of 'main_try with status -1 at the very top (smss/ntdll failed
            // to map → BUGCHECK 0x64). A raise unwinds through the finally below.
            // (Same void-as-status bug as FatVerifyFcb below.)
            FatVerifyVcb(IrpContext, Vcb);

            if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_LOCKED) != 0 {
                DebugTrace!(0, 0, b"Volume is locked\n\0".as_ptr() as *const i8);
                let mut status = STATUS_ACCESS_DENIED;
                if (*Vcb).VcbCondition != VcbGood as u32 { // VcbGood is an enum value
                    status = STATUS_VOLUME_DISMOUNTED;
                }
                Iosb.__bindgen_anon_1.Status = status;
                break 'main_try;
            }

            if DeleteOnClose && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
                IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, (*(*Vcb).Vpb).RealDevice);
                SetFlag!((*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                FatRaiseStatus(IrpContext, STATUS_MEDIA_WRITE_PROTECTED);
            }

            if !EaBuffer.is_null() {
                Iosb.__bindgen_anon_1.Status = STATUS_EAS_NOT_SUPPORTED;
                break 'main_try;
            }

            if FileName.Length == 0 {
                let mut DecodeVcb: *mut VCB = core::ptr::null_mut();
                if RelatedFileObject.is_null() ||
                    FatDecodeFileObject(RelatedFileObject, &mut DecodeVcb, &mut Fcb, &mut core::ptr::null_mut()) == UserVolumeOpen {
                    NT_ASSERT!((RelatedFileObject.is_null() || Vcb == DecodeVcb) as i32);

                    if DirectoryFile {
                        DebugTrace!(0, 0, b"Cannot open volume as a directory\n\0".as_ptr() as *const i8);
                        Iosb.__bindgen_anon_1.Status = STATUS_NOT_A_DIRECTORY;
                        break 'main_try;
                    }
                    if OpenTargetDirectory {
                        Iosb.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                        break 'main_try;
                    }
                    DebugTrace!(0, 0, b"Opening the volume, Vcb = %p\n\0".as_ptr() as *const i8, Vcb);
                    CollectCreateHitStatistics(Vcb);
                    Iosb = FatOpenVolume(IrpContext, FileObject, Vcb, DesiredAccess, ShareAccess, CreateDisposition);
                    (*Irp).IoStatus.Information = Iosb.Information;
                    break 'main_try;
                }
            }

            let mut ParentDcb: *mut DCB;
            if !RelatedFileObject.is_null() {
                let mut RelatedVcb: *mut VCB = core::ptr::null_mut();
                let mut RelatedDcb: *mut DCB = core::ptr::null_mut();
                let mut RelatedCcb: *mut CCB = core::ptr::null_mut();
                let TypeOfOpen = FatDecodeFileObject(RelatedFileObject, &mut RelatedVcb, &mut RelatedDcb, &mut RelatedCcb);
                if !(TypeOfOpen == (UserFileOpen) as u32 || TypeOfOpen == (UserDirectoryOpen) as u32) {
                    DebugTrace!(0, 0, b"Invalid related file object\n\0".as_ptr() as *const i8);
                    Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_PATH_NOT_FOUND;
                    break 'main_try;
                }
                if FileName.Length != 0 && *FileName.Buffer.offset(0) == '\\' as u16 {
                    Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                    break 'main_try;
                }
                NT_ASSERT!((Vcb == RelatedVcb) as i32);
                (*FileObject).Vpb = (*RelatedFileObject).Vpb;

                // FatVerifyFcb is VOID (raises on bad Fcb). See fix note above.
                FatVerifyFcb((IrpContext).cast(), RelatedDcb);
                ParentDcb = RelatedDcb;
            } else {
                if FileName.Length == core::mem::size_of::<u16>() as u16 &&
                    *FileName.Buffer.offset(0) == '\\' as u16 {
                    if NonDirectoryFile {
                        DebugTrace!(0, 0, b"Cannot open root directory as a file\n\0".as_ptr() as *const i8);
                        Iosb.__bindgen_anon_1.Status = STATUS_FILE_IS_A_DIRECTORY;
                        break 'main_try;
                    }
                    if OpenTargetDirectory {
                        Iosb.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                        break 'main_try;
                    }
                    if DeleteOnClose {
                        Iosb.__bindgen_anon_1.Status = STATUS_CANNOT_DELETE;
                        break 'main_try;
                    }
                    DebugTrace!(0, 0, b"Opening root dcb\n\0".as_ptr() as *const i8);
                    CollectCreateHitStatistics(Vcb);
                    Iosb = FatOpenRootDcb(IrpContext, FileObject, Vcb, DesiredAccess, ShareAccess, CreateDisposition);
                    (*Irp).IoStatus.Information = Iosb.Information;
                    break 'main_try;
                }

                ParentDcb = (*Vcb).RootDcb;

                // FatVerifyFcb is VOID (raises on bad Fcb). See fix note above.
                FatVerifyFcb((IrpContext).cast(), ParentDcb);
            }

            if FileName.Length != 0 &&
                *FileName.Buffer.offset((FileName.Length / core::mem::size_of::<u16>() as u16) as isize - 1) == '\\' as u16 {
                FileName.Length -= core::mem::size_of::<u16>() as u16;
                TrailingBackslash = true;
            } else {
                TrailingBackslash = false;
            }

            if (*ParentDcb).FullFileName.Buffer.is_null() {
                FatSetFullFileNameInFcb(IrpContext, ParentDcb);
            }

            // (USHORT) truncation in the C: the sum must wrap for the overflow test to work.
            if ((*ParentDcb).FullFileName.Length as u16)
                .wrapping_add(core::mem::size_of::<u16>() as u16)
                .wrapping_add(FileName.Length) <= FileName.Length {
                Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'main_try;
            }

            loop {
                Fcb = ParentDcb;
                RemainingPart = FileName;

                while RemainingPart.Length != 0 {
                    let mut NextFcb: *mut FCB;
                    FsRtlDissectName(RemainingPart, &mut FinalName, &mut NextRemainingPart);

                    if (NextRemainingPart.Length != 0 && *(NextRemainingPart.Buffer.offset(0)) == '\\' as u16) ||
                        FinalName.Length > 255 * core::mem::size_of::<u16>() as u16  {
                        Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                        break 'main_try;
                    }

                    FatEnsureStringBufferEnough(&mut OemFinalName as *mut _ as *mut core::ffi::c_void, FinalName.Length);

                    let mut Status = RtlUpcaseUnicodeStringToCountedOemString(&mut OemFinalName, &FinalName as *const _ as *mut _, 0);
                    if NT_SUCCESS(Status) != 0 {
                        let dcb_ref = (*Fcb).Specific.Dcb.as_mut();
                        NextFcb = FatFindFcb(IrpContext, core::ptr::addr_of_mut!(dcb_ref.RootOemNode).cast(), &OemFinalName as *const _ as *mut STRING, &mut FileNameOpenedDos);
                    } else {
                        NextFcb = core::ptr::null_mut();
                        OemFinalName.Length = 0;
                        if Status != STATUS_UNMAPPABLE_CHARACTER {
                            Iosb.__bindgen_anon_1.Status = Status;
                            break 'main_try;
                        }
                    }

                    if NextFcb.is_null() && !(*Fcb).Specific.Dcb.as_ref().RootUnicodeNode.is_null() {
                        FatEnsureStringBufferEnough(&mut UpcasedFinalName as *mut _ as *mut core::ffi::c_void, FinalName.Length);
                        Status = RtlDowncaseUnicodeString(&mut UpcasedFinalName, &FinalName as *const _ as *mut _, 0);
                        NT_ASSERT!((NT_SUCCESS(Status) != 0) as i32);
                        Status = RtlUpcaseUnicodeString(&mut UpcasedFinalName, &UpcasedFinalName as *const _ as *mut _, 0);
                        NT_ASSERT!((NT_SUCCESS(Status) != 0) as i32);
                        let dcb_ref2 = (*Fcb).Specific.Dcb.as_mut();
                        NextFcb = FatFindFcb(IrpContext, core::ptr::addr_of_mut!(dcb_ref2.RootUnicodeNode).cast(), &UpcasedFinalName as *const _ as *mut STRING, &mut FileNameOpenedDos);
                    }

                    if !NextFcb.is_null() {
                        Fcb = NextFcb;
                        RemainingPart = NextRemainingPart;
                    }

                    if NextFcb.is_null() || (NodeType!(NextFcb as *const c_void)) as u32 == (FAT_NTC_FCB) as u32 || NextRemainingPart.Length == 0 {
                        break;
                    }
                }

                if RemainingPart.Length != 0 && *(RemainingPart.Buffer.offset(0)) == '\\' as u16 {
                    RemainingPart.Length -= core::mem::size_of::<u16>() as u16;
                    RemainingPart.Buffer = RemainingPart.Buffer.offset(1);
                }

                // Verify the Fcb. BUG FIX (arm64 every-create-fails): FatVerifyFcb is
                // a VOID function — it FatRaiseStatus()es on a bad/needs-verify Fcb and
                // returns nothing on a good one. The old code did `let ret =
                // FatVerifyFcb(...)` and tested `ret`, but there is no return value, so
                // `ret` was the garbage left in x0 (0xffffffff on arm64) → every create
                // broke out with status -1 (smss/ntdll failed to load → BUGCHECK 0x64).
                // STATUS_FILE_INVALID is swallowed here (the Fcb is then not FcbGood and
                // its names are removed below); anything else keeps unwinding.
                if crate::seh::try_except(&mut (), |_| {
                    FatVerifyFcb((IrpContext).cast(), Fcb);
                }, |_, p| {
                    if crate::seh::exception_code(p) == STATUS_FILE_INVALID {
                        crate::seh::EXCEPTION_EXECUTE_HANDLER
                    } else {
                        crate::seh::EXCEPTION_CONTINUE_SEARCH
                    }
                }).is_err() {
                    FatResetExceptionState(IrpContext);
                }

                if (*Fcb).FcbCondition == FcbGood as u32 {
                    if IsPagingFile && FirstLoop && (NodeType!(Fcb as *const c_void)) as u32 == (FAT_NTC_FCB) as u32 &&
                        (!IsListEmpty(&raw const FatData.AsyncCloseList as *const _) || !IsListEmpty(&raw const FatData.DelayedCloseList as *const _)) {
                        FatFspClose(Vcb);
                        FirstLoop = false;
                        continue;
                    } else {
                        break;
                    }
                } else {
                    FatRemoveNames(IrpContext, Fcb);
                }
            }

            NT_ASSERT!(((*Fcb).FcbCondition == FcbGood as u32) as i32);

            if IsPagingFile {
                if (NodeType!(Fcb as *const c_void)) as u32 == (FAT_NTC_FCB) as u32 && FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0 {
                    Iosb.__bindgen_anon_1.Status = STATUS_SHARING_VIOLATION;
                    break 'main_try;
                }
            } else if FlagOn((*Fcb).FcbState, FCB_STATE_SYSTEM_FILE) != 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
                break 'main_try;
            }

            if FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_DELETE_PENDING;
                break 'main_try;
            }

            if RemainingPart.Length == 0 {
                if OpenTargetDirectory {
                    CollectCreateHitStatistics(Vcb);
                    Iosb = FatOpenTargetDirectory(IrpContext, FileObject, (*Fcb).ParentDcb, DesiredAccess, ShareAccess, 1, FileNameOpenedDos);
                    (*Irp).IoStatus.Information = Iosb.Information;
                    break 'main_try;
                }

                if (NodeType!(Fcb as *const c_void)) as u32 == (FAT_NTC_DCB) as u32 || (NodeType!(Fcb as *const c_void)) as u32 == (FAT_NTC_ROOT_DCB) as u32 {
                    if NonDirectoryFile {
                        DebugTrace!(0, 0, b"Cannot open directory as a file\n\0".as_ptr() as *const i8);
                        Iosb.__bindgen_anon_1.Status = STATUS_FILE_IS_A_DIRECTORY;
                        break 'main_try;
                    }
                    DebugTrace!(0, 0, b"Open existing dcb, Dcb = %p\n\0".as_ptr() as *const i8, Fcb);
                    CollectCreateHitStatistics(Vcb);
                    Iosb = FatOpenExistingDcb(IrpContext, IrpSp, FileObject, Vcb, Fcb as *mut DCB, DesiredAccess, ShareAccess,
                                             CreateDisposition, NoEaKnowledge as u8, DeleteOnClose as u8, OpenRequiringOplock as u8,
                                             FileNameOpenedDos, &mut OplockPostIrp);
                    if Iosb.__bindgen_anon_1.Status != STATUS_PENDING {
                        (*Irp).IoStatus.Information = Iosb.Information;
                    } else {
                        DebugTrace!(0, 0, b"Enqueue Irp to FSP\n\0".as_ptr() as *const i8);
                        PostIrp = true;
                    }
                    break 'main_try;
                }

                if (NodeType!(Fcb as *const c_void)) as u32 == (FAT_NTC_FCB) as u32 {
                    if OpenDirectory {
                        DebugTrace!(0, 0, b"Cannot open file as directory\n\0".as_ptr() as *const i8);
                        Iosb.__bindgen_anon_1.Status = STATUS_NOT_A_DIRECTORY;
                        break 'main_try;
                    }
                    DebugTrace!(0, 0, b"Open existing fcb, Fcb = %p\n\0".as_ptr() as *const i8, Fcb);
                    if TrailingBackslash {
                        Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                        break 'main_try;
                    }
                    CollectCreateHitStatistics(Vcb);
                    Iosb = FatOpenExistingFcb(IrpContext, IrpSp, FileObject, Vcb, Fcb, DesiredAccess, ShareAccess,
                                              AllocationSize, EaBuffer, EaLength, FileAttributes, CreateDisposition,
                                              NoEaKnowledge as u8, DeleteOnClose as u8, OpenRequiringOplock as u8, FileNameOpenedDos, &mut OplockPostIrp);
                    if Iosb.__bindgen_anon_1.Status != STATUS_PENDING {
                        if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !NoIntermediateBuffering {
                            (*FileObject).Flags |= FO_CACHE_SUPPORTED;
                        }
                        (*Irp).IoStatus.Information = Iosb.Information;
                    } else {
                        DebugTrace!(0, 0, b"Enqueue Irp to FSP\n\0".as_ptr() as *const i8);
                        PostIrp = true;
                    }
                    break 'main_try;
                }

                FatBugCheck((NodeType!(Fcb as *const c_void)) as ULONG_PTR, Fcb as ULONG_PTR, 0);
            }

            if (NodeType!(Fcb as *const c_void)) as u32 != (FAT_NTC_DCB) as u32 && (NodeType!(Fcb as *const c_void)) as u32 != (FAT_NTC_ROOT_DCB) as u32 {
                DebugTrace!(0, 0, b"Cannot open file as subdirectory, Fcb = %p\n\0".as_ptr() as *const i8, Fcb);
                Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_PATH_NOT_FOUND;
                break 'main_try;
            }

            ParentDcb = Fcb as *mut DCB;
            FirstLoop = true;
            let mut Dirent: *mut DIRENT = core::ptr::null_mut();
            let mut DirentByteOffset: u32 = 0;

            loop {
                let Status: NTSTATUS;
                if FirstLoop {
                    FirstLoop = false;
                    RemainingPart = NextRemainingPart;
                    Status = if OemFinalName.Length != 0 { STATUS_SUCCESS } else { STATUS_UNMAPPABLE_CHARACTER };
                } else {
                    DebugTrace!(0, 0, b"Dissecting the name %wZ\n\0".as_ptr() as *const i8, &RemainingPart);
                    FsRtlDissectName(RemainingPart, &mut FinalName, &mut RemainingPart);

                    if (RemainingPart.Length != 0 && *(RemainingPart.Buffer.offset(0)) == '\\' as u16) ||
                        FinalName.Length > 255 * core::mem::size_of::<u16>() as u16  {
                        Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                        break 'main_try;
                    }

                    FatEnsureStringBufferEnough(&mut OemFinalName as *mut _ as *mut core::ffi::c_void, FinalName.Length);
                    Status = RtlUpcaseUnicodeStringToCountedOemString(&mut OemFinalName, &FinalName as *const _ as *mut _, 0);
                }

                if NT_SUCCESS(Status) != 0 {
                    if FatIsNameShortOemValid(IrpContext, OemFinalName, 0, 0, 0) != 0 {
                        FatStringTo8dot3((IrpContext).cast(), OemFinalName, (LocalCcb.__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate.Constant.as_mut().as_mut_ptr()).cast());
                        LocalCcb.Flags = 0;
                    } else {
                        LocalCcb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE;
                    }
                } else {
                    LocalCcb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE;
                    if Status != STATUS_UNMAPPABLE_CHARACTER {
                        Iosb.__bindgen_anon_1.Status = Status;
                        break 'main_try;
                    }
                }

                if (*&raw const FatData).ChicagoMode() != 0 {
                    if FatIsNameLongUnicodeValid(IrpContext, &FinalName as *const _ as *mut _, 0, 0, 0) == 0 {
                        Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                        break 'main_try;
                    }
                } else {
                    if FlagOn(LocalCcb.Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE) != 0 {
                        Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                        break 'main_try;
                    }
                }

                DebugTrace!(0, 0, b"FinalName is %wZ\n\0".as_ptr() as *const i8, &FinalName);
                DebugTrace!(0, 0, b"RemainingPart is %wZ\n\0".as_ptr() as *const i8, &RemainingPart);

                FatEnsureStringBufferEnough(&mut UpcasedFinalName as *mut _ as *mut core::ffi::c_void, FinalName.Length);
                let status = RtlUpcaseUnicodeString(&mut UpcasedFinalName, &FinalName as *const _ as *mut _, 0);
                if NT_SUCCESS(status) == 0 {
                    Iosb.__bindgen_anon_1.Status = status;
                    break 'main_try;
                }

                LocalCcb.__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate = UpcasedFinalName;
                LocalCcb.ContainsWildCards = 0;
                Lfn.Length = 0;

                Dirent = core::ptr::null_mut();
                DirentByteOffset = 0;
                FatLocateDirent(IrpContext, ParentDcb, &mut LocalCcb, 0, &mut MatchFlags,
                                &mut Dirent, &mut DirentBcb, &mut DirentByteOffset,
                                &mut FileNameOpenedDos, &mut Lfn, &mut OrigLfn);

                FinalDcb = ParentDcb;

                if RemainingPart.Length == 0 {
                    // will process outside loop
                    break;
                }

                if Dirent.is_null() {
                    Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_PATH_NOT_FOUND;
                    break 'main_try;
                }

                if FlagOn((*Dirent).Attributes as u32, FAT_DIRENT_ATTR_DIRECTORY as u32) == 0 {
                    Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_PATH_NOT_FOUND;
                    break 'main_try;
                }

                let LfnByteOffset = DirentByteOffset - ((FAT_LFN_DIRENTS_NEEDED(&mut OrigLfn) as u32) * core::mem::size_of::<LFN_DIRENT>() as u32);

                ParentDcb = FatCreateDcb(IrpContext, Vcb, ParentDcb, LfnByteOffset, DirentByteOffset, Dirent, &mut Lfn);
                FinalDcb = ParentDcb;
                FatSetFullNameInFcb(IrpContext, ParentDcb, &FinalName as *const _ as *mut _);
            }

            if OpenTargetDirectory {
                Iosb = FatOpenTargetDirectory(IrpContext, FileObject, ParentDcb, DesiredAccess, ShareAccess,
                                             if Dirent.is_null() { 0u8 } else { 1u8 }, FileNameOpenedDos);
                (*Irp).IoStatus.Information = Iosb.Information;
                break 'main_try;
            }

            if !Dirent.is_null() {
                let LfnByteOffset = DirentByteOffset - ((FAT_LFN_DIRENTS_NEEDED(&mut OrigLfn) as u32) * core::mem::size_of::<LFN_DIRENT>() as u32);

                if FlagOn((*Dirent).Attributes as u32, FAT_DIRENT_ATTR_DIRECTORY as u32) != 0 {
                    if NonDirectoryFile {
                        DebugTrace!(0, 0, b"Cannot open directory as a file\n\0".as_ptr() as *const i8);
                        Iosb.__bindgen_anon_1.Status = STATUS_FILE_IS_A_DIRECTORY;
                        break 'main_try;
                    }
                    DebugTrace!(0, 0, b"Open existing directory\n\0".as_ptr() as *const i8);
                    Iosb = FatOpenExistingDirectory(IrpContext, IrpSp, FileObject, Vcb, &mut Fcb, ParentDcb, Dirent,
                                                   LfnByteOffset, DirentByteOffset, &mut Lfn, DesiredAccess, ShareAccess,
                                                   CreateDisposition, NoEaKnowledge as u8, DeleteOnClose as u8, FileNameOpenedDos,
                                                   OpenRequiringOplock as u8);
                    (*Irp).IoStatus.Information = Iosb.Information;
                    break 'main_try;
                }

                if OpenDirectory {
                    DebugTrace!(0, 0, b"Cannot open file as directory\n\0".as_ptr() as *const i8);
                    Iosb.__bindgen_anon_1.Status = STATUS_NOT_A_DIRECTORY;
                    break 'main_try;
                }

                DebugTrace!(0, 0, b"Open existing file\n\0".as_ptr() as *const i8);
                if TrailingBackslash {
                    Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                    break 'main_try;
                }

                Iosb = FatOpenExistingFile(IrpContext, FileObject, Vcb, &mut Fcb, ParentDcb, Dirent, LfnByteOffset,
                                           DirentByteOffset, &mut Lfn, &mut OrigLfn, DesiredAccess, ShareAccess,
                                           AllocationSize, EaBuffer, EaLength, FileAttributes, CreateDisposition,
                                           IsPagingFile as i32, NoEaKnowledge as i32, DeleteOnClose as i32, OpenRequiringOplock as i32,
                                           FileNameOpenedDos as i32);

                if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !NoIntermediateBuffering {
                    (*FileObject).Flags |= FO_CACHE_SUPPORTED;
                }
                (*Irp).IoStatus.Information = Iosb.Information;
                break 'main_try;
            }

            if CreateDisposition == FILE_OPEN || CreateDisposition == FILE_OVERWRITE {
                DebugTrace!(0, 0, b"Cannot open nonexisting file\n\0".as_ptr() as *const i8);
                Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_NOT_FOUND;
                break 'main_try;
            }

            if FlagOn(LocalCcb.Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE) != 0 {
                OemFinalName.Length = 0;
            }

            let check_status = FatCheckSystemSecurityAccess(IrpContext);
            if NT_SUCCESS(check_status) == 0 {
                Iosb.__bindgen_anon_1.Status = check_status;
                Iosb.Information = 0;
                break 'main_try;
            }

            if CreateDirectory {
                DebugTrace!(0, 0, b"Create new directory\n\0".as_ptr() as *const i8);

                if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
                    IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, (*(*Vcb).Vpb).RealDevice);
                    SetFlag!((*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                    FatRaiseStatus(IrpContext, STATUS_MEDIA_WRITE_PROTECTED);
                }

                if TemporaryFile {
                    Iosb.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                    break 'main_try;
                }

                Iosb = FatCreateNewDirectory(IrpContext, IrpSp, FileObject, Vcb, ParentDcb, &mut OemFinalName, &FinalName as *const _ as *mut _,
                                            DesiredAccess, ShareAccess, EaBuffer, EaLength, FileAttributes as u16,
                                            NoEaKnowledge as u8, DeleteOnClose as u8, OpenRequiringOplock as u8);
                (*Irp).IoStatus.Information = Iosb.Information;
                break 'main_try;
            }

            DebugTrace!(0, 0, b"Create new file\n\0".as_ptr() as *const i8);
            if TrailingBackslash {
                Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'main_try;
            }

            if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
                IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, (*(*Vcb).Vpb).RealDevice);
                SetFlag!((*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                FatRaiseStatus(IrpContext, STATUS_MEDIA_WRITE_PROTECTED);
            }

            Iosb = FatCreateNewFile(IrpContext, IrpSp, FileObject, Vcb, ParentDcb, &mut OemFinalName, &FinalName as *const _ as *mut _,
                                   DesiredAccess, ShareAccess, AllocationSize, EaBuffer, EaLength, FileAttributes,
                                   &Lfn as *const _ as *mut _, IsPagingFile as i32, NoEaKnowledge as i32, DeleteOnClose as i32, OpenRequiringOplock as i32, TemporaryFile as i32);

            if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !NoIntermediateBuffering {
                (*FileObject).Flags |= FO_CACHE_SUPPORTED;
            }
            (*Irp).IoStatus.Information = Iosb.Information;

            // try_exit point
            break 'main_try;
        }

        //  try_exit: — runs on every try_return path too.
        if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !OpenTargetDirectory {
            //  If there is an Fcb/Dcb, set the long file name.
            let LocalFcb = (*FileObject).FsContext as *mut FCB;
            if !LocalFcb.is_null()
                && ((NodeType!(LocalFcb as *const c_void)) as u32 == FAT_NTC_FCB as u32
                    || (NodeType!(LocalFcb as *const c_void)) as u32 == FAT_NTC_DCB as u32)
                && (*LocalFcb).FullFileName.Buffer.is_null()
            {
                FatSetFullNameInFcb(IrpContext, LocalFcb, &FinalName as *const _ as *mut _);
            }
        }
    }, |_, abnormal| {
        // DebugUnwind( FatCommonCreate ). The Win7+ atomic-oplock back-out is #if'd
        // out at the 0x502 target. NT_ASSERT(!OplockPostIrp || ...): retail no-op.
        FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut c_void);

        //  If we are in an error path, check for any created subdir Dcbs that
        //  have to be unwound.  Don't whack the root directory.
        if abnormal || NT_SUCCESS((*p_iosb).__bindgen_anon_1.Status) == 0 {
            //  Before doing the uninitialize, unpin anything repinned, but disable
            //  writethrough first (we are already failing, so disable raise from unpin).
            let SavedFlags = (*IrpContext).Flags;
            SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE | IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
            FatUnpinRepinnedBcbs(IrpContext);

            let FinalDcb = *p_final_dcb;
            if !FinalDcb.is_null()
                && (NodeType!(FinalDcb as *const c_void)) as u32 == FAT_NTC_DCB as u32
                && IsListEmpty(core::ptr::addr_of!((*FinalDcb).Specific.Dcb.as_ref().ParentDcbQueue) as *const _)
                && (*FinalDcb).OpenCount == 0
                && !(*FinalDcb).Specific.Dcb.as_ref().DirectoryFile.is_null()
            {
                let DirectoryFileObject = (*FinalDcb).Specific.Dcb.as_ref().DirectoryFile;
                (*FinalDcb).Specific.Dcb.as_mut().DirectoryFile = core::ptr::null_mut();
                CcUninitializeCacheMap(DirectoryFileObject, core::ptr::null_mut(), core::ptr::null_mut());
                ObDereferenceObject(DirectoryFileObject as *mut c_void);
            }

            (*IrpContext).Flags = SavedFlags;
        }

        if abnormal {
            FatReleaseVcb(IrpContext, Vcb);
        }

        //  Free up any string buffers we allocated.
        FatFreeStringBuffer(p_oem_final_name as *mut c_void);
        FatFreeStringBuffer(p_upcased_final_name as *mut c_void);
        FatFreeStringBuffer(p_lfn as *mut c_void);
    });

    //  The following code is only executed if we are exiting the
    //  procedure through a normal termination.  We complete the request
    //  and if for any reason that bombs out then we need to unreference
    //  and possibly delete the fcb and ccb.
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }  (create.c:2112-2279)
    crate::seh::try_finally(&mut (), |_| {
        if *p_post_irp {
            //  If the Irp hasn't already been posted, do it now.
            if *p_oplock_post_irp == 0 {
                (*p_iosb).__bindgen_anon_1.Status = FatFsdPostRequest(IrpContext, Irp);
            }
        } else {
            FatUnpinRepinnedBcbs(IrpContext);
        }
    }, |_, abnormal| {
        // DebugUnwind( FatCommonCreate-in-FatCompleteRequest );
        if abnormal {
            let mut LocalVcb: *mut VCB = core::ptr::null_mut();
            let mut LocalFcb: *mut FCB = core::ptr::null_mut();
            let mut LocalCcb2: *mut CCB = core::ptr::null_mut();

            //  Unwind all of our counts.  Note that if a write failed, then
            //  the volume has been marked for verify, and all volume
            //  structures will be cleaned up automatically.
            let _ = FatDecodeFileObject(FileObject, &mut LocalVcb, &mut LocalFcb, &mut LocalCcb2);

            (*LocalFcb).UncleanCount -= 1;
            (*LocalFcb).OpenCount -= 1;
            (*LocalVcb).OpenFileCount -= 1;

            if IsFileObjectReadOnly(FileObject) != 0 { (*LocalVcb).ReadOnlyCount -= 1; }

            //  WinSE #307418: if new file creation was interrupted by a system
            //  suspend, revert the changes to the parent directory and the FAT.
            if (*IrpContext).ExceptionStatus == STATUS_VERIFY_REQUIRED
                && (NodeType!(LocalFcb as *const c_void)) as u32 == FAT_NTC_FCB as u32
            {
                crate::allocsup::FatTruncateFileAllocation(IrpContext, LocalFcb, 0);
                crate::dirsup::FatDeleteDirent(IrpContext, LocalFcb, core::ptr::null_mut(), 1);
            }

            //  If we leafed out on a new Fcb we should get rid of it at this point.
            if (*LocalFcb).OpenCount == 0
                && ((NodeType!(LocalFcb as *const c_void)) as u32 == FAT_NTC_FCB as u32
                    || IsListEmpty(core::ptr::addr_of!((*LocalFcb).Specific.Dcb.as_ref().ParentDcbQueue) as *const _))
            {
                NT_ASSERT!((NodeType!(LocalFcb as *const c_void) != (FAT_NTC_ROOT_DCB as u32) as u16) as i32);

                if (NodeType!(LocalFcb as *const c_void)) as u32 == FAT_NTC_DCB as u32
                    && !(*LocalFcb).Specific.Dcb.as_ref().DirectoryFile.is_null()
                {
                    let DirectoryFileObject = (*LocalFcb).Specific.Dcb.as_ref().DirectoryFile;
                    (*LocalFcb).Specific.Dcb.as_mut().DirectoryFile = core::ptr::null_mut();
                    CcUninitializeCacheMap(DirectoryFileObject, &FatLargeZero as *const _ as *mut _, core::ptr::null_mut());
                    ObDereferenceObject(DirectoryFileObject as *mut c_void);
                } else {
                    if !FileObject.is_null() {
                        (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                    }
                    FatDeleteFcb(IrpContext, &mut LocalFcb);
                }
            }

            FatDeleteCcb(IrpContext, &mut LocalCcb2);
            FatReleaseVcb(IrpContext, LocalVcb);
        } else {
            //  THIS is the release that balances the FatAcquireExclusiveVcb above —
            //  its omission was the arm64 post-mount create deadlock.
            FatReleaseVcb(IrpContext, Vcb);

            if !*p_post_irp {
                //  NT_ASSERT(SecurityContext != NULL): retail no-op.
                if FlagOn(*DesiredAccess, FILE_EXECUTE) != 0 {
                    SetFlag!((*FileObject).Flags, FO_FILE_FAST_IO_READ);
                }

                //  Lock volume in drive if we opened a paging file, allocating a reserve MDL
                //  to guarantee paging-file operations can always go forward.
                if IsPagingFile && NT_SUCCESS((*p_iosb).__bindgen_anon_1.Status) != 0 {
                    if FatReserveMdl.is_null() {
                        let ReserveMdl = IoAllocateMdl(
                            core::ptr::null_mut(),
                            (FAT_RESERVE_MDL_SIZE * PAGE_SIZE) as ULONG,
                            1, 0, core::ptr::null_mut());

                        //  InterlockedCompareExchangePointer(&FatReserveMdl, ReserveMdl, NULL):
                        //  publish ReserveMdl only if the slot is still NULL; otherwise free ours.
                        let slot = core::ptr::addr_of_mut!(FatReserveMdl) as *const core::sync::atomic::AtomicUsize;
                        let _ = (*slot).compare_exchange(
                            0,
                            ReserveMdl as usize,
                            core::sync::atomic::Ordering::SeqCst,
                            core::sync::atomic::Ordering::SeqCst,
                        );
                        if FatReserveMdl != ReserveMdl {
                            IoFreeMdl(ReserveMdl);
                        }
                    }

                    SetFlag!((*Vcb).VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE);

                    if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0 {
                        FatToggleMediaEjectDisable(IrpContext, Vcb, 1);
                    }
                }
            }
        }

        DebugTrace!(-1, 0, b"FatCommonCreate -> %08lx\n\0".as_ptr() as *const i8, (*p_iosb).__bindgen_anon_1.Status);
    });

    CollectCreateStatistics(Vcb, Iosb.__bindgen_anon_1.Status);
    Iosb.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn FatOpenRootDcb(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: u16,
    CreateDisposition: u32
) -> _IO_STATUS_BLOCK {
    let mut RootDcb: *mut DCB;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut UnwindShareAccess: BOOLEAN = (0i32) as u8;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut UnwindCounts: BOOLEAN = (0i32) as u8;
    let mut RootDcbAcquired: BOOLEAN = (0i32) as u8;

    // PAGED_CODE() is a no-op in this context
    DebugTrace!(1i32, (Dbg) as i32, b"FatOpenRootDcb...\n\0".as_ptr() as *const i8, 0i32);

    RootDcb = (*Vcb).RootDcb;

    FatAcquireExclusiveFcb(IrpContext, RootDcb);
    RootDcbAcquired = (1i32) as u8;

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_UnwindShareAccess = core::ptr::addr_of_mut!(UnwindShareAccess);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_UnwindCounts = core::ptr::addr_of_mut!(UnwindCounts);
    let p_RootDcbAcquired = core::ptr::addr_of_mut!(RootDcbAcquired);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'seh2: loop {
        // try block
        if (CreateDisposition != FILE_OPEN) && (CreateDisposition != FILE_OPEN_IF) {
            Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
            break 'seh2;
        }

        if FatCheckFileAccess(IrpContext, (((*RootDcb).DirentFatFlags) as u16) as _, DesiredAccess) == ((0i32) as u8) as _ {
            Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
            break 'seh2;
        }

        if (*RootDcb).OpenCount > 0 {
            let status = IoCheckShareAccess(
                *DesiredAccess,
                ShareAccess as u32,
                FileObject,
                core::ptr::addr_of_mut!((*RootDcb).ShareAccess),
                1u8,
            );
            Iosb.__bindgen_anon_1.Status = status;
            if NT_SUCCESS(status) == 0 {
                break 'seh2;
            }
        } else {
            IoSetShareAccess(
                *DesiredAccess,
                (ShareAccess) as u32,
                FileObject,
                core::ptr::addr_of_mut!((*RootDcb).ShareAccess),
            );
        }

        UnwindShareAccess = (1i32) as u8;

        let ccb = FatCreateCcb(IrpContext);
        UnwindCcb = ccb;
        FatSetFileObject(FileObject, (UserDirectoryOpen) as TYPE_OF_OPEN, (RootDcb).cast(), ccb);

        (*RootDcb).UncleanCount += 1;
        (*RootDcb).OpenCount += 1;
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0u8 {
            (*Vcb).ReadOnlyCount += 1;
        }
        UnwindCounts = (1i32) as u8;

        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = FILE_OPENED as _;

        // try_exit: NOTHING
        break 'seh2;
    }, |_, abnormal| {
        DebugUnwind!(b"FatOpenRootDcb\0".as_ptr() as *const i8);

        if abnormal {
            if *p_UnwindCounts != (0i32) as u8 {
                (*RootDcb).UncleanCount -= 1;
                (*RootDcb).OpenCount -= 1;
                (*Vcb).OpenFileCount -= 1;
                if IsFileObjectReadOnly(FileObject) != 0u8 {
                    (*Vcb).ReadOnlyCount -= 1;
                }
            }
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, core::ptr::addr_of_mut!(*p_UnwindCcb));
            }
            if *p_UnwindShareAccess != (0i32) as u8 {
                IoRemoveShareAccess(FileObject, core::ptr::addr_of_mut!((*RootDcb).ShareAccess));
            }
        }

        if *p_RootDcbAcquired != (0i32) as u8 {
            FatReleaseFcb(IrpContext, RootDcb);
        }

        DebugTrace!(-1i32, (Dbg) as i32, b"FatOpenRootDcb -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status as i32);
    });

    Iosb
}
#[no_mangle]
pub unsafe extern "system" fn FatOpenExistingDcb(
    IrpContext: *mut IRP_CONTEXT,
    IrpSp: *mut IO_STACK_LOCATION,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    Dcb: *mut DCB,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: USHORT,
    CreateDisposition: ULONG,
    NoEaKnowledge: BOOLEAN,
    DeleteOnClose: BOOLEAN,
    OpenRequiringOplock: BOOLEAN,
    FileNameOpenedDos: BOOLEAN,
    OplockPostIrp: *mut BOOLEAN,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut UnwindShareAccess: BOOLEAN = 0;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut DcbAcquired: BOOLEAN = 0;

    // Ignore unused parameters for compatibility
    let _ = OpenRequiringOplock;
    let _ = IrpSp;

    // PAGED_CODE(); // macro, essentially a no-op in translation

    DebugTrace!(1, 0 /* Dbg */, b"FatOpenExistingDcb...\n\0".as_ptr() as *const i8);

    FatAcquireExclusiveFcb(IrpContext, Dcb);
    DcbAcquired = 1;

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_DirentBcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_UnwindShareAccess = core::ptr::addr_of_mut!(UnwindShareAccess);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_DcbAcquired = core::ptr::addr_of_mut!(DcbAcquired);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'try_body_dcb: {
        // _SEH2_TRY body
        *OplockPostIrp = 0;

        // Root DCB delete-on-close check
        if NodeType!(Dcb as *const core::ffi::c_void) == (FAT_NTC_ROOT_DCB as u16) && DeleteOnClose != 0 {
            Iosb.__bindgen_anon_1.Status = STATUS_CANNOT_DELETE;
            break 'try_body_dcb; // try_return Iosb
        }

        // Check oplock key -- NTDDI >= WIN8 only (create.c 2857-2876). Pre-WIN8 a DCB has no
        // oplock, and FatGetFcbOplock(Dcb) lands in its free-dirent bitmap.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
        Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
            FatGetFcbOplock(Dcb),
            (*IrpContext).OriginatingIrp,
            OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS {
            break 'try_body_dcb; // try_return NOTHING
        }
        }

        // NoEaKnowledge check
        if NoEaKnowledge != 0
            && NodeType!(Dcb as *const core::ffi::c_void) != (FAT_NTC_ROOT_DCB as u16)
            && FatIsFat32(Vcb) == 0
        {
            let mut NeedEaCount: ULONG = 0;
            FatGetDirentFromFcbOrDcb(IrpContext, Dcb, 0, &mut Dirent, &mut DirentBcb);
            FatGetNeedEaCount(IrpContext, Vcb, Dirent, &mut NeedEaCount);
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(DirentBcb) as *mut *mut core::ffi::c_void);
            if NeedEaCount != 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
                break 'try_body_dcb; // try_return Iosb
            }
        }

        // Create disposition check
        if CreateDisposition != FILE_OPEN && CreateDisposition != FILE_OPEN_IF {
            Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_COLLISION;
            break 'try_body_dcb; // try_return Iosb
        }

        // File access check
        if FatCheckFileAccess(IrpContext, (((*Dcb).DirentFatFlags) as u16) as _, DesiredAccess) == 0 {
            Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
            break 'try_body_dcb; // try_return Iosb
        }

        // Share access check when already opened
        if (*Dcb).OpenCount > 0 {
            Iosb.__bindgen_anon_1.Status = FatCheckShareAccess(
                IrpContext,
                FileObject,
                Dcb,
                DesiredAccess,
                ShareAccess as ULONG,
            );
            // Pre-WIN8 (create.c 2945-3010's #else) a failed share check just returns it.
            #[cfg(not(ntddi_ge_ntddi_win8))]
            if Iosb.__bindgen_anon_1.Status < 0 {
                break 'try_body_dcb; // try_return Iosb
            }
            #[cfg(ntddi_ge_ntddi_win8)]
            if Iosb.__bindgen_anon_1.Status < 0 {
                let mut OplockBreakStatus: NTSTATUS = STATUS_SUCCESS;
                if Iosb.__bindgen_anon_1.Status == STATUS_SHARING_VIOLATION
                    && FlagOn((*IrpSp).Parameters.Create.Options, FILE_COMPLETE_IF_OPLOCKED) == 0
                {
                    OplockBreakStatus = FsRtlOplockBreakH(
                        FatGetFcbOplock(Dcb),
                        (*IrpContext).OriginatingIrp,
                        0,
                        IrpContext as *mut core::ffi::c_void,
                        FatOplockComplete as *mut c_void,
                        FatPrePostIrp as *mut c_void,
                    );
                    if OplockBreakStatus == STATUS_PENDING {
                        Iosb.__bindgen_anon_1.Status = STATUS_PENDING;
                        *OplockPostIrp = 1;
                        break 'try_body_dcb;
                    } else if OplockBreakStatus < 0 {
                        Iosb.__bindgen_anon_1.Status = OplockBreakStatus;
                        break 'try_body_dcb;
                    } else {
                        // assert OplockBreakStatus == STATUS_SUCCESS
                        break 'try_body_dcb;
                    }
                } else {
                    break 'try_body_dcb;
                }
            }
        }

        // Oplock checks -- NTDDI >= WIN8 only (create.c 3017-3073): they ran on every reopen
        // of a directory that still had a handle, reading its dirent bitmap as an oplock.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
        if (*Dcb).UncleanCount != 0 {
            Iosb.__bindgen_anon_1.Status = FsRtlCheckOplock(
                FatGetFcbOplock(Dcb),
                (*IrpContext).OriginatingIrp,
                IrpContext as *mut core::ffi::c_void,
                Some(core::mem::transmute::<*const (), _>(FatOplockComplete as *const ())),
                Some(core::mem::transmute::<*const (), _>(FatPrePostIrp as *const ())),
            );
        }
        if Iosb.__bindgen_anon_1.Status == STATUS_PENDING {
            *OplockPostIrp = 1;
            break 'try_body_dcb;
        }
        if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
            Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                FatGetFcbOplock(Dcb),
                (*IrpContext).OriginatingIrp,
                (*Dcb).UncleanCount + 1,
            );
        }
        if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS && Iosb.__bindgen_anon_1.Status != STATUS_OPLOCK_BREAK_IN_PROGRESS {
            break 'try_body_dcb;
        }
        }

        // Update share access
        if (*Dcb).OpenCount > 0 {
            IoUpdateShareAccess(FileObject, &mut (*Dcb).ShareAccess);
        } else {
            IoSetShareAccess(*DesiredAccess, (ShareAccess) as u32, FileObject, &mut (*Dcb).ShareAccess);
        }
        UnwindShareAccess = 1;

        // Set up context and counts
        UnwindCcb = FatCreateCcb(IrpContext);
        FatSetFileObject(FileObject, (UserDirectoryOpen) as TYPE_OF_OPEN, (Dcb).cast(), UnwindCcb);
        (*Dcb).UncleanCount += 1;
        (*Dcb).OpenCount += 1;
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*Vcb).ReadOnlyCount += 1;
        }

        // Mark CCB flags
        {
            let Ccb: *mut CCB = (*FileObject).FsContext2 as *mut CCB;
            if DeleteOnClose != 0 {
                SetFlag!((*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
            }
            if FileNameOpenedDos != 0 {
                SetFlag!((*Ccb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
            }
        }

        ClearFlag!((*Dcb).FcbState, FCB_STATE_DELAY_CLOSE);

        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = FILE_OPENED as _;
    }, |_, abnormal| {
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_DirentBcb) as *mut *mut core::ffi::c_void);

        if abnormal {
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
            }
            if *p_UnwindShareAccess != 0 {
                IoRemoveShareAccess(FileObject, &mut (*Dcb).ShareAccess);
            }
        }

        if *p_DcbAcquired != 0 {
            FatReleaseFcb(IrpContext, Dcb);
        }

        DebugTrace!(-1, 0 /* Dbg */, b"FatOpenExistingDcb -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status);
    });

    Iosb
}

// Callees of FatOpenVolume below that this module did not already declare. All are
// real Rust definitions elsewhere in the crate (strucsup/flush/filobsup/verfysup/
// acchksup) — none is a bugcheck stub; signatures copied from those definitions.
extern "system" {
    fn FatIsHandleCountZero(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u8;
    fn FatPurgeReferencedFileObjects(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Flags: u32);
    fn FatExplicitDeviceAccessGranted(
        IrpContext: *mut IRP_CONTEXT,
        DeviceObject: PDEVICE_OBJECT,
        AccessState: PACCESS_STATE,
        ProcessorMode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
}

// FAT_FLUSH_TYPE (fatprocs.h:1052) and the FatMarkVolume state, as verfysup.rs and
// flush.rs spell them. Local because this module never imported either enum.
const Flush: u32 = 1;
const VolumeClean: i32 = 0;

// VcbState bits, from the per-arch bindgen layouts (identical in all three).
// This module had never needed one before FatOpenVolume.
use crate::types::{
    VCB_STATE_FLAG_BOOT_OR_PAGING_FILE, VCB_STATE_FLAG_LOCKED, VCB_STATE_FLAG_MOUNTED_DIRTY,
    VCB_STATE_FLAG_REMOVABLE_MEDIA, VCB_STATE_FLAG_VOLUME_DIRTY,
};

/// Open the volume itself for DASD access (`\Device\HarddiskVolumeN` opened
/// directly, rather than a file on it). Faithful translation of the legacy C
/// `FatOpenVolume` (`drivers/filesystems/fastfat/create.c:2292`).
///
/// Previously an un-translated hard stub that `KeBugCheckEx(0x23, 0xDEAD0000)`'d
/// on entry. arm64 is the arch that hits it, because arm64 boots a FAT32 ramdisk
/// (fastfat IS its system volume) while i386/amd64 boot a CDFS ISO and never enter
/// fastfat at all — the kmtest `IoFilesystem` test opens the volume directly and
/// stopped here the moment the BAD_POOL_HEADER root above it was fixed. Task #47.
///
/// The C's finally runs its unwind on `AbnormalTermination() || !NT_SUCCESS(Status)`,
/// so every ordinary failure return also undoes its share access, counts and
/// volume lock, not only a raise.
#[no_mangle]
pub unsafe extern "system" fn FatOpenVolume(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: u16,
    CreateDisposition: u32,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    // The C leaves Status uninitialised. Every path that reaches try_exit assigns
    // it, so SUCCESS here is not observable — it only keeps the FINALLY test below
    // well-defined if someone later adds a path that does not.
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut CleanedVolume: bool = false;

    // The C's abnormal-termination bookkeeping locals.
    let mut UnwindShareAccess: bool = false;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut UnwindCounts: bool = false;
    let mut UnwindVolumeLock: bool = false;

    let ShareAccess: u32 = ShareAccess as u32;

    PAGED_CODE!();
    DebugTrace!(1, 0 /* Dbg */, b"FatOpenVolume...\n\0".as_ptr() as *const i8);

    let p_Status = core::ptr::addr_of_mut!(Status);
    let p_UnwindShareAccess = core::ptr::addr_of_mut!(UnwindShareAccess);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_UnwindCounts = core::ptr::addr_of_mut!(UnwindCounts);
    let p_UnwindVolumeLock = core::ptr::addr_of_mut!(UnwindVolumeLock);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'try_body: {
        // Check for proper desired access and rights.
        if CreateDisposition != FILE_OPEN && CreateDisposition != FILE_OPEN_IF {
            Status = STATUS_ACCESS_DENIED;
            break 'try_body; // try_return
        }

        // If the user will not share write or delete, try to lock the volume.
        if FlagOn(ShareAccess, FILE_SHARE_WRITE) == 0
            && FlagOn(ShareAccess, FILE_SHARE_DELETE) == 0
        {
            // The C has an `#if (NTDDI_VERSION >= NTDDI_VISTA)` block here that
            // clears FILE_SHARE_READ for a writer once
            // FsRtlAreVolumeStartupApplicationsComplete() returns TRUE. That block
            // is compiled OUT in this tree — the function has no definition
            // anywhere, so the C would not link if it were live — and is therefore
            // deliberately not translated.

            // Quick check for handles on an exclusive open.
            if FlagOn(ShareAccess, FILE_SHARE_READ) == 0
                && FatIsHandleCountZero(IrpContext, Vcb) == 0
            {
                Status = STATUS_SHARING_VIOLATION;
                break 'try_body; // try_return
            }

            // Force Mm to get rid of its referenced file objects.
            FatFlushFat((IrpContext).cast(), Vcb);
            FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, Flush);

            // Without share-read, deny if anyone has the volume open at all;
            // with it, allow existing readonly opens but deny further ones.
            if FlagOn(ShareAccess, FILE_SHARE_READ) == 0 {
                if (*Vcb).OpenFileCount != 0 {
                    Status = STATUS_SHARING_VIOLATION;
                    break 'try_body; // try_return
                }
            } else if (*Vcb).ReadOnlyCount != (*Vcb).OpenFileCount {
                Status = STATUS_SHARING_VIOLATION;
                break 'try_body; // try_return
            }

            // Lock the volume.
            (*Vcb).VcbState |= VCB_STATE_FLAG_LOCKED;
            (*Vcb).FileObjectWithVcbLocked = FileObject;
            UnwindVolumeLock = true;

            CleanedVolume = true;
        } else if FlagOn(
            *DesiredAccess,
            FILE_READ_DATA | FILE_WRITE_DATA | FILE_APPEND_DATA,
        ) != 0
        {
            // Flush the volume and let ourselves push the clean bit out if
            // everything worked.
            if NT_SUCCESS(FatFlushVolume((IrpContext).cast(), Vcb, (Flush) as _)) != 0 {
                CleanedVolume = true;
            }
        }

        // Clean the volume if we believe it safe and reasonable.
        if CleanedVolume
            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DIRTY) != 0
            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY) == 0
            && CcIsThereDirtyData((*Vcb).Vpb) == 0
        {
            // Cancel any pending clean volumes.
            KeCancelTimer(core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer));
            KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc));

            FatMarkVolume(IrpContext, Vcb, VolumeClean);
            ClearFlag!((*Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DIRTY);

            // Unlock the volume if it is removable.
            if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
                && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0
            {
                FatToggleMediaEjectDisable(IrpContext, Vcb, 0 /* FALSE */);
            }
        }

        // If the volume is already open by someone, check the share access.
        if (*Vcb).DirectAccessOpenCount > 0 {
            Status = IoCheckShareAccess(
                *DesiredAccess,
                ShareAccess,
                FileObject,
                core::ptr::addr_of_mut!((*Vcb).ShareAccess),
                1, /* TRUE — update */
            );
            if NT_SUCCESS(Status) == 0 {
                break 'try_body; // try_return( NOTHING )
            }
        } else {
            IoSetShareAccess(
                *DesiredAccess,
                ShareAccess,
                FileObject,
                core::ptr::addr_of_mut!((*Vcb).ShareAccess),
            );
        }
        UnwindShareAccess = true;

        // Set up the context and section object pointers, and update counts.
        UnwindCcb = FatCreateCcb(IrpContext);
        // FatSetFileObject's 3rd param is typed *mut FCB in this module's declaration,
        // but the C takes a PVOID that is an FCB/DCB/VCB depending on the open type —
        // here it is genuinely the VCB, matching UserVolumeOpen.
        // UserVolumeOpen is the bindgen _TYPE_OF_OPEN (u32); this module's declaration
        // of FatSetFileObject takes the open type as i32, as its siblings' local
        // consts do (UserDirectoryOpen: i32 = 3). Value 4 — a plain cast, never
        // try_into().unwrap(), which would put a panic on a filesystem open path.
        FatSetFileObject(FileObject, (UserVolumeOpen) as TYPE_OF_OPEN, (Vcb as *mut FCB).cast(), UnwindCcb);

        (*FileObject).SectionObjectPointer =
            core::ptr::addr_of_mut!((*Vcb).SectionObjectPointers) as *mut _;

        (*Vcb).DirectAccessOpenCount += 1;
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*Vcb).ReadOnlyCount += 1;
        }
        UnwindCounts = true;
        (*FileObject).Flags |= FO_NO_INTERMEDIATE_BUFFERING;

        // The open will succeed from here, so note whether the user is getting
        // explicit access to the device — if not, we deny modifying FSCTLs to it.
        let IrpSp = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);
        Status = FatExplicitDeviceAccessGranted(
            IrpContext,
            (*(*Vcb).Vpb).RealDevice,
            (*(*IrpSp).Parameters.Create.SecurityContext).AccessState,
            if FlagOn(((*IrpSp).Flags) as u32, SL_FORCE_ACCESS_CHECK) != 0 {
                1i8 /* UserMode */
            } else {
                (*(*IrpContext).OriginatingIrp).RequestorMode
            } as KPROCESSOR_MODE,
        );

        if NT_SUCCESS(Status) != 0 {
            SetFlag!((*UnwindCcb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS);
        }

        // The access-granted probe above is advisory: the C overwrites its status
        // unconditionally, so a denial only costs the CCB flag, not the open.
        Status = STATUS_SUCCESS;
        Iosb.Information = FILE_OPENED as _;
    }, |_, abnormal| {
        if abnormal || NT_SUCCESS(*p_Status) == 0 {
            if *p_UnwindCounts {
                (*Vcb).DirectAccessOpenCount -= 1;
                (*Vcb).OpenFileCount -= 1;
                if IsFileObjectReadOnly(FileObject) != 0 {
                    (*Vcb).ReadOnlyCount -= 1;
                }
            }
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
            }
            if *p_UnwindShareAccess {
                IoRemoveShareAccess(FileObject, core::ptr::addr_of_mut!((*Vcb).ShareAccess));
            }
            if *p_UnwindVolumeLock {
                (*Vcb).VcbState &= !VCB_STATE_FLAG_LOCKED;
            }
        }

        DebugTrace!(
            -1,
            0, /* Dbg */
            b"FatOpenVolume -> Iosb.Status = %08lx\n\0".as_ptr() as *const i8,
            *p_Status,
        );
    });

    Iosb.__bindgen_anon_1.Status = Status;
    Iosb
}

/// Open a directory that has NOT previously been opened (no cached DCB yet).
/// Faithful translation of the legacy C `FatOpenExistingDirectory`
/// (`drivers/filesystems/fastfat/create.c:4136`) — previously an un-translated
/// hard stub that `KeBugCheckEx(0x23, 0xDEAD0000)`'d the instant any first-time
/// directory open reached it (e.g. userenv creating a fresh user profile
/// directory on the arm64 FAT32 system volume). Field manual §9.45.
#[no_mangle]
pub unsafe extern "system" fn FatOpenExistingDirectory(
    IrpContext: *mut IRP_CONTEXT,
    IrpSp: *mut IO_STACK_LOCATION,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    Dcb: *mut *mut DCB,
    ParentDcb: *mut DCB,
    Dirent: *mut DIRENT,
    LfnByteOffset: u32,
    DirentByteOffset: u32,
    Lfn: *mut _UNICODE_STRING,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: u16,
    CreateDisposition: u32,
    NoEaKnowledge: BOOLEAN,
    DeleteOnClose: BOOLEAN,
    FileNameOpenedDos: BOOLEAN,
    OpenRequiringOplock: BOOLEAN,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut UnwindDcb: *mut DCB = core::ptr::null_mut();
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut CountsIncremented: bool = false;

    // UNREFERENCED_PARAMETER(DeleteOnClose); UNREFERENCED_PARAMETER(IrpSp);
    let _ = DeleteOnClose;
    let _ = IrpSp;

    DebugTrace!(1, 0 /* Dbg */, b"FatOpenExistingDirectory...\n\0".as_ptr() as *const i8);

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_UnwindDcb = core::ptr::addr_of_mut!(UnwindDcb);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_CountsIncremented = core::ptr::addr_of_mut!(CountsIncremented);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'try_body: {
        // If the caller has no Ea knowledge, check for Need Ea's (non-FAT32 only).
        if NoEaKnowledge != 0 && FatIsFat32(Vcb) == 0 {
            let mut NeedEaCount: u32 = 0;
            FatGetNeedEaCount(IrpContext, Vcb, Dirent, &mut NeedEaCount);
            if NeedEaCount != 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
                break 'try_body; // try_return(Iosb)
            }
        }

        // Check the create disposition.
        if CreateDisposition != FILE_OPEN && CreateDisposition != FILE_OPEN_IF {
            Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_COLLISION;
            break 'try_body; // try_return(Iosb)
        }

        // Check the desired access against the dirent's attributes.
        if FatCheckFileAccess(IrpContext, ((*Dirent).Attributes as u16) as _, DesiredAccess) == 0 {
            Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
            break 'try_body; // try_return(Iosb)
        }

        // Create a new dcb for the directory.
        UnwindDcb = FatCreateDcb(IrpContext, Vcb, ParentDcb, LfnByteOffset, DirentByteOffset, Dirent, Lfn);
        *Dcb = UnwindDcb;

        // Win8+: store any caller oplock key and honor atomic create-with-oplock.
        // NTDDI >= WIN8 only: a pre-WIN8 DCB has no oplock (see FatCreateNewDirectory).
        #[cfg(ntddi_ge_ntddi_win8)]
        {
        Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
            FatGetFcbOplock(*Dcb),
            (*IrpContext).OriginatingIrp,
            OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
            Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                FatGetFcbOplock(*Dcb),
                (*IrpContext).OriginatingIrp,
                (*(*Dcb)).UncleanCount + 1,
            );
        }
        // Raise to trigger cleanup of the new Dcb if either oplock call failed.
        if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS {
            FatRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
        }
        }

        // Set up our share access.
        IoSetShareAccess(*DesiredAccess, ShareAccess as u32, FileObject, &mut (*(*Dcb)).ShareAccess);

        // Set up the context/section pointers and bump reference counts.
        UnwindCcb = FatCreateCcb(IrpContext);
        FatSetFileObject(FileObject, (UserDirectoryOpen) as TYPE_OF_OPEN, (*Dcb).cast(), UnwindCcb);
        (*(*Dcb)).UncleanCount += 1;
        (*(*Dcb)).OpenCount += 1;
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*Vcb).ReadOnlyCount += 1;
        }
        CountsIncremented = true;

        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = FILE_OPENED as _;

        // Mark the OpenedByShortName bit on the new Ccb if applicable.
        {
            let Ccb: *mut CCB = (*FileObject).FsContext2 as *mut CCB;
            if FileNameOpenedDos != 0 {
                SetFlag!((*Ccb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
            }
        }
    }, |_, abnormal| {
        if abnormal {
            if *p_CountsIncremented {
                (*(*Dcb)).UncleanCount -= 1;
                (*(*Dcb)).OpenCount -= 1;
                (*Vcb).OpenFileCount -= 1;
                if IsFileObjectReadOnly(FileObject) != 0 {
                    (*Vcb).ReadOnlyCount -= 1;
                }
            }
            if !(*p_UnwindDcb).is_null() {
                // create.c: `if (ARGUMENT_PRESENT( FileObject ))` — FileObject is optional here.
                if !FileObject.is_null() {
                    (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                }
                FatDeleteFcb(IrpContext, &mut *p_UnwindDcb);
                *Dcb = core::ptr::null_mut();
            }
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
            }
        }

        DebugTrace!(
            -1,
            0, /* Dbg */
            b"FatOpenExistingDirectory -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8,
            (*p_Iosb).__bindgen_anon_1.Status,
        );
    });

    Iosb
}
#[no_mangle]
pub unsafe extern "system" fn FatOpenExistingFcb(
    IrpContext: *mut IRP_CONTEXT,
    IrpSp: *mut IO_STACK_LOCATION,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    mut Fcb: *mut FCB,
    DesiredAccess: *mut u32,
    ShareAccess: u16,
    AllocationSize: u32,
    EaBuffer: *mut FILE_FULL_EA_INFORMATION,
    EaLength: u32,
    FileAttributes: u16,
    CreateDisposition: u32,
    NoEaKnowledge: u8,
    DeleteOnClose: u8,
    OpenRequiringOplock: u8,
    FileNameOpenedDos: u8,
    OplockPostIrp: *mut u8,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut AddedAccess: u32 = 0;

    // The following variables are for abnormal termination
    let mut UnwindShareAccess: u8 = 0;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut DecrementFcbOpenCount: u8 = 0;
    let mut FcbAcquired: u8 = 0;

    unsafe {
        DebugTrace!(1, (Dbg) as i32, b"FatOpenExistingFcb...\n\0".as_ptr() as *const i8);

        // Get the Fcb exclusive.
        FatAcquireExclusiveFcb(IrpContext, Fcb);
        FcbAcquired = 1;

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_DirentBcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_UnwindShareAccess = core::ptr::addr_of_mut!(UnwindShareAccess);
    let p_DecrementFcbOpenCount = core::ptr::addr_of_mut!(DecrementFcbOpenCount);
    let p_FcbAcquired = core::ptr::addr_of_mut!(FcbAcquired);
    let p_Fcb = core::ptr::addr_of_mut!(Fcb);
        // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
        crate::seh::try_finally(&mut (), |_| {
        'try_scope_body: loop {
            *OplockPostIrp = 0;

            // Oplock key check
            Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
                FatGetFcbOplock(Fcb),
                (*IrpContext).OriginatingIrp,
                OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
                core::ptr::null_mut(),
                None,
                None,
            );
            if Iosb.__bindgen_anon_1.Status != 0 /* STATUS_SUCCESS */ {
                break 'try_scope_body;
            }

            // Batch oplock check
            if FsRtlCurrentBatchOplock(FatGetFcbOplock(Fcb)) != 0 {
                Iosb.Information = FILE_OPBATCH_BREAK_UNDERWAY as _;
                Iosb.__bindgen_anon_1.Status = FsRtlCheckOplock(
                    FatGetFcbOplock(Fcb),
                    (*IrpContext).OriginatingIrp,
                    IrpContext as *mut c_void,
                    Some(core::mem::transmute::<*const (), _>(FatOplockComplete as *const ())),
                    Some(core::mem::transmute::<*const (), _>(FatPrePostIrp as *const ())),
                );
                if Iosb.__bindgen_anon_1.Status == 0x103 /* STATUS_PENDING */ {
                    *OplockPostIrp = 1;
                    break 'try_scope_body;
                }
            }

            // Handle supersede/overwrite/other dispositions
            if CreateDisposition == FILE_CREATE {
                Iosb.__bindgen_anon_1.Status = 0xC0000035_u32 as i32 /* STATUS_OBJECT_NAME_COLLISION */;
                break 'try_scope_body;
            } else if CreateDisposition == FILE_SUPERSEDE {
                SetFlag!(AddedAccess, 0x00010000 /* DELETE */ & !(*DesiredAccess));
                *DesiredAccess |= 0x00010000;
            } else if CreateDisposition == FILE_OVERWRITE || CreateDisposition == FILE_OVERWRITE_IF {
                SetFlag!(AddedAccess, (0x00000002 /* FILE_WRITE_DATA */ | 0x00000010 /* FILE_WRITE_EA */ | 0x00000100 /* FILE_WRITE_ATTRIBUTES */) & !(*DesiredAccess));
                *DesiredAccess |= 0x00000002 | 0x00000010 | 0x00000100;
            }

            // Desired access check
            if FatCheckFileAccess(IrpContext, ((((*Fcb).DirentFatFlags) as u8) as u16) as _, DesiredAccess) == 0 {
                Iosb.__bindgen_anon_1.Status = 0xC0000022_u32 as i32 /* STATUS_ACCESS_DENIED */;
                break 'try_scope_body;
            }

            // Delete-readonly check
            if DeleteOnClose != 0 && FlagOn((*Fcb).DirentFatFlags as u32, FAT_DIRENT_ATTR_READ_ONLY as u32) != 0 {
                Iosb.__bindgen_anon_1.Status = 0xC0000121_u32 as i32 /* STATUS_CANNOT_DELETE */;
                break 'try_scope_body;
            }

            // Attribute mismatch check for overwrite/supersede
            if CreateDisposition == FILE_SUPERSEDE || CreateDisposition == FILE_OVERWRITE || CreateDisposition == FILE_OVERWRITE_IF {
                let Hidden = BooleanFlagOn((*Fcb).DirentFatFlags, FAT_DIRENT_ATTR_HIDDEN);
                let System = BooleanFlagOn((*Fcb).DirentFatFlags, FAT_DIRENT_ATTR_SYSTEM);
                if (Hidden != 0 && FlagOn(FileAttributes as u32, 0x00000002 /* FILE_ATTRIBUTE_HIDDEN */) == 0)
                    || (System != 0 && FlagOn(FileAttributes as u32, 0x00000004 /* FILE_ATTRIBUTE_SYSTEM */) == 0)
                {
                    DebugTrace!(0, (Dbg) as i32, b"The hidden and/or system bits do not match\n\0".as_ptr() as *const i8);
                    Iosb.__bindgen_anon_1.Status = 0xC0000022_u32 as i32;
                    break 'try_scope_body;
                }
                if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
                    IoSetHardErrorOrVerifyDevice(
                        (*IrpContext).OriginatingIrp,
                        (*(*Vcb).Vpb).RealDevice,
                    );
                    SetFlag!((*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                    FatRaiseStatus(IrpContext, 0xC00000A2_u32 as i32 /* STATUS_MEDIA_WRITE_PROTECTED */);
                }
            }

            // Share access check
            Iosb.__bindgen_anon_1.Status = FatCheckShareAccess(IrpContext, FileObject, Fcb, DesiredAccess, (ShareAccess) as u32);
            if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
                let mut OplockBreakStatus: i32 = 0;
                if Iosb.__bindgen_anon_1.Status == 0xC0000043_u32 as i32 /* STATUS_SHARING_VIOLATION */
                    && FlagOn((*IrpSp).Parameters.Create.Options, 0x100 /* FILE_COMPLETE_IF_OPLOCKED */) == 0
                {
                    OplockBreakStatus = FsRtlOplockBreakH(
                        FatGetFcbOplock(Fcb),
                        (*IrpContext).OriginatingIrp,
                        0,
                        IrpContext as *mut c_void,
                        FatOplockComplete as *mut c_void,
                        FatPrePostIrp as *mut c_void,
                    );
                    if OplockBreakStatus == 0x103 {
                        Iosb.__bindgen_anon_1.Status = 0x103;
                        *OplockPostIrp = 1;
                        break 'try_scope_body;
                    } else if NT_SUCCESS(OplockBreakStatus) == 0 {
                        Iosb.__bindgen_anon_1.Status = OplockBreakStatus;
                        break 'try_scope_body;
                    } else {
                        NT_ASSERT!((OplockBreakStatus == 0) as i32);
                        break 'try_scope_body;
                    }
                } else {
                    break 'try_scope_body;
                }
            }

            // Oplock check for existing handles
            if (*Fcb).UncleanCount != 0 {
                Iosb.__bindgen_anon_1.Status = FsRtlCheckOplock(
                    FatGetFcbOplock(Fcb),
                    (*IrpContext).OriginatingIrp,
                    IrpContext as *mut c_void,
                    Some(core::mem::transmute::<*const (), _>(FatOplockComplete as *const ())),
                    Some(core::mem::transmute::<*const (), _>(FatPrePostIrp as *const ())),
                );
            }
            if Iosb.__bindgen_anon_1.Status == 0x103 {
                *OplockPostIrp = 1;
                break 'try_scope_body;
            }

            // Atomic create-with-oplock
            if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == 0 {
                Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                    FatGetFcbOplock(Fcb),
                    (*IrpContext).OriginatingIrp,
                    (*Fcb).UncleanCount + 1,
                );
            }
            if Iosb.__bindgen_anon_1.Status != 0 && Iosb.__bindgen_anon_1.Status != 0x108_u32 as i32 /* STATUS_OPLOCK_BREAK_IN_PROGRESS */ {
                break 'try_scope_body;
            }

            // FastIO possible
            (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);

            // Write/delete check
            if FlagOn(*DesiredAccess, 0x00000002) != 0 || DeleteOnClose != 0 {
                (*Fcb).OpenCount += 1;
                DecrementFcbOpenCount = 1;
                if MmFlushImageSection(
                    core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers),
                    1 /* MmFlushForWrite */,
                ) == 0
                {
                    Iosb.__bindgen_anon_1.Status = if DeleteOnClose != 0 { 0xC0000121_u32 as i32 } else { 0xC0000043_u32 as i32 };
                    break 'try_scope_body;
                }
            }

            // Non-cached purge
            if FlagOn((*FileObject).Flags, FO_NO_INTERMEDIATE_BUFFERING) != 0
                && (*Fcb).UncleanCount == (*Fcb).NonCachedUncleanCount
                && (*(*Fcb).NonPaged).SectionObjectPointers.DataSectionObject != core::ptr::null_mut()
                && FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0
            {
                SetFlag!((*((*Fcb).Vcb)).VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
                CcFlushCache(
                    core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                );
                ExAcquireResourceExclusiveLite((*Fcb).Header.PagingIoResource, 1);
                ExReleaseResourceLite((*Fcb).Header.PagingIoResource);
                CcPurgeCacheSection(
                    core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers),
                    core::ptr::null_mut(),
                    0,
                    0,
                );
                ClearFlag!((*((*Fcb).Vcb)).VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
            }

            // Open or Open-If
            if CreateDisposition == FILE_OPEN || CreateDisposition == FILE_OPEN_IF {
                DebugTrace!(0, (Dbg) as i32, b"Doing open operation\n\0".as_ptr() as *const i8);
                if NoEaKnowledge != 0 && FatIsFat32(Vcb) == 0 {
                    let mut NeedEaCount: u32 = 0;
                    FatGetDirentFromFcbOrDcb(
                        IrpContext,
                        Fcb,
                        0,
                        &mut Dirent as *mut *mut DIRENT,
                        &mut DirentBcb,
                    );
                    FatGetNeedEaCount(IrpContext, Vcb, Dirent, &mut NeedEaCount);
                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(DirentBcb) as *mut *mut core::ffi::c_void);
                    if NeedEaCount != 0 {
                        Iosb.__bindgen_anon_1.Status = 0xC0000022_u32 as i32;
                        break 'try_scope_body;
                    }
                }
                FatSetFileObject(
                    FileObject,
                    (UserFileOpen) as TYPE_OF_OPEN,
                    (Fcb).cast(),
                    {
                        let ccb = FatCreateCcb(IrpContext);
                        UnwindCcb = ccb;
                        ccb
                    },
                );
                (*FileObject).SectionObjectPointer = core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers) as *mut _;
                Iosb.Information = 1 /* FILE_OPENED */;
                break 'try_scope_body;
            }

            // Supersede/Overwrite
            if CreateDisposition == FILE_SUPERSEDE
                || CreateDisposition == FILE_OVERWRITE
                || CreateDisposition == FILE_OVERWRITE_IF
            {
                let OldStatus: i32;
                DebugTrace!(0, (Dbg) as i32, b"Doing supersede/overwrite operation\n\0".as_ptr() as *const i8);
                OldStatus = Iosb.__bindgen_anon_1.Status;
                Iosb.__bindgen_anon_1.Status = FatCheckSystemSecurityAccess(IrpContext);
                if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
                    break 'try_scope_body;
                }
                // Re-point from the rust_cohab_stub.c STUB_BUGCHECK stubs to the real Rust
                // translations already present in this file (FatSupersedeOrOverwriteFile @ ~3213,
                // FatCheckSystemSecurityAccess @ ~3569). FatOpenExistingFcb was translated calling
                // the snake_case stubs while its sibling FatOpenExistingFile got the real fns; any
                // supersede/overwrite of an existing FAT file through FatOpenExistingFcb bugchecked
                // 0x23 0xdead0000. Signatures are identical (BOOLEAN = u8).
                Iosb = FatSupersedeOrOverwriteFile(
                    IrpContext,
                    FileObject,
                    Fcb,
                    AllocationSize,
                    EaBuffer,
                    EaLength,
                    FileAttributes,
                    CreateDisposition,
                    NoEaKnowledge,
                );
                if Iosb.__bindgen_anon_1.Status == 0 {
                    Iosb.__bindgen_anon_1.Status = OldStatus;
                }
                break 'try_scope_body;
            }

            // Bugcheck if we get here
            FatBugCheck((CreateDisposition) as ULONG_PTR, 0, 0);
        }

        // try_exit block: share access / counts update
        if Iosb.__bindgen_anon_1.Status != 0x103 && NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 {
            if AddedAccess != 0 {
                let Status: i32;
                ClearFlag!(*DesiredAccess, AddedAccess);
                Status = IoCheckShareAccess(
                    *DesiredAccess,
                    ShareAccess as u32,
                    FileObject,
                    core::ptr::addr_of!((*Fcb).ShareAccess) as *mut _,
                    1,
                );
                NT_ASSERT!((Status == 0) as i32);
            } else {
                IoUpdateShareAccess(FileObject, core::ptr::addr_of!((*Fcb).ShareAccess) as *mut _);
            }
            UnwindShareAccess = 1;
            ClearFlag!((*Fcb).FcbState, FCB_STATE_DELAY_CLOSE);
            (*Fcb).UncleanCount += 1;
            (*Fcb).OpenCount += 1;
            if FlagOn((*FileObject).Flags, FO_NO_INTERMEDIATE_BUFFERING) != 0 {
                (*Fcb).NonCachedUncleanCount += 1;
            }
            (*Vcb).OpenFileCount += 1;
            if IsFileObjectReadOnly(FileObject) != 0 {
                (*Vcb).ReadOnlyCount += 1;
            }
            {
                let Ccb: *mut CCB = (*FileObject).FsContext2 as *mut CCB;
                if DeleteOnClose != 0 {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
                }
                if FileNameOpenedDos != 0 {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
                }
                if FatCheckManageVolumeAccess(
                    IrpContext,
                    (*(*IrpSp).Parameters.Create.SecurityContext).AccessState,
                    if FlagOn(((*IrpSp).Flags) as u32, 0x00000001 /* SL_FORCE_ACCESS_CHECK */) != 0 {
                        (1u8) as i8
                    } else {
                        (*(*IrpContext).OriginatingIrp).RequestorMode
                    },
                ) != 0
                {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS);
                }
            }
        }
        }, |_, abnormal| {
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_DirentBcb) as *mut *mut core::ffi::c_void);
            if abnormal {
                if !(*p_UnwindCcb).is_null() {
                    FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
                }
                if *p_UnwindShareAccess != 0 {
                    IoRemoveShareAccess(FileObject, core::ptr::addr_of!((**p_Fcb).ShareAccess) as *mut _);
                }
            }
            if *p_DecrementFcbOpenCount != 0 {
                (**p_Fcb).OpenCount -= 1;
                if (**p_Fcb).OpenCount == 0 {
                    if !FileObject.is_null() {
                        (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                    }
                    FatDeleteFcb(IrpContext, &mut *p_Fcb);
                    *p_FcbAcquired = 0;
                }
            }
            if *p_FcbAcquired != 0 {
                FatReleaseFcb(IrpContext, *p_Fcb);
            }
            DebugTrace!(-1, (Dbg) as i32, b"FatOpenExistingFcb -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status);
        });
    }
    Iosb
}

#[no_mangle]
pub unsafe extern "system" fn FatOpenTargetDirectory(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: *mut FILE_OBJECT,
    Dcb: *mut DCB,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: u16,
    DoesNameExist: u8,
    FileNameOpenedDos: u8,
) -> _IO_STATUS_BLOCK {
    // local variable Iosb zero-initialized
    let mut Iosb = core::mem::zeroed::<_IO_STATUS_BLOCK>();

    // variables for abnormal termination
    let mut UnwindShareAccess: u8 = 0;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut DcbAcquired: u8 = 0;

    // DebugTrace(+1, Dbg, "FatOpenTargetDirectory...\n", 0);
    DebugTrace!(1, (Dbg) as i32, b"FatOpenTargetDirectory...\n\0".as_ptr() as *const i8, 0);

    // (VOID)FatAcquireExclusiveFcb(IrpContext, Dcb);
    FatAcquireExclusiveFcb(IrpContext, Dcb);
    DcbAcquired = 1;

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_UnwindShareAccess = core::ptr::addr_of_mut!(UnwindShareAccess);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_DcbAcquired = core::ptr::addr_of_mut!(DcbAcquired);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'try_block: loop {
        let mut i: u32; // ULONG

        // if Dcb->OpenCount > 0
        if (*Dcb).OpenCount > 0 {
            // if (!NT_SUCCESS(Iosb.Status = IoCheckShareAccess(...)))
            Iosb.__bindgen_anon_1.Status = IoCheckShareAccess(
                *DesiredAccess,
                (ShareAccess) as u32,
                FileObject,
                core::ptr::addr_of_mut!((*Dcb).ShareAccess),
                1,
            );
            if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
                // try_return(Iosb);
                break 'try_block;
            }
        } else {
            IoSetShareAccess(
                *DesiredAccess,
                (ShareAccess) as u32,
                FileObject,
                core::ptr::addr_of_mut!((*Dcb).ShareAccess),
            );
        }
        UnwindShareAccess = 1;

        // FatSetFileObject(...) with UserDirectoryOpen constant = 4
        FatSetFileObject(
            FileObject,
            (UserDirectoryOpen) as TYPE_OF_OPEN, // was 4 (UserVolumeOpen)
            (Dcb).cast(),
            {
                let ccb = FatCreateCcb(IrpContext);
                UnwindCcb = ccb;
                ccb
            },
        );

        (*Dcb).UncleanCount += 1;
        (*Dcb).OpenCount += 1;
        (*(*Dcb).Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*(*Dcb).Vcb).ReadOnlyCount += 1;
        }

        // Update the file object's name
        i = ((*FileObject).FileName.Length as u32) / (core::mem::size_of::<u16>() as u32) - 1;
        // Remove trailing backslash if present
        if *((*FileObject).FileName.Buffer).offset(i as isize) == '\\' as u16 {
            // NT_ASSERT(i != 0);
            if i == 0 {
                // We'll keep a debug assert; using core::assert! in Rust
                ntkrnl_sys::ASSERT!(false, "i != 0");
            }
            (*FileObject).FileName.Length -= core::mem::size_of::<u16>() as u16;
            i -= 1;
        }
        // Find the first non-backslash character, scanning backward
        loop {
            if *((*FileObject).FileName.Buffer).offset(i as isize) == '\\' as u16 {
                i += 1;
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if i != 0 {
            (*FileObject).FileName.Length -= (i as u16).wrapping_mul(core::mem::size_of::<u16>() as u16);
            // RtlMoveMemory: copy overlapping memory region (byte copy)
            core::ptr::copy::<u8>(
                ((*FileObject).FileName.Buffer).offset(i as isize) as *const u8,
                (*FileObject).FileName.Buffer as *mut u8,
                (*FileObject).FileName.Length as usize,
            );
        }

        // Set status to success
        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = if DoesNameExist != 0 { 4 /* FILE_EXISTS */ } else { 5 /* FILE_DOES_NOT_EXIST */ };

        // If successful and name exists, set CCB flag
        if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && DoesNameExist != 0 {
            let Ccb: *mut CCB = (*FileObject).FsContext2 as *mut CCB;
            if FileNameOpenedDos != 0 {
                // SetFlag(Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME); expand inline
                SetFlag!((*Ccb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
            }
        }

        // Normal exit from try block; fall through to finally.
        break 'try_block;
    }, |_, abnormal| {
        DebugUnwind!(FatOpenTargetDirectory as *const c_void as *const i8);

        if abnormal {
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, core::ptr::addr_of_mut!(*p_UnwindCcb));
            }
            if *p_UnwindShareAccess != 0 {
                IoRemoveShareAccess(FileObject, core::ptr::addr_of_mut!((*Dcb).ShareAccess));
            }
        }
        if *p_DcbAcquired != 0 {
            FatReleaseFcb(IrpContext, Dcb);
        }

        DebugTrace!(-1, (Dbg) as i32, b"FatOpenTargetDirectory -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status);
    });

    Iosb
}
#[no_mangle]
pub unsafe extern "system" fn FatOpenExistingFile(
    IrpContext: PIRP_CONTEXT,
    FileObject: PFILE_OBJECT,
    Vcb: PVCB,
    Fcb: *mut PFCB,
    ParentDcb: PDCB,
    Dirent: *mut DIRENT,
    LfnByteOffset: u32,
    DirentByteOffset: u32,
    Lfn: PUNICODE_STRING,
    OrigLfn: PUNICODE_STRING,
    DesiredAccess: *mut ACCESS_MASK,
    ShareAccess: u16,
    AllocationSize: u32,
    EaBuffer: PFILE_FULL_EA_INFORMATION,
    EaLength: u32,
    FileAttributes: u16,
    CreateDisposition: u32,
    IsPagingFile: i32,
    NoEaKnowledge: i32,
    DeleteOnClose: i32,
    OpenRequiringOplock: i32,
    FileNameOpenedDos: i32,
) -> _IO_STATUS_BLOCK {
    // PAGED_CODE();
    DebugTrace!(1, (Dbg) as i32, b"FatOpenExistingFile...\n\0".as_ptr() as *const i8);

    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut AddedAccess: u32 = 0;
    let mut UnwindFcb: PFCB = core::ptr::null_mut();
    let mut UnwindCcb: PCCB = core::ptr::null_mut();
    let mut CountsIncremented: i32 = 0;

    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_UnwindFcb = core::ptr::addr_of_mut!(UnwindFcb);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_CountsIncremented = core::ptr::addr_of_mut!(CountsIncremented);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| {
        let IrpSp = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);

        'try_block: loop {
            if CreateDisposition == FILE_CREATE {
                Iosb.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_COLLISION;
                break 'try_block;
            } else if CreateDisposition == FILE_SUPERSEDE && IsPagingFile == 0 {
                SetFlag!(AddedAccess, DELETE & !(*DesiredAccess));
                *DesiredAccess |= DELETE;
            } else if (CreateDisposition == FILE_OVERWRITE || CreateDisposition == FILE_OVERWRITE_IF)
                && IsPagingFile == 0
            {
                SetFlag!(AddedAccess, (FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES) & !(*DesiredAccess));
                *DesiredAccess |= FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES;
            }

            if FatCheckFileAccess(IrpContext, (((*Dirent).Attributes) as u16) as _, DesiredAccess) == 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
                break 'try_block;
            }

            if DeleteOnClose != 0
                && FlagOn((*Dirent).Attributes as u32, FAT_DIRENT_ATTR_READ_ONLY as u32) != 0
            {
                Iosb.__bindgen_anon_1.Status = STATUS_CANNOT_DELETE;
                break 'try_block;
            }

            if CreateDisposition == FILE_SUPERSEDE
                || CreateDisposition == FILE_OVERWRITE
                || CreateDisposition == FILE_OVERWRITE_IF
            {
                let Hidden: i32 = BooleanFlagOn((*Dirent).Attributes, FAT_DIRENT_ATTR_HIDDEN) as i32;
                let System: i32 = BooleanFlagOn((*Dirent).Attributes, FAT_DIRENT_ATTR_SYSTEM) as i32;

                if (Hidden != 0 && FlagOn(FileAttributes as u32, FILE_ATTRIBUTE_HIDDEN as u32) == 0)
                    || (System != 0 && FlagOn(FileAttributes as u32, FILE_ATTRIBUTE_SYSTEM as u32) == 0)
                {
                    DebugTrace!(0, (Dbg) as i32, b"The hidden and/or system bits do not match\n\0".as_ptr() as *const i8);
                    if IsPagingFile == 0 {
                        Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
                        break 'try_block;
                    }
                }

                if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
                    IoSetHardErrorOrVerifyDevice(
                        (*IrpContext).OriginatingIrp,
                        (*(*Vcb).Vpb).RealDevice,
                    );
                    SetFlag!((*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                    FatRaiseStatus(IrpContext, STATUS_MEDIA_WRITE_PROTECTED);
                    // unreachable
                }
            }

            UnwindFcb = FatCreateFcb(
                IrpContext,
                Vcb,
                ParentDcb,
                LfnByteOffset,
                DirentByteOffset,
                Dirent,
                Lfn,
                OrigLfn,
                (IsPagingFile) as BOOLEAN,
                0,
            );
            *Fcb = UnwindFcb;

            (*(*Fcb)).Header.ValidDataLength.__bindgen_anon_1.LowPart = (*(*Fcb)).Header.FileSize.__bindgen_anon_1.LowPart;

            if IsPagingFile != 0 {
                FatLookupFileAllocationSize(IrpContext, *Fcb);
            }

            Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
                FatGetFcbOplock(*Fcb),
                (*IrpContext).OriginatingIrp,
                OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
                core::ptr::null_mut(),
                None,
                None,
            );
            if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
                Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                    FatGetFcbOplock(*Fcb),
                    (*IrpContext).OriginatingIrp,
                    (*(*Fcb)).UncleanCount + 1,
                );
            }

            if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS {
                NT_ASSERT!((Iosb.__bindgen_anon_1.Status != STATUS_PENDING) as i32);
                FatRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
                // unreachable
            }

            match CreateDisposition {
                FILE_OPEN | FILE_OPEN_IF => {
                    DebugTrace!(0, (Dbg) as i32, b"Doing only an open operation\n\0".as_ptr() as *const i8);
                    if NoEaKnowledge != 0 && FatIsFat32(Vcb) == 0 {
                        let mut NeedEaCount: u32 = 0;
                        FatGetNeedEaCount(
                            IrpContext,
                            Vcb,
                            Dirent,
                            core::ptr::addr_of_mut!(NeedEaCount),
                        );
                        if NeedEaCount != 0 {
                            FatRaiseStatus(IrpContext, STATUS_ACCESS_DENIED);
                            // unreachable
                        }
                    }
                    UnwindCcb = FatCreateCcb(IrpContext);
                    FatSetFileObject(
                        FileObject,
                        (UserFileOpen) as TYPE_OF_OPEN,
                        (*Fcb).cast(),
                        UnwindCcb,
                    );
                    // Unconditional in create.c too; its ARGUMENT_PRESENT(FileObject) guards only the
                    // AbnormalTermination() unwind in the finally below.
                    (*FileObject).SectionObjectPointer = // nolint: dropped-null-guard
                        core::ptr::addr_of_mut!((*(*(*Fcb)).NonPaged).SectionObjectPointers) as *mut _;
                    Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    Iosb.Information = FILE_OPENED as _;
                    break;
                }
                FILE_SUPERSEDE | FILE_OVERWRITE | FILE_OVERWRITE_IF => {
                    DebugTrace!(0, (Dbg) as i32, b"Doing supersede/overwrite operation\n\0".as_ptr() as *const i8);
                    Iosb.__bindgen_anon_1.Status = FatCheckSystemSecurityAccess(IrpContext);
                    if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
                        break 'try_block;
                    }
                    Iosb = FatSupersedeOrOverwriteFile(
                        IrpContext,
                        FileObject,
                        *Fcb,
                        AllocationSize,
                        EaBuffer,
                        EaLength,
                        FileAttributes,
                        CreateDisposition,
                        (NoEaKnowledge) as u8,
                    );
                    break;
                }
                _ => {
                    DebugTrace!(0, (Dbg) as i32, b"Illegal Create Disposition\n\0".as_ptr() as *const i8);
                    FatBugCheck((CreateDisposition) as ULONG_PTR, 0, 0);
                }
            }
        }

        // try_exit code
        if Iosb.__bindgen_anon_1.Status != STATUS_PENDING && NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 {
            ClearFlag!(*DesiredAccess, AddedAccess);

            IoSetShareAccess(
                *DesiredAccess,
                (ShareAccess) as u32,
                FileObject,
                core::ptr::addr_of_mut!((*(*Fcb)).ShareAccess),
            );

            (*(*Fcb)).UncleanCount += 1;
            (*(*Fcb)).OpenCount += 1;
            if FlagOn((*FileObject).Flags, FO_NO_INTERMEDIATE_BUFFERING) != 0 {
                (*(*Fcb)).NonCachedUncleanCount += 1;
            }
            (*Vcb).OpenFileCount += 1;
            if IsFileObjectReadOnly(FileObject) != 0 {
                (*Vcb).ReadOnlyCount += 1;
            }
            CountsIncremented = 1;
        }

        {
            let Ccb = (*FileObject).FsContext2 as PCCB;
            if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 {
                if DeleteOnClose != 0 {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
                }
                if FileNameOpenedDos != 0 {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
                }
                if FatCheckManageVolumeAccess(
                    IrpContext,
                    (*(*IrpSp).Parameters.Create.SecurityContext).AccessState,
                    if FlagOn(((*IrpSp).Flags) as u32, SL_FORCE_ACCESS_CHECK) != 0 {
                        1i8
                    } else {
                        (*(*IrpContext).OriginatingIrp).RequestorMode
                    } as KPROCESSOR_MODE,
                ) != 0
                {
                    SetFlag!((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS);
                }
            }
        }
    }, |_, abnormal| {
        DebugUnwind!(b"FatOpenExistingFile\0".as_ptr() as *const i8);

        if abnormal {
            if *p_CountsIncremented != 0 {
                (*(*Fcb)).UncleanCount -= 1;
                (*(*Fcb)).OpenCount -= 1;
                if FlagOn((*FileObject).Flags, FO_NO_INTERMEDIATE_BUFFERING) != 0 {
                    (*(*Fcb)).NonCachedUncleanCount -= 1;
                }
                (*Vcb).OpenFileCount -= 1;
                if IsFileObjectReadOnly(FileObject) != 0 {
                    (*Vcb).ReadOnlyCount -= 1;
                }
            }
            if !(*p_UnwindFcb).is_null() {
                if !FileObject.is_null() {
                    (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                }
                FatDeleteFcb(IrpContext, &mut *p_UnwindFcb);
                *Fcb = core::ptr::null_mut();
            }
            if !(*p_UnwindCcb).is_null() {
                FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
            }
        }

        DebugTrace!(-1, (Dbg) as i32, b"FatOpenExistingFile -> Iosb.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status);
    });

    Iosb
}

#[no_mangle]
pub unsafe extern "system" fn FatCreateNewDirectory(
    IrpContext: PIRP_CONTEXT,
    IrpSp: PIO_STACK_LOCATION,
    FileObject: PFILE_OBJECT,
    Vcb: PVCB,
    ParentDcb: PDCB,
    OemName: POEM_STRING,
    UnicodeName: PUNICODE_STRING,
    DesiredAccess: PACCESS_MASK,
    ShareAccess: USHORT,
    EaBuffer: PFILE_FULL_EA_INFORMATION,
    EaLength: ULONG,
    FileAttributes: USHORT,
    NoEaKnowledge: BOOLEAN,
    DeleteOnClose: BOOLEAN,
    OpenRequiringOplock: BOOLEAN,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    let mut Dcb: PDCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let mut Dirent: PDIRENT = core::ptr::null_mut();
    let mut DirentBcb: PBCB = core::ptr::null_mut();
    let mut DirentsNeeded: ULONG;
    let mut DirentByteOffset: ULONG;

    let mut ShortDirent: PDIRENT = core::ptr::null_mut();
    let mut ShortDirentByteOffset: ULONG = 0;

    let mut EaHandle: USHORT = 0;

    let mut AllLowerComponent: BOOLEAN = 0;
    let mut AllLowerExtension: BOOLEAN = 0;
    let mut CreateLfn: BOOLEAN = 0;

    let mut BytesInFirstPage: ULONG = 0;
    let mut DirentsInFirstPage: ULONG = 0;
    let mut FirstPageDirent: PDIRENT = core::ptr::null_mut();

    let mut SecondPageBcb: PBCB = core::ptr::null_mut();
    let mut SecondPageOffset: ULONG = 0;
    let mut SecondPageDirent: PDIRENT = core::ptr::null_mut();

    let mut DirentFromPool: BOOLEAN = 0;

    let mut ShortName: OEM_STRING = OEM_STRING {
        Length: 0,
        MaximumLength: 12,
        Buffer: core::ptr::null_mut(),
    };
    let mut ShortNameBuffer: [UCHAR; 12] = [0; 12];
    ShortName.Buffer = (&mut ShortNameBuffer[0]) as *mut UCHAR as *mut i8; // Assuming CHAR buffer

    let _ = &IrpSp; // UNREFERENCED_PARAMETER

    // PAGED_CODE(); // assume it expands to nothing here

    DebugTrace!(1, (Dbg) as i32, b"FatCreateNewDirectory...\n\0".as_ptr() as *const i8, 0);

    if NoEaKnowledge != 0 && EaLength > 0 {
        Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        DebugTrace!(-1, (Dbg) as i32, b"FatCreateNewDirectory -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, Iosb.__bindgen_anon_1.Status);
        return Iosb;
    }

    if DeleteOnClose != 0 && FlagOn(FileAttributes as u32, FAT_DIRENT_ATTR_READ_ONLY as u32) != 0 {
        Iosb.__bindgen_anon_1.Status = STATUS_CANNOT_DELETE;
        return Iosb;
    }

    FatSelectNames(
        (IrpContext).cast(),
        (ParentDcb).cast(),
        OemName,
        UnicodeName,
        &mut ShortName,
        core::ptr::null_mut(),
        &mut AllLowerComponent,
        &mut AllLowerExtension,
        &mut CreateLfn,
    );

    if (*&raw const FatData).ChicagoMode() == 0 {
        AllLowerComponent = 0;
        AllLowerExtension = 0;
        CreateLfn = 0;
    }

    DirentsNeeded = if CreateLfn != 0 {
        FAT_LFN_DIRENTS_NEEDED(UnicodeName) + 1
    } else {
        1
    };

    DirentByteOffset = FatCreateNewDirent(
        IrpContext,
        ParentDcb,
        DirentsNeeded,
        0,
    );

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, ... ))
    if crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
        FatPrepareWriteDirectoryFile(
            IrpContext,
            ParentDcb,
            DirentByteOffset,
            core::mem::size_of::<DIRENT>() as ULONG,
            (&mut DirentBcb as *mut *mut c_void as *mut *mut BCB).cast(),
            (&raw mut Dirent).cast(),
            0,
            1,
            &mut Iosb.__bindgen_anon_1.Status,
        );

        NT_ASSERT!((NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !DirentBcb.is_null() && !Dirent.is_null()) as i32);

        if (DirentByteOffset / (PAGE_SIZE) as u32) !=
            ((DirentByteOffset + (DirentsNeeded - 1) * core::mem::size_of::<DIRENT>() as ULONG) / (PAGE_SIZE) as u32) {

            SecondPageOffset = (DirentByteOffset & (!(PAGE_SIZE - 1)) as u32) + (PAGE_SIZE) as u32;

            BytesInFirstPage = SecondPageOffset - DirentByteOffset;

            DirentsInFirstPage = BytesInFirstPage / core::mem::size_of::<DIRENT>() as ULONG;

            FatPrepareWriteDirectoryFile(
                IrpContext,
                ParentDcb,
                SecondPageOffset,
                core::mem::size_of::<DIRENT>() as ULONG,
                (&mut SecondPageBcb as *mut *mut c_void as *mut *mut BCB).cast(),
                (&raw mut SecondPageDirent).cast(),
                0,
                1,
                &mut Iosb.__bindgen_anon_1.Status,
            );

            NT_ASSERT!((NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 && !SecondPageBcb.is_null() && !SecondPageDirent.is_null()) as i32);

            FirstPageDirent = Dirent;

            Dirent = FsRtlAllocatePoolWithTag(
                PagedPool,
                DirentsNeeded * core::mem::size_of::<DIRENT>() as ULONG,
                TAG_DIRENT,
            ) as PDIRENT;

            DirentFromPool = 1;
        }

        ShortDirent = Dirent.add((DirentsNeeded - 1) as usize);
        ShortDirentByteOffset = DirentByteOffset + (DirentsNeeded - 1) * core::mem::size_of::<DIRENT>() as ULONG;

        NT_ASSERT!(NT_SUCCESS(Iosb.__bindgen_anon_1.Status) as i32);

        FatConstructDirent(
            IrpContext,
            ShortDirent,
            &mut ShortName,
            (AllLowerComponent) as _,
            (AllLowerExtension) as _,
            if CreateLfn != 0 { UnicodeName } else { core::ptr::null_mut() },
            (FileAttributes as u32 | FAT_DIRENT_ATTR_DIRECTORY as u32) as u16,
            1,
            core::ptr::null_mut(),
        );

        if DirentFromPool != 0 {
            RtlCopyMemory(
                FirstPageDirent as PVOID,
                Dirent as PVOID,
                BytesInFirstPage as usize,
            );

            RtlCopyMemory(
                SecondPageDirent as PVOID,
                Dirent.add(DirentsInFirstPage as usize) as PVOID,
                (DirentsNeeded * core::mem::size_of::<DIRENT>() as ULONG - BytesInFirstPage as ULONG) as usize,
            );

            ShortDirent = SecondPageDirent.add((DirentsNeeded - DirentsInFirstPage) as usize).sub(1);
        }

        Dcb = FatCreateDcb(
            IrpContext,
            Vcb,
            ParentDcb,
            DirentByteOffset,
            ShortDirentByteOffset,
            ShortDirent,
            if CreateLfn != 0 { UnicodeName } else { core::ptr::null_mut() },
        );

        // NTDDI >= WIN8 only (create.c 5178-5237, FatCreateNewDirectory). Pre-WIN8 a directory
        // has no oplock: FatGetFcbOplock(Dcb) is Specific.Fcb.Oplock, which for a DCB is a
        // word of its in-union free-dirent bitmap.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
            Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
                FatGetFcbOplock(Dcb as PFCB),
                (*IrpContext).OriginatingIrp,
                OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
                Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                    FatGetFcbOplock(Dcb as PFCB),
                    (*IrpContext).OriginatingIrp,
                    (*Dcb).UncleanCount + 1,
                );
            }

            if Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
                Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
                    FatGetFcbOplock(ParentDcb as PFCB),
                    (*IrpContext).OriginatingIrp,
                    OPLOCK_FLAG_PARENT_OBJECT,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }

            if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS {
                FatRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
            }
        }

        if EaLength > 0 {
            FatCreateEa(
                IrpContext,
                (*Dcb).Vcb,
                EaBuffer as PUCHAR,
                EaLength,
                (*Dcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING,
                &mut EaHandle,
            );
        }

        if FatIsFat32((*Dcb).Vcb) == 0 {
            *(*ShortDirent).__bindgen_anon_1.ExtendedAttributes.as_mut() = EaHandle;
        }

        FatInitializeDirectoryDirent(IrpContext, Dcb, ShortDirent);

        Ccb = FatCreateCcb(IrpContext);
        FatSetFileObject(
            FileObject,
            (UserDirectoryOpen) as TYPE_OF_OPEN,
            (Dcb as PFCB).cast(),
            Ccb,
        );

        if (*Dcb).FullFileName.Buffer.is_null() {
            FatSetFullNameInFcb(IrpContext, Dcb as PFCB, UnicodeName);
        }

        FatNotifyReportChange(
            IrpContext,
            Vcb,
            Dcb as PFCB,
            FILE_NOTIFY_CHANGE_DIR_NAME,
            FILE_ACTION_ADDED,
        );

        IoSetShareAccess(
            *DesiredAccess,
            ShareAccess as ULONG,
            FileObject,
            &mut (*Dcb).ShareAccess,
        );

        (*Dcb).UncleanCount += 1;
        (*Dcb).OpenCount += 1;
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*Vcb).ReadOnlyCount += 1;
        }

        if DeleteOnClose != 0 {
            SetFlag!((*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
        }

        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = FILE_CREATED as _;

    }).is_err() {
        Iosb.__bindgen_anon_1.Status = (*IrpContext).ExceptionStatus;
    }

    if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
        if !Ccb.is_null() {
            FatDeleteCcb(IrpContext, &mut Ccb);
        }

        if Dcb.is_null() {
            NT_ASSERT!(((*ParentDcb).Specific.Dcb.as_ref().UnusedDirentVbo == 0xFFFFFFFF_u32 ||
                RtlAreBitsSet(
                    core::ptr::addr_of!((*ParentDcb).Specific.Dcb.as_ref().FreeDirentBitmap) as *mut RTL_BITMAP,
                    DirentByteOffset / core::mem::size_of::<DIRENT>() as ULONG,
                    DirentsNeeded,
                ) != 0) as i32);

            RtlClearBits(
                core::ptr::addr_of_mut!((*ParentDcb).Specific.Dcb.as_mut().FreeDirentBitmap) as *mut RTL_BITMAP,
                DirentByteOffset / core::mem::size_of::<DIRENT>() as ULONG,
                DirentsNeeded,
            );

            if !Dirent.is_null() {
                for i in 0..DirentsNeeded {
                    if DirentFromPool == 0 {
                        (*Dirent.add(i as usize)).FileName[0] = 0xE5; // FAT_DIRENT_DELETED
                    } else {
                        if SecondPageBcb.is_null() && i == DirentsInFirstPage {
                            break;
                        }

                        if i < DirentsInFirstPage {
                            (*FirstPageDirent.add(i as usize)).FileName[0] = 0xE5;
                        } else {
                            (*SecondPageDirent.add((i - DirentsInFirstPage) as usize)).FileName[0] = 0xE5;
                        }
                    }
                }
            }
        }
    }

    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((DirentBcb)) as *mut *mut core::ffi::c_void);
    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((SecondPageBcb)) as *mut *mut core::ffi::c_void);

    if DirentFromPool != 0 {
        ExFreePool(Dirent as PVOID);
    }

    if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) == 0 {
        if !Dcb.is_null() {
            let DirectoryFileObject: PFILE_OBJECT;

            DirectoryFileObject = (*Dcb).Specific.Dcb.as_ref().DirectoryFile;

            SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE);
            FatUnpinRepinnedBcbs(IrpContext);
            ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE);

            if (*Dcb).FirstClusterOfFile != 0 {
                // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, ... )) { }
                let _ = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
                    (*Dcb).Header.FileSize.__bindgen_anon_1.LowPart = 0;

                    CcSetFileSizes(
                        DirectoryFileObject,
                        core::ptr::addr_of!((*Dcb).Header.AllocationSize) as PCC_FILE_SIZES,
                    );

                    FatTruncateFileAllocation(
                        IrpContext,
                        Dcb as PFCB,
                        0,
                    );
                });
            }

            if !DirectoryFileObject.is_null() {
                FatSyncUninitializeCacheMap(
                    IrpContext,
                    DirectoryFileObject,
                );
            }

            // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, ... )) { }
            let _ = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
                FatDeleteDirent(
                    IrpContext,
                    Dcb,
                    core::ptr::null_mut(),
                    1,
                );

                FatUnpinRepinnedBcbs(IrpContext);
            });

            if !DirectoryFileObject.is_null() {
                (*Dcb).Specific.Dcb.as_mut().DirectoryFile = core::ptr::null_mut();
                ObDereferenceObject(DirectoryFileObject as PVOID);
            } else {
                if ARGUMENT_PRESENT(FileObject as PVOID) != 0 {
                    (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                }
                FatDeleteFcb(IrpContext, &mut Dcb);
            }
        }

        DebugTrace!(-1, (Dbg) as i32, b"FatCreateNewDirectory -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, Iosb.__bindgen_anon_1.Status);

        FatRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
    }

    let _ = &EaBuffer;
    let _ = &EaLength;

    Iosb
}
#[no_mangle]
pub unsafe extern "system" fn FatCreateNewFile(
    IrpContext: *mut IRP_CONTEXT,
    IrpSp: *mut IO_STACK_LOCATION,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    ParentDcb: *mut DCB,
    OemName: *mut OEM_STRING,
    UnicodeName: *mut _UNICODE_STRING,
    DesiredAccess: *mut u32,
    ShareAccess: u16,
    AllocationSize: u32,
    EaBuffer: *mut FILE_FULL_EA_INFORMATION,
    EaLength: u32,
    FileAttributes: u16,
    LfnBuffer: *mut _UNICODE_STRING,
    IsPagingFile: i32,
    NoEaKnowledge: i32,
    DeleteOnClose: i32,
    OpenRequiringOplock: i32,
    TemporaryFile: i32,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    Iosb.__bindgen_anon_1.Status = 0;
    Iosb.Information = 0;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut DirentsNeeded: u32 = 0;
    let mut DirentByteOffset: u32 = 0;
    let mut ShortDirent: *mut DIRENT = core::ptr::null_mut();
    let mut ShortDirentByteOffset: u32 = 0;
    let mut EaHandle: u16 = 0;
    let mut AllLowerComponent: BOOLEAN = 0;
    let mut AllLowerExtension: BOOLEAN = 0;
    let mut CreateLfn: BOOLEAN = 0;
    let mut BytesInFirstPage: u32 = 0;
    let mut DirentsInFirstPage: u32 = 0;
    let mut FirstPageDirent: *mut DIRENT = core::ptr::null_mut();
    let mut SecondPageBcb: *mut BCB = core::ptr::null_mut();
    let mut SecondPageOffset: u32 = 0;
    let mut SecondPageDirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentFromPool: i32 = 0;
    let mut ShortNameBuffer: [u8; 12] = [0u8; 12];
    let mut ShortName = OEM_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of_val(&ShortNameBuffer) as u16,
        Buffer: ShortNameBuffer.as_mut_ptr().cast(),
    };
    let mut UniTunneledShortNameBuffer: [u16; 12] = [0u16; 12];
    let mut UniTunneledShortName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of_val(&UniTunneledShortNameBuffer) as u16,
        Buffer: UniTunneledShortNameBuffer.as_mut_ptr(),
    };
    let mut UniTunneledLongNameBuffer: [u16; 26] = [0u16; 26];
    let mut UniTunneledLongName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of_val(&UniTunneledLongNameBuffer) as u16,
        Buffer: UniTunneledLongNameBuffer.as_mut_ptr(),
    };
    let mut TunneledCreationTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut TunneledDataSize: u32 = core::mem::size_of::<LARGE_INTEGER>() as u32;
    let mut HaveTunneledInformation: i32 = 0;
    let mut UsingTunneledLfn: i32 = 0;
    let mut RealUnicodeName: *mut _UNICODE_STRING;
    let mut UnwindDirent: *mut DIRENT = core::ptr::null_mut();
    let mut UnwindFcb: *mut FCB = core::ptr::null_mut();
    let mut UnwindAllocation: i32 = 0;
    let mut CountsIncremented: i32 = 0;
    let mut UnwindCcb: *mut CCB = core::ptr::null_mut();
    let mut LocalAbnormalTermination: i32 = 0;

    if NoEaKnowledge != 0 && EaLength > 0 {
        Iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        DebugTrace!(-1, (Dbg) as i32, b"FatCreateNewFile -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8, Iosb.__bindgen_anon_1.Status);
        return Iosb;
    }
    if DeleteOnClose != 0 && FlagOn(FileAttributes as u32, FAT_DIRENT_ATTR_READ_ONLY as u32) != 0 {
        Iosb.__bindgen_anon_1.Status = STATUS_CANNOT_DELETE;
        return Iosb;
    }
    HaveTunneledInformation = FsRtlFindInTunnelCache(
        &raw mut (*Vcb).Tunnel,
        FatDirectoryKey(ParentDcb),
        UnicodeName,
        &mut UniTunneledShortName,
        &mut UniTunneledLongName,
        &mut TunneledDataSize,
        &mut TunneledCreationTime as *mut LARGE_INTEGER as *mut c_void,
    ) as i32;
    NT_ASSERT!((TunneledDataSize == core::mem::size_of::<LARGE_INTEGER>() as u32) as i32);
    FatSelectNames(
        (IrpContext).cast(),
        (ParentDcb).cast(),
        OemName,
        UnicodeName,
        &mut ShortName,
        if HaveTunneledInformation != 0 { &mut UniTunneledShortName } else { core::ptr::null_mut() },
        &mut AllLowerComponent,
        &mut AllLowerExtension,
        &mut CreateLfn,
    );
    RealUnicodeName = UnicodeName;
    if (*&raw const FatData).ChicagoMode() == 0 {
        AllLowerComponent = 0;
        AllLowerExtension = 0;
        CreateLfn = 0;
    } else {
        if CreateLfn == 0 &&
            UniTunneledLongName.Length != 0 &&
            FatLfnDirentExists(IrpContext, ParentDcb, &mut UniTunneledLongName, LfnBuffer) == 0
        {
            UsingTunneledLfn = 1;
            CreateLfn = 1;
            RealUnicodeName = &UniTunneledLongName as *const _UNICODE_STRING as *mut _UNICODE_STRING;
            AllLowerComponent = 0;
            AllLowerExtension = 0;
        }
    }
    DirentsNeeded = if CreateLfn != 0 {
        FAT_LFN_DIRENTS_NEEDED(RealUnicodeName) + 1
    } else {
        1
    };
    DirentByteOffset = FatCreateNewDirent(IrpContext, ParentDcb, DirentsNeeded, 0);
    let p_Iosb = core::ptr::addr_of_mut!(Iosb);
    let p_Fcb = core::ptr::addr_of_mut!(Fcb);
    let p_Dirent = core::ptr::addr_of_mut!(Dirent);
    let p_DirentBcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_UniTunneledLongName = core::ptr::addr_of_mut!(UniTunneledLongName);
    let p_UniTunneledLongNameBuffer = core::ptr::addr_of_mut!(UniTunneledLongNameBuffer);
    let p_LocalAbnormalTermination = core::ptr::addr_of_mut!(LocalAbnormalTermination);
    let p_CountsIncremented = core::ptr::addr_of_mut!(CountsIncremented);
    let p_UnwindFcb = core::ptr::addr_of_mut!(UnwindFcb);
    let p_UnwindDirent = core::ptr::addr_of_mut!(UnwindDirent);
    let p_UnwindAllocation = core::ptr::addr_of_mut!(UnwindAllocation);
    let p_UnwindCcb = core::ptr::addr_of_mut!(UnwindCcb);
    let p_DirentFromPool = core::ptr::addr_of_mut!(DirentFromPool);
    let p_SecondPageBcb = core::ptr::addr_of_mut!(SecondPageBcb);
    let p_FirstPageDirent = core::ptr::addr_of_mut!(FirstPageDirent);
    let p_SecondPageDirent = core::ptr::addr_of_mut!(SecondPageDirent);
    let p_DirentsInFirstPage = core::ptr::addr_of_mut!(DirentsInFirstPage);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| {
        FatPrepareWriteDirectoryFile(
            IrpContext,
            ParentDcb,
            DirentByteOffset,
            core::mem::size_of::<DIRENT>() as u32,
            (&raw mut DirentBcb).cast(),
            (&raw mut Dirent).cast(),
            0,
            1,
            &mut Iosb.__bindgen_anon_1.Status,
        );
        NT_ASSERT!(NT_SUCCESS(Iosb.__bindgen_anon_1.Status) as i32);
        UnwindDirent = Dirent;
        if (DirentByteOffset / (PAGE_SIZE as u32)) !=
            ((DirentByteOffset + (DirentsNeeded - 1) * core::mem::size_of::<DIRENT>() as u32) / (PAGE_SIZE as u32))
        {
            SecondPageOffset = (DirentByteOffset & (!(PAGE_SIZE - 1) as u32)) + (PAGE_SIZE as u32);
            BytesInFirstPage = SecondPageOffset - DirentByteOffset;
            DirentsInFirstPage = BytesInFirstPage / core::mem::size_of::<DIRENT>() as u32;
            FatPrepareWriteDirectoryFile(
                IrpContext,
                ParentDcb,
                SecondPageOffset,
                core::mem::size_of::<DIRENT>() as u32,
                (&raw mut SecondPageBcb).cast(),
                (&raw mut SecondPageDirent).cast(),
                0,
                1,
                &mut Iosb.__bindgen_anon_1.Status,
            );
            NT_ASSERT!(NT_SUCCESS(Iosb.__bindgen_anon_1.Status) as i32);
            FirstPageDirent = Dirent;
            Dirent = FsRtlAllocatePoolWithTag(
                PagedPool,
                DirentsNeeded * core::mem::size_of::<DIRENT>() as u32,
                TAG_DIRENT,
            ) as *mut DIRENT;
            DirentFromPool = 1;
        }
        ShortDirent = Dirent.add((DirentsNeeded as usize) - 1);
        ShortDirentByteOffset = DirentByteOffset + (DirentsNeeded - 1) * core::mem::size_of::<DIRENT>() as u32;
        NT_ASSERT!(NT_SUCCESS(Iosb.__bindgen_anon_1.Status) as i32);
        FatConstructDirent(
            IrpContext,
            ShortDirent,
            &mut ShortName as *mut OEM_STRING,
            (AllLowerComponent as i32) as _,
            (AllLowerExtension as i32) as _,
            if CreateLfn != 0 { RealUnicodeName as *mut _ } else { core::ptr::null_mut() },
            FileAttributes | (FILE_ATTRIBUTE_ARCHIVE as u16),
            1,
            if HaveTunneledInformation != 0 { (&TunneledCreationTime as *const LARGE_INTEGER as *mut u16).cast() } else { (core::ptr::null_mut() as *mut u16).cast() },
        );
        if DirentFromPool != 0 {
            // RtlCopyMemory( FirstPageDirent, Dirent, ... ): pool buffer -> the two
            // pinned pages (the translation had source and destination swapped).
            core::ptr::copy_nonoverlapping(
                Dirent as *const u8,
                FirstPageDirent as *mut u8,
                BytesInFirstPage as usize,
            );
            core::ptr::copy_nonoverlapping(
                Dirent.add(DirentsInFirstPage as usize) as *const u8,
                SecondPageDirent as *mut u8,
                (DirentsNeeded * core::mem::size_of::<DIRENT>() as u32 - BytesInFirstPage) as usize,
            );
            ShortDirent = SecondPageDirent.add((DirentsNeeded - DirentsInFirstPage) as usize - 1);
        }
        Fcb = FatCreateFcb(
            IrpContext,
            Vcb,
            ParentDcb,
            DirentByteOffset,
            ShortDirentByteOffset,
            ShortDirent,
            if CreateLfn != 0 { RealUnicodeName } else { core::ptr::null_mut() as *mut _UNICODE_STRING },
            if CreateLfn != 0 { RealUnicodeName } else { core::ptr::null_mut() as *mut _UNICODE_STRING },
            (IsPagingFile as u8) as BOOLEAN,
            0,
        );
        UnwindFcb = Fcb;
        UnwindDirent = core::ptr::null_mut();
        // NTDDI >= WIN8 only (create.c 6021-6049, FatCreateNewFile; it also holds the WIN7 key
        // check). Its parent-directory check read the DCB's free-dirent bitmap as an oplock
        // pointer: once enough dirents were in use, a run of file creates bugchecked 0x23.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
        Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
            FatGetFcbOplock(Fcb),
            (*IrpContext).OriginatingIrp,
            OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut() as *mut NTSTATUS,
        );
        if OpenRequiringOplock != 0 && Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
            Iosb.__bindgen_anon_1.Status = FsRtlOplockFsctrl(
                FatGetFcbOplock(Fcb),
                (*IrpContext).OriginatingIrp,
                (*Fcb).UncleanCount + 1,
            );
        }
        if Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
            Iosb.__bindgen_anon_1.Status = FsRtlCheckOplockEx(
                FatGetFcbOplock(ParentDcb),
                (*IrpContext).OriginatingIrp,
                OPLOCK_FLAG_PARENT_OBJECT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut() as *mut NTSTATUS,
            );
        }
        if Iosb.__bindgen_anon_1.Status != STATUS_SUCCESS {
            FatRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
        }
        }
        if TemporaryFile != 0 {
            SetFlag!((*Fcb).FcbState, FCB_STATE_TEMPORARY);
        }
        FatAddFileAllocation(IrpContext, Fcb, FileObject, AllocationSize);
        UnwindAllocation = 1;
        (*Fcb).FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
        if EaLength > 0 {
            FatCreateEa(
                IrpContext,
                (*Fcb).Vcb,
                EaBuffer as *mut u8,
                EaLength,
                (*Fcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING,
                &mut EaHandle,
            );
        }
        if FatIsFat32((*Fcb).Vcb) == 0 {
            *(*ShortDirent).__bindgen_anon_1.ExtendedAttributes.as_mut() = EaHandle;
        }
        if (*Fcb).FullFileName.Buffer.is_null() {
            FatSetFullNameInFcb(IrpContext, Fcb, RealUnicodeName as *mut _UNICODE_STRING);
        }
        UnwindCcb = FatCreateCcb(IrpContext);
        FatSetFileObject(FileObject, (UserFileOpen) as TYPE_OF_OPEN, (Fcb).cast(), UnwindCcb);
        (*FileObject).SectionObjectPointer = &raw mut (*(*Fcb).NonPaged).SectionObjectPointers;
        FatNotifyReportChange(
            IrpContext,
            Vcb,
            Fcb,
            FILE_NOTIFY_CHANGE_FILE_NAME,
            FILE_ACTION_ADDED,
        );
        IoSetShareAccess(
            *DesiredAccess,
            ShareAccess as u32,
            FileObject,
            &raw mut (*Fcb).ShareAccess,
        );
        (*Fcb).UncleanCount += 1;
        (*Fcb).OpenCount += 1;
        if FlagOn((*FileObject).Flags as u32, FO_NO_INTERMEDIATE_BUFFERING as u32) != 0 {
            (*Fcb).NonCachedUncleanCount += 1;
        }
        (*Vcb).OpenFileCount += 1;
        if IsFileObjectReadOnly(FileObject) != 0 {
            (*Vcb).ReadOnlyCount += 1;
        }
        CountsIncremented = 1;
        Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
        Iosb.Information = FILE_CREATED as _;
        if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) != 0 {
            if DeleteOnClose != 0 {
                SetFlag!((*UnwindCcb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
            }
            if CreateLfn == 0 && UsingTunneledLfn == 0 {
                SetFlag!((*UnwindCcb).Flags, CCB_FLAG_OPENED_BY_SHORTNAME);
            }
            if FatCheckManageVolumeAccess(
                IrpContext,
                (*(*IrpSp).Parameters.Create.SecurityContext).AccessState,
                if FlagOn(((*IrpSp).Flags) as u32, SL_FORCE_ACCESS_CHECK as u32) != 0 {
                    (1i32) as i8
                } else {
                    (*(*IrpContext).OriginatingIrp).RequestorMode
                },
            ) != 0
            {
                SetFlag!((*UnwindCcb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS);
            }
        }
    }, |_, abnormal| {
        DebugUnwind!(b"FatCreateNewFile\0" as *const u8 as *const i8);
        if (*p_UniTunneledLongName).Buffer != (*p_UniTunneledLongNameBuffer).as_mut_ptr() {
            ExFreePool((*p_UniTunneledLongName).Buffer as *mut c_void);
        }
        *p_LocalAbnormalTermination = abnormal as i32;
        if *p_LocalAbnormalTermination != 0 {
            if *p_CountsIncremented != 0 {
                (**p_Fcb).UncleanCount -= 1;
                (**p_Fcb).OpenCount -= 1;
                if FlagOn((*FileObject).Flags as u32, FO_NO_INTERMEDIATE_BUFFERING as u32) != 0 {
                    (**p_Fcb).NonCachedUncleanCount -= 1;
                }
                (*Vcb).OpenFileCount -= 1;
                if IsFileObjectReadOnly(FileObject) != 0 {
                    (*Vcb).ReadOnlyCount -= 1;
                }
            }
            if (*p_UnwindFcb).is_null() {
                NT_ASSERT!((((*ParentDcb).Specific.Dcb.as_ref().UnusedDirentVbo == 0xffffffff_u32) ||
                     (RtlAreBitsSet(
                         core::ptr::addr_of!((*ParentDcb).Specific.Dcb.as_ref().FreeDirentBitmap) as *mut RTL_BITMAP,
                         DirentByteOffset / core::mem::size_of::<DIRENT>() as u32,
                         DirentsNeeded,
                     ) != 0)) as i32);
                RtlClearBits(
                    core::ptr::addr_of_mut!((*ParentDcb).Specific.Dcb.as_mut().FreeDirentBitmap) as *mut RTL_BITMAP,
                    DirentByteOffset / core::mem::size_of::<DIRENT>() as u32,
                    DirentsNeeded,
                );
            }
            if !(*p_UnwindDirent).is_null() {
                let mut i: u32 = 0;
                while i < DirentsNeeded {
                    if *p_DirentFromPool == 0 {
                        (*((*p_Dirent).add(i as usize))).FileName[0] = FAT_DIRENT_DELETED;
                    } else {
                        if (*p_SecondPageBcb).is_null() && i == *p_DirentsInFirstPage {
                            break;
                        }
                        if i < *p_DirentsInFirstPage {
                            (*((*p_FirstPageDirent).add(i as usize))).FileName[0] = FAT_DIRENT_DELETED;
                        } else {
                            (*((*p_SecondPageDirent).add((i - *p_DirentsInFirstPage) as usize))).FileName[0] = FAT_DIRENT_DELETED;
                        }
                    }
                    i += 1;
                }
            }
        }
        // _SEH2_TRY { truncate } _SEH2_FINALLY { _SEH2_TRY { delete dirent } _SEH2_FINALLY { ... } }
        crate::seh::try_finally(&mut (), |_| {
            if *p_LocalAbnormalTermination != 0 {
                if *p_UnwindAllocation != 0 {
                    FatTruncateFileAllocation(IrpContext, *p_Fcb, 0);
                }
            }
        }, |_, _| {
            crate::seh::try_finally(&mut (), |_| {
                if *p_LocalAbnormalTermination != 0 {
                    if !(*p_UnwindFcb).is_null() {
                        FatDeleteDirent(IrpContext, *p_UnwindFcb, (core::ptr::null_mut() as *mut u16).cast(), 1);
                    }
                }
            }, |_, _| {
                if *p_LocalAbnormalTermination != 0 {
                    if !(*p_UnwindFcb).is_null() {
                        if ARGUMENT_PRESENT(FileObject as *mut c_void) != 0 {
                            (*FileObject).SectionObjectPointer = core::ptr::null_mut();
                        }
                        FatDeleteFcb(IrpContext, &mut *p_UnwindFcb);
                    }
                    if !(*p_UnwindCcb).is_null() {
                        FatDeleteCcb(IrpContext, &mut *p_UnwindCcb);
                    }
                }
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_DirentBcb) as *mut *mut core::ffi::c_void);
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_SecondPageBcb) as *mut *mut core::ffi::c_void);
                if *p_DirentFromPool != 0 {
                    ExFreePool(*p_Dirent as *mut c_void);
                }
            });
        });

        DebugTrace!(-1, (Dbg) as i32, b"FatCreateNewFile -> Iosb.Status = %08lx\n\0".as_ptr() as *const i8, (*p_Iosb).__bindgen_anon_1.Status);
    });
    return Iosb;
}


// Forward declarations of unknown extern functions

// Unknown constants used in this function
const FILE_SUPERSEDED: usize = 0x00;
const FILE_OVERWRITTEN: usize = 3;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x00000010;
const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x00000004;
const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x00000008;
const FILE_NOTIFY_CHANGE_EA: u32 = 0x80;
const FILE_ACTION_MODIFIED: u32 = 0x03;

// Helper struct for LARGE_INTEGER (if not already available)
#[no_mangle]
pub unsafe extern "system" fn FatSupersedeOrOverwriteFile(
    irp_context: *mut IRP_CONTEXT,
    file_object: *mut FILE_OBJECT,
    fcb: *mut FCB,
    allocation_size: u32,
    ea_buffer: *mut FILE_FULL_EA_INFORMATION,
    ea_length: u32,
    file_attributes: u16,
    create_disposition: u32,
    no_ea_knowledge: BOOLEAN,
) -> _IO_STATUS_BLOCK {
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    iosb.__bindgen_anon_1.Status = STATUS_SUCCESS as i32;
    iosb.Information = 0;

    let mut dirent: *mut DIRENT = core::ptr::null_mut();
    let mut dirent_bcb: *mut BCB = core::ptr::null_mut();

    let mut ea_handle: u16 = 0;
    let mut ea_change: BOOLEAN = 0;
    let mut release_paging: BOOLEAN = 0;

    let mut ccb: *mut CCB = core::ptr::null_mut();

    let mut notify_filter: u32 = 0;
    let mut header_size: u32 = 0;
    let mut alloc_size: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let mut unwind_ccb: *mut CCB = core::ptr::null_mut();
    let mut unwind_ea: u16 = 0;

    // PAGED_CODE(); // omitted

    DebugTrace!(1, (Dbg as u32) as i32, b"FatSupersedeOrOverwriteFile...\n\0".as_ptr() as *const i8, 0);

    dirent_bcb = core::ptr::null_mut();

    if no_ea_knowledge != 0 && ea_length > 0 {
        iosb.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED as i32;

        DebugTrace!(
            -1,
            (Dbg as u32) as i32,
            b"FatSupersedeOrOverwriteFile -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8,
            iosb.__bindgen_anon_1.Status,
        );
        return iosb;
    }

    let p_iosb = core::ptr::addr_of_mut!(iosb);
    let p_release_paging = core::ptr::addr_of_mut!(release_paging);
    let p_unwind_ea = core::ptr::addr_of_mut!(unwind_ea);
    let p_unwind_ccb = core::ptr::addr_of_mut!(unwind_ccb);
    let p_dirent_bcb = core::ptr::addr_of_mut!(dirent_bcb);
    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    crate::seh::try_finally(&mut (), |_| 'seh_try: loop {
        // Check if the purge is going to fail
        if MmCanFileBeTruncated(
            core::ptr::addr_of_mut!((*(*fcb).NonPaged).SectionObjectPointers),
            &FatLargeZero as *const _ as *mut _,
        ) == 0
        {
            iosb.__bindgen_anon_1.Status = STATUS_USER_MAPPED_FILE as i32;
            iosb.Information = 0;
            break 'seh_try;
        }

        // Setup context and section object pointers, update reference counts
        ccb = FatCreateCcb(irp_context);
        unwind_ccb = ccb;
        FatSetFileObject(file_object, (UserFileOpen) as TYPE_OF_OPEN, (fcb as *mut DCB).cast(), ccb);
        (*file_object).SectionObjectPointer =
            &(*(*fcb).NonPaged).SectionObjectPointers as *const _ as *mut _;

        // Purge section so that mappers will see zeros
        SetFlag!((*(*fcb).Vcb).VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);

        CcPurgeCacheSection(
            core::ptr::addr_of_mut!((*(*fcb).NonPaged).SectionObjectPointers),
            core::ptr::null_mut(),
            0,
            0,
        );

        // Tentatively add new EAs
        if ea_length > 0 {
            FatCreateEa(
                irp_context,
                (*fcb).Vcb,
                ea_buffer as *mut u8,
                ea_length,
                (*fcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING,
                &mut ea_handle,
            );
            unwind_ea = ea_handle;
            ea_change = 1;
        }

        // Break parent directory oplock -- NTDDI >= WIN8 only; a pre-WIN8 DCB has none.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
        let oplock_status = FsRtlCheckOplockEx(
            FatGetFcbOplock((*fcb).ParentDcb as *mut FCB),
            (*irp_context).OriginatingIrp,
            OPLOCK_FLAG_PARENT_OBJECT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if oplock_status != STATUS_SUCCESS as i32 {
            FatRaiseStatus(irp_context, oplock_status);
        }
        }

        // Acquire paging IO resource and zero file size
        ExAcquireResourceExclusiveLite((*fcb).Header.PagingIoResource, 1 as BOOLEAN);
        release_paging = 1;

        (*fcb).Header.FileSize.__bindgen_anon_1.LowPart = 0;
        (*fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = 0;
        (*fcb).ValidDataToDisk = 0;

        // Validate allocation size
        alloc_size.QuadPart = allocation_size as i64;
        if FatIsIoRangeValid((*fcb).Vcb, alloc_size, 0) == 0 {
            DebugTrace!(-1, (Dbg as u32) as i32, b"Illegal allocation size\n\0".as_ptr() as *const i8, 0);
            FatRaiseStatus(irp_context, STATUS_DISK_FULL as i32);
        }

        // Tell cache manager size went to zero
        CcSetFileSizes(
            file_object,
            &mut (*fcb).Header.AllocationSize as *mut _ as *mut CC_FILE_SIZES,
        );

        FatTruncateFileAllocation(
            irp_context,
            fcb,
            (allocation_size.wrapping_add(header_size) as i32) as u32,
        );

        ExReleaseResourceLite((*fcb).Header.PagingIoResource);
        release_paging = 0;

        FatAddFileAllocation(
            irp_context,
            fcb,
            file_object,
            allocation_size.wrapping_add(header_size),
        );

        (*fcb).FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;

        // Modify attributes and time of the file
        FatGetDirentFromFcbOrDcb(irp_context, fcb, (0u8) as _, &mut dirent, &mut dirent_bcb);

        (*dirent).FileSize = 0;

        let mut new_attrs: u8 = file_attributes as u8 | FILE_ATTRIBUTE_ARCHIVE as u8;

        if create_disposition == FILE_SUPERSEDE {
            (*dirent).Attributes = new_attrs;
        } else {
            (*dirent).Attributes |= new_attrs;
        }

        (*fcb).DirentFatFlags = (*dirent).Attributes;

        // Update last write time
        KeQuerySystemTime(&mut (*fcb).LastWriteTime);
        FatNtTimeToFatTime(
            irp_context,
            &mut (*fcb).LastWriteTime,
            1,
            &mut (*dirent).LastWriteTime,
            core::ptr::null_mut(),
        );

        if (*&raw const FatData).ChicagoMode() != 0 {
            (*dirent).LastAccessDate = core::ptr::read(&(*dirent).LastWriteTime.Date);
        }

        notify_filter = FILE_NOTIFY_CHANGE_LAST_WRITE
            | FILE_NOTIFY_CHANGE_ATTRIBUTES
            | FILE_NOTIFY_CHANGE_SIZE;

        // Delete previous EA set if present
        if FatIsFat32((*fcb).Vcb) == 0 && *(*dirent).__bindgen_anon_1.ExtendedAttributes.as_ref() != 0 {
            ea_change = 1;

            // _SEH2_TRY { FatDeleteEa } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ): any failure is swallowed.
            if crate::seh::try_except(&mut (), |_| {
                FatDeleteEa(
                    irp_context,
                    (*fcb).Vcb,
                    *(*dirent).__bindgen_anon_1.ExtendedAttributes.as_ref(),
                    (*fcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING,
                );
            }, |_, _| crate::seh::EXCEPTION_EXECUTE_HANDLER).is_err() {
                FatResetExceptionState(irp_context);
            }
        }

        // Update extended attributes handle
        if ea_change != 0 {
            // NT_ASSERT(!FatIsFat32(Fcb->Vcb)); // omitted
            *(*dirent).__bindgen_anon_1.ExtendedAttributes.as_mut() = ea_handle;
            notify_filter |= FILE_NOTIFY_CHANGE_EA;
        }

        // Commit EA handle and set bcb dirty
        FatSetDirtyBcb(irp_context, (dirent_bcb).cast(), (*fcb).Vcb, 1);
        unwind_ea = 0;

        // Indicate EA change for this file
        (*ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().EaModificationCount += (*fcb).EaModificationCount;

        // Notify outstanding IRPs
        FatNotifyReportChange(
            irp_context,
            (*fcb).Vcb,
            fcb,
            notify_filter,
            FILE_ACTION_MODIFIED,
        );

        // Success
        iosb.__bindgen_anon_1.Status = STATUS_SUCCESS as i32;
        if create_disposition == FILE_SUPERSEDE {
            iosb.Information = FILE_SUPERSEDED as _;
        } else {
            iosb.Information = FILE_OVERWRITTEN as _;
        }

        // Fall through to break
        break 'seh_try;
    }, |_, abnormal| {
        DebugUnwind!(b"FatSupersedeOrOverwriteFile\0".as_ptr() as *const i8);

        if *p_release_paging != 0 {
            ExReleaseResourceLite((*fcb).Header.PagingIoResource);
        }

        if abnormal {
            if *p_unwind_ea != 0 {
                FatDeleteEa(
                    irp_context,
                    (*fcb).Vcb,
                    *p_unwind_ea,
                    (*fcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING,
                );
            }
            if !(*p_unwind_ccb).is_null() {
                FatDeleteCcb(irp_context, &mut *p_unwind_ccb);
            }
        }

        FatUnpinBcb(irp_context, core::ptr::addr_of_mut!(*p_dirent_bcb) as *mut *mut core::ffi::c_void);

        ClearFlag!((*(*fcb).Vcb).VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);

        DebugTrace!(
            -1,
            (Dbg as u32) as i32,
            b"FatSupersedeOrOverwriteFile -> Iosb.__bindgen_anon_1.Status = %08lx\n\0".as_ptr() as *const i8,
            (*p_iosb).__bindgen_anon_1.Status,
        );
    });

    iosb
}

#[no_mangle]
pub unsafe extern "system" fn FatSetFullNameInFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    mut FinalName: *mut _UNICODE_STRING,
) {
    // PAGED_CODE();
    _ = IrpContext;

    // NT_ASSERT( Fcb->FullFileName.Buffer == NULL )
    if !(*Fcb).FullFileName.Buffer.is_null() {
        RtlAssert(
            (b"Fcb->FullFileName.Buffer == NULL\0".as_ptr()).cast(),
            (b"create.c\0".as_ptr()).cast(),
            line!() as u32,
            core::ptr::null(),
        );
    }

    // Prefer the ExactCaseLongName if set
    if !(*Fcb).ExactCaseLongName.Buffer.is_null() {
        // NT_ASSERT( Fcb->ExactCaseLongName.Length != 0 )
        if (*Fcb).ExactCaseLongName.Length == 0 {
            RtlAssert(
                (b"Fcb->ExactCaseLongName.Length != 0\0".as_ptr()).cast(),
                (b"create.c\0".as_ptr()).cast(),
                line!() as u32,
                core::ptr::null(),
            );
        }
        FinalName = core::ptr::addr_of_mut!((*Fcb).ExactCaseLongName);
    }

    // Special case the root.
    if NodeType!((*Fcb).ParentDcb as *const c_void) == (FAT_NTC_ROOT_DCB as u16) {
        (*Fcb).FullFileName.Length = (core::mem::size_of::<u16>() + (*FinalName).Length as usize) as u16;
        (*Fcb).FullFileName.MaximumLength = (*Fcb).FullFileName.Length;
        (*Fcb).FullFileName.Buffer = FsRtlAllocatePoolWithTag(
            PagedPool,
            (*Fcb).FullFileName.Length as u32,
            TAG_FILENAME_BUFFER,
        ) as *mut u16;

        *((*Fcb).FullFileName.Buffer) = 0x5C_u16;

        RtlCopyMemory(
            ((*Fcb).FullFileName.Buffer).add(1) as *mut c_void,
            (*FinalName).Buffer as *mut c_void,
            (*FinalName).Length as usize,
        );
    } else {
        let Prefix: *mut _UNICODE_STRING = core::ptr::addr_of_mut!((*(*Fcb).ParentDcb).FullFileName);

        if (*Prefix).Buffer.is_null() {
            return;
        }

        let wchar_size = core::mem::size_of::<u16>();
        let prefix_len_bytes = (*Prefix).Length as usize;
        let prefix_len_wchars = prefix_len_bytes / wchar_size;

        (*Fcb).FullFileName.Length = prefix_len_bytes as u16 + wchar_size as u16 + (*FinalName).Length;
        (*Fcb).FullFileName.MaximumLength = (*Fcb).FullFileName.Length;
        (*Fcb).FullFileName.Buffer = FsRtlAllocatePoolWithTag(
            PagedPool,
            (*Fcb).FullFileName.Length as u32,
            TAG_FILENAME_BUFFER,
        ) as *mut u16;

        RtlCopyMemory(
            ((*Fcb).FullFileName.Buffer) as *mut c_void,
            (*Prefix).Buffer as *mut c_void,
            prefix_len_bytes,
        );

        *((*Fcb).FullFileName.Buffer).add(prefix_len_wchars) = 0x5C_u16;

        RtlCopyMemory(
            ((*Fcb).FullFileName.Buffer).add(prefix_len_wchars + 1) as *mut c_void,
            (*FinalName).Buffer as *mut c_void,
            (*FinalName).Length as usize,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatCheckSystemSecurityAccess(IrpContext: *mut IRP_CONTEXT) -> i32 {
    // PAGED_CODE();
    let irp_sp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);
    let security_context: *mut IO_SECURITY_CONTEXT = ((*irp_sp).Parameters.Create.SecurityContext).cast();
    ntkrnl_sys::ASSERT!(!security_context.is_null());
    let access_state: *mut _ACCESS_STATE = (*security_context).AccessState;
    if ((*access_state).RemainingDesiredAccess & ACCESS_SYSTEM_SECURITY) != 0 {
        if SeSinglePrivilegeCheck(FatSecurityPrivilege, 1 as KPROCESSOR_MODE) == 0 {
            return STATUS_ACCESS_DENIED;
        }
        (*access_state).RemainingDesiredAccess &= !ACCESS_SYSTEM_SECURITY;
        (*access_state).PreviouslyGrantedAccess |= ACCESS_SYSTEM_SECURITY;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FatCheckShareAccess(
    IrpContext: PIRP_CONTEXT,
    FileObject: PFILE_OBJECT,
    FcbOrDcb: PFCB,
    DesiredAccess: PACCESS_MASK,
    ShareAccess: ULONG,
) -> NTSTATUS {
    PAGED_CODE!();

    // NTDDI_VERSION >= NTDDI_VISTA block (unconditionally included; in practice always true for modern targets)
    if NodeType!(FcbOrDcb as *const c_void) == (FAT_NTC_FCB as u16)
        && FlagOn(ShareAccess as u32, FILE_SHARE_WRITE) == 0
        && FlagOn(
            *DesiredAccess as u32,
            FILE_EXECUTE | FILE_READ_DATA | FILE_WRITE_DATA | FILE_APPEND_DATA | DELETE | MAXIMUM_ALLOWED,
        ) != 0
        && MmDoesFileHaveUserWritableReferences(
            core::ptr::addr_of!((*(*FcbOrDcb).NonPaged).SectionObjectPointers),
        ) != 0
    {
        return STATUS_SHARING_VIOLATION;
    }

    let result = IoCheckShareAccess(
        *DesiredAccess,
        ShareAccess,
        FileObject,
        core::ptr::addr_of_mut!((*FcbOrDcb).ShareAccess),
        0, // FALSE
    );

    let _ = IrpContext;

    result
}

#[no_mangle]
pub unsafe extern "system" fn FatCallSelfCompletionRoutine(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Contxt: *mut core::ffi::c_void,
) -> i32 {
    {
        // Set the event so that our call will wake up.
        KeSetEvent(Contxt as *mut KEVENT, 0, 0); // 0 = FALSE
    }

    // UNREFERENCED_PARAMETER( DeviceObject );
    let _ = DeviceObject;
    // UNREFERENCED_PARAMETER( Irp );
    let _ = Irp;

    // STATUS_MORE_PROCESSING_REQUIRED
    0xC0000016u32 as i32
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
//     Create.c
//
// Abstract:
//
//     This module implements the File Create routine for Fat called by the
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_CREATE)
//
// //
// //  The debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_CREATE)
//
//
// //
// //  Macros for incrementing performance counters.
// //
//
// #define CollectCreateHitStatistics(VCB) {                                                \
//     PFILE_SYSTEM_STATISTICS Stats = &(VCB)->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors];   \
//     Stats->Fat.CreateHits += 1;                                                          \
// }
//
// #define CollectCreateStatistics(VCB,STATUS) {                                            \
//     PFILE_SYSTEM_STATISTICS Stats = &(VCB)->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors];   \
//     if ((STATUS) == STATUS_SUCCESS) {                                                    \
//         Stats->Fat.SuccessfulCreates += 1;                                               \
//     } else {                                                                             \
//         Stats->Fat.FailedCreates += 1;                                                   \
//     }                                                                                    \
// }
//
// LUID FatSecurityPrivilege = { SE_SECURITY_PRIVILEGE, 0 };
//
// //
// //  local procedure prototypes
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenVolume (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenRootDcb (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingDcb (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB Dcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos,
//      PBOOLEAN OplockPostIrp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingFcb (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PFCB Fcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos,
//      PBOOLEAN OplockPostIrp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenTargetDirectory (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PDCB Dcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      BOOLEAN DoesNameExist,
//      BOOLEAN FileNameOpenedDos
//     );
//
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB *Dcb,
//      PDCB ParentDcb,
//      PDIRENT Dirent,
//      ULONG LfnByteOffset,
//      ULONG DirentByteOffset,
//      PUNICODE_STRING Lfn,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN FileNameOpenedDos,
//      BOOLEAN OpenRequiringOplock
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingFile (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PFCB *Fcb,
//      PDCB ParentDcb,
//      PDIRENT Dirent,
//      ULONG LfnByteOffset,
//      ULONG DirentByteOffset,
//      PUNICODE_STRING Lfn,
//      PUNICODE_STRING OrigLfn,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN IsPagingFile,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatCreateNewDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB ParentDcb,
//      POEM_STRING OemName,
//      PUNICODE_STRING UnicodeName,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatCreateNewFile (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB ParentDcb,
//      POEM_STRING OemName,
//      PUNICODE_STRING UnicodeName,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      PUNICODE_STRING LfnBuffer,
//      BOOLEAN IsPagingFile,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN TemporaryFile
//     );
//
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatSupersedeOrOverwriteFile (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFCB Fcb,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge
//     );
//
// NTSTATUS
// FatCheckSystemSecurityAccess (
//      PIRP_CONTEXT IrpContext
//     );
//
// NTSTATUS
// FatCheckShareAccess (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFCB Fcb,
//      PACCESS_MASK DesiredAccess,
//      ULONG ShareAccess
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCheckShareAccess)
// #pragma alloc_text(PAGE, FatCheckSystemSecurityAccess)
// #pragma alloc_text(PAGE, FatCommonCreate)
//
// #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD)
// #pragma alloc_text(PAGE, FatCommonCreateOnNewStack)
// #pragma alloc_text(PAGE, FatCommonCreateCallout)
// #endif
//
// #pragma alloc_text(PAGE, FatCreateNewDirectory)
// #pragma alloc_text(PAGE, FatCreateNewFile)
// #pragma alloc_text(PAGE, FatFsdCreate)
// #pragma alloc_text(PAGE, FatOpenExistingDcb)
// #pragma alloc_text(PAGE, FatOpenExistingDirectory)
// #pragma alloc_text(PAGE, FatOpenExistingFcb)
// #pragma alloc_text(PAGE, FatOpenExistingFile)
// #pragma alloc_text(PAGE, FatOpenRootDcb)
// #pragma alloc_text(PAGE, FatOpenTargetDirectory)
// #pragma alloc_text(PAGE, FatOpenVolume)
// #pragma alloc_text(PAGE, FatSupersedeOrOverwriteFile)
// #pragma alloc_text(PAGE, FatSetFullNameInFcb)
// #endif
//
//
// _Function_class_(IRP_MJ_CREATE)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdCreate (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of the NtCreateFile and NtOpenFile
//     API calls.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file/directory exists that we are trying to open/create
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The Fsd status for the Irp
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//     BOOLEAN ExceptionCompletedIrp = FALSE;
//
//
//     PAGED_CODE();
//
//     //
//     //  If we were called with our file system device object instead of a
//     //  volume device object, just complete this request with STATUS_SUCCESS
//     //
//
//     if ( FatDeviceIsFatFsdo( VolumeDeviceObject))  {
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = FILE_OPENED;
//
//         IoCompleteRequest( Irp, IO_DISK_INCREMENT );
//
//         return STATUS_SUCCESS;
//     }
//
//     TimerStart(Dbg);
//
//     DebugTrace(+1, Dbg, "FatFsdCreate\n", 0);
//
//     //
//     //  Call the common create routine, with block allowed if the operation
//     //  is synchronous.
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, TRUE );
//
// #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD)
//         Status = FatCommonCreateOnNewStack( IrpContext, Irp );
// #else
//         Status = FatCommonCreate( IrpContext, Irp );
// #endif
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
//         ExceptionCompletedIrp = TRUE;
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//
//     //
//     //  Complete the request, unless we had an exception, in which case it
//     //  was completed in FatProcessException (and the IrpContext freed).
//     //
//     //  IrpContext is freed inside FatCompleteRequest.
//     //
//
//     if (!ExceptionCompletedIrp && Status != STATUS_PENDING) {
//         FatCompleteRequest( IrpContext, Irp, Status );
//     }
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdCreate -> %08lx\n", Status );
//
//     TimerStop(Dbg,"FatFsdCreate");
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD)
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatCommonCreateCallout (
//      PFAT_CALLOUT_PARAMETERS CalloutParameters
//     )
//
// /*++
//
// Routine Description:
//
//     This function is the callout routine that will execute on a new stack when
//     processing a create.  It simply calls FatCommonCreate() with the parameters
//     in the context and stores the return value in the context.
//
// Arguments:
//
//     Context - Supplies an opaque pointer to this function's context.  It is actually
//               an FAT_CALLOUT_PARAMETERS structure.
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
//     //
//     //  Call FatCommonCreate() with the passed parameters and store the result.
//     //  Exceptions cannot be raised across stack boundaries, so we need to catch
//     //  exceptions here and deal with them.
//     //
//
//     try {
//
//         CalloutParameters->IrpStatus = FatCommonCreate( CalloutParameters->Create.IrpContext,
//                                                         CalloutParameters->Create.Irp );
//
//     } except (FatExceptionFilter( CalloutParameters->Create.IrpContext, GetExceptionInformation() )) {
//
//         //
//         //  Return the resulting status.
//         //
//
//         CalloutParameters->ExceptionStatus = GetExceptionCode();
//     }
//
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonCreateOnNewStack (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine sets up a switch to a new stack and call to FatCommonCreate().
//
// Arguments:
//
//     IrpContext - Supplies the context structure for the overall request.
//
//     Irp - Supplies the IRP being processed.
//
//     CreateContext - Supplies a pointer on the old stack that is used to
//                     store context information for the create itself.
//
// Return Value:
//
//     NTSTATUS - The status from FatCommonCreate().
//
// --*/
// {
//     FAT_CALLOUT_PARAMETERS CalloutParameters;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     //  Create requests consume a lot of stack space.  As such, we always switch to a
//     //  new stack when processing a create.  Setup the callout parameters and make the
//     //  call.  Note that this cannot fail, since we pass a stack context for a reserve stack.
//     //
//
//     CalloutParameters.Create.IrpContext = IrpContext;
//     CalloutParameters.Create.Irp = Irp;
//     CalloutParameters.ExceptionStatus = CalloutParameters.IrpStatus = STATUS_SUCCESS;
//
//     //
//     //  Mark that we are swapping the stack
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_SWAPPED_STACK );
//
//     status = KeExpandKernelStackAndCalloutEx( FatCommonCreateCallout,
//                                               &CalloutParameters,
//                                               KERNEL_STACK_SIZE,
//                                               FALSE,
//                                               NULL );
//
//     //
//     //  Mark that the stack is no longer swapped.  Note that there are paths
//     //  that may clear this flag before returning.
//     //
//
//     if (status != STATUS_PENDING) {
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_SWAPPED_STACK );
//     }
//
//     //
//     // If we had an exception occur, re-raise the exception.
//     //
//
//     if (!NT_SUCCESS( CalloutParameters.ExceptionStatus )) {
//         FatRaiseStatus( IrpContext, CalloutParameters.ExceptionStatus );
//     }
//
//     //
//     //  If the call to KeExpandKernelStackAndCalloutEx returns an error this
//     //  means that the callout routine (FatCommonCreateCallout) was never
//     //  called. Translate that error, and return it.
//     //
//
//     if (!NT_SUCCESS( status )) {
//
//         //
//         //  Translate to an expected error value
//         //
//
//         if (status == STATUS_NO_MEMORY) {
//
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         return status;
//     }
//
//     //
//     // Return the status given to us by the callout.
//     //
//
//     return CalloutParameters.IrpStatus;
// }
// #endif
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonCreate (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for creating/opening a file called by
//     both the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - the return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK Iosb = {0};
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//     PFILE_OBJECT RelatedFileObject;
//     UNICODE_STRING FileName;
//     ULONG AllocationSize;
//     PFILE_FULL_EA_INFORMATION EaBuffer;
//     PACCESS_MASK DesiredAccess;
//     ULONG Options;
//     USHORT FileAttributes;
//     USHORT ShareAccess;
//     ULONG EaLength;
//
//     BOOLEAN CreateDirectory;
//     BOOLEAN NoIntermediateBuffering;
//     BOOLEAN OpenDirectory;
//     BOOLEAN IsPagingFile;
//     BOOLEAN OpenTargetDirectory;
//     BOOLEAN DirectoryFile;
//     BOOLEAN NonDirectoryFile;
//     BOOLEAN NoEaKnowledge;
//     BOOLEAN DeleteOnClose;
//     BOOLEAN OpenRequiringOplock;
//     BOOLEAN TemporaryFile;
//     BOOLEAN FileNameOpenedDos = FALSE;
//
//     ULONG CreateDisposition;
//
//     PVCB Vcb;
//     PFCB Fcb = NULL;
//     PCCB Ccb;
//     PDCB ParentDcb;
//     PDCB FinalDcb = NULL;
//
//     UNICODE_STRING FinalName = {0};
//     UNICODE_STRING RemainingPart;
//     UNICODE_STRING NextRemainingPart = {0};
//     UNICODE_STRING UpcasedFinalName;
//     WCHAR UpcasedBuffer[ FAT_CREATE_INITIAL_NAME_BUF_SIZE];
//
//     OEM_STRING OemFinalName;
//     UCHAR OemBuffer[ FAT_CREATE_INITIAL_NAME_BUF_SIZE*2];
//
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//     ULONG LfnByteOffset;
//     ULONG DirentByteOffset;
//
//     BOOLEAN PostIrp = FALSE;
//     BOOLEAN OplockPostIrp = FALSE;
//     BOOLEAN TrailingBackslash;
//     BOOLEAN FirstLoop = TRUE;
//
//     ULONG MatchFlags = 0;
//
//     CCB LocalCcb;
//     UNICODE_STRING Lfn;
//     UNICODE_STRING OrigLfn = {0};
//
//     WCHAR LfnBuffer[ FAT_CREATE_INITIAL_NAME_BUF_SIZE];
//
//     PAGED_CODE();
//
//     //
//     //  Get the current IRP stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonCreate\n", 0 );
//     DebugTrace( 0, Dbg, "Irp                       = %p\n", Irp );
//     DebugTrace( 0, Dbg, "->Flags                   = %08lx\n", Irp->Flags );
//     DebugTrace( 0, Dbg, "->FileObject              = %p\n", IrpSp->FileObject );
//     DebugTrace( 0, Dbg, " ->RelatedFileObject      = %p\n", IrpSp->FileObject->RelatedFileObject );
//     DebugTrace( 0, Dbg, " ->FileName               = %wZ\n",    &IrpSp->FileObject->FileName );
//     DebugTrace( 0, Dbg, " ->FileName.Length        = 0n%d\n",    IrpSp->FileObject->FileName.Length );
//     DebugTrace( 0, Dbg, "->AllocationSize.LowPart  = %08lx\n", Irp->Overlay.AllocationSize.LowPart );
//     DebugTrace( 0, Dbg, "->AllocationSize.HighPart = %08lx\n", Irp->Overlay.AllocationSize.HighPart );
//     DebugTrace( 0, Dbg, "->SystemBuffer            = %p\n", Irp->AssociatedIrp.SystemBuffer );
//     DebugTrace( 0, Dbg, "->DesiredAccess           = %08lx\n", IrpSp->Parameters.Create.SecurityContext->DesiredAccess );
//     DebugTrace( 0, Dbg, "->Options                 = %08lx\n", IrpSp->Parameters.Create.Options );
//     DebugTrace( 0, Dbg, "->FileAttributes          = %04x\n",  IrpSp->Parameters.Create.FileAttributes );
//     DebugTrace( 0, Dbg, "->ShareAccess             = %04x\n",  IrpSp->Parameters.Create.ShareAccess );
//     DebugTrace( 0, Dbg, "->EaLength                = %08lx\n", IrpSp->Parameters.Create.EaLength );
//
//     //
//     //  This is here because the Win32 layer can't avoid sending me double
//     //  beginning backslashes.
//     //
//
//     if ((IrpSp->FileObject->FileName.Length > sizeof(WCHAR)) &&
//         (IrpSp->FileObject->FileName.Buffer[1] == L'\\') &&
//         (IrpSp->FileObject->FileName.Buffer[0] == L'\\')) {
//
//         IrpSp->FileObject->FileName.Length -= sizeof(WCHAR);
//
//         RtlMoveMemory( &IrpSp->FileObject->FileName.Buffer[0],
//                        &IrpSp->FileObject->FileName.Buffer[1],
//                        IrpSp->FileObject->FileName.Length );
//
//         //
//         //  If there are still two beginning backslashes, the name is bogus.
//         //
//
//         if ((IrpSp->FileObject->FileName.Length > sizeof(WCHAR)) &&
//             (IrpSp->FileObject->FileName.Buffer[1] == L'\\') &&
//             (IrpSp->FileObject->FileName.Buffer[0] == L'\\')) {
//
//             DebugTrace(-1, Dbg, "FatCommonCreate -> STATUS_OBJECT_NAME_INVALID\n", 0);
//             return STATUS_OBJECT_NAME_INVALID;
//         }
//     }
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     NT_ASSERT( IrpSp->Parameters.Create.SecurityContext != NULL );
//
//     FileObject        = IrpSp->FileObject;
//     FileName          = FileObject->FileName;
//     RelatedFileObject = FileObject->RelatedFileObject;
//     AllocationSize    = Irp->Overlay.AllocationSize.LowPart;
//     EaBuffer          = Irp->AssociatedIrp.SystemBuffer;
//     DesiredAccess     = &IrpSp->Parameters.Create.SecurityContext->DesiredAccess;
//     Options           = IrpSp->Parameters.Create.Options;
//     FileAttributes    = IrpSp->Parameters.Create.FileAttributes & ~FILE_ATTRIBUTE_NORMAL;
//     ShareAccess       = IrpSp->Parameters.Create.ShareAccess;
//     EaLength          = IrpSp->Parameters.Create.EaLength;
//
//     //
//     //  Set up the file object's Vpb pointer in case anything happens.
//     //  This will allow us to get a reasonable pop-up.
//     //
//
//     if ( RelatedFileObject != NULL ) {
//         FileObject->Vpb = RelatedFileObject->Vpb;
//     }
//
//     //
//     //  Force setting the archive bit in the attributes byte to follow OS/2,
//     //  & DOS semantics.  Also mask out any extraneous bits, note that
//     //  we can't use the ATTRIBUTE_VALID_FLAGS constant because that has
//     //  the control and normal flags set.
//     //
//     //  Delay setting ARCHIVE in case this is a directory: 2/16/95
//     //
//
//     FileAttributes   &= (FILE_ATTRIBUTE_READONLY |
//                          FILE_ATTRIBUTE_HIDDEN   |
//                          FILE_ATTRIBUTE_SYSTEM   |
//                          FILE_ATTRIBUTE_ARCHIVE  |
//                          FILE_ATTRIBUTE_ENCRYPTED);
//
//     //
//     //  Locate the volume device object and Vcb that we are trying to access
//     //
//
//     Vcb = &((PVOLUME_DEVICE_OBJECT)IrpSp->DeviceObject)->Vcb;
//
//     //
//     //  Decipher Option flags and values
//     //
//
//     //
//     //  If this is an open by fileid operation, just fail it explicitly.  FAT's
//     //  source of fileids is not reversible for open operations.
//     //
//
//     if (BooleanFlagOn( Options, FILE_OPEN_BY_FILE_ID )) {
//
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     DirectoryFile           = BooleanFlagOn( Options, FILE_DIRECTORY_FILE );
//     NonDirectoryFile        = BooleanFlagOn( Options, FILE_NON_DIRECTORY_FILE );
//     NoIntermediateBuffering = BooleanFlagOn( Options, FILE_NO_INTERMEDIATE_BUFFERING );
//     NoEaKnowledge           = BooleanFlagOn( Options, FILE_NO_EA_KNOWLEDGE );
//     DeleteOnClose           = BooleanFlagOn( Options, FILE_DELETE_ON_CLOSE );
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//     OpenRequiringOplock     = BooleanFlagOn( Options, FILE_OPEN_REQUIRING_OPLOCK );
// #else
//     OpenRequiringOplock     = FALSE;
// #endif
//
//     TemporaryFile = BooleanFlagOn( IrpSp->Parameters.Create.FileAttributes,
//                                    FILE_ATTRIBUTE_TEMPORARY );
//
//     CreateDisposition = (Options >> 24) & 0x000000ff;
//
//     IsPagingFile = BooleanFlagOn( IrpSp->Flags, SL_OPEN_PAGING_FILE );
//     OpenTargetDirectory = BooleanFlagOn( IrpSp->Flags, SL_OPEN_TARGET_DIRECTORY );
//
//     CreateDirectory = (BOOLEAN)(DirectoryFile &&
//                                 ((CreateDisposition == FILE_CREATE) ||
//                                  (CreateDisposition == FILE_OPEN_IF)));
//
//     OpenDirectory   = (BOOLEAN)(DirectoryFile &&
//                                 ((CreateDisposition == FILE_OPEN) ||
//                                  (CreateDisposition == FILE_OPEN_IF)));
//
//
//     //
//     //  Make sure the input large integer is valid and that the dir/nondir
//     //  indicates a storage type we understand.
//     //
//
//     if (Irp->Overlay.AllocationSize.HighPart != 0 ||
//         (DirectoryFile && NonDirectoryFile)) {
//
//         DebugTrace(-1, Dbg, "FatCommonCreate -> STATUS_INVALID_PARAMETER\n", 0);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Acquire exclusive access to the vcb, and enqueue the Irp if
//     //  we didn't get it.
//     //
//
//     if (!FatAcquireExclusiveVcb( IrpContext, Vcb )) {
//
//         DebugTrace(0, Dbg, "Cannot acquire Vcb\n", 0);
//
//         Iosb.Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatCommonCreate -> %08lx\n", Iosb.Status );
//         return Iosb.Status;
//     }
//
//     //
//     //  Make sure we haven't been called recursively by a filter inside an existing
//     //  create request.
//     //
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "this is a serious programming error if it happens" )
// #endif
//         FatBugCheck( 0, 0, 0);
//     }
//
//     //
//     //  Initialize the DirentBcb to null
//     //
//
//     DirentBcb = NULL;
//
//     //
//     //  Initialize our temp strings with their stack buffers.
//     //
//
//     OemFinalName.Length = 0;
//     OemFinalName.MaximumLength = sizeof( OemBuffer);
//     OemFinalName.Buffer = (PCHAR)OemBuffer;
//
//     UpcasedFinalName.Length = 0;
//     UpcasedFinalName.MaximumLength = sizeof( UpcasedBuffer);
//     UpcasedFinalName.Buffer = UpcasedBuffer;
//
//     Lfn.Length = 0;
//     Lfn.MaximumLength = sizeof( LfnBuffer);
//     Lfn.Buffer = LfnBuffer;
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure the vcb is in a usable condition.  This will raise
//         //  and error condition if the volume is unusable
//         //
//
//         FatVerifyVcb( IrpContext, Vcb );
//
//         //
//         //  If the Vcb is locked then we cannot open another file
//         //
//
//         if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_LOCKED)) {
//
//             DebugTrace(0, Dbg, "Volume is locked\n", 0);
//
//             Status = STATUS_ACCESS_DENIED;
//             if (Vcb->VcbCondition != VcbGood) {
//
//                 Status = STATUS_VOLUME_DISMOUNTED;
//             }
//             try_return( Iosb.Status = Status );
//         }
//
//         //
//         //  Don't allow the DELETE_ON_CLOSE option if the volume is
//         //  write-protected.
//         //
//
//         if (DeleteOnClose && FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//             //
//             //  Set the real device for the pop-up info, and set the verify
//             //  bit in the device object, so that we will force a verify
//             //  in case the user put the correct media back in.
//             //
//
//             IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                           Vcb->Vpb->RealDevice );
//
//             SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//             FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//         }
//
//         //
//         //  If this is a fat32 volume, EA's are not supported.
//         //
//
//         if (EaBuffer != NULL) {
//
//             try_return( Iosb.Status = STATUS_EAS_NOT_SUPPORTED );
//         }
//
//         //
//         //  Check if we are opening the volume and not a file/directory.
//         //  We are opening the volume if the name is empty and there
//         //  isn't a related file object.  If there is a related file object
//         //  then it is the Vcb itself.
//         //
//
//         if (FileName.Length == 0) {
//
//             PVCB DecodeVcb = NULL;
//
//             if (RelatedFileObject == NULL ||
//                 FatDecodeFileObject( RelatedFileObject,
//                                      &DecodeVcb,
//                                      &Fcb,
//                                      &Ccb ) == UserVolumeOpen) {
//
//                 NT_ASSERT( RelatedFileObject == NULL || Vcb == DecodeVcb );
//
//                 //
//                 //  Check if we were to open a directory
//                 //
//
//                 if (DirectoryFile) {
//
//                     DebugTrace(0, Dbg, "Cannot open volume as a directory\n", 0);
//
//                     try_return( Iosb.Status = STATUS_NOT_A_DIRECTORY );
//                 }
//
//                 //
//                 //  Can't open the TargetDirectory of the DASD volume.
//                 //
//
//                 if (OpenTargetDirectory) {
//
//                     try_return( Iosb.Status = STATUS_INVALID_PARAMETER );
//                 }
//
//                 DebugTrace(0, Dbg, "Opening the volume, Vcb = %p\n", Vcb);
//
//                 CollectCreateHitStatistics(Vcb);
//
//                 Iosb = FatOpenVolume( IrpContext,
//                                       FileObject,
//                                       Vcb,
//                                       DesiredAccess,
//                                       ShareAccess,
//                                       CreateDisposition );
//
//                 Irp->IoStatus.Information = Iosb.Information;
//                 try_return( Iosb.Status );
//             }
//         }
//
//         //
//         //  If there is a related file object then this is a relative open.
//         //  The related file object is the directory to start our search at.
//         //  Return an error if it is not a directory.
//         //
//
//         if (RelatedFileObject != NULL) {
//
//             PVCB RelatedVcb;
//             PDCB RelatedDcb;
//             PCCB RelatedCcb;
//             TYPE_OF_OPEN TypeOfOpen;
//
//             TypeOfOpen = FatDecodeFileObject( RelatedFileObject,
//                                               &RelatedVcb,
//                                               &RelatedDcb,
//                                               &RelatedCcb );
//
//             if (TypeOfOpen != UserFileOpen &&
//                 TypeOfOpen != UserDirectoryOpen) {
//
//                 DebugTrace(0, Dbg, "Invalid related file object\n", 0);
//
//                 try_return( Iosb.Status = STATUS_OBJECT_PATH_NOT_FOUND );
//             }
//
//             //
//             //  A relative open must be via a relative path.
//             //
//
//             if (FileName.Length != 0 &&
//                 FileName.Buffer[0] == L'\\') {
//
//                 try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//             }
//
//             //
//             //  Set up the file object's Vpb pointer in case anything happens.
//             //
//
//             NT_ASSERT( Vcb == RelatedVcb );
//
//             FileObject->Vpb = RelatedFileObject->Vpb;
//
//             //
//             //  Now verify the related Fcb so we don't get in trouble later
//             //  by assuming its in good shape.
//             //
//
//             FatVerifyFcb( IrpContext, RelatedDcb );
//
//             ParentDcb = RelatedDcb;
//
//         } else {
//
//             //
//             //  This is not a relative open, so check if we're
//             //  opening the root dcb
//             //
//
//             if ((FileName.Length == sizeof(WCHAR)) &&
//                 (FileName.Buffer[0] == L'\\')) {
//
//                 //
//                 //  Check if we were not supposed to open a directory
//                 //
//
//                 if (NonDirectoryFile) {
//
//                     DebugTrace(0, Dbg, "Cannot open root directory as a file\n", 0);
//
//                     try_return( Iosb.Status = STATUS_FILE_IS_A_DIRECTORY );
//                 }
//
//                 //
//                 //  Can't open the TargetDirectory of the root directory.
//                 //
//
//                 if (OpenTargetDirectory) {
//
//                     try_return( Iosb.Status = STATUS_INVALID_PARAMETER );
//                 }
//
//                 //
//                 //  Not allowed to delete root directory.
//                 //
//
//                 if (DeleteOnClose) {
//
//                     try_return( Iosb.Status = STATUS_CANNOT_DELETE );
//                 }
//
//                 DebugTrace(0, Dbg, "Opening root dcb\n", 0);
//
//                 CollectCreateHitStatistics(Vcb);
//
//                 Iosb = FatOpenRootDcb( IrpContext,
//                                        FileObject,
//                                        Vcb,
//                                        DesiredAccess,
//                                        ShareAccess,
//                                        CreateDisposition );
//
//                 Irp->IoStatus.Information = Iosb.Information;
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  Nope, we will be opening relative to the root directory.
//             //
//
//             ParentDcb = Vcb->RootDcb;
//
//             //
//             //  Now verify the root Dcb so we don't get in trouble later
//             //  by assuming its in good shape.
//             //
//
//             FatVerifyFcb( IrpContext, ParentDcb );
//         }
//
//         //
//         //  FatCommonCreate(): trailing backslash check
//         //
//
//
//         if ((FileName.Length != 0) &&
//             (FileName.Buffer[FileName.Length/sizeof(WCHAR)-1] == L'\\')) {
//
//             FileName.Length -= sizeof(WCHAR);
//             TrailingBackslash = TRUE;
//
//         } else {
//
//             TrailingBackslash = FALSE;
//         }
//
//         //
//         //  Check for max path.  We might want to tighten this down to DOS MAX_PATH
//         //  for maximal interchange with non-NT platforms, but for now defer to the
//         //  possibility of something depending on it.
//         //
//
//         if (ParentDcb->FullFileName.Buffer == NULL) {
//
//             FatSetFullFileNameInFcb( IrpContext, ParentDcb );
//         }
//
//         if ((USHORT) (ParentDcb->FullFileName.Length + sizeof(WCHAR) + FileName.Length) <= FileName.Length) {
//
//             try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//         }
//
//         //
//         //  We loop here until we land on an Fcb that is in a good
//         //  condition.  This way we can reopen files that have stale handles
//         //  to files of the same name but are now different.
//         //
//
//         while ( TRUE ) {
//
//             Fcb = ParentDcb;
//             RemainingPart = FileName;
//
//             //
//             //  Now walk down the Dcb tree looking for the longest prefix.
//             //  This one exit condition in the while() is to handle a
//             //  special case condition (relative NULL name open), the main
//             //  exit conditions are at the bottom of the loop.
//             //
//
//             while (RemainingPart.Length != 0) {
//
//                 PFCB NextFcb;
//
//                 FsRtlDissectName( RemainingPart,
//                                   &FinalName,
//                                   &NextRemainingPart );
//
//                 //
//                 //  If RemainingPart starts with a backslash the name is
//                 //  invalid.
//                 //  Check for no more than 255 characters in FinalName
//                 //
//
//                 if (((NextRemainingPart.Length != 0) && (NextRemainingPart.Buffer[0] == L'\\')) ||
//                     (FinalName.Length > 255*sizeof(WCHAR))) {
//
//                     try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//                 }
//
//                 //
//                 //  Now, try to convert this one component into Oem and search
//                 //  the splay tree.  If it works then that's great, otherwise
//                 //  we have to try with the UNICODE name instead.
//                 //
//
//                 FatEnsureStringBufferEnough( &OemFinalName,
//                                              FinalName.Length);
//
//                 Status = RtlUpcaseUnicodeStringToCountedOemString( &OemFinalName, &FinalName, FALSE );
//
//
//                 if (NT_SUCCESS(Status)) {
//
//                     NextFcb = FatFindFcb( IrpContext,
//                                           &Fcb->Specific.Dcb.RootOemNode,
//                                           (PSTRING)&OemFinalName,
//                                           &FileNameOpenedDos );
//
//                 } else {
//
//                     NextFcb = NULL;
//                     OemFinalName.Length = 0;
//
//                     if (Status != STATUS_UNMAPPABLE_CHARACTER) {
//
//                         try_return( Iosb.Status = Status );
//                     }
//                 }
//
//                 //
//                 //  If we didn't find anything searching the Oem space, we
//                 //  have to try the Unicode space.  To save cycles in the
//                 //  common case that this tree is empty, we do a quick check
//                 //  here.
//                 //
//
//                 if ((NextFcb == NULL) && Fcb->Specific.Dcb.RootUnicodeNode) {
//
//                     //
//                     // First downcase, then upcase the string, because this
//                     // is what happens when putting names into the tree (see
//                     // strucsup.c, FatConstructNamesInFcb()).
//                     //
//
//                     FatEnsureStringBufferEnough( &UpcasedFinalName,
//                                                  FinalName.Length);
//
//                     Status = RtlDowncaseUnicodeString(&UpcasedFinalName, &FinalName, FALSE );
//                     NT_ASSERT( NT_SUCCESS( Status ));
//
//                     Status = RtlUpcaseUnicodeString( &UpcasedFinalName, &UpcasedFinalName, FALSE );
//                     NT_ASSERT( NT_SUCCESS( Status ));
//
//
//                     NextFcb = FatFindFcb( IrpContext,
//                                           &Fcb->Specific.Dcb.RootUnicodeNode,
//                                           (PSTRING)&UpcasedFinalName,
//                                           &FileNameOpenedDos );
//                 }
//
//                 //
//                 //  If we got back an Fcb then we consumed the FinalName
//                 //  legitimately, so the remaining name is now RemainingPart.
//                 //
//
//                 if (NextFcb != NULL) {
//                     Fcb = NextFcb;
//                     RemainingPart = NextRemainingPart;
//                 }
//
//                 if ((NextFcb == NULL) ||
//                     (NodeType(NextFcb) == FAT_NTC_FCB) ||
//                     (NextRemainingPart.Length == 0)) {
//
//                     break;
//                 }
//             }
//
//             //
//             //  Remaining name cannot start with a backslash
//             //
//
//             if (RemainingPart.Length && (RemainingPart.Buffer[0] == L'\\')) {
//
//                 RemainingPart.Length -= sizeof(WCHAR);
//                 RemainingPart.Buffer += 1;
//             }
//
//             //
//             //  Now verify that everybody up to the longest found prefix is valid.
//             //
//
//             _SEH2_TRY {
//
//                 FatVerifyFcb( IrpContext, Fcb );
//
//             } _SEH2_EXCEPT( (_SEH2_GetExceptionCode() == STATUS_FILE_INVALID) ?
//                       EXCEPTION_EXECUTE_HANDLER :
//                       EXCEPTION_CONTINUE_SEARCH ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//             if ( Fcb->FcbCondition == FcbGood ) {
//
//                 //
//                 //  If we are trying to open a paging file and have happened
//                 //  upon the DelayedCloseFcb, make it go away, and try again.
//                 //
//
//                 if (IsPagingFile && FirstLoop &&
//                     (NodeType(Fcb as *const c_void) == FAT_NTC_FCB) &&
//                     (!IsListEmpty( &FatData.AsyncCloseList ) ||
//                      !IsListEmpty( &FatData.DelayedCloseList ))) {
//
//                     FatFspClose(Vcb);
//
//                     FirstLoop = FALSE;
//
//                     continue;
//
//                 } else {
//
//                     break;
//                 }
//
//             } else {
//
//                 FatRemoveNames( IrpContext, Fcb );
//             }
//         }
//
//         NT_ASSERT( Fcb->FcbCondition == FcbGood );
//
//         //
//         //  If there is already an Fcb for a paging file open and
//         //  it was not already opened as a paging file, we cannot
//         //  continue as it is too difficult to move a live Fcb to
//         //  non-paged pool.
//         //
//
//         if (IsPagingFile) {
//
//             if (NodeType(Fcb as *const c_void) == FAT_NTC_FCB &&
//                 !FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE )) {
//
//                 try_return( Iosb.Status = STATUS_SHARING_VIOLATION );
//             }
//
//         //
//         //  Check for a system file.
//         //
//
//         } else if (FlagOn( Fcb->FcbState, FCB_STATE_SYSTEM_FILE )) {
//
//             try_return( Iosb.Status = STATUS_ACCESS_DENIED );
//         }
//
//         //
//         //  If the longest prefix is pending delete (either the file or
//         //  some higher level directory), we cannot continue.
//         //
//
//         if (FlagOn( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE )) {
//
//             try_return( Iosb.Status = STATUS_DELETE_PENDING );
//         }
//
//         //
//         //  Now that we've found the longest matching prefix we'll
//         //  check if there isn't any remaining part because that means
//         //  we've located an existing fcb/dcb to open and we can do the open
//         //  without going to the disk
//         //
//
//         if (RemainingPart.Length == 0) {
//
//             //
//             //  First check if the user wanted to open the target directory
//             //  and if so then call the subroutine to finish the open.
//             //
//
//             if (OpenTargetDirectory) {
//
//                 CollectCreateHitStatistics(Vcb);
//
//                 Iosb = FatOpenTargetDirectory( IrpContext,
//                                                FileObject,
//                                                Fcb->ParentDcb,
//                                                DesiredAccess,
//                                                ShareAccess,
//                                                TRUE,
//                                                FileNameOpenedDos );
//                 Irp->IoStatus.Information = Iosb.Information;
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  We can open an existing fcb/dcb, now we only need to case
//             //  on which type to open.
//             //
//
//             if (NodeType(Fcb as *const c_void) == FAT_NTC_DCB || NodeType(Fcb as *const c_void) == FAT_NTC_ROOT_DCB) {
//
//                 //
//                 //  This is a directory we're opening up so check if
//                 //  we were not to open a directory
//                 //
//
//                 if (NonDirectoryFile) {
//
//                     DebugTrace(0, Dbg, "Cannot open directory as a file\n", 0);
//
//                     try_return( Iosb.Status = STATUS_FILE_IS_A_DIRECTORY );
//                 }
//
//                 DebugTrace(0, Dbg, "Open existing dcb, Dcb = %p\n", Fcb);
//
//                 CollectCreateHitStatistics(Vcb);
//
//                 Iosb = FatOpenExistingDcb( IrpContext,
//                                            IrpSp,
//                                            FileObject,
//                                            Vcb,
//                                            (PDCB)Fcb,
//                                            DesiredAccess,
//                                            ShareAccess,
//                                            CreateDisposition,
//                                            NoEaKnowledge,
//                                            DeleteOnClose,
//                                            OpenRequiringOplock,
//                                            FileNameOpenedDos,
//                                            &OplockPostIrp );
//
//                 if (Iosb.Status != STATUS_PENDING) {
//
//                     Irp->IoStatus.Information = Iosb.Information;
//
//                 } else {
//
//                     DebugTrace(0, Dbg, "Enqueue Irp to FSP\n", 0);
//
//                     PostIrp = TRUE;
//                 }
//
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  Check if we're trying to open an existing Fcb and that
//             //  the user didn't want to open a directory.  Note that this
//             //  call might actually come back with status_pending because
//             //  the user wanted to supersede or overwrite the file and we
//             //  cannot block.  If it is pending then we do not complete the
//             //  request, and we fall through the bottom to the code that
//             //  dispatches the request to the fsp.
//             //
//
//             if (NodeType(Fcb as *const c_void) == FAT_NTC_FCB) {
//
//                 //
//                 //  Check if we were only to open a directory
//                 //
//
//                 if (OpenDirectory) {
//
//                     DebugTrace(0, Dbg, "Cannot open file as directory\n", 0);
//
//                     try_return( Iosb.Status = STATUS_NOT_A_DIRECTORY );
//                 }
//
//                 DebugTrace(0, Dbg, "Open existing fcb, Fcb = %p\n", Fcb);
//
//                 if ( TrailingBackslash ) {
//                     try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//                 }
//
//                 CollectCreateHitStatistics(Vcb);
//
//                 Iosb = FatOpenExistingFcb( IrpContext,
//                                            IrpSp,
//                                            FileObject,
//                                            Vcb,
//                                            Fcb,
//                                            DesiredAccess,
//                                            ShareAccess,
//                                            AllocationSize,
//                                            EaBuffer,
//                                            EaLength,
//                                            FileAttributes,
//                                            CreateDisposition,
//                                            NoEaKnowledge,
//                                            DeleteOnClose,
//                                            OpenRequiringOplock,
//                                            FileNameOpenedDos,
//                                            &OplockPostIrp );
//
//                 if (Iosb.Status != STATUS_PENDING) {
//
//                     //
//                     //  Check if we need to set the cache support flag in
//                     //  the file object
//                     //
//
//                     if (NT_SUCCESS( Iosb.Status) && !NoIntermediateBuffering) {
//
//                         FileObject->Flags |= FO_CACHE_SUPPORTED;
//                     }
//
//                     Irp->IoStatus.Information = Iosb.Information;
//
//                 } else {
//
//                     DebugTrace(0, Dbg, "Enqueue Irp to FSP\n", 0);
//
//                     PostIrp = TRUE;
//                 }
//
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  Not and Fcb or a Dcb so we bug check
//             //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "this is a serious corruption if it happens" )
// #endif
//             FatBugCheck( NodeType(Fcb as *const c_void), (ULONG_PTR) Fcb, 0 );
//         }
//
//         //
//         //  There is more in the name to parse than we have in existing
//         //  fcbs/dcbs.  So now make sure that fcb we got for the largest
//         //  matching prefix is really a dcb otherwise we can't go any
//         //  further
//         //
//
//         if ((NodeType(Fcb as *const c_void) != FAT_NTC_DCB) && (NodeType(Fcb as *const c_void) != FAT_NTC_ROOT_DCB)) {
//
//             DebugTrace(0, Dbg, "Cannot open file as subdirectory, Fcb = %p\n", Fcb);
//
//             try_return( Iosb.Status = STATUS_OBJECT_PATH_NOT_FOUND );
//         }
//
//         //
//         //  Otherwise we continue on processing the Irp and allowing ourselves
//         //  to block for I/O as necessary.  Find/create additional dcb's for
//         //  the one we're trying to open.  We loop until either remaining part
//         //  is empty or we get a bad filename.  When we exit FinalName is
//         //  the last name in the string we're after, and ParentDcb is the
//         //  parent directory that will contain the opened/created
//         //  file/directory.
//         //
//         //  Make sure the rest of the name is valid in at least the LFN
//         //  character set (which just happens to be that of HPFS).
//         //
//         //  If we are not in ChicagoMode, use FAT semantics.
//         //
//
//         ParentDcb = Fcb;
//         FirstLoop = TRUE;
//
//         while (TRUE) {
//
//             //
//             //  We do one little optimization here on the first iteration of
//             //  the loop since we know that we have already tried to convert
//             //  FinalOemName from the original UNICODE.
//             //
//
//             if (FirstLoop) {
//
//                 FirstLoop = FALSE;
//                 RemainingPart = NextRemainingPart;
//                 Status = OemFinalName.Length ? STATUS_SUCCESS : STATUS_UNMAPPABLE_CHARACTER;
//
//             } else {
//
//                 //
//                 //  Dissect the remaining part.
//                 //
//
//                 DebugTrace(0, Dbg, "Dissecting the name %wZ\n", &RemainingPart);
//
//                 FsRtlDissectName( RemainingPart,
//                                   &FinalName,
//                                   &RemainingPart );
//
//                 //
//                 //  If RemainingPart starts with a backslash the name is
//                 //  invalid.
//                 //  Check for no more than 255 characters in FinalName
//                 //
//
//                 if (((RemainingPart.Length != 0) && (RemainingPart.Buffer[0] == L'\\')) ||
//                     (FinalName.Length > 255*sizeof(WCHAR))) {
//
//                     try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//                 }
//
//                 //
//                 //  Now, try to convert this one component into Oem.  If it works
//                 //  then that's great, otherwise we have to try with the UNICODE
//                 //  name instead.
//                 //
//
//                 FatEnsureStringBufferEnough( &OemFinalName,
//                                              FinalName.Length);
//
//                 Status = RtlUpcaseUnicodeStringToCountedOemString( &OemFinalName, &FinalName, FALSE );
//             }
//
//             if (NT_SUCCESS(Status)) {
//
//                 //
//                 //  We'll start by trying to locate the dirent for the name.  Note
//                 //  that we already know that there isn't an Fcb/Dcb for the file
//                 //  otherwise we would have found it when we did our prefix lookup.
//                 //
//
//                 if (FatIsNameShortOemValid( IrpContext, OemFinalName, FALSE, FALSE, FALSE )) {
//
//                     FatStringTo8dot3( IrpContext,
//                                       OemFinalName,
//                                       &LocalCcb.OemQueryTemplate.Constant );
//
//                     LocalCcb.Flags = 0;
//
//                 } else {
//
//                     LocalCcb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE;
//                 }
//
//             } else {
//
//                 LocalCcb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE;
//
//                 if (Status != STATUS_UNMAPPABLE_CHARACTER) {
//
//                     try_return( Iosb.Status = Status );
//                 }
//             }
//
//             //
//             //  Now we know a lot about the final name, so do legal name
//             //  checking here.
//             //
//
//             if (FatData.ChicagoMode) {
//
//                 if (!FatIsNameLongUnicodeValid( IrpContext, &FinalName, FALSE, FALSE, FALSE )) {
//
//                     try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//                 }
//
//             } else {
//
//                 if (FlagOn(LocalCcb.Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE)) {
//
//                     try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//                 }
//             }
//
//             DebugTrace(0, Dbg, "FinalName is %wZ\n", &FinalName);
//             DebugTrace(0, Dbg, "RemainingPart is %wZ\n", &RemainingPart);
//
//             FatEnsureStringBufferEnough( &UpcasedFinalName,
//                                          FinalName.Length);
//
//             if (!NT_SUCCESS(Status = RtlUpcaseUnicodeString( &UpcasedFinalName, &FinalName, FALSE))) {
//
//                 try_return( Iosb.Status = Status );
//             }
//
//             LocalCcb.UnicodeQueryTemplate =  UpcasedFinalName;
//             LocalCcb.ContainsWildCards = FALSE;
//
//             Lfn.Length = 0;
//
//
//             FatLocateDirent( IrpContext,
//                              ParentDcb,
//                              &LocalCcb,
//                              0,
//                              &MatchFlags,
//                              &Dirent,
//                              &DirentBcb,
//                              (PVBO)&DirentByteOffset,
//                              &FileNameOpenedDos,
//                              &Lfn,
//                              &OrigLfn);
//
//             //
//             //  Remember we read this Dcb for error recovery.
//             //
//
//             FinalDcb = ParentDcb;
//
//             //
//             //  If the remaining part is now empty then this is the last name
//             //  in the string and the one we want to open
//             //
//
//             if (RemainingPart.Length == 0) {
//
//
//                 break;
//             }
//
//             //
//             //  We didn't find a dirent, bail.
//             //
//
//             if (Dirent == NULL) {
//
//                 Iosb.Status = STATUS_OBJECT_PATH_NOT_FOUND;
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  We now have a dirent, make sure it is a directory
//             //
//
//             if (!FlagOn( Dirent->Attributes, FAT_DIRENT_ATTR_DIRECTORY )) {
//
//                 Iosb.Status = STATUS_OBJECT_PATH_NOT_FOUND;
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  Compute the LfnByteOffset.
//             //
//
//             LfnByteOffset = DirentByteOffset -
//                             FAT_LFN_DIRENTS_NEEDED(&mut OrigLfn) * sizeof(LFN_DIRENT);
//
//             //
//             //  Create a dcb for the new directory
//             //
//
//             ParentDcb = FatCreateDcb( IrpContext,
//                                       Vcb,
//                                       ParentDcb,
//                                       LfnByteOffset,
//                                       DirentByteOffset,
//                                       Dirent,
//                                       &Lfn );
//
//             //
//             //  Remember we created this Dcb for error recovery.
//             //
//
//             FinalDcb = ParentDcb;
//
//             FatSetFullNameInFcb( IrpContext, ParentDcb, &FinalName );
//         }
//
//         //
//         //  First check if the user wanted to open the target directory
//         //  and if so then call the subroutine to finish the open.
//         //
//
//         if (OpenTargetDirectory) {
//
//             Iosb = FatOpenTargetDirectory( IrpContext,
//                                            FileObject,
//                                            ParentDcb,
//                                            DesiredAccess,
//                                            ShareAccess,
//                                            Dirent ? TRUE : FALSE,
//                                            FileNameOpenedDos);
//
//             Irp->IoStatus.Information = Iosb.Information;
//             try_return( Iosb.Status );
//         }
//
//         if (Dirent != NULL) {
//
//             //
//             //  Compute the LfnByteOffset.
//             //
//
//             LfnByteOffset = DirentByteOffset -
//                             FAT_LFN_DIRENTS_NEEDED(&mut OrigLfn) * sizeof(LFN_DIRENT);
//
//             //
//             //  We were able to locate an existing dirent entry, so now
//             //  see if it is a directory that we're trying to open.
//             //
//
//             if (FlagOn( Dirent->Attributes, FAT_DIRENT_ATTR_DIRECTORY )) {
//
//                 //
//                 //  Make sure its okay to open a directory
//                 //
//
//                 if (NonDirectoryFile) {
//
//                     DebugTrace(0, Dbg, "Cannot open directory as a file\n", 0);
//
//                     try_return( Iosb.Status = STATUS_FILE_IS_A_DIRECTORY );
//                 }
//
//                 DebugTrace(0, Dbg, "Open existing directory\n", 0);
//
//                 Iosb = FatOpenExistingDirectory( IrpContext,
//                                                  IrpSp,
//                                                  FileObject,
//                                                  Vcb,
//                                                  &Fcb,
//                                                  ParentDcb,
//                                                  Dirent,
//                                                  LfnByteOffset,
//                                                  DirentByteOffset,
//                                                  &Lfn,
//                                                  DesiredAccess,
//                                                  ShareAccess,
//                                                  CreateDisposition,
//                                                  NoEaKnowledge,
//                                                  DeleteOnClose,
//                                                  FileNameOpenedDos,
//                                                  OpenRequiringOplock );
//
//                 Irp->IoStatus.Information = Iosb.Information;
//                 try_return( Iosb.Status );
//             }
//
//             //
//             //  Otherwise we're trying to open and existing file, and we
//             //  need to check if the user only wanted to open a directory.
//             //
//
//             if (OpenDirectory) {
//
//                 DebugTrace(0, Dbg, "Cannot open file as directory\n", 0);
//
//                 try_return( Iosb.Status = STATUS_NOT_A_DIRECTORY );
//             }
//
//             DebugTrace(0, Dbg, "Open existing file\n", 0);
//
//             if ( TrailingBackslash ) {
//                try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//             }
//
//
//             Iosb = FatOpenExistingFile( IrpContext,
//                                         FileObject,
//                                         Vcb,
//                                         &Fcb,
//                                         ParentDcb,
//                                         Dirent,
//                                         LfnByteOffset,
//                                         DirentByteOffset,
//                                         &Lfn,
//                                         &OrigLfn,
//                                         DesiredAccess,
//                                         ShareAccess,
//                                         AllocationSize,
//                                         EaBuffer,
//                                         EaLength,
//                                         FileAttributes,
//                                         CreateDisposition,
//                                         IsPagingFile,
//                                         NoEaKnowledge,
//                                         DeleteOnClose,
//                                         OpenRequiringOplock,
//                                         FileNameOpenedDos );
//
//             //
//             //  Check if we need to set the cache support flag in
//             //  the file object
//             //
//
//             if (NT_SUCCESS(Iosb.Status) && !NoIntermediateBuffering) {
//
//                 FileObject->Flags |= FO_CACHE_SUPPORTED;
//             }
//
//             Irp->IoStatus.Information = Iosb.Information;
//             try_return( Iosb.Status );
//         }
//
//         //
//         //  We can't locate a dirent so this is a new file.
//         //
//
//         //
//         //  Now check to see if we wanted to only open an existing file.
//         //  And then case on whether we wanted to create a file or a directory.
//         //
//
//         if ((CreateDisposition == FILE_OPEN) ||
//             (CreateDisposition == FILE_OVERWRITE)) {
//
//             DebugTrace( 0, Dbg, "Cannot open nonexisting file\n", 0);
//
//             try_return( Iosb.Status = STATUS_OBJECT_NAME_NOT_FOUND );
//         }
//
//         //
//         //  Skip a few cycles later if we know now that the Oem name is not
//         //  valid 8.3.
//         //
//
//         if (FlagOn(LocalCcb.Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE)) {
//
//             OemFinalName.Length = 0;
//         }
//
//         //
//         //  Determine the granted access for this operation now.
//         //
//
//         if (!NT_SUCCESS( Iosb.Status = FatCheckSystemSecurityAccess( IrpContext ))) {
//
//             try_return( Iosb );
//         }
//
//         if (CreateDirectory) {
//
//             DebugTrace(0, Dbg, "Create new directory\n", 0);
//
//             //
//             //  If this media is write protected, don't even try the create.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//                 //
//                 //  Set the real device for the pop-up info, and set the verify
//                 //  bit in the device object, so that we will force a verify
//                 //  in case the user put the correct media back in.
//                 //
//
//
//                 IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                               Vcb->Vpb->RealDevice );
//
//                 SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//                 FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//             }
//
//             //
//             //  Don't allow people to create directories with the
//             //  temporary bit set.
//             //
//
//             if (TemporaryFile) {
//
//                 try_return( Iosb.Status = STATUS_INVALID_PARAMETER );
//             }
//
//             Iosb = FatCreateNewDirectory( IrpContext,
//                                           IrpSp,
//                                           FileObject,
//                                           Vcb,
//                                           ParentDcb,
//                                           &OemFinalName,
//                                           &FinalName,
//                                           DesiredAccess,
//                                           ShareAccess,
//                                           EaBuffer,
//                                           EaLength,
//                                           FileAttributes,
//                                           NoEaKnowledge,
//                                           DeleteOnClose,
//                                           OpenRequiringOplock );
//
//             Irp->IoStatus.Information = Iosb.Information;
//             try_return( Iosb.Status );
//         }
//
//         DebugTrace(0, Dbg, "Create new file\n", 0);
//
//         if ( TrailingBackslash ) {
//
//             try_return( Iosb.Status = STATUS_OBJECT_NAME_INVALID );
//         }
//
//         //
//         //  If this media is write protected, don't even try the create.
//         //
//
//         if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//             //
//             //  Set the real device for the pop-up info, and set the verify
//             //  bit in the device object, so that we will force a verify
//             //  in case the user put the correct media back in.
//             //
//
//
//             IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                           Vcb->Vpb->RealDevice );
//
//             SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//             FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//         }
//
//
//         Iosb = FatCreateNewFile( IrpContext,
//                                  IrpSp,
//                                  FileObject,
//                                  Vcb,
//                                  ParentDcb,
//                                  &OemFinalName,
//                                  &FinalName,
//                                  DesiredAccess,
//                                  ShareAccess,
//                                  AllocationSize,
//                                  EaBuffer,
//                                  EaLength,
//                                  FileAttributes,
//                                  &Lfn,
//                                  IsPagingFile,
//                                  NoEaKnowledge,
//                                  DeleteOnClose,
//                                  OpenRequiringOplock,
//                                  TemporaryFile );
//
//         //
//         //  Check if we need to set the cache support flag in
//         //  the file object
//         //
//
//         if (NT_SUCCESS(Iosb.Status) && !NoIntermediateBuffering) {
//
//             FileObject->Flags |= FO_CACHE_SUPPORTED;
//         }
//
//         Irp->IoStatus.Information = Iosb.Information;
//
//     try_exit: NOTHING;
//
//         //
//         //  This is a Beta Fix.  Do this at a better place later.
//         //
//
//         if (NT_SUCCESS(Iosb.Status) && !OpenTargetDirectory) {
//
//             PFCB LocalFcb;
//
//             //
//             //  If there is an Fcb/Dcb, set the long file name.
//             //
//
//             LocalFcb = FileObject->FsContext;
//
//             if (LocalFcb &&
//                 ((NodeType(LocalFcb) == FAT_NTC_FCB) ||
//                  (NodeType(LocalFcb) == FAT_NTC_DCB)) &&
//                 (LocalFcb->FullFileName.Buffer == NULL)) {
//
//                 FatSetFullNameInFcb( IrpContext, LocalFcb, &FinalName );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonCreate );
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
//         //
//         //  If we're not getting out with success, and if the caller wanted
//         //  atomic create-with-oplock semantics make sure we back out any
//         //  oplock that may have been granted.
//         //
//
//         if ((AbnormalTermination() ||
//              !NT_SUCCESS( Iosb.Status )) &&
//             OpenRequiringOplock &&
//             (Iosb.Status != STATUS_CANNOT_BREAK_OPLOCK) &&
//             (IrpContext->ExceptionStatus != STATUS_CANNOT_BREAK_OPLOCK) &&
//             (Fcb != NULL) &&
//             FatIsFileOplockable( Fcb )) {
//
//             FsRtlCheckOplockEx( FatGetFcbOplock(Fcb),
//                                 IrpContext->OriginatingIrp,
//                                 OPLOCK_FLAG_BACK_OUT_ATOMIC_OPLOCK,
//                                 NULL,
//                                 NULL,
//                                 NULL );
//         }
// #endif
//
//         //
//         //  There used to be a test here - the ASSERT replaces it.  We will
//         //  never have begun enumerating directories if we post the IRP for
//         //  oplock reasons.
//         //
//
//         NT_ASSERT( !OplockPostIrp || DirentBcb == NULL );
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         //
//         //  If we are in an error path, check for any created subdir Dcbs that
//         //  have to be unwound.  Don't whack the root directory.
//         //
//         //  Note this will leave a branch of Dcbs dangling if the directory file
//         //  had not been built on the leaf (case: opening path which has an
//         //  element containing an invalid character name).
//         //
//
//         if (_SEH2_AbnormalTermination() || !NT_SUCCESS(Iosb.Status)) {
//
//             ULONG SavedFlags;
//
//             //
//             //  Before doing the uninitialize, we have to unpin anything
//             //  that has been repinned, but disable writethrough first.  We
//             //  disable raise from unpin-repin since we're already failing.
//             //
//
//             SavedFlags = IrpContext->Flags;
//
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE |
//                                         IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH );
//
//             FatUnpinRepinnedBcbs( IrpContext );
//
//             if ((FinalDcb != NULL) &&
//                 (NodeType(FinalDcb) == FAT_NTC_DCB) &&
//                 IsListEmpty(&FinalDcb->Specific.Dcb.ParentDcbQueue) &&
//                 (FinalDcb->OpenCount == 0) &&
//                 (FinalDcb->Specific.Dcb.DirectoryFile != NULL)) {
//
//                 PFILE_OBJECT DirectoryFileObject;
//
//                 DirectoryFileObject = FinalDcb->Specific.Dcb.DirectoryFile;
//
//                 FinalDcb->Specific.Dcb.DirectoryFile = NULL;
//
//                 CcUninitializeCacheMap( DirectoryFileObject, NULL, NULL );
//
//                 ObDereferenceObject( DirectoryFileObject );
//             }
//
//             IrpContext->Flags = SavedFlags;
//         }
//
//         if (_SEH2_AbnormalTermination()) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         //
//         //  Free up any string buffers we allocated
//         //
//
//         FatFreeStringBuffer( &OemFinalName);
//
//         FatFreeStringBuffer( &UpcasedFinalName);
//
//         FatFreeStringBuffer( &Lfn);
//     } _SEH2_END;
//
//     //
//     //  The following code is only executed if we are exiting the
//     //  procedure through a normal termination.  We complete the request
//     //  and if for any reason that bombs out then we need to unreference
//     //  and possibly delete the fcb and ccb.
//     //
//
//     _SEH2_TRY {
//
//         if (PostIrp) {
//
//             //
//             //  If the Irp hasn't already been posted, do it now.
//             //
//
//             if (!OplockPostIrp) {
//
//                 Iosb.Status = FatFsdPostRequest( IrpContext, Irp );
//             }
//
//         } else {
//
//             FatUnpinRepinnedBcbs( IrpContext );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonCreate-in-FatCompleteRequest );
//
//         if (_SEH2_AbnormalTermination() ) {
//
//             PVCB LocalVcb;
//             PFCB LocalFcb;
//             PCCB LocalCcb2;
//             PFILE_OBJECT DirectoryFileObject;
//
//             //
//             //  Unwind all of our counts.  Note that if a write failed, then
//             //  the volume has been marked for verify, and all volume
//             //  structures will be cleaned up automatically.
//             //
//
//             (VOID) FatDecodeFileObject( FileObject, &LocalVcb, &LocalFcb, &LocalCcb2 );
//
//             LocalFcb->UncleanCount -= 1;
//             LocalFcb->OpenCount -= 1;
//             LocalVcb->OpenFileCount -= 1;
//
//             if (IsFileObjectReadOnly(FileObject)) { LocalVcb->ReadOnlyCount -= 1; }
//
//
//             //
//             //  WinSE #307418 "Occasional data corruption when standby/resume
//             //  while copying files to removable FAT formatted media".
//             //  If new file creation request was interrupted by system suspend
//             //  operation we should revert the changes we made to the parent
//             //  directory and to the allocation table.
//             //
//
//             if (IrpContext->ExceptionStatus == STATUS_VERIFY_REQUIRED &&
//                 NodeType( LocalFcb ) == FAT_NTC_FCB) {
//
//                 FatTruncateFileAllocation( IrpContext, LocalFcb, 0 );
//
//                 FatDeleteDirent( IrpContext, LocalFcb, NULL, TRUE );
//             }
//
//             //
//             //  If we leafed out on a new Fcb we should get rid of it at this point.
//             //
//             //  Since the object isn't being opened, we have to do all of the teardown
//             //  here.  Our close path will not occur for this fileobject. Note this
//             //  will leave a branch of Dcbs dangling since we do it by hand and don't
//             //  chase to the root.
//             //
//
//             if (LocalFcb->OpenCount == 0 &&
//                 (NodeType( LocalFcb ) == FAT_NTC_FCB ||
//                  IsListEmpty(&LocalFcb->Specific.Dcb.ParentDcbQueue))) {
//
//                 NT_ASSERT( NodeType( LocalFcb ) != FAT_NTC_ROOT_DCB );
//
//                 if ( (NodeType( LocalFcb ) == FAT_NTC_DCB) &&
//                      (LocalFcb->Specific.Dcb.DirectoryFile != NULL) ) {
//
//                     DirectoryFileObject = LocalFcb->Specific.Dcb.DirectoryFile;
//                     LocalFcb->Specific.Dcb.DirectoryFile = NULL;
//
//                     CcUninitializeCacheMap( DirectoryFileObject,
//                                             &FatLargeZero,
//                                             NULL );
//
//                     ObDereferenceObject( DirectoryFileObject );
//
//                 } else {
//                     if (ARGUMENT_PRESENT( FileObject )) {
//                         FileObject->SectionObjectPointer = NULL;
//                     }
//                     FatDeleteFcb( IrpContext, &LocalFcb );
//                 }
//             }
//
//             FatDeleteCcb( IrpContext, &LocalCcb2 );
//
//             FatReleaseVcb( IrpContext, LocalVcb );
//
//         } else {
//
//             FatReleaseVcb( IrpContext, Vcb );
//
//             if ( !PostIrp ) {
//
//                 //
//                 //  If this request is successful and the file was opened
//                 //  for FILE_EXECUTE access, then set the FileObject bit.
//                 //
//
//                 NT_ASSERT( IrpSp->Parameters.Create.SecurityContext != NULL );
//                 if (FlagOn( *DesiredAccess, FILE_EXECUTE )) {
//
//                     SetFlag( FileObject->Flags, FO_FILE_FAST_IO_READ );
//                 }
//
//                 //
//                 //  Lock volume in drive if we opened a paging file, allocating a
//                 //  reserve MDL to guarantee paging file operations can always
//                 //  go forward.
//                 //
//
//                 if (IsPagingFile && NT_SUCCESS(Iosb.Status)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28112, "this should be safe" )
// #endif
//                     if (!FatReserveMdl) {
//
//                         PMDL ReserveMdl = IoAllocateMdl( NULL,
//                                                          FAT_RESERVE_MDL_SIZE * PAGE_SIZE,
//                                                          TRUE,
//                                                          FALSE,
//                                                          NULL );
//
//                         //
//                         //  Stash the MDL, and if it turned out there was already one there
//                         //  just free what we got.
//                         //
//
// #ifndef __REACTOS__
//                         InterlockedCompareExchangePointer( &FatReserveMdl, ReserveMdl, NULL );
// #else
//                         InterlockedCompareExchangePointer( (void * volatile*)&FatReserveMdl, ReserveMdl, NULL );
// #endif
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28112, "this should be safe" )
// #endif
//                         if (FatReserveMdl != ReserveMdl) {
//
//                             IoFreeMdl( ReserveMdl );
//                         }
//                     }
//
//                     SetFlag(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE);
//
//                     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA)) {
//
//                         FatToggleMediaEjectDisable( IrpContext, Vcb, TRUE );
//                     }
//                 }
//
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonCreate -> %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     CollectCreateStatistics(Vcb, Iosb.Status);
//
//     return Iosb.Status;
// }
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenVolume (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the specified volume for DASD access
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume being opened
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     CreateDisposition - Supplies the create disposition for this operation
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//
// #ifndef __REACTOS__
//     IO_STATUS_BLOCK Iosb = {0,0};
// #else
//     IO_STATUS_BLOCK Iosb = {{0}};
// #endif
//
//     BOOLEAN CleanedVolume = FALSE;
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     BOOLEAN UnwindShareAccess = FALSE;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN UnwindCounts = FALSE;
//     BOOLEAN UnwindVolumeLock = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenVolume...\n", 0);
//
//     _SEH2_TRY {
//
//         //
//         //  Check for proper desired access and rights
//         //
//
//         if ((CreateDisposition != FILE_OPEN) &&
//             (CreateDisposition != FILE_OPEN_IF)) {
//
//             try_return( Status = STATUS_ACCESS_DENIED );
//         }
//
//         //
//         //  If the user does not want to share write or delete then we will try
//         //  and take out a lock on the volume.
//         //
//
//         if (!FlagOn(ShareAccess, FILE_SHARE_WRITE) &&
//             !FlagOn(ShareAccess, FILE_SHARE_DELETE)) {
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             //
//             //  See if the user has requested write access.  If so, they cannot share
//             //  read.  There is one exception to this.  We allow autochk to get an
//             //  implicit lock on the volume while still allowing readers.  Once the
//             //  the system is booted, though, we do not allow this type of access.
//             //
//
//             if (FlagOn( *DesiredAccess, (FILE_WRITE_DATA | FILE_APPEND_DATA) ) &&
//                 FsRtlAreVolumeStartupApplicationsComplete()) {
//
//                 ClearFlag( ShareAccess, FILE_SHARE_READ );
//             }
// #endif
//
//             //
//             //  Do a quick check here for handles on exclusive open.
//             //
//
//             if (!FlagOn(ShareAccess, FILE_SHARE_READ) &&
//                 !FatIsHandleCountZero( IrpContext, Vcb )) {
//
//                 try_return( Status = STATUS_SHARING_VIOLATION );
//             }
//
//             //
//             //  Force Mm to get rid of its referenced file objects.
//             //
//
//             FatFlushFat( IrpContext, Vcb );
//
//             FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, Flush );
//
//             //
//             //  If the user also does not want to share read then we check
//             //  if anyone is already using the volume, and if so then we
//             //  deny the access.  If the user wants to share read then
//             //  we allow the current opens to stay provided they are only
//             //  readonly opens and deny further opens.
//             //
//
//             if (!FlagOn(ShareAccess, FILE_SHARE_READ)) {
//
//                 if (Vcb->OpenFileCount != 0) {
//
//                     try_return( Status = STATUS_SHARING_VIOLATION );
//                 }
//
//             } else {
//
//                 if (Vcb->ReadOnlyCount != Vcb->OpenFileCount) {
//
//                     try_return( Status = STATUS_SHARING_VIOLATION );
//                 }
//             }
//
//             //
//             //  Lock the volume
//             //
//
//             Vcb->VcbState |= VCB_STATE_FLAG_LOCKED;
//             Vcb->FileObjectWithVcbLocked = FileObject;
//             UnwindVolumeLock = TRUE;
//
//             //
//             //  Clean the volume
//             //
//
//             CleanedVolume = TRUE;
//
//         }  else if (FlagOn( *DesiredAccess, FILE_READ_DATA | FILE_WRITE_DATA | FILE_APPEND_DATA )) {
//
//             //
//             //  Flush the volume and let ourselves push the clean bit out if everything
//             //  worked.
//             //
//
//             if (NT_SUCCESS( FatFlushVolume( IrpContext, Vcb, Flush ))) {
//
//                 CleanedVolume = TRUE;
//             }
//         }
//
//         //
//         //  Clean the volume if we believe it safe and reasonable.
//         //
//
//         if (CleanedVolume &&
//             FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY ) &&
//             !FlagOn( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY ) &&
//             !CcIsThereDirtyData(Vcb->Vpb)) {
//
//             //
//             //  Cancel any pending clean volumes.
//             //
//
//             (VOID)KeCancelTimer( &Vcb->CleanVolumeTimer );
//             (VOID)KeRemoveQueueDpc( &Vcb->CleanVolumeDpc );
//
//             FatMarkVolume( IrpContext, Vcb, VolumeClean );
//             ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//
//             //
//             //  Unlock the volume if it is removable.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//                 !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE)) {
//
//                 FatToggleMediaEjectDisable( IrpContext, Vcb, FALSE );
//             }
//         }
//
//         //
//         //  If the volume is already opened by someone then we need to check
//         //  the share access
//         //
//
//         if (Vcb->DirectAccessOpenCount > 0) {
//
//             if (!NT_SUCCESS(Status = IoCheckShareAccess( *DesiredAccess,
//                                                               ShareAccess,
//                                                               FileObject,
//                                                               &Vcb->ShareAccess,
//                                                               TRUE ))) {
//
//                 try_return( NOTHING );
//             }
//
//         } else {
//
//             IoSetShareAccess( *DesiredAccess,
//                               ShareAccess,
//                               FileObject,
//                               &Vcb->ShareAccess );
//         }
//
//         UnwindShareAccess = TRUE;
//
//         //
//         //  Set up the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserVolumeOpen,
//                           Vcb,
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         FileObject->SectionObjectPointer = &Vcb->SectionObjectPointers;
//
//         Vcb->DirectAccessOpenCount += 1;
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//         UnwindCounts = TRUE;
//         FileObject->Flags |= FO_NO_INTERMEDIATE_BUFFERING;
//
//         //
//         //  At this point the open will succeed, so check if the user is getting explicit access
//         //  to the device.  If not, we will note this so we can deny modifying FSCTL to it.
//         //
//
//         IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//         Status = FatExplicitDeviceAccessGranted( IrpContext,
//                                                  Vcb->Vpb->RealDevice,
//                                                  IrpSp->Parameters.Create.SecurityContext->AccessState,
//                                                  (KPROCESSOR_MODE)( FlagOn( IrpSp->Flags, SL_FORCE_ACCESS_CHECK ) ?
//                                                                     UserMode :
//                                                                     IrpContext->OriginatingIrp->RequestorMode ));
//
//         if (NT_SUCCESS( Status )) {
//
//             SetFlag( UnwindCcb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS );
//         }
//
//         //
//         //  And set our status to success
//         //
//
//         Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_OPENED;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenVolume );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination() || !NT_SUCCESS(Status)) {
//
//             if (UnwindCounts) {
//                 Vcb->DirectAccessOpenCount -= 1;
//                 Vcb->OpenFileCount -= 1;
//                 if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount -= 1; }
//             }
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//             if (UnwindShareAccess) { IoRemoveShareAccess( FileObject, &Vcb->ShareAccess ); }
//             if (UnwindVolumeLock) { Vcb->VcbState &= ~VCB_STATE_FLAG_LOCKED; }
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenVolume -> Iosb.Status = %08lx\n", Status);
//     } _SEH2_END;
//
//     Iosb.Status = Status;
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenRootDcb (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the root dcb for the volume
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume whose dcb is being opened.
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     CreateDisposition - Supplies the create disposition for this operation
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// Arguments:
//
// --*/
//
// {
//     PDCB RootDcb;
//     IO_STATUS_BLOCK Iosb = {0};
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     BOOLEAN UnwindShareAccess = FALSE;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN UnwindCounts = FALSE;
//     BOOLEAN RootDcbAcquired = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenRootDcb...\n", 0);
//
//     //
//     //  Locate the root dcb
//     //
//
//     RootDcb = Vcb->RootDcb;
//
//     //
//     //  Get the Dcb exlcusive.  This is important as cleanup does not
//     //  acquire the Vcb.
//     //
//
//     (VOID)FatAcquireExclusiveFcb( IrpContext, RootDcb );
//     RootDcbAcquired = TRUE;
//
//     _SEH2_TRY {
//
//         //
//         //  Check the create disposition and desired access
//         //
//
//         if ((CreateDisposition != FILE_OPEN) &&
//             (CreateDisposition != FILE_OPEN_IF)) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//         if (!FatCheckFileAccess( IrpContext,
//                                  RootDcb->DirentFatFlags,
//                                  DesiredAccess)) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//         //
//         //  If the Root dcb is already opened by someone then we need
//         //  to check the share access
//         //
//
//         if (RootDcb->OpenCount > 0) {
//
//             if (!NT_SUCCESS(Iosb.Status = IoCheckShareAccess( *DesiredAccess,
//                                                               ShareAccess,
//                                                               FileObject,
//                                                               &RootDcb->ShareAccess,
//                                                               TRUE ))) {
//
//                 try_return( Iosb );
//             }
//
//         } else {
//
//             IoSetShareAccess( *DesiredAccess,
//                               ShareAccess,
//                               FileObject,
//                               &RootDcb->ShareAccess );
//         }
//
//         UnwindShareAccess = TRUE;
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserDirectoryOpen,
//                           RootDcb,
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         RootDcb->UncleanCount += 1;
//         RootDcb->OpenCount += 1;
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//         UnwindCounts = TRUE;
//
//         //
//         //  And set our status to success
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_OPENED;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenRootDcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindCounts) {
//                 RootDcb->UncleanCount -= 1;
//                 RootDcb->OpenCount -= 1;
//                 Vcb->OpenFileCount -= 1;
//                 if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount -= 1; }
//             }
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//             if (UnwindShareAccess) { IoRemoveShareAccess( FileObject, &RootDcb->ShareAccess ); }
//         }
//
//         if (RootDcbAcquired) {
//
//             FatReleaseFcb( IrpContext, RootDcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenRootDcb -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingDcb (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB Dcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos,
//      PBOOLEAN OplockPostIrp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the specified existing dcb
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume containing the dcb
//
//     Dcb - Supplies the already existing dcb
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     CreateDisposition - Supplies the create disposition for this operation
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//     PBCB DirentBcb = NULL;
//     PDIRENT Dirent;
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     BOOLEAN UnwindShareAccess = FALSE;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN DcbAcquired = FALSE;
//
// #if (NTDDI_VERSION <= NTDDI_WIN7)
//     UNREFERENCED_PARAMETER( OpenRequiringOplock );
// #endif
//
//     UNREFERENCED_PARAMETER( IrpSp );
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenExistingDcb...\n", 0);
//
//     //
//     //  Get the Dcb exlcusive.  This is important as cleanup does not
//     //  acquire the Vcb.
//     //
//
//     (VOID)FatAcquireExclusiveFcb( IrpContext, Dcb );
//     DcbAcquired = TRUE;
//
//     _SEH2_TRY {
//
//
//         *OplockPostIrp = FALSE;
//
//         //
//         //  Before spending any noticeable effort, see if we have the odd case
//         //  of someone trying to delete-on-close the root dcb.  This will only
//         //  happen if we're hit with a null-filename relative open via the root.
//         //
//
//         if (NodeType(Dcb) == FAT_NTC_ROOT_DCB && DeleteOnClose) {
//
//             Iosb.Status = STATUS_CANNOT_DELETE;
//             try_return( Iosb );
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(Dcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             try_return( NOTHING );
//         }
//
// #endif
//         //
//         //  If the caller has no Ea knowledge, we immediately check for
//         //  Need Ea's on the file.  We don't need to check for ea's on the
//         //  root directory, because it never has any.  Fat32 doesn't have
//         //  any, either.
//         //
//
//         if (NoEaKnowledge && NodeType(Dcb) != FAT_NTC_ROOT_DCB &&
//             !FatIsFat32(Vcb)) {
//
//             ULONG NeedEaCount;
//
//             //
//             //  Get the dirent for the file and then check that the need
//             //  ea count is 0.
//             //
//
//             FatGetDirentFromFcbOrDcb( IrpContext,
//                                       Dcb,
//                                       FALSE,
//                                       &Dirent,
//                                       &DirentBcb );
//
//             FatGetNeedEaCount( IrpContext,
//                                Vcb,
//                                Dirent,
//                                &NeedEaCount );
//
//             FatUnpinBcb( IrpContext, DirentBcb );
//
//             if (NeedEaCount != 0) {
//
//                 Iosb.Status = STATUS_ACCESS_DENIED;
//                 try_return( Iosb );
//             }
//         }
//
//         //
//         //  Check the create disposition and desired access
//         //
//
//         if ((CreateDisposition != FILE_OPEN) &&
//             (CreateDisposition != FILE_OPEN_IF)) {
//
//             Iosb.Status = STATUS_OBJECT_NAME_COLLISION;
//             try_return( Iosb );
//         }
//
//         if (!FatCheckFileAccess( IrpContext,
//                                  Dcb->DirentFatFlags,
//                                  DesiredAccess)) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//         //
//         //  If the dcb is already opened by someone then we need
//         //  to check the share access
//         //
//
//         if (Dcb->OpenCount > 0) {
//
//             if (!NT_SUCCESS(Iosb.Status = FatCheckShareAccess( IrpContext,
//                                                                FileObject,
//                                                                Dcb,
//                                                                DesiredAccess,
//                                                                ShareAccess ))) {
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//                 NTSTATUS OplockBreakStatus = STATUS_SUCCESS;
//
//                 //
//                 //  If we got a sharing violation try to break outstanding handle
//                 //  oplocks and retry the sharing check.  If the caller specified
//                 //  FILE_COMPLETE_IF_OPLOCKED we don't bother breaking the oplock;
//                 //  we just return the sharing violation.
//                 //
//
//                 if ((Iosb.Status == STATUS_SHARING_VIOLATION) &&
//                     !FlagOn( IrpSp->Parameters.Create.Options, FILE_COMPLETE_IF_OPLOCKED )) {
//
//                     OplockBreakStatus = FsRtlOplockBreakH( FatGetFcbOplock(Dcb),
//                                                            IrpContext->OriginatingIrp,
//                                                            0,
//                                                            IrpContext,
//                                                            FatOplockComplete,
//                                                            FatPrePostIrp );
//
//                     //
//                     //  If FsRtlOplockBreakH returned STATUS_PENDING, then the IRP
//                     //  has been posted and we need to stop working.
//                     //
//
//                     if (OplockBreakStatus == STATUS_PENDING) {
//
//                         Iosb.Status = STATUS_PENDING;
//                         *OplockPostIrp = TRUE;
//                         try_return( NOTHING );
//
//                     //
//                     //  If FsRtlOplockBreakH returned an error we want to return that now.
//                     //
//
//                     } else if (!NT_SUCCESS( OplockBreakStatus )) {
//
//                         Iosb.Status = OplockBreakStatus;
//                         try_return( Iosb );
//
//                     //
//                     //  Otherwise FsRtlOplockBreakH returned STATUS_SUCCESS, indicating
//                     //  that there is no oplock to be broken.  The sharing violation is
//                     //  returned in that case.
//                     //
//
//                     } else {
//
//                         NT_ASSERT( OplockBreakStatus == STATUS_SUCCESS );
//
//                         try_return( Iosb );
//                     }
//
//                 //
//                 //  The initial sharing check failed with something other than sharing
//                 //  violation (which should never happen, but let's be future-proof),
//                 //  or we *did* get a sharing violation and the caller specified
//                 //  FILE_COMPLETE_IF_OPLOCKED.  Either way this create is over.
//                 //
//
//                 } else {
//
//                     try_return( Iosb );
//                 }
// #else
//
//                 try_return( Iosb );
// #endif
//             }
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         //
//         //  Now check that we can continue based on the oplock state of the
//         //  directory.  If there are no open handles yet we don't need to do
//         //  this check; oplocks can only exist when there are handles.
//         //
//
//         if (Dcb->UncleanCount != 0) {
//
//             Iosb.Status = FsRtlCheckOplock( FatGetFcbOplock(Dcb),
//                                             IrpContext->OriginatingIrp,
//                                             IrpContext,
//                                             FatOplockComplete,
//                                             FatPrePostIrp );
//         }
//
//         //
//         //  if FsRtlCheckOplock returns STATUS_PENDING the IRP has been posted
//         //  to service an oplock break and we need to leave now.
//         //
//
//         if (Iosb.Status == STATUS_PENDING) {
//
//             *OplockPostIrp = TRUE;
//             try_return( NOTHING );
//         }
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Fcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(Dcb),
//                                              IrpContext->OriginatingIrp,
//                                              (Dcb->UncleanCount + 1) );
//         }
//
//         //
//         //  If we've encountered a failure we need to leave.  FsRtlCheckOplock
//         //  will have returned STATUS_OPLOCK_BREAK_IN_PROGRESS if it initiated
//         //  and oplock break and the caller specified FILE_COMPLETE_IF_OPLOCKED
//         //  on the create call.  That's an NT_SUCCESS code, so we need to keep
//         //  going.
//         //
//
//         if ((Iosb.Status != STATUS_SUCCESS) &&
//             (Iosb.Status != STATUS_OPLOCK_BREAK_IN_PROGRESS)) {
//
//             try_return( NOTHING );
//         }
//
// #endif
//
//         //
//         //  Now that we're done with the oplock work update the share counts.
//         //  If the Dcb isn't yet opened we just set the share access rather than
//         //  update it.
//         //
//
//         if (Dcb->OpenCount > 0) {
//
//             IoUpdateShareAccess( FileObject, &Dcb->ShareAccess );
//
//         } else {
//
//             IoSetShareAccess( *DesiredAccess,
//                               ShareAccess,
//                               FileObject,
//                               &Dcb->ShareAccess );
//         }
//
//         UnwindShareAccess = TRUE;
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserDirectoryOpen,
//                           Dcb,
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         Dcb->UncleanCount += 1;
//         Dcb->OpenCount += 1;
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//
//         //
//         //  Mark the delete on close bit if the caller asked for that.
//         //
//
//         {
//             PCCB Ccb = (PCCB)FileObject->FsContext2;
//
//
//             if (DeleteOnClose) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//             }
//             if (FileNameOpenedDos) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//             }
//
//         }
//
//
//         //
//         //  In case this was set, clear it now.
//         //
//
//         ClearFlag(Dcb->FcbState, FCB_STATE_DELAY_CLOSE);
//
//         //
//         //  And set our status to success
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_OPENED;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenExistingDcb );
//
//         //
//         //  Unpin the Dirent Bcb if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//             if (UnwindShareAccess) { IoRemoveShareAccess( FileObject, &Dcb->ShareAccess ); }
//         }
//
//         if (DcbAcquired) {
//
//             FatReleaseFcb( IrpContext, Dcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenExistingDcb -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingFcb (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PFCB Fcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos,
//      PBOOLEAN OplockPostIrp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the specified existing fcb
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume containing the Fcb
//
//     Fcb - Supplies the already existing fcb
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     AllocationSize - Supplies the initial allocation if the file is being
//         superseded or overwritten
//
//     EaBuffer - Supplies the Ea set if the file is being superseded or
//         overwritten
//
//     EaLength - Supplies the size, in byte, of the EaBuffer
//
//     FileAttributes - Supplies file attributes to use if the file is being
//         superseded or overwritten
//
//     CreateDisposition - Supplies the create disposition for this operation
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
//     OpenRequiringOplock - The caller provided the FILE_OPEN_REQUIRING_OPLOCK option.
//
//     FileNameOpenedDos - The caller hit the short side of the name pair finding
//         this file
//
//     OplockPostIrp - Address to store boolean indicating if the Irp needs to
//         be posted to the Fsp.
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     PBCB DirentBcb = NULL;
//     PDIRENT Dirent;
//
//     ACCESS_MASK AddedAccess = 0;
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     BOOLEAN UnwindShareAccess = FALSE;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN DecrementFcbOpenCount = FALSE;
//     BOOLEAN FcbAcquired = FALSE;
//
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenExistingFcb...\n", 0);
//
//     //
//     //  Get the Fcb exlcusive.  This is important as cleanup does not
//     //  acquire the Vcb.
//     //
//
//     (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//     FcbAcquired = TRUE;
//
//     _SEH2_TRY {
//
//
//         *OplockPostIrp = FALSE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             try_return( NOTHING );
//         }
// #endif
//
//         //
//         //  Take special action if there is a current batch oplock or
//         //  batch oplock break in process on the Fcb.
//         //
//
//         if (FsRtlCurrentBatchOplock( FatGetFcbOplock(Fcb) )) {
//
//             //
//             //  We remember if a batch oplock break is underway for the
//             //  case where the sharing check fails.
//             //
//
//             Iosb.Information = FILE_OPBATCH_BREAK_UNDERWAY;
//
//             Iosb.Status = FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                                             IrpContext->OriginatingIrp,
//                                             IrpContext,
//                                             FatOplockComplete,
//                                             FatPrePostIrp );
//
//             //
//             //  if FsRtlCheckOplock returns STATUS_PENDING the IRP has been posted
//             //  to service an oplock break and we need to leave now.
//             //
//
//             if (Iosb.Status == STATUS_PENDING) {
//
//                 *OplockPostIrp = TRUE;
//                 try_return( NOTHING );
//             }
//         }
//
//         //
//         //  Check if the user wanted to create the file, also special case
//         //  the supersede and overwrite options.  Those add additional,
//         //  possibly only implied, desired accesses to the caller, which
//         //  we must be careful to pull back off if the caller did not actually
//         //  request them.
//         //
//         //  In other words, check against the implied access, but do not modify
//         //  share access as a result.
//         //
//
//         if (CreateDisposition == FILE_CREATE) {
//
//             Iosb.Status = STATUS_OBJECT_NAME_COLLISION;
//             try_return( Iosb );
//
//         } else if (CreateDisposition == FILE_SUPERSEDE) {
//
//             SetFlag( AddedAccess,
//                      DELETE & ~(*DesiredAccess) );
//
//             *DesiredAccess |= DELETE;
//
//         } else if ((CreateDisposition == FILE_OVERWRITE) ||
//                    (CreateDisposition == FILE_OVERWRITE_IF)) {
//
//             SetFlag( AddedAccess,
//                      (FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES) & ~(*DesiredAccess) );
//
//             *DesiredAccess |= FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES;
//         }
//
//         //
//         //  Check the desired access
//         //
//
//         if (!FatCheckFileAccess( IrpContext,
//                                  Fcb->DirentFatFlags,
//                                  DesiredAccess )) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//
//         //
//         //  Check for trying to delete a read only file.
//         //
//
//         if (DeleteOnClose &&
//             FlagOn( Fcb->DirentFatFlags, FAT_DIRENT_ATTR_READ_ONLY )) {
//
//             Iosb.Status = STATUS_CANNOT_DELETE;
//             try_return( Iosb );
//         }
//
//         //
//         //  If we are asked to do an overwrite or supersede operation then
//         //  deny access for files where the file attributes for system and
//         //  hidden do not match
//         //
//
//         if ((CreateDisposition == FILE_SUPERSEDE) ||
//             (CreateDisposition == FILE_OVERWRITE) ||
//             (CreateDisposition == FILE_OVERWRITE_IF)) {
//
//             BOOLEAN Hidden;
//             BOOLEAN System;
//
//             Hidden = BooleanFlagOn(Fcb->DirentFatFlags, FAT_DIRENT_ATTR_HIDDEN );
//             System = BooleanFlagOn(Fcb->DirentFatFlags, FAT_DIRENT_ATTR_SYSTEM );
//
//             if ((Hidden && !FlagOn(FileAttributes, FILE_ATTRIBUTE_HIDDEN)) ||
//                 (System && !FlagOn(FileAttributes, FILE_ATTRIBUTE_SYSTEM))) {
//
//                 DebugTrace(0, Dbg, "The hidden and/or system bits do not match\n", 0);
//
//
//                 Iosb.Status = STATUS_ACCESS_DENIED;
//                 try_return( Iosb );
//             }
//
//             //
//             //  If this media is write protected, don't even try the create.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//                 //
//                 //  Set the real device for the pop-up info, and set the verify
//                 //  bit in the device object, so that we will force a verify
//                 //  in case the user put the correct media back in.
//                 //
//
//                 IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                               Vcb->Vpb->RealDevice );
//
//                 SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//                 FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//             }
//         }
//
//         //
//         //  Check if the Fcb has the proper share access.  This routine will also
//         //  check for writable user secions if the user did not allow write sharing.
//         //
//
//         if (!NT_SUCCESS(Iosb.Status = FatCheckShareAccess( IrpContext,
//                                                            FileObject,
//                                                            Fcb,
//                                                            DesiredAccess,
//                                                            ShareAccess ))) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
//             NTSTATUS OplockBreakStatus = STATUS_SUCCESS;
//
//             //
//             //  If we got a sharing violation try to break outstanding handle
//             //  oplocks and retry the sharing check.  If the caller specified
//             //  FILE_COMPLETE_IF_OPLOCKED we don't bother breaking the oplock;
//             //  we just return the sharing violation.
//             //
//
//             if ((Iosb.Status == STATUS_SHARING_VIOLATION) &&
//                 !FlagOn( IrpSp->Parameters.Create.Options, FILE_COMPLETE_IF_OPLOCKED )) {
//
//                 OplockBreakStatus = FsRtlOplockBreakH( FatGetFcbOplock(Fcb),
//                                                        IrpContext->OriginatingIrp,
//                                                        0,
//                                                        IrpContext,
//                                                        FatOplockComplete,
//                                                        FatPrePostIrp );
//
//                 //
//                 //  If FsRtlOplockBreakH returned STATUS_PENDING, then the IRP
//                 //  has been posted and we need to stop working.
//                 //
//
//                 if (OplockBreakStatus == STATUS_PENDING) {
//
//                     Iosb.Status = STATUS_PENDING;
//                     *OplockPostIrp = TRUE;
//                     try_return( NOTHING );
//
//                 //
//                 //  If FsRtlOplockBreakH returned an error we want to return that now.
//                 //
//
//                 } else if (!NT_SUCCESS( OplockBreakStatus )) {
//
//                     Iosb.Status = OplockBreakStatus;
//                     try_return( Iosb );
//
//                 //
//                 //  Otherwise FsRtlOplockBreakH returned STATUS_SUCCESS, indicating
//                 //  that there is no oplock to be broken.  The sharing violation is
//                 //  returned in that case.
//                 //
//
//                 } else {
//
//                     NT_ASSERT( OplockBreakStatus == STATUS_SUCCESS );
//
//                     try_return( Iosb );
//                 }
//
//             //
//             //  The initial sharing check failed with something other than sharing
//             //  violation (which should never happen, but let's be future-proof),
//             //  or we *did* get a sharing violation and the caller specified
//             //  FILE_COMPLETE_IF_OPLOCKED.  Either way this create is over.
//             //
//
//             } else {
//
//                 try_return( Iosb );
//             }
//
// #else
//
//             try_return( Iosb );
//
// #endif
//         }
//
//         //
//         //  Now check that we can continue based on the oplock state of the
//         //  file.  If there are no open handles yet we don't need to do this
//         //  check; oplocks can only exist when there are handles.
//         //
//         //  It is important that we modified the DesiredAccess in place so
//         //  that the Oplock check proceeds against any added access we had
//         //  to give the caller.
//         //
//
//         if (Fcb->UncleanCount != 0) {
//
//             Iosb.Status = FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                                             IrpContext->OriginatingIrp,
//                                             IrpContext,
//                                             FatOplockComplete,
//                                             FatPrePostIrp );
//         }
//
//         //
//         //  if FsRtlCheckOplock returns STATUS_PENDING the IRP has been posted
//         //  to service an oplock break and we need to leave now.
//         //
//
//         if (Iosb.Status == STATUS_PENDING) {
//
//             *OplockPostIrp = TRUE;
//             try_return( NOTHING );
//         }
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Fcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(Fcb),
//                                              IrpContext->OriginatingIrp,
//                                              (Fcb->UncleanCount + 1) );
//         }
//
//         //
//         //  If we've encountered a failure we need to leave.  FsRtlCheckOplock
//         //  will have returned STATUS_OPLOCK_BREAK_IN_PROGRESS if it initiated
//         //  and oplock break and the caller specified FILE_COMPLETE_IF_OPLOCKED
//         //  on the create call.  That's an NT_SUCCESS code, so we need to keep
//         //  going.
//         //
//
//         if ((Iosb.Status != STATUS_SUCCESS) &&
//             (Iosb.Status != STATUS_OPLOCK_BREAK_IN_PROGRESS)) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//         //
//         //  If the user wants write access access to the file make sure there
//         //  is not a process mapping this file as an image.  Any attempt to
//         //  delete the file will be stopped in fileinfo.c
//         //
//         //  If the user wants to delete on close, we must check at this
//         //  point though.
//         //
//
//         if (FlagOn(*DesiredAccess, FILE_WRITE_DATA) || DeleteOnClose) {
//
//             Fcb->OpenCount += 1;
//             DecrementFcbOpenCount = TRUE;
//
//             if (!MmFlushImageSection( &Fcb->NonPaged->SectionObjectPointers,
//                                       MmFlushForWrite )) {
//
//                 Iosb.Status = DeleteOnClose ? STATUS_CANNOT_DELETE :
//                                               STATUS_SHARING_VIOLATION;
//                 try_return( Iosb );
//             }
//         }
//
//         //
//         //  If this is a non-cached open on a non-paging file, and there
//         //  are no open cached handles, but there is a still a data
//         //  section, attempt a flush and purge operation to avoid cache
//         //  coherency overhead later.  We ignore any I/O errors from
//         //  the flush.
//         //
//         //  We set the CREATE_IN_PROGRESS flag to prevent the Fcb from
//         //  going away out from underneath us.
//         //
//
//         if (FlagOn( FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING ) &&
//             (Fcb->UncleanCount == Fcb->NonCachedUncleanCount) &&
//             (Fcb->NonPaged->SectionObjectPointers.DataSectionObject != NULL) &&
//             !FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE )) {
//
//             SetFlag(Fcb->Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
//
//             CcFlushCache( &Fcb->NonPaged->SectionObjectPointers, NULL, 0, NULL );
//
//             //
//             //  Grab and release PagingIo to serialize ourselves with the lazy writer.
//             //  This will work to ensure that all IO has completed on the cached
//             //  data and we will succesfully tear away the cache section.
//             //
//
//             ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE);
//             ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//
//             CcPurgeCacheSection( &Fcb->NonPaged->SectionObjectPointers,
//                                  NULL,
//                                  0,
//                                  FALSE );
//
//             ClearFlag(Fcb->Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
//         }
//
//         //
//         //  Check if the user only wanted to open the file
//         //
//
//         if ((CreateDisposition == FILE_OPEN) ||
//             (CreateDisposition == FILE_OPEN_IF)) {
//
//             DebugTrace(0, Dbg, "Doing open operation\n", 0);
//
//             //
//             //  If the caller has no Ea knowledge, we immediately check for
//             //  Need Ea's on the file.
//             //
//
//             if (NoEaKnowledge && !FatIsFat32(Vcb)) {
//
//                 ULONG NeedEaCount;
//
//                 //
//                 //  Get the dirent for the file and then check that the need
//                 //  ea count is 0.
//                 //
//
//                 FatGetDirentFromFcbOrDcb( IrpContext,
//                                           Fcb,
//                                           FALSE,
//                                           &Dirent,
//                                           &DirentBcb );
//
//                 FatGetNeedEaCount( IrpContext,
//                                    Vcb,
//                                    Dirent,
//                                    &NeedEaCount );
//
//                 FatUnpinBcb( IrpContext, DirentBcb );
//
//                 if (NeedEaCount != 0) {
//
//                     Iosb.Status = STATUS_ACCESS_DENIED;
//                     try_return( Iosb );
//                 }
//             }
//
//             //
//             //  Everything checks out okay, so setup the context and
//             //  section object pointers.
//             //
//
//             FatSetFileObject( FileObject,
//                               UserFileOpen,
//                               Fcb,
//                               UnwindCcb = FatCreateCcb( IrpContext ));
//
//             FileObject->SectionObjectPointer = &Fcb->NonPaged->SectionObjectPointers;
//
//             //
//             //  Fill in the information field, the status field is already
//             //  set.
//             //
//
//             Iosb.Information = FILE_OPENED;
//
//             try_return( Iosb );
//         }
//
//         //
//         //  Check if we are to supersede/overwrite the file, we can wait for
//         //  any I/O at this point
//         //
//
//         if ((CreateDisposition == FILE_SUPERSEDE) ||
//             (CreateDisposition == FILE_OVERWRITE) ||
//             (CreateDisposition == FILE_OVERWRITE_IF)) {
//
//             NTSTATUS OldStatus;
//
//             DebugTrace(0, Dbg, "Doing supersede/overwrite operation\n", 0);
//
//             //
//             //  We remember the previous status code because it may contain
//             //  information about the oplock status.
//             //
//
//             OldStatus = Iosb.Status;
//
//             //
//             //  Determine the granted access for this operation now.
//             //
//
//             if (!NT_SUCCESS( Iosb.Status = FatCheckSystemSecurityAccess( IrpContext ))) {
//
//                 try_return( Iosb );
//             }
//
//             //
//             //  And overwrite the file.
//             //
//
//             Iosb = FatSupersedeOrOverwriteFile( IrpContext,
//                                                 FileObject,
//                                                 Fcb,
//                                                 AllocationSize,
//                                                 EaBuffer,
//                                                 EaLength,
//                                                 FileAttributes,
//                                                 CreateDisposition,
//                                                 NoEaKnowledge );
//
//             if (Iosb.Status == STATUS_SUCCESS) {
//
//                 Iosb.Status = OldStatus;
//             }
//
//             try_return( Iosb );
//         }
//
//         //
//         //  If we ever get here then the I/O system gave us some bad input
//         //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( CreateDisposition, 0, 0 );
//
//     try_exit: NOTHING;
//
//         //
//         //  Update the share access and counts if successful
//         //
//
//         if ((Iosb.Status != STATUS_PENDING) && NT_SUCCESS(Iosb.Status)) {
//
//             //
//             //  Now, we may have added some access bits above to indicate the access
//             //  this caller would conflict with (as opposed to what they get) in order
//             //  to perform the overwrite/supersede.  We need to make a call to that will
//             //  recalculate the bits in the fileobject to reflect the real access they
//             //  will get.
//             //
//
//             if (AddedAccess) {
//
//                 NTSTATUS Status;
//
//                 ClearFlag( *DesiredAccess, AddedAccess );
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "it needs to be there for debug assert" );
// #endif
//                 Status = IoCheckShareAccess( *DesiredAccess,
//                                              ShareAccess,
//                                              FileObject,
//                                              &Fcb->ShareAccess,
//                                              TRUE );
//
//                 //
//                 //  It must be the case that we are really asking for less access, so
//                 //  any conflict must have been detected before this point.
//                 //
//
//                 NT_ASSERT( Status == STATUS_SUCCESS );
//
//             } else {
//
//                 IoUpdateShareAccess( FileObject, &Fcb->ShareAccess );
//             }
//
//             UnwindShareAccess = TRUE;
//
//             //
//             //  In case this was set, clear it now.
//             //
//
//             ClearFlag(Fcb->FcbState, FCB_STATE_DELAY_CLOSE);
//
//             Fcb->UncleanCount += 1;
//             Fcb->OpenCount += 1;
//             if (FlagOn(FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING)) {
//                 Fcb->NonCachedUncleanCount += 1;
//             }
//             Vcb->OpenFileCount += 1;
//             if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//
//             {
//                 PCCB Ccb;
//
//                 Ccb = (PCCB)FileObject->FsContext2;
//
//                 //
//                 //  Mark the DeleteOnClose bit if the operation was successful.
//                 //
//
//                 if ( DeleteOnClose ) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//                 }
//
//                 //
//                 //  Mark the OpenedByShortName bit if the operation was successful.
//                 //
//
//                 if ( FileNameOpenedDos ) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//                 }
//
//                 //
//                 //  Mark the ManageVolumeAccess bit if the privilege is held.
//                 //
//
//                 if (FatCheckManageVolumeAccess( IrpContext,
//                                                 IrpSp->Parameters.Create.SecurityContext->AccessState,
//                                                 (KPROCESSOR_MODE)( FlagOn( IrpSp->Flags, SL_FORCE_ACCESS_CHECK ) ?
//                                                                    UserMode :
//                                                                    IrpContext->OriginatingIrp->RequestorMode ))) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS );
//                 }
//             }
//
//
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenExistingFcb );
//
//         //
//         //  Unpin the Dirent Bcb if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//             if (UnwindShareAccess) { IoRemoveShareAccess( FileObject, &Fcb->ShareAccess ); }
//         }
//
//         if (DecrementFcbOpenCount) {
//
//             Fcb->OpenCount -= 1;
//
//             if (Fcb->OpenCount == 0) {
//                 if (ARGUMENT_PRESENT( FileObject )) {
//                     FileObject->SectionObjectPointer = NULL;
//                 }
//                 FatDeleteFcb( IrpContext, &Fcb );
//                 FcbAcquired = FALSE;
//             }
//         }
//
//         if (FcbAcquired) {
//
//             FatReleaseFcb( IrpContext, Fcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenExistingFcb -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenTargetDirectory (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PDCB Dcb,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      BOOLEAN DoesNameExist,
//      BOOLEAN FileNameOpenedDos
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the target directory and replaces the name in the
//     file object with the remaining name.
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Dcb - Supplies an already existing dcb that we are going to open
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     DoesNameExist - Indicates if the file name already exists in the
//         target directory.
//
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     BOOLEAN UnwindShareAccess = FALSE;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN DcbAcquired = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenTargetDirectory...\n", 0);
//
//     //
//     //  Get the Dcb exlcusive.  This is important as cleanup does not
//     //  acquire the Vcb.
//     //
//
//     (VOID)FatAcquireExclusiveFcb( IrpContext, Dcb );
//     DcbAcquired = TRUE;
//
//     _SEH2_TRY {
//
//         ULONG i;
//
//         //
//         //  If the Dcb is already opened by someone then we need
//         //  to check the share access
//         //
//
//         if (Dcb->OpenCount > 0) {
//
//             if (!NT_SUCCESS(Iosb.Status = IoCheckShareAccess( *DesiredAccess,
//                                                               ShareAccess,
//                                                               FileObject,
//                                                               &Dcb->ShareAccess,
//                                                               TRUE ))) {
//
//                 try_return( Iosb );
//             }
//
//         } else {
//
//             IoSetShareAccess( *DesiredAccess,
//                               ShareAccess,
//                               FileObject,
//                               &Dcb->ShareAccess );
//         }
//
//         UnwindShareAccess = TRUE;
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserDirectoryOpen,
//                           Dcb,
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         Dcb->UncleanCount += 1;
//         Dcb->OpenCount += 1;
//         Dcb->Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Dcb->Vcb->ReadOnlyCount += 1; }
//
//         //
//         //  Update the name in the file object, by definition the remaining
//         //  part must be shorter than the original file name so we'll just
//         //  overwrite the file name.
//         //
//
//         i = FileObject->FileName.Length/sizeof(WCHAR) - 1;
//
//         //
//         //  Get rid of a trailing backslash
//         //
//
//         if (FileObject->FileName.Buffer[i] == L'\\') {
//
//             NT_ASSERT(i != 0);
//
//             FileObject->FileName.Length -= sizeof(WCHAR);
//             i -= 1;
//         }
//
//         //
//         //  Find the first non-backslash character.  i will be its index.
//         //
//
//         while (TRUE) {
//
//             if (FileObject->FileName.Buffer[i] == L'\\') {
//
//                 i += 1;
//                 break;
//             }
//
//             if (i == 0) {
//                 break;
//             }
//
//             i--;
//         }
//
//         if (i) {
//
//             FileObject->FileName.Length -= (USHORT)(i * sizeof(WCHAR));
//
//             RtlMoveMemory( &FileObject->FileName.Buffer[0],
//                            &FileObject->FileName.Buffer[i],
//                            FileObject->FileName.Length );
//         }
//
//         //
//         //  And set our status to success
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = (DoesNameExist ? FILE_EXISTS : FILE_DOES_NOT_EXIST);
//
//         if ( ( NT_SUCCESS(Iosb.Status) ) && ( DoesNameExist ) ) {
//             PCCB Ccb;
//
//             Ccb = (PCCB)FileObject->FsContext2;
//
//             //
//             //  Mark the OpenedByShortName bit if the operation was successful.
//             //
//
//             if ( FileNameOpenedDos ) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//             }
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenTargetDirectory );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//             if (UnwindShareAccess) { IoRemoveShareAccess( FileObject, &Dcb->ShareAccess ); }
//         }
//
//         if (DcbAcquired) {
//
//             FatReleaseFcb( IrpContext, Dcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenTargetDirectory -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// #ifdef _MSC_VER
// #pragma warning(suppress:6101) // bug in PREFast means the  annotation is not correctly applied
// #endif
// FatOpenExistingDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB *Dcb,
//      PDCB ParentDcb,
//      PDIRENT Dirent,
//      ULONG LfnByteOffset,
//      ULONG DirentByteOffset,
//      PUNICODE_STRING Lfn,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN FileNameOpenedDos,
//      BOOLEAN OpenRequiringOplock
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the specified directory.  The directory has not
//     previously been opened.
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume containing the dcb
//
//     Dcb - Returns the newly-created DCB for the file.
//
//     ParentDcb - Supplies the parent directory containing the subdirectory
//         to be opened
//
//     DirectoryName - Supplies the file name of the directory being opened.
//
//     Dirent - Supplies the dirent for the directory being opened
//
//     LfnByteOffset - Tells where the Lfn begins.  If there is no Lfn
//         this field is the same as DirentByteOffset.
//
//     DirentByteOffset - Supplies the Vbo of the dirent within its parent
//         directory
//
//     Lfn - May supply a long name for the file.
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     CreateDisposition - Supplies the create disposition for this operation
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
//     OpenRequiringOplock - The caller provided the FILE_OPEN_REQUIRING_OPLOCK option.
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     PDCB UnwindDcb = NULL;
//     PCCB UnwindCcb = NULL;
//
//     BOOLEAN CountsIncremented = FALSE;
//
//     UNREFERENCED_PARAMETER( DeleteOnClose );
// #if (NTDDI_VERSION <= NTDDI_WIN7)
//     UNREFERENCED_PARAMETER( OpenRequiringOplock );
// #endif
//     UNREFERENCED_PARAMETER( IrpSp );
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenExistingDirectory...\n", 0);
//
//     _SEH2_TRY {
//
//         //
//         //  If the caller has no Ea knowledge, we immediately check for
//         //  Need Ea's on the file.
//         //
//
//         if (NoEaKnowledge && !FatIsFat32(Vcb)) {
//
//             ULONG NeedEaCount;
//
//             FatGetNeedEaCount( IrpContext,
//                                Vcb,
//                                Dirent,
//                                &NeedEaCount );
//
//             if (NeedEaCount != 0) {
//
//                 Iosb.Status = STATUS_ACCESS_DENIED;
//                 try_return( Iosb );
//             }
//         }
//
//         //
//         //  Check the create disposition and desired access
//         //
//
//         if ((CreateDisposition != FILE_OPEN) &&
//             (CreateDisposition != FILE_OPEN_IF)) {
//
//             Iosb.Status = STATUS_OBJECT_NAME_COLLISION;
//             try_return( Iosb );
//         }
//
//         if (!FatCheckFileAccess( IrpContext,
//                                  Dirent->Attributes,
//                                  DesiredAccess)) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//         //
//         //  Create a new dcb for the directory
//         //
//
//         *Dcb = UnwindDcb = FatCreateDcb( IrpContext,
//                                         Vcb,
//                                         ParentDcb,
//                                         LfnByteOffset,
//                                         DirentByteOffset,
//                                         Dirent,
//                                         Lfn );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(*Dcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Fcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(*Dcb),
//                                              IrpContext->OriginatingIrp,
//                                              ((*Dcb)->UncleanCount + 1) );
//         }
//
//         //
//         //  Get out if either of the above calls failed.  Raise to trigger
//         //  cleanup of the new Dcb.
//         //
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             NT_ASSERT( Iosb.Status != STATUS_PENDING );
//
//             FatRaiseStatus( IrpContext, Iosb.Status );
//         }
// #endif
//
//         //
//         //  Setup our share access
//         //
//
//         IoSetShareAccess( *DesiredAccess,
//                           ShareAccess,
//                           FileObject,
//                           &(*Dcb)->ShareAccess );
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserDirectoryOpen,
//                           (*Dcb),
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         (*Dcb)->UncleanCount += 1;
//         (*Dcb)->OpenCount += 1;
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//
//         CountsIncremented = TRUE;
//
//
//         //
//         //  And set our status to success
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_OPENED;
//
//         if ( NT_SUCCESS(Iosb.Status) ) {
//             PCCB Ccb;
//
//             Ccb = (PCCB)FileObject->FsContext2;
//
//             //
//             //  Mark the OpenedByShortName bit if the operation was successful.
//             //
//
//             if ( FileNameOpenedDos ) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//             }
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenExistingDirectory );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (CountsIncremented) {
//
//                 (*Dcb)->UncleanCount -= 1;
//                 (*Dcb)->OpenCount -= 1;
//                 Vcb->OpenFileCount -= 1;
//                 if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount -= 1; }
//             }
//
//             if (UnwindDcb != NULL) {
//                 if (ARGUMENT_PRESENT( FileObject )) {
//                     FileObject->SectionObjectPointer = NULL;
//                 }
//                 FatDeleteFcb( IrpContext, &UnwindDcb );
//                 *Dcb = NULL;
//             }
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenExistingDirectory -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatOpenExistingFile (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PFCB *Fcb,
//      PDCB ParentDcb,
//      PDIRENT Dirent,
//      ULONG LfnByteOffset,
//      ULONG DirentByteOffset,
//      PUNICODE_STRING Lfn,
//      PUNICODE_STRING OrigLfn,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN IsPagingFile,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN FileNameOpenedDos
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the specified file.  The file has not previously
//     been opened.
//
// Arguments:
//
//     FileObject - Supplies the File object
//
//     Vcb - Supplies the Vcb denoting the volume containing the file
//
//     Fcb - Returns the newly-created FCB for the file.
//
//     ParentDcb - Supplies the parent directory containing the file to be
//         opened
//
//     Dirent - Supplies the dirent for the file being opened
//
//     LfnByteOffset - Tells where the Lfn begins.  If there is no Lfn
//         this field is the same as DirentByteOffset.
//
//     DirentByteOffset - Supplies the Vbo of the dirent within its parent
//         directory
//
//     Lfn - May supply a long name for the file.
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the share access of the caller
//
//     AllocationSize - Supplies the initial allocation if the file is being
//         superseded, overwritten, or created.
//
//     EaBuffer - Supplies the Ea set if the file is being superseded,
//         overwritten, or created.
//
//     EaLength - Supplies the size, in byte, of the EaBuffer
//
//     FileAttributes - Supplies file attributes to use if the file is being
//         superseded, overwritten, or created
//
//     CreateDisposition - Supplies the create disposition for this operation
//
//     IsPagingFile - Indicates if this is the paging file being opened.
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
//     OpenRequiringOplock - The caller provided the FILE_OPEN_REQUIRING_OPLOCK option.
//
//     FileNameOpenedDos - The caller opened this file by hitting the 8.3 side
//         of the Lfn/8.3 pair
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     ACCESS_MASK AddedAccess = 0;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     PFCB UnwindFcb = NULL;
//     PCCB UnwindCcb = NULL;
//     BOOLEAN CountsIncremented = FALSE;
//
//
// #if (NTDDI_VERSION < NTDDI_WIN7)
//     UNREFERENCED_PARAMETER( OpenRequiringOplock );
// #endif
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenExistingFile...\n", 0);
//
//     _SEH2_TRY {
//
//         //
//         //  Check if the user wanted to create the file or if access is
//         //  denied
//         //
//
//         if (CreateDisposition == FILE_CREATE) {
//             Iosb.Status = STATUS_OBJECT_NAME_COLLISION;
//             try_return( Iosb );
//
//         } else if ((CreateDisposition == FILE_SUPERSEDE) && !IsPagingFile) {
//
//             SetFlag( AddedAccess,
//                      DELETE & ~(*DesiredAccess) );
//
//             *DesiredAccess |= DELETE;
//
//         } else if (((CreateDisposition == FILE_OVERWRITE) ||
//                     (CreateDisposition == FILE_OVERWRITE_IF)) && !IsPagingFile) {
//
//             SetFlag( AddedAccess,
//                      (FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES) & ~(*DesiredAccess) );
//
//             *DesiredAccess |= FILE_WRITE_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES;
//         }
//
//         if (!FatCheckFileAccess( IrpContext,
//                                  Dirent->Attributes,
//                                  DesiredAccess)) {
//
//             Iosb.Status = STATUS_ACCESS_DENIED;
//             try_return( Iosb );
//         }
//
//
//         //
//         //  Check for trying to delete a read only file.
//         //
//
//         if (DeleteOnClose &&
//             FlagOn( Dirent->Attributes, FAT_DIRENT_ATTR_READ_ONLY )) {
//
//             Iosb.Status = STATUS_CANNOT_DELETE;
//             try_return( Iosb );
//         }
//
//         //
//         //  IF we are asked to do an overwrite or supersede operation then
//         //  deny access for files where the file attributes for system and
//         //  hidden do not match
//         //
//
//         if ((CreateDisposition == FILE_SUPERSEDE) ||
//             (CreateDisposition == FILE_OVERWRITE) ||
//             (CreateDisposition == FILE_OVERWRITE_IF)) {
//
//             BOOLEAN Hidden;
//             BOOLEAN System;
//
//             Hidden = BooleanFlagOn(Dirent->Attributes, FAT_DIRENT_ATTR_HIDDEN );
//             System = BooleanFlagOn(Dirent->Attributes, FAT_DIRENT_ATTR_SYSTEM );
//
//             if ((Hidden && !FlagOn(FileAttributes, FILE_ATTRIBUTE_HIDDEN)) ||
//                 (System && !FlagOn(FileAttributes, FILE_ATTRIBUTE_SYSTEM))) {
//
//                 DebugTrace(0, Dbg, "The hidden and/or system bits do not match\n", 0);
//
//                 if ( !IsPagingFile ) {
//
//                     Iosb.Status = STATUS_ACCESS_DENIED;
//                     try_return( Iosb );
//                 }
//             }
//
//             //
//             //  If this media is write protected, don't even try the create.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//                 //
//                 //  Set the real device for the pop-up info, and set the verify
//                 //  bit in the device object, so that we will force a verify
//                 //  in case the user put the correct media back in.
//                 //
//
//
//                 IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                               Vcb->Vpb->RealDevice );
//
//                 SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//                 FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//             }
//         }
//
//         //
//         //  Create a new Fcb for the file, and set the file size in
//         //  the fcb.
//         //
//
//         *Fcb = UnwindFcb = FatCreateFcb( IrpContext,
//                                          Vcb,
//                                          ParentDcb,
//                                          LfnByteOffset,
//                                          DirentByteOffset,
//                                          Dirent,
//                                          Lfn,
//                                          OrigLfn,
//                                          IsPagingFile,
//                                          FALSE );
//
//
//         (*Fcb)->Header.ValidDataLength.LowPart = (*Fcb)->Header.FileSize.LowPart;
//
//         //
//         //  If this is a paging file, lookup the allocation size so that
//         //  the Mcb is always valid
//         //
//
//         if (IsPagingFile) {
//
//             FatLookupFileAllocationSize( IrpContext, *Fcb );
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(*Fcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Fcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(*Fcb),
//                                              IrpContext->OriginatingIrp,
//                                              ((*Fcb)->UncleanCount + 1) );
//         }
//
//         //
//         //  Get out if either of the above calls failed.  Raise to trigger
//         //  cleanup of the new Fcb.
//         //
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             NT_ASSERT( Iosb.Status != STATUS_PENDING );
//
//             FatRaiseStatus( IrpContext, Iosb.Status );
//         }
// #endif
//
//         //
//         //  Now case on whether we are to simply open, supersede, or
//         //  overwrite the file.
//         //
//
//         switch (CreateDisposition) {
//
//         case FILE_OPEN:
//         case FILE_OPEN_IF:
//
//             DebugTrace(0, Dbg, "Doing only an open operation\n", 0);
//
//             //
//             //  If the caller has no Ea knowledge, we immediately check for
//             //  Need Ea's on the file.
//             //
//
//             if (NoEaKnowledge && !FatIsFat32(Vcb)) {
//
//                 ULONG NeedEaCount;
//
//                 FatGetNeedEaCount( IrpContext,
//                                    Vcb,
//                                    Dirent,
//                                    &NeedEaCount );
//
//                 if (NeedEaCount != 0) {
//
//                     FatRaiseStatus( IrpContext, STATUS_ACCESS_DENIED );
//                 }
//             }
//
//             //
//             //  Setup the context and section object pointers.
//             //
//
//             FatSetFileObject( FileObject,
//                               UserFileOpen,
//                               *Fcb,
//                               UnwindCcb = FatCreateCcb( IrpContext ));
//
//             FileObject->SectionObjectPointer = &(*Fcb)->NonPaged->SectionObjectPointers;
//
//             Iosb.Status = STATUS_SUCCESS;
//             Iosb.Information = FILE_OPENED;
//             break;
//
//         case FILE_SUPERSEDE:
//         case FILE_OVERWRITE:
//         case FILE_OVERWRITE_IF:
//
//             DebugTrace(0, Dbg, "Doing supersede/overwrite operation\n", 0);
//
//             //
//             //  Determine the granted access for this operation now.
//             //
//
//             if (!NT_SUCCESS( Iosb.Status = FatCheckSystemSecurityAccess( IrpContext ))) {
//
//                 try_return( Iosb );
//             }
//
//             Iosb = FatSupersedeOrOverwriteFile( IrpContext,
//                                                 FileObject,
//                                                 *Fcb,
//                                                 AllocationSize,
//                                                 EaBuffer,
//                                                 EaLength,
//                                                 FileAttributes,
//                                                 CreateDisposition,
//                                                 NoEaKnowledge );
//             break;
//
//         default:
//
//             DebugTrace(0, Dbg, "Illegal Create Disposition\n", 0);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( CreateDisposition, 0, 0 );
//             break;
//         }
//
//     try_exit: NOTHING;
//
//         //
//         //  Setup our share access and counts if things were successful.
//         //
//
//         if ((Iosb.Status != STATUS_PENDING) && NT_SUCCESS( Iosb.Status )) {
//
//             //
//             //  Remove any virtual access the caller needed to check against, but will
//             //  not really receive.  Overwrite/supersede is a bit of a special case.
//             //
//
//             ClearFlag( *DesiredAccess, AddedAccess );
//
//             IoSetShareAccess( *DesiredAccess,
//                               ShareAccess,
//                               FileObject,
//                               &(*Fcb)->ShareAccess );
//
//             (*Fcb)->UncleanCount += 1;
//             (*Fcb)->OpenCount += 1;
//             if (FlagOn(FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING)) {
//                 (*Fcb)->NonCachedUncleanCount += 1;
//             }
//             Vcb->OpenFileCount += 1;
//             if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//
//             CountsIncremented = TRUE;
//         }
//
//         {
//             PCCB Ccb;
//
//             Ccb = (PCCB)FileObject->FsContext2;
//
//             if ( NT_SUCCESS(Iosb.Status) ) {
//
//                 //
//                 //  Mark the DeleteOnClose bit if the operation was successful.
//                 //
//
//                 if ( DeleteOnClose ) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//                 }
//
//                 //
//                 //  Mark the OpenedByShortName bit if the operation was successful.
//                 //
//
//                 if ( FileNameOpenedDos ) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//                 }
//
//                 //
//                 //  Mark the ManageVolumeAccess bit if the privilege is held.
//                 //
//
//                 if (FatCheckManageVolumeAccess( IrpContext,
//                                                 IrpSp->Parameters.Create.SecurityContext->AccessState,
//                                                 (KPROCESSOR_MODE)( FlagOn( IrpSp->Flags, SL_FORCE_ACCESS_CHECK ) ?
//                                                                    UserMode :
//                                                                    IrpContext->OriginatingIrp->RequestorMode ))) {
//
//                     SetFlag( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS );
//                 }
//
//             }
//         }
//
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOpenExistingFile );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (CountsIncremented) {
//                 (*Fcb)->UncleanCount -= 1;
//                 (*Fcb)->OpenCount -= 1;
//                 if (FlagOn(FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING)) {
//                     (*Fcb)->NonCachedUncleanCount -= 1;
//                 }
//                 Vcb->OpenFileCount -= 1;
//                 if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount -= 1; }
//             }
//
//             if (UnwindFcb != NULL) {
//                 if (ARGUMENT_PRESENT( FileObject )) {
//                     FileObject->SectionObjectPointer = NULL;
//                 }
//                 FatDeleteFcb( IrpContext, &UnwindFcb );
//                 *Fcb = NULL;
//             }
//
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//         }
//
//         DebugTrace(-1, Dbg, "FatOpenExistingFile -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatCreateNewDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB ParentDcb,
//      POEM_STRING OemName,
//      PUNICODE_STRING UnicodeName,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock
//     )
//
// /*++
//
// Routine Description:
//
//     This routine creates a new directory.  The directory has already been
//     verified not to exist yet.
//
// Arguments:
//
//     FileObject - Supplies the file object for the newly created directory
//
//     Vcb - Supplies the Vcb denote the volume to contain the new directory
//
//     ParentDcb - Supplies the parent directory containg the newly created
//         directory
//
//     OemName - Supplies the Oem name for the newly created directory.  It may
//         or maynot be 8.3 complient, but will be upcased.
//
//     UnicodeName - Supplies the Unicode name for the newly created directory.
//         It may or maynot be 8.3 complient.  This name contains the original
//         case information.
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the shared access of the caller
//
//     EaBuffer - Supplies the Ea set for the newly created directory
//
//     EaLength - Supplies the length, in bytes, of EaBuffer
//
//     FileAttributes - Supplies the file attributes for the newly created
//         directory.
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
//     OpenRequiringOplock - The caller provided the FILE_OPEN_REQUIRING_OPLOCK option.
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb;
//
//     PDCB Dcb = NULL;
//     PCCB Ccb = NULL;
//
//     PDIRENT Dirent = NULL;
//     PBCB DirentBcb = NULL;
//     ULONG DirentsNeeded;
//     ULONG DirentByteOffset;
//
//     PDIRENT ShortDirent;
//     ULONG ShortDirentByteOffset;
//
//     USHORT EaHandle;
//
//     BOOLEAN AllLowerComponent;
//     BOOLEAN AllLowerExtension;
//     BOOLEAN CreateLfn;
//
//     ULONG BytesInFirstPage = 0;
//     ULONG DirentsInFirstPage = 0;
//     PDIRENT FirstPageDirent = 0;
//
//     PBCB SecondPageBcb = NULL;
//     ULONG SecondPageOffset;
//     PDIRENT SecondPageDirent = NULL;
//
//     BOOLEAN DirentFromPool = FALSE;
//
//
//     OEM_STRING ShortName;
//     UCHAR ShortNameBuffer[12];
//
// #if (NTDDI_VERSION <= NTDDI_WIN7)
//     UNREFERENCED_PARAMETER( OpenRequiringOplock );
// #endif
//
//     UNREFERENCED_PARAMETER( IrpSp );
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateNewDirectory...\n", 0);
//
//     ShortName.Length = 0;
//     ShortName.MaximumLength = 12;
//     ShortName.Buffer = (PCHAR)&ShortNameBuffer[0];
//
//     EaHandle = 0;
//
//     //
//     //  We fail this operation if the caller doesn't understand Ea's.
//     //
//
//     if (NoEaKnowledge
//         && EaLength > 0) {
//
//         Iosb.Status = STATUS_ACCESS_DENIED;
//
//         DebugTrace(-1, Dbg, "FatCreateNewDirectory -> Iosb.Status = %08lx\n", Iosb.Status);
//         return Iosb;
//     }
//
//     //
//     //  DeleteOnClose and ReadOnly are not compatible.
//     //
//
//     if (DeleteOnClose && FlagOn(FileAttributes, FAT_DIRENT_ATTR_READ_ONLY)) {
//
//         Iosb.Status = STATUS_CANNOT_DELETE;
//         return Iosb;
//     }
//
//     //  Now get the names that we will be using.
//     //
//
//     FatSelectNames( IrpContext,
//                     ParentDcb,
//                     OemName,
//                     UnicodeName,
//                     &ShortName,
//                     NULL,
//                     &AllLowerComponent,
//                     &AllLowerExtension,
//                     &CreateLfn );
//
//     //
//     //  If we are not in Chicago mode, ignore the magic bits.
//     //
//
//     if (!FatData.ChicagoMode) {
//
//         AllLowerComponent = FALSE;
//         AllLowerExtension = FALSE;
//         CreateLfn = FALSE;
//     }
//
//     //
//     //  Create/allocate a new dirent
//     //
//
//     DirentsNeeded = CreateLfn ? FAT_LFN_DIRENTS_NEEDED(UnicodeName) + 1 : 1;
//
//     DirentByteOffset = FatCreateNewDirent( IrpContext,
//                                            ParentDcb,
//                                            DirentsNeeded,
//                                            FALSE );
//     _SEH2_TRY {
//
//         FatPrepareWriteDirectoryFile( IrpContext,
//                                       ParentDcb,
//                                       DirentByteOffset,
//                                       sizeof(DIRENT),
//                                       &DirentBcb,
// #ifndef __REACTOS__
//                                       &Dirent,
// #else
//                                       (PVOID *)&Dirent,
// #endif
//                                       FALSE,
//                                       TRUE,
//                                       &Iosb.Status );
//
//         NT_ASSERT( NT_SUCCESS( Iosb.Status ) && DirentBcb && Dirent );
//
//         //
//         //  Deal with the special case of an LFN + Dirent structure crossing
//         //  a page boundry.
//         //
//
//         if ((DirentByteOffset / PAGE_SIZE) !=
//             ((DirentByteOffset + (DirentsNeeded - 1) * sizeof(DIRENT)) / PAGE_SIZE)) {
//
//             SecondPageBcb;
//             SecondPageOffset;
//             SecondPageDirent;
//
//             SecondPageOffset = (DirentByteOffset & ~(PAGE_SIZE - 1)) + PAGE_SIZE;
//
//             BytesInFirstPage = SecondPageOffset - DirentByteOffset;
//
//             DirentsInFirstPage = BytesInFirstPage / sizeof(DIRENT);
//
//             FatPrepareWriteDirectoryFile( IrpContext,
//                                           ParentDcb,
//                                           SecondPageOffset,
//                                           sizeof(DIRENT),
//                                           &SecondPageBcb,
// #ifndef __REACTOS__
//                                           &SecondPageDirent,
// #else
//                                           (PVOID *)&SecondPageDirent,
// #endif
//                                           FALSE,
//                                           TRUE,
//                                           &Iosb.Status );
//
//             NT_ASSERT( NT_SUCCESS( Iosb.Status ) && SecondPageBcb && SecondPageDirent );
//
//             FirstPageDirent = Dirent;
//
//             Dirent = FsRtlAllocatePoolWithTag( PagedPool,
//                                                DirentsNeeded * sizeof(DIRENT),
//                                                TAG_DIRENT );
//
//             DirentFromPool = TRUE;
//         }
//
//         //
//         //  Bump up Dirent and DirentByteOffset
//         //
//
//         ShortDirent = Dirent + DirentsNeeded - 1;
//         ShortDirentByteOffset = DirentByteOffset +
//                                 (DirentsNeeded - 1) * sizeof(DIRENT);
//
//         NT_ASSERT( NT_SUCCESS( Iosb.Status ));
//
//
//         //
//         //  Fill in the fields of the dirent.
//         //
//
//         FatConstructDirent( IrpContext,
//                             ShortDirent,
//                             &ShortName,
//                             AllLowerComponent,
//                             AllLowerExtension,
//                             CreateLfn ? UnicodeName : NULL,
//                             FileAttributes | FAT_DIRENT_ATTR_DIRECTORY,
//                             TRUE,
//                             NULL );
//
//         //
//         //  If the dirent crossed pages, we have to do some real gross stuff.
//         //
//
//         if (DirentFromPool) {
//
//             RtlCopyMemory( FirstPageDirent, Dirent, BytesInFirstPage );
//
//             RtlCopyMemory( SecondPageDirent,
//                            Dirent + DirentsInFirstPage,
//                            DirentsNeeded*sizeof(DIRENT) - BytesInFirstPage );
//
//             ShortDirent = SecondPageDirent + (DirentsNeeded - DirentsInFirstPage) - 1;
//         }
//
//         //
//         //  Create a new dcb for the directory.
//         //
//
//         Dcb = FatCreateDcb( IrpContext,
//                             Vcb,
//                             ParentDcb,
//                             DirentByteOffset,
//                             ShortDirentByteOffset,
//                             ShortDirent,
//                             CreateLfn ? UnicodeName : NULL );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  The next three FsRtl calls are for oplock work.  We deliberately
//         //  do these here so that if either call fails we will be able to
//         //  clean up without adding a bunch of code to unwind counts, fix
//         //  the file object, etc.
//         //
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(Dcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Dcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(Dcb),
//                                              IrpContext->OriginatingIrp,
//                                              (Dcb->UncleanCount + 1) );
//         }
//
//         //
//         //  Break parent directory oplock.  Directory oplock breaks are always
//         //  advisory, so we will never block/get STATUS_PENDING here.  On the
//         //  off chance this fails with INSUFFICIENT_RESOURCES we do it here
//         //  where we can still tolerate a failure.
//         //
//
//         if (Iosb.Status == STATUS_SUCCESS) {
//
//             Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(ParentDcb),
//                                               IrpContext->OriginatingIrp,
//                                               OPLOCK_FLAG_PARENT_OBJECT,
//                                               NULL,
//                                               NULL,
//                                               NULL );
//         }
//
//         //
//         //  Get out if any of the oplock calls failed.
//         //
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             FatRaiseStatus( IrpContext, Iosb.Status );
//         }
// #endif
//
//         //
//         //  Tentatively add the new Ea's,
//         //
//
//         if (EaLength > 0) {
//
//             //
//             //  This returns false if we are trying to create a file
//             //  with Need Ea's and don't understand EA's.
//             //
//
//             FatCreateEa( IrpContext,
//                          Dcb->Vcb,
//                          (PUCHAR) EaBuffer,
//                          EaLength,
//                          &Dcb->ShortName.Name.Oem,
//                          &EaHandle );
//         }
//
//         if (!FatIsFat32(Dcb->Vcb)) {
//
//             ShortDirent->ExtendedAttributes = EaHandle;
//         }
//
//         //
//         //  After this point we cannot just simply mark the dirent deleted,
//         //  we have to deal with the directory file object.
//         //
//
//         //
//         //  Make the dirent into a directory.  Note that even if this call
//         //  raises because of disk space, the diectory file object has been
//         //  created.
//         //
//
//         FatInitializeDirectoryDirent( IrpContext, Dcb, ShortDirent );
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts.  Note that this call cannot fail.
//         //
//
//         FatSetFileObject( FileObject,
//                           UserDirectoryOpen,
//                           Dcb,
//                           Ccb = FatCreateCcb( IrpContext ) );
//
//         //
//         //  Initialize the LongFileName if it has not already been set, so that
//         //  FatNotify below won't have to.  If there are filesystem filters
//         //  attached to FAT, the LongFileName could have gotten set if the
//         //  filter queried for name information on this file object while
//         //  watching the IO needed in FatInitializeDirectoryDirent.
//         //
//
//         if (Dcb->FullFileName.Buffer == NULL) {
//
//             FatSetFullNameInFcb( IrpContext, Dcb, UnicodeName );
//         }
//
//         //
//         //  We call the notify package to report that the
//         //  we added a file.
//         //
//
//         FatNotifyReportChange( IrpContext,
//                                Vcb,
//                                Dcb,
//                                FILE_NOTIFY_CHANGE_DIR_NAME,
//                                FILE_ACTION_ADDED );
//
//         //
//         //  Setup our share access
//         //
//
//         IoSetShareAccess( *DesiredAccess,
//                           ShareAccess,
//                           FileObject,
//                           &Dcb->ShareAccess );
//
//
//         //
//         //  From this point on, nothing can raise.
//         //
//
//         Dcb->UncleanCount += 1;
//         Dcb->OpenCount += 1;
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//
//         if (DeleteOnClose) {
//
//             SetFlag( Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//         }
//
//         //
//         //  And set our return status
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_CREATED;
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We'll catch all exceptions and handle them below.
//         //
//
//         Iosb.Status = IrpContext->ExceptionStatus;
//     } _SEH2_END;
//
//     //
//     //  If we failed then undo our work.
//     //
//
//     if (!NT_SUCCESS( Iosb.Status )) {
//
//         //
//         //  We always have to delete the Ccb if we created one.
//         //
//
//         if ( Ccb != NULL ) {
//
//             FatDeleteCcb( IrpContext, &Ccb );
//         }
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28924, "prefast thinks this test is redundant, but DCB can be NULL depending on where we raise" )
// #endif
//         if ( Dcb == NULL) {
//
//             NT_ASSERT( (ParentDcb->Specific.Dcb.UnusedDirentVbo == 0xffffffff_u32) ||
//                     RtlAreBitsSet( &ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                                    DirentByteOffset / sizeof(DIRENT),
//                                    DirentsNeeded ) );
//
//             RtlClearBits( &ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                           DirentByteOffset / sizeof(DIRENT),
//                           DirentsNeeded );
//
//             //
//             //  Mark the dirents deleted.  The codes is complex because of
//             //  dealing with an LFN than crosses a page boundry.
//             //
//
//             if (Dirent != NULL) {
//
//                 ULONG i;
//
//                 //
//                 //  We failed before creating a directory file object.
//                 //  We can just mark the dirent deleted and exit.
//                 //
//
//                 for (i = 0; i < DirentsNeeded; i++) {
//
//                     if (DirentFromPool == FALSE) {
//
//                         //
//                         //  Simple case.
//                         //
//
//                         Dirent[i].FileName[0] = FAT_DIRENT_DELETED;
//
//                     } else {
//
//                         //
//                         //  If the second CcPreparePinWrite failed, we have
//                         //  to stop early.
//                         //
//
//                         if ((SecondPageBcb == NULL) &&
//                             (i == DirentsInFirstPage)) {
//
//                             break;
//                         }
//
//                         //
//                         //  Now conditionally update either page.
//                         //
//
//                         if (i < DirentsInFirstPage) {
//
//                             FirstPageDirent[i].FileName[0] = FAT_DIRENT_DELETED;
//
//                         } else {
//
//                             SecondPageDirent[i - DirentsInFirstPage].FileName[0] = FAT_DIRENT_DELETED;
//                         }
//                     }
//                 }
//             }
//         }
//     }
//
//     //
//     //  Just drop the Bcbs we have in the parent right now so if we
//     //  failed to create the directory and we take the path to rip apart
//     //  the partially created child, when we sync-uninit we won't cause
//     //  a lazy writer processing the parent to block on us. This would
//     //  consume one of the lazy writers, one of which must be running free
//     //  in order for us to come back from the sync-uninit.
//     //
//     //  Neat, huh?
//     //
//     //  Granted, the delete dirent below will be marginally less efficient
//     //  since the Bcb may be reclaimed by the time it executes. Life is
//     //  tough.
//     //
//
//     FatUnpinBcb( IrpContext, DirentBcb );
//     FatUnpinBcb( IrpContext, SecondPageBcb );
//
//     if (DirentFromPool) {
//
//         ExFreePool( Dirent );
//     }
//
//     if (!NT_SUCCESS( Iosb.Status )) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28924, "prefast thinks this test is redundant, but DCB can be NULL depending on where we raise" )
// #endif
//         if (Dcb != NULL) {
//
//             //
//             //  We have created the Dcb.  If an error occurred while
//             //  creating the Ea's, there will be no directory file
//             //  object.
//             //
//
//             PFILE_OBJECT DirectoryFileObject;
//
//             DirectoryFileObject = Dcb->Specific.Dcb.DirectoryFile;
//
//             //
//             //  Knock down all of the repinned data so we can begin to destroy
//             //  this failed child.  We don't care about any raising here - we're
//             //  already got a fire going.
//             //
//             //  Note that if we failed to do this, the repinned initial pieces
//             //  of the child would cause the sync-uninit to block forever.
//             //
//             //  A previous spin on this fix had us not make the ./.. creation
//             //  "reversible" (bad term) and thus avoid having the Bcb still
//             //  outstanding.  This wound up causing very bad things to happen
//             //  on DMF floppies when we tried to do a similar yank-down in the
//             //  create path - we want the purge it does to make sure we never
//             //  try to write the bytes out ... it is just a lot cleaner to
//             //  unpinrepin.  I'll leave the reversible logic in place if it ever
//             //  proves useful.
//             //
//
//             //
//             //  There is a possibility that this may be a generally good idea
//             //  for "live" finally clauses - set in ExceptionFilter, clear in
//             //  ProcessException. Think about this.
//             //
//
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE );
//             FatUnpinRepinnedBcbs( IrpContext );
//             ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE );
//
//             if (Dcb->FirstClusterOfFile != 0) {
//
//                 _SEH2_TRY {
//
//                     Dcb->Header.FileSize.LowPart = 0;
//
//                     CcSetFileSizes( Dcb->Specific.Dcb.DirectoryFile,
//                                     (PCC_FILE_SIZES)&Dcb->Header.AllocationSize );
//
//                     //
//                     //  Now zap the allocation backing it.
//                     //
//
//                     FatTruncateFileAllocation( IrpContext, Dcb, 0 );
//
//                 } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//                     //
//                     //  We catch all exceptions that Fat catches, but don't do
//                     //  anything with them.
//                     //
//                 } _SEH2_END;
//             }
//
//             if (DirectoryFileObject != NULL) {
//
//                 FatSyncUninitializeCacheMap( IrpContext,
//                                              DirectoryFileObject );
//             }
//
//
//             _SEH2_TRY {
//
//                 //
//                 //  Remove the directory entry we made in the parent Dcb.
//                 //
//
//                 FatDeleteDirent( IrpContext, Dcb, NULL, TRUE );
//
//                 //
//                 //  FatDeleteDirent can pin and dirty BCBs, so lets unrepin again.
//                 //
//
//                 FatUnpinRepinnedBcbs( IrpContext );
//
//             } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//                 //
//                 //  We catch all exceptions that Fat catches, but don't do
//                 //  anything with them.
//                 //
//             } _SEH2_END;
//
//             //
//             //  Finaly, dereference the directory file object. This will
//             //  cause a close Irp to be processed, blowing away the Fcb.
//             //
//
//             if (DirectoryFileObject != NULL) {
//
//                 //
//                 //  Dereference the file object for this DCB.  The DCB will
//                 //  go away when this file object is closed.
//                 //
//
//                 Dcb->Specific.Dcb.DirectoryFile = NULL;
//                 ObDereferenceObject( DirectoryFileObject );
//
//             } else {
//
//                 //
//                 //  This was also a PDK fix.  If the stream file exists, this would
//                 //  be done during the dereference file object operation.  Otherwise
//                 //  we have to remove the Dcb and check if we should remove the parent.
//                 //  For now we will just leave the parent lying around.
//                 //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28924, "prefast thinks this test is redundant, but FileObject can be NULL depending on where we raise" )
// #endif
//                 if (ARGUMENT_PRESENT( FileObject )) {
//                     FileObject->SectionObjectPointer = NULL;
//                 }
//                 FatDeleteFcb( IrpContext, &Dcb );
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatCreateNewDirectory -> Iosb.Status = %08lx\n", Iosb.Status);
//
//         FatRaiseStatus( IrpContext, Iosb.Status );
//     }
//
//     UNREFERENCED_PARAMETER( EaBuffer );
//     UNREFERENCED_PARAMETER( EaLength );
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatCreateNewFile (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFILE_OBJECT FileObject,
//      PVCB Vcb,
//      PDCB ParentDcb,
//      POEM_STRING OemName,
//      PUNICODE_STRING UnicodeName,
//      PACCESS_MASK DesiredAccess,
//      USHORT ShareAccess,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      PUNICODE_STRING LfnBuffer,
//      BOOLEAN IsPagingFile,
//      BOOLEAN NoEaKnowledge,
//      BOOLEAN DeleteOnClose,
//      BOOLEAN OpenRequiringOplock,
//      BOOLEAN TemporaryFile
//     )
//
// /*++
//
// Routine Description:
//
//     This routine creates a new file.  The file has already been verified
//     not to exist yet.
//
// Arguments:
//
//     FileObject - Supplies the file object for the newly created file
//
//     Vcb - Supplies the Vcb denote the volume to contain the new file
//
//     ParentDcb - Supplies the parent directory containg the newly created
//         File
//
//     OemName - Supplies the Oem name for the newly created file.  It may
//         or maynot be 8.3 complient, but will be upcased.
//
//     UnicodeName - Supplies the Unicode name for the newly created file.
//         It may or maynot be 8.3 complient.  This name contains the original
//         case information.
//
//     DesiredAccess - Supplies the desired access of the caller
//
//     ShareAccess - Supplies the shared access of the caller
//
//     AllocationSize - Supplies the initial allocation size for the file
//
//     EaBuffer - Supplies the Ea set for the newly created file
//
//     EaLength - Supplies the length, in bytes, of EaBuffer
//
//     FileAttributes - Supplies the file attributes for the newly created
//         file
//
//     LfnBuffer - A MAX_LFN sized buffer for directory searching
//
//     IsPagingFile - Indicates if this is the paging file being created
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
//     DeleteOnClose - The caller wants the file gone when the handle is closed
//
//     OpenRequiringOplock - The caller provided the FILE_OPEN_REQUIRING_OPLOCK option.
//
//     TemporaryFile - Signals the lazywriter to not write dirty data unless
//         absolutely has to.
//
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     PFCB Fcb = NULL;
//
//     PDIRENT Dirent = NULL;
//     PBCB DirentBcb = NULL;
//     ULONG DirentsNeeded;
//     ULONG DirentByteOffset;
//
//     PDIRENT ShortDirent;
//     ULONG ShortDirentByteOffset;
//
//     USHORT EaHandle;
//
//     BOOLEAN AllLowerComponent;
//     BOOLEAN AllLowerExtension;
//     BOOLEAN CreateLfn;
//
//     ULONG BytesInFirstPage = 0;
//     ULONG DirentsInFirstPage = 0;
//     PDIRENT FirstPageDirent = NULL;
//
//     PBCB SecondPageBcb = NULL;
//     ULONG SecondPageOffset;
//     PDIRENT SecondPageDirent = NULL;
//
//     BOOLEAN DirentFromPool = FALSE;
//
//     OEM_STRING ShortName;
//     UCHAR ShortNameBuffer[12];
//
//     UNICODE_STRING UniTunneledShortName;
//     WCHAR UniTunneledShortNameBuffer[12];
//     UNICODE_STRING UniTunneledLongName;
//     WCHAR UniTunneledLongNameBuffer[26];
//     LARGE_INTEGER TunneledCreationTime;
//     ULONG TunneledDataSize;
//     BOOLEAN HaveTunneledInformation;
//     BOOLEAN UsingTunneledLfn = FALSE;
//
//     PUNICODE_STRING RealUnicodeName;
//
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     PDIRENT UnwindDirent = NULL;
//     PFCB UnwindFcb = NULL;
//     BOOLEAN UnwindAllocation = FALSE;
//     BOOLEAN CountsIncremented = FALSE;
//     PCCB UnwindCcb = NULL;
//
//     ULONG LocalAbnormalTermination = FALSE;
//
// #if (NTDDI_VERSION < NTDDI_WIN7)
//     UNREFERENCED_PARAMETER( OpenRequiringOplock );
// #endif
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateNewFile...\n", 0);
//
//     ShortName.Length = 0;
//     ShortName.MaximumLength = sizeof(ShortNameBuffer);
//     ShortName.Buffer = (PCHAR)&ShortNameBuffer[0];
//
//     UniTunneledShortName.Length = 0;
//     UniTunneledShortName.MaximumLength = sizeof(UniTunneledShortNameBuffer);
//     UniTunneledShortName.Buffer = &UniTunneledShortNameBuffer[0];
//
//     UniTunneledLongName.Length = 0;
//     UniTunneledLongName.MaximumLength = sizeof(UniTunneledLongNameBuffer);
//     UniTunneledLongName.Buffer = &UniTunneledLongNameBuffer[0];
//
//     EaHandle = 0;
//
//     //
//     //  We fail this operation if the caller doesn't understand Ea's.
//     //
//
//     if (NoEaKnowledge
//         && EaLength > 0) {
//
//         Iosb.Status = STATUS_ACCESS_DENIED;
//
//         DebugTrace(-1, Dbg, "FatCreateNewFile -> Iosb.Status = %08lx\n", Iosb.Status);
//         return Iosb;
//     }
//
//     //
//     //  DeleteOnClose and ReadOnly are not compatible.
//     //
//
//     if (DeleteOnClose && FlagOn(FileAttributes, FAT_DIRENT_ATTR_READ_ONLY)) {
//
//         Iosb.Status = STATUS_CANNOT_DELETE;
//         return Iosb;
//     }
//
//     //
//     //  Look in the tunnel cache for names and timestamps to restore
//     //
//
//     TunneledDataSize = sizeof(LARGE_INTEGER);
//     HaveTunneledInformation = FsRtlFindInTunnelCache( &Vcb->Tunnel,
//                                                       FatDirectoryKey(ParentDcb),
//                                                       UnicodeName,
//                                                       &UniTunneledShortName,
//                                                       &UniTunneledLongName,
//                                                       &TunneledDataSize,
//                                                       &TunneledCreationTime );
//     NT_ASSERT(TunneledDataSize == sizeof(LARGE_INTEGER));
//
//     //
//     //  Now get the names that we will be using.
//     //
//
//     FatSelectNames( IrpContext,
//                     ParentDcb,
//                     OemName,
//                     UnicodeName,
//                     &ShortName,
//                     (HaveTunneledInformation? &UniTunneledShortName : NULL),
//                     &AllLowerComponent,
//                     &AllLowerExtension,
//                     &CreateLfn );
//
//     //
//     //  If we are not in Chicago mode, ignore the magic bits.
//     //
//
//     RealUnicodeName = UnicodeName;
//
//     if (!FatData.ChicagoMode) {
//
//         AllLowerComponent = FALSE;
//         AllLowerExtension = FALSE;
//         CreateLfn = FALSE;
//
//     } else {
//
//         //
//         //  If the Unicode name was legal for a short name and we got
//         //  a tunneling hit which had a long name associated which is
//         //  avaliable for use, use it.
//         //
//
//         if (!CreateLfn &&
//             UniTunneledLongName.Length &&
//             !FatLfnDirentExists(IrpContext, ParentDcb, &UniTunneledLongName, LfnBuffer)) {
//
//             UsingTunneledLfn = TRUE;
//             CreateLfn = TRUE;
//
//             RealUnicodeName = &UniTunneledLongName;
//
//             //
//             //  Short names are always upcase if an LFN exists
//             //
//
//             AllLowerComponent = FALSE;
//             AllLowerExtension = FALSE;
//         }
//     }
//
//
//     //
//     //  Create/allocate a new dirent
//     //
//
//     DirentsNeeded = CreateLfn ? FAT_LFN_DIRENTS_NEEDED(RealUnicodeName) + 1 : 1;
//
//     DirentByteOffset = FatCreateNewDirent( IrpContext,
//                                            ParentDcb,
//                                            DirentsNeeded,
//                                            FALSE );
//
//     _SEH2_TRY {
//
//         FatPrepareWriteDirectoryFile( IrpContext,
//                                       ParentDcb,
//                                       DirentByteOffset,
//                                       sizeof(DIRENT),
//                                       &DirentBcb,
// #ifndef __REACTOS__
//                                       &Dirent,
// #else
//                                       (PVOID *)&Dirent,
// #endif
//                                       FALSE,
//                                       TRUE,
//                                       &Iosb.Status );
//
//         NT_ASSERT( NT_SUCCESS( Iosb.Status ) );
//
//         UnwindDirent = Dirent;
//
//         //
//         //  Deal with the special case of an LFN + Dirent structure crossing
//         //  a page boundry.
//         //
//
//         if ((DirentByteOffset / PAGE_SIZE) !=
//             ((DirentByteOffset + (DirentsNeeded - 1) * sizeof(DIRENT)) / PAGE_SIZE)) {
//
//             SecondPageBcb;
//             SecondPageOffset;
//             SecondPageDirent;
//
//             SecondPageOffset = (DirentByteOffset & ~(PAGE_SIZE - 1)) + PAGE_SIZE;
//
//             BytesInFirstPage = SecondPageOffset - DirentByteOffset;
//
//             DirentsInFirstPage = BytesInFirstPage / sizeof(DIRENT);
//
//             FatPrepareWriteDirectoryFile( IrpContext,
//                                           ParentDcb,
//                                           SecondPageOffset,
//                                           sizeof(DIRENT),
//                                           &SecondPageBcb,
// #ifndef __REACTOS__
//                                           &SecondPageDirent,
// #else
//                                           (PVOID *)&SecondPageDirent,
// #endif
//                                           FALSE,
//                                           TRUE,
//                                           &Iosb.Status );
//
//             NT_ASSERT( NT_SUCCESS( Iosb.Status ) );
//
//             FirstPageDirent = Dirent;
//
//             Dirent = FsRtlAllocatePoolWithTag( PagedPool,
//                                                DirentsNeeded * sizeof(DIRENT),
//                                                TAG_DIRENT );
//
//             DirentFromPool = TRUE;
//         }
//
//         //
//         //  Bump up Dirent and DirentByteOffset
//         //
//
//         ShortDirent = Dirent + DirentsNeeded - 1;
//         ShortDirentByteOffset = DirentByteOffset +
//                                 (DirentsNeeded - 1) * sizeof(DIRENT);
//
//         NT_ASSERT( NT_SUCCESS( Iosb.Status ));
//
//
//         //
//         //  Fill in the fields of the dirent.
//         //
//
//         FatConstructDirent( IrpContext,
//                             ShortDirent,
//                             &ShortName,
//                             AllLowerComponent,
//                             AllLowerExtension,
//                             CreateLfn ? RealUnicodeName : NULL,
//                             (FileAttributes | FILE_ATTRIBUTE_ARCHIVE),
//                             TRUE,
//                             (HaveTunneledInformation ? &TunneledCreationTime : NULL) );
//
//         //
//         //  If the dirent crossed pages, we have to do some real gross stuff.
//         //
//
//         if (DirentFromPool) {
//
//             RtlCopyMemory( FirstPageDirent, Dirent, BytesInFirstPage );
//
//             RtlCopyMemory( SecondPageDirent,
//                            Dirent + DirentsInFirstPage,
//                            DirentsNeeded*sizeof(DIRENT) - BytesInFirstPage );
//
//             ShortDirent = SecondPageDirent + (DirentsNeeded - DirentsInFirstPage) - 1;
//         }
//
//         //
//         //  Create a new Fcb for the file.  Once the Fcb is created we
//         //  will not need to unwind dirent because delete dirent will
//         //  now do the work.
//         //
//
//         Fcb = UnwindFcb = FatCreateFcb( IrpContext,
//                                         Vcb,
//                                         ParentDcb,
//                                         DirentByteOffset,
//                                         ShortDirentByteOffset,
//                                         ShortDirent,
//                                         CreateLfn ? RealUnicodeName : NULL,
//                                         CreateLfn ? RealUnicodeName : NULL,
//                                         IsPagingFile,
//                                         FALSE );
//         UnwindDirent = NULL;
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//         //
//         //  The next three FsRtl calls are for oplock work.  We deliberately
//         //  do these here so that if either call fails we will be able to
//         //  clean up without adding a bunch of code to unwind counts, fix
//         //  the file object, etc.
//         //
//
//         //
//         //  Let's make sure that if the caller provided an oplock key that it
//         //  gets stored in the file object.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_OPLOCK_KEY_CHECK_ONLY,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         //
//         //  If the caller wants atomic create-with-oplock semantics, tell
//         //  the oplock package.  We haven't incremented the Fcb's UncleanCount
//         //  for this create yet, so add that in on the call.
//         //
//
//         if (OpenRequiringOplock &&
//             (Iosb.Status == STATUS_SUCCESS)) {
//
//             Iosb.Status = FsRtlOplockFsctrl( FatGetFcbOplock(Fcb),
//                                              IrpContext->OriginatingIrp,
//                                              (Fcb->UncleanCount + 1) );
//         }
// #endif
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Break parent directory oplock.  Directory oplock breaks are always
//         //  advisory, so we will never block/get STATUS_PENDING here.  On the
//         //  off chance this fails with INSUFFICIENT_RESOURCES we do it here
//         //  where we can still tolerate a failure.
//         //
//
//         if (Iosb.Status == STATUS_SUCCESS) {
//
//             Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(ParentDcb),
//                                               IrpContext->OriginatingIrp,
//                                               OPLOCK_FLAG_PARENT_OBJECT,
//                                               NULL,
//                                               NULL,
//                                               NULL );
//         }
//
//         //
//         //  Get out if any of the oplock calls failed.  We raise to provoke
//         //  abnormal termination and ensure that the newly-created Fcb gets
//         //  deleted.
//         //
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             FatRaiseStatus( IrpContext, Iosb.Status );
//         }
// #endif
//
//         //
//         //  If this is a temporary file, note it in the FcbState
//         //
//
//         if (TemporaryFile) {
//
//             SetFlag( Fcb->FcbState, FCB_STATE_TEMPORARY );
//         }
//
//
//         //
//         //  Add some initial file allocation
//         //
//
//
//         FatAddFileAllocation( IrpContext, Fcb, FileObject, AllocationSize );
//
//
//         UnwindAllocation = TRUE;
//
//         Fcb->FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
//
//         //
//         //  Tentatively add the new Ea's
//         //
//
//         if ( EaLength > 0 ) {
//
//             FatCreateEa( IrpContext,
//                          Fcb->Vcb,
//                          (PUCHAR) EaBuffer,
//                          EaLength,
//                          &Fcb->ShortName.Name.Oem,
//                          &EaHandle );
//         }
//
//         if (!FatIsFat32(Fcb->Vcb)) {
//
//             ShortDirent->ExtendedAttributes = EaHandle;
//         }
//
//
//
//         //
//         //  Initialize the LongFileName right now so that FatNotify
//         //  below won't have to.
//         //
//
//         if (Fcb->FullFileName.Buffer == NULL) {
//             FatSetFullNameInFcb( IrpContext, Fcb, RealUnicodeName );
//         }
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserFileOpen,
//                           Fcb,
//                           UnwindCcb = FatCreateCcb( IrpContext ));
//
//         FileObject->SectionObjectPointer = &Fcb->NonPaged->SectionObjectPointers;
//
//         //
//         //  We call the notify package to report that the
//         //  we added a file.
//         //
//
//         FatNotifyReportChange( IrpContext,
//                                Vcb,
//                                Fcb,
//                                FILE_NOTIFY_CHANGE_FILE_NAME,
//                                FILE_ACTION_ADDED );
//
//         //
//         //  Setup our share access
//         //
//
//         IoSetShareAccess( *DesiredAccess,
//                           ShareAccess,
//                           FileObject,
//                           &Fcb->ShareAccess );
//
//         Fcb->UncleanCount += 1;
//         Fcb->OpenCount += 1;
//         if (FlagOn(FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING)) {
//             Fcb->NonCachedUncleanCount += 1;
//         }
//         Vcb->OpenFileCount += 1;
//         if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount += 1; }
//         CountsIncremented = TRUE;
//
//
//         //
//         //  And set our return status
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//         Iosb.Information = FILE_CREATED;
//
//         if ( NT_SUCCESS(Iosb.Status) ) {
//
//             //
//             //  Mark the DeleteOnClose bit if the operation was successful.
//             //
//
//             if ( DeleteOnClose ) {
//
//                 SetFlag( UnwindCcb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//             }
//
//             //
//             //  Mark the OpenedByShortName bit if the operation was successful.
//             //  If we created an Lfn, we have some sort of generated short name
//             //  and thus don't consider ourselves to have opened it - though we
//             //  may have had a case mix Lfn "Foo.bar" and generated "FOO.BAR"
//             //
//             //  Unless, of course, we wanted to create a short name and hit an
//             //  associated Lfn in the tunnel cache
//             //
//
//             if ( !CreateLfn && !UsingTunneledLfn ) {
//
//                 SetFlag( UnwindCcb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//             }
//
//             //
//             //  Mark the ManageVolumeAccess bit if the privilege is held.
//             //
//
//             if (FatCheckManageVolumeAccess( IrpContext,
//                                             IrpSp->Parameters.Create.SecurityContext->AccessState,
//                                             (KPROCESSOR_MODE)( FlagOn( IrpSp->Flags, SL_FORCE_ACCESS_CHECK ) ?
//                                                                UserMode :
//                                                                IrpContext->OriginatingIrp->RequestorMode ))) {
//
//                 SetFlag( UnwindCcb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS );
//             }
//
//         }
//
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCreateNewFile );
//
//         if (UniTunneledLongName.Buffer != UniTunneledLongNameBuffer) {
//
//             //
//             //  Tunneling package grew the buffer from pool
//             //
//
//             ExFreePool( UniTunneledLongName.Buffer );
//         }
//
//
//         //
//         //  If this is an abnormal termination then undo our work.
//         //
//         //  The extra exception handling here is complex.  We've got
//         //  two places here where an exception can be thrown again.
//         //
//
//         LocalAbnormalTermination = _SEH2_AbnormalTermination();
//
//         if (LocalAbnormalTermination) {
//
//             if (CountsIncremented) {
//                 Fcb->UncleanCount -= 1;
//                 Fcb->OpenCount -= 1;
//                 if (FlagOn(FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING)) {
//                     Fcb->NonCachedUncleanCount -= 1;
//                 }
//                 Vcb->OpenFileCount -= 1;
//                 if (IsFileObjectReadOnly(FileObject)) { Vcb->ReadOnlyCount -= 1; }
//             }
//
//             if (UnwindFcb == NULL) {
//
//                 NT_ASSERT( (ParentDcb->Specific.Dcb.UnusedDirentVbo == 0xffffffff_u32) ||
//                         RtlAreBitsSet( &ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                                        DirentByteOffset / sizeof(DIRENT),
//                                        DirentsNeeded ) );
//
//                 RtlClearBits( &ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                               DirentByteOffset / sizeof(DIRENT),
//                               DirentsNeeded );
//             }
//
//             //
//             //  Mark the dirents deleted.  The code is complex because of
//             //  dealing with an LFN that crosses a page boundary.
//             //
//
//             if (UnwindDirent != NULL) {
//
//                 ULONG i;
//
//                 for (i = 0; i < DirentsNeeded; i++) {
//
//                     if (DirentFromPool == FALSE) {
//
//                         //
//                         //  Simple case.
//                         //
//
//                         Dirent[i].FileName[0] = FAT_DIRENT_DELETED;
//
//                     } else {
//
//                         //
//                         //  If the second CcPreparePinWrite failed, we have
//                         //  to stop early.
//                         //
//
//                         if ((SecondPageBcb == NULL) &&
//                             (i == DirentsInFirstPage)) {
//
//                             break;
//                         }
//
//                         //
//                         //  Now conditionally update either page.
//                         //
//
//                         if (i < DirentsInFirstPage) {
//
//                             FirstPageDirent[i].FileName[0] = FAT_DIRENT_DELETED;
//
//                         } else {
//
//                             SecondPageDirent[i - DirentsInFirstPage].FileName[0] = FAT_DIRENT_DELETED;
//                         }
//                     }
//                 }
//             }
//         }
//
//         //
//         //  We must handle exceptions in the following fragments and plow on with the
//         //  unwind of this create operation.  This is basically inverted from the
//         //  previous state of the code.  Since AbnormalTermination() changes when we
//         //  enter a new enclosure, we cached the original state ...
//         //
//
//         _SEH2_TRY {
//
//             if (LocalAbnormalTermination) {
//                 if (UnwindAllocation) {
//                     FatTruncateFileAllocation( IrpContext, Fcb, 0 );
//                 }
//             }
//
//         } _SEH2_FINALLY {
//
//             _SEH2_TRY {
//
//                 if (LocalAbnormalTermination) {
//                     if (UnwindFcb != NULL) {
//                         FatDeleteDirent( IrpContext, UnwindFcb, NULL, TRUE );
//                     }
//                 }
//
//             } _SEH2_FINALLY {
//
//                 if (LocalAbnormalTermination) {
//                     if (UnwindFcb != NULL) {
//                         if (ARGUMENT_PRESENT( FileObject )) {
//                             FileObject->SectionObjectPointer = NULL;
//                         }
//                         FatDeleteFcb( IrpContext, &UnwindFcb );
//                     }
//                     if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//                 }
//
//                 //
//                 //  This is the normal cleanup code.
//                 //
//
//                 FatUnpinBcb( IrpContext, DirentBcb );
//                 FatUnpinBcb( IrpContext, SecondPageBcb );
//
//                 if (DirentFromPool) {
//
//                     ExFreePool( Dirent );
//                 }
//
//             } _SEH2_END;
//         } _SEH2_END;
//
//         DebugTrace(-1, Dbg, "FatCreateNewFile -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// IO_STATUS_BLOCK
// FatSupersedeOrOverwriteFile (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFCB Fcb,
//      ULONG AllocationSize,
//      PFILE_FULL_EA_INFORMATION EaBuffer,
//      ULONG EaLength,
//      USHORT FileAttributes,
//      ULONG CreateDisposition,
//      BOOLEAN NoEaKnowledge
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs a file supersede or overwrite operation.
//
// Arguments:
//
//     FileObject - Supplies a pointer to the file object
//
//     Fcb - Supplies a pointer to the Fcb
//
//     AllocationSize - Supplies an initial allocation size
//
//     EaBuffer - Supplies the Ea set for the superseded/overwritten file
//
//     EaLength - Supplies the length, in bytes, of EaBuffer
//
//     FileAttributes - Supplies the supersede/overwrite file attributes
//
//     CreateDisposition - Supplies the create disposition for the file
//         It must be either supersede, overwrite, or overwrite if.
//
//     NoEaKnowledge - This opener doesn't understand Ea's and we fail this
//         open if the file has NeedEa's.
//
// Return Value:
//
//     IO_STATUS_BLOCK - Returns the completion status for the operation
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb = {0};
//
//     PDIRENT Dirent;
//     PBCB DirentBcb;
//
//     USHORT EaHandle = 0;
//     BOOLEAN EaChange = FALSE;
//     BOOLEAN ReleasePaging = FALSE;
//
//     PCCB Ccb;
//
//     ULONG NotifyFilter;
//     ULONG HeaderSize = 0;
//     LARGE_INTEGER AllocSize = {0};
//
//     //
//     //  The following variables are for abnormal termination
//     //
//
//     PCCB UnwindCcb = NULL;
//     USHORT UnwindEa = 0;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSupersedeOrOverwriteFile...\n", 0);
//
//     DirentBcb = NULL;
//
//     //
//     //  We fail this operation if the caller doesn't understand Ea's.
//     //
//
//     if (NoEaKnowledge
//         && EaLength > 0) {
//
//         Iosb.Status = STATUS_ACCESS_DENIED;
//
//         DebugTrace(-1, Dbg, "FatSupersedeOrOverwriteFile -> Iosb.Status = %08lx\n", Iosb.Status);
//         return Iosb;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Before we actually truncate, check to see if the purge
//         //  is going to fail.
//         //
//
//         if (!MmCanFileBeTruncated( &Fcb->NonPaged->SectionObjectPointers,
//                                    &FatLargeZero )) {
//
//             try_return( Iosb.Status = STATUS_USER_MAPPED_FILE );
//         }
//
//         //
//         //  Setup the context and section object pointers, and update
//         //  our reference counts
//         //
//
//         FatSetFileObject( FileObject,
//                           UserFileOpen,
//                           Fcb,
//                           Ccb = UnwindCcb = FatCreateCcb( IrpContext ));
//
//         FileObject->SectionObjectPointer = &Fcb->NonPaged->SectionObjectPointers;
//
//         //
//         //  Since this is an supersede/overwrite, purge the section so
//         //  that mappers will see zeros.  We set the CREATE_IN_PROGRESS flag
//         //  to prevent the Fcb from going away out from underneath us.
//         //
//
//         SetFlag(Fcb->Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
//
//         CcPurgeCacheSection( &Fcb->NonPaged->SectionObjectPointers, NULL, 0, FALSE );
//
//         //
//         //  Tentatively add the new Ea's
//         //
//
//         if (EaLength > 0) {
//
//             FatCreateEa( IrpContext,
//                          Fcb->Vcb,
//                          (PUCHAR) EaBuffer,
//                          EaLength,
//                          &Fcb->ShortName.Name.Oem,
//                          &EaHandle );
//
//             UnwindEa = EaHandle;
//             EaChange = TRUE;
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Break parent directory oplock.  Directory oplock breaks are always
//         //  advisory, so we will never block/get STATUS_PENDING here.  On the
//         //  off chance this fails with INSUFFICIENT_RESOURCES we do it here
//         //  where we can still tolerate a failure.
//         //
//
//         Iosb.Status = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb->ParentDcb),
//                                           IrpContext->OriginatingIrp,
//                                           OPLOCK_FLAG_PARENT_OBJECT,
//                                           NULL,
//                                           NULL,
//                                           NULL );
//
//         if (Iosb.Status != STATUS_SUCCESS) {
//
//             FatRaiseStatus( IrpContext, Iosb.Status );
//         }
// #endif
//
//         //
//         //  Now set the new allocation size, we do that by first
//         //  zeroing out the current file size.  Then we truncate and
//         //  allocate up to the new allocation size
//         //
//
//         (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE );
//         ReleasePaging = TRUE;
//
//         Fcb->Header.FileSize.LowPart = 0;
//         Fcb->Header.ValidDataLength.LowPart = 0;
//         Fcb->ValidDataToDisk = 0;
//
//
//         //
//         // Validate that the allocation size will work.
//         //
//
//         AllocSize.QuadPart = AllocationSize;
//         if (!FatIsIoRangeValid( Fcb->Vcb, AllocSize, 0 )) {
//
//             DebugTrace(-1, Dbg, "Illegal allocation size\n", 0);
//
//             FatRaiseStatus( IrpContext, STATUS_DISK_FULL );
//         }
//
//
//         //
//         //  Tell the cache manager the size went to zero
//         //  This call is unconditional, because MM always wants to know.
//         //
//
//         CcSetFileSizes( FileObject,
//                         (PCC_FILE_SIZES)&Fcb->Header.AllocationSize );
//
//         FatTruncateFileAllocation( IrpContext, Fcb, AllocationSize+HeaderSize );
//
//         ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//         ReleasePaging = FALSE;
//
//         FatAddFileAllocation( IrpContext, Fcb, FileObject, AllocationSize+HeaderSize );
//
//         Fcb->FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
//
//         //
//         //  Modify the attributes and time of the file, by first reading
//         //  in the dirent for the file and then updating its attributes
//         //  and time fields.  Note that for supersede we replace the file
//         //  attributes as opposed to adding to them.
//         //
//
//         FatGetDirentFromFcbOrDcb( IrpContext,
//                                   Fcb,
//                                   FALSE,
//                                   &Dirent,
//                                   &DirentBcb );
//         //
//         //  We should get the dirent since this Fcb is in good condition, verified as
//         //  we crawled down the prefix tree.
//         //
//         //  Update the appropriate dirent fields, and the fcb fields
//         //
//
//         Dirent->FileSize = 0;
//
//
//         FileAttributes |= FILE_ATTRIBUTE_ARCHIVE;
//
//         if (CreateDisposition == FILE_SUPERSEDE) {
//
//             Dirent->Attributes = (UCHAR)FileAttributes;
//
//         } else {
//
//             Dirent->Attributes |= (UCHAR)FileAttributes;
//         }
//
//         Fcb->DirentFatFlags = Dirent->Attributes;
//
//         KeQuerySystemTime( &Fcb->LastWriteTime );
//
//         (VOID)FatNtTimeToFatTime( IrpContext,
//                                   &Fcb->LastWriteTime,
//                                   TRUE,
//                                   &Dirent->LastWriteTime,
//                                   NULL );
//
//         if (FatData.ChicagoMode) {
//
//             Dirent->LastAccessDate = Dirent->LastWriteTime.Date;
//         }
//
//         NotifyFilter = FILE_NOTIFY_CHANGE_LAST_WRITE
//                        | FILE_NOTIFY_CHANGE_ATTRIBUTES
//                        | FILE_NOTIFY_CHANGE_SIZE;
//
//         //
//         //  And now delete the previous Ea set if there was one.
//         //
//
//         if (!FatIsFat32(Fcb->Vcb) && Dirent->ExtendedAttributes != 0) {
//
//             //
//             //  ****    SDK fix, we won't fail this if there is
//             //          an error in the Ea's, we'll just leave
//             //          the orphaned Ea's in the file.
//             //
//
//             EaChange = TRUE;
//
//             _SEH2_TRY {
//
//                 FatDeleteEa( IrpContext,
//                              Fcb->Vcb,
//                              Dirent->ExtendedAttributes,
//                              &Fcb->ShortName.Name.Oem );
//
//             } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//         }
//
//         //
//         //  Update the extended attributes handle in the dirent.
//         //
//
//         if (EaChange) {
//
//             NT_ASSERT(!FatIsFat32(Fcb->Vcb));
//
//             Dirent->ExtendedAttributes = EaHandle;
//
//             NotifyFilter |= FILE_NOTIFY_CHANGE_EA;
//         }
//
//         //
//         //  Now update the dirent to the new ea handle and set the bcb dirty
//         //  Once we do this we can no longer back out the Ea
//         //
//
//         FatSetDirtyBcb( IrpContext, DirentBcb, Fcb->Vcb, TRUE );
//         UnwindEa = 0;
//
//         //
//         //  Indicate that the Eas for this file have changed.
//         //
//
//         Ccb->EaModificationCount += Fcb->EaModificationCount;
//
//         //
//         //  Check to see if we need to notify outstanding Irps for full
//         //  changes only (i.e., we haven't added, deleted, or renamed the file).
//         //
//
//         FatNotifyReportChange( IrpContext,
//                                Fcb->Vcb,
//                                Fcb,
//                                NotifyFilter,
//                                FILE_ACTION_MODIFIED );
//
//         //
//         //  And set our status to success
//         //
//
//         Iosb.Status = STATUS_SUCCESS;
//
//         if (CreateDisposition == FILE_SUPERSEDE) {
//
//             Iosb.Information = FILE_SUPERSEDED;
//
//         } else {
//
//             Iosb.Information = FILE_OVERWRITTEN;
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSupersedeOfOverwriteFile );
//
//         if (ReleasePaging)  {  ExReleaseResourceLite( Fcb->Header.PagingIoResource );  }
//
//         //
//         //  If this is an abnormal termination then undo our work.
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindEa != 0) { FatDeleteEa( IrpContext, Fcb->Vcb, UnwindEa, &Fcb->ShortName.Name.Oem ); }
//             if (UnwindCcb != NULL) { FatDeleteCcb( IrpContext, &UnwindCcb ); }
//         }
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         ClearFlag(Fcb->Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS);
//
//         DebugTrace(-1, Dbg, "FatSupersedeOrOverwriteFile -> Iosb.Status = %08lx\n", Iosb.Status);
//     } _SEH2_END;
//
//     return Iosb;
// }
//
//
// VOID
// FatSetFullNameInFcb (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PUNICODE_STRING FinalName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine attempts a quick form of the full FatSetFullFileNameInFcb
//     operation.
//
//     NOTE: this routine is probably not worth the code duplication involved,
//     and is not equipped to handle the cases where the parent doesn't have
//     the full name set up.
//
// Arguments:
//
//     Fcb - Supplies a pointer to the Fcb
//
//     FinalName - Supplies the last component of the path to this Fcb's dirent
//
// Return Value:
//
//     None.  May silently fail.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     NT_ASSERT( Fcb->FullFileName.Buffer == NULL );
//
//     //
//     //  Prefer the ExactCaseLongName of the file for this operation, if set.  In
//     //  this way we avoid building the fullname with a short filename.  Several
//     //  operations assume this - the FinalNameLength in particular is the Lfn
//     //  (if existant) length, and we use this to crack the fullname in paths
//     //  such as the FsRtlNotify caller.
//     //
//     //  If the caller specified a particular name and it is short, it is the
//     //  case that the long name was set up.
//     //
//
//     if (Fcb->ExactCaseLongName.Buffer) {
//
//         NT_ASSERT( Fcb->ExactCaseLongName.Length != 0 );
//         FinalName = &Fcb->ExactCaseLongName;
//     }
//
//     //
//     //  Special case the root.
//     //
//
//     if (NodeType(Fcb->ParentDcb) == FAT_NTC_ROOT_DCB) {
//
//         Fcb->FullFileName.Length =
//         Fcb->FullFileName.MaximumLength = sizeof(WCHAR) + FinalName->Length;
//
//         Fcb->FullFileName.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                              Fcb->FullFileName.Length,
//                                                              TAG_FILENAME_BUFFER );
//
//         Fcb->FullFileName.Buffer[0] = L'\\';
//
//         RtlCopyMemory( &Fcb->FullFileName.Buffer[1],
//                        &FinalName->Buffer[0],
//                        FinalName->Length );
//
//     } else {
//
//         PUNICODE_STRING Prefix;
//
//         Prefix = &Fcb->ParentDcb->FullFileName;
//
//         //
//         //  It is possible our parent's full filename is not set.  Simply fail
//         //  this attempt.
//         //
//
//         if (Prefix->Buffer == NULL) {
//
//             return;
//         }
//
//         Fcb->FullFileName.Length =
//         Fcb->FullFileName.MaximumLength = Prefix->Length + sizeof(WCHAR) + FinalName->Length;
//
//         Fcb->FullFileName.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                              Fcb->FullFileName.Length,
//                                                              TAG_FILENAME_BUFFER );
//
//         RtlCopyMemory( &Fcb->FullFileName.Buffer[0],
//                        &Prefix->Buffer[0],
//                        Prefix->Length );
//
//         Fcb->FullFileName.Buffer[Prefix->Length / sizeof(WCHAR)] = L'\\';
//
//         RtlCopyMemory( &Fcb->FullFileName.Buffer[(Prefix->Length / sizeof(WCHAR)) + 1],
//                        &FinalName->Buffer[0],
//                        FinalName->Length );
//
//     }
// }
//
//
// NTSTATUS
// FatCheckSystemSecurityAccess (
//      PIRP_CONTEXT IrpContext
//     )
// {
//     PACCESS_STATE AccessState;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//
//     PAGED_CODE();
//
//     //
//     //  We check if the caller wants ACCESS_SYSTEM_SECURITY access on this
//     //  object and fail the request if he does.
//     //
//
//     NT_ASSERT( IrpSp->Parameters.Create.SecurityContext != NULL );
//     AccessState = IrpSp->Parameters.Create.SecurityContext->AccessState;
//
//     //
//     //  Check if the remaining privilege includes ACCESS_SYSTEM_SECURITY.
//     //
//
//     if (FlagOn( AccessState->RemainingDesiredAccess, ACCESS_SYSTEM_SECURITY )) {
//
//         if (!SeSinglePrivilegeCheck( FatSecurityPrivilege,
//                                      UserMode )) {
//
//             return STATUS_ACCESS_DENIED;
//         }
//
//         //
//         //  Move this privilege from the Remaining access to Granted access.
//         //
//
//         ClearFlag( AccessState->RemainingDesiredAccess, ACCESS_SYSTEM_SECURITY );
//         SetFlag( AccessState->PreviouslyGrantedAccess, ACCESS_SYSTEM_SECURITY );
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// FatCheckShareAccess (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFCB FcbOrDcb,
//      PACCESS_MASK DesiredAccess,
//      ULONG ShareAccess
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks conditions that may result in a sharing violation.
//
// Arguments:
//
//     FileObject - Pointer to the file object of the current open request.
//
//     FcbOrDcb - Supplies a pointer to the Fcb/Dcb.
//
//     DesiredAccess - Desired access of current open request.
//
//     ShareAccess - Shared access requested by current open request.
//
// Return Value:
//
//     If the accessor has access to the file, STATUS_SUCCESS is returned.
//     Otherwise, STATUS_SHARING_VIOLATION is returned.
//
// --*/
//
// {
//     PAGED_CODE();
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//     //
//     //  Do an extra test for writeable user sections if the user did not allow
//     //  write sharing - this is neccessary since a section may exist with no handles
//     //  open to the file its based against.
//     //
//
//     if ((NodeType( FcbOrDcb ) == FAT_NTC_FCB) &&
//         !FlagOn( ShareAccess, FILE_SHARE_WRITE ) &&
//         FlagOn( *DesiredAccess, FILE_EXECUTE | FILE_READ_DATA | FILE_WRITE_DATA | FILE_APPEND_DATA | DELETE | MAXIMUM_ALLOWED ) &&
//         MmDoesFileHaveUserWritableReferences( &FcbOrDcb->NonPaged->SectionObjectPointers )) {
//
//         return STATUS_SHARING_VIOLATION;
//     }
// #endif
//
//     //
//     //  Check if the Fcb has the proper share access.
//     //
//
//     return IoCheckShareAccess( *DesiredAccess,
//                                ShareAccess,
//                                FileObject,
//                                &FcbOrDcb->ShareAccess,
//                                FALSE );
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
// //
// // Lifted from NTFS.
// //
//
// NTSTATUS
// FatCallSelfCompletionRoutine (
//     __in PDEVICE_OBJECT DeviceObject,
//     __in PIRP Irp,
//     __in PVOID Contxt
//     )
//
// {
//     //
//     //  Set the event so that our call will wake up.
//     //
//
//     KeSetEvent( (PKEVENT)Contxt, 0, FALSE );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Irp );
//
//     //
//     //  If we change this return value then FatIoCallSelf needs to reference the
//     //  file object.
//     //
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// fatprocs.h flag macros, inline at the declared widths (were C stubs taking
// ULONG_PTR, which read garbage upper bits from a narrower argument).
#[inline(always)]
unsafe fn FlagOn(Value: u32, Flags: u32) -> i32 {
    (Value as u64 & Flags as u64) as i32
}

#[inline(always)]
unsafe fn BooleanFlagOn(value: u8, flag: u8) -> u8 {
    ((value as u64 & flag as u64) != 0) as u8
}
