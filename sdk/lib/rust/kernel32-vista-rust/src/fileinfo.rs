//! GetFileInformationByHandleEx.c (from Wine kernel32/file.c).

use crate::common::*;

#[no_mangle]
pub unsafe extern "system" fn GetFileInformationByHandleEx(
    handle: HANDLE,
    class: FILE_INFO_BY_HANDLE_CLASS,
    info: LPVOID,
    size: DWORD,
) -> BOOL {
    let mut io: IO_STATUS_BLOCK = core::mem::zeroed();

    let status: NTSTATUS = match class {
        FileRemoteProtocolInfo
        | FileStorageInfo
        | FileDispositionInfoEx
        | FileRenameInfoEx
        | FileCaseSensitiveInfo
        | FileNormalizedNameInfo => {
            SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            return 0;
        }

        FileStreamInfo => NtQueryInformationFile(handle, &mut io, info, size, FileStreamInformation),
        FileCompressionInfo => {
            NtQueryInformationFile(handle, &mut io, info, size, FileCompressionInformation)
        }
        FileAlignmentInfo => {
            NtQueryInformationFile(handle, &mut io, info, size, FileAlignmentInformation)
        }
        FileAttributeTagInfo => {
            NtQueryInformationFile(handle, &mut io, info, size, FileAttributeTagInformation)
        }
        FileBasicInfo => NtQueryInformationFile(handle, &mut io, info, size, FileBasicInformation),
        FileStandardInfo => {
            NtQueryInformationFile(handle, &mut io, info, size, FileStandardInformation)
        }
        FileNameInfo => NtQueryInformationFile(handle, &mut io, info, size, FileNameInformation),
        FileIdInfo => NtQueryInformationFile(handle, &mut io, info, size, FileIdInformation),

        FileIdBothDirectoryRestartInfo | FileIdBothDirectoryInfo => NtQueryDirectoryFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io,
            info,
            size,
            FileIdBothDirectoryInformation,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
            (class == FileIdBothDirectoryRestartInfo) as BOOLEAN,
        ),

        FileFullDirectoryInfo | FileFullDirectoryRestartInfo => NtQueryDirectoryFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io,
            info,
            size,
            FileFullDirectoryInformation,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
            (class == FileFullDirectoryRestartInfo) as BOOLEAN,
        ),

        FileIdExtdDirectoryInfo | FileIdExtdDirectoryRestartInfo => NtQueryDirectoryFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut io,
            info,
            size,
            FileIdExtdDirectoryInformation,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
            (class == FileIdExtdDirectoryRestartInfo) as BOOLEAN,
        ),

        // FileRenameInfo, FileDispositionInfo, FileAllocationInfo,
        // FileIoPriorityHintInfo, FileEndOfFileInfo, default
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    };

    if !NT_SUCCESS(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return 0;
    }

    TRUE as BOOL
}
