use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcPurgeCacheSection, CcSetFileSizes, CcUninitializeCacheMap, ExAcquireResourceExclusiveLite, ExFreePool, ExLocalTimeToSystemTime, ExReleaseResourceLite, ExSystemTimeToLocalTime, FILE_ACCESS_INFORMATION, FILE_ALIGNMENT_INFORMATION, FILE_BASIC_INFORMATION, FILE_EA_INFORMATION, FILE_INTERNAL_INFORMATION, FILE_MODE_INFORMATION, FILE_NAME_INFORMATION, FILE_NETWORK_OPEN_INFORMATION, FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, _IO_STATUS_BLOCK, IoSetTopLevelIrp, LARGE_INTEGER, MmCanFileBeTruncated, OEM_STRING, PCC_FILE_SIZES, PDEVICE_OBJECT, PERESOURCE, PFILE_ALLOCATION_INFORMATION, PFILE_ALL_INFORMATION, PFILE_BASIC_INFORMATION, PFILE_DISPOSITION_INFORMATION, PFILE_EA_INFORMATION, PFILE_END_OF_FILE_INFORMATION, PFILE_INTERNAL_INFORMATION, PFILE_NAME_INFORMATION, PFILE_NETWORK_OPEN_INFORMATION, PFILE_OBJECT, PFILE_POSITION_INFORMATION, PFILE_RENAME_INFORMATION, PFILE_STANDARD_INFORMATION, PFILE_VALID_DATA_LENGTH_INFORMATION, PIO_STACK_LOCATION, PIRP, POEM_STRING, PUNICODE_STRING, RtlCopyUnicodeString, RtlOemStringToCountedUnicodeString, RtlUpcaseUnicodeString};
const MmFlushForDelete: ntkrnl_sys::ddk::_MMFLUSH_TYPE = ntkrnl_sys::ddk::_MMFLUSH_TYPE_MmFlushForDelete;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{BIOS_PARAMETER_BLOCK, CCB, DCB, DIRENT, EA_RANGE, FAT_BUG_CHECK_FILEINFO, FAT_DATA, FAT_DATE, FAT_TIME_STAMP, FCB, IRP_CONTEXT, NODE_TYPE_CODE, PBCB, PCCB, PDCB, PFCB, PIRP_CONTEXT, PVCB, VCB, _TYPE_OF_OPEN as TYPE_OF_OPEN, _TYPE_OF_OPEN_DirectoryFile as DirectoryFileOpen, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _FCB__bindgen_ty_1__bindgen_ty_1, _CCB__bindgen_ty_1__bindgen_ty_1};
// FILE_INFORMATION_CLASS variant constants (not in ntkrnl-sys bindgen, defined locally)
const FileAllInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 18;
const FileAllocationInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 19;
const FileAlternateNameInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 21;
const FileBasicInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 4;
const FileDispositionInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 13;
const FileEaInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 7;
const FileEndOfFileInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 20;
const FileInternalInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 6;
const FileLinkInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 11;
const FileNameInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 9;
const FileNetworkOpenInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 34;
const FilePositionInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 14;
const FileRenameInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 10;
const FileStandardInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 5;
const FileValidDataLengthInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 39;
const FileNormalizedNameInformation: ntkrnl_sys::ddk::FILE_INFORMATION_CLASS = 48;
const DPFLTR_FASTFAT_ID: u32 = 33; // _DPFLTR_TYPE_DPFLTR_FASTFAT_ID = 33
// FILE_ATTRIBUTE constants (not re-exported from glue for all variants)
const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x00000100;
// STATUS_CANT_WAIT
const STATUS_CANT_WAIT: ntkrnl_sys::base::NTSTATUS = 0xC00000D8_u32 as i32;
use ntkrnl_sys::ddk::{CACHE_MANAGER_CALLBACKS, CC_FILE_SIZES, EXCEPTION_POINTERS, FILE_ALL_INFORMATION, FILE_DISPOSITION_INFORMATION, FILE_INFORMATION_CLASS, FILE_OBJECT, IO_STACK_LOCATION, IRP, LARGE_MCB, OPLOCK, SECTION_OBJECT_POINTERS, _PUBLIC_BCB as BCB, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_FILEINFO, DPFLTR_INFO_LEVEL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::dirsup::{FatCreateNewDirent};
use crate::glue::{FILE_ATTRIBUTE_ARCHIVE, STATUS_DISK_FULL};
use crate::timesup::{FatFatDateToNtTime, FatFatTimeToNtTime};
use crate::types::{EA_SET_HEADER, PDIRENT};
use ntkrnl_sys::ddk::{FILE_ALLOCATION_INFORMATION, FILE_RENAME_INFORMATION, FILE_VALID_DATA_LENGTH_INFORMATION, IO_STATUS_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlAreNamesEqual, FsRtlCheckOplock, FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlFindInTunnelCache, FsRtlNotifyFullChangeDirectory};

extern "system" {
    fn FatSetFullFileNameInFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    } extern "C" { fn DebugTrace(level: i32, component: u32, format: *const i8, ...); } extern "system" {
    static mut FatJanOne1980: LARGE_INTEGER;
    fn FlagOn(flags: u32, flag: u32) -> i32;
    fn SetFlag(flags: *mut u32, flag: u32);
    fn NodeType(fcb: *mut FCB) -> u16;
    fn FatLookupFileAllocationSize(IrpContext: *mut IRP_CONTEXT, fcb: *mut FCB);
    fn PagedCode();
    static FatDecThirtyOne1979: LARGE_INTEGER;
    static FatTimeJanOne1980: FAT_TIME_STAMP;
    static FatOneDay: LARGE_INTEGER;
    static FatData: FAT_DATA;
    fn FatGetEaFile( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, EaDirent: *mut *mut DIRENT, EaBcb: *mut *mut BCB, Flag1: i32, Flag2: i32, );
    fn FatReadEaSet( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, ExtendedAttributes: u16, OldOemName: *mut OEM_STRING, Flag: i32, EaSetRange: *mut EA_RANGE, );
    fn FatMarkEaRangeDirty( IrpContext: *mut IRP_CONTEXT, VirtualEaFile: *mut FILE_OBJECT, EaSetRange: *mut EA_RANGE, );
    fn FatUnpinEaRange( IrpContext: *mut IRP_CONTEXT, EaSetRange: *mut EA_RANGE, );
    fn FatUnpinBcb( IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
    fn FatReleaseFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, );
    fn CcFlushCache( SectionObjectPointer: *mut core::ffi::c_void, FileOffset: *const LARGE_INTEGER, Length: u32, IoStatus: *mut _IO_STATUS_BLOCK, );
    fn FatExceptionFilter( IrpContext: *mut IRP_CONTEXT, ExceptionInformation: *mut EXCEPTION_POINTERS, ) -> i32;
    fn DebugUnwind(name: *const core::ffi::c_void);
    fn FatDecodeFileObject( FileObject: *mut FILE_OBJECT, Vcb: *mut *mut VCB, FcbOrDcb: *mut *mut FCB, Ccb: *mut *mut CCB, ) -> TYPE_OF_OPEN;
    fn FatUnicodeToUpcaseOem( IrpContext: *mut IRP_CONTEXT, OemName: *mut OEM_STRING, UnicodeName: *const _UNICODE_STRING, );
    fn FatIsNameShortOemValid( IrpContext: *mut IRP_CONTEXT, OemName: OEM_STRING, AllowQuestionMarks: BOOLEAN, AllowWildCards: BOOLEAN, IgnoreLeadingByte: BOOLEAN, ) -> BOOLEAN;
    fn FatSpaceInName(IrpContext: *mut IRP_CONTEXT, Name: *const _UNICODE_STRING) -> BOOLEAN;
    fn FatLfnDirentExists( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, LfnName: *const _UNICODE_STRING, ExistingLfn: *mut _UNICODE_STRING, ) -> BOOLEAN;
    fn FatPurgeReferencedFileObjects(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Flags: u32);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> BOOLEAN;
    fn FatNotifyReportChange( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Fcb: *mut FCB, NotifyFilter: u32, NotifyAction: u32, );
    fn FatGetDirentFromFcbOrDcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Flag: BOOLEAN, OldDirent: *mut *mut DIRENT, OldDirentBcb: *mut *mut PBCB, );
    fn FatTunnelFcbOrDcb(Fcb: *mut FCB, Ccb: *mut CCB);
    fn FatDeleteDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Lfn: *mut _UNICODE_STRING, Flag: BOOLEAN, );
    fn FatSelectNames( IrpContext: *mut IRP_CONTEXT, TargetDcb: *mut DCB, ShortName: *mut OEM_STRING, LongName: *const _UNICODE_STRING, FinalShortName: *mut OEM_STRING, TunneledShortName: *const _UNICODE_STRING, AllLowerComponent: *mut BOOLEAN, AllLowerExtension: *mut BOOLEAN, CreateLfn: *mut BOOLEAN, );
    fn FatPrepareWriteDirectoryFile( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, Offset: u32, Length: u32, Bcb: *mut *mut PBCB, Dirent: *mut *mut DIRENT, WriteThrough: BOOLEAN, Wait: BOOLEAN, IoStatus: *mut NTSTATUS, ) -> NTSTATUS;
    fn FatConstructDirent( IrpContext: *mut IRP_CONTEXT, ShortDirent: *mut DIRENT, OemName: *const OEM_STRING, AllLowerComponent: BOOLEAN, AllLowerExtension: BOOLEAN, LongName: *const _UNICODE_STRING, Attributes: u16, Flag: BOOLEAN, CreationTime: *const LARGE_INTEGER, );
    fn FatSetFullNameInFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Name: *const _UNICODE_STRING, );
    fn FatConstructNamesInFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Dirent: *const DIRENT, LongName: *const _UNICODE_STRING, );
    fn FatRemoveNames(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatUnpinRepinnedBcbs(IrpContext: *mut IRP_CONTEXT);
    fn FatGetUnicodeNameFromFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Name: *mut _UNICODE_STRING, );
    fn FatOplockComplete(Context: *mut OPLOCK, Irp: *mut IRP);
    fn FsRtlCheckOplockEx( Oplock: *mut OPLOCK, Irp: *mut IRP, Flags: u32, CompletionRoutine: *mut c_void, PostIrpRoutine: *mut c_void, Context: *mut c_void, ) -> NTSTATUS;
    fn FatIsFileOplockable(Fcb: *mut FCB) -> BOOLEAN;
    fn FsRtlCurrentBatchOplock(Oplock: *mut OPLOCK) -> BOOLEAN;
    fn FsRtlCurrentOplockH(Oplock: *mut OPLOCK) -> BOOLEAN;
    fn FatGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
    fn FatGetNextFcbBottomUp( IrpContext: *mut IRP_CONTEXT, CurrentFcb: *mut FCB, RootFcb: *mut FCB, ) -> *mut FCB;
    fn FatIsFat32(Vcb: *mut VCB) -> BOOLEAN;
    fn FatDirectoryKey(Dcb: *mut DCB) -> u64;
    // Matches namesup.rs:96 and the C (fileinfo.c:3295):
    //   FatStringTo8dot3( IrpContext, NewOemName, &LocalCcb.OemQueryTemplate.Constant )
    // The OEM_STRING goes by VALUE, and the output is the 11-byte 8.3 constant,
    // not an OEM_STRING. Declaring the input by pointer made this @12 against a
    // definition exporting @16 — an i386 link error, and on amd64/arm64 a silent
    // pointer-where-a-struct-belongs.
    fn FatStringTo8dot3( IrpContext: *mut IRP_CONTEXT, InputString: OEM_STRING, Output8dot3: *mut [u8; 11], );
    // VOID upstream and in namesup.rs — the answer comes back through CreateLfn.
    fn FatEvaluateNameCase( IrpContext: *mut IRP_CONTEXT, Name: *const _UNICODE_STRING, AllLowerComponent: *mut BOOLEAN, AllLowerExtension: *mut BOOLEAN, CreateLfn: *mut BOOLEAN, );
    fn FAT_LFN_DIRENTS_NEEDED(Name: *const _UNICODE_STRING) -> u32;
    fn FatLocateDirent( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, Ccb: *mut CCB, WildCards: u32, Flags: *mut u32, TargetDirent: *mut *mut DIRENT, TargetDirentBcb: *mut *mut PBCB, TargetDirentOffset: *mut u32, FileNameOpenedDos: *mut BOOLEAN, TargetLfn: *mut _UNICODE_STRING, TargetOrigLfn: *mut _UNICODE_STRING, );
    static TAG_FILENAME_BUFFER: u32;
    static TAG_DIRENT: u32;
    static CCB_FLAG_SKIP_SHORT_NAME_COMPARE: u32;
    static FO_FILE_MODIFIED: u32;
    static CCB_FLAG_USER_SET_LAST_WRITE: u32;
    static FILE_ACTION_RENAMED_OLD_NAME: u32;
    static FILE_ACTION_REMOVED: u32;
    static FILE_ACTION_MODIFIED: u32;
    static FILE_ACTION_ADDED: u32;
    static FILE_ACTION_RENAMED_NEW_NAME: u32;
    static FAT_DIRENT_ATTR_DIRECTORY: u8;
    static FAT_DIRENT_ATTR_READ_ONLY: u8;
    // FcbBad: see the const below (was an extern static no library defines)
    static USER_DIRECTORY_OPEN: u32; // for FatDecodeFileObject return static FAT_NTC_ROOT_DCB: u16;
    static FAT_NTC_DCB: u16;
    static FAT_NTC_FCB: u16;
    static OPLOCK_FLAG_PARENT_OBJECT: u32;
    static PAGED_CODE: ();
    static FCB_STATE_PAGING_FILE: u32;
    static VCB_STATE_FLAG_CREATE_IN_PROGRESS: u32;
    static VCB_STATE_FLAG_REMOVABLE_MEDIA: u32;
    static VCB_STATE_FLAG_DEFERRED_FLUSH: u32;
    static IRP_CONTEXT_FLAG_WAIT: u32;
    static FAT_NTC_ROOT_DCB: NODE_TYPE_CODE;
    static UserVolumeOpen: TYPE_OF_OPEN;
    fn CanFsdWait(Irp: *mut core::ffi::c_void) -> i32;
    fn FatIsIrpTopLevel(Irp: *mut core::ffi::c_void) -> u8;
    fn FatCreateIrpContext(Irp: *mut core::ffi::c_void, CanWait: i32) -> *mut core::ffi::c_void;
    fn FatProcessException(IrpContext: *mut core::ffi::c_void, Irp: *mut core::ffi::c_void, ExceptionCode: u32) -> i32;
}
extern "system" {
    fn KeGetCurrentIrql() -> u8;
    fn UNREFERENCED_PARAMETER(param: *mut ());
    fn NT_ASSERT(condition: i32);
    fn ClearFlag(flags: *mut u32, flag: u32);
    fn FatNtTimeToFatTime( irp_ctx: *mut IRP_CONTEXT, nt_time: *const LARGE_INTEGER, date_only: u8, fat_time: *mut FAT_TIME_STAMP, fat_ms: *mut u8, ) -> u8;
    fn FatSetDirtyBcb( irp_ctx: *mut IRP_CONTEXT, bcb: *mut BCB, vcb: *mut VCB, flag: u8, );
    fn FatReservedBytes(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatBytesPerFat(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatReadVolumeFile( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, ByteOffset: u32, BytesToRead: u32, BcbPtr: *mut PBCB, Buffer: *mut *mut u8, ); // VBO = u32 (was u64: i386 stdcall stack imbalance)
    // Matches the definition in fsctrl.rs: `Lbo` is LBO = LONGLONG (fat.h:34,
    // "for Fat32, LBO is >32 bits"). This decl still said i32 after that widening,
    // which on i386 is a LINK error (stdcall bakes the arg size into the name:
    // wanted _FatAddMcbEntry@20, definition exports @24) and on amd64/arm64 is
    // WORSE — it links clean and silently passes the wrong argument layout.
    fn FatAddMcbEntry( Vcb: *mut VCB, Mcb: *mut LARGE_MCB, Vbo: u32, Lbo: i64, SectorCount: u32, ) -> i32;
    fn FatNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatIsDirectoryEmpty(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8;
    fn CcPinMappedData( FileObject: PVOID, FileOffset: *const LARGE_INTEGER, Length: u32, Wait: i32, Bcb: *mut PBCB, ) -> u8;
    fn CcRepinBcb(Bcb: PBCB);
    fn CcSetDirtyPinnedData(Bcb: PBCB, Lsn: *const LARGE_INTEGER);
    fn CcUnpinData(Bcb: PBCB);
    fn CcUnpinRepinnedBcb(Bcb: PBCB, WriteThrough: i32, Iosb: *mut _IO_STATUS_BLOCK);
    fn MmFlushImageSection( SectionObjectPointers: *const SECTION_OBJECT_POINTERS, FlushOperation: u32, ) -> u8;
    // NT_SUCCESS is provided by ntkrnl_sys::base::*
    fn FatIsIoRangeValid(vcb: *mut VCB, size: LARGE_INTEGER, zero: i32) -> u8;
    fn FatInitializeCacheMap( FileObject: *mut FILE_OBJECT, sizes: *const CC_FILE_SIZES, flag: i32, callbacks: *const CACHE_MANAGER_CALLBACKS, fcb: *mut FCB, );
    fn FatAddFileAllocation( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, FileObject: *mut FILE_OBJECT, size: u32, );
    fn FatTruncateFileAllocation(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, size: u32);
    fn FatSetFileSizeInDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, null: *const c_void, );
    fn FatSetFileSizeInDirentNoRaise( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, null: *const c_void, );
    fn IsFileDeleted(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> BOOLEAN;
    fn FatBugCheck(a: u32, b: u32, c: u32);
    fn FatCreateFcb( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, TargetDcb: *mut DCB, LfnOffset: u32, DirentOffset: u32, Dirent: *mut DIRENT, Lfn: *mut _UNICODE_STRING, other: *mut _UNICODE_STRING, flag1: u8, flag2: u8, ) -> *mut FCB;
    fn FatDeleteFcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut *mut FCB, );
    fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    fn FatAcquireSharedFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB) -> BOOLEAN;
    fn FatFsdPostRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn FatVerifyFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    } extern "C" { fn KdPrintEx(ComponentId: u32, Level: u32, Format: *const i8, ...); }
extern "C" {
    // wdm FORCEINLINE helpers: cdecl bodies (`_Io...`, rust_cohab_forceinline.c), and the
    // rest of the crate declares them "C". One "system" view of the same symbol is a §9.99
    // fold: the call keeps stdcall's callee-pops bookkeeping against the cdecl thunk, esp
    // comes back 4 off, and the next esp-relative load is garbage (i386 0x50 saving a file).
    fn IoGetCurrentIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION;
} extern "system" {
    fn FatIsFastIoPossible(Fcb: *mut FCB) -> BOOLEAN;
}

/// ccfuncs.h `CcGetFileSizePointer(FO)` is a macro, not an export:
/// `((PLARGE_INTEGER)((FO)->SectionObjectPointer->SharedCacheMap) + 1)`.
#[inline(always)]
unsafe fn CcGetFileSizePointer(FileObject: *mut FILE_OBJECT) -> *mut LARGE_INTEGER {
    ((*(*FileObject).SectionObjectPointer).SharedCacheMap as *mut LARGE_INTEGER).add(1)
}

// ---- file-scope preamble (translated) ----
const FcbBad: u32 = crate::types::_FCB_CONDITION_FcbBad as u32;
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_FILEINFO;
pub const Dbg: u32 = DEBUG_TRACE_FILEINFO;

// ==== recovered from stitcher-drop: 01_FatQueryBasicInfo.rs ====
extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatQueryBasicInfo(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    FileObject: PFILE_OBJECT,
    Buffer: PFILE_BASIC_INFORMATION,
    Length: *mut i32,
) {
    const Dbg: u32 = 0; // DEBUG_TRACE_FILEINFO
    const FAT_NTC_ROOT_DCB: u16 = 0x0504; // placeholder – match actual node type
    const FCB_STATE_TEMPORARY: u32 = 0x20; // placeholder – actual value as defined
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;

    let _ = PAGED_CODE;

    // UNREFERENCED_PARAMETER(FileObject)
    let _ = FileObject;
    // UNREFERENCED_PARAMETER(IrpContext)
    let _ = IrpContext;

    DebugTrace(1, Dbg, b"FatQueryBasicInfo...\n\0".as_ptr() as *const i8, 0);

    RtlZeroMemory(Buffer as *mut _, core::mem::size_of::<FILE_BASIC_INFORMATION>());

    if (*Fcb).Header.NodeTypeCode == (FAT_NTC_ROOT_DCB) as i16 {
        ExLocalTimeToSystemTime(
            core::ptr::addr_of!(FatJanOne1980) as *mut LARGE_INTEGER,
            core::ptr::addr_of_mut!((*Buffer).LastWriteTime),
        );
        (*Buffer).CreationTime = (*Buffer).LastWriteTime;
        (*Buffer).LastAccessTime = (*Buffer).LastWriteTime;
    } else {
        (*Buffer).LastWriteTime = (*Fcb).LastWriteTime;
        (*Buffer).CreationTime = (*Fcb).CreationTime;
        (*Buffer).LastAccessTime = (*Fcb).LastAccessTime;
    }

    (*Buffer).FileAttributes = (*Fcb).DirentFatFlags as u32;

    if FlagOn(((*Fcb).FcbState as u8) as u32, FCB_STATE_TEMPORARY) != 0 {
        SetFlag(core::ptr::addr_of_mut!((*Buffer).FileAttributes), FILE_ATTRIBUTE_TEMPORARY);
    }

    if (*Buffer).FileAttributes == 0 {
        (*Buffer).FileAttributes = FILE_ATTRIBUTE_NORMAL;
    }

    *Length -= core::mem::size_of::<FILE_BASIC_INFORMATION>() as i32;

    DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);

    DebugTrace(-1, Dbg, b"FatQueryBasicInfo -> VOID\n\0".as_ptr() as *const i8, 0);
}


// ==== recovered from stitcher-drop: 02_FatQueryStandardInfo.rs ====
const FCB_STATE_DELETE_ON_CLOSE: u32 = 1; // placeholder
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;

extern "system" {
}

unsafe extern "system" fn FatQueryStandardInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Buffer: *mut FILE_STANDARD_INFORMATION,
    Length: *mut i32,
) {
    // PAGED_CODE expansion
    {
        if KeGetCurrentIrql() > 0 {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("ud2", options(nostack, nomem));
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("udf #0", options(nostack, nomem));
        }
    }

    DebugTrace(1, Dbg, b"FatQueryStandardInfo...\n\0".as_ptr() as *const i8, 0);

    core::ptr::write_bytes(Buffer as *mut u8, 0, core::mem::size_of::<FILE_STANDARD_INFORMATION>());

    (*Buffer).NumberOfLinks = 1;
    (*Buffer).DeletePending = ((*Fcb).FcbState & FCB_STATE_DELETE_ON_CLOSE) as u8;

    if (*Fcb).Header.NodeTypeCode == (FAT_NTC_FCB) as i16 {
        if (*Fcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
            FatLookupFileAllocationSize(IrpContext, Fcb);
        }
        (*Buffer).AllocationSize = (*Fcb).Header.AllocationSize;
        (*Buffer).EndOfFile = (*Fcb).Header.FileSize;
        (*Buffer).Directory = 0u8;
    } else {
        (*Buffer).Directory = 1u8;
    }

    *Length -= core::mem::size_of::<FILE_STANDARD_INFORMATION>() as i32;

    DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);

    DebugTrace(-1, Dbg, b"FatQueryStandardInfo -> VOID\n\0".as_ptr() as *const i8, 0);

    return;
}

// ==== recovered from stitcher-drop: 03_FatQueryInternalInfo.rs ====
extern "system" {
}


#[no_mangle]
pub unsafe extern "system" fn FatQueryInternalInfo(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    Buffer: PFILE_INTERNAL_INFORMATION,
    Length: PLONG,
) {
    let _ = IrpContext;

    DebugTrace(1, Dbg, b"FatQueryInternalInfo...\n\0".as_ptr() as *const i8, 0);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        (*Buffer).IndexNumber = LARGE_INTEGER {
            QuadPart: crate::fileid::fat_generate_file_id_from_fcb(Fcb),
        };
        *Length -= core::mem::size_of::<FILE_INTERNAL_INFORMATION>() as i32;
    }, |_, _abnormal| {
        DebugUnwind(FatQueryInternalInfo as *const core::ffi::c_void);
        DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);
        DebugTrace(-1, Dbg, b"FatQueryInternalInfo -> VOID\n\0".as_ptr() as *const i8, 0);
    });
}

// ==== recovered from stitcher-drop: 04_FatQueryEaInfo.rs ====
// Forward declarations for unknown functions
extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatQueryEaInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Buffer: *mut FILE_EA_INFORMATION,
    Length: *mut i32,
) {
    // PAGED_CODE(); // ignored

    let mut Bcb: *mut BCB = core::ptr::null_mut();

    // UNREFERENCED_PARAMETER(Fcb);
    let _ = Fcb;
    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    // DebugTrace(+1, Dbg, "FatQueryEaInfo...\n", 0);
    DebugTrace(1, 0u32 /* DEBUG_TRACE_FILEINFO */, b"FatQueryEaInfo...\n\0".as_ptr() as *const i8, 0);

    // _SEH2_TRY / _SEH2_FINALLY
    let p_bcb = core::ptr::addr_of_mut!(Bcb);
    crate::seh::try_finally(&mut (), |_| {
        // Zero out the output buffer
        core::ptr::write_bytes(Buffer as *mut u8, 0, core::mem::size_of::<FILE_EA_INFORMATION>());

        // Update the length
        *Length -= core::mem::size_of::<FILE_EA_INFORMATION>() as i32;
    }, |_, _abnormal| {
        DebugUnwind(FatQueryEaInfo as *const core::ffi::c_void);
        // Unpin the dirent if pinned.
        FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);
        DebugTrace(0, 0u32 /* DEBUG_TRACE_FILEINFO */, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);
        DebugTrace(-1, 0u32 /* DEBUG_TRACE_FILEINFO */, b"FatQueryEaInfo -> VOID\n\0".as_ptr() as *const i8, 0);
    });
}

// ==== recovered from stitcher-drop: 05_FatQueryPositionInfo.rs ====
extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatQueryPositionInfo(
    irp_context: *mut IRP_CONTEXT,
    file_object: *mut FILE_OBJECT,
    buffer: *mut FILE_POSITION_INFORMATION,
    length: *mut i32,
) {
    // PAGED_CODE();
    DebugTrace(1, Dbg, b"FatQueryPositionInfo...\n\0".as_ptr() as *const i8, 0);
    (*buffer).CurrentByteOffset = (*file_object).CurrentByteOffset;
    *length -= core::mem::size_of::<FILE_POSITION_INFORMATION>() as i32;
    DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *length);
    DebugTrace(-1, Dbg, b"FatQueryPositionInfo -> VOID\n\0".as_ptr() as *const i8, 0);
    let _ = irp_context;
}

// ==== recovered from stitcher-drop: 06_FatQueryNameInfo.rs ====
const CCB_FLAG_OPENED_BY_SHORTNAME: u32 = 0x0800; // placeholder for the actual flag value

extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatQueryNameInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
    Normalized: u8,
    Buffer: *mut FILE_NAME_INFORMATION,
    Length: *mut i32,
) {
    // PAGED_CODE(); – ignored for translation

    DebugTrace(1, Dbg as u32, b"FatQueryNameInfo...\n\0".as_ptr() as *const i8, 0);

    *Length -= core::mem::offset_of!(FILE_NAME_INFORMATION, FileName) as i32;

    if (*Fcb).FullFileName.Buffer.is_null() {
        FatSetFullFileNameInFcb(IrpContext, Fcb);
    }

    let mut TrimLength: i32 = 0;

    // LongName.Unicode is __BindgenUnionField<FILE_NAME_NODE>; cast backing storage
    let longname_unicode_buffer_null = {
        let fnn = core::ptr::addr_of!((*Fcb).LongName.bindgen_union_field) as *const crate::types::FILE_NAME_NODE;
        let us = core::ptr::addr_of!((*fnn).Name.bindgen_union_field) as *const UNICODE_STRING;
        (*us).Buffer.is_null()
    };
    if Normalized == 0 && !longname_unicode_buffer_null {
        if !Ccb.is_null()
            && ((*Ccb).Flags & CCB_FLAG_OPENED_BY_SHORTNAME) != 0
        {
            TrimLength = (*Fcb).FinalNameLength as i32;
        }
    }

    let mut BytesToCopy: u32;
    let mut Overflow: u8 = 0;

    if *Length < ((*Fcb).FullFileName.Length as i32) - TrimLength {
        BytesToCopy = *Length as u32;
        Overflow = 1;
    } else {
        BytesToCopy = ((*Fcb).FullFileName.Length as i32 - TrimLength) as u32;
        *Length -= BytesToCopy as i32;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*Buffer).FileName) as *mut core::ffi::c_void,
        (*Fcb).FullFileName.Buffer as *mut core::ffi::c_void,
        BytesToCopy as usize,
    );

    (*Buffer).FileNameLength = ((*Fcb).FullFileName.Length as i32 - TrimLength) as u32;

    if TrimLength != 0 {
        let mut ShortNameBuffer: [u16; 12] = [0u16; 12];
        let mut ShortName: UNICODE_STRING = UNICODE_STRING {
            Length: 0,
            MaximumLength: core::mem::size_of_val(&ShortNameBuffer) as u16,
            Buffer: ShortNameBuffer.as_mut_ptr(),
        };

        let Status: i32 = RtlOemStringToCountedUnicodeString(
            &mut ShortName as *mut _ as *mut _UNICODE_STRING,
            core::ptr::addr_of!((*Fcb).ShortName.Name.bindgen_union_field) as *const OEM_STRING,
            0,
        );

        ntkrnl_sys::ASSERT!(Status == STATUS_SUCCESS as i32);

        if Overflow == 0 {
            if *Length < ShortName.Length as i32 {
                BytesToCopy = *Length as u32;
                Overflow = 1;
            } else {
                BytesToCopy = ShortName.Length as u32;
                *Length -= BytesToCopy as i32;
            }

            let dst = (core::ptr::addr_of_mut!((*Buffer).FileName) as *mut u8)
                .add((*Buffer).FileNameLength as usize);
            RtlCopyMemory(
                dst as *mut core::ffi::c_void,
                ShortName.Buffer as *mut core::ffi::c_void,
                BytesToCopy as usize,
            );
        }

        (*Buffer).FileNameLength += ShortName.Length as u32;
    }

    if Overflow != 0 {
        *Length = -1;
    }

    DebugTrace(
        0,
        Dbg as u32,
        b"*Length = %08lx\n\0".as_ptr() as *const i8,
        *Length,
    );

    DebugTrace(-1, Dbg as u32, b"FatQueryNameInfo -> VOID\n\0".as_ptr() as *const i8, 0);

    let _ = IrpContext; // UNREFERENCED_PARAMETER
}


// ==== recovered from stitcher-drop: 07_FatQueryShortNameInfo.rs ====
extern "system" {
}

unsafe extern "system" fn FatQueryShortNameInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Buffer: *mut FILE_NAME_INFORMATION,
    Length: *mut i32,
) {
    let mut bytes_to_copy: u32;
    let mut short_name_buffer: [u16; 12] = [0; 12];
    let mut short_name: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of::<[u16; 12]>() as u16,
        Buffer: short_name_buffer.as_mut_ptr(),
    };

    let _ = PAGED_CODE;

    DebugTrace(1, Dbg, b"FatQueryNameInfo...\n\0".as_ptr() as *const i8, 0);

    *Length -= core::mem::offset_of!(FILE_NAME_INFORMATION, FileName) as i32;

    let status = RtlOemStringToCountedUnicodeString(
        &mut short_name as *mut _ as *mut _UNICODE_STRING,
        core::ptr::addr_of!((*Fcb).ShortName.Name.bindgen_union_field) as *const OEM_STRING,
        0,
    );

    NT_ASSERT((status == STATUS_SUCCESS) as i32);

    if (*Length as u32) < (short_name.Length as u32) {
        bytes_to_copy = *Length as u32;
        *Length = -1;
    } else {
        bytes_to_copy = short_name.Length as u32;
        *Length -= short_name.Length as i32;
    }

    core::ptr::copy_nonoverlapping(
        short_name.Buffer as *const u8,
        (&mut (*Buffer).FileName).as_mut_ptr() as *mut u8,
        bytes_to_copy as usize,
    );

    (*Buffer).FileNameLength = short_name.Length as u32;

    DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);
    DebugTrace(-1, Dbg, b"FatQueryNameInfo -> VOID\n\0".as_ptr() as *const i8, 0);

    let _ = IrpContext;
    let _ = Fcb;
}


// ==== recovered from stitcher-drop: 08_FatQueryNetworkInfo.rs ====
// Forward declarations for functions/globals/constants not in the import list
extern "system" {
}

const FCB_STATE_TEMPORARY: u32 = 0x20;

pub unsafe extern "system" fn FatQueryNetworkInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FileObject: *mut FILE_OBJECT,
    Buffer: *mut FILE_NETWORK_OPEN_INFORMATION,
    Length: *mut i32,
) {
    // PagedCode(); // no-op in Rust

    let _ = FileObject; // UNREFERENCED_PARAMETER

    DebugTrace(1, Dbg, b"FatQueryNetworkInfo...\n\0".as_ptr() as *const i8, 0);

    // Zero out the output buffer
    core::ptr::write_bytes(
        Buffer as *mut u8,
        0,
        core::mem::size_of::<FILE_NETWORK_OPEN_INFORMATION>(),
    );

    if (*Fcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB as i16 {
        ExLocalTimeToSystemTime(
            core::ptr::addr_of!(FatJanOne1980) as *mut _,
            core::ptr::addr_of_mut!((*Buffer).LastWriteTime),
        );
        (*Buffer).CreationTime = (*Buffer).LastWriteTime;
        (*Buffer).LastAccessTime = (*Buffer).LastWriteTime;
    } else {
        (*Buffer).LastWriteTime.QuadPart = (*Fcb).LastWriteTime.QuadPart;
        (*Buffer).CreationTime.QuadPart = (*Fcb).CreationTime.QuadPart;
        (*Buffer).LastAccessTime.QuadPart = (*Fcb).LastAccessTime.QuadPart;
    }

    (*Buffer).FileAttributes = (*Fcb).DirentFatFlags as u32;

    if FlagOn((*Fcb).FcbState, FCB_STATE_TEMPORARY) != 0 {
        let attr_ptr = core::ptr::addr_of_mut!((*Buffer).FileAttributes);
        SetFlag(core::ptr::addr_of_mut!((*Buffer).FileAttributes), 0x100u32);
    }

    if (*Buffer).FileAttributes == 0 {
        (*Buffer).FileAttributes = FILE_ATTRIBUTE_NORMAL;
    }

    if NodeType(Fcb) == FAT_NTC_FCB {
        if (*Fcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
            FatLookupFileAllocationSize(IrpContext, Fcb);
        }
        (*Buffer).AllocationSize.QuadPart = (*Fcb).Header.AllocationSize.QuadPart;
        (*Buffer).EndOfFile.QuadPart = (*Fcb).Header.FileSize.QuadPart;
    }

    *Length -= core::mem::size_of::<FILE_NETWORK_OPEN_INFORMATION>() as i32;

    DebugTrace(0, Dbg, b"*Length = %08lx\n\0".as_ptr() as *const i8, *Length);
    DebugTrace(-1, Dbg, b"FatQueryNetworkInfo -> VOID\n\0".as_ptr() as *const i8, 0);
}


// ==== recovered from stitcher-drop: 09_FatSetBasicInfo.rs ====
// Forward declarations for unknown extern functions
extern "system" {
}

// Constants for flags and attributes (guessed standard values)
const CCB_FLAG_USER_SET_CREATION: u32 = 0x40;
const CCB_FLAG_USER_SET_LAST_ACCESS: u32 = 0x20;
const FO_TEMPORARY_FILE: u32 = 0x8000;
const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x4;
const FILE_NOTIFY_CHANGE_CREATION: u32 = 0x40;
const FILE_NOTIFY_CHANGE_LAST_ACCESS: u32 = 0x20;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x10;

// Time arithmetic constants (100 ns units)
const TEN_MSEC: i64 = 100000;
const ALMOST_TEN_MSEC: i64 = TEN_MSEC - 1;
const TWO_SECONDS: i64 = 20000000;
const ALMOST_TWO_SECONDS: i64 = TWO_SECONDS - 1;

// Global references (assumed extern)
extern "system" {
}

unsafe extern "system" fn FatSetBasicInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
) -> NTSTATUS {
    // PAGED_CODE();    // ignored

    DebugTrace(1, Dbg, b"FatSetBasicInfo...\n\0".as_ptr() as *const i8, 0);

    let mut Status: NTSTATUS;
    let Buffer: *mut FILE_BASIC_INFORMATION = (*Irp).AssociatedIrp.SystemBuffer as *mut _;

    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();

    let mut CreationTime: FAT_TIME_STAMP = unsafe { core::mem::zeroed() };
    let mut CreationMSec: u8 = 0;
    let mut LastWriteTime: FAT_TIME_STAMP = unsafe { core::mem::zeroed() };
    let mut LastAccessTime: FAT_TIME_STAMP = unsafe { core::mem::zeroed() };
    let mut LastAccessDate: u16 = 0;
    let mut Attributes: u8 = 0;

    let mut ModifyCreation: bool = false;
    let mut ModifyLastWrite: bool = false;
    let mut ModifyLastAccess: bool = false;

    let mut ModifiedAttributes: bool = false; // Win8+

    let mut LargeCreationTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut LargeLastWriteTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut LargeLastAccessTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let mut NotifyFilter: u32 = 0;

    if (*Buffer).LastWriteTime.QuadPart == -1i64 {
        SetFlag(
            core::ptr::addr_of_mut!((*Ccb).Flags),
            CCB_FLAG_USER_SET_LAST_WRITE,
        );
        (*Buffer).LastWriteTime.QuadPart = 0;
    }

    if (*Buffer).LastAccessTime.QuadPart == -1i64 {
        SetFlag(
            core::ptr::addr_of_mut!((*Ccb).Flags),
            CCB_FLAG_USER_SET_LAST_ACCESS,
        );
        (*Buffer).LastAccessTime.QuadPart = 0;
    }

    if (*Buffer).CreationTime.QuadPart == -1i64 {
        SetFlag(
            core::ptr::addr_of_mut!((*Ccb).Flags),
            CCB_FLAG_USER_SET_CREATION,
        );
        (*Buffer).CreationTime.QuadPart = 0;
    }

    DirentBcb = core::ptr::null_mut();
    Status = STATUS_SUCCESS;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_status = core::ptr::addr_of_mut!(Status);
    crate::seh::try_finally(&mut (), |_| {

        let mut FatLocalDecThirtyOne1979: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        let mut FatLocalJanOne1980: LARGE_INTEGER = unsafe { core::mem::zeroed() };

        ExLocalTimeToSystemTime(
            core::ptr::addr_of!(FatDecThirtyOne1979) as *mut LARGE_INTEGER,
            core::ptr::addr_of_mut!(FatLocalDecThirtyOne1979),
        );
        ExLocalTimeToSystemTime(
            core::ptr::addr_of!(FatJanOne1980) as *mut LARGE_INTEGER,
            core::ptr::addr_of_mut!(FatLocalJanOne1980),
        );

        // Assert FcbGood
        // NT_ASSERT( (*Fcb).FcbCondition == FcbGood );

        FatGetDirentFromFcbOrDcb(
            IrpContext,
            Fcb,
            0u8, // FALSE
            core::ptr::addr_of_mut!(Dirent),
            (core::ptr::addr_of_mut!(*p_dirent_bcb)).cast(),
        );

        if (*(core::ptr::addr_of!(FatData))).ChicagoMode() != 0
            && (*Buffer).CreationTime.QuadPart != 0
        {
            LargeCreationTime = (*Buffer).CreationTime;

            if FatNtTimeToFatTime(
                IrpContext,
                core::ptr::addr_of!(LargeCreationTime),
                0u8, // FALSE
                core::ptr::addr_of_mut!(CreationTime),
                core::ptr::addr_of_mut!(CreationMSec),
            ) == 0
            {
                if LargeCreationTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart
                    && LargeCreationTime.QuadPart < FatLocalJanOne1980.QuadPart
                {
                    CreationTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
                    LargeCreationTime = FatLocalJanOne1980;
                } else {
                    DebugTrace(0, Dbg, b"Invalid CreationTime\n\0".as_ptr() as *const i8, 0);
                    *p_status = STATUS_INVALID_PARAMETER;
                    return; // try_return
                }
                CreationMSec = 0;
            }
            ModifyCreation = true;
        }

        if (*(core::ptr::addr_of!(FatData))).ChicagoMode() != 0
            && (*Buffer).LastAccessTime.QuadPart != 0
        {
            LargeLastAccessTime = (*Buffer).LastAccessTime;

            if FatNtTimeToFatTime(
                IrpContext,
                core::ptr::addr_of!(LargeLastAccessTime),
                1u8, // TRUE
                core::ptr::addr_of_mut!(LastAccessTime),
                core::ptr::null_mut(),
            ) == 0
            {
                if LargeLastAccessTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart
                    && LargeLastAccessTime.QuadPart < FatLocalJanOne1980.QuadPart
                {
                    LastAccessTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
                    LargeLastAccessTime = FatLocalJanOne1980;
                } else {
                    DebugTrace(0, Dbg, b"Invalid LastAccessTime\n\0".as_ptr() as *const i8, 0);
                    *p_status = STATUS_INVALID_PARAMETER;
                    return; // try_return
                }
            }
            LastAccessDate = core::mem::transmute::<FAT_DATE, u16>(core::ptr::read(core::ptr::addr_of!(LastAccessTime.Date)));
            ModifyLastAccess = true;
        }

        if (*Buffer).LastWriteTime.QuadPart != 0 {
            if ModifyLastAccess
                && (*Buffer).LastWriteTime.QuadPart == (*Buffer).LastAccessTime.QuadPart
            {
                ModifyLastWrite = true;
                LastWriteTime = LastAccessTime;
                LargeLastWriteTime = LargeLastAccessTime;
            } else {
                LargeLastWriteTime = (*Buffer).LastWriteTime;

                if FatNtTimeToFatTime(
                    IrpContext,
                    core::ptr::addr_of!(LargeLastWriteTime),
                    1u8, // TRUE
                    core::ptr::addr_of_mut!(LastWriteTime),
                    core::ptr::null_mut(),
                ) == 0
                {
                    if LargeLastWriteTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart
                        && LargeLastWriteTime.QuadPart < FatLocalJanOne1980.QuadPart
                    {
                        LastWriteTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
                        LargeLastWriteTime = FatLocalJanOne1980;
                    } else {
                        DebugTrace(0, Dbg, b"Invalid LastWriteTime\n\0".as_ptr() as *const i8, 0);
                        *p_status = STATUS_INVALID_PARAMETER;
                        return; // try_return
                    }
                }
                ModifyLastWrite = true;
            }
        }

        if (*Buffer).FileAttributes != 0 {
            Attributes = ((*Buffer).FileAttributes
                & (FILE_ATTRIBUTE_READONLY
                    | FILE_ATTRIBUTE_HIDDEN
                    | FILE_ATTRIBUTE_SYSTEM
                    | FILE_ATTRIBUTE_DIRECTORY
                    | FILE_ATTRIBUTE_ARCHIVE)) as u8;

            if NodeType(Fcb) == FAT_NTC_FCB {
                if FlagOn((*Buffer).FileAttributes, FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    DebugTrace(0, Dbg, b"Attempt to set dir attribute on file\n\0".as_ptr() as *const i8, 0);
                    *p_status = STATUS_INVALID_PARAMETER;
                    return; // try_return
                }
            } else {
                Attributes |= FAT_DIRENT_ATTR_DIRECTORY;
            }

            if FlagOn((*Buffer).FileAttributes, FILE_ATTRIBUTE_TEMPORARY) != 0 {
                if NodeType(Fcb) == FAT_NTC_DCB {
                    DebugTrace(0, Dbg, b"No temporary directories\n\0".as_ptr() as *const i8, 0);
                    *p_status = STATUS_INVALID_PARAMETER;
                    return; // try_return
                }
                SetFlag(core::ptr::addr_of_mut!((*Fcb).FcbState), FCB_STATE_TEMPORARY);
                let stack = IoGetCurrentIrpStackLocation(Irp);
                SetFlag(
                    core::ptr::addr_of_mut!((*(*stack).FileObject).Flags),
                    FO_TEMPORARY_FILE,
                );
            } else {
                ClearFlag(core::ptr::addr_of_mut!((*Fcb).FcbState), FCB_STATE_TEMPORARY);
                let stack = IoGetCurrentIrpStackLocation(Irp);
                ClearFlag(
                    core::ptr::addr_of_mut!((*(*stack).FileObject).Flags),
                    FO_TEMPORARY_FILE,
                );
            }

            if (*Dirent).Attributes != Attributes {
                // note: DirentFatFlags type may be u8
                (*Fcb).DirentFatFlags = Attributes;
                (*Dirent).Attributes = Attributes;
                NotifyFilter |= FILE_NOTIFY_CHANGE_ATTRIBUTES;
                ModifiedAttributes = true; // assuming >= Win8
            }
        }

        if ModifyCreation {
            (*Fcb).CreationTime = LargeCreationTime;
            (*Dirent).CreationTime = CreationTime;
            (*Dirent).CreationMSec = CreationMSec;

            NotifyFilter |= FILE_NOTIFY_CHANGE_CREATION;

            (*Fcb).CreationTime.QuadPart =
                (((*Fcb).CreationTime.QuadPart + ALMOST_TEN_MSEC) / TEN_MSEC) * TEN_MSEC;

            SetFlag(
                core::ptr::addr_of_mut!((*Ccb).Flags),
                CCB_FLAG_USER_SET_CREATION,
            );
        }

        if ModifyLastAccess {
            (*Fcb).LastAccessTime = LargeLastAccessTime;
            (*Dirent).LastAccessDate = core::mem::transmute::<u16, FAT_DATE>(LastAccessDate);

            NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_ACCESS;

            ExSystemTimeToLocalTime(
                core::ptr::addr_of!((*Fcb).LastAccessTime) as *mut LARGE_INTEGER,
                core::ptr::addr_of_mut!((*Fcb).LastAccessTime),
            );
            (*Fcb).LastAccessTime.QuadPart =
                ((*Fcb).LastAccessTime.QuadPart / FatOneDay.QuadPart) * FatOneDay.QuadPart;
            ExLocalTimeToSystemTime(
                core::ptr::addr_of!((*Fcb).LastAccessTime) as *mut LARGE_INTEGER,
                core::ptr::addr_of_mut!((*Fcb).LastAccessTime),
            );

            SetFlag(
                core::ptr::addr_of_mut!((*Ccb).Flags),
                CCB_FLAG_USER_SET_LAST_ACCESS,
            );
        }

        if ModifyLastWrite {
            (*Fcb).LastWriteTime = LargeLastWriteTime;
            (*Dirent).LastWriteTime = LastWriteTime;

            NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_WRITE;

            (*Fcb).LastWriteTime.QuadPart =
                (((*Fcb).LastWriteTime.QuadPart + ALMOST_TWO_SECONDS) / TWO_SECONDS) * TWO_SECONDS;

            SetFlag(
                core::ptr::addr_of_mut!((*Ccb).Flags),
                CCB_FLAG_USER_SET_LAST_WRITE,
            );
        }

        if NotifyFilter != 0 {
            if NodeType(Fcb) == FAT_NTC_FCB {
                (*Dirent).FileSize = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                (*Dirent).FirstClusterOfFile = (*Fcb).FirstClusterOfFile as u16;

                if FatIsFat32((*Fcb).Vcb) != 0 {
                    (*Dirent).__bindgen_anon_1.bindgen_union_field = ((*Fcb).FirstClusterOfFile >> 16) as u16;
                }
            }

            FatNotifyReportChange(
                IrpContext,
                (*Fcb).Vcb,
                Fcb,
                NotifyFilter,
                FILE_ACTION_MODIFIED,
            );

            FatSetDirtyBcb(IrpContext, *p_dirent_bcb, (*Fcb).Vcb, 1u8);
        }

        // #if (NTDDI_VERSION >= NTDDI_WIN8) -- see crate::NTDDI_WIN8
        if crate::NTDDI_WIN8 {
            // Win8+ oplock break
            if (*Fcb).ParentDcb.is_null() == false
                && (ModifyLastAccess || ModifyLastWrite || ModifiedAttributes)
            {
                let oplock = FatGetFcbOplock((*Fcb).ParentDcb);
                FsRtlCheckOplockEx(
                    oplock,
                    (*IrpContext).OriginatingIrp,
                    OPLOCK_FLAG_PARENT_OBJECT,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        }
    }, |_, _abnormal| {
        DebugUnwind(b"FatSetBasicInfo\0".as_ptr() as *const core::ffi::c_void);
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_dirent_bcb) as *mut *mut core::ffi::c_void);
        DebugTrace(-1, Dbg, b"FatSetBasicInfo -> %08lx\n\0".as_ptr() as *const i8, *p_status);
    });

    return Status;
}

// ==== recovered from stitcher-drop: 10_FatSetDispositionInfo.rs ====
extern "system" {
}

const FILE_FLOPPY_DISKETTE: u32 = 0x04;
const FAT_LARGE_ZERO: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
const PAGE_SIZE: u32 = 0x1000; // assume 4KB, adjust if needed
const TRUE: i32 = 1;
const FALSE: i32 = 0;
const STATUS_DIRECTORY_NOT_EMPTY: NTSTATUS = 0xC0000101u32 as i32; // assume, adjust

#[no_mangle]
pub unsafe extern "system" fn FatSetDispositionInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FileObject: *mut FILE_OBJECT,
    Fcb: *mut FCB,
) -> NTSTATUS {
    // PAGED_CODE();
    DebugTrace(1i32, Dbg, b"FatSetDispositionInfo...\n\0".as_ptr() as *const i8, 0i32);

    let Buffer = (*Irp).AssociatedIrp.SystemBuffer as *mut FILE_DISPOSITION_INFORMATION;

    if (*Buffer).DeleteFile != 0 {
        // Check if the file is marked read only
        if ((*Fcb).DirentFatFlags & FAT_DIRENT_ATTR_READ_ONLY) != 0 {
            DebugTrace(-1i32, Dbg, b"Cannot delete readonly file\n\0".as_ptr() as *const i8, 0i32);
            return STATUS_CANNOT_DELETE;
        }

        // Make sure there is no process mapping this file as an image.
        // C: `if (!MmFlushImageSection(...))`. Was `!MmFlushImageSection(..) == 0`, a
        // bitwise NOT on the u8 BOOLEAN that is never 0 for TRUE/FALSE, so a mapped
        // image was never refused.
        if MmFlushImageSection(
            core::ptr::addr_of!((*(*Fcb).NonPaged).SectionObjectPointers) as *const _,
            MmFlushForDelete,
        ) == 0
        {
            DebugTrace(-1i32, Dbg, b"Cannot delete user mapped image\n\0".as_ptr() as *const i8, 0i32);
            return STATUS_CANNOT_DELETE;
        }

        // Check if this is a dcb and if so then only allow the request if the directory is empty.
        if (*Fcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB as i16 {
            DebugTrace(-1i32, Dbg, b"Cannot delete root Directory\n\0".as_ptr() as *const i8, 0i32);
            return STATUS_CANNOT_DELETE;
        }

        if (*Fcb).Header.NodeTypeCode == FAT_NTC_DCB as i16 {
            DebugTrace(-1i32, Dbg, b"User wants to delete a directory\n\0".as_ptr() as *const i8, 0i32);

            if FatIsDirectoryEmpty(IrpContext, Fcb) == 0 {
                DebugTrace(-1i32, Dbg, b"Directory is not empty\n\0".as_ptr() as *const i8, 0i32);
                return STATUS_DIRECTORY_NOT_EMPTY;
            }
        }

        // If this is a floppy, touch the volume so to verify that it is not write protected.
        if ((*(*(*(*Fcb).Vcb).Vpb).RealDevice).Characteristics & FILE_FLOPPY_DISKETTE) != 0 {
            let Vcb = (*Fcb).Vcb;
            let mut LocalBcb = core::ptr::null_mut();
            let mut LocalBuffer: *mut u8 = core::ptr::null_mut();
            let mut TmpChar: u8 = 0;
            let BytesToMap: u32 = if (*Vcb).AllocationSupport.FatIndexBitSize == 12 {
                FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb)) + FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb))
            } else {
                PAGE_SIZE
            };
            let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();

            FatReadVolumeFile(
                IrpContext,
                Vcb,
                0,
                BytesToMap,
                core::ptr::addr_of_mut!(LocalBcb),
                core::ptr::addr_of_mut!(LocalBuffer) as *mut *mut u8,
            );

            // _SEH2_TRY / _SEH2_FINALLY
            let p_local_bcb = core::ptr::addr_of_mut!(LocalBcb);
            crate::seh::try_finally(&mut (), |_| {
                if CcPinMappedData(
                    (*Vcb).VirtualVolumeFile as PVOID,
                    &FAT_LARGE_ZERO,
                    BytesToMap,
                    (((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0) as i32,
                    p_local_bcb,
                ) == 0
                {
                    // Could not pin the data without waiting (cache miss).
                    FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
                }

                // Make Mm, myself, and Cc think the byte is dirty, and then force a writethrough.
                LocalBuffer = LocalBuffer.offset(FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb)) as isize);
                TmpChar = *LocalBuffer;
                *LocalBuffer = TmpChar;

                FatAddMcbEntry(
                    Vcb,
                    core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb),
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb)) as u32,
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb)) as i64,
                    ((*Vcb).Bpb.BytesPerSector) as u32,
                );
            }, |_, abnormal| {
                if abnormal && !(*p_local_bcb).is_null() {
                    FatUnpinBcb(IrpContext, p_local_bcb as *mut *mut core::ffi::c_void);
                }
            });

            CcRepinBcb(LocalBcb);
            CcSetDirtyPinnedData(LocalBcb, core::ptr::null());
            CcUnpinData(LocalBcb);

            // DbgDoit( NT_ASSERT( IrpContext->PinCount ));
            if cfg!(debug_assertions) {
                ntkrnl_sys::ASSERT!((*IrpContext).PinCount != 0);
                (*IrpContext).PinCount -= 1;
            }

            CcUnpinRepinnedBcb(LocalBcb, TRUE, core::ptr::addr_of_mut!(Iosb));

            if !NT_SUCCESS(Iosb.__bindgen_anon_1.Status) {
                FatNormalizeAndRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
            }
        } else {
            let mut Dirent: *mut DIRENT = core::ptr::null_mut();
            let mut Bcb: PBCB = core::ptr::null_mut();

            FatGetDirentFromFcbOrDcb(IrpContext, Fcb, 0u8, core::ptr::addr_of_mut!(Dirent), core::ptr::addr_of_mut!(Bcb) as *mut _);

            // _SEH2_TRY / _SEH2_FINALLY
            let p_bcb = core::ptr::addr_of_mut!(Bcb);
            crate::seh::try_finally(&mut (), |_| {
                FatSetDirtyBcb(IrpContext, *p_bcb as *mut _, (*Fcb).Vcb, 1u8);
            }, |_, _abnormal| {
                FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);
            });
        }

        // At this point either we have a file or an empty directory so we know the delete can proceed.
        (*Fcb).FcbState |= FCB_STATE_DELETE_ON_CLOSE;
        (*FileObject).DeletePending = (TRUE) as u8;

        // If this is a directory then report this delete pending to the dir notify package.
        if (*Fcb).Header.NodeTypeCode == FAT_NTC_DCB as i16 {
            FsRtlNotifyFullChangeDirectory(
                (*(*Fcb).Vcb).NotifySync,
                core::ptr::addr_of_mut!((*(*Fcb).Vcb).DirNotifyList) as PLIST_ENTRY,
                (*FileObject).FsContext,
                core::ptr::null_mut(),
                0u8,
                0u8,
                0,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
            );
        }
    } else {
        // The user doesn't want to delete the file so clear the delete on close bit
        DebugTrace(0i32, Dbg, b"User want to not delete file\n\0".as_ptr() as *const i8, 0i32);

        (*Fcb).FcbState &= !FCB_STATE_DELETE_ON_CLOSE;
        (*FileObject).DeletePending = (FALSE) as u8;
    }

    DebugTrace(-1i32, Dbg, b"FatSetDispositionInfo -> STATUS_SUCCESS\n\0".as_ptr() as *const i8, 0i32);
    STATUS_SUCCESS
}

// ==== recovered from stitcher-drop: 11_FatSetPositionInfo.rs ====
extern "system" {
}

const FO_NO_INTERMEDIATE_BUFFERING: u32 = 8;

#[no_mangle]
pub unsafe extern "system" fn FatSetPositionInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FileObject: *mut FILE_OBJECT,
) -> NTSTATUS {
    let Buffer: *mut FILE_POSITION_INFORMATION;

    let _ = PAGED_CODE;

    DebugTrace(1, Dbg, c"FatSetPositionInfo...\n".as_ptr().cast::<i8>(), 0);

    Buffer = (*Irp).AssociatedIrp.SystemBuffer as *mut FILE_POSITION_INFORMATION;

    if ((*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING) != 0 {
        let DeviceObject = (*IoGetCurrentIrpStackLocation(Irp)).DeviceObject;

        if ((*Buffer).CurrentByteOffset.__bindgen_anon_1.LowPart & (*DeviceObject).AlignmentRequirement) != 0 {
            DebugTrace(
                0,
                Dbg,
                c"Cannot set position due to aligment conflict\n".as_ptr().cast::<i8>(),
                0,
            );
            DebugTrace(
                -1,
                Dbg,
                c"FatSetPositionInfo -> %08lx\n".as_ptr().cast::<i8>(),
                STATUS_INVALID_PARAMETER,
            );

            return STATUS_INVALID_PARAMETER;
        }
    }

    DebugTrace(
        0,
        Dbg,
        c"Set the new position to %08lx\n".as_ptr().cast::<i8>(),
        (*Buffer).CurrentByteOffset.__bindgen_anon_1.LowPart,
    );

    (*FileObject).CurrentByteOffset = (*Buffer).CurrentByteOffset;

    DebugTrace(
        -1,
        Dbg,
        c"FatSetPositionInfo -> %08lx\n".as_ptr().cast::<i8>(),
        STATUS_SUCCESS,
    );

    let _ = IrpContext;

    STATUS_SUCCESS
}

// ==== recovered from stitcher-drop: 12_FatSetAllocationInfo.rs ====
extern "system" {
}

const STATUS_PURGE_FAILED: i32 = 0xC0000435_u32 as i32; // placeholder
const STATUS_USER_MAPPED_FILE: i32 = 0xC0000243_u32 as i32; // placeholder
const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x00000008; // placeholder
const FCB_STATE_TRUNCATE_ON_CLOSE: u32 = 0x00000002; // placeholder
const IRP_PAGING_IO: u32 = 0x02; // placeholder
const FO_CLEANUP_COMPLETE: u32 = 0x4000; // placeholder

#[no_mangle]
pub unsafe extern "system" fn FatSetAllocationInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Fcb: *mut FCB,
    FileObject: *mut FILE_OBJECT,
) -> i32 {
    let _ = PAGED_CODE;

    let mut Status: i32 = STATUS_SUCCESS;
    let Buffer: *mut FILE_ALLOCATION_INFORMATION =
        (*Irp).AssociatedIrp.SystemBuffer as *mut FILE_ALLOCATION_INFORMATION;
    let mut NewAllocationSize: u32 = 0;
    let mut HeaderSize: u32 = 0;
    let mut FileSizeTruncated: i32 = 0;
    let mut CacheMapInitialized: i32 = 0;
    let mut ResourceAcquired: i32 = 0;
    let mut OriginalFileSize: u32 = 0;
    let mut OriginalValidDataLength: u32 = 0;
    let mut OriginalValidDataToDisk: u32 = 0;

    NewAllocationSize = (*Buffer).AllocationSize.__bindgen_anon_1.LowPart;

    DebugTrace(
        1,
        Dbg,
        b"FatSetAllocationInfo.. to %08lx\n\0".as_ptr() as *const i8,
        NewAllocationSize,
    );

    // Allocation is only allowed on a file and not a directory
    if NodeType(Fcb) == FAT_NTC_DCB {
        DebugTrace(
            -1,
            Dbg,
            b"Cannot change allocation of a directory\n\0".as_ptr() as *const i8,
            0,
        );
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    // Check that the new file allocation is legal
    if FatIsIoRangeValid((*Fcb).Vcb, (*Buffer).AllocationSize, 0) == 0 {
        DebugTrace(
            -1,
            Dbg,
            b"Illegal allocation size\n\0".as_ptr() as *const i8,
            0,
        );
        return STATUS_DISK_FULL;
    }

    // If we haven't yet looked up the correct AllocationSize, do so.
    if (*Fcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
        FatLookupFileAllocationSize(IrpContext, Fcb);
    }

    // This is kinda gross, but if the file is not cached, but there is
    // a data section, we have to cache the file to avoid a bunch of
    // extra work.
    if (*(*FileObject).SectionObjectPointer).DataSectionObject != core::ptr::null_mut()
        && (*(*FileObject).SectionObjectPointer).SharedCacheMap == core::ptr::null_mut()
        && FlagOn((*Irp).Flags, IRP_PAGING_IO) == 0
    {
        NT_ASSERT((FlagOn((*FileObject).Flags, FO_CLEANUP_COMPLETE) == 0) as i32);

        // Now initialize the cache map.
        FatInitializeCacheMap(
            FileObject,
            &(*Fcb).Header.AllocationSize as *const _ as *const CC_FILE_SIZES,
            0,
            &raw const FatData.CacheManagerCallbacks,
            Fcb,
        );

        CacheMapInitialized = 1;
    }

    // Now mark the fact that the file needs to be truncated on close
    (*Fcb).FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;

    // Now mark that the time on the dirent needs to be updated on close.
    SetFlag(&mut (*FileObject).Flags, FO_FILE_MODIFIED);

    // _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_file_size_truncated = core::ptr::addr_of_mut!(FileSizeTruncated);
    let p_resource_acquired = core::ptr::addr_of_mut!(ResourceAcquired);
    let p_original_file_size = core::ptr::addr_of_mut!(OriginalFileSize);
    let p_original_valid_data_length = core::ptr::addr_of_mut!(OriginalValidDataLength);
    let p_original_valid_data_to_disk = core::ptr::addr_of_mut!(OriginalValidDataToDisk);
    crate::seh::try_finally(&mut (), |_| {
        if NewAllocationSize.wrapping_add(HeaderSize) > (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
            FatAddFileAllocation(
                IrpContext,
                Fcb,
                FileObject,
                NewAllocationSize.wrapping_add(HeaderSize),
            );
        } else {
            // Check here if we will be decreasing file size and synchronize with
            // paging IO.
            if (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart > NewAllocationSize.wrapping_add(HeaderSize) {
                // The way Sections for DataScan are created and used, an AV's
                // memory-mapping can come into being after we check it's safe
                // to truncate a file while continuing to hold the file here.
                // This leads to data corruption because Purge eventually fails
                // (during call to Cc to set file sizes) and stale data continues
                // to live in the cache/memory.
                if (*Fcb).PurgeFailureModeEnableCount != 0 {
                    *p_status = STATUS_PURGE_FAILED;
                    return; // try_return
                }

                // Before we actually truncate, check to see if the purge
                // is going to fail.
                if MmCanFileBeTruncated(
                    (*FileObject).SectionObjectPointer,
                    core::ptr::addr_of!((*Buffer).AllocationSize) as *mut _,
                ) == 0
                {
                    *p_status = STATUS_USER_MAPPED_FILE;
                    return; // try_return
                }

                *p_file_size_truncated = 1;

                *p_original_file_size = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                *p_original_valid_data_length = (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart;
                *p_original_valid_data_to_disk = (*Fcb).ValidDataToDisk;

                ExAcquireResourceExclusiveLite((*Fcb).Header.PagingIoResource, 1);
                *p_resource_acquired = 1;

                (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = NewAllocationSize;

                // If we reduced the file size to less than the ValidDataLength,
                // adjust the VDL.  Likewise ValidDataToDisk.
                if (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart > (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart {
                    (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                }
                if (*Fcb).ValidDataToDisk > (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart {
                    (*Fcb).ValidDataToDisk = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                }
            }

            // Now that File Size is down, actually do the truncate.
            FatTruncateFileAllocation(
                IrpContext,
                Fcb,
                NewAllocationSize.wrapping_add(HeaderSize),
            );

            // Now check if we needed to decrease the file size accordingly.
            if *p_file_size_truncated != 0 {
                // Tell the cache manager we reduced the file size.
                // The call is unconditional, because MM always wants to know.
                // C (compiled with DBG=1): _SEH2_TRY { CcSetFileSizes }
                // _SEH2_EXCEPT(FatBugCheckExceptionFilter(...)) { NOTHING; }
                let _ = crate::seh::try_except(&mut (), |_| {
                    CcSetFileSizes(
                        FileObject,
                        core::ptr::addr_of!((*Fcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                    );
                }, |_, pointers| {
                    crate::fatdata::FatBugCheckExceptionFilter(pointers as *mut EXCEPTION_POINTERS) as i32
                });

                NT_ASSERT(((*FileObject).DeleteAccess != 0 || (*FileObject).WriteAccess != 0) as i32);

                // There is no going back from this. If we run into problems updating
                // the dirent we will have to live with the consequences.
                *p_file_size_truncated = 0;

                FatSetFileSizeInDirent(IrpContext, Fcb, core::ptr::null_mut());

                // Report that we just reduced the file size.
                FatNotifyReportChange(
                    IrpContext,
                    (*Fcb).Vcb,
                    Fcb,
                    FILE_NOTIFY_CHANGE_SIZE,
                    FILE_ACTION_MODIFIED,
                );
            }
        }
    }, |_, abnormal| {
        if abnormal && *p_file_size_truncated != 0 {
            (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = *p_original_file_size;
            (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = *p_original_valid_data_length;
            (*Fcb).ValidDataToDisk = *p_original_valid_data_to_disk;

            // Make sure Cc knows the right filesize.
            if (*(*FileObject).SectionObjectPointer).SharedCacheMap != core::ptr::null_mut() {
                *CcGetFileSizePointer(FileObject) = (*Fcb).Header.FileSize;
            }

            NT_ASSERT(
                ((*Fcb).Header.FileSize.__bindgen_anon_1.LowPart <= (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart) as i32,
            );
        }

        if CacheMapInitialized != 0 {
            CcUninitializeCacheMap(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
        }

        if *p_resource_acquired != 0 {
            ExReleaseResourceLite((*Fcb).Header.PagingIoResource);
        }
    });

    DebugTrace(
        -1,
        Dbg,
        b"FatSetAllocationInfo -> %08lx\n\0".as_ptr() as *const i8,
        STATUS_SUCCESS,
    );

    Status
}

// ==== recovered from stitcher-drop: 13_FatSetValidDataLengthInfo.rs ====
#[no_mangle]
pub unsafe extern "system" fn FatSetValidDataLengthInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FileObject: *mut FILE_OBJECT,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
) -> NTSTATUS {
    const FAT_NTC_FCB: u16 = 0x0502; // typical FastFat node type
    const CCB_FLAG_MANAGE_VOLUME_ACCESS: u32 = 0x00020000; // typical value
    const STATUS_DISK_FULL: i32 = 0xC000007F_u32 as i32;
    const FO_FILE_MODIFIED: u32 = 0x1000;

    // Forward declarations for unknown external symbols
    extern "system" {
        fn FatIsIoRangeValid(Vcb: *mut VCB, length: LARGE_INTEGER, is_set: i32) -> u8;
    }

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut resource_acquired: i32 = 0;

    // PAGED_CODE();  // debug check not transliterated

    let _ = IrpContext; // UNREFERENCED_PARAMETER

    DebugTrace(1, Dbg, b"FatSetValidDataLengthInfo...\n\0".as_ptr() as *const i8, 0);

    let buffer: *mut FILE_VALID_DATA_LENGTH_INFORMATION =
        (*Irp).AssociatedIrp.SystemBuffer as *mut _;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(status);
    let p_resource_acquired = core::ptr::addr_of_mut!(resource_acquired);
    crate::seh::try_finally(&mut (), |_| {
        // if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS ))
        if Ccb.is_null() || ((*Ccb).Flags & CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        // if (NodeType(Fcb) != FAT_NTC_FCB)
        if ((*Fcb).Header.NodeTypeCode) != (FAT_NTC_FCB) as i16 {
            DebugTrace(0, Dbg, b"Cannot change VDL of a directory\n\0".as_ptr() as *const i8, 0);
            *p_status = STATUS_INVALID_DEVICE_REQUEST;
            return; // try_return
        }

        // if (!FatIsIoRangeValid( Fcb->Vcb, Buffer->ValidDataLength, 0))
        if FatIsIoRangeValid((*Fcb).Vcb, (*buffer).ValidDataLength, 0) == 0 {
            DebugTrace(0, Dbg, b"Illegal allocation size\n\0".as_ptr() as *const i8, 0);
            *p_status = STATUS_DISK_FULL;
            return; // try_return
        }

        let new_valid_data_length: u32 = (*buffer).ValidDataLength.__bindgen_anon_1.LowPart;

        // if ((NewValidDataLength < Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart) ||
        //     (NewValidDataLength > Fcb->Header.FileSize.__bindgen_anon_1.LowPart))
        if (new_valid_data_length < (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart)
            || (new_valid_data_length > (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart)
        {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        // if (!MmCanFileBeTruncated(...))
        if MmCanFileBeTruncated(
            (*FileObject).SectionObjectPointer,
            &(*buffer).ValidDataLength as *const LARGE_INTEGER as *mut LARGE_INTEGER,
        ) == 0
        {
            *p_status = STATUS_USER_MAPPED_FILE;
            return; // try_return
        }

        // if (FileObject->SectionObjectPointer->DataSectionObject != NULL)
        if !(*(*FileObject).SectionObjectPointer).DataSectionObject.is_null() {
            *p_resource_acquired =
                (ExAcquireResourceExclusiveLite((*Fcb).Header.PagingIoResource, 1)) as i32;

            CcFlushCache(
                ((*FileObject).SectionObjectPointer).cast(),
                core::ptr::null_mut(),
                0,
                &mut (*Irp).IoStatus,
            );

            if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                // Diverges from the C deliberately: `try_return( Irp->IoStatus.Status )`
                // evaluates the status without assigning it, so the C reports
                // STATUS_SUCCESS for a failed flush. Return the failure.
                *p_status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                return; // try_return
            }

            CcPurgeCacheSection(
                (*FileObject).SectionObjectPointer,
                core::ptr::null_mut(),
                0,
                0, // FALSE
            );
        }

        // Set new ValidDataLength
        (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = new_valid_data_length;
        (*Fcb).ValidDataToDisk = new_valid_data_length;

        DebugTrace(0, Dbg, b"New VDL is 0x%08lx.\n\0".as_ptr() as *const i8, new_valid_data_length);

        // if (FileObject->SectionObjectPointer->SharedCacheMap != NULL)
        if !(*(*FileObject).SectionObjectPointer).SharedCacheMap.is_null() {
            CcSetFileSizes(
                FileObject,
                &mut (*Fcb).Header.AllocationSize as *mut _ as *mut CC_FILE_SIZES,
            );
        }

        (*FileObject).Flags |= FO_FILE_MODIFIED;

        *p_status = STATUS_SUCCESS;
    }, |_, _abnormal| {
        DebugUnwind(b"FatSetValidDataLengthInfo\0".as_ptr() as *const core::ffi::c_void);

        if *p_resource_acquired != 0 {
            ExReleaseResourceLite((*Fcb).Header.PagingIoResource);
        }

        DebugTrace(-1, Dbg, b"FatSetValidDataLengthInfo -> %08lx\n\0".as_ptr() as *const i8, *p_status);
    });

    status
}

// ==== recovered from stitcher-drop: 14_FatRenameEAs.rs ====
extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatRenameEAs(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    ExtendedAttributes: u16,
    OldOemName: *mut OEM_STRING,
) {
    let mut locked_ea_fcb: i32 = 0;
    let mut ea_bcb: *mut BCB = core::ptr::null_mut();
    let mut ea_dirent: *mut DIRENT = core::ptr::null_mut();
    let mut ea_set_range: EA_RANGE = core::mem::zeroed();
    let mut ea_set_header: *mut EA_SET_HEADER = core::ptr::null_mut();
    let mut vcb: *mut VCB = core::ptr::null_mut();

    // PagedCode(); // no-op in Rust

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(ea_set_range) as *mut u8,
        0,
        core::mem::size_of::<EA_RANGE>(),
    );

    vcb = (*Fcb).Vcb;

    // _SEH2_TRY { _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { } }
    // _SEH2_FINALLY { ... }
    let p_locked_ea_fcb = core::ptr::addr_of_mut!(locked_ea_fcb);
    let p_ea_bcb = core::ptr::addr_of_mut!(ea_bcb);
    let p_ea_set_range = core::ptr::addr_of_mut!(ea_set_range);
    crate::seh::try_finally(&mut (), |_| {
        // Use a try-except to catch any errors; they are caught but not acted on.
        let _ = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
            FatGetEaFile(
                IrpContext,
                vcb,
                &mut ea_dirent,
                &mut *p_ea_bcb,
                0,
                0,
            );
            *p_locked_ea_fcb = 1;

            if !(*vcb).VirtualEaFile.is_null() {
                FatReadEaSet(
                    IrpContext,
                    vcb,
                    ExtendedAttributes,
                    OldOemName,
                    0,
                    &mut *p_ea_set_range,
                );

                ea_set_header = (*p_ea_set_range).Data as *mut EA_SET_HEADER;

                core::ptr::write_bytes(
                    (*ea_set_header).OwnerFileName.as_mut_ptr(),
                    0,
                    14,
                );

                {
                    let oem_str_ea = core::ptr::addr_of!((*Fcb).ShortName.Name.bindgen_union_field) as *const OEM_STRING;
                    core::ptr::copy_nonoverlapping(
                        (*oem_str_ea).Buffer as *const u8,
                        (*ea_set_header).OwnerFileName.as_mut_ptr(),
                        (*oem_str_ea).Length as usize,
                    );
                }

                FatMarkEaRangeDirty(
                    IrpContext,
                    (*vcb).VirtualEaFile,
                    &mut *p_ea_set_range,
                );
                FatUnpinEaRange(IrpContext, &mut *p_ea_set_range);

                CcFlushCache(
                    (*((*vcb).VirtualEaFile)).SectionObjectPointer as *mut core::ffi::c_void,
                    core::ptr::null(),
                    0,
                    core::ptr::null_mut(),
                );
            }
        });
    }, |_, _abnormal| {
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_ea_bcb) as *mut *mut core::ffi::c_void);
        FatUnpinEaRange(IrpContext, &mut *p_ea_set_range);

        if *p_locked_ea_fcb != 0 {
            FatReleaseFcb(IrpContext, (*vcb).EaFcb);
        }
    });
}


// ==== recovered from stitcher-drop: 17_FatSetRenameInfo.rs ====
// Forward declarations for external symbols not in imports
extern "system" {
    // FatDeleteFile forward-decl removed: real definition lives in this module
    // (re-stitched body below). Keeping both was an E0428 duplicate.
}

// lfn.h's #define, from the island. This was `extern static MAX_LFN_CHARACTERS: usize` over
// rust_cohab_fat.c's 4-byte ULONG: on 64-bit targets every read took TAG_DIRENT as its high
// half, so the rename name buffers pointed into nowhere and the first rename bugchecked 0x23.
const MAX_LFN_CHARACTERS: usize = crate::types::MAX_LFN_CHARACTERS as usize;

#[no_mangle]
pub unsafe extern "system" fn FatSetRenameInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Vcb: *mut VCB,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
) -> NTSTATUS {
    // Local variable declarations
    let mut AllLowerComponent: BOOLEAN = 0;
    let mut AllLowerExtension: BOOLEAN = 0;
    let mut CaseOnlyRename: BOOLEAN = 0;
    let mut ContinueWithRename: BOOLEAN = 0;
    let mut CreateLfn: BOOLEAN = 0;
    let mut DeleteSourceDirent: BOOLEAN = 0;
    let mut DeleteTarget: BOOLEAN = 0;
    let mut NewDirentFromPool: BOOLEAN = 0;
    let mut RenamedAcrossDirectories: BOOLEAN = 0;
    let mut ReplaceIfExists: BOOLEAN;
    let mut LocalCcb: CCB = core::mem::zeroed();
    let mut SourceCcb: *mut CCB = core::ptr::null_mut();
    let mut Dirent: DIRENT = unsafe { core::mem::zeroed() };
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut OldOemName: OEM_STRING = unsafe { core::mem::zeroed() };
    let mut NewOemName: OEM_STRING = unsafe { core::mem::zeroed() };
    let mut OemNameBuffer: [u8; 48] = [0u8; 48];
    let mut DotDotBcb: *mut PBCB = core::ptr::null_mut();
    let mut NewDirentBcb: *mut PBCB = core::ptr::null_mut();
    let mut OldDirentBcb: *mut PBCB = core::ptr::null_mut();
    let mut SecondPageBcb: *mut PBCB = core::ptr::null_mut();
    let mut TargetDirentBcb: *mut PBCB = core::ptr::null_mut();
    let mut TargetDcb: *mut DCB = core::ptr::null_mut();
    let mut OldParentDcb: *mut DCB = core::ptr::null_mut();
    let mut DotDotDirent: *mut DIRENT = core::ptr::null_mut();
    let mut FirstPageDirent: *mut DIRENT = core::ptr::null_mut();
    let mut NewDirent: *mut DIRENT = core::ptr::null_mut();
    let mut OldDirent: *mut DIRENT = core::ptr::null_mut();
    let mut SecondPageDirent: *mut DIRENT = core::ptr::null_mut();
    let mut ShortDirent: *mut DIRENT = core::ptr::null_mut();
    let mut TargetDirent: *mut DIRENT = core::ptr::null_mut();
    let mut TempFcb: *mut FCB = core::ptr::null_mut();
    let mut TargetFileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut IrpSp: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    let mut Links: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut BytesInFirstPage: u32 = 0;
    let mut DirentsInFirstPage: u32 = 0;
    let mut DirentsRequired: u32 = 0;
    let mut NewOffset: u32 = 0;
    let mut NotifyAction: u32 = 0;
    let mut SecondPageOffset: u32 = 0;
    let mut ShortDirentOffset: u32 = 0;
    let mut TargetDirentOffset: u32 = 0;
    let mut TargetLfnOffset: u32 = 0;
    let mut NewName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut NewNameCopy: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut NewUpcasedName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut OldName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut OldUpcasedName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut TargetLfn: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut UnicodeBuffer: *mut u16 = core::ptr::null_mut();
    let mut TargetOrigLfn: UNICODE_STRING = UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut UniTunneledShortName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut UniTunneledShortNameBuffer: [u16; 12] = [0u16; 12];
    let mut UniTunneledLongName: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut UniTunneledLongNameBuffer: [u16; 26] = [0u16; 26];
    let mut TunneledCreationTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut TunneledDataSize: u32 = 0;
    let mut HaveTunneledInformation: BOOLEAN = 0;
    let mut UsingTunneledLfn: BOOLEAN = 0;
    let mut InvalidateFcbOnRaise: BOOLEAN = 0;
    let mut DirectoryFileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut Flags: u32 = 0;

    // PAGED_CODE() is a no-op in Rust translation

    // DebugTrace call
    DebugTrace(1, Dbg, b"FatSetRenameInfo...\n\0".as_ptr() as *const i8, 0);

    // PHASE 0: Initialize variables
    CaseOnlyRename = 0;
    ContinueWithRename = 0;
    DeleteSourceDirent = 0;
    DeleteTarget = 0;
    NewDirentFromPool = 0;
    RenamedAcrossDirectories = 0;

    DotDotBcb = core::ptr::null_mut();
    NewDirentBcb = core::ptr::null_mut();
    OldDirentBcb = core::ptr::null_mut();
    SecondPageBcb = core::ptr::null_mut();
    TargetDirentBcb = core::ptr::null_mut();

    NewOemName.Length = 0;
    NewOemName.MaximumLength = 24;
    NewOemName.Buffer = OemNameBuffer.as_mut_ptr() as *mut i8;

    OldOemName.Length = 0;
    OldOemName.MaximumLength = 24;
    OldOemName.Buffer = OemNameBuffer.as_mut_ptr().add(24) as *mut i8;

    UnicodeBuffer = FsRtlAllocatePoolWithTag(PagedPool, (5 * MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u32, TAG_FILENAME_BUFFER) as *mut u16;

    NewUpcasedName.Length = 0;
    NewUpcasedName.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
    NewUpcasedName.Buffer = UnicodeBuffer.offset(0);

    OldName.Length = 0;
    OldName.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
    OldName.Buffer = UnicodeBuffer.offset(MAX_LFN_CHARACTERS as isize);

    OldUpcasedName.Length = 0;
    OldUpcasedName.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
    OldUpcasedName.Buffer = UnicodeBuffer.offset((MAX_LFN_CHARACTERS * 2) as isize);

    TargetLfn.Length = 0;
    TargetLfn.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
    TargetLfn.Buffer = UnicodeBuffer.offset((MAX_LFN_CHARACTERS * 3) as isize);

    NewNameCopy.Length = 0;
    NewNameCopy.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
    NewNameCopy.Buffer = UnicodeBuffer.offset((MAX_LFN_CHARACTERS * 4) as isize);

    UniTunneledShortName.Length = 0;
    UniTunneledShortName.MaximumLength = core::mem::size_of_val(&UniTunneledShortNameBuffer) as u16;
    UniTunneledShortName.Buffer = UniTunneledShortNameBuffer.as_mut_ptr();

    UniTunneledLongName.Length = 0;
    UniTunneledLongName.MaximumLength = core::mem::size_of_val(&UniTunneledLongNameBuffer) as u16;
    UniTunneledLongName.Buffer = UniTunneledLongNameBuffer.as_mut_ptr();

    // Remember old Oem name
    {
        let oem_str = core::ptr::addr_of!((*Fcb).ShortName.Name.bindgen_union_field) as *const OEM_STRING;
        core::ptr::copy_nonoverlapping(
            (*oem_str).Buffer as *const u8,
            OldOemName.Buffer as *mut u8,
            OldOemName.Length as usize,
        );
    }

    // Get current stack location
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    FileObject = (*IrpSp).FileObject;
    SourceCcb = (*FileObject).FsContext2 as *mut CCB;
    TargetFileObject = (*IrpSp).Parameters.SetFile.FileObject;
    ReplaceIfExists = (*IrpSp).Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.ReplaceIfExists;

    // Zero LocalCcb
    core::ptr::write_bytes(&mut LocalCcb as *mut _ as *mut u8, 0, core::mem::size_of::<CCB>());

    // PHASE 1: Test if rename is legal. _SEH2_TRY / _SEH2_FINALLY
    let p_continue_with_rename = core::ptr::addr_of_mut!(ContinueWithRename);
    let p_target_dirent_bcb = core::ptr::addr_of_mut!(TargetDirentBcb);
    crate::seh::try_finally(&mut (), |_| {
        // Check if root directory
        if (*Fcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB as i16 {
            Status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        // Check Dcb with open handles
        if (*Fcb).Header.NodeTypeCode == FAT_NTC_DCB as i16 {
            let mut BatchOplockFcb: *mut FCB;
            let mut BatchOplockCount: u32;

            loop {
                BatchOplockFcb = core::ptr::null_mut();
                BatchOplockCount = 0;

                TempFcb = FatGetNextFcbBottomUp(IrpContext, core::ptr::null_mut(), Fcb);
                while TempFcb != Fcb {
                    if (*TempFcb).UncleanCount != 0 {
                        if FatIsFileOplockable(TempFcb) != 0
                            && (FsRtlCurrentBatchOplock(FatGetFcbOplock(TempFcb)) != 0
                                || FsRtlCurrentOplockH(FatGetFcbOplock(TempFcb)) != 0)
                        {
                            BatchOplockCount += 1;
                            if BatchOplockFcb.is_null() {
                                BatchOplockFcb = TempFcb;
                            }
                        } else {
                            Status = STATUS_ACCESS_DENIED;
                            return; // try_return
                        }
                    }
                    TempFcb = FatGetNextFcbBottomUp(IrpContext, TempFcb, Fcb);
                }

                if !BatchOplockFcb.is_null() {
                    if (*Irp).IoStatus.Information != 0
                        && BatchOplockCount as usize >= (*Irp).IoStatus.Information as usize
                    {
                        Status = STATUS_ACCESS_DENIED;
                        return; // try_return
                    }

                    (*Irp).IoStatus.Information = BatchOplockCount as _;
                    Status = FsRtlCheckOplock(
                        FatGetFcbOplock(BatchOplockFcb),
                        Irp,
                        IrpContext as *mut core::ffi::c_void,
                        Some(unsafe { core::mem::transmute::<*const (), _>(FatOplockComplete as *const ()) }),
                        None,
                    );
                    if Status == STATUS_SUCCESS {
                        continue;
                    }
                    Status = STATUS_PENDING;
                    return; // try_return
                }
                break;
            }

            // Purge referenced file objects
            FatPurgeReferencedFileObjects(IrpContext, Fcb, 1 /* FAT_FLUSH_TYPE Flush; was 0x466C7573, ASCII "Flus" */);

            // Remove full file name and exact case lfn from child Fcbs
            TempFcb = FatGetNextFcbBottomUp(IrpContext, core::ptr::null_mut(), Fcb);
            while TempFcb != Fcb {
                FatAcquireExclusiveFcb(IrpContext, TempFcb);
                if !(*TempFcb).FullFileName.Buffer.is_null() {
                    ExFreePool((*TempFcb).FullFileName.Buffer as *mut c_void);
                    (*TempFcb).FullFileName.Buffer = core::ptr::null_mut();
                }
                FatReleaseFcb(IrpContext, TempFcb);
                TempFcb = FatGetNextFcbBottomUp(IrpContext, TempFcb, Fcb);
            }
        }

        // Determine if simple or fully-qualified rename
        if TargetFileObject.is_null() {
            let Buffer: *mut FILE_RENAME_INFORMATION = (*Irp).AssociatedIrp.SystemBuffer as *mut FILE_RENAME_INFORMATION;
            TargetDcb = (*Fcb).ParentDcb;
            NewName.Length = ((*Buffer).FileNameLength) as u16;
            NewName.Buffer = &raw mut (*Buffer).FileName as *mut u16;
            if NewName.Length > 255 * (core::mem::size_of::<u16>() as u16) {
                Status = STATUS_OBJECT_NAME_INVALID;
                return; // try_return
            }
            RtlCopyUnicodeString(&mut NewNameCopy as *mut UNICODE_STRING as *mut _UNICODE_STRING, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING);
        } else {
            let mut TargetVcb: *mut VCB = core::ptr::null_mut();

            let mut TargetCcb: *mut CCB = core::ptr::null_mut();
            if FatDecodeFileObject(TargetFileObject, &raw mut TargetVcb, &raw mut TargetDcb, &raw mut TargetCcb) != USER_DIRECTORY_OPEN
                || TargetVcb != Vcb
            {
                Status = STATUS_INVALID_PARAMETER;
                return; // try_return
            }
            // C code: NewName = *((PUNICODE_STRING)&TargetFileObject->FileName)
            let ptr = &raw const (*TargetFileObject).FileName as *const UNICODE_STRING;
            NewName = core::ptr::read(ptr);
            RtlCopyUnicodeString(&mut NewNameCopy as *mut UNICODE_STRING as *mut _UNICODE_STRING, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING);
        }

        // Upcase versions
        Status = RtlUpcaseUnicodeString(&mut NewUpcasedName as *mut UNICODE_STRING as *mut _UNICODE_STRING, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING, 0);
        if !NT_SUCCESS(Status) {
            return; // try_return
        }
        FatGetUnicodeNameFromFcb(IrpContext, Fcb, &mut OldName as *mut UNICODE_STRING as *mut _UNICODE_STRING);
        Status = RtlUpcaseUnicodeString(&mut OldUpcasedName as *mut UNICODE_STRING as *mut _UNICODE_STRING, &OldName as *const UNICODE_STRING as *const _UNICODE_STRING, 0);
        if !NT_SUCCESS(Status) {
            return; // try_return
        }

        // Check if same directory and names equal
        if TargetDcb == (*Fcb).ParentDcb {
            if FsRtlAreNamesEqual(&NewName as *const UNICODE_STRING as *const _UNICODE_STRING, &OldName as *const UNICODE_STRING as *const _UNICODE_STRING, 0, core::ptr::null()) != 0 {
                Status = STATUS_SUCCESS;
                return; // try_return
            }
            if FsRtlAreNamesEqual(&NewUpcasedName as *const UNICODE_STRING as *const _UNICODE_STRING, &OldUpcasedName as *const UNICODE_STRING as *const _UNICODE_STRING, 0, core::ptr::null()) != 0 {
                CaseOnlyRename = 1;
            }
        } else {
            RenamedAcrossDirectories = 1;
        }

        // Convert new name to Oem
        if NewName.Length <= 12 * (core::mem::size_of::<u16>() as u16) {
            FatUnicodeToUpcaseOem(IrpContext, &raw mut NewOemName, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING);
            if FatSpaceInName(IrpContext, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING) != 0
                || FatIsNameShortOemValid(IrpContext, NewOemName, 0, 0, 0) == 0
            {
                NewOemName.Length = 0;
            }
        } else {
            NewOemName.Length = 0;
        }

        // Tunnel cache lookup
        TunneledDataSize = core::mem::size_of::<LARGE_INTEGER>() as u32;
        HaveTunneledInformation = FsRtlFindInTunnelCache(
            core::ptr::addr_of!((*Vcb).Tunnel) as *mut _,
            FatDirectoryKey(TargetDcb),
            &NewName as *const UNICODE_STRING as *mut _UNICODE_STRING,
            &raw mut UniTunneledShortName as *mut UNICODE_STRING as *mut _UNICODE_STRING,
            &raw mut UniTunneledLongName as *mut UNICODE_STRING as *mut _UNICODE_STRING,
            &raw mut TunneledDataSize,
            &raw mut TunneledCreationTime as *mut _ as *mut c_void,
        );
        // NT_ASSERT: skip

        // Determine dirents required
        if NewOemName.Length == 0
            || {
                // FatEvaluateNameCase returns VOID (namesup.rs:611, and VOID upstream) — its
                // answer comes back through CreateLfn. The local extern decl claimed
                // `-> BOOLEAN`, so `let r = ...` read whatever happened to be in the return
                // register and `r != 0 ||` made this branch effectively RANDOM: it decides
                // whether a rename allocates long-file-name dirents. Found by
                // tools/scan_extern_arity.py; matches the C, which tests CreateLfn alone.
                FatEvaluateNameCase(
                    IrpContext,
                    &NewNameCopy as *const UNICODE_STRING as *const _UNICODE_STRING,
                    &raw mut AllLowerComponent,
                    &raw mut AllLowerExtension,
                    &raw mut CreateLfn,
                );
                CreateLfn != 0
            }
        {
            DirentsRequired = FAT_LFN_DIRENTS_NEEDED(&NewNameCopy as *const UNICODE_STRING as *const _UNICODE_STRING) + 1;
        } else {
            if UniTunneledLongName.Length != 0
                && FatLfnDirentExists(IrpContext, TargetDcb, &UniTunneledLongName as *const UNICODE_STRING as *const _UNICODE_STRING, &raw mut TargetLfn as *mut UNICODE_STRING as *mut _UNICODE_STRING) == 0
            {
                UsingTunneledLfn = 1;
                CreateLfn = 1;
                DirentsRequired = FAT_LFN_DIRENTS_NEEDED(&UniTunneledLongName as *const UNICODE_STRING as *const _UNICODE_STRING) + 1;
            } else {
                DirentsRequired = 1;
            }
        }

        // Chicago mode checks
        if (*core::ptr::addr_of!(FatData)).ChicagoMode() == 0 {
            if NewOemName.Length == 0 {
                Status = STATUS_OBJECT_NAME_INVALID;
                return; // try_return
            }
            AllLowerComponent = 0;
            AllLowerExtension = 0;
            CreateLfn = 0;
            UsingTunneledLfn = 0;
        }

        if CaseOnlyRename == 0 {
            if NewOemName.Length != 0 {
                let ccb_inner = core::ptr::addr_of_mut!(LocalCcb.__bindgen_anon_1) as *mut _CCB__bindgen_ty_1__bindgen_ty_1;
                // bindgen renders OemQueryTemplate as a union, so
                // `.bindgen_union_field` is the same ADDRESS as the C's
                // `.Constant`; only the type needed correcting.
                let oem_qt_ptr = core::ptr::addr_of_mut!((*ccb_inner).OemQueryTemplate.bindgen_union_field) as *mut [u8; 11];
                FatStringTo8dot3(IrpContext, NewOemName, oem_qt_ptr);
            } else {
                LocalCcb.Flags |= CCB_FLAG_SKIP_SHORT_NAME_COMPARE; // SetFlag
            }
            {
                let ccb_inner = core::ptr::addr_of_mut!(LocalCcb.__bindgen_anon_1) as *mut _CCB__bindgen_ty_1__bindgen_ty_1;
                core::ptr::write(
                    core::ptr::addr_of_mut!((*ccb_inner).UnicodeQueryTemplate) as *mut UNICODE_STRING,
                    NewUpcasedName,
                );
            }
            LocalCcb.ContainsWildCards = 0;
            Flags = 0;
            FatLocateDirent(
                IrpContext,
                TargetDcb,
                &raw mut LocalCcb,
                0,
                &raw mut Flags,
                &raw mut TargetDirent,
                &raw mut *p_target_dirent_bcb,
                &raw mut TargetDirentOffset,
                core::ptr::null_mut(),
                &raw mut TargetLfn as *mut UNICODE_STRING as *mut _UNICODE_STRING,
                &raw mut TargetOrigLfn as *mut UNICODE_STRING as *mut _UNICODE_STRING,
            );
            if !TargetDirent.is_null() {
                if ReplaceIfExists == 0
                    || ((*TargetDirent).Attributes & FAT_DIRENT_ATTR_DIRECTORY) != 0
                    || ((*TargetDirent).Attributes & FAT_DIRENT_ATTR_READ_ONLY) != 0
                {
                    Status = STATUS_OBJECT_NAME_COLLISION;
                    return; // try_return
                }

                // Check target file for open handles
                let target_dcb_specific = core::ptr::addr_of_mut!((*TargetDcb).Specific) as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
                let mut l: *mut LIST_ENTRY = (*target_dcb_specific).ParentDcbQueue.Flink;
                while l != core::ptr::addr_of!((*target_dcb_specific).ParentDcbQueue) as *mut LIST_ENTRY {
                    TempFcb = (l as *mut u8).offset(-(core::mem::offset_of!(FCB, ParentDcbLinks) as isize)) as *mut FCB;
                    l = (*l).Flink;
                    if (*TempFcb).DirentOffsetWithinDirectory == TargetDirentOffset
                        && ((*TempFcb).UncleanCount != 0
                            || MmFlushImageSection(
                                core::ptr::addr_of!((*(*TempFcb).NonPaged).SectionObjectPointers),
                                MmFlushForDelete,
                            ) == 0)
                    {
                        Status = STATUS_ACCESS_DENIED;
                        if FatIsFileOplockable(TempFcb) != 0
                            && (FsRtlCurrentBatchOplock(FatGetFcbOplock(TempFcb)) != 0
                                || FsRtlCurrentOplockH(FatGetFcbOplock(TempFcb)) != 0)
                        {
                            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_target_dirent_bcb)) as *mut *mut core::ffi::c_void);
                            Status = FsRtlCheckOplock(
                                FatGetFcbOplock(TempFcb),
                                Irp,
                                IrpContext as *mut core::ffi::c_void,
                                Some(unsafe { core::mem::transmute::<*const (), _>(FatOplockComplete as *const ()) }),
                                None,
                            );
                            if Status != STATUS_PENDING {
                                Status = STATUS_ACCESS_DENIED;
                            }
                        }
                        return; // try_return
                    }
                }

                TargetLfnOffset = TargetDirentOffset
                    - FAT_LFN_DIRENTS_NEEDED(&TargetOrigLfn as *const UNICODE_STRING as *const _UNICODE_STRING) as u32
                        * core::mem::size_of::<DIRENT>() as u32;
                DeleteTarget = 1;
            }
        }

        // Allocate new dirents if needed
        if TargetDcb != (*Fcb).ParentDcb
            || DirentsRequired
                != (((*Fcb).DirentOffsetWithinDirectory - (*Fcb).LfnOffsetWithinDirectory)
                    / core::mem::size_of::<DIRENT>() as u32)
                    + 1
        {
            NewOffset = FatCreateNewDirent(IrpContext, TargetDcb, DirentsRequired, 0);
            DeleteSourceDirent = 1;
        } else {
            NewOffset = (*Fcb).LfnOffsetWithinDirectory;
        }

        *p_continue_with_rename = 1;
    }, |_, _abnormal| {
        if *p_continue_with_rename == 0 {
            ExFreePool(UnicodeBuffer as *mut c_void);
            FatUnpinBcb(IrpContext, p_target_dirent_bcb as *mut *mut core::ffi::c_void);
        }
    });

    if ContinueWithRename == 0 {
        return Status;
    }

    // PHASE 2: Actually perform the rename. _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_new_dirent_from_pool = core::ptr::addr_of_mut!(NewDirentFromPool);
    let p_new_dirent = core::ptr::addr_of_mut!(NewDirent);
    let p_dot_dot_bcb = core::ptr::addr_of_mut!(DotDotBcb);
    let p_directory_file_object = core::ptr::addr_of_mut!(DirectoryFileObject);
    let p_invalidate_fcb_on_raise = core::ptr::addr_of_mut!(InvalidateFcbOnRaise);
    let p_old_dirent_bcb = core::ptr::addr_of_mut!(OldDirentBcb);
    let p_new_dirent_bcb = core::ptr::addr_of_mut!(NewDirentBcb);
    let p_second_page_bcb = core::ptr::addr_of_mut!(SecondPageBcb);
    crate::seh::try_finally(&mut (), |_| {
        if RenamedAcrossDirectories == 0 && DeleteTarget == 0 {
            NotifyAction = FILE_ACTION_RENAMED_OLD_NAME;
        } else {
            NotifyAction = FILE_ACTION_REMOVED;
        }
        FatNotifyReportChange(
            IrpContext,
            Vcb,
            Fcb,
            if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB as i16 {
                // FILE_NOTIFY_CHANGE_FILE_NAME
                0x00000001
            } else {
                // FILE_NOTIFY_CHANGE_DIR_NAME
                0x00000002
            },
            NotifyAction,
        );

        // Nested _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            FatGetDirentFromFcbOrDcb(IrpContext, Fcb, 0, &raw mut OldDirent, &raw mut *p_old_dirent_bcb);
            Dirent = unsafe { core::ptr::read(OldDirent) };

            FatTunnelFcbOrDcb(Fcb, SourceCcb);
            *p_invalidate_fcb_on_raise = 1;

            if DeleteSourceDirent != 0 {
                FatDeleteDirent(IrpContext, Fcb, core::ptr::null_mut(), 0);
            }

            if DeleteTarget != 0 {
                FatDeleteFile(
                    IrpContext,
                    TargetDcb,
                    TargetLfnOffset,
                    TargetDirentOffset,
                    TargetDirent,
                    &mut TargetLfn,
                );
            }

            FatSelectNames(
                IrpContext,
                TargetDcb,
                &raw mut NewOemName,
                &NewNameCopy as *const UNICODE_STRING as *const _UNICODE_STRING,
                &raw mut NewOemName,
                if HaveTunneledInformation != 0 {
                    &raw mut UniTunneledShortName as *mut UNICODE_STRING as *const _UNICODE_STRING
                } else {
                    core::ptr::null()
                },
                &raw mut AllLowerComponent,
                &raw mut AllLowerExtension,
                &raw mut CreateLfn,
            );

            if CreateLfn == 0 && UsingTunneledLfn != 0 {
                CreateLfn = 1;
                RtlCopyUnicodeString(&mut NewNameCopy as *mut UNICODE_STRING as *mut _UNICODE_STRING, &UniTunneledLongName as *const UNICODE_STRING as *const _UNICODE_STRING);
                AllLowerComponent = 0;
                AllLowerExtension = 0;
            }

            // Prepare write directory file for new dirents
            let mut second_status: NTSTATUS = STATUS_SUCCESS;
            FatPrepareWriteDirectoryFile(
                IrpContext,
                TargetDcb,
                NewOffset,
                core::mem::size_of::<DIRENT>() as u32,
                &raw mut *p_new_dirent_bcb,
                &raw mut *p_new_dirent,
                0,
                1,
                &raw mut second_status,
            );
            // NT_ASSERT(NT_SUCCESS(second_status));

            // Check page boundary crossing
            if (NewOffset / PAGE_SIZE as u32)
                != ((NewOffset + (DirentsRequired - 1) * core::mem::size_of::<DIRENT>() as u32)
                    / PAGE_SIZE as u32)
            {
                SecondPageOffset = (NewOffset & !(PAGE_SIZE as u32 - 1)) + PAGE_SIZE as u32;
                BytesInFirstPage = SecondPageOffset - NewOffset;
                DirentsInFirstPage = BytesInFirstPage / core::mem::size_of::<DIRENT>() as u32;

                FatPrepareWriteDirectoryFile(
                    IrpContext,
                    TargetDcb,
                    SecondPageOffset,
                    core::mem::size_of::<DIRENT>() as u32,
                    &raw mut *p_second_page_bcb,
                    &raw mut SecondPageDirent,
                    0,
                    1,
                    &raw mut second_status,
                );
                // NT_ASSERT

                FirstPageDirent = *p_new_dirent;
                *p_new_dirent = FsRtlAllocatePoolWithTag(
                    PagedPool,
                    (DirentsRequired * core::mem::size_of::<DIRENT>() as u32) as u32,
                    TAG_DIRENT,
                ) as *mut DIRENT;
                *p_new_dirent_from_pool = 1;
            }

            ShortDirent = (*p_new_dirent).offset((DirentsRequired as isize) - 1);
            ShortDirentOffset = NewOffset + (DirentsRequired - 1) * core::mem::size_of::<DIRENT>() as u32;

            let dirent_attributes = Dirent.Attributes;
            core::ptr::write(ShortDirent, core::ptr::read(&Dirent)); // *ShortDirent = Dirent

            FatConstructDirent(
                IrpContext,
                ShortDirent,
                &NewOemName,
                AllLowerComponent,
                AllLowerExtension,
                if CreateLfn != 0 { &NewNameCopy as *const UNICODE_STRING as *const _UNICODE_STRING } else { core::ptr::null() },
                dirent_attributes as u16,
                0,
                if HaveTunneledInformation != 0 {
                    &TunneledCreationTime
                } else {
                    core::ptr::null()
                },
            );

            if HaveTunneledInformation != 0 {
                (*Fcb).CreationTime = FatFatTimeToNtTime(IrpContext, core::ptr::read(core::ptr::addr_of!((*ShortDirent).CreationTime)), (*ShortDirent).CreationMSec);
                (*Fcb).LastWriteTime = FatFatTimeToNtTime(IrpContext, core::ptr::read(core::ptr::addr_of!((*ShortDirent).LastWriteTime)), 0);
                (*Fcb).LastAccessTime = FatFatDateToNtTime(IrpContext, core::ptr::read(core::ptr::addr_of!((*ShortDirent).LastAccessDate)));
            }

            if *p_new_dirent_from_pool != 0 {
                core::ptr::copy_nonoverlapping(
                    *p_new_dirent as *const u8,
                    FirstPageDirent as *mut u8,
                    BytesInFirstPage as usize,
                );
                core::ptr::copy_nonoverlapping(
                    (*p_new_dirent).offset(DirentsInFirstPage as isize) as *const u8,
                    SecondPageDirent as *mut u8,
                    (DirentsRequired as usize * core::mem::size_of::<DIRENT>()) - BytesInFirstPage as usize,
                );
                ShortDirent = SecondPageDirent
                    .offset((DirentsRequired - DirentsInFirstPage) as isize - 1);
            }

            Dirent = unsafe { core::ptr::read(ShortDirent) };
        }, |_, _abnormal| {
            FatRemoveNames(IrpContext, Fcb);
            if !(*Fcb).FullFileName.Buffer.is_null() {
                ExFreePool((*Fcb).FullFileName.Buffer as *mut c_void);
                (*Fcb).FullFileName.Buffer = core::ptr::null_mut();
            }
            if !(*Fcb).ExactCaseLongName.Buffer.is_null() {
                ExFreePool((*Fcb).ExactCaseLongName.Buffer as *mut c_void);
                (*Fcb).ExactCaseLongName.Buffer = core::ptr::null_mut();
            }
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_old_dirent_bcb)) as *mut *mut core::ffi::c_void);
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_target_dirent_bcb)) as *mut *mut core::ffi::c_void);
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_new_dirent_bcb)) as *mut *mut core::ffi::c_void);
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_second_page_bcb)) as *mut *mut core::ffi::c_void);
        });

        // Update directory offsets
        (*Fcb).LfnOffsetWithinDirectory = NewOffset;
        (*Fcb).DirentOffsetWithinDirectory = ShortDirentOffset;

        RemoveEntryList(&raw mut (*Fcb).ParentDcbLinks);

        {
            let target_dcb_specific2 = core::ptr::addr_of_mut!((*TargetDcb).Specific) as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB as i16 {
                InsertTailList(
                    core::ptr::addr_of_mut!((*target_dcb_specific2).ParentDcbQueue),
                    &raw mut (*Fcb).ParentDcbLinks,
                );
            } else {
                InsertHeadList(
                    core::ptr::addr_of_mut!((*target_dcb_specific2).ParentDcbQueue),
                    &raw mut (*Fcb).ParentDcbLinks,
                );
            }
        }

        OldParentDcb = (*Fcb).ParentDcb;
        (*Fcb).ParentDcb = TargetDcb;

        // Break parent directory oplock on old parent (advisory)
        // #if (NTDDI_VERSION >= NTDDI_WIN8) -- see crate::NTDDI_WIN8
        if crate::NTDDI_WIN8 {
            FsRtlCheckOplockEx(
                FatGetFcbOplock(OldParentDcb),
                (*IrpContext).OriginatingIrp,
                OPLOCK_FLAG_PARENT_OBJECT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if RenamedAcrossDirectories != 0 {
            // #if (NTDDI_VERSION >= NTDDI_WIN8) -- see crate::NTDDI_WIN8
            if crate::NTDDI_WIN8 {
                FsRtlCheckOplockEx(
                    FatGetFcbOplock(TargetDcb),
                    (*IrpContext).OriginatingIrp,
                    OPLOCK_FLAG_PARENT_OBJECT,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }

            // Uninitialize cachemap for source directory if empty
            {
                let old_specific = core::ptr::addr_of_mut!((*OldParentDcb).Specific) as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
                if IsListEmpty(core::ptr::addr_of!((*old_specific).ParentDcbQueue))
                    && (*OldParentDcb).OpenCount == 0
                    && !(*old_specific).DirectoryFile.is_null()
                {
                    // NT_ASSERT
                    *p_directory_file_object = (*old_specific).DirectoryFile;
                    (*old_specific).DirectoryFile = core::ptr::null_mut();
                }
            }

            if (*Fcb).Header.NodeTypeCode == FAT_NTC_DCB as i16 {
                let mut io_status: NTSTATUS = STATUS_SUCCESS;
                FatPrepareWriteDirectoryFile(
                    IrpContext,
                    Fcb,
                    core::mem::size_of::<DIRENT>() as u32,
                    core::mem::size_of::<DIRENT>() as u32,
                    &raw mut *p_dot_dot_bcb,
                    &raw mut DotDotDirent,
                    0,
                    1,
                    &raw mut io_status,
                );
                // NT_ASSERT
                (*DotDotDirent).FirstClusterOfFile = if (*TargetDcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB as i16 {
                    0
                } else {
                    (*TargetDcb).FirstClusterOfFile as u16
                };
                if FatIsFat32(Vcb) != 0 {
                    (*DotDotDirent).__bindgen_anon_1.bindgen_union_field = if (*TargetDcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB as i16 {
                        0
                    } else {
                        ((*TargetDcb).FirstClusterOfFile >> 16) as u16
                    };
                }
            }
        }

        // Setup splay table and name in Fcb
        {
            let oem_str_sn = core::ptr::addr_of_mut!((*Fcb).ShortName.Name.bindgen_union_field) as *mut OEM_STRING;
            ExFreePool((*oem_str_sn).Buffer as *mut c_void);
            (*oem_str_sn).Buffer = core::ptr::null_mut();
        }

        FatConstructNamesInFcb(
            IrpContext,
            Fcb,
            &Dirent,
            if CreateLfn != 0 { &NewName as *const UNICODE_STRING as *const _UNICODE_STRING } else { core::ptr::null() },
        );
        FatSetFullNameInFcb(IrpContext, Fcb, &NewName as *const UNICODE_STRING as *const _UNICODE_STRING);

        *p_invalidate_fcb_on_raise = 0;

        if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB as i16 {
            (*FileObject).Flags |= FO_FILE_MODIFIED;
            (*Ccb).Flags |= CCB_FLAG_USER_SET_LAST_WRITE;
        }

        if DeleteTarget != 0 {
            NotifyAction = FILE_ACTION_MODIFIED;
            FatNotifyReportChange(
                IrpContext,
                Vcb,
                Fcb,
                // ATTRIBUTES|SIZE|LAST_WRITE|LAST_ACCESS|CREATION|EA (was 0x1F: FILE_NAME..LAST_WRITE)
                0x000000FC,
                FILE_ACTION_MODIFIED,
            );
        } else if RenamedAcrossDirectories != 0 {
            NotifyAction = FILE_ACTION_ADDED;
            FatNotifyReportChange(
                IrpContext,
                Vcb,
                Fcb,
                if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB as i16 {
                    0x00000001 // FILE_NOTIFY_CHANGE_FILE_NAME
                } else {
                    0x00000002 // FILE_NOTIFY_CHANGE_DIR_NAME
                },
                FILE_ACTION_ADDED,
            );
        } else {
            NotifyAction = FILE_ACTION_RENAMED_NEW_NAME;
            FatNotifyReportChange(
                IrpContext,
                Vcb,
                Fcb,
                if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB as i16 {
                    0x00000001
                } else {
                    0x00000002
                },
                FILE_ACTION_RENAMED_NEW_NAME,
            );
        }

        if FatIsFat32(Vcb) == 0 && Dirent.__bindgen_anon_1.bindgen_union_field != 0 {
            FatRenameEAs(IrpContext, Fcb, Dirent.__bindgen_anon_1.bindgen_union_field, &raw mut OldOemName);
        }

        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_dot_dot_bcb)) as *mut *mut core::ffi::c_void);
        FatUnpinRepinnedBcbs(IrpContext);

        *p_status = STATUS_SUCCESS;
    }, |_, abnormal| {
        DebugUnwind(FatSetRenameInfo as *const core::ffi::c_void);
        ExFreePool(UnicodeBuffer as *mut c_void);
        if UniTunneledLongName.Buffer != UniTunneledLongNameBuffer.as_ptr() as *mut u16 {
            ExFreePool(UniTunneledLongName.Buffer as *mut c_void);
        }
        if *p_new_dirent_from_pool != 0 {
            ExFreePool(*p_new_dirent as *mut c_void);
        }
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_target_dirent_bcb)) as *mut *mut core::ffi::c_void);
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*p_dot_dot_bcb)) as *mut *mut core::ffi::c_void);
        if !(*p_directory_file_object).is_null() {
            DebugTrace(0, Dbg, b"Uninitialize our parent Stream Cache Map\n\0".as_ptr() as *const i8, 0);
            CcUninitializeCacheMap(*p_directory_file_object, core::ptr::null_mut(), core::ptr::null_mut());
            ObDereferenceObject(*p_directory_file_object as *mut c_void);
        }
        if abnormal && *p_invalidate_fcb_on_raise != 0 {
            (*Fcb).FcbCondition = FcbBad;
        }
        DebugTrace(-1, Dbg, b"FatSetRenameInfo -> %08lx\n\0".as_ptr() as *const i8, *p_status);
    });

    Status
}

// Local consts matching the house pattern (flush.rs, verfysup.rs) — FcbCondition
// is compared as u32; STATUS_FILE_CLOSED is not in the shared constants module.
const FcbGood: u32 = 1;
const STATUS_FILE_CLOSED: NTSTATUS = 0xC0000128_u32 as i32;

// FatSetEndOfFileInfo — recovered from the dropped-at-translate stub (rustconv
// --fn, mechanical mode, 2026-07-16; C original fileinfo.c:4387). The two SEH
// blocks (outer TRY/FINALLY + inner AdvanceOnly TRY/FINALLY) follow the house
// idiom used across this file: `loop { … break }` for try_return/try_exit, the
// FINALLY cleanup inline after, `_SEH2_AbnormalTermination()` gating the size
// unwind (always 0 under DUMMY_PSEH, like every other fastfat fn here). Fixes the
// arm64 BUGCHECK 0x23 0xdead0000 that hit when eventlog.exe (SetEndOfFile on the
// FAT32 ramdisk) reached the rust_cohab_stub.c bugcheck stub.
#[no_mangle]
pub unsafe extern "system" fn FatSetEndOfFileInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    FileObject: *mut FILE_OBJECT,
    Vcb: *mut VCB,
    Fcb: *mut FCB,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Buffer: PFILE_END_OF_FILE_INFORMATION = core::ptr::null_mut();
    let mut NewFileSize: u32 = 0;
    let mut InitialFileSize: u32 = 0;
    let mut InitialValidDataLength: u32 = 0;
    let mut InitialValidDataToDisk: u32 = 0;
    let mut CacheMapInitialized: i32 = 0;
    let mut UnwindFileSizes: i32 = 0;
    let mut ResourceAcquired: i32 = 0;

    DebugTrace(1, Dbg, b"FatSetEndOfFileInfo...\n\0".as_ptr() as *const i8, 0);

    Buffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_END_OF_FILE_INFORMATION;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_new_file_size = core::ptr::addr_of_mut!(NewFileSize);
    let p_initial_file_size = core::ptr::addr_of_mut!(InitialFileSize);
    let p_initial_valid_data_length = core::ptr::addr_of_mut!(InitialValidDataLength);
    let p_initial_valid_data_to_disk = core::ptr::addr_of_mut!(InitialValidDataToDisk);
    let p_cache_map_initialized = core::ptr::addr_of_mut!(CacheMapInitialized);
    let p_unwind_file_sizes = core::ptr::addr_of_mut!(UnwindFileSizes);
    let p_resource_acquired = core::ptr::addr_of_mut!(ResourceAcquired);
    crate::seh::try_finally(&mut (), |_| {
        'try_exit: {
            if NodeType(Fcb) != FAT_NTC_FCB {
                DebugTrace(0, Dbg, b"Cannot change size of a directory\n\0".as_ptr() as *const i8, 0);
                *p_status = STATUS_INVALID_DEVICE_REQUEST;
                break 'try_exit;
            }

            if FatIsIoRangeValid((*Fcb).Vcb, (*Buffer).EndOfFile, 0) == 0 {
                DebugTrace(0, Dbg, b"Illegal allocation size\n\0".as_ptr() as *const i8, 0);
                *p_status = STATUS_DISK_FULL;
                break 'try_exit;
            }

            *p_new_file_size = (*Buffer).EndOfFile.__bindgen_anon_1.LowPart;

            if (*Fcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT
                && (*Fcb).FcbCondition == FcbGood
            {
                FatLookupFileAllocationSize(IrpContext, Fcb);
            }

            if (*(*FileObject).SectionObjectPointer).DataSectionObject != core::ptr::null_mut()
                && (*(*FileObject).SectionObjectPointer).SharedCacheMap == core::ptr::null_mut()
                && FlagOn((*Irp).Flags, IRP_PAGING_IO) == 0
            {
                if FlagOn((*FileObject).Flags, FO_CLEANUP_COMPLETE) != 0 {
                    FatRaiseStatus(IrpContext, STATUS_FILE_CLOSED);
                }

                FatInitializeCacheMap(
                    FileObject,
                    core::ptr::addr_of_mut!((*Fcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                    0,
                    &raw const FatData.CacheManagerCallbacks,
                    Fcb,
                );

                *p_cache_map_initialized = 1;
            }

            if (*IoGetCurrentIrpStackLocation(Irp)).Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.AdvanceOnly != 0 {
                if IsFileDeleted(IrpContext, Fcb) == 0 && (*Fcb).FcbCondition == FcbGood {
                    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
                    let mut DirentBcb: *mut BCB = core::ptr::null_mut();

                    if *p_new_file_size >= (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart {
                        *p_new_file_size = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                    }

                    NT_ASSERT((*p_new_file_size <= (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart) as i32);

                    FatGetDirentFromFcbOrDcb(
                        IrpContext,
                        Fcb,
                        0u8,
                        core::ptr::addr_of_mut!(Dirent),
                        (core::ptr::addr_of_mut!(DirentBcb)).cast(),
                    );

                    // _SEH2_TRY / _SEH2_FINALLY
                    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
                    crate::seh::try_finally(&mut (), |_| {
                        if *p_new_file_size > (*Dirent).FileSize {
                            (*Dirent).FileSize = *p_new_file_size;

                            FatSetDirtyBcb(IrpContext, *p_dirent_bcb, (*Fcb).Vcb, 1);

                            FatNotifyReportChange(
                                IrpContext,
                                Vcb,
                                Fcb,
                                FILE_NOTIFY_CHANGE_SIZE,
                                FILE_ACTION_MODIFIED,
                            );
                        }
                    }, |_, _abnormal| {
                        FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
                    });
                } else {
                    DebugTrace(0, Dbg, b"Cannot set size on deleted file.\n\0".as_ptr() as *const i8, 0);
                }

                *p_status = STATUS_SUCCESS;
                break 'try_exit;
            }

            if *p_new_file_size > (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
                FatAddFileAllocation(IrpContext, Fcb, FileObject, *p_new_file_size);
            }

            if (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart != *p_new_file_size {
                if *p_new_file_size < (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart {
                    if (*Fcb).PurgeFailureModeEnableCount != 0 {
                        *p_status = STATUS_PURGE_FAILED;
                        break 'try_exit;
                    }

                    if MmCanFileBeTruncated(
                        (*FileObject).SectionObjectPointer,
                        core::ptr::addr_of_mut!((*Buffer).EndOfFile),
                    ) == 0
                    {
                        *p_status = STATUS_USER_MAPPED_FILE;
                        break 'try_exit;
                    }

                    *p_resource_acquired = ExAcquireResourceExclusiveLite((*Fcb).Header.PagingIoResource, 1) as i32;
                }

                *p_initial_file_size = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                *p_initial_valid_data_length = (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart;
                *p_initial_valid_data_to_disk = (*Fcb).ValidDataToDisk;
                *p_unwind_file_sizes = 1;

                (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = *p_new_file_size;

                if (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart > *p_new_file_size {
                    (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = *p_new_file_size;
                }

                if (*Fcb).ValidDataToDisk > *p_new_file_size {
                    (*Fcb).ValidDataToDisk = *p_new_file_size;
                }

                DebugTrace(0, Dbg, b"New file size is 0x%08lx.\n\0".as_ptr() as *const i8, *p_new_file_size as i32);

                CcSetFileSizes(
                    FileObject,
                    core::ptr::addr_of_mut!((*Fcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                );

                FatSetFileSizeInDirent(IrpContext, Fcb, core::ptr::null_mut());

                FatNotifyReportChange(
                    IrpContext,
                    Vcb,
                    Fcb,
                    FILE_NOTIFY_CHANGE_SIZE,
                    FILE_ACTION_MODIFIED,
                );

                SetFlag(core::ptr::addr_of_mut!((*Fcb).FcbState), FCB_STATE_TRUNCATE_ON_CLOSE);
            }

            (*FileObject).Flags |= FO_FILE_MODIFIED;

            *p_status = STATUS_SUCCESS;
        }

        // try_exit: NOTHING; -- still inside the try
        FatUnpinRepinnedBcbs(IrpContext);
    }, |_, abnormal| {
        DebugUnwind(b"FatSetEndOfFileInfo\0".as_ptr() as *const core::ffi::c_void);

        if abnormal && *p_unwind_file_sizes != 0 {
            (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = *p_initial_file_size;
            (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = *p_initial_valid_data_length;
            (*Fcb).ValidDataToDisk = *p_initial_valid_data_to_disk;

            if (*(*FileObject).SectionObjectPointer).SharedCacheMap != core::ptr::null_mut() {
                *CcGetFileSizePointer(FileObject) = (*Fcb).Header.FileSize;
            }

            FatSetFileSizeInDirentNoRaise(IrpContext, Fcb, core::ptr::null_mut());
        }

        if *p_cache_map_initialized != 0 {
            CcUninitializeCacheMap(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
        }

        if *p_resource_acquired != 0 {
            ExReleaseResourceLite((*Fcb).Header.PagingIoResource);
        }

        DebugTrace(-1, Dbg, b"FatSetEndOfFileInfo -> %08lx\n\0".as_ptr() as *const i8, *p_status);
    });

    Status
}



// Forward declarations for unknown functions


// Forward declarations for functions/globals/constants not in the import list





// Forward declarations for external symbols not in imports

#[no_mangle]
pub unsafe extern "system" fn FatFsdSetInformation(
    VolumeDeviceObject: *mut core::ffi::c_void,
    Irp: *mut core::ffi::c_void,
) -> i32 {
    let mut Status: i32 = 0;
    let mut IrpContext: *mut core::ffi::c_void = core::ptr::null_mut();
    let TopLevel: i32;

    let _ = PAGED_CODE;

    DebugTrace(
        1,
        0x0000_1000, // Dbg (DEBUG_TRACE_FILEINFO)
        b"FatFsdSetInformation\n\0".as_ptr() as *const i8,
        0i32,
    );

    FsRtlEnterFileSystem();

    TopLevel = FatIsIrpTopLevel(Irp) as i32;

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const *mut core::ffi::c_void, || {
        *p_irp_context = FatCreateIrpContext(Irp, CanFsdWait(Irp));
        *p_status = FatCommonSetInformation(*p_irp_context as *mut _, Irp as *mut _);
    }) {
        Status = FatProcessException(IrpContext, Irp, code as u32);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();

    DebugTrace(
        -1i32,
        0x0000_1000, // Dbg (DEBUG_TRACE_FILEINFO)
        b"FatFsdSetInformation -> %08lx\n\0".as_ptr() as *const i8,
        Status as u32,
    );

    // UNREFERENCED_PARAMETER(VolumeDeviceObject);
    let _ = &VolumeDeviceObject;

    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatCommonQueryInformation(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE(); -- comment
    let mut Status: NTSTATUS;
    let IrpSp: PIO_STACK_LOCATION;
    let FileObject: PFILE_OBJECT;
    let mut Length: LONG;
    let FileInformationClass: FILE_INFORMATION_CLASS;
    let Buffer: *mut core::ffi::c_void;
    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut FcbAcquired: BOOLEAN = 0;
    let mut VcbAcquired: BOOLEAN = 0;

    IrpSp = IoGetCurrentIrpStackLocation(Irp);
    FileObject = (*IrpSp).FileObject;

    DebugTrace(1, Dbg, b"FatCommonQueryInformation...\n\0".as_ptr() as *const i8);
    DebugTrace(0, Dbg, b"Irp                    = %p\n\0".as_ptr() as *const i8, Irp as *mut core::ffi::c_void);
    DebugTrace(
        0,
        Dbg,
        b"->Length               = %08lx\n\0".as_ptr() as *const i8,
        (*IrpSp).Parameters.QueryFile.Length as u32,
    );
    DebugTrace(
        0,
        Dbg,
        b"->FileInformationClass = %08lx\n\0".as_ptr() as *const i8,
        (*IrpSp).Parameters.QueryFile.FileInformationClass,
    );
    DebugTrace(
        0,
        Dbg,
        b"->Buffer               = %p\n\0".as_ptr() as *const i8,
        (*Irp).AssociatedIrp.SystemBuffer,
    );

    Length = (*IrpSp).Parameters.QueryFile.Length as LONG;
    FileInformationClass = (*IrpSp).Parameters.QueryFile.FileInformationClass;
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    TypeOfOpen = FatDecodeFileObject(
        FileObject,
        core::ptr::addr_of_mut!(Vcb),
        core::ptr::addr_of_mut!(Fcb),
        core::ptr::addr_of_mut!(Ccb),
    );

    Status = STATUS_SUCCESS;

    let mut IrpContext = IrpContext;
    let mut Irp = Irp;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_length = core::ptr::addr_of_mut!(Length);
    let p_fcb_acquired = core::ptr::addr_of_mut!(FcbAcquired);
    let p_vcb_acquired = core::ptr::addr_of_mut!(VcbAcquired);
    crate::seh::try_finally(&mut (), |_| {
        if TypeOfOpen == UserVolumeOpen {
            *p_status = STATUS_INVALID_PARAMETER;
        } else if TypeOfOpen == UserFileOpen || TypeOfOpen == UserDirectoryOpen || TypeOfOpen == DirectoryFileOpen {
            if FileInformationClass == FileNameInformation ||
                FileInformationClass == FileNormalizedNameInformation ||
                FileInformationClass == FileAllInformation
            {
                if FatAcquireExclusiveVcb(*p_irp_context, Vcb) == 0 {
                    DebugTrace(0, Dbg, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8);
                    *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                    *p_irp_context = core::ptr::null_mut();
                    *p_irp = core::ptr::null_mut();
                    return; // try_return( Status )
                }
                *p_vcb_acquired = 1;
            }

            if FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0
                || FlagOn((*(*Fcb).Vcb).VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
            {
                if FatAcquireSharedFcb(*p_irp_context, Fcb) == 0 {
                    DebugTrace(0, Dbg, b"Cannot acquire Fcb\n\0".as_ptr() as *const i8);
                    *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                    *p_irp_context = core::ptr::null_mut();
                    *p_irp = core::ptr::null_mut();
                    return; // try_return( Status )
                }
                *p_fcb_acquired = 1;
            }

            FatVerifyFcb(*p_irp_context, Fcb);

            match FileInformationClass {
                FileAllInformation => {
                    let AllInfo = Buffer as *mut FILE_ALL_INFORMATION;
                    *p_length -= (core::mem::size_of::<FILE_ACCESS_INFORMATION>()
                        + core::mem::size_of::<FILE_MODE_INFORMATION>()
                        + core::mem::size_of::<FILE_ALIGNMENT_INFORMATION>()) as LONG;

                    FatQueryBasicInfo(
                        *p_irp_context,
                        Fcb,
                        FileObject,
                        core::ptr::addr_of_mut!((*AllInfo).BasicInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                    FatQueryStandardInfo(
                        *p_irp_context,
                        Fcb,
                        core::ptr::addr_of_mut!((*AllInfo).StandardInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                    FatQueryInternalInfo(
                        *p_irp_context,
                        Fcb,
                        core::ptr::addr_of_mut!((*AllInfo).InternalInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                    FatQueryEaInfo(
                        *p_irp_context,
                        Fcb,
                        core::ptr::addr_of_mut!((*AllInfo).EaInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                    FatQueryPositionInfo(
                        *p_irp_context,
                        FileObject,
                        core::ptr::addr_of_mut!((*AllInfo).PositionInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                    FatQueryNameInfo(
                        *p_irp_context,
                        Fcb,
                        Ccb,
                        0u8,
                        core::ptr::addr_of_mut!((*AllInfo).NameInformation),
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileBasicInformation => {
                    FatQueryBasicInfo(
                        *p_irp_context,
                        Fcb,
                        FileObject,
                        Buffer as *mut FILE_BASIC_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileStandardInformation => {
                    FatQueryStandardInfo(
                        *p_irp_context,
                        Fcb,
                        Buffer as *mut FILE_STANDARD_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileInternalInformation => {
                    FatQueryInternalInfo(
                        *p_irp_context,
                        Fcb,
                        Buffer as *mut FILE_INTERNAL_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileEaInformation => {
                    FatQueryEaInfo(
                        *p_irp_context,
                        Fcb,
                        Buffer as *mut FILE_EA_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FilePositionInformation => {
                    FatQueryPositionInfo(
                        *p_irp_context,
                        FileObject,
                        Buffer as *mut FILE_POSITION_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileNameInformation => {
                    FatQueryNameInfo(
                        *p_irp_context,
                        Fcb,
                        Ccb,
                        0u8,
                        Buffer as *mut FILE_NAME_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileNormalizedNameInformation => {
                    FatQueryNameInfo(
                        *p_irp_context,
                        Fcb,
                        Ccb,
                        1u8,
                        Buffer as *mut FILE_NAME_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileAlternateNameInformation => {
                    FatQueryShortNameInfo(
                        *p_irp_context,
                        Fcb,
                        Buffer as *mut FILE_NAME_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                FileNetworkOpenInformation => {
                    FatQueryNetworkInfo(
                        *p_irp_context,
                        Fcb,
                        FileObject,
                        Buffer as *mut FILE_NETWORK_OPEN_INFORMATION,
                        core::ptr::addr_of_mut!(*p_length),
                    );
                }
                _ => {
                    *p_status = STATUS_INVALID_PARAMETER;
                }
            }
        } else {
            KdPrintEx(
                DPFLTR_FASTFAT_ID,
                DPFLTR_INFO_LEVEL,
                b"FATQueryFile, Illegal TypeOfOpen = %08lx\n\0".as_ptr() as *const i8,
                TypeOfOpen as u32,
            );
            *p_status = STATUS_INVALID_PARAMETER;
        }

        if *p_length < 0 {
            *p_status = STATUS_BUFFER_OVERFLOW;
            *p_length = 0;
        }
        (**p_irp).IoStatus.Information = ((*IrpSp).Parameters.QueryFile.Length as i64 - *p_length as i64) as _;
    }, |_, abnormal| {
        DebugUnwind(b"FatCommonQueryInformation\0".as_ptr() as *const core::ffi::c_void);

        if *p_fcb_acquired != 0 {
            FatReleaseFcb(*p_irp_context, Fcb);
        }
        if *p_vcb_acquired != 0 {
            FatReleaseVcb(*p_irp_context, Vcb);
        }

        if !abnormal {
            FatCompleteRequest(*p_irp_context, *p_irp, *p_status);
        }

        DebugTrace(
            -1,
            Dbg,
            b"FatCommonQueryInformation -> %08lx\n\0".as_ptr() as *const i8,
            *p_status as u32,
        );
    });

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonSetInformation(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpContext = IrpContext;
    let mut Irp = Irp;

    // PAGED_CODE();

    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatCommonSetInformation...\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"Irp                    = %p\n\0".as_ptr() as *const i8, Irp as usize);
    DebugTrace(0, Dbg, b"->Length               = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.SetFile.Length as usize);
    DebugTrace(0, Dbg, b"->FileInformationClass = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.SetFile.FileInformationClass as usize);
    DebugTrace(0, Dbg, b"->FileObject           = %p\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.SetFile.FileObject as usize);
    DebugTrace(0, Dbg, b"->ReplaceIfExists      = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.ReplaceIfExists as usize);
    DebugTrace(0, Dbg, b"->Buffer               = %p\n\0".as_ptr() as *const i8, (*Irp).AssociatedIrp.SystemBuffer as usize);

    let FileInformationClass: FILE_INFORMATION_CLASS = (*IrpSp).Parameters.SetFile.FileInformationClass;
    let FileObject: *mut FILE_OBJECT = (*IrpSp).FileObject;

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    let TypeOfOpen: TYPE_OF_OPEN = FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb);

    let mut VcbAcquired: BOOLEAN = 0;
    let mut FcbAcquired: BOOLEAN = 0;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_fcb_acquired = core::ptr::addr_of_mut!(FcbAcquired);
    let p_vcb_acquired = core::ptr::addr_of_mut!(VcbAcquired);
    crate::seh::try_finally(&mut (), |_| {
        if TypeOfOpen == UserVolumeOpen {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        } else if TypeOfOpen == UserFileOpen {
            if ((*Fcb).FcbState & FCB_STATE_PAGING_FILE) == 0
                && (FileInformationClass == FileEndOfFileInformation
                    || FileInformationClass == FileAllocationInformation
                    || FileInformationClass == FileValidDataLengthInformation)
            {
                *p_status = FsRtlCheckOplock(
                    FatGetFcbOplock(Fcb),
                    *p_irp,
                    *p_irp_context as *mut core::ffi::c_void,
                    None,
                    None,
                );
                if *p_status != STATUS_SUCCESS {
                    return; // try_return
                }
                (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);
            }
        } else if TypeOfOpen == UserDirectoryOpen {
            // nothing
        } else {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        if NodeType(Fcb) == FAT_NTC_ROOT_DCB {
            if FileInformationClass == FileDispositionInformation {
                *p_status = STATUS_CANNOT_DELETE;
                return; // try_return
            }
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        if FileInformationClass == FileDispositionInformation
            || FileInformationClass == FileRenameInformation
        {
            if FatAcquireExclusiveVcb(*p_irp_context, Vcb) == 0 {
                DebugTrace(0, Dbg, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8, 0);
                *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                *p_irp = core::ptr::null_mut();
                *p_irp_context = core::ptr::null_mut();
                return; // try_return
            }
            *p_vcb_acquired = 1;
            if ((*Vcb).VcbState & VCB_STATE_FLAG_CREATE_IN_PROGRESS) != 0 {
                FatBugCheck(0, 0, 0);
            }
        }

        if ((*Fcb).FcbState & FCB_STATE_PAGING_FILE) == 0
            || ((*(*Fcb).Vcb).VcbState & VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
        {
            if FatAcquireExclusiveFcb(*p_irp_context, Fcb) == 0 {
                DebugTrace(0, Dbg, b"Cannot acquire Fcb\n\0".as_ptr() as *const i8, 0);
                *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                *p_irp = core::ptr::null_mut();
                *p_irp_context = core::ptr::null_mut();
                return; // try_return
            }
            *p_fcb_acquired = 1;
        }

        *p_status = STATUS_SUCCESS;
        FatVerifyFcb(*p_irp_context, Fcb);

        if FatIsFileOplockable(Fcb) != 0
            && (FileInformationClass == FileRenameInformation
                || (FileInformationClass == FileDispositionInformation
                    && (*((**p_irp).AssociatedIrp.SystemBuffer as *mut FILE_DISPOSITION_INFORMATION))
                        .DeleteFile
                        != 0))
        {
            *p_status = FsRtlCheckOplock(
                FatGetFcbOplock(Fcb),
                *p_irp,
                *p_irp_context as *mut core::ffi::c_void,
                Some(unsafe { core::mem::transmute::<*const (), _>(FatOplockComplete as *const ()) }),
                None,
            );
            (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);
            if *p_status == STATUS_PENDING {
                *p_irp = core::ptr::null_mut();
                *p_irp_context = core::ptr::null_mut();
            }
            if *p_status < 0 || *p_status == STATUS_PENDING {
                return; // try_return
            }
        }

        match FileInformationClass {
            FileBasicInformation => {
                *p_status = FatSetBasicInfo(*p_irp_context, *p_irp, Fcb, Ccb);
            }
            FileDispositionInformation => {
                if ((*Vcb).VcbState & VCB_STATE_FLAG_DEFERRED_FLUSH) != 0
                    && ((**p_irp_context).Flags & IRP_CONTEXT_FLAG_WAIT) == 0
                {
                    *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                    *p_irp = core::ptr::null_mut();
                    *p_irp_context = core::ptr::null_mut();
                } else {
                    *p_status = FatSetDispositionInfo(*p_irp_context, *p_irp, FileObject, Fcb);
                }
            }
            FileRenameInformation => {
                if ((**p_irp_context).Flags & IRP_CONTEXT_FLAG_WAIT) == 0 {
                    *p_status = FatFsdPostRequest(*p_irp_context, *p_irp);
                    *p_irp = core::ptr::null_mut();
                    *p_irp_context = core::ptr::null_mut();
                } else {
                    *p_status = FatSetRenameInfo(*p_irp_context, *p_irp, Vcb, Fcb, Ccb);
                    if *p_status == STATUS_PENDING {
                        *p_irp = core::ptr::null_mut();
                        *p_irp_context = core::ptr::null_mut();
                    }
                }
            }
            FilePositionInformation => {
                *p_status = FatSetPositionInfo(*p_irp_context, *p_irp, FileObject);
            }
            FileLinkInformation => {
                *p_status = STATUS_INVALID_DEVICE_REQUEST;
            }
            FileAllocationInformation => {
                *p_status = FatSetAllocationInfo(*p_irp_context, *p_irp, Fcb, FileObject);
            }
            FileEndOfFileInformation => {
                *p_status = FatSetEndOfFileInfo(*p_irp_context, *p_irp, FileObject, Vcb, Fcb);
            }
            FileValidDataLengthInformation => {
                *p_status = FatSetValidDataLengthInfo(*p_irp_context, *p_irp, FileObject, Fcb, Ccb);
            }
            _ => {
                *p_status = STATUS_INVALID_PARAMETER;
            }
        }

        if !(*p_irp_context).is_null() {
            FatUnpinRepinnedBcbs(*p_irp_context);
        }
    }, |_, abnormal| {
        DebugUnwind(b"FatCommonSetInformation\0".as_ptr() as *const core::ffi::c_void);
        if *p_fcb_acquired != 0 {
            FatReleaseFcb(*p_irp_context, Fcb);
        }
        if *p_vcb_acquired != 0 {
            FatReleaseVcb(*p_irp_context, Vcb);
        }
        if !abnormal {
            FatCompleteRequest(*p_irp_context, *p_irp, *p_status);
        }
        DebugTrace(-1, Dbg, b"FatCommonSetInformation -> %08lx\n\0".as_ptr() as *const i8, *p_status as usize);
    });

    return Status;
}



// Forward declarations for unknown extern functions

// Constants for flags and attributes (guessed standard values)

// Time arithmetic constants (100 ns units)

// Global references (assumed extern)



// ==== re-stitched (genuine drop, authoritative) ====
#[no_mangle]
unsafe extern "system" fn FatDeleteFile(
    IrpContext: *mut IRP_CONTEXT,
    TargetDcb: *mut DCB,
    LfnOffset: u32,
    DirentOffset: u32,
    Dirent: *mut DIRENT,
    Lfn: *mut UNICODE_STRING,
) {
    // Forward declarations for unknown gap functions/macros
    extern "system" {
        fn PAGED_CODE();
        fn FatBugCheck(a: u32, b: u32, c: u32);
        fn FatCreateFcb(
            IrpContext: *mut IRP_CONTEXT,
            Vcb: *mut VCB,
            TargetDcb: *mut DCB,
            LfnOffset: u32,
            DirentOffset: u32,
            Dirent: *mut DIRENT,
            Lfn: *mut UNICODE_STRING,
            other: *mut UNICODE_STRING,
            flag1: u8,
            flag2: u8,
        ) -> *mut FCB;
        fn FatDeleteDirent(
            IrpContext: *mut IRP_CONTEXT,
            Fcb: *mut FCB,
            arg3: *mut core::ffi::c_void,
            flag: u8,
        );
        fn FatDeleteFcb(
            IrpContext: *mut IRP_CONTEXT,
            Fcb: *mut *mut FCB,
        );
        fn FatRemoveNames(
            IrpContext: *mut IRP_CONTEXT,
            Fcb: *mut FCB,
        );
        fn FatTruncateFileAllocation(
            IrpContext: *mut IRP_CONTEXT,
            Fcb: *mut FCB,
            offset: u32,
        );
        fn NT_ASSERT(condition: u32);
        fn NodeType(Fcb: *mut FCB) -> u32;
    }

    // Placeholder constants (values unknown, but required for compilation)
    const FCB_STATE_NAMES_IN_SPLAY_TREE: u32 = 0x0100;
    const FCB_STATE_DELETE_ON_CLOSE: u32 = 0x01;
    const FAT_NTC_FCB: u32 = 0x0502;

    PAGED_CODE();

    let mut Links: *mut LIST_ENTRY = (*TargetDcb).Specific.Dcb.as_mut().ParentDcbQueue.Flink;
    let queue_end = core::ptr::addr_of_mut!((*TargetDcb).Specific.Dcb.as_mut().ParentDcbQueue) as *mut LIST_ENTRY;

    while Links != queue_end {
        let offset = core::mem::offset_of!(FCB, ParentDcbLinks) as isize;
        let Fcb = (Links as *const u8).offset(-offset) as *mut FCB;

        if ((*Fcb).FcbState & FCB_STATE_NAMES_IN_SPLAY_TREE) != 0
            && (*Fcb).DirentOffsetWithinDirectory == DirentOffset
        {
            NT_ASSERT((NodeType(Fcb) == FAT_NTC_FCB) as u32);
            NT_ASSERT(((*Fcb).LfnOffsetWithinDirectory == LfnOffset) as u32);

            if (*Fcb).UncleanCount != 0 {
                FatBugCheck(0, 0, 0);
            } else {
                FatRemoveNames(IrpContext, Fcb);

                let Resource = (*Fcb).Header.PagingIoResource;
                ExAcquireResourceExclusiveLite(Resource, 1);

                (*Fcb).FcbState |= FCB_STATE_DELETE_ON_CLOSE;
                (*Fcb).ValidDataToDisk = 0;
                (*Fcb).Header.FileSize.QuadPart = 0;
                (*Fcb).Header.ValidDataLength.QuadPart = 0;
                (*Fcb).FirstClusterOfFile = 0;

                ExReleaseResourceLite(Resource);
            }
        }

        Links = (*Links).Flink;
    }

    let Fcb = FatCreateFcb(
        IrpContext,
        (*TargetDcb).Vcb,
        TargetDcb,
        LfnOffset,
        DirentOffset,
        Dirent,
        Lfn,
        core::ptr::null_mut(),
        0,
        0,
    );

    (*Fcb).Header.FileSize.u.LowPart = 0;

    // _SEH2_TRY / _SEH2_FINALLY
    let mut Fcb = Fcb;
    let p_fcb = core::ptr::addr_of_mut!(Fcb);
    crate::seh::try_finally(&mut (), |_| {
        FatTruncateFileAllocation(IrpContext, *p_fcb, 0);
        FatDeleteDirent(IrpContext, *p_fcb, core::ptr::null_mut(), 1);
    }, |_, _abnormal| {
        FatDeleteFcb(IrpContext, p_fcb);
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
//     FileInfo.c
//
// Abstract:
//
//     This module implements the File Information routines for Fat called by
//     the dispatch driver.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_FILEINFO)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_FILEINFO)
//
// VOID
// FatQueryBasicInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_BASIC_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryStandardInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_STANDARD_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// VOID
// FatQueryInternalInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_INTERNAL_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// VOID
// FatQueryEaInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_EA_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// VOID
// FatQueryPositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_POSITION_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryNameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PCCB Ccb,
//     IN BOOLEAN Normalized,
//     IN OUT PFILE_NAME_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// VOID
// FatQueryShortNameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_NAME_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryNetworkInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_NETWORK_OPEN_INFORMATION Buffer,
//     IN OUT PLONG Length
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetBasicInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetDispositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PFCB Fcb
//     );
//
// NTSTATUS
// FatSetRenameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PVCB Vcb,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     );
//
// NTSTATUS
// FatSetPositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetAllocationInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetEndOfFileInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PVCB Vcb,
//     IN PFCB Fcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetValidDataLengthInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatRenameEAs (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN USHORT ExtendedAttributes,
//     IN POEM_STRING OldOemName
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonQueryInformation)
// #pragma alloc_text(PAGE, FatCommonSetInformation)
// #pragma alloc_text(PAGE, FatFsdQueryInformation)
// #pragma alloc_text(PAGE, FatFsdSetInformation)
// #pragma alloc_text(PAGE, FatQueryBasicInfo)
// #pragma alloc_text(PAGE, FatQueryEaInfo)
// #pragma alloc_text(PAGE, FatQueryInternalInfo)
// #pragma alloc_text(PAGE, FatQueryNameInfo)
// #pragma alloc_text(PAGE, FatQueryNetworkInfo)
// #pragma alloc_text(PAGE, FatQueryShortNameInfo)
// #pragma alloc_text(PAGE, FatQueryPositionInfo)
// #pragma alloc_text(PAGE, FatQueryStandardInfo)
// #pragma alloc_text(PAGE, FatSetAllocationInfo)
// #pragma alloc_text(PAGE, FatSetBasicInfo)
// #pragma alloc_text(PAGE, FatSetDispositionInfo)
// #pragma alloc_text(PAGE, FatSetEndOfFileInfo)
// #pragma alloc_text(PAGE, FatSetValidDataLengthInfo)
// #pragma alloc_text(PAGE, FatSetPositionInfo)
// #pragma alloc_text(PAGE, FatSetRenameInfo)
// #pragma alloc_text(PAGE, FatDeleteFile)
// #pragma alloc_text(PAGE, FatRenameEAs)
// #endif
//
//
// _Function_class_(IRP_MJ_QUERY_INFORMATION)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdQueryInformation (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the Fsd part of the NtQueryInformationFile API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being queried exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatFsdQueryInformation\n", 0);
//
//     //
//     //  Call the common query routine, with blocking allowed if synchronous
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
//         Status = FatCommonQueryInformation( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdQueryInformation -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Function_class_(IRP_MJ_SET_INFORMATION)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdSetInformation (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of the NtSetInformationFile API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being set exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatFsdSetInformation\n", 0);
//
//     //
//     //  Call the common set routine, with blocking allowed if synchronous
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
//         Status = FatCommonSetInformation( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdSetInformation -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonQueryInformation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for querying file information called by both
//     the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//
//     LONG Length;
//     FILE_INFORMATION_CLASS FileInformationClass;
//     PVOID Buffer;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN FcbAcquired = FALSE;
//     BOOLEAN VcbAcquired = FALSE;
//
//     PFILE_ALL_INFORMATION AllInfo;
//
//     let _ = PAGED_CODE;
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     FileObject = IrpSp->FileObject;
//
//     DebugTrace(+1, Dbg, "FatCommonQueryInformation...\n", 0);
//     DebugTrace( 0, Dbg, "Irp                    = %p\n", Irp);
//     DebugTrace( 0, Dbg, "->Length               = %08lx\n", IrpSp->Parameters.QueryFile.Length);
//     DebugTrace( 0, Dbg, "->FileInformationClass = %08lx\n", IrpSp->Parameters.QueryFile.FileInformationClass);
//     DebugTrace( 0, Dbg, "->Buffer               = %p\n", Irp->AssociatedIrp.SystemBuffer);
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     Length = (LONG)IrpSp->Parameters.QueryFile.Length;
//     FileInformationClass = IrpSp->Parameters.QueryFile.FileInformationClass;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     Status = STATUS_SUCCESS;
//
//     _SEH2_TRY {
//
//         //
//         //  Case on the type of open we're dealing with
//         //
//
//         switch (TypeOfOpen) {
//
//         case UserVolumeOpen:
//
//             //
//             //  We cannot query the user volume open.
//             //
//
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//
//         case UserFileOpen:
//         case UserDirectoryOpen:
//         case DirectoryFile:
//
//
//             //
//             //  NameInfo requires synchronization with deletion in order to perform
//             //  the full filename query.  A lighter-weight way to do this would be per
//             //  directory as the full name is built up and since the multiple Fcb
//             //  lockorder is bottom up, this is conceivable.  At this time, though,
//             //  this change is safer.
//             //
//
//             if (FileInformationClass == FileNameInformation ||
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//                 FileInformationClass == FileNormalizedNameInformation ||
// #endif
//                 FileInformationClass == FileAllInformation ) {
//
//                 if (!FatAcquireExclusiveVcb( IrpContext, Vcb )) {
//
//                     DebugTrace(0, Dbg, "Cannot acquire Vcb\n", 0);
//
//                     Status = FatFsdPostRequest( IrpContext, Irp );
//                     IrpContext = NULL;
//                     Irp = NULL;
//
//                     try_return( Status );
//                 }
//
//                 VcbAcquired = TRUE;
//             }
//
//             //
//             //  Acquire shared access to the fcb, except for a paging file
//             //  in order to avoid deadlocks with Mm.
//             //
//             //  The "removable" check was added specifically for ReadyBoost,
//             //  which opens its cache file on a removable device as a paging file and
//             //  relies on the file system to validate its mapping information after a
//             //  power transition.
//             //
//
//             if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE ) ||
//                 FlagOn(Fcb->Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA)) {
//
//                 if (!FatAcquireSharedFcb( IrpContext, Fcb )) {
//
//                     DebugTrace(0, Dbg, "Cannot acquire Fcb\n", 0);
//
//                     Status = FatFsdPostRequest( IrpContext, Irp );
//                     IrpContext = NULL;
//                     Irp = NULL;
//
//                     try_return( Status );
//                 }
//
//                 FcbAcquired = TRUE;
//             }
//
//             //
//             //  Make sure the Fcb is in a usable condition.  This
//             //  will raise an error condition if the fcb is unusable
//             //
//
//             FatVerifyFcb( IrpContext, Fcb );
//
//             //
//             //  Based on the information class we'll do different
//             //  actions.  Each of hte procedures that we're calling fills
//             //  up the output buffer, if possible.  They will raise the
//             //  status STATUS_BUFFER_OVERFLOW for an insufficient buffer.
//             //  This is considered a somewhat unusual case and is handled
//             //  more cleanly with the exception mechanism rather than
//             //  testing a return status value for each call.
//             //
//
//             switch (FileInformationClass) {
//
//             case FileAllInformation:
//
//                 //
//                 //  For the all information class we'll typecast a local
//                 //  pointer to the output buffer and then call the
//                 //  individual routines to fill in the buffer.
//                 //
//
//                 AllInfo = Buffer;
//                 Length -= (sizeof(FILE_ACCESS_INFORMATION)
//                            + sizeof(FILE_MODE_INFORMATION)
//                            + sizeof(FILE_ALIGNMENT_INFORMATION));
//
//                 FatQueryBasicInfo( IrpContext, Fcb, FileObject, &AllInfo->BasicInformation, &Length );
//                 FatQueryStandardInfo( IrpContext, Fcb, &AllInfo->StandardInformation, &Length );
//                 FatQueryInternalInfo( IrpContext, Fcb, &AllInfo->InternalInformation, &Length );
//                 FatQueryEaInfo( IrpContext, Fcb, &AllInfo->EaInformation, &Length );
//                 FatQueryPositionInfo( IrpContext, FileObject, &AllInfo->PositionInformation, &Length );
//                 FatQueryNameInfo( IrpContext, Fcb, Ccb, FALSE, &AllInfo->NameInformation, &Length );
//
//                 break;
//
//             case FileBasicInformation:
//
//                 FatQueryBasicInfo( IrpContext, Fcb, FileObject, Buffer, &Length );
//                 break;
//
//             case FileStandardInformation:
//
//                 FatQueryStandardInfo( IrpContext, Fcb, Buffer, &Length );
//                 break;
//
//             case FileInternalInformation:
//
//                 FatQueryInternalInfo( IrpContext, Fcb, Buffer, &Length );
//                 break;
//
//             case FileEaInformation:
//
//                 FatQueryEaInfo( IrpContext, Fcb, Buffer, &Length );
//                 break;
//
//             case FilePositionInformation:
//
//                 FatQueryPositionInfo( IrpContext, FileObject, Buffer, &Length );
//                 break;
//
//             case FileNameInformation:
//
//                 FatQueryNameInfo( IrpContext, Fcb, Ccb, FALSE, Buffer, &Length );
//                 break;
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             case FileNormalizedNameInformation:
//
//                 FatQueryNameInfo( IrpContext, Fcb, Ccb, TRUE, Buffer, &Length );
//                 break;
// #endif
//
//             case FileAlternateNameInformation:
//
//                 FatQueryShortNameInfo( IrpContext, Fcb, Buffer, &Length );
//                 break;
//
//             case FileNetworkOpenInformation:
//
//                 FatQueryNetworkInfo( IrpContext, Fcb, FileObject, Buffer, &Length );
//                 break;
//
//             default:
//
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             break;
//
//         default:
//
//             KdPrintEx((DPFLTR_FASTFAT_ID,
//                        DPFLTR_INFO_LEVEL,
//                        "FATQueryFile, Illegal TypeOfOpen = %08lx\n",
//                        TypeOfOpen));
//
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//
//         //
//         //  If we overflowed the buffer, set the length to 0 and change the
//         //  status to STATUS_BUFFER_OVERFLOW.
//         //
//
//         if ( Length < 0 ) {
//
//             Status = STATUS_BUFFER_OVERFLOW;
//
//             Length = 0;
//         }
//
//         //
//         //  Set the information field to the number of bytes actually filled in
//         //  and then complete the request
//         //
//
//         Irp->IoStatus.Information = IrpSp->Parameters.QueryFile.Length - Length;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonQueryInformation );
//
//         if (FcbAcquired) { FatReleaseFcb( IrpContext, Fcb ); }
//         if (VcbAcquired) { FatReleaseVcb( IrpContext, Vcb ); }
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonQueryInformation -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonSetInformation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for setting file information called by both
//     the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//     FILE_INFORMATION_CLASS FileInformationClass;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN VcbAcquired = FALSE;
//     BOOLEAN FcbAcquired = FALSE;
//
//     let _ = PAGED_CODE;
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonSetInformation...\n", 0);
//     DebugTrace( 0, Dbg, "Irp                    = %p\n", Irp);
//     DebugTrace( 0, Dbg, "->Length               = %08lx\n", IrpSp->Parameters.SetFile.Length);
//     DebugTrace( 0, Dbg, "->FileInformationClass = %08lx\n", IrpSp->Parameters.SetFile.FileInformationClass);
//     DebugTrace( 0, Dbg, "->FileObject           = %p\n", IrpSp->Parameters.SetFile.FileObject);
//     DebugTrace( 0, Dbg, "->ReplaceIfExists      = %08lx\n", IrpSp->Parameters.SetFile.ReplaceIfExists);
//     DebugTrace( 0, Dbg, "->Buffer               = %p\n", Irp->AssociatedIrp.SystemBuffer);
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     FileInformationClass = IrpSp->Parameters.SetFile.FileInformationClass;
//     FileObject = IrpSp->FileObject;
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     _SEH2_TRY {
//
//         //
//         //  Case on the type of open we're dealing with
//         //
//
//         switch (TypeOfOpen) {
//
//         case UserVolumeOpen:
//
//             //
//             //  We cannot query the user volume open.
//             //
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//             break;
//
//         case UserFileOpen:
//
//             if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE ) &&
//                 ((FileInformationClass == FileEndOfFileInformation) ||
//                  (FileInformationClass == FileAllocationInformation) ||
//                  (FileInformationClass == FileValidDataLengthInformation))) {
//
//                 //
//                 //  We check whether we can proceed
//                 //  based on the state of the file oplocks.
//                 //
//
//                 Status = FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                                            Irp,
//                                            IrpContext,
//                                            NULL,
//                                            NULL );
//
//                 if (Status != STATUS_SUCCESS) {
//
//                     try_return( Status );
//                 }
//
//                 //
//                 //  Set the flag indicating if Fast I/O is possible
//                 //
//
//                 Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//             }
//             break;
//
//         case UserDirectoryOpen:
//
//             break;
//
//         default:
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  We can only do a set on a nonroot dcb, so we do the test
//         //  and then fall through to the user file open code.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_ROOT_DCB) {
//
//             if (FileInformationClass == FileDispositionInformation) {
//
//                 try_return( Status = STATUS_CANNOT_DELETE );
//             }
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  In the following two cases, we cannot have creates occuring
//         //  while we are here, so acquire the volume exclusive.
//         //
//
//         if ((FileInformationClass == FileDispositionInformation) ||
//             (FileInformationClass == FileRenameInformation)) {
//
//             if (!FatAcquireExclusiveVcb( IrpContext, Vcb )) {
//
//                 DebugTrace(0, Dbg, "Cannot acquire Vcb\n", 0);
//
//                 Status = FatFsdPostRequest( IrpContext, Irp );
//                 Irp = NULL;
//                 IrpContext = NULL;
//
//                 try_return( Status );
//             }
//
//             VcbAcquired = TRUE;
//
//             //
//             //  Make sure we haven't been called recursively by a filter inside an existing
//             //  create request.
//             //
//
//             if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_CREATE_IN_PROGRESS)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck( 0, 0, 0);
//             }
//         }
//
//         //
//         //  Acquire exclusive access to the Fcb,  We use exclusive
//         //  because it is probable that one of the subroutines
//         //  that we call will need to monkey with file allocation,
//         //  create/delete extra fcbs.  So we're willing to pay the
//         //  cost of exclusive Fcb access.
//         //
//         //  Note that we do not acquire the resource for paging file
//         //  operations in order to avoid deadlock with Mm.
//         //
//         //  The "removable" check was added specifically for ReadyBoost,
//         //  which opens its cache file on a removable device as a paging file and
//         //  relies on the file system to validate its mapping information after a
//         //  power transition.
//         //
//
//         if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE ) ||
//             FlagOn(Fcb->Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA)) {
//
//             if (!FatAcquireExclusiveFcb( IrpContext, Fcb )) {
//
//                 DebugTrace(0, Dbg, "Cannot acquire Fcb\n", 0);
//
//                 Status = FatFsdPostRequest( IrpContext, Irp );
//                 Irp = NULL;
//                 IrpContext = NULL;
//
//                 try_return( Status );
//             }
//
//             FcbAcquired = TRUE;
//         }
//
//         Status = STATUS_SUCCESS;
//
//         //
//         //  Make sure the Fcb is in a usable condition.  This
//         //  will raise an error condition if the fcb is unusable
//         //
//
//         FatVerifyFcb( IrpContext, Fcb );
//
//         //
//         //  Now that we've acquired the file, do an oplock check if the operation
//         //  so warrants.
//         //
//
//         if (FatIsFileOplockable( Fcb )&&
//             ((FileInformationClass == FileRenameInformation) ||
//              ((FileInformationClass == FileDispositionInformation) &&
//               ((PFILE_DISPOSITION_INFORMATION) Irp->AssociatedIrp.SystemBuffer)->DeleteFile))) {
//
//             Status = FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                                        Irp,
//                                        IrpContext,
//                                        FatOplockComplete,
//                                        NULL );
//
//             //
//             //  Set the flag indicating if Fast I/O is possible
//             //
//
//             Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//             //
//             //  If STATUS_PENDING is returned it means the oplock
//             //  package has the Irp.  Don't complete the request here.
//             //
//
//             if (Status == STATUS_PENDING) {
//
//                 Irp = NULL;
//                 IrpContext = NULL;
//             }
//
//             if (!NT_SUCCESS( Status ) ||
//                 (Status == STATUS_PENDING)) {
//
//                 try_return( Status );
//             }
//         }
//
//         //
//         //  Based on the information class we'll do different
//         //  actions.  Each of the procedures that we're calling will either
//         //  complete the request of send the request off to the fsp
//         //  to do the work.
//         //
//
//         switch (FileInformationClass) {
//
//         case FileBasicInformation:
//
//             Status = FatSetBasicInfo( IrpContext, Irp, Fcb, Ccb );
//             break;
//
//         case FileDispositionInformation:
//
//             //
//             //  If this is on deferred flush media, we have to be able to wait.
//             //
//
//             if ( FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH) &&
//                  !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) ) {
//
//                 Status = FatFsdPostRequest( IrpContext, Irp );
//                 Irp = NULL;
//                 IrpContext = NULL;
//
//             } else {
//
//                 Status = FatSetDispositionInfo( IrpContext, Irp, FileObject, Fcb );
//             }
//
//             break;
//
//         case FileRenameInformation:
//
//             //
//             //  We proceed with this operation only if we can wait
//             //
//
//             if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//                 Status = FatFsdPostRequest( IrpContext, Irp );
//                 Irp = NULL;
//                 IrpContext = NULL;
//
//             } else {
//
//                 Status = FatSetRenameInfo( IrpContext, Irp, Vcb, Fcb, Ccb );
//
//                 //
//                 //  If STATUS_PENDING is returned it means the oplock
//                 //  package has the Irp.  Don't complete the request here.
//                 //
//
//                 if (Status == STATUS_PENDING) {
//                     Irp = NULL;
//                     IrpContext = NULL;
//                 }
//             }
//
//             break;
//
//         case FilePositionInformation:
//
//             Status = FatSetPositionInfo( IrpContext, Irp, FileObject );
//             break;
//
//         case FileLinkInformation:
//
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//
//         case FileAllocationInformation:
//
//             Status = FatSetAllocationInfo( IrpContext, Irp, Fcb, FileObject );
//             break;
//
//         case FileEndOfFileInformation:
//
//             Status = FatSetEndOfFileInfo( IrpContext, Irp, FileObject, Vcb, Fcb );
//             break;
//
//         case FileValidDataLengthInformation:
//
//             Status = FatSetValidDataLengthInfo( IrpContext, Irp, FileObject, Fcb, Ccb );
//             break;
//
//         default:
//
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//
//         if ( IrpContext != NULL ) {
//
//             FatUnpinRepinnedBcbs( IrpContext );
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonSetInformation );
//
//         if (FcbAcquired) { FatReleaseFcb( IrpContext, Fcb ); }
//
//         if (VcbAcquired) { FatReleaseVcb( IrpContext, Vcb ); }
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonSetInformation -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// VOID
// FatQueryBasicInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_BASIC_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//  Description:
//
//     This routine performs the query basic information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     FileObject - Supplies the flag bit that indicates the file was modified.
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     let _ = PAGED_CODE;
//
//     UNREFERENCED_PARAMETER( FileObject );
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     DebugTrace(+1, Dbg, "FatQueryBasicInfo...\n", 0);
//
//     //
//     //  Zero out the output buffer, and set it to indicate that
//     //  the query is a normal file.  Later we might overwrite the
//     //  attribute.
//     //
//
//     RtlZeroMemory( Buffer, sizeof(FILE_BASIC_INFORMATION) );
//
//     //
//     //  Extract the data and fill in the non zero fields of the output
//     //  buffer
//     //
//
//     if (Fcb->Header.NodeTypeCode == FAT_NTC_ROOT_DCB) {
//
//         //
//         //  We have to munge a lie on the fly.  Every time we have to
//         //  use 1/1/80 we need to convert to GMT since the TZ may have
//         //  changed on us.
//         //
//
//         ExLocalTimeToSystemTime( &FatJanOne1980,
//                                  &Buffer->LastWriteTime );
//         Buffer->CreationTime = Buffer->LastAccessTime = Buffer->LastWriteTime;
//
//     } else {
//
//         Buffer->LastWriteTime = Fcb->LastWriteTime;
//         Buffer->CreationTime = Fcb->CreationTime;
//         Buffer->LastAccessTime = Fcb->LastAccessTime;
//     }
//
//     Buffer->FileAttributes = Fcb->DirentFatFlags;
//
//
//     //
//     //  If the temporary flag is set, then set it in the buffer.
//     //
//
//     if (FlagOn( Fcb->FcbState, FCB_STATE_TEMPORARY )) {
//
//         SetFlag( Buffer->FileAttributes, FILE_ATTRIBUTE_TEMPORARY );
//     }
//
//     //
//     //  If no attributes were set, set the normal bit.
//     //
//
//     if (Buffer->FileAttributes == 0) {
//
//         Buffer->FileAttributes = FILE_ATTRIBUTE_NORMAL;
//     }
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_BASIC_INFORMATION );
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryBasicInfo -> VOID\n", 0);
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryStandardInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_STANDARD_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query standard information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatQueryStandardInfo...\n", 0);
//
//     //
//     //  Zero out the output buffer, and fill in the number of links
//     //  and the delete pending flag.
//     //
//
//     RtlZeroMemory( Buffer, sizeof(FILE_STANDARD_INFORMATION) );
//
//     Buffer->NumberOfLinks = 1;
//     Buffer->DeletePending = BooleanFlagOn( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//
//     //
//     //  Case on whether this is a file or a directory, and extract
//     //  the information and fill in the fcb/dcb specific parts
//     //  of the output buffer
//     //
//
//     if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//         if (Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//             FatLookupFileAllocationSize( IrpContext, Fcb );
//         }
//
//         Buffer->AllocationSize = Fcb->Header.AllocationSize;
//         Buffer->EndOfFile = Fcb->Header.FileSize;
//
//         Buffer->Directory = FALSE;
//
//     } else {
//
//         Buffer->Directory = TRUE;
//     }
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_STANDARD_INFORMATION );
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryStandardInfo -> VOID\n", 0);
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// VOID
// FatQueryInternalInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_INTERNAL_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query internal information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     let _ = PAGED_CODE;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     DebugTrace(+1, Dbg, "FatQueryInternalInfo...\n", 0);
//
//     _SEH2_TRY {
//
//         Buffer->IndexNumber.QuadPart = FatGenerateFileIdFromFcb( Fcb );
//
//         //
//         //  Update the length and status output variables
//         //
//
//         *Length -= sizeof( FILE_INTERNAL_INFORMATION );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatQueryInternalInfo );
//
//         DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//         DebugTrace(-1, Dbg, "FatQueryInternalInfo -> VOID\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// VOID
// FatQueryEaInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_EA_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query Ea information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PBCB Bcb;
//
//     let _ = PAGED_CODE;
//
//     UNREFERENCED_PARAMETER( Fcb );
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     DebugTrace(+1, Dbg, "FatQueryEaInfo...\n", 0);
//
//     Bcb = NULL;
//
//     _SEH2_TRY {
//
//         //
//         //  Zero out the output buffer
//         //
//
//         RtlZeroMemory( Buffer, sizeof(FILE_EA_INFORMATION) );
//
//
//         //
//         //  Update the length and status output variables
//         //
//
//         *Length -= sizeof( FILE_EA_INFORMATION );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatQueryEaInfo );
//
//         //
//         //  Unpin the dirent if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, Bcb );
//
//         DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//         DebugTrace(-1, Dbg, "FatQueryEaInfo -> VOID\n", 0);
//     } _SEH2_END;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// VOID
// FatQueryPositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_POSITION_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query position information function for fat.
//
// Arguments:
//
//     FileObject - Supplies the File object being queried
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatQueryPositionInfo...\n", 0);
//
//     //
//     //  Get the current position found in the file object.
//     //
//
//     Buffer->CurrentByteOffset = FileObject->CurrentByteOffset;
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_POSITION_INFORMATION );
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryPositionInfo -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryNameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PCCB Ccb,
//     IN BOOLEAN Normalized,
//     IN OUT PFILE_NAME_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query name information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     Ccb - Supplies the Ccb for the context of the user open
//
//     Normalized - if true the caller wants a normalized name (w/out short names).
//         This means we're servicing a FileNormalizedNameInformation query.
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     ULONG BytesToCopy;
//     LONG TrimLength;
//     BOOLEAN Overflow = FALSE;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatQueryNameInfo...\n", 0);
//
//     //
//     //  Convert the name to UNICODE
//     //
//
//     *Length -= FIELD_OFFSET(FILE_NAME_INFORMATION, FileName[0]);
//
//     //
//     //  Use the full filename to build the path up.  If we wanted to be
//     //  slick in the future, we'd just build the path directly into the
//     //  return buffer and avoid constructing the full filename, but since
//     //  the full filename winds up being required so often lets not
//     //  over optimize this case yet.
//     //
//
//     if (Fcb->FullFileName.Buffer == NULL) {
//
//         FatSetFullFileNameInFcb( IrpContext, Fcb );
//     }
//
//     //
//     //  Here is where it gets a smidge tricky.  FinalNameLength is the length
//     //  of the LFN element if it exists, and since the long name is always used
//     //  to build FullFileName, we have two cases:
//     //
//     //  1) short name: use FinalNameLength to tear off the path from FullFileName
//     //      and append the UNICODE converted short name.
//     //  2) long name: just use FullFileName
//     //
//     //  We bias to the name the user thinks they opened by.  This winds
//     //  up fixing some oddball tunneling cases where intermediate filters
//     //  translate operations like delete into renames - this lets them
//     //  do the operation in the context of the name the user was using.
//     //
//     //  It also matches what NTFS does, and so we have the definition of
//     //  correct behavior.
//     //
//
//     //
//     //
//     //  Assume there is no long name and we are just going to use
//     //  FullFileName.
//     //
//
//     TrimLength = 0;
//
//     //
//     //  If a LongName exists, the caller isn't asking for the normalized name,
//     //  and the original open was by the short name then set TrimLength to point
//     //  to the place where the short name goes.
//     //
//     //  Note: The Ccb can be NULL.  The lazy writer calls to get the name of
//     //  a DirectoryOpen FILE_OBJECT that it wants to display in the lost
//     //  delayed write popup.  Handle this case by just using the FileFullName.
//     //
//
//     if (!Normalized &&
//         (Fcb->LongName.Unicode.Name.Unicode.Buffer != NULL)) {
//
//         if ((Ccb != NULL) &&
//             FlagOn(Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME)) {
//
//             TrimLength = Fcb->FinalNameLength;
//         }
//     }
//
//     if (*Length < Fcb->FullFileName.Length - TrimLength) {
//
//         BytesToCopy = *Length;
//         Overflow = TRUE;
//
//     } else {
//
//         BytesToCopy = Fcb->FullFileName.Length - TrimLength;
//         *Length -= BytesToCopy;
//     }
//
//     RtlCopyMemory( &Buffer->FileName[0],
//                    Fcb->FullFileName.Buffer,
//                    BytesToCopy );
//
//     //
//     //  Note that this is just the amount of name we've copied so far.  It'll
//     //  either be all of it (long) or the path element including the \ (short).
//     //
//
//     Buffer->FileNameLength = Fcb->FullFileName.Length - TrimLength;
//
//     //
//     //  If we trimmed off the name element, this is the short name case.  Pick
//     //  up the UNICODE conversion and append it.
//     //
//
//     if (TrimLength != 0) {
//
//         UNICODE_STRING ShortName;
//         WCHAR ShortNameBuffer[12];
//         NTSTATUS Status;
//
//         //
//         //  Convert the short name to UNICODE and figure out how much
//         //  of it can fit.  Again, we always bump the returned length
//         //  to indicate how much is available even if we can't return it.
//         //
//
//         ShortName.Length = 0;
//         ShortName.MaximumLength = sizeof(ShortNameBuffer);
//         ShortName.Buffer = ShortNameBuffer;
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//         Status = RtlOemStringToCountedUnicodeString( &ShortName,
//                                                      &Fcb->ShortName.Name.Oem,
//                                                      FALSE );
//
//         NT_ASSERT( Status == STATUS_SUCCESS );
//
//         if (!Overflow) {
//
//             if (*Length < ShortName.Length) {
//
//                 BytesToCopy = *Length;
//                 Overflow = TRUE;
//
//             } else {
//
//                 BytesToCopy = ShortName.Length;
//                 *Length -= BytesToCopy;
//             }
//
//             RtlCopyMemory( (PUCHAR)&Buffer->FileName[0] + Buffer->FileNameLength,
//                            ShortName.Buffer,
//                            BytesToCopy );
//         }
//
//         Buffer->FileNameLength += ShortName.Length;
//     }
//
//     if (Overflow) {
//
//         *Length = -1;
//     }
//
//     //
//     //  Return to caller
//     //
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryNameInfo -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// VOID
// FatQueryShortNameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PFILE_NAME_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine queries the short name of the file.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     ULONG BytesToCopy;
//     WCHAR ShortNameBuffer[12];
//     UNICODE_STRING ShortName;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatQueryNameInfo...\n", 0);
//
//     //
//     //  Convert the name to UNICODE
//     //
//
//     ShortName.Length = 0;
//     ShortName.MaximumLength = sizeof(ShortNameBuffer);
//     ShortName.Buffer = ShortNameBuffer;
//
//     *Length -= FIELD_OFFSET(FILE_NAME_INFORMATION, FileName[0]);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//     Status = RtlOemStringToCountedUnicodeString( &ShortName,
//                                                  &Fcb->ShortName.Name.Oem,
//                                                  FALSE );
//
//     NT_ASSERT( Status == STATUS_SUCCESS );
//
//     //
//     //  If we overflow, set *Length to -1 as a flag.
//     //
//
//     if (*Length < ShortName.Length) {
//
//         BytesToCopy = *Length;
//         *Length = -1;
//
//     } else {
//
//         BytesToCopy = ShortName.Length;
//         *Length -= ShortName.Length;
//     }
//
//     RtlCopyMemory( &Buffer->FileName[0],
//                    &ShortName.Buffer[0],
//                    BytesToCopy );
//
//     Buffer->FileNameLength = ShortName.Length;
//
//     //
//     //  Return to caller
//     //
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryNameInfo -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatQueryNetworkInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject,
//     IN OUT PFILE_NETWORK_OPEN_INFORMATION Buffer,
//     IN OUT PLONG Length
//     )
//
// /*++
//  Description:
//
//     This routine performs the query network open information function for fat.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified
//
//     FileObject - Supplies the flag bit that indicates the file was modified.
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     let _ = PAGED_CODE;
//
//     UNREFERENCED_PARAMETER( FileObject );
//
//     DebugTrace(+1, Dbg, "FatQueryNetworkInfo...\n", 0);
//
//     //
//     //  Zero out the output buffer, and set it to indicate that
//     //  the query is a normal file.  Later we might overwrite the
//     //  attribute.
//     //
//
//     RtlZeroMemory( Buffer, sizeof(FILE_NETWORK_OPEN_INFORMATION) );
//
//     //
//     //  Extract the data and fill in the non zero fields of the output
//     //  buffer
//     //
//
//     if (Fcb->Header.NodeTypeCode == FAT_NTC_ROOT_DCB) {
//
//         //
//         //  We have to munge a lie on the fly.  Every time we have to
//         //  use 1/1/80 we need to convert to GMT since the TZ may have
//         //  changed on us.
//         //
//
//         ExLocalTimeToSystemTime( &FatJanOne1980,
//                                  &Buffer->LastWriteTime );
//         Buffer->CreationTime = Buffer->LastAccessTime = Buffer->LastWriteTime;
//
//     } else {
//
//         Buffer->LastWriteTime.QuadPart = Fcb->LastWriteTime.QuadPart;
//         Buffer->CreationTime.QuadPart = Fcb->CreationTime.QuadPart;
//         Buffer->LastAccessTime.QuadPart = Fcb->LastAccessTime.QuadPart;
//     }
//
//     Buffer->FileAttributes = Fcb->DirentFatFlags;
//
//
//     //
//     //  If the temporary flag is set, then set it in the buffer.
//     //
//
//     if (FlagOn( Fcb->FcbState, FCB_STATE_TEMPORARY )) {
//
//         SetFlag( Buffer->FileAttributes, FILE_ATTRIBUTE_TEMPORARY );
//     }
//
//     //
//     //  If no attributes were set, set the normal bit.
//     //
//
//     if (Buffer->FileAttributes == 0) {
//
//         Buffer->FileAttributes = FILE_ATTRIBUTE_NORMAL;
//     }
//     //
//     //  Case on whether this is a file or a directory, and extract
//     //  the information and fill in the fcb/dcb specific parts
//     //  of the output buffer
//     //
//
//     if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//         if (Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//             FatLookupFileAllocationSize( IrpContext, Fcb );
//         }
//
//         Buffer->AllocationSize.QuadPart = Fcb->Header.AllocationSize.QuadPart;
//         Buffer->EndOfFile.QuadPart = Fcb->Header.FileSize.QuadPart;
//     }
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_NETWORK_OPEN_INFORMATION );
//
//     DebugTrace( 0, Dbg, "*Length = %08lx\n", *Length);
//
//     DebugTrace(-1, Dbg, "FatQueryNetworkInfo -> VOID\n", 0);
//
//     return;
// }
//
//
// //
// //  Internal Support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetBasicInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set basic information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
//     Ccb - Supplies the flag bit that control updating the last modify
//         time on cleanup.
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PFILE_BASIC_INFORMATION Buffer;
//
//     PDIRENT Dirent;
//     PBCB DirentBcb;
//
//     FAT_TIME_STAMP CreationTime = {0};
//     UCHAR CreationMSec = 0;
//     FAT_TIME_STAMP LastWriteTime = {0};
//     FAT_TIME_STAMP LastAccessTime = {0};
//     FAT_DATE LastAccessDate = {0};
//     UCHAR Attributes;
//
//     BOOLEAN ModifyCreation = FALSE;
//     BOOLEAN ModifyLastWrite = FALSE;
//     BOOLEAN ModifyLastAccess = FALSE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     BOOLEAN ModifiedAttributes = FALSE;
// #endif
//
//     LARGE_INTEGER LargeCreationTime = {0};
//     LARGE_INTEGER LargeLastWriteTime = {0};
//     LARGE_INTEGER LargeLastAccessTime = {0};
//
//     ULONG NotifyFilter = 0;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatSetBasicInfo...\n", 0);
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  If the user is specifying -1 for a field, that means
//     //  we should leave that field unchanged, even if we might
//     //  have otherwise set it ourselves.  We'll set the Ccb flag
//     //  saying that the user set the field so that we
//     //  don't do our default updating.
//     //
//     //  We set the field to 0 then so we know not to actually
//     //  set the field to the user-specified (and in this case,
//     //  illegal) value.
//     //
//
//     if (Buffer->LastWriteTime.QuadPart == -1) {
//
//         SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_LAST_WRITE );
//         Buffer->LastWriteTime.QuadPart = 0;
//     }
//
//     if (Buffer->LastAccessTime.QuadPart == -1) {
//
//         SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_LAST_ACCESS );
//         Buffer->LastAccessTime.QuadPart = 0;
//     }
//
//     if (Buffer->CreationTime.QuadPart == -1) {
//
//         SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_CREATION );
//         Buffer->CreationTime.QuadPart = 0;
//     }
//
//     DirentBcb = NULL;
//
//     Status = STATUS_SUCCESS;
//
//     _SEH2_TRY {
//
//         LARGE_INTEGER FatLocalDecThirtyOne1979;
//         LARGE_INTEGER FatLocalJanOne1980;
//
//         ExLocalTimeToSystemTime( &FatDecThirtyOne1979,
//                                  &FatLocalDecThirtyOne1979 );
//
//         ExLocalTimeToSystemTime( &FatJanOne1980,
//                                  &FatLocalJanOne1980 );
//
//         //
//         //  Get a pointer to the dirent
//         //
//
//         NT_ASSERT( Fcb->FcbCondition == FcbGood );
//
//         FatGetDirentFromFcbOrDcb( IrpContext,
//                                   Fcb,
//                                   FALSE,
//                                   &Dirent,
//                                   &DirentBcb );
//         //
//         //  Check if the user specified a non-zero creation time
//         //
//
//         if (FatData.ChicagoMode && (Buffer->CreationTime.QuadPart != 0)) {
//
//             LargeCreationTime = Buffer->CreationTime;
//
//             //
//             //  Convert the Nt time to a Fat time
//             //
//
//             if ( !FatNtTimeToFatTime( IrpContext,
//                                       &LargeCreationTime,
//                                       FALSE,
//                                       &CreationTime,
//                                       &CreationMSec )) {
//
//                 //
//                 //  Special case the value 12/31/79 and treat this as 1/1/80.
//                 //  This '79 value can happen because of time zone issues.
//                 //
//
//                 if ((LargeCreationTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart) &&
//                     (LargeCreationTime.QuadPart < FatLocalJanOne1980.QuadPart)) {
//
//                     CreationTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
//                     LargeCreationTime = FatLocalJanOne1980;
//
//                 } else {
//
//                     DebugTrace(0, Dbg, "Invalid CreationTime\n", 0);
//                     try_return( Status = STATUS_INVALID_PARAMETER );
//                 }
//
//                 //
//                 //  Don't worry about CreationMSec
//                 //
//
//                 CreationMSec = 0;
//             }
//
//             ModifyCreation = TRUE;
//         }
//
//         //
//         //  Check if the user specified a non-zero last access time
//         //
//
//         if (FatData.ChicagoMode && (Buffer->LastAccessTime.QuadPart != 0)) {
//
//             LargeLastAccessTime = Buffer->LastAccessTime;
//
//             //
//             //  Convert the Nt time to a Fat time
//             //
//
//             if ( !FatNtTimeToFatTime( IrpContext,
//                                       &LargeLastAccessTime,
//                                       TRUE,
//                                       &LastAccessTime,
//                                       NULL )) {
//
//                 //
//                 //  Special case the value 12/31/79 and treat this as 1/1/80.
//                 //  This '79 value can happen because of time zone issues.
//                 //
//
//                 if ((LargeLastAccessTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart) &&
//                     (LargeLastAccessTime.QuadPart < FatLocalJanOne1980.QuadPart)) {
//
//                     LastAccessTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
//                     LargeLastAccessTime = FatLocalJanOne1980;
//
//                 } else {
//
//                     DebugTrace(0, Dbg, "Invalid LastAccessTime\n", 0);
//                     try_return( Status = STATUS_INVALID_PARAMETER );
//                 }
//             }
//
//             LastAccessDate = LastAccessTime.Date;
//             ModifyLastAccess = TRUE;
//         }
//
//         //
//         //  Check if the user specified a non-zero last write time
//         //
//
//         if (Buffer->LastWriteTime.QuadPart != 0) {
//
//             //
//             //  First do a quick check here if the this time is the same
//             //  time as LastAccessTime.
//             //
//
//             if (ModifyLastAccess &&
//                 (Buffer->LastWriteTime.QuadPart == Buffer->LastAccessTime.QuadPart)) {
//
//                 ModifyLastWrite = TRUE;
//                 LastWriteTime = LastAccessTime;
//                 LargeLastWriteTime = LargeLastAccessTime;
//
//             } else {
//
//                 LargeLastWriteTime = Buffer->LastWriteTime;
//
//                 //
//                 //  Convert the Nt time to a Fat time
//                 //
//
//                 if ( !FatNtTimeToFatTime( IrpContext,
//                                           &LargeLastWriteTime,
//                                           TRUE,
//                                           &LastWriteTime,
//                                           NULL )) {
//
//
//                     //
//                     //  Special case the value 12/31/79 and treat this as 1/1/80.
//                     //  This '79 value can happen because of time zone issues.
//                     //
//
//                     if ((LargeLastWriteTime.QuadPart >= FatLocalDecThirtyOne1979.QuadPart) &&
//                         (LargeLastWriteTime.QuadPart < FatLocalJanOne1980.QuadPart)) {
//
//                         LastWriteTime = core::ptr::read(core::ptr::addr_of!(FatTimeJanOne1980));
//                         LargeLastWriteTime = FatLocalJanOne1980;
//
//                     } else {
//
//                         DebugTrace(0, Dbg, "Invalid LastWriteTime\n", 0);
//                         try_return( Status = STATUS_INVALID_PARAMETER );
//                     }
//                 }
//
//                 ModifyLastWrite = TRUE;
//             }
//         }
//
//
//         //
//         //  Check if the user specified a non zero file attributes byte
//         //
//
//         if (Buffer->FileAttributes != 0) {
//
//             //
//             //  Only permit the attributes that FAT understands.  The rest are silently
//             //  dropped on the floor.
//             //
//
//             Attributes = (UCHAR)(Buffer->FileAttributes & (FILE_ATTRIBUTE_READONLY |
//                                                            FILE_ATTRIBUTE_HIDDEN |
//                                                            FILE_ATTRIBUTE_SYSTEM |
//                                                            FILE_ATTRIBUTE_DIRECTORY |
//                                                            FILE_ATTRIBUTE_ARCHIVE));
//
//             //
//             //  Make sure that for a file the directory bit is not set
//             //  and that for a directory the bit is set.
//             //
//
//             if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//                 if (FlagOn(Buffer->FileAttributes, FILE_ATTRIBUTE_DIRECTORY)) {
//
//                     DebugTrace(0, Dbg, "Attempt to set dir attribute on file\n", 0);
//                     try_return( Status = STATUS_INVALID_PARAMETER );
//                 }
//
//             } else {
//
//                 Attributes |= FAT_DIRENT_ATTR_DIRECTORY;
//             }
//
//             //
//             //  Mark the FcbState temporary flag correctly.
//             //
//
//             if (FlagOn(Buffer->FileAttributes, FILE_ATTRIBUTE_TEMPORARY)) {
//
//                 //
//                 //  Don't allow the temporary bit to be set on directories.
//                 //
//
//                 if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//                     DebugTrace(0, Dbg, "No temporary directories\n", 0);
//                     try_return( Status = STATUS_INVALID_PARAMETER );
//                 }
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_TEMPORARY );
//
//                 SetFlag( IoGetCurrentIrpStackLocation(Irp)->FileObject->Flags,
//                          FO_TEMPORARY_FILE );
//
//             } else {
//
//                 ClearFlag( Fcb->FcbState, FCB_STATE_TEMPORARY );
//
//                 ClearFlag( IoGetCurrentIrpStackLocation(Irp)->FileObject->Flags,
//                            FO_TEMPORARY_FILE );
//             }
//
//             //
//             //  Now, only proceed if the requested file attributes are different
//             //  than the existing attributes.
//             //
//
//             if (Dirent->Attributes != Attributes) {
//
//                 //
//                 //  Set the new attributes byte, and mark the bcb dirty
//                 //
//
//                 Fcb->DirentFatFlags = Attributes;
//
//                 Dirent->Attributes = Attributes;
//
//                 NotifyFilter |= FILE_NOTIFY_CHANGE_ATTRIBUTES;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                 ModifiedAttributes = TRUE;
// #endif
//             }
//         }
//
//         if ( ModifyCreation ) {
//
//             //
//             //  Set the new last write time in the dirent, and mark
//             //  the bcb dirty
//             //
//
//             Fcb->CreationTime = LargeCreationTime;
//             Dirent->CreationTime = CreationTime;
//             Dirent->CreationMSec = CreationMSec;
//
//
//             NotifyFilter |= FILE_NOTIFY_CHANGE_CREATION;
//             //
//             //  Now we have to round the time in the Fcb up to the
//             //  nearest tem msec.
//             //
//
//             Fcb->CreationTime.QuadPart =
//
//                 ((Fcb->CreationTime.QuadPart + AlmostTenMSec) /
//                  TenMSec) * TenMSec;
//
//             //
//             //  Now because the user just set the creation time we
//             //  better not set the creation time on close
//             //
//
//             SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_CREATION );
//         }
//
//         if ( ModifyLastAccess ) {
//
//             //
//             //  Set the new last write time in the dirent, and mark
//             //  the bcb dirty
//             //
//
//             Fcb->LastAccessTime = LargeLastAccessTime;
//             Dirent->LastAccessDate = LastAccessDate;
//
//             NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_ACCESS;
//
//             //
//             //  Now we have to truncate the time in the Fcb down to the
//             //  current day.  This has to be in LocalTime though, so first
//             //  convert to local, trunacate, then set back to GMT.
//             //
//
//             ExSystemTimeToLocalTime( &Fcb->LastAccessTime,
//                                      &Fcb->LastAccessTime );
//
//             Fcb->LastAccessTime.QuadPart =
//
//                 (Fcb->LastAccessTime.QuadPart /
//                  FatOneDay.QuadPart) * FatOneDay.QuadPart;
//
//             ExLocalTimeToSystemTime( &Fcb->LastAccessTime,
//                                      &Fcb->LastAccessTime );
//
//             //
//             //  Now because the user just set the last access time we
//             //  better not set the last access time on close
//             //
//
//             SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_LAST_ACCESS );
//         }
//
//         if ( ModifyLastWrite ) {
//
//             //
//             //  Set the new last write time in the dirent, and mark
//             //  the bcb dirty
//             //
//
//             Fcb->LastWriteTime = LargeLastWriteTime;
//             Dirent->LastWriteTime = LastWriteTime;
//
//             NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_WRITE;
//
//             //
//             //  Now we have to round the time in the Fcb up to the
//             //  nearest two seconds.
//             //
//
//             Fcb->LastWriteTime.QuadPart =
//
//                 ((Fcb->LastWriteTime.QuadPart + AlmostTwoSeconds) /
//                  TwoSeconds) * TwoSeconds;
//
//             //
//             //  Now because the user just set the last write time we
//             //  better not set the last write time on close
//             //
//
//             SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_LAST_WRITE );
//         }
//
//         //
//         //  If we modified any of the values, we report this to the notify
//         //  package.
//         //
//         //  We also take this opportunity to set the current file size and
//         //  first cluster in the Dirent in order to support a server hack.
//         //
//
//         if (NotifyFilter != 0) {
//
//             if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//                 Dirent->FileSize = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//
//
//                 Dirent->FirstClusterOfFile = (USHORT)Fcb->FirstClusterOfFile;
//
//                 if (FatIsFat32(Fcb->Vcb)) {
//
//                     Dirent->FirstClusterOfFileHi =
//                             (USHORT)(Fcb->FirstClusterOfFile >> 16);
//                 }
//             }
//
//             FatNotifyReportChange( IrpContext,
//                                    Fcb->Vcb,
//                                    Fcb,
//                                    NotifyFilter,
//                                    FILE_ACTION_MODIFIED );
//
//             FatSetDirtyBcb( IrpContext, DirentBcb, Fcb->Vcb, TRUE );
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  If last-access, last-write, or any attribute bits changed, break
//         //  parent directory oplock.
//         //
//
//         if ((Fcb->ParentDcb != NULL) &&
//             (ModifyLastAccess ||
//              ModifyLastWrite ||
//              ModifiedAttributes)) {
//
//             FsRtlCheckOplockEx( FatGetFcbOplock(Fcb->ParentDcb),
//                                 IrpContext->OriginatingIrp,
//                                 OPLOCK_FLAG_PARENT_OBJECT,
//                                 NULL,
//                                 NULL,
//                                 NULL );
//         }
// #endif
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetBasicInfo );
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         DebugTrace(-1, Dbg, "FatSetBasicInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetDispositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set disposition information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     FileObject - Supplies the file object being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     PFILE_DISPOSITION_INFORMATION Buffer;
//     PBCB Bcb;
//     PDIRENT Dirent;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatSetDispositionInfo...\n", 0);
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Check if the user wants to delete the file or not delete
//     //  the file
//     //
//
//     if (Buffer->DeleteFile) {
//
//         //
//         //  Check if the file is marked read only
//         //
//
//         if (FlagOn(Fcb->DirentFatFlags, FAT_DIRENT_ATTR_READ_ONLY)) {
//
//             DebugTrace(-1, Dbg, "Cannot delete readonly file\n", 0);
//
//             return STATUS_CANNOT_DELETE;
//         }
//
//         //
//         //  Make sure there is no process mapping this file as an image.
//         //
//
//         if (!MmFlushImageSection( &Fcb->NonPaged->SectionObjectPointers,
//                                   MmFlushForDelete )) {
//
//             DebugTrace(-1, Dbg, "Cannot delete user mapped image\n", 0);
//
//             return STATUS_CANNOT_DELETE;
//         }
//
//         //
//         //  Check if this is a dcb and if so then only allow
//         //  the request if the directory is empty.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_ROOT_DCB) {
//
//             DebugTrace(-1, Dbg, "Cannot delete root Directory\n", 0);
//
//             return STATUS_CANNOT_DELETE;
//         }
//
//         if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//             DebugTrace(-1, Dbg, "User wants to delete a directory\n", 0);
//
//             //
//             //  Check if the directory is empty
//             //
//
//             if ( !FatIsDirectoryEmpty(IrpContext, Fcb) ) {
//
//                 DebugTrace(-1, Dbg, "Directory is not empty\n", 0);
//
//                 return STATUS_DIRECTORY_NOT_EMPTY;
//             }
//         }
//
//         //
//         //  If this is a floppy, touch the volume so to verify that it
//         //  is not write protected.
//         //
//
//         if ( FlagOn(Fcb->Vcb->Vpb->RealDevice->Characteristics, FILE_FLOPPY_DISKETTE)) {
//
//             PVCB Vcb;
//             PBCB LocalBcb = NULL;
//             UCHAR *LocalBuffer;
//             UCHAR TmpChar;
//             ULONG BytesToMap;
//
//             IO_STATUS_BLOCK Iosb;
//
//             Vcb = Fcb->Vcb;
//
//             BytesToMap = Vcb->AllocationSupport.FatIndexBitSize == 12 ?
//                          FatReservedBytes(&Vcb->Bpb) +
//                          FatBytesPerFat(&Vcb->Bpb):PAGE_SIZE;
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                0,
//                                BytesToMap,
//                                &LocalBcb,
//                                (PVOID *)&LocalBuffer );
//
//             _SEH2_TRY {
//
//                 if (!CcPinMappedData( Vcb->VirtualVolumeFile,
//                                       &FatLargeZero,
//                                       BytesToMap,
//                                       BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                                       &LocalBcb )) {
//
//                     //
//                     // Could not pin the data without waiting (cache miss).
//                     //
//
//                     FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//                 }
//
//                 //
//                 //  Make Mm, myself, and Cc think the byte is dirty, and then
//                 //  force a writethrough.
//                 //
//
//                 LocalBuffer += FatReservedBytes(&Vcb->Bpb);
//
//                 TmpChar = LocalBuffer[0];
//                 LocalBuffer[0] = TmpChar;
//
//                 FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb,
//                                 FatReservedBytes( &Vcb->Bpb ),
//                                 FatReservedBytes( &Vcb->Bpb ),
//                                 Vcb->Bpb.BytesPerSector );
//
//             } _SEH2_FINALLY {
//
//                 if (_SEH2_AbnormalTermination() && (LocalBcb != NULL)) {
//
//                     FatUnpinBcb( IrpContext, LocalBcb );
//                 }
//             } _SEH2_END;
//
//             CcRepinBcb( LocalBcb );
//             CcSetDirtyPinnedData( LocalBcb, NULL );
//             CcUnpinData( LocalBcb );
//             DbgDoit( NT_ASSERT( IrpContext->PinCount ));
//             DbgDoit( IrpContext->PinCount -= 1 );
//             CcUnpinRepinnedBcb( LocalBcb, TRUE, &Iosb );
//
//             //
//             //  If this was not successful, raise the status.
//             //
//
//             if ( !NT_SUCCESS(Iosb.__bindgen_anon_1.Status) ) {
//
//                 FatNormalizeAndRaiseStatus( IrpContext, Iosb.__bindgen_anon_1.Status );
//             }
//
//         } else {
//
//             //
//             //  Just set a Bcb dirty here.  The above code was only there to
//             //  detect a write protected floppy, while the below code works
//             //  for any write protected media and only takes a hit when the
//             //  volume in clean.
//             //
//
//             FatGetDirentFromFcbOrDcb( IrpContext,
//                                       Fcb,
//                                       FALSE,
//                                       &Dirent,
//                                       &Bcb );
//
//             //
//             //  This has to work for the usual reasons (we verified the Fcb within
//             //  volume synch).
//             //
//
//             _SEH2_TRY {
//
//                 FatSetDirtyBcb( IrpContext, Bcb, Fcb->Vcb, TRUE );
//
//             } _SEH2_FINALLY {
//
//                 FatUnpinBcb( IrpContext, Bcb );
//             } _SEH2_END;
//         }
//
//         //
//         //  At this point either we have a file or an empty directory
//         //  so we know the delete can proceed.
//         //
//
//         SetFlag( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//         FileObject->DeletePending = TRUE;
//
//         //
//         //  If this is a directory then report this delete pending to
//         //  the dir notify package.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//             FsRtlNotifyFullChangeDirectory( Fcb->Vcb->NotifySync,
//                                             &Fcb->Vcb->DirNotifyList,
//                                             FileObject->FsContext,
//                                             NULL,
//                                             FALSE,
//                                             FALSE,
//                                             0,
//                                             NULL,
//                                             NULL,
//                                             NULL );
//         }
//     } else {
//
//         //
//         //  The user doesn't want to delete the file so clear
//         //  the delete on close bit
//         //
//
//         DebugTrace(0, Dbg, "User want to not delete file\n", 0);
//
//         ClearFlag( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//         FileObject->DeletePending = FALSE;
//     }
//
//     DebugTrace(-1, Dbg, "FatSetDispositionInfo -> STATUS_SUCCESS\n", 0);
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// NTSTATUS
// FatSetRenameInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PVCB Vcb,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set name information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     Vcb - Supplies the Vcb being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
//     Ccb - Supplies the Ccb corresponding to the handle opening the source
//         file
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     BOOLEAN AllLowerComponent;
//     BOOLEAN AllLowerExtension;
//     BOOLEAN CaseOnlyRename;
//     BOOLEAN ContinueWithRename;
//     BOOLEAN CreateLfn = FALSE;
//     BOOLEAN DeleteSourceDirent;
//     BOOLEAN DeleteTarget;
//     BOOLEAN NewDirentFromPool;
//     BOOLEAN RenamedAcrossDirectories;
//     BOOLEAN ReplaceIfExists;
//
//     CCB LocalCcb;
//     PCCB SourceCcb;
//
//     DIRENT Dirent;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     OEM_STRING OldOemName;
//     OEM_STRING NewOemName;
//     UCHAR OemNameBuffer[24*2];
//
//     PBCB DotDotBcb;
//     PBCB NewDirentBcb;
//     PBCB OldDirentBcb;
//     PBCB SecondPageBcb;
//     PBCB TargetDirentBcb;
//
//     PDCB TargetDcb = NULL;
//     PDCB OldParentDcb;
//
//     PDIRENT DotDotDirent = NULL;
//     PDIRENT FirstPageDirent = NULL;
//     PDIRENT NewDirent = NULL;
//     PDIRENT OldDirent = NULL;
//     PDIRENT SecondPageDirent = NULL;
//     PDIRENT ShortDirent = NULL;
//     PDIRENT TargetDirent = NULL;
//
//     PFCB TempFcb;
//
//     PFILE_OBJECT TargetFileObject;
//     PFILE_OBJECT FileObject;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PLIST_ENTRY Links;
//
//     ULONG BytesInFirstPage = 0;
//     ULONG DirentsInFirstPage = 0;
//     ULONG DirentsRequired = 0;
//     ULONG NewOffset = 0;
//     ULONG NotifyAction = 0;
//     ULONG SecondPageOffset = 0;
//     ULONG ShortDirentOffset = 0;
//     ULONG TargetDirentOffset = 0;
//     ULONG TargetLfnOffset = 0;
//
//     // NewName comes from the IRP buffer or the TargetFileObject, so we can't
//     // go around modifying it. Instead we modify NewNameCopy.
//     UNICODE_STRING NewName;
//
//     // NB: these five UNICODE_STRINGS are allocated
//     // from one chopped up pool allocation called UnicodeBuffer.
//     UNICODE_STRING NewNameCopy;
//     UNICODE_STRING NewUpcasedName;
//     UNICODE_STRING OldName;
//     UNICODE_STRING OldUpcasedName;
//     UNICODE_STRING TargetLfn;
//     PWCHAR UnicodeBuffer;
//
//     UNICODE_STRING TargetOrigLfn = {0};
//
//     UNICODE_STRING UniTunneledShortName;
//     WCHAR UniTunneledShortNameBuffer[12];
//     UNICODE_STRING UniTunneledLongName;
//     WCHAR UniTunneledLongNameBuffer[26];
//
//     LARGE_INTEGER TunneledCreationTime;
//     ULONG TunneledDataSize;
//     BOOLEAN HaveTunneledInformation = FALSE;
//     BOOLEAN UsingTunneledLfn = FALSE;
//
//     BOOLEAN InvalidateFcbOnRaise = FALSE;
//
//     PFILE_OBJECT DirectoryFileObject = NULL;
//     ULONG Flags = 0;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatSetRenameInfo...\n", 0);
//
//     //
//     //  P H A S E  0: Initialize some variables.
//     //
//
//     CaseOnlyRename = FALSE;
//     ContinueWithRename = FALSE;
//     DeleteSourceDirent = FALSE;
//     DeleteTarget = FALSE;
//     NewDirentFromPool = FALSE;
//     RenamedAcrossDirectories = FALSE;
//
//     DotDotBcb = NULL;
//     NewDirentBcb = NULL;
//     OldDirentBcb = NULL;
//     SecondPageBcb = NULL;
//     TargetDirentBcb = NULL;
//
//     NewOemName.Length = 0;
//     NewOemName.MaximumLength = 24;
//     NewOemName.Buffer = (PCHAR)&OemNameBuffer[0];
//
//     OldOemName.Length = 0;
//     OldOemName.MaximumLength = 24;
//     OldOemName.Buffer = (PCHAR)&OemNameBuffer[24];
//
//     UnicodeBuffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                               5 * MAX_LFN_CHARACTERS * sizeof(WCHAR),
//                                               TAG_FILENAME_BUFFER );
//
//     NewUpcasedName.Length = 0;
//     NewUpcasedName.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//     NewUpcasedName.Buffer = &UnicodeBuffer[0];
//
//     OldName.Length = 0;
//     OldName.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//     OldName.Buffer = &UnicodeBuffer[MAX_LFN_CHARACTERS];
//
//     OldUpcasedName.Length = 0;
//     OldUpcasedName.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//     OldUpcasedName.Buffer = &UnicodeBuffer[MAX_LFN_CHARACTERS * 2];
//
//     TargetLfn.Length = 0;
//     TargetLfn.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//     TargetLfn.Buffer = &UnicodeBuffer[MAX_LFN_CHARACTERS * 3];
//
//     NewNameCopy.Length = 0;
//     NewNameCopy.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//     NewNameCopy.Buffer = &UnicodeBuffer[MAX_LFN_CHARACTERS * 4];
//
//     UniTunneledShortName.Length = 0;
//     UniTunneledShortName.MaximumLength = sizeof(UniTunneledShortNameBuffer);
//     UniTunneledShortName.Buffer = &UniTunneledShortNameBuffer[0];
//
//     UniTunneledLongName.Length = 0;
//     UniTunneledLongName.MaximumLength = sizeof(UniTunneledLongNameBuffer);
//     UniTunneledLongName.Buffer = &UniTunneledLongNameBuffer[0];
//
//     //
//     //  Remember the name in case we have to modify the name
//     //  value in the ea.
//     //
//
//     RtlCopyMemory( OldOemName.Buffer,
//                    Fcb->ShortName.Name.Oem.Buffer,
//                    OldOemName.Length );
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Extract information from the Irp to make our life easier
//     //
//
//     FileObject = IrpSp->FileObject;
//     SourceCcb = FileObject->FsContext2;
//     TargetFileObject = IrpSp->Parameters.SetFile.FileObject;
//     ReplaceIfExists = IrpSp->Parameters.SetFile.ReplaceIfExists;
//
//     RtlZeroMemory( &LocalCcb, sizeof(CCB) );
//
//     //
//     //  P H A S E  1:
//     //
//     //  Test if rename is legal.  Only small side-effects are not undone.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Can't rename the root directory
//         //
//
//         if ( NodeType(Fcb) == FAT_NTC_ROOT_DCB ) {
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  Check that we were not given a dcb with open handles beneath
//         //  it.  If there are only UncleanCount == 0 Fcbs beneath us, then
//         //  remove them from the prefix table, and they will just close
//         //  and go away naturally.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//             PFCB BatchOplockFcb;
//             ULONG BatchOplockCount;
//
//             //
//             //  Loop until there are no batch oplocks in the subtree below
//             //  this directory.
//             //
//
//             while (TRUE) {
//
//                 BatchOplockFcb = NULL;
//                 BatchOplockCount = 0;
//
//                 //
//                 //  First look for any UncleanCount != 0 Fcbs, and fail if we
//                 //  find any.
//                 //
//
//                 for ( TempFcb = FatGetNextFcbBottomUp(IrpContext, NULL, Fcb);
//                       TempFcb != Fcb;
//                       TempFcb = FatGetNextFcbBottomUp(IrpContext, TempFcb, Fcb) ) {
//
//                      if ( TempFcb->UncleanCount != 0 ) {
//
//                          //
//                          // If there is a batch oplock on this file then
//                          // increment our count and remember the Fcb if
//                          // this is the first.
//                          //
//
//                          if (FatIsFileOplockable( TempFcb ) &&
//                              (FsRtlCurrentBatchOplock( FatGetFcbOplock(TempFcb) )
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//                               ||
//                               FsRtlCurrentOplockH( FatGetFcbOplock(TempFcb) )
// #endif
//                               )) {
//
//                              BatchOplockCount += 1;
//                              if ( BatchOplockFcb == NULL ) {
//
//                                  BatchOplockFcb = TempFcb;
//                              }
//
//                          } else {
//
//                             try_return( Status = STATUS_ACCESS_DENIED );
//                          }
//                      }
//                 }
//
//                 //
//                 //  If this is not the first pass for rename and the number
//                 //  of batch oplocks has not decreased then give up.
//                 //
//
//                 if ( BatchOplockFcb != NULL ) {
//
//                     if ( (Irp->IoStatus.Information != 0) &&
//                          (BatchOplockCount >= Irp->IoStatus.Information) ) {
//
//                         try_return( Status = STATUS_ACCESS_DENIED );
//                     }
//
//                     //
//                     //  Try to break this batch oplock.
//                     //
//
//                     Irp->IoStatus.Information = BatchOplockCount;
//                     Status = FsRtlCheckOplock( FatGetFcbOplock(BatchOplockFcb),
//                                                Irp,
//                                                IrpContext,
//                                                FatOplockComplete,
//                                                NULL );
//
//                     //
//                     //  If the oplock was already broken then look for more
//                     //  batch oplocks.
//                     //
//
//                     if (Status == STATUS_SUCCESS) {
//
//                         continue;
//                     }
//
//                     //
//                     //  Otherwise the oplock package will post or complete the
//                     //  request.
//                     //
//
//                     try_return( Status = STATUS_PENDING );
//                 }
//
//                 break;
//             }
//
//             //
//             //  Now try to get as many of these file object, and thus Fcbs
//             //  to go away as possible, flushing first, of course.
//             //
//
//             FatPurgeReferencedFileObjects( IrpContext, Fcb, Flush );
//
//             //
//             //  OK, so there are no UncleanCount != 0, Fcbs.  Infact, there
//             //  shouldn't really be any Fcbs left at all, except obstinate
//             //  ones from user mapped sections .... Remove the full file name
//             //  and exact case lfn.
//             //
//
//             for ( TempFcb = FatGetNextFcbBottomUp(IrpContext, NULL, Fcb);
//                   TempFcb != Fcb;
//                   TempFcb = FatGetNextFcbBottomUp(IrpContext, TempFcb, Fcb) ) {
//
//                 FatAcquireExclusiveFcb( IrpContext, TempFcb );
//
//                 if (TempFcb->FullFileName.Buffer != NULL) {
//
//                     ExFreePool( TempFcb->FullFileName.Buffer );
//                     TempFcb->FullFileName.Buffer = NULL;
//                 }
//
//                 FatReleaseFcb( IrpContext, TempFcb );
//             }
//         }
//
//         //
//         //  Check if this is a simple rename or a fully-qualified rename
//         //  In both cases we need to figure out what the TargetDcb, and
//         //  NewName are.
//         //
//
//         if (TargetFileObject == NULL) {
//
//             //
//             //  In the case of a simple rename the target dcb is the
//             //  same as the source file's parent dcb, and the new file name
//             //  is taken from the system buffer
//             //
//
//             PFILE_RENAME_INFORMATION Buffer;
//
//             Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//             TargetDcb = Fcb->ParentDcb;
//
//             NewName.Length = (USHORT) Buffer->FileNameLength;
//             NewName.Buffer = (PWSTR) &Buffer->FileName;
//
//             //
//             //  Make sure the name is of legal length.
//             //
//
//             if (NewName.Length > 255*sizeof(WCHAR)) {
//
//                 try_return( Status = STATUS_OBJECT_NAME_INVALID );
//             }
//
//             RtlCopyUnicodeString(&NewNameCopy,&NewName);
//
//         } else {
//
//             //
//             //  For a fully-qualified rename the target dcb is taken from
//             //  the target file object, which must be on the same vcb as
//             //  the source.
//             //
//
//             PVCB TargetVcb;
//             PCCB TargetCcb;
//
//             if ((FatDecodeFileObject( TargetFileObject,
//                                       &TargetVcb,
//                                       &TargetDcb,
//                                       &TargetCcb ) != UserDirectoryOpen) ||
//                 (TargetVcb != Vcb)) {
//
//                 try_return( Status = STATUS_INVALID_PARAMETER );
//             }
//
//             //
//             //  This name is by definition legal.
//             //
//
//             NewName = *((PUNICODE_STRING)&TargetFileObject->FileName);
//
//             RtlCopyUnicodeString(&NewNameCopy,&NewName);
//
//         }
//
//         //
//         //  We will need an upcased version of the unicode name and the
//         //  old name as well.
//         //
//
//         Status = RtlUpcaseUnicodeString( &NewUpcasedName, &NewName, FALSE );
//
//         if (!NT_SUCCESS(Status)) {
//
//             try_return( Status );
//         }
//
//         FatGetUnicodeNameFromFcb( IrpContext, Fcb, &OldName );
//
//         Status = RtlUpcaseUnicodeString( &OldUpcasedName, &OldName, FALSE );
//
//         if (!NT_SUCCESS(Status)) {
//             try_return(Status);
//         }
//
//         //
//         //  Check if the current name and new name are equal, and the
//         //  DCBs are equal.  If they are then our work is already done.
//         //
//
//         if (TargetDcb == Fcb->ParentDcb) {
//
//             //
//             //  OK, now if we found something then check if it was an exact
//             //  match or just a case match.  If it was an exact match, then
//             //  we can bail here.
//             //
//
//             if (FsRtlAreNamesEqual( &NewName,
//                                     &OldName,
//                                     FALSE,
//                                     NULL )) {
//
//                  try_return( Status = STATUS_SUCCESS );
//             }
//
//             //
//             //  Check now for a case only rename.
//             //
//
//
//             if (FsRtlAreNamesEqual( &NewUpcasedName,
//                                     &OldUpcasedName,
//                                     FALSE,
//                                     NULL )) {
//
//                  CaseOnlyRename = TRUE;
//             }
//
//         } else {
//
//             RenamedAcrossDirectories = TRUE;
//         }
//
//         //
//         //  Upcase the name and convert it to the Oem code page.
//         //
//         //  If the new UNICODE name is already more than 12 characters,
//         //  then we know the Oem name will not be valid
//         //
//
//         if (NewName.Length <= 12*sizeof(WCHAR)) {
//
//             FatUnicodeToUpcaseOem( IrpContext, &NewOemName, &NewName );
//
//             //
//             //  If the name is not valid 8.3, zero the length.
//             //
//
//             if (FatSpaceInName( IrpContext, &NewName ) ||
//                 !FatIsNameShortOemValid( IrpContext, NewOemName, FALSE, FALSE, FALSE)) {
//
//                 NewOemName.Length = 0;
//             }
//
//         } else {
//
//             NewOemName.Length = 0;
//         }
//
//         //
//         //  Look in the tunnel cache for names and timestamps to restore
//         //
//
//         TunneledDataSize = sizeof(LARGE_INTEGER);
//         HaveTunneledInformation = FsRtlFindInTunnelCache( &Vcb->Tunnel,
//                                                           FatDirectoryKey(TargetDcb),
//                                                           &NewName,
//                                                           &UniTunneledShortName,
//                                                           &UniTunneledLongName,
//                                                           &TunneledDataSize,
//                                                           &TunneledCreationTime );
//         NT_ASSERT(TunneledDataSize == sizeof(LARGE_INTEGER));
//
//
//         //
//         //  Now we need to determine how many dirents this new name will
//         //  require.
//         //
//
//         if ((NewOemName.Length == 0) ||
//             (FatEvaluateNameCase( IrpContext,
//                                   &NewNameCopy,
//                                   &AllLowerComponent,
//                                   &AllLowerExtension,
//                                   &CreateLfn ),
//              CreateLfn)) {
//
//             DirentsRequired = FAT_LFN_DIRENTS_NEEDED(&NewNameCopy) + 1;
//
//         } else {
//
//             //
//             //  The user-given name is a short name, but we might still have
//             //  a tunneled long name we want to use. See if we can.
//             //
//
//             if (UniTunneledLongName.Length &&
//                 !FatLfnDirentExists(IrpContext, TargetDcb, &UniTunneledLongName, &TargetLfn)) {
//
//                 UsingTunneledLfn = CreateLfn = TRUE;
//                 DirentsRequired = FAT_LFN_DIRENTS_NEEDED(&UniTunneledLongName) + 1;
//
//             } else {
//
//                 //
//                 //  This really is a simple dirent.  Note that the two AllLower BOOLEANs
//                 //  are correctly set now.
//                 //
//
//                 DirentsRequired = 1;
//
//
//             }
//         }
//
//         //
//         //  Do some extra checks here if we are not in Chicago mode.
//         //
//
//         if (!FatData.ChicagoMode) {
//
//             //
//             //  If the name was not 8.3 valid, fail the rename.
//             //
//
//             if (NewOemName.Length == 0) {
//
//                 try_return( Status = STATUS_OBJECT_NAME_INVALID );
//             }
//
//             //
//             //  Don't use the magic bits.
//             //
//
//             AllLowerComponent = FALSE;
//             AllLowerExtension = FALSE;
//             CreateLfn = FALSE;
//             UsingTunneledLfn = FALSE;
//         }
//
//         if (!CaseOnlyRename) {
//
//             //
//             //  Check if the new name already exists, wait is known to be
//             //  true.
//             //
//
//             if (NewOemName.Length != 0) {
//
//                 FatStringTo8dot3( IrpContext,
//                                   NewOemName,
//                                   &LocalCcb.OemQueryTemplate.Constant );
//
//             } else {
//
//                 SetFlag( LocalCcb.Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE );
//             }
//
//             LocalCcb.UnicodeQueryTemplate = NewUpcasedName;
//             LocalCcb.ContainsWildCards = FALSE;
//
//             Flags = 0;
//
//
//             FatLocateDirent( IrpContext,
//                              TargetDcb,
//                              &LocalCcb,
//                              0,
//                              &Flags,
//                              &TargetDirent,
//                              &TargetDirentBcb,
//                              (PVBO)&TargetDirentOffset,
//                              NULL,
//                              &TargetLfn,
//                              &TargetOrigLfn );
//
//             if (TargetDirent != NULL) {
//
//
//                 //
//                 //  The name already exists, check if the user wants
//                 //  to overwrite the name, and has access to do the overwrite
//                 //  We cannot overwrite a directory.
//                 //
//
//                 if ((!ReplaceIfExists) ||
//                     (FlagOn(TargetDirent->Attributes, FAT_DIRENT_ATTR_DIRECTORY)) ||
//                     (FlagOn(TargetDirent->Attributes, FAT_DIRENT_ATTR_READ_ONLY))) {
//
//                     try_return( Status = STATUS_OBJECT_NAME_COLLISION );
//                 }
//
//                 //
//                 //  Check that the file has no open user handles, if it does
//                 //  then we will deny access.  We do the check by searching
//                 //  down the list of fcbs opened under our parent Dcb, and making
//                 //  sure none of the maching Fcbs have a non-zero unclean count or
//                 //  outstanding image sections.
//                 //
//
//                 for (Links = TargetDcb->Specific.Dcb.ParentDcbQueue.Flink;
//                      Links != &TargetDcb->Specific.Dcb.ParentDcbQueue; ) {
//
//                     TempFcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//                     //
//                     //  Advance now.  The image section flush may cause the final
//                     //  close, which will recursively happen underneath of us here.
//                     //  It would be unfortunate if we looked through free memory.
//                     //
//
//                     Links = Links->Flink;
//
//                     if ((TempFcb->DirentOffsetWithinDirectory == TargetDirentOffset) &&
//                         ((TempFcb->UncleanCount != 0) ||
//                          !MmFlushImageSection( &TempFcb->NonPaged->SectionObjectPointers,
//                                                MmFlushForDelete))) {
//
//                         //
//                         //  If there are batch oplocks on this file then break the
//                         //  oplocks before failing the rename.
//                         //
//
//                         Status = STATUS_ACCESS_DENIED;
//
//                         if (FatIsFileOplockable( TempFcb ) &&
//                             (FsRtlCurrentBatchOplock( FatGetFcbOplock(TempFcb) )
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//                              ||
//                              FsRtlCurrentOplockH( FatGetFcbOplock(TempFcb) )
// #endif
//                              )) {
//
//                             //
//                             //  Do all of our cleanup now since the IrpContext
//                             //  could go away when this request is posted.
//                             //
//
//                             FatUnpinBcb( IrpContext, TargetDirentBcb );
//
//                             Status = FsRtlCheckOplock( FatGetFcbOplock(TempFcb),
//                                                        Irp,
//                                                        IrpContext,
//                                                        FatOplockComplete,
//                                                        NULL );
//
//                             if (Status != STATUS_PENDING) {
//
//                                 Status = STATUS_ACCESS_DENIED;
//                             }
//                         }
//
//                         try_return( NOTHING );
//                     }
//                 }
//
//                 //
//                 //  OK, this target is toast.  Remember the Lfn offset.
//                 //
//
//                 TargetLfnOffset = TargetDirentOffset -
//                                   FAT_LFN_DIRENTS_NEEDED(&TargetOrigLfn) *
//                                   sizeof(DIRENT);
//
//                 DeleteTarget = TRUE;
//             }
//         }
//
//
//         //
//         //  If we will need more dirents than we have, allocate them now.
//         //
//
//         if ((TargetDcb != Fcb->ParentDcb) ||
//             (DirentsRequired !=
//              (Fcb->DirentOffsetWithinDirectory -
//               Fcb->LfnOffsetWithinDirectory) / sizeof(DIRENT) + 1)) {
//
//             //
//             //  Get some new allocation
//             //
//
//             NewOffset = FatCreateNewDirent( IrpContext,
//                                             TargetDcb,
//                                             DirentsRequired,
//                                             FALSE );
//
//             DeleteSourceDirent = TRUE;
//
//         } else {
//
//             NewOffset = Fcb->LfnOffsetWithinDirectory;
//         }
//
//         ContinueWithRename = TRUE;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         if (!ContinueWithRename) {
//
//             //
//             //  Undo everything from above.
//             //
//
//             ExFreePool( UnicodeBuffer );
//             FatUnpinBcb( IrpContext, TargetDirentBcb );
//
//         }
//     } _SEH2_END;
//
//     //
//     //  Now, if we are already done, return here.
//     //
//
//     if (!ContinueWithRename) {
//
//         return Status;
//     }
//
//     //
//     //  P H A S E  2: Actually perform the rename.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Report the fact that we are going to remove this entry.
//         //  If we renamed within the same directory and the new name for the
//         //  file did not previously exist, we report this as a rename old
//         //  name.  Otherwise this is a removed file.
//         //
//
//         if (!RenamedAcrossDirectories && !DeleteTarget) {
//
//             NotifyAction = FILE_ACTION_RENAMED_OLD_NAME;
//
//         } else {
//
//             NotifyAction = FILE_ACTION_REMOVED;
//         }
//
//         FatNotifyReportChange( IrpContext,
//                                Vcb,
//                                Fcb,
//                                ((NodeType( Fcb ) == FAT_NTC_FCB)
//                                 ? FILE_NOTIFY_CHANGE_FILE_NAME
//                                 : FILE_NOTIFY_CHANGE_DIR_NAME ),
//                                NotifyAction );
//
//         _SEH2_TRY {
//
//             //
//             //  Capture a copy of the source dirent.
//             //
//
//             FatGetDirentFromFcbOrDcb( IrpContext, Fcb, FALSE, &OldDirent, &OldDirentBcb );
//
//             Dirent = unsafe { core::ptr::read(OldDirent) };
//
//             //
//             //  Tunnel the source Fcb - the names are disappearing regardless of
//             //  whether the dirent allocation physically changed
//             //
//
//             FatTunnelFcbOrDcb( Fcb, SourceCcb );
//
//             //
//             //  From here until very nearly the end of the operation, if we raise there
//             //  is no reasonable way to suppose we'd be able to undo the damage.  Not
//             //  being a transactional filesystem, FAT is at the mercy of a lot of things
//             //  (as the astute reader has no doubt realized by now).
//             //
//
//             InvalidateFcbOnRaise = TRUE;
//
//             //
//             //  Delete our current dirent(s) if we got a new one.
//             //
//
//             if (DeleteSourceDirent) {
//
//                 FatDeleteDirent( IrpContext, Fcb, NULL, FALSE );
//             }
//
//             //
//             //  Delete a target conflict if we were meant to.
//             //
//
//             if (DeleteTarget) {
//
//                 FatDeleteFile( IrpContext,
//                                TargetDcb,
//                                TargetLfnOffset,
//                                TargetDirentOffset,
//                                TargetDirent,
//                                &TargetLfn );
//             }
//
//             //
//             //  We need to evaluate any short names required.  If there were any
//             //  conflicts in existing short names, they would have been deleted above.
//             //
//             //  It isn't neccesary to worry about the UsingTunneledLfn case. Since we
//             //  actually already know whether CreateLfn will be set either NewName is
//             //  an Lfn and !UsingTunneledLfn is implied or NewName is a short name and
//             //  we can handle that externally.
//             //
//
//             FatSelectNames( IrpContext,
//                             TargetDcb,
//                             &NewOemName,
//                             &NewNameCopy,
//                             &NewOemName,
//                             (HaveTunneledInformation ? &UniTunneledShortName : NULL),
//                             &AllLowerComponent,
//                             &AllLowerExtension,
//                             &CreateLfn );
//
//             if (!CreateLfn && UsingTunneledLfn) {
//
//                 CreateLfn = TRUE;
//                 RtlCopyUnicodeString( &NewNameCopy, &UniTunneledLongName );
//
//                 //
//                 //  Short names are always upcase if an LFN exists
//                 //
//
//                 AllLowerComponent = FALSE;
//                 AllLowerExtension = FALSE;
//             }
//
//             //
//             //  OK, now setup the new dirent(s) for the new name.
//             //
//
//             FatPrepareWriteDirectoryFile( IrpContext,
//                                           TargetDcb,
//                                           NewOffset,
//                                           sizeof(DIRENT),
//                                           &NewDirentBcb,
// #ifndef __REACTOS__
//                                           &NewDirent,
// #else
//                                           (PVOID *)&NewDirent,
// #endif
//                                           FALSE,
//                                           TRUE,
//                                           &Status );
//
//             NT_ASSERT( NT_SUCCESS( Status ) );
//
//             //
//             //  Deal with the special case of an LFN + Dirent structure crossing
//             //  a page boundry.
//             //
//
//             if ((NewOffset / PAGE_SIZE) !=
//                 ((NewOffset + (DirentsRequired - 1) * sizeof(DIRENT)) / PAGE_SIZE)) {
//
//                 SecondPageOffset = (NewOffset & ~(PAGE_SIZE - 1)) + PAGE_SIZE;
//
//                 BytesInFirstPage = SecondPageOffset - NewOffset;
//
//                 DirentsInFirstPage = BytesInFirstPage / sizeof(DIRENT);
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               TargetDcb,
//                                               SecondPageOffset,
//                                               sizeof(DIRENT),
//                                               &SecondPageBcb,
// #ifndef __REACTOS__
//                                               &SecondPageDirent,
// #else
//                                               (PVOID *)&SecondPageDirent,
// #endif
//                                               FALSE,
//                                               TRUE,
//                                               &Status );
//
//                 NT_ASSERT( NT_SUCCESS( Status ) );
//
//                 FirstPageDirent = NewDirent;
//
//                 NewDirent = FsRtlAllocatePoolWithTag( PagedPool,
//                                                       DirentsRequired * sizeof(DIRENT),
//                                                       TAG_DIRENT );
//
//                 NewDirentFromPool = TRUE;
//             }
//
//             //
//             //  Bump up Dirent and DirentOffset
//             //
//
//             ShortDirent = NewDirent + DirentsRequired - 1;
//             ShortDirentOffset = NewOffset + (DirentsRequired - 1) * sizeof(DIRENT);
//
//             //
//             //  Fill in the fields of the dirent.
//             //
//
//             *ShortDirent = Dirent;
//
//             FatConstructDirent( IrpContext,
//                                 ShortDirent,
//                                 &NewOemName,
//                                 AllLowerComponent,
//                                 AllLowerExtension,
//                                 CreateLfn ? &NewNameCopy : NULL,
//                                 Dirent.Attributes,
//                                 FALSE,
//                                 (HaveTunneledInformation ? &TunneledCreationTime : NULL) );
//
//             if (HaveTunneledInformation) {
//
//                 //
//                 //  Need to go in and fix the timestamps in the FCB. Note that we can't use
//                 //  the TunneledCreationTime since the conversions may have failed.
//                 //
//
//                 Fcb->CreationTime = FatFatTimeToNtTime(IrpContext, ShortDirent->CreationTime, ShortDirent->CreationMSec);
//                 Fcb->LastWriteTime = FatFatTimeToNtTime(IrpContext, ShortDirent->LastWriteTime, 0);
//                 Fcb->LastAccessTime = FatFatDateToNtTime(IrpContext, ShortDirent->LastAccessDate);
//             }
//
//
//             //
//             //  If the dirent crossed pages, split the contents of the
//             //  temporary pool between the two pages.
//             //
//
//             if (NewDirentFromPool) {
//
//                 RtlCopyMemory( FirstPageDirent, NewDirent, BytesInFirstPage );
//
//                 RtlCopyMemory( SecondPageDirent,
//                                NewDirent + DirentsInFirstPage,
//                                DirentsRequired*sizeof(DIRENT) - BytesInFirstPage );
//
//                 ShortDirent = SecondPageDirent +
//                               (DirentsRequired - DirentsInFirstPage) - 1;
//             }
//
//             Dirent = unsafe { core::ptr::read(ShortDirent) };
//
//         } _SEH2_FINALLY {
//
//             //
//             //  Remove the entry from the splay table, and then remove the
//             //  full file name and exact case lfn. It is important that we
//             //  always remove the name from the prefix table regardless of
//             //  other errors.
//             //
//
//             FatRemoveNames( IrpContext, Fcb );
//
//             if (Fcb->FullFileName.Buffer != NULL) {
//
//                 ExFreePool( Fcb->FullFileName.Buffer );
//                 Fcb->FullFileName.Buffer = NULL;
//             }
//
//             if (Fcb->ExactCaseLongName.Buffer) {
//
//                 ExFreePool( Fcb->ExactCaseLongName.Buffer );
//                 Fcb->ExactCaseLongName.Buffer = NULL;
//             }
//
//             FatUnpinBcb( IrpContext, OldDirentBcb );
//             FatUnpinBcb( IrpContext, TargetDirentBcb );
//             FatUnpinBcb( IrpContext, NewDirentBcb );
//             FatUnpinBcb( IrpContext, SecondPageBcb );
//         } _SEH2_END;
//
//         //
//         //  Now we need to update the location of the file's directory
//         //  offset and move the fcb from its current parent dcb to
//         //  the target dcb.
//         //
//
//         Fcb->LfnOffsetWithinDirectory = NewOffset;
//         Fcb->DirentOffsetWithinDirectory = ShortDirentOffset;
//
//         RemoveEntryList( &Fcb->ParentDcbLinks );
//
//         //
//         //  There is a deep reason we put files on the tail, others on the head,
//         //  which is to allow us to easily enumerate all child directories before
//         //  child files. This is important to let us maintain whole-volume lockorder
//         //  via BottomUp enumeration.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//             InsertTailList( &TargetDcb->Specific.Dcb.ParentDcbQueue,
//                             &Fcb->ParentDcbLinks );
//
//         } else {
//
//             InsertHeadList( &TargetDcb->Specific.Dcb.ParentDcbQueue,
//                             &Fcb->ParentDcbLinks );
//         }
//
//         OldParentDcb = Fcb->ParentDcb;
//         Fcb->ParentDcb = TargetDcb;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Break parent directory oplock on the old parent.  Directory oplock
//         //  breaks are always advisory, so we will never block/get STATUS_PENDING
//         //  here.
//         //
//
//         FsRtlCheckOplockEx( FatGetFcbOplock(OldParentDcb),
//                             IrpContext->OriginatingIrp,
//                             OPLOCK_FLAG_PARENT_OBJECT,
//                             NULL,
//                             NULL,
//                             NULL );
// #endif
//
//         //
//         //  If we renamed across directories, some cleanup is now in order.
//         //
//
//         if (RenamedAcrossDirectories) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             //
//             //  Break parent directory oplock on the new parent.  Directory oplock
//             //  breaks are always advisory, so we will never block/get STATUS_PENDING
//             //  here.
//             //
//
//             FsRtlCheckOplockEx( FatGetFcbOplock(TargetDcb),
//                                 IrpContext->OriginatingIrp,
//                                 OPLOCK_FLAG_PARENT_OBJECT,
//                                 NULL,
//                                 NULL,
//                                 NULL );
// #endif
//
//             //
//             //  See if we need to uninitialize the cachemap for the source directory.
//             //  Do this now in case we get unlucky and raise trying to finalize the
//             //  operation.
//             //
//
//             if (IsListEmpty(&OldParentDcb->Specific.Dcb.ParentDcbQueue) &&
//                 (OldParentDcb->OpenCount == 0) &&
//                 (OldParentDcb->Specific.Dcb.DirectoryFile != NULL)) {
//
//                 NT_ASSERT( NodeType(OldParentDcb) == FAT_NTC_DCB );
//
//                 DirectoryFileObject = OldParentDcb->Specific.Dcb.DirectoryFile;
//
//                 OldParentDcb->Specific.Dcb.DirectoryFile = NULL;
//             }
//
//             //
//             //  If we move a directory across directories, we have to change
//             //  the cluster number in its .. entry
//             //
//
//             if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               Fcb,
//                                               sizeof(DIRENT),
//                                               sizeof(DIRENT),
//                                               &DotDotBcb,
// #ifndef __REACTOS__
//                                               &DotDotDirent,
// #else
//                                               (PVOID *)&DotDotDirent,
// #endif
//                                               FALSE,
//                                               TRUE,
//                                               &Status );
//
//                 NT_ASSERT( NT_SUCCESS( Status ) );
//
//                 DotDotDirent->FirstClusterOfFile = (USHORT)
//                     ( NodeType(TargetDcb) == FAT_NTC_ROOT_DCB ?
//                       0 : TargetDcb->FirstClusterOfFile);
//
//                 if (FatIsFat32( Vcb )) {
//
//                     DotDotDirent->FirstClusterOfFileHi = (USHORT)
//                     ( NodeType( TargetDcb ) == FAT_NTC_ROOT_DCB ?
//                       0 : (TargetDcb->FirstClusterOfFile >> 16));
//                 }
//             }
//         }
//
//         //
//         //  Now we need to setup the splay table and the name within
//         //  the fcb.  Free the old short name at this point.
//         //
//
//         ExFreePool( Fcb->ShortName.Name.Oem.Buffer );
//         Fcb->ShortName.Name.Oem.Buffer = NULL;
//
//
//         FatConstructNamesInFcb( IrpContext,
//                                 Fcb,
//                                 &Dirent,
//                                 CreateLfn ? &NewName : NULL );
//
//         FatSetFullNameInFcb( IrpContext, Fcb, &NewName );
//
//         //
//         //  The rest of the actions taken are not related to correctness of
//         //  the in-memory structures, so we shouldn't toast the Fcb if we
//         //  raise from here to the end.
//         //
//
//         InvalidateFcbOnRaise = FALSE;
//
//         //
//         //  If a file, set the file as modified so that the archive bit
//         //  is set.  We prevent this from adjusting the write time by
//         //  indicating the user flag in the ccb.
//         //
//
//         if (Fcb->Header.NodeTypeCode == FAT_NTC_FCB) {
//
//             SetFlag( FileObject->Flags, FO_FILE_MODIFIED );
//             SetFlag( Ccb->Flags, CCB_FLAG_USER_SET_LAST_WRITE );
//         }
//
//         //
//         //  We have three cases to report.
//         //
//         //      1.  If we overwrote an existing file, we report this as
//         //          a modified file.
//         //
//         //      2.  If we renamed to a new directory, then we added a file.
//         //
//         //      3.  If we renamed in the same directory, then we report the
//         //          the renamednewname.
//         //
//
//         if (DeleteTarget) {
//
//             FatNotifyReportChange( IrpContext,
//                                    Vcb,
//                                    Fcb,
//                                    FILE_NOTIFY_CHANGE_ATTRIBUTES
//                                    | FILE_NOTIFY_CHANGE_SIZE
//                                    | FILE_NOTIFY_CHANGE_LAST_WRITE
//                                    | FILE_NOTIFY_CHANGE_LAST_ACCESS
//                                    | FILE_NOTIFY_CHANGE_CREATION
//                                    | FILE_NOTIFY_CHANGE_EA,
//                                    FILE_ACTION_MODIFIED );
//
//         } else if (RenamedAcrossDirectories) {
//
//             FatNotifyReportChange( IrpContext,
//                                    Vcb,
//                                    Fcb,
//                                    ((NodeType( Fcb ) == FAT_NTC_FCB)
//                                     ? FILE_NOTIFY_CHANGE_FILE_NAME
//                                     : FILE_NOTIFY_CHANGE_DIR_NAME ),
//                                    FILE_ACTION_ADDED );
//
//         } else {
//
//             FatNotifyReportChange( IrpContext,
//                                    Vcb,
//                                    Fcb,
//                                    ((NodeType( Fcb ) == FAT_NTC_FCB)
//                                     ? FILE_NOTIFY_CHANGE_FILE_NAME
//                                     : FILE_NOTIFY_CHANGE_DIR_NAME ),
//                                    FILE_ACTION_RENAMED_NEW_NAME );
//         }
//
//         //
//         //  We need to update the file name in the dirent.  This value
//         //  is never used elsewhere, so we don't concern ourselves
//         //  with any error we may encounter.  We let chkdsk fix the
//         //  disk at some later time.
//         //
//
//         if (!FatIsFat32(Vcb) &&
//             Dirent.__bindgen_anon_1.bindgen_union_field != 0) {
//
//             FatRenameEAs( IrpContext,
//                           Fcb,
//                           Dirent.__bindgen_anon_1.bindgen_union_field,
//                           &OldOemName );
//         }
//
//         FatUnpinBcb( IrpContext, DotDotBcb );
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//         //
//         //  Set our final status
//         //
//
//         Status = STATUS_SUCCESS;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetRenameInfo );
//
//         ExFreePool( UnicodeBuffer );
//
//         if (UniTunneledLongName.Buffer != UniTunneledLongNameBuffer) {
//
//             //
//             //  Free pool if the buffer was grown on tunneling lookup
//             //
//
//             ExFreePool(UniTunneledLongName.Buffer);
//         }
//
//         if (NewDirentFromPool) {
//
//             ExFreePool( NewDirent );
//         }
//
//         FatUnpinBcb( IrpContext, TargetDirentBcb );
//         FatUnpinBcb( IrpContext, DotDotBcb );
//
//         //
//         //  Uninitialize the cachemap for the source directory if we need to.
//         //
//
//         if (DirectoryFileObject) {
//
//             DebugTrace(0, Dbg, "Uninitialize our parent Stream Cache Map\n", 0);
//
//             CcUninitializeCacheMap( DirectoryFileObject, NULL, NULL );
//
//             ObDereferenceObject( DirectoryFileObject );
//         }
//
//         //
//         //  If this was an abnormal termination, then we are in trouble.
//         //  Should the operation have been in a sensitive state there is
//         //  nothing we can do but invalidate the Fcb.
//         //
//
//         if (_SEH2_AbnormalTermination() && InvalidateFcbOnRaise) {
//
//             Fcb->FcbCondition = FcbBad;
//         }
//
//         DebugTrace(-1, Dbg, "FatSetRenameInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// NTSTATUS
// FatSetPositionInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set position information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     FileObject - Supplies the file object being processed
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     PFILE_POSITION_INFORMATION Buffer;
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatSetPositionInfo...\n", 0);
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Check if the file does not use intermediate buffering.  If it
//     //  does not use intermediate buffering then the new position we're
//     //  supplied must be aligned properly for the device
//     //
//
//     if (FlagOn( FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING )) {
//
//         PDEVICE_OBJECT DeviceObject;
//
//         DeviceObject = IoGetCurrentIrpStackLocation( Irp )->DeviceObject;
//
//         if ((Buffer->CurrentByteOffset.__bindgen_anon_1.LowPart & DeviceObject->AlignmentRequirement) != 0) {
//
//             DebugTrace(0, Dbg, "Cannot set position due to aligment conflict\n", 0);
//             DebugTrace(-1, Dbg, "FatSetPositionInfo -> %08lx\n", STATUS_INVALID_PARAMETER);
//
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     //
//     //  The input parameter is fine so set the current byte offset and
//     //  complete the request
//     //
//
//     DebugTrace(0, Dbg, "Set the new position to %08lx\n", Buffer->CurrentByteOffset);
//
//     FileObject->CurrentByteOffset = Buffer->CurrentByteOffset;
//
//     DebugTrace(-1, Dbg, "FatSetPositionInfo -> %08lx\n", STATUS_SUCCESS);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetAllocationInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFCB Fcb,
//     IN PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set Allocation information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
//     FileObject - Supplies the FileObject being processed, already known not to
//         be the root dcb
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_ALLOCATION_INFORMATION Buffer;
//     ULONG NewAllocationSize = 0;
//     ULONG HeaderSize = 0;
//
//     BOOLEAN FileSizeTruncated = FALSE;
//     BOOLEAN CacheMapInitialized = FALSE;
//     BOOLEAN ResourceAcquired = FALSE;
//     ULONG OriginalFileSize = 0;
//     ULONG OriginalValidDataLength = 0;
//     ULONG OriginalValidDataToDisk = 0;
//
//     let _ = PAGED_CODE;
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     NewAllocationSize = Buffer->AllocationSize.__bindgen_anon_1.LowPart;
//
//     DebugTrace(+1, Dbg, "FatSetAllocationInfo.. to %08lx\n", NewAllocationSize);
//
//     //
//     //  Allocation is only allowed on a file and not a directory
//     //
//
//     if (NodeType(Fcb) == FAT_NTC_DCB) {
//
//         DebugTrace(-1, Dbg, "Cannot change allocation of a directory\n", 0);
//
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     //
//     //  Check that the new file allocation is legal
//     //
//
//
//     if (!FatIsIoRangeValid( Fcb->Vcb, Buffer->AllocationSize, 0)) {
//
//         DebugTrace(-1, Dbg, "Illegal allocation size\n", 0);
//
//         return STATUS_DISK_FULL;
//     }
//
//
//     //
//     //  If we haven't yet looked up the correct AllocationSize, do so.
//     //
//
//     if (Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//         FatLookupFileAllocationSize( IrpContext, Fcb );
//     }
//
//     //
//     //  This is kinda gross, but if the file is not cached, but there is
//     //  a data section, we have to cache the file to avoid a bunch of
//     //  extra work.
//     //
//
//     if ((FileObject->SectionObjectPointer->DataSectionObject != NULL) &&
//         (FileObject->SectionObjectPointer->SharedCacheMap == NULL) &&
//         !FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//
//         NT_ASSERT( !FlagOn( FileObject->Flags, FO_CLEANUP_COMPLETE ) );
//
//         //
//         //  Now initialize the cache map.
//         //
//
//         FatInitializeCacheMap( FileObject,
//                                (PCC_FILE_SIZES)&Fcb->Header.AllocationSize,
//                                FALSE,
//                                &FatData.CacheManagerCallbacks,
//                                Fcb );
//
//         CacheMapInitialized = TRUE;
//     }
//
//     //
//     //  Now mark the fact that the file needs to be truncated on close
//     //
//
//     Fcb->FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
//
//     //
//     //  Now mark that the time on the dirent needs to be updated on close.
//     //
//
//     SetFlag( FileObject->Flags, FO_FILE_MODIFIED );
//
//     _SEH2_TRY {
//
//         //
//         //  Increase or decrease the allocation size.
//         //
//
//         if (NewAllocationSize+HeaderSize > Fcb->Header.AllocationSize.__bindgen_anon_1.LowPart) {
//
//             FatAddFileAllocation( IrpContext, Fcb, FileObject, NewAllocationSize+HeaderSize);
//
//         } else {
//
//             //
//             //  Check here if we will be decreasing file size and synchonize with
//             //  paging IO.
//             //
//
//             if ( Fcb->Header.FileSize.__bindgen_anon_1.LowPart > NewAllocationSize+HeaderSize ) {
//
//                 //
//                 //  The way Sections for DataScan are created and used, an AV's
//                 //  memory-mapping can come into being after we check it's safe
//                 //  to truncate a file while continuing to hold  the file here.
//                 //  This leads to data corruption because Purge eventually fails
//                 //  (during call to Cc to set file sizes) and stale data continues
//                 //  to live in the cache/memory.
//                 //
//
//                 if (Fcb->PurgeFailureModeEnableCount != 0) {
//
//                     try_return( Status = STATUS_PURGE_FAILED );
//                 }
//
//                 //
//                 //  Before we actually truncate, check to see if the purge
//                 //  is going to fail.
//                 //
//
//                 if (!MmCanFileBeTruncated( FileObject->SectionObjectPointer,
//                                            &Buffer->AllocationSize )) {
//
//                     try_return( Status = STATUS_USER_MAPPED_FILE );
//                 }
//
//                 FileSizeTruncated = TRUE;
//
//                 OriginalFileSize = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//                 OriginalValidDataLength = Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart;
//                 OriginalValidDataToDisk = Fcb->ValidDataToDisk;
//
//                 (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE );
//                 ResourceAcquired = TRUE;
//
//                 Fcb->Header.FileSize.__bindgen_anon_1.LowPart = NewAllocationSize;
//
//                 //
//                 //  If we reduced the file size to less than the ValidDataLength,
//                 //  adjust the VDL.  Likewise ValidDataToDisk.
//                 //
//
//                 if (Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart > Fcb->Header.FileSize.__bindgen_anon_1.LowPart) {
//
//                     Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//                 }
//                 if (Fcb->ValidDataToDisk > Fcb->Header.FileSize.__bindgen_anon_1.LowPart) {
//
//                     Fcb->ValidDataToDisk = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//                 }
//
//             }
//
//             //
//             //  Now that File Size is down, actually do the truncate.
//             //
//
//             FatTruncateFileAllocation( IrpContext, Fcb, NewAllocationSize+HeaderSize );
//
//             //
//             //  Now check if we needed to decrease the file size accordingly.
//             //
//
//             if ( FileSizeTruncated ) {
//
//                 //
//                 //  Tell the cache manager we reduced the file size.
//                 //  The call is unconditional, because MM always wants to know.
//                 //
//
// #if DBG
//                 _SEH2_TRY {
// #endif
//
//                     CcSetFileSizes( FileObject, (PCC_FILE_SIZES)&Fcb->Header.AllocationSize );
//
// #if DBG
//                 } _SEH2_EXCEPT(FatBugCheckExceptionFilter( _SEH2_GetExceptionInformation() )) {
//
//                       NOTHING;
//                 } _SEH2_END;
// #endif
//
//                 NT_ASSERT( FileObject->DeleteAccess || FileObject->WriteAccess );
//
//                 //
//                 //  There is no going back from this. If we run into problems updating
//                 //  the dirent we will have to live with the consequences. Not sending
//                 //  the notifies is likewise pretty benign compared to failing the entire
//                 //  operation and trying to back out everything, which could fail for the
//                 //  same reasons.
//                 //
//                 //  If you want a transacted filesystem, use NTFS ...
//                 //
//
//                 FileSizeTruncated = FALSE;
//
//                 FatSetFileSizeInDirent( IrpContext, Fcb, NULL );
//
//                 //
//                 //  Report that we just reduced the file size.
//                 //
//
//                 FatNotifyReportChange( IrpContext,
//                                        Fcb->Vcb,
//                                        Fcb,
//                                        FILE_NOTIFY_CHANGE_SIZE,
//                                        FILE_ACTION_MODIFIED );
//             }
//         }
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         if ( _SEH2_AbnormalTermination() && FileSizeTruncated ) {
//
//             Fcb->Header.FileSize.__bindgen_anon_1.LowPart = OriginalFileSize;
//             Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart = OriginalValidDataLength;
//             Fcb->ValidDataToDisk = OriginalValidDataToDisk;
//
//             //
//             //  Make sure Cc knows the right filesize.
//             //
//
//             if (FileObject->SectionObjectPointer->SharedCacheMap != NULL) {
//
//                 *CcGetFileSizePointer(FileObject) = Fcb->Header.FileSize;
//             }
//
//             NT_ASSERT( Fcb->Header.FileSize.__bindgen_anon_1.LowPart <= Fcb->Header.AllocationSize.__bindgen_anon_1.LowPart );
//         }
//
//         if (CacheMapInitialized) {
//
//             CcUninitializeCacheMap( FileObject, NULL, NULL );
//         }
//
//         if (ResourceAcquired) {
//
//             ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//
//         }
//
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatSetAllocationInfo -> %08lx\n", STATUS_SUCCESS);
//
//     return Status;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetEndOfFileInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PVCB Vcb,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set End of File information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     FileObject - Supplies the file object being processed
//
//     Vcb - Supplies the Vcb being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PFILE_END_OF_FILE_INFORMATION Buffer;
//
//     ULONG NewFileSize = 0;
//     ULONG InitialFileSize = 0;
//     ULONG InitialValidDataLength = 0;
//     ULONG InitialValidDataToDisk = 0;
//
//     BOOLEAN CacheMapInitialized = FALSE;
//     BOOLEAN UnwindFileSizes = FALSE;
//     BOOLEAN ResourceAcquired = FALSE;
//
//
//     let _ = PAGED_CODE;
//
//     DebugTrace(+1, Dbg, "FatSetEndOfFileInfo...\n", 0);
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     _SEH2_TRY {
//
//         //
//         //  File Size changes are only allowed on a file and not a directory
//         //
//
//         if (NodeType(Fcb) != FAT_NTC_FCB) {
//
//             DebugTrace(0, Dbg, "Cannot change size of a directory\n", 0);
//
//             try_return( Status = STATUS_INVALID_DEVICE_REQUEST );
//         }
//
//
//         //
//         //  Check that the new file size is legal
//         //
//
//
//         if (!FatIsIoRangeValid( Fcb->Vcb, Buffer->EndOfFile, 0)) {
//
//             DebugTrace(0, Dbg, "Illegal allocation size\n", 0);
//
//             try_return( Status = STATUS_DISK_FULL );
//         }
//
//         NewFileSize = Buffer->EndOfFile.__bindgen_anon_1.LowPart;
//
//
//         //
//         //  If we haven't yet looked up the correct AllocationSize, do so.
//         //
//
//         if ((Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT)  &&
//             (Fcb->FcbCondition == FcbGood)) {
//
//             FatLookupFileAllocationSize( IrpContext, Fcb );
//         }
//
//         //
//         //  This is kinda gross, but if the file is not cached, but there is
//         //  a data section, we have to cache the file to avoid a bunch of
//         //  extra work.
//         //
//
//         if ((FileObject->SectionObjectPointer->DataSectionObject != NULL) &&
//             (FileObject->SectionObjectPointer->SharedCacheMap == NULL) &&
//             !FlagOn(Irp->Flags, IRP_PAGING_IO)) {
//
//             if (FlagOn( FileObject->Flags, FO_CLEANUP_COMPLETE ))  {
//
//                 //
//                 //  This IRP has raced (and lost) with a close (=>cleanup)
//                 //  on the same fileobject.  We don't want to reinitialise the
//                 //  cachemap here now because we'll leak it (unless we do so &
//                 //  then tear it down again here,  which is too much of a change at
//                 //  this stage).   So we'll just say the file is closed - which
//                 //  is arguably the right thing to do anyway,  since a caller
//                 //  racing operations in this way is broken.  The only stumbling
//                 //  block is possibly filters - do they operate on cleaned
//                 //  up fileobjects?
//                 //
//
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CLOSED);
//             }
//
//             //
//             //  Now initialize the cache map.
//             //
//
//             FatInitializeCacheMap( FileObject,
//                                    (PCC_FILE_SIZES)&Fcb->Header.AllocationSize,
//                                    FALSE,
//                                    &FatData.CacheManagerCallbacks,
//                                    Fcb );
//
//             CacheMapInitialized = TRUE;
//         }
//
//         //
//         //  Do a special case here for the lazy write of file sizes.
//         //
//
//         if (IoGetCurrentIrpStackLocation(Irp)->Parameters.SetFile.AdvanceOnly) {
//
//             //
//             //  Only attempt this if the file hasn't been "deleted on close" and
//             //  this is a good FCB.
//             //
//
//             if (!IsFileDeleted( IrpContext, Fcb ) && (Fcb->FcbCondition == FcbGood)) {
//
//                 PDIRENT Dirent = NULL;
//                 PBCB DirentBcb;
//
//
//                 //
//                 //  Never have the dirent filesize larger than the fcb filesize
//                 //
//
//                 if (NewFileSize >= Fcb->Header.FileSize.__bindgen_anon_1.LowPart) {
//
//                     NewFileSize = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//                 }
//
//                 //
//                 //  Make sure we don't set anything higher than the alloc size.
//                 //
//
//
//                 NT_ASSERT( NewFileSize <= Fcb->Header.AllocationSize.__bindgen_anon_1.LowPart );
//
//
//                 //
//                 //  Only advance the file size, never reduce it with this call
//                 //
//
//                 FatGetDirentFromFcbOrDcb( IrpContext,
//                                           Fcb,
//                                           FALSE,
//                                           &Dirent,
//                                           &DirentBcb );
//                 _SEH2_TRY {
//
//
//                     if ( NewFileSize > Dirent->FileSize ) {
//                         Dirent->FileSize = NewFileSize;
//
//                         FatSetDirtyBcb( IrpContext, DirentBcb, Fcb->Vcb, TRUE );
//
//                         //
//                         //  Report that we just changed the file size.
//                         //
//
//                         FatNotifyReportChange( IrpContext,
//                                                Vcb,
//                                                Fcb,
//                                                FILE_NOTIFY_CHANGE_SIZE,
//                                                FILE_ACTION_MODIFIED );
//
//                     }
//
//                 } _SEH2_FINALLY {
//
//                     FatUnpinBcb( IrpContext, DirentBcb );
//                 } _SEH2_END;
//
//             } else {
//
//                 DebugTrace(0, Dbg, "Cannot set size on deleted file.\n", 0);
//             }
//
//             try_return( Status = STATUS_SUCCESS );
//         }
//
//         //
//         //  Check if the new file size is greater than the current
//         //  allocation size.  If it is then we need to increase the
//         //  allocation size.
//         //
//
//
//         if ( (NewFileSize) > Fcb->Header.AllocationSize.__bindgen_anon_1.LowPart ) {
//
//             //
//             //  Change the file allocation
//             //
//
//             FatAddFileAllocation( IrpContext, Fcb, FileObject, NewFileSize );
//         }
//
//
//         //
//         //  At this point we have enough allocation for the file.
//         //  So check if we are really changing the file size
//         //
//
//         if (Fcb->Header.FileSize.__bindgen_anon_1.LowPart != NewFileSize) {
//
//             if ( NewFileSize < Fcb->Header.FileSize.__bindgen_anon_1.LowPart ) {
//
//                 //
//                 //  The way Sections for DataScan are created and used, an AV's
//                 //  memory-mapping can come into being after we check it's safe
//                 //  to truncate a file while continuing to hold  the file here.
//                 //  This leads to data corruption because Purge eventually fails
//                 //  (during call to Cc to set file sizes) and stale data continues
//                 //  to live in the cache/memory.
//                 //
//
//                 if (Fcb->PurgeFailureModeEnableCount != 0) {
//
//                     try_return( Status = STATUS_PURGE_FAILED );
//                 }
//
//                 //
//                 //  Before we actually truncate, check to see if the purge
//                 //  is going to fail.
//                 //
//
//                 if (!MmCanFileBeTruncated( FileObject->SectionObjectPointer,
//                                            &Buffer->EndOfFile )) {
//
//                     try_return( Status = STATUS_USER_MAPPED_FILE );
//                 }
//
//                 //
//                 //  This call is unconditional, because MM always wants to know.
//                 //  Also serialize here with paging io since we are truncating
//                 //  the file size.
//                 //
//
//                 ResourceAcquired =
//                     ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE );
//             }
//
//             //
//             //  Set the new file size
//             //
//
//             InitialFileSize = Fcb->Header.FileSize.__bindgen_anon_1.LowPart;
//             InitialValidDataLength = Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart;
//             InitialValidDataToDisk = Fcb->ValidDataToDisk;
//             UnwindFileSizes = TRUE;
//
//             Fcb->Header.FileSize.__bindgen_anon_1.LowPart = NewFileSize;
//
//             //
//             //  If we reduced the file size to less than the ValidDataLength,
//             //  adjust the VDL.  Likewise ValidDataToDisk.
//             //
//
//             if (Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart > NewFileSize) {
//
//                 Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart = NewFileSize;
//             }
//
//             if (Fcb->ValidDataToDisk > NewFileSize) {
//
//                 Fcb->ValidDataToDisk = NewFileSize;
//             }
//
//             DebugTrace(0, Dbg, "New file size is 0x%08lx.\n", NewFileSize);
//
//             //
//             //  We must now update the cache mapping (benign if not cached).
//             //
//
//             CcSetFileSizes( FileObject,
//                             (PCC_FILE_SIZES)&Fcb->Header.AllocationSize );
//
//             FatSetFileSizeInDirent( IrpContext, Fcb, NULL );
//
//             //
//             //  Report that we just changed the file size.
//             //
//
//             FatNotifyReportChange( IrpContext,
//                                    Vcb,
//                                    Fcb,
//                                    FILE_NOTIFY_CHANGE_SIZE,
//                                    FILE_ACTION_MODIFIED );
//
//             //
//             //  Mark the fact that the file will need to checked for
//             //  truncation on cleanup.
//             //
//
//             SetFlag( Fcb->FcbState, FCB_STATE_TRUNCATE_ON_CLOSE );
//         }
//
//         //
//         //  Set this handle as having modified the file
//         //
//
//         FileObject->Flags |= FO_FILE_MODIFIED;
//
//         //
//         //  Set our return status to success
//         //
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetEndOfFileInfo );
//
//         if (_SEH2_AbnormalTermination() && UnwindFileSizes) {
//
//             Fcb->Header.FileSize.__bindgen_anon_1.LowPart = InitialFileSize;
//             Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart = InitialValidDataLength;
//             Fcb->ValidDataToDisk = InitialValidDataToDisk;
//
//             if (FileObject->SectionObjectPointer->SharedCacheMap != NULL) {
//
//                 *CcGetFileSizePointer(FileObject) = Fcb->Header.FileSize;
//             }
//
//             //
//             //  WinSE bug #307418 "Occasional data corruption when
//             //  standby/resume while copying files to removable FAT
//             //  formatted media".
//             //  On system suspend FatUnpinRepinnedBcbs() can fail
//             //  because the underlying drive is already marked with DO_VERIFY
//             //  flag. FatUnpinRepinnedBcbs() will raise in this case and
//             //  the file size changes will be un-rolled in FCB but the change
//             //  to Dirent file still can make it to the disk since its BCB
//             //  will not be purged by FatUnpinRepinnedBcbs(). In this case
//             //  we'll also try to un-roll the change to Dirent to keep
//             //  in-memory and on-disk metadata in sync.
//             //
//
//             FatSetFileSizeInDirentNoRaise( IrpContext, Fcb, NULL );
//
//         }
//
//         if (CacheMapInitialized) {
//
//             CcUninitializeCacheMap( FileObject, NULL, NULL );
//         }
//
//         if ( ResourceAcquired ) {
//
//             ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//         }
//
//         DebugTrace(-1, Dbg, "FatSetEndOfFileInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetValidDataLengthInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PFILE_OBJECT FileObject,
//     IN PFCB Fcb,
//     IN PCCB Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the set valid data length information for fat.  It either
//     completes the request or enqueues it off to the fsp.
//
// Arguments:
//
//     Irp - Supplies the irp being processed
//
//     FileObject - Supplies the file object being processed
//
//     Fcb - Supplies the Fcb or Dcb being processed, already known not to
//         be the root dcb
//
//     Ccb - Supplies the Ccb corresponding to the handle opening the source
//         file
//
// Return Value:
//
//     NTSTATUS - The result of this operation if it completes without
//                an exception.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PFILE_VALID_DATA_LENGTH_INFORMATION Buffer;
//
//     ULONG NewValidDataLength;
//     BOOLEAN ResourceAcquired = FALSE;
//
//     let _ = PAGED_CODE;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     DebugTrace(+1, Dbg, "FatSetValidDataLengthInfo...\n", 0);
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     _SEH2_TRY {
//
//         //
//         //  User must have manage volume privilege to explicitly tweak the VDL
//         //
//
//         if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  Valid data length changes are only allowed on a file and not a directory
//         //
//
//         if (NodeType(Fcb) != FAT_NTC_FCB) {
//
//             DebugTrace(0, Dbg, "Cannot change VDL of a directory\n", 0);
//
//             try_return( Status = STATUS_INVALID_DEVICE_REQUEST );
//         }
//
//         //
//         //  Check that the new file size is legal
//         //
//
//
//         if (!FatIsIoRangeValid( Fcb->Vcb, Buffer->ValidDataLength, 0)) {
//
//             DebugTrace(0, Dbg, "Illegal allocation size\n", 0);
//
//             try_return( Status = STATUS_DISK_FULL );
//         }
//
//
//         NewValidDataLength = Buffer->ValidDataLength.__bindgen_anon_1.LowPart;
//
//         //
//         //  VDL can only move forward
//         //
//
//         if ((NewValidDataLength < Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart) ||
//             (NewValidDataLength > Fcb->Header.FileSize.__bindgen_anon_1.LowPart)) {
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  We can't change the VDL without being able to purge. This should stay
//         //  constant since we own everything exclusive
//         //
//
//         if (!MmCanFileBeTruncated( FileObject->SectionObjectPointer,
//                                    &Buffer->ValidDataLength )) {
//
//             try_return( Status = STATUS_USER_MAPPED_FILE );
//         }
//
//         //
//         //  Flush old data out and purge the cache so we can see new data.
//         //
//
//         if (FileObject->SectionObjectPointer->DataSectionObject != NULL) {
//
//             ResourceAcquired =
//                 ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE );
//
//             CcFlushCache( FileObject->SectionObjectPointer,
//                           NULL,
//                           0,
//                           &Irp->IoStatus );
//
//             if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//                 try_return( Irp->IoStatus.Status );
//             }
//
//             CcPurgeCacheSection( FileObject->SectionObjectPointer,
//                                  NULL,
//                                  0,
//                                  FALSE );
//         }
//
//         //
//         //  Set the new ValidDataLength, Likewise ValidDataToDisk.
//         //
//
//         Fcb->Header.ValidDataLength.__bindgen_anon_1.LowPart = NewValidDataLength;
//         Fcb->ValidDataToDisk = NewValidDataLength;
//
//         DebugTrace(0, Dbg, "New VDL is 0x%08lx.\n", NewValidDataLength);
//
//         //
//         //  We must now update the cache mapping.
//         //
//
//         if (FileObject->SectionObjectPointer->SharedCacheMap != NULL) {
//
//             CcSetFileSizes( FileObject,
//                             (PCC_FILE_SIZES)&Fcb->Header.AllocationSize );
//         }
//
//         //
//         //  Set this handle as having modified the file
//         //
//
//         FileObject->Flags |= FO_FILE_MODIFIED;
//
//         //
//         //  Set our return status to success
//         //
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetValidDataLengthInfo );
//
//         if (ResourceAcquired) {
//
//             ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//         }
//
//         DebugTrace(-1, Dbg, "FatSetValidDataLengthInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
//
// //
// //  Internal Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatRenameEAs (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN USHORT ExtendedAttributes,
//     IN POEM_STRING OldOemName
//     )
// {
//     BOOLEAN LockedEaFcb = FALSE;
//
//     PBCB EaBcb = NULL;
//     PDIRENT EaDirent;
//     EA_RANGE EaSetRange;
//     PEA_SET_HEADER EaSetHeader;
//
//     PVCB Vcb;
//
//     let _ = PAGED_CODE;
//
//     RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
//
//     Vcb = Fcb->Vcb;
//
//     _SEH2_TRY {
//
//         //
//         //  Use a try-except to catch any errors.
//         //
//
//         _SEH2_TRY {
//
//
//             //
//             //  Try to get the Ea file object.  Return FALSE on failure.
//             //
//
//             FatGetEaFile( IrpContext,
//                           Vcb,
//                           &EaDirent,
//                           &EaBcb,
//                           FALSE,
//                           FALSE );
//
//             LockedEaFcb = TRUE;
//
//             //
//             //  If we didn't get the file because it doesn't exist, then the
//             //  disk is corrupted.  We do nothing here.
//             //
//
//             if (Vcb->VirtualEaFile != NULL) {
//
//                 //
//                 //  Try to pin down the Ea set header for the index in the
//                 //  dirent.  If the operation doesn't complete, return FALSE
//                 //  from this routine.
//                 //
//
//                 FatReadEaSet( IrpContext,
//                               Vcb,
//                               ExtendedAttributes,
//                               OldOemName,
//                               FALSE,
//                               &EaSetRange );
//
//                 EaSetHeader = (PEA_SET_HEADER) EaSetRange.Data;
//
//                 //
//                 //  We now have the Ea set header for this file.  We simply
//                 //  overwrite the owning file name.
//                 //
//
//                 RtlZeroMemory( EaSetHeader->OwnerFileName, 14 );
//
//                 RtlCopyMemory( EaSetHeader->OwnerFileName,
//                                Fcb->ShortName.Name.Oem.Buffer,
//                                Fcb->ShortName.Name.Oem.Length );
//
//                 FatMarkEaRangeDirty( IrpContext, Vcb->VirtualEaFile, &EaSetRange );
//                 FatUnpinEaRange( IrpContext, &EaSetRange );
//
//                 CcFlushCache( Vcb->VirtualEaFile->SectionObjectPointer, NULL, 0, NULL );
//             }
//
//         } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//             //
//             //  We catch all exceptions that Fat catches, but don't do
//             //  anything with them.
//             //
//         } _SEH2_END;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Unpin the EaDirent and the EaSetHeader if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, EaBcb );
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         //
//         //  Release the Fcb for the Ea file if locked.
//         //
//
//         if (LockedEaFcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->EaFcb );
//         }
//     } _SEH2_END;
//
//     return;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDeleteFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB TargetDcb,
//     IN ULONG LfnOffset,
//     IN ULONG DirentOffset,
//     IN PDIRENT Dirent,
//     IN PUNICODE_STRING Lfn
//     )
// {
//     PFCB Fcb;
//     PLIST_ENTRY Links;
//
//     let _ = PAGED_CODE;
//
//     //
//     //  We can do the replace by removing the other Fcb(s) from
//     //  the prefix table.
//     //
//
//     for (Links = TargetDcb->Specific.Dcb.ParentDcbQueue.Flink;
//          Links != &TargetDcb->Specific.Dcb.ParentDcbQueue;
//          Links = Links->Flink) {
//
//         Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//         if (FlagOn(Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE) &&
//             (Fcb->DirentOffsetWithinDirectory == DirentOffset)) {
//
//             NT_ASSERT( NodeType(Fcb) == FAT_NTC_FCB );
//             NT_ASSERT( Fcb->LfnOffsetWithinDirectory == LfnOffset );
//
//             if ( Fcb->UncleanCount != 0 ) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck(0,0,0);
//
//             } else {
//
//                 PERESOURCE Resource;
//
//                 //
//                 //  Make this fcb "appear" deleted, synchronizing with
//                 //  paging IO.
//                 //
//
//                 FatRemoveNames( IrpContext, Fcb );
//
//                 Resource = Fcb->Header.PagingIoResource;
//
//                 (VOID)ExAcquireResourceExclusiveLite( Resource, TRUE );
//
//                 SetFlag(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE);
//
//                 Fcb->ValidDataToDisk = 0;
//                 Fcb->Header.FileSize.QuadPart =
//                 Fcb->Header.ValidDataLength.QuadPart = 0;
//
//                 Fcb->FirstClusterOfFile = 0;
//
//                 ExReleaseResourceLite( Resource );
//             }
//         }
//     }
//
//     //
//     //  The file is not currently opened so we can delete the file
//     //  that is being overwritten.  To do the operation we dummy
//     //  up an fcb, truncate allocation, delete the fcb, and delete
//     //  the dirent.
//     //
//
//     Fcb = FatCreateFcb( IrpContext,
//                         TargetDcb->Vcb,
//                         TargetDcb,
//                         LfnOffset,
//                         DirentOffset,
//                         Dirent,
//                         Lfn,
//                         NULL,
//                         FALSE,
//                         FALSE );
//
//     Fcb->Header.FileSize.__bindgen_anon_1.LowPart = 0;
//
//     _SEH2_TRY {
//
//         FatTruncateFileAllocation( IrpContext, Fcb, 0 );
//
//         FatDeleteDirent( IrpContext, Fcb, NULL, TRUE );
//
//     } _SEH2_FINALLY {
//
//         FatDeleteFcb( IrpContext, &Fcb );
//     } _SEH2_END;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// FatFsdQueryInformation — FSD dispatch entry (translated from commented-out C)
#[no_mangle]
pub unsafe extern "system" fn FatFsdQueryInformation(
    _VolumeDeviceObject: *mut core::ffi::c_void,
    Irp: *mut core::ffi::c_void,
) -> i32 {
    let mut Status: i32 = 0;
    let mut IrpContext: *mut core::ffi::c_void = core::ptr::null_mut();
    let TopLevel: i32;
    DebugTrace(1, Dbg, b"FatFsdQueryInformation\n\0".as_ptr() as *const i8, 0i32);
    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp) as i32;
    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const *mut core::ffi::c_void, || {
        *p_irp_context = FatCreateIrpContext(Irp, CanFsdWait(Irp));
        *p_status = FatCommonQueryInformation(*p_irp_context as PIRP_CONTEXT, Irp as PIRP);
    }) {
        Status = FatProcessException(IrpContext, Irp, code as u32);
    }
    if TopLevel != 0 { IoSetTopLevelIrp(core::ptr::null_mut()); }
    FsRtlExitFileSystem();
    DebugTrace(-1, Dbg, b"FatFsdQueryInformation -> %08lx\n\0".as_ptr() as *const i8, Status as u32);
    Status
}
