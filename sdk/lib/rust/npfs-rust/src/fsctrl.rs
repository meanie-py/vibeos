//! fsctrl.rs — fsctrl.c: NPFS I/O Controls (NpFsdFileSystemControl + NpCommonFileSystemControl
//! dispatch over FSCTL_PIPE_* codes, and the per-FSCTL handlers).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{
    BOOLEAN, LONG, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, UCHAR, ULONG, ULONG_PTR, USHORT,
};
use ntkrnl_sys::base::{
    InitializeListHead, InsertTailList,
};
use ntkrnl_sys::ddk::*;

use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCompletionRoutine, IoGetNextIrpStackLocation, IoSetNextIrpStackLocation};
use ntkrnl_sys::ke::{KeBugCheckEx, KeSetEvent};

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;
use ntkrnl_sys::constants::STATUS_NOT_FOUND;

// cross-module fns (function→module map):
use crate::fileobsup::NpDecodeFileObject;
use crate::secursup::{NpImpersonateClientContext, NpUninitializeSecurity};
use crate::statesup::{NpSetDisconnectedPipeState, NpSetListeningPipeState};
use crate::readsup::NpReadDataQueue;
use crate::writesup::NpWriteDataQueue;
use crate::datasup::NpAddDataQueueEntry;
use crate::prefxsup::NpFindPrefix;
use crate::waitsup::NpAddWaiter;

// File ID number for NPFS bugchecking support (npfs.h: NPFS_BUGCHECK_FSCTRL)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_FSCTRL;

// ---- constants NOT in glue.rs (harvested from SDK headers; flagged in summary) -------------

// CTL_CODE(FILE_DEVICE_NAMED_PIPE=0x11, fn, method, access); values verified against
// sdk/include/xdk/iotypes.h (FSCTL_PIPE_*). CTL_CODE = (t<<16)|(a<<14)|(f<<2)|m,
// METHOD_BUFFERED=0, METHOD_NEITHER=3, FILE_ANY_ACCESS=0, FILE_READ_DATA=1, FILE_WRITE_DATA=2.
// sdk/include/xdk/iotypes.h:7258  FSCTL_PIPE_ASSIGN_EVENT
const FSCTL_PIPE_ASSIGN_EVENT: ULONG = 0x0011_0000;
// sdk/include/xdk/iotypes.h:7259  FSCTL_PIPE_DISCONNECT
const FSCTL_PIPE_DISCONNECT: ULONG = 0x0011_0004;
// sdk/include/xdk/iotypes.h:7260  FSCTL_PIPE_LISTEN
const FSCTL_PIPE_LISTEN: ULONG = 0x0011_0008;
// sdk/include/xdk/iotypes.h:7261  FSCTL_PIPE_PEEK
const FSCTL_PIPE_PEEK: ULONG = 0x0011_400C;
// sdk/include/xdk/iotypes.h:7262  FSCTL_PIPE_QUERY_EVENT
const FSCTL_PIPE_QUERY_EVENT: ULONG = 0x0011_0010;
// sdk/include/xdk/iotypes.h:7263  FSCTL_PIPE_TRANSCEIVE
const FSCTL_PIPE_TRANSCEIVE: ULONG = 0x0011_C017;
// sdk/include/xdk/iotypes.h:7264  FSCTL_PIPE_WAIT
const FSCTL_PIPE_WAIT: ULONG = 0x0011_0018;
// sdk/include/xdk/iotypes.h:7265  FSCTL_PIPE_IMPERSONATE
const FSCTL_PIPE_IMPERSONATE: ULONG = 0x0011_001C;
// sdk/include/xdk/iotypes.h:7266  FSCTL_PIPE_SET_CLIENT_PROCESS
const FSCTL_PIPE_SET_CLIENT_PROCESS: ULONG = 0x0011_0020;
// sdk/include/xdk/iotypes.h:7267  FSCTL_PIPE_QUERY_CLIENT_PROCESS
const FSCTL_PIPE_QUERY_CLIENT_PROCESS: ULONG = 0x0011_0024;
/// CTL_CODE(FILE_DEVICE_NAMED_PIPE, 12, METHOD_BUFFERED, FILE_ANY_ACCESS).
const FSCTL_PIPE_GET_CONNECTION_ATTRIBUTE: ULONG = 0x0011_0030;
// sdk/include/xdk/iotypes.h:7276  FSCTL_PIPE_INTERNAL_READ
const FSCTL_PIPE_INTERNAL_READ: ULONG = 0x0011_5FF4;
// sdk/include/xdk/iotypes.h:7277  FSCTL_PIPE_INTERNAL_WRITE
const FSCTL_PIPE_INTERNAL_WRITE: ULONG = 0x0011_9FF8;
// sdk/include/xdk/iotypes.h:7278  FSCTL_PIPE_INTERNAL_TRANSCEIVE
const FSCTL_PIPE_INTERNAL_TRANSCEIVE: ULONG = 0x0011_DFFF;
// sdk/include/xdk/iotypes.h:7279  FSCTL_PIPE_INTERNAL_READ_OVFLOW
const FSCTL_PIPE_INTERNAL_READ_OVFLOW: ULONG = 0x0011_6000;

// sdk/include/psdk/ntstatus.h:505  STATUS_ILLEGAL_FUNCTION ((NTSTATUS)0xC00000AF)
const STATUS_ILLEGAL_FUNCTION: NTSTATUS = 0xC000_00AFu32 as NTSTATUS;

// sdk/include/xdk/iotypes.h:4411  IRP_MN_KERNEL_CALL 0x04
const IRP_MN_KERNEL_CALL: UCHAR = 0x04;

// sdk/include/xdk/iotypes.h:1861  IRP_BUFFERED_IO 0x00000010
const IRP_BUFFERED_IO: ULONG = 0x0000_0010;
// sdk/include/xdk/iotypes.h:1862  IRP_DEALLOCATE_BUFFER 0x00000020
const IRP_DEALLOCATE_BUFFER: ULONG = 0x0000_0020;
// sdk/include/xdk/iotypes.h:17    POOL_QUOTA_FAIL_INSTEAD_OF_RAISE 0x8
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ULONG = 0x8;
// sdk/include/xdk/iotypes.h:598   IO_NO_INCREMENT 0
const IO_NO_INCREMENT: i8 = 0;
// sdk/include/xdk/obtypes.h:7     OBJ_NAME_PATH_SEPARATOR ((WCHAR)L'\\')
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;
// sdk/include/xdk/ntbasedef.h:619 UNICODE_NULL ((WCHAR)0)
const UNICODE_NULL_SIZE: USHORT = core::mem::size_of::<u16>() as USHORT;

/* GLOBALS ********************************************************************/

// C: `IO_STATUS_BLOCK NpUserIoStatusBlock;`
pub static mut NpUserIoStatusBlock: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

/* FUNCTIONS ******************************************************************/

pub unsafe extern "system" fn NpInternalTransceive(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn NpInternalRead(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Overflow: BOOLEAN,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn NpInternalWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

/// Whether the attribute name in the first `len` bytes of `buf` is exactly `name`.
/// Divergence from the C, whose `strncmp(buf, name, len)` lets any prefix match ("C" names
/// ClientProcessId), while Windows matches the whole name: the caller passes the name with
/// or without its NUL, nothing shorter.
unsafe fn attribute_is(buf: *const u8, len: usize, name: &[u8]) -> bool {
    let got = core::slice::from_raw_parts(buf, len);
    got.len() >= name.len() && &got[..name.len()] == name && got[name.len()..].iter().all(|&b| b == 0)
}

/// FSCTL_PIPE_GET_CONNECTION_ATTRIBUTE (upstream aa9e876b724, CORE-17834; NVDA asks for it).
/// Upstream's partial implementation: "ClientProcessId" and "ClientSessionId" from the CCB's
/// client process, without the extended-attribute store Windows keeps them in.
pub unsafe extern "system" fn NpGetConnectionAttribute(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Ccb: PNP_CCB = null_mut();
    // PAGED_CODE();

    let IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    let NodeTypeCode = NpDecodeFileObject((*IoStackLocation).FileObject, null_mut(), &mut Ccb, null_mut());
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_PIPE_DISCONNECTED;
    }

    let SystemBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut u8;
    let InputBufferLength = (*IoStackLocation).Parameters.FileSystemControl.InputBufferLength;
    let OutputBufferLength = (*IoStackLocation).Parameters.FileSystemControl.OutputBufferLength;
    if InputBufferLength == 0 || OutputBufferLength == 0 || SystemBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

    if (*Ccb).Process.is_null()
        || (*Ccb).NamedPipeState as ULONG == FILE_PIPE_DISCONNECTED_STATE
        || (*Ccb).NamedPipeState as ULONG == FILE_PIPE_LISTENING_STATE
    {
        /* ClientProcessId and ClientSessionId are not defined before a client connects */
        Status = STATUS_NOT_FOUND;
    } else if (OutputBufferLength as usize) < core::mem::size_of::<ULONG>() {
        /* Both attributes are a ULONG */
        Status = STATUS_BUFFER_TOO_SMALL;
    } else if attribute_is(SystemBuffer, InputBufferLength as usize, b"ClientProcessId") {
        *(SystemBuffer as *mut ULONG) = PsGetProcessId((*Ccb).Process) as ULONG_PTR as ULONG;
        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
    } else if attribute_is(SystemBuffer, InputBufferLength as usize, b"ClientSessionId") {
        *(SystemBuffer as *mut ULONG) = PsGetProcessSessionId((*Ccb).Process);
        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
    } else {
        Status = STATUS_NOT_FOUND;
    }

    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    Status
}

pub unsafe extern "system" fn NpQueryClientProcess(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStackLocation: PIO_STACK_LOCATION;
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut Ccb: PNP_CCB = null_mut();
    let ClientSession: PNP_CLIENT_PROCESS;
    let QueryBuffer: PNP_CLIENT_PROCESS;
    let Length: ULONG;
    // PAGED_CODE();

    /* Get the current stack location */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    /* Decode the file object and check the node type */
    NodeTypeCode = NpDecodeFileObject((*IoStackLocation).FileObject, null_mut(), &mut Ccb, null_mut());
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_PIPE_DISCONNECTED;
    }

    /* Get the length of the query buffer */
    Length = (*IoStackLocation).Parameters.QueryFile.Length;
    if Length < 8 {
        return STATUS_INVALID_PARAMETER;
    }

    QueryBuffer = (*Irp).AssociatedIrp.SystemBuffer as PNP_CLIENT_PROCESS;

    /* Lock the Ccb */
    ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

    /* Get the CCBs client session and check if it's set */
    ClientSession = (*Ccb).ClientSession as PNP_CLIENT_PROCESS;
    if ClientSession != null_mut() {
        /* Copy first 2 fields */
        (*QueryBuffer).Unknown = (*ClientSession).Unknown;
        (*QueryBuffer).Process = (*ClientSession).Process;
    } else {
        /* Copy the process from the CCB */
        (*QueryBuffer).Unknown = null_mut();
        (*QueryBuffer).Process = (*Ccb).Process as PVOID;
    }

    /* Does the caller provide a large enough buffer for the full data? */
    if Length as usize >= core::mem::size_of::<NP_CLIENT_PROCESS>() {
        (*Irp).IoStatus.Information = core::mem::size_of::<NP_CLIENT_PROCESS>() as _;

        /* Do we have a ClientSession structure? */
        if ClientSession != null_mut() {
            /* Copy length and the data */
            (*QueryBuffer).DataLength = (*ClientSession).DataLength;
            core::ptr::copy_nonoverlapping(
                (*ClientSession).Buffer.as_ptr() as *const u8,
                (*QueryBuffer).Buffer.as_mut_ptr() as *mut u8,
                (*ClientSession).DataLength as usize,
            );

            /* NULL terminate the buffer */
            // NT_ASSERT(QueryBuffer->DataLength <= 30);
            (*QueryBuffer).Buffer
                [(*QueryBuffer).DataLength as usize / core::mem::size_of::<u16>()] = 0;
        } else {
            /* No data */
            (*QueryBuffer).DataLength = 0;
            (*QueryBuffer).Buffer[0] = 0;
        }
    } else {
        // FIELD_OFFSET(NP_CLIENT_PROCESS, DataLength)
        (*Irp).IoStatus.Information = core::mem::offset_of!(NP_CLIENT_PROCESS, DataLength) as _;
    }

    /* Unlock the Ccb */
    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpSetClientProcess(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStackLocation: PIO_STACK_LOCATION;
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut Ccb: PNP_CCB = null_mut();
    let Length: ULONG;
    let InputBuffer: PNP_CLIENT_PROCESS;
    let ClientSession: PNP_CLIENT_PROCESS;
    let OldClientSession: PNP_CLIENT_PROCESS;
    // PAGED_CODE();

    /* Get the current stack location */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    /* Only kernel calls are allowed! */
    if (*IoStackLocation).MinorFunction != IRP_MN_KERNEL_CALL {
        return STATUS_ACCESS_DENIED;
    }

    /* Decode the file object and check the node type */
    NodeTypeCode = NpDecodeFileObject((*IoStackLocation).FileObject, null_mut(), &mut Ccb, null_mut());
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_PIPE_DISCONNECTED;
    }

    /* Get the length of the query buffer and check if it's valid */
    Length = (*IoStackLocation).Parameters.QueryFile.Length;
    if Length as usize != core::mem::size_of::<NP_CLIENT_PROCESS>() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Get the buffer and check if the data Length is valid */
    InputBuffer = (*Irp).AssociatedIrp.SystemBuffer as PNP_CLIENT_PROCESS;
    if (*InputBuffer).DataLength > 30 {
        return STATUS_INVALID_PARAMETER;
    }

    /* Allocate a new structure */
    ClientSession = ExAllocatePoolWithQuotaTag(
        PagedPool,
        core::mem::size_of::<NP_CLIENT_PROCESS>() as _,
        NPFS_CLIENT_INFO_TAG, // 'iFpN'
    ) as PNP_CLIENT_PROCESS;

    /* Copy the full input buffer */
    core::ptr::copy_nonoverlapping(
        InputBuffer as *const u8,
        ClientSession as *mut u8,
        core::mem::size_of::<NP_CLIENT_PROCESS>(),
    );

    /* Lock the Ccb */
    ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

    /* Get the old ClientSession and set the new */
    OldClientSession = (*Ccb).ClientSession as PNP_CLIENT_PROCESS;
    (*Ccb).ClientSession = ClientSession as PVOID;

    /* Copy the process to the CCB */
    (*Ccb).Process = (*ClientSession).Process as PEPROCESS;

    /* Unlock the Ccb */
    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));

    /* Check if there was already a ClientSession */
    if OldClientSession != null_mut() {
        /* Free it */
        ExFreePoolWithTag(OldClientSession as PVOID, NPFS_CLIENT_INFO_TAG);
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpAssignEvent(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn NpQueryEvent(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn NpImpersonate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let Status: NTSTATUS;
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);
    if NodeTypeCode == NPFS_NTC_CCB && NamedPipeEnd == FILE_PIPE_SERVER_END {
        Status = NpImpersonateClientContext(Ccb);
    } else {
        Status = STATUS_ILLEGAL_FUNCTION;
    }

    Status
}

pub unsafe extern "system" fn NpDisconnect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let Status: NTSTATUS;
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);
    if NodeTypeCode == NPFS_NTC_CCB {
        if NamedPipeEnd == FILE_PIPE_SERVER_END {
            ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

            Status = NpSetDisconnectedPipeState(Ccb, List);

            NpUninitializeSecurity(Ccb);

            ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        } else {
            Status = STATUS_ILLEGAL_FUNCTION;
        }
    } else {
        Status = STATUS_PIPE_DISCONNECTED;
    }

    Status
}

pub unsafe extern "system" fn NpListen(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let Status: NTSTATUS;
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);
    if NodeTypeCode == NPFS_NTC_CCB {
        if NamedPipeEnd == FILE_PIPE_SERVER_END {
            ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

            Status = NpSetListeningPipeState(Ccb, Irp, List);

            NpUninitializeSecurity(Ccb);

            ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        } else {
            Status = STATUS_ILLEGAL_FUNCTION;
        }
    } else {
        Status = STATUS_ILLEGAL_FUNCTION;
    }

    Status
}

pub unsafe extern "system" fn NpPeek(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let Type: NODE_TYPE_CODE;
    let OutputLength: ULONG;
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let PeekBuffer: PFILE_PIPE_PEEK_BUFFER;
    let DataQueue: PNP_DATA_QUEUE;
    let mut BytesPeeked: ULONG_PTR;
    let mut IoStatus: IO_STATUS_BLOCK;
    let mut Status: NTSTATUS;
    let DataEntry: PNP_DATA_QUEUE_ENTRY;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    OutputLength = (*IoStack).Parameters.FileSystemControl.OutputBufferLength;
    Type = NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);

    if Type == 0 {
        return STATUS_PIPE_DISCONNECTED;
    }

    if (Type != NPFS_NTC_CCB)
        || ((OutputLength as usize) < core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data))
    {
        return STATUS_INVALID_PARAMETER;
    }

    PeekBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_PIPE_PEEK_BUFFER;
    if NamedPipeEnd != FILE_PIPE_CLIENT_END {
        if NamedPipeEnd != FILE_PIPE_SERVER_END {
            // NpBugCheck(NamedPipeEnd, 0, 0);
            KeBugCheckEx(
                NPFS_FILE_SYSTEM,
                ((NPFS_BUGCHECK_FILE_ID << 16) | 377) as _,
                NamedPipeEnd as _,
                0,
                0,
            );
        }

        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    } else {
        DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    }

    if (*Ccb).NamedPipeState as ULONG != FILE_PIPE_CONNECTED_STATE {
        if (*Ccb).NamedPipeState as ULONG != FILE_PIPE_CLOSING_STATE {
            return STATUS_INVALID_PIPE_STATE;
        }

        if (*DataQueue).QueueState != WriteEntries {
            return STATUS_PIPE_BROKEN;
        }
    }

    (*PeekBuffer).NamedPipeState = 0;
    (*PeekBuffer).ReadDataAvailable = 0;
    (*PeekBuffer).NumberOfMessages = 0;
    (*PeekBuffer).MessageLength = 0;
    (*PeekBuffer).NamedPipeState = (*Ccb).NamedPipeState as ULONG;
    BytesPeeked = core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data) as ULONG_PTR;

    if (*DataQueue).QueueState == WriteEntries {
        DataEntry = CONTAINING_RECORD!(
            (*DataQueue).Queue.Flink,
            NP_DATA_QUEUE_ENTRY,
            QueueEntry
        );
        // ASSERT((DataEntry->DataEntryType == Buffered) || (DataEntry->DataEntryType == Unbuffered));

        (*PeekBuffer).ReadDataAvailable = (*DataQueue).BytesInQueue - (*DataQueue).ByteOffset;
        if (*(*Ccb).Fcb).NamedPipeType as ULONG == FILE_PIPE_MESSAGE_TYPE {
            (*PeekBuffer).NumberOfMessages = (*DataQueue).EntriesInQueue;
            (*PeekBuffer).MessageLength = (*DataEntry).DataSize - (*DataQueue).ByteOffset;
        }

        if OutputLength as usize == core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data) {
            Status = if (*PeekBuffer).ReadDataAvailable != 0 {
                STATUS_BUFFER_OVERFLOW
            } else {
                STATUS_SUCCESS
            };
        } else {
            IoStatus = NpReadDataQueue(
                DataQueue,
                TRUE,
                FALSE,
                (*PeekBuffer).Data.as_mut_ptr() as PVOID,
                OutputLength - core::mem::offset_of!(FILE_PIPE_PEEK_BUFFER, Data) as ULONG,
                ((*(*Ccb).Fcb).NamedPipeType as ULONG == FILE_PIPE_MESSAGE_TYPE) as ULONG,
                Ccb,
                List,
            );
            Status = IoStatus.__bindgen_anon_1.Status;
            BytesPeeked += IoStatus.Information as usize;
        }
    } else {
        Status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.Information = BytesPeeked as _;
    Status
}

pub unsafe extern "system" fn NpCompleteTransceiveIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    // PAGED_CODE();

    if !(*Irp).AssociatedIrp.SystemBuffer.is_null() {
        ExFreePool((*Irp).AssociatedIrp.SystemBuffer);
    }

    IoFreeIrp(Irp);
    STATUS_MORE_PROCESSING_REQUIRED
}

pub unsafe extern "system" fn NpTransceive(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let InBuffer: PVOID;
    let OutBuffer: PVOID;
    let InLength: ULONG;
    let OutLength: ULONG;
    let mut BytesWritten: ULONG = 0;
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    let NonPagedCcb: PNP_NONPAGED_CCB;
    let ReadQueue: PNP_DATA_QUEUE;
    let WriteQueue: PNP_DATA_QUEUE;
    let EventBuffer: PNP_EVENT_BUFFER;
    let mut Status: NTSTATUS;
    let NewIrp: PIRP;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    InLength = (*IoStack).Parameters.FileSystemControl.InputBufferLength;
    InBuffer = (*IoStack).Parameters.FileSystemControl.Type3InputBuffer;
    OutLength = (*IoStack).Parameters.FileSystemControl.OutputBufferLength;
    OutBuffer = (*Irp).UserBuffer;

    if (*Irp).RequestorMode == UserMode {
        // C: _SEH2_TRY { ProbeForRead/ProbeForWrite } _SEH2_EXCEPT { return code }.
        // No SEH here, so the Result probes return the code instead of raising.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(InBuffer, InLength as usize, 1, 1) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(OutBuffer, OutLength as usize, 1, 1) {
            return status;
        }
    }

    NodeTypeCode = NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_PIPE_DISCONNECTED;
    }

    NonPagedCcb = (*Ccb).NonPagedCcb;
    ExAcquireResourceExclusiveLite(addr_of_mut!((*NonPagedCcb).Lock), TRUE);

    // Quickie cleanup helper (FINALLY-like): the C `goto Quickie` always reaches the
    // ExReleaseResourceLite at the end. We use a labeled block + break to centralize it.
    'quickie: {
        if (*Ccb).NamedPipeState as ULONG != FILE_PIPE_CONNECTED_STATE {
            Status = STATUS_INVALID_PIPE_STATE;
            break 'quickie;
        }

        if NamedPipeEnd != FILE_PIPE_CLIENT_END {
            if NamedPipeEnd != FILE_PIPE_SERVER_END {
                // NpBugCheck(NamedPipeEnd, 0, 0);
                KeBugCheckEx(
                    NPFS_FILE_SYSTEM,
                    ((NPFS_BUGCHECK_FILE_ID << 16) | 523) as _,
                    NamedPipeEnd as _,
                    0,
                    0,
                );
            }
            ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
            WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
        } else {
            ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
            WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
        }

        EventBuffer = (*NonPagedCcb).EventBuffer[NamedPipeEnd as usize];

        if (*(*Ccb).Fcb).NamedPipeConfiguration as ULONG != FILE_PIPE_FULL_DUPLEX
            || (*Ccb).ReadMode[NamedPipeEnd as usize] as ULONG != FILE_PIPE_MESSAGE_MODE
        {
            Status = STATUS_INVALID_PIPE_STATE;
            break 'quickie;
        }

        if (*ReadQueue).QueueState != Empty {
            Status = STATUS_PIPE_BUSY;
            break 'quickie;
        }

        Status = NpWriteDataQueue(
            WriteQueue,
            FILE_PIPE_MESSAGE_MODE,
            InBuffer,
            InLength,
            (*(*Ccb).Fcb).NamedPipeType as ULONG,
            &mut BytesWritten,
            Ccb,
            NamedPipeEnd,
            (*Irp).Tail.Overlay.Thread,
            List,
        );
        if Status == STATUS_MORE_PROCESSING_REQUIRED {
            // ASSERT(WriteQueue->QueueState != ReadEntries);
            NewIrp = IoAllocateIrp((*DeviceObject).StackSize, TRUE);
            if NewIrp.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'quickie;
            }

            type CompletionFn =
                unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
            IoSetCompletionRoutine(
                NewIrp,
                core::mem::transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
                    NpCompleteTransceiveIrp,
                ),
                null_mut(),
                TRUE,
                TRUE,
                TRUE,
            );

            if BytesWritten != 0 {
                (*NewIrp).AssociatedIrp.SystemBuffer = ExAllocatePoolWithQuotaTag(
                    PagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE as POOL_TYPE,
                    BytesWritten as _,
                    NPFS_WRITE_BLOCK_TAG,
                );
                if (*NewIrp).AssociatedIrp.SystemBuffer.is_null() {
                    IoFreeIrp(NewIrp);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'quickie;
                }

                // _SEH2_TRY { RtlCopyMemory } _SEH2_EXCEPT { Status = code; goto Quickie }
                // Under USE_DUMMY_PSEH the EXCEPT handler is dropped; the copy still runs.
                core::ptr::copy_nonoverlapping(
                    ((InBuffer as ULONG_PTR) + InLength as ULONG_PTR
                        - BytesWritten as ULONG_PTR) as *const u8,
                    (*NewIrp).AssociatedIrp.SystemBuffer as *mut u8,
                    BytesWritten as usize,
                );
            } else {
                (*NewIrp).AssociatedIrp.SystemBuffer = null_mut();
            }

            IoStack = IoGetNextIrpStackLocation(NewIrp);
            IoSetNextIrpStackLocation(NewIrp);

            (*NewIrp).Tail.Overlay.Thread = (*Irp).Tail.Overlay.Thread;
            (*NewIrp).IoStatus.Information = BytesWritten as _;

            (*IoStack).Parameters.Write.Length = BytesWritten;
            (*IoStack).MajorFunction = IRP_MJ_WRITE as UCHAR;

            if BytesWritten > 0 {
                (*NewIrp).Flags = IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO;
            }
            (*NewIrp).UserIosb = addr_of_mut!(NpUserIoStatusBlock);

            Status = NpAddDataQueueEntry(
                NamedPipeEnd,
                Ccb,
                WriteQueue,
                WriteEntries,
                Unbuffered,
                BytesWritten,
                NewIrp,
                null_mut(),
                0,
            );
            if Status != STATUS_PENDING {
                (*NewIrp).IoStatus.__bindgen_anon_1.Status = Status;
                InsertTailList(List, irp_list_entry(NewIrp));
            }
        }

        if !NT_SUCCESS(Status) {
            break 'quickie;
        }

        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID as PVOID, IO_NO_INCREMENT as LONG, FALSE);
        }
        // ASSERT(ReadQueue->QueueState == Empty);
        Status = NpAddDataQueueEntry(
            NamedPipeEnd,
            Ccb,
            ReadQueue,
            ReadEntries,
            Buffered,
            OutLength,
            Irp,
            null_mut(),
            0,
        );
        if NT_SUCCESS(Status) {
            if !EventBuffer.is_null() {
                KeSetEvent((*EventBuffer).Event as PVOID as PVOID, IO_NO_INCREMENT as LONG, FALSE);
            }
        }
    } // 'quickie

    // Quickie:
    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    Status
}

pub unsafe extern "system" fn NpWaitForNamedPipe(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let InLength: ULONG;
    let NameLength: ULONG;
    let mut SourceString: UNICODE_STRING = core::mem::zeroed();
    let mut Prefix: UNICODE_STRING = core::mem::zeroed();
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let Buffer: PFILE_PIPE_WAIT_FOR_BUFFER;
    let mut Status: NTSTATUS;
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut NextEntry: *mut LIST_ENTRY;
    let mut Fcb: PNP_FCB;
    let OriginalBuffer: *mut u16;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    InLength = (*IoStack).Parameters.FileSystemControl.InputBufferLength;

    SourceString.Buffer = null_mut();

    if NpDecodeFileObject((*IoStack).FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd)
        != NPFS_NTC_ROOT_DCB
    {
        Status = STATUS_ILLEGAL_FUNCTION;
        // goto Quickie;
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    Buffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_PIPE_WAIT_FOR_BUFFER;
    if (InLength as usize) < core::mem::size_of::<FILE_PIPE_WAIT_FOR_BUFFER>() {
        Status = STATUS_INVALID_PARAMETER;
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    NameLength = (*Buffer).NameLength;
    if (NameLength > (0xFFFF - UNICODE_NULL_SIZE as ULONG))
        || ((NameLength + core::mem::offset_of!(FILE_PIPE_WAIT_FOR_BUFFER, Name) as ULONG)
            > InLength)
    {
        Status = STATUS_INVALID_PARAMETER;
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    SourceString.Length =
        (NameLength as USHORT).wrapping_add(core::mem::size_of::<u16>() as USHORT);
    SourceString.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        SourceString.Length as _,
        NPFS_WRITE_BLOCK_TAG,
    ) as *mut u16;
    if SourceString.Buffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    *SourceString.Buffer.offset(0) = OBJ_NAME_PATH_SEPARATOR;
    core::ptr::copy_nonoverlapping(
        (*Buffer).Name.as_ptr() as *const u8,
        SourceString.Buffer.offset(1) as *mut u8,
        (*Buffer).NameLength as usize,
    );

    Status = STATUS_SUCCESS;
    OriginalBuffer = SourceString.Buffer;
    //Status = NpTranslateAlias(&SourceString);
    if !NT_SUCCESS(Status) {
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    Fcb = NpFindPrefix(addr_of_mut!(SourceString), 1, addr_of_mut!(Prefix));
    Fcb = ((Fcb as ULONG_PTR) & !1usize as ULONG_PTR) as PNP_FCB;

    NodeTypeCode = if !Fcb.is_null() { (*Fcb).NodeType } else { 0 };
    if NodeTypeCode != NPFS_NTC_FCB {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
        if !SourceString.Buffer.is_null() {
            ExFreePool(SourceString.Buffer as PVOID);
        }
        return Status;
    }

    NextEntry = (*Fcb).CcbList.Flink;
    while NextEntry != addr_of_mut!((*Fcb).CcbList) {
        Ccb = CONTAINING_RECORD!(NextEntry, NP_CCB, CcbEntry);
        if (*Ccb).NamedPipeState as ULONG == FILE_PIPE_LISTENING_STATE {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry != addr_of_mut!((*Fcb).CcbList) {
        Status = STATUS_SUCCESS;
    } else {
        Status = NpAddWaiter(
            addr_of_mut!((*NpVcb).WaitQueue),
            (*Fcb).Timeout,
            Irp,
            if OriginalBuffer == SourceString.Buffer {
                null_mut()
            } else {
                addr_of_mut!(SourceString)
            },
        );
    }

    // Quickie:
    if !SourceString.Buffer.is_null() {
        ExFreePool(SourceString.Buffer as PVOID);
    }
    Status
}

pub unsafe extern "system" fn NpCommonFileSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Fsctl: ULONG;
    let mut Overflow: BOOLEAN = FALSE;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let Status: NTSTATUS;
    // PAGED_CODE();

    InitializeListHead(addr_of_mut!(DeferredList));
    Fsctl = (*IoGetCurrentIrpStackLocation(Irp))
        .Parameters
        .FileSystemControl
        .FsControlCode;

    // The C switch falls through from INTERNAL_READ_OVFLOW into INTERNAL_READ. We model that
    // by matching both codes in one arm and setting Overflow only for the OVFLOW code.
    match Fsctl {
        FSCTL_PIPE_PEEK => {
            NpAcquireExclusiveVcb();
            Status = NpPeek(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_INTERNAL_WRITE => {
            NpAcquireSharedVcb();
            Status = NpInternalWrite(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_TRANSCEIVE => {
            NpAcquireSharedVcb();
            Status = NpTransceive(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_INTERNAL_TRANSCEIVE => {
            NpAcquireSharedVcb();
            Status = NpInternalTransceive(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_INTERNAL_READ_OVFLOW | FSCTL_PIPE_INTERNAL_READ => {
            if Fsctl == FSCTL_PIPE_INTERNAL_READ_OVFLOW {
                Overflow = TRUE;
                // on purpose — fall through to INTERNAL_READ
            }
            NpAcquireSharedVcb();
            Status = NpInternalRead(DeviceObject, Irp, Overflow, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_QUERY_CLIENT_PROCESS => {
            NpAcquireSharedVcb();
            Status = NpQueryClientProcess(DeviceObject, Irp);
        }
        FSCTL_PIPE_ASSIGN_EVENT => {
            NpAcquireExclusiveVcb();
            Status = NpAssignEvent(DeviceObject, Irp);
        }
        FSCTL_PIPE_DISCONNECT => {
            NpAcquireExclusiveVcb();
            Status = NpDisconnect(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_LISTEN => {
            NpAcquireSharedVcb();
            Status = NpListen(DeviceObject, Irp, addr_of_mut!(DeferredList));
        }
        FSCTL_PIPE_QUERY_EVENT => {
            NpAcquireExclusiveVcb();
            Status = NpQueryEvent(DeviceObject, Irp);
        }
        FSCTL_PIPE_WAIT => {
            NpAcquireExclusiveVcb();
            Status = NpWaitForNamedPipe(DeviceObject, Irp);
        }
        FSCTL_PIPE_IMPERSONATE => {
            NpAcquireExclusiveVcb();
            Status = NpImpersonate(DeviceObject, Irp);
        }
        FSCTL_PIPE_SET_CLIENT_PROCESS => {
            NpAcquireExclusiveVcb();
            Status = NpSetClientProcess(DeviceObject, Irp);
        }
        FSCTL_PIPE_GET_CONNECTION_ATTRIBUTE => {
            NpAcquireSharedVcb();
            Status = NpGetConnectionAttribute(DeviceObject, Irp);
        }
        _ => {
            return STATUS_NOT_SUPPORTED;
        }
    }

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));

    Status
}

pub unsafe extern "system" fn NpFsdFileSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();

    FsRtlEnterFileSystem();

    Status = NpCommonFileSystemControl(DeviceObject, Irp);

    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

/* EOF */

// ProbeForRead / ProbeForWrite are real ntoskrnl exports (private externs in
// ntkrnl-sys::inline::common); redeclare them here as mup-rust does. ULONG_PTR == usize,
// ULONG == u32 on all three targets.
extern "system" {
}
