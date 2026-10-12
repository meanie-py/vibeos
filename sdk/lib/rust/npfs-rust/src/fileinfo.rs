//! fileinfo.rs — fileinfo.c: Pipes Information (query/set file information handlers + dispatch)
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty}; // ULONG, USHORT, UCHAR, BOOLEAN, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, ULONG_PTR ...
use ntkrnl_sys::ddk::*; // bindgen structs/extern fns: DEVICE_OBJECT, IRP, FILE_*_INFORMATION, ...
use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::ke::KeBugCheckEx;

// cross-module fns this file calls
use crate::fileobsup::NpDecodeFileObject;
use crate::create::NpCheckForNotify;

// File ID number for NPFS bugchecking support
// #define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_FILEINFO)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_FILEINFO;

// ---- ADDED constants (not in glue.rs) -------------------------------------
// sdk/include/xdk/iotypes.h:3382  #define FILE_ATTRIBUTE_NORMAL  0x00000080
const FILE_ATTRIBUTE_NORMAL: ULONG = 0x00000080;

/* FUNCTIONS ******************************************************************/

pub unsafe extern "system" fn NpSetBasicInfo(
    Ccb: PNP_CCB,
    Buffer: PFILE_BASIC_INFORMATION,
) -> NTSTATUS {
    // PAGED_CODE();  (debug-only; no-op under this build)
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpSetPipeInfo(
    Fcb: PNP_FCB,
    Ccb: PNP_CCB,
    Buffer: PFILE_PIPE_INFORMATION,
    NamedPipeEnd: ULONG,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let ReadQueue: PNP_DATA_QUEUE;
    let WriteQueue: PNP_DATA_QUEUE;
    // PAGED_CODE();  (debug-only; no-op under this build)

    if (*Buffer).ReadMode == FILE_PIPE_MESSAGE_MODE
        && (*Fcb).NamedPipeType as ULONG == FILE_PIPE_BYTE_STREAM_TYPE
    {
        return STATUS_INVALID_PARAMETER;
    }

    if NamedPipeEnd != FILE_PIPE_CLIENT_END {
        if NamedPipeEnd != FILE_PIPE_SERVER_END {
            // NpBugCheck(NamedPipeEnd, 0, 0);  (fileinfo.c:48)
            KeBugCheckEx(
                NPFS_FILE_SYSTEM,
                ((NPFS_BUGCHECK_FILE_ID << 16) | 48u32) as _,
                NamedPipeEnd as _,
                0 as _,
                0 as _,
            );
        }
        ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
        WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    } else {
        ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
        WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    }

    if (*Buffer).CompletionMode != FILE_PIPE_COMPLETE_OPERATION
        || (*Ccb).CompletionMode[NamedPipeEnd as usize] as ULONG == FILE_PIPE_COMPLETE_OPERATION
        || ((*ReadQueue).QueueState != ReadEntries && (*WriteQueue).QueueState != WriteEntries)
    {
        (*Ccb).ReadMode[NamedPipeEnd as usize] = ((*Buffer).ReadMode & 0xFF) as UCHAR;
        (*Ccb).CompletionMode[NamedPipeEnd as usize] = ((*Buffer).CompletionMode & 0xFF) as UCHAR;

        NpCheckForNotify((*Fcb).ParentDcb as PNP_DCB, FALSE, List);
        Status = STATUS_SUCCESS;
    } else {
        Status = STATUS_PIPE_BUSY;
    }

    Status
}

pub unsafe extern "system" fn NpCommonSetInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    let InfoClass: ULONG;
    let Buffer: PVOID;
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    // PAGED_CODE();  (debug-only; no-op under this build)

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == 0 {
        return STATUS_PIPE_DISCONNECTED;
    }
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_INVALID_PARAMETER;
    }

    InfoClass = (*IoStack).Parameters.QueryFile.FileInformationClass as ULONG;
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    if InfoClass == _FILE_INFORMATION_CLASS_FileBasicInformation as ULONG {
        return NpSetBasicInfo(Ccb, Buffer as PFILE_BASIC_INFORMATION);
    }

    if InfoClass != _FILE_INFORMATION_CLASS_FilePipeInformation as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    NpSetPipeInfo(Fcb, Ccb, Buffer as PFILE_PIPE_INFORMATION, NamedPipeEnd, List)
}

pub unsafe extern "system" fn NpFsdSetInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    // PAGED_CODE();  (debug-only; no-op under this build)

    InitializeListHead(addr_of_mut!(DeferredList));

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    Status = NpCommonSetInformation(DeviceObject, Irp, addr_of_mut!(DeferredList));

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

pub unsafe extern "system" fn NpQueryBasicInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_BASIC_INFORMATION = Buffer as PFILE_BASIC_INFORMATION;

    *Length -= core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as _,
    );
    (*InfoBuffer).FileAttributes = FILE_ATTRIBUTE_NORMAL;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryStandardInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
    NamedPipeEnd: ULONG,
) -> NTSTATUS {
    let DataQueue: PNP_DATA_QUEUE;
    let InfoBuffer: PFILE_STANDARD_INFORMATION = Buffer as PFILE_STANDARD_INFORMATION;

    *Length -= core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _,
    );

    if NamedPipeEnd == FILE_PIPE_SERVER_END {
        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    } else {
        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    }

    (*InfoBuffer).AllocationSize.__bindgen_anon_1.LowPart =
        (*Ccb).DataQueue[FILE_PIPE_INBOUND as usize].Quota
            + (*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize].Quota;
    (*InfoBuffer).AllocationSize.__bindgen_anon_1.HighPart = 0;

    if (*DataQueue).QueueState == WriteEntries {
        (*InfoBuffer).EndOfFile.__bindgen_anon_1.HighPart = 0;
        (*InfoBuffer).EndOfFile.__bindgen_anon_1.LowPart =
            (*DataQueue).BytesInQueue - (*DataQueue).ByteOffset;
    }

    (*InfoBuffer).Directory = FALSE;
    (*InfoBuffer).NumberOfLinks = 1;
    (*InfoBuffer).DeletePending = TRUE;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryEaInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_EA_INFORMATION = Buffer as PFILE_EA_INFORMATION;

    *Length -= core::mem::size_of::<FILE_EA_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_EA_INFORMATION>() as _,
    );

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryNameInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_NAME_INFORMATION = Buffer as PFILE_NAME_INFORMATION;
    let mut NameLength: USHORT;
    let Status: NTSTATUS;
    let Name: *mut u16;

    // Charge the header up to FileName, not sizeof(FILE_NAME_INFORMATION): the sizeof
    // counts FileName[1] and padding, which the C (and so ReactOS npfs) charged too. That
    // overstated IoStatus.Information by 4, so the I/O manager copied 4 stale pool bytes
    // past the name, and on overflow it cut the name 2 characters short (NpfsFileInfo.c).
    *Length -= core::mem::offset_of!(FILE_NAME_INFORMATION, FileName) as ULONG;

    if (*Ccb).NodeType == NPFS_NTC_ROOT_DCB_CCB {
        NameLength = (*(*NpVcb).RootDcb).FullName.Length;
        Name = (*(*NpVcb).RootDcb).FullName.Buffer;
    } else {
        NameLength = (*(*Ccb).Fcb).FullName.Length;
        Name = (*(*Ccb).Fcb).FullName.Buffer;
    }

    // FileNameLength is the FULL length even when the copy is cut short, so a caller
    // that gets STATUS_BUFFER_OVERFLOW learns how much to allocate; the C stored the
    // truncated length.
    (*InfoBuffer).FileNameLength = NameLength as ULONG;

    if *Length < NameLength as ULONG {
        Status = STATUS_BUFFER_OVERFLOW;
        NameLength = *Length as USHORT;
    } else {
        Status = STATUS_SUCCESS;
    }

    RtlCopyMemory(
        addr_of_mut!((*InfoBuffer).FileName) as PVOID,
        Name as PCVOID,
        NameLength as _,
    );

    *Length -= NameLength as ULONG;
    Status
}

pub unsafe extern "system" fn NpQueryInternalInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_INTERNAL_INFORMATION = Buffer as PFILE_INTERNAL_INFORMATION;

    *Length -= core::mem::size_of::<FILE_INTERNAL_INFORMATION>() as ULONG;

    // A nonzero id per pipe instance: the C zeroed it, and NpfsFileInfo.c requires a
    // nonzero IndexNumber. The CCB is unique to the instance and lives as long as it.
    (*InfoBuffer).IndexNumber.QuadPart = Ccb as usize as i64;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryPositionInfo(
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
    NamedPipeEnd: ULONG,
) -> NTSTATUS {
    let DataQueue: PNP_DATA_QUEUE;
    let InfoBuffer: PFILE_POSITION_INFORMATION = Buffer as PFILE_POSITION_INFORMATION;

    *Length -= core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_POSITION_INFORMATION>() as _,
    );

    if NamedPipeEnd == FILE_PIPE_SERVER_END {
        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    } else {
        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    }

    if (*DataQueue).QueueState == WriteEntries {
        (*InfoBuffer).CurrentByteOffset.QuadPart =
            ((*DataQueue).BytesInQueue - (*DataQueue).ByteOffset) as LONGLONG;
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryPipeLocalInfo(
    Fcb: PNP_FCB,
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
    NamedPipeEnd: ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_PIPE_LOCAL_INFORMATION = Buffer as PFILE_PIPE_LOCAL_INFORMATION;
    let InQueue: PNP_DATA_QUEUE;
    let OutQueue: PNP_DATA_QUEUE;

    *Length -= core::mem::size_of::<FILE_PIPE_LOCAL_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_PIPE_LOCAL_INFORMATION>() as _,
    );

    InQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    OutQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);

    (*InfoBuffer).NamedPipeType = (*Fcb).NamedPipeType as ULONG;
    (*InfoBuffer).NamedPipeConfiguration = (*Fcb).NamedPipeConfiguration as ULONG;
    (*InfoBuffer).MaximumInstances = (*Fcb).MaximumInstances;
    (*InfoBuffer).CurrentInstances = (*Fcb).CurrentInstances;
    (*InfoBuffer).InboundQuota = (*InQueue).Quota;
    (*InfoBuffer).OutboundQuota = (*OutQueue).Quota;
    (*InfoBuffer).NamedPipeState = (*Ccb).NamedPipeState as ULONG;
    (*InfoBuffer).NamedPipeEnd = NamedPipeEnd;

    if NamedPipeEnd == FILE_PIPE_SERVER_END {
        if (*InQueue).QueueState == WriteEntries {
            (*InfoBuffer).ReadDataAvailable = (*InQueue).BytesInQueue - (*InQueue).ByteOffset;
        }
        (*InfoBuffer).WriteQuotaAvailable = (*OutQueue).Quota - (*OutQueue).QuotaUsed;
    } else {
        if (*OutQueue).QueueState == WriteEntries {
            (*InfoBuffer).ReadDataAvailable = (*OutQueue).BytesInQueue - (*OutQueue).ByteOffset;
        }
        (*InfoBuffer).WriteQuotaAvailable = (*OutQueue).Quota - (*InQueue).QuotaUsed;
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryPipeInfo(
    Fcb: PNP_FCB,
    Ccb: PNP_CCB,
    Buffer: PVOID,
    Length: *mut ULONG,
    NamedPipeEnd: ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_PIPE_INFORMATION = Buffer as PFILE_PIPE_INFORMATION;

    *Length -= core::mem::size_of::<FILE_PIPE_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_PIPE_INFORMATION>() as _,
    );

    (*InfoBuffer).ReadMode = (*Ccb).ReadMode[NamedPipeEnd as usize] as ULONG;
    (*InfoBuffer).CompletionMode = (*Ccb).CompletionMode[NamedPipeEnd as usize] as ULONG;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCommonQueryInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut NamedPipeEnd: ULONG = 0;
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let InfoClass: ULONG;
    let mut Length: ULONG;
    let Buffer: PVOID;
    let AllInfo: PFILE_ALL_INFORMATION;
    let mut Status: NTSTATUS;
    // PAGED_CODE();  (debug-only; no-op under this build)

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == 0 {
        return STATUS_PIPE_DISCONNECTED;
    }

    Buffer = (*Irp).AssociatedIrp.SystemBuffer;
    Length = (*IoStack).Parameters.QueryFile.Length;
    InfoClass = (*IoStack).Parameters.QueryFile.FileInformationClass as ULONG;

    if NodeTypeCode != NPFS_NTC_CCB {
        if NodeTypeCode != NPFS_NTC_ROOT_DCB
            || InfoClass != _FILE_INFORMATION_CLASS_FileNameInformation as ULONG
        {
            return STATUS_INVALID_PARAMETER;
        }
    }

    if InfoClass == _FILE_INFORMATION_CLASS_FileNameInformation as ULONG {
        Status = NpQueryNameInfo(Ccb, Buffer, addr_of_mut!(Length));
    } else if InfoClass == _FILE_INFORMATION_CLASS_FilePositionInformation as ULONG {
        Status = NpQueryPositionInfo(Ccb, Buffer, addr_of_mut!(Length), NamedPipeEnd);
    } else if InfoClass == _FILE_INFORMATION_CLASS_FilePipeInformation as ULONG {
        Status = NpQueryPipeInfo(Fcb, Ccb, Buffer, addr_of_mut!(Length), NamedPipeEnd);
    } else if InfoClass == _FILE_INFORMATION_CLASS_FilePipeLocalInformation as ULONG {
        Status = NpQueryPipeLocalInfo(Fcb, Ccb, Buffer, addr_of_mut!(Length), NamedPipeEnd);
    } else if InfoClass == _FILE_INFORMATION_CLASS_FileBasicInformation as ULONG {
        Status = NpQueryBasicInfo(Ccb, Buffer, addr_of_mut!(Length));
    } else if InfoClass == _FILE_INFORMATION_CLASS_FileStandardInformation as ULONG {
        Status = NpQueryStandardInfo(Ccb, Buffer, addr_of_mut!(Length), NamedPipeEnd);
    } else if InfoClass == _FILE_INFORMATION_CLASS_FileInternalInformation as ULONG {
        Status = NpQueryInternalInfo(Ccb, Buffer, addr_of_mut!(Length));
    } else if InfoClass == _FILE_INFORMATION_CLASS_FileAllInformation as ULONG {
        Length -= (core::mem::size_of::<FILE_ACCESS_INFORMATION>()
            + core::mem::size_of::<FILE_MODE_INFORMATION>()
            + core::mem::size_of::<FILE_ALIGNMENT_INFORMATION>()) as ULONG;
        AllInfo = Buffer as PFILE_ALL_INFORMATION;
        NpQueryBasicInfo(
            Ccb,
            addr_of_mut!((*AllInfo).BasicInformation) as PVOID,
            addr_of_mut!(Length),
        );
        NpQueryStandardInfo(
            Ccb,
            addr_of_mut!((*AllInfo).StandardInformation) as PVOID,
            addr_of_mut!(Length),
            NamedPipeEnd,
        );
        NpQueryInternalInfo(
            Ccb,
            addr_of_mut!((*AllInfo).InternalInformation) as PVOID,
            addr_of_mut!(Length),
        );
        NpQueryEaInfo(
            Ccb,
            addr_of_mut!((*AllInfo).EaInformation) as PVOID,
            addr_of_mut!(Length),
        );
        NpQueryPositionInfo(
            Ccb,
            addr_of_mut!((*AllInfo).PositionInformation) as PVOID,
            addr_of_mut!(Length),
            NamedPipeEnd,
        );
        Status = NpQueryNameInfo(
            Ccb,
            addr_of_mut!((*AllInfo).NameInformation) as PVOID,
            addr_of_mut!(Length),
        );
    } else if InfoClass == _FILE_INFORMATION_CLASS_FileEaInformation as ULONG {
        Status = NpQueryEaInfo(Ccb, Buffer, addr_of_mut!(Length));
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    (*Irp).IoStatus.Information =
        ((*IoStack).Parameters.QueryFile.Length - Length) as _;
    Status
}

pub unsafe extern "system" fn NpFsdQueryInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();  (debug-only; no-op under this build)

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    Status = NpCommonQueryInformation(DeviceObject, Irp);

    NpReleaseVcb();
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

/* EOF */
