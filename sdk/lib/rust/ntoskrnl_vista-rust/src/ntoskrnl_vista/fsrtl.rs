use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, LPGUID, PGUID, PIRP, PREPARSE_DATA_BUFFER, PREPARSE_GUID_DATA_BUFFER, REPARSE_DATA_BUFFER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ECP_LIST {
    pub Signature: ULONG,
    pub Flags: ULONG,
    pub EcpList: _LIST_ENTRY,
}
pub type PECP_LIST = *mut ECP_LIST;

pub type ECP_HEADER_FLAGS = ULONG;

#[repr(C)]
pub struct ECP_HEADER {
    pub Signature: ULONG,
    pub Spare: ULONG,
    pub ListEntry: _LIST_ENTRY,
    pub EcpType: GUID,
    pub CleanupCallback: PFSRTL_EXTRA_CREATE_PARAMETER_CLEANUP_CALLBACK,
    pub Flags: ECP_HEADER_FLAGS,
    pub Size: ULONG,
    pub ListAllocatedFrom: PVOID,
    pub Filter: PVOID,
}
pub type PECP_HEADER = *mut ECP_HEADER;

pub const ECP_HEADER_SIZE: usize = core::mem::size_of::<ECP_HEADER>();
#[no_mangle]
pub unsafe extern "system" fn FsRtlRemoveDotsFromPath(
    mut OriginalString: PWSTR,
    mut PathLength: USHORT,
    mut NewLength: *mut USHORT,
) -> NTSTATUS {
    let mut Length: USHORT = 0;
    let mut ReadPos: USHORT = 0;
    let mut WritePos: USHORT = 0;

    Length = (PathLength as usize / core::mem::size_of::<WCHAR>()) as USHORT;

    if Length as i32 == 3
        && *OriginalString.offset(0 as isize) as i32 == '\\' as i32
        && *OriginalString.offset(1 as isize) as i32 == '.' as i32
        && *OriginalString.offset(2 as isize) as i32 == '.' as i32
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    if Length as i32 == 2
        && *OriginalString.offset(0 as isize) as i32 == '.' as i32
        && *OriginalString.offset(1 as isize) as i32 == '.' as i32
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    if Length as i32 > 2
        && *OriginalString.offset(0 as isize) as i32 == '.' as i32
        && *OriginalString.offset(1 as isize) as i32 == '.' as i32
        && *OriginalString.offset(2 as isize) as i32 == '\\' as i32
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    ReadPos = 0 as USHORT;
    WritePos = 0 as USHORT;
    while (ReadPos as i32) < Length as i32 {
        'inner: while (ReadPos as i32) > 0 && (ReadPos as i32) < Length as i32 {
            'body: {
                if (ReadPos as i32) < Length as i32 - 1
                    && *OriginalString.offset(ReadPos as isize) as i32 == '\\' as i32
                    && *OriginalString.offset((ReadPos as i32 + 1) as isize) as i32 == '\\' as i32
                {
                    break 'body;
                }

                if *OriginalString.offset(ReadPos as isize) as i32 != '.' as i32 {
                    break 'inner;
                }

                if ReadPos as i32 == Length as i32 - 1 {
                    if *OriginalString.offset((ReadPos as i32 - 1) as isize) as i32 == '\\' as i32 {
                        if WritePos as i32 > 1 {
                            WritePos = WritePos.wrapping_sub(1);
                        }
                        break 'body;
                    }

                    *OriginalString.offset(WritePos as isize) = '.' as i32 as WCHAR;
                    WritePos = WritePos.wrapping_add(1);
                    break 'body;
                }

                if *OriginalString.offset((ReadPos as i32 + 1) as isize) as i32 == '\\' as i32 {
                    if *OriginalString.offset((ReadPos as i32 - 1) as isize) as i32 != '\\' as i32 {
                        *OriginalString.offset(WritePos as isize) = '.' as i32 as WCHAR;
                        WritePos = WritePos.wrapping_add(1);
                        break 'body;
                    }
                } else {
                    if *OriginalString.offset((ReadPos as i32 + 1) as isize) as i32 != '.' as i32
                        || *OriginalString.offset((ReadPos as i32 - 1) as isize) as i32 != '\\' as i32
                        || ((ReadPos as i32 != Length as i32 - 2)
                            && *OriginalString.offset((ReadPos as i32 + 2) as isize) as i32 != '\\' as i32)
                    {
                        *OriginalString.offset(WritePos as isize) = '.' as i32 as WCHAR;
                        WritePos = WritePos.wrapping_add(1);
                        break 'body;
                    }

                    WritePos = (WritePos as i32 - 2) as USHORT;
                    while (WritePos as i16 as i32) > 0
                        && *OriginalString.offset(WritePos as isize) as i32 != '\\' as i32
                    {
                        WritePos = WritePos.wrapping_sub(1);
                    }

                    if (WritePos as i16 as i32) < 0
                        || *OriginalString.offset(WritePos as isize) as i32 != '\\' as i32
                    {
                        return STATUS_IO_REPARSE_DATA_INVALID;
                    }

                    if WritePos as i32 == 0
                        && ReadPos as i32 == Length as i32 - 2
                    {
                        WritePos = 1 as USHORT;
                    }
                }

                ReadPos = ReadPos.wrapping_add(1);
            }
            ReadPos = ReadPos.wrapping_add(1);
        }

        if ReadPos as i32 >= Length as i32 {
            break;
        }

        *OriginalString.offset(WritePos as isize) = *OriginalString.offset(ReadPos as isize);
        ReadPos = ReadPos.wrapping_add(1);
        WritePos = WritePos.wrapping_add(1);
    }

    *NewLength = (WritePos as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT;

    while (WritePos as i32) < Length as i32 {
        *OriginalString.offset(WritePos as isize) = UNICODE_NULL;
        WritePos = WritePos.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn IsNullGuid(Guid: PGUID) -> BOOLEAN {
    if (*Guid).Data1 == 0
        && (*Guid).Data2 == 0
        && (*Guid).Data3 == 0
        && *(core::ptr::addr_of_mut!((*Guid).Data4) as *mut ULONG).add(0) == 0
        && *(core::ptr::addr_of_mut!((*Guid).Data4) as *mut ULONG).add(1) == 0
    {
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn IsEven(mut Digit: USHORT) -> BOOLEAN {
    return ((Digit & 1) != 1) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlValidateReparsePointBuffer(
    BufferLength: ULONG,
    ReparseBuffer: PREPARSE_DATA_BUFFER,
) -> NTSTATUS {
    let mut DataLength: USHORT;
    let mut ReparseTag: ULONG;
    let mut GuidBuffer: PREPARSE_GUID_DATA_BUFFER;

    /* Validate data size range */
    if BufferLength < (REPARSE_DATA_BUFFER_HEADER_SIZE) as u32 || BufferLength > MAXIMUM_REPARSE_DATA_BUFFER_SIZE {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    GuidBuffer = ReparseBuffer as PREPARSE_GUID_DATA_BUFFER;
    DataLength = (*ReparseBuffer).ReparseDataLength;
    ReparseTag = (*ReparseBuffer).ReparseTag;

    /* Validate size consistency */
    if (DataLength as ULONG + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG) != BufferLength
        && (DataLength as ULONG + REPARSE_GUID_DATA_BUFFER_HEADER_SIZE as ULONG) != BufferLength
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    /* REPARSE_DATA_BUFFER is reserved for MS tags */
    if (DataLength as ULONG + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG) == BufferLength
        && (IsReparseTagMicrosoft!(ReparseTag) as ::core::ffi::c_int) == 0
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    /* If that a GUID data buffer, its GUID cannot be null, and it cannot contain a MS tag */
    if (DataLength as ULONG + REPARSE_GUID_DATA_BUFFER_HEADER_SIZE as ULONG) == BufferLength
        && (((IsReparseTagMicrosoft!(ReparseTag) as ::core::ffi::c_int) == 0
            && IsNullGuid(core::ptr::addr_of_mut!((*GuidBuffer).ReparseGuid)) as ::core::ffi::c_int != 0)
            || (ReparseTag == IO_REPARSE_TAG_MOUNT_POINT || ReparseTag == IO_REPARSE_TAG_SYMLINK))
    {
        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    /* Check the data for MS non reserved tags */
    if (ReparseTag & 0xFFF0000) == 0
        && ReparseTag != IO_REPARSE_TAG_RESERVED_ZERO
        && ReparseTag != IO_REPARSE_TAG_RESERVED_ONE
    {
        /* If that's a mount point, validate the MountPointReparseBuffer branch */
        if ReparseTag == IO_REPARSE_TAG_MOUNT_POINT {
            /* We need information */
            if (DataLength) as i32 >= REPARSE_DATA_BUFFER_HEADER_SIZE {
                /* Substitue must be the first in row */
                if (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset == 0 {
                    /* Substitude must be null-terminated */
                    if ((*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.PrintNameOffset as usize)
                        == ((*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength as usize)
                            + core::mem::size_of_val(&UNICODE_NULL)
                    {
                        /* There must just be the Offset/Length fields + buffer + 2 null chars */
                        if DataLength as usize
                            == ((*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.PrintNameLength as usize
                                + (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength as usize
                                + (FIELD_OFFSET!(REPARSE_DATA_BUFFER, __bindgen_anon_1.MountPointReparseBuffer.PathBuffer) as usize
                                    - FIELD_OFFSET!(REPARSE_DATA_BUFFER, __bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset) as usize)
                                + 2 * core::mem::size_of_val(&UNICODE_NULL))
                        {
                            return STATUS_SUCCESS;
                        }
                    }
                }
            }
        } else {
            /* If that's not a symlink, accept the MS tag as it */
            if ReparseTag != IO_REPARSE_TAG_SYMLINK {
                return STATUS_SUCCESS;
            }

            /* We need information */
            if (DataLength) as i32 >= FIELDS_SIZE {
                /* Validate lengths */
                if (*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength != 0
                    && (*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameLength != 0
                {
                    /* Validate unicode strings */
                    if IsEven((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength) as ::core::ffi::c_int != 0
                        && IsEven((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameLength) as ::core::ffi::c_int != 0
                        && IsEven((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameOffset) as ::core::ffi::c_int != 0
                        && IsEven((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameOffset) as ::core::ffi::c_int != 0
                    {
                        // C sums these as LONG (USHORT operands, FIELD_OFFSET's LONG); every term
                        // is non-negative and bounded by a USHORT plus small offsets, so ULONG
                        // compares identically.
                        if (DataLength as ULONG + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG)
                            >= ((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameOffset as ULONG
                                + (*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength as ULONG
                                + FIELDS_SIZE as ULONG
                                + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG)
                            && (DataLength as ULONG + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG)
                                >= ((*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameLength as ULONG
                                    + (*ReparseBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameOffset as ULONG
                                    + FIELDS_SIZE as ULONG
                                    + REPARSE_DATA_BUFFER_HEADER_SIZE as ULONG)
                        {
                            return STATUS_SUCCESS;
                        }
                    }
                }
            }
        }

        return STATUS_IO_REPARSE_DATA_INVALID;
    }

    return STATUS_IO_REPARSE_TAG_INVALID;
}
#[no_mangle]
pub unsafe extern "system" fn FsRtlGetEcpListFromIrp(
    Irp: PIRP,
    EcpList: *mut PECP_LIST,
) -> NTSTATUS {
    return IoGetIrpExtraCreateParameter(Irp, (EcpList as *mut *mut _ECP_LIST).cast());
}
#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextExtraCreateParameter(
    EcpList: PECP_LIST,
    CurrentEcpContext: PVOID,
    NextEcpType: LPGUID,
    NextEcpContext: *mut PVOID,
    NextEcpContextSize: PULONG,
) -> NTSTATUS {
    let mut CurrentEntry: PECP_HEADER = core::ptr::null_mut();
    'FailEmpty: {
        /* If we have no context ... */
        if CurrentEcpContext.is_null() {
            if IsListEmpty(core::ptr::addr_of_mut!((*EcpList).EcpList)) != 0 {
                break 'FailEmpty;
            }

            /* Simply consider first entry */
            CurrentEntry = CONTAINING_RECORD!(
                (*EcpList).EcpList.Flink,
                ECP_HEADER,
                ListEntry
            );
        } else {
            /* Otherwise, consider the entry matching the given context */
            CurrentEntry = ECP_CONTEXT_TO_HEADER!(CurrentEcpContext);

            /* Make sure we didn't reach the end */
            if core::ptr::addr_of_mut!((*CurrentEntry).ListEntry)
                == core::ptr::addr_of_mut!((*EcpList).EcpList)
            {
                break 'FailEmpty;
            }
        }

        /* We must have an entry */
        if CurrentEntry.is_null() {
            break 'FailEmpty;
        }

        /* If caller wants a context, give it */
        if !NextEcpContext.is_null() {
            *NextEcpContext = ECP_HEADER_TO_CONTEXT!(CurrentEntry);
        }

        /* Same for its size (which the size minus the header overhead) */
        if !NextEcpContextSize.is_null() {
            *NextEcpContextSize = (*CurrentEntry).Size
                - (core::mem::size_of::<ECP_HEADER>() as ULONG);
        }

        /* And copy the type if asked to */
        if !NextEcpType.is_null() {
            RtlCopyMemory!(
                NextEcpType as PVOID,
                core::ptr::addr_of_mut!((*CurrentEntry).EcpType) as PVOID,
                core::mem::size_of::<GUID>(),
            );
        }

        /* Job done */
        return STATUS_SUCCESS;
    }

    /* Failure case: just zero everything */
    if !NextEcpContext.is_null() {
        *NextEcpContext = core::ptr::null_mut();
    }

    if !NextEcpContextSize.is_null() {
        *NextEcpContextSize = 0;
    }

    if !NextEcpType.is_null() {
        RtlZeroMemory!(NextEcpType as PVOID, core::mem::size_of::<GUID>());
    }

    /* And return failure */
    return STATUS_NOT_FOUND;
}
