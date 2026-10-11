#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{FILE_BOTH_DIR_INFORMATION, FILE_DIRECTORY_INFORMATION, FILE_FULL_DIR_INFORMATION, FILE_ID_BOTH_DIR_INFORMATION, FILE_ID_FULL_DIR_INFORMATION, FILE_NAMES_INFORMATION, IoGetCurrentIrpStackLocation, PIO_STACK_LOCATION, PIRP, PSTRING, RtlUpcaseUnicodeString};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, CCB_FLAG_ENUM_INITIALIZED, CCB_FLAG_ENUM_MATCH_ALL, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD, CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY, CCB_FLAG_ENUM_RETURN_NEXT, CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD, CCB_FLAG_ENUM_VERSION_MATCH_ALL, CCB_FLAG_IGNORE_CASE, CD_NAME, DIRENT, DIRENT_FLAG_CONSTANT_ENTRY, FCB, FCB_INDEX, FILE_ENUM_CONTEXT, IRP_CONTEXT, PCCB, PFCB, PFILE_ENUM_CONTEXT, PIRP_CONTEXT, PVCB};
use ntkrnl_sys::ddk::{FILE_OBJECT, IO_STACK_LOCATION, IRP, POOL_TYPE, _FILE_INFORMATION_CLASS_FileBothDirectoryInformation as FileBothDirectoryInformation, _FILE_INFORMATION_CLASS_FileDirectoryInformation as FileDirectoryInformation, _FILE_INFORMATION_CLASS_FileFullDirectoryInformation as FileFullDirectoryInformation, _FILE_INFORMATION_CLASS_FileIdBothDirectoryInformation as FileIdBothDirectoryInformation, _FILE_INFORMATION_CLASS_FileIdFullDirectoryInformation as FileIdFullDirectoryInformation, _FILE_INFORMATION_CLASS_FileNamesInformation as FileNamesInformation, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlNotifyFullChangeDirectory};
use crate::glue::{CDFS_BUG_CHECK_DIRCTRL, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, STATUS_NO_MORE_FILES};
use crate::types::{CD_ATTRIBUTE_ASSOC, CD_ATTRIBUTE_DIRECTORY, CD_ATTRIBUTE_HIDDEN};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
    fn CdDecodeFileObject( IrpContext: *mut IRP_CONTEXT, FileObject: *mut FILE_OBJECT, Fcb: *mut *mut FCB, Ccb: *mut *mut CCB, ) -> u32;
}

extern "C" {
    fn CdAcquireVcbShared(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Exclusive: BOOLEAN) -> u8;
}
extern "C" {
    fn CdVerifyVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn CdReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}
extern "C" {
    fn CdLockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdUnlockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdFreePool(pBuffer: *mut *mut u16);
}
extern "C" {
    fn CdConvertNameToCdName(IrpContext: PIRP_CONTEXT, WildCardName: *mut CD_NAME);
}
extern "C" {
    fn CdRaiseStatus(IrpContext: PIRP_CONTEXT, Status: NTSTATUS) -> !;
}
extern "C" {
    fn CdLookupInitialFileDirent(IrpContext: PIRP_CONTEXT, Fcb: PFCB, FileContext: PFILE_ENUM_CONTEXT, DirentOffset: ULONG);
}
extern "C" {
    fn CdLookupNextInitialFileDirent(IrpContext: PIRP_CONTEXT, Fcb: PFCB, FileContext: PFILE_ENUM_CONTEXT) -> BOOLEAN;
}
extern "C" {
    fn CdCleanupFileContext(IrpContext: PIRP_CONTEXT, FileContext: PFILE_ENUM_CONTEXT);
}
extern "C" {
    fn CdInitializeFileContext(IrpContext: PIRP_CONTEXT, FileContext: PFILE_ENUM_CONTEXT);
}
extern "C" {
    fn CdUpdateDirentName(IrpContext: PIRP_CONTEXT, Dirent: *mut DIRENT, IgnoreCase: u32);
}
extern "C" {
    fn CdVerifyOrCreateDirStreamFile(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
}
extern "system" {
    fn FsRtlAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: ULONG, Tag: ULONG) -> *mut core::ffi::c_void;
}
extern "system" {
    fn FsRtlDoesNameContainWildCards(Name: *mut _UNICODE_STRING) -> BOOLEAN;
}
extern "C" {
    fn CdGenerate8dot3Name( IrpContext: *mut IRP_CONTEXT, pFileName: *const _UNICODE_STRING, DirentOffset: u32, Buffer: *mut u16, Length: *mut u16, );
}
extern "C" {
    fn CdIs8dot3Name(IrpContext: *mut IRP_CONTEXT, FileName: _UNICODE_STRING) -> i32;
}
extern "C" {
    fn CdIsNameInExpression( IrpContext: *mut IRP_CONTEXT, CurrentName: *mut CD_NAME, SearchExpression: *mut CD_NAME, WildcardFlags: u32, CheckVersion: BOOLEAN, ) -> BOOLEAN;
}
extern "C" {
    fn CdLookupLastFileDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, FileContext: *mut FILE_ENUM_CONTEXT, );
}
extern "C" {
    fn FlagOn(Flags: u32, Flag: u32) -> i32;
}
extern "C" {
    fn RtlEqualMemory( Buffer1: *const core::ffi::c_void, Buffer2: *const core::ffi::c_void, Length: usize, ) -> i32;
}
extern "C" {
    fn CdMapUserBuffer(IrpContext: *mut IRP_CONTEXT, UserBuffer: *mut *mut u8);
}
extern "C" {
    fn CdAcquireFileShared(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdVerifyFcbOperation(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u32;
}
extern "C" {
    fn CdConvertCdTimeToNtTime(IrpContext: *mut IRP_CONTEXT, CdTime: *mut i8, NtTime: *mut LARGE_INTEGER);
}
extern "C" {
    fn CdSetFidFromParentAndDirent(FileId: *mut i64, Fcb: *mut FCB, Dirent: *mut DIRENT);
}
extern "C" {
    fn LlSectorAlign(FileSize: i64) -> i64;
}
extern "C" {
    fn CdReleaseFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_DIRCTRL;

// Forward declarations for unknown extern functions

// Forward declarations for unknown symbols (deduplicated by caller)

// Forward declarations for unknown externals used by this function.

const UserDirectoryOpen: u32 = 3; // TYPE_OF_OPEN (cdprocs.h): Unopened=0,Stream=1,Volume=2,Directory=3,File=4
const IRP_MN_QUERY_DIRECTORY: u8 = 0x01;
const IRP_MN_NOTIFY_CHANGE_DIRECTORY: u8 = 0x02;
#[no_mangle]
pub unsafe extern "C" fn CdCommonDirControl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    unsafe {
        let Status: NTSTATUS;
        let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        let mut Fcb: *mut FCB = core::ptr::null_mut();
        let mut Ccb: *mut CCB = core::ptr::null_mut();

        PAGED_CODE!();

        if CdDecodeFileObject(
            IrpContext,
            (*IrpSp).FileObject,
            core::ptr::addr_of_mut!(Fcb),
            core::ptr::addr_of_mut!(Ccb),
        ) != UserDirectoryOpen
        {
            CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
            return STATUS_INVALID_PARAMETER;
        }

        Status = match (*IrpSp).MinorFunction {
            IRP_MN_QUERY_DIRECTORY => {
                CdQueryDirectory(IrpContext, Irp, IrpSp, Fcb, Ccb)
            }
            IRP_MN_NOTIFY_CHANGE_DIRECTORY => {
                CdNotifyChangeDirectory(IrpContext, Irp, IrpSp, Ccb)
            }
            _ => {
                CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
                STATUS_INVALID_DEVICE_REQUEST
            }
        };

        Status
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryDirectory(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    IrpSp: *mut IO_STACK_LOCATION,
    Fcb: *mut FCB,
    Ccb: *mut CCB,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Information: u32 = 0;
    let mut LastEntry: u32 = 0;
    let mut NextEntry: u32 = 0;
    let mut FileNameBytes: u32 = 0;
    let mut SeparatorBytes: u32 = 0;
    let mut VersionStringBytes: u32 = 0;
    let mut FileContext: FILE_ENUM_CONTEXT = core::mem::zeroed();
    let mut ThisDirent: *mut DIRENT = core::ptr::null_mut();
    let mut InitialQuery: BOOLEAN = 0;
    let mut ReturnNextEntry: BOOLEAN = 0; // FALSE
    let mut ReturnSingleEntry: BOOLEAN = 0;
    let mut Found: BOOLEAN = 0;
    let mut DoCcbUpdate: BOOLEAN = 0; // FALSE
    let UserBuffer: *mut u8;
    let mut BytesRemainingInBuffer: u32 = 0;
    let BaseLength: u32;
    let mut DirInfo: *mut FILE_BOTH_DIR_INFORMATION = core::ptr::null_mut();
    let mut NamesInfo: *mut FILE_NAMES_INFORMATION = core::ptr::null_mut();
    let mut IdFullDirInfo: *mut FILE_ID_FULL_DIR_INFORMATION = core::ptr::null_mut();
    let mut IdBothDirInfo: *mut FILE_ID_BOTH_DIR_INFORMATION = core::ptr::null_mut();

    // PAGED_CODE(); – omitted, annotation only.

    // Determine BaseLength based on FileInformationClass.
    let irp_sp = &*IrpSp;
    BaseLength = match irp_sp.Parameters.QueryDirectory.FileInformationClass {
        FileDirectoryInformation => {
            core::mem::offset_of!(FILE_DIRECTORY_INFORMATION, FileName) as u32
        }
        FileFullDirectoryInformation => {
            core::mem::offset_of!(FILE_FULL_DIR_INFORMATION, FileName) as u32
        }
        FileIdFullDirectoryInformation => {
            core::mem::offset_of!(FILE_ID_FULL_DIR_INFORMATION, FileName) as u32
        }
        FileNamesInformation => {
            core::mem::offset_of!(FILE_NAMES_INFORMATION, FileName) as u32
        }
        FileBothDirectoryInformation => {
            core::mem::offset_of!(FILE_BOTH_DIR_INFORMATION, FileName) as u32
        }
        FileIdBothDirectoryInformation => {
            core::mem::offset_of!(FILE_ID_BOTH_DIR_INFORMATION, FileName) as u32
        }
        _ => {
            CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_INFO_CLASS);
            return STATUS_INVALID_INFO_CLASS;
        }
    };

    // Get the user buffer.
    let mut user_buf_raw: *mut u8 = core::ptr::null_mut();
    CdMapUserBuffer(IrpContext, &mut user_buf_raw);
    UserBuffer = user_buf_raw;

    // Initialize the file enumeration context.
    CdInitializeFileContext(IrpContext, &mut FileContext);

    // Acquire the directory.
    CdAcquireFileShared(IrpContext, Fcb);

    // try { ... } finally { cleanup, Ccb update, CdReleaseFile } — a real __finally
    // (ntkrnl_sys::seh), so a raise inside still releases the directory. The body is the
    // translated loop unchanged; the finally reads the locals it needs through pointers
    // taken here, and runs only after the body has returned or been unwound.
    let p_file_context = core::ptr::addr_of_mut!(FileContext);
    let p_do_ccb_update = core::ptr::addr_of_mut!(DoCcbUpdate);
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_this_dirent = core::ptr::addr_of_mut!(ThisDirent);
    let p_return_next_entry = core::ptr::addr_of_mut!(ReturnNextEntry);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            // ------ Start of the outer SEH try block ------
            // We emulate SEH try/finally with a labeled loop + a finally block after it.
            // Any `try_leave(Status)` becomes `Status = expr; break 'outer_finally;`.
            'outer_finally: loop {
                // Verify the Fcb is still good.
                CdVerifyFcbOperation(IrpContext, Fcb);

                // Initialize the enumeration parameters.
                CdInitializeEnumeration(
                    IrpContext,
                    IrpSp,
                    Fcb,
                    Ccb,
                    &mut FileContext,
                    &mut ReturnNextEntry,
                    &mut ReturnSingleEntry,
                    &mut InitialQuery,
                );

                // Store the initial dirent pointer.
                ThisDirent = core::ptr::addr_of_mut!((*FileContext.InitialDirent).Dirent);

                // Main enumeration loop.
                loop {
                    // If a single entry was returned and it is not the first, exit.
                    if (NextEntry != 0) && (ReturnSingleEntry != 0) {
                        DoCcbUpdate = 1;
                        Status = STATUS_SUCCESS;
                        break 'outer_finally;
                    }

                    // Find the next matching dirent.
                    Found = CdEnumerateIndex(IrpContext, Ccb, &mut FileContext, ReturnNextEntry as i32) as BOOLEAN;
                    // Prepare to search for the next entry after current.
                    ReturnNextEntry = 1; // TRUE

                    if Found == 0 {
                        if NextEntry == 0 {
                            Status = STATUS_NO_MORE_FILES;
                            if InitialQuery != 0 {
                                Status = STATUS_NO_SUCH_FILE;
                            }
                        }
                        DoCcbUpdate = 1;
                        break 'outer_finally; // try_leave(Status)
                    }

                    // New dirent found.
                    ThisDirent = core::ptr::addr_of_mut!((*FileContext.InitialDirent).Dirent);

                    // Calculate name lengths.
                    SeparatorBytes = 0;
                    VersionStringBytes = 0;

                    FileNameBytes = (*ThisDirent).CdFileName.FileName.Length as u32;

                    // Determine version string bytes.
                    let is_directory = ((*ThisDirent).DirentFlags & CD_ATTRIBUTE_DIRECTORY as u8) != 0;
                    let search_has_version = ((*Ccb).SearchExpression.VersionString.Length as u32) != 0;
                    let have_illegal_semicolon = is_directory; // directories allow illegal ";"
                    let dirent_has_version = ((*ThisDirent).CdFileName.VersionString.Length as u32) != 0;

                    if (search_has_version || have_illegal_semicolon) && dirent_has_version {
                        SeparatorBytes = 2u32;
                        VersionStringBytes = (*ThisDirent).CdFileName.VersionString.Length as u32;
                    }

                    // Check if we exceeded the buffer.
                    if NextEntry > irp_sp.Parameters.QueryDirectory.Length {
                        ReturnNextEntry = 0; // FALSE
                        DoCcbUpdate = 1;
                        Status = STATUS_SUCCESS;
                        break 'outer_finally;
                    }

                    // Compute remaining bytes in user buffer (rounded down to WCHAR boundary).
                    BytesRemainingInBuffer = irp_sp.Parameters.QueryDirectory.Length - NextEntry;
                    BytesRemainingInBuffer &= !1u32; // Clear low bit.

                    // Will this entry fit?
                    if (BaseLength + FileNameBytes + SeparatorBytes + VersionStringBytes) > BytesRemainingInBuffer {
                        if NextEntry != 0 {
                            ReturnNextEntry = 0;
                            DoCcbUpdate = 1;
                            Status = STATUS_SUCCESS;
                            break 'outer_finally;
                        }

                        if (BaseLength + FileNameBytes) > BytesRemainingInBuffer {
                            FileNameBytes = BytesRemainingInBuffer - BaseLength;
                        }

                        VersionStringBytes = 0;
                        SeparatorBytes = 0;
                        Status = STATUS_BUFFER_OVERFLOW;
                        ReturnSingleEntry = 1;
                    }

                    // try { fill the entry } except (EXECUTE_HANDLER) { Information = 0;
                    // try_leave(Status = GetExceptionCode()) }: a fault on the caller's buffer is
                    // "the only reason any exception would have occurred at this level" (C).
                    let fill = ntkrnl_sys::seh::try_except(
                        &mut (),
                        |_| {
                                // Zero the base part of the entry.
                                let dest = (UserBuffer as *mut u8).add(NextEntry as usize);
                                core::ptr::write_bytes(dest, 0, BaseLength as usize);

                                // Populate the buffer based on file information class.
                                match irp_sp.Parameters.QueryDirectory.FileInformationClass {
                                    FileBothDirectoryInformation
                                    | FileFullDirectoryInformation
                                    | FileIdBothDirectoryInformation
                                    | FileIdFullDirectoryInformation
                                    | FileDirectoryInformation => {
                                        DirInfo = (UserBuffer as *mut u8)
                                            .add(NextEntry as usize)
                                            as *mut FILE_BOTH_DIR_INFORMATION;

                                        let cd_time = (*ThisDirent).CdTime;
                                        CdConvertCdTimeToNtTime(
                                            IrpContext,
                                            cd_time,
                                            core::ptr::addr_of_mut!((*DirInfo).CreationTime) as *mut LARGE_INTEGER,
                                        );
                                        (*DirInfo).LastWriteTime = (*DirInfo).CreationTime;
                                        (*DirInfo).ChangeTime = (*DirInfo).CreationTime;

                                        if is_directory {
                                            (*DirInfo).EndOfFile.QuadPart = 0;
                                            (*DirInfo).AllocationSize.QuadPart = 0;
                                            (*DirInfo).FileAttributes |= FILE_ATTRIBUTE_DIRECTORY;
                                        } else {
                                            (*DirInfo).EndOfFile.QuadPart = FileContext.FileSize;
                                            (*DirInfo).AllocationSize.QuadPart =
                                                LlSectorAlign(FileContext.FileSize);
                                            (*DirInfo).FileAttributes |= FILE_ATTRIBUTE_READONLY;
                                        }

                                        if ((*ThisDirent).DirentFlags & CD_ATTRIBUTE_HIDDEN as u8) != 0 {
                                            (*DirInfo).FileAttributes |= FILE_ATTRIBUTE_HIDDEN;
                                        }

                                        (*DirInfo).FileIndex = (*ThisDirent).DirentOffset;
                                        (*DirInfo).FileNameLength =
                                            (FileNameBytes + SeparatorBytes + VersionStringBytes) as u32;
                                    }
                                    FileNamesInformation => {
                                        NamesInfo = (UserBuffer as *mut u8)
                                            .add(NextEntry as usize)
                                            as *mut FILE_NAMES_INFORMATION;
                                        (*NamesInfo).FileIndex = (*ThisDirent).DirentOffset;
                                        (*NamesInfo).FileNameLength =
                                            (FileNameBytes + SeparatorBytes + VersionStringBytes) as u32;
                                    }
                                    _ => {} // ReactOS default case
                                }

                                // Fill FileId for the two extended classes.
                                match irp_sp.Parameters.QueryDirectory.FileInformationClass {
                                    FileIdBothDirectoryInformation => {
                                        IdBothDirInfo = (UserBuffer as *mut u8)
                                            .add(NextEntry as usize)
                                            as *mut FILE_ID_BOTH_DIR_INFORMATION;
                                        CdSetFidFromParentAndDirent(
                                            core::ptr::addr_of_mut!((*IdBothDirInfo).FileId) as *mut i64,
                                            Fcb,
                                            ThisDirent,
                                        );
                                    }
                                    FileIdFullDirectoryInformation => {
                                        IdFullDirInfo = (UserBuffer as *mut u8)
                                            .add(NextEntry as usize)
                                            as *mut FILE_ID_FULL_DIR_INFORMATION;
                                        CdSetFidFromParentAndDirent(
                                            core::ptr::addr_of_mut!((*IdFullDirInfo).FileId) as *mut i64,
                                            Fcb,
                                            ThisDirent,
                                        );
                                    }
                                    _ => {}
                                }

                                // Copy the file name and version string.
                                if FileNameBytes != 0 {
                                    let dest_name = (UserBuffer as *mut u8)
                                        .add(NextEntry as usize + BaseLength as usize);
                                    core::ptr::copy_nonoverlapping(
                                        (*ThisDirent).CdFileName.FileName.Buffer as *const u8,
                                        dest_name,
                                        FileNameBytes as usize,
                                    );

                                    if SeparatorBytes != 0 {
                                        let sep_dest = (UserBuffer as *mut u16)
                                            .add(((NextEntry + BaseLength + FileNameBytes) / 2) as usize);
                                        *sep_dest = b';' as u16; // L';'

                                        if VersionStringBytes != 0 {
                                            let ver_dest = (UserBuffer as *mut u8).add(
                                                (NextEntry + BaseLength + FileNameBytes + core::mem::size_of::<u16>() as u32)
                                                    as usize,
                                            );
                                            core::ptr::copy_nonoverlapping(
                                                (*ThisDirent).CdFileName.VersionString.Buffer as *const u8,
                                                ver_dest,
                                                VersionStringBytes as usize,
                                            );
                                        }
                                    }
                                }

                                // Short name handling (only if we are returning success and appropriate class).
                                if (Status == STATUS_SUCCESS)
                                    && (irp_sp.Parameters.QueryDirectory.FileInformationClass
                                        == FileBothDirectoryInformation
                                        || irp_sp.Parameters.QueryDirectory.FileInformationClass
                                            == FileIdBothDirectoryInformation)
                                    && ((*Ccb).SearchExpression.VersionString.Length as u32) == 0
                                    && ((*ThisDirent).Flags & DIRENT_FLAG_CONSTANT_ENTRY as u8) == 0
                                {
                                    if FileContext.ShortName.FileName.Length != 0 {
                                        core::ptr::copy_nonoverlapping(
                                            FileContext.ShortName.FileName.Buffer as *const u8,
                                            (*DirInfo).ShortName.as_mut_ptr() as *mut u8,
                                            FileContext.ShortName.FileName.Length as usize,
                                        );
                                        (*DirInfo).ShortNameLength =
                                            FileContext.ShortName.FileName.Length as i8;
                                    } else {
                                        let is_8dot3_name = CdIs8dot3Name(
                                            IrpContext,
                                            (*ThisDirent).CdFileName.FileName,
                                        );
                                        if is_8dot3_name == 0 {
                                            CdGenerate8dot3Name(
                                                IrpContext,
                                                core::ptr::addr_of!((*ThisDirent).CdCaseFileName.FileName),
                                                (*ThisDirent).DirentOffset,
                                                (*DirInfo).ShortName.as_mut_ptr(),
                                                core::ptr::addr_of_mut!(FileContext.ShortName.FileName.Length),
                                            );
                                            (*DirInfo).ShortNameLength =
                                                FileContext.ShortName.FileName.Length as i8;
                                        }
                                    }
                                }

                                // Advance to the next entry in the buffer.
                                FileNameBytes += SeparatorBytes + VersionStringBytes;
                                Information = NextEntry + BaseLength + FileNameBytes;

                                // Update the previous entry's offset.
                                // Add2Ptr( UserBuffer, LastEntry, PULONG ): byte offset by LastEntry.
                                let prev_offset_ptr =
                                    (UserBuffer as *mut u8).add(LastEntry as usize) as *mut u32;
                                *prev_offset_ptr = NextEntry - LastEntry;

                                // Prepare for next iteration.
                                InitialQuery = 0;
                                LastEntry = NextEntry;
                                NextEntry = (Information + 7) & !7u32; // QuadAlign
                        },
                        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
                    );
                    if let Err(code) = fill {
                        Information = 0;
                        Status = code;
                        break 'outer_finally;
                    }
                } // end of while(true)

                // The C's `DoCcbUpdate = TRUE;` after its while(TRUE) is dead there too:
                // every exit from that loop is a try_leave, each setting it first.
            } // end of 'outer_finally loop
        },
        |_, _| {
            CdCleanupFileContext(IrpContext, p_file_context);

            if (*p_do_ccb_update != 0) && (((*p_status as u32) >> 30) != 3) {
                // !NT_ERROR( Status )
                CdLockFcb(IrpContext, Fcb);

                (*Ccb).CurrentDirentOffset = (**p_this_dirent).DirentOffset;
                (*Ccb).Flags &= !CCB_FLAG_ENUM_RETURN_NEXT;
                if *p_return_next_entry != 0 {
                    (*Ccb).Flags |= CCB_FLAG_ENUM_RETURN_NEXT;
                }

                CdUnlockFcb(IrpContext, Fcb);
            }

            CdReleaseFile(IrpContext, Fcb);
        },
    );

    (*Irp).IoStatus.Information = Information as _;
    CdCompleteRequest(IrpContext, Irp, Status);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdNotifyChangeDirectory(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
    Ccb: PCCB,
) -> NTSTATUS {

    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
    const SL_WATCH_TREE: u32 = 0x00000001;

    PAGED_CODE!();

    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

    CdAcquireVcbShared(IrpContext, (*IrpContext).Vcb, 0);

    // try { verify; register the notify } finally { CdReleaseVcb } (ntkrnl_sys::seh).
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            CdVerifyVcb(IrpContext, (*IrpContext).Vcb);

            FsRtlNotifyFullChangeDirectory(
                (*(*IrpContext).Vcb).NotifySync,
                core::ptr::addr_of!((*(*IrpContext).Vcb).DirNotifyList) as PLIST_ENTRY,
                Ccb as PVOID,
                core::ptr::addr_of!((*(*IrpSp).FileObject).FileName) as PSTRING,
                (((*IrpSp).Flags as u32 & SL_WATCH_TREE) != 0) as BOOLEAN,
                0,
                (*IrpSp).Parameters.NotifyDirectory.CompletionFilter,
                Irp,
                None,
                core::ptr::null_mut(),
            );
        },
        |_, _| CdReleaseVcb(IrpContext, (*IrpContext).Vcb),
    );

    CdCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn CdInitializeEnumeration(
    IrpContext: PIRP_CONTEXT,
    IrpSp: PIO_STACK_LOCATION,
    Fcb: PFCB,
    Ccb: PCCB,
    FileContext: PFILE_ENUM_CONTEXT,
    ReturnNextEntry: PBOOLEAN,
    ReturnSingleEntry: PBOOLEAN,
    InitialQuery: PBOOLEAN,
) {
    // IRP stack-location flags (sdk/include/xdk/iotypes.h).
    let SL_RESTART_SCAN: u32 = 0x00000001;
    let SL_RETURN_SINGLE_ENTRY: u32 = 0x00000002;
    let SL_INDEX_SPECIFIED: u32 = 0x00000004;
    // CCB_FLAG_* come from crate::types (real values in cdstruc.h); do not redefine.
    // TAG_ENUM_EXPRESSION = 'eedC' (cdprocs.h) = 0x65656443.
    let TAG_ENUM_EXPRESSION: u32 = 0x65656443;
    let CdPagedPool: POOL_TYPE = 1; // PagedPool

    // PAGED_CODE(); // no-op in this translation

    let Status: NTSTATUS;
    let mut FileName: *mut _UNICODE_STRING;
    let mut WildCardName: CD_NAME = core::mem::zeroed();
    let mut SearchExpression: CD_NAME = core::mem::zeroed();
    let mut CcbFlags: ULONG;

    let DirentOffset: ULONG;
    let mut LastDirentOffset: ULONG;
    let KnownOffset: BOOLEAN;
    let mut Found: BOOLEAN;

    // If the user has specified that the scan be restarted, and has specified a new query pattern, reinitialize the CCB.
    if ((*IrpSp).Flags as u32 & SL_RESTART_SCAN) != 0 {
        CdLockFcb(IrpContext, Fcb);

        FileName = (*IrpSp).Parameters.QueryDirectory.FileName;
        if !FileName.is_null() && (*FileName).Length > 0 {
            if ((*Ccb).Flags & CCB_FLAG_ENUM_MATCH_ALL) == 0 {
                CdFreePool(&mut (*Ccb).SearchExpression.FileName.Buffer);
            }
            // ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_MATCH_ALL);
            (*Ccb).Flags &= !CCB_FLAG_ENUM_MATCH_ALL;
            // ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_INITIALIZED);
            (*Ccb).Flags &= !CCB_FLAG_ENUM_INITIALIZED;
            // ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD);
            (*Ccb).Flags &= !CCB_FLAG_ENUM_NAME_EXP_HAS_WILD;
        }

        CdUnlockFcb(IrpContext, Fcb);
    }

    // If this is the initial query then build a search expression from the input file name.
    if ((*Ccb).Flags & CCB_FLAG_ENUM_INITIALIZED) == 0 {
        FileName = (*IrpSp).Parameters.QueryDirectory.FileName;
        CcbFlags = 0;

        // If the filename is not specified or is a single '*' then we will match all names.
        if FileName.is_null()
            || (*FileName).Buffer.is_null()
            || (*FileName).Length == 0
            || ((*FileName).Length == 2u16 && (*FileName).Buffer.read_unaligned() == '*' as u16)
        {
            // SetFlag( CcbFlags, CCB_FLAG_ENUM_MATCH_ALL );
            CcbFlags |= CCB_FLAG_ENUM_MATCH_ALL;
            // RtlZeroMemory( &SearchExpression, sizeof( SearchExpression ) );
            core::ptr::write_bytes(
                &mut SearchExpression as *mut CD_NAME as *mut u8,
                0,
                core::mem::size_of::<CD_NAME>(),
            );
        } else {
            // Create a CdName to check for wild cards.
            WildCardName.FileName = *FileName;
            CdConvertNameToCdName(IrpContext, &mut WildCardName);

            // The name better have at least one character.
            if WildCardName.FileName.Length == 0 {
                CdRaiseStatus(IrpContext, STATUS_INVALID_PARAMETER);
            }

            // Check for wildcards in the separate components.
            if FsRtlDoesNameContainWildCards(&mut WildCardName.FileName) != 0 {
                // SetFlag( CcbFlags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD );
                CcbFlags |= CCB_FLAG_ENUM_NAME_EXP_HAS_WILD;
            }

            if (WildCardName.VersionString.Length != 0)
                && (FsRtlDoesNameContainWildCards(&mut WildCardName.VersionString) != 0)
            {
                // SetFlag( CcbFlags, CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD );
                CcbFlags |= CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD;

                // Check if this is a wild card only and match all version strings.
                if (WildCardName.VersionString.Length == 2u16)
                    && (WildCardName.VersionString.Buffer.read_unaligned() == '*' as u16)
                {
                    // SetFlag( CcbFlags, CCB_FLAG_ENUM_VERSION_MATCH_ALL );
                    CcbFlags |= CCB_FLAG_ENUM_VERSION_MATCH_ALL;
                }
            }

            // Now create the search expression to store in the Ccb.
            SearchExpression.FileName.Buffer = FsRtlAllocatePoolWithTag(
                CdPagedPool,
                (*FileName).Length as ULONG,
                TAG_ENUM_EXPRESSION,
            ) as *mut u16;

            SearchExpression.FileName.MaximumLength = (*FileName).Length;

            // Either copy the name directly or perform the upcase.
            if ((*Ccb).Flags & CCB_FLAG_IGNORE_CASE) != 0 {
                Status = RtlUpcaseUnicodeString(
                    &mut SearchExpression.FileName as *mut _UNICODE_STRING,
                    FileName,
                    0,
                );
                // This should never fail.
                // __analysis_assert( Status == STATUS_SUCCESS );
                // NT_ASSERT( Status == STATUS_SUCCESS );
                // NT_ASSERT( Status == STATUS_SUCCESS );
                NT_ASSERT!((Status == STATUS_SUCCESS) as i32);
            } else {
                // RtlCopyMemory
                core::ptr::copy_nonoverlapping(
                    (*FileName).Buffer as *const u8,
                    SearchExpression.FileName.Buffer as *mut u8,
                    (*FileName).Length as usize,
                );
            }

            // Now split into the separate name and version components.
            SearchExpression.FileName.Length = WildCardName.FileName.Length;
            SearchExpression.VersionString.Length = WildCardName.VersionString.Length;
            SearchExpression.VersionString.MaximumLength = WildCardName.VersionString.MaximumLength;

            // SearchExpression.VersionString.Buffer = Add2Ptr(...)
            let byte_offset = (SearchExpression.FileName.Length as usize) + 2usize; // sizeof(WCHAR) = 2
            SearchExpression.VersionString.Buffer =
                (SearchExpression.FileName.Buffer as *mut u8).add(byte_offset) as *mut u16;
        }

        // But we do not want to return the constant "." and ".." entries for the root directory...
        if Fcb as *mut core::ffi::c_void == (*(*Fcb).Vcb).RootIndexFcb as *mut core::ffi::c_void {
            // SetFlag( CcbFlags, CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY );
            CcbFlags |= CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY;
        }

        // Now lock the Fcb in order to update the Ccb with the initial enumeration values.
        CdLockFcb(IrpContext, Fcb);

        // Check again that this is the initial search.
        if ((*Ccb).Flags & CCB_FLAG_ENUM_INITIALIZED) == 0 {
            // Update the values in the Ccb.
            (*Ccb).CurrentDirentOffset = (*(core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX)).StreamOffset;
            (*Ccb).SearchExpression = SearchExpression;

            // Set the appropriate flags in the Ccb.
            // SetFlag( Ccb->Flags, CcbFlags | CCB_FLAG_ENUM_INITIALIZED );
            (*Ccb).Flags |= CcbFlags | CCB_FLAG_ENUM_INITIALIZED;
        } else {
            // Otherwise cleanup any buffer allocated here.
            if (CcbFlags & CCB_FLAG_ENUM_MATCH_ALL) == 0 {
                CdFreePool(&mut SearchExpression.FileName.Buffer);
            }
        }
    } else {
        // Otherwise lock the Fcb so we can read the current enumeration values.
        CdLockFcb(IrpContext, Fcb);
    }

    // Capture the current state of the enumeration.
    // If the user specified an index then use his offset. We always return the next entry in this case.
    if ((*IrpSp).Flags as u32 & SL_INDEX_SPECIFIED) != 0 {
        KnownOffset = 0; // FALSE
        DirentOffset = (*IrpSp).Parameters.QueryDirectory.FileIndex;
        *ReturnNextEntry = 1; // TRUE
    } else if ((*IrpSp).Flags as u32 & SL_RESTART_SCAN) != 0 {
        // If we are restarting the scan then go from the self entry.
        KnownOffset = 1; // TRUE
        DirentOffset = (*(core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX)).StreamOffset;
        *ReturnNextEntry = 0; // FALSE
    } else {
        // Otherwise use the values from the Ccb.
        KnownOffset = 1; // TRUE
        DirentOffset = (*Ccb).CurrentDirentOffset;
        *ReturnNextEntry = (((*Ccb).Flags & CCB_FLAG_ENUM_RETURN_NEXT) != 0) as u8;
    }

    // Unlock the Fcb.
    CdUnlockFcb(IrpContext, Fcb);

    // Tell our caller this is the initial query.
    *InitialQuery = 0; // FALSE
    if (DirentOffset == (*(core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX)).StreamOffset) && (*ReturnNextEntry == 0) {
        *InitialQuery = 1; // TRUE
    }

    // If there is no file object then create it now.
    CdVerifyOrCreateDirStreamFile(IrpContext, Fcb);

    // Determine the offset in the stream to position the FileContext.
    if KnownOffset != 0 {
        CdLookupInitialFileDirent(IrpContext, Fcb, FileContext, DirentOffset);
    } else {
        LastDirentOffset = (*(core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX)).StreamOffset;
        Found = 1; // TRUE

        CdLookupInitialFileDirent(IrpContext, Fcb, FileContext, LastDirentOffset);

        // If the requested offset is prior to the beginning offset in the stream then don't return the next entry.
        if DirentOffset < LastDirentOffset {
            *ReturnNextEntry = 0; // FALSE
        } else {
            // Else look for the last entry which ends past the desired index.
            loop {
                // If we have passed the index value then exit.
                if (*(*FileContext).InitialDirent).Dirent.DirentOffset > DirentOffset {
                    Found = 0; // FALSE
                    break;
                }

                // Remember the current position in case we need to go back.
                LastDirentOffset = (*(*FileContext).InitialDirent).Dirent.DirentOffset;

                // Exit if the next entry is beyond the desired index value.
                if LastDirentOffset
                    + (*(*FileContext).InitialDirent).Dirent.DirentLength
                    > DirentOffset
                {
                    break;
                }

                Found = CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext);

                if Found == 0 {
                    break;
                }
            }

            // If we didn't find the entry then go back to the last known entry.
            if Found == 0 {
                CdCleanupFileContext(IrpContext, FileContext);
                CdInitializeFileContext(IrpContext, FileContext);
                CdLookupInitialFileDirent(IrpContext, Fcb, FileContext, LastDirentOffset);
            }
        }
    }

    // Only update the dirent name if we will need it for some reason.
    (*FileContext).ShortName.FileName.Length = 0;

    if (*ReturnNextEntry == 0) || ((*Ccb).SearchExpression.VersionString.Length == 0) {
        // Update the name in the dirent into filename and version components.
        CdUpdateDirentName(
            IrpContext,
            core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent),
            ((((*Ccb).Flags & CCB_FLAG_IGNORE_CASE) != 0) as BOOLEAN) as _,
        );
    }

    // Look at the flag in the IrpSp indicating whether to return just one entry.
    *ReturnSingleEntry = 0; // FALSE
    if ((*IrpSp).Flags as u32 & SL_RETURN_SINGLE_ENTRY) != 0 {
        *ReturnSingleEntry = 1; // TRUE
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdEnumerateIndex(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *const CCB,
    FileContext: *mut FILE_ENUM_CONTEXT,
    ReturnNextEntry: i32,
) -> i32 {
    PAGED_CODE!();

    let mut PreviousDirent: *mut DIRENT = core::ptr::null_mut();
    let mut ThisDirent: *mut DIRENT;

    // &FileContext->InitialDirent->Dirent
    ThisDirent = {
        let initial_dirent = (*FileContext).InitialDirent;
        core::ptr::addr_of_mut!((*initial_dirent).Dirent)
    };

    let mut Found: i32 = 0;
    let mut return_next = ReturnNextEntry;

    loop {
        if return_next != 0 {
            if CdLookupNextInitialFileDirent(IrpContext, (*Ccb).Fcb, FileContext) == 0 {
                break;
            }
            PreviousDirent = ThisDirent;

            // Recompute ThisDirent from updated InitialDirent
            let initial_dirent = (*FileContext).InitialDirent;
            ThisDirent = core::ptr::addr_of_mut!((*initial_dirent).Dirent);

            CdUpdateDirentName(
                IrpContext,
                ThisDirent,
                (FlagOn((*Ccb).Flags, CCB_FLAG_IGNORE_CASE) as BOOLEAN) as _,
            );
        } else {
            return_next = 1;
        }

        // Don't bother if we have a constant entry and are ignoring them.
        if FlagOn((*ThisDirent).Flags as u32, DIRENT_FLAG_CONSTANT_ENTRY) != 0
            && FlagOn((*Ccb).Flags, CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY) != 0
        {
            continue;
        }

        // Look at the current entry if it is not an associated file
        if FlagOn((*ThisDirent).DirentFlags as u32, CD_ATTRIBUTE_ASSOC) == 0 {
            // Check if this entry matches the previous entry except
            // for version number and whether we should return the entry.
            if PreviousDirent.is_null()
                || ((*Ccb).SearchExpression.VersionString.Length != 0)
                || ((*PreviousDirent).CdCaseFileName.FileName.Length
                    != (*ThisDirent).CdCaseFileName.FileName.Length)
                || FlagOn((*PreviousDirent).DirentFlags as u32, CD_ATTRIBUTE_ASSOC) != 0
                || RtlEqualMemory(
                    (*PreviousDirent).CdCaseFileName.FileName.Buffer as *const core::ffi::c_void,
                    (*ThisDirent).CdCaseFileName.FileName.Buffer as *const core::ffi::c_void,
                    (*ThisDirent).CdCaseFileName.FileName.Length as usize,
                ) == 0
            {
                // If we match all names then return.
                if FlagOn((*Ccb).Flags, CCB_FLAG_ENUM_MATCH_ALL) != 0 {
                    (*FileContext).ShortName.FileName.Length = 0;
                    Found = 1;
                    break;
                }

                // Check if the long name matches the search expression.
                if CdIsNameInExpression(
                    IrpContext,
                    core::ptr::addr_of!((*ThisDirent).CdCaseFileName) as *mut CD_NAME,
                    core::ptr::addr_of!((*Ccb).SearchExpression) as *mut CD_NAME,
                    (*Ccb).Flags,
                    1, // TRUE
                ) != 0
                {
                    Found = 1;
                    (*FileContext).ShortName.FileName.Length = 0;
                    break;
                }

                // The long name didn't match — try short name.
                if ((*Ccb).SearchExpression.VersionString.Length == 0)
                    && FlagOn((*ThisDirent).Flags as u32, DIRENT_FLAG_CONSTANT_ENTRY) == 0
                    && CdIs8dot3Name(IrpContext, (*ThisDirent).CdFileName.FileName) == 0
                {
                    CdGenerate8dot3Name(
                        IrpContext,
                        core::ptr::addr_of!((*ThisDirent).CdCaseFileName.FileName),
                        (*ThisDirent).DirentOffset,
                        (*FileContext).ShortName.FileName.Buffer,
                        core::ptr::addr_of_mut!((*FileContext).ShortName.FileName.Length),
                    );

                    // Check if this name matches.
                    if CdIsNameInExpression(
                        IrpContext,
                        core::ptr::addr_of_mut!((*FileContext).ShortName),
                        core::ptr::addr_of!((*Ccb).SearchExpression) as *mut CD_NAME,
                        (*Ccb).Flags,
                        0, // FALSE
                    ) != 0
                    {
                        Found = 1;
                        break;
                    }
                }
            }
        }
    }

    // Walk through all file dirents if we found an entry.
    if Found != 0 {
        CdLookupLastFileDirent(IrpContext, (*Ccb).Fcb, FileContext);
    }

    Found
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
//     This module implements the File Directory Control routines for Cdfs called
//     by the Fsd/Fsp dispatch drivers.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_DIRCTRL)
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PIO_STACK_LOCATION IrpSp,
//      PFCB Fcb,
//      PCCB Ccb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdNotifyChangeDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PIO_STACK_LOCATION IrpSp,
//      PCCB Ccb
//     );
//
// VOID
// CdInitializeEnumeration (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFCB Fcb,
//      PCCB Ccb,
//      PFILE_ENUM_CONTEXT FileContext,
//      PBOOLEAN ReturnNextEntry,
//      PBOOLEAN ReturnSingleEntry,
//      PBOOLEAN InitialQuery
//     );
//
// BOOLEAN
// CdEnumerateIndex (
//      PIRP_CONTEXT IrpContext,
//      PCCB Ccb,
//      PFILE_ENUM_CONTEXT FileContext,
//      BOOLEAN ReturnNextEntry
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonDirControl)
// #pragma alloc_text(PAGE, CdEnumerateIndex)
// #pragma alloc_text(PAGE, CdInitializeEnumeration)
// #pragma alloc_text(PAGE, CdNotifyChangeDirectory)
// #pragma alloc_text(PAGE, CdQueryDirectory)
// #endif
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonDirControl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the entry point for the directory control operations.  These
//     are directory enumerations and directory notify calls.  We verify the
//     user's handle is for a directory and then call the appropriate routine.
//
// Arguments:
//
//     Irp - Irp for this request.
//
// Return Value:
//
//     NTSTATUS - Status returned from the lower level routines.
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the user file object and fail this request if it is not
//     //  a user directory.
//     //
//
//     if (CdDecodeFileObject( IrpContext,
//                             IrpSp->FileObject,
//                             &Fcb,
//                             &Ccb ) != UserDirectoryOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  We know this is a directory control so we'll case on the
//     //  minor function, and call a internal worker routine to complete
//     //  the irp.
//     //
//
//     switch (IrpSp->MinorFunction) {
//
//     case IRP_MN_QUERY_DIRECTORY:
//
//         Status = CdQueryDirectory( IrpContext, Irp, IrpSp, Fcb, Ccb );
//         break;
//
//     case IRP_MN_NOTIFY_CHANGE_DIRECTORY:
//
//         Status = CdNotifyChangeDirectory( IrpContext, Irp, IrpSp, Ccb );
//         break;
//
//     default:
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     return Status;
// }
//
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PIO_STACK_LOCATION IrpSp,
//      PFCB Fcb,
//      PCCB Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query directory operation.  It is responsible
//     for either completing of enqueuing the input Irp.  We store the state of the
//     search in the Ccb.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     IrpSp - Stack location for this Irp.
//
//     Fcb - Fcb for this directory.
//
//     Ccb - Ccb for this directory open.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Information = 0;
//
//     ULONG LastEntry = 0;
//     ULONG NextEntry = 0;
//
//     ULONG FileNameBytes;
//     ULONG SeparatorBytes;
//     ULONG VersionStringBytes;
//
//     FILE_ENUM_CONTEXT FileContext;
//     PDIRENT ThisDirent = NULL;
//     BOOLEAN InitialQuery;
//     BOOLEAN ReturnNextEntry = FALSE;
//     BOOLEAN ReturnSingleEntry;
//     BOOLEAN Found;
//     BOOLEAN DoCcbUpdate = FALSE;
//
//     PCHAR UserBuffer;
//     ULONG BytesRemainingInBuffer;
//
//     ULONG BaseLength;
//
//     PFILE_BOTH_DIR_INFORMATION DirInfo = NULL;
//     PFILE_NAMES_INFORMATION NamesInfo;
//     PFILE_ID_FULL_DIR_INFORMATION IdFullDirInfo;
//     PFILE_ID_BOTH_DIR_INFORMATION IdBothDirInfo;
//
//     PAGED_CODE();
//
//     //
//     //  Check if we support this search mode.  Also remember the size of the base part of
//     //  each of these structures.
//     //
//
//     switch (IrpSp->Parameters.QueryDirectory.FileInformationClass) {
//
//     case FileDirectoryInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_DIRECTORY_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     case FileFullDirectoryInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_FULL_DIR_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     case FileIdFullDirectoryInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_ID_FULL_DIR_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     case FileNamesInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_NAMES_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     case FileBothDirectoryInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_BOTH_DIR_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     case FileIdBothDirectoryInformation:
//
//         BaseLength = FIELD_OFFSET( FILE_ID_BOTH_DIR_INFORMATION,
//                                    FileName[0] );
//         break;
//
//     default:
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_INFO_CLASS );
//         return STATUS_INVALID_INFO_CLASS;
//     }
//
//     //
//     //  Get the user buffer.
//     //
//
//     CdMapUserBuffer( IrpContext, &UserBuffer);
//
//     //
//     //  Initialize our search context.
//     //
//
//     CdInitializeFileContext( IrpContext, &FileContext );
//
//     //
//     //  Acquire the directory.
//     //
//
//     CdAcquireFileShared( IrpContext, Fcb );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb is still good.
//         //
//
//         CdVerifyFcbOperation( IrpContext, Fcb );
//
//         //
//         //  Start by getting the initial state for the enumeration.  This will set up the Ccb with
//         //  the initial search parameters and let us know the starting offset in the directory
//         //  to search.
//         //
//
//         CdInitializeEnumeration( IrpContext,
//                                  IrpSp,
//                                  Fcb,
//                                  Ccb,
//                                  &FileContext,
//                                  &ReturnNextEntry,
//                                  &ReturnSingleEntry,
//                                  &InitialQuery );
//
//         //
//         //  The current dirent is stored in the InitialDirent field.  We capture
//         //  this here so that we have a valid restart point even if we don't
//         //  find a single entry.
//         //
//
//         ThisDirent = &FileContext.InitialDirent->Dirent;
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
//         while (TRUE) {
//
//             //
//             //  If the user had requested only a single match and we have
//             //  returned that, then we stop at this point.  We update the Ccb with
//             //  the status based on the last entry returned.
//             //
//
//             if ((NextEntry != 0) && ReturnSingleEntry) {
//
//                 DoCcbUpdate = TRUE;
//                 try_leave( Status );
//             }
//
//             //
//             //  We try to locate the next matching dirent.  Our search if based on a starting
//             //  dirent offset, whether we should return the current or next entry, whether
//             //  we should be doing a short name search and finally whether we should be
//             //  checking for a version match.
//             //
//
//             Found = CdEnumerateIndex( IrpContext, Ccb, &FileContext, ReturnNextEntry );
//
//             //
//             //  Initialize the value for the next search.
//             //
//
//             ReturnNextEntry = TRUE;
//
//             //
//             //  If we didn't receive a dirent, then we are at the end of the
//             //  directory.  If we have returned any files, we exit with
//             //  success, otherwise we return STATUS_NO_MORE_FILES.
//             //
//
//             if (!Found) {
//
//                 if (NextEntry == 0) {
//
//                     Status = STATUS_NO_MORE_FILES;
//
//                     if (InitialQuery) {
//
//                         Status = STATUS_NO_SUCH_FILE;
//                     }
//                 }
//
//                 DoCcbUpdate = TRUE;
//                 try_leave( Status );
//             }
//
//             //
//             //  Remember the dirent for the file we just found.
//             //
//
//             ThisDirent = &FileContext.InitialDirent->Dirent;
//
//             //
//             //  Here are the rules concerning filling up the buffer:
//             //
//             //  1.  The Io system garentees that there will always be
//             //      enough room for at least one base record.
//             //
//             //  2.  If the full first record (including file name) cannot
//             //      fit, as much of the name as possible is copied and
//             //      STATUS_BUFFER_OVERFLOW is returned.
//             //
//             //  3.  If a subsequent record cannot completely fit into the
//             //      buffer, none of it (as in 0 bytes) is copied, and
//             //      STATUS_SUCCESS is returned.  A subsequent query will
//             //      pick up with this record.
//             //
//
//             //
//             //  Let's compute the number of bytes we need to transfer the current entry.
//             //
//
//             SeparatorBytes =
//             VersionStringBytes = 0;
//
//             //
//             //  We can look directly at the dirent that we found.
//             //
//
//             FileNameBytes = ThisDirent->CdFileName.FileName.Length;
//
//             //
//             //  Compute the number of bytes for the version string if
//             //  we will return this. Allow directories with illegal ";".
//             //
//
//             if (((Ccb->SearchExpression.VersionString.Length != 0) ||
//                  (FlagOn(ThisDirent->DirentFlags, CD_ATTRIBUTE_DIRECTORY))) &&
//                 (ThisDirent->CdFileName.VersionString.Length != 0)) {
//
//                 SeparatorBytes = 2;
//
//                 VersionStringBytes = ThisDirent->CdFileName.VersionString.Length;
//             }
//
//             //
//             //  If the slot for the next entry would be beyond the length of the
//             //  user's buffer just exit (we know we've returned at least one entry
//             //  already). This will happen when we align the pointer past the end.
//             //
//
//             if (NextEntry > IrpSp->Parameters.QueryDirectory.Length) {
//
//                 ReturnNextEntry = FALSE;
//                 DoCcbUpdate = TRUE;
//                 try_leave( Status = STATUS_SUCCESS );
//             }
//
//             //
//             //  Compute the number of bytes remaining in the buffer.  Round this
//             //  down to a WCHAR boundary so we can copy full characters.
//             //
//
//             BytesRemainingInBuffer = IrpSp->Parameters.QueryDirectory.Length - NextEntry;
//             ClearFlag( BytesRemainingInBuffer, 1 );
//
//             //
//             //  If this won't fit and we have returned a previous entry then just
//             //  return STATUS_SUCCESS.
//             //
//
//             if ((BaseLength + FileNameBytes + SeparatorBytes + VersionStringBytes) > BytesRemainingInBuffer) {
//
//                 //
//                 //  If we already found an entry then just exit.
//                 //
//
//                 if (NextEntry != 0) {
//
//                     ReturnNextEntry = FALSE;
//                     DoCcbUpdate = TRUE;
//                     try_leave( Status = STATUS_SUCCESS );
//                 }
//
//                 //
//                 //  Don't even try to return the version string if it doesn't all fit.
//                 //  Reduce the FileNameBytes to just fit in the buffer.
//                 //
//
//                 if ((BaseLength + FileNameBytes) > BytesRemainingInBuffer) {
//
//                     FileNameBytes = BytesRemainingInBuffer - BaseLength;
//                 }
//
//                 //
//                 //  Don't return any version string bytes.
//                 //
//
//                 VersionStringBytes =
//                 SeparatorBytes = 0;
//
//                 //
//                 //  Use a status code of STATUS_BUFFER_OVERFLOW.  Also set
//                 //  ReturnSingleEntry so that we will exit the loop at the top.
//                 //
//
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 ReturnSingleEntry = TRUE;
//             }
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
//                 //
//                 //  Zero and initialize the base part of the current entry.
//                 //
//
//                 RtlZeroMemory( Add2Ptr( UserBuffer, NextEntry, PVOID ),
//                                BaseLength );
//
//                 //
//                 //  Now we have an entry to return to our caller.
//                 //  We'll case on the type of information requested and fill up
//                 //  the user buffer if everything fits.
//                 //
//
//                 switch (IrpSp->Parameters.QueryDirectory.FileInformationClass) {
//
//                 case FileBothDirectoryInformation:
//                 case FileFullDirectoryInformation:
//                 case FileIdBothDirectoryInformation:
//                 case FileIdFullDirectoryInformation:
//                 case FileDirectoryInformation:
//
//                     DirInfo = Add2Ptr( UserBuffer, NextEntry, PFILE_BOTH_DIR_INFORMATION );
//
//                     //
//                     //  Use the create time for all the time stamps.
//                     //
//
//                     CdConvertCdTimeToNtTime( IrpContext,
//                                              FileContext.InitialDirent->Dirent.CdTime,
//                                              &DirInfo->CreationTime );
//
//                     DirInfo->LastWriteTime = DirInfo->ChangeTime = DirInfo->CreationTime;
//
//                     //
//                     //  Set the attributes and sizes separately for directories and
//                     //  files.
//                     //
//
//                     if (FlagOn( ThisDirent->DirentFlags, CD_ATTRIBUTE_DIRECTORY )) {
//
//                         DirInfo->EndOfFile.QuadPart = DirInfo->AllocationSize.QuadPart = 0;
//
//                         SetFlag( DirInfo->FileAttributes, FILE_ATTRIBUTE_DIRECTORY);
//
//                     } else {
//
//                         DirInfo->EndOfFile.QuadPart = FileContext.FileSize;
//                         DirInfo->AllocationSize.QuadPart = LlSectorAlign( FileContext.FileSize );
//
//                         SetFlag( DirInfo->FileAttributes, FILE_ATTRIBUTE_READONLY);
//                     }
//
//                     if (FlagOn( ThisDirent->DirentFlags,
//                                 CD_ATTRIBUTE_HIDDEN )) {
//
//                         SetFlag( DirInfo->FileAttributes, FILE_ATTRIBUTE_HIDDEN );
//                     }
//
//                     DirInfo->FileIndex = ThisDirent->DirentOffset;
//
//                     DirInfo->FileNameLength = FileNameBytes + SeparatorBytes + VersionStringBytes;
//
//                     break;
//
//                 case FileNamesInformation:
//
//                     NamesInfo = Add2Ptr( UserBuffer, NextEntry, PFILE_NAMES_INFORMATION );
//
//                     NamesInfo->FileIndex = ThisDirent->DirentOffset;
//
//                     NamesInfo->FileNameLength = FileNameBytes + SeparatorBytes + VersionStringBytes;
//
//                     break;
//
//                 /* ReactOS Change: GCC "enumeration value not handled in switch" */
//                 default: break;
//                 }
//
//                 //
//                 //  Fill in the FileId
//                 //
//
//                 switch (IrpSp->Parameters.QueryDirectory.FileInformationClass) {
//
//                 case FileIdBothDirectoryInformation:
//
//                     IdBothDirInfo = Add2Ptr( UserBuffer, NextEntry, PFILE_ID_BOTH_DIR_INFORMATION );
//                     CdSetFidFromParentAndDirent( IdBothDirInfo->FileId, Fcb, ThisDirent );
//                     break;
//
//                 case FileIdFullDirectoryInformation:
//
//                     IdFullDirInfo = Add2Ptr( UserBuffer, NextEntry, PFILE_ID_FULL_DIR_INFORMATION );
//                     CdSetFidFromParentAndDirent( IdFullDirInfo->FileId, Fcb, ThisDirent );
//                     break;
//
//                 default:
//                     break;
//                 }
//
//                 //
//                 //  Now copy as much of the name as possible.  We also may have a version
//                 //  string to copy.
//                 //
//
//                 if (FileNameBytes != 0) {
//
//                     //
//                     //  This is a Unicode name, we can copy the bytes directly.
//                     //
//
//                     RtlCopyMemory( Add2Ptr( UserBuffer, NextEntry + BaseLength, PVOID ),
//                                    ThisDirent->CdFileName.FileName.Buffer,
//                                    FileNameBytes );
//
//                     if (SeparatorBytes != 0) {
//
//                         *(Add2Ptr( UserBuffer,
//                                    NextEntry + BaseLength + FileNameBytes,
//                                    PWCHAR )) = L';';
//
//                         if (VersionStringBytes != 0) {
//
//                             RtlCopyMemory( Add2Ptr( UserBuffer,
//                                                     NextEntry + BaseLength + FileNameBytes + sizeof( WCHAR ),
//                                                     PVOID ),
//                                            ThisDirent->CdFileName.VersionString.Buffer,
//                                            VersionStringBytes );
//                         }
//                     }
//                 }
//
//                 //
//                 //  Fill in the short name if we got STATUS_SUCCESS.  The short name
//                 //  may already be in the file context.  Otherwise we will check
//                 //  whether the long name is 8.3.  Special case the self and parent
//                 //  directory names.
//                 //
//
//                 if ((Status == STATUS_SUCCESS) &&
//                     (IrpSp->Parameters.QueryDirectory.FileInformationClass == FileBothDirectoryInformation ||
//                      IrpSp->Parameters.QueryDirectory.FileInformationClass == FileIdBothDirectoryInformation) &&
//                     (Ccb->SearchExpression.VersionString.Length == 0) &&
//                     !FlagOn( ThisDirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY )) {
//
//                     //
//                     //  If we already have the short name then copy into the user's buffer.
//                     //
//
//                     if (FileContext.ShortName.FileName.Length != 0) {
//
//                         RtlCopyMemory( DirInfo->ShortName,
//                                        FileContext.ShortName.FileName.Buffer,
//                                        FileContext.ShortName.FileName.Length );
//
//                         DirInfo->ShortNameLength = (CCHAR) FileContext.ShortName.FileName.Length;
//
//                     //
//                     //  If the short name length is currently zero then check if
//                     //  the long name is not 8.3.  We can copy the short name in
//                     //  unicode form directly into the caller's buffer.
//                     //
//
//                     } else {
//
//                         if (!CdIs8dot3Name( IrpContext,
//                                             ThisDirent->CdFileName.FileName )) {
//
//                             CdGenerate8dot3Name( IrpContext,
//                                                  &ThisDirent->CdCaseFileName.FileName,
//                                                  ThisDirent->DirentOffset,
//                                                  DirInfo->ShortName,
//                                                  &FileContext.ShortName.FileName.Length );
//
//                             DirInfo->ShortNameLength = (CCHAR) FileContext.ShortName.FileName.Length;
//                         }
//                     }
//
//                 }
//
//                 //
//                 //  Sum the total number of bytes for the information field.
//                 //
//
//                 FileNameBytes += SeparatorBytes + VersionStringBytes;
//
//                 //
//                 //  Update the information with the number of bytes stored in the
//                 //  buffer.  We quad-align the existing buffer to add any necessary
//                 //  pad bytes.
//                 //
//
//                 Information = NextEntry + BaseLength + FileNameBytes;
//
//                 //
//                 //  Go back to the previous entry and fill in the update to this entry.
//                 //
//
//                 *(Add2Ptr( UserBuffer, LastEntry, PULONG )) = NextEntry - LastEntry;
//
//                 //
//                 //  Set up our variables for the next dirent.
//                 //
//
//                 InitialQuery = FALSE;
//
//                 LastEntry = NextEntry;
//                 NextEntry = QuadAlign( Information );
//
// #ifdef _MSC_VER
// #pragma warning(suppress: 6320)
// #endif
//             } _SEH2_EXCEPT (EXCEPTION_EXECUTE_HANDLER) {
//
//                   //
//                   //  We had a problem filling in the user's buffer, so stop and
//                   //  fail this request.  This is the only reason any exception
//                   //  would have occured at this level.
//                   //
//
//                   Information = 0;
//                   try_leave( Status = _SEH2_GetExceptionCode());
//             } _SEH2_END;
//         }
//
//         DoCcbUpdate = TRUE;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Cleanup our search context - *before* aquiring the FCB mutex exclusive,
//         //  else can block on threads in cdcreateinternalstream/purge which
//         //  hold the FCB but are waiting for all maps in this stream to be released.
//         //
//
//         CdCleanupFileContext( IrpContext, &FileContext );
//
//         //
//         //  Now we can safely aqure the FCB mutex if we need to.
//         //
//
//         if (DoCcbUpdate && !NT_ERROR( Status )) {
//
//             //
//             //  Update the Ccb to show the current state of the enumeration.
//             //
//
//             CdLockFcb( IrpContext, Fcb );
//
//             Ccb->CurrentDirentOffset = ThisDirent->DirentOffset;
//
//             ClearFlag( Ccb->Flags, CCB_FLAG_ENUM_RETURN_NEXT );
//
//             if (ReturnNextEntry) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_ENUM_RETURN_NEXT );
//             }
//
//             CdUnlockFcb( IrpContext, Fcb );
//         }
//
//         //
//         //  Release the Fcb.
//         //
//
//         CdReleaseFile( IrpContext, Fcb );
//     } _SEH2_END;
//
//     //
//     //  Complete the request here.
//     //
//
//     Irp->IoStatus.Information = Information;
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdNotifyChangeDirectory (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PIO_STACK_LOCATION IrpSp,
//      PCCB Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the notify change directory operation.  It is
//     responsible for either completing of enqueuing the input Irp.  Although there
//     will never be a notify signalled on a CDROM disk we still support this call.
//
//     We have already checked that this is not an OpenById handle.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     IrpSp - Io stack location for this request.
//
//     Ccb - Handle to the directory being watched.
//
// Return Value:
//
//     NTSTATUS - STATUS_PENDING, any other error will raise.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Always set the wait bit in the IrpContext so the initial wait can't fail.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Acquire the Vcb shared.
//     //
//
//     CdAcquireVcbShared( IrpContext, IrpContext->Vcb, FALSE );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Vcb.
//         //
//
//         CdVerifyVcb( IrpContext, IrpContext->Vcb );
//
//         //
//         //  Call the Fsrtl package to process the request.  We cast the
//         //  unicode strings to ansi strings as the dir notify package
//         //  only deals with memory matching.
//         //
//
//         FsRtlNotifyFullChangeDirectory( IrpContext->Vcb->NotifySync,
//                                         &IrpContext->Vcb->DirNotifyList,
//                                         Ccb,
//                                         (PSTRING) &IrpSp->FileObject->FileName,
//                                         BooleanFlagOn( IrpSp->Flags, SL_WATCH_TREE ),
//                                         FALSE,
//                                         IrpSp->Parameters.NotifyDirectory.CompletionFilter,
//                                         Irp,
//                                         NULL,
//                                         NULL );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb.
//         //
//
//         CdReleaseVcb( IrpContext, IrpContext->Vcb );
//     } _SEH2_END;
//
//     //
//     //  Cleanup the IrpContext.
//     //
//
//     CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//
//     return STATUS_PENDING;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdInitializeEnumeration (
//      PIRP_CONTEXT IrpContext,
//      PIO_STACK_LOCATION IrpSp,
//      PFCB Fcb,
//      PCCB Ccb,
//      PFILE_ENUM_CONTEXT FileContext,
//      PBOOLEAN ReturnNextEntry,
//      PBOOLEAN ReturnSingleEntry,
//      PBOOLEAN InitialQuery
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initialize the enumeration variables and structures.
//     We look at the state of a previous enumeration from the Ccb as well as any
//     input values from the user.  On exit we will position the FileContext at
//     a file in the directory and let the caller know whether this entry or the
//     next entry should be returned.
//
// Arguments:
//
//     IrpSp - Irp stack location for this request.
//
//     Fcb - Fcb for this directory.
//
//     Ccb - Ccb for the directory handle.
//
//     FileContext - FileContext to use for this enumeration.
//
//     ReturnNextEntry - Address to store whether we should return the entry at
//         the FileContext position or the next entry.
//
//     ReturnSingleEntry - Address to store whether we should only return
//         a single entry.
//
//     InitialQuery - Address to store whether this is the first enumeration
//         query on this handle.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PUNICODE_STRING FileName;
//     CD_NAME WildCardName;
//     CD_NAME SearchExpression;
//
//     ULONG CcbFlags;
//
//     ULONG DirentOffset;
//     ULONG LastDirentOffset;
//     BOOLEAN KnownOffset;
//
//     BOOLEAN Found;
//
//     PAGED_CODE();
//
//     //
//     //  If the user has specified that the scan be restarted, and has specicified
//     //  a new query pattern, reinitialize the CCB.
//     //
//
//     if (FlagOn( IrpSp->Flags, SL_RESTART_SCAN )) {
//
//         CdLockFcb( IrpContext, Fcb );
//
//         FileName = (PUNICODE_STRING) IrpSp->Parameters.QueryDirectory.FileName;
//         if (FileName && FileName->Length > 0) {
//
//             if (!FlagOn( Ccb->Flags, CCB_FLAG_ENUM_MATCH_ALL )) {
//
//                 CdFreePool( &Ccb->SearchExpression.FileName.Buffer );
//             }
//
//             ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_MATCH_ALL);
//             ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_INITIALIZED);
//             ClearFlag(Ccb->Flags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD);
//         }
//
//         CdUnlockFcb( IrpContext, Fcb );
//     }
//
//     //
//     //  If this is the initial query then build a search expression from the input
//     //  file name.
//     //
//
//     if (!FlagOn( Ccb->Flags, CCB_FLAG_ENUM_INITIALIZED )) {
//
//         FileName = IrpSp->Parameters.QueryDirectory.FileName;
//
//         CcbFlags = 0;
//
//         //
//         //  If the filename is not specified or is a single '*' then we will
//         //  match all names.
//         //
//
//         if ((FileName == NULL) ||
//             (FileName->Buffer == NULL) ||
//             (FileName->Length == 0) ||
//             ((FileName->Length == sizeof( WCHAR )) &&
//              (FileName->Buffer[0] == L'*'))) {
//
//             SetFlag( CcbFlags, CCB_FLAG_ENUM_MATCH_ALL );
//             RtlZeroMemory( &SearchExpression, sizeof( SearchExpression ));
//
//         //
//         //  Otherwise build the CdName from the name in the stack location.
//         //  This involves building both the name and version portions and
//         //  checking for wild card characters.  We also upcase the string if
//         //  this is a case-insensitive search.
//         //
//
//         } else {
//
//             //
//             //  Create a CdName to check for wild cards.
//             //
//
//             WildCardName.FileName = *FileName;
//
//             CdConvertNameToCdName( IrpContext, &WildCardName );
//
//             //
//             //  The name better have at least one character.
//             //
//
//             if (WildCardName.FileName.Length == 0) {
//
//                 CdRaiseStatus( IrpContext, STATUS_INVALID_PARAMETER );
//             }
//
//             //
//             //  Check for wildcards in the separate components.
//             //
//
//             if (FsRtlDoesNameContainWildCards( &WildCardName.FileName)) {
//
//                 SetFlag( CcbFlags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD );
//             }
//
//             if ((WildCardName.VersionString.Length != 0) &&
//                 (FsRtlDoesNameContainWildCards( &WildCardName.VersionString ))) {
//
//                 SetFlag( CcbFlags, CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD );
//
//                 //
//                 //  Check if this is a wild card only and match all version
//                 //  strings.
//                 //
//
//                 if ((WildCardName.VersionString.Length == sizeof( WCHAR )) &&
//                     (WildCardName.VersionString.Buffer[0] == L'*')) {
//
//                     SetFlag( CcbFlags, CCB_FLAG_ENUM_VERSION_MATCH_ALL );
//                 }
//             }
//
//             //
//             //  Now create the search expression to store in the Ccb.
//             //
//
//             SearchExpression.FileName.Buffer = FsRtlAllocatePoolWithTag( CdPagedPool,
//                                                                          FileName->Length,
//                                                                          TAG_ENUM_EXPRESSION );
//
//             SearchExpression.FileName.MaximumLength = FileName->Length;
//
//             //
//             //  Either copy the name directly or perform the upcase.
//             //
//
//             if (FlagOn( Ccb->Flags, CCB_FLAG_IGNORE_CASE )) {
//
//                 Status = RtlUpcaseUnicodeString( (PUNICODE_STRING) &SearchExpression.FileName,
//                                                  FileName,
//                                                  FALSE );
//
//                 //
//                 //  This should never fail.
//                 //
//                 __analysis_assert( Status == STATUS_SUCCESS );
//                 NT_ASSERT( Status == STATUS_SUCCESS );
//
//             } else {
//
//                 RtlCopyMemory( SearchExpression.FileName.Buffer,
//                                FileName->Buffer,
//                                FileName->Length );
//             }
//
//             //
//             //  Now split into the separate name and version components.
//             //
//
//             SearchExpression.FileName.Length = WildCardName.FileName.Length;
//             SearchExpression.VersionString.Length = WildCardName.VersionString.Length;
//             SearchExpression.VersionString.MaximumLength = WildCardName.VersionString.MaximumLength;
//
//             SearchExpression.VersionString.Buffer = Add2Ptr( SearchExpression.FileName.Buffer,
//                                                              SearchExpression.FileName.Length + sizeof( WCHAR ),
//                                                              PWCHAR );
//         }
//
//         //
//         //  But we do not want to return the constant "." and ".." entries for
//         //  the root directory, for consistency with the rest of Microsoft's
//         //  filesystems.
//         //
//
//         if (Fcb == Fcb->Vcb->RootIndexFcb) {
//
//             SetFlag( CcbFlags, CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY );
//         }
//
//         //
//         //  Now lock the Fcb in order to update the Ccb with the inital
//         //  enumeration values.
//         //
//
//         CdLockFcb( IrpContext, Fcb );
//
//         //
//         //  Check again that this is the initial search.
//         //
//
//         if (!FlagOn( Ccb->Flags, CCB_FLAG_ENUM_INITIALIZED )) {
//
//             //
//             //  Update the values in the Ccb.
//             //
//
//             Ccb->CurrentDirentOffset = Fcb->StreamOffset;
//             Ccb->SearchExpression = SearchExpression;
//
//             //
//             //  Set the appropriate flags in the Ccb.
//             //
//
//             SetFlag( Ccb->Flags, CcbFlags | CCB_FLAG_ENUM_INITIALIZED );
//
//         //
//         //  Otherwise cleanup any buffer allocated here.
//         //
//
//         } else {
//
//             if (!FlagOn( CcbFlags, CCB_FLAG_ENUM_MATCH_ALL )) {
//
//                 CdFreePool( &SearchExpression.FileName.Buffer );
//             }
//         }
//
//     //
//     //  Otherwise lock the Fcb so we can read the current enumeration values.
//     //
//
//     } else {
//
//         CdLockFcb( IrpContext, Fcb );
//     }
//
//     //
//     //  Capture the current state of the enumeration.
//     //
//     //  If the user specified an index then use his offset.  We always
//     //  return the next entry in this case.
//     //
//
//     if (FlagOn( IrpSp->Flags, SL_INDEX_SPECIFIED )) {
//
//         KnownOffset = FALSE;
//         DirentOffset = IrpSp->Parameters.QueryDirectory.FileIndex;
//         *ReturnNextEntry = TRUE;
//
//     //
//     //  If we are restarting the scan then go from the self entry.
//     //
//
//     } else if (FlagOn( IrpSp->Flags, SL_RESTART_SCAN )) {
//
//         KnownOffset = TRUE;
//         DirentOffset = Fcb->StreamOffset;
//         *ReturnNextEntry = FALSE;
//
//     //
//     //  Otherwise use the values from the Ccb.
//     //
//
//     } else {
//
//         KnownOffset = TRUE;
//         DirentOffset = Ccb->CurrentDirentOffset;
//         *ReturnNextEntry = BooleanFlagOn( Ccb->Flags, CCB_FLAG_ENUM_RETURN_NEXT );
//     }
//
//     //
//     //  Unlock the Fcb.
//     //
//
//     CdUnlockFcb( IrpContext, Fcb );
//
//     //
//     //  We have the starting offset in the directory and whether to return
//     //  that entry or the next.  If we are at the beginning of the directory
//     //  and are returning that entry, then tell our caller this is the
//     //  initial query.
//     //
//
//     *InitialQuery = FALSE;
//
//     if ((DirentOffset == Fcb->StreamOffset) &&
//         !(*ReturnNextEntry)) {
//
//         *InitialQuery = TRUE;
//     }
//
//     //
//     //  If there is no file object then create it now.
//     //
//
//     CdVerifyOrCreateDirStreamFile( IrpContext, Fcb);
//
//     //
//     //  Determine the offset in the stream to position the FileContext and
//     //  whether this offset is known to be a file offset.
//     //
//     //  If this offset is known to be safe then go ahead and position the
//     //  file context.  This handles the cases where the offset is the beginning
//     //  of the stream, the offset is from a previous search or this is the
//     //  initial query.
//     //
//
//     if (KnownOffset) {
//
//         CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, DirentOffset );
//
//     //
//     //  Otherwise we walk through the directory from the beginning until
//     //  we reach the entry which contains this offset.
//     //
//
//     } else {
//
//         LastDirentOffset = Fcb->StreamOffset;
//         Found = TRUE;
//
//         CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, LastDirentOffset );
//
//         //
//         //  If the requested offset is prior to the beginning offset in the stream
//         //  then don't return the next entry.
//         //
//
//         if (DirentOffset < LastDirentOffset) {
//
//             *ReturnNextEntry = FALSE;
//
//         //
//         //  Else look for the last entry which ends past the desired index.
//         //
//
//         } else {
//
//             //
//             //  Keep walking through the directory until we run out of
//             //  entries or we find an entry which ends beyond the input
//             //  index value.
//             //
//
//             do {
//
//                 //
//                 //  If we have passed the index value then exit.
//                 //
//
//                 if (FileContext->InitialDirent->Dirent.DirentOffset > DirentOffset) {
//
//                     Found = FALSE;
//                     break;
//                 }
//
//                 //
//                 //  Remember the current position in case we need to go back.
//                 //
//
//                 LastDirentOffset = FileContext->InitialDirent->Dirent.DirentOffset;
//
//                 //
//                 //  Exit if the next entry is beyond the desired index value.
//                 //
//
//                 if (LastDirentOffset + FileContext->InitialDirent->Dirent.DirentLength > DirentOffset) {
//
//                     break;
//                 }
//
//                 Found = CdLookupNextInitialFileDirent( IrpContext, Fcb, FileContext );
//
//             } while (Found);
//
//             //
//             //  If we didn't find the entry then go back to the last known entry.
//             //  This can happen if the index lies in the unused range at the
//             //  end of a sector.
//             //
//
//             if (!Found) {
//
//                 CdCleanupFileContext( IrpContext, FileContext );
//                 CdInitializeFileContext( IrpContext, FileContext );
//
//                 CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, LastDirentOffset );
//             }
//         }
//     }
//
//     //
//     //  Only update the dirent name if we will need it for some reason.
//     //  Don't update this name if we are returning the next entry and
//     //  the search string has a version component.
//     //
//
//     FileContext->ShortName.FileName.Length = 0;
//
//     if (!(*ReturnNextEntry) ||
//         (Ccb->SearchExpression.VersionString.Length == 0)) {
//
//         //
//         //  Update the name in the dirent into filename and version components.
//         //
//
//         CdUpdateDirentName( IrpContext,
//                             &FileContext->InitialDirent->Dirent,
//                             FlagOn( Ccb->Flags, CCB_FLAG_IGNORE_CASE ));
//     }
//
//     //
//     //  Look at the flag in the IrpSp indicating whether to return just
//     //  one entry.
//     //
//
//     *ReturnSingleEntry = FALSE;
//
//     if (FlagOn( IrpSp->Flags, SL_RETURN_SINGLE_ENTRY )) {
//
//         *ReturnSingleEntry = TRUE;
//     }
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// BOOLEAN
// CdEnumerateIndex (
//      PIRP_CONTEXT IrpContext,
//      PCCB Ccb,
//      PFILE_ENUM_CONTEXT FileContext,
//      BOOLEAN ReturnNextEntry
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the worker routine for index enumeration.  We are positioned
//     at some dirent in the directory and will either return the first match
//     at that point or look to the next entry.  The Ccb contains details about
//     the type of matching to do.  If the user didn't specify a version in
//     his search string then we only return the first version of a sequence
//     of files with versions.  We also don't return any associated files.
//
// Arguments:
//
//     Ccb - Ccb for this directory handle.
//
//     FileContext - File context already positioned at some entry in the directory.
//
//     ReturnNextEntry - Indicates if we are returning this entry or should start
//         with the next entry.
//
// Return Value:
//
//     BOOLEAN - TRUE if next entry is found, FALSE otherwise.
//
// --*/
//
// {
//     PDIRENT PreviousDirent = NULL;
//     PDIRENT ThisDirent = &FileContext->InitialDirent->Dirent;
//
//     BOOLEAN Found = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Loop until we find a match or exaust the directory.
//     //
//
//     while (TRUE) {
//
//         //
//         //  Move to the next entry unless we want to consider the current
//         //  entry.
//         //
//
//         if (ReturnNextEntry) {
//
//             if (!CdLookupNextInitialFileDirent( IrpContext, Ccb->Fcb, FileContext )) {
//
//                 break;
//             }
//
//             PreviousDirent = ThisDirent;
//             ThisDirent = &FileContext->InitialDirent->Dirent;
//
//             CdUpdateDirentName( IrpContext, ThisDirent, FlagOn( Ccb->Flags, CCB_FLAG_IGNORE_CASE ));
//
//         } else {
//
//             ReturnNextEntry = TRUE;
//         }
//
//         //
//         //  Don't bother if we have a constant entry and are ignoring them.
//         //
//
//         if (FlagOn( ThisDirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY ) &&
//             FlagOn( Ccb->Flags, CCB_FLAG_ENUM_NOMATCH_CONSTANT_ENTRY )) {
//
//             continue;
//         }
//
//         //
//         //  Look at the current entry if it is not an associated file
//         //  and the name doesn't match the previous file if the version
//         //  name is not part of the search.
//         //
//
//         if (!FlagOn( ThisDirent->DirentFlags, CD_ATTRIBUTE_ASSOC )) {
//
//             //
//             //  Check if this entry matches the previous entry except
//             //  for version number and whether we should return the
//             //  entry in that case.  Go directly to the name comparison
//             //  if:
//             //
//             //      There is no previous entry.
//             //      The search expression has a version component.
//             //      The name length doesn't match the length of the previous entry.
//             //      The base name strings don't match.
//             //
//
//             if ((PreviousDirent == NULL) ||
//                 (Ccb->SearchExpression.VersionString.Length != 0) ||
//                 (PreviousDirent->CdCaseFileName.FileName.Length != ThisDirent->CdCaseFileName.FileName.Length) ||
//                 FlagOn( PreviousDirent->DirentFlags, CD_ATTRIBUTE_ASSOC ) ||
//                 !RtlEqualMemory( PreviousDirent->CdCaseFileName.FileName.Buffer,
//                                  ThisDirent->CdCaseFileName.FileName.Buffer,
//                                  ThisDirent->CdCaseFileName.FileName.Length )) {
//
//                 //
//                 //  If we match all names then return to our caller.
//                 //
//
//                 if (FlagOn( Ccb->Flags, CCB_FLAG_ENUM_MATCH_ALL )) {
//
//                     FileContext->ShortName.FileName.Length = 0;
//                     Found = TRUE;
//                     break;
//                 }
//
//                 //
//                 //  Check if the long name matches the search expression.
//                 //
//
//                 if (CdIsNameInExpression( IrpContext,
//                                           &ThisDirent->CdCaseFileName,
//                                           &Ccb->SearchExpression,
//                                           Ccb->Flags,
//                                           TRUE )) {
//
//                     //
//                     //  Let our caller know we found an entry.
//                     //
//
//                     Found = TRUE;
//                     FileContext->ShortName.FileName.Length = 0;
//                     break;
//                 }
//
//                 //
//                 //  The long name didn't match so we need to check for a
//                 //  possible short name match.  There is no match if the
//                 //  long name is 8dot3 or the search expression has a
//                 //  version component.  Special case the self and parent
//                 //  entries.
//                 //
//
//                 if ((Ccb->SearchExpression.VersionString.Length == 0) &&
//                     !FlagOn( ThisDirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY ) &&
//                     !CdIs8dot3Name( IrpContext,
//                                     ThisDirent->CdFileName.FileName )) {
//
//                     CdGenerate8dot3Name( IrpContext,
//                                          &ThisDirent->CdCaseFileName.FileName,
//                                          ThisDirent->DirentOffset,
//                                          FileContext->ShortName.FileName.Buffer,
//                                          &FileContext->ShortName.FileName.Length );
//
//                     //
//                     //  Check if this name matches.
//                     //
//
//                     if (CdIsNameInExpression( IrpContext,
//                                               &FileContext->ShortName,
//                                               &Ccb->SearchExpression,
//                                               Ccb->Flags,
//                                               FALSE )) {
//
//                         //
//                         //  Let our caller know we found an entry.
//                         //
//
//                         Found = TRUE;
//                         break;
//                     }
//                 }
//             }
//         }
//     }
//
//     //
//     //  If we found the entry then make sure we walk through all of the
//     //  file dirents.
//     //
//
//     if (Found) {
//
//         CdLookupLastFileDirent( IrpContext, Ccb->Fcb, FileContext );
//     }
//
//     return Found;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
