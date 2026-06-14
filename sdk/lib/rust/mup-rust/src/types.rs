//! types.rs — the MUP driver-private node structs (mup.h) + dfs.h DFS_NAME_CONTEXT +
//! the NDK MUP_PROVIDER_REGISTRATION_INFO (muptypes.h). All `#[repr(C)]` with the exact
//! C field order. These are driver-private: the kernel only ever sees them through opaque
//! pointers (DeviceExtension, FsContext/FsContext2) — EXCEPT the embedded
//! UNICODE_PREFIX_TABLE_ENTRY (Rtl prefix routines), ERESOURCE (Ex resource routines) and
//! SHARE_ACCESS (Io share-access routines), which use the real ntkrnl-sys ddk layouts so
//! the kernel reads/writes them correctly.
#![allow(non_snake_case, non_camel_case_types, dead_code)]

use ntkrnl_sys::base::{BOOLEAN, LONG, NTSTATUS, ULONG};
use ntkrnl_sys::ddk::{
    ERESOURCE, HANDLE, LARGE_INTEGER, LIST_ENTRY, PDEVICE_OBJECT, PFILE_OBJECT, PIRP,
    PQUERY_PATH_REQUEST, SHARE_ACCESS, UNICODE_PREFIX_TABLE_ENTRY, UNICODE_STRING,
    WORK_QUEUE_ITEM,
};

pub type PMUP_VCB = *mut MUP_VCB;
pub type PMUP_FCB = *mut MUP_FCB;
pub type PMUP_CCB = *mut MUP_CCB;
pub type PMUP_MIC = *mut MUP_MIC;
pub type PMUP_UNC = *mut MUP_UNC;
pub type PMUP_PFX = *mut MUP_PFX;
pub type PMUP_MQC = *mut MUP_MQC;
pub type PFORWARDED_IO_CONTEXT = *mut FORWARDED_IO_CONTEXT;
pub type PQUERY_PATH_CONTEXT = *mut QUERY_PATH_CONTEXT;
pub type PDFS_NAME_CONTEXT = *mut DFS_NAME_CONTEXT;
pub type PMUP_PROVIDER_REGISTRATION_INFO = *mut MUP_PROVIDER_REGISTRATION_INFO;

#[repr(C)]
pub struct MUP_VCB {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub ShareAccess: SHARE_ACCESS,
}

#[repr(C)]
pub struct MUP_FCB {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub FileObject: PFILE_OBJECT,
    pub CcbList: LIST_ENTRY,
}

#[repr(C)]
pub struct MUP_CCB {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub Fcb: PMUP_FCB,
    pub CcbListEntry: LIST_ENTRY,
    pub DeviceObject: PDEVICE_OBJECT,
    pub FileObject: PFILE_OBJECT,
}

#[repr(C)]
pub struct MUP_MIC {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub Irp: PIRP,
    pub LastSuccess: NTSTATUS,
    pub LastFailed: NTSTATUS,
    pub Fcb: PMUP_FCB,
}

#[repr(C)]
pub struct MUP_UNC {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub ProviderListEntry: LIST_ENTRY,
    pub DeviceName: UNICODE_STRING,
    pub DeviceHandle: HANDLE,
    pub DeviceObject: PDEVICE_OBJECT,
    pub FileObject: PFILE_OBJECT,
    pub ProviderOrder: ULONG,
    pub MailslotsSupported: BOOLEAN,
    pub Registered: BOOLEAN,
}

#[repr(C)]
pub struct MUP_PFX {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub PrefixTableEntry: UNICODE_PREFIX_TABLE_ENTRY,
    pub AcceptedPrefix: UNICODE_STRING,
    pub Reserved: ULONG,
    pub ValidityTimeout: LARGE_INTEGER,
    pub UncProvider: PMUP_UNC,
    pub ExternalAlloc: BOOLEAN,
    pub InTable: BOOLEAN,
    pub KeepExtraRef: BOOLEAN,
    pub Padding: BOOLEAN,
    pub PrefixListEntry: LIST_ENTRY,
}

#[repr(C)]
pub struct MUP_MQC {
    pub NodeType: ULONG,
    pub NodeStatus: ULONG,
    pub NodeReferences: LONG,
    pub NodeSize: ULONG,
    pub Irp: PIRP,
    pub FileObject: PFILE_OBJECT,
    pub LatestProvider: PMUP_UNC,
    pub QueryPathListLock: ERESOURCE,
    pub Prefix: PMUP_PFX,
    pub LatestStatus: NTSTATUS,
    pub QueryPathList: LIST_ENTRY,
    pub MQCListEntry: LIST_ENTRY,
}

#[repr(C)]
pub struct FORWARDED_IO_CONTEXT {
    pub Ccb: PMUP_CCB,
    pub MasterIoContext: PMUP_MIC,
    pub WorkQueueItem: WORK_QUEUE_ITEM,
    pub DeviceObject: PDEVICE_OBJECT,
    pub Irp: PIRP,
}

#[repr(C)]
pub struct QUERY_PATH_CONTEXT {
    pub MasterQueryContext: PMUP_MQC,
    pub UncProvider: PMUP_UNC,
    pub QueryPathRequest: PQUERY_PATH_REQUEST,
    pub QueryPathListEntry: LIST_ENTRY,
    pub Irp: PIRP,
}

#[repr(C)]
pub struct DFS_NAME_CONTEXT {
    pub UNCFileName: UNICODE_STRING,
    pub NameContextType: LONG,
    pub Flags: ULONG,
}

#[repr(C)]
pub struct MUP_PROVIDER_REGISTRATION_INFO {
    pub RedirectorDeviceNameOffset: ULONG,
    pub RedirectorDeviceNameLength: ULONG,
    pub Reserved: [ULONG; 2],
    pub MailslotsSupported: BOOLEAN,
}
