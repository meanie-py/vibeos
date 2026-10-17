#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::base::{NTSTATUS, BOOLEAN, ULONG, PULONG, LONGLONG, STATUS_SUCCESS};
// One declaration per external symbol: ddk_shims.rs.
use crate::ddk_shims::FlagOn;
// One declaration per external symbol: types.rs.
use crate::types::CdDecodeFileObject;
// The crate's own functions: the definition is the declaration.
use crate::cddata::CdCompleteRequest;
use crate::namesup::CdGenerate8dot3Name;
use crate::namesup::CdIs8dot3Name;
use crate::ddk_shims::CdRaiseStatus;
use crate::dirsup::CdUpdateDirentFromRawDirent;
use crate::verfysup::CdVerifyFcbOperation;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_BUFFER_OVERFLOW, STATUS_OBJECT_NAME_NOT_FOUND};
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::ddk::{ExAcquireResourceSharedLite, ExReleaseResourceLite, FILE_ACCESS_INFORMATION, FILE_ALIGNMENT_INFORMATION, FILE_BASIC_INFORMATION, FILE_EA_INFORMATION, FILE_INTERNAL_INFORMATION, FILE_MODE_INFORMATION, FILE_NETWORK_OPEN_INFORMATION, FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, LARGE_INTEGER, PDEVICE_OBJECT, PFILE_BASIC_INFORMATION, PFILE_INTERNAL_INFORMATION, PFILE_NETWORK_OPEN_INFORMATION, PFILE_OBJECT, PFILE_POSITION_INFORMATION, PFILE_STANDARD_INFORMATION, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP};
use ntkrnl_sys::ddk::{
    _FILE_INFORMATION_CLASS_FileAllInformation as FileAllInformation,
    _FILE_INFORMATION_CLASS_FileAlternateNameInformation as FileAlternateNameInformation,
    _FILE_INFORMATION_CLASS_FileBasicInformation as FileBasicInformation,
    _FILE_INFORMATION_CLASS_FileEaInformation as FileEaInformation,
    _FILE_INFORMATION_CLASS_FileInternalInformation as FileInternalInformation,
    _FILE_INFORMATION_CLASS_FileNameInformation as FileNameInformation,
    _FILE_INFORMATION_CLASS_FileNetworkOpenInformation as FileNetworkOpenInformation,
    _FILE_INFORMATION_CLASS_FilePositionInformation as FilePositionInformation,
    _FILE_INFORMATION_CLASS_FileStandardInformation as FileStandardInformation,
};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, CCB_FLAG_OPEN_BY_ID, CCB_FLAG_OPEN_WITH_VERSION, CD_NAME, COMPOUND_PATH_ENTRY, DIRENT, DIRENT_ENUM_CONTEXT, FCB, FCB_INDEX, FILE_ENUM_CONTEXT, FILE_ID, IRP_CONTEXT, PATH_ENTRY, PCCB, PFCB, PIRP_CONTEXT, TYPE_OF_OPEN, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen};
use ntkrnl_sys::ddk::{FILE_ALL_INFORMATION, FILE_NAME_INFORMATION, FILE_OBJECT, IO_STACK_LOCATION, _UNICODE_STRING, _FSRTL_ADVANCED_FCB_HEADER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use crate::glue::{STATUS_DISK_CORRUPT_ERROR, FILE_ATTRIBUTE_DIRECTORY};

// FCB_STATE_INITIALIZED — cdstruc.h: (0x00000001)
const FCB_STATE_INITIALIZED: ULONG = 0x00000001;
// FO_NO_INTERMEDIATE_BUFFERING — iotypes.h: 0x00000008
const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;

// Accessor for the unnamed FSRTL_ADVANCED_FCB_HEADER spliced into the FCB
// (C: Fcb->AllocationSize / Fcb->FileSize / Fcb->Resource / Fcb->IsFastIoPossible).
#[inline(always)]
unsafe fn cd_fcb_header(Fcb: *mut FCB) -> *mut _FSRTL_ADVANCED_FCB_HEADER {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_1) as *mut _FSRTL_ADVANCED_FCB_HEADER
}
// Accessor for the unnamed FCB_INDEX spliced into the FCB (C: Fcb->Ordinal etc.).
#[inline(always)]
unsafe fn cd_fcb_index(Fcb: *mut FCB) -> *mut FCB_INDEX {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX
}

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
}
extern "C" {
    fn CdFastDecodeFileObject(FileObject: PFILE_OBJECT, Fcb: *mut PFCB) -> u32; // TYPE_OF_OPEN
}

extern "C" {
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
}
extern "C" {
}
extern "C" {
    fn CdAcquireFileShared(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
}
extern "C" {
    fn CdLockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdUnlockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdReleaseFile(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdVerifyOrCreateDirStreamFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdFidIsDirectory(FileId: FILE_ID) -> BOOLEAN;
}
extern "C" {
    fn CdInitializeCompoundPathEntry(IrpContext: *mut IRP_CONTEXT, Entry: *mut COMPOUND_PATH_ENTRY);
}
extern "C" {
    fn CdInitializeFileContext(IrpContext: *mut IRP_CONTEXT, Context: *mut FILE_ENUM_CONTEXT);
}
extern "C" {
    fn CdLookupPathEntry( IrpContext: *mut IRP_CONTEXT, Offset: ULONG, Ordinal: ULONG, SomeFlag: BOOLEAN, Entry: *mut COMPOUND_PATH_ENTRY, );
}
extern "C" {
    fn CdUpdatePathEntryName(IrpContext: *mut IRP_CONTEXT, PathEntry: *mut PATH_ENTRY, Flag: u32);
}
extern "C" {
    fn CdFindDirectory( IrpContext: *mut IRP_CONTEXT, ParentFcb: *mut FCB, Name: *mut CD_NAME, Flag: BOOLEAN, FileContext: *mut FILE_ENUM_CONTEXT, ) -> BOOLEAN;
}
extern "C" {
}
extern "C" {
    fn CdInitializeDirContext(IrpContext: *mut IRP_CONTEXT, Context: *mut DIRENT_ENUM_CONTEXT);
}
extern "C" {
    fn CdInitializeDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    fn CdLookupDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Offset: ULONG, Context: *mut DIRENT_ENUM_CONTEXT, );
}
extern "C" {
}
extern "C" {
    fn CdUpdateDirentName(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT, Flag: u32);
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdCleanupDirContext(IrpContext: *mut IRP_CONTEXT, Context: *mut DIRENT_ENUM_CONTEXT);
}
extern "C" {
    fn CdCleanupDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    fn CdCleanupCompoundPathEntry(IrpContext: *mut IRP_CONTEXT, Entry: *mut COMPOUND_PATH_ENTRY);
}
extern "C" {
    fn CdCleanupFileContext(IrpContext: *mut IRP_CONTEXT, Context: *mut FILE_ENUM_CONTEXT);
}
extern "C" {
    fn CdQueryFidPathTableOffset(FileId: FILE_ID) -> ULONG;
}
extern "C" {
    fn CdQueryFidDirentOffset(FileId: FILE_ID) -> ULONG;
}
extern "C" {
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = 0x000d0000; // CDFS_BUG_CHECK_FILEINFO

// The translated function
#[no_mangle]
pub unsafe extern "C" fn CdCommonQueryInfo(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // Local variables
    let mut Status: NTSTATUS = STATUS_SUCCESS; // STATUS_SUCCESS from ntkrnl_sys::base
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp as *mut _);
    let initial_length: u32 = (*IrpSp).Parameters.QueryFile.Length;
    let mut Length: u32 = initial_length;
    let FileInformationClass: u32 = (*IrpSp).Parameters.QueryFile.FileInformationClass;
    let Buffer: *mut FILE_ALL_INFORMATION = (*Irp).AssociatedIrp.SystemBuffer as *mut _;

    let TypeOfOpen: u32;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let mut ReleaseFcb: u8 = 0; // BOOLEAN FALSE

    // PAGED_CODE();  (ignored)

    // Decode file object
    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        (*IrpSp).FileObject,
        &mut Fcb,
        &mut Ccb,
    );

    // SEH2_TRY .. SEH2_FINALLY .. SEH2_END emulation
    // The try block will be enclosed in a loop to allow a clean break to the finally block.
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_ReleaseFcb = core::ptr::addr_of_mut!(ReleaseFcb);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'try_block: loop {
                // Outer switch on TypeOfOpen (emulated with a loop containing a match)
                'outer_switch: loop {
                    match TypeOfOpen {
                        UserDirectoryOpen | UserFileOpen => {
                            // Acquire shared access
                            CdAcquireFileShared(IrpContext, Fcb);
                            ReleaseFcb = 1; // TRUE

                            // Make sure directory sizes are correct
                            if FlagOn((*Fcb).FcbState, FCB_STATE_INITIALIZED) == 0 {
                                // NT_ASSERT macro
                                NT_ASSERT!((TypeOfOpen == UserDirectoryOpen) as i32);
                                CdVerifyOrCreateDirStreamFile(IrpContext, Fcb);
                            }

                            // Verify Fcb is usable
                            CdVerifyFcbOperation(IrpContext, Fcb);

                            // Inner switch on FileInformationClass
                            match FileInformationClass {
                                FileAllInformation => {
                                    // Do not allow operation on file opened by ID
                                    if FlagOn((*Ccb).Flags, CCB_FLAG_OPEN_BY_ID) != 0 {
                                        Status = STATUS_INVALID_PARAMETER;
                                        break; // exit inner match
                                    }

                                    Length -= (core::mem::size_of::<FILE_ACCESS_INFORMATION>()
                                        + core::mem::size_of::<FILE_MODE_INFORMATION>()
                                        + core::mem::size_of::<FILE_ALIGNMENT_INFORMATION>()) as u32;

                                    CdQueryBasicInfo(
                                        IrpContext,
                                        Fcb,
                                        core::ptr::addr_of_mut!((*Buffer).BasicInformation) as *mut FILE_BASIC_INFORMATION,
                                        &mut Length,
                                    );
                                    CdQueryStandardInfo(
                                        IrpContext,
                                        Fcb,
                                        core::ptr::addr_of_mut!((*Buffer).StandardInformation) as *mut FILE_STANDARD_INFORMATION,
                                        &mut Length,
                                    );
                                    CdQueryInternalInfo(
                                        IrpContext,
                                        Fcb,
                                        core::ptr::addr_of_mut!((*Buffer).InternalInformation) as *mut FILE_INTERNAL_INFORMATION,
                                        &mut Length,
                                    );
                                    CdQueryEaInfo(
                                        IrpContext,
                                        Fcb,
                                        core::ptr::addr_of_mut!((*Buffer).EaInformation) as *mut FILE_EA_INFORMATION,
                                        &mut Length,
                                    );
                                    CdQueryPositionInfo(
                                        IrpContext,
                                        (*IrpSp).FileObject,
                                        core::ptr::addr_of_mut!((*Buffer).PositionInformation) as *mut FILE_POSITION_INFORMATION,
                                        &mut Length,
                                    );
                                    Status = CdQueryNameInfo(
                                        IrpContext,
                                        (*IrpSp).FileObject,
                                        core::ptr::addr_of_mut!((*Buffer).NameInformation) as *mut FILE_NAME_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FileBasicInformation => {
                                    CdQueryBasicInfo(
                                        IrpContext,
                                        Fcb,
                                        Buffer as *mut FILE_BASIC_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FileStandardInformation => {
                                    CdQueryStandardInfo(
                                        IrpContext,
                                        Fcb,
                                        Buffer as *mut FILE_STANDARD_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FileInternalInformation => {
                                    CdQueryInternalInfo(
                                        IrpContext,
                                        Fcb,
                                        Buffer as *mut FILE_INTERNAL_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FileEaInformation => {
                                    CdQueryEaInfo(
                                        IrpContext,
                                        Fcb,
                                        Buffer as *mut FILE_EA_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FilePositionInformation => {
                                    CdQueryPositionInfo(
                                        IrpContext,
                                        (*IrpSp).FileObject,
                                        Buffer as *mut FILE_POSITION_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                FileNameInformation => {
                                    if FlagOn((*Ccb).Flags, CCB_FLAG_OPEN_BY_ID) == 0 {
                                        Status = CdQueryNameInfo(
                                            IrpContext,
                                            (*IrpSp).FileObject,
                                            Buffer as *mut FILE_NAME_INFORMATION,
                                            &mut Length,
                                        );
                                    } else {
                                        Status = STATUS_INVALID_PARAMETER;
                                    }
                                }
                                FileAlternateNameInformation => {
                                    if FlagOn((*Ccb).Flags, CCB_FLAG_OPEN_BY_ID) == 0 {
                                        Status = CdQueryAlternateNameInfo(
                                            IrpContext,
                                            Fcb,
                                            Ccb,
                                            Buffer as *mut FILE_NAME_INFORMATION,
                                            &mut Length,
                                        );
                                    } else {
                                        Status = STATUS_INVALID_PARAMETER;
                                    }
                                }
                                FileNetworkOpenInformation => {
                                    CdQueryNetworkInfo(
                                        IrpContext,
                                        Fcb,
                                        Buffer as *mut FILE_NETWORK_OPEN_INFORMATION,
                                        &mut Length,
                                    );
                                }
                                _ => {
                                    Status = STATUS_INVALID_PARAMETER;
                                }
                            } // end inner match

                            // break outer switch after inner switch has executed (corresponds to C break; after inner switch)
                            break 'outer_switch;
                        }
                        _ => {
                            Status = STATUS_INVALID_PARAMETER;
                            break 'outer_switch;
                        }
                    } // end match TypeOfOpen
                } // end 'outer_switch loop

                // Set information field (after outer switch, inside try)
                (*Irp).IoStatus.Information = (initial_length - Length) as _;

                // exit try block to run finally
                break 'try_block;
            } // end 'try loop
        },
        |_, _| {
            if (*p_ReleaseFcb) != 0 {
                CdReleaseFile(IrpContext, Fcb);
            }
        },
    );


    // _SEH2_END done

    // Complete the request and return status
    CdCompleteRequest((IrpContext).cast(), Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdCommonSetInfo(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    PAGED_CODE!();

    TypeOfOpen = CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb);

    if (TypeOfOpen != UserFileOpen)
        || ((*IrpSp).Parameters.QueryFile.FileInformationClass != FilePositionInformation)
    {
        CdCompleteRequest((IrpContext).cast(), Irp, Status);
        return Status;
    }

    CdAcquireFileShared(IrpContext, Fcb);

    // _SEH2_TRY
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'se2_try: loop {
                CdVerifyFcbOperation(IrpContext, Fcb);

                let Buffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_POSITION_INFORMATION;

                let file_object = (*IrpSp).FileObject;
                let flags = (*file_object).Flags;

                // FlagOn(FO_NO_INTERMEDIATE_BUFFERING) check + alignment check
                if (flags & FO_NO_INTERMEDIATE_BUFFERING) != 0
                    && ((*Buffer).CurrentByteOffset.__bindgen_anon_1.LowPart
                        & (*(*Fcb).Vcb).BlockMask)
                        != 0
                {
                    // try_return(NOTHING) -> goto try_exit, which is after the loop body
                    break 'se2_try;
                }

                CdLockFcb(IrpContext, Fcb);
                // FileObject->CurrentByteOffset = Buffer->CurrentByteOffset;
                core::ptr::copy_nonoverlapping::<LARGE_INTEGER>(
                    &(*Buffer).CurrentByteOffset as *const LARGE_INTEGER,
                    &mut (*file_object).CurrentByteOffset as *mut LARGE_INTEGER,
                    1,
                );
                CdUnlockFcb(IrpContext, Fcb);

                Status = STATUS_SUCCESS;
                // try_exit: NOTHING;
                break 'se2_try; // normal exit to finally
            }
        },
        |_, _| {
            CdReleaseFile(IrpContext, Fcb);
        },
    );

    // _SEH2_END

    CdCompleteRequest((IrpContext).cast(), Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn CdFastQueryBasicInfo(
    FileObject: PFILE_OBJECT,
    Wait: i32, // BOOLEAN
    Buffer: PFILE_BASIC_INFORMATION,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> i32 {
    let mut Result: i32 = 0;
    let TypeOfOpen: u32;
    let mut Fcb: PFCB = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(DeviceObject as *mut core::ffi::c_void);
    ASSERT_FILE_OBJECT!(FileObject);
    FsRtlEnterFileSystem();

    TypeOfOpen = CdFastDecodeFileObject(FileObject, core::ptr::addr_of_mut!(Fcb));

    if TypeOfOpen != UserFileOpen && (TypeOfOpen != UserDirectoryOpen || FlagOn((*Fcb).FcbState, FCB_STATE_INITIALIZED) == 0) {
        FsRtlExitFileSystem();
        return 0;
    }

    if ExAcquireResourceSharedLite((*cd_fcb_header(Fcb)).Resource, Wait as BOOLEAN) == 0 {
        FsRtlExitFileSystem();
        return 0;
    }

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) != 0 {
                let creation: i64 = (*Fcb).CreationTime;
                (*Buffer).CreationTime.QuadPart = creation;
                (*Buffer).LastWriteTime.QuadPart = creation;
                (*Buffer).ChangeTime.QuadPart = creation;
                (*Buffer).LastAccessTime.QuadPart = 0;
                (*Buffer).FileAttributes = (*Fcb).FileAttributes;
                (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
                (*IoStatus).Information = core::mem::size_of::<FILE_BASIC_INFORMATION>() as _;
                Result = 1;
            }
        },
        |_, _| {
            ExReleaseResourceLite((*cd_fcb_header(Fcb)).Resource);
            FsRtlExitFileSystem();
        },
    );

    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn CdFastQueryStdInfo(
    file_object: PFILE_OBJECT,
    wait: u8,
    buffer: PFILE_STANDARD_INFORMATION,
    io_status: PIO_STATUS_BLOCK,
    _device_object: PDEVICE_OBJECT,
) -> u8 {
    let mut result: u8 = 0;
    let mut fcb: PFCB = core::ptr::null_mut();

    // PAGED_CODE();
    // UNREFERENCED_PARAMETER( device_object );

    ASSERT_FILE_OBJECT!(file_object);

    FsRtlEnterFileSystem();

    let type_of_open = CdFastDecodeFileObject(file_object, core::ptr::addr_of_mut!(fcb));

    if (type_of_open != UserFileOpen) &&
       ((type_of_open != UserDirectoryOpen) || FlagOn((*fcb).FcbState, FCB_STATE_INITIALIZED) == 0) {
        FsRtlExitFileSystem();
        return 0;
    }

    if ExAcquireResourceSharedLite((*cd_fcb_header(fcb)).Resource, wait as BOOLEAN) == 0 {
        FsRtlExitFileSystem();
        return 0;
    }

    // _SEH2_TRY block
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            {
                if CdVerifyFcbOperation(core::ptr::null_mut(), fcb) != 0 {
                    if FlagOn((*fcb).FileAttributes, FILE_ATTRIBUTE_DIRECTORY) != 0 {
                        (*buffer).AllocationSize.QuadPart = 0;
                        (*buffer).EndOfFile.QuadPart = 0;
                        (*buffer).Directory = 1;
                    } else {
                        (*buffer).AllocationSize.QuadPart = (*cd_fcb_header(fcb)).AllocationSize.QuadPart;
                        (*buffer).EndOfFile.QuadPart = (*cd_fcb_header(fcb)).FileSize.QuadPart;
                        (*buffer).Directory = 0;
                    }

                    (*buffer).NumberOfLinks = 1;
                    (*buffer).DeletePending = 0;

                    (*io_status).__bindgen_anon_1.Status = STATUS_SUCCESS;
                    (*io_status).Information = core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _;

                    result = 1;
                }
            }
        },
        |_, _| {
            ExReleaseResourceLite((*cd_fcb_header(fcb)).Resource);
            FsRtlExitFileSystem();
        },
    );


    result
}
#[no_mangle]
pub unsafe extern "system" fn CdFastQueryNetworkInfo(
    FileObject: PFILE_OBJECT,
    Wait: i32,
    Buffer: PFILE_NETWORK_OPEN_INFORMATION,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> i32 {
    let mut result: i32 = 0;
    let type_of_open: u32;
    let mut fcb: PFCB = core::ptr::null_mut();

    let _ = DeviceObject;

    ASSERT_FILE_OBJECT!(FileObject);

    FsRtlEnterFileSystem();

    type_of_open = CdFastDecodeFileObject(FileObject, &mut fcb);

    if (type_of_open != UserFileOpen)
        && !(type_of_open == UserDirectoryOpen && FlagOn((*fcb).FcbState, FCB_STATE_INITIALIZED) != 0)
    {
        FsRtlExitFileSystem();
        return 0;
    }

    if ntkrnl_sys::ddk::ExAcquireResourceSharedLite((*cd_fcb_header(fcb)).Resource, Wait as BOOLEAN) == 0 {
        FsRtlExitFileSystem();
        return 0;
    }

    // _SEH2_TRY
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            loop {
                if CdVerifyFcbOperation(core::ptr::null_mut(), fcb) != 0 {
                    let time = (*fcb).CreationTime;
                    (*Buffer).CreationTime.QuadPart = time;
                    (*Buffer).LastWriteTime.QuadPart = time;
                    (*Buffer).ChangeTime.QuadPart = time;
                    (*Buffer).LastAccessTime.QuadPart = 0;

                    (*Buffer).FileAttributes = (*fcb).FileAttributes;

                    if FlagOn((*fcb).FileAttributes, FILE_ATTRIBUTE_DIRECTORY) != 0 {
                        (*Buffer).AllocationSize.QuadPart = 0;
                        (*Buffer).EndOfFile.QuadPart = 0;
                    } else {
                        (*Buffer).AllocationSize.QuadPart = (*cd_fcb_header(fcb)).AllocationSize.QuadPart;
                        (*Buffer).EndOfFile.QuadPart = (*cd_fcb_header(fcb)).FileSize.QuadPart;
                    }

                    (*IoStatus).__bindgen_anon_1.Status = ntkrnl_sys::base::STATUS_SUCCESS;
                    (*IoStatus).Information = core::mem::size_of::<ntkrnl_sys::ddk::FILE_NETWORK_OPEN_INFORMATION>() as _;
                    result = 1;
                }
                break;
            }
        },
        |_, _| {
            ntkrnl_sys::ddk::ExReleaseResourceLite((*cd_fcb_header(fcb)).Resource);
            FsRtlExitFileSystem();
        },
    );


    result
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryBasicInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Buffer: *mut ntkrnl_sys::ddk::FILE_BASIC_INFORMATION,
    Length: *mut u32,
) {
    // PAGED_CODE();
    let _ = IrpContext;
    core::ptr::write_bytes(Buffer as *mut u8, 0, core::mem::size_of::<ntkrnl_sys::ddk::FILE_BASIC_INFORMATION>());
    let creation: LONGLONG = (*Fcb).CreationTime;
    (*Buffer).CreationTime.QuadPart = creation;
    (*Buffer).LastWriteTime.QuadPart = creation;
    (*Buffer).ChangeTime.QuadPart = creation;
    (*Buffer).LastAccessTime.QuadPart = 0;
    (*Buffer).FileAttributes = (*Fcb).FileAttributes;
    *Length -= core::mem::size_of::<ntkrnl_sys::ddk::FILE_BASIC_INFORMATION>() as u32;
}

#[no_mangle]
pub unsafe extern "C" fn CdQueryStandardInfo(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    Buffer: *mut FILE_STANDARD_INFORMATION,
    Length: *mut u32,
) {
    // PAGED_CODE();  (ignored)
    let _ = IrpContext;  // UNREFERENCED_PARAMETER

    // There is only one link and delete is never pending on a Cdrom file.
    (*Buffer).NumberOfLinks = 1;
    (*Buffer).DeletePending = 0;

    // We get the sizes from the header.  Return a size of zero
    // for all directories.
    if FlagOn((*Fcb).FileAttributes, FILE_ATTRIBUTE_DIRECTORY) != 0 {
        (*Buffer).AllocationSize.QuadPart = 0;
        (*Buffer).EndOfFile.QuadPart = 0;
        (*Buffer).Directory = 1;
    } else {
        (*Buffer).AllocationSize.QuadPart = (*cd_fcb_header(Fcb)).AllocationSize.QuadPart;
        (*Buffer).EndOfFile.QuadPart = (*cd_fcb_header(Fcb)).FileSize.QuadPart;
        (*Buffer).Directory = 0;
    }

    // Update the length and status output variables
    *Length = (*Length).wrapping_sub(core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32);

    return;
}

#[no_mangle]
pub unsafe extern "C" fn CdQueryInternalInfo(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    Buffer: PFILE_INTERNAL_INFORMATION,
    Length: *mut u32,
) {
    unsafe {
        // PAGED_CODE();
        let _ = IrpContext; // UNREFERENCED_PARAMETER(IrpContext);
        (*Buffer).IndexNumber = (*Fcb).FileId;
        *Length -= core::mem::size_of::<FILE_INTERNAL_INFORMATION>() as u32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryEaInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Buffer: *mut FILE_EA_INFORMATION,
    Length: *mut u32,
) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);
    UNREFERENCED_PARAMETER!(Fcb as *mut core::ffi::c_void);
    (*Buffer).EaSize = 0;
    *Length -= core::mem::size_of::<FILE_EA_INFORMATION>() as u32;
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryPositionInfo(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: PFILE_OBJECT,
    Buffer: PFILE_POSITION_INFORMATION,
    Length: PULONG,
) {
    let _ = IrpContext;
    (*Buffer).CurrentByteOffset = (*FileObject).CurrentByteOffset;
    *Length = (*Length).wrapping_sub(core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32);
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryNameInfo(
    irp_context: *mut IRP_CONTEXT,
    file_object: *mut FILE_OBJECT,
    buffer: *mut FILE_NAME_INFORMATION,
    length: *mut u32,
) -> i32 {
    unsafe {
        // PAGED_CODE() macro
        PAGED_CODE!();

        // UNREFERENCED_PARAMETER(IrpContext)
        let _ = irp_context;

        // NT_ASSERT(*Length >= sizeof(ULONG))
        assert!(*length as usize >= core::mem::size_of::<u32>());

        let mut status: i32 = STATUS_SUCCESS;
        let mut length_to_copy: u32;

        // Buffer->FileNameLength = LengthToCopy = FileObject->FileName.Length
        length_to_copy = (*file_object).FileName.Length as u32;
        (*buffer).FileNameLength = length_to_copy;

        // *Length -= sizeof(ULONG)
        *length -= core::mem::size_of::<u32>() as u32;

        if length_to_copy > *length {
            length_to_copy = *length;
            status = STATUS_BUFFER_OVERFLOW;
        }

        // RtlCopyMemory(Buffer->FileName, FileObject->FileName.Buffer, LengthToCopy)
        RtlCopyMemory(
            (*buffer).FileName.as_mut_ptr() as *mut core::ffi::c_void,
            (*file_object).FileName.Buffer as *const core::ffi::c_void,
            length_to_copy as usize,
        );

        *length -= length_to_copy;

        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryAlternateNameInfo(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
    Buffer: *mut FILE_NAME_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    let mut DirContext: DIRENT_ENUM_CONTEXT = core::mem::zeroed();
    let mut Dirent: DIRENT = core::mem::zeroed();
    let mut CompoundPathEntry: COMPOUND_PATH_ENTRY = core::mem::zeroed();
    let mut FileContext: FILE_ENUM_CONTEXT = core::mem::zeroed();

    let mut ParentFcb: PFCB = core::ptr::null_mut();
    let mut ReleaseParentFcb: BOOLEAN = 0;
    let mut CleanupFileLookup: BOOLEAN = 0;
    let mut CleanupDirectoryLookup: BOOLEAN = 0;

    let mut ShortNameBuffer: [u16; 12] = [0; 12]; // BYTE_COUNT_8_DOT_3 / 2
    let mut ShortNameLength: u16 = 0;

    PAGED_CODE!();

    (*Buffer).FileNameLength = 0;

    if (Fcb == (*(*Fcb).Vcb).RootIndexFcb)
        || FlagOn((*Ccb).Flags, CCB_FLAG_OPEN_WITH_VERSION) != 0
    {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_CleanupFileLookup = core::ptr::addr_of_mut!(CleanupFileLookup);
    let p_CleanupDirectoryLookup = core::ptr::addr_of_mut!(CleanupDirectoryLookup);
    let p_ReleaseParentFcb = core::ptr::addr_of_mut!(ReleaseParentFcb);
    let p_ParentFcb = core::ptr::addr_of_mut!(ParentFcb);
    let p_DirContext = core::ptr::addr_of_mut!(DirContext);
    let p_Dirent = core::ptr::addr_of_mut!(Dirent);
    let p_CompoundPathEntry = core::ptr::addr_of_mut!(CompoundPathEntry);
    let p_FileContext = core::ptr::addr_of_mut!(FileContext);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            let _ = 'try_block: loop {
                ParentFcb = (*Fcb).ParentFcb;
                CdAcquireFileShared(IrpContext, ParentFcb);
                ReleaseParentFcb = 1;

                CdVerifyOrCreateDirStreamFile(IrpContext, ParentFcb);

                let name_to_use: *mut _UNICODE_STRING;
                let dirent_offset: ULONG;

                if CdFidIsDirectory((*Fcb).FileId) != 0 {
                    CdInitializeCompoundPathEntry(IrpContext, &mut CompoundPathEntry);
                    CdInitializeFileContext(IrpContext, &mut FileContext);
                    CleanupDirectoryLookup = 1;

                    CdLookupPathEntry(
                        IrpContext,
                        CdQueryFidPathTableOffset((*Fcb).FileId),
                        (*cd_fcb_index(Fcb)).Ordinal,
                        0,
                        &mut CompoundPathEntry,
                    );

                    CdUpdatePathEntryName(IrpContext, &mut CompoundPathEntry.PathEntry, 1);

                    if CdFindDirectory(
                        IrpContext,
                        ParentFcb,
                        &mut CompoundPathEntry.PathEntry.CdCaseDirName,
                        1,
                        &mut FileContext,
                    ) == 0 {
                        CdRaiseStatus(IrpContext, STATUS_DISK_CORRUPT_ERROR);
                    }

                    name_to_use = core::ptr::addr_of_mut!((*FileContext.InitialDirent).Dirent.CdCaseFileName.FileName) as *mut _UNICODE_STRING;
                    dirent_offset = (*FileContext.InitialDirent).Dirent.DirentOffset;
                } else {
                    CdInitializeDirContext(IrpContext, &mut DirContext);
                    CdInitializeDirent(IrpContext, &mut Dirent);
                    CleanupFileLookup = 1;

                    CdLookupDirent(
                        IrpContext,
                        ParentFcb,
                        CdQueryFidDirentOffset((*Fcb).FileId),
                        &mut DirContext,
                    );

                    CdUpdateDirentFromRawDirent(
                        IrpContext,
                        ParentFcb,
                        &mut DirContext,
                        &mut Dirent,
                    );

                    CdUpdateDirentName(IrpContext, &mut Dirent, 1);

                    name_to_use = core::ptr::addr_of_mut!(Dirent.CdCaseFileName.FileName) as *mut _UNICODE_STRING;
                    dirent_offset = Dirent.DirentOffset;
                }

                if CdIs8dot3Name((IrpContext).cast(), *name_to_use) != 0 {
                    Status = STATUS_OBJECT_NAME_NOT_FOUND;
                    break 'try_block;
                }

                CdGenerate8dot3Name(
                    (IrpContext).cast(),
                    name_to_use,
                    dirent_offset,
                    ShortNameBuffer.as_mut_ptr(),
                    &mut ShortNameLength,
                );

                (*Buffer).FileNameLength = ShortNameLength as u32;

                if (*Buffer).FileNameLength as usize + core::mem::size_of::<u32>() > *Length as usize {
                    (*Buffer).FileNameLength =
                        (*Length as u32).wrapping_sub(core::mem::size_of::<u32>() as u32);
                    Status = STATUS_BUFFER_OVERFLOW;
                }

                core::ptr::copy_nonoverlapping(
                    ShortNameBuffer.as_ptr() as *const u8,
                    (*Buffer).FileName.as_mut_ptr() as *mut u8,
                    (*Buffer).FileNameLength as usize,
                );

                break 'try_block;
            };
        },
        |_, _| {
            if (*p_CleanupFileLookup) != 0 {
                CdCleanupDirContext(IrpContext, &mut (*p_DirContext));
                CdCleanupDirent(IrpContext, &mut (*p_Dirent));
            } else if (*p_CleanupDirectoryLookup) != 0 {
                CdCleanupCompoundPathEntry(IrpContext, &mut (*p_CompoundPathEntry));
                CdCleanupFileContext(IrpContext, &mut (*p_FileContext));
            }
            if (*p_ReleaseParentFcb) != 0 {
                CdReleaseFile(IrpContext, *p_ParentFcb );
            }
        },
    );


    Status
}

#[no_mangle]
pub unsafe extern "C" fn CdQueryNetworkInfo(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    Buffer: PFILE_NETWORK_OPEN_INFORMATION,
    Length: PULONG,
) {
    // PAGED_CODE(); is a debug assertion – skipped in mechanical translation

    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    // Buffer->LastWriteTime.QuadPart = Buffer->CreationTime.QuadPart = Buffer->ChangeTime.QuadPart = Fcb->CreationTime;
    let creation_quad: LONGLONG = (*Fcb).CreationTime;
    (*Buffer).LastWriteTime.QuadPart = creation_quad;
    (*Buffer).CreationTime.QuadPart = creation_quad;
    (*Buffer).ChangeTime.QuadPart = creation_quad;

    (*Buffer).LastAccessTime.QuadPart = 0;

    (*Buffer).FileAttributes = (*Fcb).FileAttributes;

    if ((*Fcb).FileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        (*Buffer).AllocationSize.QuadPart = 0;
        (*Buffer).EndOfFile.QuadPart = 0;
    } else {
        (*Buffer).AllocationSize.QuadPart = (*cd_fcb_header(Fcb)).AllocationSize.QuadPart;
        (*Buffer).EndOfFile.QuadPart = (*cd_fcb_header(Fcb)).FileSize.QuadPart;
    }

    // *Length -= sizeof( FILE_NETWORK_OPEN_INFORMATION );
    *Length -= core::mem::size_of::<FILE_NETWORK_OPEN_INFORMATION>() as u32;

    return;
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
//     This module implements the File Information routines for Cdfs called by
//     the Fsd/Fsp dispatch drivers.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_FILEINFO)
//
// //
// //  Local support routines
// //
//
// VOID
// CdQueryBasicInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_BASIC_INFORMATION Buffer,
//      PULONG Length
//     );
//
// VOID
// CdQueryStandardInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_STANDARD_INFORMATION Buffer,
//      PULONG Length
//     );
//
// VOID
// CdQueryInternalInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_INTERNAL_INFORMATION Buffer,
//      PULONG Length
//     );
//
// VOID
// CdQueryEaInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_EA_INFORMATION Buffer,
//      PULONG Length
//     );
//
// VOID
// CdQueryPositionInfo (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFILE_POSITION_INFORMATION Buffer,
//      PULONG Length
//     );
//
// NTSTATUS
// CdQueryNameInfo (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFILE_NAME_INFORMATION Buffer,
//      PULONG Length
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryAlternateNameInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCCB Ccb,
//      PFILE_NAME_INFORMATION Buffer,
//      PULONG Length
//     );
//
// VOID
// CdQueryNetworkInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_NETWORK_OPEN_INFORMATION Buffer,
//      PULONG Length
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonQueryInfo)
// #pragma alloc_text(PAGE, CdCommonSetInfo)
// #pragma alloc_text(PAGE, CdFastQueryBasicInfo)
// #pragma alloc_text(PAGE, CdFastQueryStdInfo)
// #pragma alloc_text(PAGE, CdFastQueryNetworkInfo)
// #pragma alloc_text(PAGE, CdQueryAlternateNameInfo)
// #pragma alloc_text(PAGE, CdQueryBasicInfo)
// #pragma alloc_text(PAGE, CdQueryEaInfo)
// #pragma alloc_text(PAGE, CdQueryInternalInfo)
// #pragma alloc_text(PAGE, CdQueryNameInfo)
// #pragma alloc_text(PAGE, CdQueryNetworkInfo)
// #pragma alloc_text(PAGE, CdQueryPositionInfo)
// #pragma alloc_text(PAGE, CdQueryStandardInfo)
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonQueryInfo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for query file information called by both the
//     fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp to process.
//
// Return Value:
//
//     NTSTATUS - The return status for this operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     ULONG Length;
//     FILE_INFORMATION_CLASS FileInformationClass;
//     PFILE_ALL_INFORMATION Buffer;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN ReleaseFcb = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     Length = IrpSp->Parameters.QueryFile.Length;
//     FileInformationClass = IrpSp->Parameters.QueryFile.FileInformationClass;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  We only support query on file and directory handles.
//         //
//
//         switch (TypeOfOpen) {
//
//         case UserDirectoryOpen :
//         case UserFileOpen :
//
//             //
//             //  Acquire shared access to this file.  NOTE that this could be
//             //  a recursive acquire,  if we already preacquired in
//             //  CdAcquireForCreateSection().
//             //
//
//             CdAcquireFileShared( IrpContext, Fcb );
//             ReleaseFcb = TRUE;
//
//             //
//             //  Make sure we have the correct sizes for a directory.
//             //
//
//             if (!FlagOn( Fcb->FcbState, FCB_STATE_INITIALIZED )) {
//
//                 NT_ASSERT( TypeOfOpen == UserDirectoryOpen );
//                 CdVerifyOrCreateDirStreamFile( IrpContext, Fcb);
//             }
//
//             //
//             //  Make sure the Fcb is in a usable condition.  This will raise
//             //  an error condition if the volume is unusable
//             //
//
//             CdVerifyFcbOperation( IrpContext, Fcb );
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
//                 //  We don't allow this operation on a file opened by file Id.
//                 //
//
//                 if (FlagOn( Ccb->Flags, CCB_FLAG_OPEN_BY_ID )) {
//
//                     Status = STATUS_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 //
//                 //  In this case go ahead and call the individual routines to
//                 //  fill in the buffer.  Only the name routine will
//                 //  pointer to the output buffer and then call the
//                 //  individual routines to fill in the buffer.
//                 //
//
//                 Length -= (sizeof( FILE_ACCESS_INFORMATION ) +
//                            sizeof( FILE_MODE_INFORMATION ) +
//                            sizeof( FILE_ALIGNMENT_INFORMATION ));
//
//                 CdQueryBasicInfo( IrpContext, Fcb, &Buffer->BasicInformation, &Length );
//                 CdQueryStandardInfo( IrpContext, Fcb, &Buffer->StandardInformation, &Length );
//                 CdQueryInternalInfo( IrpContext, Fcb, &Buffer->InternalInformation, &Length );
//                 CdQueryEaInfo( IrpContext, Fcb, &Buffer->EaInformation, &Length );
//                 CdQueryPositionInfo( IrpContext, IrpSp->FileObject, &Buffer->PositionInformation, &Length );
//                 Status = CdQueryNameInfo( IrpContext, IrpSp->FileObject, &Buffer->NameInformation, &Length );
//
//                 break;
//
//             case FileBasicInformation:
//
//                 CdQueryBasicInfo( IrpContext, Fcb, (PFILE_BASIC_INFORMATION) Buffer, &Length );
//                 break;
//
//             case FileStandardInformation:
//
//                 CdQueryStandardInfo( IrpContext, Fcb, (PFILE_STANDARD_INFORMATION) Buffer, &Length );
//                 break;
//
//             case FileInternalInformation:
//
//                 CdQueryInternalInfo( IrpContext, Fcb, (PFILE_INTERNAL_INFORMATION) Buffer, &Length );
//                 break;
//
//             case FileEaInformation:
//
//                 CdQueryEaInfo( IrpContext, Fcb, (PFILE_EA_INFORMATION) Buffer, &Length );
//                 break;
//
//             case FilePositionInformation:
//
//                 CdQueryPositionInfo( IrpContext, IrpSp->FileObject, (PFILE_POSITION_INFORMATION) Buffer, &Length );
//                 break;
//
//             case FileNameInformation:
//
//                 //
//                 //  We don't allow this operation on a file opened by file Id.
//                 //
//
//                 if (!FlagOn( Ccb->Flags, CCB_FLAG_OPEN_BY_ID )) {
//
//                     Status = CdQueryNameInfo( IrpContext, IrpSp->FileObject, (PFILE_NAME_INFORMATION) Buffer, &Length );
//
//                 } else {
//
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//
//                 break;
//
//             case FileAlternateNameInformation:
//
//                 if (!FlagOn( Ccb->Flags, CCB_FLAG_OPEN_BY_ID )) {
//
//                     Status = CdQueryAlternateNameInfo( IrpContext, Fcb, Ccb, (PFILE_NAME_INFORMATION) Buffer, &Length );
//
//                 } else {
//
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//
//                 break;
//
//             case FileNetworkOpenInformation:
//
//                 CdQueryNetworkInfo( IrpContext, Fcb, (PFILE_NETWORK_OPEN_INFORMATION) Buffer, &Length );
//                 break;
//
//             default :
//
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//
//             break;
//
//         default :
//
//             Status = STATUS_INVALID_PARAMETER;
//         }
//
//         //
//         //  Set the information field to the number of bytes actually filled in
//         //  and then complete the request
//         //
//
//         Irp->IoStatus.Information = IrpSp->Parameters.QueryFile.Length - Length;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the file.
//         //
//
//         if (ReleaseFcb) {
//
//             CdReleaseFile( IrpContext, Fcb );
//         }
//     } _SEH2_END;
//
//     //
//     //  Complete the request if we didn't raise.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonSetInfo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for set file information called by both the
//     fsd and fsp threads.  We only support operations which set the file position.
//
// Arguments:
//
//     Irp - Supplies the Irp to process.
//
// Return Value:
//
//     NTSTATUS - The return status for this operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PFILE_POSITION_INFORMATION Buffer;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     //
//     //  We only support a SetPositionInformation on a user file.
//     //
//
//     if ((TypeOfOpen != UserFileOpen) ||
//         (IrpSp->Parameters.QueryFile.FileInformationClass != FilePositionInformation)) {
//
//         CdCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     //
//     //  Acquire shared access to this file.
//     //
//
//     CdAcquireFileShared( IrpContext, Fcb );
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure the Fcb is in a usable condition.  This
//         //  will raise an error condition if the fcb is unusable
//         //
//
//         CdVerifyFcbOperation( IrpContext, Fcb );
//
//         Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//         //
//         //  Check if the file does not use intermediate buffering.  If it
//         //  does not use intermediate buffering then the new position we're
//         //  supplied must be aligned properly for the device
//         //
//
//         if (FlagOn( IrpSp->FileObject->Flags, FO_NO_INTERMEDIATE_BUFFERING ) &&
//             ((Buffer->CurrentByteOffset.LowPart & Fcb->Vcb->BlockMask) != 0)) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  The input parameter is fine so set the current byte offset and
//         //  complete the request
//         //
//
//         //
//         //  Lock the Fcb to provide synchronization.
//         //
//
//         CdLockFcb( IrpContext, Fcb );
//         IrpSp->FileObject->CurrentByteOffset = Buffer->CurrentByteOffset;
//         CdUnlockFcb( IrpContext, Fcb );
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         CdReleaseFile( IrpContext, Fcb );
//     } _SEH2_END;
//
//     //
//     //  Complete the request if there was no raise.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// _Function_class_(FAST_IO_QUERY_BASIC_INFO)
// _IRQL_requires_same_
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastQueryBasicInfo (
//      PFILE_OBJECT FileObject,
//      BOOLEAN Wait,
//      PFILE_BASIC_INFORMATION Buffer,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for basic file information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the basic information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Result = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     ASSERT_FILE_OBJECT( FileObject );
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Decode the file object to find the type of open and the data
//     //  structures.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     //
//     //  We only support this request on user file or directory objects.
//     //
//
//     if ((TypeOfOpen != UserFileOpen) &&
//         ((TypeOfOpen != UserDirectoryOpen) || !FlagOn( Fcb->FcbState, FCB_STATE_INITIALIZED))) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Acquire the file shared to access the Fcb.
//     //
//
//     if (!ExAcquireResourceSharedLite( Fcb->Resource, Wait )) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Only deal with 'good' Fcb's.
//         //
//
//         if (CdVerifyFcbOperation( NULL, Fcb )) {
//
//             //
//             //  Fill in the input buffer from the Fcb fields.
//             //
//
//             Buffer->CreationTime.QuadPart =
//             Buffer->LastWriteTime.QuadPart =
//             Buffer->ChangeTime.QuadPart = Fcb->CreationTime;
//
//             Buffer->LastAccessTime.QuadPart = 0;
//
//             Buffer->FileAttributes = Fcb->FileAttributes;
//
//             //
//             //  Update the IoStatus block with the size of this data.
//             //
//
//             IoStatus->Status = STATUS_SUCCESS;
//             IoStatus->Information = sizeof( FILE_BASIC_INFORMATION );
//
//             Result = TRUE;
//         }
//
//     } _SEH2_FINALLY {
//
//         ExReleaseResourceLite( Fcb->Resource );
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Result;
// }
//
//
// _Function_class_(FAST_IO_QUERY_STANDARD_INFO)
// _IRQL_requires_same_
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastQueryStdInfo (
//      PFILE_OBJECT FileObject,
//      BOOLEAN Wait,
//      PFILE_STANDARD_INFORMATION Buffer,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for standard file information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the basic information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Result = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     ASSERT_FILE_OBJECT( FileObject );
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Decode the file object to find the type of open and the data
//     //  structures.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     //
//     //  We only support this request on initialized user file or directory objects.
//     //
//
//     if ((TypeOfOpen != UserFileOpen) &&
//         ((TypeOfOpen != UserDirectoryOpen) || !FlagOn( Fcb->FcbState, FCB_STATE_INITIALIZED ))) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Acquire the file shared to access the Fcb.
//     //
//
//     if (!ExAcquireResourceSharedLite( Fcb->Resource, Wait )) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Only deal with 'good' Fcb's.
//         //
//
//         if (CdVerifyFcbOperation( NULL, Fcb )) {
//
//             //
//             //  Check whether this is a directory.
//             //
//
//             if (FlagOn( Fcb->FileAttributes, FILE_ATTRIBUTE_DIRECTORY )) {
//
//                 Buffer->AllocationSize.QuadPart =
//                 Buffer->EndOfFile.QuadPart = 0;
//
//                 Buffer->Directory = TRUE;
//
//             } else {
//
//                 Buffer->AllocationSize.QuadPart = Fcb->AllocationSize.QuadPart;
//                 Buffer->EndOfFile.QuadPart = Fcb->FileSize.QuadPart;
//
//                 Buffer->Directory = FALSE;
//             }
//
//             Buffer->NumberOfLinks = 1;
//             Buffer->DeletePending = FALSE;
//
//             //
//             //  Update the IoStatus block with the size of this data.
//             //
//
//             IoStatus->Status = STATUS_SUCCESS;
//             IoStatus->Information = sizeof( FILE_STANDARD_INFORMATION );
//
//             Result = TRUE;
//         }
//
//     } _SEH2_FINALLY {
//
//         ExReleaseResourceLite( Fcb->Resource );
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Result;
// }
//
//
// _Function_class_(FAST_IO_QUERY_NETWORK_OPEN_INFO)
// _IRQL_requires_same_
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastQueryNetworkInfo (
//      PFILE_OBJECT FileObject,
//      BOOLEAN Wait,
//      PFILE_NETWORK_OPEN_INFORMATION Buffer,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for network file information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the basic information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Result = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     ASSERT_FILE_OBJECT( FileObject );
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Decode the file object to find the type of open and the data
//     //  structures.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     //
//     //  We only support this request on user file or directory objects.
//     //
//
//     if ((TypeOfOpen != UserFileOpen) &&
//         ((TypeOfOpen != UserDirectoryOpen) || !FlagOn( Fcb->FcbState, FCB_STATE_INITIALIZED))) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Acquire the file shared to access the Fcb.
//     //
//
//     if (!ExAcquireResourceSharedLite( Fcb->Resource, Wait )) {
//
//         FsRtlExitFileSystem();
//         return FALSE;
//     }
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Only deal with 'good' Fcb's.
//         //
//
//         if (CdVerifyFcbOperation( NULL, Fcb )) {
//
//             //
//             //  Fill in the input buffer from the Fcb fields.
//             //
//
//             Buffer->CreationTime.QuadPart =
//             Buffer->LastWriteTime.QuadPart =
//             Buffer->ChangeTime.QuadPart = Fcb->CreationTime;
//
//             Buffer->LastAccessTime.QuadPart = 0;
//
//             Buffer->FileAttributes = Fcb->FileAttributes;
//
//             //
//             //  Check whether this is a directory.
//             //
//
//             if (FlagOn( Fcb->FileAttributes, FILE_ATTRIBUTE_DIRECTORY )) {
//
//                 Buffer->AllocationSize.QuadPart =
//                 Buffer->EndOfFile.QuadPart = 0;
//
//             } else {
//
//                 Buffer->AllocationSize.QuadPart = Fcb->AllocationSize.QuadPart;
//                 Buffer->EndOfFile.QuadPart = Fcb->FileSize.QuadPart;
//             }
//
//             //
//             //  Update the IoStatus block with the size of this data.
//             //
//
//             IoStatus->Status = STATUS_SUCCESS;
//             IoStatus->Information = sizeof( FILE_NETWORK_OPEN_INFORMATION );
//
//             Result = TRUE;
//         }
//
//     } _SEH2_FINALLY {
//
//         ExReleaseResourceLite( Fcb->Resource );
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Result;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryBasicInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_BASIC_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
//  Description:
//
//     This routine performs the query basic information function for Cdfs
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  We only support creation, last modify and last write times on Cdfs.
//     //
//
//     RtlZeroMemory(Buffer, sizeof(FILE_BASIC_INFORMATION));
//
//     Buffer->LastWriteTime.QuadPart =
//     Buffer->CreationTime.QuadPart =
//     Buffer->ChangeTime.QuadPart = Fcb->CreationTime;
//
//     Buffer->LastAccessTime.QuadPart = 0;
//
//     Buffer->FileAttributes = Fcb->FileAttributes;
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_BASIC_INFORMATION );
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryStandardInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_STANDARD_INFORMATION Buffer,
//      PULONG Length
//     )
// /*++
//
// Routine Description:
//
//     This routine performs the query standard information function for cdfs.
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  There is only one link and delete is never pending on a Cdrom file.
//     //
//
//     Buffer->NumberOfLinks = 1;
//     Buffer->DeletePending = FALSE;
//
//     //
//     //  We get the sizes from the header.  Return a size of zero
//     //  for all directories.
//     //
//
//     if (FlagOn( Fcb->FileAttributes, FILE_ATTRIBUTE_DIRECTORY )) {
//
//         Buffer->AllocationSize.QuadPart =
//         Buffer->EndOfFile.QuadPart = 0;
//
//         Buffer->Directory = TRUE;
//
//     } else {
//
//         Buffer->AllocationSize.QuadPart = Fcb->AllocationSize.QuadPart;
//         Buffer->EndOfFile.QuadPart = Fcb->FileSize.QuadPart;
//
//         Buffer->Directory = FALSE;
//     }
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_STANDARD_INFORMATION );
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryInternalInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_INTERNAL_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query internal information function for cdfs.
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Index number is the file Id number in the Fcb.
//     //
//
//     Buffer->IndexNumber = Fcb->FileId;
//     *Length -= sizeof( FILE_INTERNAL_INFORMATION );
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryEaInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_EA_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query Ea information function for cdfs.
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//     UNREFERENCED_PARAMETER( Fcb );
//
//     //
//     //  No Ea's on Cdfs volumes.
//     //
//
//     Buffer->EaSize = 0;
//     *Length -= sizeof( FILE_EA_INFORMATION );
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryPositionInfo (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFILE_POSITION_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query position information function for cdfs.
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
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
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdQueryNameInfo (
//      PIRP_CONTEXT IrpContext,
//      PFILE_OBJECT FileObject,
//      PFILE_NAME_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query name information function for cdfs.
//
// Arguments:
//
//     FileObject - Supplies the file object containing the name.
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     NTSTATUS - STATUS_BUFFER_OVERFLOW if the entire name can't be copied.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG LengthToCopy;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     NT_ASSERT(*Length >= sizeof(ULONG));
//
//     //
//     //  Simply copy the name in the file object to the user's buffer.
//     //
//
//     //
//     //  Place the size of the filename in the user's buffer and reduce the remaining
//     //  size to match.
//     //
//
//     Buffer->FileNameLength = LengthToCopy = FileObject->FileName.Length;
//     *Length -= sizeof(ULONG);
//
//     if (LengthToCopy > *Length) {
//
//         LengthToCopy = *Length;
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     RtlCopyMemory( Buffer->FileName, FileObject->FileName.Buffer, LengthToCopy );
//
//     //
//     //  Reduce the available bytes by the amount stored into this buffer.  In the overflow
//     //  case, this simply drops to zero.  The returned filenamelength will indicate to the
//     //  caller how much space is required.
//     //
//
//     *Length -= LengthToCopy;
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryAlternateNameInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCCB Ccb,
//      PFILE_NAME_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query alternate name information function.
//     We lookup the dirent for this file and then check if there is a
//     short name.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being queried, it has been verified.
//
//     Ccb - Ccb for this open handle.
//
//     Buffer - Supplies a pointer to the buffer where the information is to
//         be returned.
//
//     Length - Supplies the length of the buffer in bytes, and receives the
//         remaining bytes free in the buffer upon return.
//
// Return Value:
//
//     NTSTATUS - STATUS_SUCCESS if the whole name would fit into the user buffer,
//                STATUS_OBJECT_NAME_NOT_FOUND if we can't return the name,
//                STATUS_BUFFER_OVERFLOW otherwise.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DIRENT_ENUM_CONTEXT DirContext = {0};
//     DIRENT Dirent = {0};
//
//     PUNICODE_STRING NameToUse;
//     ULONG DirentOffset;
//
//     COMPOUND_PATH_ENTRY CompoundPathEntry = {{0}};/* ReactOS Change: GCC "missing braces around initializer" */
//     FILE_ENUM_CONTEXT FileContext;
//
//     PFCB ParentFcb = NULL;
//     BOOLEAN ReleaseParentFcb = FALSE;
//
//     BOOLEAN CleanupFileLookup = FALSE;
//     BOOLEAN CleanupDirectoryLookup = FALSE;
//
//     WCHAR ShortNameBuffer[ BYTE_COUNT_8_DOT_3 / 2 ];
//     USHORT ShortNameLength;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the buffer length to zero.
//     //
//
//     Buffer->FileNameLength = 0;
//
//     //
//     //  If this is the root or this file was opened using a version number then
//     //  there is no short name.
//     //
//
//     if ((Fcb == Fcb->Vcb->RootIndexFcb) ||
//         FlagOn( Ccb->Flags, CCB_FLAG_OPEN_WITH_VERSION)) {
//
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     //
//     //  Use a try-finally to cleanup the structures.
//     //
//
//     _SEH2_TRY {
//
//         ParentFcb = Fcb->ParentFcb;
//         CdAcquireFileShared( IrpContext, ParentFcb );
//         ReleaseParentFcb = TRUE;
//
//         CdVerifyOrCreateDirStreamFile( IrpContext, ParentFcb);
//
//         if (CdFidIsDirectory( Fcb->FileId)) {
//
//             //
//             //  Fcb is for a directory, so we need to dig the dirent from the parent.  In
//             //  order to do this we need to get the name of the directory from its pathtable
//             //  entry and then search in the parent for a matching dirent.
//             //
//             //  This could be optimized somewhat.
//             //
//
//             CdInitializeCompoundPathEntry( IrpContext, &CompoundPathEntry );
//             CdInitializeFileContext( IrpContext, &FileContext );
//
//             CleanupDirectoryLookup = TRUE;
//
//             CdLookupPathEntry( IrpContext,
//                                CdQueryFidPathTableOffset( Fcb->FileId ),
//                                Fcb->Ordinal,
//                                FALSE,
//                                &CompoundPathEntry );
//
//             CdUpdatePathEntryName( IrpContext, &CompoundPathEntry.PathEntry, TRUE );
//
//             if (!CdFindDirectory( IrpContext,
//                                   ParentFcb,
//                                   &CompoundPathEntry.PathEntry.CdCaseDirName,
//                                   TRUE,
//                                   &FileContext )) {
//
//                 //
//                 //  If we failed to find the child directory by name in the parent
//                 //  something is quite wrong with this disc.
//                 //
//
//                 CdRaiseStatus( IrpContext, STATUS_DISK_CORRUPT_ERROR );
//             }
//
//             NameToUse = &FileContext.InitialDirent->Dirent.CdCaseFileName.FileName;
//             DirentOffset = FileContext.InitialDirent->Dirent.DirentOffset;
//
//         } else {
//
//             //
//             //  Initialize the search dirent structures.
//             //
//
//             CdInitializeDirContext( IrpContext, &DirContext );
//             CdInitializeDirent( IrpContext, &Dirent );
//
//             CleanupFileLookup = TRUE;
//
//             CdLookupDirent( IrpContext,
//                             ParentFcb,
//                             CdQueryFidDirentOffset( Fcb->FileId ),
//                             &DirContext );
//
//             CdUpdateDirentFromRawDirent( IrpContext,
//                                          ParentFcb,
//                                          &DirContext,
//                                          &Dirent );
//
//             //
//             //  Now update the dirent name.
//             //
//
//             CdUpdateDirentName( IrpContext, &Dirent, TRUE );
//
//             NameToUse = &Dirent.CdCaseFileName.FileName;
//             DirentOffset = Dirent.DirentOffset;
//         }
//
//         //
//         //  If the name is 8.3 then fail this request.
//         //
//
//         if (CdIs8dot3Name( IrpContext,
//                            *NameToUse )) {
//
//
//             try_return( Status = STATUS_OBJECT_NAME_NOT_FOUND );
//         }
//
//         CdGenerate8dot3Name( IrpContext,
//                              NameToUse,
//                              DirentOffset,
//                              ShortNameBuffer,
//                              &ShortNameLength );
//
//         //
//         //  We now have the short name.  We have left it in Unicode form so copy it directly.
//         //
//
//         Buffer->FileNameLength = ShortNameLength;
//
//         if (Buffer->FileNameLength + sizeof( ULONG ) > *Length) {
//
//             Buffer->FileNameLength = *Length - sizeof( ULONG );
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//
//         RtlCopyMemory( Buffer->FileName, ShortNameBuffer, Buffer->FileNameLength );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         if (CleanupFileLookup) {
//
//             CdCleanupDirContext( IrpContext, &DirContext );
//             CdCleanupDirent( IrpContext, &Dirent );/* ReactOS Change: GCC "passing argument 1 from incompatible pointer type" */
//
//         } else if (CleanupDirectoryLookup) {
//
//             CdCleanupCompoundPathEntry( IrpContext, &CompoundPathEntry );
//             CdCleanupFileContext( IrpContext, &FileContext );
//         }
//
//         if (ReleaseParentFcb) {
//
//             CdReleaseFile( IrpContext, ParentFcb );
//         }
//     } _SEH2_END;
//
//     //
//     //  Reduce the available bytes by the amount stored into this buffer.
//     //
//
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND) {
//
//         *Length -= sizeof( ULONG ) + Buffer->FileNameLength;
//     }
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdQueryNetworkInfo (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_NETWORK_OPEN_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
//  Description:
//
//     This routine performs the query network open information function for Cdfs
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
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  We only support creation, last modify and last write times on Cdfs.
//     //
//
//     Buffer->LastWriteTime.QuadPart =
//     Buffer->CreationTime.QuadPart =
//     Buffer->ChangeTime.QuadPart = Fcb->CreationTime;
//
//     Buffer->LastAccessTime.QuadPart = 0;
//
//     Buffer->FileAttributes = Fcb->FileAttributes;
//
//     //
//     //  We get the sizes from the header.  Return a size of zero
//     //  for all directories.
//     //
//
//     if (FlagOn( Fcb->FileAttributes, FILE_ATTRIBUTE_DIRECTORY )) {
//
//         Buffer->AllocationSize.QuadPart =
//         Buffer->EndOfFile.QuadPart = 0;
//
//     } else {
//
//         Buffer->AllocationSize.QuadPart = Fcb->AllocationSize.QuadPart;
//         Buffer->EndOfFile.QuadPart = Fcb->FileSize.QuadPart;
//     }
//
//     //
//     //  Update the length and status output variables
//     //
//
//     *Length -= sizeof( FILE_NETWORK_OPEN_INFORMATION );
//
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
